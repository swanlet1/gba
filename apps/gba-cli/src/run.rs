//! Command execution logic for GBA CLI.
//!
//! This module contains the main command handlers for the CLI.

use gba_core::config::ProjectConfig;
use gba_pm::{Context as PromptContext, PromptManager};
use std::fs;
use std::path::Path;
use tracing::{debug, info, instrument, warn};

use crate::cli::RunArgs;
use crate::config::ConfigManager;
use crate::error::{CliError, Result as CliResult};
use crate::output::OutputFormatter;
use crate::ui::Tui;

/// Output formatter instance.
static OUTPUT: std::sync::OnceLock<OutputFormatter> = std::sync::OnceLock::new();

/// Get the output formatter.
fn output() -> &'static OutputFormatter {
    OUTPUT.get_or_init(OutputFormatter::new)
}

/// Initialize a GBA project.
///
/// # Arguments
///
/// * `project_path` - Path to the project directory.
/// * `main_branch` - Name of the main branch.
/// * `repo_url` - Optional repository URL.
///
/// # Errors
///
/// Returns an error if initialization fails.
#[instrument(skip(project_path))]
pub async fn init(project_path: &Path, main_branch: &str, repo_url: Option<&str>) -> CliResult<()> {
    let out = output();

    info!("Initializing GBA project at {}", project_path.display());

    // Check if .gba directory already exists
    let gba_dir = project_path.join(".gba");
    if gba_dir.exists() {
        out.warning("GBA project already initialized");
        return Ok(());
    }

    out.section("Initializing GBA Project");
    out.info(&format!("Path: {}", project_path.display()));
    out.info(&format!("Main branch: {}", main_branch));
    if let Some(url) = repo_url {
        out.info(&format!("Repository: {}", url));
    }

    // Create .gba directory structure
    out.info("Creating directory structure...");

    let templates_dir = gba_dir.join("templates");
    let features_dir = gba_dir.join("features");

    fs::create_dir_all(&templates_dir)?;
    fs::create_dir_all(&features_dir)?;

    // Create features README
    let readme_path = features_dir.join("README.md");
    let readme_content = "# Features Directory\n\n\
        This directory contains state files for each feature being developed.\n\n\
        State files track the progress of task execution and are excluded from git.\n";
    fs::write(&readme_path, readme_content)?;

    // Detect repository name from path
    let repo_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("project");

    // Detect repository URL from git if not provided
    let detected_url = if repo_url.is_none() {
        detect_repo_url(project_path)
    } else {
        None
    };

    let final_repo_url = repo_url.or(detected_url.as_deref()).unwrap_or("unknown");

    // Create default configuration
    out.info("Creating configuration file...");

    let config = ProjectConfig {
        version: "1.0".to_string(),
        project: Default::default(),
        agent: Default::default(),
        prompts: Default::default(),
        repository: Default::default(),
        logging: Default::default(),
        worktree: Default::default(),
        limits: Default::default(),
    };

    // Update project metadata
    let config_yaml = format!(
        r#"# GBA Project Configuration
version: "{}"

# Project metadata
project:
  name: "{}"
  repository:
    url: "{}"
    mainBranch: "{}"

# Agent defaults
agent:
  model: "{}"
  maxTokens: {}
  temperature: {}
  timeout: {}

# Prompt templates configuration
prompts:
  directory: "./.gba/templates"
  useBundled: true

# Repository scanning settings
repository:
  excludePatterns: {}
  maxFileSize: {}

# Logging configuration
logging:
  level: "{}"
  format: "{}"

# Worktree configuration
worktree:
  directory: "./.trees"
  branchPrefix: "{}"

# Execution limits
limits:
  maxTurns: {}
  maxCostUsd: {}
"#,
        config.version,
        repo_name,
        final_repo_url,
        main_branch,
        config.agent.model,
        config.agent.max_tokens,
        config.agent.temperature,
        config.agent.timeout,
        serde_yaml::to_string(&config.repository.exclude_patterns).unwrap(),
        config.repository.max_file_size,
        config.logging.level,
        config.logging.format,
        config.worktree.branch_prefix,
        config.limits.max_turns,
        config.limits.max_cost_usd
    );

    let config_path = ConfigManager::config_file_path(project_path);
    fs::write(&config_path, config_yaml)?;

    out.success("GBA project initialized successfully!");
    out.info(&format!("Configuration file: {}", config_path.display()));
    out.info("You can now run 'gba run' to execute tasks.");

    Ok(())
}

