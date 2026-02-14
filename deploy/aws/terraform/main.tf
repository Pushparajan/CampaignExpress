# =============================================================================
# Campaign Express — AWS Infrastructure (Terraform)
# =============================================================================
# Provisions: VPC, EKS, ECR, ElastiCache Redis, Secrets Manager, IAM
# =============================================================================

terraform {
  required_version = ">= 1.5"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }

  backend "s3" {
    bucket         = "campaign-express-tfstate"
    key            = "infrastructure/terraform.tfstate"
    region         = "us-east-1"
    dynamodb_table = "campaign-express-tflock"
    encrypt        = true
  }
}

provider "aws" {
  region = var.aws_region

  default_tags {
    tags = merge(var.tags, {
      Project     = var.project_name
      Environment = var.environment
      ManagedBy   = "terraform"
    })
  }
}

locals {
  name   = "${var.project_name}-${var.environment}"
  azs    = var.availability_zones
  vpc_id = module.vpc.vpc_id
}

# =============================================================================
# VPC
# =============================================================================

module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "~> 5.0"

  name = "${local.name}-vpc"
  cidr = var.vpc_cidr

  azs             = local.azs
  private_subnets = [for i, az in local.azs : cidrsubnet(var.vpc_cidr, 4, i)]
  public_subnets  = [for i, az in local.azs : cidrsubnet(var.vpc_cidr, 4, i + 4)]
  intra_subnets   = [for i, az in local.azs : cidrsubnet(var.vpc_cidr, 4, i + 8)]

  enable_nat_gateway   = true
  single_nat_gateway   = var.environment != "prod"
  enable_dns_hostnames = true
  enable_dns_support   = true

  # EKS requirements
  public_subnet_tags = {
    "kubernetes.io/role/elb"                    = 1
    "kubernetes.io/cluster/${local.name}-eks"   = "shared"
  }
  private_subnet_tags = {
    "kubernetes.io/role/internal-elb"           = 1
    "kubernetes.io/cluster/${local.name}-eks"   = "shared"
  }
}

# =============================================================================
# ECR — Container Registry
# =============================================================================

resource "aws_ecr_repository" "app" {
  name                 = var.project_name
  image_tag_mutability = "MUTABLE"
  force_delete         = var.environment != "prod"

  image_scanning_configuration {
    scan_on_push = true
  }

  encryption_configuration {
    encryption_type = "AES256"
  }
}

resource "aws_ecr_lifecycle_policy" "app" {
  repository = aws_ecr_repository.app.name

  policy = jsonencode({
    rules = [
      {
        rulePriority = 1
        description  = "Keep last 30 tagged images"
        selection = {
          tagStatus   = "tagged"
          tagPrefixList = ["v"]
          countType   = "imageCountMoreThan"
          countNumber = 30
        }
        action = { type = "expire" }
      },
      {
        rulePriority = 2
        description  = "Remove untagged images after 7 days"
        selection = {
          tagStatus   = "untagged"
          countType   = "sinceImagePushed"
          countUnit   = "days"
          countNumber = 7
        }
        action = { type = "expire" }
      }
    ]
  })
}

# =============================================================================
# EKS — Kubernetes Cluster
# =============================================================================

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.0"

  cluster_name    = "${local.name}-eks"
  cluster_version = var.kubernetes_version

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  cluster_endpoint_public_access  = true
  cluster_endpoint_private_access = true

  enable_cluster_creator_admin_permissions = true

  cluster_addons = {
    coredns                = { most_recent = true }
    kube-proxy             = { most_recent = true }
    vpc-cni                = { most_recent = true }
    aws-ebs-csi-driver     = { most_recent = true, service_account_role_arn = module.ebs_csi_irsa.iam_role_arn }
  }

  eks_managed_node_groups = {
    # ── System node group ──────────────────────────────────────────────────
    system = {
      name            = "${local.name}-system"
      instance_types  = [var.system_instance_type]
      desired_size    = var.system_desired_count
      min_size        = var.system_desired_count
      max_size        = var.system_desired_count + 2
      disk_size       = 100
      capacity_type   = "ON_DEMAND"

      labels = {
        role = "system"
      }
    }

    # ── Bidding engine node group (main workload) ──────────────────────────
    bidding = {
      name            = "${local.name}-bidding"
      instance_types  = [var.bidding_instance_type]
      desired_size    = var.bidding_desired_count
      min_size        = var.bidding_min_count
      max_size        = var.bidding_max_count
      disk_size       = 200
      capacity_type   = "ON_DEMAND"

      labels = {
        role = "bidding"
      }

      taints = {
        bidding = {
          key    = "workload"
          value  = "bidding"
          effect = "NO_SCHEDULE"
        }
      }
    }

    # ── ClickHouse node group (storage-optimized) ──────────────────────────
    clickhouse = {
      name            = "${local.name}-clickhouse"
      instance_types  = [var.clickhouse_instance_type]
      desired_size    = var.clickhouse_desired_count
      min_size        = var.clickhouse_min_count
      max_size        = var.clickhouse_max_count
      disk_size       = 500
      capacity_type   = "ON_DEMAND"

      labels = {
        role    = "clickhouse"
        storage = "nvme"
      }

      taints = {
        clickhouse = {
          key    = "workload"
          value  = "clickhouse"
          effect = "NO_SCHEDULE"
        }
      }
    }
  }
}

