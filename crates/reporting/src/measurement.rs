//! Unified measurement — standardized event schema, cross-channel reporting
//! breakdowns, and unified experimentation integration.
//!
//! Addresses FR-MSR-UNI-001 through FR-MSR-UNI-003.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

// ─── Standardized Event Schema (FR-MSR-UNI-001) ─────────────────────

/// A standardized measurement event with consistent structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasurementEvent {
    pub event_id: Uuid,
    pub event_type: MeasurementEventType,
    pub source: EventSource,
    pub timestamp: DateTime<Utc>,
    /// Unique activation/decision that originated this event chain.
    pub activation_id: Option<String>,
    pub decision_id: Option<String>,
    pub campaign_id: Option<String>,
    pub experiment_id: Option<Uuid>,
    pub variant_id: Option<Uuid>,
    pub user_id: Option<String>,
    pub channel: Option<String>,
    pub dimensions: std::collections::HashMap<String, String>,
    pub metrics: std::collections::HashMap<String, f64>,
}

/// Standardized event types across all channels.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementEventType {
    /// Content was delivered to the user.
    Delivered,
    /// User viewed the content (impression/open).
    Viewed,
    /// User clicked on a CTA.
    Clicked,
    /// User completed the desired action.
    Converted,
    /// User bounced/unsubscribed.
    Bounced,
    /// Revenue attributed to this touchpoint.
    Revenue,
    /// Content was suppressed by a rule.
    Suppressed,
    /// Experiment assignment event.
    ExperimentAssigned,
    /// Custom event.
    Custom(String),
}

/// Where the event was generated.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventSource {
    BidEngine,
    JourneyEngine,
    DirectActivation,
    ExternalWebhook,
    ClientSdk,
}

// ─── Cross-Channel Reporting (FR-MSR-UNI-002) ───────────────────────

/// A reporting breakdown request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingBreakdown {
    pub name: String,
    pub group_by: Vec<BreakdownDimension>,
    pub metrics: Vec<ReportMetric>,
    pub filters: Vec<ReportFilter>,
    pub time_range: TimeRange,
}

/// Dimensions to break down by.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BreakdownDimension {
    Channel,
    Campaign,
    ActivationSource,
    Experiment,
    Variant,
    Segment,
    Region,
    DeviceType,
    DayOfWeek,
    HourOfDay,
}

/// Metrics to include in the report.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportMetric {
    Deliveries,
    Impressions,
    Clicks,
    Conversions,
    Revenue,
    Ctr,
    ConversionRate,
    CostPerConversion,
    Roas,
    UniqueReach,
}

/// A filter for the report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

/// Filter operators.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Equals,
    NotEquals,
    In,
    GreaterThan,
    LessThan,
    Between,
}

/// Time range for reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// A single row in the breakdown report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRow {
    pub dimensions: std::collections::HashMap<String, String>,
    pub metrics: std::collections::HashMap<String, f64>,
}

/// Full breakdown report result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakdownReport {
    pub name: String,
    pub rows: Vec<ReportRow>,
    pub totals: std::collections::HashMap<String, f64>,
    pub generated_at: DateTime<Utc>,
}

// ─── Unified Experimentation (FR-MSR-UNI-003) ───────────────────────

/// Experiment measurement linking decisions to outcomes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentMeasurement {
    pub experiment_id: Uuid,
    pub experiment_name: String,
    pub variants: Vec<VariantMeasurement>,
    pub start_date: DateTime<Utc>,
    pub end_date: Option<DateTime<Utc>>,
    pub status: ExperimentMeasurementStatus,
    pub winner: Option<Uuid>,
    pub confidence_level: f64,
}

/// Measurement data for one experiment variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantMeasurement {
    pub variant_id: Uuid,
    pub variant_name: String,
    pub is_control: bool,
    pub sample_size: u64,
    pub deliveries: u64,
    pub impressions: u64,
    pub clicks: u64,
    pub conversions: u64,
    pub revenue: f64,
    pub ctr: f64,
    pub conversion_rate: f64,
    pub lift_vs_control: Option<f64>,
}

/// Status of experiment measurement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExperimentMeasurementStatus {
    Collecting,
    SignificanceReached,
    Inconclusive,
    Completed,
}

// ─── Measurement Engine ──────────────────────────────────────────────

/// Unified measurement engine for event collection, reporting, and experimentation.
pub struct MeasurementEngine {
    events: DashMap<Uuid, MeasurementEvent>,
    experiments: DashMap<Uuid, ExperimentMeasurement>,
}

impl MeasurementEngine {
    pub fn new() -> Self {
        info!("Measurement engine initialized");
        Self {
            events: DashMap::new(),
            experiments: DashMap::new(),
        }
    }

