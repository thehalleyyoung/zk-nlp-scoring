//! Spectacles CLI - command-line interface for the Spectacles framework.
//!
//! Subcommands: score, batch-score, differential-test, compile-circuit,
//!              estimate-size, hash, verify-proof

use std::fs;
use spectacles_core::scoring::{
    self, ScoringPair, TripleMetric,
    exact_match::ExactMatchScorer,
    token_f1::TokenF1Scorer,
    bleu::{BleuScorer, SmoothingMethod},
    rouge::{RougeNScorer, RougeLScorer},
    differential::{DifferentialTester, standard_test_suite, random_test_pairs},
};
use spectacles_core::utils::{
    hash::SpectaclesHasher,
    serialization::{CompactProof, ProofHeader, estimate_proof_size},
};
use serde::{Serialize, Deserialize};

/// Available metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetricType {
    ExactMatch,
    TokenF1,
    Bleu,
    Rouge1,
    Rouge2,
    RougeL,
}

impl MetricType {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "exact_match" | "exact-match" | "em" => Some(Self::ExactMatch),
            "token_f1" | "token-f1" | "f1" => Some(Self::TokenF1),
            "bleu" => Some(Self::Bleu),
            "rouge1" | "rouge-1" | "rouge_1" => Some(Self::Rouge1),
            "rouge2" | "rouge-2" | "rouge_2" => Some(Self::Rouge2),
            "rougel" | "rouge-l" | "rouge_l" => Some(Self::RougeL),
            _ => None,
        }
    }
    
    fn all() -> Vec<&'static str> {
        vec!["exact_match", "token_f1", "bleu", "rouge1", "rouge2", "rougel"]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::ExactMatch => "exact_match",
            Self::TokenF1 => "token_f1",
            Self::Bleu => "bleu",
            Self::Rouge1 => "rouge1",
            Self::Rouge2 => "rouge2",
            Self::RougeL => "rougel",
        }
    }
}

/// Output format for score/batch-score commands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
    Csv,
    Markdown,
}

impl OutputFormat {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "text" | "txt" => Some(Self::Text),
            "json" => Some(Self::Json),
            "csv" => Some(Self::Csv),
            "markdown" | "md" | "markdown-table" => Some(Self::Markdown),
            _ => None,
        }
    }
}

/// A single score result row for structured output
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScoreRow {
    metric: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    candidate: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
    score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    precision: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    recall: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    brevity_penalty: Option<f64>,
}

/// Parse a comma-separated list of metrics
fn parse_metrics(s: &str) -> Result<Vec<MetricType>, String> {
    s.split(',')
        .map(|m| {
            MetricType::from_str(m.trim())
                .ok_or_else(|| format!("Unknown metric: '{}'. Available: {:?}", m.trim(), MetricType::all()))
        })
        .collect()
}

/// Score a single pair with a given metric, returning a ScoreRow
fn score_pair(metric: MetricType, pair: &ScoringPair) -> ScoreRow {
    match metric {
        MetricType::ExactMatch => {
            let scorer = ExactMatchScorer::case_insensitive();
            let s = scorer.score_reference(pair);
            ScoreRow {
                metric: metric.name().to_string(),
                index: None, candidate: None, reference: None,
                score: if s { 1.0 } else { 0.0 },
                precision: None, recall: None, brevity_penalty: None,
            }
        }
        MetricType::TokenF1 => {
            let scorer = TokenF1Scorer::default_scorer();
            let s = scorer.score_reference(pair);
            ScoreRow {
                metric: metric.name().to_string(),
                index: None, candidate: None, reference: None,
                score: s.f1,
                precision: Some(s.precision), recall: Some(s.recall),
                brevity_penalty: None,
            }
        }
        MetricType::Bleu => {
            let scorer = BleuScorer::with_smoothing(SmoothingMethod::Add1);
            let s = scorer.score_reference(pair);
            ScoreRow {
                metric: metric.name().to_string(),
                index: None, candidate: None, reference: None,
                score: s.score,
                precision: None, recall: None,
                brevity_penalty: Some(s.brevity_penalty),
            }
        }
        MetricType::Rouge1 => {
            let scorer = RougeNScorer::rouge1();
            let s = scorer.score_reference(pair);
            ScoreRow {
                metric: metric.name().to_string(),
                index: None, candidate: None, reference: None,
                score: s.f1,
                precision: Some(s.precision), recall: Some(s.recall),
                brevity_penalty: None,
            }
        }
        MetricType::Rouge2 => {
            let scorer = RougeNScorer::rouge2();
            let s = scorer.score_reference(pair);
            ScoreRow {
                metric: metric.name().to_string(),
                index: None, candidate: None, reference: None,
                score: s.f1,
                precision: Some(s.precision), recall: Some(s.recall),
                brevity_penalty: None,
            }
        }
        MetricType::RougeL => {
            let scorer = RougeLScorer::default_scorer();
            let s = scorer.score_reference(pair);
            ScoreRow {
                metric: metric.name().to_string(),
                index: None, candidate: None, reference: None,
                score: s.f1,
                precision: Some(s.precision), recall: Some(s.recall),
                brevity_penalty: None,
            }
        }
    }
}

