//! Contamination threshold calibration via distributional modeling.
//!
//! Addresses the key reviewer criticism that the contamination threshold τ lacks
//! distributional calibration against baseline n-gram overlap from background
//! language statistics, risking uncontrolled false positive/negative rates.
//!
//! # Approach
//!
//! Models baseline n-gram overlap between independent texts using a Poisson
//! approximation: when two texts of lengths L₁, L₂ are drawn independently from
//! a vocabulary of size V with n-gram order k, the expected intersection size is
//!
//!   E[|A ∩ B|] ≈ V^k · (1 - (1 - 1/V^k)^{L₁}) · (1 - (1 - 1/V^k)^{L₂})
//!
//! which for large V^k simplifies to E ≈ L₁·L₂/V^k. The overlap count follows
//! an approximately Poisson distribution with parameter λ = E. The threshold τ
//! is then set at a specified false-positive rate α by inverting the Poisson CDF.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ngram::{NGramConfig, NGramSet};

// ---------------------------------------------------------------------------
// Calibration parameters
// ---------------------------------------------------------------------------

/// Estimated corpus statistics for threshold calibration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CorpusStatistics {
    /// Effective vocabulary size (number of distinct tokens/characters).
    pub vocab_size: usize,
    /// N-gram order used for overlap detection.
    pub ngram_order: usize,
    /// Typical document length in tokens/characters.
    pub typical_doc_length: usize,
    /// Total distinct n-grams observed in a reference sample.
    pub observed_distinct_ngrams: usize,
    /// Empirical n-gram frequency distribution (ngram_hash → count).
    pub frequency_distribution: Option<HashMap<u64, usize>>,
}

impl CorpusStatistics {
    /// Estimate from a sample of texts using the given n-gram configuration.
    pub fn estimate_from_sample(texts: &[&str], config: &NGramConfig) -> Self {
        let mut all_ngrams: HashMap<u64, usize> = HashMap::new();
        let mut total_length = 0usize;

        for text in texts {
            let set = NGramSet::from_text(text, config.clone());
            let sorted = set.to_sorted_vec();
            for fp in &sorted {
                *all_ngrams.entry(*fp).or_insert(0) += 1;
            }
            total_length += sorted.len();
        }

        let vocab_size = all_ngrams.len().max(1);
        let typical_doc_length = if texts.is_empty() {
            0
        } else {
            total_length / texts.len()
        };

        CorpusStatistics {
            vocab_size,
            ngram_order: config.n,
            typical_doc_length,
            observed_distinct_ngrams: all_ngrams.len(),
            frequency_distribution: Some(all_ngrams),
        }
    }

    /// Effective n-gram space size: V^k.
    pub fn ngram_space_size(&self) -> f64 {
        (self.vocab_size as f64).powi(self.ngram_order as i32)
    }
}

// ---------------------------------------------------------------------------
// Poisson model for baseline overlap
// ---------------------------------------------------------------------------

/// Models baseline n-gram overlap between independent texts.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BaselineOverlapModel {
    /// Expected number of overlapping n-grams under independence.
    pub expected_overlap: f64,
    /// Poisson rate parameter λ.
    pub lambda: f64,
    /// Standard deviation of the overlap count.
    pub std_dev: f64,
    /// Normalized expected overlap (as fraction of benchmark n-grams).
    pub expected_overlap_fraction: f64,
}

impl BaselineOverlapModel {
    /// Compute the baseline overlap model from corpus statistics.
    ///
    /// For texts of lengths L₁ (benchmark) and L₂ (training) drawn from a
    /// vocabulary with V^k possible n-grams:
    ///   λ = V^k · (1 - e^{-L₁/V^k}) · (1 - e^{-L₂/V^k})
    pub fn from_statistics(
        stats: &CorpusStatistics,
        benchmark_ngrams: usize,
        training_ngrams: usize,
    ) -> Self {
        let vk = stats.ngram_space_size();
        if vk < 1.0 {
            return Self {
                expected_overlap: 0.0,
                lambda: 0.0,
                std_dev: 0.0,
                expected_overlap_fraction: 0.0,
            };
        }

        let l1 = benchmark_ngrams as f64;
        let l2 = training_ngrams as f64;

        // Probability that a specific n-gram appears in a random text of length L:
        // p(L) = 1 - (1 - 1/V^k)^L ≈ 1 - e^{-L/V^k}
        let p1 = 1.0 - (-l1 / vk).exp();
        let p2 = 1.0 - (-l2 / vk).exp();

        let lambda = vk * p1 * p2;
        let std_dev = lambda.sqrt();

        let expected_overlap_fraction = if benchmark_ngrams > 0 {
            lambda / benchmark_ngrams as f64
        } else {
            0.0
        };

        Self {
            expected_overlap: lambda,
            lambda,
            std_dev,
            expected_overlap_fraction,
        }
    }

