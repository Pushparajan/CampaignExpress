//! BI tools adaptors — connectors for Power BI and Excel reporting.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BiProvider {
    PowerBi,
    Excel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiConfig {
    pub provider: BiProvider,
    pub api_base_url: String,
    pub api_token: String,
    pub workspace_id: Option<String>,
    pub dataset_id: Option<String>,
    pub refresh_on_push: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetSchema {
    pub name: String,
    pub tables: Vec<TableSchema>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub is_nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Currency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPushResult {
    pub provider: BiProvider,
    pub dataset: String,
    pub rows_pushed: u32,
    pub tables_updated: Vec<String>,
    pub pushed_at: DateTime<Utc>,
    pub refresh_triggered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelExport {
    pub id: Uuid,
    pub name: String,
    pub sheets: Vec<ExcelSheet>,
    pub file_size_bytes: u64,
    pub generated_at: DateTime<Utc>,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelSheet {
    pub name: String,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

// ---------------------------------------------------------------------------
// Adaptor
// ---------------------------------------------------------------------------

pub struct BiToolsAdaptor {
    configs: DashMap<String, BiConfig>,
    schemas: DashMap<String, DatasetSchema>,
    exports: DashMap<Uuid, ExcelExport>,
}

impl BiToolsAdaptor {
    pub fn new() -> Self {
        Self {
            configs: DashMap::new(),
            schemas: DashMap::new(),
            exports: DashMap::new(),
        }
    }

    /// Register a named BI provider configuration.
    pub fn register_provider(&self, name: &str, config: BiConfig) {
        tracing::info!(provider = name, "Registering BI tools provider");
        self.configs.insert(name.to_string(), config);
    }

    /// Define or replace a dataset schema.
    pub fn define_dataset_schema(&self, name: &str, tables: Vec<TableSchema>) {
        let schema = DatasetSchema {
            name: name.to_string(),
            tables,
            created_at: Utc::now(),
        };
        self.schemas.insert(name.to_string(), schema);
    }

    /// Simulate pushing data rows to a Power BI dataset table.
    pub fn push_data(
        &self,
        provider_name: &str,
        table_name: &str,
        rows: Vec<Vec<serde_json::Value>>,
    ) -> Option<DataPushResult> {
        let config = self.configs.get(provider_name)?;
        let row_count = rows.len() as u32;

        tracing::info!(
            provider = provider_name,
            table = table_name,
            rows = row_count,
            "Pushing data to BI provider"
        );

        Some(DataPushResult {
            provider: config.provider.clone(),
            dataset: config
                .dataset_id
                .clone()
                .unwrap_or_else(|| "default".to_string()),
            rows_pushed: row_count,
            tables_updated: vec![table_name.to_string()],
            pushed_at: Utc::now(),
            refresh_triggered: config.refresh_on_push,
        })
    }

    /// Create an Excel export record with a download URL.
    pub fn generate_excel_export(&self, name: &str, sheets: Vec<ExcelSheet>) -> ExcelExport {
        let id = Uuid::new_v4();

        // Estimate file size based on content
        let estimated_size: u64 = sheets
            .iter()
            .map(|s| {
                let header_bytes = s.headers.iter().map(|h| h.len() as u64).sum::<u64>();
                let row_bytes: u64 = s
                    .rows
                    .iter()
                    .map(|row| row.iter().map(|v| v.to_string().len() as u64).sum::<u64>())
                    .sum();
                header_bytes + row_bytes + 1024 // overhead per sheet
            })
            .sum();

        let export = ExcelExport {
            id,
            name: name.to_string(),
            sheets,
            file_size_bytes: estimated_size,
            generated_at: Utc::now(),
            download_url: format!(
                "https://exports.campaignexpress.io/excel/{}/{}.xlsx",
                id, name
            ),
        };

        self.exports.insert(id, export.clone());

        tracing::info!(export_id = %id, name, "Generated Excel export");
        export
    }

    /// Retrieve an Excel export by ID.
    pub fn get_export(&self, id: &Uuid) -> Option<ExcelExport> {
        self.exports.get(id).map(|e| e.clone())
    }

    /// List all generated Excel exports.
    pub fn list_exports(&self) -> Vec<ExcelExport> {
        self.exports.iter().map(|e| e.value().clone()).collect()
    }

    /// Create a pre-defined campaign report dataset schema.
    pub fn create_campaign_report_dataset(&self) -> DatasetSchema {
        let campaigns_table = TableSchema {
            name: "campaigns".to_string(),
            columns: vec![
                ColumnDef {
                    name: "name".to_string(),
                    data_type: DataType::String,
                    is_nullable: false,
                },
                ColumnDef {
                    name: "status".to_string(),
                    data_type: DataType::String,
                    is_nullable: false,
                },
                ColumnDef {
                    name: "budget".to_string(),
                    data_type: DataType::Currency,
                    is_nullable: false,
                },
                ColumnDef {
                    name: "spend".to_string(),
                    data_type: DataType::Currency,
                    is_nullable: false,
                },
                ColumnDef {
                    name: "impressions".to_string(),
                    data_type: DataType::Integer,
                    is_nullable: false,
                },
                ColumnDef {
                    name: "clicks".to_string(),
                    data_type: DataType::Integer,
                    is_nullable: false,
                },
                ColumnDef {
                    name: "conversions".to_string(),
                    data_type: DataType::Integer,
                    is_nullable: false,
                },
                ColumnDef {
                    name: "revenue".to_string(),
                    data_type: DataType::Currency,
                    is_nullable: false,
                },
            ],
        };

        let channels_table = TableSchema {
            name: "channels".to_string(),
            columns: vec![
                ColumnDef {
                    name: "name".to_string(),
                    data_type: DataType::String,
                    is_nullable: false,
                },
                ColumnDef {
                    name: "sends".to_string(),
                    data_type: DataType::Integer,
                    is_nullable: false,
                },
                ColumnDef {
                    name: "deliveries".to_string(),
                    data_type: DataType::Integer,
                    is_nullable: false,
                },
                ColumnDef {
                    name: "opens".to_string(),
                    data_type: DataType::Integer,
                    is_nullable: false,
                },
                ColumnDef {
                    name: "clicks".to_string(),
                    data_type: DataType::Integer,
                    is_nullable: false,
                },
            ],
        };

        let segments_table = TableSchema {
            name: "segments".to_string(),
            columns: vec![
                ColumnDef {
                    name: "name".to_string(),
                    data_type: DataType::String,
                    is_nullable: false,
                },
                ColumnDef {
                    name: "size".to_string(),
                    data_type: DataType::Integer,
                    is_nullable: false,
                },
                ColumnDef {
                    name: "engagement_rate".to_string(),
                    data_type: DataType::Float,
                    is_nullable: false,
                },
            ],
        };

        let schema = DatasetSchema {
            name: "campaign_report".to_string(),
            tables: vec![campaigns_table, channels_table, segments_table],
            created_at: Utc::now(),
        };

        self.schemas
            .insert("campaign_report".to_string(), schema.clone());

        schema
    }

    /// Push sample campaign data to a configured Power BI workspace.
    pub fn seed_power_bi_defaults(&self, provider_name: &str) -> Option<DataPushResult> {
        let config = self.configs.get(provider_name)?;

        // Ensure we have the campaign report schema
        if !self.schemas.contains_key("campaign_report") {
            self.create_campaign_report_dataset();
        }

        let sample_rows: Vec<Vec<serde_json::Value>> = vec![
            vec![
                serde_json::json!("Summer Sale 2025"),
                serde_json::json!("active"),
                serde_json::json!(50000),
                serde_json::json!(32500),
                serde_json::json!(1_200_000),
                serde_json::json!(45_000),
                serde_json::json!(3_200),
                serde_json::json!(128_000),
            ],
            vec![
                serde_json::json!("Back to School"),
                serde_json::json!("active"),
                serde_json::json!(35000),
                serde_json::json!(18_750),
                serde_json::json!(850_000),
                serde_json::json!(28_000),
                serde_json::json!(1_800),
                serde_json::json!(72_000),
            ],
            vec![
                serde_json::json!("Holiday Preview"),
                serde_json::json!("draft"),
                serde_json::json!(75000),
                serde_json::json!(0),
                serde_json::json!(0),
                serde_json::json!(0),
                serde_json::json!(0),
                serde_json::json!(0),
            ],
        ];

        let row_count = sample_rows.len() as u32;

        tracing::info!(
            provider = provider_name,
            rows = row_count,
            "Seeding Power BI with default campaign data"
        );

        Some(DataPushResult {
            provider: config.provider.clone(),
            dataset: config
                .dataset_id
                .clone()
                .unwrap_or_else(|| "campaign_report".to_string()),
            rows_pushed: row_count,
            tables_updated: vec![
                "campaigns".to_string(),
                "channels".to_string(),
                "segments".to_string(),
            ],
            pushed_at: Utc::now(),
            refresh_triggered: config.refresh_on_push,
        })
    }
}

impl Default for BiToolsAdaptor {
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

    fn power_bi_config() -> BiConfig {
        BiConfig {
            provider: BiProvider::PowerBi,
            api_base_url: "https://api.powerbi.com/v1.0/myorg".to_string(),
            api_token: "test-token".to_string(),
            workspace_id: Some("ws-abc123".to_string()),
            dataset_id: Some("ds-campaign-metrics".to_string()),
            refresh_on_push: true,
        }
    }

    #[test]
    fn test_define_schema() {
        let adaptor = BiToolsAdaptor::new();
        let table = TableSchema {
            name: "test_table".to_string(),
            columns: vec![
                ColumnDef {
                    name: "id".to_string(),
                    data_type: DataType::Integer,
                    is_nullable: false,
                },
                ColumnDef {
                    name: "name".to_string(),
                    data_type: DataType::String,
                    is_nullable: true,
                },
            ],
        };

        adaptor.define_dataset_schema("test_dataset", vec![table]);

        let schema = adaptor.schemas.get("test_dataset").unwrap();
        assert_eq!(schema.name, "test_dataset");
        assert_eq!(schema.tables.len(), 1);
        assert_eq!(schema.tables[0].columns.len(), 2);
    }

    #[test]
    fn test_push_data() {
        let adaptor = BiToolsAdaptor::new();
        adaptor.register_provider("pbi", power_bi_config());

        let rows = vec![
            vec![serde_json::json!(1), serde_json::json!("Campaign A")],
            vec![serde_json::json!(2), serde_json::json!("Campaign B")],
        ];

        let result = adaptor
            .push_data("pbi", "campaigns", rows)
            .expect("push should succeed");

        assert_eq!(result.provider, BiProvider::PowerBi);
        assert_eq!(result.rows_pushed, 2);
        assert_eq!(result.dataset, "ds-campaign-metrics");
        assert!(result.refresh_triggered);

        // Unknown provider returns None
        assert!(adaptor.push_data("unknown", "t", vec![]).is_none());
    }

    #[test]
    fn test_generate_excel_export() {
        let adaptor = BiToolsAdaptor::new();

        let sheet = ExcelSheet {
            name: "Campaign Summary".to_string(),
            headers: vec![
                "Name".to_string(),
                "Status".to_string(),
                "Budget".to_string(),
            ],
            rows: vec![
                vec![
                    serde_json::json!("Summer Sale"),
                    serde_json::json!("active"),
                    serde_json::json!(50000),
                ],
                vec![
                    serde_json::json!("Winter Promo"),
                    serde_json::json!("draft"),
                    serde_json::json!(30000),
                ],
            ],
        };

        let export = adaptor.generate_excel_export("Q3 Report", vec![sheet]);
        assert_eq!(export.name, "Q3 Report");
        assert_eq!(export.sheets.len(), 1);
        assert!(export.file_size_bytes > 0);
        assert!(export.download_url.contains("Q3 Report"));

        // Verify retrieval
        let fetched = adaptor.get_export(&export.id).expect("export should exist");
        assert_eq!(fetched.id, export.id);

        // List exports
        let all = adaptor.list_exports();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn test_campaign_report_dataset() {
        let adaptor = BiToolsAdaptor::new();
        let schema = adaptor.create_campaign_report_dataset();

        assert_eq!(schema.name, "campaign_report");
        assert_eq!(schema.tables.len(), 3);

        let table_names: Vec<&str> = schema.tables.iter().map(|t| t.name.as_str()).collect();
        assert!(table_names.contains(&"campaigns"));
        assert!(table_names.contains(&"channels"));
        assert!(table_names.contains(&"segments"));

        // Campaigns table should have 8 columns
        let campaigns = schema
            .tables
            .iter()
            .find(|t| t.name == "campaigns")
            .unwrap();
        assert_eq!(campaigns.columns.len(), 8);
    }

    #[test]
    fn test_seed_power_bi_defaults() {
        let adaptor = BiToolsAdaptor::new();
        adaptor.register_provider("pbi", power_bi_config());

        let result = adaptor
            .seed_power_bi_defaults("pbi")
            .expect("seed should succeed");

        assert_eq!(result.provider, BiProvider::PowerBi);
        assert_eq!(result.rows_pushed, 3);
        assert!(result.refresh_triggered);
        assert_eq!(result.tables_updated.len(), 3);

        // Unknown provider returns None
        assert!(adaptor.seed_power_bi_defaults("unknown").is_none());
    }
}
