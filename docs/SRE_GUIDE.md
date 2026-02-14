# CampaignExpress — Operating Guide for Kubernetes/SRE Specialists (College Freshers)

## Table of Contents

1. [Welcome](#welcome)
2. [What You'll Be Working On](#what-youll-be-working-on)
3. [Prerequisites & Setup](#prerequisites--setup)
4. [Key Technologies & Concepts](#key-technologies--concepts)
5. [Infrastructure Overview](#infrastructure-overview)
6. [Your Operational Workflow](#your-operational-workflow)
7. [Common Tasks & Examples](#common-tasks--examples)
8. [Best Practices & Tips](#best-practices--tips)
9. [Learning Resources](#learning-resources)
10. [Getting Help](#getting-help)

---

## Welcome

Welcome to the CampaignExpress SRE (Site Reliability Engineering) team! As a college fresher, you're joining a team that keeps a high-performance platform running 24/7, serving **50 million offers per hour** across a **20-node Kubernetes cluster**. This guide will help you understand infrastructure, operations, and how to keep services reliable.

### What Makes CampaignExpress Infrastructure Special?

- **Cloud-Native**: Built for Kubernetes from day one
- **High Availability**: Multi-node, multi-zone deployment with automatic failover
- **Auto-Scaling**: Horizontal Pod Autoscaling based on CPU, memory, and custom metrics
- **Observability**: Prometheus metrics, Grafana dashboards, distributed tracing with Tempo
- **GitOps**: Infrastructure as Code with Terraform and Kustomize
- **Security**: Network policies, cert-manager for TLS, External Secrets integration

### Your Role as an SRE

You'll be responsible for:
- **Reliability**: Keeping services up and running (99.9%+ uptime)
- **Performance**: Ensuring fast response times (<10ms for bid requests)
- **Scalability**: Handling traffic spikes and growth
- **Security**: Protecting infrastructure and data
- **Monitoring**: Watching metrics, logs, and alerts
- **Incident Response**: Troubleshooting and fixing issues quickly
- **Automation**: Building tools to reduce manual toil

---

## What You'll Be Working On

### Infrastructure Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Azure Cloud (AKS)                        │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              HAProxy Load Balancer                  │   │
│  │         (Ingress Controller + TLS Termination)      │   │
│  └────────────────┬────────────────────────────────────┘   │
│                   │                                         │
│  ┌────────────────┴────────────────────────────────────┐   │
│  │          Campaign Express Pods (20 nodes)           │   │
│  │    Each runs: API Server + 20 Bid Agents           │   │
│  │    Horizontal Pod Autoscaling (HPA) enabled         │   │
│  └────────┬────────────────────────────┬────────────────┘  │
│           │                            │                    │
│  ┌────────▼──────┐  ┌────────────────▼────────┐          │
│  │ NATS JetStream│  │  Redis Cluster (6 nodes)│          │
│  │  StatefulSet  │  │   Persistent Storage    │          │
│  │  3 replicas   │  │   Cluster mode          │          │
│  └───────────────┘  └─────────────────────────┘          │
│                                                             │
│  ┌──────────────────┐  ┌──────────────────────────────┐   │
│  │   ClickHouse DB  │  │  Monitoring Stack            │   │
│  │   StatefulSet    │  │  - Prometheus                │   │
│  │   1 replica      │  │  - Grafana                   │   │
│  │   Analytics data │  │  - AlertManager              │   │
│  └──────────────────┘  │  - Tempo (tracing)           │   │
│                        │  - Loki (logs)               │   │
│                        └──────────────────────────────┘   │
│                                                             │
│  ┌──────────────────────────────────────────────────────┐ │
│  │             Azure Services                           │ │
│  │  - Azure Container Registry (ACR)                   │ │
│  │  - Azure Key Vault (secrets)                        │ │
│  │  - Azure Monitor (cloud monitoring)                 │ │
│  └──────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Your Responsibilities by Area

#### 1. Kubernetes Operations
- Deploying and updating applications
- Managing StatefulSets and Deployments
- Configuring autoscaling (HPA, VPA)
- Managing Persistent Volumes
- Networking and Service configuration

#### 2. Monitoring & Alerting
- Grafana dashboard creation and maintenance
- Prometheus alert rule configuration
- Log aggregation with Loki
- Distributed tracing with Tempo
- On-call incident response

#### 3. Infrastructure as Code (IaC)
- Terraform for Azure resources (AKS, Redis, ACR, Key Vault)
- Kustomize for Kubernetes manifests
- Helm charts for application deployment
- GitOps workflows with Git

#### 4. CI/CD Pipelines
- GitHub Actions workflows
- Docker image building and scanning
- Automated deployments to staging/production
- Rollback procedures

#### 5. Security
- Network policies (pod-to-pod communication)
- TLS certificate management (cert-manager)
- Secret management (External Secrets Operator + Azure Key Vault)
- RBAC configuration
- Security scanning and compliance

---

## Prerequisites & Setup

### 1. Install Core Tools

#### kubectl (Kubernetes CLI)
```bash
# On macOS:
brew install kubectl

# On Ubuntu/Debian:
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl

# Verify
kubectl version --client
```

#### Docker
```bash
# Follow instructions at: https://docs.docker.com/get-docker/

# Verify
docker --version
docker ps
```

#### Helm (Kubernetes Package Manager)
```bash
# On macOS:
brew install helm

# On Ubuntu/Debian:
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash

# Verify
helm version
```

#### Terraform (Infrastructure as Code)
```bash
# On macOS:
brew tap hashicorp/tap
brew install hashicorp/tap/terraform

# On Ubuntu/Debian:
wget -O- https://apt.releases.hashicorp.com/gpg | sudo gpg --dearmor -o /usr/share/keyrings/hashicorp-archive-keyring.gpg
echo "deb [signed-by=/usr/share/keyrings/hashicorp-archive-keyring.gpg] https://apt.releases.hashicorp.com $(lsb_release -cs) main" | sudo tee /etc/apt/sources.list.d/hashicorp.list
sudo apt update && sudo apt install terraform

# Verify
terraform version
```

#### Azure CLI (if using Azure)
```bash
# On macOS:
brew install azure-cli

# On Ubuntu/Debian:
curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash

# Login
az login

# Verify
az account show
```

### 2. Install Kubernetes Tools

```bash
# k9s - Terminal UI for Kubernetes
brew install k9s
# or
curl -sS https://webinstall.dev/k9s | bash

# kubectx + kubens - Switch contexts/namespaces easily
brew install kubectx
# or
sudo apt install kubectx

# stern - Multi-pod log tailing
brew install stern
# or
curl -sS https://webinstall.dev/stern | bash

# kustomize - Template-free Kubernetes configuration
brew install kustomize
# or
curl -s "https://raw.githubusercontent.com/kubernetes-sigs/kustomize/master/hack/install_kustomize.sh" | bash
```

### 3. Set Up Local Kubernetes Cluster

For development and testing:

```bash
# Option 1: Minikube (full Kubernetes on your laptop)
brew install minikube
minikube start --cpus=4 --memory=8192

# Option 2: Docker Desktop (includes Kubernetes)
# Enable Kubernetes in Docker Desktop settings

# Option 3: kind (Kubernetes in Docker)
brew install kind
kind create cluster --name campaign-express
```

### 4. Clone the Repository

```bash
git clone https://github.com/Pushparajan/CampaignExpress.git
cd CampaignExpress
```

### 5. Connect to Your Kubernetes Cluster

```bash
# For Azure AKS:
az aks get-credentials --resource-group campaign-express-prod --name campaign-express-aks

# Verify connection
kubectl cluster-info
kubectl get nodes

# Set default namespace
kubectl config set-context --current --namespace=campaign-express
```

**🎉 Setup Complete!** You're ready to manage Kubernetes infrastructure!

---

## Key Technologies & Concepts

### 1. Kubernetes Core Concepts

#### Pods
The smallest deployable unit in Kubernetes. Contains one or more containers.

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: campaign-express-pod
  labels:
    app: campaign-express
spec:
  containers:
  - name: campaign-express
    image: ghcr.io/pushparajan/campaign-express:v1.2.3
    ports:
    - containerPort: 8080
      name: http
    - containerPort: 9090
      name: grpc
    resources:
      requests:
        memory: "512Mi"
        cpu: "500m"
      limits:
        memory: "2Gi"
        cpu: "2000m"
```

#### Deployments
Manages a set of identical Pods with desired state and rolling updates.

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: campaign-express
spec:
  replicas: 20  # 20 instances
  selector:
    matchLabels:
      app: campaign-express
  template:
    metadata:
      labels:
        app: campaign-express
    spec:
      containers:
      - name: campaign-express
        image: ghcr.io/pushparajan/campaign-express:v1.2.3
        # ... container spec ...
```

#### Services
Expose Pods via a stable network endpoint.

```yaml
apiVersion: v1
kind: Service
metadata:
  name: campaign-express-svc
spec:
  selector:
    app: campaign-express
  ports:
  - name: http
    port: 80
    targetPort: 8080
  - name: grpc
    port: 9090
    targetPort: 9090
  type: ClusterIP  # or LoadBalancer for external access
```

#### StatefulSets
Like Deployments, but for stateful applications (databases, queues).

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: redis
spec:
  serviceName: redis
  replicas: 6  # 6-node Redis cluster
  selector:
    matchLabels:
      app: redis
  template:
    metadata:
      labels:
        app: redis
    spec:
      containers:
      - name: redis
        image: redis:7-alpine
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 20Gi
```

#### ConfigMaps & Secrets
Store configuration and sensitive data.

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: campaign-config
data:
  RUST_LOG: "info"
  AGENTS_PER_NODE: "20"
  NATS_URL: "nats://nats:4222"

---
apiVersion: v1
kind: Secret
metadata:
  name: redis-password
type: Opaque
data:
  password: bXlzZWNyZXRwYXNzd29yZA==  # base64 encoded
```

### 2. Horizontal Pod Autoscaling (HPA)

Automatically scale Pods based on metrics:

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: campaign-express-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: campaign-express
  minReplicas: 10
  maxReplicas: 50
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### 3. Persistent Volumes (PV) & Persistent Volume Claims (PVC)

Store data that survives Pod restarts:

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: clickhouse-data
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 100Gi
  storageClassName: managed-premium  # Azure Premium SSD
```

### 4. Networking

#### Ingress
Expose HTTP/HTTPS routes to services.

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: campaign-express-ingress
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  ingressClassName: haproxy
  tls:
  - hosts:
    - api.campaignexpress.com
    secretName: campaign-tls
  rules:
  - host: api.campaignexpress.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: campaign-express-svc
            port:
              number: 80
```

#### Network Policies
Control traffic between Pods.

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: campaign-express-netpol
spec:
  podSelector:
    matchLabels:
      app: campaign-express
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: haproxy
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: nats
    ports:
    - protocol: TCP
      port: 4222
  - to:
    - podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
```

### 5. Monitoring with Prometheus

#### Metrics Collection
Prometheus scrapes metrics from Pods:

```yaml
apiVersion: v1
kind: Service
metadata:
  name: campaign-express-metrics
  labels:
    app: campaign-express
  annotations:
    prometheus.io/scrape: "true"
    prometheus.io/port: "9091"
    prometheus.io/path: "/metrics"
spec:
  selector:
    app: campaign-express
  ports:
  - name: metrics
    port: 9091
```

#### Alert Rules
Define when to alert:

```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: campaign-express-alerts
spec:
  groups:
  - name: campaign-express
    interval: 30s
    rules:
    - alert: HighErrorRate
      expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
      for: 5m
      labels:
        severity: critical
      annotations:
        summary: "High error rate detected"
        description: "Error rate is {{ $value }} errors/sec"
    
    - alert: HighLatency
      expr: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m])) > 0.01
      for: 10m
      labels:
        severity: warning
      annotations:
        summary: "High latency detected"
        description: "P95 latency is {{ $value }}s"
```

### 6. GitOps with Kustomize

**Base configuration** (shared across environments):
```yaml
# deploy/k8s/base/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: campaign-express
spec:
  replicas: 1  # Override in overlays
  template:
    spec:
      containers:
      - name: campaign-express
        image: ghcr.io/pushparajan/campaign-express:latest
```

**Production overlay**:
```yaml
# deploy/k8s/overlays/prod/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
bases:
  - ../../base
replicas:
  - name: campaign-express
    count: 20
images:
  - name: ghcr.io/pushparajan/campaign-express
    newTag: v1.2.3
configMapGenerator:
  - name: campaign-config
    literals:
      - RUST_LOG=info
      - AGENTS_PER_NODE=20
```

Deploy with:
```bash
kubectl apply -k deploy/k8s/overlays/prod
```

### 7. Infrastructure as Code with Terraform

```hcl
# deploy/terraform/azure/main.tf

resource "azurerm_kubernetes_cluster" "aks" {
  name                = "campaign-express-aks"
  location            = "East US"
  resource_group_name = azurerm_resource_group.rg.name
  dns_prefix          = "campaign-express"
  
  default_node_pool {
    name       = "default"
    node_count = 3
    vm_size    = "Standard_D4s_v3"
  }
  
  identity {
    type = "SystemAssigned"
  }
}

resource "azurerm_redis_cache" "redis" {
  name                = "campaign-express-redis"
  location            = azurerm_resource_group.rg.location
  resource_group_name = azurerm_resource_group.rg.name
  capacity            = 1
  family              = "P"  # Premium
  sku_name            = "Premium"
  enable_non_ssl_port = false
  
  redis_configuration {
    maxmemory_policy = "allkeys-lru"
  }
}
```

Apply with:
```bash
cd deploy/terraform/azure
terraform init
terraform plan
terraform apply
```

---

## Infrastructure Overview

### CampaignExpress Architecture in Kubernetes

```
Namespace: campaign-express
├── Deployments
│   ├── campaign-express (20 replicas)
│   ├── haproxy (2 replicas)
│   └── ui (3 replicas)
│
├── StatefulSets
│   ├── nats (3 replicas with persistent storage)
│   ├── redis (6 replicas in cluster mode)
│   └── clickhouse (1 replica with large volume)
│
├── Services
│   ├── campaign-express-svc (ClusterIP)
│   ├── haproxy-lb (LoadBalancer)
│   ├── nats-svc (ClusterIP)
│   ├── redis-svc (ClusterIP)
│   └── clickhouse-svc (ClusterIP)
│
├── Ingress
│   └── campaign-ingress (TLS with Let's Encrypt)
│
├── ConfigMaps
│   ├── campaign-config (environment variables)
│   └── haproxy-config (load balancer settings)
│
├── Secrets
│   ├── redis-password (from Azure Key Vault)
│   ├── clickhouse-credentials (from Azure Key Vault)
│   └── campaign-tls (TLS certificate)
│
├── HPA
│   └── campaign-express-hpa (scale 10-50 replicas)
│
└── NetworkPolicies
    ├── campaign-netpol
    ├── redis-netpol
    └── nats-netpol

Namespace: monitoring
├── Deployments
│   ├── prometheus (1 replica)
│   ├── grafana (1 replica)
│   └── alertmanager (1 replica)
│
└── StatefulSets
    ├── tempo (distributed tracing)
    └── loki (log aggregation)
```

### Resource Allocation

#### Per Campaign Express Pod
- **CPU**: 500m request, 2000m limit
- **Memory**: 512Mi request, 2Gi limit
- **Storage**: None (stateless)

#### NATS (per replica)
- **CPU**: 250m request, 1000m limit
- **Memory**: 256Mi request, 1Gi limit
- **Storage**: 10Gi persistent volume

#### Redis (per node)
- **CPU**: 500m request, 1000m limit
- **Memory**: 2Gi request, 4Gi limit
- **Storage**: 20Gi persistent volume

#### ClickHouse
- **CPU**: 2000m request, 4000m limit
- **Memory**: 4Gi request, 8Gi limit
- **Storage**: 100Gi persistent volume

---

## Your Operational Workflow

### Daily Operations

#### 1. Morning Health Check

```bash
# Check cluster health
kubectl get nodes
kubectl top nodes

# Check all pods
kubectl get pods -n campaign-express
kubectl get pods -n monitoring

# Check for failed pods
kubectl get pods --field-selector=status.phase!=Running -A

# Check recent events
kubectl get events -n campaign-express --sort-by='.lastTimestamp' | tail -20
```

#### 2. Monitor Key Metrics

```bash
# Open Grafana
kubectl port-forward -n monitoring svc/grafana 3000:3000
# Visit http://localhost:3000

# Check Prometheus alerts
kubectl port-forward -n monitoring svc/alertmanager 9093:9093
# Visit http://localhost:9093

# Quick CLI metrics check
kubectl top pods -n campaign-express --sort-by=cpu
kubectl top pods -n campaign-express --sort-by=memory
```

#### 3. Review Logs

```bash
# Tail logs from all campaign-express pods
stern campaign-express -n campaign-express

# Check for errors in the last hour
kubectl logs -n campaign-express -l app=campaign-express --since=1h | grep -i error

# View logs in Grafana Loki
# Grafana → Explore → Loki → {namespace="campaign-express"}
```

#### 4. Capacity Planning

```bash
# Check HPA status
kubectl get hpa -n campaign-express

# Check resource usage trends
# Go to Grafana → Dashboards → Kubernetes Cluster

# Check persistent volume usage
kubectl get pvc -n campaign-express
kubectl exec -it clickhouse-0 -n campaign-express -- df -h /var/lib/clickhouse
```

### Deployment Workflow

#### 1. Deploy a New Version

```bash
# Update image tag in Kustomize overlay
cd deploy/k8s/overlays/staging
kustomize edit set image ghcr.io/pushparajan/campaign-express:v1.2.4

# Preview changes
kubectl diff -k deploy/k8s/overlays/staging

# Apply to staging
kubectl apply -k deploy/k8s/overlays/staging

# Watch rollout
kubectl rollout status deployment/campaign-express -n campaign-express

# Check new pods
kubectl get pods -n campaign-express -l app=campaign-express
```

#### 2. Rollback if Needed

```bash
# View rollout history
kubectl rollout history deployment/campaign-express -n campaign-express

# Rollback to previous version
kubectl rollout undo deployment/campaign-express -n campaign-express

# Or rollback to specific revision
kubectl rollout undo deployment/campaign-express -n campaign-express --to-revision=3
```

#### 3. Blue-Green Deployment (Zero Downtime)

```bash
# Deploy "green" version alongside "blue"
kubectl apply -f deploy/k8s/green-deployment.yaml

# Wait for green to be ready
kubectl wait --for=condition=available --timeout=300s deployment/campaign-express-green

# Switch traffic to green
kubectl patch service campaign-express-svc -p '{"spec":{"selector":{"version":"green"}}}'

# Monitor for issues
# If issues arise, switch back to blue
kubectl patch service campaign-express-svc -p '{"spec":{"selector":{"version":"blue"}}}'

# After successful switch, delete blue
kubectl delete deployment campaign-express-blue
```

### Incident Response Workflow

#### 1. Incident Detected (Alert Fires)

```bash
# Check alert in AlertManager
open http://localhost:9093

# Quickly assess impact
kubectl get pods -n campaign-express
kubectl top nodes
kubectl top pods -n campaign-express
```

#### 2. Identify Root Cause

```bash
# Check pod logs
kubectl logs -f campaign-express-xyz123 -n campaign-express

# Check recent events
kubectl describe pod campaign-express-xyz123 -n campaign-express

# Check resource usage
kubectl top pod campaign-express-xyz123 -n campaign-express

# Check dependencies (NATS, Redis, ClickHouse)
kubectl get pods -l app=nats -n campaign-express
kubectl logs -l app=redis -n campaign-express --tail=50
```

#### 3. Mitigate

Common mitigations:
- **Pod crash loop**: Check logs, rollback if recent deploy
- **High CPU/memory**: Scale up HPA, increase resource limits
- **Network issues**: Check NetworkPolicies, Service endpoints
- **Storage full**: Expand PVC, clean up old data

```bash
# Scale up immediately
kubectl scale deployment campaign-express -n campaign-express --replicas=30

# Restart crashed pods
kubectl delete pod campaign-express-xyz123 -n campaign-express

# Expand PVC (if supported by storage class)
kubectl patch pvc clickhouse-data -n campaign-express -p '{"spec":{"resources":{"requests":{"storage":"150Gi"}}}}'
```

#### 4. Post-Incident

```bash
# Document in incident log
# Prepare postmortem
# Create follow-up tasks to prevent recurrence
```

---

## Common Tasks & Examples

### Task 1: Deploy a Configuration Change

**Scenario**: Update NATS URL in config.

```bash
# Edit ConfigMap
kubectl edit configmap campaign-config -n campaign-express

# Or apply a new version
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: ConfigMap
metadata:
  name: campaign-config
  namespace: campaign-express
data:
  NATS_URL: "nats://nats-new:4222"
  RUST_LOG: "info"
EOF

# Restart pods to pick up new config
kubectl rollout restart deployment/campaign-express -n campaign-express
```

### Task 2: Scale Application

**Scenario**: Handle increased traffic.

```bash
# Manual scaling
kubectl scale deployment campaign-express -n campaign-express --replicas=30

# Or update HPA
kubectl patch hpa campaign-express-hpa -n campaign-express -p '{"spec":{"maxReplicas":60}}'

# Verify
kubectl get hpa campaign-express-hpa -n campaign-express
```

### Task 3: Debug a Crashing Pod

**Scenario**: Pod keeps restarting.

```bash
# Check pod status
kubectl get pod campaign-express-abc123 -n campaign-express

# View logs from current container
kubectl logs campaign-express-abc123 -n campaign-express

# View logs from previous container (if it crashed)
kubectl logs campaign-express-abc123 -n campaign-express --previous

# Describe pod to see events
kubectl describe pod campaign-express-abc123 -n campaign-express

# Check if it's a resource issue
kubectl top pod campaign-express-abc123 -n campaign-express

# Exec into pod (if it's running)
kubectl exec -it campaign-express-abc123 -n campaign-express -- /bin/sh

# Check application health endpoint
kubectl exec -it campaign-express-abc123 -n campaign-express -- curl localhost:8080/health
```

### Task 4: Update TLS Certificate

**Scenario**: Renew Let's Encrypt certificate.

```bash
# cert-manager automatically renews, but if you need to force:
kubectl delete certificate campaign-tls -n campaign-express

# Wait for cert-manager to re-issue
kubectl get certificate campaign-tls -n campaign-express -w

# Verify new cert
kubectl get secret campaign-tls -n campaign-express -o json | jq -r '.data."tls.crt"' | base64 -d | openssl x509 -noout -dates
```

### Task 5: Backup and Restore Data

**Scenario**: Backup ClickHouse data.

```bash
# Create a snapshot of the PVC
kubectl get pvc clickhouse-data -n campaign-express -o yaml > clickhouse-pvc-backup.yaml

# Or use Velero for backup/restore
velero backup create clickhouse-backup --include-namespaces campaign-express --selector app=clickhouse

# Restore if needed
velero restore create --from-backup clickhouse-backup
```

### Task 6: Investigate High Latency

**Scenario**: API response times are slow.

```bash
# Check Prometheus metrics
kubectl port-forward -n monitoring svc/prometheus 9090:9090
# Visit http://localhost:9090
# Query: histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# Check if pods are throttled (CPU limit reached)
kubectl describe pod campaign-express-abc123 -n campaign-express | grep -i throttl

# Check node resources
kubectl top nodes

# Check if Redis is slow
kubectl exec -it redis-0 -n campaign-express -- redis-cli --latency

# Check NATS throughput
kubectl logs -l app=nats -n campaign-express | grep -i "throughput"
```

### Task 7: Set Up a New Alert

**Scenario**: Alert when cache hit rate drops below 80%.

```yaml
# Add to deploy/monitoring/prometheus-rules.yaml
- alert: LowCacheHitRate
  expr: rate(ml_cache_hits_total[5m]) / (rate(ml_cache_hits_total[5m]) + rate(ml_cache_misses_total[5m])) < 0.8
  for: 10m
  labels:
    severity: warning
    team: sre
  annotations:
    summary: "Cache hit rate is low"
    description: "Cache hit rate is {{ $value | humanizePercentage }}, below 80%"
    runbook: "https://wiki.company.com/runbooks/low-cache-hit-rate"
```

```bash
# Apply the new rule
kubectl apply -f deploy/monitoring/prometheus-rules.yaml

# Verify rule is loaded
kubectl exec -n monitoring prometheus-0 -- promtool check rules /etc/prometheus/rules/*.yaml
```

---

## Best Practices & Tips

### 1. Always Use Version Control

```bash
# ✅ Good: Track all changes in Git
vim deploy/k8s/overlays/prod/deployment.yaml
git add deploy/k8s/overlays/prod/deployment.yaml
git commit -m "feat: increase replicas to 25"
git push

# ❌ Bad: Making ad-hoc changes
kubectl edit deployment campaign-express  # Changes are not tracked!
```

### 2. Use Namespaces for Isolation

```bash
# ✅ Good: Separate environments
kubectl create namespace staging
kubectl create namespace production

# Deploy to specific namespace
kubectl apply -k deploy/k8s/overlays/staging -n staging
```

### 3. Set Resource Requests and Limits

```yaml
# ✅ Good: Define resources
resources:
  requests:
    memory: "512Mi"
    cpu: "500m"
  limits:
    memory: "2Gi"
    cpu: "2000m"

# ❌ Bad: No resource constraints
# Pods can starve other pods of resources
```

### 4. Implement Readiness and Liveness Probes

```yaml
# ✅ Good: Health checks
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10
readinessProbe:
  httpGet:
    path: /ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
```

### 5. Use PodDisruptionBudgets

```yaml
# ✅ Good: Ensure availability during disruptions
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: campaign-express-pdb
spec:
  minAvailable: 15  # Keep at least 15 pods running
  selector:
    matchLabels:
      app: campaign-express
```

### 6. Enable Monitoring for Everything

```yaml
# ✅ Good: Add Prometheus annotations
metadata:
  annotations:
    prometheus.io/scrape: "true"
    prometheus.io/port: "9091"
    prometheus.io/path: "/metrics"
```

### 7. Use Secrets for Sensitive Data

```bash
# ✅ Good: Use Kubernetes Secrets
kubectl create secret generic redis-password --from-literal=password=mysecret

# ❌ Bad: Hardcode in ConfigMap
data:
  REDIS_PASSWORD: "mysecret"  # Plain text!
```

### 8. Test Changes in Staging First

```bash
# ✅ Good: Test in staging, then promote
kubectl apply -k deploy/k8s/overlays/staging
# ... test thoroughly ...
kubectl apply -k deploy/k8s/overlays/prod

# ❌ Bad: Deploy directly to production
kubectl apply -k deploy/k8s/overlays/prod  # YOLO!
```

### 9. Automate Repetitive Tasks

```bash
# ✅ Good: Create scripts for common tasks
cat > check-health.sh <<'EOF'
#!/bin/bash
echo "=== Cluster Nodes ==="
kubectl get nodes
echo "=== Pods Status ==="
kubectl get pods -n campaign-express
echo "=== Recent Events ==="
kubectl get events -n campaign-express --sort-by='.lastTimestamp' | tail -10
EOF
chmod +x check-health.sh
```

### 10. Document Your Runbooks

Create runbooks for common scenarios:
- How to deploy
- How to rollback
- How to scale
- How to handle outages
- On-call procedures

---

## Learning Resources

### Kubernetes

1. **[Kubernetes Documentation](https://kubernetes.io/docs/)** - Official docs (start here!)
2. **[Kubernetes the Hard Way](https://github.com/kelseyhightower/kubernetes-the-hard-way)** - Learn by setting up from scratch
3. **[Play with Kubernetes](https://labs.play-with-k8s.com/)** - Free online K8s playground
4. **[CKAD Exam Prep](https://github.com/dgkanatsios/CKAD-exercises)** - Hands-on exercises

### Site Reliability Engineering

1. **[Google SRE Book](https://sre.google/sre-book/table-of-contents/)** - Free online book
2. **[The Site Reliability Workbook](https://sre.google/workbook/table-of-contents/)** - Practical examples
3. **[SRE Weekly Newsletter](https://sreweekly.com/)** - Weekly SRE news

### Monitoring & Observability

1. **[Prometheus Documentation](https://prometheus.io/docs/)** - Metrics collection
2. **[Grafana Tutorials](https://grafana.com/tutorials/)** - Dashboarding
3. **[Distributed Tracing Guide](https://opentelemetry.io/docs/)** - OpenTelemetry

### Infrastructure as Code

1. **[Terraform Tutorials](https://learn.hashicorp.com/terraform)** - HashiCorp learning
2. **[Kustomize Tutorial](https://kubectl.docs.kubernetes.io/guides/introduction/kustomize/)** - K8s configuration management
3. **[Helm Docs](https://helm.sh/docs/)** - Package manager

### Cloud Platforms

1. **[Azure Kubernetes Service Docs](https://docs.microsoft.com/en-us/azure/aks/)** - AKS specific
2. **[AWS EKS Workshop](https://www.eksworkshop.com/)** - If using AWS
3. **[GKE Tutorials](https://cloud.google.com/kubernetes-engine/docs/tutorials)** - If using Google Cloud

### Tools

1. **[k9s Documentation](https://k9scli.io/)** - Terminal UI for K8s
2. **[kubectl Cheat Sheet](https://kubernetes.io/docs/reference/kubectl/cheatsheet/)** - Quick reference
3. **[Stern GitHub](https://github.com/stern/stern)** - Multi-pod log tailing

### Books

1. **"Kubernetes in Action" by Marko Lukša** - Comprehensive K8s guide
2. **"Kubernetes Patterns" by Bilgin Ibryam & Roland Huß** - Design patterns
3. **"Site Reliability Engineering" by Google** - SRE principles

---

## Getting Help

### When Things Go Wrong

1. **Check Logs**
   ```bash
   kubectl logs -f pod-name -n campaign-express
   ```

2. **Check Events**
   ```bash
   kubectl describe pod pod-name -n campaign-express
   ```

3. **Check Metrics**
   ```bash
   kubectl top pod pod-name -n campaign-express
   ```

4. **Ask for Help**
   - Post in `#sre-help` Slack channel
   - Page on-call engineer if production issue
   - Schedule 1-on-1 with senior SRE

### Common Issues and Solutions

**"Pods are in Pending state"**
- Check if enough resources: `kubectl describe pod <pod>`
- Check if PVC bound: `kubectl get pvc`
- Check node capacity: `kubectl describe nodes`

**"Pods are in CrashLoopBackOff"**
- Check logs: `kubectl logs <pod> --previous`
- Check resource limits: `kubectl describe pod <pod>`
- Check configuration: `kubectl get configmap`

**"Service not accessible"**
- Check Service endpoints: `kubectl get endpoints`
- Check NetworkPolicies: `kubectl get networkpolicies`
- Test connectivity: `kubectl run -it --rm debug --image=busybox -- wget -O- http://service-name`

**"High latency / slow responses"**
- Check pod resources: `kubectl top pods`
- Check if pods are throttled: `kubectl describe pods | grep -i throttl`
- Check dependencies (Redis, NATS): `kubectl logs -l app=redis`

**"Certificate errors"**
- Check cert-manager: `kubectl get certificates`
- Check certificate expiry: `kubectl describe certificate`
- Force renewal: `kubectl delete certificate <name>`

---

## Final Thoughts

As a fresher SRE, remember:

- **Reliability is Job #1** - Users depend on services being available
- **Automate toil** - Don't do manually what you can script
- **Measure everything** - If you can't measure it, you can't improve it
- **Learn from incidents** - Every outage is a learning opportunity
- **Communicate clearly** - During incidents, over-communicate
- **Sleep is important** - Burnt-out SREs make mistakes

**Key Mindsets**:
- 🔍 Observe before acting (check metrics/logs first)
- 🤖 Automate repetitive tasks
- 📊 Data-driven decision making
- 🛡️ Defense in depth (multiple layers of protection)
- 📚 Document everything (your future self will thank you)

**Remember**: Being on-call is a responsibility, not a burden. You're the guardian of the platform!

Welcome to the SRE team! 🚀

---

*For questions specific to CampaignExpress infrastructure, reach out to your team lead or post in the #sre-help Slack channel.*
