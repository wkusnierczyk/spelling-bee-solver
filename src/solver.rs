//! The algorithmic core: Trie-based solver.

use std::collections::{HashSet, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::config::Config;
use crate::error::SbsError;

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
            node = node.children.entry(ch).or_insert_with(TrieNode::default);
        }
        node.is_end_of_word = true;
    }
}

pub struct Solver {
    trie: TrieNode,
    config: Config,
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
            return Err(SbsError::DictionaryError(
                format!("Dictionary file not found at {:?}. Did you run 'make setup'?", path)
            ));
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

    pub fn solve(&self) -> Result<HashSet<String>, SbsError> {
        let letters_str = self.config.letters.as_ref()
            .ok_or(SbsError::ConfigError("No letters provided".to_string()))?
            .to_lowercase();
        
        let required_str = self.config.present.as_ref()
            .ok_or(SbsError::ConfigError("No required letter provided".to_string()))?
            .to_lowercase();

        let min_len = self.config.minimal_word_length.unwrap_or(4);
        let max_len = self.config.maximal_word_length.unwrap_or(usize::MAX);
        
        let allowed_chars: HashSet<char> = letters_str.chars().collect();
        let required_chars: HashSet<char> = required_str.chars().collect();
        
        let mut results = HashSet::new();

        // Start DFS from root
        self.find_words(&self.trie, String::new(), &allowed_chars, &required_chars, min_len, max_len, &mut results);

        Ok(results)
    }

    fn find_words(
        &self, 
        node: &TrieNode, 
        current_word: String, 
        allowed: &HashSet<char>, 
        required: &HashSet<char>,
        min_len: usize,
        max_len: usize,
        results: &mut HashSet<String>
    ) {
        if current_word.len() > max_len {
            return;
        }

        if node.is_end_of_word {
            if current_word.len() >= min_len {
                // Check if all required chars are present
                let mut all_req_present = true;
                for req in required {
                    if !current_word.contains(*req) {
                        all_req_present = false;
                        break;
                    }
                }
                if all_req_present {
                    results.insert(current_word.clone());
                }
            }
        }

        for (ch, next_node) in &node.children {
            if allowed.contains(ch) {
                // If config.repeats is set, we would check counts here.
                // For now assuming infinite repeats allowed per standard rules.
                let mut next_word = current_word.clone();
                next_word.push(*ch);
                self.find_words(next_node, next_word, allowed, required, min_len, max_len, results);
            }
        }
    }
}
