use rand::Rng;
use uuid::Uuid;

use crate::types::DcoTemplate;

/// Scores creative combinations using historical performance and Thompson Sampling.
#[derive(Debug, Clone)]
pub struct VariantScorer;

impl VariantScorer {
    /// Create a new scorer instance.
    pub fn new() -> Self {
        Self
    }

    /// Score a batch of combinations against user segments.
    ///
    /// For each combination the score is a weighted blend of:
    ///   - historical CTR (Thompson-sampled for exploration)
    ///   - historical CVR
    ///   - user-segment affinity (fraction of variant metadata segments that
    ///     overlap with the request segments)
    ///
    /// Returns a `Vec<f32>` in the same order as `combinations`.
    pub fn score_combinations(
        &self,
        template: &DcoTemplate,
        combinations: &[Vec<(Uuid, Uuid)>],
        user_segments: &[u32],
    ) -> Vec<f32> {
        combinations
            .iter()
            .map(|combo| self.score_single(template, combo, user_segments))
            .collect()
    }

    /// Score a single combination.
    fn score_single(
        &self,
        template: &DcoTemplate,
        combination: &[(Uuid, Uuid)],
        user_segments: &[u32],
    ) -> f32 {
        let mut total_score: f32 = 0.0;
        let mut component_count: f32 = 0.0;

        for (component_id, variant_id) in combination {
            if let Some(comp) = template.components.iter().find(|c| &c.id == component_id) {
                if let Some(variant) = comp.variants.iter().find(|v| &v.id == variant_id) {
                    let perf = &variant.performance;

                    // Thompson-sampled CTR for explore/exploit balance
                    let sampled_ctr = self.thompson_sample(perf.impressions, perf.clicks);

                    // CVR contribution
                    let cvr_score = perf.cvr as f32;

                    // User-segment affinity: check how many of the user's segments
                    // appear in the variant metadata's "segments" array (if present).
                    let segment_score = if let Some(arr) = variant.metadata.get("segments") {
                        if let Some(arr) = arr.as_array() {
                            let matching = arr
                                .iter()
                                .filter_map(|v| v.as_u64().map(|n| n as u32))
                                .filter(|s| user_segments.contains(s))
                                .count();
                            if arr.is_empty() {
                                0.0_f32
                            } else {
                                matching as f32 / arr.len() as f32
                            }
                        } else {
                            0.0
                        }
                    } else {
                        0.0
                    };

                    // Weighted combination: CTR 50%, CVR 30%, segment affinity 20%
                    let variant_score = sampled_ctr * 0.5 + cvr_score * 0.3 + segment_score * 0.2;

                    total_score += variant_score;
                    component_count += 1.0;
                }
            }
        }

        if component_count > 0.0 {
            total_score / component_count
        } else {
            0.0
        }
    }

    /// Thompson Sampling using a Beta-distribution approximation.
    ///
    /// alpha = clicks + 1, beta = (impressions - clicks) + 1.
    /// Returns a sampled value in [0, 1].
    pub fn thompson_sample(&self, impressions: u64, clicks: u64) -> f32 {
        let alpha = clicks as f64 + 1.0;
        let beta = (impressions.saturating_sub(clicks)) as f64 + 1.0;

        // Approximate Beta sample via the ratio of Gamma samples.
        // For a Gamma(a, 1) we use the Marsaglia-Tsang method built into rand,
        // but the simple approach is to use the relationship:
        //   X ~ Gamma(alpha), Y ~ Gamma(beta)  =>  X / (X + Y) ~ Beta(alpha, beta)
        let mut rng = rand::thread_rng();

        let x = gamma_sample(&mut rng, alpha);
        let y = gamma_sample(&mut rng, beta);

        if x + y > 0.0 {
            (x / (x + y)) as f32
        } else {
            0.5
        }
    }
}

impl Default for VariantScorer {
    fn default() -> Self {
        Self::new()
    }
}

/// Sample from Gamma(shape, 1) using a simple method.
///
/// For shape >= 1 this uses the Marsaglia-Tsang method.
/// For shape < 1 it uses the Ahrens-Dieter boost.
fn gamma_sample<R: Rng>(rng: &mut R, shape: f64) -> f64 {
    if shape < 1.0 {
        // Boost: Gamma(a) = Gamma(a+1) * U^(1/a)
        let u: f64 = rng.gen();
        return gamma_sample(rng, shape + 1.0) * u.powf(1.0 / shape);
    }

    // Marsaglia-Tsang method for shape >= 1
    let d = shape - 1.0 / 3.0;
    let c = 1.0 / (9.0 * d).sqrt();

    loop {
        // Box-Muller for a standard normal
        let u1: f64 = rng.gen::<f64>().max(1e-15);
        let u2: f64 = rng.gen();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();

        let v = (1.0 + c * z).powi(3);
        if v <= 0.0 {
            continue;
        }

        let u: f64 = rng.gen();
        // Acceptance criterion
        if u < 1.0 - 0.0331 * z.powi(4) || u.ln() < 0.5 * z * z + d * (1.0 - v + v.ln()) {
            return d * v;
        }
    }
}
