//! Performance and Latency Benchmark Suite for Borno Core Dictionary Engine (Milestone M6)
//!
//! Validates:
//! 1. Component Benchmark 1: Exact `contains()` lookup latency (10,000 iterations)
//! 2. Component Benchmark 2: Prefix completion query latency (10,000 iterations)
//! 3. Component Benchmark 3: Grapheme-cluster Levenshtein typo search (1,000 iterations)
//! 4. End-to-end keystroke latency across full engine pipeline (10,000 keystrokes)
//! 5. Memory overhead evaluation (<= 25 MB hard gate)

use bornomala_core::dictionary::{distance, DictionaryEngine};
use bornomala_core::{BornoEngine, EngineEvent};
use std::time::Instant;

fn compute_percentiles(mut durations_micros: Vec<f64>) -> (f64, f64, f64, f64) {
    if durations_micros.is_empty() {
        return (0.0, 0.0, 0.0, 0.0);
    }
    durations_micros.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = durations_micros.len();
    let mean = durations_micros.iter().sum::<f64>() / n as f64;
    let p50 = durations_micros[(n as f64 * 0.50).min((n - 1) as f64) as usize];
    let p95 = durations_micros[(n as f64 * 0.95).min((n - 1) as f64) as usize];
    let p99 = durations_micros[(n as f64 * 0.99).min((n - 1) as f64) as usize];
    (mean, p50, p95, p99)
}

#[test]
fn test_benchmark_exact_lookup_latency() {
    let engine = DictionaryEngine::global();
    let words = [
        "বাংলাদেশ", "মানুষ", "কথা", "কাজ", "বাংলা", "শিক্ষা", "সমাজ", "জীবন",
    ];

    let iterations = 10_000;
    let mut durations = Vec::with_capacity(iterations);

    for i in 0..iterations {
        let word = words[i % words.len()];
        let t0 = Instant::now();
        let found = engine.trie().contains(word);
        let elapsed = t0.elapsed();
        durations.push(elapsed.as_nanos() as f64 / 1_000.0);
        assert!(found);
    }

    let (mean, p50, p95, p99) = compute_percentiles(durations);

    println!(
        "\n--- Exact `contains()` Benchmark (10,000 iterations) ---\n\
         Mean: {:.3} µs | p50: {:.3} µs | p95: {:.3} µs | p99: {:.3} µs",
        mean, p50, p95, p99
    );

    #[cfg(not(debug_assertions))]
    assert!(
        p95 <= 50.0,
        "Exact lookup p95 exceeded provisional budget: {:.3} µs (budget: 50.0 µs)",
        p95
    );
}

#[test]
fn test_benchmark_prefix_completion_latency() {
    let engine = DictionaryEngine::global();
    let prefixes = ["বাং", "ভাব", "কম্", "দেশ", "প্র", "মান"];

    let iterations = 10_000;
    let mut durations = Vec::with_capacity(iterations);

    for i in 0..iterations {
        let prefix = prefixes[i % prefixes.len()];
        let t0 = Instant::now();
        let completions = engine.trie().find_completions(prefix, 3);
        let elapsed = t0.elapsed();
        durations.push(elapsed.as_nanos() as f64 / 1_000.0);
        assert!(!completions.is_empty());
    }

    let (mean, p50, p95, p99) = compute_percentiles(durations);

    println!(
        "\n--- Prefix Completion Query Benchmark (10,000 iterations) ---\n\
         Mean: {:.3} µs | p50: {:.3} µs | p95: {:.3} µs | p99: {:.3} µs",
        mean, p50, p95, p99
    );

    #[cfg(not(debug_assertions))]
    assert!(
        p95 <= 500.0,
        "Prefix completion p95 exceeded hard gate: {:.3} µs (budget: 500.0 µs)",
        p95
    );
}

