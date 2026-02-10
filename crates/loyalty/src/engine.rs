//! Core loyalty engine: handles star earning, redemption, tier transitions,
//! birthday rewards, and promotional multiplier management.

use campaign_core::config::LoyaltyConfig;
use campaign_core::loyalty::*;
use chrono::Utc;
use tracing::{debug, info};

/// Loyalty program engine — stateless computation over LoyaltyProfile data.
pub struct LoyaltyEngine {
    config: LoyaltyConfig,
}

impl LoyaltyEngine {
    pub fn new(config: &LoyaltyConfig) -> Self {
        info!(
            enabled = config.enabled,
            gold = config.gold_threshold,
            reserve = config.reserve_threshold,
            "Loyalty engine initialized"
        );
        Self {
            config: config.clone(),
        }
    }

    /// Earn Stars from a purchase or activity.
    pub fn earn_stars(
        &self,
        profile: &mut LoyaltyProfile,
        request: &EarnStarsRequest,
    ) -> EarnStarsResponse {
        let base_stars = (request.amount_cents as f32 / 100.0).ceil() as u32;
        let mut rate = profile.effective_earn_rate();

        // Channel bonuses
        for bonus in &profile.earning_bonuses {
            if bonus.channel == request.channel {
                if bonus.valid_until.map(|v| Utc::now() < v).unwrap_or(true) {
                    rate += bonus.multiplier - 1.0;
                }
            }
        }

        // Referral bonus: 2x
        if request.is_referral {
            rate *= 2.0;
        }

        // Digital purchase bonus: 1.2x
        if request.is_digital {
            rate *= 1.2;
        }

        let stars_earned = (base_stars as f32 * rate).ceil() as u32;

        profile.stars_balance += stars_earned;
        profile.stars_qualifying += stars_earned;
        profile.lifetime_stars += stars_earned as u64;
        profile.last_earn = Some(Utc::now());

        metrics::counter!("loyalty.stars_earned").increment(stars_earned as u64);

        // Check tier upgrade
        let _old_tier = profile.tier;
        let tier_changed = self.evaluate_tier(profile);
        let new_tier = if tier_changed {
            Some(profile.tier)
        } else {
            None
        };

        debug!(
            user_id = %profile.user_id,
            stars_earned = stars_earned,
            balance = profile.stars_balance,
            rate = rate,
            tier = ?profile.tier,
            "Stars earned"
        );

        EarnStarsResponse {
            user_id: profile.user_id.clone(),
            stars_earned,
            new_balance: profile.stars_balance,
            earn_rate_applied: rate,
            tier: profile.tier,
            tier_changed,
            new_tier,
        }
    }

    /// Redeem Stars for a reward.
    pub fn redeem_stars(
        &self,
        profile: &mut LoyaltyProfile,
        request: &RedeemRequest,
    ) -> RedeemResponse {
        let cost = request.redemption_tier.stars_required();
        let min_tier = request.redemption_tier.min_tier();

        if profile.tier < min_tier {
            return RedeemResponse {
                user_id: profile.user_id.clone(),
                success: false,
                stars_deducted: 0,
                new_balance: profile.stars_balance,
                reward_description: String::new(),
                error: Some(format!(
                    "Requires {} tier, you are {:?}",
                    min_tier.earn_multiplier(),
                    profile.tier
                )),
            };
        }

        if profile.stars_balance < cost {
            return RedeemResponse {
                user_id: profile.user_id.clone(),
                success: false,
                stars_deducted: 0,
                new_balance: profile.stars_balance,
                reward_description: String::new(),
                error: Some(format!(
                    "Insufficient stars: need {}, have {}",
                    cost, profile.stars_balance
                )),
            };
        }

        profile.stars_balance -= cost;
        profile.total_redemptions += 1;
        profile.last_redeem = Some(Utc::now());

        metrics::counter!("loyalty.stars_redeemed").increment(cost as u64);
        metrics::counter!("loyalty.redemptions").increment(1);

        let description = format!(
            "{:?} reward (${:.2} value)",
            request.redemption_tier,
            request.redemption_tier.dollar_value()
        );

        info!(
            user_id = %profile.user_id,
            tier = ?request.redemption_tier,
            cost = cost,
            new_balance = profile.stars_balance,
            "Stars redeemed"
        );

        RedeemResponse {
            user_id: profile.user_id.clone(),
            success: true,
            stars_deducted: cost,
            new_balance: profile.stars_balance,
            reward_description: description,
            error: None,
        }
    }

