//! Connector Runtime: circuit breaker, retry with exponential backoff,
//! dead letter queue (DLQ), and per-connector metrics/observability.
//!
//! Addresses FR-CNX-001 through FR-CNX-005.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use uuid::Uuid;

// ─── Circuit Breaker ────────────────────────────────────────────────────

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CircuitState {
    /// Normal operation; requests pass through.
    Closed,
    /// Too many failures; requests are rejected.
    Open,
    /// Testing recovery; limited requests allowed.
    HalfOpen,
}

/// Configuration for a circuit breaker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit.
    pub failure_threshold: u32,
    /// Duration the circuit stays open before moving to half-open.
    pub open_duration_secs: u64,
    /// Number of successful requests in half-open to close the circuit.
    pub half_open_successes: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            open_duration_secs: 30,
            half_open_successes: 3,
        }
    }
}

/// Circuit breaker protecting a single connector.
pub struct CircuitBreaker {
    pub config: CircuitBreakerConfig,
    state: parking_lot::Mutex<CircuitState>,
    failure_count: AtomicU64,
    success_count: AtomicU64,
    last_failure_at: parking_lot::Mutex<Option<DateTime<Utc>>>,
    opened_at: parking_lot::Mutex<Option<DateTime<Utc>>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: parking_lot::Mutex::new(CircuitState::Closed),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            last_failure_at: parking_lot::Mutex::new(None),
            opened_at: parking_lot::Mutex::new(None),
        }
    }

    /// Check if a request is allowed through the circuit.
    pub fn allow_request(&self) -> bool {
        let mut state = self.state.lock();
        match *state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if open duration has elapsed
                let opened = self.opened_at.lock();
                if let Some(opened_at) = *opened {
                    let elapsed = (Utc::now() - opened_at).num_seconds() as u64;
                    if elapsed >= self.config.open_duration_secs {
                        *state = CircuitState::HalfOpen;
                        self.success_count.store(0, Ordering::Relaxed);
                        info!("circuit breaker transitioning to half-open");
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful request.
    pub fn record_success(&self) {
        let mut state = self.state.lock();
        match *state {
            CircuitState::HalfOpen => {
                let count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                if count >= self.config.half_open_successes as u64 {
                    *state = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                    info!("circuit breaker closed after recovery");
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitState::Open => {}
        }
    }

    /// Record a failed request.
    pub fn record_failure(&self) {
        let mut state = self.state.lock();
        *self.last_failure_at.lock() = Some(Utc::now());

        match *state {
            CircuitState::Closed => {
                let count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;
                if count >= self.config.failure_threshold as u64 {
                    *state = CircuitState::Open;
                    *self.opened_at.lock() = Some(Utc::now());
                    warn!(failures = count, "circuit breaker opened due to failures");
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open goes back to open
                *state = CircuitState::Open;
                *self.opened_at.lock() = Some(Utc::now());
                self.success_count.store(0, Ordering::Relaxed);
                warn!("circuit breaker re-opened from half-open");
            }
            CircuitState::Open => {}
        }
    }

    /// Current state.
    pub fn state(&self) -> CircuitState {
        *self.state.lock()
    }
}

// ─── Retry Policy ───────────────────────────────────────────────────────

/// Retry configuration with exponential backoff.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Initial backoff duration in milliseconds.
    pub initial_backoff_ms: u64,
    /// Maximum backoff duration in milliseconds.
    pub max_backoff_ms: u64,
    /// Backoff multiplier per attempt.
    pub backoff_multiplier: f64,
    /// Whether to add jitter.
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 30_000,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Compute the backoff duration for a given attempt (0-indexed).
    pub fn backoff_for_attempt(&self, attempt: u32) -> Duration {
        let base_ms = self.initial_backoff_ms as f64 * self.backoff_multiplier.powi(attempt as i32);
        let capped_ms = base_ms.min(self.max_backoff_ms as f64);

        let final_ms = if self.jitter {
            // Simple deterministic jitter: vary by ±25%
            let jitter_factor = 0.75 + (attempt as f64 * 0.1 % 0.5);
            capped_ms * jitter_factor
        } else {
            capped_ms
        };

        Duration::from_millis(final_ms as u64)
    }
}

// ─── Dead Letter Queue ──────────────────────────────────────────────────

/// A failed record in the dead letter queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterRecord {
    pub id: Uuid,
    pub connector_name: String,
    pub payload: serde_json::Value,
    pub error: String,
    pub attempt_count: u32,
    pub first_failed_at: DateTime<Utc>,
    pub last_failed_at: DateTime<Utc>,
    pub retryable: bool,
}

/// Dead letter queue for records that failed processing.
pub struct DeadLetterQueue {
    records: parking_lot::Mutex<VecDeque<DeadLetterRecord>>,
    max_size: usize,
    total_enqueued: AtomicU64,
    total_replayed: AtomicU64,
}

impl DeadLetterQueue {
    pub fn new(max_size: usize) -> Self {
        Self {
            records: parking_lot::Mutex::new(VecDeque::new()),
            max_size,
            total_enqueued: AtomicU64::new(0),
            total_replayed: AtomicU64::new(0),
        }
    }

    /// Enqueue a failed record.
    pub fn enqueue(&self, record: DeadLetterRecord) {
        let mut queue = self.records.lock();
        if queue.len() >= self.max_size {
            queue.pop_front(); // Evict oldest
        }
        queue.push_back(record);
        self.total_enqueued.fetch_add(1, Ordering::Relaxed);
    }

    /// Dequeue a record for replay.
    pub fn dequeue(&self) -> Option<DeadLetterRecord> {
        let mut queue = self.records.lock();
        let record = queue.pop_front()?;
        self.total_replayed.fetch_add(1, Ordering::Relaxed);
        Some(record)
    }

    /// Peek at records without removing them.
    pub fn peek(&self, limit: usize) -> Vec<DeadLetterRecord> {
        let queue = self.records.lock();
        queue.iter().take(limit).cloned().collect()
    }

    /// Current queue depth.
    pub fn depth(&self) -> usize {
        self.records.lock().len()
    }

    /// Get DLQ metrics.
    pub fn metrics(&self) -> DlqMetrics {
        DlqMetrics {
            depth: self.depth(),
            total_enqueued: self.total_enqueued.load(Ordering::Relaxed),
            total_replayed: self.total_replayed.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqMetrics {
    pub depth: usize,
    pub total_enqueued: u64,
    pub total_replayed: u64,
}

// ─── Per-Connector Metrics ──────────────────────────────────────────────

/// Metrics for a single connector instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorMetrics {
    pub connector_name: String,
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_failed: u64,
    pub avg_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub circuit_state: CircuitState,
    pub dlq_depth: usize,
    pub last_success_at: Option<DateTime<Utc>>,
    pub last_failure_at: Option<DateTime<Utc>>,
}

/// Runtime for a single connector, combining circuit breaker, retry, DLQ, and metrics.
pub struct ConnectorRuntime {
    pub name: String,
    pub circuit_breaker: CircuitBreaker,
    pub retry_policy: RetryPolicy,
    pub dlq: DeadLetterQueue,
    requests_total: AtomicU64,
    requests_success: AtomicU64,
    requests_failed: AtomicU64,
    latency_sum_ms: AtomicU64,
    latency_max_ms: AtomicU64,
    last_success_at: parking_lot::Mutex<Option<DateTime<Utc>>>,
    last_failure_at: parking_lot::Mutex<Option<DateTime<Utc>>>,
}

impl ConnectorRuntime {
    pub fn new(name: String, cb_config: CircuitBreakerConfig, retry: RetryPolicy) -> Self {
        Self {
            name,
            circuit_breaker: CircuitBreaker::new(cb_config),
            retry_policy: retry,
            dlq: DeadLetterQueue::new(10_000),
            requests_total: AtomicU64::new(0),
            requests_success: AtomicU64::new(0),
            requests_failed: AtomicU64::new(0),
            latency_sum_ms: AtomicU64::new(0),
            latency_max_ms: AtomicU64::new(0),
            last_success_at: parking_lot::Mutex::new(None),
            last_failure_at: parking_lot::Mutex::new(None),
        }
    }

    /// Check if a request is allowed.
    pub fn allow_request(&self) -> bool {
        self.circuit_breaker.allow_request()
    }

    /// Record a successful operation with its latency.
    pub fn record_success(&self, latency_ms: u64) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.requests_success.fetch_add(1, Ordering::Relaxed);
        self.latency_sum_ms.fetch_add(latency_ms, Ordering::Relaxed);
        self.latency_max_ms.fetch_max(latency_ms, Ordering::Relaxed);
        *self.last_success_at.lock() = Some(Utc::now());
        self.circuit_breaker.record_success();
    }

    /// Record a failed operation, optionally DLQ-ing the payload.
    pub fn record_failure(&self, error: &str, payload: Option<serde_json::Value>, attempt: u32) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
        self.requests_failed.fetch_add(1, Ordering::Relaxed);
        *self.last_failure_at.lock() = Some(Utc::now());
        self.circuit_breaker.record_failure();

        if let Some(payload) = payload {
            if attempt >= self.retry_policy.max_retries {
                let now = Utc::now();
                self.dlq.enqueue(DeadLetterRecord {
                    id: Uuid::new_v4(),
                    connector_name: self.name.clone(),
                    payload,
                    error: error.to_string(),
                    attempt_count: attempt,
                    first_failed_at: now,
                    last_failed_at: now,
                    retryable: true,
                });
            }
        }
    }

    /// Get current metrics snapshot.
    pub fn metrics(&self) -> ConnectorMetrics {
        let total = self.requests_total.load(Ordering::Relaxed);
        let success = self.requests_success.load(Ordering::Relaxed);
        let failed = self.requests_failed.load(Ordering::Relaxed);
        let sum_ms = self.latency_sum_ms.load(Ordering::Relaxed);
        let max_ms = self.latency_max_ms.load(Ordering::Relaxed);

        let avg = if success > 0 {
            sum_ms as f64 / success as f64
        } else {
            0.0
        };

        ConnectorMetrics {
            connector_name: self.name.clone(),
            requests_total: total,
            requests_success: success,
            requests_failed: failed,
            avg_latency_ms: avg,
            p99_latency_ms: max_ms as f64,
            circuit_state: self.circuit_breaker.state(),
            dlq_depth: self.dlq.depth(),
            last_success_at: *self.last_success_at.lock(),
            last_failure_at: *self.last_failure_at.lock(),
        }
    }
}

/// Registry of all connector runtimes.
pub struct ConnectorRegistry {
    connectors: DashMap<String, ConnectorRuntime>,
}

impl Default for ConnectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectorRegistry {
    pub fn new() -> Self {
        Self {
            connectors: DashMap::new(),
        }
    }

    /// Register a new connector runtime.
    pub fn register(
        &self,
        name: String,
        cb_config: CircuitBreakerConfig,
        retry: RetryPolicy,
    ) -> bool {
        if self.connectors.contains_key(&name) {
            return false;
        }
        let runtime = ConnectorRuntime::new(name.clone(), cb_config, retry);
        self.connectors.insert(name, runtime);
        true
    }

    /// Get metrics for all registered connectors.
    pub fn all_metrics(&self) -> Vec<ConnectorMetrics> {
        self.connectors
            .iter()
            .map(|e| e.value().metrics())
            .collect()
    }

    /// Access a connector runtime by name.
    pub fn get(
        &self,
        name: &str,
    ) -> Option<dashmap::mapref::one::Ref<'_, String, ConnectorRuntime>> {
        self.connectors.get(name)
    }

    /// Seed demo connectors.
    pub fn seed_demo(&self) {
        let connectors = ["salesforce", "adobe", "segment", "tealium", "hightouch"];
        for name in connectors {
            self.register(
                name.to_string(),
                CircuitBreakerConfig::default(),
                RetryPolicy::default(),
            );
        }
        info!(
            count = connectors.len(),
            "demo connector runtimes registered"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_lifecycle() {
        let cb = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 3,
            open_duration_secs: 0, // instant recovery for test
            half_open_successes: 2,
        });

        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());

        // Trigger failures
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure(); // 3rd failure -> open
        assert_eq!(cb.state(), CircuitState::Open);

        // After open_duration=0, should transition to half-open
        assert!(cb.allow_request());
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Successes in half-open
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.record_success(); // 2nd -> closed
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_retry_backoff() {
        let policy = RetryPolicy {
            max_retries: 5,
            initial_backoff_ms: 100,
            max_backoff_ms: 5000,
            backoff_multiplier: 2.0,
            jitter: false,
        };

        assert_eq!(policy.backoff_for_attempt(0), Duration::from_millis(100));
        assert_eq!(policy.backoff_for_attempt(1), Duration::from_millis(200));
        assert_eq!(policy.backoff_for_attempt(2), Duration::from_millis(400));
        assert_eq!(policy.backoff_for_attempt(5), Duration::from_millis(3200));
    }

    #[test]
    fn test_dlq_lifecycle() {
        let dlq = DeadLetterQueue::new(3);
        let now = Utc::now();

        for i in 0..4 {
            dlq.enqueue(DeadLetterRecord {
                id: Uuid::new_v4(),
                connector_name: "test".to_string(),
                payload: serde_json::json!({"i": i}),
                error: "fail".to_string(),
                attempt_count: 3,
                first_failed_at: now,
                last_failed_at: now,
                retryable: true,
            });
        }

        // Max size 3, so oldest was evicted
        assert_eq!(dlq.depth(), 3);

        let record = dlq.dequeue().unwrap();
        assert_eq!(record.payload["i"], 1); // index 0 was evicted
        assert_eq!(dlq.depth(), 2);

        let metrics = dlq.metrics();
        assert_eq!(metrics.total_enqueued, 4);
        assert_eq!(metrics.total_replayed, 1);
    }

    #[test]
    fn test_connector_runtime_metrics() {
        let runtime = ConnectorRuntime::new(
            "test-conn".to_string(),
            CircuitBreakerConfig::default(),
            RetryPolicy::default(),
        );

        runtime.record_success(10);
        runtime.record_success(20);
        runtime.record_failure("timeout", None, 0);

        let m = runtime.metrics();
        assert_eq!(m.requests_total, 3);
        assert_eq!(m.requests_success, 2);
        assert_eq!(m.requests_failed, 1);
        assert_eq!(m.avg_latency_ms, 15.0);
        assert_eq!(m.connector_name, "test-conn");
    }
}
