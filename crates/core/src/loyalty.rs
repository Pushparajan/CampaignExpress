//! Loyalty program domain types — tier-aware, omnichannel reward system.
//!
//! Modeled after retail loyalty programs (Starbucks, Sephora, Target):
//! - Three-tier structure: Green → Gold → Reserve
//! - Points (Stars) earning with tier multipliers
//! - Scaled redemption tiers
//! - Birthday rewards, VIP perks, promotional events

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

// ─── Tier System ────────────────────────────────────────────────────────────

/// Loyalty tier levels with escalating benefits.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LoyaltyTier {
    /// Entry-level. 1x earn rate. Base perks.
    Green,
    /// 500 Stars in 12 months. 1.2x earn rate. Stars never expire.
    Gold,
    /// 2500 Stars in 12 months. 1.7x earn rate. Exclusive access.
    Reserve,
}

impl LoyaltyTier {
    /// Points multiplier for this tier.
    pub fn earn_multiplier(&self) -> f32 {
        match self {
            LoyaltyTier::Green => 1.0,
            LoyaltyTier::Gold => 1.2,
            LoyaltyTier::Reserve => 1.7,
        }
    }

    /// Stars required in a 12-month qualifying period to reach this tier.
    pub fn qualification_threshold(&self) -> u32 {
        match self {
            LoyaltyTier::Green => 0,
            LoyaltyTier::Gold => 500,
            LoyaltyTier::Reserve => 2500,
        }
    }

    /// Birthday reward redemption window in days.
    pub fn birthday_window_days(&self) -> u32 {
        match self {
            LoyaltyTier::Green => 1,
            LoyaltyTier::Gold => 7,
            LoyaltyTier::Reserve => 30,
        }
    }

    /// Whether Stars expire for this tier.
    pub fn stars_expire(&self) -> bool {
        matches!(self, LoyaltyTier::Green)
    }

    /// Numeric encoding for SNN feature vector.
    pub fn as_feature(&self) -> f32 {
        match self {
            LoyaltyTier::Green => 0.0,
            LoyaltyTier::Gold => 0.5,
            LoyaltyTier::Reserve => 1.0,
        }
    }
}

impl Default for LoyaltyTier {
    fn default() -> Self {
        LoyaltyTier::Green
    }
}

// ─── Loyalty Profile ────────────────────────────────────────────────────────

/// Complete loyalty profile for a user, stored alongside their UserProfile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyProfile {
    pub user_id: String,
    pub tier: LoyaltyTier,
    pub stars_balance: u32,
    /// Stars earned in the current 12-month qualifying period.
    pub stars_qualifying: u32,
    /// Start of the current qualifying period.
    pub qualifying_period_start: DateTime<Utc>,
    pub lifetime_stars: u64,
    pub total_redemptions: u32,
    pub last_earn: Option<DateTime<Utc>>,
    pub last_redeem: Option<DateTime<Utc>>,
    pub birthday: Option<NaiveDate>,
    pub birthday_reward_used: bool,
    pub enrolled_at: DateTime<Utc>,
    /// Earning channel bonuses the user qualifies for.
    pub earning_bonuses: Vec<EarningBonus>,
    /// Active promotional multipliers.
    pub active_promotions: Vec<PromotionMultiplier>,
    pub preferred_channel: Option<LoyaltyChannel>,
}

impl Default for LoyaltyProfile {
    fn default() -> Self {
        Self {
            user_id: String::new(),
            tier: LoyaltyTier::Green,
            stars_balance: 0,
            stars_qualifying: 0,
            qualifying_period_start: Utc::now(),
            lifetime_stars: 0,
            total_redemptions: 0,
            last_earn: None,
            last_redeem: None,
            birthday: None,
            birthday_reward_used: false,
            enrolled_at: Utc::now(),
            earning_bonuses: Vec::new(),
            active_promotions: Vec::new(),
            preferred_channel: None,
        }
    }
}

impl LoyaltyProfile {
    /// Effective earn rate: tier multiplier * active promotion multipliers.
    pub fn effective_earn_rate(&self) -> f32 {
        let base = self.tier.earn_multiplier();
        let promo: f32 = self
            .active_promotions
            .iter()
            .map(|p| p.multiplier - 1.0)
            .sum::<f32>();
        base + promo
    }

