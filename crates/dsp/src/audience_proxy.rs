//! Paid media audience proxy — segment proxy definitions, incremental
//! audience sync, creative export to DSPs, and match-rate estimation.
//!
//! Addresses FR-PAID-PROXY-001 through FR-PAID-PROXY-005.

use chrono::{DateTime, Timelike, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;

// ─── Segment Proxy (FR-PAID-PROXY-001) ──────────────────────────────

/// A segment proxy maps an internal segment to an external DSP audience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentProxy {
    pub proxy_id: String,
    pub internal_segment_id: u32,
    pub segment_name: String,
    pub dsp_platform: DspTarget,
    pub external_audience_id: Option<String>,
    pub status: ProxyStatus,
    pub member_count: u64,
    pub last_synced: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Target DSP platform for segment proxy.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DspTarget {
    GoogleDv360,
    MetaAds,
    TheTradeDesk,
    AmazonDsp,
}

impl DspTarget {
    pub fn display_name(&self) -> &'static str {
        match self {
            DspTarget::GoogleDv360 => "Google DV360",
            DspTarget::MetaAds => "Meta Ads",
            DspTarget::TheTradeDesk => "The Trade Desk",
            DspTarget::AmazonDsp => "Amazon DSP",
        }
    }
}

/// Status of a segment proxy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyStatus {
    Pending,
    Active,
    Syncing,
    Error,
    Paused,
}

// ─── Incremental Audience Sync (FR-PAID-PROXY-002) ──────────────────

/// Result of an incremental audience sync.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudienceSyncResult {
    pub proxy_id: String,
    pub dsp_platform: DspTarget,
    pub users_added: u64,
    pub users_removed: u64,
    pub users_unchanged: u64,
    pub total_synced: u64,
    pub sync_duration_ms: u64,
    pub errors: Vec<String>,
    pub synced_at: DateTime<Utc>,
}

/// Audience sync delta — tracks which users to add/remove.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudienceDelta {
    pub segment_id: u32,
    pub additions: Vec<String>,
    pub removals: Vec<String>,
    pub computed_at: DateTime<Utc>,
}

// ─── Creative Export (FR-PAID-PROXY-003) ─────────────────────────────

/// A creative asset exported to a DSP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreativeExportEntry {
    pub export_id: String,
    pub creative_id: String,
    pub dsp_platform: DspTarget,
    pub external_creative_id: Option<String>,
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub file_size_bytes: u64,
    pub status: ExportStatus,
    pub exported_at: DateTime<Utc>,
}

/// Status of a creative export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportStatus {
    Pending,
    Uploaded,
    Approved,
    Rejected,
    Failed,
}

// ─── Match-Rate Estimation (FR-PAID-PROXY-004) ──────────────────────

/// Estimated match rate between our audience and a DSP's user base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRateEstimate {
    pub proxy_id: String,
    pub dsp_platform: DspTarget,
    pub our_audience_size: u64,
    pub estimated_matched: u64,
    pub match_rate_percent: f64,
    pub confidence: MatchRateConfidence,
    pub estimated_at: DateTime<Utc>,
}

/// Confidence level of match rate estimate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchRateConfidence {
    Low,
    Medium,
    High,
}

// ─── Budget Pacing (FR-PAID-PROXY-005) ───────────────────────────────

/// Budget pacing status for a DSP campaign.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetPacing {
    pub campaign_id: String,
    pub dsp_platform: DspTarget,
    pub daily_budget: f64,
    pub spent_today: f64,
    pub pacing_percent: f64,
    pub projected_daily_spend: f64,
    pub status: PacingStatus,
    pub updated_at: DateTime<Utc>,
}

/// Pacing status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PacingStatus {
    OnTrack,
    Underpacing,
    Overpacing,
    Exhausted,
}

// ─── Audience Proxy Engine ───────────────────────────────────────────

/// Engine for managing segment proxies, audience sync, and creative export.
pub struct AudienceProxyEngine {
    proxies: DashMap<String, SegmentProxy>,
    sync_history: DashMap<String, Vec<AudienceSyncResult>>,
    creative_exports: DashMap<String, CreativeExportEntry>,
    budget_pacing: DashMap<String, BudgetPacing>,
}

impl AudienceProxyEngine {
    pub fn new() -> Self {
        info!("Audience proxy engine initialized");
        Self {
            proxies: DashMap::new(),
            sync_history: DashMap::new(),
            creative_exports: DashMap::new(),
            budget_pacing: DashMap::new(),
        }
    }

    /// Create a segment proxy mapping.
    pub fn create_proxy(
        &self,
        internal_segment_id: u32,
        segment_name: &str,
        dsp_platform: DspTarget,
    ) -> SegmentProxy {
        let proxy = SegmentProxy {
            proxy_id: format!(
                "proxy_{}_{:?}_{}",
                internal_segment_id,
                dsp_platform,
                Utc::now().timestamp_millis()
            ),
            internal_segment_id,
            segment_name: segment_name.to_string(),
            dsp_platform,
            external_audience_id: None,
            status: ProxyStatus::Pending,
            member_count: 0,
            last_synced: None,
            created_at: Utc::now(),
        };
        self.proxies.insert(proxy.proxy_id.clone(), proxy.clone());
        proxy
    }

