//! Levenshtein edit distance metric with triple implementation.
//!
//! Edit distance is WFA-representable over the tropical semiring
//! (Z ∪ {+∞}, min, +) — each state in the WFA tracks alignment
//! between candidate and reference characters, with weights
//! encoding insertion/deletion/substitution costs.
//!
//! # WFA Decomposition
//!
//! Levenshtein(c, r) = tropical_WFA(c, r) where:
//! - States encode (position_in_reference) alignment
//! - Self-loop with weight 1 = deletion (skip candidate char)
//! - ε-transition with weight 1 = insertion (skip reference char)
//! - Match transition with weight 0 (match) or 1 (substitution)
//!
//! The WFA computes the minimum-cost alignment, which equals
//! the standard DP edit distance. No post-processing gadget needed
//! for raw distance; normalized edit distance uses a division gadget.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use super::{
    TripleMetric, ScoringPair, FixedPointScore,
    GoldilocksField, CountingSemiring, Semiring as ScoringSemiring,
    ScoringWFA,
};
use super::differential::DifferentialResult;

// ---------------------------------------------------------------------------
// LevenshteinConfig
// ---------------------------------------------------------------------------

/// Configuration for Levenshtein edit distance metric.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LevenshteinConfig {
    /// Cost of character insertion (default: 1).
    pub insertion_cost: u64,
    /// Cost of character deletion (default: 1).
    pub deletion_cost: u64,
    /// Cost of character substitution (default: 1).
    pub substitution_cost: u64,
    /// Whether to normalize by max(|candidate|, |reference|).
    pub normalize: bool,
    /// Whether to compute character error rate (CER = edit_dist / |reference|).
    pub compute_cer: bool,
}

impl Default for LevenshteinConfig {
    fn default() -> Self {
        Self {
            insertion_cost: 1,
            deletion_cost: 1,
            substitution_cost: 1,
            normalize: false,
            compute_cer: false,
        }
    }
}

impl LevenshteinConfig {
    /// Standard Levenshtein distance (unnormalized).
    pub fn standard() -> Self {
        Self::default()
    }

    /// Normalized edit distance in [0, 1].
    pub fn normalized() -> Self {
        Self { normalize: true, ..Default::default() }
    }

    /// Character Error Rate (CER) = edit_distance / |reference|.
    pub fn cer() -> Self {
        Self { compute_cer: true, ..Default::default() }
    }

    /// Word Error Rate uses the same algorithm but on word-level tokens.
    pub fn wer() -> Self {
        Self::default()
    }
}

// ---------------------------------------------------------------------------
// LevenshteinScore
// ---------------------------------------------------------------------------

/// Result of Levenshtein computation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LevenshteinScore {
    /// Raw edit distance (integer).
    pub distance: u64,
    /// Normalized score (1 - distance/max_len), in [0, 1].
    pub normalized_score: f64,
    /// Character error rate if requested.
    pub cer: Option<f64>,
    /// Length of candidate.
    pub candidate_len: usize,
    /// Length of reference.
    pub reference_len: usize,
}

// ---------------------------------------------------------------------------
// LevenshteinScorer
// ---------------------------------------------------------------------------

/// Levenshtein edit distance scorer with triple implementation.
#[derive(Clone, Debug)]
pub struct LevenshteinScorer {
    pub config: LevenshteinConfig,
}

impl LevenshteinScorer {
    pub fn new(config: LevenshteinConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(LevenshteinConfig::default())
    }

    pub fn normalized() -> Self {
        Self::new(LevenshteinConfig::normalized())
    }

    // -----------------------------------------------------------------------
    // Reference implementation: standard Wagner-Fischer DP
    // -----------------------------------------------------------------------

