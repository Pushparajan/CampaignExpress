use serde::Deserialize;

/// Root application configuration. Loaded from environment variables
/// with the prefix `CAMPAIGN_EXPRESS__` and TOML config files.
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_node_id")]
    pub node_id: String,
    #[serde(default = "default_agents_per_node")]
    pub agents_per_node: usize,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub nats: NatsConfig,
    #[serde(default)]
    pub redis: RedisConfig,
    #[serde(default)]
    pub clickhouse: ClickHouseConfig,
    #[serde(default)]
    pub npu: NpuConfig,
    #[serde(default)]
    pub metrics: MetricsConfig,
    #[serde(default)]
    pub loyalty: LoyaltyConfig,
    #[serde(default)]
    pub dsp: DspIntegrationConfig,
    #[serde(default)]
    pub journey: JourneyConfig,
    #[serde(default)]
    pub dco: DcoConfig,
    #[serde(default)]
    pub cdp: CdpGlobalConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApiConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NatsConfig {
    #[serde(default = "default_nats_urls")]
    pub urls: Vec<String>,
    #[serde(default = "default_stream_name")]
    pub stream_name: String,
    #[serde(default = "default_consumer_prefix")]
    pub consumer_prefix: String,
    #[serde(default = "default_nats_max_reconnects")]
    pub max_reconnects: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    #[serde(default = "default_redis_urls")]
    pub urls: Vec<String>,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
    #[serde(default = "default_ttl_secs")]
    pub ttl_secs: u64,
    #[serde(default = "default_connect_timeout_ms")]
    pub connect_timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClickHouseConfig {
    #[serde(default = "default_clickhouse_url")]
    pub url: String,
    #[serde(default = "default_clickhouse_db")]
    pub database: String,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_flush_interval_ms")]
    pub flush_interval_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NpuConfig {
    #[serde(default = "default_model_path")]
    pub model_path: String,
    #[serde(default = "default_device")]
    pub device: String,
    #[serde(default = "default_num_threads")]
    pub num_threads: usize,
    #[serde(default = "default_npu_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_inference_timeout_ms")]
    pub inference_timeout_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MetricsConfig {
    #[serde(default = "default_metrics_port")]
    pub port: u16,
}

// Default functions
fn default_node_id() -> String {
    "node-01".to_string()
}
fn default_agents_per_node() -> usize {
    20
}
fn default_host() -> String {
    "0.0.0.0".to_string()
}
fn default_http_port() -> u16 {
    8080
}
fn default_grpc_port() -> u16 {
    9090
}
fn default_nats_urls() -> Vec<String> {
    vec!["nats://localhost:4222".to_string()]
}
fn default_stream_name() -> String {
    "campaign-bids".to_string()
}
fn default_consumer_prefix() -> String {
    "agent".to_string()
}
fn default_nats_max_reconnects() -> usize {
    60
}
fn default_redis_urls() -> Vec<String> {
    vec!["redis://localhost:6379".to_string()]
}
fn default_pool_size() -> u32 {
    32
}
fn default_ttl_secs() -> u64 {
    3600
}
fn default_connect_timeout_ms() -> u64 {
    5000
}
fn default_clickhouse_url() -> String {
    "http://localhost:8123".to_string()
}
fn default_clickhouse_db() -> String {
    "campaign_express".to_string()
}
fn default_batch_size() -> usize {
    10000
}
fn default_flush_interval_ms() -> u64 {
    1000
}
fn default_model_path() -> String {
    "/models/colanet.onnx".to_string()
}
fn default_device() -> String {
    "cpu".to_string()
}
fn default_num_threads() -> usize {
    4
}
fn default_npu_batch_size() -> usize {
    64
}
fn default_inference_timeout_ms() -> u64 {
    5
}
fn default_metrics_port() -> u16 {
    9091
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            http_port: default_http_port(),
            grpc_port: default_grpc_port(),
        }
    }
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            urls: default_nats_urls(),
            stream_name: default_stream_name(),
            consumer_prefix: default_consumer_prefix(),
            max_reconnects: default_nats_max_reconnects(),
        }
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            urls: default_redis_urls(),
            pool_size: default_pool_size(),
            ttl_secs: default_ttl_secs(),
            connect_timeout_ms: default_connect_timeout_ms(),
        }
    }
}

impl Default for ClickHouseConfig {
    fn default() -> Self {
        Self {
            url: default_clickhouse_url(),
            database: default_clickhouse_db(),
            batch_size: default_batch_size(),
            flush_interval_ms: default_flush_interval_ms(),
        }
    }
}