/// Emit score rows in the chosen output format
fn emit_rows(rows: &[ScoreRow], format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            for row in rows {
                let idx = row.index.map(|i| format!("[{}] ", i)).unwrap_or_default();
                let mut parts = vec![format!("{}{}  score={:.4}", idx, row.metric, row.score)];
                if let Some(p) = row.precision { parts.push(format!("P={:.4}", p)); }
                if let Some(r) = row.recall { parts.push(format!("R={:.4}", r)); }
                if let Some(bp) = row.brevity_penalty { parts.push(format!("BP={:.4}", bp)); }
                println!("{}", parts.join("  "));
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(rows).unwrap();
            println!("{}", json);
        }
        OutputFormat::Csv => {
            println!("metric,index,score,precision,recall,brevity_penalty");
            for row in rows {
                println!("{},{},{:.4},{},{},{}",
                    row.metric,
                    row.index.map(|i| i.to_string()).unwrap_or_default(),
                    row.score,
                    row.precision.map(|v| format!("{:.4}", v)).unwrap_or_default(),
                    row.recall.map(|v| format!("{:.4}", v)).unwrap_or_default(),
                    row.brevity_penalty.map(|v| format!("{:.4}", v)).unwrap_or_default(),
                );
            }
        }
        OutputFormat::Markdown => {
            println!("| metric | index | score | precision | recall | BP |");
            println!("|--------|-------|-------|-----------|--------|----|");
            for row in rows {
                println!("| {} | {} | {:.4} | {} | {} | {} |",
                    row.metric,
                    row.index.map(|i| i.to_string()).unwrap_or_default(),
                    row.score,
                    row.precision.map(|v| format!("{:.4}", v)).unwrap_or_default(),
                    row.recall.map(|v| format!("{:.4}", v)).unwrap_or_default(),
                    row.brevity_penalty.map(|v| format!("{:.4}", v)).unwrap_or_default(),
                );
            }
        }
    }
}

/// Load scoring pairs from a file, auto-detecting format by extension.
/// Supports: .json (array, HuggingFace datasets, SQuAD), .jsonl, .csv, .tsv.
fn load_pairs(path: &str) -> Result<Vec<ScoringPair>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path, e))?;

    if path.ends_with(".csv") {
        load_pairs_csv(&content)
    } else if path.ends_with(".tsv") {
        load_pairs_tsv(&content)
    } else if path.ends_with(".jsonl") {
        load_pairs_jsonl(&content)
    } else {
        // Try JSON: ScoringPair array, HuggingFace datasets, SQuAD, then JSONL fallback
        serde_json::from_str::<Vec<ScoringPair>>(&content)
            .or_else(|_| load_pairs_hf_json(&content))
            .or_else(|_| load_pairs_squad_json(&content))
            .or_else(|_| load_pairs_jsonl(&content))
            .map_err(|e| format!("Failed to parse input as JSON or JSONL: {}", e))
    }
}

fn load_pairs_csv(content: &str) -> Result<Vec<ScoringPair>, String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(content.as_bytes());

    let headers = rdr.headers()
        .map_err(|e| format!("CSV header error: {}", e))?
        .clone();

    // Find column indices (support reference/hypothesis or reference/candidate)
    let ref_idx = headers.iter().position(|h| {
        let h = h.trim().to_lowercase();
        h == "reference" || h == "ref"
    }).ok_or_else(|| "CSV must have a 'reference' column".to_string())?;

    let hyp_idx = headers.iter().position(|h| {
        let h = h.trim().to_lowercase();
        h == "hypothesis" || h == "candidate" || h == "hyp"
    }).ok_or_else(|| "CSV must have a 'hypothesis' or 'candidate' column".to_string())?;

    let mut pairs = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| format!("CSV parse error: {}", e))?;
        let reference = record.get(ref_idx).unwrap_or("").to_string();
        let candidate = record.get(hyp_idx).unwrap_or("").to_string();
        pairs.push(ScoringPair { candidate, reference });
    }
    Ok(pairs)
}

