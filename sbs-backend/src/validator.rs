//! External dictionary validation and lookup.

use crate::error::SbsError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// HTTP request timeout for validator API calls.
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

/// Delay between consecutive API calls to avoid rate limiting.
const THROTTLE_DELAY: Duration = Duration::from_millis(100);

/// A validated word entry with definition and reference URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordEntry {
    pub word: String,
    pub definition: String,
    pub url: String,
}

/// Summary of validation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub candidates: usize,
    pub validated: usize,
    pub entries: Vec<WordEntry>,
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

/// Build a shared HTTP client with timeout.
fn http_client() -> Result<reqwest::blocking::Client, SbsError> {
    reqwest::blocking::Client::builder()
        .timeout(HTTP_TIMEOUT)
        .build()
        .map_err(|e| SbsError::ValidationError(format!("Failed to create HTTP client: {}", e)))
}

/// Trait for external dictionary validators.
pub trait Validator: Send + Sync {
    fn name(&self) -> &str;
    fn lookup(&self, word: &str) -> Result<Option<WordEntry>, SbsError>;

    /// Validate a list of words with throttling. Returns a summary with counts.
    fn validate_words(&self, words: &[String]) -> ValidationSummary {
        self.validate_words_with_progress(words, &|_, _| {})
    }

    /// Validate a list of words with throttling and progress callback.
    fn validate_words_with_progress(
        &self,
        words: &[String],
        on_progress: &dyn Fn(usize, usize),
    ) -> ValidationSummary {
        let candidates = words.len();
        let mut entries = Vec::new();
        for (i, word) in words.iter().enumerate() {
            if i > 0 {
                std::thread::sleep(THROTTLE_DELAY);
            }
            match self.lookup(word) {
                Ok(Some(entry)) => entries.push(entry),
                Ok(None) => {}
                Err(e) => {
                    log::warn!("Validation error for '{}': {}", word, e);
                }
            }
            on_progress(i + 1, candidates);
        }
        let validated = entries.len();
        ValidationSummary {
            candidates,
            validated,
            entries,
        }
    }
}

/// Free Dictionary API validator (no API key required).
pub struct FreeDictionaryValidator {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl FreeDictionaryValidator {
    pub fn new() -> Result<Self, SbsError> {
        Ok(Self {
            base_url: "https://api.dictionaryapi.dev/api/v2/entries/en".to_string(),
            client: http_client()?,
        })
    }

    pub fn with_base_url(base_url: &str) -> Result<Self, SbsError> {
        Ok(Self {
            base_url: base_url.to_string(),
            client: http_client()?,
        })
    }
}

impl Validator for FreeDictionaryValidator {
    fn name(&self) -> &str {
        "Free Dictionary"
    }

    fn lookup(&self, word: &str) -> Result<Option<WordEntry>, SbsError> {
        let url = format!("{}/{}", self.base_url, word);
        let response = self
            .client
            .get(&url)
            .send()
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
    client: reqwest::blocking::Client,
}

impl MerriamWebsterValidator {
    pub fn new(api_key: &str) -> Result<Self, SbsError> {
        Ok(Self {
            api_key: api_key.to_string(),
            client: http_client()?,
        })
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
        let response = self
            .client
            .get(&url)
            .send()
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
    client: reqwest::blocking::Client,
}

impl WordnikValidator {
    pub fn new(api_key: &str) -> Result<Self, SbsError> {
        Ok(Self {
            api_key: api_key.to_string(),
            client: http_client()?,
        })
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
        let response = self
            .client
            .get(&url)
            .send()
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
    client: reqwest::blocking::Client,
}

impl CustomValidator {
    pub fn new(base_url: &str) -> Result<Self, SbsError> {
        Ok(Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: http_client()?,
        })
    }

    /// Probe the custom URL to check if it returns valid dictionary responses.
    pub fn probe(&self) -> Result<bool, SbsError> {
        let test_url = format!("{}/test", self.base_url);
        let response = self
            .client
            .get(&test_url)
            .send()
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
        let inner = FreeDictionaryValidator::with_base_url(&self.base_url)?;
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
        ValidatorKind::FreeDictionary => Ok(Box::new(FreeDictionaryValidator::new()?)),
        ValidatorKind::MerriamWebster => {
            let key = api_key.ok_or_else(|| {
                SbsError::ValidationError(
                    "Merriam-Webster requires an API key (--api-key)".to_string(),
                )
            })?;
            Ok(Box::new(MerriamWebsterValidator::new(key)?))
        }
        ValidatorKind::Wordnik => {
            let key = api_key.ok_or_else(|| {
                SbsError::ValidationError("Wordnik requires an API key (--api-key)".to_string())
            })?;
            Ok(Box::new(WordnikValidator::new(key)?))
        }
        ValidatorKind::Custom => {
            let url = custom_url.ok_or_else(|| {
                SbsError::ValidationError(
                    "Custom validator requires a URL (--validator-url)".to_string(),
                )
            })?;
            let validator = CustomValidator::new(url)?;
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

    #[test]
    fn test_validation_summary_serialization() {
        let summary = ValidationSummary {
            candidates: 10,
            validated: 3,
            entries: vec![WordEntry {
                word: "test".to_string(),
                definition: "A trial".to_string(),
                url: "https://example.com/test".to_string(),
            }],
        };
        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"candidates\":10"));
        assert!(json.contains("\"validated\":3"));
    }

