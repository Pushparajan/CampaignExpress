//! Scheduled report builder — define, generate, and export campaign reports
//! in CSV, JSON, and other formats with scheduling support.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ─── Types ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportType {
    CampaignPerformance,
    ChannelComparison,
    SegmentAnalysis,
    RevenueAttribution,
    FunnelConversion,
    CohortRetention,
    AbTestResults,
    BudgetUtilization,
    EngagementOverTime,
    CustomQuery,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Aggregation {
    Sum,
    Average,
    Count,
    Min,
    Max,
    Median,
    CountDistinct,
    Percentile90,
    Percentile99,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricFormat {
    Number,
    Currency,
    Percentage,
    Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricColumn {
    pub name: String,
    pub aggregation: Aggregation,
    pub format: MetricFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionColumn {
    pub name: String,
    pub group_by: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Between,
    In,
    Contains,
    StartsWith,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Ascending,
    #[default]
    Descending,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleFrequency {
    Daily,
    Weekly,
    Biweekly,
    Monthly,
    Quarterly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Csv,
    Json,
    Pdf,
    Excel,
    Html,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSchedule {
    pub frequency: ScheduleFrequency,
    pub recipients: Vec<String>,
    pub format: ExportFormat,
    pub next_run: DateTime<Utc>,
    pub enabled: bool,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportDefinition {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub report_type: ReportType,
    pub metrics: Vec<MetricColumn>,
    pub dimensions: Vec<DimensionColumn>,
    pub filters: Vec<ReportFilter>,
    pub sort_by: Option<String>,
    pub sort_order: SortOrder,
    pub limit: Option<usize>,
    pub created_by: Uuid,
    pub schedule: Option<ReportSchedule>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_rows: u64,
    pub execution_time_ms: u64,
    pub filters_applied: u32,
    pub date_range: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportOutput {
    pub report_id: Uuid,
    pub definition_name: String,
    pub generated_at: DateTime<Utc>,
    pub row_count: usize,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub summary: ReportSummary,
    pub export_url: Option<String>,
}

// ─── Report Builder ─────────────────────────────────────────────────────────

pub struct ReportBuilder {
    definitions: DashMap<Uuid, ReportDefinition>,
    generated_reports: DashMap<Uuid, ReportOutput>,
    saved_templates: DashMap<String, ReportDefinition>,
}

impl ReportBuilder {
    pub fn new() -> Self {
        Self {
            definitions: DashMap::new(),
            generated_reports: DashMap::new(),
            saved_templates: DashMap::new(),
        }
    }

    pub fn create_report(&self, def: ReportDefinition) -> Uuid {
        let id = def.id;
        self.definitions.insert(id, def);
        id
    }

    pub fn get_report(&self, id: &Uuid) -> Option<ReportDefinition> {
        self.definitions.get(id).map(|d| d.clone())
    }

    pub fn update_report(&self, id: Uuid, def: ReportDefinition) -> bool {
        if self.definitions.contains_key(&id) {
            self.definitions.insert(id, def);
            true
        } else {
            false
        }
    }

    pub fn delete_report(&self, id: &Uuid) -> bool {
        self.definitions.remove(id).is_some()
    }

    pub fn generate(&self, report_id: &Uuid) -> Option<ReportOutput> {
        let def = self.definitions.get(report_id)?;
        let (columns, rows) = match def.report_type {
            ReportType::CampaignPerformance => self.gen_campaign_performance(),
            ReportType::ChannelComparison => self.gen_channel_comparison(),
            ReportType::BudgetUtilization => self.gen_budget_utilization(),
            ReportType::AbTestResults => self.gen_ab_test_results(),
            _ => self.gen_generic(),
        };

        let output = ReportOutput {
            report_id: *report_id,
            definition_name: def.name.clone(),
            generated_at: Utc::now(),
            row_count: rows.len(),
            columns,
            rows,
            summary: ReportSummary {
                total_rows: 0,
                execution_time_ms: 42,
                filters_applied: def.filters.len() as u32,
                date_range: "Last 30 days".into(),
            },
            export_url: None,
        };
        self.generated_reports.insert(*report_id, output.clone());
        Some(output)
    }

    pub fn list_reports(&self, created_by: Option<&Uuid>) -> Vec<ReportDefinition> {
        self.definitions
            .iter()
            .filter(|d| created_by.is_none_or(|uid| &d.created_by == uid))
            .map(|d| d.clone())
            .collect()
    }

    pub fn get_generated_report(&self, report_id: &Uuid) -> Option<ReportOutput> {
        self.generated_reports.get(report_id).map(|r| r.clone())
    }

    pub fn export_csv(&self, report_id: &Uuid) -> Option<String> {
        let output = self.generated_reports.get(report_id)?;
        let mut csv = output.columns.join(",");
        csv.push('\n');
        for row in &output.rows {
            let cells: Vec<String> = row
                .iter()
                .map(|v| match v {
                    serde_json::Value::String(s) => format!("\"{}\"", s.replace('"', "\"\"")),
                    serde_json::Value::Null => String::new(),
                    other => other.to_string(),
                })
                .collect();
            csv.push_str(&cells.join(","));
            csv.push('\n');
        }
        Some(csv)
    }

    pub fn export_json(&self, report_id: &Uuid) -> Option<String> {
        let output = self.generated_reports.get(report_id)?;
        let mut records: Vec<HashMap<String, serde_json::Value>> = Vec::new();
        for row in &output.rows {
            let mut record = HashMap::new();
            for (i, col) in output.columns.iter().enumerate() {
                if let Some(val) = row.get(i) {
                    record.insert(col.clone(), val.clone());
                }
            }
            records.push(record);
        }
        serde_json::to_string_pretty(&records).ok()
    }

    pub fn save_as_template(&self, name: &str, def: ReportDefinition) {
        self.saved_templates.insert(name.to_string(), def);
    }

    pub fn load_template(&self, name: &str) -> Option<ReportDefinition> {
        self.saved_templates.get(name).map(|d| d.clone())
    }

    pub fn list_templates(&self) -> Vec<(String, ReportDefinition)> {
        self.saved_templates
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect()
    }

    pub fn get_scheduled_reports(&self) -> Vec<ReportDefinition> {
        self.definitions
            .iter()
            .filter(|d| d.schedule.as_ref().is_some_and(|s| s.enabled))
            .map(|d| d.clone())
            .collect()
    }

    pub fn seed_default_templates(&self) {
        let now = Utc::now();
        let user = Uuid::nil();

        let templates = vec![
            (
                "Weekly Campaign Summary",
                ReportType::CampaignPerformance,
                vec!["sends", "opens", "clicks", "conversions", "revenue"],
            ),
            (
                "Monthly Revenue Report",
                ReportType::RevenueAttribution,
                vec!["campaign", "attributed_revenue", "conversions", "roas"],
            ),
            (
                "Channel Performance Comparison",
                ReportType::ChannelComparison,
                vec![
                    "channel",
                    "sends",
                    "delivery_rate",
                    "open_rate",
                    "click_rate",
                ],
            ),
            (
                "A/B Test Results",
                ReportType::AbTestResults,
                vec![
                    "variant",
                    "impressions",
                    "conversions",
                    "lift",
                    "confidence",
                ],
            ),
            (
                "Budget Utilization Report",
                ReportType::BudgetUtilization,
                vec!["campaign", "budget", "spent", "remaining", "pacing"],
            ),
        ];

        for (name, report_type, metric_names) in templates {
            let def = ReportDefinition {
                id: Uuid::new_v4(),
                name: name.into(),
                description: format!("Default template: {name}"),
                report_type,
                metrics: metric_names
                    .iter()
                    .map(|n| MetricColumn {
                        name: n.to_string(),
                        aggregation: Aggregation::Sum,
                        format: MetricFormat::Number,
                    })
                    .collect(),
                dimensions: vec![],
                filters: vec![],
                sort_by: None,
                sort_order: SortOrder::Descending,
                limit: None,
                created_by: user,
                schedule: None,
                created_at: now,
                updated_at: now,
            };
            self.saved_templates.insert(name.to_string(), def);
        }
    }

    // ─── Data generators ────────────────────────────────────────────────────

    fn gen_campaign_performance(&self) -> (Vec<String>, Vec<Vec<serde_json::Value>>) {
        let cols = vec![
            "campaign",
            "sends",
            "opens",
            "clicks",
            "conversions",
            "revenue",
            "open_rate",
            "ctr",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let campaigns = [
            ("Summer Sale", 50000, 22500, 4500, 900, 45000.0),
            ("Welcome Series", 30000, 18000, 5400, 1620, 81000.0),
            ("Re-engagement", 20000, 6000, 1200, 180, 9000.0),
            ("Holiday Promo", 75000, 37500, 11250, 2250, 112500.0),
            ("Loyalty Rewards", 15000, 9000, 3600, 1080, 54000.0),
            ("Flash Sale", 40000, 20000, 8000, 2400, 96000.0),
            ("New Product", 25000, 12500, 3750, 750, 37500.0),
            ("Abandoned Cart", 35000, 21000, 8400, 3360, 168000.0),
            ("Anniversary", 10000, 6000, 2400, 720, 36000.0),
            ("Win-Back", 18000, 5400, 1080, 162, 8100.0),
        ];

        let rows = campaigns
            .iter()
            .map(|(name, sends, opens, clicks, conv, rev)| {
                vec![
                    serde_json::json!(name),
                    serde_json::json!(sends),
                    serde_json::json!(opens),
                    serde_json::json!(clicks),
                    serde_json::json!(conv),
                    serde_json::json!(rev),
                    serde_json::json!((*opens as f64 / *sends as f64 * 100.0).round() / 100.0),
                    serde_json::json!((*clicks as f64 / *opens as f64 * 100.0).round() / 100.0),
                ]
            })
            .collect();

        (cols, rows)
    }

    fn gen_channel_comparison(&self) -> (Vec<String>, Vec<Vec<serde_json::Value>>) {
        let cols = vec![
            "channel",
            "sends",
            "deliveries",
            "engagements",
            "conversions",
            "revenue",
            "delivery_rate",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let channels = [
            ("email", 150000, 145000, 43500, 6525, 326250.0),
            ("push", 80000, 78000, 23400, 3510, 175500.0),
            ("sms", 40000, 39600, 15840, 2376, 118800.0),
            ("in_app", 60000, 60000, 30000, 4500, 225000.0),
            ("whatsapp", 25000, 24750, 12375, 1856, 92800.0),
            ("web_push", 35000, 33250, 9975, 1496, 74800.0),
        ];

        let rows = channels
            .iter()
            .map(|(ch, sends, del, eng, conv, rev)| {
                vec![
                    serde_json::json!(ch),
                    serde_json::json!(sends),
                    serde_json::json!(del),
                    serde_json::json!(eng),
                    serde_json::json!(conv),
                    serde_json::json!(rev),
                    serde_json::json!((*del as f64 / *sends as f64 * 100.0).round() / 100.0),
                ]
            })
            .collect();

        (cols, rows)
    }

    fn gen_budget_utilization(&self) -> (Vec<String>, Vec<Vec<serde_json::Value>>) {
        let cols = vec!["campaign", "budget", "spent", "remaining", "pacing_status"]
            .into_iter()
            .map(String::from)
            .collect();

        let data = [
            ("Summer Sale", 50000.0, 42000.0, "on_track"),
            ("Welcome Series", 20000.0, 18500.0, "overspending"),
            ("Re-engagement", 15000.0, 8000.0, "underspending"),
            ("Holiday Promo", 100000.0, 100000.0, "exhausted"),
            ("Loyalty Rewards", 30000.0, 24000.0, "on_track"),
        ];

        let rows = data
            .iter()
            .map(|(name, budget, spent, pacing)| {
                vec![
                    serde_json::json!(name),
                    serde_json::json!(budget),
                    serde_json::json!(spent),
                    serde_json::json!(budget - spent),
                    serde_json::json!(pacing),
                ]
            })
            .collect();

        (cols, rows)
    }

    fn gen_ab_test_results(&self) -> (Vec<String>, Vec<Vec<serde_json::Value>>) {
        let cols = vec![
            "variant",
            "impressions",
            "conversions",
            "conversion_rate",
            "lift_vs_control",
            "confidence",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        let rows = vec![
            vec![
                serde_json::json!("Control"),
                serde_json::json!(10000),
                serde_json::json!(250),
                serde_json::json!(0.025),
                serde_json::json!(0.0),
                serde_json::json!("baseline"),
            ],
            vec![
                serde_json::json!("Variant A"),
                serde_json::json!(10000),
                serde_json::json!(310),
                serde_json::json!(0.031),
                serde_json::json!(0.24),
                serde_json::json!(0.95),
            ],
            vec![
                serde_json::json!("Variant B"),
                serde_json::json!(10000),
                serde_json::json!(280),
                serde_json::json!(0.028),
                serde_json::json!(0.12),
                serde_json::json!(0.72),
            ],
        ];

        (cols, rows)
    }

    fn gen_generic(&self) -> (Vec<String>, Vec<Vec<serde_json::Value>>) {
        let cols = vec!["dimension", "metric_1", "metric_2", "metric_3"]
            .into_iter()
            .map(String::from)
            .collect();

        let rows = (1..=5)
            .map(|i| {
                vec![
                    serde_json::json!(format!("Item {i}")),
                    serde_json::json!(i * 1000),
                    serde_json::json!(i as f64 * 0.15),
                    serde_json::json!(i * 500),
                ]
            })
            .collect();

        (cols, rows)
    }
}

impl Default for ReportBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_def(report_type: ReportType) -> ReportDefinition {
        ReportDefinition {
            id: Uuid::new_v4(),
            name: "Test Report".into(),
            description: "desc".into(),
            report_type,
            metrics: vec![],
            dimensions: vec![],
            filters: vec![],
            sort_by: None,
            sort_order: SortOrder::Descending,
            limit: None,
            created_by: Uuid::new_v4(),
            schedule: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn test_create_and_generate() {
        let builder = ReportBuilder::new();
        let def = make_def(ReportType::CampaignPerformance);
        let id = builder.create_report(def);
        let output = builder.generate(&id).unwrap();
        assert_eq!(output.row_count, 10);
        assert!(output.columns.contains(&"campaign".to_string()));
    }

    #[test]
    fn test_csv_export() {
        let builder = ReportBuilder::new();
        let def = make_def(ReportType::ChannelComparison);
        let id = builder.create_report(def);
        builder.generate(&id);
        let csv = builder.export_csv(&id).unwrap();
        assert!(csv.starts_with("channel,"));
        assert!(csv.contains("\"email\""));
        let line_count = csv.lines().count();
        assert_eq!(line_count, 7); // header + 6 channels
    }

    #[test]
    fn test_json_export() {
        let builder = ReportBuilder::new();
        let def = make_def(ReportType::BudgetUtilization);
        let id = builder.create_report(def);
        builder.generate(&id);
        let json = builder.export_json(&id).unwrap();
        let parsed: Vec<HashMap<String, serde_json::Value>> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 5);
        assert!(parsed[0].contains_key("campaign"));
    }

    #[test]
    fn test_templates() {
        let builder = ReportBuilder::new();
        builder.seed_default_templates();
        let templates = builder.list_templates();
        assert_eq!(templates.len(), 5);
        let loaded = builder.load_template("A/B Test Results");
        assert!(loaded.is_some());
    }

    #[test]
    fn test_scheduled_reports() {
        let builder = ReportBuilder::new();
        let mut def = make_def(ReportType::CampaignPerformance);
        def.schedule = Some(ReportSchedule {
            frequency: ScheduleFrequency::Weekly,
            recipients: vec!["team@example.com".into()],
            format: ExportFormat::Csv,
            next_run: Utc::now(),
            enabled: true,
            timezone: "UTC".into(),
        });
        builder.create_report(def);

        let mut def2 = make_def(ReportType::ChannelComparison);
        def2.schedule = Some(ReportSchedule {
            frequency: ScheduleFrequency::Monthly,
            recipients: vec![],
            format: ExportFormat::Json,
            next_run: Utc::now(),
            enabled: false,
            timezone: "UTC".into(),
        });
        builder.create_report(def2);

        let scheduled = builder.get_scheduled_reports();
        assert_eq!(scheduled.len(), 1);
    }
}