fn load_pairs_jsonl(content: &str) -> Result<Vec<ScoringPair>, String> {
    content.lines()
        .filter(|line| !line.trim().is_empty())
        .enumerate()
        .map(|(i, line)| {
            serde_json::from_str::<ScoringPair>(line)
                .or_else(|_| hf_object_to_pair(
                    &serde_json::from_str::<serde_json::Value>(line)
                        .map_err(|e| format!("JSONL parse error on line {}: {}", i + 1, e))?
                ).ok_or_else(|| format!("JSONL line {} missing reference/candidate fields", i + 1)))
                .map_err(|e| format!("JSONL parse error on line {}: {}", i + 1, e))
        })
        .collect()
}

/// Extract a ScoringPair from a JSON object with HuggingFace-style field names.
/// Supports: reference/label/target → reference, candidate/prediction/generated/hypothesis → candidate.
fn hf_object_to_pair(obj: &serde_json::Value) -> Option<ScoringPair> {
    let map = obj.as_object()?;
    let reference = ["reference", "ref", "label", "target"]
        .iter()
        .find_map(|k| map.get(*k).and_then(|v| v.as_str()))?
        .to_string();
    let candidate = ["candidate", "hypothesis", "hyp", "prediction", "generated"]
        .iter()
        .find_map(|k| map.get(*k).and_then(|v| v.as_str()))?
        .to_string();
    Some(ScoringPair { candidate, reference })
}

/// Load pairs from HuggingFace datasets JSON format.
/// Accepts `{"data": [...]}` or a bare array of objects with flexible field names.
fn load_pairs_hf_json(content: &str) -> Result<Vec<ScoringPair>, String> {
    let value: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let items = if let Some(arr) = value.as_array() {
        arr.clone()
    } else if let Some(obj) = value.as_object() {
        obj.get("data")
            .and_then(|v| v.as_array())
            .cloned()
            .ok_or_else(|| "Expected 'data' array in HF dataset JSON".to_string())?
    } else {
        return Err("Expected JSON array or object with 'data' key".to_string());
    };

    items.iter()
        .enumerate()
        .map(|(i, item)| {
            hf_object_to_pair(item)
                .ok_or_else(|| format!("HF JSON item {} missing reference/candidate fields", i))
        })
        .collect()
}

/// Load pairs from SQuAD-style JSON: `{"data": [{"paragraphs": [{"qas": [...]}]}]}`.
/// Extracts (reference=first answer text, candidate=question) pairs.
fn load_pairs_squad_json(content: &str) -> Result<Vec<ScoringPair>, String> {
    let value: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let data = value.get("data")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "SQuAD JSON must have a 'data' array".to_string())?;

    let mut pairs = Vec::new();
    for article in data {
        let paragraphs = article.get("paragraphs")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "SQuAD article missing 'paragraphs'".to_string())?;
        for para in paragraphs {
            let qas = para.get("qas")
                .and_then(|v| v.as_array())
                .ok_or_else(|| "SQuAD paragraph missing 'qas'".to_string())?;
            for qa in qas {
                let question = qa.get("question")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let answer = qa.get("answers")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|a| a.get("text"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                // Use prediction field if present (for evaluated SQuAD), else question
                let candidate = qa.get("prediction")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&question)
                    .to_string();
                pairs.push(ScoringPair { candidate, reference: answer });
            }
        }
    }

    if pairs.is_empty() {
        return Err("No QA pairs found in SQuAD JSON".to_string());
    }
    Ok(pairs)
}

