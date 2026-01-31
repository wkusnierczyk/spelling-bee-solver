//! External dictionary validation and lookup.

use crate::error::SbsError;
use serde::{Deserialize, Serialize};

/// A validated word entry with definition and reference URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordEntry {
    pub word: String,
    pub definition: String,
    pub url: String,
}

/// Supported external dictionary validators.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ValidatorKind {
    FreeDictionary,
    MerriamWebster,
    Wordnik,
    Custom,
}

impl ValidatorKind {
    pub fn display_name(&self) -> &str {
        match self {
            ValidatorKind::FreeDictionary => "Free Dictionary",
            ValidatorKind::MerriamWebster => "Merriam-Webster",
            ValidatorKind::Wordnik => "Wordnik",
            ValidatorKind::Custom => "Custom",
        }
    }
}

impl std::str::FromStr for ValidatorKind {
    type Err = SbsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "free-dictionary" => Ok(ValidatorKind::FreeDictionary),
            "merriam-webster" => Ok(ValidatorKind::MerriamWebster),
            "wordnik" => Ok(ValidatorKind::Wordnik),
            "custom" => Ok(ValidatorKind::Custom),
            _ => Err(SbsError::ValidationError(format!(
                "Unknown validator: '{}'. Valid options: free-dictionary, merriam-webster, wordnik, custom",
                s
            ))),
        }
    }
}

/// Trait for external dictionary validators.
pub trait Validator: Send + Sync {
    fn name(&self) -> &str;
    fn lookup(&self, word: &str) -> Result<Option<WordEntry>, SbsError>;
}

/// Free Dictionary API validator (no API key required).
pub struct FreeDictionaryValidator {
    base_url: String,
}

impl FreeDictionaryValidator {
    pub fn new() -> Self {
        Self {
            base_url: "https://api.dictionaryapi.dev/api/v2/entries/en".to_string(),
        }
    }

    pub fn with_base_url(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }
}

impl Default for FreeDictionaryValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Validator for FreeDictionaryValidator {
    fn name(&self) -> &str {
        "Free Dictionary"
    }

    fn lookup(&self, word: &str) -> Result<Option<WordEntry>, SbsError> {
        let url = format!("{}/{}", self.base_url, word);
        let response = reqwest::blocking::get(&url)
            .map_err(|e| SbsError::ValidationError(format!("HTTP error: {}", e)))?;

        if response.status() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(SbsError::ValidationError(format!(
                "API returned status {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response
            .json()
            .map_err(|e| SbsError::ValidationError(format!("JSON parse error: {}", e)))?;

        let definition = body
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|entry| entry.get("meanings"))
            .and_then(|m| m.as_array())
            .and_then(|arr| arr.first())
            .and_then(|meaning| meaning.get("definitions"))
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .and_then(|def| def.get("definition"))
            .and_then(|d| d.as_str())
            .unwrap_or("No definition available")
            .to_string();

        let entry_url = format!("https://en.wiktionary.org/wiki/{}", word);

        Ok(Some(WordEntry {
            word: word.to_string(),
            definition,
            url: entry_url,
        }))
    }
}

/// Merriam-Webster API validator (requires free API key).
pub struct MerriamWebsterValidator {
    api_key: String,
}

impl MerriamWebsterValidator {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }
}

impl Validator for MerriamWebsterValidator {
    fn name(&self) -> &str {
        "Merriam-Webster"
    }

