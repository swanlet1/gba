//! Prompt manager for loading and managing prompt templates.

use crate::template::TemplateEngine;
use std::sync::{Arc, Mutex};

/// Prompt manager for loading and managing prompt templates.
#[derive(Debug, Clone)]
pub struct PromptManager {
    /// Template engine.
    engine: Arc<Mutex<TemplateEngine>>,
}

impl PromptManager {
    /// Create a new prompt manager.
    pub fn new() -> Self {
        Self {
            engine: Arc::new(Mutex::new(
                TemplateEngine::new().expect("Failed to create template engine"),
            )),
        }
    }

    /// Get the template engine.
    pub fn engine(&self) -> Arc<Mutex<TemplateEngine>> {
        Arc::clone(&self.engine)
    }

    /// Load a prompt template from a file.
    pub fn load_template(&self, name: &str, content: &str) -> crate::Result<()> {
        self.engine
            .lock()
            .map_err(|e| crate::PromptError::Template(format!("Lock error: {e}")))?
            .add_template(name.to_string(), content.to_string())?;
        Ok(())
    }
}

impl Default for PromptManager {
    fn default() -> Self {
        Self::new()
    }
}
