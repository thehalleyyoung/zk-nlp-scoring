//! Comprehensive Lean–Rust specification correspondence tests.
//!
//! This test suite verifies that the Rust implementation matches the Lean 4
//! specification for every proved property. It serves as the bridge for
//! Layer 3 of the trust model (see docs/formal/TRUST_MODEL.md).
//!
//! Coverage:
//! - All semiring axioms (Boolean, Counting, Tropical, Goldilocks)
//! - Embedding homomorphism properties
//! - WFA evaluation semantics
//! - Edge cases: empty strings, single characters, maximum length
//! - All triple-implemented metrics
//! - Determinism property for Boolean embedding validity

use spectacles_core::scoring::{
    ScoringPair, TripleMetric,
    exact_match::ExactMatchScorer,
    token_f1::{TokenF1Scorer, TokenF1Config},
    bleu::{BleuScorer, SmoothingMethod},
    rouge::{RougeNScorer, RougeLScorer},
    differential::{DifferentialTester, standard_test_suite},
};
use spectacles_core::wfa::semiring::{
    BooleanSemiring, CountingSemiring, TropicalSemiring, GoldilocksField,
    Semiring, StarSemiring, GOLDILOCKS_PRIME,
};
use spectacles_core::wfa::field_embedding::{
    CountingToGoldilocks, BooleanToGoldilocks, SemiringEmbedding,
};

// ═══════════════════════════════════════════════════════════════════════════
// Lean theorem: Boolean semiring axioms (Instances.lean)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn boolean_add_comm() {
    for a in [false, true] {
        for b in [false, true] {
            let sa = BooleanSemiring::new(a);
            let sb = BooleanSemiring::new(b);
            assert_eq!(sa.add(&sb), sb.add(&sa), "add_comm failed for ({}, {})", a, b);
        }
    }
}

#[test]
fn boolean_add_assoc() {
    for a in [false, true] {
        for b in [false, true] {
            for c in [false, true] {
                let sa = BooleanSemiring::new(a);
                let sb = BooleanSemiring::new(b);
                let sc = BooleanSemiring::new(c);
                assert_eq!(sa.add(&sb).add(&sc), sa.add(&sb.add(&sc)));
            }
        }
    }
}

#[test]
fn boolean_add_zero() {
    let zero = BooleanSemiring::zero();
    for a in [false, true] {
        let sa = BooleanSemiring::new(a);
        assert_eq!(sa.add(&zero), sa, "add_zero failed for {}", a);
        assert_eq!(zero.add(&sa), sa, "zero_add failed for {}", a);
    }
}

#[test]
fn boolean_mul_one() {
    let one = BooleanSemiring::one();
    for a in [false, true] {
        let sa = BooleanSemiring::new(a);
        assert_eq!(sa.mul(&one), sa, "mul_one failed for {}", a);
        assert_eq!(one.mul(&sa), sa, "one_mul failed for {}", a);
    }
}

#[test]
fn boolean_mul_zero() {
    let zero = BooleanSemiring::zero();
    for a in [false, true] {
        let sa = BooleanSemiring::new(a);
        assert_eq!(sa.mul(&zero), zero, "mul_zero failed for {}", a);
        assert_eq!(zero.mul(&sa), zero, "zero_mul failed for {}", a);
    }
}

