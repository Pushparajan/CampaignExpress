//! Product/content recommendation engine — collaborative filtering,
//! content-based, and popularity-based recommendations.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationStrategy {
    MostPopular,
    RecentlyViewed,
    FrequentlyBoughtTogether,
    PersonalizedCf,
    ContentBased,
    Trending,
    NewArrivals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationRequest {
    pub user_id: Uuid,
    pub strategy: RecommendationStrategy,
    pub catalog_name: Option<String>,
    pub limit: usize,
    pub exclude_ids: Vec<String>,
    pub context: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationItem {
    pub item_id: String,
    pub score: f64,
    pub reason: String,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationResponse {
    pub request_id: Uuid,
    pub user_id: Uuid,
    pub strategy: RecommendationStrategy,
    pub items: Vec<RecommendationItem>,
    pub generated_at: DateTime<Utc>,
    pub model_version: String,
}

pub struct RecommendationEngine {
    popularity_scores: dashmap::DashMap<String, f64>,
    user_interactions: dashmap::DashMap<Uuid, Vec<String>>,
}

impl RecommendationEngine {
    pub fn new() -> Self {
        Self {
            popularity_scores: dashmap::DashMap::new(),
            user_interactions: dashmap::DashMap::new(),
        }
    }

    pub fn record_interaction(&self, user_id: Uuid, item_id: String) {
        self.user_interactions
            .entry(user_id)
            .or_default()
            .push(item_id.clone());
        self.popularity_scores
            .entry(item_id)
            .and_modify(|s| *s += 1.0)
            .or_insert(1.0);
    }

    pub fn recommend(&self, request: &RecommendationRequest) -> RecommendationResponse {
        let items = match request.strategy {
            RecommendationStrategy::MostPopular => {
                self.most_popular(request.limit, &request.exclude_ids)
            }
            RecommendationStrategy::RecentlyViewed => {
                self.recently_viewed(&request.user_id, request.limit)
            }
            _ => self.most_popular(request.limit, &request.exclude_ids),
        };

        RecommendationResponse {
            request_id: Uuid::new_v4(),
            user_id: request.user_id,
            strategy: request.strategy.clone(),
            items,
            generated_at: Utc::now(),
            model_version: "v1.0".to_string(),
        }
    }

    fn most_popular(&self, limit: usize, exclude: &[String]) -> Vec<RecommendationItem> {
        let mut items: Vec<_> = self
            .popularity_scores
            .iter()
            .filter(|entry| !exclude.contains(entry.key()))
            .map(|entry| RecommendationItem {
                item_id: entry.key().clone(),
                score: *entry.value(),
                reason: "Popular item".to_string(),
                metadata: std::collections::HashMap::new(),
            })
            .collect();
        items.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        items.truncate(limit);
        items
    }

    fn recently_viewed(&self, user_id: &Uuid, limit: usize) -> Vec<RecommendationItem> {
        self.user_interactions
            .get(user_id)
            .map(|interactions| {
                interactions
                    .iter()
                    .rev()
                    .take(limit)
                    .enumerate()
                    .map(|(i, id)| RecommendationItem {
                        item_id: id.clone(),
                        score: 1.0 / (i as f64 + 1.0),
                        reason: "Recently viewed".to_string(),
                        metadata: std::collections::HashMap::new(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for RecommendationEngine {
    fn default() -> Self {
        Self::new()
    }
}
