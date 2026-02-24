//! GBA Core - Core execution engine for Claude Agent SDK wrapper.
//!
//! This crate provides the core functionality for interacting with the Claude Agent SDK,
//! enabling users to easily add new functionality around a repository.

#![warn(rust_2024_compatibility, missing_docs, missing_debug_implementations)]

pub mod agent;
pub mod config;
pub mod context_builder;
pub mod error;
pub mod task;

pub use agent::{Agent, Chunk};
pub use config::{
    AgentConfig, ConfigError, LimitsConfig, LoggingConfig, ProjectConfig, ProjectMetadata,
    PromptsConfig, RepositoryConfig, RepositoryMetadata, WorktreeConfig,
};
pub use error::{CoreError, Result};
pub use task::{Context, Response, Task};

/// Re-export common types for convenience.
pub mod prelude {
    pub use crate::{
        Agent, AgentConfig, Chunk, Context, CoreError, ProjectConfig, Response, Result, Task,
    };
}
