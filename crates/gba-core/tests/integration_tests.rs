// Integration tests for GBA Core
//
// These tests verify the integration between different components.

use gba_core::config::{AgentConfig, ProjectConfig};
use gba_core::context_builder::ContextBuilderConfig;
use gba_core::task::{Context, File, Task};
use std::path::PathBuf;

#[test]
fn test_should_integration_task_creation_with_defaults() {
    let context = Context {
        repository_path: PathBuf::from("/test/repo"),
        branch: "main".to_string(),
        files: vec![],
        metadata: Default::default(),
    };

    let task = Task::with_defaults("Test task", context);

    assert_eq!(task.prompt, "Test task");
    assert_eq!(
        task.system_prompt,
        "You are an expert software development assistant."
    );
    assert_eq!(task.max_turns, 100);
}

#[test]
fn test_should_integration_file_serialization_round_trip() {
    let file = File {
        path: PathBuf::from("src/main.rs"),
        content: "fn main() {}".to_string(),
        language: "rust".to_string(),
    };

    let json = serde_json::to_string(&file).expect("Failed to serialize");
    let deserialized: File = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(file.path, deserialized.path);
    assert_eq!(file.content, deserialized.content);
    assert_eq!(file.language, deserialized.language);
}

#[test]
fn test_should_integration_project_config_serialization() {
    let config = ProjectConfig::default_config();

    let yaml = serde_yaml::to_string(&config).expect("Failed to serialize");
    let deserialized: ProjectConfig = serde_yaml::from_str(&yaml).expect("Failed to deserialize");

    assert_eq!(config.version, deserialized.version);
    assert_eq!(config.agent.model, deserialized.agent.model);
    assert_eq!(config.agent.max_tokens, deserialized.agent.max_tokens);
}

#[test]
fn test_should_integration_agent_config_validation() {
    let config = AgentConfig::default();

    // Check that temperature is within valid range
    assert!(config.temperature >= 0.0);
    assert!(config.temperature <= 2.0);

    // Check that max_tokens is positive
    assert!(config.max_tokens > 0);

    // Check that timeout is positive
    assert!(config.timeout > 0);
}

#[test]
fn test_should_integration_context_builder_config_defaults() {
    let config = ContextBuilderConfig::default();

    assert!(!config.exclude_patterns.is_empty());
    assert!(config.exclude_patterns.contains(&"target/".to_string()));
    assert!(config.max_files > 0);
    assert!(config.max_file_size > 0);
}

#[test]
fn test_should_integration_task_usage_serialization() {
    let usage = gba_core::task::Usage {
        input_tokens: 1000,
        output_tokens: 500,
        total_cost_usd: 0.05,
    };

    let json = serde_json::to_string(&usage).expect("Failed to serialize");
    let deserialized: gba_core::task::Usage =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(usage.input_tokens, deserialized.input_tokens);
    assert_eq!(usage.output_tokens, deserialized.output_tokens);
    assert_eq!(usage.total_cost_usd, deserialized.total_cost_usd);
}

#[test]
fn test_should_integration_response_with_usage() {
    let response = gba_core::task::Response {
        content: "Test response".to_string(),
        tool_calls: vec![],
        usage: gba_core::task::Usage {
            input_tokens: 100,
            output_tokens: 50,
            total_cost_usd: 0.01,
        },
    };

    assert_eq!(response.content, "Test response");
    assert!(response.tool_calls.is_empty());
    assert_eq!(response.usage.input_tokens, 100);
    assert_eq!(response.usage.output_tokens, 50);
}

#[tokio::test]
async fn test_should_integration_build_minimal_context() {
    let context =
        gba_core::context_builder::build_minimal_context(PathBuf::from("/test/repo"), "main")
            .await
            .expect("Failed to build minimal context");

    assert_eq!(context.repository_path, PathBuf::from("/test/repo"));
    assert_eq!(context.branch, "main");
    assert!(context.files.is_empty());
    assert!(context.metadata.is_empty());
}

#[test]
fn test_should_integration_file_context_merge_metadata() {
    let mut context = Context::default();

    let file = File {
        path: PathBuf::from("src/main.rs"),
        content: "fn main() {}".to_string(),
        language: "rust".to_string(),
    };

    context.files.push(file);

    // Add some metadata
    context
        .metadata
        .insert("project".to_string(), serde_json::json!("test-project"));

    assert_eq!(context.files.len(), 1);
    assert_eq!(context.files[0].language, "rust");
    assert_eq!(
        context.metadata.get("project"),
        Some(&serde_json::json!("test-project"))
    );
}

#[test]
fn test_should_integration_context_builder_config_chainable() {
    let config = ContextBuilderConfig::new()
        .with_max_files(25)
        .with_max_file_size(512_000)
        .with_exclude_patterns(vec!["test/".to_string(), "build/".to_string()])
        .with_include_extensions(vec!["rs".to_string(), "toml".to_string()]);

    assert_eq!(config.max_files, 25);
    assert_eq!(config.max_file_size, 512_000);
    assert_eq!(config.exclude_patterns.len(), 2);
    assert_eq!(config.include_extensions.len(), 2);
}
