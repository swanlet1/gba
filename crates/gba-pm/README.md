# GBA Prompt Manager - Template-based Prompt Management

This crate provides functionality for managing and rendering prompts using the Minijinja templating engine.

## Features

- Template-based prompt rendering with Jinja2 syntax
- Front matter support for template configuration
- Bundled templates for common use cases
- Custom template loading from directories
- Context variable injection
- Template validation

## Usage

### Basic Usage

```rust
use gba_pm::{PromptManager, Context, FileContext};

let mut prompt_manager = PromptManager::new()?;

// Register a template
prompt_manager.register("greeting", r#"
---
usePreset: true
tools: []
---
Hello, {{ name }}! You are working on: {{ feature_name }}
"#)?;

// Create context
let mut context = Context::new("/path/to/repo", "main", "Help me");
context.add_extra("name", serde_json::json!("Alice"));
context.add_extra("feature_name", serde_json::json!("add-auth"));

// Render the prompt
let prompt = prompt_manager.get_prompt("greeting", &context)?;
println!("{}", prompt);
```

### Using Bundled Templates

```rust
use gba_pm::PromptManager;

let prompt_manager = PromptManager::new()?;
prompt_manager.load_all_bundled_templates()?;

let prompts = prompt_manager.list_prompts();
println!("Available prompts: {:?}", prompts);
```

### Local Templates with Fallback

```rust
use gba_pm::PromptManager;
use std::path::PathBuf;

let local_dir = PathBuf::from("./.gba/templates");
let prompt_manager = PromptManager::with_local_dir(local_dir, true)?;

// This will first look for templates in local_dir,
// then fall back to bundled templates
```

### Template Configuration

Templates use YAML front matter for configuration:

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

### Context Building Helpers

```rust
use gba_pm::Context;

// For planning
let context = Context::for_planning(
    "/path/to/repo",
    "main",
    "add-auth",
    "0001",
    "Add authentication system"
);

// For implementation
let context = Context::for_implementation(
    "/path/to/repo",
    "add-auth",
    "0001",
    "Add authentication",
    "/trees/0001",
    "gba/0001-add-auth",
    "The implementation plan..."
);

// For verification
let context = Context::for_verification(
    "add-auth",
    "0001",
    "Add authentication",
    "Implementation summary..."
);
```

### Template Engine Direct Usage

```rust
use gba_pm::TemplateEngine;

let mut engine = TemplateEngine::new()?;
engine.add_template("hello", "Hello, {{ name }}!")?;

let mut context = std::collections::HashMap::new();
context.insert("name", "World");

let result = engine.render("hello", minijinja::value::Value::from_serialize(&context))?;
println!("{}", result); // "Hello, World!"
```

## Available Bundled Templates

| Template | Purpose | Use Case |
|----------|---------|----------|
| `init` | Initialize GBA project | Project initialization |
| `plan` | Create implementation plan | Planning phase |
| `implement` | Execute implementation | Implementation phase |
| `verify` | Verify implementation | Verification phase |
| `review` | Code review | Manual code review |
| `resume` | Resume interrupted task | Task resumption |

## Error Handling

All operations return `Result<T, PromptError>` where `PromptError` can be:

- `Template(String)` - Template rendering errors
- `NotFound(String)` - Template not found
- `InvalidSyntax(String)` - Invalid template syntax
- `InvalidVariable(String)` - Invalid context variable
- `MissingVariable(String)` - Required context variable missing
- `Io(std::io::Error)` - I/O errors
- `Serde(serde_json::Error)` - Serialization errors
- `Yaml(serde_yaml::Error)` - YAML parsing errors

## License

MIT License - see the main project LICENSE.md for details.
