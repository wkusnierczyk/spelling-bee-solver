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
    anywhere: &'a HashSet<char>,
    required: &'a HashSet<char>,
    required_start: Option<char>,
    case_sensitive: bool,
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
        let case_sensitive = self.config.case_sensitive.unwrap_or(false);

        let letters_str = self
            .config
            .letters
            .as_ref()
            .ok_or(SbsError::ConfigError("No letters provided".to_string()))?;

        let required_str = self.config.present.as_ref().ok_or(SbsError::ConfigError(
            "No required letter provided".to_string(),
        ))?;

        let min_len = self.config.minimal_word_length.unwrap_or(4);
        let max_len = self.config.maximal_word_length.unwrap_or(usize::MAX);
        let max_repeats = self.config.repeats;

        let (allowed_chars, anywhere_chars, required_chars, required_start) = if case_sensitive {
            // Uppercase letters in `letters` can only appear at position 0
            let mut start_only: HashSet<char> = HashSet::new();
            let mut anywhere: HashSet<char> = HashSet::new();
            for ch in letters_str.chars() {
                if ch.is_uppercase() {
                    start_only.insert(ch.to_lowercase().next().unwrap());
                } else {
                    anywhere.insert(ch);
                }
            }
            let allowed: HashSet<char> = start_only.union(&anywhere).copied().collect();

            // Uppercase in `present` means required at start (max 1)
            let mut req_start: Option<char> = None;
            let mut required: HashSet<char> = HashSet::new();
            for ch in required_str.chars() {
                if ch.is_uppercase() {
                    let lower = ch.to_lowercase().next().unwrap();
                    if req_start.is_some() {
                        return Err(SbsError::ConfigError(
                            "At most one uppercase required letter allowed in case-sensitive mode"
                                .to_string(),
                        ));
                    }
                    req_start = Some(lower);
                    required.insert(lower);
                } else {
                    required.insert(ch);
                }
            }

            (allowed, anywhere, required, req_start)
        } else {
            let lowered = letters_str.to_lowercase();
            let allowed: HashSet<char> = lowered.chars().collect();
            let anywhere = allowed.clone();
            let required: HashSet<char> = required_str.to_lowercase().chars().collect();
            (allowed, anywhere, required, None)
        };

        let mut results = HashSet::new();

        let mut ctx = SearchContext {
            allowed: &allowed_chars,
            anywhere: &anywhere_chars,
            required: &required_chars,
            required_start,
            case_sensitive,
            min_len,
            max_len,
            max_repeats,
            results: &mut results,
        };

        let mut char_counts = HashMap::new();

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
                if *char_counts.get(req).unwrap_or(&0) == 0 {
                    all_req_present = false;
                    break;
                }
            }
            // If case-sensitive and required_start is set, first char must match
            if all_req_present {
                if let Some(start_char) = ctx.required_start {
                    if !current_word.starts_with(start_char) {
                        all_req_present = false;
                    }
                }
            }
            if all_req_present {
                ctx.results.insert(current_word.clone());
            }
        }

        let depth = current_word.len();

        // Recursive Backtracking
        for (ch, next_node) in &node.children {
            // In case-sensitive mode, start-only chars can only appear at depth 0
            let char_allowed = if ctx.case_sensitive && depth > 0 {
                ctx.anywhere.contains(ch)
            } else {
                ctx.allowed.contains(ch)
            };

            if char_allowed {
                // Check repetition limit
                let count = *char_counts.get(ch).unwrap_or(&0);
                if let Some(limit) = ctx.max_repeats {
                    if count >= limit {
                        continue;
                    }
                }

                let mut next_word = current_word.clone();
                next_word.push(*ch);
                *char_counts.entry(*ch).or_insert(0) += 1;

                Self::find_words(next_node, next_word, char_counts, ctx);

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
        let dict = Dictionary::from_words(&["bad", "fade", "faced", "zzzz", "bed"]);

        let results = solver.solve(&dict).expect("Solver failed");

        assert!(results.contains("fade"));
        assert!(results.contains("faced"));
        assert!(!results.contains("bad"));
        assert!(!results.contains("zzzz"));
    }

    #[test]
    fn test_solver_multiple_required_letters() {
        let config = Config::new().with_letters("abcdefg").with_present("af");

        let solver = Solver::new(config);
        let dict = Dictionary::from_words(&["fade", "faced", "bead", "cafe", "face"]);

        let results = solver.solve(&dict).expect("Solver failed");

        assert!(results.contains("fade"), "contains both a and f");
        assert!(results.contains("faced"), "contains both a and f");
        assert!(results.contains("cafe"), "contains both a and f");
        assert!(results.contains("face"), "contains both a and f");
        assert!(!results.contains("bead"), "missing f");
    }

    #[test]
    fn test_solver_min_length() {
        let mut config = Config::new().with_letters("abcde").with_present("a");
        config.minimal_word_length = Some(5);

        let solver = Solver::new(config);
        let dict = Dictionary::from_words(&["abcd", "abcde", "ace", "abcdef"]);

        let results = solver.solve(&dict).expect("Solver failed");

        assert!(
            !results.contains("abcd"),
            "4-letter word should be excluded with min=5"
        );
        assert!(
            results.contains("abcde"),
            "5-letter word should be included"
        );
        assert!(!results.contains("ace"), "3-letter word should be excluded");
    }

    #[test]
    fn test_solver_max_length() {
        let mut config = Config::new().with_letters("abcde").with_present("a");
        config.minimal_word_length = Some(2);
        config.maximal_word_length = Some(4);

        let solver = Solver::new(config);
        let dict = Dictionary::from_words(&["ab", "abc", "abcd", "abcde"]);

        let results = solver.solve(&dict).expect("Solver failed");

        assert!(
            results.contains("abcd"),
            "4-letter word should be included with max=4"
        );
        assert!(
            !results.contains("abcde"),
            "5-letter word should be excluded with max=4"
        );
    }

    #[test]
    fn test_solver_min_and_max_length() {
        let mut config = Config::new().with_letters("abcde").with_present("a");
        config.minimal_word_length = Some(3);
        config.maximal_word_length = Some(4);

        let solver = Solver::new(config);
        let dict = Dictionary::from_words(&["ab", "abc", "abcd", "abcde"]);

        let results = solver.solve(&dict).expect("Solver failed");

        assert!(!results.contains("ab"), "2-letter word excluded");
        assert!(results.contains("abc"), "3-letter word included");
        assert!(results.contains("abcd"), "4-letter word included");
        assert!(!results.contains("abcde"), "5-letter word excluded");
    }

    #[test]
    fn test_solver_default_min_length() {
        let config = Config::new().with_letters("abcde").with_present("a");

        let solver = Solver::new(config);
        let dict = Dictionary::from_words(&["ab", "abc", "abcd", "abcde"]);

        let results = solver.solve(&dict).expect("Solver failed");

        assert!(
            !results.contains("ab"),
            "2-letter word excluded by default min=4"
        );
        assert!(
            !results.contains("abc"),
            "3-letter word excluded by default min=4"
        );
        assert!(
            results.contains("abcd"),
            "4-letter word included by default min=4"
        );
    }

    #[test]
    fn test_solver_repeats() {
        let mut config = Config::new().with_letters("ab").with_present("a");
        config.repeats = Some(1);
        config.minimal_word_length = Some(2);

        let solver = Solver::new(config);
        let dict = Dictionary::from_words(&["aa", "ab"]);

        let results = solver.solve(&dict).expect("Solver failed");

        assert!(results.contains("ab"), "Result should contain 'ab'");
        assert!(
            !results.contains("aa"),
            "Result should NOT contain 'aa' due to repeat limit"
        );
    }

    // --- Case-sensitive mode tests ---

    #[test]
    fn test_solver_case_insensitive_default() {
        // Uppercase input should be normalized when case_sensitive is off (default)
        let config = Config::new().with_letters("ABCDEFG").with_present("A");

        let solver = Solver::new(config);
        let dict = Dictionary::from_words(&["fade", "faced", "bad"]);

        let results = solver.solve(&dict).expect("Solver failed");

        assert!(
            results.contains("fade"),
            "uppercase input should be normalized"
        );
        assert!(results.contains("faced"));
    }

    #[test]
    fn test_solver_case_sensitive_start_only() {
        // 'W' uppercase means 'w' can only appear at position 0
        // 'a', 'r', 'e' are lowercase, can appear anywhere
        let mut config = Config::new().with_letters("Ware").with_present("a");
        config.case_sensitive = Some(true);
        config.minimal_word_length = Some(3);

        let solver = Solver::new(config);
        // "war" starts with w — OK
        // "raw" has w not at start — should be excluded
        // "ware" starts with w — OK
        // "area" has no w — OK (w is not required)
        let dict = Dictionary::from_words(&["war", "raw", "ware", "area", "aw"]);

        let results = solver.solve(&dict).expect("Solver failed");

        assert!(results.contains("war"), "w at position 0 is allowed");
        assert!(!results.contains("raw"), "w at position > 0 is not allowed");
        assert!(results.contains("ware"), "w at position 0 is allowed");
        assert!(
            results.contains("area"),
            "no w needed, all letters are anywhere-letters"
        );
    }

    #[test]
    fn test_solver_case_sensitive_required_start() {
        // Uppercase in present means that letter must be at position 0
        let mut config = Config::new().with_letters("Ware").with_present("W");
        config.case_sensitive = Some(true);
        config.minimal_word_length = Some(3);

        let solver = Solver::new(config);
        let dict = Dictionary::from_words(&["war", "raw", "ware", "area", "era"]);

        let results = solver.solve(&dict).expect("Solver failed");

        assert!(
            results.contains("war"),
            "starts with w, w is required start"
        );
        assert!(!results.contains("raw"), "w not at start");
        assert!(results.contains("ware"), "starts with w");
        assert!(!results.contains("area"), "does not start with w");
        assert!(!results.contains("era"), "does not start with w");
    }

    #[test]
    fn test_solver_case_sensitive_both_cases() {
        // Both 'W' (start-only) and 'w' (anywhere) in letters
        let mut config = Config::new().with_letters("Wware").with_present("a");
        config.case_sensitive = Some(true);
        config.minimal_word_length = Some(3);

        let solver = Solver::new(config);
        let dict = Dictionary::from_words(&["war", "raw", "ware", "awe"]);

        let results = solver.solve(&dict).expect("Solver failed");

        // Both 'W' (start-only) and 'w' (anywhere) contribute to allowed.
        // 'w' (lowercase) is in anywhere, so w can appear at any position.
        assert!(results.contains("war"));
        assert!(results.contains("raw"), "lowercase w allows w anywhere");
        assert!(results.contains("ware"));
        assert!(results.contains("awe"), "lowercase w allows w anywhere");
    }

    #[test]
    fn test_solver_case_sensitive_walrus_scenario() {
        let mut config = Config::new().with_letters("Walrus").with_present("Wl");
        config.case_sensitive = Some(true);
        config.minimal_word_length = Some(4);

        let solver = Solver::new(config);
        let dict =
            Dictionary::from_words(&["awls", "laws", "slaw", "wall", "walls", "walrus", "lure"]);

        let results = solver.solve(&dict).expect("Solver failed");

        assert!(
            !results.contains("awls"),
            "awls does not start with w"
        );
        assert!(!results.contains("laws"), "laws does not start with w");
        assert!(!results.contains("slaw"), "slaw does not start with w");
        assert!(!results.contains("lure"), "lure does not contain l... wait it does, but no w start");
        assert!(results.contains("wall"), "wall starts with w and contains l");
        assert!(results.contains("walls"), "walls starts with w and contains l");
        assert!(results.contains("walrus"), "walrus starts with w and contains l");
    }

    #[test]
    fn test_solver_case_sensitive_all_lowercase_no_constraint() {
        // When case_sensitive=true but all letters are lowercase, no positional constraint applies
        let mut config = Config::new().with_letters("walrus").with_present("wl");
        config.case_sensitive = Some(true);
        config.minimal_word_length = Some(4);

        let solver = Solver::new(config);
        let dict = Dictionary::from_words(&["awls", "laws", "wall", "walrus"]);

        let results = solver.solve(&dict).expect("Solver failed");

        assert!(
            results.contains("awls"),
            "all lowercase: w is an anywhere letter, no start constraint"
        );
        assert!(
            results.contains("laws"),
            "all lowercase: w is an anywhere letter"
        );
        assert!(results.contains("wall"));
        assert!(results.contains("walrus"));
    }

    #[test]
    fn test_solver_case_sensitive_multiple_uppercase_required_error() {
        let mut config = Config::new().with_letters("ABcde").with_present("AB");
        config.case_sensitive = Some(true);

        let solver = Solver::new(config);
        let dict = Dictionary::from_words(&["abcd"]);

        let result = solver.solve(&dict);
        assert!(
            result.is_err(),
            "multiple uppercase required letters should error"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("At most one uppercase"));
    }
}
