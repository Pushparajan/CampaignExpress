//! Asynchronous analytics logger that batches events and writes to ClickHouse.
//! Uses a channel-based architecture for non-blocking event submission.

use campaign_core::config::ClickHouseConfig;
use campaign_core::types::{AnalyticsEvent, EventType};
use chrono::Utc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Analytics logger with background batch writer.
pub struct AnalyticsLogger {
    sender: mpsc::Sender<AnalyticsEvent>,
    node_id: String,
}

impl AnalyticsLogger {
    /// Create a new analytics logger and spawn the background writer.
    pub async fn new(config: &ClickHouseConfig, node_id: String) -> anyhow::Result<Self> {
        let (sender, receiver) = mpsc::channel::<AnalyticsEvent>(100_000);

        let writer = BatchWriter::new(config).await?;
        let batch_size = config.batch_size;
        let flush_interval = std::time::Duration::from_millis(config.flush_interval_ms);

        // Spawn background batch writer
        tokio::spawn(async move {
            writer.run(receiver, batch_size, flush_interval).await;
        });

        info!("Analytics logger initialized with ClickHouse backend");

        Ok(Self { sender, node_id })
    }

    /// Log an analytics event (non-blocking).
    #[allow(clippy::too_many_arguments)]
    pub async fn log_event(
        &self,
        event_type: EventType,
        request_id: String,
        agent_id: String,
        impression_id: Option<String>,
        user_id: Option<String>,
        offer_id: Option<String>,
        bid_price: Option<f64>,
        win_price: Option<f64>,
        inference_latency_us: Option<u64>,
        total_latency_us: Option<u64>,
    ) {
        let event = AnalyticsEvent {
            event_id: Uuid::new_v4(),
            event_type,
            request_id,
            impression_id,
            user_id,
            offer_id,
            bid_price,
            win_price,
            agent_id,
            node_id: self.node_id.clone(),
            inference_latency_us,
            total_latency_us,
            timestamp: Utc::now(),
        };

        if let Err(e) = self.sender.try_send(event) {
            metrics::counter!("analytics.dropped").increment(1);
            warn!("Analytics event dropped: {}", e);
        } else {
            metrics::counter!("analytics.queued").increment(1);
        }
    }
}

/// Background writer that batches events and flushes to ClickHouse.
struct BatchWriter {
    client: clickhouse::Client,
}

impl BatchWriter {
    async fn new(config: &ClickHouseConfig) -> anyhow::Result<Self> {
        let client = clickhouse::Client::default()
            .with_url(&config.url)
            .with_database(&config.database);

        // Create the analytics table if it doesn't exist
        Self::ensure_schema(&client).await?;

        Ok(Self { client })
    }

    async fn ensure_schema(client: &clickhouse::Client) -> anyhow::Result<()> {
        client
            .query(
                "CREATE TABLE IF NOT EXISTS analytics_events (
                    event_id UUID,
                    event_type String,
                    request_id String,
                    impression_id Nullable(String),
                    user_id Nullable(String),
                    offer_id Nullable(String),
                    bid_price Nullable(Float64),
                    win_price Nullable(Float64),
                    agent_id String,
                    node_id String,
                    inference_latency_us Nullable(UInt64),
                    total_latency_us Nullable(UInt64),
                    timestamp DateTime64(3)
                ) ENGINE = MergeTree()
                ORDER BY (timestamp, event_type, node_id)
                PARTITION BY toYYYYMM(timestamp)
                TTL timestamp + INTERVAL 90 DAY",
            )
            .execute()
            .await?;

        info!("ClickHouse schema verified");
        Ok(())
    }

    async fn run(
        self,
        mut receiver: mpsc::Receiver<AnalyticsEvent>,
        batch_size: usize,
        flush_interval: std::time::Duration,
    ) {
        let mut buffer: Vec<AnalyticsEvent> = Vec::with_capacity(batch_size);
        let mut interval = tokio::time::interval(flush_interval);

        loop {
            tokio::select! {
                Some(event) = receiver.recv() => {
                    buffer.push(event);
                    if buffer.len() >= batch_size {
                        self.flush(&mut buffer).await;
                    }
                }
                _ = interval.tick() => {
                    if !buffer.is_empty() {
                        self.flush(&mut buffer).await;
                    }
                }
            }
        }
    }

    async fn flush(&self, buffer: &mut Vec<AnalyticsEvent>) {
        let count = buffer.len();
        debug!(count = count, "Flushing analytics batch to ClickHouse");

        // Serialize events as NDJSON and insert
        let mut json_rows = Vec::with_capacity(buffer.len());
        for e in buffer.iter() {
            if let Ok(json) = serde_json::to_string(e) {
                json_rows.push(json);
            }
        }

        if json_rows.is_empty() {
            buffer.clear();
            return;
        }

        let insert_sql = format!(
            "INSERT INTO analytics_events FORMAT JSONEachRow {}",
            json_rows.join("\n")
        );

        match self.client.query(&insert_sql).execute().await {
            Ok(_) => {
                metrics::counter!("analytics.flushed").increment(count as u64);
                debug!(count = count, "Analytics batch flushed successfully");
            }
            Err(e) => {
                metrics::counter!("analytics.flush_errors").increment(1);
                error!(error = %e, count = count, "Failed to flush analytics batch");
            }
        }

        buffer.clear();
    }
}
