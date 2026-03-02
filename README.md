# Spectacles

**Machine-checkable zero-knowledge proofs that NLP benchmark scores were computed correctly.**

Spectacles compiles NLP scoring metrics (BLEU, ROUGE, Token F1, Exact Match, chrF)
into weighted finite automata (WFA) over typed semirings, then generates
STARK zero-knowledge proofs certifying score correctness. A companion Lean 4
formalization proves the underlying algebraic pipeline is sound.

---

## Table of Contents

1. [Quickstart](#quickstart)
2. [How It Works](#how-it-works)
3. [Supported Metrics](#supported-metrics)
4. [CLI Reference](#cli-reference)
   - [score](#score)
   - [batch-score](#batch-score)
   - [differential-test](#differential-test)
   - [compile-circuit](#compile-circuit)
   - [estimate-size](#estimate-size)
   - [hash](#hash)
   - [verify-proof](#verify-proof)
5. [Input & Output Formats](#input--output-formats)
6. [Batch Processing](#batch-processing)
7. [Architecture Overview](#architecture-overview)
8. [Weighted Finite Automata & Semirings](#weighted-finite-automata--semirings)
9. [STARK Proof System](#stark-proof-system)
10. [Lean 4 Formalization](#lean-4-formalization)
11. [What's Proved](#whats-proved)
12. [Key Numbers](#key-numbers)
13. [Limitations](#limitations)
14. [Further Documentation](#further-documentation)
15. [License](#license)

---

## Quickstart

```bash
# Build (release mode recommended for proof generation)
cargo build --release

# Run the fast semiring unit tests (191 tests, < 1 s)
cargo test -p spectacles-core --lib semiring

# Score a single pair
cargo run --release --bin spectacles-cli -- \
  score --metric bleu \
  --candidate "the cat sat" \
  --reference "the cat sat on the mat"

# Batch-score a CSV file with multiple metrics at once
cargo run --release --bin spectacles-cli -- \
  batch-score --input pairs.csv \
  --metrics exact_match,bleu,rouge_1 \
  --output-format markdown

# Run differential tests (reference ↔ WFA ↔ circuit agreement)
cargo run --release --bin spectacles-cli -- \
  differential-test --count 200

# Lean 4 formalization (requires Lean 4.8.0)
cd lean && lake build
# → Build completed successfully (13 sorry warnings, 0 errors)

# Lean–Rust conformance bridge
cargo run --release --bin lean_rust_conformance
# → 8/8 checks passed (100.0%), 0 disagreements
```

---

## How It Works

Spectacles operates through a three-stage pipeline:

1. **Metric → WFA compilation.**  Each NLP scoring function is decomposed into
   a weighted finite automaton over an appropriate semiring. For example, Exact
   Match uses the Boolean semiring (∨, ∧), BLEU uses the Counting semiring
   (ℕ, +, ×) with bounded clipping, and ROUGE-L uses the Tropical semiring
   (min, +) for longest-common-subsequence via shortest paths.

2. **WFA → STARK circuit compilation.**  The WFA's transition table and
   semiring operations are compiled to an Algebraic Intermediate Representation
   (AIR) over the Goldilocks field (F_p, p = 2⁶⁴ − 2³² + 1).  Two compilation
   tiers handle the algebra:
   - **Tier 1** — direct semiring homomorphism φ: S → F_p (Boolean, Counting).
   - **Tier 2** — comparison gadgets with 62-bit decomposition (Tropical).

3. **STARK proof generation & verification.**  A FRI-based STARK prover
   generates a succinct proof that the circuit accepts; a verifier checks the
   proof in ~1.5 ms without re-running the scoring computation.  Each proof
   includes a BLAKE3 Merkle commitment to the WFA transition table,
   cryptographically binding the result to the specific metric definition.

Every metric has **three independent implementations** (reference, automaton,
circuit) that are cross-validated by the differential testing framework.

---

## Supported Metrics

| Metric | CLI Name(s) | Semiring | Compilation Tier | Description |
|--------|------------|----------|-----------------|-------------|
| **Exact Match** | `exact_match`, `exact-match`, `em` | Boolean | Tier 1 | Binary string equality after normalization |
| **Token F1** | `token_f1`, `token-f1`, `f1` | Counting | Tier 1 + F-measure gadget | Token-level precision/recall/F1 |
| **BLEU** | `bleu` | Counting | Tier 1 + geo-mean + BP gadgets | N-gram precision with brevity penalty (1–4 grams) |
| **ROUGE-1** | `rouge1`, `rouge-1`, `rouge_1` | Counting | Tier 1 + F-measure gadget | Unigram overlap (P/R/F1) |
| **ROUGE-2** | `rouge2`, `rouge-2`, `rouge_2` | Counting | Tier 1 + F-measure gadget | Bigram overlap (P/R/F1) |
| **ROUGE-L** | `rougel`, `rouge-l`, `rouge_l` | Tropical (Max-Plus) | Tier 2 | Longest common subsequence (P/R/F1) |
| **chrF** | (library only) | Counting | Tier 1 + β-F-measure gadget | Character n-gram F-score |

The WFA coverage census identifies 21 of 31 common NLP metrics as
WFA-representable. Neural metrics (BERTScore, COMET, BLEURT) are inherently
non-WFA and out of scope.

---

## CLI Reference

The binary is `spectacles-cli`.  Every command is invoked as:

```
spectacles-cli <COMMAND> [OPTIONS]
```

### `score`

Score a single candidate/reference pair.

```bash
spectacles-cli score [OPTIONS]
```

| Flag | Short | Description |
|------|-------|-------------|
| `--metric` | `-m` | Metric name (default: `bleu`). See table above. |
| `--candidate` | `-c` | Candidate (hypothesis) text. **Required.** |
| `--reference` | `-r` | Reference (gold) text. **Required.** |
| `--triple` | `-t` | Run all three implementations and report agreement. |
| `--output-format` | | Output format: `text` (default), `json`, `csv`, `markdown`. |

**Examples:**

```bash
# Basic BLEU score (text output)
spectacles-cli score -m bleu -c "the cat sat" -r "the cat sat on the mat"

# Token F1 with JSON output
spectacles-cli score -m token_f1 -c "hello world" -r "hello there" --output-format json

# Triple-implementation agreement check
spectacles-cli score -m exact_match -c "yes" -r "Yes" --triple
```

### `batch-score`

Score multiple pairs from a file. Supports JSON, JSONL, and CSV inputs.

```bash
spectacles-cli batch-score [OPTIONS]
```

| Flag | Short | Description |
|------|-------|-------------|
| `--input` | `-i` | Input file path. **Required.** Format auto-detected by extension. |
| `--metric` | `-m` | Single metric name. |
| `--metrics` / `--metric-list` | | Comma-separated list of metrics (e.g. `exact_match,bleu,rouge_1`). |
| `--output-format` | | Output format: `text` (default), `json`, `csv`, `markdown`. |

Either `--metric` or `--metrics` is required. Using `--metrics` runs all
requested metrics in a single invocation instead of requiring separate runs.

**Examples:**

```bash
# CSV input, multiple metrics, markdown table output
spectacles-cli batch-score -i data/pairs.csv --metrics exact_match,bleu,rouge_1 --output-format markdown

# JSONL input, single metric
spectacles-cli batch-score -i data/pairs.jsonl -m token_f1

# JSON array input, CSV output
spectacles-cli batch-score -i data/pairs.json --metrics bleu --output-format csv
```

### `differential-test`

Run differential tests: verify that reference, WFA, and circuit
implementations agree on random and structured inputs.

```bash
spectacles-cli differential-test [OPTIONS]
```

| Flag | Short | Description |
|------|-------|-------------|
| `--count` | `-n` | Number of random test pairs (default: 50). |
| `--seed` | `-s` | RNG seed for reproducibility (default: 42). |

**Example:**

```bash
spectacles-cli differential-test --count 500 --seed 12345
```

### `compile-circuit`

Compile a scoring metric to an arithmetic circuit and report size estimates.

```bash
spectacles-cli compile-circuit [OPTIONS]
```

| Flag | Short | Description |
|------|-------|-------------|
| `--metric` | `-m` | Metric to compile (default: `exact_match`). |
| `--max-length` | `-l` | Maximum input length in tokens (default: 100). |

### `estimate-size`

Estimate STARK proof size for a given circuit configuration.

```bash
spectacles-cli estimate-size [OPTIONS]
```

| Flag | Description |
|------|-------------|
| `--constraints` | Number of AIR constraints (default: 1000). |
| `--wires` | Number of wire columns (default: 500). |
| `--security` | Security level in bits (default: 128). |

### `hash`

Hash input data with BLAKE3 (optionally domain-separated).

```bash
spectacles-cli hash [OPTIONS] [DATA]
```

| Flag | Short | Description |
|------|-------|-------------|
| `--domain` | `-d` | Domain separation tag. |
| `--data` | | Data to hash (also accepted as positional argument). |

**Example:**

```bash
spectacles-cli hash --domain "spectacles.score" "hello world"
```

### `verify-proof`

Verify a previously generated proof file by checking its header, checksum,
or commitment.

```bash
spectacles-cli verify-proof [OPTIONS]
```

| Flag | Short | Description |
|------|-------|-------------|
| `--proof` | `-p` | Path to proof file (`.spct` binary or `.json`). Also accepted positionally. |

For JSON proof files, the command parses the `CompactProof` structure, prints
metric/score/hash details, and re-computes the BLAKE3 commitment to verify
integrity. For binary `.spct` files, it validates the header magic bytes,
format version, and payload checksum.

**Example:**

```bash
spectacles-cli verify-proof --proof results/bleu_proof.json
spectacles-cli verify-proof results/score.spct
```

---

## Input & Output Formats

### Input Formats (batch-score)

The `batch-score` command auto-detects format by file extension:

| Extension | Format | Description |
|-----------|--------|-------------|
| `.json` | JSON array | Array of objects: `[{"candidate": "...", "reference": "..."}, ...]` |
| `.jsonl` | JSON Lines | One JSON object per line: `{"candidate": "...", "reference": "..."}` |
| `.csv` | CSV | Header row with `reference` and `hypothesis` (or `candidate`) columns |

**CSV example:**

```csv
reference,hypothesis
the cat sat on the mat,the cat sat
hello world,hello there
```

Column names are case-insensitive. Accepted column names:
- Reference: `reference`, `ref`
- Hypothesis: `hypothesis`, `candidate`, `hyp`

**JSONL example:**

```jsonl
{"candidate": "the cat sat", "reference": "the cat sat on the mat"}
{"candidate": "hello there", "reference": "hello world"}
```

### Output Formats

All scoring commands support `--output-format` with four options:

| Format | Flag value | Description |
|--------|-----------|-------------|
| **Text** | `text` (default) | Human-readable, one result per line |
| **JSON** | `json` | JSON array of score objects |
| **CSV** | `csv` | Header + data rows: `metric,index,score,precision,recall,brevity_penalty` |
| **Markdown** | `markdown`, `md` | Pipe-delimited table suitable for documentation |

---

## Batch Processing

Batch scoring is designed for evaluating entire datasets efficiently:

```bash
# Score 1000 pairs with three metrics in one run
spectacles-cli batch-score \
  -i corpus.csv \
  --metrics exact_match,bleu,rouge_1 \
  --output-format json > results.json
```

The output contains one row per (pair, metric) combination. With `--metrics
exact_match,bleu,rouge_1` and 100 input pairs, you get 300 result rows.

For corpus-level aggregation (e.g., corpus BLEU), use the library API directly
via `BleuScorer::reference_score_corpus()`.

---

## Architecture Overview

### Rust Crates

```
crates/
├── spectacles-core/          # Core library
│   └── src/
│       ├── lib.rs
│       ├── pipeline.rs       # End-to-end metric certification with STARK proofs
│       ├── scoring/          # 7 metrics × 3 implementations + differential testing
│       │   ├── exact_match.rs
│       │   ├── token_f1.rs
│       │   ├── bleu.rs
│       │   ├── rouge.rs
│       │   ├── chrf.rs
│       │   ├── regex_match.rs
│       │   ├── pass_at_k.rs
│       │   ├── differential.rs
│       │   └── tokenizer.rs
│       ├── wfa/              # Semirings, WFA operations, minimization, equivalence
│       │   └── semiring.rs   # 10+ semiring types, EmbeddableSemiring trait
│       ├── circuit/          # Goldilocks field, AIR constraints, STARK prover, FRI
│       ├── evalspec/         # EvalSpec DSL: parser, type checker, compiler
│       ├── protocol/         # Certificates, transcripts, model identity attestation
│       ├── psi/              # OPRF-based PSI + multi-layer contamination detection
│       └── utils/            # BLAKE3 hashing, Merkle trees, serialization, math
├── spectacles-cli/           # Command-line interface (this README documents it)
├── spectacles-integration/   # Integration test harness
└── spectacles-examples/      # Benchmark & experiment binaries
    └── src/bin/
        ├── comprehensive_benchmark.rs
        ├── compilation_correctness.rs
        ├── stark_scaling_2048.rs
        ├── e2e_certification.rs
        └── lean_rust_conformance.rs
```

### Lean 4 Formalization

```
lean/Spectacles/
├── Semiring/             # Algebraic foundations
│   ├── Basic.lean            # Semiring typeclass definition
│   ├── Instances.lean        # Bool, Nat, Tropical instances
│   ├── Goldilocks.lean       # F_p field (p = 2⁶⁴ − 2³² + 1) + Semiring instance
│   ├── Embedding.lean        # Injective homomorphisms Bool→F_p, Nat→F_p
│   ├── KleeneSemiring.lean   # Kleene algebra extension
│   └── Embeddable.lean       # EmbeddableSemiring typeclass + non-embeddability proof
├── WFA/                  # Weighted finite automata theory
├── Circuit/              # ZK circuit compilation
│   ├── TraceLayout.lean      # Column allocation for AIR trace
│   ├── Compiler.lean         # AIR constraints + Tier 1/2 soundness
│   └── Gadgets.lean          # Comparison gadgets (non-interference, soundness)
└── EvalSpec/             # Metric specification DSL
    ├── Syntax.lean           # AST + types
    ├── Typing.lean           # HasType judgment
    ├── Semantics.lean        # Big-step evaluation semantics
    └── Compiler.lean         # Type preservation proof
```

---

## Weighted Finite Automata & Semirings

A **weighted finite automaton** (WFA) generalizes classical finite automata by
assigning weights from a semiring to transitions and states. Given a semiring
(S, ⊕, ⊗, 0̄, 1̄):

- **Initial weights** λ: Q → S assign a weight to each starting state.
- **Transition weights** δ: Q × Σ × Q → S assign weights to transitions.
- **Final weights** ρ: Q → S assign weights to accepting states.
- The **weight of a path** is the ⊗-product of its transition weights.
- The **total weight of a string** is the ⊕-sum over all accepting paths.

Spectacles implements the following semirings:

| Semiring | (⊕, ⊗, 0̄, 1̄) | Use |
|----------|----------------|-----|
| **Boolean** | (∨, ∧, false, true) | Exact Match: accepts/rejects |
| **Counting** (ℕ) | (+, ×, 0, 1) | Token F1, BLEU, ROUGE-N: count overlaps |
| **Bounded Counting** | (min(+, B), ×, 0, 1) | BLEU clipped n-gram counts |
| **Tropical** (min-plus) | (min, +, ∞, 0) | ROUGE-L: shortest-path ↔ LCS |
| **Max-Plus** | (max, +, −∞, 0) | ROUGE-L alternative: longest path |
| **Goldilocks F_p** | (+mod p, ×mod p, 0, 1) | Circuit target field |

The `EmbeddableSemiring` trait (proved in Lean) identifies which semirings
admit an injective homomorphism into the Goldilocks field — this determines
whether a metric uses Tier 1 (algebraic) or Tier 2 (gadget-based) compilation.

---

## STARK Proof System

Spectacles uses a **FRI-based STARK** (Scalable Transparent ARgument of
Knowledge) for proof generation:

1. **Trace generation.**  The prover evaluates the WFA on the input, recording
   a trace of state weights at each step. This trace forms the "execution
   trace" of the AIR program.

2. **Constraint enforcement.**  AIR (Algebraic Intermediate Representation)
   constraints encode the WFA transition rules and semiring laws as polynomial
   equations of degree ≤ 2 over the Goldilocks field.

3. **FRI commitment.**  The prover commits to the trace via a Merkle tree of
   Reed-Solomon evaluations and uses FRI (Fast Reed-Solomon Interactive Oracle
   Proofs of Proximity) to prove low-degree of the constraint composition
   polynomial.

4. **Metric binding.**  Each proof includes a BLAKE3 Merkle commitment to the
   WFA transition table, binding the proof to the specific metric definition.

### Performance

| WFA States | Prove Time | Verify Time | Proof Size |
|-----------|-----------|-------------|------------|
| 4 | 6 ms | 0.4 ms | 27 KiB |
| 64 | 71 ms | 0.9 ms | 107 KiB |
| 256 | 1,191 ms | 1.4 ms | 204 KiB |
| **400** | **3,821 ms** | **1.5 ms** | **270 KiB** |
| **1,024** | **49.9 s** | **2.3 ms** | **490 KiB** |

Scaling model: `prove_time ≈ 0.009 × n^2.18` (R² = 0.989).
Verification stays sub-3 ms at all tested scales.

### Proof Format

Binary proofs use the `.spct` format:

| Field | Size | Description |
|-------|------|-------------|
| Magic | 4 B | `SPCT` |
| Version | 4 B | Major.minor (currently 1.0) |
| Format byte | 1 B | 0=CompactBinary, 1=JSON, 2=CBOR |
| Payload size | 8 B | Little-endian u64 |
| Checksum | 32 B | BLAKE3 hash of payload |
| Payload | var | Serialized proof data |

JSON proofs use the `CompactProof` structure with fields: `metric_id`,
`candidate_hash`, `reference_hash`, `score_numerator`, `score_denominator`,
`commitment`, and `proof_data`.

---

## Lean 4 Formalization

The Lean 4 formalization (in `lean/`) provides machine-checked proofs that
the algebraic pipeline is sound. Build with:

```bash
cd lean && lake build
# → Build completed successfully (13 sorry warnings, 0 errors)
```

### Lean–Rust Conformance

Since Lean proofs cannot be formally extracted to Rust, Spectacles bridges
the gap with 68,748 differential tests and a dedicated conformance checker:

```bash
cargo run --release --bin lean_rust_conformance
# → 8/8 checks passed (100.0%), 0 disagreements
```

The conformance checker verifies that the Rust semiring implementations
produce identical results to the Lean definitions on a structured test suite.

### Axioms

The formalization rests on 3 axioms:

1. **Cryptographic axiom** — BLAKE3 is collision-resistant (standard assumption).
2. **Specification axiom** — The EvalSpec DSL correctly captures metric semantics.
3. **Mathematical axiom** — Certain modular arithmetic lemmas (pending Mathlib).

---

## What's Proved

| Claim | Status | File |
|-------|--------|------|
| Semiring axioms (Bool, Nat, Tropical, Goldilocks) | ✅ Proved | `Semiring/*.lean` |
| Injective embeddings Bool→F_p, Nat→F_p | ✅ Proved | `Semiring/Embedding.lean` |
| EmbeddableSemiring typeclass + instances | ✅ Mostly proved | `Semiring/Embeddable.lean` |
| Tropical non-embeddability (why Tier 2 needed) | 📝 Stated | `Semiring/Embeddable.lean` |
| Tier 1/2 soundness (modulo STARK axiom) | ✅ Proved | `Circuit/Compiler.lean` |
| Constraint degree ≤ 2 | ✅ Proved | `Circuit/Compiler.lean` |
| Gadget non-interference | ✅ Proved | `Circuit/Gadgets.lean` |
| Comparison gadget soundness (circuit-level) | ✅ Proved | `Circuit/Gadgets.lean` |
| EvalSpec DSL: syntax, types, semantics | ✅ Formalized | `EvalSpec/*.lean` |
| End-to-end: STARK → trace → WFA execution | ✅ Proved | `Circuit/Compiler.lean` |

**Previously vacuous, now genuine:**
- `constraint_satisfied` was `True` → now evaluates `SymExpr` on trace rows
- `h_stark_accepts` was `True` → now `STARKVerifierResult.accepted = true`
- Comparison gadget was tautological → now circuit-level `ComparisonGadgetWitness`

---

## Key Numbers

| Measurement | Result |
|-------------|--------|
| Differential tests | 68,748 triple-agreement, **0 disagreements** |
| Rust semiring unit tests | **191 passing** |
| STARK verify time (400 states) | **1.5 ms** |
| STARK proof size (400 states) | **270 KiB** |
| E2E STARK certification proofs | **76 proofs** verified (4–1,024 states) |
| WFA metric coverage | **21/31** NLP metrics (67.7%) |
| Lean sorrys | **13** remaining (down from 15+) |
| Lean axioms | **3** (1 cryptographic, 1 specification, 1 mathematical) |
| Multi-layer contamination | **100% accuracy** (vs 83.3% n-gram-only) |
| Bugs found by verification | **2** (Montgomery reduction, Lagrange interpolation) |

---

## Limitations

- **Lean-to-Rust gap**: Bridged by 68,748 differential tests, not formal
  extraction. No verified code generation from Lean to Rust.
- **13 remaining sorrys**: 5 engineering, 5 mathematical (need Mathlib),
  3 modular arithmetic. All have proof sketches.
- **Metric scope**: String-matching metrics only. Neural metrics (BERTScore,
  COMET, BLEURT) are inherently non-WFA and out of scope.
- **STARK scaling**: Demonstrated up to 1,024 WFA states. Proving time grows
  as O(n²) in state count; 2,048+ states are extrapolated only.
- **Model identity**: TEE-based attestation is protocol-level design;
  production deployment requires hardware support.

---

## Further Documentation

| Document | Location | Description |
|----------|----------|-------------|
| Tool paper | `docs/paper/tool_paper.pdf` | Full results-oriented paper (41 pages) |
| Architecture guide | `docs/ARCHITECTURE.md` | Build, test, and project structure |
| API reference | `docs/api.md` | Detailed Rust API documentation |
| Grounding | `docs/meta/grounding.json` | Every paper claim mapped to evidence |
| Critique triage | `docs/meta/critique_triage.json` | Category B assessment |
| Benchmark data | `data/*.json` | Raw experimental results |

---

## License

Research use only.
