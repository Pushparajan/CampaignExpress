//! Product/content recommendation engine — collaborative filtering,
//! content-based, and popularity-based recommendations.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    pub context: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationItem {
    pub item_id: String,
    pub score: f64,
    pub reason: String,
    pub metadata: HashMap<String, serde_json::Value>,
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
    popularity_scores: DashMap<String, f64>,
    user_interactions: DashMap<Uuid, Vec<String>>,
    /// Co-occurrence matrix: item_a -> { item_b -> score }.
    /// Items interacted with by the same user increment co-occurrence by 1.0.
    item_cooccurrence: DashMap<String, DashMap<String, f64>>,
    /// Feature vectors for items used in content-based filtering.
    item_features: DashMap<String, HashMap<String, f64>>,
    /// Timestamps of interactions per item, used for trending calculation.
    interaction_timestamps: DashMap<String, Vec<DateTime<Utc>>>,
    /// Creation timestamps for items, used for new-arrivals ranking.
    item_created_at: DashMap<String, DateTime<Utc>>,
}

impl RecommendationEngine {
    pub fn new() -> Self {
        Self {
            popularity_scores: DashMap::new(),
            user_interactions: DashMap::new(),
            item_cooccurrence: DashMap::new(),
            item_features: DashMap::new(),
            interaction_timestamps: DashMap::new(),
            item_created_at: DashMap::new(),
        }
    }

    /// Record a user-item interaction. Updates popularity scores, user history,
    /// co-occurrence matrix, and interaction timestamps.
    pub fn record_interaction(&self, user_id: Uuid, item_id: String) {
        // Update popularity
        self.popularity_scores
            .entry(item_id.clone())
            .and_modify(|s| *s += 1.0)
            .or_insert(1.0);

        // Record interaction timestamp
        self.interaction_timestamps
            .entry(item_id.clone())
            .or_default()
            .push(Utc::now());

        // Get existing items for this user before pushing the new one,
        // so we can update co-occurrence with all previous items.
        let previous_items: Vec<String> = self
            .user_interactions
            .get(&user_id)
            .map(|v| v.clone())
            .unwrap_or_default();

        // Update co-occurrence: the new item co-occurs with every item
        // already in the user's history (bidirectional).
        for existing_item in &previous_items {
            if *existing_item == item_id {
                continue;
            }
            // item_id <-> existing_item
            self.item_cooccurrence
                .entry(item_id.clone())
                .or_default()
                .entry(existing_item.clone())
                .and_modify(|s| *s += 1.0)
                .or_insert(1.0);
            self.item_cooccurrence
                .entry(existing_item.clone())
                .or_default()
                .entry(item_id.clone())
                .and_modify(|s| *s += 1.0)
                .or_insert(1.0);
        }

        // Add to user interaction history
        self.user_interactions
            .entry(user_id)
            .or_default()
            .push(item_id);
    }

    /// Register feature vector for an item (used by content-based filtering).
    pub fn set_item_features(&self, item_id: String, features: HashMap<String, f64>) {
        self.item_features.insert(item_id, features);
    }

    /// Register an item with its creation timestamp (used by new-arrivals).
    pub fn register_item(&self, item_id: String, created_at: DateTime<Utc>) {
        self.item_created_at.insert(item_id, created_at);
    }

