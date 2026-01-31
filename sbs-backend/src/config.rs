//! Configuration management.

use crate::error::SbsError;
use crate::validator::ValidatorKind;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

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

    // Validator selection
    pub validator: Option<ValidatorKind>,
    #[serde(rename = "api-key")]
    pub api_key: Option<String>,
    #[serde(rename = "validator-url")]
    pub validator_url: Option<String>,
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
            validator: None,
            api_key: None,
            validator_url: None,
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