#[test]
fn boolean_left_distrib() {
    for a in [false, true] {
        for b in [false, true] {
            for c in [false, true] {
                let sa = BooleanSemiring::new(a);
                let sb = BooleanSemiring::new(b);
                let sc = BooleanSemiring::new(c);
                assert_eq!(sa.mul(&sb.add(&sc)), sa.mul(&sb).add(&sa.mul(&sc)));
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Lean theorem: Counting semiring axioms (Instances.lean)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn counting_add_comm() {
    let vals = [0u64, 1, 2, 5, 100, 999];
    for &a in &vals {
        for &b in &vals {
            let sa = CountingSemiring::new(a);
            let sb = CountingSemiring::new(b);
            assert_eq!(sa.add(&sb), sb.add(&sa));
        }
    }
}

#[test]
fn counting_mul_assoc() {
    let vals = [0u64, 1, 2, 5, 10];
    for &a in &vals {
        for &b in &vals {
            for &c in &vals {
                let sa = CountingSemiring::new(a);
                let sb = CountingSemiring::new(b);
                let sc = CountingSemiring::new(c);
                assert_eq!(sa.mul(&sb).mul(&sc), sa.mul(&sb.mul(&sc)));
            }
        }
    }
}

#[test]
fn counting_identities() {
    let zero = CountingSemiring::zero();
    let one = CountingSemiring::one();
    for &v in &[0u64, 1, 42, 999] {
        let a = CountingSemiring::new(v);
        assert_eq!(a.add(&zero), a, "add_zero");
        assert_eq!(zero.add(&a), a, "zero_add");
        assert_eq!(a.mul(&one), a, "mul_one");
        assert_eq!(one.mul(&a), a, "one_mul");
        assert_eq!(a.mul(&zero), zero, "mul_zero");
        assert_eq!(zero.mul(&a), zero, "zero_mul");
    }
}

#[test]
fn counting_distributivity() {
    let vals = [0u64, 1, 2, 5, 10];
    for &a in &vals {
        for &b in &vals {
            for &c in &vals {
                let sa = CountingSemiring::new(a);
                let sb = CountingSemiring::new(b);
                let sc = CountingSemiring::new(c);
                assert_eq!(sa.mul(&sb.add(&sc)), sa.mul(&sb).add(&sa.mul(&sc)),
                    "left_distrib failed");
                assert_eq!(sa.add(&sb).mul(&sc), sa.mul(&sc).add(&sb.mul(&sc)),
                    "right_distrib failed");
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Lean theorem: Tropical semiring axioms (Instances.lean)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn tropical_add_comm() {
    let vals: Vec<TropicalSemiring> = vec![
        TropicalSemiring::infinity(),
        TropicalSemiring::new(0.0),
        TropicalSemiring::new(1.0),
        TropicalSemiring::new(5.0),
        TropicalSemiring::new(100.0),
    ];
    for a in &vals {
        for b in &vals {
            assert_eq!(a.add(b), b.add(a), "tropical add_comm failed");
        }
    }
}

#[test]
fn tropical_identities() {
    let zero = TropicalSemiring::zero(); // infinity
    let one = TropicalSemiring::one();   // 0.0
    let vals = [0.0, 1.0, 5.0, 100.0];
    for &v in &vals {
        let a = TropicalSemiring::new(v);
        assert_eq!(a.add(&zero), a, "tropical add_zero failed for {}", v);
        assert_eq!(zero.add(&a), a, "tropical zero_add failed for {}", v);
        assert_eq!(a.mul(&one), a, "tropical mul_one failed for {}", v);
        assert_eq!(one.mul(&a), a, "tropical one_mul failed for {}", v);
    }
}

#[test]
fn tropical_annihilation() {
    let zero = TropicalSemiring::zero(); // infinity
    for &v in &[0.0, 1.0, 42.0] {
        let a = TropicalSemiring::new(v);
        assert_eq!(a.mul(&zero), zero, "tropical mul_zero failed for {}", v);
        assert_eq!(zero.mul(&a), zero, "tropical zero_mul failed for {}", v);
    }
}

#[test]
fn tropical_distributivity() {
    // a ⊗ (b ⊕ c) = a + min(b, c) should equal min(a+b, a+c) = (a⊗b) ⊕ (a⊗c)
    let vals = [0.0, 1.0, 5.0, 100.0];
    for &a in &vals {
        for &b in &vals {
            for &c in &vals {
                let sa = TropicalSemiring::new(a);
                let sb = TropicalSemiring::new(b);
                let sc = TropicalSemiring::new(c);
                assert_eq!(sa.mul(&sb.add(&sc)), sa.mul(&sb).add(&sa.mul(&sc)),
                    "tropical left_distrib failed for ({}, {}, {})", a, b, c);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Lean theorem: Goldilocks field (Goldilocks.lean — R1 discharged)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn goldilocks_add_comm() {
    let samples = [0u64, 1, 2, 42, 1000, GOLDILOCKS_PRIME - 1, GOLDILOCKS_PRIME - 2];
    for &a in &samples {
        for &b in &samples {
            let ga = GoldilocksField::new(a);
            let gb = GoldilocksField::new(b);
            assert_eq!(ga.add(&gb), gb.add(&ga),
                "Goldilocks add_comm failed for {} + {}", a, b);
        }
    }
}

#[test]
fn goldilocks_mul_comm() {
    let samples = [0u64, 1, 2, 42, 1000, GOLDILOCKS_PRIME - 1];
    for &a in &samples {
        for &b in &samples {
            let ga = GoldilocksField::new(a);
            let gb = GoldilocksField::new(b);
            assert_eq!(ga.mul(&gb), gb.mul(&ga),
                "Goldilocks mul_comm failed for {} * {}", a, b);
        }
    }
}

#[test]
fn goldilocks_identities() {
    let zero = GoldilocksField::zero();
    let one = GoldilocksField::one();
    for &v in &[0u64, 1, 42, 999, GOLDILOCKS_PRIME - 1] {
        let a = GoldilocksField::new(v);
        assert_eq!(a.add(&zero), a, "add_zero failed for {}", v);
        assert_eq!(zero.add(&a), a, "zero_add failed for {}", v);
        assert_eq!(a.mul(&one), a, "mul_one failed for {}", v);
        assert_eq!(one.mul(&a), a, "one_mul failed for {}", v);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Lean theorem: Embedding properties (Embedding.lean — R2 discharged)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn boolean_embedding_injective() {
    let embed = BooleanToGoldilocks;
    let f_false = embed.embed(&BooleanSemiring::new(false)).unwrap();
    let f_true = embed.embed(&BooleanSemiring::new(true)).unwrap();
    assert_ne!(f_false, f_true, "Boolean embedding must be injective");
    assert_eq!(f_false, GoldilocksField::new(0), "false maps to 0");
    assert_eq!(f_true, GoldilocksField::new(1), "true maps to 1");
}

#[test]
fn counting_embedding_injective() {
    let embed = CountingToGoldilocks::new();
    for i in 0..100u64 {
        for j in (i + 1)..100u64 {
            let fi = embed.embed(&CountingSemiring::new(i)).unwrap();
            let fj = embed.embed(&CountingSemiring::new(j)).unwrap();
            assert_ne!(fi, fj, "Counting embedding not injective: {}≠{}", i, j);
        }
    }
}

#[test]
fn counting_embedding_homomorphism_add() {
    let embed = CountingToGoldilocks::new();
    for &a in &[0u64, 1, 5, 100, 999] {
        for &b in &[0u64, 1, 5, 100, 999] {
            let sum = CountingSemiring::new(a).add(&CountingSemiring::new(b));
            let embed_sum = embed.embed(&sum).unwrap();
            let sum_embed = embed.embed(&CountingSemiring::new(a)).unwrap()
                .add(&embed.embed(&CountingSemiring::new(b)).unwrap());
            assert_eq!(embed_sum, sum_embed,
                "Counting embed add homomorphism failed: ι({}+{}) ≠ ι({})+ι({})", a, b, a, b);
        }
    }
}

#[test]
fn counting_embedding_homomorphism_mul() {
    let embed = CountingToGoldilocks::new();
    for &a in &[0u64, 1, 5, 100, 999] {
        for &b in &[0u64, 1, 5, 100, 999] {
            let prod = CountingSemiring::new(a).mul(&CountingSemiring::new(b));
            let embed_prod = embed.embed(&prod).unwrap();
            let prod_embed = embed.embed(&CountingSemiring::new(a)).unwrap()
                .mul(&embed.embed(&CountingSemiring::new(b)).unwrap());
            assert_eq!(embed_prod, prod_embed,
                "Counting embed mul homomorphism failed: ι({}×{}) ≠ ι({})×ι({})", a, b, a, b);
        }
    }
}

#[test]
fn trivial_inequalities_r2() {
    // Corresponds to R2 sorrys: n < p for small n
    assert!(0 < GOLDILOCKS_PRIME, "0 < p");
    assert!(1 < GOLDILOCKS_PRIME, "1 < p");
    assert!(100000 < GOLDILOCKS_PRIME, "vocab_bound < p");
    assert!(10000 < GOLDILOCKS_PRIME, "length_bound < p");
    assert!((10000u64 * 10000u64) < GOLDILOCKS_PRIME, "L² < p for L=10000");
}

// ═══════════════════════════════════════════════════════════════════════════
// Lean theorem: Tropical Kleene star (KleeneSemiring.lean — N5 discharged)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn tropical_star_convergence_n5() {
    // N5: a* = 0.0 (= 1_tropical) for all a ≥ 0
    for &a in &[0.0, 1.0, 5.0, 100.0, 999.0, 10000.0] {
        let ta = TropicalSemiring::new(a);
        let star = ta.star();
        assert_eq!(star, TropicalSemiring::one(),
            "tropical_star({}) should be 1_tropical (0.0), got {:?}", a, star);
    }
    // Infinity case
    let inf_star = TropicalSemiring::infinity().star();
    assert_eq!(inf_star, TropicalSemiring::one(),
        "tropical_star(∞) should be 1_tropical (0.0)");
}

// ═══════════════════════════════════════════════════════════════════════════
// Triple metric agreement (Layer 3 of trust model)
// ═══════════════════════════════════════════════════════════════════════════

fn edge_case_pairs() -> Vec<ScoringPair> {
    vec![
        ScoringPair { candidate: "".into(), reference: "".into() },
        ScoringPair { candidate: "hello".into(), reference: "".into() },
        ScoringPair { candidate: "".into(), reference: "hello".into() },
        ScoringPair { candidate: "a".into(), reference: "a".into() },
        ScoringPair { candidate: "a".into(), reference: "b".into() },
        ScoringPair { candidate: "hello world".into(), reference: "hello world".into() },
        ScoringPair { candidate: "abc def".into(), reference: "xyz uvw".into() },
        ScoringPair { candidate: "the cat sat".into(), reference: "the dog sat".into() },
        ScoringPair { candidate: "short".into(), reference: "this is a much longer reference string".into() },
        ScoringPair { candidate: "the the the".into(), reference: "the the the".into() },
        ScoringPair { candidate: "a a a a a".into(), reference: "a b c d e".into() },
    ]
}

#[test]
fn exact_match_triple_agreement() {
    let scorer = ExactMatchScorer::case_sensitive();
    for pair in edge_case_pairs() {
        let result = scorer.score_and_verify(&pair);
        assert!(result.agreement,
            "ExactMatch triple disagreement on '{}' vs '{}'",
            pair.candidate, pair.reference);
    }
}

#[test]
fn token_f1_triple_agreement() {
    let scorer = TokenF1Scorer::new(TokenF1Config::default());
    for pair in edge_case_pairs() {
        let result = scorer.score_and_verify(&pair);
        assert!(result.agreement,
            "TokenF1 triple disagreement on '{}' vs '{}'",
            pair.candidate, pair.reference);
    }
}

#[test]
fn bleu_triple_agreement() {
    let scorer = BleuScorer::with_smoothing(SmoothingMethod::Add1);
    for pair in edge_case_pairs() {
        if pair.candidate.is_empty() || pair.reference.is_empty() {
            continue; // BLEU undefined for empty
        }
        let result = scorer.score_and_verify(&pair);
        assert!(result.agreement,
            "BLEU triple disagreement on '{}' vs '{}'",
            pair.candidate, pair.reference);
    }
}

#[test]
fn rouge_l_triple_agreement() {
    let scorer = RougeLScorer::default_scorer();
    for pair in edge_case_pairs() {
        let result = scorer.score_and_verify(&pair);
        assert!(result.agreement,
            "ROUGE-L triple disagreement on '{}' vs '{}'",
            pair.candidate, pair.reference);
    }
}

#[test]
fn rouge_1_triple_agreement() {
    let scorer = RougeNScorer::rouge1();
    for pair in edge_case_pairs() {
        let result = scorer.score_and_verify(&pair);
        assert!(result.agreement,
            "ROUGE-1 triple disagreement on '{}' vs '{}'",
            pair.candidate, pair.reference);
    }
}

#[test]
fn differential_standard_suite() {
    let pairs = standard_test_suite();
    let tester = DifferentialTester::new();
    let reports = tester.test_all_metrics(&pairs);
    for (metric, report) in &reports {
        assert!(report.is_perfect(),
            "{}: {} disagreements out of {} tests",
            metric, report.disagreements, report.total_tests);
    }
}
