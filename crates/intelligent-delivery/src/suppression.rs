//! Global suppression lists — do-not-contact management with per-channel
//! and global suppression, expiry support, and bulk operations.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Reason why an identifier was added to the suppression list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SuppressionReason {
    #[default]
    UserOptOut,
    Bounced,
    Complained,
    Regulatory,
    AdminAction,
    Blocklisted,
}

/// A single suppression record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuppressionEntry {
    pub id: Uuid,
    /// The suppressed identifier (email, phone, user-id, etc.).
    pub identifier: String,
    /// `None` means global (all channels); `Some(channel)` scopes to one channel.
    pub channel: Option<String>,
    pub reason: SuppressionReason,
    pub created_at: DateTime<Utc>,
    /// If set, the entry automatically expires at this time.
    pub expires_at: Option<DateTime<Utc>>,
    pub created_by: String,
}

// ---------------------------------------------------------------------------
// SuppressionList
// ---------------------------------------------------------------------------

/// Thread-safe suppression list backed by `DashMap`.
pub struct SuppressionList {
    /// Map from identifier -> Vec<SuppressionEntry>.
    entries: DashMap<String, Vec<SuppressionEntry>>,
}

impl SuppressionList {
    /// Create an empty suppression list.
    pub fn new() -> Self {
        Self {
            entries: DashMap::new(),
        }
    }

    /// Add a suppression entry for `identifier`.
    ///
    /// * `channel` - `None` for global suppression, `Some(ch)` for channel-specific.
    /// * `ttl_days` - optional time-to-live in days; entry auto-expires after this period.
    pub fn add(
        &self,
        identifier: &str,
        channel: Option<String>,
        reason: SuppressionReason,
        created_by: &str,
        ttl_days: Option<u32>,
    ) -> SuppressionEntry {
        let now = Utc::now();
        let entry = SuppressionEntry {
            id: Uuid::new_v4(),
            identifier: identifier.to_string(),
            channel,
            reason,
            created_at: now,
            expires_at: ttl_days.map(|d| now + Duration::days(i64::from(d))),
            created_by: created_by.to_string(),
        };

        self.entries
            .entry(identifier.to_string())
            .or_default()
            .push(entry.clone());

        tracing::info!(
            identifier,
            reason = ?entry.reason,
            "suppression entry added"
        );
        entry
    }

    /// Remove suppression entries for `identifier`.
    ///
    /// * If `channel` is `None`, removes **all** entries for the identifier.
    /// * If `channel` is `Some(ch)`, removes only entries matching that channel.
    ///
    /// Returns the number of entries removed.
    pub fn remove(&self, identifier: &str, channel: Option<&str>) -> usize {
        let mut removed = 0usize;

        if let Some(mut list) = self.entries.get_mut(identifier) {
            let before = list.len();
            match channel {
                None => {
                    removed = before;
                    list.clear();
                }
                Some(ch) => {
                    list.retain(|e| e.channel.as_deref() != Some(ch));
                    removed = before - list.len();
                }
            }
        }

        // Clean up empty key.
        if let Some(list) = self.entries.get(identifier) {
            if list.is_empty() {
                drop(list);
                self.entries.remove(identifier);
            }
        }

        if removed > 0 {
            tracing::info!(identifier, removed, "suppression entries removed");
        }
        removed
    }

    /// Check whether `identifier` is suppressed.
    ///
    /// * If `channel` is `None`, returns `true` if there is any active global entry.
    /// * If `channel` is `Some(ch)`, returns `true` if there is an active global entry
    ///   **or** an active entry for that specific channel.
    ///
    /// Expired entries are ignored.
    pub fn is_suppressed(&self, identifier: &str, channel: Option<&str>) -> bool {
        let now = Utc::now();

        let list = match self.entries.get(identifier) {
            Some(l) => l,
            None => return false,
        };

        list.iter().any(|entry| {
            // Skip expired entries.
            if let Some(exp) = entry.expires_at {
                if exp <= now {
                    return false;
                }
            }

            match (&entry.channel, channel) {
                // Global suppression applies to everything.
                (None, _) => true,
                // Channel-specific suppression matches the requested channel.
                (Some(entry_ch), Some(req_ch)) => entry_ch == req_ch,
                // Channel-specific entry does not match a global-only query.
                (Some(_), None) => false,
            }
        })
    }

