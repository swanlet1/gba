//! Agent implementation for interacting with Claude Agent SDK.

use std::fmt;
use std::path::PathBuf;

use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ContentBlock, Message, PermissionMode, SettingSource, SystemPrompt, query,
};

use crate::config::AgentConfig;
use crate::context_builder::{ContextBuilderConfig, build_context};
use crate::error::{CoreError, Result};
use crate::task::{Context as TaskContext, Response, Task};

/// Agent for interacting with Claude Agent SDK.
///
/// The agent provides methods for executing tasks with prompts and context
/// using the Claude Agent SDK's simple query API.
///
/// # Examples
///
/// ```no_run
/// use gba_core::{Agent, AgentConfig};
/// use gba_core::task::Context;
///
/// #[tokio::main]
/// async fn main() -> Result<(), gba_core::CoreError> {
///     let config = AgentConfig::default();
///     let agent = Agent::new(config);
///
///     let response = agent.execute(
///         "Hello Claude",
///         &Context::default(),
///     ).await?;
///
///     println!("{}", response.content);
///
///     Ok(())
/// }
/// ```
pub struct Agent {
    /// Agent configuration.
    config: AgentConfig,
    /// Working directory for the agent.
    working_dir: PathBuf,
}

impl fmt::Debug for Agent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Agent")
            .field("working_dir", &self.working_dir)
            .field("config", &self.config)
            .finish()
    }
}

impl Agent {
    /// Create a new agent with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Agent configuration including model and other settings.
    ///
    /// # Errors
    ///
    /// Returns an error if the working directory cannot be determined.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gba_core::{Agent, AgentConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), gba_core::CoreError> {
    ///     let config = AgentConfig::default();
    ///     let agent = Agent::new(config);
    ///     Ok(())
    /// }
    /// ```
    #[tracing::instrument(skip(config))]
    pub fn new(config: AgentConfig) -> Self {
        let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        tracing::info!("Created agent with model: {}", config.model);

        Self { config, working_dir }
    }

    /// Execute a task with the given prompt and context.
    ///
    /// This method executes a task using the query API, collecting all
    /// messages before returning.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The task prompt to execute.
    /// * `context` - The task context containing repository information.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The query fails
    /// - The response cannot be parsed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gba_core::{Agent, AgentConfig, Context};
    /// use std::path::PathBuf;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), gba_core::CoreError> {
    ///     let config = AgentConfig::default();
    ///     let agent = Agent::new(config);
    ///
    ///     let context = Context {
    ///         repository_path: PathBuf::from("/path/to/repo"),
    ///         branch: "main".to_string(),
    ///         files: vec![],
    ///         metadata: Default::default(),
    ///     };
    ///
    ///     let response = agent.execute(
    ///         "Implement feature X",
    ///         &context,
    ///     ).await?;
    ///
    ///     println!("{}", response.content);
    ///     Ok(())
    /// }
    /// ```
    #[tracing::instrument(skip(self, prompt, context))]
    pub async fn execute(&self, prompt: &str, context: &TaskContext) -> Result<Response> {
        tracing::info!("Executing task with prompt: {}", prompt);

        // Build the full prompt with context
        let full_prompt = self.build_prompt(prompt, context);

        // Build options
        let options = Self::build_options(&self.config)?;

        // Send the query using the simple query API
        let messages = query(&full_prompt, Some(options))
            .await
            .map_err(|e| CoreError::ClaudeAgent(format!("Failed to send query: {e}")))?;

        // Collect all messages
        let mut response = Response::default();

        for message in &messages {
            match message {
                Message::User(user_msg) => {
                    // Track user messages if needed
                    if let Some(ref content) = user_msg.content {
                        for block in content {
                            if let ContentBlock::Text(text) = block {
                                tracing::debug!("User message: {}", text.text);
                            }
                        }
                    }
                }
                Message::Assistant(msg) => {
                    for block in &msg.message.content {
                        match block {
                            ContentBlock::Text(text) => {
                                response.content.push_str(&text.text);
                            }
                            ContentBlock::ToolUse(tool) => {
                                tracing::debug!("Tool used: {} ({})", tool.name, tool.id);
                            }
                            ContentBlock::ToolResult(result) => {
                                tracing::debug!("Tool result: {}", result.tool_use_id);
                            }
                            _ => {}
                        }
                    }
                }
                Message::Result(result) => {
                    tracing::info!(
                        "Query completed. Turns: {}, Duration: {}ms",
                        result.num_turns,
                        result.duration_ms
                    );

                    if let Some(ref usage) = result.usage {
                        // Parse usage from JSON value
                        if let Some(input_tokens) =
                            usage.get("input_tokens").and_then(|v| v.as_u64())
                        {
                            response.usage.input_tokens = input_tokens as u32;
                        }
                        if let Some(output_tokens) =
                            usage.get("output_tokens").and_then(|v| v.as_u64())
                        {
                            response.usage.output_tokens = output_tokens as u32;
                        }
                    }
                    if let Some(cost) = result.total_cost_usd {
                        response.usage.total_cost_usd = cost;
                    }
                    tracing::info!(
                        "Usage: Input tokens: {}, Output tokens: {}, Cost: ${:.4}",
                        response.usage.input_tokens,
                        response.usage.output_tokens,
                        response.usage.total_cost_usd,
                    );
                }
                Message::System(_) | Message::StreamEvent(_) | Message::ControlCancelRequest(_) => {
                    // Ignore system messages, stream events, and control requests
                }
            }
        }

        Ok(response)
    }

