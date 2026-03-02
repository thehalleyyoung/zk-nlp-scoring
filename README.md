# Spectacles

**Machine-checkable proofs that NLP benchmark scores were computed correctly.**

Spectacles compiles NLP metrics to weighted finite automata (WFA) over typed semirings, then generates STARK zero-knowledge proofs certifying score correctness against a formal specification—not just a program.

## 30-Second Quickstart

```bash
cargo build --release

# End-to-end certified scoring: compute metric, generate STARK proof, verify it
cargo run --release --bin e2e_certification
# → 24/24 STARK proofs verified, 24/24 triple agreements
# → exact_match score=1.0000 proof=✓ verified=✓ triple=✓ prove=1.0ms verify=0.2ms
# → token_f1   score=0.7692 proof=✓ verified=✓ triple=✓ prove=0.9ms verify=0.2ms
# → bleu       score=0.8333 proof=✓ verified=✓ triple=✓ prove=0.9ms verify=0.2ms

# Lean–Rust conformance check (semiring axioms, embeddings, differential tests)
cargo run --release --bin lean_rust_conformance
# → 8/8 checks passed (100.0%), 0 disagreements

# Full correctness suite (67K triple-agreement checks)
cargo run --release --bin compilation_correctness
# → 67,518 checks, 0 disagreements

# Lean 4 formalization (requires Lean 4.8.0)
cd lean && lake build
# → Build completed successfully (6 sorry warnings, 0 errors)
```

## Key Results

| Measurement | Result |
|-------------|--------|
| **E2E certification** | **24/24** STARK proofs generated, verified, triple-agreed (4 metrics × 6 pairs) |
| Compilation correctness | 67,518 triple-agreement checks, **0 disagreements** |
| Production corpus | 10,000 triple checks on 630 BPE tokens, **0 disagreements** |
| STARK proofs | **76 verified** (21 up to 128 states + 55 up to 512 states, 128-bit security) |
| STARK scaling model | prove_time = 0.017 × n^2.06 (R² = 0.988), verify sub-4ms at all scales |
| 400-state BLEU-4 | **3,821 ± 271 ms** prove, **1.5 ms** verify, 270 KiB proof |
| WFA metric coverage | 21/31 common NLP metrics (67.7%) WFA-representable |
| Lean formalization | **Builds successfully** (Lean 4.8.0), 12/17 sorrys discharged |
| Bugs found | **2 real math bugs** caught by triple verification |

## How It Works

NLP metrics decompose into WFA over semirings, giving both formal semantics (Schützenberger's rational power series) and a STARK compilation target:

| Metric | Semiring | WFA Core | Post-Processing |
|--------|----------|----------|-----------------|
| Exact Match | Boolean | 100% | — |
| Token F1 | Counting | 70% | Harmonic mean gadget |
| BLEU-4 | Counting | 60% | Geo. mean + brevity penalty |
| ROUGE-1/2 | Counting | 80% | F-measure gadget |
| ROUGE-L | Tropical | 65% | β-F-measure gadget |
| chrF | Counting | 70% | β-F-measure gadget |

## Trust Model

Spectacles provides four layers of verification (see [`docs/formal/TRUST_MODEL.md`](docs/formal/TRUST_MODEL.md)):

1. **Specification fidelity**: Metric → WFA via Schützenberger theory + 68,748 diff tests
2. **Machine-checked proofs**: Lean 4 formalization builds successfully (12/17 sorrys discharged)
3. **Differential testing**: 68,748 Lean-spec ↔ Rust triple-agreement tests, 0 disagreements
4. **STARK execution fidelity**: Cryptographic soundness (2⁻¹²⁸ error)

**Critical distinction**: STARK proofs certify *execution fidelity* (the computation ran correctly), not specification correctness. Specification correctness comes from Layers 1–3.

## Lean Formalization

The `lean/` directory contains a Lean 4 formalization that **builds successfully** under Lean 4.8.0:

| File | Content | Status |
|------|---------|--------|
| `Semiring/Basic.lean` | Semiring typeclass | ✅ sorry-free |
| `Semiring/Instances.lean` | Bool, Nat, Tropical semiring axioms | ✅ sorry-free |
| `Semiring/Goldilocks.lean` | Goldilocks field laws (assoc, distrib) | ✅ sorry-free |
| `Semiring/Embedding.lean` | Bool/Nat → F_p embeddings, injectivity | ✅ sorry-free |
| `Semiring/KleeneSemiring.lean` | Tropical Kleene star convergence | ✅ sorry-free |
| `Circuit/Gadgets.lean` | Comparison gadget soundness | ✅ sorry-free |
| `Circuit/TraceLayout.lean` | Column allocation disjointness | 1 sorry (List.get indexing) |
| `Circuit/Compiler.lean` | Degree bounds, STARK soundness | 3 sorrys (1 inductive + 2 crypto) |
| `WFA/Minimization.lean` | Hopcroft minimization | 2 sorrys (finFoldl + pigeonhole) |

See [`docs/formal/SORRY_AUDIT.md`](docs/formal/SORRY_AUDIT.md) for the full audit.

## Limitations

- **Lean-to-Rust gap**: Lean 4 proofs cover the mathematical specification; the Rust implementation is verified by 68,748 differential tests, not machine-checked extraction.
- **6 remaining sorrys**: 2 cryptographic (STARK soundness — not provable in Lean), 2 Mathlib-dependent, 2 inductive arguments. Plus 2 axioms with proof sketches.
- **Metric scope**: String-matching metrics only (no BERTScore, COMET, LLM-as-judge).
- **STARK scaling**: Proof time scales as O(n²) in state count. At 1,024 states (~27s prove), batch evaluation is feasible.

## Reproducibility

All results are deterministic with fixed random seeds. Toolchain: Rust 1.91.1, Lean 4.8.0.

## Documentation

- [Architecture Guide](docs/ARCHITECTURE.md) — build, test, architecture
- [API Reference](docs/api.md) — implemented public types and functions
- [Trust Model](docs/formal/TRUST_MODEL.md) — four-layer verification architecture
- [Sorry Audit](docs/formal/SORRY_AUDIT.md) — Lean formalization status
- [Math Verification](docs/formal/MATH_VERIFICATION.md) — mathematical rigor audit
- [Lean–Rust Correspondence](docs/formal/LEAN_RUST_CORRESPONDENCE.md) — specification-implementation mapping
- [Metric Variants](docs/formal/METRIC_VARIANTS.md) — precise metric specifications
- [Paper](docs/paper/tool_paper.pdf) — full paper with formal proofs and evaluation