    /// Record a standardized measurement event.
    pub fn record_event(&self, event: MeasurementEvent) {
        self.events.insert(event.event_id, event);
    }

    /// Create a convenience event with common fields.
    #[allow(clippy::too_many_arguments)]
    pub fn emit(
        &self,
        event_type: MeasurementEventType,
        source: EventSource,
        channel: &str,
        user_id: Option<&str>,
        activation_id: Option<&str>,
        campaign_id: Option<&str>,
        metrics: std::collections::HashMap<String, f64>,
    ) -> MeasurementEvent {
        let event = MeasurementEvent {
            event_id: Uuid::new_v4(),
            event_type,
            source,
            timestamp: Utc::now(),
            activation_id: activation_id.map(|s| s.to_string()),
            decision_id: None,
            campaign_id: campaign_id.map(|s| s.to_string()),
            experiment_id: None,
            variant_id: None,
            user_id: user_id.map(|s| s.to_string()),
            channel: Some(channel.to_string()),
            dimensions: std::collections::HashMap::new(),
            metrics,
        };
        self.events.insert(event.event_id, event.clone());
        event
    }

    /// Generate a cross-channel breakdown report.
    pub fn breakdown(&self, request: &ReportingBreakdown) -> BreakdownReport {
        // Collect matching events
        let matching: Vec<MeasurementEvent> = self
            .events
            .iter()
            .filter(|e| {
                let ev = e.value();
                ev.timestamp >= request.time_range.start && ev.timestamp <= request.time_range.end
            })
            .map(|e| e.value().clone())
            .collect();

        // Group by dimensions
        let mut groups: std::collections::HashMap<String, Vec<&MeasurementEvent>> =
            std::collections::HashMap::new();

        // Collect references from matching
        let event_refs: Vec<&MeasurementEvent> = matching.iter().collect();

        for event in &event_refs {
            let key = request
                .group_by
                .iter()
                .map(|dim| match dim {
                    BreakdownDimension::Channel => event.channel.clone().unwrap_or_default(),
                    BreakdownDimension::Campaign => event.campaign_id.clone().unwrap_or_default(),
                    BreakdownDimension::ActivationSource => {
                        format!("{:?}", event.source)
                    }
                    BreakdownDimension::Experiment => event
                        .experiment_id
                        .map(|id| id.to_string())
                        .unwrap_or_default(),
                    BreakdownDimension::Variant => event
                        .variant_id
                        .map(|id| id.to_string())
                        .unwrap_or_default(),
                    _ => event
                        .dimensions
                        .get(&format!("{:?}", dim))
                        .cloned()
                        .unwrap_or_default(),
                })
                .collect::<Vec<_>>()
                .join("|");

            groups.entry(key).or_default().push(event);
        }

        // Compute metrics for each group
        let mut rows = Vec::new();
        let mut totals: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

        for (key, events) in &groups {
            let mut row_metrics = std::collections::HashMap::new();
            let dims: Vec<&str> = key.split('|').collect();
            let mut row_dims = std::collections::HashMap::new();
            for (i, dim) in request.group_by.iter().enumerate() {
                row_dims.insert(format!("{:?}", dim), dims.get(i).unwrap_or(&"").to_string());
            }

            for metric in &request.metrics {
                let val = self.compute_metric(metric, events);
                row_metrics.insert(format!("{:?}", metric), val);
                *totals.entry(format!("{:?}", metric)).or_insert(0.0) += val;
            }

            rows.push(ReportRow {
                dimensions: row_dims,
                metrics: row_metrics,
            });
        }

        BreakdownReport {
            name: request.name.clone(),
            rows,
            totals,
            generated_at: Utc::now(),
        }
    }

    /// Compute a single metric from a set of events.
    fn compute_metric(&self, metric: &ReportMetric, events: &[&MeasurementEvent]) -> f64 {
        match metric {
            ReportMetric::Deliveries => events
                .iter()
                .filter(|e| e.event_type == MeasurementEventType::Delivered)
                .count() as f64,
            ReportMetric::Impressions => events
                .iter()
                .filter(|e| e.event_type == MeasurementEventType::Viewed)
                .count() as f64,
            ReportMetric::Clicks => events
                .iter()
                .filter(|e| e.event_type == MeasurementEventType::Clicked)
                .count() as f64,
            ReportMetric::Conversions => events
                .iter()
                .filter(|e| e.event_type == MeasurementEventType::Converted)
                .count() as f64,
            ReportMetric::Revenue => events
                .iter()
                .filter(|e| e.event_type == MeasurementEventType::Revenue)
                .filter_map(|e| e.metrics.get("revenue"))
                .sum(),
            ReportMetric::Ctr => {
                let impressions = events
                    .iter()
                    .filter(|e| e.event_type == MeasurementEventType::Viewed)
                    .count() as f64;
                let clicks = events
                    .iter()
                    .filter(|e| e.event_type == MeasurementEventType::Clicked)
                    .count() as f64;
                if impressions > 0.0 {
                    clicks / impressions * 100.0
                } else {
                    0.0
                }
            }
            ReportMetric::ConversionRate => {
                let clicks = events
                    .iter()
                    .filter(|e| e.event_type == MeasurementEventType::Clicked)
                    .count() as f64;
                let conversions = events
                    .iter()
                    .filter(|e| e.event_type == MeasurementEventType::Converted)
                    .count() as f64;
                if clicks > 0.0 {
                    conversions / clicks * 100.0
                } else {
                    0.0
                }
            }
            ReportMetric::UniqueReach => {
                let unique_users: std::collections::HashSet<_> =
                    events.iter().filter_map(|e| e.user_id.as_ref()).collect();
                unique_users.len() as f64
            }
            _ => 0.0,
        }
    }

