//! Capacity planner — forecasts resource exhaustion using trend analysis,
//! linear regression, and growth rate extrapolation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::warn;

/// A data point for capacity trend analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityDataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
}

/// Resource capacity forecast.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityForecast {
    pub resource_name: String,
    pub current_value: f64,
    pub limit: f64,
    pub current_usage_pct: f64,
    pub growth_rate_per_day: f64,
    pub days_until_exhaustion: Option<u32>,
    pub projected_value_7d: f64,
    pub projected_value_30d: f64,
    pub urgency: ForecastUrgency,
    pub recommendation: String,
}

/// Urgency classification for a forecast.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastUrgency {
    /// > 90 days to exhaustion or shrinking.
    Low,
    /// 30-90 days to exhaustion.
    Medium,
    /// 7-30 days to exhaustion.
    High,
    /// < 7 days to exhaustion.
    Critical,
}

/// Full capacity planning report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityReport {
    pub forecasts: Vec<CapacityForecast>,
    pub critical_resources: Vec<String>,
    pub scaling_recommendations: Vec<ScalingRecommendation>,
    pub generated_at: DateTime<Utc>,
}

/// Scaling recommendation for a specific resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingRecommendation {
    pub resource: String,
    pub current_capacity: String,
    pub recommended_capacity: String,
    pub reason: String,
    pub estimated_cost_impact: String,
}

/// Capacity planner that forecasts resource exhaustion.
pub struct CapacityPlanner;

impl CapacityPlanner {
    /// Forecast exhaustion for a single resource using linear regression.
    pub fn forecast(
        resource_name: impl Into<String>,
        data_points: &[CapacityDataPoint],
        limit: f64,
    ) -> CapacityForecast {
        let resource_name = resource_name.into();

        if data_points.is_empty() {
            return CapacityForecast {
                resource_name,
                current_value: 0.0,
                limit,
                current_usage_pct: 0.0,
                growth_rate_per_day: 0.0,
                days_until_exhaustion: None,
                projected_value_7d: 0.0,
                projected_value_30d: 0.0,
                urgency: ForecastUrgency::Low,
                recommendation: "Insufficient data for forecast".into(),
            };
        }

        let current_value = data_points.last().map(|p| p.value).unwrap_or(0.0);
        let current_usage_pct = if limit > 0.0 {
            current_value / limit * 100.0
        } else {
            0.0
        };

        // Linear regression: y = mx + b
        let (_slope, _intercept) = Self::linear_regression(data_points);

        // Convert slope from per-data-point to per-day
        let growth_rate_per_day = if data_points.len() >= 2 {
            let first_ts = data_points.first().unwrap().timestamp;
            let last_ts = data_points.last().unwrap().timestamp;
            let days = (last_ts - first_ts).num_hours().max(1) as f64 / 24.0;
            let total_growth =
                data_points.last().unwrap().value - data_points.first().unwrap().value;
            total_growth / days
        } else {
            0.0
        };

        // Days until exhaustion
        let days_until_exhaustion = if growth_rate_per_day > 0.0 {
            let remaining = limit - current_value;
            if remaining > 0.0 {
                Some((remaining / growth_rate_per_day).ceil() as u32)
            } else {
                Some(0)
            }
        } else {
            None // Not growing or shrinking
        };

        // Projections
        let projected_value_7d = (current_value + growth_rate_per_day * 7.0).max(0.0);
        let projected_value_30d = (current_value + growth_rate_per_day * 30.0).max(0.0);

        let urgency = match days_until_exhaustion {
            Some(d) if d <= 7 => ForecastUrgency::Critical,
            Some(d) if d <= 30 => ForecastUrgency::High,
            Some(d) if d <= 90 => ForecastUrgency::Medium,
            _ => ForecastUrgency::Low,
        };

        let recommendation = match urgency {
            ForecastUrgency::Critical => format!(
                "URGENT: {} will exhaust in ~{} days. Scale immediately.",
                resource_name,
                days_until_exhaustion.unwrap_or(0)
            ),
            ForecastUrgency::High => format!(
                "Plan scaling for {} within 2 weeks (exhaustion in ~{} days).",
                resource_name,
                days_until_exhaustion.unwrap_or(0)
            ),
            ForecastUrgency::Medium => {
                format!("Monitor {} growth. Schedule scaling review.", resource_name)
            }
            ForecastUrgency::Low => {
                if growth_rate_per_day <= 0.0 {
                    format!(
                        "{} is stable or shrinking. No action needed.",
                        resource_name
                    )
                } else {
                    format!("{} growing slowly. Review in 90 days.", resource_name)
                }
            }
        };

        if urgency == ForecastUrgency::Critical {
            warn!(
                resource = %resource_name,
                days = ?days_until_exhaustion,
                "Critical: resource exhaustion imminent"
            );
        }

        CapacityForecast {
            resource_name,
            current_value,
            limit,
            current_usage_pct,
            growth_rate_per_day,
            days_until_exhaustion,
            projected_value_7d,
            projected_value_30d,
            urgency,
            recommendation,
        }
    }

