# =============================================================================
# Campaign Express — AWS Terraform Outputs
# =============================================================================

output "vpc_id" {
  description = "VPC ID"
  value       = module.vpc.vpc_id
}

output "eks_cluster_name" {
  description = "EKS cluster name"
  value       = module.eks.cluster_name
}

output "eks_cluster_endpoint" {
  description = "EKS cluster API endpoint"
  value       = module.eks.cluster_endpoint
}

output "eks_cluster_certificate" {
  description = "EKS cluster CA certificate"
  value       = module.eks.cluster_certificate_authority_data
  sensitive   = true
}

output "ecr_repository_url" {
  description = "ECR repository URL for campaign-express image"
  value       = aws_ecr_repository.app.repository_url
}

output "redis_endpoint" {
  description = "ElastiCache Redis configuration endpoint"
  value       = aws_elasticache_replication_group.redis.configuration_endpoint_address
}

output "redis_port" {
  description = "ElastiCache Redis port"
  value       = aws_elasticache_replication_group.redis.port
}

output "redis_auth_secret_arn" {
  description = "ARN of the Redis auth token in Secrets Manager"
  value       = aws_secretsmanager_secret.redis_auth.arn
}

output "clickhouse_password_secret_arn" {
  description = "ARN of the ClickHouse password in Secrets Manager"
  value       = aws_secretsmanager_secret.clickhouse_password.arn
}

output "nats_auth_secret_arn" {
  description = "ARN of the NATS auth token in Secrets Manager"
  value       = aws_secretsmanager_secret.nats_auth.arn
}

output "external_secrets_role_arn" {
  description = "IAM role ARN for External Secrets Operator"
  value       = module.external_secrets_irsa.iam_role_arn
}

output "aws_region" {
  description = "AWS region"
  value       = var.aws_region
}