impl Default for NpuConfig {
    fn default() -> Self {
        Self {
            model_path: default_model_path(),
            device: default_device(),
            num_threads: default_num_threads(),
            batch_size: default_npu_batch_size(),
            inference_timeout_ms: default_inference_timeout_ms(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            port: default_metrics_port(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            node_id: default_node_id(),
            agents_per_node: default_agents_per_node(),
            api: ApiConfig::default(),
            nats: NatsConfig::default(),
            redis: RedisConfig::default(),
            clickhouse: ClickHouseConfig::default(),
            npu: NpuConfig::default(),
            metrics: MetricsConfig::default(),
            loyalty: LoyaltyConfig::default(),
            dsp: DspIntegrationConfig::default(),
            journey: JourneyConfig::default(),
            dco: DcoConfig::default(),
            cdp: CdpGlobalConfig::default(),
        }
    }
}

// ─── Loyalty Config ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct LoyaltyConfig {
    #[serde(default = "default_loyalty_enabled")]
    pub enabled: bool,
    #[serde(default = "default_star_expiry_days")]
    pub star_expiry_days: u32,
    #[serde(default = "default_gold_threshold")]
    pub gold_threshold: u32,
    #[serde(default = "default_reserve_threshold")]
    pub reserve_threshold: u32,
    #[serde(default = "default_qualifying_period_months")]
    pub qualifying_period_months: u32,
}

fn default_loyalty_enabled() -> bool { true }
fn default_star_expiry_days() -> u32 { 180 }
fn default_gold_threshold() -> u32 { 500 }
fn default_reserve_threshold() -> u32 { 2500 }
fn default_qualifying_period_months() -> u32 { 12 }

impl Default for LoyaltyConfig {
    fn default() -> Self {
        Self {
            enabled: default_loyalty_enabled(),
            star_expiry_days: default_star_expiry_days(),
            gold_threshold: default_gold_threshold(),
            reserve_threshold: default_reserve_threshold(),
            qualifying_period_months: default_qualifying_period_months(),
        }
    }
}

// ─── DSP Integration Config ─────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct DspIntegrationConfig {
    #[serde(default = "default_dsp_enabled")]
    pub enabled: bool,
    #[serde(default = "default_dsp_timeout_ms")]
    pub default_timeout_ms: u64,
    #[serde(default = "default_dsp_max_concurrent")]
    pub max_concurrent_requests: usize,
}

fn default_dsp_enabled() -> bool { false }
fn default_dsp_timeout_ms() -> u64 { 200 }
fn default_dsp_max_concurrent() -> usize { 1000 }

impl Default for DspIntegrationConfig {
    fn default() -> Self {
        Self {
            enabled: default_dsp_enabled(),
            default_timeout_ms: default_dsp_timeout_ms(),
            max_concurrent_requests: default_dsp_max_concurrent(),
        }
    }
}

// ─── Journey Config ─────────────────────────────────────────────────────
#[derive(Debug, Clone, Deserialize)]
pub struct JourneyConfig {
    #[serde(default = "default_journey_enabled")]
    pub enabled: bool,
    #[serde(default = "default_max_active_journeys")]
    pub max_active_journeys: usize,
    #[serde(default = "default_max_instances_per_journey")]
    pub max_instances_per_journey: usize,
    #[serde(default = "default_evaluation_interval_ms")]
    pub evaluation_interval_ms: u64,
}

fn default_journey_enabled() -> bool { true }
fn default_max_active_journeys() -> usize { 100 }
fn default_max_instances_per_journey() -> usize { 1_000_000 }
fn default_evaluation_interval_ms() -> u64 { 100 }

impl Default for JourneyConfig {
    fn default() -> Self {
        Self {
            enabled: default_journey_enabled(),
            max_active_journeys: default_max_active_journeys(),
            max_instances_per_journey: default_max_instances_per_journey(),
            evaluation_interval_ms: default_evaluation_interval_ms(),
        }
    }
}

// ─── DCO Config ─────────────────────────────────────────────────────────
#[derive(Debug, Clone, Deserialize)]
pub struct DcoConfig {
    #[serde(default = "default_dco_enabled")]
    pub enabled: bool,
    #[serde(default = "default_max_combinations")]
    pub max_combinations: usize,
    #[serde(default = "default_exploration_rate")]
    pub exploration_rate: f64,
}

fn default_dco_enabled() -> bool { true }
fn default_max_combinations() -> usize { 1000 }
fn default_exploration_rate() -> f64 { 0.1 }

impl Default for DcoConfig {
    fn default() -> Self {
        Self {
            enabled: default_dco_enabled(),
            max_combinations: default_max_combinations(),
            exploration_rate: default_exploration_rate(),
        }
    }
}

// ─── CDP Global Config ──────────────────────────────────────────────────
#[derive(Debug, Clone, Deserialize)]
pub struct CdpGlobalConfig {
    #[serde(default = "default_cdp_enabled")]
    pub enabled: bool,
    #[serde(default = "default_sync_interval_secs")]
    pub default_sync_interval_secs: u64,
    #[serde(default = "default_webhook_secret")]
    pub webhook_secret: String,
}

fn default_cdp_enabled() -> bool { false }
fn default_sync_interval_secs() -> u64 { 300 }
fn default_webhook_secret() -> String { "cdp-webhook-secret".to_string() }

impl Default for CdpGlobalConfig {
    fn default() -> Self {
        Self {
            enabled: default_cdp_enabled(),
            default_sync_interval_secs: default_sync_interval_secs(),
            webhook_secret: default_webhook_secret(),
        }
    }
}

impl AppConfig {
    /// Load configuration from environment variables and optional config file.
    pub fn load() -> Result<Self, config::ConfigError> {
        let builder = config::Config::builder()
            .add_source(
                config::Environment::with_prefix("CAMPAIGN_EXPRESS")
                    .separator("__")
                    .try_parsing(true)
                    .list_separator(","),
            );

        let config = builder.build()?;
        config.try_deserialize()
    }
}
