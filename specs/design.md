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
│       ├── templates/       # Bundled templates (included in binary)
│       │   ├── init.jinja2
│       │   ├── plan.jinja2
│       │   ├── implement.jinja2
│       │   ├── verify.jinja2
│       │   ├── review.jinja2
│       │   └── resume.jinja2
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

### Task Resume Flow

```
Execute Task (gba run)
    │
    ▼
┌─────────────────────┐
│ Check State File    │ (.gba/features/{feature}/state.yml)
└──────────┬──────────┘
           │
           ▼
      ┌────┴─────┐
      │          │
      ▼          ▼
┌─────────┐  ┌─────────┐
│ State   │  │ State   │
│ Exists? │  │ Missing │
└────┬────┘  └────┬────┘
     │             │
     ▼             ▼
┌─────────┐  ┌─────────┐
│ Check   │  │ Start   │
│ Status  │  │ Fresh   │
└────┬────┘  └────┬────┘
     │             │
     ▼             │
┌─────────┐        │
│ Status? │        │
└────┬────┘        │
     │             │
     ├─ Completed  │
     │  └─► Report │
     │             │
     ├─ Failed     │
     │  └─► Ask User│
     │             │
     └─ In Progress
        └─► Load Context
             │
             ▼
┌─────────────────────┐
│ Render Resume       │
│ Prompt with Context │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ Continue Execution  │
│ from Last Phase     │
└─────────────────────┘
```

## Prompt Templates

### Template Location

GBA supports two template locations:

1. **Bundled templates**: Included in the binary at `crates/gba-pm/templates/`
2. **Local templates**: User project templates at `.gba/templates/`

```
# Bundled templates (in gba-pm crate)
crates/gba-pm/templates/
├── init.jinja2              # Initialize GBA project
├── plan.jinja2              # Create implementation plan
├── implement.jinja2         # Execute implementation
├── verify.jinja2            # Verification and testing
├── review.jinja2            # Code review (optional/manual)
└── resume.jinja2            # Resume interrupted task

# Local templates (in user project)
.gba/templates/
├── init.jinja2              # Optional override
├── plan.jinja2              # Optional override
└── ...                     # Any templates overridden here take precedence
```

**Template resolution:**
1. First check `.gba/templates/` (local override)
2. If not found, use bundled templates from `gba-pm`
3. If `use_bundled: false` in config, only local templates are used

### Template Metadata

Each template specifies agent behavior using YAML front matter:

```yaml
---
# System Prompt Configuration
# usePreset: true → Use Claude Code preset ("claude_code")
# usePreset: false → Use custom systemPrompt as plain text
systemPrompt: "You are an expert..."  # Optional: append to preset if usePreset: true
usePreset: true  # true = use Claude Code preset, false = use systemPrompt

# Tool Configuration
# Empty array = all Claude Code tools available
# Specific tools = only those tools enabled
tools: []  # or ["Read", "Write"] for restricted tools

# Execution limits (optional, overrides global config)
maxTurns: 100
---
```

**Front matter fields:**

| Field | Type | Description |
|-------|------|-------------|
| `systemPrompt` | String | Custom system prompt text (if `usePreset: false`, used directly; if `usePreset: true`, appended to Claude Code preset) |
| `usePreset` | Boolean | `true` = use Claude Code preset (`"claude_code"`), `false` = use `systemPrompt` only |
| `tools` | `Vec<String>` | Tools to enable (empty array `[]` = all tools available, specific list = restricted tools) |
| `maxTurns` | Number | Maximum turns for this task (optional, default from config) |

**Rust mapping:**

```rust
// When usePreset: true
let system_prompt = SystemPrompt::Preset(
    SystemPromptPreset::with_append("claude_code", system_prompt_from_template)
);

// When usePreset: false
let system_prompt = SystemPrompt::Text(system_prompt_from_template);
```

**Template Configuration Summary:**

| Template | usePreset | tools | Rationale |
|----------|-----------|-------|-----------|
| `init.jinja2` | `false` | `["Write", "Bash"]` | File creation only, doesn't need code understanding |
| `plan.jinja2` | `false` | `["Read"]` | Architecture planning only needs to read existing code |
| `implement.jinja2` | `true` | `[]` | Full coding work, needs Claude Code preset and all tools |
| `review.jinja2` | `true` | `["Read"]` | Code review needs code understanding but doesn't write |
| `verify.jinja2` | `true` | `["Read", "Bash"]` | Verification needs to run tests and read code |
| `resume.jinja2` | *dynamic* | *dynamic* | Inherits from original task's template (context-injected) |

