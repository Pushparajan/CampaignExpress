#!/usr/bin/env bash
# =============================================================================
# Campaign Express — Full AWS Deployment Script
# =============================================================================
# Deploys the complete application stack on AWS via CLI:
#   1. Infrastructure (Terraform): VPC, EKS, ECR, ElastiCache Redis, Secrets
#   2. Container image: Build & push to ECR
#   3. In-cluster services: NATS, ClickHouse, monitoring stack
#   4. Application: Campaign Express via Helm
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
#   ./deploy-aws.sh                    # Full deployment (all stages)
#   ./deploy-aws.sh --stage infra      # Only provision AWS infrastructure
#   ./deploy-aws.sh --stage build      # Only build & push Docker image
#   ./deploy-aws.sh --stage services   # Only deploy in-cluster services
#   ./deploy-aws.sh --stage app        # Only deploy the application
#   ./deploy-aws.sh --stage monitor    # Only deploy monitoring stack
#   ./deploy-aws.sh --destroy          # Tear down everything
#
#   Environment overrides:
#     AWS_REGION=us-west-2 ENVIRONMENT=staging ./deploy-aws.sh
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

# ── Stage 1: Infrastructure (Terraform) ────────────────────────────────────

deploy_infrastructure() {
  banner "Stage 1: Provisioning AWS Infrastructure"

  # Create S3 backend bucket + DynamoDB lock table if they don't exist
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

  cd "$PROJECT_ROOT"

  # Configure kubectl for EKS
  log "Configuring kubectl for EKS cluster '$CLUSTER_NAME'..."
  aws eks update-kubeconfig \
    --region "$AWS_REGION" \
    --name "$CLUSTER_NAME" \
    --alias "$CLUSTER_NAME"

  ok "Infrastructure provisioned successfully"
}

