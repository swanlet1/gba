//! Error types for GBA CLI.

use thiserror::Error;

/// Result type alias for GBA CLI.
pub type Result<T> = std::result::Result<T, CliError>;

/// CLI error types.
#[derive(Debug, Error)]
#[allow(dead_code)]
pub enum CliError {
    /// Error from GBA Core.
    #[error("Core error: {0}")]
    Core(#[from] gba_core::CoreError),

    /// Error from GBA Prompt Manager.
    #[error("Prompt manager error: {0}")]
    Prompt(#[from] gba_pm::PromptError),

    /// Error from configuration operations.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Error from IO operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Error from argument parsing.
    #[error("Invalid arguments: {0}")]
    InvalidArgs(String),

    /// User canceled operation.
    #[error("Operation canceled by user")]
    Canceled,

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

impl CliError {
    /// Create a configuration error.
    #[must_use]
    #[allow(dead_code)]
    pub const fn config(message: String) -> Self {
        Self::Config(message)
    }

    /// Create an invalid arguments error.
    #[must_use]
    #[allow(dead_code)]
    pub const fn invalid_args(message: String) -> Self {
        Self::InvalidArgs(message)
    }

    /// Create an internal error.
    #[must_use]
    #[allow(dead_code)]
    pub const fn internal(message: String) -> Self {
        Self::Internal(message)
    }
}
