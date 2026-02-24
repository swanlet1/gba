// Integration tests for GBA Prompt Manager
//
// These tests verify the integration between different components.

use gba_pm::{Context, FileContext, PromptManager, PromptTemplate, TemplateConfig};

#[test]
fn test_should_integration_prompt_manager_with_complex_template() {
    let mut prompt_manager = PromptManager::new().expect("Failed to create prompt manager");

    let template_content = r#"---
systemPrompt: "You are an expert software architect"
usePreset: false
tools:
  - Read
  - Write
maxTurns: 50
---
You are working on feature: {{ feature_name }}

Description: {{ feature_description }}

## Context

{% for file in files %}
- {{ file.path }} ({{ file.language }})
{% endfor %}

## Instructions

{{ user_message }}
"#;

    prompt_manager
        .register("complex_task", template_content)
        .expect("Failed to register template");

    let mut context = Context::new("/repo", "main", "Implement feature X");
    context.add_extra("feature_name", serde_json::json!("add-auth"));
    context.add_extra(
        "feature_description",
        serde_json::json!("Add authentication system"),
    );
    context.add_extra("user_message", serde_json::json!("Please implement it"));

    context.add_file(FileContext::new("src/main.rs", "fn main() {}", "rust"));
    context.add_file(FileContext::new(
        "Cargo.toml",
        "[package]\nname = \"my-app\"",
        "toml",
    ));

    let prompt = prompt_manager
        .get_prompt("complex_task", &context)
        .expect("Failed to get prompt");

    assert!(prompt.contains("add-auth"));
    assert!(prompt.contains("Add authentication system"));
    assert!(prompt.contains("src/main.rs"));
    assert!(prompt.contains("rust"));
}

#[test]
fn test_should_integration_prompt_template_parse_and_serialize() {
    let source = r#"---
systemPrompt: "Custom prompt"
usePreset: true
tools:
  - Read
  - Write
  - Bash
maxTurns: 75
---
Template content here"#;

    let prompt_template = PromptTemplate::parse(source).expect("Failed to parse template");

    assert_eq!(prompt_template.config.system_prompt, "Custom prompt");
    assert!(prompt_template.config.use_preset);
    assert_eq!(prompt_template.config.tools, vec!["Read", "Write", "Bash"]);
    assert_eq!(prompt_template.config.max_turns, 75);
    assert!(prompt_template.template.contains("Template content here"));

    // Test serialization
    let json = serde_json::to_string(&prompt_template).expect("Failed to serialize");
    let deserialized: PromptTemplate = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(
        prompt_template.config.system_prompt,
        deserialized.config.system_prompt
    );
    assert_eq!(
        prompt_template.config.use_preset,
        deserialized.config.use_preset
    );
    assert_eq!(prompt_template.config.tools, deserialized.config.tools);
}

#[test]
fn test_should_integration_context_creation_methods() {
    // Test different context creation methods
    let planning_context = Context::for_planning(
        "/repo",
        "main",
        "add-auth",
        "0001",
        "Add authentication system",
    );
    assert_eq!(planning_context.feature_name, "add-auth");
    assert_eq!(planning_context.feature_id, "0001");
    assert_eq!(planning_context.main_branch, "main");

    let implementation_context = Context::for_implementation(
        "/repo",
        "add-auth",
        "0001",
        "Add authentication",
        "/trees/0001",
        "gba/0001-add-auth",
        "The plan...",
    );
    assert_eq!(implementation_context.worktree_path, "/trees/0001");
    assert_eq!(implementation_context.worktree_branch, "gba/0001-add-auth");
    assert!(implementation_context.use_preset);

    let verification_context = Context::for_verification(
        "add-auth",
        "0001",
        "Add authentication",
        "Implementation summary...",
    );
    assert_eq!(verification_context.task_kind, "verification");
    assert!(verification_context.tools.len() > 0);

    let review_context =
        Context::for_review("add-auth", "0001", "Add authentication", "diff content...");
    assert_eq!(review_context.task_kind, "review");
    assert_eq!(review_context.diff_content, "diff content...");

    let resume_context = Context::for_resume(
        "add-auth",
        "0001",
        "Add authentication",
        "implementation",
        "phase_2",
        "step_1",
        10,
        0.75,
        "/trees/0001",
        "gba/0001-add-auth",
        "The plan...",
        true,
        vec!["Write".to_string()],
    );
    assert_eq!(resume_context.current_phase, "phase_2");
    assert_eq!(resume_context.turns_so_far, 10);
    assert_eq!(resume_context.cost_so_far, 0.75);
}

