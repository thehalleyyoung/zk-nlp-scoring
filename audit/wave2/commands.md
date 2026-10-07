# Fresh commands and boundaries

From `crates/spectacles-p3` in the owned wave2 clone:

```bash
CARGO_TARGET_DIR=/Users/halley/Documents/repo-upgrades/journal-campaign/wave2/zk/build-target CARGO_BUILD_JOBS=1 RAYON_NUM_THREADS=1 cargo test --offline --release -- --test-threads=1 --nocapture > ../../audit/wave2/source-test.log 2>&1
```

The target cache was copied read-only from the prior campaign lane into this
owned target, then Cargo rebuilt the backend and all standalone tests from
source. The log records the compile and18 successes. No dependency upgrade
or global install. Warnings about two inherited unused imports were retained.
The original workspace (4,113 tests), Lean, RISC Zero and full cloud benchmark
were not rerun.

```bash
python3 scripts/wave2/independent_counts.py
python3 scripts/build_paper.py
python3 scripts/wave2/check_site.py
```

Final PDF log has no undefined references or overfull boxes. PDF pages1,3,6
were rendered and visually inspected for layout. This is representative
inspection, not a full page-by-page audit. The publication PDF archive manifest
lives outside this clone at `journal-campaign/pdf-archive/zk/wave2-manifest.json`.
The canonical source repository was read and cloned, never modified.
