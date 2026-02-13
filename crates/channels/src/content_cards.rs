//! Content cards — persistent, dismissible cards in a user's feed.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentCardType {
    Classic,
    CaptionedImage,
    Banner,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentCard {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub card_type: ContentCardType,
    pub title: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub url: Option<String>,
    pub pinned: bool,
    pub dismissible: bool,
    pub extras: std::collections::HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentCardFeed {
    pub user_id: Uuid,
    pub cards: Vec<ContentCard>,
    pub last_synced: DateTime<Utc>,
    pub total_unread: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardImpression {
    pub card_id: Uuid,
    pub user_id: Uuid,
    pub viewed_at: DateTime<Utc>,
    pub clicked: bool,
    pub dismissed: bool,
}

pub struct ContentCardEngine {
    feeds: dashmap::DashMap<Uuid, Vec<ContentCard>>,
    impressions: dashmap::DashMap<Uuid, Vec<CardImpression>>,
}

impl ContentCardEngine {
    pub fn new() -> Self {
        Self {
            feeds: dashmap::DashMap::new(),
            impressions: dashmap::DashMap::new(),
        }
    }

    pub fn add_card(&self, user_id: Uuid, card: ContentCard) {
        self.feeds.entry(user_id).or_default().push(card);
    }

    pub fn get_feed(&self, user_id: &Uuid) -> ContentCardFeed {
        let cards = self
            .feeds
            .get(user_id)
            .map(|c| c.clone())
            .unwrap_or_default();
        let total_unread = cards.len() as u32;
        ContentCardFeed {
            user_id: *user_id,
            cards,
            last_synced: Utc::now(),
            total_unread,
        }
    }

    pub fn dismiss_card(&self, user_id: &Uuid, card_id: &Uuid) {
        if let Some(mut cards) = self.feeds.get_mut(user_id) {
            cards.retain(|c| &c.id != card_id);
        }
    }

    pub fn record_impression(&self, impression: CardImpression) {
        self.impressions
            .entry(impression.card_id)
            .or_default()
            .push(impression);
    }
}

impl Default for ContentCardEngine {
    fn default() -> Self {
        Self::new()
    }
}