ensure_tf_backend() {
  local bucket="campaign-express-tfstate"
  local table="campaign-express-tflock"

  if ! aws s3api head-bucket --bucket "$bucket" 2>/dev/null; then
    log "Creating Terraform state bucket: $bucket"
    aws s3api create-bucket \
      --bucket "$bucket" \
      --region "$AWS_REGION" \
      $([ "$AWS_REGION" != "us-east-1" ] && echo "--create-bucket-configuration LocationConstraint=$AWS_REGION")

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

# ── Stage 2: Build & Push Docker Image ─────────────────────────────────────

build_and_push() {
  banner "Stage 2: Building & Pushing Docker Image"

  # Resolve ECR URL if not already set
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

  # Build
  log "Building Docker image: $ECR_REPO_URL:$IMAGE_TAG"
  docker build \
    -t "$ECR_REPO_URL:$IMAGE_TAG" \
    -t "$ECR_REPO_URL:latest" \
    -f "$DEPLOY_DIR/docker/Dockerfile" \
    "$PROJECT_ROOT"

  # Push with retries
  log "Pushing image to ECR..."
  retry 4 2 docker push "$ECR_REPO_URL:$IMAGE_TAG"
  retry 4 2 docker push "$ECR_REPO_URL:latest"

  ok "Image pushed: $ECR_REPO_URL:$IMAGE_TAG"
}

# ── Stage 3: Deploy In-Cluster Services ────────────────────────────────────

deploy_services() {
  banner "Stage 3: Deploying In-Cluster Services"

  ensure_kubeconfig

  # Create namespace
  log "Creating namespace '$NAMESPACE'..."
  kubectl create namespace "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -

  # Create gp3 StorageClass
  deploy_storage_class

  # Deploy services in dependency order
  deploy_nats
  deploy_clickhouse
  deploy_redis_in_cluster
  deploy_haproxy

  ok "All in-cluster services deployed"
}

deploy_storage_class() {
  log "Creating gp3 StorageClass..."
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
  log "Deploying NATS cluster..."
  kubectl apply -f "$DEPLOY_DIR/nats/nats-deployment.yaml"
  wait_for_pods "app.kubernetes.io/name=nats" "$NAMESPACE" 180
}

deploy_clickhouse() {
  log "Deploying ClickHouse..."
  kubectl apply -f "$DEPLOY_DIR/clickhouse/clickhouse-deployment.yaml"
  wait_for_pods "app.kubernetes.io/name=clickhouse" "$NAMESPACE" 300
}

deploy_redis_in_cluster() {
  # If ElastiCache is provisioned, skip in-cluster Redis.
  # We still deploy it as a fallback / for environments without ElastiCache.
  if [ -n "${REDIS_ENDPOINT:-}" ] && [ "$REDIS_ENDPOINT" != "null" ]; then
    log "ElastiCache Redis detected ($REDIS_ENDPOINT) — skipping in-cluster Redis"
    return 0
  fi

  log "Deploying in-cluster Redis (no ElastiCache endpoint found)..."
  kubectl apply -f "$DEPLOY_DIR/redis/redis-deployment.yaml"
  wait_for_pods "app.kubernetes.io/name=redis" "$NAMESPACE" 300
}

deploy_haproxy() {
  log "Deploying HAProxy..."
  kubectl apply -f "$DEPLOY_DIR/haproxy/haproxy-deployment.yaml"
  wait_for_pods "app.kubernetes.io/name=haproxy" "$NAMESPACE" 120
}

# ── Stage 4: Deploy Application (Helm) ─────────────────────────────────────

deploy_application() {
  banner "Stage 4: Deploying Campaign Express Application"

  ensure_kubeconfig

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

  log "Installing/upgrading Campaign Express via Helm..."
  helm upgrade --install campaign-express \
    "$DEPLOY_DIR/helm/campaign-express" \
    --namespace "$NAMESPACE" \
    --create-namespace \
    -f "$SCRIPT_DIR/values-aws.yaml" \
    "${set_flags[@]}" \
    --wait \
    --timeout 10m

  ok "Campaign Express deployed"

  # Show deployment status
  echo ""
  log "Deployment status:"
  kubectl get deployments -n "$NAMESPACE"
  echo ""
  kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/name=campaign-express --no-headers | head -5
  echo "  ... (showing first 5 pods)"
}

# ── Stage 5: Deploy Monitoring Stack ───────────────────────────────────────

deploy_monitoring() {
  banner "Stage 5: Deploying Monitoring Stack"

  ensure_kubeconfig

  # Prometheus
  log "Deploying Prometheus..."
  kubectl apply -f "$DEPLOY_DIR/monitoring/prometheus/prometheus-deployment.yaml"

  # AlertManager
  log "Deploying AlertManager..."
  if [ -f "$DEPLOY_DIR/monitoring/alertmanager/alertmanager-deployment.yaml" ]; then
    kubectl apply -f "$DEPLOY_DIR/monitoring/alertmanager/alertmanager-config.yaml"
    kubectl apply -f "$DEPLOY_DIR/monitoring/alertmanager/alert-rules.yaml"
    kubectl apply -f "$DEPLOY_DIR/monitoring/alertmanager/alertmanager-deployment.yaml"
  fi

  # Grafana
  log "Deploying Grafana..."
  kubectl apply -f "$DEPLOY_DIR/monitoring/grafana/grafana-deployment.yaml"

  # Tempo (tracing)
  if [ -f "$DEPLOY_DIR/monitoring/tracing/tempo-deployment.yaml" ]; then
    log "Deploying Tempo..."
    kubectl apply -f "$DEPLOY_DIR/monitoring/tracing/tempo-deployment.yaml"
  fi

  # Loki (logging)
  if [ -f "$DEPLOY_DIR/monitoring/logging/loki-stack.yaml" ]; then
    log "Deploying Loki..."
    kubectl apply -f "$DEPLOY_DIR/monitoring/logging/loki-stack.yaml"
  fi

  ok "Monitoring stack deployed"
}

# ── Destroy ─────────────────────────────────────────────────────────────────

destroy() {
  banner "DESTROYING All Resources"

  warn "This will destroy ALL AWS infrastructure for '$PROJECT_NAME-$ENVIRONMENT'."
  echo ""
  read -rp "Type 'yes' to confirm destruction: " confirm
  if [ "$confirm" != "yes" ]; then
    log "Destruction cancelled."
    exit 0
  fi

  # Delete Helm release
  log "Removing Helm release..."
  helm uninstall campaign-express --namespace "$NAMESPACE" 2>/dev/null || true

  # Delete K8s resources
  log "Deleting Kubernetes resources..."
  kubectl delete namespace "$NAMESPACE" --ignore-not-found=true --timeout=120s 2>/dev/null || true

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

# ── Health Check ────────────────────────────────────────────────────────────

health_check() {
  banner "Health Check"

  log "Cluster info:"
  kubectl cluster-info 2>/dev/null || { err "Cannot reach cluster"; return 1; }

  echo ""
  log "Namespace '$NAMESPACE' resources:"
  kubectl get all -n "$NAMESPACE" 2>/dev/null || true

  echo ""
  log "Pod status summary:"
  kubectl get pods -n "$NAMESPACE" -o json 2>/dev/null | \
    jq -r '.items | group_by(.status.phase) | .[] | "\(.[0].status.phase): \(length)"' 2>/dev/null || true

  echo ""
  log "Service endpoints:"
  kubectl get svc -n "$NAMESPACE" -o wide 2>/dev/null || true

  echo ""
  log "HPA status:"
  kubectl get hpa -n "$NAMESPACE" 2>/dev/null || true

  # Test health endpoint if port-forward is available
  local pod
  pod=$(kubectl get pod -n "$NAMESPACE" -l app.kubernetes.io/name=campaign-express \
    -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")
  if [ -n "$pod" ]; then
    log "Testing health endpoint on pod $pod..."
    kubectl exec -n "$NAMESPACE" "$pod" -- curl -sf http://localhost:8080/health 2>/dev/null && \
      ok "Health check passed" || warn "Health check failed or curl not available"
  fi
}

# ── Utilities ───────────────────────────────────────────────────────────────

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

  echo "  Project:      $PROJECT_NAME"
  echo "  Environment:  $ENVIRONMENT"
  echo "  Region:       $AWS_REGION"
  echo "  EKS Cluster:  $CLUSTER_NAME"
  echo "  ECR Image:    $ecr_url:$IMAGE_TAG"
  echo "  Redis:        $redis"
  echo "  Namespace:    $NAMESPACE"
  echo ""
  echo "  Useful commands:"
  echo "    kubectl get pods -n $NAMESPACE"
  echo "    kubectl logs -f deploy/campaign-express -n $NAMESPACE"
  echo "    kubectl port-forward svc/campaign-express 8080:8080 -n $NAMESPACE"
  echo "    kubectl port-forward svc/grafana 3000:3000 -n $NAMESPACE"
  echo ""
}

usage() {
  cat <<EOF
Campaign Express — AWS Deployment Script

Usage: $(basename "$0") [OPTIONS]

Options:
  --stage <stage>    Run a specific deployment stage:
                       infra     — Provision AWS infrastructure (Terraform)
                       build     — Build & push Docker image to ECR
                       services  — Deploy in-cluster services (NATS, ClickHouse, etc.)
                       app       — Deploy Campaign Express application (Helm)
                       monitor   — Deploy monitoring stack
  --destroy          Tear down all resources
  --health           Run health checks
  --help             Show this help message

Environment Variables:
  AWS_REGION         AWS region (default: us-east-1)
  ENVIRONMENT        Deployment environment: dev|staging|prod (default: prod)
  PROJECT_NAME       Project name prefix (default: campaign-express)
  IMAGE_TAG          Docker image tag (default: git short SHA)

Examples:
  # Full deployment
  ./deploy-aws.sh

  # Deploy only infrastructure
  ./deploy-aws.sh --stage infra

  # Deploy to staging in us-west-2
  AWS_REGION=us-west-2 ENVIRONMENT=staging ./deploy-aws.sh

  # Build and push image only
  ./deploy-aws.sh --stage build

  # Check health of deployment
  ./deploy-aws.sh --health

  # Tear everything down
  ./deploy-aws.sh --destroy
EOF
}

# ── Main ────────────────────────────────────────────────────────────────────

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
  echo "  Region:      $AWS_REGION"
  echo "  Environment: $ENVIRONMENT"
  echo "  Image Tag:   $IMAGE_TAG"
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
    infra)    deploy_infrastructure ;;
    build)    build_and_push ;;
    services) deploy_services ;;
    app)      deploy_application ;;
    monitor)  deploy_monitoring ;;
    "")
      # Full deployment — all stages
      deploy_infrastructure
      build_and_push
      deploy_services
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
