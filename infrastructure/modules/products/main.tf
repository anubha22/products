resource "azurerm_storage_account" "products" {
  name                     = var.namespace
  resource_group_name      = var.resource_group_name
  location                 = var.location
  account_kind             = "StorageV2"
  account_tier             = "Standard"
  account_replication_type = "RAGRS"

  static_website {
    error_404_document = "404.html"
    index_document     = "index.html"
  }

  tags = {
    environment = var.environment
  }
}

resource "azurerm_storage_container" "docs" {
  name                  = "docs"
  storage_account_name  = azurerm_storage_account.products.name
  container_access_type = "blob"
}

resource "azurerm_storage_container" "temporary-docs" {
  name                  = "temporary-docs"
  storage_account_name  = azurerm_storage_account.products.name
  container_access_type = "blob"
}

resource "azurerm_storage_management_policy" "products" {
  storage_account_id = azurerm_storage_account.products.id

  rule {
    name    = "removeTemporaryBlobs"
    enabled = true
    filters {
      prefix_match = ["temporary-docs"]
      blob_types   = ["blockBlob"]
    }
    actions {
      base_blob {
        tier_to_cool_after_days_since_modification_greater_than    = 1
        tier_to_archive_after_days_since_modification_greater_than = 2
        delete_after_days_since_modification_greater_than          = 5
      }
      snapshot {
        delete_after_days_since_creation_greater_than = 5
      }
    }
  }
}

resource "azurerm_search_service" "search" {
  name                = var.namespace
  resource_group_name = var.resource_group_name
  location            = var.location
  sku                 = var.search_sku

  tags = {
    environment = var.environment
  }
}

resource "azurerm_container_registry" "products" {
  name                = "${var.namespace}registry"
  resource_group_name = var.resource_group_name
  location            = var.location
  sku                 = "Standard"
  admin_enabled       = true

  tags = {
    environment = var.environment
  }
}