    /// Progress toward next tier as a fraction [0.0, 1.0].
    pub fn tier_progress(&self) -> f32 {
        let next_threshold = match self.tier {
            LoyaltyTier::Green => LoyaltyTier::Gold.qualification_threshold(),
            LoyaltyTier::Gold => LoyaltyTier::Reserve.qualification_threshold(),
            LoyaltyTier::Reserve => return 1.0,
        };
        (self.stars_qualifying as f32 / next_threshold as f32).min(1.0)
    }

    /// Days since last redemption (for SNN recency signal).
    pub fn days_since_redeem(&self) -> Option<i64> {
        self.last_redeem
            .map(|r| (Utc::now() - r).num_days())
    }

    /// Encode loyalty state as feature vector components for SNN input.
    /// Returns: [tier, balance_norm, progress, earn_rate, redeem_recency,
    ///           birthday_eligible, channel, lifetime_norm]
    pub fn as_feature_vector(&self) -> [f32; 8] {
        let balance_norm = (self.stars_balance as f32 / 5000.0).min(1.0);
        let redeem_recency = self
            .days_since_redeem()
            .map(|d| 1.0 / (1.0 + d as f32 / 30.0))
            .unwrap_or(0.0);
        let birthday_eligible = if self.is_birthday_eligible() { 1.0 } else { 0.0 };
        let channel = match self.preferred_channel {
            Some(LoyaltyChannel::InStore) => 0.0,
            Some(LoyaltyChannel::App) => 0.33,
            Some(LoyaltyChannel::Web) => 0.66,
            Some(LoyaltyChannel::DriveThrough) => 1.0,
            None => -1.0,
        };
        let lifetime_norm = (self.lifetime_stars as f32 / 50000.0).min(1.0);

        [
            self.tier.as_feature(),
            balance_norm,
            self.tier_progress(),
            self.effective_earn_rate() / 2.0,
            redeem_recency,
            birthday_eligible,
            channel,
            lifetime_norm,
        ]
    }

    /// Whether the user is currently eligible for a birthday reward.
    pub fn is_birthday_eligible(&self) -> bool {
        if self.birthday_reward_used {
            return false;
        }
        let Some(bday) = self.birthday else {
            return false;
        };
        let today = Utc::now().date_naive();
        let this_year_bday = NaiveDate::from_ymd_opt(today.year(), bday.month(), bday.day());
        match this_year_bday {
            Some(b) => {
                let diff = (today - b).num_days().abs();
                diff <= self.tier.birthday_window_days() as i64
            }
            None => false,
        }
    }
}

use chrono::Datelike;

// ─── Earning & Redemption ───────────────────────────────────────────────────

/// Channel-specific earning bonus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningBonus {
    pub channel: LoyaltyChannel,
    /// Extra multiplier (e.g. 1.2 = 20% bonus).
    pub multiplier: f32,
    pub valid_until: Option<DateTime<Utc>>,
}

/// Promotional multiplier (Double Star Days, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromotionMultiplier {
    pub promotion_id: String,
    pub name: String,
    pub multiplier: f32,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

/// Channels where loyalty interactions happen.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoyaltyChannel {
    InStore,
    App,
    Web,
    DriveThrough,
}

/// Request to earn Stars from a purchase or activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarnStarsRequest {
    pub user_id: String,
    pub amount_cents: u64,
    pub channel: LoyaltyChannel,
    pub transaction_id: String,
    /// Referral bonus: 2x.
    pub is_referral: bool,
    /// Digital-only purchase: 1.2x bonus.
    pub is_digital: bool,
}

/// Result of earning Stars.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarnStarsResponse {
    pub user_id: String,
    pub stars_earned: u32,
    pub new_balance: u32,
    pub earn_rate_applied: f32,
    pub tier: LoyaltyTier,
    pub tier_changed: bool,
    pub new_tier: Option<LoyaltyTier>,
}

/// Redemption tiers — scaled reward catalog.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RedemptionTier {
    /// 25 Stars — drink customization.
    Customization,
    /// 60 Stars — $2 off any purchase.
    Discount,
    /// 100 Stars — brewed coffee/tea or bakery item.
    BasicItem,
    /// 200 Stars — handcrafted drink or hot breakfast.
    PremiumItem,
    /// 300 Stars — sandwich, protein box.
    PremiumFood,
    /// 400 Stars — merchandise or packaged coffee.
    Merchandise,
    /// 500 Stars — exclusive service (stylist session, VIP event).
    ExclusiveService,
}