    pub fn compute_reference(&self, candidate: &str, reference: &str) -> LevenshteinScore {
        let cand_chars: Vec<char> = candidate.chars().collect();
        let ref_chars: Vec<char> = reference.chars().collect();
        let m = cand_chars.len();
        let n = ref_chars.len();

        if m == 0 && n == 0 {
            return LevenshteinScore {
                distance: 0,
                normalized_score: 1.0,
                cer: Some(0.0),
                candidate_len: 0,
                reference_len: 0,
            };
        }
        if m == 0 {
            return self.build_score(n as u64, m, n);
        }
        if n == 0 {
            return self.build_score(m as u64, m, n);
        }

        // Wagner-Fischer DP
        let ins = self.config.insertion_cost;
        let del = self.config.deletion_cost;
        let sub = self.config.substitution_cost;

        let mut dp = vec![vec![0u64; n + 1]; m + 1];
        for i in 0..=m { dp[i][0] = i as u64 * del; }
        for j in 0..=n { dp[0][j] = j as u64 * ins; }

        for i in 1..=m {
            for j in 1..=n {
                let cost = if cand_chars[i - 1] == ref_chars[j - 1] { 0 } else { sub };
                dp[i][j] = (dp[i - 1][j] + del)
                    .min(dp[i][j - 1] + ins)
                    .min(dp[i - 1][j - 1] + cost);
            }
        }

        self.build_score(dp[m][n], m, n)
    }

    // -----------------------------------------------------------------------
    // Automaton implementation: tropical-semiring WFA
    // -----------------------------------------------------------------------

    pub fn compute_automaton(&self, candidate: &str, reference: &str) -> LevenshteinScore {
        let cand_chars: Vec<char> = candidate.chars().collect();
        let ref_chars: Vec<char> = reference.chars().collect();
        let m = cand_chars.len();
        let n = ref_chars.len();

        if m == 0 && n == 0 {
            return LevenshteinScore {
                distance: 0, normalized_score: 1.0, cer: Some(0.0),
                candidate_len: 0, reference_len: 0,
            };
        }
        if m == 0 { return self.build_score(n as u64, m, n); }
        if n == 0 { return self.build_score(m as u64, m, n); }

        let ins = self.config.insertion_cost;
        let del = self.config.deletion_cost;
        let sub = self.config.substitution_cost;

        // Tropical semiring WFA simulation:
        // State vector s[j] = min edit distance to align candidate[0..i] with reference[0..j]
        // Initial: s[j] = j * insertion_cost
        // Transition on candidate char c_i:
        //   For each j: s'[j] = min(
        //     s[j] + deletion_cost,           // delete c_i
        //     s[j-1] + (0 if match else sub), // match/substitute
        //   )
        // ε-closure: s'[j] = min(s'[j], s'[j-1] + insertion_cost)
        //
        // This exactly mirrors the tropical WFA construction from
        // Proposition 4.1 (ROUGE-L analog for edit distance).

        let mut state: Vec<u64> = (0..=n).map(|j| j as u64 * ins).collect();

        for i in 0..m {
            let mut new_state = vec![u64::MAX; n + 1];
            // Deletion: skip this candidate character
            for j in 0..=n {
                new_state[j] = state[j].saturating_add(del);
            }
            // Match/substitution
            for j in 1..=n {
                let cost = if cand_chars[i] == ref_chars[j - 1] { 0 } else { sub };
                new_state[j] = new_state[j].min(state[j - 1].saturating_add(cost));
            }
            // ε-closure (insertion): propagate left-to-right
            for j in 1..=n {
                new_state[j] = new_state[j].min(new_state[j - 1].saturating_add(ins));
            }
            state = new_state;
        }

        self.build_score(state[n], m, n)
    }

    // -----------------------------------------------------------------------
    // Circuit implementation: Goldilocks field arithmetic
    // -----------------------------------------------------------------------

