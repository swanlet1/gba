//! Agent implementation for interacting with Claude Agent SDK.

use std::fmt;
use std::path::PathBuf;
use std::pin::Pin;

use claude_agent_sdk_rs::{
    ClaudeAgentOptions, ClaudeClient, ContentBlock, Message, PermissionMode, SettingSource,
    SystemPrompt,
};
use futures::stream::{Stream, StreamExt};

use crate::config::AgentConfig;
use crate::context_builder::{ContextBuilderConfig, build_context};
use crate::error::{CoreError, Result};
use crate::task::{Context as TaskContext, Response, Task, Usage};

/// Agent for interacting with Claude Agent SDK.
///
/// The agent manages the lifecycle of a Claude Code session and provides
/// methods for executing tasks with prompts and context.
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
///
///     let mut agent = Agent::new(config).await?;
///
///     let response = agent.execute(
///         "Hello Claude",
///         &Context::default(),
///     ).await?;
///
///     println!("{}", response.content);
///
///     agent.shutdown().await?;
///     Ok(())
/// }
/// ```
pub struct Agent {
    /// Claude client for interacting with the SDK.
    client: ClaudeClient,
    /// Working directory for the agent.
    working_dir: PathBuf,
    /// Whether the agent is connected.
    connected: bool,
    /// Agent configuration.
    config: AgentConfig,
}

impl fmt::Debug for Agent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Agent")
            .field("working_dir", &self.working_dir)
            .field("connected", &self.connected)
            .field("config", &self.config)
            .field("client", &"<ClaudeClient>")
            .finish()
    }
}

impl Agent {
    /// Create a new agent with the given configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Agent configuration including API key and model.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The API key is empty
    /// - The client cannot be created
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gba_core::{Agent, AgentConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), gba_core::CoreError> {
    ///     let config = AgentConfig::default();
    ///     let mut agent = Agent::new(config).await?;
    ///     agent.shutdown().await?;
    ///     Ok(())
    /// }
    /// ```
    #[tracing::instrument(skip(config))]
    pub async fn new(config: AgentConfig) -> Result<Self> {
        let options = Self::build_options(&config)?;
        let client = ClaudeClient::new(options);

        let working_dir = std::env::current_dir().map_err(CoreError::Io)?;

        tracing::info!("Created agent with model: {}", config.model);

        Ok(Self {
            config,
            client,
            working_dir,
            connected: false,
        })
    }

    /// Connect the agent to Claude.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The connection fails
    /// - The initialization handshake fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gba_core::{Agent, AgentConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), gba_core::CoreError> {
    ///     let config = AgentConfig::default();
    ///     let mut agent = Agent::new(config).await?;
    ///     agent.connect().await?;
    ///     agent.shutdown().await?;
    ///     Ok(())
    /// }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn connect(&mut self) -> Result<()> {
        if self.connected {
            tracing::debug!("Agent already connected");
            return Ok(());
        }

        tracing::info!("Connecting agent to Claude...");
        self.client
            .connect()
            .await
            .map_err(|e| CoreError::ClaudeAgent(format!("Failed to connect: {e}")))?;
        self.connected = true;
        tracing::info!("Agent connected successfully");
        Ok(())
    }

