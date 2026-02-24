//! CLI argument parsing for GBA CLI.

use clap::{Parser, Subcommand};

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
    pub path: String,

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
    /// Path to initialize the project.
    #[arg(short, long)]
    pub path: Option<String>,

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

    /// Task kind (planning, implementation, verification).
    #[arg(short, long)]
    pub kind: String,

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

impl Args {
    /// Parse command line arguments.
    #[must_use]
    #[allow(dead_code)]
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        // Use default values for parsing test
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
}