/// Load pairs from TSV (tab-separated values): reference\tcandidate per line.
fn load_pairs_tsv(content: &str) -> Result<Vec<ScoringPair>, String> {
    let mut lines = content.lines().filter(|l| !l.trim().is_empty());

    // Check if first line is a header
    let first = lines.next().ok_or_else(|| "TSV file is empty".to_string())?;
    let mut pairs = Vec::new();
    let first_parts: Vec<&str> = first.split('\t').collect();

    let is_header = first_parts.len() >= 2 && {
        let a = first_parts[0].trim().to_lowercase();
        let b = first_parts[1].trim().to_lowercase();
        ["reference", "ref", "label", "target"].contains(&a.as_str())
            && ["hypothesis", "candidate", "hyp", "prediction", "generated"].contains(&b.as_str())
    };

    if !is_header {
        if first_parts.len() >= 2 {
            pairs.push(ScoringPair {
                reference: first_parts[0].to_string(),
                candidate: first_parts[1].to_string(),
            });
        }
    }

    for line in lines {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            pairs.push(ScoringPair {
                reference: parts[0].to_string(),
                candidate: parts[1].to_string(),
            });
        }
    }
    Ok(pairs)
}

/// CLI entry point
fn main() {
    env_logger::init();
    
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }
    
    let result = match args[1].as_str() {
        "score" => cmd_score(&args[2..]),
        "differential-test" => cmd_differential_test(&args[2..]),
        "compile-circuit" => cmd_compile_circuit(&args[2..]),
        "estimate-size" => cmd_estimate_size(&args[2..]),
        "batch-score" => cmd_batch_score(&args[2..]),
        "hash" => cmd_hash(&args[2..]),
        "verify-proof" => cmd_verify_proof(&args[2..]),
        "help" | "--help" | "-h" => {
            print_usage();
            Ok(())
        }
        "version" | "--version" | "-V" => {
            println!("spectacles-cli 0.1.0");
            Ok(())
        }
        other => {
            eprintln!("Unknown command: {}", other);
            print_usage();
            Err(format!("Unknown command: {}", other))
        }
    };
    
    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn print_usage() {
    println!("spectacles-cli - Spectacles ZK scoring framework");
    println!();
    println!("USAGE:");
    println!("  spectacles-cli <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("  score              Score a candidate against a reference");
    println!("  batch-score        Score multiple pairs from a file (JSON, JSONL, CSV, TSV, HF datasets, SQuAD)");
    println!("  differential-test  Run differential tests across implementations");
    println!("  compile-circuit    Compile a scoring metric to an arithmetic circuit");
    println!("  estimate-size      Estimate proof size for a configuration");
    println!("  hash               Hash input data with BLAKE3");
    println!("  verify-proof       Verify a previously generated proof file");
    println!("  help               Print this help message");
    println!("  version            Print version");
    println!();
    println!("SCORE OPTIONS:");
    println!("  --metric, -m       Metric name (exact_match, token_f1, bleu, rouge1, rouge2, rougel)");
    println!("  --candidate, -c    Candidate text");
    println!("  --reference, -r    Reference text");
    println!("  --triple, -t       Run all three implementations and check agreement");
    println!("  --output-format    Output format: text (default), json, csv, markdown");
    println!();
    println!("BATCH-SCORE OPTIONS:");
    println!("  --input, -i        Input file (JSON, JSONL, CSV, TSV, HF datasets, or SQuAD format)");
    println!("  --metric, -m       Single metric name");
    println!("  --metrics          Comma-separated metric list (e.g. exact_match,bleu,rouge_1)");
    println!("  --output-format    Output format: text (default), json, csv, markdown");
    println!();
    println!("VERIFY-PROOF OPTIONS:");
    println!("  --proof, -p        Path to proof file (.spct or .json)");
    println!();
    println!("EXAMPLES:");
    println!("  spectacles-cli score --metric bleu --candidate \"the cat sat\" --reference \"the cat sat on the mat\"");
    println!("  spectacles-cli score --metric token_f1 -c \"hello world\" -r \"hello there\" --output-format json");
    println!("  spectacles-cli batch-score --input pairs.csv --metrics exact_match,bleu,rouge_1 --output-format markdown");
    println!("  spectacles-cli batch-score --input pairs.jsonl --metric bleu");
    println!("  spectacles-cli differential-test --count 100");
    println!("  spectacles-cli verify-proof --proof result.spct");
    println!("  spectacles-cli estimate-size --constraints 1000 --wires 500");
}

fn cmd_score(args: &[String]) -> Result<(), String> {
    let mut metric_str = "bleu".to_string();
    let mut candidate = String::new();
    let mut reference = String::new();
    let mut triple = false;
    let mut output_format = OutputFormat::Text;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--metric" | "-m" => {
                i += 1;
                if i < args.len() { metric_str = args[i].clone(); }
            }
            "--candidate" | "-c" => {
                i += 1;
                if i < args.len() { candidate = args[i].to_string(); }
            }
            "--reference" | "-r" => {
                i += 1;
                if i < args.len() { reference = args[i].to_string(); }
            }
            "--triple" | "-t" => {
                triple = true;
            }
            "--output-format" => {
                i += 1;
                if i < args.len() {
                    output_format = OutputFormat::from_str(&args[i])
                        .ok_or_else(|| format!("Unknown output format: {}. Use: text, json, csv, markdown", args[i]))?;
                }
            }
            _ => {}
        }
        i += 1;
    }
    
    if candidate.is_empty() || reference.is_empty() {
        return Err("Both --candidate and --reference are required".to_string());
    }
    
    let metric = MetricType::from_str(&metric_str)
        .ok_or_else(|| format!("Unknown metric: {}. Available: {:?}", metric_str, MetricType::all()))?;
    
    let pair = ScoringPair {
        candidate: candidate.clone(),
        reference: reference.clone(),
    };

    // Triple mode: always use text output for the 3-way comparison display
    if triple {
        return cmd_score_triple(metric, &pair);
    }

    let row = score_pair(metric, &pair);
    emit_rows(&[row], output_format);
    Ok(())
}

