//! The algorithmic core: Trie-based solver.

use crate::config::Config;
use crate::error::SbsError;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Represents a node in the Trie.
#[derive(Default, Debug)]
struct TrieNode {
    children: HashMap<char, TrieNode>,
    is_end_of_word: bool,
}

impl TrieNode {
    fn insert(&mut self, word: &str) {
        let mut node = self;
        for ch in word.chars() {
            // Fix: Use or_default()
            node = node.children.entry(ch).or_default();
        }
        node.is_end_of_word = true;
    }
}

pub struct Solver {
    trie: TrieNode,
    config: Config,
}

/// Context struct to reduce argument count in recursion
struct SearchContext<'a> {
    allowed: &'a HashSet<char>,
    required: &'a HashSet<char>,
    min_len: usize,
    max_len: usize,
    results: &'a mut HashSet<String>,
}

impl Solver {
    pub fn new(config: Config) -> Self {
        Self {
            trie: TrieNode::default(),
            config,
        }
    }

    /// Init: Loads the dictionary specified in config into the Trie
    pub fn load_dictionary(&mut self) -> Result<(), SbsError> {
        let path = &self.config.dictionary;

        if !path.exists() {
            return Err(SbsError::DictionaryError(format!(
                "Dictionary file not found at {:?}. Did you run 'make setup'?",
                path
            )));
        }

        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let word = line?;
            // Simple sanitization: trim and lower case
            let clean_word = word.trim().to_lowercase();
            if !clean_word.is_empty() && clean_word.chars().all(char::is_alphabetic) {
                self.trie.insert(&clean_word);
            }
        }
        Ok(())
    }

    /// Helper for testing to inject words directly
    #[cfg(test)]
    pub fn load_words_slice(&mut self, words: &[&str]) {
        for word in words {
            self.trie.insert(word);
        }
    }

    pub fn solve(&self) -> Result<HashSet<String>, SbsError> {
        let letters_str = self
            .config
            .letters
            .as_ref()
            .ok_or(SbsError::ConfigError("No letters provided".to_string()))?
            .to_lowercase();

        let required_str = self
            .config
            .present
            .as_ref()
            .ok_or(SbsError::ConfigError(
                "No required letter provided".to_string(),
            ))?
            .to_lowercase();

        let min_len = self.config.minimal_word_length.unwrap_or(4);
        let max_len = self.config.maximal_word_length.unwrap_or(usize::MAX);

        let allowed_chars: HashSet<char> = letters_str.chars().collect();
        let required_chars: HashSet<char> = required_str.chars().collect();

        let mut results = HashSet::new();

        // Create context to pass down
        let mut ctx = SearchContext {
            allowed: &allowed_chars,
            required: &required_chars,
            min_len,
            max_len,
            results: &mut results,
        };

        // Start DFS from root
        // Fix: Removed &self from recursion as it was unused
        Self::find_words(&self.trie, String::new(), &mut ctx);

        Ok(results)
    }

    fn find_words(node: &TrieNode, current_word: String, ctx: &mut SearchContext) {
        if current_word.len() > ctx.max_len {
            return;
        }

        // Fix: Collapsed if statement
        if node.is_end_of_word && current_word.len() >= ctx.min_len {
            // Check if all required chars are present
            let mut all_req_present = true;
            for req in ctx.required {
                if !current_word.contains(*req) {
                    all_req_present = false;
                    break;
                }
            }
            if all_req_present {
                ctx.results.insert(current_word.clone());
            }
        }

        for (ch, next_node) in &node.children {
            if ctx.allowed.contains(ch) {
                let mut next_word = current_word.clone();
                next_word.push(*ch);
                Self::find_words(next_node, next_word, ctx);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_basic() {
        let mut config = Config::new().with_letters("abcdefg").with_present("a");
        // Override dictionary path to avoid IO in unit test,
        // strictly speaking load_dictionary won't be called here.

        let mut solver = Solver::new(config);

        // Inject mock dictionary
        solver.load_words_slice(&[
            "bad",   // too short
            "fade",  // valid
            "faced", // valid
            "zzzz",  // invalid letters
            "bed",   // valid length, but 'e' might not be in letters if we change config
        ]);

        let results = solver.solve().expect("Solver failed");

        assert!(results.contains("fade"));
        assert!(results.contains("faced"));
        assert!(!results.contains("bad")); // length < 4
        assert!(!results.contains("zzzz")); // z not allowed
    }

    #[test]
    fn test_missing_required_letter() {
        let config = Config::new().with_letters("abcdefg").with_present("z"); // z is required but not in list (impossible, but logic should handle)

        let mut solver = Solver::new(config);
        solver.load_words_slice(&["faced"]);

        let results = solver.solve().expect("Solver failed");
        assert!(results.is_empty());
    }
}
