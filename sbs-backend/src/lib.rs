//! Core library for the Spelling Bee Solver.

pub mod config;
pub mod dictionary;
pub mod error;
pub mod solver;
#[cfg(feature = "validator")]
pub mod validator;

pub use config::Config;
pub use dictionary::Dictionary;
pub use error::SbsError;
pub use solver::Solver;
#[cfg(feature = "validator")]
pub use validator::{
    create_validator, CustomValidator, FreeDictionaryValidator, MerriamWebsterValidator,
    ValidationSummary, Validator, ValidatorKind, WordEntry, WordnikValidator,
};