    /// Register an experiment for unified measurement.
    pub fn register_experiment(
        &self,
        experiment_id: Uuid,
        name: &str,
        variant_names: Vec<(Uuid, String, bool)>,
    ) -> ExperimentMeasurement {
        let variants: Vec<VariantMeasurement> = variant_names
            .into_iter()
            .map(|(id, name, is_control)| VariantMeasurement {
                variant_id: id,
                variant_name: name,
                is_control,
                sample_size: 0,
                deliveries: 0,
                impressions: 0,
                clicks: 0,
                conversions: 0,
                revenue: 0.0,
                ctr: 0.0,
                conversion_rate: 0.0,
                lift_vs_control: None,
            })
            .collect();

        let measurement = ExperimentMeasurement {
            experiment_id,
            experiment_name: name.to_string(),
            variants,
            start_date: Utc::now(),
            end_date: None,
            status: ExperimentMeasurementStatus::Collecting,
            winner: None,
            confidence_level: 0.0,
        };

        self.experiments.insert(experiment_id, measurement.clone());
        measurement
    }

    /// Record an experiment event and update variant metrics.
    pub fn record_experiment_event(
        &self,
        experiment_id: &Uuid,
        variant_id: &Uuid,
        event_type: &MeasurementEventType,
        revenue: f64,
    ) {
        if let Some(mut exp) = self.experiments.get_mut(experiment_id) {
            for variant in &mut exp.variants {
                if variant.variant_id == *variant_id {
                    variant.sample_size += 1;
                    match event_type {
                        MeasurementEventType::Delivered => variant.deliveries += 1,
                        MeasurementEventType::Viewed => variant.impressions += 1,
                        MeasurementEventType::Clicked => variant.clicks += 1,
                        MeasurementEventType::Converted => {
                            variant.conversions += 1;
                            variant.revenue += revenue;
                        }
                        _ => {}
                    }

                    // Recompute rates
                    if variant.impressions > 0 {
                        variant.ctr = variant.clicks as f64 / variant.impressions as f64 * 100.0;
                    }
                    if variant.clicks > 0 {
                        variant.conversion_rate =
                            variant.conversions as f64 / variant.clicks as f64 * 100.0;
                    }
                    break;
                }
            }

            // Compute lift vs control
            let control_cvr = exp
                .variants
                .iter()
                .find(|v| v.is_control)
                .map(|v| v.conversion_rate)
                .unwrap_or(0.0);

            for variant in &mut exp.variants {
                if !variant.is_control && control_cvr > 0.0 {
                    variant.lift_vs_control =
                        Some((variant.conversion_rate - control_cvr) / control_cvr * 100.0);
                }
            }
        }
    }

    /// Get experiment measurement.
    pub fn get_experiment(&self, experiment_id: &Uuid) -> Option<ExperimentMeasurement> {
        self.experiments.get(experiment_id).map(|e| e.clone())
    }

    /// Get total event count.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}

impl Default for MeasurementEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_emit_and_count_events() {
        let engine = MeasurementEngine::new();

        engine.emit(
            MeasurementEventType::Delivered,
            EventSource::BidEngine,
            "email",
            Some("user_1"),
            Some("act_1"),
            Some("camp_1"),
            std::collections::HashMap::new(),
        );
        engine.emit(
            MeasurementEventType::Viewed,
            EventSource::BidEngine,
            "email",
            Some("user_1"),
            Some("act_1"),
            Some("camp_1"),
            std::collections::HashMap::new(),
        );
        engine.emit(
            MeasurementEventType::Clicked,
            EventSource::BidEngine,
            "email",
            Some("user_1"),
            Some("act_1"),
            Some("camp_1"),
            std::collections::HashMap::new(),
        );

