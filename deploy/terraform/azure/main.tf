# =============================================================================
# Campaign Express — Azure AKS Terraform Configuration
# =============================================================================
# Provisions: AKS cluster, Azure Cache for Redis, ClickHouse VM,
# Container Registry, Key Vault, and networking.
# =============================================================================

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.80"
    }
  }

  backend "azurerm" {
    resource_group_name  = "campaign-express-tfstate"
    storage_account_name = "cetfstate"
    container_name       = "tfstate"
    key                  = "campaign-express.tfstate"
  }
}

provider "azurerm" {
  features {
    key_vault {
      purge_soft_delete_on_destroy = false
    }
  }
}

# ── Data Sources ──────────────────────────────────────────────────────────────

data "azurerm_client_config" "current" {}

# ── Resource Group ────────────────────────────────────────────────────────────

resource "azurerm_resource_group" "main" {
  name     = "${var.project_name}-${var.environment}-rg"
  location = var.location

  tags = local.tags
}

# ── Networking ────────────────────────────────────────────────────────────────

resource "azurerm_virtual_network" "main" {
  name                = "${var.project_name}-${var.environment}-vnet"
  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location
  address_space       = ["10.0.0.0/8"]

  tags = local.tags
}

resource "azurerm_subnet" "aks" {
  name                 = "aks-subnet"
  resource_group_name  = azurerm_resource_group.main.name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = ["10.240.0.0/16"]
}

resource "azurerm_subnet" "redis" {
  name                 = "redis-subnet"
  resource_group_name  = azurerm_resource_group.main.name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = ["10.241.0.0/24"]
}

resource "azurerm_subnet" "clickhouse" {
  name                 = "clickhouse-subnet"
  resource_group_name  = azurerm_resource_group.main.name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = ["10.241.1.0/24"]
}

# ── Container Registry ────────────────────────────────────────────────────────

resource "azurerm_container_registry" "main" {
  name                = replace("${var.project_name}${var.environment}acr", "-", "")
  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location
  sku                 = "Premium"
  admin_enabled       = false

  georeplications {
    location = var.dr_location
    tags     = local.tags
  }

  tags = local.tags
}

# ── AKS Cluster ───────────────────────────────────────────────────────────────

resource "azurerm_kubernetes_cluster" "main" {
  name                = "${var.project_name}-${var.environment}-aks"
  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location
  dns_prefix          = "${var.project_name}-${var.environment}"
  kubernetes_version  = var.kubernetes_version

  default_node_pool {
    name                = "system"
    vm_size             = "Standard_D4s_v5"
    node_count          = 3
    vnet_subnet_id      = azurerm_subnet.aks.id
    os_disk_size_gb     = 100
    max_pods            = 110
    enable_auto_scaling = true
    min_count           = 3
    max_count           = 5

    node_labels = {
      "role" = "system"
    }
  }

  identity {
    type = "SystemAssigned"
  }

  network_profile {
    network_plugin    = "azure"
    network_policy    = "calico"
    service_cidr      = "10.0.0.0/16"
    dns_service_ip    = "10.0.0.10"
    load_balancer_sku = "standard"
  }

  oms_agent {
    log_analytics_workspace_id = azurerm_log_analytics_workspace.main.id
  }

  tags = local.tags
}

# ── Bidding Engine Node Pool (with NPU support) ──────────────────────────────

resource "azurerm_kubernetes_cluster_node_pool" "bidding" {
  name                  = "bidding"
  kubernetes_cluster_id = azurerm_kubernetes_cluster.main.id
  vm_size               = var.bidding_vm_size
  node_count            = var.bidding_node_count
  vnet_subnet_id        = azurerm_subnet.aks.id
  os_disk_size_gb       = 200
  max_pods              = 30
  enable_auto_scaling   = true
  min_count             = var.bidding_min_count
  max_count             = var.bidding_max_count

  node_labels = {
    "role"         = "bidding"
    "amd.com/xdna" = "true"
  }

  node_taints = [
    "workload=bidding:NoSchedule"
  ]

  tags = local.tags
}

# ── Redis Cache ───────────────────────────────────────────────────────────────

resource "azurerm_redis_cache" "main" {
  name                = "${var.project_name}-${var.environment}-redis"
  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location
  capacity            = var.redis_capacity
  family              = "P"
  sku_name            = "Premium"
  enable_non_ssl_port = false
  minimum_tls_version = "1.2"
  shard_count         = var.redis_shard_count
  replicas_per_master = 1
  subnet_id           = azurerm_subnet.redis.id

  redis_configuration {
    maxmemory_policy       = "allkeys-lru"
    maxmemory_reserved     = 512
    maxfragmentationmemory_reserved = 512
  }

  tags = local.tags
}

# ── Key Vault ─────────────────────────────────────────────────────────────────

resource "azurerm_key_vault" "main" {
  name                       = "${var.project_name}-${var.environment}-kv"
  resource_group_name        = azurerm_resource_group.main.name
  location                   = azurerm_resource_group.main.location
  tenant_id                  = data.azurerm_client_config.current.tenant_id
  sku_name                   = "standard"
  soft_delete_retention_days = 90
  purge_protection_enabled   = true

  access_policy {
    tenant_id = data.azurerm_client_config.current.tenant_id
    object_id = azurerm_kubernetes_cluster.main.kubelet_identity[0].object_id

    secret_permissions = ["Get", "List"]
  }

  tags = local.tags
}

# ── Log Analytics ─────────────────────────────────────────────────────────────

resource "azurerm_log_analytics_workspace" "main" {
  name                = "${var.project_name}-${var.environment}-logs"
  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location
  sku                 = "PerGB2018"
  retention_in_days   = 30

  tags = local.tags
}

# ── NATS ──────────────────────────────────────────────────────────────────────

# NATS JetStream is deployed inside AKS via Helm — see deploy/helm/

# ── ClickHouse ────────────────────────────────────────────────────────────────

# For production ClickHouse, use a managed service (Altinity.Cloud, ClickHouse Cloud)
# or deploy via the ClickHouse Kubernetes Operator inside AKS.

# ── Locals ────────────────────────────────────────────────────────────────────

locals {
  tags = {
    Project     = var.project_name
    Environment = var.environment
    ManagedBy   = "terraform"
  }
}
