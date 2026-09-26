use base64::prelude::*;
use json_patch::{diff, Patch};
use serde_json::{json, Value};
use tracing::{debug, info};

use crate::crd::{EnvVarSpec, MutationActions, ResourceSelector};
use crate::error::Result;
use crate::state::CompiledRule;

pub struct MutationEngine;

impl MutationEngine {
    /// Determines if a given rule matches the target resource and context
    pub fn matches(
        selector: &ResourceSelector,
        kind: &str,
        namespace: Option<&str>,
        resource: &Value,
    ) -> bool {
        // 1. Kind check (defaults to Pod if not specified)
        if !selector.kinds.is_empty() {
            if !selector.kinds.iter().any(|k| k.eq_ignore_ascii_case(kind)) {
                return false;
            }
        } else if !kind.eq_ignore_ascii_case("Pod") {
            return false;
        }

        // 2. Namespace check
        if let Some(ns) = namespace {
            // Exclude namespaces check
            if selector.exclude_namespaces.iter().any(|ex| ex == ns) {
                return false;
            }
            // Include namespaces check
            if !selector.namespaces.is_empty() && !selector.namespaces.iter().any(|inc| inc == ns) {
                return false;
            }
        }

        // 3. Match labels check
        if !selector.match_labels.is_empty() {
            let labels = resource
                .pointer("/metadata/labels")
                .and_then(|v| v.as_object());
            match labels {
                Some(labels_obj) => {
                    for (k, v) in &selector.match_labels {
                        match labels_obj.get(k).and_then(|val| val.as_str()) {
                            Some(target_val) if target_val == v => continue,
                            _ => return false,
                        }
                    }
                }
                None => return false,
            }
        }

        // 4. Match annotations check
        if !selector.match_annotations.is_empty() {
            let annotations = resource
                .pointer("/metadata/annotations")
                .and_then(|v| v.as_object());
            match annotations {
                Some(ann_obj) => {
                    for (k, v) in &selector.match_annotations {
                        match ann_obj.get(k).and_then(|val| val.as_str()) {
                            Some(target_val) if target_val == v => continue,
                            _ => return false,
                        }
                    }
                }
                None => return false,
            }
        }

        true
    }

