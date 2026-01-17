//! The algorithmic core: Trie-based solver.

use crate::config::Config;
use crate::dictionary::{Dictionary, TrieNode};
use crate::error::SbsError;
use std::collections::HashSet;

pub struct Solver {
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
        Self { config }
    }

    /// Solves the puzzle using the provided Dictionary.
    /// The Dictionary is now passed in, allowing it to be shared across requests.
    pub fn solve(&self, dictionary: &Dictionary) -> Result<HashSet<String>, SbsError> {
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

        let mut ctx = SearchContext {
            allowed: &allowed_chars,
            required: &required_chars,
            min_len,
            max_len,
            results: &mut results,
        };

        // Start DFS from root of the provided dictionary
        Self::find_words(&dictionary.root, String::new(), &mut ctx);

        Ok(results)
    }

    fn find_words(node: &TrieNode, current_word: String, ctx: &mut SearchContext) {
        if current_word.len() > ctx.max_len {
            return;
        }

        if node.is_end_of_word && current_word.len() >= ctx.min_len {
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
        let config = Config::new().with_letters("abcdefg").with_present("a");

        let solver = Solver::new(config);

        // Inject mock dictionary via new Dictionary API
        let dict = Dictionary::from_words(&["bad", "fade", "faced", "zzzz", "bed"]);

        let results = solver.solve(&dict).expect("Solver failed");

        assert!(results.contains("fade"));
        assert!(results.contains("faced"));
        assert!(!results.contains("bad"));
        assert!(!results.contains("zzzz"));
    }
}
