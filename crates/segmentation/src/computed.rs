//! Computed properties — derived user attributes recalculated in real-time.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputationType {
    Count {
        event_name: String,
        within_days: u32,
    },
    Sum {
        event_name: String,
        property: String,
        within_days: u32,
    },
    Average {
        event_name: String,
        property: String,
        within_days: u32,
    },
    Min {
        event_name: String,
        property: String,
        within_days: u32,
    },
    Max {
        event_name: String,
        property: String,
        within_days: u32,
    },
    MostRecent {
        event_name: String,
    },
    FirstOccurrence {
        event_name: String,
    },
    UniqueCount {
        event_name: String,
        property: String,
        within_days: u32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedProperty {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub computation: ComputationType,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct ComputedPropertyEngine {
    properties: dashmap::DashMap<Uuid, ComputedProperty>,
    cache: dashmap::DashMap<(Uuid, Uuid), serde_json::Value>,
}

impl ComputedPropertyEngine {
    pub fn new() -> Self {
        Self {
            properties: dashmap::DashMap::new(),
            cache: dashmap::DashMap::new(),
        }
    }

    pub fn register_property(&self, property: ComputedProperty) {
        self.properties.insert(property.id, property);
    }

    pub fn get_value(&self, user_id: &Uuid, property_id: &Uuid) -> Option<serde_json::Value> {
        self.cache.get(&(*user_id, *property_id)).map(|v| v.clone())
    }

    pub fn set_value(&self, user_id: Uuid, property_id: Uuid, value: serde_json::Value) {
        self.cache.insert((user_id, property_id), value);
    }

    pub fn get_all_for_user(
        &self,
        user_id: &Uuid,
    ) -> std::collections::HashMap<String, serde_json::Value> {
        let mut result = std::collections::HashMap::new();
        for prop in self.properties.iter() {
            if let Some(val) = self.cache.get(&(*user_id, *prop.key())) {
                result.insert(prop.value().name.clone(), val.clone());
            }
        }
        result
    }

    pub fn list_properties(&self) -> Vec<ComputedProperty> {
        self.properties.iter().map(|p| p.value().clone()).collect()
    }
}

impl Default for ComputedPropertyEngine {
    fn default() -> Self {
        Self::new()
    }
}