/// Detect repository URL from git.
fn detect_repo_url(project_path: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["-C", project_path.to_str()?, "remote", "get-url", "origin"])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Execute the run command.
///
/// # Arguments
///
/// * `config` - Configuration manager.
/// * `args` - Run command arguments.
///
/// # Errors
///
/// Returns an error if execution fails.
#[instrument(skip(config))]
pub async fn run(config: ConfigManager, args: RunArgs) -> CliResult<()> {
    let out = output();

    info!(
        feature = %args.feature,
        kind = %args.kind,
        tui = args.tui,
        resume = args.resume,
        "Starting run command"
    );

    // Display feature information
    out.feature_info(
        &args.feature,
        &format!("{:04}", feature_id_from_name(&args.feature)),
        args.description.as_deref(),
    );

    // Check if resuming or starting fresh
    if args.resume {
        check_feature_state(&config, &args.feature)?;
    }

    // Initialize prompt manager
    let prompt_manager = init_prompt_manager(&config)?;

    // Get template name
    let template_name = args.kind.template_name();

    // Verify template exists
    if !prompt_manager.has_prompt(template_name) {
        return Err(CliError::template_not_found(template_name.to_string()));
    }

    // Build context for rendering
    let context = build_run_context(&config, &args)?;

    // Get the prompt
    out.subsection("Rendering Prompt");
    let prompt = prompt_manager.get_prompt(template_name, &context)?;
    debug!("Prompt rendered successfully");

    out.prompt_output(template_name, &prompt);

    // In TUI mode, start the TUI
    if args.tui {
        out.info("Starting TUI mode...");
        let mut tui = Tui::new()?;
        tui.draw()?;
        tui.exit()?;
        out.success("TUI completed");
    } else {
        out.info("Executing task (non-TUI mode)...");
        // TODO: Integrate with gba-core Agent for actual execution
        out.success("Task would be executed here");
    }

    Ok(())
}

/// List available prompts.
///
/// # Arguments
///
/// * `config` - Configuration manager.
/// * `verbose` - Whether to show verbose output.
///
/// # Errors
///
/// Returns an error if listing fails.
pub fn list_prompts(config: ConfigManager, verbose: bool) -> CliResult<()> {
    let out = output();

    info!("Listing available prompts");

    // Initialize prompt manager
    let prompt_manager = init_prompt_manager(&config)?;

    // Get available templates
    let templates = prompt_manager.list_prompts();

    if templates.is_empty() {
        out.warning("No templates found");
        return Ok(());
    }

    out.prompt_list(&templates, verbose);

    Ok(())
}

/// Execute a single prompt.
///
/// # Arguments
///
/// * `config` - Configuration manager.
/// * `template` - Template name to use.
/// * `message` - User message to include.
///
/// # Errors
///
/// Returns an error if execution fails.
#[instrument(skip(config))]
pub async fn execute_prompt(config: ConfigManager, template: &str, message: &str) -> CliResult<()> {
    let out = output();

    info!("Executing prompt: {}", template);

    // Initialize prompt manager
    let prompt_manager = init_prompt_manager(&config)?;

    // Verify template exists
    if !prompt_manager.has_prompt(template) {
        return Err(CliError::template_not_found(template.to_string()));
    }

    // Build basic context
    let repo_path = config.project_path().to_str().unwrap_or(".");
    let context = PromptContext::new(repo_path, "main", message);

    // Get the prompt
    let prompt = prompt_manager.get_prompt(template, &context)?;

    out.prompt_output(template, &prompt);

    // TODO: Integrate with gba-core Agent for actual execution
    out.info("Prompt would be sent to agent for execution");

    Ok(())
}

/// Initialize the prompt manager.
///
/// # Arguments
///
/// * `config` - Configuration manager.
///
/// # Errors
///
/// Returns an error if initialization fails.
fn init_prompt_manager(config: &ConfigManager) -> Result<PromptManager, CliError> {
    let templates_dir = config.templates_dir();
    let use_bundled = config.config().prompts.use_bundled;

    debug!(
        "Initializing prompt manager with templates dir: {}",
        templates_dir.display()
    );

    PromptManager::with_local_dir(templates_dir, use_bundled)
        .map_err(|e| CliError::Config(format!("Failed to initialize prompt manager: {e}")))
}

/// Build context for run command.
///
/// # Arguments
///
/// * `config` - Configuration manager.
/// * `args` - Run command arguments.
///
/// # Errors
///
/// Returns an error if context building fails.
fn build_run_context(config: &ConfigManager, args: &RunArgs) -> Result<PromptContext, CliError> {
    let repo_path = config.project_path().to_str().unwrap_or(".");
    let main_branch = config.config().project.repository.main_branch.clone();
    let feature_id = format!("{:04}", feature_id_from_name(&args.feature));

    let user_message = args
        .description
        .clone()
        .unwrap_or_else(|| format!("{} for feature: {}", args.kind, args.feature));

    let mut context = PromptContext::new(repo_path, &main_branch, &user_message);

    // Add feature context
    context.add_extra("feature_name", serde_json::json!(args.feature));
    context.add_extra("feature_id", serde_json::json!(feature_id));
    context.add_extra("feature_description", serde_json::json!(args.description));
    context.add_extra("main_branch", serde_json::json!(main_branch));

    Ok(context)
}

