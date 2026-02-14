.PHONY: build build-release test lint fmt check docker-build docker-push \
       deploy-staging deploy-prod k8s-apply compose-up compose-down clean \
       aws-deploy aws-infra aws-build aws-services aws-app aws-monitor aws-health aws-destroy

# Variables
IMAGE_NAME ?= campaign-express
IMAGE_TAG ?= $(shell git rev-parse --short HEAD 2>/dev/null || echo "dev")
REGISTRY ?= ghcr.io/pushparajan
FULL_IMAGE = $(REGISTRY)/$(IMAGE_NAME):$(IMAGE_TAG)

# =============================================================================
# Build
# =============================================================================

build:
	cargo build --workspace

build-release:
	cargo build --release --workspace

# =============================================================================
# Quality
# =============================================================================

test:
	cargo test --workspace

test-integration:
	cargo test --workspace -- --ignored

lint:
	cargo clippy --workspace --all-targets -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

check: fmt-check lint test

# =============================================================================
# Docker
# =============================================================================

docker-build:
	docker build -t $(FULL_IMAGE) -f deploy/docker/Dockerfile .
	docker tag $(FULL_IMAGE) $(REGISTRY)/$(IMAGE_NAME):latest

docker-push: docker-build
	docker push $(FULL_IMAGE)
	docker push $(REGISTRY)/$(IMAGE_NAME):latest

# =============================================================================
# Local Development
# =============================================================================

compose-up:
	docker compose -f deploy/docker/docker-compose.yml up -d

compose-down:
	docker compose -f deploy/docker/docker-compose.yml down

compose-logs:
	docker compose -f deploy/docker/docker-compose.yml logs -f

run-local:
	RUST_LOG=campaign_express=debug \
	CAMPAIGN_EXPRESS__NPU__DEVICE=cpu \
	CAMPAIGN_EXPRESS__AGENTS_PER_NODE=2 \
	cargo run --bin campaign-express -- --api-only

# =============================================================================
# Kubernetes Deployment
# =============================================================================

k8s-apply-base:
	kubectl apply -k deploy/k8s/base

deploy-staging:
	kubectl apply -k deploy/k8s/overlays/staging

deploy-prod:
	kubectl apply -k deploy/k8s/overlays/production

deploy-infra:
	kubectl apply -f deploy/nats/nats-deployment.yaml
	kubectl apply -f deploy/redis/redis-deployment.yaml
	kubectl apply -f deploy/clickhouse/clickhouse-deployment.yaml
	kubectl apply -f deploy/haproxy/haproxy-deployment.yaml

deploy-monitoring:
	kubectl apply -f deploy/monitoring/prometheus/prometheus-deployment.yaml
	kubectl apply -f deploy/monitoring/grafana/grafana-deployment.yaml

deploy-all: deploy-infra deploy-monitoring deploy-prod

# =============================================================================
# AWS Deployment
# =============================================================================

aws-deploy:
	deploy/aws/deploy-aws.sh

aws-infra:
	deploy/aws/deploy-aws.sh --stage infra

aws-build:
	deploy/aws/deploy-aws.sh --stage build

aws-services:
	deploy/aws/deploy-aws.sh --stage services

aws-app:
	deploy/aws/deploy-aws.sh --stage app

aws-monitor:
	deploy/aws/deploy-aws.sh --stage monitor

aws-health:
	deploy/aws/deploy-aws.sh --health

aws-destroy:
	deploy/aws/deploy-aws.sh --destroy

# =============================================================================
# Cleanup
# =============================================================================

clean:
	cargo clean
	rm -rf target/