/// Triple-mode scoring (text only): shows reference/automaton/circuit agreement
fn cmd_score_triple(metric: MetricType, pair: &ScoringPair) -> Result<(), String> {
    println!("Metric: {:?}", metric);
    println!("Candidate: {:?}", pair.candidate);
    println!("Reference: {:?}", pair.reference);
    println!();

    match metric {
        MetricType::ExactMatch => {
            let scorer = ExactMatchScorer::case_insensitive();
            let result = scorer.score_and_verify(pair);
            println!("Reference impl:  {}", result.reference);
            println!("Automaton impl:  {}", result.automaton);
            println!("Circuit impl:    {}", result.circuit);
            println!("Agreement:       {}", result.agreement);
        }
        MetricType::TokenF1 => {
            let scorer = TokenF1Scorer::default_scorer();
            let result = scorer.score_and_verify(pair);
            println!("Reference impl:  P={:.4} R={:.4} F1={:.4}", result.reference.precision, result.reference.recall, result.reference.f1);
            println!("Automaton impl:  P={:.4} R={:.4} F1={:.4}", result.automaton.precision, result.automaton.recall, result.automaton.f1);
            println!("Circuit impl:    P={:.4} R={:.4} F1={:.4}", result.circuit.precision, result.circuit.recall, result.circuit.f1);
            println!("Agreement:       {}", result.agreement);
        }
        MetricType::Bleu => {
            let scorer = BleuScorer::with_smoothing(SmoothingMethod::Add1);
            let result = scorer.score_and_verify(pair);
            println!("Reference impl:  {:.4}", result.reference.score);
            println!("Automaton impl:  {:.4}", result.automaton.score);
            println!("Circuit impl:    {:.4}", result.circuit.score);
            println!("Agreement:       {}", result.agreement);
        }
        MetricType::Rouge1 => {
            let scorer = RougeNScorer::rouge1();
            let result = scorer.score_and_verify(pair);
            println!("Reference impl:  F1={:.4}", result.reference.f1);
            println!("Automaton impl:  F1={:.4}", result.automaton.f1);
            println!("Circuit impl:    F1={:.4}", result.circuit.f1);
            println!("Agreement:       {}", result.agreement);
        }
        MetricType::Rouge2 => {
            let scorer = RougeNScorer::rouge2();
            let result = scorer.score_and_verify(pair);
            println!("Reference impl:  F1={:.4}", result.reference.f1);
            println!("Automaton impl:  F1={:.4}", result.automaton.f1);
            println!("Circuit impl:    F1={:.4}", result.circuit.f1);
            println!("Agreement:       {}", result.agreement);
        }
        MetricType::RougeL => {
            let scorer = RougeLScorer::default_scorer();
            let result = scorer.score_and_verify(pair);
            println!("Reference impl:  F1={:.4}", result.reference.f1);
            println!("Automaton impl:  F1={:.4}", result.automaton.f1);
            println!("Circuit impl:    F1={:.4}", result.circuit.f1);
            println!("Agreement:       {}", result.agreement);
        }
    }
    Ok(())
}

