# General Kubernetes Webhook Mutation Controller

A high-performance, generic Kubernetes Mutating Admission Webhook server and Controller written in Rust using `kube-rs` and `Axum`.

---

## Comprehensive Documentation

Full documentation is available in the [`docs/`](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/docs/README.md) directory:
- [**Documentation Index**](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/docs/README.md)
- [**Architecture & Design**](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/docs/architecture.md)
- [**CRD Specification Reference**](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/docs/crd-reference.md)
- [**TLS & Security Architecture**](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/docs/tls-and-security.md)
- [**Operator & Deployment Guide**](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/docs/operator-guide.md)
- [**Mutation Cookbook & Practical Recipes**](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/docs/examples.md)

---

## Features

- **Controller & Webhook in One**: Runs both an Axum HTTPS admission webhook and a `kube::runtime::Controller` concurrently in a single async binary.
- **Generic Mutation Rules via CRD**: Declare mutations dynamically with the `MutationRule` CRD. Modify pods without rebuilding or redeploying code.
- **Ultra-low Admission Latency**: Maintains compiled rules in an in-memory `RuleStore` for sub-millisecond RFC 6902 JSON patch evaluation.
- **Kubernetes Controller Patterns**:
  - `kube::runtime::Controller` reconciliation loop
  - Finalizer pattern (`mutation.webhook.io/finalizer`) for clean rule removal
  - Status subresource reporting (`valid`, `observedGeneration`, error messages)
  - Configurable error backoff and periodic requeuing
- **Zero-Dependency TLS Self-Bootstrapping**:
  - Automatically generates self-signed CA & server certificates with proper Kubernetes SANs using `rcgen`.
  - Automatically patches `caBundle` in `MutatingWebhookConfiguration`.
  - Also supports mounting external certificates from `cert-manager`.

---

## Mutation Capabilities

`MutationRule` supports:
1. **Target Selectors**:
   - `kinds`: Target specific resource kinds (default: `Pod`)
   - `namespaces`: Whitelist specific namespaces
   - `excludeNamespaces`: Exclude sensitive namespaces (e.g. `kube-system`)
   - `matchLabels`: Key-value matching on metadata labels
   - `matchAnnotations`: Key-value matching on metadata annotations
2. **Mutations**:
   - `addLabels`: Merge labels into resource metadata
   - `addAnnotations`: Merge annotations into resource metadata
   - `addEnv`: Inject environment variables into all containers
   - `addContainers`: Append sidecar containers to Pod specs
   - `addInitContainers`: Append init containers
   - `addVolumes`: Inject storage volumes
   - `addVolumeMounts`: Inject volume mounts into containers
   - `rawPatches`: Apply arbitrary RFC 6902 JSON patch operations

---

## Getting Started

### 1. Build and Run Unit Tests
```bash
cargo test
```

### 2. Export CRD Manifest
```bash
cargo run -- --export-crd > crds/mutationrule.yaml
```

### 3. Deploy to Kubernetes
```bash
# 1. Apply CRD
kubectl apply -f crds/mutationrule.yaml

# 2. Apply RBAC, Service, and Webhook Configuration
kubectl apply -f deploy/rbac.yaml
kubectl apply -f deploy/service.yaml
kubectl apply -f deploy/webhook-configuration.yaml

# 3. Deploy the Controller
kubectl apply -f deploy/deployment.yaml

# 4. Apply a sample MutationRule
kubectl apply -f deploy/example-rule.yaml
```

---

## Project Structure

```
├── Cargo.toml
├── Dockerfile
├── crds/
│   └── mutationrule.yaml              # Generated OpenAPI v3 CRD
├── deploy/
│   ├── rbac.yaml                      # ServiceAccount & ClusterRole
│   ├── service.yaml                   # ClusterIP Service routing 443 -> 8443
│   ├── webhook-configuration.yaml    # MutatingWebhookConfiguration
│   ├── deployment.yaml                # Controller deployment
│   └── example-rule.yaml              # Sample MutationRule resource
└── src/
    ├── lib.rs                         # Crate root
    ├── main.rs                        # CLI entrypoint & async orchestrator
    ├── crd.rs                         # MutationRule CRD & schemars derives
    ├── controller.rs                  # kube::runtime::Controller reconciler & finalizer
    ├── state.rs                       # In-memory RuleStore
    ├── error.rs                       # Custom Error & Result types
    ├── engine/
    │   ├── mod.rs                     # Engine module
    │   └── patcher.rs                 # JSON patch diffing & rule evaluation
    ├── tls/
    │   ├── mod.rs                     # TLS module
    │   └── bootstrap.rs               # rcgen certificate generation & caBundle patcher
    └── webhook/
        ├── mod.rs                     # Axum router & rustls server
        └── handlers.rs                # /mutate, /healthz, /readyz endpoints
```
