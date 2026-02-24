//! GBA Core - Core execution engine for Claude Agent SDK wrapper.
//!
//! This crate provides the core functionality for interacting with the Claude Agent SDK,
//! enabling users to easily add new functionality around a repository.

#![warn(rust_2024_compatibility, missing_docs, missing_debug_implementations)]

pub mod agent;
pub mod error;

pub use agent::Agent;
pub use error::{CoreError, Result};

/// Re-export common types for convenience.
pub mod prelude {
    pub use crate::{Agent, CoreError, Result};
}
