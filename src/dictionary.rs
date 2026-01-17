//! Dictionary data structure and loading logic.

use crate::error::SbsError;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Represents a node in the Trie.
/// Public so Solver can traverse it.
#[derive(Default, Debug)]
pub struct TrieNode {
    pub children: HashMap<char, TrieNode>,
    pub is_end_of_word: bool,
}

impl TrieNode {
    fn insert(&mut self, word: &str) {
        let mut node = self;
        for ch in word.chars() {
            node = node.children.entry(ch).or_default();
        }
        node.is_end_of_word = true;
    }
}

/// A read-only container for the word list.
pub struct Dictionary {
    pub root: TrieNode,
}

impl Dictionary {
    pub fn new() -> Self {
        Self {
            root: TrieNode::default(),
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, SbsError> {
        let path_ref = path.as_ref();
        if !path_ref.exists() {
            return Err(SbsError::DictionaryError(format!(
                "Dictionary file not found at {:?}.",
                path_ref
            )));
        }

        let file = File::open(path_ref)?;
        let reader = BufReader::new(file);
        let mut root = TrieNode::default();

        for line in reader.lines() {
            let word = line?;
            let clean_word = word.trim().to_lowercase();
            if !clean_word.is_empty() && clean_word.chars().all(char::is_alphabetic) {
                root.insert(&clean_word);
            }
        }
        Ok(Self { root })
    }

    // Helper for tests
    pub fn from_words(words: &[&str]) -> Self {
        let mut root = TrieNode::default();
        for w in words {
            root.insert(w);
        }
        Self { root }
    }
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}
