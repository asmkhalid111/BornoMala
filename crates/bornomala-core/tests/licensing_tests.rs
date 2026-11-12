//! Automated Licensing Verification & Lexicon Integrity Tests (WP-06 Task 5)
//!
//! Deterministic, offline, read-only verification covering:
//! 1. Repository root `THIRD-PARTY-NOTICES.md` presence, size (> 30 KB), required attributions/licenses, and zero placeholders.
//! 2. `crates/bornomala-core/data/lexicon.bin` structural integrity (`BLX1` magic, byte size `1,116,626`, entry count `40,001`)
//!    and byte-for-byte parity with `EMBEDDED_LEXICON_BYTES` (structural integrity check, not cryptographic verification).
//! 3. Production `PrefixTrie` lookups for curated `বর্ণমালা` (`Some(25)`) and high-frequency benchmark `বাংলাদেশ` (`Some(4987)`).
//! 4. Provenance metadata consistency in `docs/lexicon_provenance.json` against the production binary/trie.
//! 5. Bounded technical isolation from Hunspell dictionary artifacts and Cargo runtime dependencies.
//!
//! SCOPE LIMITATION: These tests verify artifact presence, structural invariants, metadata consistency,
//! and documented disclosures. They do not verify the gitignored Leipzig archive independently,
//! trace per-word corpus origin, or certify legal compliance or distribution sufficiency.

use bornomala_core::dictionary::{DictionaryEngine, PrefixTrie, EMBEDDED_LEXICON_BYTES};
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_LEXICON_BYTE_LEN: usize = 1_116_626;
const EXPECTED_LEXICON_WORD_COUNT: usize = 40_001;
const MIN_ROOT_NOTICES_BYTES: usize = 30_720;

fn root_notices_path() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../THIRD-PARTY-NOTICES.md"))
}

fn repo_relative_path(manifest_relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(manifest_relative)
}

fn read_utf8_file(path: &Path, description: &str) -> String {
    assert!(
        path.exists(),
        "{} does not exist at expected manifest-relative path: {}",
        description,
        path.display()
    );
    assert!(
        path.is_file(),
        "{} is not a regular file at path: {}",
        description,
        path.display()
    );
    fs::read_to_string(path).unwrap_or_else(|err| {
        panic!(
            "Failed to read {} as UTF-8 from {}: {}",
            description,
            path.display(),
            err
        )
    })
}

fn parse_blx1_entry_count(bytes: &[u8], source_label: &str) -> u32 {
    assert!(
        bytes.len() >= 8,
        "{} is truncated ({} bytes; expected at least 8 bytes for BLX1 header)",
        source_label,
        bytes.len()
    );
    assert_eq!(
        &bytes[0..4],
        b"BLX1",
        "{} header magic mismatch (expected BLX1)",
        source_label
    );
    u32::from_le_bytes(
        bytes[4..8]
            .try_into()
            .expect("header slice length is verified to be 4 bytes"),
    )
}

#[test]
fn test_root_third_party_notices_exists_and_size() {
    let path = root_notices_path();
    let notices = read_utf8_file(path, "Repository root THIRD-PARTY-NOTICES.md");

    assert!(
        !notices.is_empty(),
        "THIRD-PARTY-NOTICES.md must not be empty"
    );
    assert!(
        notices.len() > MIN_ROOT_NOTICES_BYTES,
        "THIRD-PARTY-NOTICES.md size ({} bytes) must exceed 30 KB ({} bytes)",
        notices.len(),
        MIN_ROOT_NOTICES_BYTES
    );
    assert!(
        notices.starts_with("# Third-Party Notices & Licensing Acknowledgements"),
        "THIRD-PARTY-NOTICES.md must start with the authoritative document heading"
    );
}

#[test]
fn test_root_third_party_notices_required_attributions_and_licenses() {
    let notices = read_utf8_file(root_notices_path(), "Repository root THIRD-PARTY-NOTICES.md");

    let required_markers = [
        "Leipzig Corpora Collection",
        "Creative Commons Attribution 4.0",
        "GCC Runtime Library Exception",
        "Apache License",
        "The MIT License",
        "GNU Lesser General Public License",
        "ben_news_2020_300K",
        "Unicode-3.0",
        "Unicode License Agreement",
        "option-ext",
        "Mozilla Public License Version 2.0",
        "RV-11",
    ];

    for marker in required_markers {
        assert!(
            notices.contains(marker),
            "THIRD-PARTY-NOTICES.md is missing required attribution/license marker: {:?}",
            marker
        );
    }
}

