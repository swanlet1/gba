# GBA Design Document

## Overview

GBA (GeekTime Bootcamp Agent) is a Rust-based CLI tool that wraps the Claude Agent SDK, enabling users to easily add new functionality around a repository. The project is organized as a workspace with three main components:

- **gba-core**: Core execution engine
- **gba-pm**: Prompt manager
- **gba-cli**: Command line interface

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         gba-cli (UI Layer)                      │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────┐  │  │
│  │  │   args   │  │   ui     │  │  config  │  │  logs  │  │  │
│  │  │  parser  │  │  tui/tc  │  │  loader  │  │ output │  │  │
│  │  └──────────┘  └──────────┘  └──────────┘  └────────┘  │  │
│  └──────────────────────────────────────────────────────────┘  │
└──────────────────────────────┬──────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                      gba-pm (Prompt Layer)                      │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  ┌──────────┐  ┌──────────────────┐  ┌──────────────┐  │  │
│  │  │  prompt  │  │ template engine  │  │  prompt      │  │  │
│  │  │ registry │  │  (minijinja)     │  │  loader      │  │  │
│  │  └──────────┘  └──────────────────┘  └──────────────┘  │  │
│  └──────────────────────────────────────────────────────────┘  │
└──────────────────────────────┬──────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                     gba-core (Core Engine)                      │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  ┌──────────┐  ┌──────────────────┐  ┌──────────────┐  │  │
│  │  │  agent   │  │  claude sdk      │  │  task        │  │  │
│  │  │  runner  │  │  adapter         │  │  executor    │  │  │
│  │  └──────────┘  └──────────────────┘  └──────────────┘  │  │
│  │  ┌──────────────────────────────────────────────────┐   │  │
│  │  │  error handling │  tracing │  config │  state   │   │  │
│  │  └──────────────────────────────────────────────────┘   │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │  Claude Agent SDK   │
                    │  claude-agent-sdk   │
                    └─────────────────────┘
```

### Directory Structure

```
gba/
├── Cargo.toml                 # Workspace configuration
├── rust-toolchain.toml        # Pinned Rust version
├── specs/                     # Design documentation
│   └── design.md             # This document
├── crates/
│   ├── gba-core/             # Core execution engine
│   │   ├── src/
│   │   │   ├── lib.rs       # Public API exports
│   │   │   ├── agent.rs     # Agent runner and execution
│   │   │   ├── error.rs     # Error types
│   │   │   ├── config.rs    # Configuration types
│   │   │   └── task.rs      # Task execution logic
│   │   └── Cargo.toml
│   └── gba-pm/              # Prompt manager
│       ├── src/
│       │   ├── lib.rs       # Public API exports
│       │   ├── prompt.rs    # Prompt management
│       │   ├── template.rs  # Template engine wrapper
│       │   └── error.rs     # Error types
│       └── Cargo.toml
└── apps/
    └── gba-cli/             # CLI application
        ├── src/
        │   ├── main.rs      # Entry point
        │   ├── cli.rs       # CLI argument parsing
        │   ├── ui.rs        # UI/TUI implementation
        │   └── run.rs       # Run command logic
        └── Cargo.toml
```

## Component Responsibilities

### gba-core: Core Execution Engine

**Purpose**: The core execution engine that manages agent lifecycle, executes tasks, and interfaces with the Claude Agent SDK.

**Responsibilities**:
- Agent lifecycle management (start, stop, restart)
- Task scheduling and execution
- Claude Agent SDK integration
- Configuration management
- Error handling and context propagation
- Tracing and observability

**Public Interface**:

```rust
// Agent configuration
pub struct AgentConfig {
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    // ...
}

// Agent runner
pub struct Agent {
    config: AgentConfig,
    client: ClaudeClient,
    // ...
}

impl Agent {
    /// Create a new agent with the given configuration
    pub async fn new(config: AgentConfig) -> Result<Self>;

    /// Execute a task with the given prompt and context
    pub async fn execute(&self, prompt: &str, context: &Context) -> Result<Response>;

    /// Execute a streaming task
    pub async fn execute_stream(&self, prompt: &str, context: &Context)
        -> Result<impl Stream<Item = Result<Chunk>>>;

    /// Stop the agent gracefully
    pub async fn shutdown(self) -> Result<()>;
}

// Task context
pub struct Context {
    pub repository: Repository,
    pub files: Vec<File>,
    pub metadata: HashMap<String, Value>,
}

// Response type
pub struct Response {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub usage: Usage,
}
```

### gba-pm: Prompt Manager

**Purpose**: Template-based prompt management using Minijinja for flexible prompt rendering.

**Responsibilities**:
- Prompt template loading and caching
- Template rendering with context variables
- Prompt registry for managing named prompts
- Template validation and error handling

**Public Interface**:

```rust
// Template engine
pub struct TemplateEngine {
    env: minijinja::Environment<'static>,
}

