//! Task execution logic for GBA Core.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Task execution context.
///
/// This context provides information about the repository, files, and metadata
/// needed for task execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    /// Repository path.
    pub repository_path: PathBuf,

    /// Repository branch.
    pub branch: String,

    /// Files to include in the context.
    #[serde(default)]
    pub files: Vec<File>,

    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// File representation in task context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct File {
    /// File path relative to repository root.
    pub path: PathBuf,

    /// File content.
    pub content: String,

    /// File language (for syntax highlighting/analysis).
    #[serde(default)]
    pub language: String,
}

/// Agent response.
///
/// Represents the response from the agent after executing a task.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    /// Response content.
    #[serde(default)]
    pub content: String,

    /// Tool calls made during execution.
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,

    /// Usage statistics.
    #[serde(default)]
    pub usage: Usage,
}

/// Tool call made during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    /// Tool name.
    pub name: String,

    /// Tool arguments.
    pub arguments: serde_json::Value,
}

/// Usage statistics for the response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    /// Input tokens used.
    #[serde(default)]
    pub input_tokens: u32,

    /// Output tokens used.
    #[serde(default)]
    pub output_tokens: u32,

    /// Total cost in USD.
    #[serde(default)]
    pub total_cost_usd: f64,
}

/// Task for execution.
///
/// Represents a task to be executed by the agent.
#[derive(Debug, Clone)]
pub struct Task {
    /// Task prompt.
    pub prompt: String,

    /// Task context.
    pub context: Context,

    /// System prompt to use.
    pub system_prompt: String,

    /// Maximum turns for this task.
    pub max_turns: u32,
}

impl Task {
    /// Create a new task.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The task prompt.
    /// * `context` - The task context.
    ///
    /// # Examples
    ///
    /// ```
    /// use gba_core::{Context, Task};
    ///
    /// let context = Context {
    ///     repository_path: "/path/to/repo".into(),
    ///     branch: "main".to_string(),
    ///     files: vec![],
    ///     metadata: Default::default(),
    /// };
    ///
    /// let task = Task::new(
    ///     "Implement feature X".to_string(),
    ///     context,
    ///     "Default system prompt".to_string(),
    ///     100,
    /// );
    /// ```
    #[must_use]
    pub const fn new(
        prompt: String,
        context: Context,
        system_prompt: String,
        max_turns: u32,
    ) -> Self {
        Self {
            prompt,
            context,
            system_prompt,
            max_turns,
        }
    }

    /// Create a new task with default system prompt and max turns.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The task prompt.
    /// * `context` - The task context.
    ///
    /// # Examples
    ///
    /// ```
    /// use gba_core::{Context, Task};
    ///
    /// let context = Context {
    ///     repository_path: "/path/to/repo".into(),
    ///     branch: "main".to_string(),
    ///     files: vec![],
    ///     metadata: Default::default(),
    /// };
    ///
    /// let task = Task::with_defaults("Implement feature X", context);
    /// ```
    #[must_use]
    pub fn with_defaults(prompt: impl Into<String>, context: Context) -> Self {
        Self::new(
            prompt.into(),
            context,
            "You are an expert software development assistant.".to_string(),
            100,
        )
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            repository_path: PathBuf::new(),
            branch: "main".to_string(),
            files: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let context = Context {
            repository_path: "/path/to/repo".into(),
            branch: "main".to_string(),
            files: vec![],
            metadata: HashMap::new(),
        };

        let task = Task::new(
            "Implement feature".to_string(),
            context.clone(),
            "System prompt".to_string(),
            50,
        );

        assert_eq!(task.prompt, "Implement feature");
        assert_eq!(task.max_turns, 50);
    }

    #[test]
    fn test_task_with_defaults() {
        let context = Context::default();
        let task = Task::with_defaults("Test prompt", context);

        assert_eq!(task.prompt, "Test prompt");
        assert_eq!(
            task.system_prompt,
            "You are an expert software development assistant."
        );
        assert_eq!(task.max_turns, 100);
    }

    #[test]
    fn test_context_default() {
        let context = Context::default();
        assert_eq!(context.branch, "main");
        assert!(context.files.is_empty());
        assert!(context.metadata.is_empty());
    }

    #[test]
    fn test_response_serialization() {
        let response = Response {
            content: "Test response".to_string(),
            tool_calls: vec![ToolCall {
                name: "Read".to_string(),
                arguments: serde_json::json!({"path": "test.rs"}),
            }],
            usage: Usage {
                input_tokens: 100,
                output_tokens: 50,
                total_cost_usd: 0.01,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: Response = serde_json::from_str(&json).unwrap();

        assert_eq!(response.content, deserialized.content);
        assert_eq!(response.tool_calls.len(), deserialized.tool_calls.len());
        assert_eq!(response.usage.input_tokens, deserialized.usage.input_tokens);
    }
}
