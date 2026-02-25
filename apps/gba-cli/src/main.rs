//! GBA CLI - Command line interface for GBA.
//!
//! This crate provides a CLI for interacting with GBA using clap and ratatui.

use anyhow::{Context as AnyhowContext, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use tracing::{Level, debug, info};
use tracing_subscriber::{EnvFilter, prelude::*};

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

    // Console filter - only warnings and errors to stdout
    let console_filter = EnvFilter::new("warn,gba_cli=warn");

    // Check if we have a config with log file settings
    let project_path = if args.path.as_os_str() == "." {
        std::env::current_dir().context("Failed to get current directory")?
    } else {
        args.path.clone()
    };

    // Try to load config for log file settings
    let log_file = if let Some(config) = ConfigManager::try_load(&project_path) {
        let cfg = config.config();
        if !cfg.logging.file.is_empty() {
            Some(Path::new(&cfg.logging.file).to_path_buf())
        } else {
            // Default to .gba/logs/gba.log if file is empty but directory exists
            let gba_dir = project_path.join(".gba");
            if gba_dir.exists() {
                Some(gba_dir.join("logs").join("gba.log"))
            } else {
                // Use global default log location
                get_default_log_file()
            }
        }
    } else {
        // No config, use global default log location
        get_default_log_file()
    };

    if let Some(ref file_path) = log_file {
        // Create parent directory if needed
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create log directory")?;
        }

        // File appender
        let file_appender = tracing_appender::rolling::daily(
            file_path.parent().unwrap_or(Path::new(".")),
            file_path
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("gba.log")),
        );

        tracing_subscriber::registry()
            .with(filter.clone())
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(std::io::stdout)
                    .with_ansi(true)
                    .with_target(false)
                    .with_thread_ids(false)
                    .with_file(false)
                    .with_line_number(false)
                    .with_filter(console_filter),
            )
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(file_appender)
                    .with_ansi(false)
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .init();
    } else {
        // Console only logging (fallback when no log file can be created)
        tracing_subscriber::registry()
            .with(filter)
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(std::io::stdout)
                    .with_ansi(true)
                    .with_target(false)
                    .with_thread_ids(false)
                    .with_file(false)
                    .with_line_number(false),
            )
            .init();
    }

    Ok(())
}

/// Execute init command.
async fn execute_init(args: cli::InitArgs) -> Result<()> {
    let project_path = args
        .path
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    run::init(&project_path, &args.main_branch, args.repo_url.as_deref()).await?;

    Ok(())
}

/// Get the default log file location.
///
/// Returns `~/.gba/logs/gba.log` or None if home directory cannot be determined.
#[must_use]
fn get_default_log_file() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".gba").join("logs").join("gba.log"))
}

/// Execute run command.
async fn execute_run(project_path: PathBuf, args: cli::RunArgs) -> Result<()> {
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
