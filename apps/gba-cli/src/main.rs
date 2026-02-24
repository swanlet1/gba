//! GBA CLI - Command line interface for GBA.
//!
//! This crate provides a CLI for interacting with GBA using clap and ratatui.

use anyhow::{Context as AnyhowContext, Result};
use clap::Parser;
use std::path::PathBuf;
use tracing::{Level, debug, info};
use tracing_subscriber::{EnvFilter, fmt};

mod cli;
mod config;
mod error;
mod output;
mod run;
mod ui;

use cli::{Args, Command};
use config::ConfigManager;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    init_tracing(&args)?;

    debug!("GBA CLI starting with command: {:?}", args.command);

    // Resolve the project path
    let project_path = if args.path.as_os_str() == "." {
        std::env::current_dir().context("Failed to get current directory")?
    } else {
        args.path
    };

    debug!("Project path: {}", project_path.display());

    // Execute command
    match args.command {
        Command::Init(init_args) => execute_init(init_args).await?,
        Command::Run(run_args) => execute_run(project_path, run_args).await?,
        Command::ListPrompts(list_args) => execute_list_prompts(project_path, list_args).await?,
        Command::Prompt(prompt_args) => execute_prompt(project_path, prompt_args).await?,
    }

    info!("GBA CLI completed successfully");
    Ok(())
}

/// Initialize tracing subscriber.
fn init_tracing(args: &Args) -> Result<()> {
    let log_level = if args.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    let filter = EnvFilter::from_default_env()
        .add_directive(log_level.into())
        .add_directive("gba_core=info".parse()?)
        .add_directive("gba_pm=info".parse()?)
        .add_directive("gba_cli=info".parse()?);

    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    Ok(())
}

/// Execute init command.
async fn execute_init(args: cli::InitArgs) -> Result<()> {
    info!("Initializing GBA project");

    let project_path = args
        .path
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    run::init(&project_path, &args.main_branch, args.repo_url.as_deref()).await?;

    Ok(())
}

/// Execute run command.
async fn execute_run(project_path: PathBuf, args: cli::RunArgs) -> Result<()> {
    info!(
        feature = %args.feature,
        kind = %args.kind,
        tui = args.tui,
        resume = args.resume,
        "Running task"
    );

    // Load configuration
    let config = ConfigManager::load(&project_path).with_context(|| {
        format!(
            "Failed to load configuration from {}",
            project_path.display()
        )
    })?;

    run::run(config, args).await?;

    Ok(())
}

/// Execute list-prompts command.
async fn execute_list_prompts(project_path: PathBuf, args: cli::ListPromptsArgs) -> Result<()> {
    info!("Listing available prompts");

    let config = ConfigManager::load(&project_path).with_context(|| {
        format!(
            "Failed to load configuration from {}",
            project_path.display()
        )
    })?;

    run::list_prompts(config, args.verbose)?;

    Ok(())
}

/// Execute prompt command.
async fn execute_prompt(project_path: PathBuf, args: cli::PromptArgs) -> Result<()> {
    info!("Executing prompt: {}", args.template);

    let config = ConfigManager::load(&project_path).with_context(|| {
        format!(
            "Failed to load configuration from {}",
            project_path.display()
        )
    })?;

    run::execute_prompt(config, &args.template, &args.message).await?;

    Ok(())
}