    /// Perform an incremental audience sync for a proxy.
    pub fn sync_audience(
        &self,
        proxy_id: &str,
        delta: AudienceDelta,
    ) -> Option<AudienceSyncResult> {
        let mut proxy = self.proxies.get_mut(proxy_id)?;
        proxy.status = ProxyStatus::Syncing;

        let added = delta.additions.len() as u64;
        let removed = delta.removals.len() as u64;
        let previous_count = proxy.member_count;

        // Apply delta
        proxy.member_count = (previous_count + added).saturating_sub(removed);
        proxy.last_synced = Some(Utc::now());
        proxy.status = ProxyStatus::Active;

        let result = AudienceSyncResult {
            proxy_id: proxy_id.to_string(),
            dsp_platform: proxy.dsp_platform.clone(),
            users_added: added,
            users_removed: removed,
            users_unchanged: previous_count.saturating_sub(removed),
            total_synced: proxy.member_count,
            sync_duration_ms: 150,
            errors: Vec::new(),
            synced_at: Utc::now(),
        };

        self.sync_history
            .entry(proxy_id.to_string())
            .or_default()
            .push(result.clone());

        Some(result)
    }

    /// Export a creative to a DSP.
    pub fn export_creative(
        &self,
        creative_id: &str,
        dsp_platform: DspTarget,
        format: &str,
        width: u32,
        height: u32,
        file_size_bytes: u64,
    ) -> CreativeExportEntry {
        let entry = CreativeExportEntry {
            export_id: format!("exp_{}_{}", creative_id, Utc::now().timestamp_millis()),
            creative_id: creative_id.to_string(),
            dsp_platform,
            external_creative_id: None,
            format: format.to_string(),
            width,
            height,
            file_size_bytes,
            status: ExportStatus::Pending,
            exported_at: Utc::now(),
        };
        self.creative_exports
            .insert(entry.export_id.clone(), entry.clone());
        entry
    }

    /// Confirm a creative export was accepted by the DSP.
    pub fn confirm_export(
        &self,
        export_id: &str,
        external_id: &str,
    ) -> Result<CreativeExportEntry, String> {
        let mut entry = self
            .creative_exports
            .get_mut(export_id)
            .ok_or("Export not found")?;
        entry.external_creative_id = Some(external_id.to_string());
        entry.status = ExportStatus::Uploaded;
        Ok(entry.clone())
    }

    /// Estimate match rate for a segment proxy.
    pub fn estimate_match_rate(
        &self,
        proxy_id: &str,
        historical_match_rates: &[(DspTarget, f64)],
    ) -> Option<MatchRateEstimate> {
        let proxy = self.proxies.get(proxy_id)?;

        // Use historical match rate for this DSP, or default estimate
        let platform_rate = historical_match_rates
            .iter()
            .find(|(p, _)| *p == proxy.dsp_platform)
            .map(|(_, r)| *r)
            .unwrap_or(0.6);

        let estimated_matched = (proxy.member_count as f64 * platform_rate) as u64;

        let confidence = if proxy.member_count > 100_000 {
            MatchRateConfidence::High
        } else if proxy.member_count > 10_000 {
            MatchRateConfidence::Medium
        } else {
            MatchRateConfidence::Low
        };

        Some(MatchRateEstimate {
            proxy_id: proxy_id.to_string(),
            dsp_platform: proxy.dsp_platform.clone(),
            our_audience_size: proxy.member_count,
            estimated_matched,
            match_rate_percent: platform_rate * 100.0,
            confidence,
            estimated_at: Utc::now(),
        })
    }

    /// Update budget pacing for a DSP campaign.
    pub fn update_pacing(
        &self,
        campaign_id: &str,
        dsp_platform: DspTarget,
        daily_budget: f64,
        spent_today: f64,
    ) -> BudgetPacing {
        let hours_elapsed = Utc::now().time().hour() as f64 + 1.0;
        let hourly_rate = if hours_elapsed > 0.0 {
            spent_today / hours_elapsed
        } else {
            0.0
        };
        let projected = hourly_rate * 24.0;
        let pacing_percent = if daily_budget > 0.0 {
            spent_today / daily_budget * 100.0
        } else {
            0.0
        };

        let expected_pacing = (hours_elapsed / 24.0) * 100.0;
        let status = if pacing_percent >= 100.0 {
            PacingStatus::Exhausted
        } else if pacing_percent > expected_pacing * 1.2 {
            PacingStatus::Overpacing
        } else if pacing_percent < expected_pacing * 0.8 {
            PacingStatus::Underpacing
        } else {
            PacingStatus::OnTrack
        };

        let pacing = BudgetPacing {
            campaign_id: campaign_id.to_string(),
            dsp_platform,
            daily_budget,
            spent_today,
            pacing_percent,
            projected_daily_spend: projected,
            status,
            updated_at: Utc::now(),
        };

        self.budget_pacing
            .insert(campaign_id.to_string(), pacing.clone());
        pacing
    }

