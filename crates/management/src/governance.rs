//! Governance workflows: revision-aware approvals, diff views, collaboration,
//! and policy-as-code go-live gates for campaigns, creatives, and templates.
//!
//! Addresses FR-GOV-001 through FR-GOV-005.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ─── Revision Model (FR-GOV-001) ──────────────────────────────────────

/// Type of governed object.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GovernedObjectType {
    Campaign,
    Creative,
    Template,
}

/// An immutable revision snapshot of a governed object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revision {
    pub id: Uuid,
    pub object_type: GovernedObjectType,
    pub object_id: Uuid,
    pub revision_number: u32,
    pub snapshot: HashMap<String, serde_json::Value>,
    pub submitted_by: Uuid,
    pub submitted_at: DateTime<Utc>,
    pub approval_id: Option<Uuid>,
    pub status: RevisionStatus,
    pub parent_revision: Option<Uuid>,
}

/// Status of a revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevisionStatus {
    Draft,
    Submitted,
    Approved,
    Rejected,
    Superseded,
}

/// Revision manager.
pub struct RevisionManager {
    revisions: DashMap<Uuid, Revision>,
    /// object_id -> ordered list of revision ids
    object_revisions: DashMap<Uuid, Vec<Uuid>>,
}

impl RevisionManager {
    pub fn new() -> Self {
        Self {
            revisions: DashMap::new(),
            object_revisions: DashMap::new(),
        }
    }

    /// Create a new revision for a governed object.
    pub fn create_revision(
        &self,
        object_type: GovernedObjectType,
        object_id: Uuid,
        snapshot: HashMap<String, serde_json::Value>,
        submitted_by: Uuid,
    ) -> Revision {
        let rev_number = self
            .object_revisions
            .get(&object_id)
            .map(|v| v.len() as u32 + 1)
            .unwrap_or(1);

        let parent = self
            .object_revisions
            .get(&object_id)
            .and_then(|v| v.last().copied());

        // Mark previous revision as superseded
        if let Some(parent_id) = parent {
            if let Some(mut prev) = self.revisions.get_mut(&parent_id) {
                if prev.status == RevisionStatus::Approved
                    || prev.status == RevisionStatus::Rejected
                {
                    prev.status = RevisionStatus::Superseded;
                }
            }
        }

        let revision = Revision {
            id: Uuid::new_v4(),
            object_type,
            object_id,
            revision_number: rev_number,
            snapshot,
            submitted_by,
            submitted_at: Utc::now(),
            approval_id: None,
            status: RevisionStatus::Draft,
            parent_revision: parent,
        };

        self.revisions.insert(revision.id, revision.clone());
        self.object_revisions
            .entry(object_id)
            .or_default()
            .push(revision.id);

        revision
    }

    /// Submit a revision for approval.
    pub fn submit(&self, revision_id: &Uuid) -> Result<Revision, String> {
        let mut entry = self
            .revisions
            .get_mut(revision_id)
            .ok_or("Revision not found")?;
        if entry.status != RevisionStatus::Draft {
            return Err("Only draft revisions can be submitted".to_string());
        }
        entry.status = RevisionStatus::Submitted;
        Ok(entry.clone())
    }

    /// Approve a revision.
    pub fn approve(&self, revision_id: &Uuid, approval_id: Uuid) -> Result<Revision, String> {
        let mut entry = self
            .revisions
            .get_mut(revision_id)
            .ok_or("Revision not found")?;
        if entry.status != RevisionStatus::Submitted {
            return Err("Only submitted revisions can be approved".to_string());
        }
        entry.status = RevisionStatus::Approved;
        entry.approval_id = Some(approval_id);
        Ok(entry.clone())
    }

    /// Reject a revision.
    pub fn reject(&self, revision_id: &Uuid) -> Result<Revision, String> {
        let mut entry = self
            .revisions
            .get_mut(revision_id)
            .ok_or("Revision not found")?;
        if entry.status != RevisionStatus::Submitted {
            return Err("Only submitted revisions can be rejected".to_string());
        }
        entry.status = RevisionStatus::Rejected;
        Ok(entry.clone())
    }

    /// Get the latest revision for an object.
    pub fn latest(&self, object_id: &Uuid) -> Option<Revision> {
        self.object_revisions
            .get(object_id)
            .and_then(|v| v.last().copied())
            .and_then(|id| self.revisions.get(&id).map(|r| r.clone()))
    }

