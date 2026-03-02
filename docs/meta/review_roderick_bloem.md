# Spectacles: Machine-Checkable Proofs for NLP Benchmark Scoring

**Reviewer:** Roderick Bloem (machine_learning_specification_safety_researcher)

## Summary

Spectacles tackles a real and underappreciated problem in ML evaluation: how do we know that reported benchmark scores were computed correctly? The proposed solution—compiling NLP metrics to weighted finite automata, formalizing them via Schützenberger's rational power series, and certifying execution with STARK proofs—is technically sophisticated and architecturally novel. The E2E certification pipeline, with its 24/24 verified proofs and zero-disagreement triple checks, demonstrates a working system rather than a paper design.

From a specification safety perspective, however, the central question is whether the WFA formalization faithfully captures the intended metric semantics—not merely a syntactic approximation thereof. NLP metrics like BLEU, ROUGE, and METEOR carry implicit assumptions about tokenization, normalization, and edge-case handling that are notoriously underspecified. A formally verified proof that a WFA computes a particular rational power series is only as meaningful as the specification's fidelity to practitioner intent. The project's coverage result—21/31 common metrics are WFA-representable—raises the complementary question of what safety guarantees exist for the remaining 10 metrics, and whether the boundary is principled or accidental.

The trust model of the certification pipeline deserves particular scrutiny. The verifier receives a STARK proof and must decide whether to trust the claimed score. But what exactly has been proven? That a circuit ran correctly, that the circuit implements a WFA, that the WFA corresponds to a formal specification, or that the formal specification matches the "true" metric? Each link in this chain has a different assurance level, and the current presentation does not always distinguish between them clearly.

## Strengths

1. **Addresses a genuine gap in ML evaluation integrity.** The problem of benchmark score integrity is well-motivated: recent work on data contamination, benchmark gaming, and irreproducible evaluations all point to the need for verifiable scoring. Spectacles is, to my knowledge, the first system to provide cryptographic certificates for NLP metric computation. This positions the work at the frontier of ML safety infrastructure, where trust in evaluation is as important as trust in training.

2. **End-to-end pipeline with practical proof artifacts.** The system does not stop at formalization—it produces concrete, independently verifiable STARK proofs. The 270 KiB proof size and 1.5ms verification time for BLEU-4 are practically deployable. The fact that 76 proofs verify at 128-bit security demonstrates engineering maturity. This is not a toy system; it is a working prototype that could be integrated into evaluation pipelines.

3. **Principled metric coverage characterization.** The use of Schützenberger's theorem to classify which metrics admit WFA representations is a principled approach to understanding the scope of the method. Rather than ad hoc case-by-case arguments, the rational power series framework provides a uniform criterion. The 21/31 figure is honest and informative—it tells practitioners exactly which metrics can and cannot be certified by this approach.

4. **Bug-finding as a safety dividend.** The 2 real mathematical bugs discovered during formalization illustrate a point often made in the formal methods community: the process of specification is itself a safety intervention. Even if the verification is incomplete, the discipline of writing formal specifications surfaces errors that would otherwise persist undetected in widely-used metric implementations. This strengthens the safety case for the approach.

## Weaknesses

1. **Specification fidelity is the unexamined assumption.** The entire certification pipeline assumes that the Lean formal specification correctly captures the intended metric semantics. But NLP metrics are notoriously underspecified: BLEU has at least 4 common variants (with different smoothing, brevity penalties, and tokenization); ROUGE has configurable stemming and stopword removal; many metrics have ambiguous behavior on empty inputs or degenerate cases. The paper does not systematically address how specification ambiguity is resolved, or whether the WFA formalization corresponds to any particular reference implementation. A "correct" proof of the wrong specification provides false assurance—a dangerous outcome from a safety perspective.

2. **The trust model conflates distinct assurance levels.** The E2E certification claim bundles together several distinct guarantees: (a) the STARK proof verifies circuit execution, (b) the circuit implements the WFA correctly, (c) the WFA corresponds to the Lean specification, and (d) the Lean specification captures the metric's mathematical definition. Levels (a) and (b) are cryptographically and computationally verifiable, but (c) is validated only by differential testing and (d) is an unverifiable assumption about specification intent. The system's safety guarantees are bounded by the weakest link, and that link—specification fidelity—is essentially unaddressed.

3. **Coverage limitations erode the practical safety case.** Only 21 of 31 common NLP metrics are WFA-representable, meaning nearly a third of the metrics in common use cannot be certified by this system. From a safety perspective, this is problematic: if an evaluation pipeline uses even one non-certifiable metric alongside certified ones, the overall integrity guarantee is broken. The project does not propose fallback mechanisms (e.g., approximate certification, semantic-preserving metric substitution, or hybrid verification) for the non-WFA metrics, leaving a significant gap in practical deployability.

4. **The "machine-checkable" claim requires significant qualification.** With 15 `sorry`s in the Lean formalization and an unverified Lean-to-Rust compilation step, the system does not deliver fully machine-checked guarantees. For a safety-critical application—and verifiable ML evaluation is arguably safety-adjacent—this level of incompleteness is concerning. The differential testing (67,518 triple-agreement checks) provides probabilistic assurance but not the deterministic guarantee that "machine-checkable" implies. The terminology should be revised to reflect the actual assurance level: "partially mechanized" or "machine-assisted" would be more accurate.

## Questions for Authors

- How do you handle specification ambiguity for metrics with multiple common variants (e.g., BLEU with different smoothing methods)? Does the Lean specification commit to one variant, and if so, how was that choice validated against practitioner expectations? Is there a mechanism for parameterizing the specification over variant choices?

- What is the threat model for a malicious prover? Specifically, if the prover controls the WFA compilation step, could they substitute a different (but similarly-structured) WFA that produces a desired score, and would the STARK proof still verify? What prevents a "specification swap" attack?

- Have you considered the implications of partial coverage for safety-critical evaluation scenarios? For instance, if a regulatory body required certified evaluation of an LLM, and the evaluation suite included both WFA-representable and non-representable metrics, what would Spectacles' recommendation be?

## Overall Assessment

Spectacles is a creative and technically ambitious system that addresses a real need in ML evaluation integrity. The automata-theoretic framing is principled, the STARK proof pipeline is practical, and the bug-finding results demonstrate immediate value. However, the safety guarantees are weaker than the presentation suggests: specification fidelity is assumed rather than verified, the trust model conflates distinct assurance levels, and coverage gaps limit practical deployability. The project would benefit from a rigorous treatment of specification validation (perhaps through stakeholder-driven specification review or conformance testing against multiple reference implementations) and a clearer articulation of the actual assurance boundary. It is a promising direction with significant work remaining before the "machine-checkable" claim can be taken at face value.

**Score: 7/10**
