# Spectacles

**Score-verified NLP evaluation certificates via weighted finite automata compiled to STARK zero-knowledge proof circuits.**

Spectacles proves that benchmark scores were computed correctly — machine-checkable proofs of evaluation integrity with metric-specific STARK binding, multi-layer contamination detection, and model identity attestation.

## 30-Second Quickstart

```bash
cargo build --release

# Certify a score with a STARK proof (< 1 second)
cargo run --release --bin comprehensive_benchmark
# → 205 pairs × 6 metrics = 1,230 triple checks, 0 disagreements
# → 25 metric-bound STARK proofs generated and verified
# → Each proof includes BLAKE3 commitment to WFA transition table

# Full compilation correctness suite
cargo run --release --bin compilation_correctness
# → 68,748 tests, 0 disagreements

# STARK scaling to 1,024 states
cargo run --release --bin stark_scaling_2048
# → 400-state BLEU-4 target: 3.8s prove, 1.5ms verify
# → 1,024 states: 49.9s prove, 2.3ms verify
```

## Key Results

| Metric | Value | Source |
|--------|-------|--------|
| Compilation checks | **68,748 tests, 0 disagreements** | `compilation_correctness.json` |
| Comprehensive benchmark | **1,230 checks across 6 metrics, 0 disagreements** | `comprehensive_benchmark_results.json` |
| E2E certification proofs | **49 metric-bound STARK proofs** across 5 metrics | `comprehensive_benchmark_results.json` |
| STARK proofs verified | **76 proofs** (4–1,024 states) | `stark_scaling_regression.json` |
| 400-state BLEU-4 target | **ACHIEVED**: 3.8s prove, 1.5ms verify | measured |
| Max proven states | **1,024** (49.9s prove, 2.3ms verify) | measured |
| WFA metric coverage | **21/31 metrics (67.7%) WFA-representable** | census table in paper |
| Multi-layer contamination | **100% accuracy** (vs 83.3% n-gram-only) | `contamination_multi_layer_v2.json` |
| Bugs found by verification | **2** (Montgomery reduction, Lagrange interpolation) | paper §7 |

## How It Works

NLP scoring functions decompose into **weighted finite automata over typed semirings**:

| Metric | Semiring | Compilation Tier |
|--------|----------|-----------------|
| Exact Match | Boolean | Tier 1 (algebraic) |
| Token F1 | Counting | Tier 1 + F-measure gadget |
| BLEU-4 | Counting | Tier 1 + geo mean + BP gadgets |
| ROUGE-1/2 | Counting | Tier 1 + F-measure gadget |
| chrF | Counting | Tier 1 + β-F-measure gadget |
| ROUGE-L | Tropical | Tier 2 (comparison gadgets) |

**Two-tier compilation** to STARK circuits:
- **Tier 1**: Direct semiring homomorphism φ: S → F_p (Goldilocks field, p = 2^64 − 2^32 + 1)
- **Tier 2**: Comparison gadgets with 62-bit decomposition for tropical min-plus

**Metric-specific STARK binding**: Each proof encodes the actual WFA transition constraints and includes a BLAKE3 Merkle commitment to the transition table, cryptographically binding the proof to the specific metric definition.

## Benchmark Results (Per-Category)

| Category | Pairs | EM | Tok F1 | BLEU | R-1 | R-L | chrF |
|----------|-------|----|--------|------|-----|-----|------|
| MMLU-style | 50 | 0.58 | 0.61 | 0.92 | 0.61 | 0.61 | 0.42 |
| SQuAD-style | 50 | 0.02 | 0.49 | 0.21 | 0.49 | 0.48 | 0.42 |
| Translation | 40 | 0.00 | 0.55 | 0.30 | 0.55 | 0.53 | 0.44 |
| Code completion | 40 | 0.28 | 0.73 | 0.60 | 0.73 | 0.73 | 0.77 |
| Edge cases | 25 | 0.36 | 0.60 | 0.74 | 0.60 | 0.60 | 0.51 |

All 1,230 triple checks (205 pairs × 6 metrics) show 0 disagreements between reference, WFA, and circuit implementations.

## STARK Scaling

| States | Prove Time | Verify Time | Proof Size |
|--------|-----------|-------------|------------|
| 64 | 71 ms | 0.9 ms | 107 KiB |
| 256 | 1,191 ms | 1.4 ms | 204 KiB |
| **400** | **3,821 ms** | **1.5 ms** | **270 KiB** |
| **1,024** | **49,883 ms** | **2.3 ms** | **490 KiB** |

Scaling model: prove_time = 0.009 × n^2.18 (R² = 0.989). Verification stays sub-3ms at all scales.

## Project Structure

```
spectacles-core/src/
  pipeline.rs  — End-to-end metric certification with STARK proofs
  evalspec/    — EvalSpec DSL: parser, type checker, compiler, semantics
  wfa/         — Semirings, WFA operations, minimization, equivalence
  circuit/     — Goldilocks field, AIR constraints, STARK prover, FRI
  scoring/     — 7 metrics × 3 implementations, differential testing
  psi/         — OPRF-based PSI + multi-layer contamination detection
  protocol/    — Certificates, transcripts, model identity attestation
spectacles-examples/src/bin/
  comprehensive_benchmark  — 205 pairs × 6 metrics + 25 STARK certifications
  compilation_correctness  — 68,748 triple-agreement checks
  stark_scaling_2048       — Scaling benchmark to 1,024 states
  e2e_certification        — End-to-end STARK certification
```

## Limitations

- **No verified extraction**: Lean–Rust gap bridged by 68,748 empirical tests, not formal proof
- **STARK scaling**: Demonstrated up to 1,024 states; 2,048+ extrapolated
- **3 novel Lean sorrys remaining**: N1 (Hopcroft), N2 (trace layout), N3 (degree bounds) — all with proof sketches
- **Metric coverage**: String-matching metrics only; neural metrics (BERTScore, COMET) are NonWFA
- **Model identity**: TEE attestation is protocol-level design; production deployment requires hardware

## Documentation

- `tool_paper.pdf` — Full paper (41 pages)
- `api.md` — API reference
- `grounding.json` — Every paper claim mapped to evidence
- `reproduce.sh` — Reproduce all experiments

## License

Research use only.