    /// Get all proxies for a segment.
    pub fn proxies_for_segment(&self, segment_id: u32) -> Vec<SegmentProxy> {
        self.proxies
            .iter()
            .filter(|e| e.value().internal_segment_id == segment_id)
            .map(|e| e.value().clone())
            .collect()
    }

    /// Get sync history for a proxy.
    pub fn sync_history(&self, proxy_id: &str) -> Vec<AudienceSyncResult> {
        self.sync_history
            .get(proxy_id)
            .map(|v| v.clone())
            .unwrap_or_default()
    }

    /// List all proxies.
    pub fn list_proxies(&self) -> Vec<SegmentProxy> {
        self.proxies.iter().map(|e| e.value().clone()).collect()
    }
}

impl Default for AudienceProxyEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_segment_proxy() {
        let engine = AudienceProxyEngine::new();
        let proxy = engine.create_proxy(42, "High Value Users", DspTarget::MetaAds);

        assert_eq!(proxy.internal_segment_id, 42);
        assert_eq!(proxy.status, ProxyStatus::Pending);
        assert_eq!(proxy.member_count, 0);
    }

    #[test]
    fn test_incremental_sync() {
        let engine = AudienceProxyEngine::new();
        let proxy = engine.create_proxy(1, "Test Segment", DspTarget::GoogleDv360);

        let delta = AudienceDelta {
            segment_id: 1,
            additions: vec!["u1".into(), "u2".into(), "u3".into()],
            removals: vec![],
            computed_at: Utc::now(),
        };

        let result = engine.sync_audience(&proxy.proxy_id, delta).unwrap();
        assert_eq!(result.users_added, 3);
        assert_eq!(result.users_removed, 0);
        assert_eq!(result.total_synced, 3);

        // Second sync with removals
        let delta2 = AudienceDelta {
            segment_id: 1,
            additions: vec!["u4".into()],
            removals: vec!["u1".into()],
            computed_at: Utc::now(),
        };

        let result2 = engine.sync_audience(&proxy.proxy_id, delta2).unwrap();
        assert_eq!(result2.users_added, 1);
        assert_eq!(result2.users_removed, 1);
        assert_eq!(result2.total_synced, 3);

        let history = engine.sync_history(&proxy.proxy_id);
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_creative_export() {
        let engine = AudienceProxyEngine::new();
        let export = engine.export_creative(
            "creative_123",
            DspTarget::TheTradeDesk,
            "image/png",
            728,
            90,
            45000,
        );

        assert_eq!(export.status, ExportStatus::Pending);
        assert_eq!(export.width, 728);

        // Confirm export
        let confirmed = engine
            .confirm_export(&export.export_id, "ttd_ext_456")
            .unwrap();
        assert_eq!(confirmed.status, ExportStatus::Uploaded);
        assert_eq!(
            confirmed.external_creative_id.as_deref(),
            Some("ttd_ext_456")
        );
    }

    #[test]
    fn test_match_rate_estimation() {
        let engine = AudienceProxyEngine::new();
        let proxy = engine.create_proxy(10, "Lookalike Audience", DspTarget::MetaAds);

        // Set member count
        {
            let mut p = engine.proxies.get_mut(&proxy.proxy_id).unwrap();
            p.member_count = 50_000;
        }

        let rates = vec![(DspTarget::MetaAds, 0.75), (DspTarget::GoogleDv360, 0.60)];

        let estimate = engine.estimate_match_rate(&proxy.proxy_id, &rates).unwrap();
        assert_eq!(estimate.match_rate_percent, 75.0);
        assert_eq!(estimate.estimated_matched, 37_500);
        assert_eq!(estimate.confidence, MatchRateConfidence::Medium);
    }

    #[test]
    fn test_budget_pacing() {
        let engine = AudienceProxyEngine::new();
        let pacing = engine.update_pacing("camp_1", DspTarget::AmazonDsp, 1000.0, 100.0);

        assert_eq!(pacing.daily_budget, 1000.0);
        assert_eq!(pacing.spent_today, 100.0);
        assert!(pacing.projected_daily_spend > 0.0);
    }

    #[test]
    fn test_proxies_for_segment() {
        let engine = AudienceProxyEngine::new();
        engine.create_proxy(5, "Seg 5", DspTarget::MetaAds);
        engine.create_proxy(5, "Seg 5", DspTarget::GoogleDv360);
        engine.create_proxy(6, "Seg 6", DspTarget::MetaAds);

        let proxies = engine.proxies_for_segment(5);
        assert_eq!(proxies.len(), 2);
    }
}
