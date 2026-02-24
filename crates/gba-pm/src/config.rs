//! Configuration types for GBA Prompt Manager.

use serde::{Deserialize, Serialize};

use crate::error::{PromptError, Result};

/// Template configuration extracted from front matter.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateConfig {
    /// System prompt text (or empty if using preset).
    #[serde(default)]
    pub system_prompt: String,

    /// Whether to use Claude Code preset.
    #[serde(default = "default_use_preset")]
    pub use_preset: bool,

    /// Tools to enable (empty = all tools).
    #[serde(default)]
    pub tools: Vec<String>,

    /// Maximum number of turns allowed.
    #[serde(default = "default_max_turns")]
    pub max_turns: u32,
}

fn default_use_preset() -> bool {
    true
}

fn default_max_turns() -> u32 {
    100
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            system_prompt: String::new(),
            use_preset: true,
            tools: Vec::new(),
            max_turns: 100,
        }
    }
}

/// Template with its configuration and source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptTemplate {
    /// Template configuration from front matter.
    pub config: TemplateConfig,

    /// The actual template source.
    pub template: String,
}

impl PromptTemplate {
    /// Parse a template with front matter.
    ///
    /// # Errors
    ///
    /// Returns an error if the front matter cannot be parsed.
    #[tracing::instrument(skip(source))]
    pub fn parse(source: &str) -> Result<Self> {
        let (config, template) = extract_front_matter(source)?;
        Ok(Self { config, template })
    }
}

/// Extract YAML front matter from a template source.
///
/// Front matter is delimited by `---` at the beginning and end.
///
/// # Errors
///
/// Returns an error if the front matter cannot be parsed.
fn extract_front_matter(source: &str) -> Result<(TemplateConfig, String)> {
    let lines: Vec<&str> = source.lines().collect();

    // Check for front matter delimiter at start
    if lines.first().is_none_or(|l| l.trim() != "---") {
        // No front matter, use default config and entire source as template
        return Ok((TemplateConfig::default(), source.to_string()));
    }

    // Find the end delimiter
    let end_idx = lines
        .iter()
        .skip(1)
        .position(|l| l.trim() == "---")
        .map(|i| i + 1);

    let Some(end_idx) = end_idx else {
        // No end delimiter found, use default config and entire source as template
        return Ok((TemplateConfig::default(), source.to_string()));
    };

    // Extract front matter (lines 1 to end_idx)
    let front_matter = lines[1..end_idx].join("\n");

    // Parse YAML
    let config: TemplateConfig = serde_yaml::from_str(&front_matter)
        .map_err(|e| PromptError::Template(format!("Failed to parse front matter: {e}")))?;

    // Extract template (lines after end_idx)
    let template = lines[end_idx + 1..].join("\n");

    Ok((config, template))
}

/// Template context for rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    /// Repository path.
    #[serde(default)]
    pub repo_path: String,

    /// Branch name.
    #[serde(default = "default_branch")]
    pub branch: String,

    /// Files in context.
    #[serde(default)]
    pub files: Vec<FileContext>,

    /// User message.
    #[serde(default)]
    pub user_message: String,

    /// Additional context variables.
    #[serde(flatten)]
    #[serde(default)]
    pub extra: serde_json::Value,
}

fn default_branch() -> String {
    "main".to_string()
}

/// File context for templates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileContext {
    /// File path.
    pub path: String,

    /// File content.
    pub content: String,

    /// File language.
    #[serde(default)]
    pub language: String,
}

impl Context {
    /// Create a new context with the given values.
    ///
    /// # Arguments
    ///
    /// * `repo_path` - The repository path.
    /// * `branch` - The branch name.
    /// * `user_message` - The user message.
    #[must_use]
    pub fn new(
        repo_path: impl Into<String>,
        branch: impl Into<String>,
        user_message: impl Into<String>,
    ) -> Self {
        Self {
            repo_path: repo_path.into(),
            branch: branch.into(),
            user_message: user_message.into(),
            files: Vec::new(),
            extra: serde_json::Value::Object(Default::default()),
        }
    }

    /// Add a file to the context.
    pub fn add_file(&mut self, file: FileContext) {
        self.files.push(file);
    }

    /// Add an extra variable to the context.
    pub fn add_extra(&mut self, key: impl Into<String>, value: serde_json::Value) {
        if let serde_json::Value::Object(ref mut map) = self.extra {
            map.insert(key.into(), value);
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            repo_path: String::new(),
            branch: "main".to_string(),
            user_message: String::new(),
            files: Vec::new(),
            extra: serde_json::Value::Object(Default::default()),
        }
    }
}

impl FileContext {
    /// Create a new file context.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path.
    /// * `content` - The file content.
    /// * `language` - The file language.
    #[must_use]
    pub fn new(
        path: impl Into<String>,
        content: impl Into<String>,
        language: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
            language: language.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_front_matter() {
        let source = r#"---
systemPrompt: "Test prompt"
usePreset: true
tools: []
---
Template content here"#;

        let (config, template) = extract_front_matter(source).unwrap();
        assert_eq!(config.system_prompt, "Test prompt");
        assert_eq!(config.use_preset, true);
        assert_eq!(template.trim(), "Template content here");
    }

    #[test]
    fn test_extract_no_front_matter() {
        let source = "Just template content";
        let (config, template) = extract_front_matter(source).unwrap();
        assert_eq!(config.system_prompt, "");
        assert_eq!(template, source);
    }

    #[test]
    fn test_prompt_template_parse() {
        let source = r#"---
systemPrompt: "You are helpful"
usePreset: false
tools:
  - Read
  - Write
---
You are helping with {{ task }}"#;

        let prompt_template = PromptTemplate::parse(source).unwrap();
        assert_eq!(prompt_template.config.system_prompt, "You are helpful");
        assert_eq!(prompt_template.config.use_preset, false);
        assert_eq!(prompt_template.config.tools, vec!["Read", "Write"]);
    }

    #[test]
    fn test_context_creation() {
        let context = Context::new("/repo/path", "main", "Help me");
        assert_eq!(context.repo_path, "/repo/path");
        assert_eq!(context.branch, "main");
        assert_eq!(context.user_message, "Help me");
    }

    #[test]
    fn test_file_context() {
        let file = FileContext::new("test.rs", "fn main() {}", "rust");
        assert_eq!(file.path, "test.rs");
        assert_eq!(file.content, "fn main() {}");
        assert_eq!(file.language, "rust");
    }

    #[test]
    fn test_context_add_file() {
        let mut context = Context::default();
        context.add_file(FileContext::new("test.rs", "content", "rust"));
        assert_eq!(context.files.len(), 1);
        assert_eq!(context.files[0].path, "test.rs");
    }

    #[test]
    fn test_context_add_extra() {
        let mut context = Context::default();
        context.add_extra("key", serde_json::json!("value"));
        if let serde_json::Value::Object(map) = &context.extra {
            assert_eq!(map.get("key"), Some(&serde_json::json!("value")));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_template_config_defaults() {
        let config = TemplateConfig::default();
        assert_eq!(config.use_preset, true);
        assert_eq!(config.max_turns, 100);
        assert!(config.tools.is_empty());
    }
}