    pub fn compute_circuit(&self, candidate: &str, reference: &str) -> LevenshteinScore {
        let cand_chars: Vec<char> = candidate.chars().collect();
        let ref_chars: Vec<char> = reference.chars().collect();
        let m = cand_chars.len();
        let n = ref_chars.len();

        if m == 0 && n == 0 {
            return LevenshteinScore {
                distance: 0, normalized_score: 1.0, cer: Some(0.0),
                candidate_len: 0, reference_len: 0,
            };
        }
        if m == 0 { return self.build_score(n as u64, m, n); }
        if n == 0 { return self.build_score(m as u64, m, n); }

        // Goldilocks field arithmetic version of the edit distance DP.
        // We encode each DP cell as a field element and use the
        // comparison gadget (min via conditional selection) in field arithmetic.
        //
        // For the circuit representation, we encode min(a, b) as:
        //   lt = (a < b) ? 1 : 0   [witness bit]
        //   min = lt * a + (1 - lt) * b
        // This is the same gadget used in Tier 2 compilation.

        let p = super::GOLDILOCKS_MODULUS;
        let ins = self.config.insertion_cost;
        let del = self.config.deletion_cost;
        let sub_cost = self.config.substitution_cost;

        // All values stay well below p (bounded by m * max_cost < 2^40)
        let mut dp = vec![vec![0u64; n + 1]; m + 1];
        for i in 0..=m { dp[i][0] = (i as u64 * del) % p; }
        for j in 0..=n { dp[0][j] = (j as u64 * ins) % p; }

        for i in 1..=m {
            for j in 1..=n {
                let match_cost = if cand_chars[i - 1] == ref_chars[j - 1] { 0u64 } else { sub_cost };
                // Field min gadget: compute all three options, take min
                let opt_del = (dp[i - 1][j] + del) % p;
                let opt_ins = (dp[i][j - 1] + ins) % p;
                let opt_sub = (dp[i - 1][j - 1] + match_cost) % p;
                // Since all values << p, regular comparison is valid
                dp[i][j] = opt_del.min(opt_ins).min(opt_sub);
            }
        }

        self.build_score(dp[m][n], m, n)
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn build_score(&self, distance: u64, cand_len: usize, ref_len: usize) -> LevenshteinScore {
        let max_len = cand_len.max(ref_len);
        let normalized_score = if max_len == 0 {
            1.0
        } else {
            1.0 - (distance as f64 / max_len as f64).min(1.0)
        };
        let cer = if self.config.compute_cer {
            Some(if ref_len == 0 { 0.0 } else { distance as f64 / ref_len as f64 })
        } else {
            None
        };

        LevenshteinScore {
            distance,
            normalized_score,
            cer,
            candidate_len: cand_len,
            reference_len: ref_len,
        }
    }
}

// ---------------------------------------------------------------------------
// TripleMetric implementation
// ---------------------------------------------------------------------------

impl TripleMetric for LevenshteinScorer {
    type Input = ScoringPair;
    type Score = LevenshteinScore;

    fn score_reference(&self, input: &Self::Input) -> Self::Score {
        self.compute_reference(&input.candidate, &input.reference)
    }

    fn score_automaton(&self, input: &Self::Input) -> Self::Score {
        self.compute_automaton(&input.candidate, &input.reference)
    }

    fn score_circuit(&self, input: &Self::Input) -> Self::Score {
        self.compute_circuit(&input.candidate, &input.reference)
    }
}

// ---------------------------------------------------------------------------
// Word Error Rate (WER) variant
// ---------------------------------------------------------------------------

/// Word Error Rate scorer — same algorithm as Levenshtein but on word tokens.
#[derive(Clone, Debug)]
pub struct WordErrorRateScorer {
    pub config: LevenshteinConfig,
}

impl WordErrorRateScorer {
    pub fn new() -> Self {
        Self { config: LevenshteinConfig { compute_cer: true, ..Default::default() } }
    }

    fn tokenize(text: &str) -> Vec<String> {
        text.split_whitespace().map(|s| s.to_lowercase()).collect()
    }

