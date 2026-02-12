//! Backup scheduling and management for Campaign Express infrastructure.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackupTarget {
    Redis,
    ClickHouse,
    Configs,
    Models,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackupStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSchedule {
    pub id: Uuid,
    pub target: BackupTarget,
    pub cron_expression: String,
    pub retention_days: u32,
    pub storage_path: String,
    pub enabled: bool,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRecord {
    pub id: Uuid,
    pub schedule_id: Uuid,
    pub target: BackupTarget,
    pub status: BackupStatus,
    pub size_bytes: u64,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub storage_path: String,
    pub error: Option<String>,
}

pub struct BackupManager {
    schedules: DashMap<Uuid, BackupSchedule>,
    records: DashMap<Uuid, BackupRecord>,
}

impl BackupManager {
    pub fn new() -> Self {
        info!("Backup manager initialized");
        let mgr = Self {
            schedules: DashMap::new(),
            records: DashMap::new(),
        };
        mgr.seed_demo_data();
        mgr
    }

    pub fn create_schedule(
        &self,
        target: BackupTarget,
        cron_expression: String,
        retention_days: u32,
        storage_path: String,
    ) -> BackupSchedule {
        let schedule = BackupSchedule {
            id: Uuid::new_v4(),
            target,
            cron_expression,
            retention_days,
            storage_path,
            enabled: true,
            last_run: None,
            next_run: Utc::now() + Duration::hours(1),
        };
        self.schedules.insert(schedule.id, schedule.clone());
        schedule
    }

    pub fn trigger_backup(&self, schedule_id: Uuid) -> Option<BackupRecord> {
        self.schedules.get(&schedule_id).map(|schedule| {
            let now = Utc::now();
            let record = BackupRecord {
                id: Uuid::new_v4(),
                schedule_id,
                target: schedule.target.clone(),
                status: BackupStatus::Completed,
                size_bytes: 1_048_576 * 50, // simulated 50 MB
                started_at: now,
                completed_at: Some(now + Duration::seconds(30)),
                storage_path: format!("{}/backup-{}.tar.gz", schedule.storage_path, now.timestamp()),
                error: None,
            };
            self.records.insert(record.id, record.clone());
            record
        })
    }

    pub fn list_schedules(&self) -> Vec<BackupSchedule> {
        self.schedules.iter().map(|r| r.value().clone()).collect()
    }

    pub fn list_backups(&self) -> Vec<BackupRecord> {
        let mut records: Vec<BackupRecord> = self.records.iter().map(|r| r.value().clone()).collect();
        records.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        records
    }

    pub fn get_latest_backup(&self, target: &BackupTarget) -> Option<BackupRecord> {
        let mut matching: Vec<BackupRecord> = self
            .records
            .iter()
            .filter(|r| &r.value().target == target)
            .map(|r| r.value().clone())
            .collect();
        matching.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        matching.into_iter().next()
    }

    fn seed_demo_data(&self) {
        let now = Utc::now();

        let targets = vec![
            (BackupTarget::Redis, "0 2 * * *", 30, "/backups/redis"),
            (BackupTarget::ClickHouse, "0 3 * * *", 90, "/backups/clickhouse"),
            (BackupTarget::Configs, "0 0 * * *", 365, "/backups/configs"),
            (BackupTarget::Models, "0 4 * * 0", 60, "/backups/models"),
        ];

        for (target, cron, retention, path) in targets {
            let schedule_id = Uuid::new_v4();
            let schedule = BackupSchedule {
                id: schedule_id,
                target: target.clone(),
                cron_expression: cron.to_string(),
                retention_days: retention,
                storage_path: path.to_string(),
                enabled: true,
                last_run: Some(now - Duration::hours(12)),
                next_run: now + Duration::hours(12),
            };
            self.schedules.insert(schedule_id, schedule);

            let record = BackupRecord {
                id: Uuid::new_v4(),
                schedule_id,
                target,
                status: BackupStatus::Completed,
                size_bytes: 1_048_576 * 120,
                started_at: now - Duration::hours(12),
                completed_at: Some(now - Duration::hours(12) + Duration::minutes(5)),
                storage_path: format!("{}/backup-{}.tar.gz", path, (now - Duration::hours(12)).timestamp()),
                error: None,
            };
            self.records.insert(record.id, record);
        }
    }
}

impl Default for BackupManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_schedule() {
        let manager = BackupManager::new();
        let schedule = manager.create_schedule(
            BackupTarget::Redis,
            "0 2 * * *".to_string(),
            30,
            "/backups/redis".to_string(),
        );
        assert!(schedule.enabled);
        assert_eq!(schedule.target, BackupTarget::Redis);

        let record = manager.trigger_backup(schedule.id).unwrap();
        assert_eq!(record.status, BackupStatus::Completed);
        assert_eq!(record.schedule_id, schedule.id);

        let latest = manager.get_latest_backup(&BackupTarget::Redis).unwrap();
        assert_eq!(latest.id, record.id);
    }
}
