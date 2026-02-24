//! Prompt manager for loading and managing prompt templates.

use crate::config::{Context, PromptTemplate, TemplateConfig};
use crate::error::{PromptError, Result};
use crate::template::TemplateEngine;
use minijinja::value::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, instrument, warn};

/// Prompt manager for loading and managing prompt templates.
#[derive(Debug)]
pub struct PromptManager {
    /// Template engine.
    engine: TemplateEngine,
    /// Registry of loaded templates with their configurations.
    registry: HashMap<String, TemplateConfig>,
    /// Local templates directory path.
    local_templates_dir: Option<PathBuf>,
}

impl PromptManager {
    /// Create a new prompt manager.
    ///
    /// # Errors
    ///
    /// Returns an error if the template engine cannot be created.
    #[instrument]
    pub fn new() -> Result<Self> {
        let engine = TemplateEngine::new()?;
        Ok(Self {
            engine,
            registry: HashMap::new(),
            local_templates_dir: None,
        })
    }

    /// Create a new prompt manager with a local templates directory.
    ///
    /// # Arguments
    ///
    /// * `local_dir` - Path to the local templates directory.
    /// * `use_bundled` - Whether to fall back to bundled templates.
    ///
    /// # Errors
    ///
    /// Returns an error if the template engine cannot be created.
    #[instrument(skip(local_dir))]
    pub fn with_local_dir(local_dir: PathBuf, use_bundled: bool) -> Result<Self> {
        let mut engine = TemplateEngine::new()?;

        // Load local templates if directory exists
        if local_dir.exists() {
            debug!(
                "Loading templates from local directory: {}",
                local_dir.display()
            );
            engine.load_templates_from_dir(&local_dir)?;
        }

        // Load bundled templates as fallback
        if use_bundled || !local_dir.exists() {
            debug!("Loading bundled templates");
            engine.load_all_bundled_templates()?;
        }

        Ok(Self {
            engine,
            registry: HashMap::new(),
            local_templates_dir: Some(local_dir),
        })
    }

    /// Register a prompt template from a string.
    ///
    /// # Arguments
    ///
    /// * `name` - Name to register the template under.
    /// * `content` - Template content including front matter.
    ///
    /// # Errors
    ///
    /// Returns an error if the template cannot be parsed or added.
    #[instrument(skip_all)]
    pub fn register(&mut self, name: impl Into<String>, content: &str) -> Result<()> {
        let name = name.into();
        let prompt_template = PromptTemplate::parse(content)?;

        // Store the configuration in registry
        self.registry
            .insert(name.clone(), prompt_template.config.clone());

        // Add the template to the engine
        self.engine.add_template(&name, prompt_template.template)?;

        debug!("Registered prompt template: {}", name);
        Ok(())
    }

    /// Get a rendered prompt by name.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the template to render.
    /// * `context` - Context for rendering.
    ///
    /// # Errors
    ///
    /// Returns an error if the template is not found or rendering fails.
    #[instrument(skip(context))]
    pub fn get_prompt(&self, name: &str, context: &Context) -> Result<String> {
        // Convert Context to minijinja Value
        self.engine.render(name, Value::from_serialize(context))
    }

    /// Get the configuration for a registered template.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the template.
    ///
    /// # Errors
    ///
    /// Returns an error if the template is not found.
    #[instrument]
    pub fn get_config(&self, name: &str) -> Result<TemplateConfig> {
        self.registry
            .get(name)
            .cloned()
            .ok_or_else(|| PromptError::NotFound(name.to_string()))
    }

    /// List all registered prompt names.
    #[must_use]
    pub fn list_prompts(&self) -> Vec<String> {
        self.registry.keys().cloned().collect()
    }

    /// Check if a template exists.
    #[must_use]
    pub fn has_prompt(&self, name: &str) -> bool {
        self.registry.contains_key(name) || self.engine.env().get_template(name).is_ok()
    }

