//! Core library for the Spelling Bee Solver.

pub mod config;
pub mod dictionary;
pub mod error;
pub mod solver;

pub use config::Config;
pub use error::SbsError;
pub use solver::Solver;
