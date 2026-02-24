# GBA (GeekTime Bootcamp Agent)

GBA is a Rust-based CLI tool that wraps the Claude Agent SDK, enabling users to easily add new functionality around a repository.

## Overview

GBA provides a structured workflow for developing features with AI assistance:

1. **Planning** - Create detailed implementation plans
2. **Implementation** - Execute the plan with automatic commits
3. **Verification** - Review and test the implementation
4. **Resumption** - Resume interrupted tasks from where they left off

## Architecture

GBA is organized as a workspace with three main components:

- **gba-core**: Core execution engine that manages agent lifecycle and executes tasks
- **gba-pm**: Prompt manager for template-based prompt rendering using Minijinja
- **gba-cli**: Command line interface with TUI support

## Installation

```bash
# Build from source
cargo build --release

# The binary will be available at target/release/gba
```

## Quick Start

```bash
# Initialize a GBA project in your repository
gba init

# Create an implementation plan for a feature
gba run --feature add-auth --kind planning --description "Add authentication"

# Implement the feature
gba run --feature add-auth --kind implementation

# Verify the implementation
gba run --feature add-auth --kind verification

# List available prompts
gba list-prompts

# Run with TUI mode
gba run --feature add-auth --kind implementation --tui

# Resume an interrupted task
gba run --feature add-auth --kind implementation --resume
```

## Configuration

GBA uses a project-specific configuration file at `.gba/config.yml`:

```yaml
# GBA Project Configuration
version: "1.0"

# Project metadata
project:
  name: "my-project"
  repository:
    url: "https://github.com/user/repo.git"
    mainBranch: "main"

# Agent defaults
agent:
  model: "claude-sonnet-4-20250514"
  maxTokens: 4096
  temperature: 0.7
  timeout: 300

# Prompt templates configuration
prompts:
  directory: "./.gba/templates"
  useBundled: true

# Repository scanning settings
repository:
  excludePatterns:
    - "target/"
    - ".git/"
    - "node_modules/"
  maxFileSize: 1048576  # 1MB

# Logging configuration
logging:
  level: "info"
  format: "human"

# Worktree configuration
worktree:
  directory: "./.trees"
  branchPrefix: "gba/"

# Execution limits
limits:
  maxTurns: 100
  maxCostUsd: 10.0
```

## Templates

GBA uses Jinja2 templates for prompts. Templates are resolved in this order:

1. Local templates at `.gba/templates/` (user overrides)
2. Bundled templates from the `gba-pm` crate

Each template includes front matter for configuration:

```yaml
---
systemPrompt: "You are an expert software developer"
usePreset: true
tools:
  - Read
  - Write
maxTurns: 100
---
```

### Available Templates

| Template | Purpose | usePreset | tools |
|----------|---------|-----------|-------|
| `init` | Initialize GBA project | `false` | `Write`, `Bash` |
| `plan` | Create implementation plan | `false` | `Read` |
| `implement` | Execute implementation | `true` | `[]` (all tools) |
| `verify` | Verify implementation | `true` | `Read`, `Bash` |
| `review` | Code review | `true` | `Read` |
| `resume` | Resume interrupted task | *dynamic* | *dynamic* |

## Usage Examples

### Using GBA as a Library

```rust
use gba_core::{Agent, AgentConfig};
use gba_pm::{PromptManager, Context};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an agent
    let config = AgentConfig::default();
    let mut agent = Agent::new(config).await?;
    agent.connect().await?;

    // Create a prompt manager
    let mut prompt_manager = PromptManager::new()?;
    prompt_manager.register("my_task", "Hello, {{ name }}!")?;

    // Render a prompt
    let context = Context::new("/path/to/repo", "main", "Help me");
    let prompt = prompt_manager.get_prompt("my_task", &context)?;

    // Execute the task
    let response = agent.execute(&prompt, &Default::default()).await?;

    println!("{}", response.content);
    agent.shutdown().await?;

    Ok(())
}
```

### Custom Templates

Create custom templates in `.gba/templates/`:

```jinja2
---
systemPrompt: "You are a senior engineer"
usePreset: true
tools: []
---

You are working on: {{ feature_name }}

## Context

{{ implementation_plan }}

## Instructions

Please implement the following requirements...
```

## Development

```bash
# Build the project
cargo build

# Run tests
cargo test

# Format code
cargo +nightly fmt

# Check for issues
cargo clippy -- -D warnings
```

## Project Structure

```
gba/
├── Cargo.toml                 # Workspace configuration
├── specs/
│   └── design.md             # Design documentation
├── crates/
│   ├── gba-core/             # Core execution engine
│   │   └── src/
│   │       ├── lib.rs       # Public API exports
│   │       ├── agent.rs     # Agent runner and execution
│   │       ├── error.rs     # Error types
│   │       ├── config.rs    # Configuration types
│   │       ├── task.rs      # Task execution logic
│   │       └── context_builder.rs  # Repository scanning
│   └── gba-pm/              # Prompt manager
│       ├── src/
│       │   ├── lib.rs       # Public API exports
│       │   ├── prompt.rs    # Prompt management
│       │   ├── template.rs  # Template engine wrapper
│       │   ├── config.rs    # Context types
│       │   └── error.rs     # Error types
│       └── templates/       # Bundled templates
└── apps/
    └── gba-cli/             # CLI application
        └── src/
            ├── main.rs      # Entry point
            ├── cli.rs       # CLI argument parsing
            ├── run.rs       # Command handlers
            ├── config.rs    # Configuration management
            ├── error.rs     # Error types
            ├── output.rs    # Output formatting
            └── ui.rs        # TUI implementation
```

## License

MIT License - see [LICENSE.md](LICENSE.md) for details.
