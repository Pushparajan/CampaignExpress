//! Session management — tracks user sessions across mobile SDK interactions.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub device_id: String,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub event_count: u32,
    pub duration_seconds: Option<u64>,
    pub app_version: String,
}

pub struct SessionManager {
    active_sessions: std::collections::HashMap<String, UserSession>,
    session_timeout_seconds: u64,
}

impl SessionManager {
    pub fn new(session_timeout_seconds: u64) -> Self {
        Self {
            active_sessions: std::collections::HashMap::new(),
            session_timeout_seconds,
        }
    }

    pub fn start_session(
        &mut self,
        device_id: String,
        user_id: Option<Uuid>,
        app_version: String,
    ) -> UserSession {
        let now = Utc::now();
        let session = UserSession {
            id: Uuid::new_v4(),
            user_id,
            device_id: device_id.clone(),
            started_at: now,
            last_activity: now,
            ended_at: None,
            event_count: 0,
            duration_seconds: None,
            app_version,
        };
        self.active_sessions.insert(device_id, session.clone());
        session
    }

    pub fn touch(&mut self, device_id: &str) -> Option<&UserSession> {
        if let Some(session) = self.active_sessions.get_mut(device_id) {
            session.last_activity = Utc::now();
            session.event_count += 1;
        }
        self.active_sessions.get(device_id)
    }

    pub fn end_session(&mut self, device_id: &str) -> Option<UserSession> {
        if let Some(mut session) = self.active_sessions.remove(device_id) {
            let now = Utc::now();
            session.ended_at = Some(now);
            session.duration_seconds = Some((now - session.started_at).num_seconds().max(0) as u64);
            Some(session)
        } else {
            None
        }
    }

    pub fn get_session(&self, device_id: &str) -> Option<&UserSession> {
        self.active_sessions.get(device_id)
    }

    pub fn cleanup_expired(&mut self) -> Vec<UserSession> {
        let now = Utc::now();
        let timeout = chrono::Duration::seconds(self.session_timeout_seconds as i64);
        let expired_ids: Vec<String> = self
            .active_sessions
            .iter()
            .filter(|(_, s)| now - s.last_activity > timeout)
            .map(|(k, _)| k.clone())
            .collect();

        let mut ended = Vec::new();
        for id in expired_ids {
            if let Some(session) = self.end_session(&id) {
                ended.push(session);
            }
        }
        ended
    }
}
