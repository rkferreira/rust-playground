# General Kubernetes Webhook Mutation Controller Documentation

Welcome to the comprehensive documentation for the **General Kubernetes Webhook Mutation Controller**. This project provides a generic, low-latency, CRD-driven Mutating Admission Webhook server and Kubernetes Controller written in Rust.

---

## Documentation Index

| Guide | Description |
|---|---|
| [1. Architecture & Design](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/docs/architecture.md) | Dual-runtime system architecture, controller reconciler loop, admission flow, and memory model. |
| [2. CRD Specification & Reference](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/docs/crd-reference.md) | Complete reference for the `MutationRule` CRD (selectors, action specs, priorities, and status conditions). |
| [3. TLS & Security Architecture](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/docs/tls-and-security.md) | Zero-dependency self-bootstrapping TLS, CA rotation, automatic `caBundle` patching, and cert-manager integration. |
| [4. Operator & Deployment Guide](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/docs/operator-guide.md) | Production deployment steps, configuration flags, RBAC requirements, and troubleshooting. |
| [5. Mutation Cookbook & Examples](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/docs/examples.md) | Real-world examples: sidecar injection, security contexts, environment variables, and RFC 6902 raw patches. |

---

## High-Level System Overview

```
                          Kubernetes API Server
                         /                     \
       (1) AdmissionReview (HTTPS)              \ (2) Watch CRD Events
                       /                         \
                      v                           v
         +-------------------------+  +-------------------------+
         |    Axum Webhook Server  |  |     Kube Controller     |
         |     (/mutate endpoint)  |  |    (Reconciliation)     |
         +-------------------------+  +-------------------------+
                      \                           /
                       \                         /
                        v                       v
                    +-------------------------------+
                    |     In-Memory RuleStore       |
                    |   (Arc<RwLock<CompiledRule>>) |
                    +-------------------------------+
```

### Core Tenets
1. **Low-Latency Admission**: Admission requests run directly against an in-memory compiled cache without calling the Kubernetes API within the request handler.
2. **Dynamic Configuration**: Rules are declared via Kubernetes Custom Resources (`MutationRule`). Modifications take effect immediately without restarting pods.
3. **Batteries-Included TLS**: Bootstraps its own internal CA, signs server certificates with proper Kubernetes SANs, and auto-patches the `MutatingWebhookConfiguration.caBundle`.
4. **Safety & Extensibility**: Combines structured high-level mutation primitives (env vars, sidecars, annotations) with raw RFC 6902 JSON patch support for unlimited flexibility.