    /// Generate a full capacity report for all tracked resources.
    pub fn generate_report(
        resource_data: Vec<(&str, Vec<CapacityDataPoint>, f64)>,
    ) -> CapacityReport {
        let mut forecasts: Vec<CapacityForecast> = resource_data
            .into_iter()
            .map(|(name, points, limit)| Self::forecast(name, &points, limit))
            .collect();

        forecasts.sort_by(|a, b| {
            a.days_until_exhaustion
                .unwrap_or(u32::MAX)
                .cmp(&b.days_until_exhaustion.unwrap_or(u32::MAX))
        });

        let critical_resources: Vec<String> = forecasts
            .iter()
            .filter(|f| {
                f.urgency == ForecastUrgency::Critical || f.urgency == ForecastUrgency::High
            })
            .map(|f| f.resource_name.clone())
            .collect();

        let scaling_recommendations: Vec<ScalingRecommendation> = forecasts
            .iter()
            .filter(|f| {
                f.urgency == ForecastUrgency::Critical || f.urgency == ForecastUrgency::High
            })
            .map(|f| ScalingRecommendation {
                resource: f.resource_name.clone(),
                current_capacity: format!("{:.0} / {:.0}", f.current_value, f.limit),
                recommended_capacity: format!("{:.0}", f.projected_value_30d * 1.5),
                reason: f.recommendation.clone(),
                estimated_cost_impact: "Requires cost analysis".into(),
            })
            .collect();

        CapacityReport {
            forecasts,
            critical_resources,
            scaling_recommendations,
            generated_at: Utc::now(),
        }
    }

    /// Simple linear regression: returns (slope, intercept).
    fn linear_regression(points: &[CapacityDataPoint]) -> (f64, f64) {
        let n = points.len() as f64;
        if n < 2.0 {
            return (0.0, points.first().map(|p| p.value).unwrap_or(0.0));
        }

        let x: Vec<f64> = (0..points.len()).map(|i| i as f64).collect();
        let y: Vec<f64> = points.iter().map(|p| p.value).collect();

        let sum_x: f64 = x.iter().sum();
        let sum_y: f64 = y.iter().sum();
        let sum_xy: f64 = x.iter().zip(y.iter()).map(|(xi, yi)| xi * yi).sum();
        let sum_x2: f64 = x.iter().map(|xi| xi * xi).sum();

        let denominator = n * sum_x2 - sum_x * sum_x;
        if denominator.abs() < f64::EPSILON {
            return (0.0, sum_y / n);
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;

        (slope, intercept)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_growing_data(days: usize, start: f64, growth_per_day: f64) -> Vec<CapacityDataPoint> {
        let now = Utc::now();
        (0..days)
            .map(|i| CapacityDataPoint {
                timestamp: now - Duration::days((days - i) as i64),
                value: start + growth_per_day * i as f64,
            })
            .collect()
    }

    #[test]
    fn test_growing_resource_forecast() {
        let data = make_growing_data(30, 50.0, 1.0); // 50 + 1/day
        let forecast = CapacityPlanner::forecast("disk_gb", &data, 100.0);
        assert!(forecast.growth_rate_per_day > 0.0);
        assert!(forecast.days_until_exhaustion.is_some());
        assert!(forecast.projected_value_30d > forecast.current_value);
    }

    #[test]
    fn test_stable_resource() {
        let data = make_growing_data(30, 30.0, 0.0);
        let forecast = CapacityPlanner::forecast("stable", &data, 100.0);
        assert!(forecast.days_until_exhaustion.is_none());
        assert_eq!(forecast.urgency, ForecastUrgency::Low);
    }

    #[test]
    fn test_critical_exhaustion() {
        let data = make_growing_data(7, 90.0, 2.0); // 90 + 2/day, limit 100
        let forecast = CapacityPlanner::forecast("critical", &data, 105.0);
        // Should detect approaching limit
        assert!(forecast.current_usage_pct > 90.0);
    }

    #[test]
    fn test_empty_data() {
        let forecast = CapacityPlanner::forecast("empty", &[], 100.0);
        assert_eq!(forecast.current_value, 0.0);
        assert_eq!(forecast.urgency, ForecastUrgency::Low);
    }

    #[test]
    fn test_report_generation() {
        let data1 = make_growing_data(30, 80.0, 1.0);
        let data2 = make_growing_data(30, 20.0, 0.1);

        let report =
            CapacityPlanner::generate_report(vec![("disk", data1, 100.0), ("cpu", data2, 100.0)]);

        assert_eq!(report.forecasts.len(), 2);
        // Disk should be first (closer to exhaustion)
        assert_eq!(report.forecasts[0].resource_name, "disk");
    }
}
