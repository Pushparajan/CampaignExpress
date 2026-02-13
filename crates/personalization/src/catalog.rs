//! Product/content catalog management — store items that can be referenced
//! in messages and recommendations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<CatalogField>,
    pub item_count: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogField {
    pub name: String,
    pub field_type: CatalogFieldType,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogFieldType {
    String,
    Number,
    Boolean,
    Url,
    DateTime,
    Array,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogItem {
    pub id: String,
    pub catalog_id: Uuid,
    pub data: std::collections::HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct CatalogEngine {
    catalogs: dashmap::DashMap<Uuid, Catalog>,
    items: dashmap::DashMap<(Uuid, String), CatalogItem>,
}

impl CatalogEngine {
    pub fn new() -> Self {
        Self {
            catalogs: dashmap::DashMap::new(),
            items: dashmap::DashMap::new(),
        }
    }

    pub fn create_catalog(&self, catalog: Catalog) {
        self.catalogs.insert(catalog.id, catalog);
    }

    pub fn add_item(&self, item: CatalogItem) {
        let key = (item.catalog_id, item.id.clone());
        self.items.insert(key, item);
    }

    pub fn get_item(&self, catalog_id: &Uuid, item_id: &str) -> Option<CatalogItem> {
        self.items
            .get(&(*catalog_id, item_id.to_string()))
            .map(|i| i.clone())
    }

    #[allow(clippy::unnecessary_map_or)]
    pub fn search_items(&self, catalog_id: &Uuid, field: &str, value: &str) -> Vec<CatalogItem> {
        self.items
            .iter()
            .filter(|entry| {
                let item = entry.value();
                item.catalog_id == *catalog_id
                    && item
                        .data
                        .get(field)
                        .and_then(|v| v.as_str())
                        .map_or(false, |v| v.contains(value))
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn list_catalogs(&self) -> Vec<Catalog> {
        self.catalogs.iter().map(|c| c.value().clone()).collect()
    }

    pub fn catalog_item_count(&self, catalog_id: &Uuid) -> usize {
        self.items
            .iter()
            .filter(|entry| &entry.value().catalog_id == catalog_id)
            .count()
    }
}

impl Default for CatalogEngine {
    fn default() -> Self {
        Self::new()
    }
}