impl RedemptionTier {
    /// Stars required for this redemption level.
    pub fn stars_required(&self) -> u32 {
        match self {
            RedemptionTier::Customization => 25,
            RedemptionTier::Discount => 60,
            RedemptionTier::BasicItem => 100,
            RedemptionTier::PremiumItem => 200,
            RedemptionTier::PremiumFood => 300,
            RedemptionTier::Merchandise => 400,
            RedemptionTier::ExclusiveService => 500,
        }
    }

    /// Dollar value equivalent.
    pub fn dollar_value(&self) -> f64 {
        match self {
            RedemptionTier::Customization => 0.80,
            RedemptionTier::Discount => 2.00,
            RedemptionTier::BasicItem => 3.50,
            RedemptionTier::PremiumItem => 6.50,
            RedemptionTier::PremiumFood => 8.00,
            RedemptionTier::Merchandise => 15.00,
            RedemptionTier::ExclusiveService => 25.00,
        }
    }

    /// Minimum tier required to access this redemption.
    pub fn min_tier(&self) -> LoyaltyTier {
        match self {
            RedemptionTier::ExclusiveService => LoyaltyTier::Gold,
            _ => LoyaltyTier::Green,
        }
    }
}

/// Request to redeem Stars.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemRequest {
    pub user_id: String,
    pub redemption_tier: RedemptionTier,
    pub channel: LoyaltyChannel,
}

/// Result of a redemption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedeemResponse {
    pub user_id: String,
    pub success: bool,
    pub stars_deducted: u32,
    pub new_balance: u32,
    pub reward_description: String,
    pub error: Option<String>,
}

// ─── Loyalty Offer Types ────────────────────────────────────────────────────

/// Loyalty-aware offer generated by the SNN for personalization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyOffer {
    pub offer_id: String,
    pub offer_type: LoyaltyOfferType,
    pub headline: String,
    pub description: String,
    pub predicted_ctr: f32,
    pub tier_requirement: Option<LoyaltyTier>,
    /// Points bonus multiplier (e.g. 1.5 = 150% points).
    pub points_multiplier: Option<f32>,
    /// Points cost if this is a redemption offer.
    pub redemption_cost: Option<u32>,
    /// Dollar value of the offer.
    pub dollar_value: Option<f64>,
}

/// Types of loyalty-aware offers the SNN can generate.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoyaltyOfferType {
    /// Tier-multiplied earning offer: "150% points on boots".
    TierEarning,
    /// Redemption nudge: "Redeem 500pt for stylist session".
    RedemptionNudge,
    /// Birthday/promotional: "Double points weekend".
    PromotionalBonus,
    /// VIP exclusive perk: "VIP event invite".
    VipPerk,
    /// Standard product offer (no loyalty component).
    Standard,
}

// ─── Loyalty Analytics Events ───────────────────────────────────────────────

/// Loyalty-specific event types for ClickHouse analytics.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoyaltyEventType {
    StarsEarned,
    StarsRedeemed,
    TierUpgrade,
    TierDowngrade,
    BirthdayRewardClaimed,
    PromotionActivated,
    LoyaltyOfferServed,
    LoyaltyOfferClicked,
    LoyaltyOfferRedeemed,
    LoyaltyOfferIgnored,
}

/// RL reward signal for SNN training feedback loop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyRewardSignal {
    pub offer_id: String,
    pub user_id: String,
    pub signal_type: LoyaltySignalType,
    pub reward_value: f32,
    pub tier: LoyaltyTier,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum LoyaltySignalType {
    /// User clicked the loyalty offer → +1.0 reward.
    Clicked,
    /// User redeemed via the offer → +2.0 reward (high LTV).
    Redeemed,
    /// User ignored the offer → -0.1 (refine spikes).
    Ignored,
}

impl LoyaltySignalType {
    pub fn reward_value(&self) -> f32 {
        match self {
            LoyaltySignalType::Clicked => 1.0,
            LoyaltySignalType::Redeemed => 2.0,
            LoyaltySignalType::Ignored => -0.1,
        }
    }
}
