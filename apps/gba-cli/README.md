# GBA CLI - Command Line Interface

The GBA CLI provides a command-line interface for interacting with the GBA system. It supports both simple command-line mode and TUI (Terminal User Interface) mode.

## Installation

```bash
# Build from source
cargo build --release

# The binary will be at target/release/gba
```

## Commands

### `gba init` - Initialize a GBA Project

Initialize a new GBA project in the current directory or a specified path.

```bash
gba init
gba init --path ./my-project
gba init --main-branch master --repo-url https://github.com/user/repo.git
```

This creates:
- `.gba/` directory structure
- `.gba/config.yml` configuration file
- `.gba/templates/` directory for custom templates
- `.gba/features/` directory for state files

### `gba run` - Run an Agent Task

Execute a task on a repository.

```bash
gba run --feature <name> --kind <kind> [options]
```

**Options:**
- `-f, --feature <NAME>` - Feature name to work on
- `-k, --kind <KIND>` - Task kind (planning, implementation, verification)
- `-d, --description <TEXT>` - Feature description
- `--tui` - Use TUI mode
- `--resume` - Resume from previous state

**Examples:**

```bash
# Create a plan
gba run --feature add-auth --kind planning --description "Add authentication system"

# Implement the feature
gba run --feature add-auth --kind implementation --tui

# Resume an interrupted task
gba run --feature add-auth --kind implementation --resume

# Verify the implementation
gba run --feature add-auth --kind verification
```

### `gba list-prompts` - List Available Prompts

List all available prompt templates.

```bash
gba list-prompts
gba list-prompts --verbose
```

### `gba prompt` - Execute a Single Prompt

Execute a single prompt template.

```bash
gba prompt --template <name> --message <text>
```

**Examples:**

```bash
gba prompt --template hello --message "Hello, Claude!"
gba prompt -t plan -m "Create a plan for adding user profiles"
```

## Global Options

- `-p, --path <PATH>` - Path to the GBA project directory (default: current directory)
- `-v, --verbose` - Enable verbose output

## Configuration

The CLI reads configuration from `.gba/config.yml` in the project directory. See the main README for configuration options.

## TUI Mode

TUI mode provides a terminal-based interactive interface:

```bash
gba run --feature my-feature --kind implementation --tui
```

The TUI displays:
- Header with project information
- Main content area with task status
- Footer with help text and controls

**TUI Controls:**
- `q` - Quit
- `r` - Resume (if paused)

## Workflow Examples

### Complete Feature Development

```bash
# 1. Initialize project
gba init

# 2. Create an implementation plan
gba run -f add-auth -k planning -d "Add authentication system"

# 3. Implement the feature
gba run -f add-auth -k implementation --tui

# 4. Verify the implementation
gba run -f add-auth -k verification

# 5. (Optional) Manual code review
gba run -f add-auth -k review
```

### Resuming Interrupted Work

```bash
# If work was interrupted
gba run -f add-auth -k implementation --resume

# The CLI will load the previous state and continue from where it left off
```

## Error Handling

The CLI provides clear error messages with suggestions for resolution. Common errors include:

- **Not a GBA project**: Run `gba init` first
- **Template not found**: Use `gba list-prompts` to see available templates
- **Configuration errors**: Check `.gba/config.yml` for syntax issues

## Exit Codes

- `0` - Success
- `1` - Error occurred

## License

MIT License - see the main project LICENSE.md for details.
