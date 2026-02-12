//! Onboarding engine — guided step-by-step flows for new tenants plus a
//! library of campaign templates to accelerate first-launch.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Current status of an individual onboarding step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingStatus {
    NotStarted,
    InProgress,
    Completed,
    Skipped,
}

/// A single step within the onboarding checklist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: OnboardingStatus,
    pub order: u32,
    pub required: bool,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Aggregate progress tracker for a tenant's onboarding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingProgress {
    pub tenant_id: Uuid,
    pub steps: Vec<OnboardingStep>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub completion_percent: f64,
}

/// A reusable campaign template offered during onboarding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub config: serde_json::Value,
    pub popularity: u32,
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// In-memory onboarding engine backed by `DashMap`.
pub struct OnboardingEngine {
    progress: Arc<DashMap<Uuid, OnboardingProgress>>,
    templates: Arc<DashMap<Uuid, CampaignTemplate>>,
}

impl Default for OnboardingEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl OnboardingEngine {
    /// Create a new onboarding engine.
    pub fn new() -> Self {
        info!("OnboardingEngine initialized");
        Self {
            progress: Arc::new(DashMap::new()),
            templates: Arc::new(DashMap::new()),
        }
    }

    /// Start the onboarding flow for a tenant, creating default steps.
    pub fn start_onboarding(&self, tenant_id: Uuid) -> OnboardingProgress {
        let steps = vec![
            OnboardingStep {
                id: "account_setup".into(),
                title: "Account Setup".into(),
                description: "Configure your organization name, logo, and timezone".into(),
                status: OnboardingStatus::InProgress,
                order: 1,
                required: true,
                completed_at: None,
            },
            OnboardingStep {
                id: "team_invite".into(),
                title: "Invite Team Members".into(),
                description: "Add colleagues and assign roles".into(),
                status: OnboardingStatus::NotStarted,
                order: 2,
                required: false,
                completed_at: None,
            },
            OnboardingStep {
                id: "first_campaign".into(),
                title: "Create First Campaign".into(),
                description: "Launch your first ad campaign using a template or from scratch".into(),
                status: OnboardingStatus::NotStarted,
                order: 3,
                required: true,
                completed_at: None,
            },
            OnboardingStep {
                id: "connect_dsp".into(),
                title: "Connect DSP".into(),
                description: "Integrate with your demand-side platform for programmatic buying"
                    .into(),
                status: OnboardingStatus::NotStarted,
                order: 4,
                required: true,
                completed_at: None,
            },
            OnboardingStep {
                id: "configure_channels".into(),
                title: "Configure Channels".into(),
                description: "Set up delivery channels: display, email, push, SMS".into(),
                status: OnboardingStatus::NotStarted,
                order: 5,
                required: true,
                completed_at: None,
            },
            OnboardingStep {
                id: "install_pixel".into(),
                title: "Install Tracking Pixel".into(),
                description: "Add the CampaignExpress pixel to your website for conversion tracking".into(),
                status: OnboardingStatus::NotStarted,
                order: 6,
                required: true,
                completed_at: None,
            },
            OnboardingStep {
                id: "launch_campaign".into(),
                title: "Launch Campaign".into(),
                description: "Review and launch your first campaign to start serving offers".into(),
                status: OnboardingStatus::NotStarted,
                order: 7,
                required: true,
                completed_at: None,
            },
        ];

        let progress = OnboardingProgress {
            tenant_id,
            steps,
            started_at: Utc::now(),
            completed_at: None,
            completion_percent: 0.0,
        };

        self.progress.insert(tenant_id, progress.clone());
        progress
    }

    /// Mark a step as completed and recalculate the overall completion percent.
    pub fn complete_step(
        &self,
        tenant_id: Uuid,
        step_id: &str,
    ) -> Option<OnboardingProgress> {
        self.progress.get_mut(&tenant_id).map(|mut prog| {
            let now = Utc::now();

            for step in &mut prog.steps {
                if step.id == step_id && step.status != OnboardingStatus::Completed {
                    step.status = OnboardingStatus::Completed;
                    step.completed_at = Some(now);
                }
            }

            // Recalculate completion
            let total = prog.steps.len() as f64;
            let done = prog
                .steps
                .iter()
                .filter(|s| {
                    s.status == OnboardingStatus::Completed
                        || s.status == OnboardingStatus::Skipped
                })
                .count() as f64;
            prog.completion_percent = if total > 0.0 {
                (done / total) * 100.0
            } else {
                0.0
            };

            // Mark overall complete if all required steps are done
            let all_required_done = prog
                .steps
                .iter()
                .filter(|s| s.required)
                .all(|s| {
                    s.status == OnboardingStatus::Completed
                        || s.status == OnboardingStatus::Skipped
                });
            if all_required_done && prog.completed_at.is_none() {
                prog.completed_at = Some(now);
            }

            prog.clone()
        })
    }

