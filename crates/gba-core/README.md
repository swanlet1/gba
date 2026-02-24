# GBA Core - Core Execution Engine

This crate provides the core functionality for interacting with the Claude Agent SDK, enabling users to easily add new functionality around a repository.

## Features

- Agent lifecycle management (start, stop, restart)
- Task execution with prompts and context
- Streaming response support
- Repository scanning and context building
- Configuration management
- Comprehensive error handling

## Usage

### Basic Usage

```rust
use gba_core::{Agent, AgentConfig, Context};

#[tokio::main]
async fn main() -> Result<(), gba_core::CoreError> {
    let config = AgentConfig::default();
    let mut agent = Agent::new(config).await?;
    agent.connect().await?;

    let context = Context {
        repository_path: "/path/to/repo".into(),
        branch: "main".to_string(),
        files: vec![],
        metadata: Default::default(),
    };

    let response = agent.execute("Hello, Claude!", &context).await?;
    println!("{}", response.content);

    agent.shutdown().await?;
    Ok(())
}
```

### Streaming Responses

```rust
use gba_core::{Agent, AgentConfig, Chunk};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), gba_core::CoreError> {
    let config = AgentConfig::default();
    let mut agent = Agent::new(config).await?;
    agent.connect().await?;

    let context = Context::default();
    let mut stream = agent.execute_stream("Explain Rust ownership", &context).await?;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result? {
            Chunk::Text(text) => print!("{}", text),
            Chunk::Done(usage) => {
                println!("\nCost: ${}", usage.total_cost_usd);
            }
        }
    }

    agent.shutdown().await?;
    Ok(())
}
```

### Context Building

```rust
use gba_core::context_builder::{build_context, ContextBuilderConfig};

#[tokio::main]
async fn main() -> Result<(), gba_core::CoreError> {
    let config = ContextBuilderConfig::default()
        .with_max_files(50)
        .with_include_extensions(vec!["rs".to_string()]);

    let context = build_context(
        PathBuf::from("/path/to/repo"),
        "main",
        &config,
    ).await?;

    println!("Found {} files", context.files.len());
    Ok(())
}
```

## Configuration

Create an `AgentConfig` to customize the agent behavior:

```rust
use gba_core::AgentConfig;

let config = AgentConfig {
    model: "claude-sonnet-4-20250514".to_string(),
    max_tokens: 4096,
    temperature: 0.7,
    timeout: 300,
};
```

## Error Handling

All operations return `Result<T, CoreError>` where `CoreError` can be:

- `ClaudeAgent(String)` - Errors from the Claude Agent SDK
- `Config(String)` - Configuration errors
- `Io(std::io::Error)` - I/O errors
- `Serde(serde_json::Error)` - Serialization errors

## License

MIT License - see the main project LICENSE.md for details.
