# MutationRule Custom Resource Definition (CRD) Reference

The `MutationRule` resource is namespaced and defines matching criteria and transformations for Kubernetes objects.

- **Group**: `mutation.webhook.io`
- **Version**: `v1alpha1`
- **Kind**: `MutationRule`
- **Plural**: `mutationrules`
- **Scope**: `Namespaced`

---

## Specification (`spec`)

| Field | Type | Default | Description |
|---|---|---|---|
| `priority` | `integer (int32)` | `100` | Execution priority. Rules with higher numbers execute first. |
| `selector` | `object` | `{}` | Criteria for selecting target resources. |
| `mutations` | `object` | Required | Set of mutations to apply to matching resources. |

---

## 1. Selector (`spec.selector`)

Controls which resources will be intercepted and mutated.

| Field | Type | Default | Description |
|---|---|---|---|
| `kinds` | `array of string` | `[]` | Resource kinds to match (e.g. `["Pod", "Deployment"]`). If empty, defaults to matching `Pod`. |
| `namespaces` | `array of string` | `[]` | Whitelist of namespaces to match. If empty, matches all namespaces (except excluded). |
| `excludeNamespaces`| `array of string` | `[]` | Namespaces to explicitly ignore (e.g. `["kube-system"]`). |
| `matchLabels` | `map[string]string`| `{}` | Map of labels that must all match on the resource `metadata.labels`. |
| `matchAnnotations`| `map[string]string`| `{}` | Map of annotations that must all match on `metadata.annotations`. |

---

## 2. Mutations (`spec.mutations`)

Specifies the mutations to apply. At least one mutation field must be non-empty for the rule to be marked valid.

### 2.1 Metadata Mutations

#### `addLabels` (`map[string]string`)
Key-value pairs to add to `metadata.labels`. Existing labels with the same key will be overwritten with the specified value.

#### `addAnnotations` (`map[string]string`)
Key-value pairs to add to `metadata.annotations`. Existing annotations with the same key will be overwritten.

---

### 2.2 Container Mutations

#### `addEnv` (`array of EnvVarSpec`)
Environment variables to inject into all containers in the Pod spec:
```yaml
addEnv:
  - name: "LOG_LEVEL"
    value: "info"
  - name: "POD_IP"
    valueFrom:
      fieldRef:
        fieldPath: status.podIP
```
*Note: If an environment variable with the same `name` already exists in a container, its value is replaced.*

#### `addContainers` (`array of object`)
Full Kubernetes container specifications appended to `spec.containers`. Useful for injecting sidecar proxies, logging daemons, or security agents.

#### `addInitContainers` (`array of object`)
Full Kubernetes container specifications appended to `spec.initContainers`. If `initContainers` does not exist on the target pod, it is automatically created.

---

### 2.3 Storage Mutations

#### `addVolumes` (`array of object`)
Kubernetes volume definitions (e.g. `emptyDir`, `configMap`, `secret`) appended to `spec.volumes`.

#### `addVolumeMounts` (`array of object`)
Volume mounts injected into **all** containers in the Pod:
```yaml
addVolumeMounts:
  - name: "shared-logs"
    mountPath: "/var/log/app"
```

---

### 2.4 Advanced Raw RFC 6902 Patches

#### `rawPatches` (`array of object`)
Direct RFC 6902 JSON patch operations applied sequentially after high-level mutations:
```yaml
rawPatches:
  - op: "add"
    path: "/spec/securityContext/runAsNonRoot"
    value: true
  - op: "replace"
    path: "/spec/dnsPolicy"
    value: "ClusterFirstWithHostNet"
```

---

## 3. Status Subresource (`status`)

| Field | Type | Description |
|---|---|---|
| `valid` | `boolean` | `true` if the rule passed syntax and validation checks; `false` otherwise. |
| `observedGeneration` | `integer` | The generation of the `MutationRule` that was reconciled. |
| `errorMessage` | `string` | Human-readable explanation if `valid` is `false`. |

### Kubernetes CLI Output (`kubectl get mutationrules`)
The CRD includes `additionalPrinterColumns` to surface rule health immediately:
```bash
$ kubectl get mutationrules
NAME                           VALID   AGE
inject-observability-sidecar   true    5m21s
invalid-empty-rule             false   1m10s
```
