//! Core library for the Spelling Bee Solver.

pub mod config;
pub mod dictionary;
pub mod error;
pub mod solver;
pub mod validator;

pub use config::Config;
pub use dictionary::Dictionary;
pub use error::SbsError;
pub use solver::Solver;
pub use validator::{
    create_validator, CustomValidator, FreeDictionaryValidator, MerriamWebsterValidator,
    ValidationSummary, ValidatorKind, WordEntry, WordnikValidator,
};