#[test]
fn test_should_integration_context_with_extra_variables() {
    let mut context = Context::new("/repo", "main", "Test message");

    // Add various types of extra variables
    context.add_extra("string_value", serde_json::json!("hello"));
    context.add_extra("number_value", serde_json::json!(42));
    context.add_extra("bool_value", serde_json::json!(true));
    context.add_extra("array_value", serde_json::json!(["a", "b", "c"]));
    context.add_extra(
        "object_value",
        serde_json::json!({"key1": "value1", "key2": "value2"}),
    );

    if let serde_json::Value::Object(map) = &context.extra {
        assert_eq!(map.len(), 5);
        assert!(map.contains_key("string_value"));
        assert!(map.contains_key("number_value"));
        assert!(map.contains_key("bool_value"));
        assert!(map.contains_key("array_value"));
        assert!(map.contains_key("object_value"));
    } else {
        panic!("Expected extra to be an object");
    }
}

#[test]
fn test_should_integration_template_config_serialization() {
    let config = TemplateConfig {
        system_prompt: "You are helpful".to_string(),
        use_preset: false,
        tools: vec!["Read".to_string(), "Write".to_string()],
        max_turns: 150,
    };

    let yaml = serde_yaml::to_string(&config).expect("Failed to serialize");
    let deserialized: TemplateConfig = serde_yaml::from_str(&yaml).expect("Failed to deserialize");

    assert_eq!(config.system_prompt, deserialized.system_prompt);
    assert_eq!(config.use_preset, deserialized.use_preset);
    assert_eq!(config.tools, deserialized.tools);
    assert_eq!(config.max_turns, deserialized.max_turns);
}

#[test]
fn test_should_integration_file_context_serialization() {
    let file = FileContext::new(
        "src/lib.rs",
        "pub fn hello() { println!(\"Hello\"); }",
        "rust",
    );

    let json = serde_json::to_string(&file).expect("Failed to serialize");
    let deserialized: FileContext = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(file.path, deserialized.path);
    assert_eq!(file.content, deserialized.content);
    assert_eq!(file.language, deserialized.language);
}

#[test]
fn test_should_integration_prompt_manager_list_and_has() {
    let mut prompt_manager = PromptManager::new().expect("Failed to create prompt manager");

    prompt_manager
        .register("first", "---\n---\nFirst template")
        .expect("Failed to register");
    prompt_manager
        .register("second", "---\n---\nSecond template")
        .expect("Failed to register");
    prompt_manager
        .register("third", "---\n---\nThird template")
        .expect("Failed to register");

    // Test list_prompts
    let prompts = prompt_manager.list_prompts();
    assert_eq!(prompts.len(), 3);
    assert!(prompts.contains(&"first".to_string()));
    assert!(prompts.contains(&"second".to_string()));
    assert!(prompts.contains(&"third".to_string()));

    // Test has_prompt
    assert!(prompt_manager.has_prompt("first"));
    assert!(prompt_manager.has_prompt("second"));
    assert!(prompt_manager.has_prompt("third"));
    assert!(!prompt_manager.has_prompt("nonexistent"));
}

#[test]
fn test_should_integration_template_registry() {
    use gba_pm::prompt::TemplateRegistry;

    let mut registry = TemplateRegistry::new();

    let config1 = TemplateConfig {
        system_prompt: "Prompt 1".to_string(),
        use_preset: true,
        tools: vec![],
        max_turns: 100,
    };

    let template1 = PromptTemplate {
        config: config1.clone(),
        template: "Template 1 content".to_string(),
    };

    let config2 = TemplateConfig {
        system_prompt: "Prompt 2".to_string(),
        use_preset: false,
        tools: vec!["Read".to_string()],
        max_turns: 50,
    };

    let template2 = PromptTemplate {
        config: config2.clone(),
        template: "Template 2 content".to_string(),
    };

    registry.register("template1", template1);
    registry.register("template2", template2);

    // Test contains
    assert!(registry.contains("template1"));
    assert!(registry.contains("template2"));
    assert!(!registry.contains("template3"));

    // Test get
    let retrieved = registry.get("template1").expect("Failed to get template");
    assert_eq!(retrieved.config.system_prompt, "Prompt 1");

    // Test list
    let list = registry.list();
    assert_eq!(list.len(), 2);
    assert!(list.contains(&"template1"));
    assert!(list.contains(&"template2"));
}
