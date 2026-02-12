//! Frequency capping — limits how often users receive messages per channel.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CappingChannel {
    Push,
    Email,
    Sms,
    InApp,
    ContentCard,
    WhatsApp,
    WebPush,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CappingWindow {
    PerHour,
    PerDay,
    PerWeek,
    PerMonth,
    PerCampaign,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyRule {
    pub id: Uuid,
    pub channel: CappingChannel,
    pub window: CappingWindow,
    pub max_messages: u32,
    pub priority: u8,
    pub tag: Option<String>,
}

#[derive(Debug, Clone)]
struct MessageRecord {
    timestamp: DateTime<Utc>,
    channel: CappingChannel,
    #[allow(dead_code)]
    campaign_id: Uuid,
}

pub struct FrequencyCapEngine {
    rules: Vec<FrequencyRule>,
    user_history: dashmap::DashMap<Uuid, Vec<MessageRecord>>,
}

impl FrequencyCapEngine {
    pub fn new(rules: Vec<FrequencyRule>) -> Self {
        Self {
            rules,
            user_history: dashmap::DashMap::new(),
        }
    }

    pub fn can_send(&self, user_id: &Uuid, channel: &CappingChannel) -> bool {
        let now = Utc::now();
        let history = self.user_history.get(user_id);

        for rule in &self.rules {
            if !Self::channel_matches(&rule.channel, channel) {
                continue;
            }
            let window_start = Self::window_start(now, &rule.window);
            let count = history
                .as_ref()
                .map(|h| {
                    h.iter()
                        .filter(|r| {
                            r.timestamp >= window_start
                                && Self::channel_matches(&rule.channel, &r.channel)
                        })
                        .count() as u32
                })
                .unwrap_or(0);

            if count >= rule.max_messages {
                return false;
            }
        }
        true
    }

    pub fn record_send(&self, user_id: Uuid, channel: CappingChannel, campaign_id: Uuid) {
        self.user_history
            .entry(user_id)
            .or_default()
            .push(MessageRecord {
                timestamp: Utc::now(),
                channel,
                campaign_id,
            });
    }

    fn channel_matches(rule_channel: &CappingChannel, msg_channel: &CappingChannel) -> bool {
        matches!(rule_channel, CappingChannel::All)
            || std::mem::discriminant(rule_channel) == std::mem::discriminant(msg_channel)
    }

    fn window_start(now: DateTime<Utc>, window: &CappingWindow) -> DateTime<Utc> {
        match window {
            CappingWindow::PerHour => now - Duration::hours(1),
            CappingWindow::PerDay => now - Duration::days(1),
            CappingWindow::PerWeek => now - Duration::weeks(1),
            CappingWindow::PerMonth => now - Duration::days(30),
            CappingWindow::PerCampaign => DateTime::<Utc>::MIN_UTC,
        }
    }
}
