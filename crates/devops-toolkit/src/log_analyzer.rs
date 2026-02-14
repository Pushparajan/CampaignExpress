//! Log analyzer — detects error patterns, scores anomalies, and correlates
//! events across services for rapid root-cause analysis.

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Log severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

/// A structured log entry for analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub service: String,
    pub message: String,
    pub node_id: Option<String>,
    pub request_id: Option<String>,
    pub trace_id: Option<String>,
}

/// A detected log pattern (recurring error/warning).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub pattern_id: String,
    pub signature: String,
    pub level: LogLevel,
    pub services: Vec<String>,
    pub count: u64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub rate_per_minute: f64,
    pub is_anomaly: bool,
    pub suggested_action: String,
}

/// Known error signature to match against.
#[derive(Debug, Clone)]
pub struct ErrorSignature {
    pub pattern: String,
    pub category: String,
    pub suggested_action: String,
    pub anomaly_threshold_per_min: f64,
}

/// Log analysis report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogAnalysisReport {
    pub total_entries_analyzed: u64,
    pub error_count: u64,
    pub warn_count: u64,
    pub patterns: Vec<DetectedPattern>,
    pub anomalies: Vec<DetectedPattern>,
    pub top_error_services: Vec<(String, u64)>,
    pub error_rate_per_minute: f64,
    pub analysis_window: String,
    pub generated_at: DateTime<Utc>,
}

/// Log analyzer that detects patterns and anomalies.
pub struct LogAnalyzer {
    entries: Vec<LogEntry>,
    known_signatures: Vec<ErrorSignature>,
    pattern_counts: DashMap<String, (u64, DateTime<Utc>, DateTime<Utc>)>,
    max_entries: usize,
}

