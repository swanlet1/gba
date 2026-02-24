//! GBA CLI - Command line interface for GBA.
//!
//! This crate provides a CLI for interacting with GBA using clap and ratatui.

use anyhow::Result;
use clap::Parser;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

mod cli;
mod error;
mod run;
mod ui;

use cli::Args;
use cli::Command;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    init_tracing(&args)?;

    // Execute command
    match args.command {
        Command::Init(init_args) => execute_init(init_args).await?,
        Command::Run(run_args) => execute_run(run_args).await?,
        Command::ListPrompts(list_args) => execute_list_prompts(list_args).await?,
        Command::Prompt(prompt_args) => execute_prompt(prompt_args).await?,
    }

    Ok(())
}

/// Initialize tracing subscriber.
fn init_tracing(args: &Args) -> Result<()> {
    let filter = if args.verbose {
        EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into())
    } else {
        EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into())
    };

    fmt().with_env_filter(filter).with_target(false).init();

    Ok(())
}

/// Execute init command.
async fn execute_init(_args: cli::InitArgs) -> Result<()> {
    info!("Initializing GBA project");
    // TODO: Implement init logic
    Ok(())
}

/// Execute run command.
async fn execute_run(args: cli::RunArgs) -> Result<()> {
    info!(
        feature = %args.feature,
        kind = %args.kind,
        "Running task"
    );
    run::run(args).await?;
    Ok(())
}

/// Execute list-prompts command.
async fn execute_list_prompts(_args: cli::ListPromptsArgs) -> Result<()> {
    info!("Listing available prompts");
    // TODO: Implement list prompts logic
    Ok(())
}

/// Execute prompt command.
async fn execute_prompt(_args: cli::PromptArgs) -> Result<()> {
    info!("Executing prompt");
    // TODO: Implement prompt logic
    Ok(())
}