    /// Reload templates from the configured directories.
    ///
    /// # Errors
    ///
    /// Returns an error if templates cannot be reloaded.
    #[instrument]
    pub fn reload(&mut self) -> Result<()> {
        // Create new engine
        let mut engine = TemplateEngine::new()?;

        // Reload local templates
        if let Some(ref local_dir) = self.local_templates_dir
            && local_dir.exists()
        {
            debug!(
                "Reloading templates from local directory: {}",
                local_dir.display()
            );
            engine.load_templates_from_dir(local_dir)?;
        }

        // Always reload bundled templates as fallback
        debug!("Reloading bundled templates");
        engine.load_all_bundled_templates()?;

        self.engine = engine;

        // Re-register parsed templates
        for _config in self.registry.values() {
            // We need to re-parse to get the template content
            warn!("Re-registered templates may need to be re-added from source");
        }

        Ok(())
    }

    /// Get a reference to the template engine.
    #[must_use]
    pub const fn engine(&self) -> &TemplateEngine {
        &self.engine
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default prompt manager")
    }
}

/// Template registry for managing named templates.
#[derive(Debug)]
pub struct TemplateRegistry {
    templates: HashMap<String, PromptTemplate>,
}

impl TemplateRegistry {
    /// Create a new template registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Register a template.
    ///
    /// # Arguments
    ///
    /// * `name` - Template name.
    /// * `template` - The prompt template.
    pub fn register(&mut self, name: impl Into<String>, template: PromptTemplate) {
        self.templates.insert(name.into(), template);
    }

    /// Get a template by name.
    ///
    /// # Arguments
    ///
    /// * `name` - Template name.
    ///
    /// # Returns
    ///
    /// Returns `Some` if the template exists, `None` otherwise.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&PromptTemplate> {
        self.templates.get(name)
    }

    /// List all registered template names.
    #[must_use]
    pub fn list(&self) -> Vec<&str> {
        self.templates.keys().map(|s| s.as_str()).collect()
    }

    /// Check if a template is registered.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.templates.contains_key(name)
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_prompt_manager_new() {
        let pm = PromptManager::new().unwrap();
        assert_eq!(pm.list_prompts().len(), 0);
    }

    #[test]
    fn test_prompt_manager_register() {
        let mut pm = PromptManager::new().unwrap();
        let content = r#"---
systemPrompt: "You are helpful"
usePreset: true
tools: []
---
Hello, {{ name }}!"#;

        pm.register("test", content).unwrap();
        assert!(pm.has_prompt("test"));

        let config = pm.get_config("test").unwrap();
        assert_eq!(config.system_prompt, "You are helpful");
    }

    #[test]
    fn test_prompt_manager_get_prompt() {
        let mut pm = PromptManager::new().unwrap();
        let content = r#"---
systemPrompt: "You are helpful"
usePreset: true
tools: []
---
Hello, {{ main_branch }}!"#;

        pm.register("greeting", content).unwrap();

        let mut context = Context::new("/repo", "main", "Help");
        context.add_extra("main_branch", json!("develop"));

        let result = pm.get_prompt("greeting", &context).unwrap();
        assert_eq!(result, "Hello, develop!");
    }

    #[test]
    fn test_prompt_manager_list_prompts() {
        let mut pm = PromptManager::new().unwrap();
        pm.register("first", "---\n---\nFirst").unwrap();
        pm.register("second", "---\n---\nSecond").unwrap();

        let prompts = pm.list_prompts();
        assert_eq!(prompts.len(), 2);
        assert!(prompts.contains(&"first".to_string()));
        assert!(prompts.contains(&"second".to_string()));
    }

    #[test]
    fn test_template_registry() {
        let mut registry = TemplateRegistry::new();
        let config = TemplateConfig {
            system_prompt: "Test".to_string(),
            use_preset: true,
            tools: vec![],
            max_turns: 50,
        };
        let template = PromptTemplate {
            config: config.clone(),
            template: "Test template".to_string(),
        };

        registry.register("test", template);
        assert!(registry.contains("test"));

        let retrieved = registry.get("test").unwrap();
        assert_eq!(retrieved.config.system_prompt, "Test");
    }

    #[test]
    fn test_template_registry_list() {
        let mut registry = TemplateRegistry::new();
        registry.register(
            "first",
            PromptTemplate {
                config: TemplateConfig::default(),
                template: String::new(),
            },
        );
        registry.register(
            "second",
            PromptTemplate {
                config: TemplateConfig::default(),
                template: String::new(),
            },
        );

        let list = registry.list();
        assert_eq!(list.len(), 2);
    }
}
