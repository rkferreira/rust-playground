# Architecture & Internal Design

This document details the internal design and mechanics of the General Kubernetes Webhook Mutation Controller.

---

## 1. Dual-Runtime Cooperative Model

A traditional Kubernetes operator typically operates asynchronously via event loops, while an admission webhook operates synchronously under strict deadlines (typically 2 to 5 seconds).

This project integrates both roles into a single binary managed by the `tokio` multi-threaded async runtime:

```
+--------------------------------------------------------------------------------+
|                                Tokio Async Runtime                             |
|                                                                                |
|   +------------------------------------+   +-------------------------------+   |
|   |         Task 1: Controller         |   |      Task 2: HTTPS Webhook    |   |
|   |------------------------------------|   |-------------------------------|   |
|   | - kube::runtime::Controller stream |   | - axum::Router                |   |
|   | - Watches MutationRule CRDs        |   | - rustls TLS acceptor         |   |
|   | - Finalizer & Status Subresource   |   | - /mutate, /healthz, /readyz  |   |
|   +-----------------+------------------+   +---------------+---------------+   |
|                     |                                      |                   |
|                     | Writes (upsert/delete)               | Reads (sorted)    |
|                     v                                      v                   |
|   +------------------------------------------------------------------------+   |
|   |                      Shared In-Memory RuleStore                        |   |
|   |                  std::sync::Arc<tokio::sync::RwLock>                   |   |
|   +------------------------------------------------------------------------+   |
+--------------------------------------------------------------------------------+
```

---

## 2. Kubernetes Controller Mechanics

The controller is implemented in [`src/controller.rs`](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/src/controller.rs) using `kube::runtime::Controller`:

### 2.1 Reconciler Loop
The reconciler is triggered upon:
- Addition, modification, or deletion of any `MutationRule` CRD.
- Periodic refresh requeue (default interval: 3600 seconds).
- Error retry backoff (default interval: 10 seconds).

```rust
async fn reconciler(rule: Arc<MutationRule>, ctx: Arc<ControllerContext>) -> Result<Action>
```

### 2.2 Finalizer Pattern
When a `MutationRule` is deleted, Kubernetes places a deletion timestamp on the resource. The controller uses the `kube::runtime::finalizer` pattern to ensure the rule is completely removed from the in-memory cache before allowing Kubernetes to finalize and prune the CRD from `etcd`:

- Finalizer string: `mutation.webhook.io/finalizer`
- On `Apply`: The rule is parsed, validated, compiled, and added to the `RuleStore`.
- On `Cleanup`: The rule is removed from `RuleStore`, and the finalizer is stripped.

### 2.3 Status Subresource Updates
The controller inspects rule syntax (e.g., ensuring at least one action is specified) and patches the status subresource (`/status`):
- `status.valid`: Boolean indicating if the rule passed semantic validation.
- `status.observedGeneration`: The generation number of the resource validated by the controller.
- `status.errorMessage`: Description of validation failure if invalid.

---

## 3. Webhook Server & Admission Lifecycle

The webhook server is built on [`axum`](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/src/webhook/mod.rs) and served via [`axum-server`](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/src/webhook/mod.rs) using native async TLS (`rustls`).

### 3.1 Admission Request Pipeline

```
1. Kubernetes API Server issues POST /mutate with AdmissionReview<Pod>
2. axum deserializes JSON payload into AdmissionReviewRequest
3. Extract request.object (cloned into mutable JSON Value)
4. Fetch active rules from RuleStore (sorted by priority descending)
5. Iterate through matching rules and apply transformations
6. Compute RFC 6902 JSON Patch (json_patch::diff)
7. Base64-encode patch array and construct AdmissionResponse
8. Return HTTP 200 OK with AdmissionReview response
```

### 3.2 Sub-Millisecond Execution Guarantee
Because mutating webhooks intercept Pod creation, latency must remain minimal.
- **Zero API roundtrips**: No external calls to the Kubernetes API occur inside the HTTP handler.
- **Pre-indexed memory**: Matching rules are evaluated directly in memory.
- **Minimal locking**: Read locks on `RuleStore` are held only long enough to clone rule references.

---

## 4. Graceful Shutdown & Signal Handling

The entrypoint in [`src/main.rs`](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/src/main.rs) uses `tokio::select!` to listen for `SIGINT` (Ctrl+C) and `SIGTERM` signals sent by Kubernetes when a pod is being terminated or rolled out.

Upon receiving the signal:
1. Stops accepting new admission requests on the HTTPS listener.
2. In-flight admission requests are completed gracefully.
3. Controller stream terminates cleanly.
