use thiserror::Error;

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