impl TemplateEngine {
    /// Create a new template engine
    pub fn new() -> Self;

    /// Load templates from a directory
    pub async fn load_templates(&mut self, path: &Path) -> Result<()>;

    /// Add a template from a string
    pub fn add_template(&mut self, name: &str, source: &str) -> Result<()>;

    /// Render a template with the given context
    pub fn render(&self, name: &str, ctx: &Context) -> Result<String>;
}

// Prompt manager
pub struct PromptManager {
    engine: TemplateEngine,
    registry: HashMap<String, TemplateRef>,
}

impl PromptManager {
    /// Create a new prompt manager
    pub fn new(engine: TemplateEngine) -> Self;

    /// Register a named prompt
    pub fn register(&mut self, name: &str, template: &str) -> Result<()>;

    /// Get a rendered prompt by name
    pub fn get_prompt(&self, name: &str, ctx: &Context) -> Result<String>;

    /// List all registered prompt names
    pub fn list_prompts(&self) -> Vec<&String>;
}

// Template context
#[derive(serde::Serialize, Debug)]
pub struct Context {
    pub repo_path: String,
    pub branch: String,
    pub files: Vec<FileContext>,
    pub user_message: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(serde::Serialize, Debug)]
pub struct FileContext {
    pub path: String,
    pub content: String,
    pub language: String,
}
```

### gba-cli: Command Line Interface

**Purpose**: User-facing CLI that provides commands for interacting with GBA.

**Responsibilities**:
- Command-line argument parsing with clap
- TUI interface with ratatui
- Configuration file management
- User interaction handling
- Logging and output formatting
- Integration with gba-core and gba-pm

**Public Interface**:

```rust
// Main CLI structure (internal)
struct Args {
    #[command(subcommand)]
    command: Option<Command>,
}

// Commands
enum Command {
    /// Initialize a new GBA project
    Init(InitArgs),
    /// Run an agent on a repository
    Run(RunArgs),
    /// List available prompts
    ListPrompts(ListPromptsArgs),
    /// Execute a single prompt
    Prompt(PromptArgs),
}

// Run command
struct RunArgs {
    /// Repository path to work on
    #[arg(short, long)]
    repo: PathBuf,

    /// Agent configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Use TUI mode
    #[arg(long)]
    tui: bool,
}

// Example main flow
async fn run(args: RunArgs) -> anyhow::Result<()> {
    // 1. Load configuration
    let config = load_config(args.config)?;

    // 2. Initialize prompt manager
    let pm = PromptManager::new(...)?;

    // 3. Create agent
    let agent = Agent::new(config).await?;

    // 4. Execute task
    let response = agent.execute(prompt, context).await?;

    // 5. Display result
    display(response);

    Ok(())
}
```

## Key Flows

### Task Execution Flow

```
User Input (CLI)
    │
    ▼
┌─────────────────────┐
│ Parse Arguments     │ (clap)
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Load Configuration  │ (config file)
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Initialize Context  │ (repo scan, file loading)
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Render Prompt       │ (gba-pm)
│ using templates     │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Execute Agent       │ (gba-core)
│ via Claude SDK      │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Display Response    │ (TUI/stdout)
└─────────────────────┘
```

### Agent Lifecycle Flow

```
Start
    │
    ▼
┌─────────────────────┐
│ Validate Config     │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Create Claude       │
│ Client              │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Ready State         │ ←──┐
└──────────┬──────────┘    │
           │              │
           │ Execute      │ Stop
           ▼              │
┌─────────────────────┐    │
│ Process Request     │    │
│ - Render Prompt     │    │
│ - Call SDK          │    │
│ - Handle Stream     │    │
└──────────┬──────────┘    │
           │              │
           ▼              │
┌─────────────────────┐    │
│ Return Response     │ ───┘
└─────────────────────┘
```

### Prompt Rendering Flow

```
Request Prompt
    │
    ▼
