# Review: Spectacles — Verified Compiler from Semiring-Weighted Finite Automata to Zero-Knowledge Scoring Circuits

**Reviewer:** Ilya Sutskever (Generative Models Expert)

## Summary

Spectacles constructs an impressive formal pipeline — WFA decomposition over semirings, verified compilation to ZK circuits, OPRF-based contamination detection — for guaranteeing the integrity of LLM benchmark evaluation. The engineering and mathematical depth are substantial. But I keep returning to a basic question: are the seven metrics this system can verify the metrics that actually tell us whether a language model is good?

## Strengths

1. **The trust problem is real and currently unsolved.** Benchmark gaming, score fabrication, and data contamination are genuine pathologies in LLM evaluation today. The authors correctly identify that the community lacks any mechanism for *cryptographically verifiable* evaluation claims. This is a real gap, and Spectacles takes a serious approach to filling it.

2. **The algebraic decomposition is mathematically beautiful.** Expressing exact match, F1, BLEU, ROUGE, and pass@k as weighted finite automata over appropriate semirings is a lovely piece of applied algebra. The two-tier compilation — clean algebraic compilation where the semiring structure cooperates, gadget-assisted compilation for the tropical semiring — reflects genuine understanding of where the math works cleanly and where it needs help.

3. **Proof generation and verification times are practical.** 1-30 seconds for proof generation and sub-100ms verification are numbers that could actually support real-world deployment. Many formal verification systems produce theoretically beautiful proofs that take hours to generate; Spectacles avoids this trap.

## Weaknesses

1. **The metrics that matter most are excluded.** The seven verified metrics — exact match, F1, BLEU, ROUGE, pass@k — are surface-level text comparison metrics. They tell you whether the model produced the right tokens, not whether it understood the question, reasoned correctly, or generated something genuinely useful. BERTScore and LLM-as-judge are excluded, but these are precisely the metrics that correlate with human judgment of model quality. Verifying the wrong metrics with perfect integrity is not progress.

2. **The bet on formal metrics may be losing.** The trajectory of LLM evaluation is toward open-ended assessment: human preference rankings, arena-style head-to-head comparisons, task completion in realistic environments. These evaluation paradigms are fundamentally incompatible with WFA decomposition. Spectacles is building verified infrastructure for a measurement paradigm that frontier labs are already moving beyond.

3. **~142K LoC is enormous for seven metrics.** The ratio of engineering complexity to functional scope is concerning. Twenty thousand lines per verified metric suggests that extending to even twenty metrics would require a team-sized engineering effort maintained indefinitely. Compare this to a simple approach: publish evaluation code, use deterministic builds, let anyone reproduce. The marginal trust gain from ZK proofs over reproducibility may not justify the complexity.

4. **Contamination is a problem, but perhaps not this problem.** The OPRF-based PSI approach assumes a clear boundary between training data and evaluation data, and that the model provider will participate honestly in the protocol. But the deepest contamination concerns — memorization of reformulated problems, indirect exposure through web-crawled discussions of benchmark answers — are not addressable through set intersection on training corpora. The contamination that matters is the contamination you cannot define as set membership.

5. **End-to-end neural evaluation may obsolete formal metrics entirely.** If the field converges on using powerful models to evaluate less powerful models — and the evidence suggests it is — then the relevant "metric" becomes a neural network forward pass, not a finite automaton computation. The entire WFA-to-ZK pipeline would need to be rebuilt around verified neural inference, which is a fundamentally different and much harder problem.

## Questions for Authors

- If you could verify any three metrics beyond your current seven, which would you choose, and what are the specific technical barriers to including them in the WFA framework?
- What is your response to the argument that reproducible evaluation (publish code + deterministic execution) provides sufficient trust guarantees for most practical purposes, without cryptographic machinery?
- How do you see Spectacles adapting if the dominant evaluation paradigm shifts entirely to model-based judging within the next two to three years?

## Overall Assessment

Spectacles demonstrates exceptional mathematical and engineering depth in service of a genuine problem. The WFA-to-ZK compilation pipeline is rigorous, the Lean 4 proofs are admirable, and the performance numbers are practical. But the system verifies metrics that are increasingly peripheral to how we actually assess language model quality, and the enormous complexity cost makes extension difficult. I respect the vision but question the bet. **Score: 5/10 — technically outstanding work that may be solving yesterday's evaluation problem.**
