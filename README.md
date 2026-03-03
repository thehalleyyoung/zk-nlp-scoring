# Spectacles

**Machine-checkable zero-knowledge proofs that NLP benchmark scores were computed correctly.**

Spectacles compiles NLP scoring metrics (BLEU, ROUGE, Token F1, Exact Match, chrF)
into weighted finite automata (WFA), then lowers those automata to STARK arithmetic
circuits over the Goldilocks field. The result: a cryptographic proof that a reported
score matches the actual computation, verifiable in milliseconds.

## Table of Contents

- [Key Results](#key-results)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [CLI Reference](#cli-reference)
  - [score](#score)
  - [batch-score](#batch-score)
  - [compile-circuit](#compile-circuit)
  - [estimate-size](#estimate-size)
  - [hash](#hash)
  - [verify-proof](#verify-proof)
  - [differential-test](#differential-test)
- [Supported Metrics](#supported-metrics)
- [Output Formats](#output-formats)
- [Triple Agreement Verification](#triple-agreement-verification)
- [Batch Scoring](#batch-scoring-in-depth)
- [Architecture](#architecture)
- [STARK Proof System](#stark-proof-system)
- [Workspace Crates](#workspace-crates)
- [License](#license)

---

## Key Results

| Metric | Value | Source |
|--------|-------|--------|
| Compilation checks | **68,748 tests, 0 disagreements** | `data/compilation_correctness.json` |
| Comprehensive benchmark | **1,230 checks across 6 metrics, 0 disagreements** | `data/comprehensive_benchmark_results.json` |
| E2E certification proofs | **49 metric-bound STARK proofs** across 5 metrics | `data/comprehensive_benchmark_results.json` |
| STARK proofs verified | **76 proofs** (4–1,024 states) | `data/stark_scaling_regression.json` |
| 400-state BLEU-4 target | **3.8s prove, 1.5ms verify** | measured |
| Max proven states | **1,024** (49.9s prove, 2.3ms verify) | measured |
| WFA metric coverage | **21/31 metrics (67.7%) WFA-representable** | census |
| Multi-layer contamination | **100% accuracy** (vs 83.3% n-gram-only) | `data/contamination_multi_layer_v2.json` |

---

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) 1.75+ (edition 2021)

### Build from source

```bash
git clone <repository-url>
cd zk-nlp-scoring
cargo build --release
```

The CLI binary is located at `target/release/spectacles-cli`.

### Verify the build

```console
$ ./target/release/spectacles-cli version
spectacles-cli 0.1.0
```

---

## Quick Start

Score a candidate string against a reference:

```console
$ ./target/release/spectacles-cli score --metric bleu \
    --candidate "the cat sat on the mat" \
    --reference "the cat is on the mat"
bleu  score=0.4889  BP=1.0000
```

Check exact match:

```console
$ ./target/release/spectacles-cli score --metric exact_match \
    --candidate "hello world" --reference "hello world"
exact_match  score=1.0000

$ ./target/release/spectacles-cli score --metric exact_match \
    --candidate "hello" --reference "world"
exact_match  score=0.0000
```

Get JSON output:

```console
$ ./target/release/spectacles-cli score --metric bleu \
    --candidate "the cat sat on the mat" \
    --reference "the cat is on the mat" \
    --output-format json
[
  {
    "metric": "bleu",
    "score": 0.488923022434901,
    "brevity_penalty": 1.0
  }
]
```

---

## CLI Reference

```
spectacles-cli - Spectacles ZK scoring framework

USAGE:
  spectacles-cli <COMMAND> [OPTIONS]

COMMANDS:
  score              Score a candidate against a reference
  batch-score        Score multiple pairs from a file (JSON, JSONL, CSV, TSV, HF datasets, SQuAD)
  differential-test  Run differential tests across implementations
  compile-circuit    Compile a scoring metric to an arithmetic circuit
  estimate-size      Estimate proof size for a configuration
  hash               Hash input data with BLAKE3
  verify-proof       Verify a previously generated proof file
  help               Print this help message
  version            Print version
```

### `score`

Score a single candidate–reference pair with one metric.

**Options:**

| Flag | Short | Description |
|------|-------|-------------|
| `--metric` | `-m` | Metric name: `exact_match`, `token_f1`, `bleu`, `rouge1`, `rouge2`, `rougel` |
| `--candidate` | `-c` | Candidate text |
| `--reference` | `-r` | Reference text |
| `--triple` | `-t` | Run all three implementations (reference, automaton, circuit) and check agreement |
| `--output-format` | | Output format: `text` (default), `json`, `csv`, `markdown` |

#### Exact Match

Returns 1.0 if strings are identical, 0.0 otherwise:

```console
$ ./target/release/spectacles-cli score --metric exact_match \
    --candidate "hello world" --reference "hello world"
exact_match  score=1.0000

$ ./target/release/spectacles-cli score --metric exact_match \
    --candidate "hello" --reference "world"
exact_match  score=0.0000
```

#### BLEU

Bilingual Evaluation Understudy score with brevity penalty:

```console
$ ./target/release/spectacles-cli score --metric bleu \
    --candidate "the cat sat on the mat" \
    --reference "the cat is on the mat"
bleu  score=0.4889  BP=1.0000
```

#### Token F1

Token-level F1 with precision and recall:

```console
$ ./target/release/spectacles-cli score --metric token_f1 \
    --candidate "the quick brown fox" \
    --reference "the slow brown dog"
token_f1  score=0.5000  P=0.5000  R=0.5000
```

Two out of four tokens match ("the", "brown"), giving precision = 2/4 = 0.5,
recall = 2/4 = 0.5, F1 = 0.5.

#### ROUGE-1

Unigram overlap F-measure:

```console
$ ./target/release/spectacles-cli score --metric rouge1 \
    --candidate "reference text here" \
    --reference "candidate text here"
rouge1  score=0.6667  P=0.6667  R=0.6667
```

#### ROUGE-2

Bigram overlap F-measure:

```console
$ ./target/release/spectacles-cli score --metric rouge2 \
    --candidate "the quick brown fox jumps over" \
    --reference "the quick red fox jumps high"
rouge2  score=0.4000  P=0.4000  R=0.4000
```

#### ROUGE-L

Longest common subsequence F-measure:

```console
$ ./target/release/spectacles-cli score --metric rougel \
    --candidate "the quick brown fox jumps over the lazy dog" \
    --reference "the fast brown fox leaps over a lazy dog"
rougel  score=0.6667  P=0.6667  R=0.6667
```

---

### `batch-score`

Score multiple candidate–reference pairs from a file.

**Options:**

| Flag | Short | Description |
|------|-------|-------------|
| `--input` | `-i` | Input file path (CSV, TSV, JSON, JSONL, HF datasets, SQuAD) |
| `--metric` | `-m` | Single metric name |
| `--metrics` | | Comma-separated list of metrics (e.g., `exact_match,bleu,rouge1`) |
| `--output-format` | | Output format: `text` (default), `json`, `csv`, `markdown` |

#### CSV input, single metric

Given a CSV file `pairs.csv`:

```csv
candidate,reference
"the cat sat on the mat","the cat is on the mat"
"hello world","hello world"
"the quick brown fox","the slow brown dog"
"machine learning is great","deep learning is great"
```

```console
$ ./target/release/spectacles-cli batch-score --input pairs.csv --metric bleu
Scoring 4 pairs with 1 metric(s): bleu
[0] bleu  score=0.4889  BP=1.0000
[1] bleu  score=1.0000  BP=1.0000
[2] bleu  score=0.3976  BP=1.0000
[3] bleu  score=0.6687  BP=1.0000
```

#### JSONL input

Given a JSONL file `pairs.jsonl`:

```jsonl
{"candidate": "the cat sat on the mat", "reference": "the cat is on the mat"}
{"candidate": "hello world", "reference": "hello world"}
{"candidate": "a quick test", "reference": "a fast test"}
```

```console
$ ./target/release/spectacles-cli batch-score --input pairs.jsonl --metric token_f1
Scoring 3 pairs with 1 metric(s): token_f1
[0] token_f1  score=0.8333  P=0.8333  R=0.8333
[1] token_f1  score=1.0000  P=1.0000  R=1.0000
[2] token_f1  score=0.6667  P=0.6667  R=0.6667
```

#### Multiple metrics at once

Use `--metrics` with a comma-separated list:

```console
$ ./target/release/spectacles-cli batch-score --input pairs.csv \
    --metrics exact_match,bleu,rouge1
Scoring 4 pairs with 3 metric(s): exact_match, bleu, rouge1
[0] exact_match  score=0.0000
[0] bleu         score=0.4889  BP=1.0000
[0] rouge1       score=0.8333  P=0.8333  R=0.8333
...
```

See [Output Formats](#output-formats) for `--output-format` variants.

---

### `compile-circuit`

Compile a scoring metric to a STARK arithmetic circuit.

```console
$ ./target/release/spectacles-cli compile-circuit --metric exact_match
Compiling circuit for metric: exact_match
Max input length: 100
Estimated circuit size: 1000 constraints
Estimated proof size: 23520 bytes (23.0 KB)

$ ./target/release/spectacles-cli compile-circuit --metric bleu
Compiling circuit for metric: bleu
Max input length: 100
Estimated circuit size: 1000 constraints
Estimated proof size: 23520 bytes (23.0 KB)

$ ./target/release/spectacles-cli compile-circuit --metric token_f1
Compiling circuit for metric: token_f1
Max input length: 100
Estimated circuit size: 1000 constraints
Estimated proof size: 23520 bytes (23.0 KB)

$ ./target/release/spectacles-cli compile-circuit --metric rouge1
Compiling circuit for metric: rouge1
Max input length: 100
Estimated circuit size: 1000 constraints
Estimated proof size: 23520 bytes (23.0 KB)

$ ./target/release/spectacles-cli compile-circuit --metric rougel
Compiling circuit for metric: rougel
Max input length: 100
Estimated circuit size: 1000 constraints
Estimated proof size: 23520 bytes (23.0 KB)
```

---

### `estimate-size`

Estimate proof size for a given constraint and wire count:

```console
$ ./target/release/spectacles-cli estimate-size --constraints 1000 --wires 500
Proof size estimate:
  Constraints:    1000
  Wires:          500
  Security bits:  128
  Estimated size: 23520 bytes (23.0 KB)

$ ./target/release/spectacles-cli estimate-size --constraints 5000 --wires 2500
Proof size estimate:
  Constraints:    5000
  Wires:          2500
  Security bits:  128
  Estimated size: 74592 bytes (72.8 KB)
```

---

### `hash`

Hash input data with BLAKE3:

```console
$ ./target/release/spectacles-cli hash "hello world"
d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24

$ ./target/release/spectacles-cli hash "spectacles framework"
31e6db321b693d0247da20810cd769f21cee7501224aa3839ffe4e20f126d1e7
```

The BLAKE3 hash is used internally for Merkle commitments to WFA transition tables,
cryptographically binding each STARK proof to the specific metric definition.

---

### `verify-proof`

Verify a previously generated STARK proof file:

```
spectacles-cli verify-proof --proof result.spct
```

Accepts `.spct` (binary) or `.json` proof files generated by the certification pipeline.

---

### `differential-test`

Run differential tests comparing the reference, automaton, and circuit implementations:

```
spectacles-cli differential-test --count 100
```

> **Note:** Differential tests with STARK proofs can be slow (minutes to hours depending
> on `--count`). For quick validation, use `--triple` on the `score` command instead.

---

## Supported Metrics

| Metric | CLI name | Semiring | Description |
|--------|----------|----------|-------------|
| Exact Match | `exact_match` | Boolean | 1.0 if strings are identical, 0.0 otherwise |
| Token F1 | `token_f1` | Counting | Token-level precision, recall, and F1 |
| BLEU | `bleu` | Counting | BLEU-4 with brevity penalty |
| ROUGE-1 | `rouge1` | Counting | Unigram overlap F-measure |
| ROUGE-2 | `rouge2` | Counting | Bigram overlap F-measure |
| ROUGE-L | `rougel` | Tropical | Longest common subsequence F-measure |

Each metric is implemented three times:
1. **Reference** — standard textbook implementation
2. **Automaton** — weighted finite automaton over a typed semiring
3. **Circuit** — arithmetic circuit over the Goldilocks field (p = 2⁶⁴ − 2³² + 1)

---

## Output Formats

The `--output-format` flag is available on both `score` and `batch-score`.

### Text (default)

```console
$ ./target/release/spectacles-cli score --metric token_f1 \
    --candidate "the quick brown fox" --reference "the slow brown dog"
token_f1  score=0.5000  P=0.5000  R=0.5000
```

### JSON

```console
$ ./target/release/spectacles-cli score --metric token_f1 \
    --candidate "the quick brown fox" --reference "the slow brown dog" \
    --output-format json
[
  {
    "metric": "token_f1",
    "score": 0.5,
    "precision": 0.5,
    "recall": 0.5
  }
]
```

### CSV

```console
$ ./target/release/spectacles-cli score --metric bleu \
    --candidate "the cat sat on the mat" \
    --reference "the cat is on the mat" \
    --output-format csv
metric,index,score,precision,recall,brevity_penalty
bleu,,0.4889,,,1.0000
```

### Markdown

```console
$ ./target/release/spectacles-cli score --metric bleu \
    --candidate "the cat sat on the mat" \
    --reference "the cat is on the mat" \
    --output-format markdown
| metric | index | score | precision | recall | BP |
|--------|-------|-------|-----------|--------|----|
| bleu |  | 0.4889 |  |  | 1.0000 |
```

### Batch score with markdown

```console
$ ./target/release/spectacles-cli batch-score --input pairs.csv \
    --metrics exact_match,bleu,rouge1 --output-format markdown
Scoring 4 pairs with 3 metric(s): exact_match, bleu, rouge1
| metric | index | score | precision | recall | BP |
|--------|-------|-------|-----------|--------|----|
| exact_match | 0 | 0.0000 |  |  |  |
| exact_match | 1 | 1.0000 |  |  |  |
| exact_match | 2 | 0.0000 |  |  |  |
| exact_match | 3 | 0.0000 |  |  |  |
| bleu | 0 | 0.4889 |  |  | 1.0000 |
| bleu | 1 | 1.0000 |  |  | 1.0000 |
| bleu | 2 | 0.3976 |  |  | 1.0000 |
| bleu | 3 | 0.6687 |  |  | 1.0000 |
| rouge1 | 0 | 0.8333 | 0.8333 | 0.8333 |  |
| rouge1 | 1 | 1.0000 | 1.0000 | 1.0000 |  |
| rouge1 | 2 | 0.5000 | 0.5000 | 0.5000 |  |
| rouge1 | 3 | 0.7500 | 0.7500 | 0.7500 |  |
```

### Batch score with CSV output

```console
$ ./target/release/spectacles-cli batch-score --input pairs.csv \
    --metrics exact_match,bleu,rouge1 --output-format csv
Scoring 4 pairs with 3 metric(s): exact_match, bleu, rouge1
metric,index,score,precision,recall,brevity_penalty
exact_match,0,0.0000,,,
exact_match,1,1.0000,,,
exact_match,2,0.0000,,,
exact_match,3,0.0000,,,
bleu,0,0.4889,,,1.0000
bleu,1,1.0000,,,1.0000
bleu,2,0.3976,,,1.0000
bleu,3,0.6687,,,1.0000
rouge1,0,0.8333,0.8333,0.8333,
rouge1,1,1.0000,1.0000,1.0000,
rouge1,2,0.5000,0.5000,0.5000,
rouge1,3,0.7500,0.7500,0.7500,
```

### Batch score with JSON output

```console
$ ./target/release/spectacles-cli batch-score --input pairs.csv \
    --metrics exact_match,bleu,rouge1 --output-format json
Scoring 4 pairs with 3 metric(s): exact_match, bleu, rouge1
[
  {
    "metric": "exact_match",
    "index": 0,
    "score": 0.0
  },
  {
    "metric": "exact_match",
    "index": 1,
    "score": 1.0
  },
  ...
  {
    "metric": "rouge1",
    "index": 3,
    "score": 0.75,
    "precision": 0.75,
    "recall": 0.75
  }
]
```

---

## Triple Agreement Verification

The `--triple` flag runs all three implementations (reference, automaton, circuit)
and confirms they produce identical results. This is the core correctness guarantee.

### Exact Match — triple check

```console
$ ./target/release/spectacles-cli score --metric exact_match \
    --candidate "hello world" --reference "hello world" --triple
Metric: ExactMatch
Candidate: "hello world"
Reference: "hello world"

Reference impl:  true
Automaton impl:  true
Circuit impl:    true
Agreement:       true
```

### Token F1 — triple check

```console
$ ./target/release/spectacles-cli score --metric token_f1 \
    --candidate "the quick brown fox" --reference "the slow brown dog" --triple
Metric: TokenF1
Candidate: "the quick brown fox"
Reference: "the slow brown dog"

Reference impl:  P=0.5000 R=0.5000 F1=0.5000
Automaton impl:  P=0.5000 R=0.5000 F1=0.5000
Circuit impl:    P=0.5000 R=0.5000 F1=0.5000
Agreement:       true
```

### BLEU — triple check

```console
$ ./target/release/spectacles-cli score --metric bleu \
    --candidate "the cat sat on the mat" \
    --reference "the cat is on the mat" --triple
Metric: Bleu
Candidate: "the cat sat on the mat"
Reference: "the cat is on the mat"

Reference impl:  0.4889
Automaton impl:  0.4889
Circuit impl:    0.4889
Agreement:       true
```

### ROUGE-1 — triple check

```console
$ ./target/release/spectacles-cli score --metric rouge1 \
    --candidate "the quick brown fox jumps over the lazy dog" \
    --reference "the fast brown fox leaps over a lazy dog" --triple
Metric: Rouge1
Candidate: "the quick brown fox jumps over the lazy dog"
Reference: "the fast brown fox leaps over a lazy dog"

Reference impl:  F1=0.6667
Automaton impl:  F1=0.6667
Circuit impl:    F1=0.6667
Agreement:       true
```

### ROUGE-2 — triple check

```console
$ ./target/release/spectacles-cli score --metric rouge2 \
    --candidate "the quick brown fox jumps over the lazy dog" \
    --reference "the fast brown fox leaps over a lazy dog" --triple
Metric: Rouge2
Candidate: "the quick brown fox jumps over the lazy dog"
Reference: "the fast brown fox leaps over a lazy dog"

Reference impl:  F1=0.2500
Automaton impl:  F1=0.2500
Circuit impl:    F1=0.2500
Agreement:       true
```

### ROUGE-L — triple check

```console
$ ./target/release/spectacles-cli score --metric rougel \
    --candidate "the quick brown fox jumps over the lazy dog" \
    --reference "the fast brown fox leaps over a lazy dog" --triple
Metric: RougeL
Candidate: "the quick brown fox jumps over the lazy dog"
Reference: "the fast brown fox leaps over a lazy dog"

Reference impl:  F1=0.6667
Automaton impl:  F1=0.6667
Circuit impl:    F1=0.6667
Agreement:       true
```

---

## Batch Scoring In Depth

### Supported input formats

| Format | Extension | Fields |
|--------|-----------|--------|
| CSV | `.csv` | `candidate`, `reference` columns |
| TSV | `.tsv` | `candidate`, `reference` columns (tab-separated) |
| JSON | `.json` | Array of `{"candidate": "...", "reference": "..."}` |
| JSONL | `.jsonl` | One JSON object per line |
| HF datasets | varies | Hugging Face dataset format |
| SQuAD | `.json` | SQuAD-format JSON |

### Workflow example

1. Prepare your evaluation pairs as CSV:

```csv
candidate,reference
"the cat sat on the mat","the cat is on the mat"
"hello world","hello world"
"the quick brown fox","the slow brown dog"
"machine learning is great","deep learning is great"
```

2. Run all metrics and get a markdown report:

```console
$ ./target/release/spectacles-cli batch-score --input pairs.csv \
    --metrics exact_match,bleu,token_f1,rouge1,rouge2,rougel \
    --output-format markdown
```

3. Pipe JSON output for programmatic consumption:

```console
$ ./target/release/spectacles-cli batch-score --input pairs.csv \
    --metrics bleu,rouge1 --output-format json > results.json
```

---

## Architecture

Spectacles uses a **two-tier compilation** pipeline:

```
NLP Metric
    │
    ▼
Weighted Finite Automaton (WFA) over typed semiring
    │
    ├─ Tier 1: Direct semiring homomorphism φ: S → F_p
    │          (Boolean, Counting semirings → Goldilocks field)
    │
    └─ Tier 2: Comparison gadgets with 62-bit decomposition
               (Tropical min-plus semiring → Goldilocks field)
    │
    ▼
STARK Arithmetic Circuit
    │
    ▼
Zero-Knowledge Proof (BLAKE3 Merkle commitment to transition table)
```

### Semiring decomposition

| Metric | Semiring | Compilation Tier |
|--------|----------|-----------------|
| Exact Match | Boolean | Tier 1 (algebraic) |
| Token F1 | Counting | Tier 1 + F-measure gadget |
| BLEU-4 | Counting | Tier 1 + geometric mean + brevity penalty gadgets |
| ROUGE-1/2 | Counting | Tier 1 + F-measure gadget |
| ROUGE-L | Tropical | Tier 2 (comparison gadgets) |

---

## STARK Proof System

Each STARK proof includes a BLAKE3 Merkle commitment to the WFA transition table,
cryptographically binding the proof to the specific metric definition.

### Scaling benchmarks

| States | Prove Time | Verify Time | Proof Size |
|--------|-----------|-------------|------------|
| 64 | 71 ms | 0.9 ms | 107 KiB |
| 256 | 1,191 ms | 1.4 ms | 204 KiB |
| 400 | 3,821 ms | 1.5 ms | 270 KiB |
| 1,024 | 49,883 ms | 2.3 ms | 490 KiB |

Verification is constant-time (~1–2 ms) regardless of circuit size.

### Running STARK benchmarks

```bash
cargo run --release --bin stark_scaling_2048
cargo run --release --bin comprehensive_benchmark
```

> **Warning:** STARK proof generation is computationally intensive.
> The `comprehensive_benchmark` generates 25+ proofs and may take several minutes.

---

## Workspace Crates

| Crate | Description |
|-------|-------------|
| `spectacles-core` | WFA engine, semirings, circuit compiler, STARK prover/verifier |
| `spectacles-cli` | Command-line interface (`spectacles-cli` binary) |
| `spectacles-integration` | Integration tests and cross-implementation validation |
| `spectacles-examples` | Benchmark binaries (`comprehensive_benchmark`, `stark_scaling_2048`, etc.) |

### Core modules

```
crates/spectacles-core/src/
├── circuit/      # Arithmetic circuit representation and compilation
├── evalspec/     # Evaluation specification types
├── protocol/     # STARK proof protocol
├── psi/          # Private set intersection
├── scoring/      # Reference metric implementations
├── wfa/          # Weighted finite automata engine
├── utils/        # Shared utilities
├── pipeline.rs   # End-to-end certification pipeline
└── lib.rs        # Public API
```

---

## License

Research use only.
