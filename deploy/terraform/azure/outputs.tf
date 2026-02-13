# =============================================================================
# Campaign Express — Terraform Outputs
# =============================================================================

output "resource_group_name" {
  value = azurerm_resource_group.main.name
}

output "aks_cluster_name" {
  value = azurerm_kubernetes_cluster.main.name
}

output "aks_kube_config" {
  value     = azurerm_kubernetes_cluster.main.kube_config_raw
  sensitive = true
}

output "acr_login_server" {
  value = azurerm_container_registry.main.login_server
}

output "redis_hostname" {
  value = azurerm_redis_cache.main.hostname
}

output "redis_primary_access_key" {
  value     = azurerm_redis_cache.main.primary_access_key
  sensitive = true
}

output "redis_ssl_port" {
  value = azurerm_redis_cache.main.ssl_port
}

output "key_vault_uri" {
  value = azurerm_key_vault.main.vault_uri
}

output "log_analytics_workspace_id" {
  value = azurerm_log_analytics_workspace.main.id
}

output "clickhouse_node_pool_id" {
  value = azurerm_kubernetes_cluster_node_pool.clickhouse.id
}

output "clickhouse_disk_ids" {
  value = azurerm_managed_disk.clickhouse_data[*].id
}