    /// Mock validator for testing validate_words throttling and summary.
    struct MockValidator {
        known_words: Vec<String>,
    }

    impl Validator for MockValidator {
        fn name(&self) -> &str {
            "Mock"
        }

        fn lookup(&self, word: &str) -> Result<Option<WordEntry>, SbsError> {
            if self.known_words.contains(&word.to_string()) {
                Ok(Some(WordEntry {
                    word: word.to_string(),
                    definition: format!("Definition of {}", word),
                    url: format!("https://example.com/{}", word),
                }))
            } else {
                Ok(None)
            }
        }
    }

    #[test]
    fn test_validate_words_filters_and_counts() {
        let validator = MockValidator {
            known_words: vec!["apple".to_string(), "banana".to_string()],
        };

        let words = vec![
            "apple".to_string(),
            "xyzzy".to_string(),
            "banana".to_string(),
            "qqqqq".to_string(),
        ];

        let summary = validator.validate_words(&words);

        assert_eq!(summary.candidates, 4);
        assert_eq!(summary.validated, 2);
        assert_eq!(summary.entries.len(), 2);
        assert_eq!(summary.entries[0].word, "apple");
        assert_eq!(summary.entries[1].word, "banana");
    }

    #[test]
    fn test_validate_words_empty_input() {
        let validator = MockValidator {
            known_words: vec![],
        };

        let summary = validator.validate_words(&[]);
        assert_eq!(summary.candidates, 0);
        assert_eq!(summary.validated, 0);
        assert!(summary.entries.is_empty());
    }

    #[test]
    fn test_validate_words_none_validated() {
        let validator = MockValidator {
            known_words: vec![],
        };

        let words = vec!["foo".to_string(), "bar".to_string()];
        let summary = validator.validate_words(&words);

        assert_eq!(summary.candidates, 2);
        assert_eq!(summary.validated, 0);
        assert!(summary.entries.is_empty());
    }

    #[test]
    fn test_free_dictionary_parses_response() {
        // Test the JSON parsing logic directly by simulating a response body
        let json_body = serde_json::json!([{
            "word": "hello",
            "meanings": [{
                "partOfSpeech": "noun",
                "definitions": [{
                    "definition": "A greeting"
                }]
            }]
        }]);

        let definition = json_body
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
            .unwrap_or("No definition available");

        assert_eq!(definition, "A greeting");
    }

    #[test]
    fn test_merriam_webster_parses_found_response() {
        let json_body = serde_json::json!([{
            "shortdef": ["a greeting or expression of goodwill"]
        }]);

        let arr = json_body.as_array().unwrap();
        assert!(!arr[0].is_string()); // It's an object, so word was found

        let definition = arr[0]
            .get("shortdef")
            .and_then(|sd| sd.as_array())
            .and_then(|arr| arr.first())
            .and_then(|d| d.as_str())
            .unwrap_or("No definition available");

        assert_eq!(definition, "a greeting or expression of goodwill");
    }

    #[test]
    fn test_merriam_webster_parses_not_found_response() {
        // When word is not found, MW returns an array of suggestion strings
        let json_body = serde_json::json!(["hello", "hallo", "hullo"]);

        let arr = json_body.as_array().unwrap();
        assert!(arr[0].is_string()); // Suggestions = word not found
    }

    #[test]
    fn test_wordnik_parses_response() {
        let json_body = serde_json::json!([{
            "text": "Used as a greeting",
            "partOfSpeech": "interjection"
        }]);

        let arr = json_body.as_array().unwrap();
        let definition = arr[0]
            .get("text")
            .and_then(|t| t.as_str())
            .unwrap_or("No definition available");

        assert_eq!(definition, "Used as a greeting");
    }

    #[test]
    fn test_wordnik_empty_response_is_not_found() {
        let json_body = serde_json::json!([]);
        let arr = json_body.as_array().unwrap();
        assert!(arr.is_empty()); // Empty = not found
    }
}
