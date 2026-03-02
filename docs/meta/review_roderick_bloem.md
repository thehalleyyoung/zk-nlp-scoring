# Review: Spectacles — A Verified Compiler from Semiring-Weighted Automata to Zero-Knowledge Scoring Circuits

**Reviewer:** Roderick Bloem
**Persona:** Machine Learning Specification and Safety Researcher
**Date:** 2026-03-02

---

## Summary

Spectacles builds a pipeline for generating cryptographic certificates that NLP benchmark scores were computed correctly. The system compiles metrics (BLEU, ROUGE, exact match, etc.) into weighted finite automata over typed semirings, then generates STARK proofs certifying faithful execution. A secondary component uses OPRF-based private set intersection (PSI) to bound training-test contamination without revealing either corpus. The system covers 21 of 31 surveyed NLP metrics (67.7%) and demonstrates proof generation at scale (3.8s prove, 1.5ms verify for 400-state BLEU-4). 68,748 differential tests validate agreement between reference implementations, WFA evaluators, and circuit evaluators.

As a researcher focused on ML evaluation integrity and safety, I approach this work with a core question: does Spectacles address the problems that actually threaten benchmark integrity in practice? The answer is nuanced. The system provides genuine value for a narrow threat model (score fabrication by a dishonest reporter), but the dominant threats to benchmark integrity — data contamination, benchmark gaming through training optimization, overfitting to evaluation distributions, and metric misalignment with human judgment — are either partially addressed or entirely out of scope. The engineering is impressive, but the motivation needs substantial strengthening.

## Strengths

**S1. A real problem deserving attention.** Benchmark integrity is a genuine crisis in modern ML. Leaderboard gaming, cherry-picked results, and unreproducible evaluations are endemic. The idea of generating a tamper-evident certificate for a benchmark score — analogous to an audited financial statement — has practical appeal. Even if the current system doesn't address all threats, establishing the principle that evaluation results should be cryptographically verifiable is valuable for the field.

**S2. Practical STARK performance.** The verification overhead is surprisingly low: 1.5ms verification time and 270 KiB proof size for a 400-state BLEU-4 WFA. This is fast enough for real-time verification and small enough for on-chain attestation. The O(n²) scaling in state count is a concern for complex metrics, but the sub-4ms verification at all scales means that the cost is borne by the prover (who is trying to earn trust) rather than the verifier (who needs to check quickly). This asymmetry aligns well with the trust model's incentive structure.

**S3. Multi-layer contamination detection improves over n-gram baselines.** The data reports 100% accuracy for multi-layer contamination detection versus 83.3% for n-gram-only approaches. While the details of this comparison would benefit from more rigorous evaluation (what is the test distribution? what counts as "accuracy"?), combining n-gram overlap with structural and statistical signals is a sound approach that addresses known weaknesses of pure n-gram contamination detection.

**S4. The coverage census is intellectually honest.** Rather than claiming universal applicability, the project provides a rigorous analysis of which metrics are WFA-representable and which are not, grounded in the algebraic structure of rational power series. The 21/31 figure is derived from first principles (metrics requiring neural models or infinite state are provably non-WFA), not from implementation limitations. This kind of principled scope definition is rare and commendable.

**S5. End-to-end demonstration with real metric implementations.** The system isn't a theoretical construction — it implements 7 metrics with full triple verification, generates and verifies real STARK proofs, and demonstrates on realistic input pairs (205 pairs across 5 categories: MMLU-style, SQuAD-style, translation, code completion, edge cases). The comprehensive benchmark producing 1,230 checks with zero disagreements is credible evidence that the implementation works correctly on the tested inputs.

## Weaknesses

**W1. The threat model addresses the wrong problem.** The most pressing integrity threats in NLP benchmarking are not score fabrication — they are benchmark gaming, contamination, and metric misalignment. Consider the typical scenario: a company reports that their model achieves 92.3% on MMLU. The threat is not that they computed 92.3% incorrectly from genuine test answers; the threat is that the model was trained (directly or indirectly) on MMLU questions, that the reported number is cherry-picked from many runs, or that MMLU scores don't reflect the capabilities users care about.

Spectacles' G1 guarantee (score integrity via STARK) certifies that the arithmetic was done correctly. But nobody seriously doubts the arithmetic. The fraud occurs upstream: in data selection, training pipeline configuration, and selective reporting. A cryptographic proof that `mean(correct_answers) = 0.923` is true does not help if the correct_answers vector was computed on contaminated data.