    /// Evaluate and update tier based on qualifying period stars.
    /// Returns true if tier changed.
    pub fn evaluate_tier(&self, profile: &mut LoyaltyProfile) -> bool {
        let old_tier = profile.tier;

        let new_tier = if profile.stars_qualifying >= self.config.reserve_threshold {
            LoyaltyTier::Reserve
        } else if profile.stars_qualifying >= self.config.gold_threshold {
            LoyaltyTier::Gold
        } else {
            LoyaltyTier::Green
        };

        if new_tier != old_tier {
            profile.tier = new_tier;
            if new_tier > old_tier {
                metrics::counter!("loyalty.tier_upgrades").increment(1);
                info!(
                    user_id = %profile.user_id,
                    old = ?old_tier,
                    new = ?new_tier,
                    "Tier upgrade"
                );
            } else {
                metrics::counter!("loyalty.tier_downgrades").increment(1);
            }
            true
        } else {
            false
        }
    }

    /// Generate loyalty-aware offers for the SNN inference pipeline.
    /// Returns offer types to consider based on user's loyalty state.
    pub fn suggest_offer_types(&self, profile: &LoyaltyProfile) -> Vec<LoyaltyOfferType> {
        let mut types = vec![LoyaltyOfferType::Standard];

        // Tier earning offers for all enrolled members
        types.push(LoyaltyOfferType::TierEarning);

        // Redemption nudge if user has enough stars and hasn't redeemed recently
        let days_since = profile.days_since_redeem().unwrap_or(999);
        if profile.stars_balance >= 100 && days_since > 14 {
            types.push(LoyaltyOfferType::RedemptionNudge);
        }

        // Near-threshold nudge (within 20% of next tier)
        if profile.tier_progress() > 0.8 && profile.tier < LoyaltyTier::Reserve {
            types.push(LoyaltyOfferType::RedemptionNudge);
        }

        // Birthday promo
        if profile.is_birthday_eligible() {
            types.push(LoyaltyOfferType::PromotionalBonus);
        }

        // VIP perks for Gold+
        if profile.tier >= LoyaltyTier::Gold {
            types.push(LoyaltyOfferType::VipPerk);
        }

        // Active promotions
        if !profile.active_promotions.is_empty() {
            types.push(LoyaltyOfferType::PromotionalBonus);
        }

        types.dedup();
        types
    }

    /// Get or create a default loyalty profile for a user.
    pub fn get_or_create_profile(&self, user_id: &str) -> LoyaltyProfile {
        // In production: fetch from Redis/DB. Here we return a default.
        LoyaltyProfile {
            user_id: user_id.to_string(),
            ..Default::default()
        }
    }

    pub fn config(&self) -> &LoyaltyConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> LoyaltyConfig {
        LoyaltyConfig::default()
    }

    fn test_profile() -> LoyaltyProfile {
        LoyaltyProfile {
            user_id: "test-user".to_string(),
            stars_balance: 200,
            stars_qualifying: 200,
            ..Default::default()
        }
    }

