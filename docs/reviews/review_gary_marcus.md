# Review: Spectacles — Verified Compiler from Semiring-Weighted Finite Automata to Zero-Knowledge Scoring Circuits

**Reviewer:** Gary Marcus (Neural-Symbolic Integration Expert)

## Summary

Spectacles is exactly the kind of rigorous symbolic infrastructure that the AI field desperately needs and chronically undervalues. The system provides end-to-end verified compilation from weighted finite automata over semirings to zero-knowledge scoring circuits, addressing the very real problem that we cannot currently trust LLM benchmark results. This is symbolic AI at its finest — principled, mechanically verified, and aimed at a genuine failure mode.

## Strengths

1. **A triumph of symbolic methods where they belong.** This is a textbook case of applying formal methods to a problem that *requires* formal guarantees. You cannot "approximately" verify that a benchmark score wasn't fabricated — you need mathematical certainty. The WFA algebra → verified compilation → ZK proof pipeline is exactly the right architecture, and no amount of neural hand-waving could substitute for it.

2. **The specification equivalence result is underappreciated.** Decidable equivalence for EvalSpec specifications means you can formally prove that two different metric implementations compute the same function. This is a foundational contribution that transcends the immediate application. The denotational semantics for EvalSpec give the DSL real mathematical meaning, not just operational convenience.

3. **Honest about scope limitations.** I am so tired of papers that claim universality. The authors explicitly bound their contribution to seven WFA-decomposable metrics and clearly state that BERTScore and LLM-as-judge fall outside scope. This intellectual honesty is refreshing and, frankly, characteristic of work rooted in formal methods rather than hype.

4. **The Lean 4 formalization is gold-standard verification.** Sorry-free mechanized proofs of the KleeneSemiring typeclass and compilation correctness pipelines are the real deal. This isn't "we checked a few test cases" — it's mathematical proof that the compilation preserves semantics. The two-tier approach (algebraic for clean semiring cases, gadget-assisted for tropical) shows the authors understand both the theory and its practical limits.

## Weaknesses

1. **Seven metrics may not be enough to matter.** While I applaud the honesty about scope, seven metrics is a thin foundation for an entire verification infrastructure. The system needs a clear roadmap for extending to at least 15-20 metrics to achieve critical mass. The gap between "what we can verify" and "what people actually use" needs to shrink.

2. **The compilation complexity is a practical barrier.** ~117-142K lines of code is an enormous surface area to maintain. Formal verification of the core is wonderful, but the surrounding infrastructure — the OPRF-based PSI, the STARK integration, the EvalSpec toolchain — presumably contains unverified code that could harbor bugs. What fraction of the total codebase is actually covered by the Lean proofs?

3. **Extension to neural metrics remains an open problem.** The fundamental limitation is that metrics like BERTScore involve neural network forward passes, which are not naturally expressible as WFA computations. The authors need to address whether there is a principled path to verified neural metric evaluation, or whether Spectacles is inherently limited to symbolic metrics.

4. **Adoption requires a trust ecosystem that doesn't exist yet.** Verified evaluation is only valuable if stakeholders — benchmark organizers, model developers, regulators — actually demand cryptographic proofs of score integrity. Building the technical infrastructure without the institutional infrastructure risks creating a solution without users.

## Questions for Authors

- What fraction of the ~142K LoC is covered by Lean 4 mechanized proofs, and what is your strategy for ensuring correctness of the unverified portions?
- Is there a theoretical barrier to extending the WFA framework to handle metrics that involve continuous-valued neural computations, or is this purely an engineering challenge?
- Have you engaged with any benchmark organizations (e.g., HELM, Open LLM Leaderboard) about integrating Spectacles into their evaluation pipelines?

## Overall Assessment

Spectacles represents the best of what symbolic methods can contribute to the current AI landscape: rigorous guarantees for a problem where rigor genuinely matters. The WFA-to-ZK compilation pipeline is elegant, the Lean 4 formalization is exemplary, and the intellectual honesty about scope is a model for the field. The main risk is that the metric coverage is too narrow for practical impact. **Score: 8/10 — a landmark in verified AI evaluation infrastructure, needing broader metric coverage to fulfill its promise.**
