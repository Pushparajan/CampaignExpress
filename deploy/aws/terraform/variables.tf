# =============================================================================
# Campaign Express — AWS Terraform Variables
# =============================================================================

variable "project_name" {
  description = "Project name used as prefix for all resources"
  type        = string
  default     = "campaign-express"
}

variable "environment" {
  description = "Deployment environment (dev, staging, prod)"
  type        = string
  default     = "prod"

  validation {
    condition     = contains(["dev", "staging", "prod"], var.environment)
    error_message = "Environment must be dev, staging, or prod."
  }
}

variable "aws_region" {
  description = "AWS region for primary deployment"
  type        = string
  default     = "us-east-1"
}

variable "kubernetes_version" {
  description = "EKS Kubernetes version"
  type        = string
  default     = "1.29"
}

# ── VPC ──────────────────────────────────────────────────────────────────────

variable "vpc_cidr" {
  description = "CIDR block for the VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "availability_zones" {
  description = "Availability zones to deploy across"
  type        = list(string)
  default     = ["us-east-1a", "us-east-1b", "us-east-1c"]
}

# ── Bidding Node Group ──────────────────────────────────────────────────────

variable "bidding_instance_type" {
  description = "EC2 instance type for bidding engine nodes"
  type        = string
  default     = "c6i.4xlarge"
}

variable "bidding_desired_count" {
  description = "Desired number of bidding nodes"
  type        = number
  default     = 20
}

variable "bidding_min_count" {
  description = "Minimum bidding nodes (autoscaler)"
  type        = number
  default     = 10
}

variable "bidding_max_count" {
  description = "Maximum bidding nodes (autoscaler)"
  type        = number
  default     = 40
}

# ── System Node Group ──────────────────────────────────────────────────────

variable "system_instance_type" {
  description = "EC2 instance type for system workloads"
  type        = string
  default     = "m6i.xlarge"
}

variable "system_desired_count" {
  description = "Desired number of system nodes"
  type        = number
  default     = 3
}

# ── ClickHouse Node Group ──────────────────────────────────────────────────

variable "clickhouse_instance_type" {
  description = "EC2 instance type for ClickHouse (storage-optimized)"
  type        = string
  default     = "i3.2xlarge"
}

variable "clickhouse_desired_count" {
  description = "Desired number of ClickHouse nodes"
  type        = number
  default     = 3
}

variable "clickhouse_min_count" {
  description = "Minimum ClickHouse nodes"
  type        = number
  default     = 3
}

variable "clickhouse_max_count" {
  description = "Maximum ClickHouse nodes"
  type        = number
  default     = 6
}

# ── ElastiCache Redis ────────────────────────────────────────────────────────

variable "redis_node_type" {
  description = "ElastiCache Redis node type"
  type        = string
  default     = "cache.r6g.xlarge"
}

variable "redis_num_shards" {
  description = "Number of Redis shards (node groups)"
  type        = number
  default     = 3
}

variable "redis_replicas_per_shard" {
  description = "Replicas per shard"
  type        = number
  default     = 1
}

variable "redis_engine_version" {
  description = "Redis engine version"
  type        = string
  default     = "7.1"
}

# ── Tags ─────────────────────────────────────────────────────────────────────

variable "tags" {
  description = "Common tags for all resources"
  type        = map(string)
  default     = {}
}
