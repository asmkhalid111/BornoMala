//! Bounded Damerau-Levenshtein edit distance over UAX #29 extended grapheme clusters.

use unicode_segmentation::UnicodeSegmentation;
use crate::dictionary::trie::{PrefixTrie, TrieNode};

/// Maximum number of nodes visited during trie-based fuzzy correction search.
pub const MAX_VISITED_NODES: usize = 1000;

/// Computes the Damerau-Levenshtein edit distance between two strings using
/// UAX #29 extended grapheme clusters.
///
/// Supported atomic edit operations (each cost 1):
/// - Insertion of a grapheme cluster
/// - Deletion of a grapheme cluster
/// - Substitution of a grapheme cluster
/// - Adjacent transposition of two grapheme clusters
pub fn grapheme_damerau_levenshtein(s1: &str, s2: &str) -> usize {
    let a: Vec<&str> = s1.graphemes(true).collect();
    let b: Vec<&str> = s2.graphemes(true).collect();

    let m = a.len();
    let n = b.len();

    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }

    // dp matrix: (m + 1) x (n + 1)
    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };

            let mut dist = (dp[i - 1][j] + 1) // deletion
                .min(dp[i][j - 1] + 1)        // insertion
                .min(dp[i - 1][j - 1] + cost); // substitution

            // Adjacent transposition check
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                dist = dist.min(dp[i - 2][j - 2] + 1);
            }

            dp[i][j] = dist;
        }
    }

    dp[m][n]
}

/// Discovers close spelling corrections in the trie within bounded grapheme edit distance.
///
/// Invariants:
/// - Active ONLY when query character length >= 3.
/// - Distance threshold: D <= 1 for queries with <= 5 graphemes; D <= 2 for queries with >= 6.
/// - Bounded by `MAX_VISITED_NODES = 1,000`.
/// - Returned candidates are sorted by distance ascending, then frequency descending, then lexicographical order.
pub fn search_spelling_corrections(
    trie: &PrefixTrie,
    query: &str,
    max_results: usize,
) -> Vec<(String, u32, usize)> {
    if query.chars().count() < 3 || max_results == 0 {
        return Vec::new();
    }

    let q_graphemes: Vec<&str> = query.graphemes(true).collect();
    let q_len = q_graphemes.len();
    let max_dist = if q_len <= 5 { 1 } else { 2 };

    let mut visited_count = 0;
    let mut results: Vec<(String, u32, usize)> = Vec::new();
    let mut path = String::with_capacity(query.len() + 8);

    traverse_for_corrections(
        &trie.root,
        &mut path,
        query,
        q_len,
        max_dist,
        &mut visited_count,
        &mut results,
    );

    // Sort: distance ascending, then frequency descending, then word ascending
    results.sort_by(|a, b| {
        a.2.cmp(&b.2)
            .then_with(|| b.1.cmp(&a.1))
            .then_with(|| a.0.cmp(&b.0))
    });

    if results.len() > max_results {
        results.truncate(max_results);
    }

    results
}

fn traverse_for_corrections(
    node: &TrieNode,
    path: &mut String,
    query: &str,
    q_grapheme_len: usize,
    max_dist: usize,
    visited_count: &mut usize,
    results: &mut Vec<(String, u32, usize)>,
) {
    *visited_count += 1;
    if *visited_count > MAX_VISITED_NODES {
        return;
    }

    let path_grapheme_len = path.graphemes(true).count();

    // Check terminal word match
    if node.frequency > 0 && !path.is_empty() {
        let dist = grapheme_damerau_levenshtein(query, path);
        if dist > 0 && dist <= max_dist {
            results.push((path.clone(), node.frequency, dist));
        }
    }

    // Prune exploration if path grapheme length already exceeds query length by more than max_dist
    if path_grapheme_len > q_grapheme_len + max_dist {
        return;
    }

    // Prioritize child exploration: if next character matches query, visit that child FIRST
    let next_query_char = query.chars().nth(path.chars().count());

    let mut child_indices: Vec<usize> = (0..node.children.len()).collect();
    if let Some(target_char) = next_query_char {
        if let Some(pos) = child_indices.iter().position(|&i| node.children[i].0 == target_char) {
            child_indices.swap(0, pos);
        }
    }

    // Continue DFS
    for idx in child_indices {
        let (ch, ref child_node) = node.children[idx];
        if *visited_count >= MAX_VISITED_NODES {
            break;
        }
        path.push(ch);
        traverse_for_corrections(
            child_node,
            path,
            query,
            q_grapheme_len,
            max_dist,
            visited_count,
            results,
        );
        path.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grapheme_distance_exact_match() {
        assert_eq!(grapheme_damerau_levenshtein("বাংলা", "বাংলা"), 0);
        assert_eq!(grapheme_damerau_levenshtein("কষ্ট", "কষ্ট"), 0);
    }

    #[test]
    fn test_grapheme_distance_substitution() {
        // "কষ্ট" (["ক", "ষ্ট"]) vs "কস্ট" (["ক", "স্ট"]) -> 1 substitution
        assert_eq!(grapheme_damerau_levenshtein("কষ্ট", "কস্ট"), 1);
    }

    #[test]
    fn test_grapheme_distance_transposition() {
        // Transposition of two adjacent clusters
        assert_eq!(grapheme_damerau_levenshtein("কখ", "খক"), 1);
        // Swap adjacent "ক" and "ষ্ট" -> "ষ্টকখ"
        assert_eq!(grapheme_damerau_levenshtein("কষ্টখ", "ষ্টকখ"), 1);
        // Swap adjacent "ষ্ট" and "খ" -> "কখষ্ট"
        assert_eq!(grapheme_damerau_levenshtein("কষ্টখ", "কখষ্ট"), 1);
    }

    #[test]
    fn test_search_corrections_discovers_typos() {
        let mut trie = PrefixTrie::new();
        trie.insert("বাংলা", 690);
        trie.insert("বাংলাদেশ", 4987);

        // "বাংল" (missing aa kar) -> distance 1 to "বাংলা"
        let suggestions = search_spelling_corrections(&trie, "বাংল", 3);
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].0, "বাংলা");
        assert_eq!(suggestions[0].2, 1);
    }
}