    fn lookup(&self, word: &str) -> Result<Option<WordEntry>, SbsError> {
        let url = format!(
            "https://dictionaryapi.com/api/v3/references/collegiate/json/{}?key={}",
            word, self.api_key
        );
        let response = reqwest::blocking::get(&url)
            .map_err(|e| SbsError::ValidationError(format!("HTTP error: {}", e)))?;

        if !response.status().is_success() {
            return Err(SbsError::ValidationError(format!(
                "API returned status {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response
            .json()
            .map_err(|e| SbsError::ValidationError(format!("JSON parse error: {}", e)))?;

        // Merriam-Webster returns an array of strings (suggestions) if word not found,
        // or an array of objects if found.
        let arr = body
            .as_array()
            .ok_or_else(|| SbsError::ValidationError("Unexpected response format".to_string()))?;

        if arr.is_empty() {
            return Ok(None);
        }

        // If first element is a string, word was not found (suggestions returned).
        if arr[0].is_string() {
            return Ok(None);
        }

        let definition = arr[0]
            .get("shortdef")
            .and_then(|sd| sd.as_array())
            .and_then(|arr| arr.first())
            .and_then(|d| d.as_str())
            .unwrap_or("No definition available")
            .to_string();

        let entry_url = format!("https://www.merriam-webster.com/dictionary/{}", word);

        Ok(Some(WordEntry {
            word: word.to_string(),
            definition,
            url: entry_url,
        }))
    }
}

/// Wordnik API validator (requires free API key).
pub struct WordnikValidator {
    api_key: String,
}

impl WordnikValidator {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }
}

impl Validator for WordnikValidator {
    fn name(&self) -> &str {
        "Wordnik"
    }

    fn lookup(&self, word: &str) -> Result<Option<WordEntry>, SbsError> {
        let url = format!(
            "https://api.wordnik.com/v4/word.json/{}/definitions?limit=1&api_key={}",
            word, self.api_key
        );
        let response = reqwest::blocking::get(&url)
            .map_err(|e| SbsError::ValidationError(format!("HTTP error: {}", e)))?;

        if response.status() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            return Err(SbsError::ValidationError(format!(
                "API returned status {}",
                response.status()
            )));
        }

        let body: serde_json::Value = response
            .json()
            .map_err(|e| SbsError::ValidationError(format!("JSON parse error: {}", e)))?;

        let arr = match body.as_array() {
            Some(a) if !a.is_empty() => a,
            _ => return Ok(None),
        };

        let definition = arr[0]
            .get("text")
            .and_then(|t| t.as_str())
            .unwrap_or("No definition available")
            .to_string();

        let entry_url = format!("https://www.wordnik.com/words/{}", word);

        Ok(Some(WordEntry {
            word: word.to_string(),
            definition,
            url: entry_url,
        }))
    }
}

/// Custom URL validator (assumes Free Dictionary API-compatible JSON format).
pub struct CustomValidator {
    base_url: String,
}

impl CustomValidator {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Probe the custom URL to check if it returns valid dictionary responses.
    pub fn probe(&self) -> Result<bool, SbsError> {
        let test_url = format!("{}/test", self.base_url);
        let response = reqwest::blocking::get(&test_url)
            .map_err(|e| SbsError::ValidationError(format!("Probe failed: {}", e)))?;

        if !response.status().is_success() {
            return Ok(false);
        }

        let body: serde_json::Value = response
            .json()
            .map_err(|_| SbsError::ValidationError("Probe: invalid JSON response".to_string()))?;

        // Check if response looks like a dictionary entry (array with meanings)
        let looks_valid = body
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|entry| entry.get("meanings"))
            .is_some();

        Ok(looks_valid)
    }
}

impl Validator for CustomValidator {
    fn name(&self) -> &str {
        "Custom"
    }

