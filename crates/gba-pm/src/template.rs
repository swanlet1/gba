//! Template engine implementation using Minijinja.

use crate::error::{PromptError, Result};
use minijinja::{Environment, value::Value};

/// Template engine for rendering prompts.
#[derive(Debug, Clone)]
pub struct TemplateEngine {
    /// Minijinja environment.
    env: Environment<'static>,
}

impl TemplateEngine {
    /// Create a new template engine.
    pub fn new() -> Result<Self> {
        Ok(Self {
            env: Environment::new(),
        })
    }

    /// Render a template with the given context.
    pub fn render(&self, template_name: &str, context: Value) -> Result<String> {
        self.env
            .get_template(template_name)
            .map_err(|e| PromptError::Template(e.to_string()))?
            .render(context)
            .map_err(|e| PromptError::Template(e.to_string()))
    }

    /// Get a reference to the underlying environment for reading templates.
    pub fn env(&self) -> &Environment<'static> {
        &self.env
    }

    /// Add a template to the environment.
    pub fn add_template(&mut self, name: String, content: String) -> Result<()> {
        self.env
            .add_template_owned(name, content)
            .map_err(|e| PromptError::Template(e.to_string()))?;
        Ok(())
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default template engine")
    }
}
