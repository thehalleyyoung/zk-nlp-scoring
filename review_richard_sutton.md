# Review: Spectacles — Verified Compiler from Semiring-Weighted Finite Automata to Zero-Knowledge Scoring Circuits

**Reviewer:** Richard S. Sutton (Reinforcement Learning Specialist)

## Summary

Spectacles presents a cryptographically elegant pipeline for compiling semiring-weighted finite automata into zero-knowledge scoring circuits, targeting contamination detection and score integrity in LLM evaluation. The system is technically impressive — OPRF-based PSI, STARK verification, Lean 4 proofs — but I question whether the problem it solves will persist long enough to justify the investment. We may be building a cathedral for a town that's about to move.

## Strengths

1. **Cryptographic architecture is genuinely clever.** The combination of OPRF-based private set intersection for contamination with STARK-verified scoring is a clean decomposition of two distinct trust problems. The authors correctly identify that contamination and score integrity are orthogonal concerns requiring different cryptographic primitives.

2. **The WFA-to-circuit compilation is principled.** Decomposing metrics over semirings and providing a two-tier compilation strategy (algebraic for clean cases, gadget-assisted for tropical) shows real mathematical taste. The decidable specification equivalence result is a nice theoretical contribution in its own right.

3. **Lean 4 formalization sets a high bar.** Sorry-free mechanized proofs of the KleeneSemiring typeclass and compilation correctness are commendable. If you're going to build formal verification infrastructure, this is how to do it — no half-measures.

## Weaknesses

1. **The metric scope is fatally narrow.** Seven WFA-decomposable metrics (exact match, F1, BLEU, ROUGE, pass@k) represent yesterday's evaluation paradigm. The metrics that actually matter for frontier models — coherence, reasoning depth, instruction following — are not decomposable over finite semirings. The authors acknowledge excluding BERTScore and LLM-as-judge, but these exclusions gut the system's relevance.

2. **No learning component whatsoever.** The entire system is static: fixed metrics, fixed compilation, fixed verification. There is no mechanism for the evaluation pipeline to adapt as our understanding of model quality evolves. In a field where the target is moving quarterly, a system that cannot learn is a system that cannot survive.

3. **Contamination may be a transitional problem.** As training corpora scale to encompass effectively all public text, and as synthetic data generation matures, the sharp boundary between "training data" and "evaluation data" dissolves. The elaborate PSI machinery assumes a world where contamination is detectable because training sets are bounded — an assumption with a limited shelf life.

4. **Enormous complexity for diminishing returns.** ~142K lines of code for verified evaluation of seven metrics. The engineering cost per verified metric is staggering. Meanwhile, a well-designed held-out evaluation with basic statistical controls achieves 90% of the trust guarantees at 1% of the complexity. The marginal value of cryptographic verification over good experimental hygiene is unclear.

5. **The evaluation landscape is shifting beneath this work.** Open-ended evaluation — human preference, arena-style comparisons, task completion in real environments — is rapidly displacing fixed-metric benchmarking. Spectacles verifies scores on benchmarks that the community is already abandoning. The verification is correct, but correctness of an irrelevant quantity is not useful.

## Questions for Authors

- What is your concrete plan for extending beyond the seven current metrics, and can you characterize the class of metrics that are fundamentally incompatible with your WFA framework?
- Have you estimated the total person-years invested in this system, and how does the cost-per-verified-metric compare to simply running evaluations on fresh, never-published benchmarks?
- How do you respond to the argument that contamination detection becomes meaningless when training data approaches the entirety of public text?

## Overall Assessment

Spectacles is a technically masterful solution to a problem whose importance is rapidly declining. The cryptographic and algebraic machinery is beautiful, and the Lean 4 formalization is exemplary — but the system verifies metrics the field is moving away from, for a contamination threat that may be transitional. **Score: 5/10 — impressive engineering on a shrinking problem.**
