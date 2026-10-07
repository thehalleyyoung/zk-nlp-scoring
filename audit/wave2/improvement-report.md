# Improved checkpoint

Baseline a9ce322 already repaired proof-height binding. This round reduces a
different bottleneck: deployment had a low-level verifier whose digest came
from the prover. New `verify_committed_statement` compares it with a digest
obtained independently from the owner before calling the existing verifier.
The low-level API remains backward-compatible and explicitly documented.

Fresh evidence: `source-test.log` contains a 3m35s offline source rebuild and
18 passing tests. The prospective `plan.md` predates execution. Owner and
substitute references give Python-independent clipped counts [2,1] and [4,3],
respectively. Across two modes, honest controls pass and substitution fails
only when the external owner digest is fixed. Digest transplantation fails.
This finding changes application verification policy; it does not imply a
new cryptographic attack, full audit or guarantee against benchmark omission.

Paper, SECURITY, README and reproduction instructions incorporate the result
and conditions. South2024/P4Bench distinctions bound novelty; a full matched
method comparison is not claimed. Root `tool_paper.pdf` is source-built; the
superseded tracked PDF is hash-archived outside the repository. Root site and
Pages workflow are locally prepared, never pushed or deployed.

Remaining: complete AIR/hiding audit; authenticated complete manifests;
full matched prior-method implementation; dataset rights/authorship/license
resolution; historical spend reconciliation. Outcome: improved checkpoint,
NOT_YET for Zenodo release, journal readiness unestablished.