#[test]
fn test_benchmark_spelling_correction_latency() {
    let engine = DictionaryEngine::global();
    let typo_tokens = ["বামগ্লা", "বৃস্তি", "কস্ট"];

    let iterations = 1_000;
    let mut durations = Vec::with_capacity(iterations);

    for i in 0..iterations {
        let token = typo_tokens[i % typo_tokens.len()];
        let t0 = Instant::now();
        let corrections = distance::search_spelling_corrections(engine.trie(), token, 3);
        let elapsed = t0.elapsed();
        durations.push(elapsed.as_nanos() as f64 / 1_000.0);
        assert!(!corrections.is_empty());
    }

    let (mean, p50, p95, p99) = compute_percentiles(durations);

    println!(
        "\n--- Spelling Correction Search Benchmark (1,000 iterations) ---\n\
         Mean: {:.3} µs ({:.3} ms) | p50: {:.3} µs | p95: {:.3} µs ({:.3} ms) | p99: {:.3} µs",
        mean,
        mean / 1000.0,
        p50,
        p95,
        p95 / 1000.0,
        p99
    );

    #[cfg(not(debug_assertions))]
    assert!(
        p95 <= 2000.0,
        "Spelling correction p95 exceeded hard gate: {:.3} ms (budget: 2.0 ms)",
        p95 / 1000.0
    );
}

#[test]
fn test_benchmark_end_to_end_keystroke_latency() {
    let mut engine = BornoEngine::new();
    engine.set_enable_dictionary_assistance(true);
    engine.set_enable_word_completion(true);
    engine.set_max_dictionary_suggestions(3);

    let test_tokens = [
        "khoma", "bangla", "biggan", "bristi", "ami", "desh",
        "shonar", "bangladesh", "kono", "bhasha",
    ];

    let mut durations_micros = Vec::new();
    let num_tokens = 1_000;

    for i in 0..num_tokens {
        let token = test_tokens[i % test_tokens.len()];
        for ch in token.chars() {
            let t0 = Instant::now();
            engine.process_event(EngineEvent::InputChar(ch));
            let elapsed = t0.elapsed();
            durations_micros.push(elapsed.as_nanos() as f64 / 1_000.0);
        }

        // Commit token
        let t0 = Instant::now();
        engine.process_event(EngineEvent::Commit);
        let elapsed = t0.elapsed();
        durations_micros.push(elapsed.as_nanos() as f64 / 1_000.0);
    }

    let (mean, p50, p95, p99) = compute_percentiles(durations_micros.clone());

    println!(
        "\n--- Borno Core End-to-End M6 Keystroke Latency ({} keystrokes) ---\n\
         Mean: {:.4} ms ({:.2} µs)\n\
         p50:  {:.4} ms ({:.2} µs)\n\
         p95:  {:.4} ms ({:.2} µs)\n\
         p99:  {:.4} ms ({:.2} µs)",
        durations_micros.len(),
        mean / 1000.0,
        mean,
        p50 / 1000.0,
        p50,
        p95 / 1000.0,
        p95,
        p99 / 1000.0,
        p99
    );

    #[cfg(not(debug_assertions))]
    {
        let hard_gate_ms = 1.0;
        let p95_ms = p95 / 1000.0;
        assert!(
            p95_ms <= hard_gate_ms,
            "End-to-end p95 keystroke latency exceeded gate: {:.4} ms (budget: {:.1} ms)",
            p95_ms,
            hard_gate_ms
        );
    }
}

#[test]
fn test_benchmark_heap_memory_overhead() {
    let engine = DictionaryEngine::global();
    let trie = engine.trie();

    // Node count and approximate in-memory size
    let node_count = trie.node_count();
    let word_count = trie.word_count;

    // Approximate memory:
    // TrieNode has: Option<u32> (8 bytes aligned), u32 (4 bytes), Vec<(char, usize)> (24 bytes)
    // Children vec typically 1-3 elements (16 bytes capacity + heap)
    let estimated_node_bytes = node_count * 48;
    let estimated_mb = estimated_node_bytes as f64 / (1024.0 * 1024.0);

    println!(
        "\n--- Memory Overhead Estimation ---\n\
         Words: {}\n\
         Trie Nodes: {}\n\
         Estimated In-Memory Heap: {:.2} MB (Hard Gate: <= 25 MB)",
        word_count, node_count, estimated_mb
    );

    assert!(
        estimated_mb <= 25.0,
        "Heap overhead exceeded 25 MB gate: {:.2} MB",
        estimated_mb
    );
}