    /// Return all entries (including expired) for a given identifier.
    pub fn get_entries(&self, identifier: &str) -> Vec<SuppressionEntry> {
        self.entries
            .get(identifier)
            .map(|list| list.clone())
            .unwrap_or_default()
    }

    /// Bulk-add suppressions. Returns the number of entries added.
    pub fn bulk_add(&self, items: Vec<(String, Option<String>, SuppressionReason)>) -> usize {
        let count = items.len();
        for (identifier, channel, reason) in items {
            self.add(&identifier, channel, reason, "bulk_import", None);
        }
        tracing::info!(count, "bulk suppression import completed");
        count
    }

    /// Purge all expired entries across the entire list. Returns the number
    /// of entries removed.
    pub fn purge_expired(&self) -> usize {
        let now = Utc::now();
        let mut purged = 0usize;
        let mut keys_to_remove = Vec::new();

        for mut entry in self.entries.iter_mut() {
            let before = entry.value().len();
            entry.value_mut().retain(|e| {
                if let Some(exp) = e.expires_at {
                    exp > now
                } else {
                    true // no expiry -> keep
                }
            });
            purged += before - entry.value().len();
            if entry.value().is_empty() {
                keys_to_remove.push(entry.key().clone());
            }
        }

        for key in keys_to_remove {
            self.entries.remove(&key);
        }

        if purged > 0 {
            tracing::info!(purged, "expired suppression entries purged");
        }
        purged
    }

    /// Total number of suppression entries across all identifiers.
    pub fn count(&self) -> usize {
        self.entries.iter().map(|e| e.value().len()).sum()
    }

    /// Seed 5 example suppressions for demo/testing purposes.
    pub fn seed_demo_data(&self) {
        self.add(
            "bounced@example.com",
            Some("email".to_string()),
            SuppressionReason::Bounced,
            "system",
            None,
        );
        self.add(
            "optout@example.com",
            None,
            SuppressionReason::UserOptOut,
            "user_self_service",
            None,
        );
        self.add(
            "+15551234567",
            Some("sms".to_string()),
            SuppressionReason::Regulatory,
            "compliance_team",
            Some(365),
        );
        self.add(
            "complainer@example.com",
            Some("email".to_string()),
            SuppressionReason::Complained,
            "postmaster",
            Some(90),
        );
        self.add(
            "blocked-user-99",
            None,
            SuppressionReason::Blocklisted,
            "admin",
            None,
        );

        tracing::info!("demo suppression data seeded (5 entries)");
    }
}

impl Default for SuppressionList {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_check() {
        let list = SuppressionList::new();

        list.add(
            "user@example.com",
            None,
            SuppressionReason::UserOptOut,
            "test",
            None,
        );

        // Global suppression should match any channel query.
        assert!(list.is_suppressed("user@example.com", None));
        assert!(list.is_suppressed("user@example.com", Some("email")));
        assert!(list.is_suppressed("user@example.com", Some("sms")));

        // Unknown identifier is not suppressed.
        assert!(!list.is_suppressed("other@example.com", None));
    }

    #[test]
    fn test_channel_specific_suppression() {
        let list = SuppressionList::new();

        list.add(
            "user@example.com",
            Some("email".to_string()),
            SuppressionReason::Bounced,
            "test",
            None,
        );

        // Should be suppressed for email but not for sms or global-only queries.
        assert!(list.is_suppressed("user@example.com", Some("email")));
        assert!(!list.is_suppressed("user@example.com", Some("sms")));
        assert!(!list.is_suppressed("user@example.com", None));
    }

