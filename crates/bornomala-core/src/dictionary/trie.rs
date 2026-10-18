//! In-memory compact prefix trie with branch-and-bound pruning for Borno Core.

use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct TrieNode {
    pub frequency: u32,
    pub max_subtree_freq: u32,
    pub children: Vec<(char, TrieNode)>,
}

impl TrieNode {
    pub fn new() -> Self {
        Self {
            frequency: 0,
            max_subtree_freq: 0,
            children: Vec::new(),
        }
    }
}

impl Default for TrieNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Compact prefix trie for Bangla lexicon lookups and prefix completions.
#[derive(Debug, Clone)]
pub struct PrefixTrie {
    pub root: TrieNode,
    pub word_count: usize,
}

impl Default for PrefixTrie {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Eq, PartialEq)]
struct CompletionItem {
    word: String,
    freq: u32,
}

impl Ord for CompletionItem {
    fn cmp(&self, other: &Self) -> Ordering {
        // We want a min-heap by (freq, reverse word) so the lowest score is at the top
        other.freq.cmp(&self.freq)
            .then_with(|| self.word.cmp(&other.word))
    }
}

impl PartialOrd for CompletionItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PrefixTrie {
    /// Creates an empty prefix trie.
    pub fn new() -> Self {
        Self {
            root: TrieNode::new(),
            word_count: 0,
        }
    }

    /// Loads and constructs a prefix trie from binary BLX1 formatted bytes.
    pub fn from_binary(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 8 || &data[0..4] != b"BLX1" {
            return Err("Invalid BLX1 magic header");
        }
        let count = u32::from_le_bytes(data[4..8].try_into().unwrap()) as usize;
        let mut trie = PrefixTrie::new();
        let mut cursor = 8;

        for _ in 0..count {
            if cursor + 2 > data.len() {
                return Err("Unexpected EOF reading word length");
            }
            let len = u16::from_le_bytes(data[cursor..cursor + 2].try_into().unwrap()) as usize;
            cursor += 2;

            if cursor + len + 4 > data.len() {
                return Err("Unexpected EOF reading word bytes and frequency");
            }
            let word_bytes = &data[cursor..cursor + len];
            cursor += len;

            let word = std::str::from_utf8(word_bytes).map_err(|_| "Invalid UTF-8 in word entry")?;
            let freq = u32::from_le_bytes(data[cursor..cursor + 4].try_into().unwrap());
            cursor += 4;

            trie.insert(word, freq);
        }

        Ok(trie)
    }

    /// Inserts a word and its observed frequency into the trie.
    pub fn insert(&mut self, word: &str, frequency: u32) {
        if word.is_empty() {
            return;
        }
        let mut curr = &mut self.root;
        curr.max_subtree_freq = curr.max_subtree_freq.max(frequency);

        for ch in word.chars() {
            let idx = match curr.children.binary_search_by_key(&ch, |(c, _)| *c) {
                Ok(i) => i,
                Err(i) => {
                    curr.children.insert(i, (ch, TrieNode::new()));
                    i
                }
            };
            curr = &mut curr.children[idx].1;
            curr.max_subtree_freq = curr.max_subtree_freq.max(frequency);
        }

        if curr.frequency == 0 {
            self.word_count += 1;
        }
        curr.frequency = curr.frequency.max(frequency);
    }

    /// Returns true if the word is an exact match in the trie.
    pub fn contains(&self, word: &str) -> bool {
        self.get_frequency(word).is_some()
    }

    /// Returns the frequency of the word if present in the trie.
    pub fn get_frequency(&self, word: &str) -> Option<u32> {
        let mut curr = &self.root;
        for ch in word.chars() {
            let idx = curr.children.binary_search_by_key(&ch, |(c, _)| *c).ok()?;
            curr = &curr.children[idx].1;
        }
        if curr.frequency > 0 {
            Some(curr.frequency)
        } else {
            None
        }
    }