    /// Mutates the target JSON value in-place according to the action
    pub fn apply_mutations(resource: &mut Value, actions: &MutationActions) -> Result<()> {
        // 1. Add Labels
        if !actions.add_labels.is_empty() {
            if resource.get("metadata").is_none() {
                resource["metadata"] = json!({});
            }
            if resource["metadata"].get("labels").is_none() {
                resource["metadata"]["labels"] = json!({});
            }
            if let Some(labels) = resource["metadata"]["labels"].as_object_mut() {
                for (k, v) in &actions.add_labels {
                    labels.insert(k.clone(), Value::String(v.clone()));
                }
            }
        }

        // 2. Add Annotations
        if !actions.add_annotations.is_empty() {
            if resource.get("metadata").is_none() {
                resource["metadata"] = json!({});
            }
            if resource["metadata"].get("annotations").is_none() {
                resource["metadata"]["annotations"] = json!({});
            }
            if let Some(annotations) = resource["metadata"]["annotations"].as_object_mut() {
                for (k, v) in &actions.add_annotations {
                    annotations.insert(k.clone(), Value::String(v.clone()));
                }
            }
        }

        // Helper to locate container arrays (supports Pod and Workload pod templates)
        let mut container_paths = Vec::new();
        if resource.pointer("/spec/containers").is_some() {
            container_paths.push("/spec/containers");
        } else if resource.pointer("/spec/template/spec/containers").is_some() {
            container_paths.push("/spec/template/spec/containers");
        }

        // 3. Inject Environment Variables into containers
        if !actions.add_env.is_empty() {
            for c_path in &container_paths {
                if let Some(containers) = resource.pointer_mut(c_path).and_then(|v| v.as_array_mut()) {
                    for container in containers {
                        if container.get("env").is_none() {
                            container["env"] = json!([]);
                        }
                        if let Some(env_arr) = container["env"].as_array_mut() {
                            for env_spec in &actions.add_env {
                                inject_env_var(env_arr, env_spec);
                            }
                        }
                    }
                }
            }
        }

        // 4. Inject Volume Mounts into containers
        if !actions.add_volume_mounts.is_empty() {
            for c_path in &container_paths {
                if let Some(containers) = resource.pointer_mut(c_path).and_then(|v| v.as_array_mut()) {
                    for container in containers {
                        if container.get("volumeMounts").is_none() {
                            container["volumeMounts"] = json!([]);
                        }
                        if let Some(vm_arr) = container["volumeMounts"].as_array_mut() {
                            for vm in &actions.add_volume_mounts {
                                vm_arr.push(vm.clone());
                            }
                        }
                    }
                }
            }
        }

        // 5. Inject Sidecar Containers
        if !actions.add_containers.is_empty() {
            for c_path in &container_paths {
                if let Some(containers) = resource.pointer_mut(c_path).and_then(|v| v.as_array_mut()) {
                    for sidecar in &actions.add_containers {
                        containers.push(sidecar.clone());
                    }
                }
            }
        }

        // 6. Inject Init Containers
        if !actions.add_init_containers.is_empty() {
            let init_path = if resource.pointer("/spec").is_some() {
                "/spec/initContainers"
            } else if resource.pointer("/spec/template/spec").is_some() {
                "/spec/template/spec/initContainers"
            } else {
                "/spec/initContainers"
            };

            let parent_path = if init_path.starts_with("/spec/template/spec") {
                "/spec/template/spec"
            } else {
                "/spec"
            };

            if resource.pointer(parent_path).is_some() {
                if resource.pointer(init_path).is_none() {
                    if let Some(spec) = resource.pointer_mut(parent_path) {
                        spec["initContainers"] = json!([]);
                    }
                }
                if let Some(init_arr) = resource.pointer_mut(init_path).and_then(|v| v.as_array_mut()) {
                    for init_c in &actions.add_init_containers {
                        init_arr.push(init_c.clone());
                    }
                }
            }
        }

        // 7. Inject Volumes
        if !actions.add_volumes.is_empty() {
            let vol_path = if resource.pointer("/spec").is_some() {
                "/spec/volumes"
            } else if resource.pointer("/spec/template/spec").is_some() {
                "/spec/template/spec/volumes"
            } else {
                "/spec/volumes"
            };

            let parent_path = if vol_path.starts_with("/spec/template/spec") {
                "/spec/template/spec"
            } else {
                "/spec"
            };

            if resource.pointer(parent_path).is_some() {
                if resource.pointer(vol_path).is_none() {
                    if let Some(spec) = resource.pointer_mut(parent_path) {
                        spec["volumes"] = json!([]);
                    }
                }
                if let Some(vol_arr) = resource.pointer_mut(vol_path).and_then(|v| v.as_array_mut()) {
                    for vol in &actions.add_volumes {
                        vol_arr.push(vol.clone());
                    }
                }
            }
        }

        // 8. Raw RFC 6902 patches
        if !actions.raw_patches.is_empty() {
            let patch_val = Value::Array(actions.raw_patches.clone());
            let patch: Patch = serde_json::from_value(patch_val)?;
            json_patch::patch(resource, &patch)?;
        }

        Ok(())
    }
}

fn inject_env_var(env_arr: &mut Vec<Value>, env_spec: &EnvVarSpec) {
    // If env var already exists, overwrite it, otherwise append
    let mut found = false;
    for existing in env_arr.iter_mut() {
        if let Some(name) = existing.get("name").and_then(|n| n.as_str()) {
            if name == env_spec.name {
                if let Some(val) = &env_spec.value {
                    existing["value"] = Value::String(val.clone());
                    existing.as_object_mut().unwrap().remove("valueFrom");
                } else if let Some(vf) = &env_spec.value_from {
                    existing["valueFrom"] = vf.clone();
                    existing.as_object_mut().unwrap().remove("value");
                }
                found = true;
                break;
            }
        }
    }

    if !found {
        let mut new_env = json!({ "name": env_spec.name });
        if let Some(val) = &env_spec.value {
            new_env["value"] = Value::String(val.clone());
        }
        if let Some(vf) = &env_spec.value_from {
            new_env["valueFrom"] = vf.clone();
        }
        env_arr.push(new_env);
    }
}

