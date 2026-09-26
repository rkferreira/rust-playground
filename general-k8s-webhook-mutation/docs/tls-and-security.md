# TLS & Security Architecture

Kubernetes requires all Admission Webhooks to communicate exclusively over **HTTPS** (TLS) with a valid certificate trust chain.

This document describes how TLS is handled, including the self-bootstrapping mechanism and production configurations with `cert-manager`.

---

## 1. Zero-Dependency Self-Bootstrapping TLS

To simplify testing and deployments in environments without an existing PKI operator, the controller features a built-in TLS bootstrapper in [`src/tls/bootstrap.rs`](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/src/tls/bootstrap.rs).

```
1. Startup: Controller generates internal X.509 CA KeyPair & Certificate (rcgen)
2. Issues Server KeyPair & Certificate signed by this CA with SANs:
   - <service-name>
   - <service-name>.<namespace>
   - <service-name>.<namespace>.svc
   - <service-name>.<namespace>.svc.cluster.local
3. Patches MutatingWebhookConfiguration in Kubernetes:
   Updates webhooks[*].clientConfig.caBundle with base64-encoded CA certificate
4. Starts axum HTTPS listener using server certificate and private key in-memory
```

### Advantages
- **Zero external dependencies**: Does not require `cert-manager`, OpenSSL CLI binaries, or external scripts.
- **Automated CA Bundle sync**: Eliminates manual base64 encoding and copy-pasting into Kubernetes manifests.

### Enabling Self-Bootstrapping
Ensure the following flags / environment variables are configured on the controller:
```yaml
env:
  - name: SELF_BOOTSTRAP_TLS
    value: "true"
  - name: WEBHOOK_CONFIG_NAME
    value: "webhook-mutation-rules"
  - name: SERVICE_NAME
    value: "webhook-mutation-service"
  - name: NAMESPACE
    value: "default"
```

---

## 2. Production Integration with cert-manager

For clusters with an enterprise PKI managed by `cert-manager`, you can disable self-bootstrapping and mount certificates issued by a `Certificate` resource.

### Step 1: Create cert-manager Issuer & Certificate
```yaml
apiVersion: cert-manager.io/v1
kind: Issuer
metadata:
  name: webhook-selfsigned-issuer
  namespace: default
spec:
  selfSigned: {}
---
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: webhook-server-cert
  namespace: default
spec:
  secretName: webhook-tls-secret
  dnsNames:
    - webhook-mutation-service.default.svc
    - webhook-mutation-service.default.svc.cluster.local
  issuerRef:
    name: webhook-selfsigned-issuer
    kind: Issuer
```

### Step 2: Configure Webhook Manifest for cert-manager CA Injection
Add the `cert-manager.io/inject-ca-from` annotation to `MutatingWebhookConfiguration`:
```yaml
apiVersion: admissionregistration.k8s.io/v1
kind: MutatingWebhookConfiguration
metadata:
  name: webhook-mutation-rules
  annotations:
    cert-manager.io/inject-ca-from: default/webhook-server-cert
webhooks:
  - name: mutate.rules.mutation.webhook.io
    ...
```

### Step 3: Mount the Secret in the Controller Deployment
Pass `--self-bootstrap-tls=false` and `--cert-dir=/etc/webhook/certs`:
```yaml
containers:
  - name: webhook
    command:
      - /usr/local/bin/general-k8s-webhook-mutation
      - --self-bootstrap-tls=false
      - --cert-dir=/etc/webhook/certs
    volumeMounts:
      - name: webhook-certs
        mountPath: /etc/webhook/certs
        readOnly: true
volumes:
  - name: webhook-certs
    secret:
      secretName: webhook-tls-secret
```

---

## 3. RBAC Permissions Matrix

The controller requires specific permissions defined in [`deploy/rbac.yaml`](file:///Users/rodrigoke/rust/general-k8s-webhook-mutation/deploy/rbac.yaml):

| API Group | Resources | Verbs | Justification |
|---|---|---|---|
| `mutation.webhook.io` | `mutationrules` | `get`, `list`, `watch`, `patch`, `update` | Watching and compiling rules |
| `mutation.webhook.io` | `mutationrules/status`, `mutationrules/finalizers` | `get`, `patch`, `update` | Updating rule validity status and managing finalizers |
| `admissionregistration.k8s.io` | `mutatingwebhookconfigurations` | `get`, `list`, `watch`, `patch`, `update` | Auto-patching `caBundle` in self-bootstrap mode |

---

## 4. Workload Security Hardening

For production environments, ensure the controller pod adheres to the **Restricted** Kubernetes Pod Security Standard:
```yaml
securityContext:
  runAsNonRoot: true
  runAsUser: 10001
  runAsGroup: 10001
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
  capabilities:
    drop:
      - ALL
```