The G2 guarantee (contamination via PSI) partially addresses this, but only for n-gram-level contamination. A model can be contaminated through paraphrasing, reasoning chain exposure, or distribution-level overlap that n-gram PSI cannot detect. The system's contamination detection operates at the surface form level, missing the deeper forms of contamination that are most concerning in practice.

**W2. 67.7% metric coverage excludes the metrics that matter most today.** The 21/31 WFA coverage figure sounds impressive, but the excluded metrics include BERTScore, COMET, BLEURT, and perplexity — precisely the metrics that the ML community is moving toward because of known weaknesses in string-matching metrics like BLEU and ROUGE. The trend in NLP evaluation is away from n-gram overlap and toward learned metrics that correlate better with human judgment. Spectacles is building verified infrastructure for the metrics that the community is moving away from.

This is not merely a current limitation to be addressed in future work. The excluded metrics are provably non-WFA-representable because they require neural network evaluation. No extension of the WFA framework can capture BERTScore without fundamentally changing the architecture. The project is therefore building on a foundation (WFA over semirings) that has an inherent ceiling on the metrics it can support, and that ceiling excludes the future direction of NLP evaluation.

**W3. The prove-time overhead limits practical deployment.** While 1.5ms verification is fast, 3.8s proof generation for a single 400-state BLEU-4 WFA is substantial in the context of ML evaluation pipelines. A typical evaluation run might compute 10 metrics on 10,000 examples, requiring 100,000 proofs. At 3.8 seconds each, that's over 4 days of proof generation — compared to seconds for the actual metric computation. Even with parallelization and smaller WFAs (most metrics use < 100 states), the overhead is orders of magnitude larger than the computation being certified.

The scaling model (`prove_time = 0.017 × n^2.06`) means that more complex metrics (larger WFA state spaces) face even worse overhead. BLEU-4 with full trie enumeration for vocabulary size 50,000 would have millions of states, making STARK proof generation infeasible. The practical system presumably uses the "flat" n-gram construction with much smaller state counts, but this raises questions about what exactly the WFA represents when it compresses a 50,000-vocabulary BLEU computation into a few hundred states.

**W4. The system doesn't address selective reporting.** Even with perfect score integrity and contamination detection, a malicious actor can run the evaluation 100 times with different random seeds, hyperparameters, or data splits, and report only the best result. This "file drawer problem" is a major source of inflated benchmark numbers in practice. Spectacles certifies individual evaluation runs but provides no mechanism for attesting that a reported score is representative rather than cherry-picked. A comprehensive evaluation integrity system would need to address selective reporting, perhaps through pre-registration of evaluation protocols or commitment to evaluation parameters before seeing results.

**W5. Metric variant ambiguity undermines the certification claim.** The `METRIC_VARIANTS.md` documentation reveals the depth of the specification problem: BLEU alone has multiple smoothing options (None, Add-1, Add-k, Exponential), tokenization choices (whitespace vs. BPE vs. character-level), and corpus-level vs. sentence-level computation. When Spectacles certifies "BLEU = 0.423," the certificate is bound to a specific variant defined by the EvalSpec configuration. But the ML community routinely compares BLEU scores computed with different variants, tokenizers, and normalization schemes. The SacreBLEU project exists precisely because of this problem.

A certificate that says "this specific BLEU variant on this specific tokenization produced this score" is technically correct but practically insufficient if the recipient doesn't know or verify that the variant matches their expectations. The metric specification itself is part of the trust chain, and the system provides no mechanism for the verifier to inspect or validate that the metric specification matches a community-agreed standard.

**W6. The contamination PSI protocol has significant deployment barriers.** The commit-then-execute protocol requires the model provider to (a) commit to their training n-gram set before the protocol begins, (b) complete the PSI protocol, and (c) reveal C_train post-protocol for verification. Requirement (c) is particularly problematic: revealing the training corpus's n-gram set exposes proprietary training data. The PSI_SECURITY_ANALYSIS acknowledges this limitation but doesn't resolve it. In practice, companies like OpenAI, Anthropic, and Google would be unwilling to reveal their training corpora's n-gram fingerprints to third-party evaluators. Without a zero-knowledge proof of commitment consistency (acknowledged as future work), the contamination detection component is deployable only in cooperative, non-adversarial settings — precisely the settings where it's least needed.

