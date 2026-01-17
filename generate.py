import os
import stat

def create_file(path, content):
    """Creates a file at the given path with the specified content."""
    directory = os.path.dirname(path)
    if directory and not os.path.exists(directory):
        os.makedirs(directory)
    with open(path, "w") as f:
        f.write(content)
    
    # Make shell scripts executable
    if path.endswith(".sh"):
        st = os.stat(path)
        os.chmod(path, st.st_mode | stat.S_IEXEC)
        
    print(f"Created: {path}")

def generate_project():
    """Generates the Spelling Bee Solver project structure."""

    # --- 1. Cargo.toml ---
    cargo_toml = """[package]
name = "sbs"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Spelling Bee Solver Core Library"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
reqwest = { version = "0.11", features = ["blocking", "json"] }
thiserror = "1.0"
log = "0.4"
clap = { version = "4.0", features = ["derive"] } # Added for CLI later

[dev-dependencies]
tempfile = "3.3"
"""

    # --- 2. Makefile ---
    makefile = """# Build System for Spelling Bee Solver

PROJECT_NAME = sbs
CARGO = cargo

.PHONY: all build setup test clean format lint doc release help

all: setup format build test

setup: ## Download default dictionary data
\t@echo "Setting up dictionary..."
\t@./scripts/setup_dictionary.sh

build: ## Build the project in debug mode
\t$(CARGO) build

release: ## Build the project in release mode
\t$(CARGO) build --release

test: ## Run unit and integration tests
\t$(CARGO) test

format: ## Format the code using rustfmt
\t$(CARGO) fmt

lint: ## Lint the code using clippy
\t$(CARGO) clippy -- -D warnings

clean: ## Clean build artifacts and data
\t$(CARGO) clean
\trm -f data/dictionary.txt

doc: ## Generate documentation
\t$(CARGO) doc --open

help: ## Display this help screen
\t@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\\033[36m%-20s\\033[0m %s\\n", $$1, $$2}'
"""

    # --- 3. Scripts: setup_dictionary.sh ---
    setup_dict_sh = """#!/bin/bash
# setup_dictionary.sh
# Downloads a standard English word list for the solver

DATA_DIR="data"
DICT_FILE="$DATA_DIR/dictionary.txt"
# Using words_alpha.txt (only letters, no numbers/symbols)
URL="https://raw.githubusercontent.com/dwyl/english-words/master/words_alpha.txt"

mkdir -p "$DATA_DIR"

if [ -f "$DICT_FILE" ]; then
    echo "Dictionary already exists at $DICT_FILE"
else
    echo "Downloading dictionary from $URL..."
    if command -v curl >/dev/null 2>&1; then
        curl -L -o "$DICT_FILE" "$URL"
    elif command -v wget >/dev/null 2>&1; then
        wget -O "$DICT_FILE" "$URL"
    else
        echo "Error: Neither curl nor wget found. Please download $URL to $DICT_FILE manually."
        exit 1
    fi
    echo "Dictionary downloaded successfully."
fi
"""

    # --- 4. README.md ---
    readme_md = """
# Spelling Bee Solver (SBS)

## Overview
A high-performance library and tool to solve Spelling Bee puzzles.

## Architecture
The core logic relies on a Trie-based recursive backtracking algorithm to efficiently filter valid words from a seed dictionary.

### Data Flow
1. **Bootstrap**: System loads a local dictionary (default: `data/dictionary.txt`) into a Trie.
2. **Input**: User provides config (letters, center letter).
3. **Process**: Recursive search on the Trie.
4. **Validation (Optional)**: Candidates checked against external API if configured.

## Usage

### Setup
Download the default dictionary:
```bash
make setup
```

### Building
```bash
make build
```

### Testing
```bash
make test
```

## Configuration
Configuration is handled via JSON/YAML files. See `tests/fixtures` for examples.

## License
MIT
"""

    # --- 5. .gitignore ---
    gitignore = """
/target
/data/dictionary.txt
**/*.rs.bk
Cargo.lock
.idea
.vscode
.DS_Store
"""

    # --- 6. src/lib.rs ---
    lib_rs = """//! Core library for the Spelling Bee Solver.

pub mod config;
pub mod dictionary;
pub mod solver;
pub mod error;

pub use config::Config;
pub use solver::Solver;
pub use error::SbsError;
"""

    # --- 7. src/error.rs ---
    error_rs = """use thiserror::Error;

#[derive(Error, Debug)]
pub enum SbsError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Dictionary error: {0}")]
    DictionaryError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}
"""

    # --- 8. src/config.rs ---
    config_rs = """//! Configuration management.

use serde::{Deserialize, Serialize};
use crate::error::SbsError;
use std::path::{Path, PathBuf};
use std::fs;

const DEFAULT_MIN_LENGTH: usize = 4;
const DEFAULT_SIZE: usize = 7;
const DEFAULT_DICT_PATH: &str = "data/dictionary.txt";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DictionaryConfig {
    pub id: String,
    pub name: String,
    pub api: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub letters: Option<String>,
    pub present: Option<String>, // The obligatory letter(s)
    pub size: Option<usize>,
    #[serde(rename = "minimal-word-length")]
    pub minimal_word_length: Option<usize>,
    #[serde(rename = "maximal-word-length")]
    pub maximal_word_length: Option<usize>,
    pub output: Option<String>,
    pub repeats: Option<usize>,
    
    // Path to the seed dictionary for generation
    #[serde(default = "default_dict_path")]
    pub dictionary: PathBuf,
    
    // External APIs for validation
    pub external_dictionaries: Option<Vec<DictionaryConfig>>,
}

fn default_dict_path() -> PathBuf {
    PathBuf::from(DEFAULT_DICT_PATH)
}

impl Config {
    pub fn new() -> Self {
        Self {
            letters: None,
            present: None,
            size: Some(DEFAULT_SIZE),
            minimal_word_length: Some(DEFAULT_MIN_LENGTH),
            maximal_word_length: None,
            output: None,
            repeats: None,
            dictionary: default_dict_path(),
            external_dictionaries: None,
        }
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, SbsError> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)
            .map_err(|e| SbsError::SerializationError(e.to_string()))?;
        Ok(config)
    }

    /// Fluent API: Set letters
    pub fn with_letters(mut self, letters: &str) -> Self {
        self.letters = Some(letters.to_string());
        self
    }

    /// Fluent API: Set obligatory letter
    pub fn with_present(mut self, present: &str) -> Self {
        self.present = Some(present.to_string());
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
"""

    # --- 9. src/solver.rs ---
    solver_rs = """//! The algorithmic core: Trie-based solver.

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
"""

    # --- 10. src/dictionary.rs ---
    dictionary_rs = """//! Handles external dictionary API interactions.

use crate::config::DictionaryConfig;
use crate::error::SbsError;

pub struct DictionaryClient {
    config: DictionaryConfig,
}

impl DictionaryClient {
    pub fn new(config: DictionaryConfig) -> Self {
        Self { config }
    }

    pub fn validate(&self, _word: &str) -> Result<bool, SbsError> {
        // Placeholder for external API validation
        Ok(true)
    }
}
"""

    # --- Create Files ---
    create_file("Cargo.toml", cargo_toml)
    create_file("Makefile", makefile)
    create_file("scripts/setup_dictionary.sh", setup_dict_sh)
    create_file("README.md", readme_md)
    create_file(".gitignore", gitignore)
    create_file("src/lib.rs", lib_rs)
    create_file("src/config.rs", config_rs)
    create_file("src/solver.rs", solver_rs)
    create_file("src/dictionary.rs", dictionary_rs)
    create_file("src/error.rs", error_rs)

    print("\\nProject structure generated successfully.")

if __name__ == "__main__":
    generate_project()