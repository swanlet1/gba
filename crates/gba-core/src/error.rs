//! Error types for GBA Core.

use thiserror::Error;

/// Result type alias for GBA Core.
pub type Result<T> = std::result::Result<T, CoreError>;

/// Core error types.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Error from Claude Agent SDK.
    #[error("Claude Agent SDK error: {0}")]
    ClaudeAgent(String),

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
