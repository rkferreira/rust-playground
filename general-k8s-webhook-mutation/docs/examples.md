# Mutation Cookbook & Practical Recipes

This guide contains real-world `MutationRule` examples for common Kubernetes operational use cases.

---

## Recipe 1: Injecting Global Observability Environment Variables

Inject OpenTelemetry configuration into all pods in user namespaces without altering developer manifests:

```yaml
apiVersion: mutation.webhook.io/v1alpha1
kind: MutationRule
metadata:
  name: inject-otel-env
  namespace: default
spec:
  priority: 200
  selector:
    kinds: ["Pod"]
    excludeNamespaces:
      - kube-system
      - kube-public
      - kube-node-lease
  mutations:
    addEnv:
      - name: OTEL_SERVICE_NAME
        valueFrom:
          fieldRef:
            fieldPath: metadata.name
      - name: OTEL_EXPORTER_OTLP_ENDPOINT
        value: "http://otel-collector.observability.svc:4317"
      - name: OTEL_PROPAGATORS
        value: "tracecontext,baggage"
```

---

## Recipe 2: Sidecar Container & Shared Volume Injection

Inject a log forwarder sidecar into all pods labeled with `logging: forward`:

```yaml
apiVersion: mutation.webhook.io/v1alpha1
kind: MutationRule
metadata:
  name: inject-fluentbit-sidecar
  namespace: default
spec:
  priority: 100
  selector:
    kinds: ["Pod"]
    matchLabels:
      logging: "forward"
  mutations:
    addLabels:
      sidecar.logging.io/injected: "true"
    addVolumes:
      - name: shared-logs
        emptyDir: {}
    addVolumeMounts:
      - name: shared-logs
        mountPath: /var/log/application
    addContainers:
      - name: log-collector
        image: fluent/fluent-bit:2.2
        volumeMounts:
          - name: shared-logs
            mountPath: /var/log/application
            readOnly: true
        resources:
          limits:
            cpu: 100m
            memory: 64Mi
          requests:
            cpu: 20m
            memory: 32Mi
```

---

## Recipe 3: Enforcing Security Contexts via Raw RFC 6902 Patches

Use `rawPatches` to enforce non-root execution and drop all Linux capabilities:

```yaml
apiVersion: mutation.webhook.io/v1alpha1
kind: MutationRule
metadata:
  name: enforce-security-baseline
  namespace: default
spec:
  priority: 300
  selector:
    kinds: ["Pod"]
    excludeNamespaces:
      - kube-system
  mutations:
    rawPatches:
      # Enforce pod-level non-root
      - op: "add"
        path: "/spec/securityContext"
        value:
          runAsNonRoot: true
          runAsUser: 10001
          fsGroup: 10001
```

---

## Recipe 4: Cost-Allocation & Team Labels

Automatically inject metadata labels based on namespace policies:

```yaml
apiVersion: mutation.webhook.io/v1alpha1
kind: MutationRule
metadata:
  name: tag-team-finance
  namespace: finance-workloads
spec:
  priority: 50
  selector:
    kinds: ["Pod"]
    namespaces:
      - finance-workloads
  mutations:
    addLabels:
      cost-center: "finance-101"
      managed-by: "platform-team"
    addAnnotations:
      audit.security.io/compliance-level: "pci-dss"
```

---

## Recipe 5: Node Affinity / Spot Instance Tolerations

Add tolerations for Spot/Preemptible node pools to batch processing pods:

```yaml
apiVersion: mutation.webhook.io/v1alpha1
kind: MutationRule
metadata:
  name: allow-spot-nodes
  namespace: default
spec:
  priority: 80
  selector:
    kinds: ["Pod"]
    matchLabels:
      workload-type: "batch"
  mutations:
    rawPatches:
      - op: "add"
        path: "/spec/tolerations"
        value:
          - key: "cloud.google.com/gke-spot"
            operator: "Equal"
            value: "true"
            effect: "NoSchedule"
          - key: "node.kubernetes.io/preemptible"
            operator: "Exists"
            effect: "NoSchedule"
```