    #[test]
    fn test_expiry_respected() {
        let list = SuppressionList::new();

        // Manually insert an entry that is already expired.
        let entry = SuppressionEntry {
            id: Uuid::new_v4(),
            identifier: "expired@example.com".to_string(),
            channel: None,
            reason: SuppressionReason::AdminAction,
            created_at: Utc::now() - Duration::days(10),
            expires_at: Some(Utc::now() - Duration::days(1)),
            created_by: "test".to_string(),
        };
        list.entries
            .entry("expired@example.com".to_string())
            .or_default()
            .push(entry);

        // Should not be suppressed because entry is expired.
        assert!(!list.is_suppressed("expired@example.com", None));
    }

    #[test]
    fn test_remove_all() {
        let list = SuppressionList::new();
        list.add("u@x.com", None, SuppressionReason::Bounced, "t", None);
        list.add(
            "u@x.com",
            Some("email".to_string()),
            SuppressionReason::Complained,
            "t",
            None,
        );

        assert_eq!(list.count(), 2);
        let removed = list.remove("u@x.com", None);
        assert_eq!(removed, 2);
        assert_eq!(list.count(), 0);
        assert!(!list.is_suppressed("u@x.com", None));
    }

    #[test]
    fn test_remove_by_channel() {
        let list = SuppressionList::new();
        list.add("u@x.com", None, SuppressionReason::Bounced, "t", None);
        list.add(
            "u@x.com",
            Some("email".to_string()),
            SuppressionReason::Complained,
            "t",
            None,
        );

        let removed = list.remove("u@x.com", Some("email"));
        assert_eq!(removed, 1);
        assert_eq!(list.count(), 1);
        // Global entry should still be there.
        assert!(list.is_suppressed("u@x.com", None));
    }

    #[test]
    fn test_bulk_add() {
        let list = SuppressionList::new();
        let items = vec![
            ("a@x.com".into(), None, SuppressionReason::Bounced),
            (
                "b@x.com".into(),
                Some("sms".into()),
                SuppressionReason::Regulatory,
            ),
            ("c@x.com".into(), None, SuppressionReason::UserOptOut),
        ];

        let added = list.bulk_add(items);
        assert_eq!(added, 3);
        assert_eq!(list.count(), 3);
        assert!(list.is_suppressed("a@x.com", None));
        assert!(list.is_suppressed("b@x.com", Some("sms")));
    }

    #[test]
    fn test_purge_expired() {
        let list = SuppressionList::new();

        // One non-expiring entry.
        list.add("keep@x.com", None, SuppressionReason::Bounced, "t", None);

        // One already-expired entry (manually inserted).
        let expired = SuppressionEntry {
            id: Uuid::new_v4(),
            identifier: "gone@x.com".to_string(),
            channel: None,
            reason: SuppressionReason::AdminAction,
            created_at: Utc::now() - Duration::days(100),
            expires_at: Some(Utc::now() - Duration::seconds(1)),
            created_by: "test".to_string(),
        };
        list.entries
            .entry("gone@x.com".to_string())
            .or_default()
            .push(expired);

        assert_eq!(list.count(), 2);

        let purged = list.purge_expired();
        assert_eq!(purged, 1);
        assert_eq!(list.count(), 1);
        assert!(list.is_suppressed("keep@x.com", None));
        assert!(!list.is_suppressed("gone@x.com", None));
    }

    #[test]
    fn test_get_entries() {
        let list = SuppressionList::new();
        list.add("u@x.com", None, SuppressionReason::Bounced, "t", None);
        list.add(
            "u@x.com",
            Some("email".into()),
            SuppressionReason::Complained,
            "t",
            None,
        );

        let entries = list.get_entries("u@x.com");
        assert_eq!(entries.len(), 2);

        let empty = list.get_entries("nonexistent@x.com");
        assert!(empty.is_empty());
    }

    #[test]
    fn test_seed_demo_data() {
        let list = SuppressionList::new();
        list.seed_demo_data();
        assert_eq!(list.count(), 5);

        assert!(list.is_suppressed("bounced@example.com", Some("email")));
        assert!(list.is_suppressed("optout@example.com", None));
        assert!(list.is_suppressed("+15551234567", Some("sms")));
        assert!(list.is_suppressed("blocked-user-99", Some("push")));
    }
}