    /// Get a tenant's onboarding progress.
    pub fn get_progress(&self, tenant_id: Uuid) -> Option<OnboardingProgress> {
        self.progress.get(&tenant_id).map(|p| p.clone())
    }

    /// List all campaign templates.
    pub fn list_templates(&self) -> Vec<CampaignTemplate> {
        self.templates.iter().map(|e| e.value().clone()).collect()
    }

    /// Seed the template library with 5 starter templates.
    pub fn seed_templates(&self) {
        let templates = vec![
            CampaignTemplate {
                id: Uuid::new_v4(),
                name: "E-commerce Retargeting".into(),
                description: "Re-engage shoppers who abandoned carts with personalized product ads"
                    .into(),
                category: "retargeting".into(),
                config: serde_json::json!({
                    "objective": "conversions",
                    "channels": ["display", "email"],
                    "audience": "cart_abandoners",
                    "frequency_cap": 3,
                    "lookback_days": 7
                }),
                popularity: 95,
            },
            CampaignTemplate {
                id: Uuid::new_v4(),
                name: "Brand Awareness".into(),
                description:
                    "Broad reach campaign to build brand recognition with new audiences".into(),
                category: "awareness".into(),
                config: serde_json::json!({
                    "objective": "reach",
                    "channels": ["display", "video"],
                    "audience": "prospecting",
                    "frequency_cap": 5,
                    "lookback_days": 30
                }),
                popularity: 80,
            },
            CampaignTemplate {
                id: Uuid::new_v4(),
                name: "Lead Generation".into(),
                description: "Capture qualified leads through targeted form-fill campaigns".into(),
                category: "lead_gen".into(),
                config: serde_json::json!({
                    "objective": "leads",
                    "channels": ["display", "email", "push"],
                    "audience": "in_market",
                    "frequency_cap": 4,
                    "lookback_days": 14
                }),
                popularity: 75,
            },
            CampaignTemplate {
                id: Uuid::new_v4(),
                name: "App Install".into(),
                description:
                    "Drive mobile app installations with optimized creative and deep links".into(),
                category: "app_install".into(),
                config: serde_json::json!({
                    "objective": "installs",
                    "channels": ["display", "push"],
                    "audience": "mobile_users",
                    "frequency_cap": 2,
                    "lookback_days": 7
                }),
                popularity: 70,
            },
            CampaignTemplate {
                id: Uuid::new_v4(),
                name: "Loyalty Re-engagement".into(),
                description: "Win back lapsed loyalty members with personalized incentives".into(),
                category: "retention".into(),
                config: serde_json::json!({
                    "objective": "retention",
                    "channels": ["email", "push", "sms"],
                    "audience": "lapsed_members",
                    "frequency_cap": 2,
                    "lookback_days": 90
                }),
                popularity: 65,
            },
        ];

        for tmpl in templates {
            self.templates.insert(tmpl.id, tmpl);
        }

        info!("Seeded 5 campaign templates");
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_onboarding_flow() {
        let engine = OnboardingEngine::new();
        let tenant = Uuid::new_v4();

        let progress = engine.start_onboarding(tenant);
        assert_eq!(progress.steps.len(), 7);
        assert_eq!(progress.completion_percent, 0.0);
        assert!(progress.completed_at.is_none());

        // First step should be InProgress
        assert_eq!(progress.steps[0].status, OnboardingStatus::InProgress);
        assert_eq!(progress.steps[0].id, "account_setup");

        // Remaining steps NotStarted
        for step in &progress.steps[1..] {
            assert_eq!(step.status, OnboardingStatus::NotStarted);
        }

        // Seed and list templates
        engine.seed_templates();
        let templates = engine.list_templates();
        assert_eq!(templates.len(), 5);
    }

    #[test]
    fn test_complete_step() {
        let engine = OnboardingEngine::new();
        let tenant = Uuid::new_v4();

        engine.start_onboarding(tenant);

        // Complete account_setup
        let progress = engine.complete_step(tenant, "account_setup").unwrap();
        let step = progress.steps.iter().find(|s| s.id == "account_setup").unwrap();
        assert_eq!(step.status, OnboardingStatus::Completed);
        assert!(step.completed_at.is_some());
        assert!(progress.completion_percent > 0.0);

        // Complete all required steps
        let required_ids: Vec<String> = progress
            .steps
            .iter()
            .filter(|s| s.required && s.status != OnboardingStatus::Completed)
            .map(|s| s.id.clone())
            .collect();
        for id in &required_ids {
            engine.complete_step(tenant, id);
        }

        let final_progress = engine.get_progress(tenant).unwrap();
        // All required done -> completed_at should be set
        assert!(final_progress.completed_at.is_some());
    }
}