### Template Context Variables

Templates have access to these context variables:

| Variable | Templates | Description |
|----------|-----------|-------------|
| `main_branch` | init | Main branch name |
| `feature_name` | plan, implement, verify, review, resume | Feature/task name |
| `feature_id` | plan, implement, verify, review, resume | Feature ID |
| `feature_description` | plan, implement, verify, review, resume | Feature description |
| `worktree_path` | implement, resume | Path to worktree |
| `worktree_branch` | implement, resume | Worktree branch name |
| `current_phase` | resume | Current phase |
| `current_step` | resume | Current step |
| `turns_so_far` | resume | Number of turns so far |
| `cost_so_far` | resume | Cost incurred so far |
| `use_preset` | resume | Boolean: whether to use Claude Code preset (injected based on task kind) |
| `tools` | resume | Array of tools (injected based on task kind) |
| `implementation_plan` | implement, resume | The implementation plan |
| `implementation_summary` | verify | Summary of implementation |
| `diff_content` | review | Git diff content for review |

**Convention over Configuration:**
- Repository path is the current working directory
- Branch is the current branch or worktree branch
- Agent infers context from the environment and state file
- Minimal configuration through variables, maximum convention

### Task Resume Capability

The `resume.jinja2` template provides the ability to resume interrupted tasks. When `gba run` is called:

1. **Check for existing state**: Look for `.gba/features/{feature}/state.yml`
2. **Evaluate state status**:
   - If `completed`: Report results and exit
   - If `failed`: Ask user whether to retry or start fresh
   - If `in_progress`: Load context and resume
   - If missing: Start fresh execution
3. **Render resume prompt**: Inject current state and progress into the template
4. **Continue execution**: Agent resumes from the last known phase/step

The resume prompt includes:
- Current phase and step information
- Turns and cost incurred so far
- Implementation plan reference
- Worktree information
- Instructions to avoid redoing completed work

### Template Definitions

#### init.jinja2 - Initialize GBA Project

```jinja2
---
systemPrompt: "You are an expert software development assistant helping to initialize a GBA (GeekTime Bootcamp Agent) project."
usePreset: false
tools:
  - Write
  - Bash
---

You are initializing a GBA project for this repository.

## Instructions

Please initialize the GBA project by completing the following steps:

### Step 1: Create Directory Structure
...

### Step 2: Update .gitignore
...

### Step 3: Create config.yml
...
```

**User Prompt Example (after rendering):**
```
You are initializing a GBA project for this repository.

## Instructions

Please initialize the GBA project by completing the following steps:

### Step 1: Create Directory Structure
...

### Step 2: Update .gitignore
...

### Step 3: Create config.yml
...
```

#### plan.jinja2 - Create Implementation Plan

```jinja2
---
systemPrompt: "You are an expert software architect creating a detailed implementation plan for a feature."
usePreset: false
tools:
  - Read
---

You are creating an implementation plan for the feature: {{ feature_name }}
...
```

**User Prompt Example:**
```
You are creating an implementation plan for the feature: add-pr-link

## Feature Details

Feature ID: 0003
Description: Add PR link to status.yml

## Repository Context

Main branch: main

## Instructions

Create a detailed implementation plan for this feature. The plan should:
...
```

#### implement.jinja2 - Execute Implementation

```jinja2
---
systemPrompt: "You are an expert software developer implementing a feature according to a detailed plan."
usePreset: true
tools: []
---

You are implementing the feature: {{ feature_name }}

## Feature Details
...
```

**User Prompt Example:**
```
You are implementing the feature: add-pr-link

## Feature Details

Feature ID: 0003
Description: Add PR link to status.yml

## Implementation Plan

[The generated plan from the plan phase]

## Repository Context

Worktree branch: gba/0003-add-pr-link

## Instructions
...
```

#### verify.jinja2 - Verification and Testing

```jinja2
---
systemPrompt: "You are an expert quality assurance engineer verifying the implementation of a feature."
usePreset: true
tools:
  - Read
  - Bash
---

You are verifying the implementation of the feature: {{ feature_name }}
...
```

#### review.jinja2 - Code Review

```jinja2
---
systemPrompt: "You are an expert code reviewer conducting a thorough review of the implementation."
usePreset: true
tools:
  - Read
---

You are conducting a code review for the feature: {{ feature_name }}
...
```

#### resume.jinja2 - Resume Interrupted Task