/// Generate a feature ID from a feature name.
fn feature_id_from_name(name: &str) -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    (hasher.finish() % 10000) as u32
}

/// Check feature state for resumption.
///
/// # Arguments
///
/// * `config` - Configuration manager.
/// * `feature` - Feature name.
///
/// # Errors
///
/// Returns an error if state check fails.
fn check_feature_state(config: &ConfigManager, feature: &str) -> Result<(), CliError> {
    let feature_id = format!("{:04}", feature_id_from_name(feature));
    let state_path = config.feature_state_path(&feature_id);

    if !state_path.exists() {
        warn!("No previous state found, starting fresh");
        return Ok(());
    }

    info!("Found previous state at {}", state_path.display());

    // TODO: Load and validate state file
    let _state_content = fs::read_to_string(&state_path)?;

    Ok(())
}

/// Create implementation plan.
///
/// # Arguments
///
/// * `config` - Configuration manager.
/// * `feature_name` - Feature name.
/// * `description` - Optional feature description.
///
/// # Errors
///
/// Returns an error if planning fails.
#[instrument(skip(config))]
#[allow(dead_code)]
pub async fn create_plan(
    config: &ConfigManager,
    feature_name: &str,
    description: Option<&str>,
) -> CliResult<()> {
    info!(
        feature = %feature_name,
        description = description.unwrap_or("No description"),
        "Creating implementation plan"
    );

    let out = output();
    out.section("Creating Implementation Plan");
    out.feature_info(feature_name, "0001", description);

    // Initialize prompt manager
    let prompt_manager = init_prompt_manager(config)?;

    // Build context
    let repo_path = config.project_path().to_str().unwrap_or(".");
    let main_branch = config.config().project.repository.main_branch.clone();
    let feature_id = format!("{:04}", feature_id_from_name(feature_name));

    let mut context = PromptContext::new(
        repo_path,
        &main_branch,
        description.unwrap_or("Create implementation plan"),
    );

    context.add_extra("feature_name", serde_json::json!(feature_name));
    context.add_extra("feature_id", serde_json::json!(feature_id));
    context.add_extra("feature_description", serde_json::json!(description));
    context.add_extra("main_branch", serde_json::json!(main_branch));

    // Get and render the plan template
    if let Ok(prompt) = prompt_manager.get_prompt("plan", &context) {
        out.prompt_output("plan", &prompt);
    }

    Ok(())
}

/// Execute implementation.
///
/// # Arguments
///
/// * `config` - Configuration manager.
/// * `feature_name` - Feature name.
///
/// # Errors
///
/// Returns an error if implementation fails.
#[instrument]
#[allow(dead_code)]
pub async fn execute_implementation(_config: &ConfigManager, feature_name: &str) -> CliResult<()> {
    info!(feature = %feature_name, "Executing implementation");

    let out = output();
    out.section("Executing Implementation");

    // TODO: Implement execution logic
    out.info("Implementation would be executed here");

    Ok(())
}

/// Verify implementation.
///
/// # Arguments
///
/// * `_config` - Configuration manager.
/// * `feature_name` - Feature name.
///
/// # Errors
///
/// Returns an error if verification fails.
#[instrument(skip(_config))]
#[allow(dead_code)]
pub async fn verify_implementation(_config: &ConfigManager, feature_name: &str) -> CliResult<()> {
    info!(feature = %feature_name, "Verifying implementation");

    let out = output();
    out.section("Verifying Implementation");

    // TODO: Implement verification logic
    out.info("Verification would be executed here");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::TaskKind;

    #[test]
    fn test_feature_id_from_name() {
        let id1 = feature_id_from_name("test-feature");
        let id2 = feature_id_from_name("test-feature");
        assert_eq!(id1, id2);

        let id3 = feature_id_from_name("different-feature");
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_build_run_context() {
        let temp_dir = std::env::temp_dir().join("gba-test-build-context");
        fs::create_dir_all(&temp_dir).unwrap();
        let gba_dir = temp_dir.join(".gba");
        fs::create_dir_all(&gba_dir).unwrap();

        let config_path = gba_dir.join("config.yml");
        let default_config = ProjectConfig::default_config();
        let config_yaml = serde_yaml::to_string(&default_config).unwrap();
        fs::write(&config_path, config_yaml).unwrap();

        let config_manager = ConfigManager::load(&temp_dir).unwrap();

        let args = RunArgs {
            feature: "test".to_string(),
            kind: TaskKind::Planning,
            description: Some("Test feature".to_string()),
            tui: false,
            resume: false,
        };

        let result = build_run_context(&config_manager, &args);
        assert!(result.is_ok());

        fs::remove_dir_all(temp_dir).ok();
    }
}