# EBS CSI Driver IRSA
module "ebs_csi_irsa" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.0"

  role_name             = "${local.name}-ebs-csi"
  attach_ebs_csi_policy = true

  oidc_providers = {
    main = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["kube-system:ebs-csi-controller-sa"]
    }
  }
}

# =============================================================================
# ElastiCache — Redis Cluster (Cluster Mode Enabled)
# =============================================================================

resource "aws_elasticache_subnet_group" "redis" {
  name       = "${local.name}-redis"
  subnet_ids = module.vpc.private_subnets
}

resource "aws_security_group" "redis" {
  name_prefix = "${local.name}-redis-"
  vpc_id      = module.vpc.vpc_id
  description = "Security group for ElastiCache Redis"

  ingress {
    description     = "Redis from EKS"
    from_port       = 6379
    to_port         = 6379
    protocol        = "tcp"
    security_groups = [module.eks.node_security_group_id]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_elasticache_replication_group" "redis" {
  replication_group_id = "${local.name}-redis"
  description          = "Campaign Express Redis cluster"

  engine               = "redis"
  engine_version       = var.redis_engine_version
  node_type            = var.redis_node_type
  port                 = 6379
  parameter_group_name = aws_elasticache_parameter_group.redis.name

  num_node_groups         = var.redis_num_shards
  replicas_per_node_group = var.redis_replicas_per_shard

  automatic_failover_enabled = true
  multi_az_enabled           = true
  at_rest_encryption_enabled = true
  transit_encryption_enabled = true
  auth_token                 = random_password.redis_auth.result

  subnet_group_name  = aws_elasticache_subnet_group.redis.name
  security_group_ids = [aws_security_group.redis.id]

  snapshot_retention_limit = var.environment == "prod" ? 7 : 1
  snapshot_window          = "03:00-05:00"
  maintenance_window       = "sun:05:00-sun:07:00"

  log_delivery_configuration {
    destination      = aws_cloudwatch_log_group.redis_slow.name
    destination_type = "cloudwatch-logs"
    log_format       = "json"
    log_type         = "slow-log"
  }
}

resource "aws_elasticache_parameter_group" "redis" {
  name   = "${local.name}-redis-params"
  family = "redis7"

  parameter {
    name  = "maxmemory-policy"
    value = "allkeys-lru"
  }

  parameter {
    name  = "tcp-keepalive"
    value = "300"
  }

  parameter {
    name  = "timeout"
    value = "0"
  }
}

resource "random_password" "redis_auth" {
  length  = 32
  special = false
}

resource "aws_cloudwatch_log_group" "redis_slow" {
  name              = "/elasticache/${local.name}/slow-log"
  retention_in_days = 14
}

# =============================================================================
# Secrets Manager
# =============================================================================

resource "aws_secretsmanager_secret" "redis_auth" {
  name                    = "${local.name}/redis-auth-token"
  recovery_window_in_days = var.environment == "prod" ? 30 : 0
}

resource "aws_secretsmanager_secret_version" "redis_auth" {
  secret_id     = aws_secretsmanager_secret.redis_auth.id
  secret_string = random_password.redis_auth.result
}

resource "aws_secretsmanager_secret" "clickhouse_password" {
  name                    = "${local.name}/clickhouse-password"
  recovery_window_in_days = var.environment == "prod" ? 30 : 0
}

resource "aws_secretsmanager_secret_version" "clickhouse_password" {
  secret_id     = aws_secretsmanager_secret.clickhouse_password.id
  secret_string = random_password.clickhouse.result
}

resource "random_password" "clickhouse" {
  length  = 32
  special = false
}

resource "aws_secretsmanager_secret" "nats_auth" {
  name                    = "${local.name}/nats-auth-token"
  recovery_window_in_days = var.environment == "prod" ? 30 : 0
}

resource "aws_secretsmanager_secret_version" "nats_auth" {
  secret_id     = aws_secretsmanager_secret.nats_auth.id
  secret_string = random_password.nats.result
}

resource "random_password" "nats" {
  length  = 32
  special = false
}

# =============================================================================
# IAM — IRSA for External Secrets Operator
# =============================================================================

module "external_secrets_irsa" {
  source  = "terraform-aws-modules/iam/aws//modules/iam-role-for-service-accounts-eks"
  version = "~> 5.0"

  role_name = "${local.name}-external-secrets"

  oidc_providers = {
    main = {
      provider_arn               = module.eks.oidc_provider_arn
      namespace_service_accounts = ["campaign-express:external-secrets-sa"]
    }
  }
}

resource "aws_iam_role_policy" "external_secrets" {
  name = "${local.name}-external-secrets-policy"
  role = module.external_secrets_irsa.iam_role_name

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "secretsmanager:GetSecretValue",
          "secretsmanager:DescribeSecret"
        ]
        Resource = "arn:aws:secretsmanager:${var.aws_region}:*:secret:${local.name}/*"
      }
    ]
  })
}

# =============================================================================
# CloudWatch Log Group for EKS
# =============================================================================

resource "aws_cloudwatch_log_group" "eks" {
  name              = "/aws/eks/${local.name}-eks/cluster"
  retention_in_days = 30
}
