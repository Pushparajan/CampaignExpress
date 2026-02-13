//! Public status page management for Campaign Express platform components.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComponentStatus {
    Operational,
    DegradedPerformance,
    PartialOutage,
    MajorOutage,
    Maintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusComponent {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub status: ComponentStatus,
    pub group: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusUpdate {
    pub id: Uuid,
    pub component_id: Uuid,
    pub title: String,
    pub message: String,
    pub status: ComponentStatus,
    pub created_at: DateTime<Utc>,
}

pub struct StatusPageManager {
    components: DashMap<Uuid, StatusComponent>,
    updates: DashMap<Uuid, StatusUpdate>,
}

impl StatusPageManager {
    pub fn new() -> Self {
        info!("Status page manager initialized");
        let mgr = Self {
            components: DashMap::new(),
            updates: DashMap::new(),
        };
        mgr.seed_demo_data();
        mgr
    }

    pub fn add_component(
        &self,
        name: String,
        description: String,
        group: String,
    ) -> StatusComponent {
        let component = StatusComponent {
            id: Uuid::new_v4(),
            name,
            description,
            status: ComponentStatus::Operational,
            group,
            updated_at: Utc::now(),
        };
        self.components.insert(component.id, component.clone());
        component
    }

    pub fn update_status(
        &self,
        component_id: Uuid,
        title: String,
        message: String,
        status: ComponentStatus,
    ) -> Option<StatusUpdate> {
        self.components.get_mut(&component_id).map(|mut comp| {
            comp.status = status.clone();
            comp.updated_at = Utc::now();

            let update = StatusUpdate {
                id: Uuid::new_v4(),
                component_id,
                title,
                message,
                status,
                created_at: Utc::now(),
            };
            self.updates.insert(update.id, update.clone());
            update
        })
    }

    pub fn get_status_page(&self) -> serde_json::Value {
        let mut groups: std::collections::BTreeMap<String, Vec<serde_json::Value>> =
            std::collections::BTreeMap::new();

        for entry in self.components.iter() {
            let comp = entry.value();
            let component_json = serde_json::json!({
                "id": comp.id,
                "name": comp.name,
                "description": comp.description,
                "status": comp.status,
                "updated_at": comp.updated_at,
            });
            groups
                .entry(comp.group.clone())
                .or_default()
                .push(component_json);
        }

        let all_operational = self
            .components
            .iter()
            .all(|r| r.value().status == ComponentStatus::Operational);

        let overall_status = if all_operational {
            "all_systems_operational"
        } else {
            "some_systems_affected"
        };

        serde_json::json!({
            "overall_status": overall_status,
            "updated_at": Utc::now(),
            "groups": groups,
        })
    }

    pub fn list_updates(&self) -> Vec<StatusUpdate> {
        let mut updates: Vec<StatusUpdate> =
            self.updates.iter().map(|r| r.value().clone()).collect();
        updates.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        updates
    }

    fn seed_demo_data(&self) {
        let now = Utc::now();

        let components = vec![
            ("API Gateway", "REST and gRPC API endpoint", "Core Services"),
            (
                "Bidding Engine",
                "Real-time bid processing pipeline",
                "Core Services",
            ),
            (
                "NATS Cluster",
                "Message queue for bid distribution",
                "Infrastructure",
            ),
            (
                "Redis Cluster",
                "Distributed caching layer",
                "Infrastructure",
            ),
            (
                "ClickHouse",
                "Analytics and reporting database",
                "Infrastructure",
            ),
            (
                "NPU Engine",
                "Neural processing unit inference engine",
                "ML Pipeline",
            ),
            (
                "Management UI",
                "Campaign management dashboard",
                "User Facing",
            ),
        ];

        for (name, description, group) in components {
            let component = StatusComponent {
                id: Uuid::new_v4(),
                name: name.to_string(),
                description: description.to_string(),
                status: ComponentStatus::Operational,
                group: group.to_string(),
                updated_at: now,
            };
            self.components.insert(component.id, component);
        }
    }
}

impl Default for StatusPageManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_page() {
        let manager = StatusPageManager::new();

        // All seeded components should be operational
        let page = manager.get_status_page();
        assert_eq!(page["overall_status"], "all_systems_operational");

        // Add a component and degrade it
        let comp = manager.add_component(
            "Test Service".to_string(),
            "A test service".to_string(),
            "Testing".to_string(),
        );
        assert_eq!(comp.status, ComponentStatus::Operational);

        let update = manager
            .update_status(
                comp.id,
                "Degraded Performance".to_string(),
                "Elevated latency detected".to_string(),
                ComponentStatus::DegradedPerformance,
            )
            .unwrap();
        assert_eq!(update.status, ComponentStatus::DegradedPerformance);

        let page = manager.get_status_page();
        assert_eq!(page["overall_status"], "some_systems_affected");

        let updates = manager.list_updates();
        assert!(!updates.is_empty());
    }
}