#[test]
fn test_root_third_party_notices_no_placeholders() {
    let notices = read_utf8_file(root_notices_path(), "Repository root THIRD-PARTY-NOTICES.md");

    let forbidden_placeholders = [
        "[Full text of",
        "TODO",
        "TBD",
        "FIXME",
        "XXX",
        "[...]",
    ];

    for placeholder in forbidden_placeholders {
        assert!(
            !notices.contains(placeholder),
            "THIRD-PARTY-NOTICES.md must not contain unresolved placeholder marker: {:?}",
            placeholder
        );
    }
}

#[test]
fn test_lexicon_binary_structural_integrity_and_disk_parity() {
    // Structural verification (byte size, BLX1 magic header, entry count, disk/embedded parity),
    // intentionally avoiding external cryptographic hash dependencies.
    let disk_path = repo_relative_path("data/lexicon.bin");
    assert!(
        disk_path.is_file(),
        "On-disk lexicon binary does not exist as a regular file: {}",
        disk_path.display()
    );
    let disk_bytes = fs::read(&disk_path).unwrap_or_else(|err| {
        panic!("Failed to read on-disk lexicon binary {}: {}", disk_path.display(), err)
    });

    assert_eq!(
        disk_bytes.len(),
        EXPECTED_LEXICON_BYTE_LEN,
        "On-disk lexicon.bin byte size mismatch"
    );
    assert_eq!(
        EMBEDDED_LEXICON_BYTES.len(),
        EXPECTED_LEXICON_BYTE_LEN,
        "Embedded lexicon byte slice size mismatch"
    );

    let disk_count = parse_blx1_entry_count(&disk_bytes, "On-disk lexicon.bin");
    let embedded_count = parse_blx1_entry_count(EMBEDDED_LEXICON_BYTES, "Embedded lexicon bytes");

    assert_eq!(
        disk_count as usize,
        EXPECTED_LEXICON_WORD_COUNT,
        "On-disk lexicon.bin BLX1 header entry count mismatch"
    );
    assert_eq!(
        embedded_count as usize,
        EXPECTED_LEXICON_WORD_COUNT,
        "Embedded lexicon BLX1 header entry count mismatch"
    );
    assert!(
        disk_bytes == EMBEDDED_LEXICON_BYTES,
        "On-disk crates/bornomala-core/data/lexicon.bin bytes differ from EMBEDDED_LEXICON_BYTES"
    );

    let parsed_trie = PrefixTrie::from_binary(EMBEDDED_LEXICON_BYTES)
        .expect("PrefixTrie::from_binary must parse embedded production lexicon");
    assert_eq!(
        parsed_trie.word_count,
        EXPECTED_LEXICON_WORD_COUNT,
        "Parsed production PrefixTrie::word_count mismatch"
    );
}

#[test]
fn test_production_dictionary_frequencies() {
    let trie = DictionaryEngine::global().trie();

    assert_eq!(
        trie.word_count,
        EXPECTED_LEXICON_WORD_COUNT,
        "Global DictionaryEngine trie word count mismatch"
    );
    assert_eq!(
        trie.get_frequency("বর্ণমালা"),
        Some(25),
        "Curated Tatsama GATE-04 entry 'বর্ণমালা' must have exact frequency 25 in production trie"
    );
    assert_eq!(
        trie.get_frequency("বাংলাদেশ"),
        Some(4987),
        "Benchmark entry 'বাংলাদেশ' must have exact frequency 4987 in production trie"
    );
}

#[test]
fn test_lexicon_provenance_metadata_and_cross_checks() {
    // Verifies recorded provenance JSON metadata and agreement with production lexicon structural counts.
    // Does not independently hash the gitignored archive at scripts/.cache/ben_news_2020_300K.tar.gz.
    let provenance_path = repo_relative_path("../../docs/lexicon_provenance.json");
    let raw_json = read_utf8_file(&provenance_path, "Lexicon provenance metadata");
    let provenance: serde_json::Value = serde_json::from_str(&raw_json).unwrap_or_else(|err| {
        panic!(
            "Failed to parse {} as JSON: {}",
            provenance_path.display(),
            err
        )
    });

    assert_eq!(
        provenance["source_archive"].as_str(),
        Some("ben_news_2020_300K.tar.gz"),
        "Provenance source_archive mismatch"
    );
    assert_eq!(
        provenance["source_sha256"].as_str(),
        Some("7624300acee975a4344510bdacb000dd63c6709aa76ef3e922805c031bc22b49"),
        "Provenance source_sha256 digest record mismatch"
    );

    let stats = &provenance["preprocessing_stats"];
    let recorded_vocab_count = stats["final_vocabulary_count"]
        .as_u64()
        .expect("preprocessing_stats.final_vocabulary_count must be an integer");
    let recorded_binary_bytes = stats["binary_size_bytes"]
        .as_u64()
        .expect("preprocessing_stats.binary_size_bytes must be an integer");

    let header_entry_count = parse_blx1_entry_count(EMBEDDED_LEXICON_BYTES, "Embedded lexicon bytes") as u64;
    let trie_word_count = DictionaryEngine::global().trie().word_count as u64;

    assert_eq!(recorded_vocab_count, EXPECTED_LEXICON_WORD_COUNT as u64);
    assert_eq!(
        recorded_vocab_count,
        header_entry_count,
        "Provenance final_vocabulary_count must match BLX1 header count"
    );
    assert_eq!(
        recorded_vocab_count,
        trie_word_count,
        "Provenance final_vocabulary_count must match loaded PrefixTrie::word_count"
    );

    assert_eq!(recorded_binary_bytes, EXPECTED_LEXICON_BYTE_LEN as u64);
    assert_eq!(
        recorded_binary_bytes,
        EMBEDDED_LEXICON_BYTES.len() as u64,
        "Provenance binary_size_bytes must match EMBEDDED_LEXICON_BYTES byte length"
    );

    let hunspell_oracle = &provenance["hunspell_validation_oracle"];
    assert_eq!(
        hunspell_oracle["usage"].as_str(),
        Some("Offline validation oracle only; zero hunspell data redistributed in runtime binary"),
        "Provenance hunspell_validation_oracle.usage mismatch"
    );
    assert_eq!(
        hunspell_oracle["verified_word_count"].as_u64(),
        Some(15_823),
        "Provenance hunspell_validation_oracle.verified_word_count mismatch"
    );
    assert_eq!(
        hunspell_oracle["verified_percentage"].as_str(),
        Some("39.56%"),
        "Provenance hunspell_validation_oracle.verified_percentage mismatch"
    );
}