fn cmd_differential_test(args: &[String]) -> Result<(), String> {
    let mut count = 50usize;
    let mut seed = 42u64;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--count" | "-n" => {
                i += 1;
                if i < args.len() {
                    count = args[i].parse().map_err(|_| "Invalid count")?;
                }
            }
            "--seed" | "-s" => {
                i += 1;
                if i < args.len() {
                    seed = args[i].parse().map_err(|_| "Invalid seed")?;
                }
            }
            _ => {}
        }
        i += 1;
    }
    
    println!("Running differential tests with {} pairs (seed={})", count, seed);
    println!();
    
    let mut pairs = standard_test_suite();
    pairs.extend(random_test_pairs(count, seed));
    
    let tester = DifferentialTester::new();
    let reports = tester.test_all_metrics(&pairs);
    
    let mut all_pass = true;
    for (metric, report) in &reports {
        let status = if report.is_perfect() { "PASS" } else { "FAIL" };
        if !report.is_perfect() { all_pass = false; }
        println!("[{}] {}: {}/{} agree ({:.1}%)",
            status, metric, report.agreements, report.total_tests,
            report.agreement_rate * 100.0);
        
        for detail in &report.disagreement_details {
            println!("  Disagreement #{}: ({:?}, {:?}): {}",
                detail.test_index, detail.candidate, detail.reference, detail.description);
        }
    }
    
    println!();
    if all_pass {
        println!("All differential tests PASSED.");
    } else {
        println!("Some differential tests FAILED.");
    }
    
    Ok(())
}

fn cmd_compile_circuit(args: &[String]) -> Result<(), String> {
    let mut metric_str = "exact_match".to_string();
    let mut max_len = 100usize;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--metric" | "-m" => {
                i += 1;
                if i < args.len() { metric_str = args[i].clone(); }
            }
            "--max-length" | "-l" => {
                i += 1;
                if i < args.len() {
                    max_len = args[i].parse().map_err(|_| "Invalid max-length")?;
                }
            }
            _ => {}
        }
        i += 1;
    }
    
    println!("Compiling circuit for metric: {}", metric_str);
    println!("Max input length: {}", max_len);
    
    let _circuit = scoring::ScoringCircuit::new();
    
    let size = estimate_proof_size(max_len * 10, max_len * 5, 128);
    println!("Estimated circuit size: {} constraints", max_len * 10);
    println!("Estimated proof size: {} bytes ({:.1} KB)", size, size as f64 / 1024.0);
    
    Ok(())
}

fn cmd_estimate_size(args: &[String]) -> Result<(), String> {
    let mut constraints = 1000usize;
    let mut wires = 500usize;
    let mut security = 128usize;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--constraints" => {
                i += 1;
                if i < args.len() {
                    constraints = args[i].parse().map_err(|_| "Invalid constraints")?;
                }
            }
            "--wires" => {
                i += 1;
                if i < args.len() {
                    wires = args[i].parse().map_err(|_| "Invalid wires")?;
                }
            }
            "--security" => {
                i += 1;
                if i < args.len() {
                    security = args[i].parse().map_err(|_| "Invalid security")?;
                }
            }
            _ => {}
        }
        i += 1;
    }
    
    let size = estimate_proof_size(constraints, wires, security);
    println!("Proof size estimate:");
    println!("  Constraints:    {}", constraints);
    println!("  Wires:          {}", wires);
    println!("  Security bits:  {}", security);
    println!("  Estimated size: {} bytes ({:.1} KB)", size, size as f64 / 1024.0);
    
    Ok(())
}

