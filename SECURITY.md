# Security status and threat model

## Summary

| Component | Status |
|---|---|
| `crates/spectacles-core/src/circuit/{stark,fri,merkle}.rs` (home-grown STARK, "legacy") | Experimental; not sound (see `docs/AUDIT_legacy_stark.md`). Not used for any security claim. |
| `crates/spectacles-p3` (Plonky3 0.8.0 uni-STARK, hiding FRI, Goldilocks) | Supported proving path. The AIR (`air.rs`) is ours and unaudited; the proof system is Plonky3's. |
| `zkvm/risc0` | Baseline: the same statement as a RISC Zero 3.0.6 guest program. |

## Threat model for the Plonky3 backend

Parties: a **benchmark owner** holds a private test set; an **evaluator**
(possibly the same party) runs a model and reports scores; a **verifier**
(reader of a leaderboard, model developer, auditor) checks the report.

1. Before evaluation, the owner publishes, for each item, the digest
   `d = Rp64_256(salt || ref_token_ids || 0-padding)` (Rescue-Prime as in
   winter-crypto 0.13.1). Salt: 8 uniformly random Goldilocks elements.
2. The candidate (model output) for each item is public, or at least known to
   the verifier; its tokenization (sacreBLEU 13a words or characters) and the
   token-id map (BLAKE2b-64 mod p) are public functions.
3. The prover proves: there exist salt and reference tokens (non-zero tokens
   followed only by zero padding) hashing to `d` such that the clipped n-gram
   match counts `m_1..m_N` and the reference length are the public values.
   The public values also carry `h = Rp64_256(N, T, |cand|, cand)`. The AIR is
   compiled from the candidate and pins `h` to that candidate with a
   first-row constraint; the verifier (`spectacles_p3::verify_statement`)
   compiles the AIR from the candidate it holds and checks `h` itself.
   A deployment uses `verify_committed_statement` with `expected_digest`
   retrieved from the owner's independently authenticated pre-evaluation manifest.
   The lower-level `verify_statement` checks internal proof consistency only;
   accepting the digest supplied by a prover permits reference substitution. It
   also pins the proof degree to `log2(T)` in plain mode and `log2(T)+1`
   in hiding mode, so the verified reference capacity is `T-8` tokens.
   Plonky3 absorbs the public values into the Fiat-Shamir transcript after
   the trace commitment and before the combination challenge and the
   out-of-domain point, so the candidate (whose ids are constants of the AIR)
   is fixed before any challenge is drawn. The verifier computes BLEU or chrF
   from the statistics and the public candidate-side counts.

Guarantees, assuming Plonky3's FRI-based STARK is sound and zero-knowledge at
the configured parameters and Rp64_256 is collision resistant:
* **Integrity.** The reported statistics are those of the committed reference
  and the given candidate. The reference is fixed by `d` before evaluation;
  the candidate is fixed by `h` before the transcript's challenges, so a
  prover cannot adapt either to the challenges.
* **Confidentiality.** The proof reveals only the public values (hiding FRI
  with salted Merkle leaves). The statistics themselves leak information about
  the reference (e.g. how many candidate unigrams occur in it). This leakage is
  inherent to publishing a per-item score; aggregating before publishing
  reduces it but needs a different statement.

Not covered: that the candidate is the model's output (needs a
model-inference proof or a trusted evaluation environment), that the reference tokenization
was done correctly at commit time, test-set contamination, and the meaning of
the metric. Token ids are 64-bit; two distinct tokens colliding would create
false matches. No collision occurs among the 11,970 distinct tokens of the
WMT22 data used here (`results/inputs/prep_summary.json`), and a deliberate
second preimage on a 64-bit id costs about 2^64 hash evaluations.

The salts used in this repository's evaluation are derived deterministically
from public item ids (`salt_for` in `p3_bench.rs`) so that digests are
reproducible; those commitments are not hiding. A deployment draws each salt
uniformly at random and keeps it secret.

## Parameters and estimated soundness

Configuration used for all reported numbers (`config.rs`): Goldilocks base
field, degree-2 extension challenge field (128-bit), blowup 8, 60 FRI queries,
16 bits of query grinding, binary folding, Poseidon2-Goldilocks (width 8,
4-element digest) Merkle trees and duplex challenger, 4 random codewords for
hiding. Plonky3's own estimator (`p3_uni_stark::security`), evaluated on the
AIR of every proved item with the committed (zero-knowledge, doubled) domain size (`results/p3/security_all_*.json`), gives at least 106
bits under the ethSTARK conjecture, 95 bits proven in the list-decoding regime
and 65 bits in the unique-decoding regime for all 4,074 BLEU statements, and
102 / 92 / 65 bits for the chrF sample. The sweep in
`results/p3/security_sweep_*.json` shows the conjectured level stays at or below
108 bits in zero-knowledge mode, limited by the 128-bit challenge field; more queries raise
only the proven bounds.

## What is and is not audited

* Plonky3: maintained, used in production by several zkVMs, and audited in
  parts; we did not find an audit report that covers the 0.8.0 `HidingFriPcs`
  zero-knowledge mode specifically. Its README states that verifier panics on
  adversarial inputs are not yet ruled out.
* RISC Zero 3.0.6: audited zkVM releases exist; we use it unmodified.
* Our AIR (`crates/spectacles-p3/src/air.rs`) and the Rescue re-implementation
  have had one independent review, whose attack tests are kept as regression
  tests (`crates/spectacles-p3/tests/audit_regressions.rs`, `audit/forge/`).
  Evidence: a pen-and-paper argument (paper, Section 4), the
  Rescue sponge checked against winter-crypto, randomized single-cell
  corruption tests, public-value tampering tests, and equality of the proved
  statistics with sacreBLEU on all WMT22 en-de sentences we ran.

## Application commitment boundary (wave 2)

The prospective experiment in `audit/wave2/plan.md` separates internal proof
validity from the owner's commitment policy. A valid substituted-reference
proof may pass internal verification; this is not a STARK forgery. The
explicit owner-digest API rejects it. Item ids, manifest signatures, manifest
completeness, prevention of omitted/duplicated items and pre-evaluation timing
remain application responsibilities. The API does not authenticate a manifest
or certify that scores were produced by a model.