┌─────────────────────┐
│ Look up Template    │ (by name)
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Prepare Context     │ (inject vars)
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Render Template     │ (minijinja)
│ - Evaluate filters  │
│ - Process loops     │
│ - Include partials  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Return Rendered     │
│ Prompt              │
└─────────────────────┘
```

## Design Principles

### SOLID Principles

1. **Single Responsibility**: Each crate and module has a single, well-defined purpose
2. **Open/Closed**: Extensible through plugins/templates without modifying core code
3. **Liskov Substitution**: Trait abstractions allow interchangeable implementations
4. **Interface Segregation**: Small, focused interfaces (e.g., separate `Agent` vs `TemplateEngine`)
5. **Dependency Inversion**: Depend on abstractions (traits), not concrete implementations

### Rust Best Practices

- Use `Result<T>` for fallible operations, never `Option` for errors
- Leverage `thiserror` for domain-specific error types
- Use `async` with `tokio` for I/O-bound operations
- Zero-cost abstractions with traits and generics
- Memory safety with ownership and borrowing
- Type-level guarantees (e.g., `NonZeroU32`, type-state pattern)

## Development Plan

### Phase 1: Foundation (Week 1)

**Tasks**:
1. Set up workspace structure and dependencies
2. Implement core error types for all crates
3. Create configuration types and loading logic
4. Set up basic project structure (docs, tests, CI)

**Deliverables**:
- Complete workspace setup
- All three crates compile with basic skeleton
- Error handling framework in place
- CI/CD pipeline configured

### Phase 2: Prompt Manager (Week 2)

**Tasks**:
1. Implement `TemplateEngine` with minijinja integration
2. Create `PromptManager` with registry functionality
3. Add template loading from directories
4. Implement context variable injection
5. Add template validation

**Deliverables**:
- Fully functional prompt manager
- Template loading and rendering
- Unit tests for template engine
- Example templates

### Phase 3: Core Engine (Week 3-4)

**Tasks**:
1. Integrate Claude Agent SDK
2. Implement `Agent` lifecycle management
3. Create task execution logic
4. Add streaming support
5. Implement context building (repo scanning)
6. Add configuration management

**Deliverables**:
- Functional agent runner
- Task execution with prompts
- Streaming responses
- Configuration file support

### Phase 4: CLI (Week 5)

**Tasks**:
1. Implement argument parsing with clap
2. Create command handlers (init, run, list)
3. Add basic TUI with ratatui
4. Implement output formatting
5. Add logging configuration

**Deliverables**:
- Working CLI with all commands
- TUI interface
- User documentation
- Help text and examples

### Phase 5: Polish (Week 6)

**Tasks**:
1. Add comprehensive tests (unit, integration)
2. Improve error messages
3. Add benchmarks
4. Write documentation
5. Performance optimization
6. Security audit

**Deliverables**:
- High test coverage
- Complete documentation
- Performance benchmarks
- v0.1.0 release

## Configuration Format

### Example Config (config.yaml)

```yaml
# GBA Configuration
agent:
  model: "claude-sonnet-4-20250514"
  max_tokens: 4096
  temperature: 0.7
  timeout: 300

prompts:
  directory: "./prompts"
  default: "code-review"

repository:
  exclude_patterns:
    - "target/"
    - ".git/"
  max_file_size: 1048576  # 1MB

logging:
  level: "info"
  format: "human"  # or "json" for production
```

### Example Template (prompts/code-review.jinja2)

```jinja2
You are a code review assistant reviewing the following changes.

Repository: {{ repo_path }}
Branch: {{ branch }}

Files changed:
{% for file in files %}
- {{ file.path }} ({{ file.language }})

```{{ file.content }}```

{% endfor %}

User message: {{ user_message }}

Please provide a thorough code review focusing on:
1. Correctness and potential bugs
2. Performance considerations
3. Code style and conventions
4. Documentation completeness
```

## Testing Strategy

### Unit Tests
- Individual module testing
- Mock external dependencies
- Edge case coverage

### Integration Tests
- End-to-end workflow testing
- Real Claude API (with test keys)
- Template rendering with real templates

### Property-Based Tests
- Use `proptest` for invariants
- Template rendering correctness
- Configuration parsing

## Security Considerations

1. **API Keys**: Use `secrecy` crate to handle API keys in memory
2. **Input Validation**: Validate all user inputs, file paths, templates
3. **Code Execution**: Never execute arbitrary code from LLM responses
4. **Path Traversal**: Validate file access paths, restrict to repo directory
5. **TLS**: Use `rustls` for all network communications

## Performance Considerations

1. **Caching**: Cache rendered prompts, file contents
2. **Streaming**: Stream responses to minimize latency
3. **Parallel Processing**: Parallel file scanning with Rayon
4. **Memory**: Use `Cow<str>` to avoid unnecessary allocations
5. **Async**: Proper async/await for I/O operations

## Future Enhancements

1. **Plugin System**: Allow custom plugins for additional functionality
2. **Multiple Agents**: Support running multiple agents concurrently
3. **Web UI**: Browser-based interface using WebSocket
4. **History**: Persistent command history and results
5. **Workflows**: Multi-step task workflows
6. **Export**: Export results to various formats (markdown, PDF, etc.)