```jinja2
---
systemPrompt: "You are continuing work on an interrupted task. Analyze the current state and continue from where you left off."
usePreset: "{{ use_preset }}"
tools:
{% for tool in tools %}
  - {{ tool }}
{% endfor %}
---

You are resuming work on the {{ task_kind }} of feature: {{ feature_name }}
...
```

**Note:** The resume template receives `use_preset` and `tools` from context, which are injected by the engine based on the original task kind being resumed. For example:
- Resuming "implementation" task → `use_preset: true`, `tools: []`
- Resuming "planning" task → `use_preset: false`, `tools: ["Read"]`
systemPrompt: "You are an expert software architect creating a detailed implementation plan for a feature."
usePreset: true
tools: []
---

You are creating an implementation plan for the feature: {{ feature_name }}

## Feature Details

Feature ID: {{ feature_id }}
Description: {{ feature_description }}

## Repository Context

Main branch: {{ main_branch }}

## Instructions

Create a detailed implementation plan for this feature. The plan should:

1. Break down the work into logical phases (e.g., design, implementation, testing, documentation)
2. Each phase should have clear, actionable steps
3. Identify dependencies between phases
4. Estimate complexity for each phase
5. Identify potential risks or challenges
6. Specify what needs to be tested

The plan should be specific to this repository and the feature being implemented.
Consider the existing codebase structure, coding standards, and patterns already in use.
```

#### implement.jinja2 - Execute Implementation

```jinja2
---
systemPrompt: "You are an expert software developer implementing a feature according to a detailed plan."
usePreset: true
tools: []
---

You are implementing the feature: {{ feature_name }}

## Feature Details

Feature ID: {{ feature_id }}
Description: {{ feature_description }}

## Implementation Plan

{{ implementation_plan }}

## Repository Context

Worktree branch: {{ worktree_branch }}

## Instructions

Implement the feature following the plan above. Work in the git worktree at: {{ worktree_path }}

### During Implementation

For each phase in the plan:
1. Execute the steps in order
2. Make meaningful git commits as you complete chunks of work
   - Use descriptive commit messages (e.g., "feat: add user authentication", "fix: handle edge case in login")
   - Follow conventional commit format: `<type>(<scope>): <description>`
   - Types: feat, fix, docs, style, refactor, test, chore
3. Run tests after each commit and ensure all pass
4. Update the state file (.gba/features/{{ feature_id }}/state.yml) with current progress

### Final Steps

After completing all implementation phases:

1. **Final Review**: Run a quick self-review of your changes
2. **Create Pull Request**: Use `gh pr create` command to create a comprehensive PR

The PR description must follow this format and include:

```markdown
## Summary
[Brief description of what this PR accomplishes]

## Changes
[Bulleted list of main changes, organized by file or component]

## Test Plan
[Checklist of testing performed]
- [ ] Unit tests pass locally
- [ ] Integration tests pass locally
- [ ] Manual testing completed for [specific scenarios]
- [ ] Edge cases verified
- [ ] No regressions detected

## Breaking Changes & Migration Notes
[Any breaking changes or migration steps required - or "None"]

## Documentation
[List any documentation changes or state if none]

## Checklist
- [ ] All tests pass
- [ ] Code follows project conventions
- [ ] Self-review completed
- [ ] No unnecessary dependencies added
```

Example `gh pr create` command:
```bash
gh pr create \
  --title "feat: {{ feature_name }}" \
  --body "$(cat <<'EOF'
## Summary
[Your summary here]

## Changes
- [Change 1]
- [Change 2]

## Test Plan
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Breaking Changes & Migration Notes
None

## Documentation
Updated README.md with new feature documentation

## Checklist
- [ ] All tests pass
- [ ] Code follows project conventions
- [ ] Self-review completed
- [ ] No unnecessary dependencies added
EOF
)"
```

3. **Save PR Link**: After the PR is created, save the PR URL in the state file:
   ```yaml
   result:
     pr_link: "https://github.com/user/repo/pull/123"
   ```
```

#### verify.jinja2 - Verification and Testing

```jinja2
---
systemPrompt: "You are an expert quality assurance engineer verifying the implementation of a feature."
usePreset: true
tools: []
---

You are verifying the implementation of the feature: {{ feature_name }}

## Feature Details

Feature ID: {{ feature_id }}
Description: {{ feature_description }}

## Implementation Summary

{{ implementation_summary }}

## Verification Tasks

Please verify the implementation by:

1. **Code Review Results**: Review the code review feedback and ensure all critical and important issues have been addressed
2. **Test Coverage**: Verify that tests are comprehensive and cover:
   - Unit tests for individual functions
   - Integration tests for workflows
   - Edge cases and error scenarios
3. **Documentation**: Verify that:
   - Public APIs are documented
   - README/CHANGELOG are updated if needed
   - New features are explained in documentation
4. **Functionality**: Verify that:
   - The feature works as described
   - No regressions were introduced
   - Error handling is appropriate
5. **Performance**: Check if there are any performance concerns

## Instructions

1. Run all tests and report results
2. Check documentation completeness
3. Verify the feature works end-to-end
4. Report any issues found
5. Mark the feature as verified if all checks pass, or list remaining issues
```

#### review.jinja2 - Code Review

```jinja2
---
systemPrompt: "You are an expert code reviewer conducting a thorough review of the implementation."
usePreset: true
tools: []
---

You are conducting a code review for the feature: {{ feature_name }}

## Feature Details

Feature ID: {{ feature_id }}
Description: {{ feature_description }}

## Changes to Review

{{ diff_content }}

## Review Criteria

Please conduct a thorough code review focusing on:

1. **Correctness**: Are there bugs or logic errors?
2. **Performance**: Are there performance concerns or optimizations?
3. **Security**: Are there security vulnerabilities?
4. **Code Style**: Does the code follow Rust best practices and conventions?
5. **Documentation**: Is the code properly documented?
6. **Testing**: Are there adequate tests? Are edge cases covered?
7. **Error Handling**: Are errors handled properly?
8. **API Design**: Is the API clean and intuitive?

## Instructions

1. Review all changes in the diff
2. Provide specific feedback with line references where applicable
3. Categorize issues as: critical, important, or minor
4. Suggest concrete fixes for each issue
5. If critical issues are found, they must be addressed before the PR can be merged
```

#### resume.jinja2 - Resume Interrupted Task

```jinja2
---
systemPrompt: "You are continuing work on an interrupted task. Analyze the current state and continue from where you left off."
usePreset: true
tools: []
---

You are resuming work on the {{ task_kind }} of feature: {{ feature_name }}

## Task Context

Feature ID: {{ feature_id }}
Description: {{ feature_description }}

## Current Progress

Current Phase: {{ current_phase }}
Current Step: {{ current_step }}

## Execution Statistics

Turns completed: {{ turns_so_far }}
Cost incurred so far: ${{ cost_so_far }}

## Implementation Plan (Reference)

{{ implementation_plan }}

## Instructions

You are continuing work that was previously interrupted. Please:

1. **Assess Current State**: Check what has been completed by:
   - Reviewing the state file
   - Checking git status and recent commits
   - Examining any existing work in the worktree

2. **Continue Implementation**:
   - Resume from where the work was interrupted
   - Do NOT redo work that was already completed
   - Continue with the next logical step
   - Update the state file as progress is made

3. **Update State**: After each significant milestone, update the state file with:
   - Current phase and step
   - Turns completed
   - Cost incurred
   - Any completed work

## Important Notes

- The previous execution may have been interrupted mid-task
- Use git history to determine what was committed
- Uncommitted changes may indicate in-progress work
- Be conservative - if unsure whether something was completed, verify it before assuming
```

### Template Configuration Structure

The `PromptTemplate` struct represents a loaded template with its configuration:

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptTemplate {
    /// Template configuration from front matter
    pub config: TemplateConfig,
    /// The actual template source
    pub template: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateConfig {
    /// System prompt text (or empty if using preset)
    pub system_prompt: String,
    /// Whether to use Claude Code preset
    pub use_preset: bool,
    /// Tools to enable (empty = all tools)
    pub tools: Vec<String>,
    /// Maximum number of turns allowed
    #[serde(default = "default_max_turns")]
    pub max_turns: u32,
}

fn default_max_turns() -> u32 {
    100
}
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

## Configuration Format

### GBA Project Configuration (`.gba/config.yml`)

The `.gba/config.yml` file is a **per-project configuration** that stores project-specific settings for GBA. This configuration is:

- **Created automatically** when `gba init` is run
- **Stored in the `.gba/` directory** alongside templates and feature state
- **Versioned in git** so it can be shared across the team
- **Required for all GBA operations** to provide project context

**Why is this needed?**

1. **Consistency**: Ensures all team members use the same GBA settings for the project
2. **Convenience**: Avoids repeating configuration values on every command
3. **Project-specific customization**: Different projects may need different models, prompts, or settings
4. **Integration**: Links GBA to the repository structure and conventions

```yaml
# GBA Project Configuration
version: "1.0"