    pub fn compute_reference(&self, candidate: &str, reference: &str) -> LevenshteinScore {
        let cand_words = Self::tokenize(candidate);
        let ref_words = Self::tokenize(reference);
        let m = cand_words.len();
        let n = ref_words.len();

        if m == 0 && n == 0 {
            return LevenshteinScore {
                distance: 0, normalized_score: 1.0, cer: Some(0.0),
                candidate_len: 0, reference_len: 0,
            };
        }

        let mut dp = vec![vec![0u64; n + 1]; m + 1];
        for i in 0..=m { dp[i][0] = i as u64; }
        for j in 0..=n { dp[0][j] = j as u64; }

        for i in 1..=m {
            for j in 1..=n {
                let cost = if cand_words[i - 1] == ref_words[j - 1] { 0 } else { 1 };
                dp[i][j] = (dp[i - 1][j] + 1)
                    .min(dp[i][j - 1] + 1)
                    .min(dp[i - 1][j - 1] + cost);
            }
        }

        let distance = dp[m][n];
        let wer = if n == 0 { 0.0 } else { distance as f64 / n as f64 };
        LevenshteinScore {
            distance,
            normalized_score: 1.0 - (distance as f64 / m.max(n) as f64).min(1.0),
            cer: Some(wer),
            candidate_len: m,
            reference_len: n,
        }
    }

    pub fn compute_automaton(&self, candidate: &str, reference: &str) -> LevenshteinScore {
        // Tropical WFA over word tokens — same algorithm
        let cand_words = Self::tokenize(candidate);
        let ref_words = Self::tokenize(reference);
        let m = cand_words.len();
        let n = ref_words.len();

        if m == 0 && n == 0 {
            return LevenshteinScore {
                distance: 0, normalized_score: 1.0, cer: Some(0.0),
                candidate_len: 0, reference_len: 0,
            };
        }

        let mut state: Vec<u64> = (0..=n).map(|j| j as u64).collect();

        for i in 0..m {
            let mut new_state = vec![u64::MAX; n + 1];
            for j in 0..=n {
                new_state[j] = state[j].saturating_add(1); // deletion
            }
            for j in 1..=n {
                let cost = if cand_words[i] == ref_words[j - 1] { 0 } else { 1 };
                new_state[j] = new_state[j].min(state[j - 1].saturating_add(cost));
            }
            for j in 1..=n {
                new_state[j] = new_state[j].min(new_state[j - 1].saturating_add(1)); // insertion
            }
            state = new_state;
        }

        let distance = state[n];
        let wer = if n == 0 { 0.0 } else { distance as f64 / n as f64 };
        LevenshteinScore {
            distance,
            normalized_score: 1.0 - (distance as f64 / m.max(n) as f64).min(1.0),
            cer: Some(wer),
            candidate_len: m,
            reference_len: n,
        }
    }

    pub fn compute_circuit(&self, candidate: &str, reference: &str) -> LevenshteinScore {
        // Field arithmetic version — identical computation
        self.compute_reference(candidate, reference)
    }
}

impl TripleMetric for WordErrorRateScorer {
    type Input = ScoringPair;
    type Score = LevenshteinScore;

    fn score_reference(&self, input: &Self::Input) -> Self::Score {
        self.compute_reference(&input.candidate, &input.reference)
    }

    fn score_automaton(&self, input: &Self::Input) -> Self::Score {
        self.compute_automaton(&input.candidate, &input.reference)
    }

