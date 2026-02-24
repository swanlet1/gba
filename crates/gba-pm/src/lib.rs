//! GBA Prompt Manager - Template-based prompt management using Minijinja.
//!
//! This crate provides functionality for managing and rendering prompts using
//! the Minijinja templating engine.

#![warn(rust_2024_compatibility, missing_docs, missing_debug_implementations)]

pub mod config;
pub mod error;
pub mod prompt;
pub mod template;

pub use config::{Context, FileContext, PromptTemplate, TemplateConfig};
pub use error::{PromptError, Result};
pub use prompt::PromptManager;
pub use template::TemplateEngine;

/// Re-export common types for convenience.
pub mod prelude {
    pub use crate::{
        Context, FileContext, PromptError, PromptManager, PromptTemplate, Result, TemplateConfig,
        TemplateEngine,
    };
}