/// Evaluates rules against the target resource and generates a base64-encoded RFC 6902 JSON patch
pub fn evaluate_and_patch(
    rules: &[CompiledRule],
    kind: &str,
    namespace: Option<&str>,
    original: &Value,
) -> Result<Option<String>> {
    let mut mutated = original.clone();
    let mut matched_any = false;

    for rule in rules {
        if MutationEngine::matches(&rule.spec.selector, kind, namespace, original) {
            info!(rule = %rule.name, kind = %kind, namespace = ?namespace, "Applying mutation rule");
            MutationEngine::apply_mutations(&mut mutated, &rule.spec.mutations)?;
            matched_any = true;
        }
    }

    if !matched_any {
        debug!("No mutation rules matched resource");
        return Ok(None);
    }

    // Compute RFC 6902 diff
    let patch = diff(original, &mutated);
    if patch.0.is_empty() {
        debug!("Mutations produced an identical document, no patch needed");
        return Ok(None);
    }

    let patch_json = serde_json::to_string(&patch)?;
    debug!(%patch_json, "Generated JSON patch");
    let base64_patch = BASE64_STANDARD.encode(patch_json);
    Ok(Some(base64_patch))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn test_label_and_annotation_mutation() {
        let original = json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": {
                "name": "test-pod",
                "namespace": "default"
            },
            "spec": {
                "containers": [
                    { "name": "app", "image": "nginx" }
                ]
            }
        });

        let mut actions = MutationActions::default();
        actions.add_labels.insert("injected-by".into(), "rust-webhook".into());
        actions.add_annotations.insert("sidecar.istio.io/inject".into(), "false".into());

        let mut mutated = original.clone();
        MutationEngine::apply_mutations(&mut mutated, &actions).unwrap();

        assert_eq!(
            mutated.pointer("/metadata/labels/injected-by").unwrap().as_str().unwrap(),
            "rust-webhook"
        );
        assert_eq!(
            mutated.pointer("/metadata/annotations/sidecar.istio.io~1inject").unwrap().as_str().unwrap(),
            "false"
        );

        let patch = diff(&original, &mutated);
        assert!(!patch.0.is_empty());
    }

    #[test]
    fn test_sidecar_and_env_injection() {
        let original = json!({
            "apiVersion": "v1",
            "kind": "Pod",
            "metadata": { "name": "web" },
            "spec": {
                "containers": [
                    { "name": "main", "image": "my-app:1.0" }
                ]
            }
        });

        let mut actions = MutationActions::default();
        actions.add_env.push(EnvVarSpec {
            name: "ENV".into(),
            value: Some("production".into()),
            value_from: None,
        });
        actions.add_containers.push(json!({
            "name": "log-collector",
            "image": "fluentbit:latest"
        }));

        let mut mutated = original.clone();
        MutationEngine::apply_mutations(&mut mutated, &actions).unwrap();

        let containers = mutated.pointer("/spec/containers").unwrap().as_array().unwrap();
        assert_eq!(containers.len(), 2);
        assert_eq!(containers[0]["env"][0]["name"], "ENV");
        assert_eq!(containers[0]["env"][0]["value"], "production");
        assert_eq!(containers[1]["name"], "log-collector");
    }

    #[test]
    fn test_namespace_exclude_filter() {
        let selector = ResourceSelector {
            kinds: vec!["Pod".into()],
            namespaces: vec![],
            exclude_namespaces: vec!["kube-system".into()],
            match_labels: BTreeMap::new(),
            match_annotations: BTreeMap::new(),
        };

        let pod = json!({ "metadata": { "name": "kube-dns" } });
        assert!(!MutationEngine::matches(&selector, "Pod", Some("kube-system"), &pod));
        assert!(MutationEngine::matches(&selector, "Pod", Some("default"), &pod));
    }
}
