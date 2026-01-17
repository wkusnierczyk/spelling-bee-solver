//! Core library for the Spelling Bee Solver.

pub mod config;
pub mod dictionary;
pub mod solver;
pub mod error;

pub use config::Config;
pub use solver::Solver;
pub use error::SbsError;