    /// Compute the baseline model using empirical frequency distribution.
    ///
    /// More accurate than the uniform model: uses the actual n-gram frequency
    /// distribution to compute expected overlap via
    ///   E[|A ∩ B|] = Σ_g p_A(g) · p_B(g)
    /// where p_X(g) = 1 - (1 - f_g)^{L_X} and f_g is the frequency of n-gram g.
    pub fn from_empirical_distribution(
        stats: &CorpusStatistics,
        benchmark_ngrams: usize,
        training_ngrams: usize,
    ) -> Self {
        let freq_dist = match &stats.frequency_distribution {
            Some(fd) => fd,
            None => return Self::from_statistics(stats, benchmark_ngrams, training_ngrams),
        };

        let total_count: usize = freq_dist.values().sum();
        if total_count == 0 {
            return Self {
                expected_overlap: 0.0,
                lambda: 0.0,
                std_dev: 0.0,
                expected_overlap_fraction: 0.0,
            };
        }

        let l1 = benchmark_ngrams as f64;
        let l2 = training_ngrams as f64;
        let total = total_count as f64;

        let mut expected = 0.0;
        for (_ngram, &count) in freq_dist {
            let f_g = count as f64 / total;
            let p1 = 1.0 - (1.0 - f_g).powf(l1);
            let p2 = 1.0 - (1.0 - f_g).powf(l2);
            expected += p1 * p2;
        }

        let std_dev = expected.sqrt(); // Poisson approximation
        let expected_overlap_fraction = if benchmark_ngrams > 0 {
            expected / benchmark_ngrams as f64
        } else {
            0.0
        };

        Self {
            expected_overlap: expected,
            lambda: expected,
            std_dev,
            expected_overlap_fraction,
        }
    }
}

// ---------------------------------------------------------------------------
// Threshold calibration
// ---------------------------------------------------------------------------

/// Result of threshold calibration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CalibrationResult {
    /// Calibrated threshold τ (as overlap fraction).
    pub threshold: f64,
    /// Target false positive rate α.
    pub target_alpha: f64,
    /// Actual false positive rate at the calibrated threshold.
    pub actual_alpha: f64,
    /// Expected false negative rate β at the calibrated threshold
    /// for a given contamination level.
    pub estimated_beta: Option<f64>,
    /// Baseline overlap model used.
    pub baseline: BaselineOverlapModel,
    /// Number of standard deviations above the mean.
    pub z_score: f64,
    /// Confidence interval for baseline overlap fraction [lower, upper].
    pub baseline_ci: [f64; 2],
}

/// Calibrate contamination threshold from corpus statistics.
pub struct ThresholdCalibrator {
    pub stats: CorpusStatistics,
}

impl ThresholdCalibrator {
    pub fn new(stats: CorpusStatistics) -> Self {
        Self { stats }
    }

    /// Calibrate from a sample of texts.
    pub fn from_sample(texts: &[&str], config: &NGramConfig) -> Self {
        let stats = CorpusStatistics::estimate_from_sample(texts, config);
        Self { stats }
    }