fn cmd_batch_score(args: &[String]) -> Result<(), String> {
    let mut metric_str = String::new();
    let mut metrics_str = String::new();
    let mut input_file = String::new();
    let mut output_format = OutputFormat::Text;
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--metric" | "-m" => {
                i += 1;
                if i < args.len() { metric_str = args[i].clone(); }
            }
            "--metrics" | "--metric-list" => {
                i += 1;
                if i < args.len() { metrics_str = args[i].clone(); }
            }
            "--input" | "-i" => {
                i += 1;
                if i < args.len() { input_file = args[i].to_string(); }
            }
            "--output-format" => {
                i += 1;
                if i < args.len() {
                    output_format = OutputFormat::from_str(&args[i])
                        .ok_or_else(|| format!("Unknown output format: {}. Use: text, json, csv, markdown", args[i]))?;
                }
            }
            _ => {}
        }
        i += 1;
    }
    
    if input_file.is_empty() {
        return Err("--input is required for batch scoring".to_string());
    }

    // Resolve which metrics to run
    let metrics: Vec<MetricType> = if !metrics_str.is_empty() {
        parse_metrics(&metrics_str)?
    } else if !metric_str.is_empty() {
        vec![MetricType::from_str(&metric_str)
            .ok_or_else(|| format!("Unknown metric: {}", metric_str))?]
    } else {
        return Err("Either --metric or --metrics is required".to_string());
    };

    let pairs = load_pairs(&input_file)?;

    eprintln!("Scoring {} pairs with {} metric(s): {}",
        pairs.len(),
        metrics.len(),
        metrics.iter().map(|m| m.name()).collect::<Vec<_>>().join(", "),
    );

    let mut rows = Vec::new();
    for metric in &metrics {
        for (idx, pair) in pairs.iter().enumerate() {
            let mut row = score_pair(*metric, pair);
            row.index = Some(idx);
            rows.push(row);
        }
    }

    emit_rows(&rows, output_format);
    Ok(())
}

fn cmd_hash(args: &[String]) -> Result<(), String> {
    let mut domain = String::new();
    let mut data = String::new();
    
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--domain" | "-d" => {
                i += 1;
                if i < args.len() { domain = args[i].to_string(); }
            }
            "--data" => {
                i += 1;
                if i < args.len() { data = args[i].to_string(); }
            }
            _ => {
                if data.is_empty() {
                    data = args[i].to_string();
                }
            }
        }
        i += 1;
    }
    
    let hasher = if domain.is_empty() {
        SpectaclesHasher::new()
    } else {
        SpectaclesHasher::with_domain(&domain)
    };
    
    let hash = hasher.hash_hex(data.as_bytes());
    println!("{}", hash);
    
    Ok(())
}

fn print_proof_info(proof: &CompactProof) {
    println!("Metric:           {}", proof.metric_id);
    println!("Candidate hash:   {}", hex::encode(proof.candidate_hash));
    println!("Reference hash:   {}", hex::encode(proof.reference_hash));
    println!("Score:            {}/{}", proof.score_numerator, proof.score_denominator);
    println!("Commitment:       {}", hex::encode(proof.commitment));
    println!("Proof data size:  {} bytes", proof.proof_data.len());
}

fn cmd_verify_proof(args: &[String]) -> Result<(), String> {
    let mut proof_path = String::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--proof" | "-p" => {
                i += 1;
                if i < args.len() { proof_path = args[i].to_string(); }
            }
            _ => {
                if proof_path.is_empty() {
                    proof_path = args[i].to_string();
                }
            }
        }
        i += 1;
    }

    if proof_path.is_empty() {
        return Err("--proof <file> is required".to_string());
    }

    let data = fs::read(&proof_path)
        .map_err(|e| format!("Failed to read proof file {}: {}", proof_path, e))?;

    // Try JSON format first
    if proof_path.ends_with(".json") {
        let proof: CompactProof = serde_json::from_slice(&data)
            .map_err(|e| format!("Failed to parse proof JSON: {}", e))?;
        print_proof_info(&proof);
        // Re-hash and verify commitment
        let hasher = SpectaclesHasher::new();
        let computed = hasher.hash(&proof.proof_data);
        let commitment_ok = computed == proof.commitment;
        println!("Commitment check: {}", if commitment_ok { "PASS" } else { "FAIL" });
        return Ok(());
    }

    // Binary format: validate header
    let header = ProofHeader::from_bytes(&data)
        .map_err(|e| format!("Invalid proof header: {}", e))?;
    println!("Proof file:       {}", proof_path);
    println!("Format version:   {}.{}", header.version.major, header.version.minor);
    println!("Payload size:     {} bytes", header.payload_size);
    println!("Checksum:         {}", hex::encode(&header.checksum));

    // Verify checksum over payload
    let payload_start = 49; // header size
    if data.len() >= payload_start {
        let payload = &data[payload_start..];
        let hasher = SpectaclesHasher::new();
        let computed = hasher.hash(payload);
        let checksum_ok = computed == header.checksum;
        println!("Checksum verify:  {}", if checksum_ok { "PASS" } else { "FAIL" });
    } else {
        println!("Checksum verify:  SKIP (no payload)");
    }

    Ok(())
}
