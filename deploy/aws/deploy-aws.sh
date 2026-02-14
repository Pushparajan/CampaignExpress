#!/usr/bin/env bash
# =============================================================================
# Campaign Express — Full AWS Deployment Script (End-to-End)
# =============================================================================
# Deploys the COMPLETE application stack on AWS via CLI:
#
#   Stage 1 — infra:      VPC, EKS, ECR, ElastiCache Redis, Secrets Manager
#   Stage 2 — build:      Docker images (backend + frontend) → ECR
#   Stage 3 — operators:  cert-manager, External Secrets Operator, AWS LB Controller
#   Stage 4 — services:   NATS JetStream, ClickHouse, Redis, HAProxy, Neuron plugin
#   Stage 5 — security:   Network Policies, External Secrets (AWS SM), TLS certs
#   Stage 6 — app:        Campaign Express backend (Helm) + Next.js frontend
#   Stage 7 — monitor:    Prometheus, AlertManager, Grafana, Tempo, Loki
#
# Tech stack deployed:
#   Rust 1.77 (Tokio/Axum/Tonic/Prost) · Next.js 14 (React 18/TanStack/Tailwind)
#   NATS JetStream · Redis 7 (ElastiCache) · ClickHouse 24 · ndarray ML inference
#   Prometheus · Grafana 10 · AlertManager · Tempo · Loki
#   cert-manager · External Secrets · K8s NetworkPolicies · HAProxy
#   AWS Neuron (Inferentia 2/3) device plugin
#
# Prerequisites:
#   - AWS CLI v2 configured (aws configure / AWS_PROFILE)
#   - Terraform >= 1.5
#   - Docker
#   - kubectl
#   - helm >= 3.0
#   - jq
#
# Usage:
#   ./deploy-aws.sh                       # Full deployment (all stages)
#   ./deploy-aws.sh --stage infra         # Only provision AWS infrastructure
#   ./deploy-aws.sh --stage build         # Only build & push Docker images
#   ./deploy-aws.sh --stage operators     # Only install K8s operators
#   ./deploy-aws.sh --stage services      # Only deploy in-cluster services
#   ./deploy-aws.sh --stage security      # Only deploy network policies + secrets
#   ./deploy-aws.sh --stage app           # Only deploy the application
#   ./deploy-aws.sh --stage monitor       # Only deploy monitoring stack
#   ./deploy-aws.sh --destroy             # Tear down everything
#
#   Environment overrides:
#     AWS_REGION=us-west-2 ENVIRONMENT=staging ./deploy-aws.sh
#     ENABLE_INFERENTIA=true ./deploy-aws.sh     # Deploy Neuron device plugin
# =============================================================================
set -euo pipefail

# ── Configuration ───────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TF_DIR="$SCRIPT_DIR/terraform"
DEPLOY_DIR="$PROJECT_ROOT/deploy"

AWS_REGION="${AWS_REGION:-us-east-1}"
ENVIRONMENT="${ENVIRONMENT:-prod}"
PROJECT_NAME="${PROJECT_NAME:-campaign-express}"
CLUSTER_NAME="${PROJECT_NAME}-${ENVIRONMENT}-eks"
NAMESPACE="campaign-express"
IMAGE_TAG="${IMAGE_TAG:-$(git -C "$PROJECT_ROOT" rev-parse --short HEAD 2>/dev/null || echo 'latest')}"
ENABLE_INFERENTIA="${ENABLE_INFERENTIA:-false}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# ── Helpers ─────────────────────────────────────────────────────────────────

