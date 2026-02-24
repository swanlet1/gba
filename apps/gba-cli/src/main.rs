//! GBA CLI - Command line interface for GBA.
//!
//! This crate provides a CLI for interacting with GBA using clap and ratatui.

use clap::Parser;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // TODO: Implement CLI logic based on args
    println!("GBA CLI v{}", env!("CARGO_PKG_VERSION"));

    if let Some(command) = args.command {
        println!("Subcommand: {:?}", command);
    }

    Ok(())
}

/// Command line arguments for GBA CLI.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Subcommand to execute.
    #[command(subcommand)]
    command: Option<Command>,
}

/// Available subcommands.
#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Initialize a new GBA project.
    Init(InitArgs),
    /// Run an agent on a repository.
    Run(RunArgs),
}

/// Arguments for the init subcommand.
#[derive(Debug, clap::Args)]
struct InitArgs {
    /// Path to initialize the project.
    #[arg(short, long)]
    path: Option<String>,
}

/// Arguments for the run subcommand.
#[derive(Debug, clap::Args)]
struct RunArgs {
    /// Repository path to work on.
    #[arg(short, long)]
    repo: String,

    /// Agent configuration file.
    #[arg(short, long)]
    config: Option<String>,
}
