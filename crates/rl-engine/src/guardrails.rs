//! AI Guardrails — business rules engine that validates AI decisions before execution.

use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailConfig {
    pub campaign_id: Uuid,
    pub rules: Vec<GuardrailRule>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuardrailRule {
    FrequencyCap {
        max_per_day: u32,
        max_per_week: u32,
        max_per_month: u32,
    },
    TimeRestriction {
        allowed_hours: Vec<u32>,
        allowed_days: Vec<u32>,
    },
    IncentiveCap {
        max_discount_percent: f64,
        never_discount_segments: Vec<String>,
    },
    BudgetCap {
        max_daily_spend: f64,
        max_monthly_spend: f64,
    },
    SegmentRestriction {
        blocked_segments: Vec<String>,
        blocked_channels: Vec<String>,
    },
    ChannelRestriction {
        required_consent_type: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailDecision {
    pub allowed: bool,
    pub violations: Vec<GuardrailViolation>,
    pub evaluated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailViolation {
    pub rule_type: String,
    pub description: String,
    pub severity: ViolationSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationSeverity {
    Block,
    Warn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionContext {
    pub user_id: Uuid,
    pub user_segments: Vec<String>,
    pub channel: String,
    pub discount_percent: Option<f64>,
    pub campaign_spend_today: f64,
    pub campaign_spend_month: f64,
    pub user_sends_today: u32,
    pub user_sends_week: u32,
    pub user_sends_month: u32,
    pub has_consent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationLog {
    pub campaign_id: Uuid,
    pub total_evaluated: u64,
    pub total_blocked: u64,
    pub violations_by_type: std::collections::HashMap<String, u64>,
}

pub struct GuardrailsEngine {
    configs: dashmap::DashMap<Uuid, GuardrailConfig>,
    violation_counts: dashmap::DashMap<(Uuid, String), u64>,
    total_evaluated: dashmap::DashMap<Uuid, u64>,
    total_blocked: dashmap::DashMap<Uuid, u64>,
}

impl GuardrailsEngine {
    pub fn new() -> Self {
        Self {
            configs: dashmap::DashMap::new(),
            violation_counts: dashmap::DashMap::new(),
            total_evaluated: dashmap::DashMap::new(),
            total_blocked: dashmap::DashMap::new(),
        }
    }

    pub fn configure(&self, config: GuardrailConfig) {
        self.configs.insert(config.campaign_id, config);
    }

    pub fn validate(&self, campaign_id: &Uuid, context: &ActionContext) -> GuardrailDecision {
        let config = match self.configs.get(campaign_id) {
            Some(c) => c,
            None => {
                return GuardrailDecision {
                    allowed: true,
                    violations: Vec::new(),
                    evaluated_at: Utc::now(),
                }
            }
        };

        let mut violations = Vec::new();
        let now = Utc::now();

        for rule in &config.rules {
            match rule {
                GuardrailRule::FrequencyCap {
                    max_per_day,
                    max_per_week,
                    max_per_month,
                } => {
                    if context.user_sends_today >= *max_per_day {
                        violations.push(GuardrailViolation {
                            rule_type: "frequency_cap".to_string(),
                            description: format!(
                                "Daily cap exceeded ({}/{})",
                                context.user_sends_today, max_per_day
                            ),
                            severity: ViolationSeverity::Block,
                        });
                    }
                    if context.user_sends_week >= *max_per_week {
                        violations.push(GuardrailViolation {
                            rule_type: "frequency_cap".to_string(),
                            description: format!(
                                "Weekly cap exceeded ({}/{})",
                                context.user_sends_week, max_per_week
                            ),
                            severity: ViolationSeverity::Block,
                        });
                    }
                    if context.user_sends_month >= *max_per_month {
                        violations.push(GuardrailViolation {
                            rule_type: "frequency_cap".to_string(),
                            description: format!(
                                "Monthly cap exceeded ({}/{})",
                                context.user_sends_month, max_per_month
                            ),
                            severity: ViolationSeverity::Block,
                        });
                    }
                }
                GuardrailRule::TimeRestriction {
                    allowed_hours,
                    allowed_days,
                } => {
                    let hour = now.hour();
                    let day = now.weekday().num_days_from_monday() + 1;
                    if !allowed_hours.contains(&hour) {
                        violations.push(GuardrailViolation {
                            rule_type: "time_restriction".to_string(),
                            description: format!("Hour {} not in allowed hours", hour),
                            severity: ViolationSeverity::Block,
                        });
                    }
                    if !allowed_days.contains(&day) {
                        violations.push(GuardrailViolation {
                            rule_type: "time_restriction".to_string(),
                            description: format!("Day {} not in allowed days", day),
                            severity: ViolationSeverity::Block,
                        });
                    }
                }
                GuardrailRule::IncentiveCap {
                    max_discount_percent,
                    never_discount_segments,
                } => {
                    if let Some(discount) = context.discount_percent {
                        if discount > *max_discount_percent {
                            violations.push(GuardrailViolation {
                                rule_type: "incentive_cap".to_string(),
                                description: format!(
                                    "Discount {}% exceeds max {}%",
                                    discount, max_discount_percent
                                ),
                                severity: ViolationSeverity::Block,
                            });
                        }
                        if discount > 0.0 {
                            for seg in &context.user_segments {
                                if never_discount_segments.contains(seg) {
                                    violations.push(GuardrailViolation {
                                        rule_type: "incentive_cap".to_string(),
                                        description: format!(
                                            "Segment '{}' is excluded from discounts",
                                            seg
                                        ),
                                        severity: ViolationSeverity::Block,
                                    });
                                }
                            }
                        }
                    }
                }
                GuardrailRule::BudgetCap {
                    max_daily_spend,
                    max_monthly_spend,
                } => {
                    if context.campaign_spend_today >= *max_daily_spend {
                        violations.push(GuardrailViolation {
                            rule_type: "budget_cap".to_string(),
                            description: format!(
                                "Daily budget exceeded (${:.2}/${:.2})",
                                context.campaign_spend_today, max_daily_spend
                            ),
                            severity: ViolationSeverity::Block,
                        });
                    }
                    if context.campaign_spend_month >= *max_monthly_spend {
                        violations.push(GuardrailViolation {
                            rule_type: "budget_cap".to_string(),
                            description: format!(
                                "Monthly budget exceeded (${:.2}/${:.2})",
                                context.campaign_spend_month, max_monthly_spend
                            ),
                            severity: ViolationSeverity::Block,
                        });
                    }
                }
                GuardrailRule::SegmentRestriction {
                    blocked_segments,
                    blocked_channels,
                } => {
                    for seg in &context.user_segments {
                        if blocked_segments.contains(seg) {
                            violations.push(GuardrailViolation {
                                rule_type: "segment_restriction".to_string(),
                                description: format!("Segment '{}' is blocked", seg),
                                severity: ViolationSeverity::Block,
                            });
                        }
                    }
                    if blocked_channels.contains(&context.channel) {
                        violations.push(GuardrailViolation {
                            rule_type: "channel_restriction".to_string(),
                            description: format!(
                                "Channel '{}' is blocked for user segments",
                                context.channel
                            ),
                            severity: ViolationSeverity::Block,
                        });
                    }
                }
                GuardrailRule::ChannelRestriction { .. } => {
                    if !context.has_consent {
                        violations.push(GuardrailViolation {
                            rule_type: "consent".to_string(),
                            description: "User has not provided consent".to_string(),
                            severity: ViolationSeverity::Block,
                        });
                    }
                }
            }
        }

        self.total_evaluated
            .entry(*campaign_id)
            .and_modify(|c| *c += 1)
            .or_insert(1);

        let blocked = violations
            .iter()
            .any(|v| matches!(v.severity, ViolationSeverity::Block));

        if blocked {
            self.total_blocked
                .entry(*campaign_id)
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }

        for v in &violations {
            self.violation_counts
                .entry((*campaign_id, v.rule_type.clone()))
                .and_modify(|c| *c += 1)
                .or_insert(1);
        }

        GuardrailDecision {
            allowed: !blocked,
            violations,
            evaluated_at: now,
        }
    }

    pub fn get_violation_log(&self, campaign_id: &Uuid) -> ViolationLog {
        let mut by_type = std::collections::HashMap::new();
        for entry in self.violation_counts.iter() {
            let (cid, rule_type) = entry.key();
            if cid == campaign_id {
                by_type.insert(rule_type.clone(), *entry.value());
            }
        }
        ViolationLog {
            campaign_id: *campaign_id,
            total_evaluated: self
                .total_evaluated
                .get(campaign_id)
                .map(|v| *v)
                .unwrap_or(0),
            total_blocked: self.total_blocked.get(campaign_id).map(|v| *v).unwrap_or(0),
            violations_by_type: by_type,
        }
    }
}

impl Default for GuardrailsEngine {
    fn default() -> Self {
        Self::new()
    }
}