log()   { echo -e "${BLUE}[$(date +%H:%M:%S)]${NC} $*"; }
ok()    { echo -e "${GREEN}[OK]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
err()   { echo -e "${RED}[ERROR]${NC} $*" >&2; }
banner() {
  echo ""
  echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
  echo -e "${CYAN}  $*${NC}"
  echo -e "${CYAN}═══════════════════════════════════════════════════════════════${NC}"
  echo ""
}

retry() {
  local max_attempts=$1; shift
  local delay=$1; shift
  local attempt=1
  while true; do
    if "$@"; then return 0; fi
    if (( attempt >= max_attempts )); then
      err "Command failed after $max_attempts attempts: $*"
      return 1
    fi
    warn "Attempt $attempt/$max_attempts failed. Retrying in ${delay}s..."
    sleep "$delay"
    delay=$(( delay * 2 ))
    attempt=$(( attempt + 1 ))
  done
}

wait_for_pods() {
  local label=$1
  local ns=${2:-$NAMESPACE}
  local timeout=${3:-300}
  log "Waiting for pods with label '$label' in namespace '$ns'..."
  kubectl wait --for=condition=ready pod -l "$label" -n "$ns" --timeout="${timeout}s" 2>/dev/null || {
    warn "Pods not ready within ${timeout}s, checking status..."
    kubectl get pods -l "$label" -n "$ns" -o wide
    return 1
  }
  ok "Pods with label '$label' are ready"
}

# ── Prerequisite Checks ────────────────────────────────────────────────────

check_prerequisites() {
  banner "Checking Prerequisites"

  local missing=()

  for cmd in aws terraform docker kubectl helm jq; do
    if command -v "$cmd" &>/dev/null; then
      ok "$cmd found: $(command -v "$cmd")"
    else
      missing+=("$cmd")
    fi
  done

  if (( ${#missing[@]} > 0 )); then
    err "Missing required tools: ${missing[*]}"
    echo ""
    echo "Install them:"
    echo "  aws       — https://docs.aws.amazon.com/cli/latest/userguide/install-cliv2.html"
    echo "  terraform — https://developer.hashicorp.com/terraform/downloads"
    echo "  docker    — https://docs.docker.com/get-docker/"
    echo "  kubectl   — https://kubernetes.io/docs/tasks/tools/"
    echo "  helm      — https://helm.sh/docs/intro/install/"
    echo "  jq        — https://jqlang.github.io/jq/download/"
    exit 1
  fi

  # Verify AWS credentials
  if ! aws sts get-caller-identity &>/dev/null; then
    err "AWS credentials not configured. Run 'aws configure' or set AWS_PROFILE."
    exit 1
  fi

  local account_id
  account_id=$(aws sts get-caller-identity --query Account --output text)
  ok "AWS Account: $account_id (Region: $AWS_REGION)"
}

# =============================================================================
# Stage 1: Infrastructure (Terraform)
# =============================================================================

deploy_infrastructure() {
  banner "Stage 1: Provisioning AWS Infrastructure"
  log "VPC + EKS + ECR + ElastiCache Redis + Secrets Manager + IAM"

  ensure_tf_backend

  cd "$TF_DIR"

  log "Initializing Terraform..."
  terraform init -input=false

  log "Planning infrastructure changes..."
  terraform plan \
    -var "aws_region=$AWS_REGION" \
    -var "environment=$ENVIRONMENT" \
    -var "project_name=$PROJECT_NAME" \
    -out=tfplan

  log "Applying infrastructure..."
  terraform apply -input=false tfplan
  rm -f tfplan

  # Export outputs for later stages
  export ECR_REPO_URL
  ECR_REPO_URL=$(terraform output -raw ecr_repository_url)
  export REDIS_ENDPOINT
  REDIS_ENDPOINT=$(terraform output -raw redis_endpoint)
  export REDIS_PORT
  REDIS_PORT=$(terraform output -raw redis_port)
  export ESO_ROLE_ARN
  ESO_ROLE_ARN=$(terraform output -raw external_secrets_role_arn)

  cd "$PROJECT_ROOT"

  # Configure kubectl for EKS
  log "Configuring kubectl for EKS cluster '$CLUSTER_NAME'..."
  aws eks update-kubeconfig \
    --region "$AWS_REGION" \
    --name "$CLUSTER_NAME" \
    --alias "$CLUSTER_NAME"

  ok "Infrastructure provisioned"
}

ensure_tf_backend() {
  local bucket="campaign-express-tfstate"
  local table="campaign-express-tflock"

  if ! aws s3api head-bucket --bucket "$bucket" 2>/dev/null; then
    log "Creating Terraform state bucket: $bucket"
    if [ "$AWS_REGION" = "us-east-1" ]; then
      aws s3api create-bucket --bucket "$bucket" --region "$AWS_REGION"
    else
      aws s3api create-bucket --bucket "$bucket" --region "$AWS_REGION" \
        --create-bucket-configuration "LocationConstraint=$AWS_REGION"
    fi

    aws s3api put-bucket-versioning \
      --bucket "$bucket" \
      --versioning-configuration Status=Enabled

    aws s3api put-bucket-encryption \
      --bucket "$bucket" \
      --server-side-encryption-configuration '{
        "Rules": [{"ApplyServerSideEncryptionByDefault": {"SSEAlgorithm": "AES256"}}]
      }'

    aws s3api put-public-access-block \
      --bucket "$bucket" \
      --public-access-block-configuration \
        BlockPublicAcls=true,IgnorePublicAcls=true,BlockPublicPolicy=true,RestrictPublicBuckets=true

    ok "S3 bucket created: $bucket"
  fi

  if ! aws dynamodb describe-table --table-name "$table" --region "$AWS_REGION" &>/dev/null; then
    log "Creating Terraform lock table: $table"
    aws dynamodb create-table \
      --table-name "$table" \
      --attribute-definitions AttributeName=LockID,AttributeType=S \
      --key-schema AttributeName=LockID,KeyType=HASH \
      --billing-mode PAY_PER_REQUEST \
      --region "$AWS_REGION"

    aws dynamodb wait table-exists --table-name "$table" --region "$AWS_REGION"
    ok "DynamoDB lock table created: $table"
  fi
}

# =============================================================================
# Stage 2: Build & Push Docker Images (Backend + Frontend)
# =============================================================================

build_and_push() {
  banner "Stage 2: Building & Pushing Docker Images"
  log "Backend (Rust 1.77 multi-stage) + Frontend (Next.js 14)"

  if [ -z "${ECR_REPO_URL:-}" ]; then
    ECR_REPO_URL=$(get_tf_output "ecr_repository_url")
  fi

  local account_id
  account_id=$(aws sts get-caller-identity --query Account --output text)
  local ecr_registry="${account_id}.dkr.ecr.${AWS_REGION}.amazonaws.com"

  # Authenticate Docker with ECR
  log "Authenticating Docker with ECR..."
  aws ecr get-login-password --region "$AWS_REGION" | \
    docker login --username AWS --password-stdin "$ecr_registry"

  # ── Backend image ──────────────────────────────────────────────────────
  log "Building backend image: $ECR_REPO_URL:$IMAGE_TAG"
  docker build \
    -t "$ECR_REPO_URL:$IMAGE_TAG" \
    -t "$ECR_REPO_URL:latest" \
    -f "$DEPLOY_DIR/docker/Dockerfile" \
    "$PROJECT_ROOT"

  log "Pushing backend image..."
  retry 4 2 docker push "$ECR_REPO_URL:$IMAGE_TAG"
  retry 4 2 docker push "$ECR_REPO_URL:latest"
  ok "Backend image pushed: $ECR_REPO_URL:$IMAGE_TAG"

  # ── Frontend image ────────────────────────────────────────────────────
  # Create UI ECR repo if it doesn't exist
  local ui_repo="${PROJECT_NAME}-ui"
  local ui_repo_url="${ecr_registry}/${ui_repo}"
  if ! aws ecr describe-repositories --repository-names "$ui_repo" --region "$AWS_REGION" &>/dev/null; then
    log "Creating ECR repository for frontend: $ui_repo"
    aws ecr create-repository --repository-name "$ui_repo" --region "$AWS_REGION" \
      --image-scanning-configuration scanOnPush=true >/dev/null
  fi

  log "Building frontend image: $ui_repo_url:$IMAGE_TAG"
  docker build \
    -t "$ui_repo_url:$IMAGE_TAG" \
    -t "$ui_repo_url:latest" \
    --build-arg "NEXT_PUBLIC_API_URL=http://campaign-express.${NAMESPACE}.svc.cluster.local:8080" \
    -f "$SCRIPT_DIR/ui.Dockerfile" \
    "$PROJECT_ROOT"

  log "Pushing frontend image..."
  retry 4 2 docker push "$ui_repo_url:$IMAGE_TAG"
  retry 4 2 docker push "$ui_repo_url:latest"
  ok "Frontend image pushed: $ui_repo_url:$IMAGE_TAG"

  export UI_REPO_URL="$ui_repo_url"
}

# =============================================================================
# Stage 3: Install Kubernetes Operators
# =============================================================================

deploy_operators() {
  banner "Stage 3: Installing Kubernetes Operators"
  log "cert-manager + External Secrets Operator + AWS Load Balancer Controller"

  ensure_kubeconfig

  install_cert_manager
  install_external_secrets_operator
  install_aws_lb_controller

  ok "All operators installed"
}

install_cert_manager() {
  log "Installing cert-manager (Let's Encrypt TLS)..."

  helm repo add jetstack https://charts.jetstack.io 2>/dev/null || true
  helm repo update jetstack

  helm upgrade --install cert-manager jetstack/cert-manager \
    --namespace cert-manager \
    --create-namespace \
    --set crds.enabled=true \
    --set global.leaderElection.namespace=cert-manager \
    --wait \
    --timeout 5m

  wait_for_pods "app.kubernetes.io/instance=cert-manager" "cert-manager" 120
  ok "cert-manager installed"
}

install_external_secrets_operator() {
  log "Installing External Secrets Operator (AWS Secrets Manager integration)..."

  helm repo add external-secrets https://charts.external-secrets.io 2>/dev/null || true
  helm repo update external-secrets

  # Resolve IRSA role ARN
  local eso_role_arn="${ESO_ROLE_ARN:-$(get_tf_output external_secrets_role_arn 2>/dev/null || echo '')}"

  local set_flags=()
  if [ -n "$eso_role_arn" ] && [ "$eso_role_arn" != "null" ]; then
    set_flags+=(--set "serviceAccount.annotations.eks\\.amazonaws\\.com/role-arn=$eso_role_arn")
  fi

  helm upgrade --install external-secrets external-secrets/external-secrets \
    --namespace external-secrets \
    --create-namespace \
    "${set_flags[@]}" \
    --wait \
    --timeout 5m

  wait_for_pods "app.kubernetes.io/instance=external-secrets" "external-secrets" 120
  ok "External Secrets Operator installed"
}

install_aws_lb_controller() {
  log "Installing AWS Load Balancer Controller (ALB Ingress)..."

  helm repo add eks https://aws.github.io/eks-charts 2>/dev/null || true
  helm repo update eks

  local cluster_name="$CLUSTER_NAME"

  helm upgrade --install aws-load-balancer-controller eks/aws-load-balancer-controller \
    --namespace kube-system \
    --set "clusterName=$cluster_name" \
    --set "region=$AWS_REGION" \
    --set "vpcId=$(get_tf_output vpc_id 2>/dev/null || echo '')" \
    --set serviceAccount.create=true \
    --set serviceAccount.name=aws-load-balancer-controller \
    --wait \
    --timeout 5m

  ok "AWS Load Balancer Controller installed"
}

# =============================================================================
# Stage 4: Deploy In-Cluster Services
# =============================================================================

deploy_services() {
  banner "Stage 4: Deploying In-Cluster Services"
  log "NATS JetStream · ClickHouse 24 · Redis 7 · HAProxy · Neuron plugin"

  ensure_kubeconfig

  # Create namespace
  log "Creating namespace '$NAMESPACE'..."
  kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -

  # Create gp3 StorageClass for EBS CSI
  deploy_storage_class

  # Deploy services in dependency order
  deploy_nats
  deploy_clickhouse
  deploy_redis_in_cluster
  deploy_haproxy
  deploy_neuron_plugin

  ok "All in-cluster services deployed"
}

deploy_storage_class() {
  log "Creating gp3 StorageClass (EBS CSI)..."
  kubectl apply -f - <<'EOF'
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: gp3
  annotations:
    storageclass.kubernetes.io/is-default-class: "true"
provisioner: ebs.csi.aws.com
parameters:
  type: gp3
  fsType: ext4
  iops: "3000"
  throughput: "125"
reclaimPolicy: Delete
volumeBindingMode: WaitForFirstConsumer
allowVolumeExpansion: true
EOF
  ok "gp3 StorageClass created"
}

deploy_nats() {
  log "Deploying NATS JetStream cluster (3 replicas, async-nats 0.35)..."
  kubectl apply -f "$DEPLOY_DIR/nats/nats-deployment.yaml"
  wait_for_pods "app.kubernetes.io/name=nats" "$NAMESPACE" 180
}

deploy_clickhouse() {
  log "Deploying ClickHouse 24 (2 replicas, analytics DB)..."
  kubectl apply -f "$DEPLOY_DIR/clickhouse/clickhouse-deployment.yaml"
  wait_for_pods "app.kubernetes.io/name=clickhouse" "$NAMESPACE" 300
}

deploy_redis_in_cluster() {
  # If ElastiCache is provisioned, skip in-cluster Redis
  if [ -z "${REDIS_ENDPOINT:-}" ]; then
    REDIS_ENDPOINT=$(get_tf_output "redis_endpoint" 2>/dev/null || echo "")
  fi

  if [ -n "$REDIS_ENDPOINT" ] && [ "$REDIS_ENDPOINT" != "null" ]; then
    log "ElastiCache Redis detected ($REDIS_ENDPOINT) — skipping in-cluster Redis"
    return 0
  fi

  log "Deploying in-cluster Redis 7 cluster (6 nodes, LRU)..."
  kubectl apply -f "$DEPLOY_DIR/redis/redis-deployment.yaml"
  wait_for_pods "app.kubernetes.io/name=redis" "$NAMESPACE" 300
}

deploy_haproxy() {
  log "Deploying HAProxy load balancer (leastconn + rate limiting)..."
  kubectl apply -f "$DEPLOY_DIR/haproxy/haproxy-deployment.yaml"
  wait_for_pods "app.kubernetes.io/name=haproxy" "$NAMESPACE" 120
}

deploy_neuron_plugin() {
  if [ "$ENABLE_INFERENTIA" != "true" ]; then
    log "Inferentia disabled — skipping Neuron device plugin (set ENABLE_INFERENTIA=true to enable)"
    return 0
  fi

  log "Deploying AWS Neuron device plugin (Inferentia 2/3)..."
  kubectl apply -f "$SCRIPT_DIR/neuron-device-plugin.yaml"
  ok "Neuron device plugin deployed"
}

# =============================================================================
# Stage 5: Security — Network Policies, External Secrets, TLS
# =============================================================================

deploy_security() {
  banner "Stage 5: Deploying Security Layer"
  log "Network Policies · External Secrets (AWS SM) · cert-manager certs"

  ensure_kubeconfig

  deploy_network_policies
  deploy_external_secrets
  deploy_tls_certificates

  ok "Security layer deployed"
}

deploy_network_policies() {
  log "Applying Kubernetes NetworkPolicies (zero-trust baseline)..."

  if [ -f "$DEPLOY_DIR/k8s/base/network-policies.yaml" ]; then
    kubectl apply -f "$DEPLOY_DIR/k8s/base/network-policies.yaml"
    ok "8 NetworkPolicies applied (deny-all + allow rules)"
  else
    warn "network-policies.yaml not found — skipping"
  fi
}

deploy_external_secrets() {
  log "Deploying ExternalSecrets (AWS Secrets Manager)..."

  # Patch region into the manifest
  local eso_manifest="$SCRIPT_DIR/external-secrets-aws.yaml"
  local secret_prefix="${PROJECT_NAME}-${ENVIRONMENT}"

  # Apply with region substitution
  sed \
    -e "s|region: us-east-1|region: $AWS_REGION|g" \
    -e "s|campaign-express-prod/|${secret_prefix}/|g" \
    "$eso_manifest" | kubectl apply -f -

  # Create the IRSA service account for ESO in the app namespace
  local eso_role_arn="${ESO_ROLE_ARN:-$(get_tf_output external_secrets_role_arn 2>/dev/null || echo '')}"
  if [ -n "$eso_role_arn" ] && [ "$eso_role_arn" != "null" ]; then
    kubectl apply -f - <<EOF
apiVersion: v1
kind: ServiceAccount
metadata:
  name: external-secrets-sa
  namespace: $NAMESPACE
  annotations:
    eks.amazonaws.com/role-arn: "$eso_role_arn"
EOF
    ok "ExternalSecrets service account created with IRSA"
  fi

  ok "ExternalSecrets deployed (redis, clickhouse, nats, twilio, sendgrid)"
}

deploy_tls_certificates() {
  log "Applying cert-manager ClusterIssuers and Certificates..."

  if [ -f "$DEPLOY_DIR/k8s/base/cert-manager.yaml" ]; then
    kubectl apply -f "$DEPLOY_DIR/k8s/base/cert-manager.yaml"
    ok "cert-manager ClusterIssuers + Certificate applied"
  else
    warn "cert-manager.yaml not found — skipping"
  fi
}

# =============================================================================
# Stage 6: Deploy Application (Backend + Frontend)
# =============================================================================

deploy_application() {
  banner "Stage 6: Deploying Campaign Express Application"
  log "Backend (Rust/Axum/Tonic via Helm) + Frontend (Next.js 14)"

  ensure_kubeconfig

  deploy_backend
  deploy_frontend

  # Show deployment status
  echo ""
  log "Deployment status:"
  kubectl get deployments -n "$NAMESPACE"
  echo ""
  kubectl get pods -n "$NAMESPACE" --no-headers | head -10
  echo "  ... (showing first 10 pods)"
}

deploy_backend() {
  log "Installing Campaign Express backend via Helm..."

  # Resolve values from Terraform outputs
  if [ -z "${ECR_REPO_URL:-}" ]; then
    ECR_REPO_URL=$(get_tf_output "ecr_repository_url" 2>/dev/null || echo "")
  fi
  if [ -z "${REDIS_ENDPOINT:-}" ]; then
    REDIS_ENDPOINT=$(get_tf_output "redis_endpoint" 2>/dev/null || echo "")
  fi

  # Build Helm set flags
  local set_flags=(
    --set "image.tag=$IMAGE_TAG"
  )

  if [ -n "$ECR_REPO_URL" ] && [ "$ECR_REPO_URL" != "null" ]; then
    set_flags+=(--set "image.repository=$ECR_REPO_URL")
  fi

  if [ -n "$REDIS_ENDPOINT" ] && [ "$REDIS_ENDPOINT" != "null" ]; then
    set_flags+=(--set "redis.url=rediss://${REDIS_ENDPOINT}:6379")
  fi

  helm upgrade --install campaign-express \
    "$DEPLOY_DIR/helm/campaign-express" \
    --namespace "$NAMESPACE" \
    --create-namespace \
    -f "$SCRIPT_DIR/values-aws.yaml" \
    "${set_flags[@]}" \
    --wait \
    --timeout 10m

  ok "Backend deployed (20 replicas, HPA 10-40)"
}

deploy_frontend() {
  log "Deploying Next.js 14 frontend..."

  local account_id
  account_id=$(aws sts get-caller-identity --query Account --output text)
  local ecr_registry="${account_id}.dkr.ecr.${AWS_REGION}.amazonaws.com"
  local ui_image="${UI_REPO_URL:-${ecr_registry}/${PROJECT_NAME}-ui}:${IMAGE_TAG}"

  # Patch image in UI deployment and apply
  sed "s|image: campaign-express-ui:latest|image: ${ui_image}|g" \
    "$SCRIPT_DIR/ui-deployment.yaml" | kubectl apply -f -

  wait_for_pods "app.kubernetes.io/name=campaign-express-ui" "$NAMESPACE" 180
  ok "Frontend deployed (Next.js 14, React 18, TanStack Query 5, Tailwind CSS)"
}

# =============================================================================
# Stage 7: Deploy Monitoring Stack
# =============================================================================

deploy_monitoring() {
  banner "Stage 7: Deploying Monitoring & Observability Stack"
  log "Prometheus · AlertManager · Grafana 10 · Tempo (tracing) · Loki (logging)"

  ensure_kubeconfig

  # Prometheus
  log "Deploying Prometheus..."
  kubectl apply -f "$DEPLOY_DIR/monitoring/prometheus/prometheus-deployment.yaml"

  # AlertManager (HA — 2 replicas)
  log "Deploying AlertManager (11 alert rules)..."
  if [ -f "$DEPLOY_DIR/monitoring/alertmanager/alertmanager-deployment.yaml" ]; then
    kubectl apply -f "$DEPLOY_DIR/monitoring/alertmanager/alertmanager-config.yaml"
    kubectl apply -f "$DEPLOY_DIR/monitoring/alertmanager/alert-rules.yaml"
    kubectl apply -f "$DEPLOY_DIR/monitoring/alertmanager/alertmanager-deployment.yaml"
  fi

  # Grafana
  log "Deploying Grafana 10 (dashboards + datasources)..."
  kubectl apply -f "$DEPLOY_DIR/monitoring/grafana/grafana-deployment.yaml"

  # Tempo (distributed tracing)
  if [ -f "$DEPLOY_DIR/monitoring/tracing/tempo-deployment.yaml" ]; then
    log "Deploying Tempo (tracing)..."
    kubectl apply -f "$DEPLOY_DIR/monitoring/tracing/tempo-deployment.yaml"
  fi

  # Loki (log aggregation)
  if [ -f "$DEPLOY_DIR/monitoring/logging/loki-stack.yaml" ]; then
    log "Deploying Loki (logging)..."
    kubectl apply -f "$DEPLOY_DIR/monitoring/logging/loki-stack.yaml"
  fi

  ok "Monitoring stack deployed"
}

# =============================================================================
# Destroy
# =============================================================================

destroy() {
  banner "DESTROYING All Resources"

  warn "This will destroy ALL AWS infrastructure for '$PROJECT_NAME-$ENVIRONMENT'."
  echo ""
  read -rp "Type 'yes' to confirm destruction: " confirm
  if [ "$confirm" != "yes" ]; then
    log "Destruction cancelled."
    exit 0
  fi

  # Delete Helm releases
  log "Removing Helm releases..."
  helm uninstall campaign-express --namespace "$NAMESPACE" 2>/dev/null || true
  helm uninstall aws-load-balancer-controller --namespace kube-system 2>/dev/null || true
  helm uninstall external-secrets --namespace external-secrets 2>/dev/null || true
  helm uninstall cert-manager --namespace cert-manager 2>/dev/null || true

  # Delete K8s resources
  log "Deleting Kubernetes namespaces..."
  kubectl delete namespace "$NAMESPACE" --ignore-not-found=true --timeout=120s 2>/dev/null || true
  kubectl delete namespace external-secrets --ignore-not-found=true --timeout=60s 2>/dev/null || true
  kubectl delete namespace cert-manager --ignore-not-found=true --timeout=60s 2>/dev/null || true

  # Delete UI ECR repo
  log "Deleting frontend ECR repository..."
  aws ecr delete-repository --repository-name "${PROJECT_NAME}-ui" --region "$AWS_REGION" --force 2>/dev/null || true

  # Destroy Terraform infrastructure
  log "Destroying Terraform infrastructure..."
  cd "$TF_DIR"
  terraform init -input=false
  terraform destroy \
    -var "aws_region=$AWS_REGION" \
    -var "environment=$ENVIRONMENT" \
    -var "project_name=$PROJECT_NAME" \
    -auto-approve
  cd "$PROJECT_ROOT"

  ok "All resources destroyed"
}

# =============================================================================
# Health Check
# =============================================================================

health_check() {
  banner "Health Check — Full Stack"

  log "Cluster info:"
  kubectl cluster-info 2>/dev/null || { err "Cannot reach cluster"; return 1; }

  echo ""
  log "Namespace '$NAMESPACE' — all resources:"
  kubectl get all -n "$NAMESPACE" 2>/dev/null || true

  echo ""
  log "Pod status summary:"
  kubectl get pods -n "$NAMESPACE" -o json 2>/dev/null | \
    jq -r '.items | group_by(.status.phase) | .[] | "  \(.[0].status.phase): \(length)"' 2>/dev/null || true

  echo ""
  log "Service endpoints:"
  kubectl get svc -n "$NAMESPACE" -o wide 2>/dev/null || true

  echo ""
  log "HPA status:"
  kubectl get hpa -n "$NAMESPACE" 2>/dev/null || true

  echo ""
  log "Ingress status:"
  kubectl get ingress -n "$NAMESPACE" 2>/dev/null || true

  echo ""
  log "Operators:"
  echo -n "  cert-manager:       "; kubectl get pods -n cert-manager --no-headers 2>/dev/null | wc -l | xargs -I{} echo "{} pods"
  echo -n "  external-secrets:   "; kubectl get pods -n external-secrets --no-headers 2>/dev/null | wc -l | xargs -I{} echo "{} pods"
  echo -n "  lb-controller:      "; kubectl get pods -n kube-system -l app.kubernetes.io/name=aws-load-balancer-controller --no-headers 2>/dev/null | wc -l | xargs -I{} echo "{} pods"

  echo ""
  log "ExternalSecrets sync status:"
  kubectl get externalsecrets -n "$NAMESPACE" 2>/dev/null || true

  echo ""
  log "NetworkPolicies:"
  kubectl get networkpolicies -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l | xargs -I{} echo "  {} policies applied"

  # Test health endpoint
  local pod
  pod=$(kubectl get pod -n "$NAMESPACE" -l app.kubernetes.io/name=campaign-express \
    -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
  if [ -n "$pod" ]; then
    log "Testing backend health endpoint on pod $pod..."
    kubectl exec -n "$NAMESPACE" "$pod" -- curl -sf http://localhost:8080/health 2>/dev/null && \
      ok "Backend health check passed" || warn "Backend health check failed or curl not available"
  fi

  local ui_pod
  ui_pod=$(kubectl get pod -n "$NAMESPACE" -l app.kubernetes.io/name=campaign-express-ui \
    -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
  if [ -n "$ui_pod" ]; then
    log "Testing frontend on pod $ui_pod..."
    kubectl exec -n "$NAMESPACE" "$ui_pod" -- wget -q --spider http://localhost:3000/ 2>/dev/null && \
      ok "Frontend health check passed" || warn "Frontend health check failed"
  fi
}

# =============================================================================
# Utilities
# =============================================================================

ensure_kubeconfig() {
  if ! kubectl cluster-info &>/dev/null; then
    log "Configuring kubectl for EKS cluster '$CLUSTER_NAME'..."
    aws eks update-kubeconfig \
      --region "$AWS_REGION" \
      --name "$CLUSTER_NAME" \
      --alias "$CLUSTER_NAME"
  fi
}

get_tf_output() {
  local key=$1
  cd "$TF_DIR"
  local val
  val=$(terraform output -raw "$key" 2>/dev/null || echo "")
  cd "$PROJECT_ROOT"
  echo "$val"
}

print_summary() {
  banner "Deployment Summary"

  local ecr_url="${ECR_REPO_URL:-$(get_tf_output ecr_repository_url 2>/dev/null || echo 'N/A')}"
  local redis="${REDIS_ENDPOINT:-$(get_tf_output redis_endpoint 2>/dev/null || echo 'N/A')}"

  cat <<SUMMARY
  Project:        $PROJECT_NAME
  Environment:    $ENVIRONMENT
  Region:         $AWS_REGION
  EKS Cluster:    $CLUSTER_NAME
  Backend Image:  $ecr_url:$IMAGE_TAG
  Frontend Image: ${UI_REPO_URL:-N/A}:$IMAGE_TAG
  Redis:          $redis
  Namespace:      $NAMESPACE
  Inferentia:     $ENABLE_INFERENTIA

  Stack:
    Backend     Rust 1.77 (Tokio 1.36 / Axum 0.7 / Tonic 0.12 / Prost 0.13)
    Frontend    Next.js 14 / React 18 / TanStack Query 5 / Tailwind CSS
    Messaging   NATS JetStream (async-nats 0.35)
    Cache       Redis 7 (ElastiCache cluster mode) + DashMap L1
    Analytics   ClickHouse 24
    Inference   ndarray 0.15 (CPU) + Inferentia 2/3 backends
    Monitoring  Prometheus + AlertManager + Grafana 10 + Tempo + Loki
    Security    cert-manager + External Secrets (AWS SM) + NetworkPolicies
    Networking  HAProxy + AWS ALB Ingress Controller

  Useful commands:
    kubectl get pods -n $NAMESPACE
    kubectl logs -f deploy/campaign-express -n $NAMESPACE
    kubectl port-forward svc/campaign-express 8080:8080 -n $NAMESPACE
    kubectl port-forward svc/campaign-express-ui 3000:3000 -n $NAMESPACE
    kubectl port-forward svc/grafana 3000:3000 -n $NAMESPACE

SUMMARY
}

usage() {
  cat <<EOF
Campaign Express — AWS Deployment Script (End-to-End)

Usage: $(basename "$0") [OPTIONS]

Options:
  --stage <stage>    Run a specific deployment stage:
                       infra      — Provision AWS infra (VPC, EKS, ECR, ElastiCache)
                       build      — Build & push Docker images (backend + frontend)
                       operators  — Install K8s operators (cert-manager, ESO, ALB)
                       services   — Deploy in-cluster services (NATS, ClickHouse, etc.)
                       security   — Deploy NetworkPolicies, ExternalSecrets, TLS
                       app        — Deploy application (backend Helm + frontend)
                       monitor    — Deploy monitoring stack
  --destroy          Tear down all resources
  --health           Run full-stack health checks
  --help             Show this help message

Environment Variables:
  AWS_REGION           AWS region (default: us-east-1)
  ENVIRONMENT          Deployment environment: dev|staging|prod (default: prod)
  PROJECT_NAME         Project name prefix (default: campaign-express)
  IMAGE_TAG            Docker image tag (default: git short SHA)
  ENABLE_INFERENTIA    Deploy AWS Neuron device plugin (default: false)

Examples:
  # Full end-to-end deployment (all 7 stages)
  ./deploy-aws.sh

  # Deploy only infrastructure
  ./deploy-aws.sh --stage infra

  # Deploy to staging in us-west-2 with Inferentia
  AWS_REGION=us-west-2 ENVIRONMENT=staging ENABLE_INFERENTIA=true ./deploy-aws.sh

  # Build and push images only
  ./deploy-aws.sh --stage build

  # Install operators + security layer
  ./deploy-aws.sh --stage operators && ./deploy-aws.sh --stage security

  # Full-stack health check
  ./deploy-aws.sh --health

  # Tear everything down
  ./deploy-aws.sh --destroy
EOF
}

# =============================================================================
# Main
# =============================================================================

main() {
  local stage=""
  local do_destroy=false
  local do_health=false

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --stage)    stage="$2"; shift 2 ;;
      --destroy)  do_destroy=true; shift ;;
      --health)   do_health=true; shift ;;
      --help|-h)  usage; exit 0 ;;
      *)          err "Unknown option: $1"; usage; exit 1 ;;
    esac
  done

  banner "Campaign Express — AWS Deployment"
  echo "  Region:       $AWS_REGION"
  echo "  Environment:  $ENVIRONMENT"
  echo "  Image Tag:    $IMAGE_TAG"
  echo "  Inferentia:   $ENABLE_INFERENTIA"
  echo ""

  check_prerequisites

  if $do_destroy; then
    destroy
    exit 0
  fi

  if $do_health; then
    health_check
    exit 0
  fi

  case "$stage" in
    infra)     deploy_infrastructure ;;
    build)     build_and_push ;;
    operators) deploy_operators ;;
    services)  deploy_services ;;
    security)  deploy_security ;;
    app)       deploy_application ;;
    monitor)   deploy_monitoring ;;
    "")
      # Full end-to-end deployment — all 7 stages
      deploy_infrastructure
      build_and_push
      deploy_operators
      deploy_services
      deploy_security
      deploy_application
      deploy_monitoring
      health_check
      ;;
    *)
      err "Unknown stage: $stage"
      usage
      exit 1
      ;;
  esac

  print_summary
  ok "Deployment complete!"
}

main "$@"