    /// Calibrate the threshold τ for a desired false-positive rate α.
    ///
    /// The threshold is set such that P(overlap > τ | no contamination) ≤ α.
    /// Uses the Poisson model with a normal approximation for large λ, or
    /// exact Poisson quantiles for small λ.
    pub fn calibrate(
        &self,
        benchmark_ngrams: usize,
        training_ngrams: usize,
        alpha: f64,
    ) -> CalibrationResult {
        let baseline = if self.stats.frequency_distribution.is_some() {
            BaselineOverlapModel::from_empirical_distribution(
                &self.stats,
                benchmark_ngrams,
                training_ngrams,
            )
        } else {
            BaselineOverlapModel::from_statistics(
                &self.stats,
                benchmark_ngrams,
                training_ngrams,
            )
        };

        // Use normal approximation to Poisson for threshold computation.
        // P(X > k) ≤ α  ⟹  k = λ + z_α · √λ
        let z_alpha = normal_quantile(1.0 - alpha);
        let threshold_count = baseline.lambda + z_alpha * baseline.std_dev;
        let threshold_count = threshold_count.max(0.0);

        let threshold = if benchmark_ngrams > 0 {
            threshold_count / benchmark_ngrams as f64
        } else {
            0.0
        };

        // Actual false positive rate via Poisson survival function
        let actual_alpha = poisson_survival(baseline.lambda, threshold_count.ceil() as usize);

        // 95% CI for baseline overlap fraction
        let ci_lower = (baseline.lambda - 1.96 * baseline.std_dev).max(0.0)
            / benchmark_ngrams.max(1) as f64;
        let ci_upper = (baseline.lambda + 1.96 * baseline.std_dev)
            / benchmark_ngrams.max(1) as f64;

        CalibrationResult {
            threshold,
            target_alpha: alpha,
            actual_alpha,
            estimated_beta: None,
            baseline,
            z_score: z_alpha,
            baseline_ci: [ci_lower, ci_upper],
        }
    }

    /// Calibrate with power analysis: also estimates the false negative rate β
    /// for detecting a specified contamination level.
    pub fn calibrate_with_power(
        &self,
        benchmark_ngrams: usize,
        training_ngrams: usize,
        alpha: f64,
        contamination_level: f64,
    ) -> CalibrationResult {
        let mut result = self.calibrate(benchmark_ngrams, training_ngrams, alpha);

        // Under contamination at level c, the expected overlap increases by
        // c * benchmark_ngrams (contaminated n-grams contribute to overlap).
        let lambda_contaminated =
            result.baseline.lambda + contamination_level * benchmark_ngrams as f64;
        let threshold_count = result.threshold * benchmark_ngrams as f64;

        // β = P(X ≤ threshold | contamination) where X ~ Poisson(λ_contaminated)
        let beta = poisson_cdf(lambda_contaminated, threshold_count.floor() as usize);
        result.estimated_beta = Some(beta);

        result
    }

    /// Empirically calibrate by Monte Carlo simulation.
    ///
    /// Generates `num_trials` pairs of independent random texts and measures
    /// the distribution of overlap fractions.
    pub fn empirical_calibrate(
        &self,
        config: &NGramConfig,
        num_trials: usize,
        alpha: f64,
    ) -> CalibrationResult {
        use super::protocol::SyntheticCorpusGenerator;

        let mut overlaps: Vec<f64> = Vec::with_capacity(num_trials);

        for _ in 0..num_trials {
            let (corpus_a, corpus_b) =
                SyntheticCorpusGenerator::generate_with_overlap(5, 0.0);
            let text_a = corpus_a.join(" ");
            let text_b = corpus_b.join(" ");

            let set_a = NGramSet::from_text(&text_a, config.clone());
            let set_b = NGramSet::from_text(&text_b, config.clone());

            let overlap = if set_a.len() > 0 {
                set_a.intersection_cardinality(&set_b) as f64 / set_a.len() as f64
            } else {
                0.0
            };
            overlaps.push(overlap);
        }

        overlaps.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Threshold at (1 - α) quantile
        let idx = ((1.0 - alpha) * overlaps.len() as f64).ceil() as usize;
        let idx = idx.min(overlaps.len().saturating_sub(1));
        let threshold = overlaps[idx];

        // Compute baseline statistics
        let mean: f64 = overlaps.iter().sum::<f64>() / overlaps.len() as f64;
        let variance: f64 =
            overlaps.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / overlaps.len() as f64;
        let std_dev = variance.sqrt();

        let z_score = if std_dev > 0.0 {
            (threshold - mean) / std_dev
        } else {
            0.0
        };

        // Count false positives at this threshold
        let fp_count = overlaps.iter().filter(|&&x| x > threshold).count();
        let actual_alpha = fp_count as f64 / overlaps.len() as f64;

        CalibrationResult {
            threshold,
            target_alpha: alpha,
            actual_alpha,
            estimated_beta: None,
            baseline: BaselineOverlapModel {
                expected_overlap: mean * self.stats.typical_doc_length as f64,
                lambda: mean * self.stats.typical_doc_length as f64,
                std_dev: std_dev * self.stats.typical_doc_length as f64,
                expected_overlap_fraction: mean,
            },
            z_score,
            baseline_ci: [
                (mean - 1.96 * std_dev).max(0.0),
                mean + 1.96 * std_dev,
            ],
        }
    }
}