    /// Get a specific revision by id.
    pub fn get(&self, revision_id: &Uuid) -> Option<Revision> {
        self.revisions.get(revision_id).map(|r| r.clone())
    }

    /// Get all revisions for an object.
    pub fn history(&self, object_id: &Uuid) -> Vec<Revision> {
        self.object_revisions
            .get(object_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.revisions.get(id).map(|r| r.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for RevisionManager {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Approval Routing (FR-GOV-002) ────────────────────────────────────

/// Role-based approver routing rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproverRoute {
    pub id: Uuid,
    pub name: String,
    pub role: String,
    pub object_types: Vec<GovernedObjectType>,
    pub conditions: Vec<RouteCondition>,
    pub due_date_hours: u32,
    pub escalation_hours: Option<u32>,
    pub escalation_to: Option<String>,
    pub required_fields: Vec<String>,
}

/// A condition for approval routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteCondition {
    BudgetAbove(f64),
    ChannelIs(String),
    RegionIs(String),
    ObjectTypeIs(GovernedObjectType),
    Always,
}

/// Reminder for an overdue approval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalReminder {
    pub approval_id: Uuid,
    pub approver_role: String,
    pub due_date: DateTime<Utc>,
    pub escalated: bool,
    pub escalation_target: Option<String>,
}

/// Approval routing engine.
pub struct ApprovalRouter {
    routes: DashMap<Uuid, ApproverRoute>,
}

impl ApprovalRouter {
    pub fn new() -> Self {
        let router = Self {
            routes: DashMap::new(),
        };
        router.seed_default_routes();
        router
    }

    fn seed_default_routes(&self) {
        let routes = vec![
            ApproverRoute {
                id: Uuid::new_v4(),
                name: "Brand Review".to_string(),
                role: "brand_manager".to_string(),
                object_types: vec![GovernedObjectType::Creative, GovernedObjectType::Template],
                conditions: vec![RouteCondition::Always],
                due_date_hours: 48,
                escalation_hours: Some(72),
                escalation_to: Some("brand_director".to_string()),
                required_fields: vec!["brand_guideline_check".to_string()],
            },
            ApproverRoute {
                id: Uuid::new_v4(),
                name: "Legal Review".to_string(),
                role: "legal".to_string(),
                object_types: vec![GovernedObjectType::Campaign],
                conditions: vec![RouteCondition::ChannelIs("sms".to_string())],
                due_date_hours: 72,
                escalation_hours: Some(96),
                escalation_to: Some("legal_director".to_string()),
                required_fields: vec![
                    "legal_compliance_check".to_string(),
                    "opt_out_mechanism".to_string(),
                ],
            },
            ApproverRoute {
                id: Uuid::new_v4(),
                name: "Finance Review".to_string(),
                role: "finance".to_string(),
                object_types: vec![GovernedObjectType::Campaign],
                conditions: vec![RouteCondition::BudgetAbove(10000.0)],
                due_date_hours: 48,
                escalation_hours: None,
                escalation_to: None,
                required_fields: vec!["budget_approval".to_string()],
            },
            ApproverRoute {
                id: Uuid::new_v4(),
                name: "Compliance Review".to_string(),
                role: "compliance".to_string(),
                object_types: vec![GovernedObjectType::Campaign],
                conditions: vec![RouteCondition::ChannelIs("whatsapp".to_string())],
                due_date_hours: 48,
                escalation_hours: Some(72),
                escalation_to: Some("compliance_director".to_string()),
                required_fields: vec!["regulatory_check".to_string()],
            },
        ];

        for route in routes {
            self.routes.insert(route.id, route);
        }
    }

    /// Determine which approval routes match a given object.
    pub fn route(
        &self,
        object_type: &GovernedObjectType,
        budget: f64,
        channels: &[String],
        region: &str,
    ) -> Vec<ApproverRoute> {
        let routes: Vec<ApproverRoute> = self.routes.iter().map(|e| e.value().clone()).collect();

        routes
            .into_iter()
            .filter(|route| {
                if !route.object_types.contains(object_type) {
                    return false;
                }
                route.conditions.iter().any(|cond| match cond {
                    RouteCondition::Always => true,
                    RouteCondition::BudgetAbove(threshold) => budget > *threshold,
                    RouteCondition::ChannelIs(ch) => channels.iter().any(|c| c == ch),
                    RouteCondition::RegionIs(r) => region == r,
                    RouteCondition::ObjectTypeIs(ot) => ot == object_type,
                })
            })
            .collect()
    }

    /// Generate reminders for overdue approvals.
    pub fn check_reminders(
        &self,
        pending_approvals: &[(Uuid, String, DateTime<Utc>)],
    ) -> Vec<ApprovalReminder> {
        let now = Utc::now();
        let routes: Vec<ApproverRoute> = self.routes.iter().map(|e| e.value().clone()).collect();

        pending_approvals
            .iter()
            .filter_map(|(approval_id, role, due_date)| {
                if *due_date < now {
                    let route = routes.iter().find(|r| r.role == *role);
                    let escalated = route
                        .and_then(|r| r.escalation_hours)
                        .is_some_and(|eh| *due_date + chrono::Duration::hours(eh as i64) < now);

                    Some(ApprovalReminder {
                        approval_id: *approval_id,
                        approver_role: role.clone(),
                        due_date: *due_date,
                        escalated,
                        escalation_target: if escalated {
                            route.and_then(|r| r.escalation_to.clone())
                        } else {
                            None
                        },
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

impl Default for ApprovalRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Diff View (FR-GOV-003) ──────────────────────────────────────────

/// A single field change between two revisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDiff {
    pub field: String,
    pub old_value: Option<serde_json::Value>,
    pub new_value: Option<serde_json::Value>,
    pub change_type: ChangeType,
}

/// Type of change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Removed,
}

/// Full diff between two revisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionDiff {
    pub from_revision: u32,
    pub to_revision: u32,
    pub changes: Vec<FieldDiff>,
    pub total_changes: usize,
}

/// Diff engine for comparing revisions.
pub struct DiffEngine;

impl DiffEngine {
    /// Compute the diff between two revision snapshots.
    pub fn diff(
        old: &HashMap<String, serde_json::Value>,
        new: &HashMap<String, serde_json::Value>,
        old_rev: u32,
        new_rev: u32,
    ) -> RevisionDiff {
        let mut changes = Vec::new();

        // Check for modified and removed fields
        for (key, old_val) in old {
            match new.get(key) {
                Some(new_val) if old_val != new_val => {
                    changes.push(FieldDiff {
                        field: key.clone(),
                        old_value: Some(old_val.clone()),
                        new_value: Some(new_val.clone()),
                        change_type: ChangeType::Modified,
                    });
                }
                None => {
                    changes.push(FieldDiff {
                        field: key.clone(),
                        old_value: Some(old_val.clone()),
                        new_value: None,
                        change_type: ChangeType::Removed,
                    });
                }
                _ => {}
            }
        }

        // Check for added fields
        for (key, new_val) in new {
            if !old.contains_key(key) {
                changes.push(FieldDiff {
                    field: key.clone(),
                    old_value: None,
                    new_value: Some(new_val.clone()),
                    change_type: ChangeType::Added,
                });
            }
        }

        let total = changes.len();
        RevisionDiff {
            from_revision: old_rev,
            to_revision: new_rev,
            changes,
            total_changes: total,
        }
    }
}

// ─── Collaboration (FR-GOV-004) ──────────────────────────────────────

/// A threaded comment on a governed object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: Uuid,
    pub object_id: Uuid,
    pub revision_id: Option<Uuid>,
    pub parent_comment_id: Option<Uuid>,
    pub author_id: Uuid,
    pub author_name: String,
    pub body: String,
    pub mentions: Vec<Uuid>,
    pub attachments: Vec<Attachment>,
    pub resolved: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// An attachment on a comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Uuid,
    pub filename: String,
    pub url: String,
    pub mime_type: String,
    pub size_bytes: u64,
}

/// A "request changes" task mapped to a specific field/section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeTask {
    pub id: Uuid,
    pub object_id: Uuid,
    pub revision_id: Uuid,
    pub field: String,
    pub description: String,
    pub assigned_to: Option<Uuid>,
    pub completed: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Collaboration engine for comments, mentions, and change tasks.
pub struct CollaborationEngine {
    comments: DashMap<Uuid, Comment>,
    tasks: DashMap<Uuid, ChangeTask>,
}

impl CollaborationEngine {
    pub fn new() -> Self {
        Self {
            comments: DashMap::new(),
            tasks: DashMap::new(),
        }
    }

    /// Add a comment on a governed object (optionally on a specific revision).
    #[allow(clippy::too_many_arguments)]
    pub fn add_comment(
        &self,
        object_id: Uuid,
        revision_id: Option<Uuid>,
        parent_comment_id: Option<Uuid>,
        author_id: Uuid,
        author_name: String,
        body: String,
        mentions: Vec<Uuid>,
    ) -> Comment {
        let now = Utc::now();
        let comment = Comment {
            id: Uuid::new_v4(),
            object_id,
            revision_id,
            parent_comment_id,
            author_id,
            author_name,
            body,
            mentions,
            attachments: Vec::new(),
            resolved: false,
            created_at: now,
            updated_at: now,
        };
        self.comments.insert(comment.id, comment.clone());
        comment
    }

    /// Get all comments for an object (threaded).
    pub fn get_comments(&self, object_id: &Uuid) -> Vec<Comment> {
        self.comments
            .iter()
            .filter(|e| e.value().object_id == *object_id)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Get comments mentioning a specific user.
    pub fn mentions_for_user(&self, user_id: &Uuid) -> Vec<Comment> {
        self.comments
            .iter()
            .filter(|e| e.value().mentions.contains(user_id))
            .map(|e| e.value().clone())
            .collect()
    }

    /// Resolve a comment thread.
    pub fn resolve_comment(&self, comment_id: &Uuid) -> Result<(), String> {
        let mut entry = self
            .comments
            .get_mut(comment_id)
            .ok_or("Comment not found")?;
        entry.resolved = true;
        entry.updated_at = Utc::now();
        Ok(())
    }

    /// Create a change task mapped to a specific field.
    pub fn create_task(
        &self,
        object_id: Uuid,
        revision_id: Uuid,
        field: String,
        description: String,
        assigned_to: Option<Uuid>,
        created_by: Uuid,
    ) -> ChangeTask {
        let task = ChangeTask {
            id: Uuid::new_v4(),
            object_id,
            revision_id,
            field,
            description,
            assigned_to,
            completed: false,
            created_by,
            created_at: Utc::now(),
            completed_at: None,
        };
        self.tasks.insert(task.id, task.clone());
        task
    }

    /// Complete a change task.
    pub fn complete_task(&self, task_id: &Uuid) -> Result<ChangeTask, String> {
        let mut entry = self.tasks.get_mut(task_id).ok_or("Task not found")?;
        entry.completed = true;
        entry.completed_at = Some(Utc::now());
        Ok(entry.clone())
    }

    /// Get all tasks for an object.
    pub fn tasks_for_object(&self, object_id: &Uuid) -> Vec<ChangeTask> {
        self.tasks
            .iter()
            .filter(|e| e.value().object_id == *object_id)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Check if all tasks for a revision are completed.
    pub fn all_tasks_complete(&self, revision_id: &Uuid) -> bool {
        self.tasks
            .iter()
            .filter(|e| e.value().revision_id == *revision_id)
            .all(|e| e.value().completed)
    }
}

impl Default for CollaborationEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Policy Rules (FR-GOV-005) ────────────────────────────────────────

/// A policy rule that must pass before go-live.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub check_type: PolicyCheckType,
    pub blocking: bool,
    pub applies_to: Vec<GovernedObjectType>,
}

/// Type of policy check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyCheckType {
    BrandCompliancePass,
    LegalReviewComplete,
    UnsubscribeLinkPresent,
    AssetRightsValid,
    BudgetApproved,
    FrequencyCapSet,
    QuietHoursConfigured,
    ConditionalApproval { channel: String },
}

/// Result of evaluating a policy rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCheckResult {
    pub rule_id: Uuid,
    pub rule_name: String,
    pub passed: bool,
    pub blocking: bool,
    pub message: String,
}

/// Policy engine for go-live gates.
pub struct PolicyEngine {
    rules: DashMap<Uuid, PolicyRule>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        let engine = Self {
            rules: DashMap::new(),
        };
        engine.seed_default_policies();
        engine
    }

    fn seed_default_policies(&self) {
        let policies = vec![
            PolicyRule {
                id: Uuid::new_v4(),
                name: "Brand Compliance".to_string(),
                description: "All creatives must pass brand guideline validation".to_string(),
                check_type: PolicyCheckType::BrandCompliancePass,
                blocking: true,
                applies_to: vec![GovernedObjectType::Campaign, GovernedObjectType::Creative],
            },
            PolicyRule {
                id: Uuid::new_v4(),
                name: "Legal Review".to_string(),
                description: "Legal review must be completed for regulated channels".to_string(),
                check_type: PolicyCheckType::LegalReviewComplete,
                blocking: true,
                applies_to: vec![GovernedObjectType::Campaign],
            },
            PolicyRule {
                id: Uuid::new_v4(),
                name: "Unsubscribe Link".to_string(),
                description: "Email campaigns must include an unsubscribe link".to_string(),
                check_type: PolicyCheckType::UnsubscribeLinkPresent,
                blocking: true,
                applies_to: vec![GovernedObjectType::Campaign, GovernedObjectType::Template],
            },
            PolicyRule {
                id: Uuid::new_v4(),
                name: "Asset Rights".to_string(),
                description: "All assets must have valid usage rights".to_string(),
                check_type: PolicyCheckType::AssetRightsValid,
                blocking: true,
                applies_to: vec![GovernedObjectType::Creative],
            },
            PolicyRule {
                id: Uuid::new_v4(),
                name: "Frequency Cap".to_string(),
                description: "Frequency cap should be configured for messaging campaigns"
                    .to_string(),
                check_type: PolicyCheckType::FrequencyCapSet,
                blocking: false,
                applies_to: vec![GovernedObjectType::Campaign],
            },
        ];

        for rule in policies {
            self.rules.insert(rule.id, rule);
        }
    }

    /// Evaluate all applicable policies for a governed object.
    pub fn evaluate(
        &self,
        object_type: &GovernedObjectType,
        checks_passed: &HashMap<String, bool>,
    ) -> Vec<PolicyCheckResult> {
        let rules: Vec<PolicyRule> = self
            .rules
            .iter()
            .filter(|e| e.value().applies_to.contains(object_type))
            .map(|e| e.value().clone())
            .collect();

        rules
            .iter()
            .map(|rule| {
                let check_key = format!("{:?}", rule.check_type);
                let passed = checks_passed.get(&check_key).copied().unwrap_or(false);

                PolicyCheckResult {
                    rule_id: rule.id,
                    rule_name: rule.name.clone(),
                    passed,
                    blocking: rule.blocking,
                    message: if passed {
                        format!("{}: passed", rule.name)
                    } else if rule.blocking {
                        format!("{}: BLOCKED — {}", rule.name, rule.description)
                    } else {
                        format!("{}: warning — {}", rule.name, rule.description)
                    },
                }
            })
            .collect()
    }

    /// Check if all blocking policies pass (i.e., go-live is allowed).
    pub fn can_go_live(
        &self,
        object_type: &GovernedObjectType,
        checks_passed: &HashMap<String, bool>,
    ) -> bool {
        let results = self.evaluate(object_type, checks_passed);
        results.iter().all(|r| r.passed || !r.blocking)
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_revision_lifecycle() {
        let manager = RevisionManager::new();
        let object_id = Uuid::new_v4();
        let user = Uuid::new_v4();

        // Create revision 1
        let mut snapshot1 = HashMap::new();
        snapshot1.insert("name".to_string(), serde_json::json!("Summer Sale"));
        snapshot1.insert("budget".to_string(), serde_json::json!(5000));

        let rev1 =
            manager.create_revision(GovernedObjectType::Campaign, object_id, snapshot1, user);
        assert_eq!(rev1.revision_number, 1);
        assert_eq!(rev1.status, RevisionStatus::Draft);
        assert!(rev1.parent_revision.is_none());

        // Submit and approve
        manager.submit(&rev1.id).unwrap();
        manager.approve(&rev1.id, Uuid::new_v4()).unwrap();

        // Create revision 2 (resubmit)
        let mut snapshot2 = HashMap::new();
        snapshot2.insert("name".to_string(), serde_json::json!("Summer Sale v2"));
        snapshot2.insert("budget".to_string(), serde_json::json!(7500));

        let rev2 =
            manager.create_revision(GovernedObjectType::Campaign, object_id, snapshot2, user);
        assert_eq!(rev2.revision_number, 2);
        assert!(rev2.parent_revision.is_some());

        // Revision 1 should now be superseded
        let rev1_updated = manager.get(&rev1.id).unwrap();
        assert_eq!(rev1_updated.status, RevisionStatus::Superseded);

        // History should show both
        let history = manager.history(&object_id);
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_diff_engine() {
        let mut old = HashMap::new();
        old.insert("name".to_string(), serde_json::json!("Campaign A"));
        old.insert("budget".to_string(), serde_json::json!(5000));
        old.insert("channel".to_string(), serde_json::json!("email"));

        let mut new = HashMap::new();
        new.insert("name".to_string(), serde_json::json!("Campaign A v2"));
        new.insert("budget".to_string(), serde_json::json!(7500));
        new.insert(
            "target_audience".to_string(),
            serde_json::json!("high_value"),
        );

        let diff = DiffEngine::diff(&old, &new, 1, 2);
        assert_eq!(diff.from_revision, 1);
        assert_eq!(diff.to_revision, 2);

        // name: modified, budget: modified, channel: removed, target_audience: added
        assert_eq!(diff.total_changes, 4);
        assert!(diff
            .changes
            .iter()
            .any(|c| c.field == "name" && c.change_type == ChangeType::Modified));
        assert!(diff
            .changes
            .iter()
            .any(|c| c.field == "channel" && c.change_type == ChangeType::Removed));
        assert!(diff
            .changes
            .iter()
            .any(|c| c.field == "target_audience" && c.change_type == ChangeType::Added));
    }

    #[test]
    fn test_approval_routing() {
        let router = ApprovalRouter::new();

        // High-budget campaign with SMS channel
        let routes = router.route(
            &GovernedObjectType::Campaign,
            25000.0,
            &["sms".to_string()],
            "US",
        );
        // Should match: legal (sms), finance (>10k), compliance (sms from seed data, if whatsapp — check)
        assert!(routes.iter().any(|r| r.role == "legal"));
        assert!(routes.iter().any(|r| r.role == "finance"));
    }

    #[test]
    fn test_collaboration_comments_and_tasks() {
        let engine = CollaborationEngine::new();
        let object_id = Uuid::new_v4();
        let revision_id = Uuid::new_v4();
        let alice = Uuid::new_v4();
        let bob = Uuid::new_v4();

        // Alice comments and mentions Bob
        let comment = engine.add_comment(
            object_id,
            Some(revision_id),
            None,
            alice,
            "Alice".to_string(),
            "Hey @Bob, can you review the headline?".to_string(),
            vec![bob],
        );

        // Bob replies
        engine.add_comment(
            object_id,
            Some(revision_id),
            Some(comment.id),
            bob,
            "Bob".to_string(),
            "Sure, looks good but shorten it.".to_string(),
            vec![],
        );

        let comments = engine.get_comments(&object_id);
        assert_eq!(comments.len(), 2);

        let bob_mentions = engine.mentions_for_user(&bob);
        assert_eq!(bob_mentions.len(), 1);

        // Create change task
        let task = engine.create_task(
            object_id,
            revision_id,
            "headline".to_string(),
            "Shorten headline to 30 chars".to_string(),
            Some(alice),
            bob,
        );

        assert!(!engine.all_tasks_complete(&revision_id));

        engine.complete_task(&task.id).unwrap();
        assert!(engine.all_tasks_complete(&revision_id));
    }

    #[test]
    fn test_policy_engine_go_live_gate() {
        let engine = PolicyEngine::new();

        // Missing required checks — should block
        let checks: HashMap<String, bool> = HashMap::new();
        assert!(!engine.can_go_live(&GovernedObjectType::Campaign, &checks));

        let results = engine.evaluate(&GovernedObjectType::Campaign, &checks);
        let blocking_failures: Vec<_> =
            results.iter().filter(|r| !r.passed && r.blocking).collect();
        assert!(!blocking_failures.is_empty());

        // All checks pass
        let mut checks = HashMap::new();
        checks.insert("BrandCompliancePass".to_string(), true);
        checks.insert("LegalReviewComplete".to_string(), true);
        checks.insert("UnsubscribeLinkPresent".to_string(), true);
        checks.insert("FrequencyCapSet".to_string(), true);

        assert!(engine.can_go_live(&GovernedObjectType::Campaign, &checks));
    }

    #[test]
    fn test_policy_non_blocking_warning() {
        let engine = PolicyEngine::new();

        // All blocking pass, but non-blocking fails
        let mut checks = HashMap::new();
        checks.insert("BrandCompliancePass".to_string(), true);
        checks.insert("LegalReviewComplete".to_string(), true);
        checks.insert("UnsubscribeLinkPresent".to_string(), true);
        // FrequencyCapSet is non-blocking, so omitting it should still allow go-live

        assert!(engine.can_go_live(&GovernedObjectType::Campaign, &checks));

        let results = engine.evaluate(&GovernedObjectType::Campaign, &checks);
        let warnings: Vec<_> = results
            .iter()
            .filter(|r| !r.passed && !r.blocking)
            .collect();
        assert!(!warnings.is_empty()); // FrequencyCapSet should be a warning
    }
}
