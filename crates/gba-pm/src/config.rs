//! Configuration types for GBA Prompt Manager.

use serde::{Deserialize, Serialize};
use tracing::instrument;

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
    #[instrument(skip(source))]
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

    /// Main branch name.
    #[serde(default = "default_branch")]
    pub main_branch: String,

    /// Feature/task name.
    #[serde(default)]
    pub feature_name: String,

    /// Feature ID.
    #[serde(default)]
    pub feature_id: String,

    /// Feature description.
    #[serde(default)]
    pub feature_description: String,

    /// Worktree path.
    #[serde(default)]
    pub worktree_path: String,

    /// Worktree branch name.
    #[serde(default)]
    pub worktree_branch: String,

    /// Current phase (for resume).
    #[serde(default)]
    pub current_phase: String,

    /// Current step (for resume).
    #[serde(default)]
    pub current_step: String,

    /// Turns completed so far (for resume).
    #[serde(default)]
    pub turns_so_far: u32,

    /// Cost incurred so far in USD (for resume).
    #[serde(default)]
    pub cost_so_far: f64,

    /// Whether to use preset (for resume).
    #[serde(default = "default_bool_true")]
    pub use_preset: bool,

    /// Tools array (for resume).
    #[serde(default)]
    pub tools: Vec<String>,

    /// Implementation plan.
    #[serde(default)]
    pub implementation_plan: String,

    /// Implementation summary.
    #[serde(default)]
    pub implementation_summary: String,

    /// Diff content for review.
    #[serde(default)]
    pub diff_content: String,

    /// Task kind (for resume).
    #[serde(default)]
    pub task_kind: String,

    /// Branch name (legacy).
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

