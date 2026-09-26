use std::collections::BTreeMap;
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Custom Resource Definition for generic Kubernetes mutation rules.
#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[kube(
    group = "mutation.webhook.io",
    version = "v1alpha1",
    kind = "MutationRule",
    namespaced,
    status = "MutationRuleStatus",
    printcolumn = r#"{"name":"Valid", "type":"boolean", "jsonPath":".status.valid", "description":"Whether the rule is valid"}"#,
    printcolumn = r#"{"name":"Age", "type":"date", "jsonPath":".metadata.creationTimestamp"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct MutationRuleSpec {
    /// Target selector rules
    #[serde(default)]
    pub selector: ResourceSelector,

    /// Set of mutations to apply when the target matches
    pub mutations: MutationActions,

    /// Execution priority (higher priority rules run first)
    #[serde(default = "default_priority")]
    pub priority: i32,
}

fn default_priority() -> i32 {
    100
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSelector {
    /// Kinds to target (e.g. ["Pod", "Deployment"]). Empty means ["Pod"].
    #[serde(default)]
    pub kinds: Vec<String>,

    /// Specific namespaces to target. If empty, matches all namespaces (unless excluded).
    #[serde(default)]
    pub namespaces: Vec<String>,

    /// Namespaces to explicitly exclude (e.g. ["kube-system", "kube-public"]).
    #[serde(default)]
    pub exclude_namespaces: Vec<String>,

    /// Match labels on the resource metadata
    #[serde(default)]
    pub match_labels: BTreeMap<String, String>,

    /// Match annotations on the resource metadata
    #[serde(default)]
    pub match_annotations: BTreeMap<String, String>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MutationActions {
    /// Labels to merge into the resource metadata
    #[serde(default)]
    pub add_labels: BTreeMap<String, String>,

    /// Annotations to merge into the resource metadata
    #[serde(default)]
    pub add_annotations: BTreeMap<String, String>,

    /// Environment variables to inject into all containers
    #[serde(default)]
    pub add_env: Vec<EnvVarSpec>,

    /// Containers (e.g. sidecars) to append to the Pod spec containers
    #[serde(default)]
    pub add_containers: Vec<serde_json::Value>,

    /// Init containers to append to the Pod spec initContainers
    #[serde(default)]
    pub add_init_containers: Vec<serde_json::Value>,

    /// Volumes to append to the Pod spec volumes
    #[serde(default)]
    pub add_volumes: Vec<serde_json::Value>,

    /// Volume mounts to inject into all containers
    #[serde(default)]
    pub add_volume_mounts: Vec<serde_json::Value>,

    /// Direct RFC 6902 JSON patch operations (advanced flexibility)
    #[serde(default)]
    pub raw_patches: Vec<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EnvVarSpec {
    pub name: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub value_from: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Clone, Debug, JsonSchema, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct MutationRuleStatus {
    pub valid: bool,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub observed_generation: Option<i64>,
}
