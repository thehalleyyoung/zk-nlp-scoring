# Reproducing the results

All heavy runs were executed on Modal CPU containers through
`scripts/modal/run_remote.py` (Debian, rustc 1.88 for the original workspace,
latest stable for `spectacles-p3` and the RISC Zero host, RISC Zero toolchain
installed with `rzup`). Every run is logged in `results/modal_costs.jsonl`;
raw stdout/stderr of each run is kept in the Modal volume
`spectacles-cargo-cache` under `out/<tag>/` and copied into `results/`.
Locally, everything except the timings can be reproduced with
`cargo` (Rust stable), Python 3.11+ and `pip install sacrebleu==2.4.3`.

## 1. Inputs (WMT22 en-de)

```bash
# data/wmt22/*.de were downloaded from
# https://github.com/wmt-conference/wmt22-news-systems at commit
# 20240a2e96a50e823ea8cdf37fd8af52547146e3 (see data/wmt22/SOURCE.txt, SHA256SUMS)
python3 scripts/prepare_wmt22.py     # -> results/inputs/{bleu,chrf}_<system>.jsonl, prep_summary.json
python3 scripts/sample_items.py      # -> results/inputs/{bleu,chrf}_sample200.jsonl (seed 2026)
```

## 2. Plonky3 backend (`crates/spectacles-p3`)

```bash
cd crates/spectacles-p3 && cargo test --release          # Rescue vs winter-crypto, negative tests
# Modal (16 cores, RAYON_NUM_THREADS=16):
MODAL_CPU=8  modal run --detach scripts/modal/run_remote.py --tag p3_tests_sweep \
  --extra-setup "rustup toolchain install stable --profile minimal; export CARGO_TARGET_DIR=/cache/target-p3" \
  --cmd "bash scripts/modal/p3_tests_sweep.sh"        # results/p3/cargo_test.log, security_sweep_*.json
MODAL_CPU=16 MODAL_MEM_GB=32 modal run --detach scripts/modal/run_remote.py --tag p3_experiments \
  --extra-setup "rustup toolchain install stable --profile minimal; export CARGO_TARGET_DIR=/cache/target-p3; export RAYON_NUM_THREADS=16" \
  --cmd "bash scripts/modal/p3_experiments.sh"        # results/p3/{bleu,chrf}_*.json
MODAL_CPU=4 MODAL_MEM_GB=8 modal run --detach scripts/modal/run_remote.py --tag p3_security_all \
  --extra-setup "rustup toolchain install stable --profile minimal; export CARGO_TARGET_DIR=/cache/target-p3" \
  --cmd "bash scripts/modal/p3_security_all.sh"       # results/p3/security_all_*.json
python3 scripts/analyze_p3.py                          # -> results/summary_p3.json
```

The binaries can also be run directly, e.g.
`target/release/p3_bench results/inputs/bleu_sample200.jsonl out.json --mode zk --min-rows 128`.

All of section 2, plus the audit regression tests and the adaptive-forgery
test (`audit/forge/`), in one Modal run (the run that produced the committed
`results/p3/` and `results/audit/`):

```bash
MODAL_CPU=16 MODAL_MEM_GB=32 modal run --detach scripts/modal/run_remote.py --tag p3_fix_all2 \
  --extra-setup "rustup toolchain install stable --profile minimal; export CARGO_TARGET_DIR=/cache/target-p3; export RAYON_NUM_THREADS=16" \
  --cmd "bash scripts/modal/p3_fix_all.sh"
```

## 3. RISC Zero baseline (`zkvm/risc0`)

```bash
MODAL_CPU=16 MODAL_MEM_GB=32 modal run --detach scripts/modal/run_remote.py --tag r0_experiments \
  --extra-setup "source scripts/modal/r0_setup.sh" --cmd "bash scripts/modal/r0_experiments.sh"
python3 scripts/analyze_r0.py                          # -> results/summary_r0.json
```

## 4. Original workspace tests and differential binaries

```bash
MODAL_CPU=8 MODAL_MEM_GB=48 modal run --detach scripts/modal/run_remote.py --tag core_rerun2 \
  --cmd "bash scripts/modal/core_rerun.sh"             # results/core/*
cargo test --release -p spectacles-core --test legacy_stark_soundness_attacks
```

## 5. Lean

```bash
python3 scripts/lean_audit.py                          # runs lake build in lean/, writes results/lean_audit.json
```

## 6. Paper

```bash
python3 scripts/make_paper_numbers.py                  # results/*.json -> paper/numbers.tex, paper/rq3.tex
python3 scripts/build_paper.py                        # -> root tool_paper.pdf
```

## 7. Declared trace-size boundary check

```bash
CARGO_BUILD_JOBS=1 RAYON_NUM_THREADS=1 cargo test --offline --release \
  --manifest-path crates/spectacles-p3/Cargo.toml --test journal_attacks \
  -- --test-threads=1 --nocapture
```

`results/journal_audit/` retains the source-built baseline counterexample and
the repaired run. The six trace-size cases use declared `T=64`, actual sizes
64/128/256 and both proof modes. The test uses smaller FRI parameters than
the timed evaluation; its purpose is checking statement binding.

## 8. Owner-commitment substitution boundary

```bash
cd crates/spectacles-p3
CARGO_BUILD_JOBS=1 RAYON_NUM_THREADS=1 cargo test --offline --release \
  --test commitment_binding -- --test-threads=1 --nocapture
```

Frozen plan and raw source-build output: `audit/wave2/`. This fresh test uses
small FRI settings for a statement-policy boundary, not the timed security
configuration. Compare `verify_statement` with `verify_committed_statement`
using a digest obtained independently of the prover. No paid calls were made
in this round. Historical provider spending is estimated at $3.9396 and is
not reconciled against provider receipts.
