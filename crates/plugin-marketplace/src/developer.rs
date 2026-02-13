//! Developer portal — registration, plugin publishing, analytics, and monetization.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperAccount {
    pub id: Uuid,
    pub name: String,
    pub company: Option<String>,
    pub email: String,
    pub website: Option<String>,
    pub support_email: String,
    pub api_key: String,
    pub is_verified: bool,
    pub plugins_published: u32,
    pub total_installs: u64,
    pub total_revenue: f64,
    pub created_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperRegistration {
    pub name: String,
    pub company: Option<String>,
    pub email: String,
    pub website: Option<String>,
    pub support_email: String,
    pub accepted_tos: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperAnalytics {
    pub developer_id: Uuid,
    pub total_plugins: u32,
    pub total_installs: u64,
    pub total_active_installs: u64,
    pub total_revenue: f64,
    pub average_rating: f64,
    pub total_reviews: u64,
    pub monthly_revenue: f64,
    pub top_plugin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewDecision {
    Approved,
    Rejected { reason: String },
    NeedsChanges { feedback: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSubmission {
    pub id: Uuid,
    pub developer_id: Uuid,
    pub plugin_slug: String,
    pub version: String,
    pub package_url: String,
    pub status: SubmissionStatus,
    pub reviewer_notes: Option<String>,
    pub submitted_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionStatus {
    PendingReview,
    InReview,
    Approved,
    Rejected,
    NeedsChanges,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutRecord {
    pub id: Uuid,
    pub developer_id: Uuid,
    pub amount: f64,
    pub currency: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub status: PayoutStatus,
    pub paid_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayoutStatus {
    Pending,
    Processing,
    Paid,
    Failed,
}

pub struct DeveloperPortal {
    accounts: dashmap::DashMap<Uuid, DeveloperAccount>,
    submissions: dashmap::DashMap<Uuid, Vec<PluginSubmission>>,
    payouts: dashmap::DashMap<Uuid, Vec<PayoutRecord>>,
}

impl DeveloperPortal {
    pub fn new() -> Self {
        Self {
            accounts: dashmap::DashMap::new(),
            submissions: dashmap::DashMap::new(),
            payouts: dashmap::DashMap::new(),
        }
    }

    pub fn register(&self, reg: DeveloperRegistration) -> anyhow::Result<DeveloperAccount> {
        if !reg.accepted_tos {
            anyhow::bail!("Must accept Terms of Service");
        }

        let account = DeveloperAccount {
            id: Uuid::new_v4(),
            name: reg.name,
            company: reg.company,
            email: reg.email,
            website: reg.website,
            support_email: reg.support_email,
            api_key: format!("ce_dev_{}", Uuid::new_v4().to_string().replace('-', "")),
            is_verified: false,
            plugins_published: 0,
            total_installs: 0,
            total_revenue: 0.0,
            created_at: Utc::now(),
            verified_at: None,
        };

        self.accounts.insert(account.id, account.clone());
        Ok(account)
    }

    pub fn get_account(&self, id: &Uuid) -> Option<DeveloperAccount> {
        self.accounts.get(id).map(|a| a.clone())
    }

    pub fn submit_plugin(&self, submission: PluginSubmission) {
        self.submissions
            .entry(submission.developer_id)
            .or_default()
            .push(submission);
    }

    pub fn get_submissions(&self, developer_id: &Uuid) -> Vec<PluginSubmission> {
        self.submissions
            .get(developer_id)
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    pub fn get_analytics(&self, developer_id: &Uuid) -> DeveloperAnalytics {
        let account = self.accounts.get(developer_id);
        DeveloperAnalytics {
            developer_id: *developer_id,
            total_plugins: account.as_ref().map(|a| a.plugins_published).unwrap_or(0),
            total_installs: account.as_ref().map(|a| a.total_installs).unwrap_or(0),
            total_active_installs: 0,
            total_revenue: account.as_ref().map(|a| a.total_revenue).unwrap_or(0.0),
            average_rating: 4.5,
            total_reviews: 0,
            monthly_revenue: 0.0,
            top_plugin: None,
        }
    }

    pub fn get_payouts(&self, developer_id: &Uuid) -> Vec<PayoutRecord> {
        self.payouts
            .get(developer_id)
            .map(|p| p.clone())
            .unwrap_or_default()
    }
}

impl Default for DeveloperPortal {
    fn default() -> Self {
        Self::new()
    }
}
