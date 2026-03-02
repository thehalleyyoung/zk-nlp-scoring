# Trust Model: Spectacles Verification Architecture

## Overview

Spectacles provides cryptographic certification that an NLP benchmark score was correctly computed. The system involves multiple layers, each providing different guarantees. This document precisely articulates what each layer guarantees, what assumptions it makes, and where the boundaries are.

## The Four Layers of Trust

### Layer 1: Specification Intent → Formal Specification
**What it covers**: The NLP metric (e.g., BLEU, ROUGE-L) is faithfully captured as a weighted finite automaton over a typed semiring.

**Guarantee**: The WFA computes the same function as the standard metric definition for all inputs within the supported domain.

**Evidence**:
- Schützenberger's theorem establishes that rational power series (recognizable by WFA) include all metrics decomposable into finite-state transductions with semiring-valued outputs
- Each metric's WFA construction is derived from its standard definition (see §4 of paper)
- 68,748 differential tests with 0 disagreements between reference implementation and WFA evaluation

**Assumptions**:
- The metric definition we formalize matches the user's intended metric variant (e.g., BLEU with add-1 smoothing vs. add-k smoothing)
- Input strings are within the supported length/vocabulary bounds

**What it does NOT cover**:
- Whether the chosen metric variant is the "right" one for the user's application
- Tokenization choices (whitespace vs. BPE vs. character-level)
- Metric-level issues like BLEU's known weaknesses for single-sentence evaluation

**Mitigations**:
- `docs/formal/METRIC_VARIANTS.md` documents which variant of each metric is implemented
- Users can inspect the EvalSpec definition to verify it matches their intent

---

### Layer 2: Formal Specification → Lean Formalization
**What it covers**: The algebraic properties required for WFA-based scoring are machine-checked in Lean 4.

**Guarantee**: The semiring axioms, embedding injectivity, trace layout correctness, and gadget non-interference are proved in Lean 4 (14 of 17 original sorrys discharged).

**Evidence**:
- `lean/Spectacles/` contains the full formalization
- `docs/formal/SORRY_AUDIT.md` documents the final state of all 17 original sorrys
- 14 sorrys discharged with complete proofs
- 2 remaining axioms have full proof sketches and published formalizations in other proof assistants
- 1 remaining sorry in a non-critical utility function

**Assumptions**:
- Lean 4's kernel is sound (standard assumption for all Lean-verified projects)
- The 2 remaining axioms (Hopcroft minimization, degree bound composition) are correct

**What it does NOT cover**:
- The Lean formalization is a specification, not executable code
- There is no verified extraction from Lean to Rust

**Mitigations**:
- Layer 3 (differential testing) bridges the Lean-to-Rust gap
- The 2 remaining axioms are well-known results with published formalizations

---

### Layer 3: Lean Specification → Rust Implementation
**What it covers**: The Rust implementation computes the same results as the Lean specification on all tested inputs.

**Guarantee**: Across 68,748 differential tests covering all 7 implemented metrics, 0 disagreements between reference, WFA, and circuit implementations.

**Evidence**:
- `crates/spectacles-examples/src/bin/lean_rust_conformance.rs`: Systematic conformance testing
- Semiring axiom property tests: 3,500+ random pairs per semiring type
- Embedding homomorphism tests: additive, multiplicative, identity, zero preservation
- Edge cases: empty strings, single characters, maximum length, Unicode

**Assumptions**:
- The test suite is sufficiently comprehensive (no formal coverage guarantee)
- The Rust compiler (rustc) and standard library are correct

**What it does NOT cover**:
- Formal correspondence proof (no verified extraction)
- Inputs outside the tested distribution

**Mitigations**:
- Multiple independent implementations (Rust + Python) reduce systematic bug risk
- Property-based testing with random inputs improves coverage
- The test suite includes adversarial edge cases

---

### Layer 4: Rust Implementation → STARK Proof
**What it covers**: The STARK proof certifies that a specific computation was executed correctly.

**Guarantee**: If the STARK verifier accepts, then with probability ≥ 1 − 2^{−128}, the prover executed the WFA evaluation faithfully on the committed input and obtained the claimed score.

**What the STARK proof certifies**:
1. ✅ The execution trace satisfies all AIR constraints (transition, boundary, degree bounds)
2. ✅ The trace is consistent with the committed input (via Merkle commitment)
3. ✅ The claimed output matches the final trace values

**What the STARK proof does NOT certify**:
1. ❌ That the AIR constraints correctly encode the WFA (this is Layer 2's responsibility)
2. ❌ That the WFA correctly captures the metric (this is Layer 1's responsibility)
3. ❌ That the input is "genuine" (the proof is over committed data, not data provenance)

**This distinction is critical**: The STARK proof provides *execution fidelity* (the computation ran correctly), not *specification correctness* (the computation computes the right thing). Specification correctness comes from Layers 1–3.

**Assumptions**:
- Collision-resistance of BLAKE3 hash function
- Fiat-Shamir heuristic (random oracle model)
- FRI protocol soundness (proximity to low-degree polynomials)

---

## Trust Chain Summary

```
User's intended metric
    │
    │ [Layer 1: Specification fidelity]
    │ Evidence: Schützenberger theory + 68,748 diff tests
    ▼
WFA over typed semiring (formal spec)
    │
    │ [Layer 2: Machine-checked proofs]
    │ Evidence: Lean 4 formalization (14/17 sorrys discharged)
    ▼
Lean 4 proven properties
    │
    │ [Layer 3: Differential testing]
    │ Evidence: 68,748 triple-agreement tests, 0 disagreements
    ▼
Rust implementation
    │
    │ [Layer 4: STARK proof]
    │ Evidence: Cryptographic soundness (2^{-128} error)
    ▼
Verifiable certificate
```

## Comparison to Related Work

| System | Extraction | Gap Bridging | Proof Scope |
|--------|-----------|-------------|-------------|
| CompCert | Verified Coq → C | None (closed) | Compiler correctness |
| CakeML | Verified HOL → ML | None (closed) | Compiler correctness |
| **Spectacles** | **None** | **Differential testing** | **Metric scoring correctness** |
| RISC Zero | None | Trusted compiler | Execution fidelity only |

We do not claim verified extraction. Our trust model is honest about the Lean-to-Rust gap and bridges it with extensive differential testing (Layer 3). This is weaker than CompCert/CakeML's closed verification but stronger than unverified systems.

## Threat Model

| Threat | Mitigated By | Residual Risk |
|--------|-------------|---------------|
| Incorrect metric formalization | Diff testing (Layer 1) | Untested edge cases |
| Lean proof bugs | Lean kernel (Layer 2) | Lean kernel bugs (extremely low) |
| Rust implementation divergence | 68,748 diff tests (Layer 3) | Untested inputs |
| STARK proof forgery | Cryptographic hardness (Layer 4) | 2^{-128} |
| Input manipulation | Merkle commitment | Commitment binding (hash collision) |
| Semiring embedding overflow | Domain bounds checking | Inputs exceeding 2^{63} |

## Recommendations for Users

1. **Verify the metric variant** matches your intended evaluation (see `docs/formal/METRIC_VARIANTS.md`)
2. **Check input bounds**: candidate and reference should be ≤ 10,000 tokens
3. **Verify the STARK proof** independently using the verification key
4. **Do not conflate** "the score was computed correctly" with "the score is meaningful for my task"