fn default_bool_true() -> bool {
    true
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
        let branch = branch.into();
        Self {
            repo_path: repo_path.into(),
            main_branch: branch.clone(),
            branch,
            user_message: user_message.into(),
            files: Vec::new(),
            extra: serde_json::Value::Object(Default::default()),
            // Initialize defaults
            feature_name: String::new(),
            feature_id: String::new(),
            feature_description: String::new(),
            worktree_path: String::new(),
            worktree_branch: String::new(),
            current_phase: String::new(),
            current_step: String::new(),
            turns_so_far: 0,
            cost_so_far: 0.0,
            use_preset: true,
            tools: Vec::new(),
            implementation_plan: String::new(),
            implementation_summary: String::new(),
            diff_content: String::new(),
            task_kind: String::new(),
        }
    }

    /// Create a context for feature planning.
    #[must_use]
    pub fn for_planning(
        repo_path: impl Into<String>,
        main_branch: impl Into<String>,
        feature_name: impl Into<String>,
        feature_id: impl Into<String>,
        feature_description: impl Into<String>,
    ) -> Self {
        let main_branch = main_branch.into();
        Self {
            repo_path: repo_path.into(),
            main_branch: main_branch.clone(),
            branch: main_branch,
            feature_name: feature_name.into(),
            feature_id: feature_id.into(),
            feature_description: feature_description.into(),
            user_message: String::new(),
            files: Vec::new(),
            extra: serde_json::Value::Object(Default::default()),
            worktree_path: String::new(),
            worktree_branch: String::new(),
            current_phase: String::new(),
            current_step: String::new(),
            turns_so_far: 0,
            cost_so_far: 0.0,
            use_preset: false,
            tools: vec!["Read".to_string()],
            implementation_plan: String::new(),
            implementation_summary: String::new(),
            diff_content: String::new(),
            task_kind: "planning".to_string(),
        }
    }

    /// Create a context for implementation.
    #[must_use]
    pub fn for_implementation(
        repo_path: impl Into<String>,
        feature_name: impl Into<String>,
        feature_id: impl Into<String>,
        feature_description: impl Into<String>,
        worktree_path: impl Into<String>,
        worktree_branch: impl Into<String>,
        implementation_plan: impl Into<String>,
    ) -> Self {
        Self {
            repo_path: repo_path.into(),
            feature_name: feature_name.into(),
            feature_id: feature_id.into(),
            feature_description: feature_description.into(),
            worktree_path: worktree_path.into(),
            worktree_branch: worktree_branch.into(),
            implementation_plan: implementation_plan.into(),
            user_message: String::new(),
            files: Vec::new(),
            extra: serde_json::Value::Object(Default::default()),
            main_branch: String::new(),
            branch: String::new(),
            current_phase: String::new(),
            current_step: String::new(),
            turns_so_far: 0,
            cost_so_far: 0.0,
            use_preset: true,
            tools: Vec::new(),
            implementation_summary: String::new(),
            diff_content: String::new(),
            task_kind: "implementation".to_string(),
        }
    }

    /// Create a context for verification.
    #[must_use]
    pub fn for_verification(
        feature_name: impl Into<String>,
        feature_id: impl Into<String>,
        feature_description: impl Into<String>,
        implementation_summary: impl Into<String>,
    ) -> Self {
        Self {
            feature_name: feature_name.into(),
            feature_id: feature_id.into(),
            feature_description: feature_description.into(),
            implementation_summary: implementation_summary.into(),
            user_message: String::new(),
            files: Vec::new(),
            extra: serde_json::Value::Object(Default::default()),
            repo_path: String::new(),
            main_branch: String::new(),
            branch: String::new(),
            worktree_path: String::new(),
            worktree_branch: String::new(),
            current_phase: String::new(),
            current_step: String::new(),
            turns_so_far: 0,
            cost_so_far: 0.0,
            use_preset: true,
            tools: vec!["Read".to_string(), "Bash".to_string()],
            implementation_plan: String::new(),
            diff_content: String::new(),
            task_kind: "verification".to_string(),
        }
    }

    /// Create a context for code review.
    #[must_use]
    pub fn for_review(
        feature_name: impl Into<String>,
        feature_id: impl Into<String>,
        feature_description: impl Into<String>,
        diff_content: impl Into<String>,
    ) -> Self {
        Self {
            feature_name: feature_name.into(),
            feature_id: feature_id.into(),
            feature_description: feature_description.into(),
            diff_content: diff_content.into(),
            user_message: String::new(),
            files: Vec::new(),
            extra: serde_json::Value::Object(Default::default()),
            repo_path: String::new(),
            main_branch: String::new(),
            branch: String::new(),
            worktree_path: String::new(),
            worktree_branch: String::new(),
            current_phase: String::new(),
            current_step: String::new(),
            turns_so_far: 0,
            cost_so_far: 0.0,
            use_preset: true,
            tools: vec!["Read".to_string()],
            implementation_plan: String::new(),
            implementation_summary: String::new(),
            task_kind: "review".to_string(),
        }
    }

    /// Create a context for resuming a task.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn for_resume(
        feature_name: impl Into<String>,
        feature_id: impl Into<String>,
        feature_description: impl Into<String>,
        task_kind: impl Into<String>,
        current_phase: impl Into<String>,
        current_step: impl Into<String>,
        turns_so_far: u32,
        cost_so_far: f64,
        worktree_path: impl Into<String>,
        worktree_branch: impl Into<String>,
        implementation_plan: impl Into<String>,
        use_preset: bool,
        tools: Vec<String>,
    ) -> Self {
        Self {
            feature_name: feature_name.into(),
            feature_id: feature_id.into(),
            feature_description: feature_description.into(),
            task_kind: task_kind.into(),
            current_phase: current_phase.into(),
            current_step: current_step.into(),
            turns_so_far,
            cost_so_far,
            worktree_path: worktree_path.into(),
            worktree_branch: worktree_branch.into(),
            implementation_plan: implementation_plan.into(),
            use_preset,
            tools,
            user_message: String::new(),
            files: Vec::new(),
            extra: serde_json::Value::Object(Default::default()),
            repo_path: String::new(),
            main_branch: String::new(),
            branch: String::new(),
            implementation_summary: String::new(),
            diff_content: String::new(),
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

    /// Validate that required context variables are present.
    ///
    /// # Errors
    ///
    /// Returns an error if required variables are missing.
    pub fn validate(&self) -> Result<()> {
        if self.main_branch.is_empty() {
            return Err(PromptError::MissingVariable("main_branch".to_string()));
        }
        Ok(())
    }
}

impl Default for Context {
    fn default() -> Self {
        Self {
            repo_path: String::new(),
            main_branch: "main".to_string(),
            branch: "main".to_string(),
            user_message: String::new(),
            files: Vec::new(),
            extra: serde_json::Value::Object(Default::default()),
            feature_name: String::new(),
            feature_id: String::new(),
            feature_description: String::new(),
            worktree_path: String::new(),
            worktree_branch: String::new(),
            current_phase: String::new(),
            current_step: String::new(),
            turns_so_far: 0,
            cost_so_far: 0.0,
            use_preset: true,
            tools: Vec::new(),
            implementation_plan: String::new(),
            implementation_summary: String::new(),
            diff_content: String::new(),
            task_kind: String::new(),
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
        assert!(config.use_preset);
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
        assert!(!prompt_template.config.use_preset);
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
    fn test_context_for_planning() {
        let context =
            Context::for_planning("/repo", "main", "add-auth", "0001", "Add authentication");
        assert_eq!(context.main_branch, "main");
        assert_eq!(context.feature_name, "add-auth");
        assert_eq!(context.feature_id, "0001");
        assert_eq!(context.feature_description, "Add authentication");
        assert_eq!(context.task_kind, "planning");
    }

    #[test]
    fn test_context_for_implementation() {
        let context = Context::for_implementation(
            "/repo",
            "add-auth",
            "0001",
            "Add authentication",
            "/trees/0001",
            "gba/0001-add-auth",
            "Plan content",
        );
        assert_eq!(context.feature_name, "add-auth");
        assert_eq!(context.worktree_path, "/trees/0001");
        assert_eq!(context.worktree_branch, "gba/0001-add-auth");
        assert!(context.use_preset);
        assert!(context.tools.is_empty());
    }

    #[test]
    fn test_context_for_verification() {
        let context =
            Context::for_verification("add-auth", "0001", "Add authentication", "Summary");
        assert_eq!(context.feature_name, "add-auth");
        assert_eq!(context.implementation_summary, "Summary");
        assert_eq!(context.task_kind, "verification");
        assert_eq!(context.tools, vec!["Read", "Bash"]);
    }

    #[test]
    fn test_context_for_review() {
        let context = Context::for_review("add-auth", "0001", "Add authentication", "diff content");
        assert_eq!(context.feature_name, "add-auth");
        assert_eq!(context.diff_content, "diff content");
        assert_eq!(context.task_kind, "review");
        assert_eq!(context.tools, vec!["Read"]);
    }

    #[test]
    fn test_context_for_resume() {
        let context = Context::for_resume(
            "add-auth",
            "0001",
            "Add authentication",
            "implementation",
            "phase_1",
            "step_2",
            5,
            0.50,
            "/trees/0001",
            "gba/0001-add-auth",
            "Plan content",
            true,
            vec!["Write".to_string()],
        );
        assert_eq!(context.task_kind, "implementation");
        assert_eq!(context.current_phase, "phase_1");
        assert_eq!(context.turns_so_far, 5);
        assert_eq!(context.cost_so_far, 0.50);
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
    fn test_context_validate() {
        let context = Context::default();
        assert!(context.validate().is_ok());
    }

    #[test]
    fn test_context_validate_missing_main_branch() {
        let context = Context {
            main_branch: String::new(),
            ..Default::default()
        };
        assert!(context.validate().is_err());
    }

    #[test]
    fn test_template_config_defaults() {
        let config = TemplateConfig::default();
        assert!(config.use_preset);
        assert_eq!(config.max_turns, 100);
        assert!(config.tools.is_empty());
    }
}
