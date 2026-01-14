//! Error types for Mole-RS

use thiserror::Error;

pub type Result<T> = std::result::Result<T, MoleError>;

#[derive(Error, Debug)]
pub enum MoleError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("Path not found: {path}")]
    PathNotFound { path: String },

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Operation cancelled by user")]
    Cancelled,

    #[error("Requires elevated privileges (sudo)")]
    RequiresSudo,

    #[error("Command failed: {command} - {message}")]
    CommandFailed { command: String, message: String },

    #[error("{0}")]
    Other(String),
}
