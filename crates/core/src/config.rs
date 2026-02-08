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