Additionally, the absence of VOPRF means a sophisticated provider can selectively corrupt OPRF evaluations to suppress contamination matches. The "spot-check" defense (including known test elements) catches random corruption but not targeted suppression of specific contaminated n-grams.

## Minor Issues

**M1. The comprehensive benchmark categories seem arbitrary.** The 5 categories (MMLU-style, SQuAD-style, translation, code completion, edge cases) with 25–50 pairs each are too small for statistical significance. Performance characteristics of STARK proofs may vary with input distribution, and 205 total pairs cannot adequately characterize this variation.

**M2. The "bugs found" framing is misleading.** The 2 bugs (Montgomery inverse, Lagrange interpolation) were in the Rust implementation, not in the mathematical specification. They were found by careful code review and manual verification, not by the Lean formalization failing to prove a theorem. The formalization didn't "find" these bugs — a human auditor did while verifying the implementation against the specification. This is still valuable, but attributing the bug-finding to the formal verification process overstates the case.

**M3. Model identity attestation is punted to hardware.** The docs mention TEE-based model identity attestation, but this is described as "protocol-level design" requiring production hardware. Without model identity guarantees, the entire certification chain is vulnerable to model swapping: a provider could run the evaluation on a different (better) model than the one they claim to be certifying.

**M4. No comparison to simpler alternatives.** The paper doesn't compare against simpler approaches to evaluation integrity: (a) reproducibility via deterministic evaluation with published code and data, (b) third-party evaluation by trusted auditors (e.g., HELM), (c) hash-based attestation of inputs and outputs without ZK machinery. It's possible that the ZK overhead provides little incremental value over these simpler approaches for the use cases where Spectacles is deployable.

**M5. The distinction between corpus-level and sentence-level BLEU is important.** The metric variants document specifies "Corpus BLEU" but the demonstration uses 205 pairs with per-pair scores. Corpus BLEU is computed over the entire test set, not per-sentence. If the WFA computes sentence-level BLEU scores that are then averaged, this is not the same as corpus-level BLEU (which uses corpus-level precision counts). The certificate should specify exactly which aggregation is being certified.

## Questions for Authors

1. **Who is the intended user?** In a concrete deployment scenario, who generates the proof, who verifies it, and what action does the verifier take based on the result? If the answer is "leaderboard maintainers verify submissions," how does this interact with the current practice of self-reported results?

2. **What is the cost-benefit of STARK proofs vs. reproducibility?** If I can reproduce your evaluation by running your published code on your published data with your published model weights, what additional assurance does a STARK proof provide? The ZK property (hiding the input) is relevant for proprietary test sets, but most major benchmarks are public.

3. **Can you quantify the false negative rate of the contamination detector?** Specifically, what fraction of paraphrase-level or reasoning-chain-level contamination does the n-gram PSI approach miss? The 100% accuracy figure is on what test distribution?

4. **How does the system handle streaming or incremental evaluation?** Modern evaluation pipelines often compute metrics incrementally as examples are processed. Does the STARK proof require the entire evaluation trace, or can proofs be composed?

5. **What prevents a provider from generating a valid STARK proof for a score computed on a cherry-picked subset of test examples?** The certificate binds to a committed input, but who verifies that the committed input is the complete, correct test set?

6. **Given the O(n²) scaling, what is the practical limit on WFA state count?** The paper demonstrates up to 1,024 states (~50s prove). What state counts do realistic metrics actually require? Is 1,024 states sufficient for production BLEU-4 with standard vocabulary sizes?

## Recommendation

**Weak Accept.** Spectacles demonstrates strong engineering and a principled algebraic approach to a real problem. The STARK performance numbers are encouraging, and the coverage analysis is honest. However, the system addresses score fabrication — a minor threat in practice — while the dominant threats (contamination beyond n-gram level, selective reporting, benchmark gaming, metric misalignment) are either partially addressed or out of scope. The 67.7% coverage ceiling excludes the neural metrics that represent the future direction of NLP evaluation. The contamination PSI protocol faces deployment barriers that limit it to cooperative settings. The work should be accepted for its technical contributions (WFA compilation, semiring-parametric circuits, practical STARK proofs) but the motivation and impact claims should be substantially revised to reflect the narrow threat model that the system actually addresses.

**Confidence:** 4/5