// ---------------------------------------------------------------------------
// Statistical helper functions
// ---------------------------------------------------------------------------

/// Normal quantile function (probit) using the rational approximation
/// from Abramowitz & Stegun (1964), formula 26.2.23.
/// Accurate to ~4.5 × 10⁻⁴ for p ∈ (0, 1).
fn normal_quantile(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    if (p - 0.5).abs() < 1e-15 {
        return 0.0;
    }

    let sign = if p < 0.5 { -1.0 } else { 1.0 };
    let p_adj = if p < 0.5 { p } else { 1.0 - p };

    let t = (-2.0 * p_adj.ln()).sqrt();

    // Rational approximation coefficients
    const C0: f64 = 2.515517;
    const C1: f64 = 0.802853;
    const C2: f64 = 0.010328;
    const D1: f64 = 1.432788;
    const D2: f64 = 0.189269;
    const D3: f64 = 0.001308;

    let numerator = C0 + C1 * t + C2 * t * t;
    let denominator = 1.0 + D1 * t + D2 * t * t + D3 * t * t * t;

    sign * (t - numerator / denominator)
}

/// Poisson CDF: P(X ≤ k) where X ~ Poisson(λ).
/// Uses the regularized incomplete gamma function identity.
fn poisson_cdf(lambda: f64, k: usize) -> f64 {
    if lambda <= 0.0 {
        return 1.0;
    }
    // P(X ≤ k) = Γ(k+1, λ) / k! = 1 - Q(k+1, λ)
    // where Q is the regularized upper incomplete gamma function.
    // For moderate k, compute directly via summation.
    if k > 200 {
        // Normal approximation for large k
        let z = (k as f64 + 0.5 - lambda) / lambda.sqrt();
        return normal_cdf(z);
    }

    let mut sum = 0.0;
    let mut term = (-lambda).exp(); // P(X = 0)
    sum += term;
    for i in 1..=k {
        term *= lambda / i as f64;
        sum += term;
    }
    sum.min(1.0)
}

/// Poisson survival function: P(X > k) = 1 - P(X ≤ k).
fn poisson_survival(lambda: f64, k: usize) -> f64 {
    1.0 - poisson_cdf(lambda, k)
}

/// Standard normal CDF using error function approximation.
fn normal_cdf(z: f64) -> f64 {
    0.5 * (1.0 + erf(z / std::f64::consts::SQRT_2))
}

