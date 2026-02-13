//! Campaign approval workflows and campaign calendar.
//!
//! Provides a multi-step approval workflow engine for campaign lifecycle
//! management and a calendar for tracking campaign events.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Workflow Stage
// ---------------------------------------------------------------------------

/// Represents the current lifecycle stage of a campaign.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowStage {
    #[default]
    Draft,
    InReview,
    Approved,
    Rejected,
    Scheduled,
    Live,
    Paused,
    Completed,
    Archived,
}

// ---------------------------------------------------------------------------
// Approval Action
// ---------------------------------------------------------------------------

/// An action that moves a campaign from one stage to another.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApprovalAction {
    /// Draft -> InReview
    Submit,
    /// InReview -> Approved
    Approve,
    /// InReview -> Rejected
    Reject,
    /// InReview -> Draft
    RequestChanges,
    /// Approved -> Scheduled
    Schedule,
    /// Scheduled | Approved -> Live
    GoLive,
    /// Live -> Paused
    Pause,
    /// Paused -> Live
    Resume,
    /// Live -> Completed
    Complete,
    /// Completed -> Archived
    Archive,
}

// ---------------------------------------------------------------------------
// Workflow Transition
// ---------------------------------------------------------------------------

/// A recorded transition in a campaign's workflow history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransition {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub from_stage: WorkflowStage,
    pub to_stage: WorkflowStage,
    pub action: ApprovalAction,
    pub actor_id: Uuid,
    pub actor_role: String,
    pub comment: Option<String>,
    pub timestamp: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Approval Rule
// ---------------------------------------------------------------------------

/// A configurable rule that governs what is needed to approve a campaign.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRule {
    pub id: Uuid,
    pub name: String,
    pub required_role: String,
    pub min_approvals: u32,
    pub auto_approve_below_budget: Option<f64>,
    pub require_creative_review: bool,
    pub require_legal_review: bool,
    pub channels: Vec<String>,
}

// ---------------------------------------------------------------------------
// Approval Request / Approver Status
// ---------------------------------------------------------------------------

/// The overall status of an approval request.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    #[default]
    Pending,
    Approved,
    Rejected,
}

/// The decision of an individual approver.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApproverDecision {
    #[default]
    Pending,
    Approved,
    Rejected,
}

/// Tracks the decision status of a single approver.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproverStatus {
    pub user_id: Uuid,
    pub role: String,
    pub decision: Option<ApproverDecision>,
    pub comment: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
}

/// A request for one or more approvers to review a campaign.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub requested_by: Uuid,
    pub approvers: Vec<ApproverStatus>,
    pub rule_id: Uuid,
    pub status: ApprovalStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Workflow Engine
// ---------------------------------------------------------------------------

/// Multi-step approval workflow engine for campaign lifecycle management.
pub struct WorkflowEngine {
    /// campaign_id -> current stage
    pub stages: DashMap<Uuid, WorkflowStage>,
    /// campaign_id -> ordered list of transitions
    pub transitions: DashMap<Uuid, Vec<WorkflowTransition>>,
    /// rule_id -> approval rule
    pub approval_rules: DashMap<Uuid, ApprovalRule>,
    /// request_id -> approval request
    pub approval_requests: DashMap<Uuid, ApprovalRequest>,
}

impl WorkflowEngine {
    /// Create a new, empty workflow engine.
    pub fn new() -> Self {
        Self {
            stages: DashMap::new(),
            transitions: DashMap::new(),
            approval_rules: DashMap::new(),
            approval_requests: DashMap::new(),
        }
    }

    /// Register a campaign and place it in the `Draft` stage.
    pub fn register_campaign(&self, campaign_id: Uuid) -> WorkflowStage {
        self.stages.insert(campaign_id, WorkflowStage::Draft);
        self.transitions.insert(campaign_id, Vec::new());
        WorkflowStage::Draft
    }

    /// Get the current stage for a campaign.
    pub fn get_stage(&self, campaign_id: &Uuid) -> Option<WorkflowStage> {
        self.stages.get(campaign_id).map(|s| *s)
    }

