# Metric Variant Specifications

This document specifies exactly which variant of each NLP metric is implemented
in Spectacles, addressing specification ambiguity concerns.

## Implemented Metrics

### 1. Exact Match
- **Variant**: Case-sensitive string equality after whitespace normalization
- **Normalization**: Leading/trailing whitespace stripped, internal whitespace collapsed
- **Reference**: SQuAD evaluation script (Rajpurkar et al., 2016)
- **WFA**: Boolean semiring, linear-chain DFA

### 2. Token F1
- **Variant**: Unigram F1 with bag-of-words matching
- **Tokenizer**: Whitespace tokenizer (configurable)
- **Duplicate handling**: Multi-set intersection (each token counted up to its minimum frequency)
- **F1 formula**: 2 × precision × recall / (precision + recall), with 0/0 = 0
- **Reference**: SQuAD evaluation script
- **WFA**: Counting semiring, separate WFAs for TP, |candidate|, |reference|

### 3. BLEU-4
- **Variant**: Corpus BLEU with add-1 smoothing (Chen & Cherry, 2014)
- **N-gram order**: 4 (configurable via `BleuConfig`)
- **Brevity penalty**: Standard exponential BP = min(1, exp(1 - r/c))
- **Smoothing**: Add-1 to all n-gram counts (configurable: None, Add1, AddK, Exponential)
- **Reference**: SacreBLEU (Post, 2018) default settings
- **WFA**: Counting semiring, separate WFA per n-gram order

### 4. ROUGE-1
- **Variant**: Unigram recall-oriented overlap
- **Formula**: ROUGE-1 = |unigrams(candidate) ∩ unigrams(reference)| / |unigrams(reference)|
- **Reference**: ROUGE package (Lin, 2004), `rouge_score` Python library
- **WFA**: Counting semiring

### 5. ROUGE-L
- **Variant**: Longest Common Subsequence (LCS) based, sentence-level
- **Formula**: R_lcs = LCS(c,r)/|r|, P_lcs = LCS(c,r)/|c|, F_lcs = (1+β²)R·P/(R+β²P) with β=1
- **Reference**: ROUGE package, `rouge_score` Python library
- **WFA**: Tropical semiring (min, +), Tier 2 gadget-assisted compilation
- **Key property**: Unambiguous WFA for fixed reference (Proposition 4.1)

### 6. ChrF
- **Variant**: Character n-gram F-score, β=2 (ChrF++)
- **N-gram orders**: 1-6 character n-grams (configurable)
- **Word n-grams**: 0-2 word n-grams when using ChrF++
- **Reference**: Popović (2015), `sacrebleu` implementation
- **WFA**: Counting semiring, separate WFA per n-gram order

### 7. Edit Distance (Levenshtein)
- **Variant**: Standard Levenshtein distance with unit costs
- **Operations**: Insertion (cost 1), deletion (cost 1), substitution (cost 1)
- **Normalization**: Raw distance (normalized variant available via division)
- **Reference**: Standard DP formulation
- **WFA**: Tropical semiring, alignment automaton

## Metrics Analyzed but Not Implemented

| Metric | Reason | Semiring | Feasibility |
|--------|--------|----------|-------------|
| METEOR | Requires WordNet synonymy (infinite state) | N/A | Not WFA-representable |
| BERTScore | Neural embedding similarity | N/A | Not WFA-representable |
| COMET | Neural regression model | N/A | Not WFA-representable |
| BLEURT | Fine-tuned neural model | N/A | Not WFA-representable |
| Perplexity | Requires neural LM | N/A | Not WFA-representable |
| MAUVE | Distributional divergence | N/A | Not WFA-representable |
| CodeBLEU | Requires AST parsing | N/A | Partially WFA (lexical) |
| TER | Shift operations over phrases | Tropical | Feasible but complex |
| NIST | Information-weighted n-gram | Real | Feasible |
| Token Accuracy | Per-position accuracy | Boolean | Trivially WFA |

## Coverage Analysis

- **31 NLP metrics surveyed** (see `data/wfa_coverage_census.json`)
- **21 WFA-representable** (68% coverage)
- **7 fully implemented** with triple verification
- **10 not WFA-representable** (require neural models, infinite state, or non-regular operations)

The 21/31 coverage is rigorously justified: each non-WFA metric requires either:
1. Neural model evaluation (BERTScore, BLEURT, COMET, MAUVE, Perplexity) — inherently non-algebraic
2. External knowledge bases (METEOR synonymy) — infinite alphabet
3. Context-free or context-sensitive operations (CodeBLEU AST) — beyond regular