        assert_eq!(engine.event_count(), 3);
    }

    #[test]
    fn test_cross_channel_breakdown() {
        let engine = MeasurementEngine::new();

        // Email events
        for _ in 0..10 {
            engine.emit(
                MeasurementEventType::Delivered,
                EventSource::BidEngine,
                "email",
                Some("user_1"),
                None,
                Some("camp_1"),
                std::collections::HashMap::new(),
            );
        }
        for _ in 0..5 {
            engine.emit(
                MeasurementEventType::Viewed,
                EventSource::BidEngine,
                "email",
                Some("user_1"),
                None,
                Some("camp_1"),
                std::collections::HashMap::new(),
            );
        }

        // Push events
        for _ in 0..8 {
            engine.emit(
                MeasurementEventType::Delivered,
                EventSource::JourneyEngine,
                "push",
                Some("user_2"),
                None,
                Some("camp_1"),
                std::collections::HashMap::new(),
            );
        }

        let report = engine.breakdown(&ReportingBreakdown {
            name: "Channel Performance".to_string(),
            group_by: vec![BreakdownDimension::Channel],
            metrics: vec![ReportMetric::Deliveries, ReportMetric::Impressions],
            filters: vec![],
            time_range: TimeRange {
                start: Utc::now() - Duration::hours(1),
                end: Utc::now() + Duration::hours(1),
            },
        });

        assert_eq!(report.rows.len(), 2); // email + push
    }

    #[test]
    fn test_experiment_measurement() {
        let engine = MeasurementEngine::new();
        let exp_id = Uuid::new_v4();
        let control_id = Uuid::new_v4();
        let treatment_id = Uuid::new_v4();

        engine.register_experiment(
            exp_id,
            "Homepage CTA Test",
            vec![
                (control_id, "Control".to_string(), true),
                (treatment_id, "Treatment A".to_string(), false),
            ],
        );

        // Record events for control
        for _ in 0..100 {
            engine.record_experiment_event(
                &exp_id,
                &control_id,
                &MeasurementEventType::Viewed,
                0.0,
            );
        }
        for _ in 0..10 {
            engine.record_experiment_event(
                &exp_id,
                &control_id,
                &MeasurementEventType::Clicked,
                0.0,
            );
        }
        for _ in 0..2 {
            engine.record_experiment_event(
                &exp_id,
                &control_id,
                &MeasurementEventType::Converted,
                50.0,
            );
        }

        // Record events for treatment (better performing)
        for _ in 0..100 {
            engine.record_experiment_event(
                &exp_id,
                &treatment_id,
                &MeasurementEventType::Viewed,
                0.0,
            );
        }
        for _ in 0..15 {
            engine.record_experiment_event(
                &exp_id,
                &treatment_id,
                &MeasurementEventType::Clicked,
                0.0,
            );
        }
        for _ in 0..5 {
            engine.record_experiment_event(
                &exp_id,
                &treatment_id,
                &MeasurementEventType::Converted,
                50.0,
            );
        }

        let exp = engine.get_experiment(&exp_id).unwrap();
        let control = exp.variants.iter().find(|v| v.is_control).unwrap();
        let treatment = exp.variants.iter().find(|v| !v.is_control).unwrap();

        assert_eq!(control.impressions, 100);
        assert_eq!(control.clicks, 10);
        assert_eq!(treatment.clicks, 15);
        assert!(treatment.ctr > control.ctr);
        assert!(treatment.lift_vs_control.unwrap() > 0.0);
    }

    #[test]
    fn test_revenue_metric() {
        let engine = MeasurementEngine::new();

        let mut metrics = std::collections::HashMap::new();
        metrics.insert("revenue".to_string(), 99.99);
        engine.emit(
            MeasurementEventType::Revenue,
            EventSource::DirectActivation,
            "email",
            Some("user_1"),
            None,
            Some("camp_1"),
            metrics,
        );

        let mut metrics2 = std::collections::HashMap::new();
        metrics2.insert("revenue".to_string(), 49.99);
        engine.emit(
            MeasurementEventType::Revenue,
            EventSource::DirectActivation,
            "email",
            Some("user_2"),
            None,
            Some("camp_1"),
            metrics2,
        );

        let report = engine.breakdown(&ReportingBreakdown {
            name: "Revenue Report".to_string(),
            group_by: vec![BreakdownDimension::Channel],
            metrics: vec![ReportMetric::Revenue],
            filters: vec![],
            time_range: TimeRange {
                start: Utc::now() - Duration::hours(1),
                end: Utc::now() + Duration::hours(1),
            },
        });

        let total_revenue = report.totals.get("Revenue").copied().unwrap_or(0.0);
        assert!((total_revenue - 149.98).abs() < 0.01);
    }
}
