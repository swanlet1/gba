//! CLI argument parsing for GBA CLI.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// GBA CLI - GeekTime Bootcamp Agent
///
/// A CLI tool that wraps the Claude Agent SDK for adding functionality around repositories.
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Command,

    /// Path to the GBA project directory.
    #[arg(short, long, default_value = ".")]
    pub path: PathBuf,

    /// Verbose output.
    #[arg(short, long)]
    pub verbose: bool,
}

/// Available subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Initialize a new GBA project.
    Init(InitArgs),

    /// Run an agent on a repository.
    Run(RunArgs),

    /// List available prompts.
    ListPrompts(ListPromptsArgs),

    /// Execute a single prompt.
    Prompt(PromptArgs),
}

/// Arguments for the init subcommand.
#[derive(Debug, clap::Args)]
pub struct InitArgs {
    /// Path to initialize the project (defaults to current directory).
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// Main branch name.
    #[arg(long, default_value = "main")]
    pub main_branch: String,

    /// Repository URL.
    #[arg(short, long)]
    pub repo_url: Option<String>,
}

/// Arguments for the run subcommand.
#[derive(Debug, clap::Args)]
pub struct RunArgs {
    /// Feature name to work on.
    #[arg(short, long)]
    pub feature: String,

    /// Task kind.
    #[arg(short, long)]
    pub kind: TaskKind,

    /// Feature description.
    #[arg(short, long)]
    pub description: Option<String>,

    /// Use TUI mode.
    #[arg(long)]
    pub tui: bool,

    /// Resume from previous state.
    #[arg(long)]
    pub resume: bool,
}

/// Task kind for execution.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TaskKind {
    /// Create an implementation plan.
    Planning,

    /// Execute the implementation.
    Implementation,

    /// Verify the implementation.
    Verification,
}

impl std::fmt::Display for TaskKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Planning => write!(f, "planning"),
            Self::Implementation => write!(f, "implementation"),
            Self::Verification => write!(f, "verification"),
        }
    }
}

impl TaskKind {
    /// Get the template name for this task kind.
    #[must_use]
    pub const fn template_name(&self) -> &str {
        match self {
            Self::Planning => "plan",
            Self::Implementation => "implement",
            Self::Verification => "verify",
        }
    }
}

/// Arguments for the list-prompts subcommand.
#[derive(Debug, clap::Args)]
pub struct ListPromptsArgs {
    /// Show detailed information about each prompt.
    #[arg(short, long)]
    pub verbose: bool,
}

/// Arguments for the prompt subcommand.
#[derive(Debug, clap::Args)]
pub struct PromptArgs {
    /// Template name to use.
    #[arg(short, long)]
    pub template: String,

    /// User message.
    #[arg(short, long)]
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::try_parse_from([
            "gba",
            "run",
            "--feature",
            "test",
            "--kind",
            "implementation",
        ]);
        assert!(args.is_ok());
        if let Ok(args) = args {
            assert!(matches!(args.command, Command::Run(_)));
        }
    }

    #[test]
    fn test_task_kind_display() {
        assert_eq!(TaskKind::Planning.to_string(), "planning");
        assert_eq!(TaskKind::Implementation.to_string(), "implementation");
        assert_eq!(TaskKind::Verification.to_string(), "verification");
    }

    #[test]
    fn test_task_kind_template_name() {
        assert_eq!(TaskKind::Planning.template_name(), "plan");
        assert_eq!(TaskKind::Implementation.template_name(), "implement");
        assert_eq!(TaskKind::Verification.template_name(), "verify");
    }
}
