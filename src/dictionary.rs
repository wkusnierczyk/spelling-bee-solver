//! Handles external dictionary API interactions.

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
