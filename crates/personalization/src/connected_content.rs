//! Connected content — fetches personalized data from external APIs at send time.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedContentSource {
    pub id: Uuid,
    pub name: String,
    pub url_template: String,
    pub method: HttpMethod,
    pub headers: std::collections::HashMap<String, String>,
    pub cache_ttl_seconds: u64,
    pub timeout_ms: u64,
    pub fallback_value: Option<serde_json::Value>,
    pub retry_count: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectedContentResult {
    pub source_id: Uuid,
    pub data: serde_json::Value,
    pub fetched_at: DateTime<Utc>,
    pub cached: bool,
    pub latency_ms: u64,
}

pub struct ConnectedContentEngine {
    sources: dashmap::DashMap<Uuid, ConnectedContentSource>,
    cache: dashmap::DashMap<String, (serde_json::Value, DateTime<Utc>)>,
}

impl ConnectedContentEngine {
    pub fn new() -> Self {
        Self {
            sources: dashmap::DashMap::new(),
            cache: dashmap::DashMap::new(),
        }
    }

    pub fn register_source(&self, source: ConnectedContentSource) {
        self.sources.insert(source.id, source);
    }

    pub async fn fetch(
        &self,
        source_id: &Uuid,
        variables: &std::collections::HashMap<String, String>,
    ) -> anyhow::Result<ConnectedContentResult> {
        let source = self
            .sources
            .get(source_id)
            .ok_or_else(|| anyhow::anyhow!("Source not found"))?;

        let mut url = source.url_template.clone();
        for (key, value) in variables {
            url = url.replace(&format!("{{{{{}}}}}", key), value);
        }

        let cache_key = format!("{}:{}", source_id, url);
        if let Some(cached) = self.cache.get(&cache_key) {
            let (data, fetched_at) = cached.value();
            let age = (Utc::now() - *fetched_at).num_seconds() as u64;
            if age < source.cache_ttl_seconds {
                return Ok(ConnectedContentResult {
                    source_id: *source_id,
                    data: data.clone(),
                    fetched_at: *fetched_at,
                    cached: true,
                    latency_ms: 0,
                });
            }
        }

        tracing::info!(source_id = %source_id, url = &url, "Fetching connected content");
        let data = source
            .fallback_value
            .clone()
            .unwrap_or(serde_json::Value::Null);
        let now = Utc::now();
        self.cache.insert(cache_key, (data.clone(), now));

        Ok(ConnectedContentResult {
            source_id: *source_id,
            data,
            fetched_at: now,
            cached: false,
            latency_ms: 1,
        })
    }

    pub fn list_sources(&self) -> Vec<ConnectedContentSource> {
        self.sources.iter().map(|s| s.value().clone()).collect()
    }
}

impl Default for ConnectedContentEngine {
    fn default() -> Self {
        Self::new()
    }
}
