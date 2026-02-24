//! Agent implementation for interacting with Claude Agent SDK.

/// Agent for interacting with Claude Agent SDK.
#[derive(Debug)]
pub struct Agent {
    /// Agent configuration.
    config: AgentConfig,
}

impl Agent {
    /// Create a new agent with the given configuration.
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }

    /// Get the agent configuration.
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }
}

/// Agent configuration.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// API key for Claude.
    pub api_key: String,
    /// Model to use.
    pub model: String,
}

impl AgentConfig {
    /// Create a new agent configuration.
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
        }
    }
}