    pub fn recommend(&self, request: &RecommendationRequest) -> RecommendationResponse {
        let items = match request.strategy {
            RecommendationStrategy::MostPopular => {
                self.most_popular(request.limit, &request.exclude_ids)
            }
            RecommendationStrategy::RecentlyViewed => {
                self.recently_viewed(&request.user_id, request.limit)
            }
            RecommendationStrategy::PersonalizedCf => {
                self.personalized_cf(&request.user_id, request.limit, &request.exclude_ids)
            }
            RecommendationStrategy::ContentBased => {
                self.content_based(&request.user_id, request.limit, &request.exclude_ids)
            }
            RecommendationStrategy::FrequentlyBoughtTogether => self.frequently_bought_together(
                &request.user_id,
                request.limit,
                &request.exclude_ids,
            ),
            RecommendationStrategy::Trending => self.trending(request.limit, &request.exclude_ids),
            RecommendationStrategy::NewArrivals => {
                self.new_arrivals(request.limit, &request.exclude_ids)
            }
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
                metadata: HashMap::new(),
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
                        metadata: HashMap::new(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Personalized collaborative filtering: find items that co-occur with
    /// the user's past interactions, ranked by aggregate co-occurrence score.
    /// Items the user has already interacted with are excluded.
    fn personalized_cf(
        &self,
        user_id: &Uuid,
        limit: usize,
        exclude: &[String],
    ) -> Vec<RecommendationItem> {
        let user_items = match self.user_interactions.get(user_id) {
            Some(items) => items.clone(),
            None => return Vec::new(),
        };

        let user_item_set: std::collections::HashSet<&String> = user_items.iter().collect();

        // Aggregate co-occurrence scores for candidate items
        let mut candidate_scores: HashMap<String, f64> = HashMap::new();
        for item in &user_items {
            if let Some(cooccurrences) = self.item_cooccurrence.get(item) {
                for entry in cooccurrences.iter() {
                    let candidate = entry.key().clone();
                    let score = *entry.value();
                    if !user_item_set.contains(&candidate) && !exclude.contains(&candidate) {
                        *candidate_scores.entry(candidate).or_insert(0.0) += score;
                    }
                }
            }
        }

        let mut items: Vec<RecommendationItem> = candidate_scores
            .into_iter()
            .map(|(item_id, score)| RecommendationItem {
                item_id,
                score,
                reason: "Users who interacted with X also liked this".to_string(),
                metadata: HashMap::new(),
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

    /// Content-based filtering: compute cosine similarity between the user's
    /// average feature vector (from past interactions) and all item feature vectors.
    fn content_based(
        &self,
        user_id: &Uuid,
        limit: usize,
        exclude: &[String],
    ) -> Vec<RecommendationItem> {
        let user_items = match self.user_interactions.get(user_id) {
            Some(items) => items.clone(),
            None => return Vec::new(),
        };

        let user_item_set: std::collections::HashSet<&String> = user_items.iter().collect();

        // Build the user's average feature vector from interacted items
        let mut user_vector: HashMap<String, f64> = HashMap::new();
        let mut count = 0usize;
        for item_id in &user_items {
            if let Some(features) = self.item_features.get(item_id) {
                for (key, val) in features.iter() {
                    *user_vector.entry(key.clone()).or_insert(0.0) += val;
                }
                count += 1;
            }
        }

        if count == 0 {
            return Vec::new();
        }

        // Average the vector
        for val in user_vector.values_mut() {
            *val /= count as f64;
        }

        let user_magnitude = vector_magnitude(&user_vector);
        if user_magnitude == 0.0 {
            return Vec::new();
        }

        // Score all items by cosine similarity
        let mut items: Vec<RecommendationItem> = self
            .item_features
            .iter()
            .filter(|entry| !user_item_set.contains(entry.key()) && !exclude.contains(entry.key()))
            .map(|entry| {
                let item_id = entry.key().clone();
                let item_vec = entry.value();
                let dot = dot_product(&user_vector, item_vec);
                let item_mag = vector_magnitude(item_vec);
                let similarity = if item_mag > 0.0 {
                    dot / (user_magnitude * item_mag)
                } else {
                    0.0
                };
                RecommendationItem {
                    item_id,
                    score: similarity,
                    reason: "Similar to items you've interacted with".to_string(),
                    metadata: HashMap::new(),
                }
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

    /// Frequently bought together: given the user's most recent interaction,
    /// find the top co-occurring items.
    fn frequently_bought_together(
        &self,
        user_id: &Uuid,
        limit: usize,
        exclude: &[String],
    ) -> Vec<RecommendationItem> {
        let recent_item = match self.user_interactions.get(user_id) {
            Some(items) if !items.is_empty() => items.last().unwrap().clone(),
            _ => return Vec::new(),
        };

        let user_items: std::collections::HashSet<String> = self
            .user_interactions
            .get(user_id)
            .map(|v| v.iter().cloned().collect())
            .unwrap_or_default();

        let cooccurrences = match self.item_cooccurrence.get(&recent_item) {
            Some(c) => c,
            None => return Vec::new(),
        };

        let mut items: Vec<RecommendationItem> = cooccurrences
            .iter()
            .filter(|entry| !user_items.contains(entry.key()) && !exclude.contains(entry.key()))
            .map(|entry| RecommendationItem {
                item_id: entry.key().clone(),
                score: *entry.value(),
                reason: format!("Frequently bought together with {}", recent_item),
                metadata: HashMap::new(),
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

    /// Trending: items with the highest interaction count in the last 24 hours,
    /// with an exponential decay factor favoring more recent interactions.
    fn trending(&self, limit: usize, exclude: &[String]) -> Vec<RecommendationItem> {
        let now = Utc::now();
        let window = chrono::Duration::hours(24);
        let cutoff = now - window;
        let decay_half_life_secs: f64 = 6.0 * 3600.0; // 6 hours

        let mut items: Vec<RecommendationItem> = self
            .interaction_timestamps
            .iter()
            .filter(|entry| !exclude.contains(entry.key()))
            .filter_map(|entry| {
                let item_id = entry.key().clone();
                let timestamps = entry.value();

                let score: f64 = timestamps
                    .iter()
                    .filter(|ts| **ts >= cutoff)
                    .map(|ts| {
                        let age_secs = (now - *ts).num_seconds().max(0) as f64;
                        // Exponential decay: more recent interactions contribute more
                        (-age_secs * (2.0_f64.ln()) / decay_half_life_secs).exp()
                    })
                    .sum();

                if score > 0.0 {
                    Some(RecommendationItem {
                        item_id,
                        score,
                        reason: "Trending now".to_string(),
                        metadata: HashMap::new(),
                    })
                } else {
                    None
                }
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

    /// New arrivals: items sorted by creation date, newest first.
    fn new_arrivals(&self, limit: usize, exclude: &[String]) -> Vec<RecommendationItem> {
        let mut items: Vec<(String, DateTime<Utc>)> = self
            .item_created_at
            .iter()
            .filter(|entry| !exclude.contains(entry.key()))
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();

        items.sort_by(|a, b| b.1.cmp(&a.1));
        items.truncate(limit);

        items
            .into_iter()
            .enumerate()
            .map(|(i, (item_id, _created))| RecommendationItem {
                item_id,
                score: 1.0 / (i as f64 + 1.0),
                reason: "New arrival".to_string(),
                metadata: HashMap::new(),
            })
            .collect()
    }
}

impl Default for RecommendationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute the dot product of two sparse feature vectors.
fn dot_product(a: &HashMap<String, f64>, b: &HashMap<String, f64>) -> f64 {
    a.iter()
        .filter_map(|(key, val_a)| b.get(key).map(|val_b| val_a * val_b))
        .sum()
}

/// Compute the L2 magnitude of a sparse feature vector.
fn vector_magnitude(v: &HashMap<String, f64>) -> f64 {
    v.values().map(|x| x * x).sum::<f64>().sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_most_popular() {
        let engine = RecommendationEngine::new();
        let user = Uuid::new_v4();
        engine.record_interaction(user, "item_a".to_string());
        engine.record_interaction(user, "item_a".to_string());
        engine.record_interaction(user, "item_b".to_string());

        let req = RecommendationRequest {
            user_id: Uuid::new_v4(),
            strategy: RecommendationStrategy::MostPopular,
            catalog_name: None,
            limit: 10,
            exclude_ids: vec![],
            context: HashMap::new(),
        };
        let resp = engine.recommend(&req);
        assert!(!resp.items.is_empty());
        // item_a has 2 interactions, item_b has 1
        assert_eq!(resp.items[0].item_id, "item_a");
        assert!(resp.items[0].score > resp.items[1].score);
    }

    #[test]
    fn test_recently_viewed() {
        let engine = RecommendationEngine::new();
        let user = Uuid::new_v4();
        engine.record_interaction(user, "item_a".to_string());
        engine.record_interaction(user, "item_b".to_string());
        engine.record_interaction(user, "item_c".to_string());

        let req = RecommendationRequest {
            user_id: user,
            strategy: RecommendationStrategy::RecentlyViewed,
            catalog_name: None,
            limit: 2,
            exclude_ids: vec![],
            context: HashMap::new(),
        };
        let resp = engine.recommend(&req);
        assert_eq!(resp.items.len(), 2);
        // Most recent first
        assert_eq!(resp.items[0].item_id, "item_c");
        assert_eq!(resp.items[1].item_id, "item_b");
    }

    #[test]
    fn test_personalized_cf() {
        let engine = RecommendationEngine::new();
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();

        // User1 interacts with A and B
        engine.record_interaction(user1, "item_a".to_string());
        engine.record_interaction(user1, "item_b".to_string());

        // User2 interacts with A and C (A co-occurs with C via user2)
        engine.record_interaction(user2, "item_a".to_string());
        engine.record_interaction(user2, "item_c".to_string());

        // For user1 who has [A, B]: CF should recommend C (co-occurs with A)
        let req = RecommendationRequest {
            user_id: user1,
            strategy: RecommendationStrategy::PersonalizedCf,
            catalog_name: None,
            limit: 5,
            exclude_ids: vec![],
            context: HashMap::new(),
        };
        let resp = engine.recommend(&req);
        assert!(!resp.items.is_empty());
        assert_eq!(resp.items[0].item_id, "item_c");
        assert_eq!(
            resp.items[0].reason,
            "Users who interacted with X also liked this"
        );
    }

    #[test]
    fn test_content_based() {
        let engine = RecommendationEngine::new();
        let user = Uuid::new_v4();

        // Set features: item_a and item_c are similar (both high on "genre_action")
        let mut features_a = HashMap::new();
        features_a.insert("genre_action".to_string(), 1.0);
        features_a.insert("genre_comedy".to_string(), 0.0);
        engine.set_item_features("item_a".to_string(), features_a);

        let mut features_b = HashMap::new();
        features_b.insert("genre_action".to_string(), 0.0);
        features_b.insert("genre_comedy".to_string(), 1.0);
        engine.set_item_features("item_b".to_string(), features_b);

        let mut features_c = HashMap::new();
        features_c.insert("genre_action".to_string(), 0.9);
        features_c.insert("genre_comedy".to_string(), 0.1);
        engine.set_item_features("item_c".to_string(), features_c);

        // User interacts with item_a (action)
        engine.record_interaction(user, "item_a".to_string());

        let req = RecommendationRequest {
            user_id: user,
            strategy: RecommendationStrategy::ContentBased,
            catalog_name: None,
            limit: 5,
            exclude_ids: vec![],
            context: HashMap::new(),
        };
        let resp = engine.recommend(&req);
        assert!(!resp.items.is_empty());
        // item_c should rank higher than item_b (more similar to item_a)
        assert_eq!(resp.items[0].item_id, "item_c");
        assert!(resp.items[0].score > resp.items[1].score);
    }

    #[test]
    fn test_frequently_bought_together() {
        let engine = RecommendationEngine::new();
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();

        // Both users buy item_a and item_b together
        engine.record_interaction(user1, "item_a".to_string());
        engine.record_interaction(user1, "item_b".to_string());
        engine.record_interaction(user2, "item_a".to_string());
        engine.record_interaction(user2, "item_b".to_string());

        // New user interacts with item_a
        let user3 = Uuid::new_v4();
        engine.record_interaction(user3, "item_a".to_string());

        let req = RecommendationRequest {
            user_id: user3,
            strategy: RecommendationStrategy::FrequentlyBoughtTogether,
            catalog_name: None,
            limit: 5,
            exclude_ids: vec![],
            context: HashMap::new(),
        };
        let resp = engine.recommend(&req);
        assert!(!resp.items.is_empty());
        assert_eq!(resp.items[0].item_id, "item_b");
        assert!(resp.items[0]
            .reason
            .starts_with("Frequently bought together with"));
    }

    #[test]
    fn test_trending() {
        let engine = RecommendationEngine::new();
        let user1 = Uuid::new_v4();
        let user2 = Uuid::new_v4();
        let user3 = Uuid::new_v4();

        // Multiple recent interactions with item_a
        engine.record_interaction(user1, "item_a".to_string());
        engine.record_interaction(user2, "item_a".to_string());
        engine.record_interaction(user3, "item_a".to_string());

        // Single interaction with item_b
        engine.record_interaction(user1, "item_b".to_string());

        let req = RecommendationRequest {
            user_id: Uuid::new_v4(),
            strategy: RecommendationStrategy::Trending,
            catalog_name: None,
            limit: 5,
            exclude_ids: vec![],
            context: HashMap::new(),
        };
        let resp = engine.recommend(&req);
        assert!(!resp.items.is_empty());
        // item_a should be trending higher (3 interactions vs 1)
        assert_eq!(resp.items[0].item_id, "item_a");
        assert_eq!(resp.items[0].reason, "Trending now");
    }

    #[test]
    fn test_new_arrivals() {
        let engine = RecommendationEngine::new();

        let old_time = Utc::now() - chrono::Duration::days(30);
        let new_time = Utc::now() - chrono::Duration::hours(1);

        engine.register_item("old_item".to_string(), old_time);
        engine.register_item("new_item".to_string(), new_time);

        let req = RecommendationRequest {
            user_id: Uuid::new_v4(),
            strategy: RecommendationStrategy::NewArrivals,
            catalog_name: None,
            limit: 5,
            exclude_ids: vec![],
            context: HashMap::new(),
        };
        let resp = engine.recommend(&req);
        assert_eq!(resp.items.len(), 2);
        assert_eq!(resp.items[0].item_id, "new_item");
        assert_eq!(resp.items[1].item_id, "old_item");
        assert_eq!(resp.items[0].reason, "New arrival");
    }

    #[test]
    fn test_exclude_ids() {
        let engine = RecommendationEngine::new();
        let user = Uuid::new_v4();
        engine.record_interaction(user, "item_a".to_string());
        engine.record_interaction(user, "item_b".to_string());

        let req = RecommendationRequest {
            user_id: Uuid::new_v4(),
            strategy: RecommendationStrategy::MostPopular,
            catalog_name: None,
            limit: 10,
            exclude_ids: vec!["item_a".to_string()],
            context: HashMap::new(),
        };
        let resp = engine.recommend(&req);
        assert_eq!(resp.items.len(), 1);
        assert_eq!(resp.items[0].item_id, "item_b");
    }

    #[test]
    fn test_default_impl() {
        let engine = RecommendationEngine::default();
        let req = RecommendationRequest {
            user_id: Uuid::new_v4(),
            strategy: RecommendationStrategy::MostPopular,
            catalog_name: None,
            limit: 10,
            exclude_ids: vec![],
            context: HashMap::new(),
        };
        let resp = engine.recommend(&req);
        assert!(resp.items.is_empty());
    }
}