    #[test]
    fn test_earn_stars_basic() {
        let engine = LoyaltyEngine::new(&test_config());
        let mut profile = test_profile();

        let req = EarnStarsRequest {
            user_id: "test-user".to_string(),
            amount_cents: 500,
            channel: LoyaltyChannel::App,
            transaction_id: "tx-1".to_string(),
            is_referral: false,
            is_digital: false,
        };

        let resp = engine.earn_stars(&mut profile, &req);
        assert_eq!(resp.stars_earned, 5); // $5 * 1.0x = 5 stars
        assert_eq!(resp.new_balance, 205);
        assert!(!resp.tier_changed);
    }

    #[test]
    fn test_earn_stars_gold_multiplier() {
        let engine = LoyaltyEngine::new(&test_config());
        let mut profile = test_profile();
        profile.tier = LoyaltyTier::Gold;

        let req = EarnStarsRequest {
            user_id: "test-user".to_string(),
            amount_cents: 1000,
            channel: LoyaltyChannel::InStore,
            transaction_id: "tx-2".to_string(),
            is_referral: false,
            is_digital: false,
        };

        let resp = engine.earn_stars(&mut profile, &req);
        assert_eq!(resp.stars_earned, 12); // $10 * 1.2x = 12
    }

    #[test]
    fn test_earn_stars_referral_bonus() {
        let engine = LoyaltyEngine::new(&test_config());
        let mut profile = test_profile();

        let req = EarnStarsRequest {
            user_id: "test-user".to_string(),
            amount_cents: 1000,
            channel: LoyaltyChannel::App,
            transaction_id: "tx-3".to_string(),
            is_referral: true,
            is_digital: false,
        };

        let resp = engine.earn_stars(&mut profile, &req);
        assert_eq!(resp.stars_earned, 20); // $10 * 1.0x * 2.0 referral = 20
    }

    #[test]
    fn test_redeem_stars_success() {
        let engine = LoyaltyEngine::new(&test_config());
        let mut profile = test_profile();
        profile.stars_balance = 300;

        let req = RedeemRequest {
            user_id: "test-user".to_string(),
            redemption_tier: RedemptionTier::PremiumItem,
            channel: LoyaltyChannel::App,
        };

        let resp = engine.redeem_stars(&mut profile, &req);
        assert!(resp.success);
        assert_eq!(resp.stars_deducted, 200);
        assert_eq!(resp.new_balance, 100);
    }

    #[test]
    fn test_redeem_stars_insufficient() {
        let engine = LoyaltyEngine::new(&test_config());
        let mut profile = test_profile();
        profile.stars_balance = 50;

        let req = RedeemRequest {
            user_id: "test-user".to_string(),
            redemption_tier: RedemptionTier::PremiumItem,
            channel: LoyaltyChannel::App,
        };

        let resp = engine.redeem_stars(&mut profile, &req);
        assert!(!resp.success);
        assert!(resp.error.is_some());
    }

    #[test]
    fn test_tier_upgrade() {
        let engine = LoyaltyEngine::new(&test_config());
        let mut profile = test_profile();
        profile.stars_qualifying = 510;

        let changed = engine.evaluate_tier(&mut profile);
        assert!(changed);
        assert_eq!(profile.tier, LoyaltyTier::Gold);
    }

    #[test]
    fn test_suggest_offer_types_gold() {
        let engine = LoyaltyEngine::new(&test_config());
        let mut profile = test_profile();
        profile.tier = LoyaltyTier::Gold;
        profile.stars_balance = 400;

        let types = engine.suggest_offer_types(&profile);
        assert!(types.contains(&LoyaltyOfferType::Standard));
        assert!(types.contains(&LoyaltyOfferType::TierEarning));
        assert!(types.contains(&LoyaltyOfferType::VipPerk));
        assert!(types.contains(&LoyaltyOfferType::RedemptionNudge));
    }

    #[test]
    fn test_loyalty_feature_vector() {
        let profile = test_profile();
        let features = profile.as_feature_vector();
        assert_eq!(features.len(), 8);
        assert_eq!(features[0], 0.0); // Green tier
    }
}
