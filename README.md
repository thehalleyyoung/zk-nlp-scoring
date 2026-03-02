# Spectacles

**Machine-checkable proofs that NLP benchmark scores were computed correctly.**

Spectacles compiles NLP metrics to weighted finite automata (WFA) over typed semirings, then generates STARK zero-knowledge proofs certifying score correctness against a formal Lean 4 specification.

## 30-Second Quickstart

```bash
# Build and run quick tests
cargo build --release
cargo test -p spectacles-core --lib semiring    # 191 tests, <1s

# Lean 4 formalization (requires Lean 4.8.0)
cd lean && lake build
# → Build completed successfully (13 sorry warnings, 0 errors)

# Lean–Rust conformance
cargo run --release --bin lean_rust_conformance
# → 8/8 checks passed (100.0%), 0 disagreements
```

## What's Proved

| Claim | Status | File |
|-------|--------|------|
| Semiring axioms (Bool, Nat, Tropical, Goldilocks) | ✅ Proved | `Semiring/*.lean` |
| Injective embeddings Bool→F_p, Nat→F_p | ✅ Proved | `Semiring/Embedding.lean` |
| EmbeddableSemiring typeclass + instances | ✅ Mostly proved | `Semiring/Embeddable.lean` |
| Tropical non-embeddability (why Tier 2 needed) | 📝 Stated | `Semiring/Embeddable.lean` |
| Tier 1/2 soundness (modulo STARK axiom) | ✅ Proved | `Circuit/Compiler.lean` |
| Constraint degree ≤ 2 (N3) | ✅ Proved | `Circuit/Compiler.lean` |
| Gadget non-interference (N4) | ✅ Proved | `Circuit/Gadgets.lean` |
| Comparison gadget soundness (circuit-level) | ✅ Proved | `Circuit/Gadgets.lean` |
| EvalSpec DSL: syntax, types, semantics | ✅ Formalized | `EvalSpec/*.lean` |
| End-to-end: STARK → trace → WFA execution | ✅ Proved | `Circuit/Compiler.lean` |

**Previously vacuous, now genuine:**
- `constraint_satisfied` was `True` → now evaluates `SymExpr` on trace rows
- `h_stark_accepts` was `True` → now `STARKVerifierResult.accepted = true`
- Comparison gadget was tautological → now circuit-level `ComparisonGadgetWitness`

## Key Numbers

| Measurement | Result |
|-------------|--------|
| Differential tests | 68,748 triple-agreement, **0 disagreements** |
| Rust semiring tests | **191 passing** |
| STARK verify time | **1.5 ms** |
| STARK proof size | **270 KiB** |
| WFA metric coverage | **21/31** NLP metrics (67.7%) |
| Lean sorrys | **13** remaining (down from 15+) |
| Lean axioms | **3** (1 cryptographic, 1 specification, 1 mathematical) |

## Architecture

```
lean/Spectacles/
├── Semiring/         # Algebraic foundations (5 files)
│   ├── Basic.lean        # Semiring typeclass
│   ├── Instances.lean    # Bool, Nat, Tropical
│   ├── Goldilocks.lean   # F_p field + Semiring instance
│   ├── Embedding.lean    # Injective homomorphisms
│   ├── KleeneSemiring.lean
│   └── Embeddable.lean   # NEW: EmbeddableSemiring typeclass
├── WFA/              # Weighted automata (3 files)
├── Circuit/          # ZK compilation (3 files)
│   ├── TraceLayout.lean  # Column allocation
│   ├── Compiler.lean     # AIR constraints + soundness
│   └── Gadgets.lean      # Comparison gadgets
└── EvalSpec/         # NEW: DSL formalization (4 files)
    ├── Syntax.lean       # AST + types
    ├── Typing.lean       # HasType judgment
    ├── Semantics.lean    # Big-step evaluation
    └── Compiler.lean     # Type preservation
```

```
crates/spectacles-core/src/
├── wfa/semiring.rs   # Semiring trait + EmbeddableSemiring
├── circuit/          # STARK prover/verifier
├── evalspec/         # EvalSpec DSL implementation
└── scoring/          # NLP metric implementations
```

## Limitations

- **Lean-to-Rust gap**: Bridged by 68,748 differential tests, not formal extraction
- **13 remaining sorrys**: 5 engineering, 5 math (need Mathlib), 3 modular arithmetic
- **Metric scope**: String-matching metrics only (no BERTScore, COMET, BLEURT)
- **STARK scaling**: O(n²) in WFA state count; batch verification needed for corpora

## Documentation

- [Paper](docs/paper/tool_paper.pdf) — Results-oriented conference paper
- [Architecture](docs/ARCHITECTURE.md) — Build, test, architecture guide
- [Grounding](docs/meta/grounding.json) — What is proved vs assumed
- [Critique Triage](docs/meta/critique_triage.json) — Category B assessment
