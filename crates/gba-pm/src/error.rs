//! Error types for GBA Prompt Manager.

use thiserror::Error;

/// Result type alias for GBA Prompt Manager.
pub type Result<T> = std::result::Result<T, PromptError>;

/// Prompt manager error types.
#[derive(Debug, Error)]
pub enum PromptError {
    /// Error from Minijinja templating.
    #[error("Template error: {0}")]
    Template(String),

    /// Template not found.
    #[error("Template '{0}' not found")]
    NotFound(String),

    /// Invalid template syntax.
    #[error("Invalid template syntax: {0}")]
    InvalidSyntax(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