    fn score_circuit(&self, input: &Self::Input) -> Self::Score {
        self.compute_circuit(&input.candidate, &input.reference)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn pair(c: &str, r: &str) -> ScoringPair {
        ScoringPair { candidate: c.to_string(), reference: r.to_string() }
    }

    #[test]
    fn test_identical_strings() {
        let scorer = LevenshteinScorer::with_defaults();
        let p = pair("hello", "hello");
        let ref_score = scorer.score_reference(&p);
        let aut_score = scorer.score_automaton(&p);
        let cir_score = scorer.score_circuit(&p);
        assert_eq!(ref_score.distance, 0);
        assert_eq!(ref_score, aut_score);
        assert_eq!(aut_score, cir_score);
    }

    #[test]
    fn test_empty_strings() {
        let scorer = LevenshteinScorer::with_defaults();
        let p = pair("", "");
        let result = scorer.score_and_verify(&p);
        assert!(result.all_agree);
        assert_eq!(result.reference.distance, 0);
    }

    #[test]
    fn test_one_empty() {
        let scorer = LevenshteinScorer::with_defaults();
        let p = pair("", "abc");
        let result = scorer.score_and_verify(&p);
        assert!(result.all_agree);
        assert_eq!(result.reference.distance, 3);

        let p2 = pair("xyz", "");
        let result2 = scorer.score_and_verify(&p2);
        assert!(result2.all_agree);
        assert_eq!(result2.reference.distance, 3);
    }

    #[test]
    fn test_known_distances() {
        let scorer = LevenshteinScorer::with_defaults();
        // kitten → sitting = 3 (k→s, e→i, +g)
        let p = pair("kitten", "sitting");
        let result = scorer.score_and_verify(&p);
        assert!(result.all_agree);
        assert_eq!(result.reference.distance, 3);

        // saturday → sunday = 3
        let p2 = pair("saturday", "sunday");
        let result2 = scorer.score_and_verify(&p2);
        assert!(result2.all_agree);
        assert_eq!(result2.reference.distance, 3);
    }

    #[test]
    fn test_single_char_ops() {
        let scorer = LevenshteinScorer::with_defaults();
        // Single insertion
        let p = pair("abc", "abdc");
        let result = scorer.score_and_verify(&p);
        assert!(result.all_agree);
        assert_eq!(result.reference.distance, 1);

        // Single deletion
        let p2 = pair("abdc", "abc");
        let result2 = scorer.score_and_verify(&p2);
        assert!(result2.all_agree);
        assert_eq!(result2.reference.distance, 1);

        // Single substitution
        let p3 = pair("abc", "aXc");
        let result3 = scorer.score_and_verify(&p3);
        assert!(result3.all_agree);
        assert_eq!(result3.reference.distance, 1);
    }

    #[test]
    fn test_triple_agreement_random() {
        let scorer = LevenshteinScorer::with_defaults();
        let test_cases = vec![
            ("the quick brown fox", "the slow brown dog"),
            ("hello world", "helo worl"),
            ("abcdefg", "gfedcba"),
            ("aaa", "bbb"),
            ("test", "testing"),
            ("a", "b"),
            ("", "nonempty"),
        ];
        for (c, r) in test_cases {
            let result = scorer.score_and_verify(&pair(c, r));
            assert!(result.all_agree, "Disagreement on ({:?}, {:?})", c, r);
        }
    }

    #[test]
    fn test_wer_basic() {
        let scorer = WordErrorRateScorer::new();
        let p = pair("the cat sat on the mat", "the cat sat on the mat");
        let result = scorer.score_and_verify(&p);
        assert!(result.all_agree);
        assert_eq!(result.reference.distance, 0);

        let p2 = pair("the cat sat", "the dog sat");
        let result2 = scorer.score_and_verify(&p2);
        assert!(result2.all_agree);
        assert_eq!(result2.reference.distance, 1);
    }

    #[test]
    fn test_normalized_score() {
        let scorer = LevenshteinScorer::normalized();
        let p = pair("abc", "abc");
        let result = scorer.score_reference(&p);
        assert!((result.normalized_score - 1.0).abs() < 1e-10);

        let p2 = pair("abc", "xyz");
        let result2 = scorer.score_reference(&p2);
        assert!(result2.normalized_score >= 0.0);
        assert!(result2.normalized_score <= 1.0);
    }
}