# Project metadata
project:
  name: "my-project"
  repository:
    url: "https://github.com/user/repo.git"
    mainBranch: "main"

# Agent defaults - used unless overridden by command-line flags
agent:
  model: "claude-sonnet-4-20250514"
  maxTokens: 4096
  temperature: 0.7
  timeout: 300

# Prompt templates configuration
prompts:
  directory: "./.gba/templates"
  useBundled: true  # Fall back to bundled templates if not found locally

# Repository scanning settings
repository:
  # Patterns to exclude when scanning files
  excludePatterns:
    - "target/"
    - ".git/"
    - "node_modules/"
  # Maximum file size to include in context (bytes)
  maxFileSize: 1048576  # 1MB

# Logging configuration
logging:
  level: "info"  # debug, info, warn, error
  format: "human"  # "human" or "json" for production

# Worktree configuration
worktree:
  # Base directory for git worktrees
  directory: "./.trees"
  # Branch prefix for feature worktrees
  branchPrefix: "gba/"

# Execution limits
limits:
  # Maximum number of agent turns per task
  maxTurns: 100
  # Maximum total cost per task in USD
  maxCostUsd: 10.0
```

### Feature State File (`.gba/features/{feature}/state.yml`)

The `state.yml` file tracks the execution state of each feature/task. This file:

- **Created when** `gba run` is initiated for a feature
- **Updated continuously** during task execution to track progress
- **Stored in gitignored location** (`.gba/features/*/state.yml`) to avoid committing transient state
- **Used for resumption** when tasks are interrupted and later resumed

**Why is this needed?**

1. **Progress Tracking**: Records what phases and steps have been completed
2. **Resumption Support**: Allows interrupted tasks to be resumed from the last known state
3. **Cost Monitoring**: Tracks turns and API costs for budget management
4. **Result Persistence**: Stores final results like PR links for reference
5. **Debugging**: Provides a record of execution history for troubleshooting

```yaml
# Feature State File
# This file tracks the execution state of a feature task
# DO NOT commit to git - listed in .gitignore

# Feature identification
feature:
  name: "add-pr-in-status-yml"
  id: "0003"
  description: "Add PR link to status.yml"

# Task configuration
task:
  kind: "implementation"  # planning, implementation, verification
  description: "Implement PR link in status.yml"
  template: "implement"  # The template used for this task

# Current status
status:
  state: "in_progress"  # pending, in_progress, completed, failed
  current_phase: "phase_3"
  current_step: "step_2"
  message: "Currently working on PR link implementation"

# Execution statistics
execution:
  turns: 15  # Number of agent interactions
  # Cost breakdown
  cost:
    input_tokens: 125000
    output_tokens: 45000
    total_cost_usd: 1.70

# Final results (populated when completed)
result:
  pr_link: "https://github.com/user/repo/pull/123"
  summary: "Successfully added PR link to status.yml"
  files_changed: 5
  commits_created: 3

# Context for resumption
context:
  # Worktree information for resume
  worktree:
    path: "./.trees/0003-add-pr-in-status-yml"
    branch: "gba/0003-add-pr-in-status-yml"

  # Last checkpoint - useful for resume
  last_checkpoint:
    timestamp: "2026-02-24T11:45:00Z"
    description: "Created PR with gh pr create"

  # Any persistent context for the agent
  agent_context:
    implementation_plan: |
      The plan involves:
      1. Update status.yml structure
      2. Modify state serialization code
      3. Add tests
      4. Update documentation

# Timestamps
timestamps:
  created_at: "2026-02-24T10:30:00Z"
  updated_at: "2026-02-24T11:45:00Z"
  completed_at: null  # Populated when state is completed
```

### Task Kinds

GBA supports the following task kinds:

1. **`planning`**: Create a detailed plan for implementing a feature
   - Break down work into phases
   - Identify dependencies and risks
   - Specify testing requirements
   - Used as input for implementation phase

2. **`implementation`**: Execute the plan and implement the feature
   - Follow the implementation plan phase by phase
   - Write code, tests, and documentation
   - Create git commits for meaningful changes
   - Create a pull request upon completion

3. **`verification`**: Verify the implementation (code review, testing)
   - Review code for correctness and style
   - Verify test coverage and quality
   - Check documentation completeness
   - Ensure no regressions were introduced

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