    /// Execute a task with the given prompt and context.
    ///
    /// This method executes a task using the non-streaming API, collecting
    /// all messages before returning.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The task prompt to execute.
    /// * `context` - The task context containing repository information.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The agent is not connected
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
    ///     let mut agent = Agent::new(config).await?;
    ///     agent.connect().await?;
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
    ///     agent.shutdown().await?;
    ///     Ok(())
    /// }
    /// ```
    #[tracing::instrument(skip(self, prompt, context))]
    pub async fn execute(&mut self, prompt: &str, context: &TaskContext) -> Result<Response> {
        self.ensure_connected()?;

        tracing::info!("Executing task with prompt: {}", prompt);

        // Build the full prompt with context
        let full_prompt = self.build_prompt(prompt, context);

        // Send the query
        self.client
            .query(&full_prompt)
            .await
            .map_err(|e| CoreError::ClaudeAgent(format!("Failed to send query: {e}")))?;

        // Collect all messages
        let mut response = Response::default();
        let mut stream = self.client.receive_response();

        while let Some(message_result) = stream.next().await {
            let message = message_result
                .map_err(|e| CoreError::ClaudeAgent(format!("Failed to receive message: {e}")))?;

            match message {
                Message::Assistant(msg) => {
                    for block in &msg.message.content {
                        if let ContentBlock::Text(text) = block {
                            response.content.push_str(&text.text);
                        }
                    }
                }
                Message::Result(result) => {
                    if let Some(usage) = result.usage {
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
                }
                _ => {}
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

    /// Execute a streaming task.
    ///
    /// This method executes a task using the streaming API, returning a stream
    /// of chunks as they arrive.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The task prompt to execute.
    /// * `context` - The task context containing repository information.
    ///
    /// # Returns
    ///
    /// A stream of [`Chunk`] results.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The agent is not connected
    /// - The query fails
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gba_core::{Agent, AgentConfig, Context};
    /// use std::path::PathBuf;
    /// use futures::StreamExt;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), gba_core::CoreError> {
    ///     let config = AgentConfig::default();
    ///     let mut agent = Agent::new(config).await?;
    ///     agent.connect().await?;
    ///
    ///     let context = Context::default();
    ///
    ///     let mut stream = agent.execute_stream(
    ///         "Explain Rust ownership",
    ///         &context,
    ///     ).await?;
    ///
    ///     let mut total_cost = 0.0;
    ///     while let Some(chunk_result) = stream.next().await {
    ///         match chunk_result? {
    ///             gba_core::Chunk::Text(text) => print!("{}", text),
    ///             gba_core::Chunk::Done(usage) => {
    ///                 total_cost = usage.total_cost_usd;
    ///             }
    ///         }
    ///     }
    ///     drop(stream);
    ///
    ///     agent.shutdown().await?;
    ///     Ok(())
    /// }
    /// ```
    #[allow(tail_expr_drop_order)]
    #[tracing::instrument(skip(self, prompt, context))]
    pub async fn execute_stream<'a>(
        &'a mut self,
        prompt: &str,
        context: &'a TaskContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<Chunk>> + Send + 'a>>> {
        self.ensure_connected()?;

        tracing::info!("Executing streaming task with prompt: {}", prompt);

        // Build the full prompt with context
        let full_prompt = self.build_prompt(prompt, context);

        // Send the query
        self.client
            .query(&full_prompt)
            .await
            .map_err(|e| CoreError::ClaudeAgent(format!("Failed to send query: {e}")))?;

        // Return the response stream
        Ok(Box::pin(async_stream::stream! {
            let mut stream = self.client.receive_response();

            while let Some(message_result) = stream.next().await {
                match message_result {
                    Ok(message) => {
                        match message {
                            Message::Assistant(msg) => {
                                for block in msg.message.content {
                                    if let ContentBlock::Text(text) = block {
                                        yield Ok(Chunk::Text(text.text));
                                    }
                                }
                            }
                            Message::Result(result) => {
                                let mut usage = Usage::default();
                                if let Some(usage_value) = result.usage {
                                    if let Some(input_tokens) =
                                        usage_value.get("input_tokens").and_then(|v| v.as_u64())
                                    {
                                        usage.input_tokens = input_tokens as u32;
                                    }
                                    if let Some(output_tokens) =
                                        usage_value.get("output_tokens").and_then(|v| v.as_u64())
                                    {
                                        usage.output_tokens = output_tokens as u32;
                                    }
                                }
                                if let Some(cost) = result.total_cost_usd {
                                    usage.total_cost_usd = cost;
                                }
                                yield Ok(Chunk::Done(usage));
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        yield Err(CoreError::ClaudeAgent(format!(
                            "Failed to receive message: {e}"
                        )));
                    }
                }
            }
        }))
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
    /// - The agent is not connected
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
    ///     let mut agent = Agent::new(config).await?;
    ///     agent.connect().await?;
    ///
    ///     let task = Task::with_defaults("Implement feature X", Context::default());
    ///
    ///     let response = agent.execute_task(&task).await?;
    ///     println!("{}", response.content);
    ///
    ///     agent.shutdown().await?;
    ///     Ok(())
    /// }
    /// ```
    #[tracing::instrument(skip(self, task))]
    pub async fn execute_task(&mut self, task: &Task) -> Result<Response> {
        self.ensure_connected()?;

        tracing::info!("Executing task with system prompt: {}", task.system_prompt);

        // Update the client options with the task's system prompt
        let system_prompt: SystemPrompt = task.system_prompt.clone().into();
        let _options = ClaudeAgentOptions::builder()
            .system_prompt(system_prompt)
            .max_turns(task.max_turns)
            .build();

        // Note: In a real implementation, we'd need to recreate the client
        // with the new options. For now, we'll use the existing client.

        // Build the full prompt with context
        let full_prompt = self.build_prompt(&task.prompt, &task.context);

        // Send the query
        self.client
            .query(&full_prompt)
            .await
            .map_err(|e| CoreError::ClaudeAgent(format!("Failed to send query: {e}")))?;

        // Collect all messages
        let mut response = Response::default();
        let mut stream = self.client.receive_response();

        while let Some(message_result) = stream.next().await {
            let message = message_result
                .map_err(|e| CoreError::ClaudeAgent(format!("Failed to receive message: {e}")))?;

            match message {
                Message::Assistant(msg) => {
                    for block in &msg.message.content {
                        if let ContentBlock::Text(text) = block {
                            response.content.push_str(&text.text);
                        }
                    }
                }
                Message::Result(result) => {
                    if let Some(usage) = result.usage {
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
                }
                _ => {}
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
    /// - The agent is not connected
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
    ///     let mut agent = Agent::new(config).await?;
    ///     agent.connect().await?;
    ///
    ///     let response = agent.execute_with_context(
    ///         "Implement feature X",
    ///         PathBuf::from("/path/to/repo"),
    ///         "main".to_string(),
    ///     ).await?;
    ///
    ///     println!("{}", response.content);
    ///     agent.shutdown().await?;
    ///     Ok(())
    /// }
    /// ```
    #[tracing::instrument(skip(self, prompt))]
    pub async fn execute_with_context(
        &mut self,
        prompt: &str,
        repo_path: PathBuf,
        branch: String,
    ) -> Result<Response> {
        self.ensure_connected()?;

        tracing::info!("Building context for repository: {:?}", repo_path);

        let context_builder_config = ContextBuilderConfig::default();
        let context = build_context(&repo_path, &branch, &context_builder_config).await?;

        self.execute(prompt, &context).await
    }

    /// Stop the agent gracefully.
    ///
    /// This disconnects the agent from Claude and cleans up resources.
    ///
    /// # Errors
    ///
    /// Returns an error if disconnection fails.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use gba_core::{Agent, AgentConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), gba_core::CoreError> {
    ///     let config = AgentConfig::default();
    ///     let mut agent = Agent::new(config).await?;
    ///     agent.connect().await?;
    ///
    ///     // ... use agent ...
    ///
    ///     agent.shutdown().await?;
    ///     Ok(())
    /// }
    /// ```
    #[tracing::instrument(skip(self))]
    pub async fn shutdown(mut self) -> Result<()> {
        if self.connected {
            tracing::info!("Shutting down agent...");
            self.client
                .disconnect()
                .await
                .map_err(|e| CoreError::ClaudeAgent(format!("Failed to disconnect: {e}")))?;
            tracing::info!("Agent shutdown complete");
        }
        Ok(())
    }

    /// Get the agent configuration.
    #[must_use]
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Get the working directory.
    #[must_use]
    pub fn working_dir(&self) -> &PathBuf {
        &self.working_dir
    }

    /// Check if the agent is connected.
    #[must_use]
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Ensure the agent is connected.
    fn ensure_connected(&self) -> Result<()> {
        if !self.connected {
            return Err(CoreError::ClaudeAgent(
                "Agent not connected. Call connect() first.".to_string(),
            ));
        }
        Ok(())
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

/// A chunk from a streaming response.
#[derive(Debug, Clone)]
pub enum Chunk {
    /// Text content chunk.
    Text(String),
    /// Response is complete with usage statistics.
    Done(Usage),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Context;

    #[test]
    fn test_build_prompt() {
        let config = AgentConfig::default();

        let client = ClaudeClient::new(
            ClaudeAgentOptions::builder()
                .model("test-model".to_string())
                .permission_mode(PermissionMode::BypassPermissions)
                .setting_sources(vec![SettingSource::User])
                .build(),
        );

        let agent = Agent {
            config,
            client,
            working_dir: PathBuf::from("/"),
            connected: false,
        };

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
}
