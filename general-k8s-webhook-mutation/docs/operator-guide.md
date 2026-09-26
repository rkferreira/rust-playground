# Operator & Deployment Guide

This guide covers deployment, operational management, monitoring, and troubleshooting for the General Kubernetes Webhook Mutation Controller.

---

## 1. CLI Arguments & Environment Variables

All parameters can be supplied via command-line flags or environment variables:

| Flag | Environment Variable | Default | Description |
|---|---|---|---|
| `-p`, `--port` | `PORT` | `8443` | Port for the HTTPS admission webhook listener. |
| `--host` | `HOST` | `0.0.0.0` | Bind IP address for the web server. |
| `--cert-dir` | `CERT_DIR` | None | Directory path containing `tls.crt` and `tls.key`. |
| `--self-bootstrap-tls` | `SELF_BOOTSTRAP_TLS` | `true` | Generate internal self-signed TLS certificates. |
| `--webhook-config-name`| `WEBHOOK_CONFIG_NAME` | None | Name of the `MutatingWebhookConfiguration` to patch. |
| `--service-name` | `SERVICE_NAME` | `webhook-mutation-service` | Service name used for certificate DNS SANs. |
| `--namespace` | `NAMESPACE` | `default` | Namespace where the webhook service resides. |
| `--export-crd` | N/A | `false` | Prints the generated CRD YAML to standard output and exits. |

---

## 2. Deployment Walkthrough

### Step 1: Install CustomResourceDefinition (CRD)
Generate and apply the CRD manifest:
```bash
# From workspace:
kubectl apply -f crds/mutationrule.yaml

# Verify registration
kubectl get crd mutationrules.mutation.webhook.io
```

### Step 2: Deploy RBAC, Service, and Webhook Registration
```bash
kubectl apply -f deploy/rbac.yaml
kubectl apply -f deploy/service.yaml
kubectl apply -f deploy/webhook-configuration.yaml
```

### Step 3: Deploy the Controller Pod
```bash
kubectl apply -f deploy/deployment.yaml

# Check rollout status
kubectl rollout status deployment/webhook-mutation-controller
```

---

## 3. Observability & Probes

### 3.1 Probes
The controller exposes two HTTP probes over HTTPS on the main port (8443):

| Endpoint | Purpose | Description |
|---|---|---|
| `GET /healthz` | Liveness Probe | Returns `200 OK` ("ok") if the Tokio runtime and HTTP server are responsive. |
| `GET /readyz` | Readiness Probe | Returns `200 OK` ("ready (active rules: N)") indicating how many rules are loaded in memory. |

### 3.2 Structured Logging & Tracing
The controller uses the `tracing` framework. Configure verbosity with `RUST_LOG`:
```yaml
env:
  - name: RUST_LOG
    # Info for general logs, debug for mutation evaluation diffs
    value: "info,general_k8s_webhook_mutation=debug"
```

Sample log output:
```
INFO general_k8s_webhook_mutation: Starting TLS webhook server addr=0.0.0.0:8443
INFO general_k8s_webhook_mutation: Connected to Kubernetes cluster
INFO general_k8s_webhook_mutation::controller: Reconciling and compiling MutationRule rule="default/inject-observability-sidecar"
INFO general_k8s_webhook_mutation::controller: Successfully stored MutationRule rule="default/inject-observability-sidecar" active_rules=1
INFO general_k8s_webhook_mutation::webhook::handlers: Processing admission mutation request uid="705ab4f5-6393" kind="Pod" operation="CREATE"
INFO general_k8s_webhook_mutation::engine::patcher: Applying mutation rule rule="inject-observability-sidecar" kind="Pod" namespace=Some("default")
DEBUG general_k8s_webhook_mutation::engine::patcher: Generated JSON patch patch_json="[{\"op\":\"add\",\"path\":\"/metadata/labels/mutated-by\",\"value\":\"rust-webhook\"}]"
```

---

## 4. Troubleshooting & Best Practices

### Issue: Pod Creation Times Out or Fails with `failed calling webhook`
1. Check webhook pod status:
   ```bash
   kubectl get pods -l app.kubernetes.io/name=general-k8s-webhook-mutation
   kubectl logs -l app.kubernetes.io/name=general-k8s-webhook-mutation
   ```
2. Check `caBundle` in `MutatingWebhookConfiguration`:
   ```bash
   kubectl get mutatingwebhookconfigurations webhook-mutation-rules -o yaml
   ```
   If `caBundle` is empty, check whether the controller has RBAC permission to patch `mutatingwebhookconfigurations`.
3. Safe Testing with `failurePolicy: Ignore`:
   During initial testing in non-production clusters, you can set `failurePolicy: Ignore` in `deploy/webhook-configuration.yaml`. This ensures that even if the webhook goes down, pod creation continues without failing.

### Issue: Rule Is Not Applying to Target Pods
1. Inspect rule status:
   ```bash
   kubectl describe mutationrule <rule-name>
   ```
   Look at `Status.Valid` and `Status.ErrorMessage`.
2. Check namespace exclusion:
   Ensure the target pod is not in an excluded namespace (such as `kube-system`).
3. Label matching:
   Verify that all labels specified in `spec.selector.matchLabels` exist with identical string keys and values on the target pod's `metadata.labels`.