/// Error function approximation (Horner form of Abramowitz & Stegun 7.1.26).
fn erf(x: f64) -> f64 {
    let sign = if x >= 0.0 { 1.0 } else { -1.0 };
    let x = x.abs();

    const A1: f64 = 0.254829592;
    const A2: f64 = -0.284496736;
    const A3: f64 = 1.421413741;
    const A4: f64 = -1.453152027;
    const A5: f64 = 1.061405429;
    const P: f64 = 0.3275911;

    let t = 1.0 / (1.0 + P * x);
    let poly = ((((A5 * t + A4) * t + A3) * t + A2) * t + A1) * t;
    sign * (1.0 - poly * (-x * x).exp())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_quantile_basic() {
        let z50 = normal_quantile(0.5);
        assert!((z50).abs() < 1e-10, "z(0.5) should be 0, got {}", z50);

        let z95 = normal_quantile(0.95);
        assert!((z95 - 1.645).abs() < 0.01, "z(0.95) ≈ 1.645, got {}", z95);

        let z99 = normal_quantile(0.99);
        assert!((z99 - 2.326).abs() < 0.01, "z(0.99) ≈ 2.326, got {}", z99);
    }

    #[test]
    fn test_poisson_cdf_basic() {
        // Poisson(1.0): P(X ≤ 0) = e^{-1} ≈ 0.3679
        let p = poisson_cdf(1.0, 0);
        assert!((p - 0.3679).abs() < 0.001, "P(X≤0|λ=1) ≈ 0.3679, got {}", p);

        // Poisson(1.0): P(X ≤ 2) ≈ 0.9197
        let p = poisson_cdf(1.0, 2);
        assert!((p - 0.9197).abs() < 0.001, "P(X≤2|λ=1) ≈ 0.9197, got {}", p);
    }

    #[test]
    fn test_erf_basic() {
        assert!((erf(0.0)).abs() < 1e-6);
        assert!((erf(1.0) - 0.8427).abs() < 0.001);
        assert!((erf(-1.0) + 0.8427).abs() < 0.001);
    }

    #[test]
    fn test_baseline_overlap_model_independent_texts() {
        let stats = CorpusStatistics {
            vocab_size: 100,
            ngram_order: 3,
            typical_doc_length: 500,
            observed_distinct_ngrams: 50000,
            frequency_distribution: None,
        };

        let model = BaselineOverlapModel::from_statistics(&stats, 500, 1000);
        // V^k = 100^3 = 1,000,000. L1=500, L2=1000
        // λ = 10^6 * (1 - e^{-500/10^6}) * (1 - e^{-1000/10^6})
        // ≈ 10^6 * 0.0005 * 0.001 = 0.5
        assert!(model.lambda < 2.0, "λ should be small for large V^k, got {}", model.lambda);
        assert!(model.expected_overlap_fraction < 0.01);
    }

    #[test]
    fn test_calibration_threshold_increases_with_stricter_alpha() {
        let stats = CorpusStatistics {
            vocab_size: 50,
            ngram_order: 2,
            typical_doc_length: 200,
            observed_distinct_ngrams: 2500,
            frequency_distribution: None,
        };

        let cal = ThresholdCalibrator::new(stats);
        let result_05 = cal.calibrate(200, 200, 0.05);
        let result_01 = cal.calibrate(200, 200, 0.01);

        // Stricter α (lower false positive tolerance) → higher threshold
        assert!(
            result_01.threshold >= result_05.threshold,
            "τ(α=0.01) = {} should be ≥ τ(α=0.05) = {}",
            result_01.threshold,
            result_05.threshold
        );
    }

    #[test]
    fn test_calibration_with_power() {
        let stats = CorpusStatistics {
            vocab_size: 50,
            ngram_order: 2,
            typical_doc_length: 200,
            observed_distinct_ngrams: 2500,
            frequency_distribution: None,
        };

        let cal = ThresholdCalibrator::new(stats);
        let result = cal.calibrate_with_power(200, 200, 0.05, 0.1);

        assert!(result.estimated_beta.is_some());
        let beta = result.estimated_beta.unwrap();
        // With 10% contamination, false negative rate should be reasonable
        assert!(beta >= 0.0 && beta <= 1.0, "β should be in [0,1], got {}", beta);
    }

    #[test]
    fn test_corpus_statistics_from_sample() {
        let texts = vec![
            "the quick brown fox jumps over the lazy dog",
            "a fast red fox leaps across a sleepy hound",
            "neural networks learn representations from data",
        ];
        let config = NGramConfig::char_ngrams(3);
        let stats = CorpusStatistics::estimate_from_sample(&texts, &config);

        assert!(stats.vocab_size > 0);
        assert_eq!(stats.ngram_order, 3);
        assert!(stats.typical_doc_length > 0);
        assert!(stats.frequency_distribution.is_some());
    }
}