    /// Finds up to `max_results` prefix completions sorted by descending frequency,
    /// using mathematically admissible branch-and-bound pruning.
    pub fn find_completions(&self, prefix: &str, max_results: usize) -> Vec<(String, u32)> {
        if max_results == 0 || prefix.is_empty() {
            return Vec::new();
        }

        // Navigate to the prefix node
        let mut curr = &self.root;
        for ch in prefix.chars() {
            match curr.children.binary_search_by_key(&ch, |(c, _)| *c) {
                Ok(idx) => curr = &curr.children[idx].1,
                Err(_) => return Vec::new(),
            }
        }

        let mut heap: std::collections::BinaryHeap<CompletionItem> =
            std::collections::BinaryHeap::with_capacity(max_results);

        let mut buffer = prefix.to_string();
        self.collect_pruned(curr, &mut buffer, max_results, &mut heap);

        // Convert heap to sorted vector: descending frequency, then lexicographical
        let mut results: Vec<(String, u32)> = heap
            .into_iter()
            .map(|item| (item.word, item.freq))
            .collect();

        results.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| a.0.cmp(&b.0))
        });

        results
    }

    /// Returns the total number of nodes in the trie.
    pub fn node_count(&self) -> usize {
        fn count_nodes(node: &TrieNode) -> usize {
            1 + node.children.iter().map(|(_, child)| count_nodes(child)).sum::<usize>()
        }
        count_nodes(&self.root)
    }

    fn collect_pruned(
        &self,
        node: &TrieNode,
        buffer: &mut String,
        max_results: usize,
        heap: &mut std::collections::BinaryHeap<CompletionItem>,
    ) {
        let min_score = if heap.len() == max_results {
            heap.peek().map(|item| item.freq).unwrap_or(0)
        } else {
            0
        };

        // If the entire subtree upper bound cannot beat the current threshold, prune!
        if heap.len() == max_results && node.max_subtree_freq <= min_score {
            return;
        }

        if node.frequency > 0 {
            if heap.len() < max_results {
                heap.push(CompletionItem {
                    word: buffer.clone(),
                    freq: node.frequency,
                });
            } else if node.frequency > min_score
                || (node.frequency == min_score && heap.peek().map_or(false, |p| buffer.as_str() < p.word.as_str()))
            {
                heap.pop();
                heap.push(CompletionItem {
                    word: buffer.clone(),
                    freq: node.frequency,
                });
            }
        }

        // Best-first exploration: sort child indices by descending max_subtree_freq
        let mut child_indices: Vec<usize> = (0..node.children.len()).collect();
        child_indices.sort_by(|&a, &b| {
            node.children[b].1.max_subtree_freq.cmp(&node.children[a].1.max_subtree_freq)
        });

        for idx in child_indices {
            let (ch, ref child_node) = node.children[idx];
            let current_min = if heap.len() == max_results {
                heap.peek().map(|item| item.freq).unwrap_or(0)
            } else {
                0
            };

            // Admissible pruning check
            if heap.len() == max_results && child_node.max_subtree_freq <= current_min {
                continue;
            }

            buffer.push(ch);
            self.collect_pruned(child_node, buffer, max_results, heap);
            buffer.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trie_insert_and_contains() {
        let mut trie = PrefixTrie::new();
        assert!(!trie.contains("বাংলাদেশ"));

        trie.insert("বাংলাদেশ", 4987);
        trie.insert("বাংলা", 690);
        trie.insert("বাঙালি", 285);

        assert!(trie.contains("বাংলাদেশ"));
        assert!(trie.contains("বাংলা"));
        assert!(trie.contains("বাঙালি"));
        assert!(!trie.contains("বাং")); // Non-terminal prefix

        assert_eq!(trie.get_frequency("বাংলাদেশ"), Some(4987));
        assert_eq!(trie.get_frequency("বাংলা"), Some(690));
        assert_eq!(trie.get_frequency("বাং"), None);
        assert_eq!(trie.word_count, 3);
    }

    #[test]
    fn test_trie_completions_ordered_by_frequency() {
        let mut trie = PrefixTrie::new();
        trie.insert("বাংলাদেশ", 4987);
        trie.insert("বাংলা", 690);
        trie.insert("বাংলামোটর", 150);
        trie.insert("বাংলানিউজ", 50);

        let completions = trie.find_completions("বাং", 3);
        assert_eq!(completions.len(), 3);
        assert_eq!(completions[0], ("বাংলাদেশ".to_string(), 4987));
        assert_eq!(completions[1], ("বাংলা".to_string(), 690));
        assert_eq!(completions[2], ("বাংলামোটর".to_string(), 150));
    }

    #[test]
    fn test_trie_completions_lexicographic_tie_breaking() {
        let mut trie = PrefixTrie::new();
        trie.insert("বাংলাখ", 100);
        trie.insert("বাংলাক", 100);

        let completions = trie.find_completions("বাংলা", 2);
        assert_eq!(completions.len(), 2);
        assert_eq!(completions[0].0, "বাংলাক");
        assert_eq!(completions[1].0, "বাংলাখ");
    }

    #[test]
    fn test_trie_binary_roundtrip() {
        let mut trie = PrefixTrie::new();
        trie.insert("কষ্ট", 294);
        trie.insert("বৃষ্টি", 347);

        // Serialize manually to BLX1
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"BLX1");
        bytes.extend_from_slice(&(2u32).to_le_bytes());

        for (word, freq) in [("কষ্ট", 294u32), ("বৃষ্টি", 347u32)] {
            let w_bytes = word.as_bytes();
            bytes.extend_from_slice(&(w_bytes.len() as u16).to_le_bytes());
            bytes.extend_from_slice(w_bytes);
            bytes.extend_from_slice(&freq.to_le_bytes());
        }

        let loaded = PrefixTrie::from_binary(&bytes).expect("Failed to load BLX1 binary");
        assert_eq!(loaded.word_count, 2);
        assert_eq!(loaded.get_frequency("কষ্ট"), Some(294));
        assert_eq!(loaded.get_frequency("বৃষ্টি"), Some(347));
    }
}