    /// Get the full transition history for a campaign.
    pub fn get_history(&self, campaign_id: &Uuid) -> Vec<WorkflowTransition> {
        self.transitions
            .get(campaign_id)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// Attempt to transition a campaign via the given action.
    ///
    /// Returns the new stage on success, or an error message if the
    /// transition is not valid from the current stage.
    pub fn transition(
        &self,
        campaign_id: Uuid,
        action: ApprovalAction,
        actor_id: Uuid,
        actor_role: &str,
        comment: Option<String>,
    ) -> Result<WorkflowStage, String> {
        let current = self
            .stages
            .get(&campaign_id)
            .map(|s| *s)
            .ok_or_else(|| format!("Campaign {} not found", campaign_id))?;

        let new_stage = Self::validate_transition(current, action)?;

        // Record transition
        let transition = WorkflowTransition {
            id: Uuid::new_v4(),
            campaign_id,
            from_stage: current,
            to_stage: new_stage,
            action,
            actor_id,
            actor_role: actor_role.to_string(),
            comment,
            timestamp: Utc::now(),
        };

        self.stages.insert(campaign_id, new_stage);
        self.transitions
            .entry(campaign_id)
            .or_default()
            .push(transition);

        Ok(new_stage)
    }

    /// Validate that `action` is allowed from `current` and return the target stage.
    fn validate_transition(
        current: WorkflowStage,
        action: ApprovalAction,
    ) -> Result<WorkflowStage, String> {
        match (current, action) {
            (WorkflowStage::Draft, ApprovalAction::Submit) => Ok(WorkflowStage::InReview),
            (WorkflowStage::InReview, ApprovalAction::Approve) => Ok(WorkflowStage::Approved),
            (WorkflowStage::InReview, ApprovalAction::Reject) => Ok(WorkflowStage::Rejected),
            (WorkflowStage::InReview, ApprovalAction::RequestChanges) => Ok(WorkflowStage::Draft),
            (WorkflowStage::Approved, ApprovalAction::Schedule) => Ok(WorkflowStage::Scheduled),
            (WorkflowStage::Scheduled, ApprovalAction::GoLive)
            | (WorkflowStage::Approved, ApprovalAction::GoLive) => Ok(WorkflowStage::Live),
            (WorkflowStage::Live, ApprovalAction::Pause) => Ok(WorkflowStage::Paused),
            (WorkflowStage::Paused, ApprovalAction::Resume) => Ok(WorkflowStage::Live),
            (WorkflowStage::Live, ApprovalAction::Complete) => Ok(WorkflowStage::Completed),
            (WorkflowStage::Completed, ApprovalAction::Archive) => Ok(WorkflowStage::Archived),
            _ => Err(format!(
                "Invalid transition: cannot perform {:?} from {:?}",
                action, current
            )),
        }
    }

    /// Add an approval rule.
    pub fn add_approval_rule(&self, rule: ApprovalRule) {
        self.approval_rules.insert(rule.id, rule);
    }

    /// Submit a campaign for approval, creating an `ApprovalRequest`.
    ///
    /// `approver_ids` is a list of `(user_id, role)` tuples.
    pub fn submit_for_approval(
        &self,
        campaign_id: Uuid,
        requested_by: Uuid,
        approver_ids: Vec<(Uuid, String)>,
    ) -> ApprovalRequest {
        // Pick the first available rule, or create a fallback id.
        let rule_id = self
            .approval_rules
            .iter()
            .next()
            .map(|r| r.id)
            .unwrap_or_else(Uuid::new_v4);

        let approvers = approver_ids
            .into_iter()
            .map(|(user_id, role)| ApproverStatus {
                user_id,
                role,
                decision: None,
                comment: None,
                decided_at: None,
            })
            .collect();

        let request = ApprovalRequest {
            id: Uuid::new_v4(),
            campaign_id,
            requested_by,
            approvers,
            rule_id,
            status: ApprovalStatus::Pending,
            created_at: Utc::now(),
            resolved_at: None,
        };

        self.approval_requests.insert(request.id, request.clone());
        request
    }

    /// Record an individual approver's decision on a request.
    ///
    /// When enough approvals are collected (based on the associated rule's
    /// `min_approvals`), the request is auto-resolved and the campaign is
    /// transitioned accordingly.
    pub fn record_approval_decision(
        &self,
        request_id: Uuid,
        approver_id: Uuid,
        approved: bool,
        comment: Option<String>,
    ) -> Option<ApprovalRequest> {
        let mut entry = self.approval_requests.get_mut(&request_id)?;
        let request = entry.value_mut();

        // Find the approver and record the decision.
        for approver in &mut request.approvers {
            if approver.user_id == approver_id {
                approver.decision = Some(if approved {
                    ApproverDecision::Approved
                } else {
                    ApproverDecision::Rejected
                });
                approver.comment = comment.clone();
                approver.decided_at = Some(Utc::now());
                break;
            }
        }

        // Count approvals and rejections.
        let approved_count = request
            .approvers
            .iter()
            .filter(|a| a.decision == Some(ApproverDecision::Approved))
            .count() as u32;

        let rejected_count = request
            .approvers
            .iter()
            .filter(|a| a.decision == Some(ApproverDecision::Rejected))
            .count();

        // Check against the rule's min_approvals.
        let min_approvals = self
            .approval_rules
            .get(&request.rule_id)
            .map(|r| r.min_approvals)
            .unwrap_or(1);

        if approved_count >= min_approvals {
            request.status = ApprovalStatus::Approved;
            request.resolved_at = Some(Utc::now());

            // Auto-transition campaign: InReview -> Approved
            let campaign_id = request.campaign_id;
            drop(entry); // release borrow before mutating stages
            let _ = self.transition(
                campaign_id,
                ApprovalAction::Approve,
                approver_id,
                "system",
                Some("Auto-approved via workflow engine".to_string()),
            );
        } else if rejected_count > 0 {
            request.status = ApprovalStatus::Rejected;
            request.resolved_at = Some(Utc::now());

            let campaign_id = request.campaign_id;
            drop(entry);
            let _ = self.transition(
                campaign_id,
                ApprovalAction::Reject,
                approver_id,
                "system",
                Some("Rejected via workflow engine".to_string()),
            );
        } else {
            drop(entry);
        }

        self.approval_requests.get(&request_id).map(|r| r.clone())
    }

    /// Return all pending approval requests where the given user is a listed approver.
    pub fn get_pending_approvals(&self, approver_id: &Uuid) -> Vec<ApprovalRequest> {
        self.approval_requests
            .iter()
            .filter(|entry| {
                let req = entry.value();
                req.status == ApprovalStatus::Pending
                    && req.approvers.iter().any(|a| a.user_id == *approver_id)
            })
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Seed default approval rules.
    pub fn seed_default_rules(&self) {
        let standard = ApprovalRule {
            id: Uuid::new_v4(),
            name: "Standard Campaign".to_string(),
            required_role: "manager".to_string(),
            min_approvals: 1,
            auto_approve_below_budget: Some(1000.0),
            require_creative_review: false,
            require_legal_review: false,
            channels: vec![],
        };
        self.add_approval_rule(standard);

        let high_budget = ApprovalRule {
            id: Uuid::new_v4(),
            name: "High Budget Campaign".to_string(),
            required_role: "director".to_string(),
            min_approvals: 2,
            auto_approve_below_budget: None,
            require_creative_review: false,
            require_legal_review: true,
            channels: vec![],
        };
        self.add_approval_rule(high_budget);

        let regulated = ApprovalRule {
            id: Uuid::new_v4(),
            name: "Regulated Channel".to_string(),
            required_role: "compliance".to_string(),
            min_approvals: 2,
            auto_approve_below_budget: None,
            require_creative_review: false,
            require_legal_review: true,
            channels: vec!["sms".to_string(), "whatsapp".to_string()],
        };
        self.add_approval_rule(regulated);
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Campaign Calendar
// ---------------------------------------------------------------------------

/// The type of calendar event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CalendarEventType {
    Launch,
    End,
    Milestone,
    Review,
    Deadline,
}

/// A single event on the campaign calendar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub campaign_name: String,
    pub event_type: CalendarEventType,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    /// Hex colour, e.g. `"#FF5733"`.
    pub color: String,
    pub notes: Option<String>,
}

/// A simple in-memory campaign calendar backed by `DashMap`.
pub struct CampaignCalendar {
    pub events: DashMap<Uuid, CalendarEvent>,
}

impl CampaignCalendar {
    /// Create a new, empty calendar.
    pub fn new() -> Self {
        Self {
            events: DashMap::new(),
        }
    }

    /// Add an event to the calendar.
    pub fn add_event(&self, event: CalendarEvent) {
        self.events.insert(event.id, event);
    }

    /// Return all events whose `start_date` falls within `[from, to]`.
    pub fn get_events_in_range(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Vec<CalendarEvent> {
        self.events
            .iter()
            .filter(|e| e.start_date >= from && e.start_date <= to)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Return all events for a given campaign.
    pub fn get_campaign_events(&self, campaign_id: &Uuid) -> Vec<CalendarEvent> {
        self.events
            .iter()
            .filter(|e| e.campaign_id == *campaign_id)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Remove an event by id. Returns `true` if the event existed.
    pub fn remove_event(&self, event_id: &Uuid) -> bool {
        self.events.remove(event_id).is_some()
    }

    /// Return events whose `start_date` falls within the next `days` days.
    pub fn get_upcoming(&self, days: u32) -> Vec<CalendarEvent> {
        let now = Utc::now();
        let until = now + Duration::days(i64::from(days));
        self.get_events_in_range(now, until)
    }
}

impl Default for CampaignCalendar {
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

    // -----------------------------------------------------------------------
    // 1. Full workflow transition chain
    // -----------------------------------------------------------------------
    #[test]
    fn test_full_workflow_chain() {
        let engine = WorkflowEngine::new();
        let campaign = Uuid::new_v4();
        let actor = Uuid::new_v4();

        assert_eq!(engine.register_campaign(campaign), WorkflowStage::Draft);

        // Draft -> InReview
        let stage = engine
            .transition(campaign, ApprovalAction::Submit, actor, "editor", None)
            .unwrap();
        assert_eq!(stage, WorkflowStage::InReview);

        // InReview -> Approved
        let stage = engine
            .transition(campaign, ApprovalAction::Approve, actor, "manager", None)
            .unwrap();
        assert_eq!(stage, WorkflowStage::Approved);

        // Approved -> Scheduled
        let stage = engine
            .transition(campaign, ApprovalAction::Schedule, actor, "planner", None)
            .unwrap();
        assert_eq!(stage, WorkflowStage::Scheduled);

        // Scheduled -> Live
        let stage = engine
            .transition(campaign, ApprovalAction::GoLive, actor, "ops", None)
            .unwrap();
        assert_eq!(stage, WorkflowStage::Live);

        // Live -> Completed
        let stage = engine
            .transition(campaign, ApprovalAction::Complete, actor, "ops", None)
            .unwrap();
        assert_eq!(stage, WorkflowStage::Completed);

        // Verify history length
        let history = engine.get_history(&campaign);
        assert_eq!(history.len(), 5);
    }

    // -----------------------------------------------------------------------
    // 2. Rejection flow
    // -----------------------------------------------------------------------
    #[test]
    fn test_rejection_and_resubmit() {
        let engine = WorkflowEngine::new();
        let campaign = Uuid::new_v4();
        let actor = Uuid::new_v4();

        engine.register_campaign(campaign);

        // Draft -> InReview
        engine
            .transition(campaign, ApprovalAction::Submit, actor, "editor", None)
            .unwrap();

        // InReview -> Rejected
        let stage = engine
            .transition(
                campaign,
                ApprovalAction::Reject,
                actor,
                "manager",
                Some("Needs more work".to_string()),
            )
            .unwrap();
        assert_eq!(stage, WorkflowStage::Rejected);

        // Rejected campaigns go back to Draft via RequestChanges is not valid
        // from Rejected. The typical flow is: the stage is set to Draft
        // manually or re-registered. Let's confirm that resubmitting from
        // Rejected is invalid and that the correct path is to re-register.
        // Actually, per the spec, RequestChanges goes InReview -> Draft.
        // After rejection the campaign stays Rejected. Let's re-register to
        // simulate a "resubmit" by resetting to Draft.
        engine.stages.insert(campaign, WorkflowStage::Draft);

        // Draft -> InReview again
        let stage = engine
            .transition(campaign, ApprovalAction::Submit, actor, "editor", None)
            .unwrap();
        assert_eq!(stage, WorkflowStage::InReview);

        let history = engine.get_history(&campaign);
        assert_eq!(history.len(), 3);
    }

    // -----------------------------------------------------------------------
    // 3. Invalid transition
    // -----------------------------------------------------------------------
    #[test]
    fn test_invalid_transition() {
        let engine = WorkflowEngine::new();
        let campaign = Uuid::new_v4();
        let actor = Uuid::new_v4();

        engine.register_campaign(campaign);

        // Draft -> Live should fail
        let result = engine.transition(campaign, ApprovalAction::GoLive, actor, "ops", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid transition"));

        // Stage should still be Draft
        assert_eq!(engine.get_stage(&campaign), Some(WorkflowStage::Draft));
    }

    // -----------------------------------------------------------------------
    // 4. Approval request flow with multiple approvers
    // -----------------------------------------------------------------------
    #[test]
    fn test_approval_request_multiple_approvers() {
        let engine = WorkflowEngine::new();
        engine.seed_default_rules();

        let campaign = Uuid::new_v4();
        let requester = Uuid::new_v4();
        let approver_a = Uuid::new_v4();
        let approver_b = Uuid::new_v4();

        engine.register_campaign(campaign);

        // Move to InReview first
        engine
            .transition(campaign, ApprovalAction::Submit, requester, "editor", None)
            .unwrap();

        // Submit for approval with two approvers
        let request = engine.submit_for_approval(
            campaign,
            requester,
            vec![
                (approver_a, "manager".to_string()),
                (approver_b, "director".to_string()),
            ],
        );
        assert_eq!(request.status, ApprovalStatus::Pending);
        assert_eq!(request.approvers.len(), 2);

        // Verify pending approvals
        let pending_a = engine.get_pending_approvals(&approver_a);
        assert_eq!(pending_a.len(), 1);
        let pending_b = engine.get_pending_approvals(&approver_b);
        assert_eq!(pending_b.len(), 1);

        // First approver approves — rule requires min 1, so this should resolve.
        let updated = engine
            .record_approval_decision(request.id, approver_a, true, Some("LGTM".to_string()))
            .unwrap();
        assert_eq!(updated.status, ApprovalStatus::Approved);

        // Campaign should now be Approved
        assert_eq!(engine.get_stage(&campaign), Some(WorkflowStage::Approved));

        // No more pending requests for approver B (request resolved)
        let pending_b = engine.get_pending_approvals(&approver_b);
        assert_eq!(pending_b.len(), 0);
    }

    // -----------------------------------------------------------------------
    // 5. Calendar range queries
    // -----------------------------------------------------------------------
    #[test]
    fn test_calendar_range_queries() {
        let calendar = CampaignCalendar::new();
        let campaign = Uuid::new_v4();

        let now = Utc::now();

        // Event in the past
        let past_event = CalendarEvent {
            id: Uuid::new_v4(),
            campaign_id: campaign,
            campaign_name: "Past Campaign".to_string(),
            event_type: CalendarEventType::End,
            start_date: now - Duration::days(30),
            end_date: None,
            color: "#888888".to_string(),
            notes: None,
        };
        calendar.add_event(past_event);

        // Event today
        let today_event = CalendarEvent {
            id: Uuid::new_v4(),
            campaign_id: campaign,
            campaign_name: "Current Campaign".to_string(),
            event_type: CalendarEventType::Launch,
            start_date: now + Duration::hours(1),
            end_date: Some(now + Duration::days(14)),
            color: "#FF5733".to_string(),
            notes: Some("Go-live!".to_string()),
        };
        let today_id = today_event.id;
        calendar.add_event(today_event);

        // Event 5 days from now
        let soon_event = CalendarEvent {
            id: Uuid::new_v4(),
            campaign_id: campaign,
            campaign_name: "Upcoming Review".to_string(),
            event_type: CalendarEventType::Review,
            start_date: now + Duration::days(5),
            end_date: None,
            color: "#33FF57".to_string(),
            notes: None,
        };
        calendar.add_event(soon_event);

        // Event 60 days from now
        let far_event = CalendarEvent {
            id: Uuid::new_v4(),
            campaign_id: Uuid::new_v4(), // different campaign
            campaign_name: "Future Milestone".to_string(),
            event_type: CalendarEventType::Milestone,
            start_date: now + Duration::days(60),
            end_date: None,
            color: "#3357FF".to_string(),
            notes: None,
        };
        calendar.add_event(far_event);

        // Range query: next 7 days should include today_event and soon_event
        let next_week = calendar.get_events_in_range(now, now + Duration::days(7));
        assert_eq!(next_week.len(), 2);

        // Campaign events for our campaign should be 3 (past + today + soon)
        let campaign_events = calendar.get_campaign_events(&campaign);
        assert_eq!(campaign_events.len(), 3);

        // Upcoming 10 days should include today_event and soon_event
        let upcoming = calendar.get_upcoming(10);
        assert_eq!(upcoming.len(), 2);

        // Remove today's event
        assert!(calendar.remove_event(&today_id));
        assert!(!calendar.remove_event(&today_id)); // already gone

        // Now upcoming 10 days = 1
        let upcoming = calendar.get_upcoming(10);
        assert_eq!(upcoming.len(), 1);
    }
}