    /// Execute a task with a [`Task`] object.
    ///
    /// This method provides a more structured way to execute tasks by using
    /// a [`Task`] object that includes the prompt, context, system prompt, and
    /// execution limits.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to execute.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The query fails
    /// - The response cannot be parsed
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gba_core::{Agent, AgentConfig, Task, Context};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), gba_core::CoreError> {
    ///     let config = AgentConfig::default();
    ///     let agent = Agent::new(config);
    ///
    ///     let task = Task::with_defaults("Implement feature X", Context::default());
    ///
    ///     let response = agent.execute_task(&task).await?;
    ///     println!("{}", response.content);
    ///     Ok(())
    /// }
    /// ```
    #[tracing::instrument(skip(self, task))]
    pub async fn execute_task(&self, task: &Task) -> Result<Response> {
        tracing::info!(
            "Executing task with system prompt: {} ({} turns)",
            task.system_prompt,
            task.max_turns
        );

        // Build options with task-specific settings
        let system_prompt: SystemPrompt = task.system_prompt.clone().into();
        let options = ClaudeAgentOptions::builder()
            .model(self.config.model.clone())
            .system_prompt(system_prompt)
            .permission_mode(PermissionMode::BypassPermissions)
            .setting_sources(vec![SettingSource::User, SettingSource::Project])
            .max_turns(task.max_turns)
            .build();

        // Build the full prompt with context
        let full_prompt = self.build_prompt(&task.prompt, &task.context);

        // Send the query
        let messages = query(&full_prompt, Some(options))
            .await
            .map_err(|e| CoreError::ClaudeAgent(format!("Failed to send query: {e}")))?;

        // Collect all messages
        let mut response = Response::default();

        for message in &messages {
            match message {
                Message::Assistant(msg) => {
                    for block in &msg.message.content {
                        if let ContentBlock::Text(text) = block {
                            response.content.push_str(&text.text);
                        }
                    }
                }
                Message::Result(result) => {
                    if let Some(ref usage) = result.usage {
                        if let Some(input_tokens) =
                            usage.get("input_tokens").and_then(|v| v.as_u64())
                        {
                            response.usage.input_tokens = input_tokens as u32;
                        }
                        if let Some(output_tokens) =
                            usage.get("output_tokens").and_then(|v| v.as_u64())
                        {
                            response.usage.output_tokens = output_tokens as u32;
                        }
                    }
                    if let Some(cost) = result.total_cost_usd {
                        response.usage.total_cost_usd = cost;
                    }
                }
                Message::User(_) | Message::System(_) | Message::StreamEvent(_) | Message::ControlCancelRequest(_) => {
                    // Ignore other message types
                }
            }
        }

        tracing::info!(
            "Task completed. Input tokens: {}, Output tokens: {}, Cost: ${:.4}",
            response.usage.input_tokens,
            response.usage.output_tokens,
            response.usage.total_cost_usd,
        );

        Ok(response)
    }

    /// Execute a task with context building.
    ///
    /// This method automatically builds context from the repository and
    /// executes the task.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The task prompt to execute.
    /// * `repo_path` - Path to the repository.
    /// * `branch` - The branch name.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Context building fails
    /// - The query fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gba_core::{Agent, AgentConfig};
    /// use std::path::PathBuf;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), gba_core::CoreError> {
    ///     let config = AgentConfig::default();
    ///     let agent = Agent::new(config);
    ///
    ///     let response = agent.execute_with_context(
    ///         "Implement feature X",
    ///         PathBuf::from("/path/to/repo"),
    ///         "main".to_string(),
    ///     ).await?;
    ///
    ///     println!("{}", response.content);
    ///     Ok(())
    /// }
    /// ```
    #[tracing::instrument(skip(self, prompt))]
    pub async fn execute_with_context(
        &self,
        prompt: &str,
        repo_path: PathBuf,
        branch: String,
    ) -> Result<Response> {
        tracing::info!("Building context for repository: {:?}", repo_path);

        let context_builder_config = ContextBuilderConfig::default();
        let context = build_context(&repo_path, &branch, &context_builder_config).await?;

        self.execute(prompt, &context).await
    }

    /// Get the agent configuration.
    #[must_use]
    pub const fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Get the working directory.
    #[must_use]
    pub const fn working_dir(&self) -> &PathBuf {
        &self.working_dir
    }

    /// Build the full prompt with context.
    fn build_prompt(&self, prompt: &str, context: &TaskContext) -> String {
        let mut full_prompt = String::new();

        // Add context information
        full_prompt.push_str("\n## Repository Context\n\n");
        full_prompt.push_str(&format!(
            "Repository path: {}\n",
            context.repository_path.display()
        ));
        full_prompt.push_str(&format!("Branch: {}\n", context.branch));
        if !context.files.is_empty() {
            full_prompt.push_str(&format!("Files: {}\n\n", context.files.len()));

            for file in &context.files {
                full_prompt.push_str(&format!(
                    "### {}\n\n```\n{}\n```\n\n",
                    file.path.display(),
                    file.content
                ));
            }
        } else {
            full_prompt.push('\n');
        }

        // Add metadata
        if !context.metadata.is_empty() {
            full_prompt.push_str("\n## Metadata\n\n");
            for (key, value) in &context.metadata {
                full_prompt.push_str(&format!("{}: {}\n", key, value));
            }
            full_prompt.push('\n');
        }

        // Add the main prompt
        full_prompt.push_str("\n## Task\n\n");
        full_prompt.push_str(prompt);

        full_prompt
    }

    /// Build Claude Agent Options from AgentConfig.
    fn build_options(config: &AgentConfig) -> Result<ClaudeAgentOptions> {
        let system_prompt_text = "You are a helpful coding assistant.";
        let system_prompt: SystemPrompt = system_prompt_text.into();

        let options = ClaudeAgentOptions::builder()
            .model(config.model.clone())
            .system_prompt(system_prompt)
            .permission_mode(PermissionMode::BypassPermissions)
            .setting_sources(vec![SettingSource::User, SettingSource::Project])
            .build();

        Ok(options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Context;

    #[test]
    fn test_build_prompt() {
        let config = AgentConfig::default();
        let agent = Agent::new(config);

        let context = Context {
            repository_path: PathBuf::from("/repo"),
            branch: "main".to_string(),
            files: vec![],
            metadata: Default::default(),
        };

        let prompt = agent.build_prompt("Hello", &context);
        assert!(prompt.contains("Hello"));
        assert!(prompt.contains("/repo"));
        assert!(prompt.contains("main"));
    }

    #[test]
    fn test_agent_new() {
        let config = AgentConfig::default();
        let agent = Agent::new(config);

        assert!(!agent.working_dir().as_os_str().is_empty());
        assert_eq!(agent.config().model, "claude-sonnet-4-20250514");
    }
}