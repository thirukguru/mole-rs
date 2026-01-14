//! Core module - shared utilities and types

pub mod config;
pub mod errors;
pub mod filesystem;
pub mod paths;
pub mod security;
pub mod system;

#[cfg(test)]
mod tests;

pub use config::Config;
pub use errors::{MoleError, Result};
pub use paths::CleanupPaths;
pub use security::{SecurityValidator, PathValidation};