impl LogAnalyzer {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            known_signatures: Self::default_signatures(),
            pattern_counts: DashMap::new(),
            max_entries,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(100_000)
    }

    /// Ingest a log entry for analysis.
    pub fn ingest(&mut self, entry: LogEntry) {
        // Update pattern counts for error/warn
        if entry.level >= LogLevel::Warn {
            let sig = self.extract_signature(&entry.message);
            let now = entry.timestamp;
            let mut count = self.pattern_counts.entry(sig).or_insert((0, now, now));
            count.0 += 1;
            if now < count.1 {
                count.1 = now;
            }
            if now > count.2 {
                count.2 = now;
            }
        }

        self.entries.push(entry);

        // Evict oldest entries when buffer is full
        if self.entries.len() > self.max_entries {
            self.entries.drain(..self.entries.len() / 2);
        }
    }

    /// Ingest a batch of log entries.
    pub fn ingest_batch(&mut self, entries: Vec<LogEntry>) {
        for entry in entries {
            self.ingest(entry);
        }
    }

    /// Run full analysis on ingested logs.
    pub fn analyze(&self, window_minutes: u64) -> LogAnalysisReport {
        let cutoff = Utc::now() - Duration::minutes(window_minutes as i64);
        let window_entries: Vec<&LogEntry> = self
            .entries
            .iter()
            .filter(|e| e.timestamp >= cutoff)
            .collect();

        let total = window_entries.len() as u64;
        let error_count = window_entries
            .iter()
            .filter(|e| e.level >= LogLevel::Error)
            .count() as u64;
        let warn_count = window_entries
            .iter()
            .filter(|e| e.level == LogLevel::Warn)
            .count() as u64;

        // Detect patterns
        let mut patterns: Vec<DetectedPattern> = self
            .pattern_counts
            .iter()
            .map(|entry| {
                let sig = entry.key().clone();
                let (count, first_seen, last_seen) = *entry.value();
                let duration_mins = (last_seen - first_seen).num_minutes().max(1) as f64;
                let rate = count as f64 / duration_mins;

                let is_anomaly = self
                    .known_signatures
                    .iter()
                    .find(|ks| sig.contains(&ks.pattern))
                    .is_some_and(|ks| rate > ks.anomaly_threshold_per_min);

                let suggested_action = self
                    .known_signatures
                    .iter()
                    .find(|ks| sig.contains(&ks.pattern))
                    .map(|ks| ks.suggested_action.clone())
                    .unwrap_or_else(|| "Investigate log pattern".into());

                let services = self.find_services_for_pattern(&sig);

                DetectedPattern {
                    pattern_id: format!("pat-{:x}", sig.len() + count as usize),
                    signature: sig,
                    level: LogLevel::Error,
                    services,
                    count,
                    first_seen,
                    last_seen,
                    rate_per_minute: rate,
                    is_anomaly,
                    suggested_action,
                }
            })
            .collect();

        patterns.sort_by(|a, b| b.count.cmp(&a.count));

        let anomalies: Vec<DetectedPattern> =
            patterns.iter().filter(|p| p.is_anomaly).cloned().collect();

        // Top error services
        let mut service_errors: std::collections::HashMap<String, u64> =
            std::collections::HashMap::new();
        for entry in &window_entries {
            if entry.level >= LogLevel::Error {
                *service_errors.entry(entry.service.clone()).or_default() += 1;
            }
        }
        let mut top_error_services: Vec<(String, u64)> = service_errors.into_iter().collect();
        top_error_services.sort_by(|a, b| b.1.cmp(&a.1));
        top_error_services.truncate(10);

        let error_rate_per_minute = if window_minutes > 0 {
            error_count as f64 / window_minutes as f64
        } else {
            0.0
        };

        LogAnalysisReport {
            total_entries_analyzed: total,
            error_count,
            warn_count,
            patterns,
            anomalies,
            top_error_services,
            error_rate_per_minute,
            analysis_window: format!("{}m", window_minutes),
            generated_at: Utc::now(),
        }
    }

    fn extract_signature(&self, message: &str) -> String {
        // Normalize the message: remove UUIDs, numbers, IPs
        let mut sig = message.to_string();
        // Remove UUIDs
        while let Some(start) = sig.find(|c: char| c.is_ascii_hexdigit()) {
            let end = sig[start..]
                .find(|c: char| !c.is_ascii_hexdigit() && c != '-')
                .map(|e| start + e)
                .unwrap_or(sig.len());
            if end - start > 8 {
                sig.replace_range(start..end, "<ID>");
            } else {
                break;
            }
        }
        // Truncate to first 80 chars for grouping
        if sig.len() > 80 {
            sig.truncate(80);
        }
        sig
    }

    fn find_services_for_pattern(&self, sig: &str) -> Vec<String> {
        let mut services: Vec<String> = self
            .entries
            .iter()
            .filter(|e| e.message.contains(sig.split("<ID>").next().unwrap_or("")))
            .map(|e| e.service.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        services.sort();
        services.truncate(5);
        services
    }

    fn default_signatures() -> Vec<ErrorSignature> {
        vec![
            ErrorSignature {
                pattern: "connection refused".into(),
                category: "connectivity".into(),
                suggested_action: "Check target service health, verify network policies".into(),
                anomaly_threshold_per_min: 5.0,
            },
            ErrorSignature {
                pattern: "timeout".into(),
                category: "latency".into(),
                suggested_action: "Check service latency, increase timeout, scale up".into(),
                anomaly_threshold_per_min: 10.0,
            },
            ErrorSignature {
                pattern: "out of memory".into(),
                category: "resource".into(),
                suggested_action: "Increase memory limits, check for leaks, restart pod".into(),
                anomaly_threshold_per_min: 1.0,
            },
            ErrorSignature {
                pattern: "redis".into(),
                category: "cache".into(),
                suggested_action: "Check Redis health, connection pool, memory usage".into(),
                anomaly_threshold_per_min: 5.0,
            },
            ErrorSignature {
                pattern: "clickhouse".into(),
                category: "analytics".into(),
                suggested_action: "Check ClickHouse load, disk space, merge queue".into(),
                anomaly_threshold_per_min: 5.0,
            },
            ErrorSignature {
                pattern: "npu".into(),
                category: "inference".into(),
                suggested_action: "Check NPU device, reload model, verify ONNX".into(),
                anomaly_threshold_per_min: 3.0,
            },
            ErrorSignature {
                pattern: "permission denied".into(),
                category: "security".into(),
                suggested_action: "Check RBAC, service account, network policies".into(),
                anomaly_threshold_per_min: 2.0,
            },
            ErrorSignature {
                pattern: "disk full".into(),
                category: "storage".into(),
                suggested_action: "Expand PVC, clean old data, check retention policy".into(),
                anomaly_threshold_per_min: 1.0,
            },
            ErrorSignature {
                pattern: "rate limit".into(),
                category: "throttling".into(),
                suggested_action: "Review rate limit config, check for abuse, scale out".into(),
                anomaly_threshold_per_min: 10.0,
            },
            ErrorSignature {
                pattern: "certificate".into(),
                category: "tls".into(),
                suggested_action: "Check cert expiry, renew via cert-manager, verify CA".into(),
                anomaly_threshold_per_min: 2.0,
            },
        ]
    }

    /// Clear all ingested data.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.pattern_counts.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(level: LogLevel, service: &str, msg: &str) -> LogEntry {
        LogEntry {
            timestamp: Utc::now(),
            level,
            service: service.into(),
            message: msg.into(),
            node_id: Some("node-1".into()),
            request_id: None,
            trace_id: None,
        }
    }

    #[test]
    fn test_ingest_and_analyze() {
        let mut analyzer = LogAnalyzer::with_defaults();
        analyzer.ingest(make_entry(LogLevel::Info, "api", "Request handled"));
        analyzer.ingest(make_entry(
            LogLevel::Error,
            "cache",
            "redis connection refused",
        ));
        analyzer.ingest(make_entry(
            LogLevel::Error,
            "cache",
            "redis connection refused",
        ));
        analyzer.ingest(make_entry(LogLevel::Warn, "npu", "inference timeout"));

        let report = analyzer.analyze(60);
        assert_eq!(report.total_entries_analyzed, 4);
        assert_eq!(report.error_count, 2);
        assert_eq!(report.warn_count, 1);
        assert!(!report.patterns.is_empty());
    }

    #[test]
    fn test_top_error_services() {
        let mut analyzer = LogAnalyzer::with_defaults();
        for _ in 0..10 {
            analyzer.ingest(make_entry(LogLevel::Error, "api-server", "500 error"));
        }
        for _ in 0..3 {
            analyzer.ingest(make_entry(LogLevel::Error, "cache", "redis error"));
        }

        let report = analyzer.analyze(60);
        assert_eq!(report.top_error_services[0].0, "api-server");
        assert_eq!(report.top_error_services[0].1, 10);
    }

    #[test]
    fn test_buffer_eviction() {
        let mut analyzer = LogAnalyzer::new(100);
        for i in 0..200 {
            analyzer.ingest(make_entry(LogLevel::Info, "api", &format!("entry {i}")));
        }
        assert!(analyzer.entries.len() <= 150);
    }
}
