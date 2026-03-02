# Spectacles: Machine-Checkable Proofs for NLP Benchmark Scoring

**Reviewer:** Cesar Sanchez (formal_verification_AI_researcher)

## Summary

Spectacles proposes an ambitious end-to-end pipeline that compiles NLP evaluation metrics into weighted finite automata over typed semirings and then generates STARK zero-knowledge proofs certifying score correctness. The formal backbone rests on Schützenberger's rational power series, providing an algebraically clean foundation for characterizing which string metrics admit WFA representations. The reported results—24/24 STARK proofs verified, 67,518 triple-agreement checks with zero disagreements, and practical proof sizes (270 KiB for 400-state BLEU-4)—are impressive engineering achievements.

However, from a formal verification standpoint, there are substantial gaps between the "machine-checkable" branding and the actual guarantees delivered. The 15 remaining Lean `sorry`s represent unproven obligations in the formalization, and the Lean-to-Rust extraction gap means the pipeline's end-to-end correctness is not, in fact, machine-checked. The system proves that a STARK circuit was executed faithfully, but the relationship between that circuit and the mathematical specification of the metric requires trust in unverified compilation steps.

The project sits at a fascinating intersection of formal methods, automata theory, and cryptographic proof systems. The intellectual contribution—framing NLP metric verification as a problem in rational power series theory—is genuinely novel. But the verification claims need to be stated with considerably more precision to avoid overpromising.

## Strengths

1. **Novel problem formulation via rational power series.** Grounding NLP metric verification in Schützenberger's theory of recognizable formal power series is a theoretically elegant choice. This provides a principled characterization of which metrics are WFA-representable (the 21/31 coverage figure) and connects the verification problem to a well-studied algebraic framework. The characterization is not ad hoc—it follows from deep structure in formal language theory.

2. **Comprehensive empirical validation through triple agreement.** The 67,518 triple-agreement checks comparing the Lean specification, the Rust WFA engine, and reference Python implementations provide strong differential testing evidence. Zero disagreements across this volume of test cases is a meaningful empirical signal, even if it does not constitute formal proof. The methodology is sound as a complement to formal verification.

3. **Practical STARK proof performance.** The reported performance numbers—3,821ms prove time, 1.5ms verify time, 270 KiB proof size for 400-state BLEU-4—demonstrate that the approach is not merely theoretical. These numbers are within practical bounds for real deployment scenarios, and the 128-bit security level is standard for cryptographic applications. The 76 verified STARK proofs at this security level represent a solid engineering effort.

4. **Discovery of real mathematical bugs.** The fact that the formalization process uncovered 2 genuine mathematical bugs in metric implementations is a compelling argument for the value of formal specification, independent of the completeness of the verification. This demonstrates the practical utility of the approach even in its current, incomplete state.

## Weaknesses

1. **The 15 Lean `sorry`s undermine the "machine-checkable" claim.** Each `sorry` is an axiom injected into the proof kernel—any one of them could introduce inconsistency. Without knowing the precise nature of these obligations, calling the proofs "machine-checkable" is misleading. Some `sorry`s may indeed be routine (e.g., decidability instances for concrete types), but others may hide genuinely difficult proof obligations. The paper should categorize each `sorry` by difficulty and potential impact on soundness. Until they are discharged, the Lean formalization is a partial formalization, not a complete one.

2. **The Lean-to-Rust gap is a fundamental soundness issue.** The STARK proofs verify execution of Rust-compiled circuits, but the Lean formalization specifies the mathematical semantics. The connection between the two is validated only by differential testing, not by verified extraction or a machine-checked bisimulation. This is not a minor engineering gap—it is the central soundness question. A bug in the WFA compilation from Lean semantics to Rust circuits would invalidate the entire certification pipeline, and differential testing cannot guarantee absence of such bugs.

3. **STARK proofs attest to circuit execution, not metric correctness.** There is a subtle but critical distinction: the STARK proof certifies that a particular arithmetic circuit was executed correctly on given inputs. It does not, by itself, certify that the circuit computes the intended NLP metric. The gap between "circuit fidelity" and "metric correctness" is bridged only by trust in the compilation pipeline, which, as noted above, is not formally verified end-to-end. The claim of "machine-checkable proofs of score correctness" conflates these two distinct guarantees.

4. **Triple agreement is not a substitute for formal verification.** While 67,518 zero-disagreement checks are encouraging, differential testing is fundamentally limited: it can find bugs but cannot prove their absence. If all three implementations share a common misunderstanding of the metric specification (e.g., an ambiguous edge case in BLEU tokenization), triple agreement would not detect the error. The reliance on triple agreement as a primary validation mechanism for the compilation pipeline weakens the formal verification narrative.

## Questions for Authors

- Can you provide a detailed categorization of the 15 remaining `sorry`s, including (a) the specific proof obligation, (b) your assessment of difficulty, and (c) whether each `sorry` appears in a lemma that is on the critical path for the end-to-end correctness theorem? How many are in the WFA-to-circuit compilation correctness proof versus the metric-to-WFA compilation?

- What is your threat model for the Lean-to-Rust gap? Specifically, have you considered producing a verified compiler or extraction mechanism (e.g., via CakeML-style verified compilation or Lean 4's native code generation) to close this gap, and what are the technical obstacles?

- The 10 metrics that are not WFA-representable—can you characterize precisely why they fail (e.g., which algebraic property of the required semiring is violated), and do any of them admit approximations within the WFA framework with bounded error guarantees?

## Overall Assessment

Spectacles presents a genuinely novel and intellectually ambitious approach to certifying NLP evaluation scores, with solid automata-theoretic foundations and practical STARK proof performance. However, the verification story has significant gaps: the 15 `sorry`s, the unverified Lean-to-Rust compilation, and the conflation of circuit execution fidelity with metric correctness all weaken the "machine-checkable" claim. The project would benefit from a more precise articulation of exactly what is proven and what remains trusted, along with a roadmap for closing the verification gaps. As it stands, Spectacles is a strong contribution to the intersection of formal methods and NLP evaluation, but it does not yet deliver on the full promise of machine-checked correctness.

**Score: 6/10**
