# =============================================================================
# Campaign Express — Terraform Variables
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

variable "location" {
  description = "Azure region for primary deployment"
  type        = string
  default     = "eastus2"
}

variable "dr_location" {
  description = "Azure region for disaster recovery / geo-replication"
  type        = string
  default     = "westus2"
}

variable "kubernetes_version" {
  description = "AKS Kubernetes version"
  type        = string
  default     = "1.29"
}

# ── Bidding Node Pool ─────────────────────────────────────────────────────────

variable "bidding_vm_size" {
  description = "VM size for bidding engine nodes (NPU-capable)"
  type        = string
  default     = "Standard_D16s_v5"
}

variable "bidding_node_count" {
  description = "Initial number of bidding nodes"
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

# ── Redis ─────────────────────────────────────────────────────────────────────

variable "redis_capacity" {
  description = "Redis cache capacity (1-4 for Premium)"
  type        = number
  default     = 2
}

variable "redis_shard_count" {
  description = "Number of Redis shards (Premium tier)"
  type        = number
  default     = 3
}
