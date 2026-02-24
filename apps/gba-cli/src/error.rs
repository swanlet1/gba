//! Error types for GBA CLI.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for GBA CLI.
pub type Result<T> = std::result::Result<T, CliError>;

/// CLI error types.
#[derive(Debug, Error)]
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
    #[allow(dead_code)]
    InvalidArgs(String),

    /// User canceled operation.
    #[error("Operation canceled by user")]
    #[allow(dead_code)]
    Canceled,

    /// Internal error.
    #[error("Internal error: {0}")]
    #[allow(dead_code)]
    Internal(String),

    /// Template not found.
    #[error("Template '{0}' not found")]
    TemplateNotFound(String),

    /// Invalid template name.
    #[error("Invalid template name: {0}")]
    #[allow(dead_code)]
    InvalidTemplateName(String),

    /// Not a GBA project.
    #[error("Not a GBA project: {0}")]
    NotGbaProject(PathBuf),

    /// Feature state not found.
    #[error("Feature state not found: {0}")]
    #[allow(dead_code)]
    FeatureStateNotFound(String),

    /// Agent execution failed.
    #[error("Agent execution failed: {0}")]
    #[allow(dead_code)]
    ExecutionFailed(String),
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

    /// Create a template not found error.
    #[must_use]
    pub const fn template_not_found(name: String) -> Self {
        Self::TemplateNotFound(name)
    }

    /// Create an execution failed error.
    #[must_use]
    #[allow(dead_code)]
    pub const fn execution_failed(message: String) -> Self {
        Self::ExecutionFailed(message)
    }
}

impl From<crate::config::ConfigLoadError> for CliError {
    fn from(err: crate::config::ConfigLoadError) -> Self {
        match err {
            crate::config::ConfigLoadError::NotFound(path) => {
                Self::Config(format!("Configuration file not found: {}", path.display()))
            }
            crate::config::ConfigLoadError::LoadError(e) => Self::Config(e.to_string()),
            crate::config::ConfigLoadError::InvalidPath(path) => {
                Self::Config(format!("Invalid project path: {}", path.display()))
            }
            crate::config::ConfigLoadError::NotGbaProject(path) => Self::NotGbaProject(path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_error_display() {
        let err = CliError::Config("test error".to_string());
        assert_eq!(err.to_string(), "Configuration error: test error");

        let err = CliError::TemplateNotFound("test".to_string());
        assert_eq!(err.to_string(), "Template 'test' not found");
    }
}
