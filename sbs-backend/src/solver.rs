//! The algorithmic core: Trie-based solver.

use crate::config::Config;
use crate::dictionary::{Dictionary, TrieNode};
use crate::error::SbsError;
use std::collections::{HashMap, HashSet};

pub struct Solver {
    config: Config,
}

/// Context struct to reduce argument count in recursion
struct SearchContext<'a> {
    allowed: &'a HashSet<char>,
    required: &'a HashSet<char>,
    min_len: usize,
    max_len: usize,
    max_repeats: Option<usize>,
    results: &'a mut HashSet<String>,
}

impl Solver {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

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
        let max_repeats = self.config.repeats;

        let allowed_chars: HashSet<char> = letters_str.chars().collect();
        let required_chars: HashSet<char> = required_str.chars().collect();

        let mut results = HashSet::new();

        let mut ctx = SearchContext {
            allowed: &allowed_chars,
            required: &required_chars,
            min_len,
            max_len,
            max_repeats,
            results: &mut results,
        };

        // Track character usage counts
        let mut char_counts = HashMap::new();

        // Start DFS from root
        Self::find_words(&dictionary.root, String::new(), &mut char_counts, &mut ctx);

        Ok(results)
    }

    fn find_words(
        node: &TrieNode,
        current_word: String,
        char_counts: &mut HashMap<char, usize>,
        ctx: &mut SearchContext,
    ) {
        if current_word.len() > ctx.max_len {
            return;
        }

        // Check Valid Word
        if node.is_end_of_word && current_word.len() >= ctx.min_len {
            let mut all_req_present = true;
            for req in ctx.required {
                // Check counts directly instead of string scan
                if *char_counts.get(req).unwrap_or(&0) == 0 {
                    all_req_present = false;
                    break;
                }
            }
            if all_req_present {
                ctx.results.insert(current_word.clone());
            }
        }

        // Recursive Backtracking
        for (ch, next_node) in &node.children {
            if ctx.allowed.contains(ch) {
                // Check repetition limit
                let count = *char_counts.get(ch).unwrap_or(&0);
                if let Some(limit) = ctx.max_repeats {
                    if count >= limit {
                        continue; // Skip this branch
                    }
                }

                // Update state
                let mut next_word = current_word.clone();
                next_word.push(*ch);
                *char_counts.entry(*ch).or_insert(0) += 1;

                // Recurse
                Self::find_words(next_node, next_word, char_counts, ctx);

                // Backtrack (Restore state)
                *char_counts.entry(*ch).or_insert(0) -= 1;
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

        // Inject mock dictionary
        let dict = Dictionary::from_words(&["bad", "fade", "faced", "zzzz", "bed"]);

        let results = solver.solve(&dict).expect("Solver failed");

        assert!(results.contains("fade"));
        assert!(results.contains("faced"));
        assert!(!results.contains("bad"));
        assert!(!results.contains("zzzz"));
    }

    #[test]
    fn test_solver_repeats() {
        let mut config = Config::new().with_letters("ab").with_present("a");
        config.repeats = Some(1);
        config.minimal_word_length = Some(2); // FIX: Allow short words for this test

        let solver = Solver::new(config);
        let dict = Dictionary::from_words(&["aa", "ab"]);

        let results = solver.solve(&dict).expect("Solver failed");

        assert!(results.contains("ab"), "Result should contain 'ab'");
        assert!(
            !results.contains("aa"),
            "Result should NOT contain 'aa' due to repeat limit"
        );
    }
}