    fn lookup(&self, word: &str) -> Result<Option<WordEntry>, SbsError> {
        // Reuse Free Dictionary parsing logic since custom validators are expected
        // to be API-compatible.
        let inner = FreeDictionaryValidator::with_base_url(&self.base_url);
        inner.lookup(word)
    }
}

/// Create a boxed validator from a kind, API key, and optional custom URL.
pub fn create_validator(
    kind: &ValidatorKind,
    api_key: Option<&str>,
    custom_url: Option<&str>,
) -> Result<Box<dyn Validator>, SbsError> {
    match kind {
        ValidatorKind::FreeDictionary => Ok(Box::new(FreeDictionaryValidator::new())),
        ValidatorKind::MerriamWebster => {
            let key = api_key.ok_or_else(|| {
                SbsError::ValidationError(
                    "Merriam-Webster requires an API key (--api-key)".to_string(),
                )
            })?;
            Ok(Box::new(MerriamWebsterValidator::new(key)))
        }
        ValidatorKind::Wordnik => {
            let key = api_key.ok_or_else(|| {
                SbsError::ValidationError("Wordnik requires an API key (--api-key)".to_string())
            })?;
            Ok(Box::new(WordnikValidator::new(key)))
        }
        ValidatorKind::Custom => {
            let url = custom_url.ok_or_else(|| {
                SbsError::ValidationError(
                    "Custom validator requires a URL (--validator-url)".to_string(),
                )
            })?;
            let validator = CustomValidator::new(url);
            if !validator.probe()? {
                return Err(SbsError::ValidationError(format!(
                    "Custom URL '{}' does not appear to be a compatible dictionary API. \
                     Expected Free Dictionary API-compatible JSON format.",
                    url
                )));
            }
            Ok(Box::new(validator))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_kind_from_str() {
        assert_eq!(
            "free-dictionary".parse::<ValidatorKind>().unwrap(),
            ValidatorKind::FreeDictionary
        );
        assert_eq!(
            "merriam-webster".parse::<ValidatorKind>().unwrap(),
            ValidatorKind::MerriamWebster
        );
        assert_eq!(
            "wordnik".parse::<ValidatorKind>().unwrap(),
            ValidatorKind::Wordnik
        );
        assert_eq!(
            "custom".parse::<ValidatorKind>().unwrap(),
            ValidatorKind::Custom
        );
        assert!("unknown".parse::<ValidatorKind>().is_err());
    }

    #[test]
    fn test_validator_kind_display_name() {
        assert_eq!(
            ValidatorKind::FreeDictionary.display_name(),
            "Free Dictionary"
        );
        assert_eq!(
            ValidatorKind::MerriamWebster.display_name(),
            "Merriam-Webster"
        );
        assert_eq!(ValidatorKind::Wordnik.display_name(), "Wordnik");
        assert_eq!(ValidatorKind::Custom.display_name(), "Custom");
    }

    #[test]
    fn test_word_entry_serialization() {
        let entry = WordEntry {
            word: "test".to_string(),
            definition: "A procedure for evaluation".to_string(),
            url: "https://example.com/test".to_string(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"word\":\"test\""));
        assert!(json.contains("\"definition\""));
        assert!(json.contains("\"url\""));

        let deserialized: WordEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.word, "test");
    }

    #[test]
    fn test_create_validator_free_dictionary() {
        let v = create_validator(&ValidatorKind::FreeDictionary, None, None).unwrap();
        assert_eq!(v.name(), "Free Dictionary");
    }

    #[test]
    fn test_create_validator_merriam_webster_requires_key() {
        let result = create_validator(&ValidatorKind::MerriamWebster, None, None);
        assert!(result.is_err());

        let v = create_validator(&ValidatorKind::MerriamWebster, Some("test-key"), None).unwrap();
        assert_eq!(v.name(), "Merriam-Webster");
    }

    #[test]
    fn test_create_validator_wordnik_requires_key() {
        let result = create_validator(&ValidatorKind::Wordnik, None, None);
        assert!(result.is_err());

        let v = create_validator(&ValidatorKind::Wordnik, Some("test-key"), None).unwrap();
        assert_eq!(v.name(), "Wordnik");
    }

    #[test]
    fn test_create_validator_custom_requires_url() {
        let result = create_validator(&ValidatorKind::Custom, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_validator_kind_serde_roundtrip() {
        let kind = ValidatorKind::FreeDictionary;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"free-dictionary\"");
        let deserialized: ValidatorKind = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, kind);
    }
}