#[test]
fn test_hunspell_artifact_and_dependency_isolation() {
    // Bounded isolation checks: verifies absence of Hunspell dictionary files in data/,
    // absence of Hunspell Cargo dependencies, and TSV/BLX1/trie entry-count consistency.
    // Does not claim to prove individual word provenance from corpus vs validation oracle overlap.
    let data_dir = repo_relative_path("data");
    assert!(
        data_dir.is_dir(),
        "Core data directory does not exist: {}",
        data_dir.display()
    );

    for entry_res in fs::read_dir(&data_dir).expect("Failed to list crates/bornomala-core/data/") {
        let entry = entry_res.expect("Failed to read data directory entry");
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                assert!(
                    !ext.eq_ignore_ascii_case("dic") && !ext.eq_ignore_ascii_case("aff"),
                    "Forbidden Hunspell dictionary artifact present in core data directory: {}",
                    path.display()
                );
            }
        }
    }

    let manifests = [
        ("Workspace Cargo.toml", "../../Cargo.toml"),
        ("Core Cargo.toml", "Cargo.toml"),
        ("Desktop Cargo.toml", "../../src-tauri/Cargo.toml"),
    ];
    for (label, rel_path) in manifests {
        let path = repo_relative_path(rel_path);
        let content = read_utf8_file(&path, label);
        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            assert!(
                !trimmed.to_ascii_lowercase().contains("hunspell"),
                "{} line {} unexpectedly references 'hunspell': {}",
                label,
                idx + 1,
                trimmed
            );
        }
    }

    let lock_path = repo_relative_path("../../Cargo.lock");
    let lock_content = read_utf8_file(&lock_path, "Workspace Cargo.lock");
    for (idx, line) in lock_content.lines().enumerate() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("name = \"") {
            if let Some(pkg_name) = rest.strip_suffix('"') {
                let pkg_lower = pkg_name.to_ascii_lowercase();
                assert!(
                    !pkg_lower.starts_with("hunspell") && !pkg_lower.contains("hunspell"),
                    "Cargo.lock line {} resolves forbidden Hunspell package: {}",
                    idx + 1,
                    pkg_name
                );
            }
        }
    }

    let tsv_path = repo_relative_path("data/lexicon.tsv");
    let tsv_content = read_utf8_file(&tsv_path, "Production lexicon.tsv");
    let mut lines = tsv_content.lines();
    let header = lines.next().expect("lexicon.tsv must contain a header line");
    assert_eq!(header.trim(), "rank\tword\tfrequency");

    let data_rows = lines.filter(|line| !line.trim().is_empty()).count();
    let binary_entry_count = parse_blx1_entry_count(EMBEDDED_LEXICON_BYTES, "Embedded lexicon bytes") as usize;
    let trie_word_count = DictionaryEngine::global().trie().word_count;

    assert_eq!(
        data_rows,
        EXPECTED_LEXICON_WORD_COUNT,
        "lexicon.tsv data row count mismatch"
    );
    assert_eq!(
        data_rows,
        binary_entry_count,
        "lexicon.tsv data row count must equal lexicon.bin header count"
    );
    assert_eq!(
        data_rows,
        trie_word_count,
        "lexicon.tsv data row count must equal PrefixTrie::word_count"
    );
}

