//! Template engine implementation using Minijinja.

use crate::error::{PromptError, Result};
use minijinja::{Environment, value::Value};
use std::path::Path;
use tracing::instrument;

/// Template engine for rendering prompts.
#[derive(Debug)]
pub struct TemplateEngine {
    /// Minijinja environment.
    env: Environment<'static>,
}

impl TemplateEngine {
    /// Create a new template engine.
    #[instrument]
    pub fn new() -> Result<Self> {
        let mut env = Environment::new();
        // Set up default configuration
        env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);
        Ok(Self { env })
    }

    /// Create a new template engine with the given path loader.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory containing templates.
    ///
    /// # Errors
    ///
    /// Returns an error if the path cannot be accessed.
    #[instrument(skip_all)]
    pub fn with_loader(path: &Path) -> Result<Self> {
        let mut env = Environment::new();
        env.set_loader(minijinja::path_loader(path));
        env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);
        Ok(Self { env })
    }

    /// Render a template with the given context.
    ///
    /// # Arguments
    ///
    /// * `template_name` - Name of the template to render.
    /// * `context` - Context variables for rendering.
    ///
    /// # Errors
    ///
    /// Returns an error if the template is not found or rendering fails.
    #[instrument(skip(context))]
    pub fn render(&self, template_name: &str, context: Value) -> Result<String> {
        self.env
            .get_template(template_name)
            .map_err(|e| PromptError::NotFound(format!("{template_name}: {e}")))?
            .render(context)
            .map_err(|e| PromptError::Template(format!("Render error for '{template_name}': {e}")))
    }

    /// Get a reference to the underlying environment for reading templates.
    #[must_use]
    pub fn env(&self) -> &Environment<'static> {
        &self.env
    }

    /// Add a template to the environment from a string.
    ///
    /// # Arguments
    ///
    /// * `name` - Template name.
    /// * `content` - Template source content.
    ///
    /// # Errors
    ///
    /// Returns an error if the template cannot be added.
    #[instrument(skip_all)]
    pub fn add_template(
        &mut self,
        name: impl Into<String>,
        content: impl Into<String>,
    ) -> Result<()> {
        let name = name.into();
        let content = content.into();
        self.env
            .add_template_owned(name, content)
            .map_err(|e| PromptError::Template(format!("Failed to add template: {e}")))?;
        Ok(())
    }

    /// Load templates from a directory.
    ///
    /// This scans the directory and loads all `.jinja2` files.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory containing templates.
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be accessed or templates cannot be loaded.
    #[instrument(skip_all)]
    pub fn load_templates_from_dir(&mut self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let entries = std::fs::read_dir(path).map_err(PromptError::Io)?;

        for entry in entries {
            let entry = entry.map_err(PromptError::Io)?;
            let file_path = entry.path();

            if file_path.is_file()
                && let Some(extension) = file_path.extension()
                && extension == "jinja2"
                && let Some(name) = file_path.file_stem()
            {
                let name = name.to_string_lossy().to_string();
                let content = std::fs::read_to_string(&file_path).map_err(PromptError::Io)?;
                self.add_template(&name, content)?;
            }
        }

        Ok(())
    }

    /// Load a bundled template by name.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the bundled template (without extension).
    ///
    /// # Errors
    ///
    /// Returns an error if the template is not found or cannot be loaded.
    #[instrument]
    pub fn load_bundled_template(&mut self, name: &str) -> Result<()> {
        let template_name = format!("{name}.jinja2");
        let content = get_bundled_template(&template_name).ok_or_else(|| {
            PromptError::NotFound(format!("Bundled template '{template_name}' not found"))
        })?;
        self.add_template(name, content)
    }

    /// Load all bundled templates.
    ///
    /// # Errors
    ///
    /// Returns an error if any bundled template cannot be loaded.
    #[instrument]
    pub fn load_all_bundled_templates(&mut self) -> Result<()> {
        const TEMPLATES: &[&str] = &["init", "plan", "implement", "verify", "review", "resume"];

        for name in TEMPLATES {
            self.load_bundled_template(name)?;
        }

        Ok(())
    }
}

impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default template engine")
    }
}

/// Get a bundled template by name.
///
/// Returns `None` if the template does not exist.
fn get_bundled_template(name: &str) -> Option<String> {
    match name {
        "init.jinja2" => Some(include_str!("../templates/init.jinja2").to_string()),
        "plan.jinja2" => Some(include_str!("../templates/plan.jinja2").to_string()),
        "implement.jinja2" => Some(include_str!("../templates/implement.jinja2").to_string()),
        "verify.jinja2" => Some(include_str!("../templates/verify.jinja2").to_string()),
        "review.jinja2" => Some(include_str!("../templates/review.jinja2").to_string()),
        "resume.jinja2" => Some(include_str!("../templates/resume.jinja2").to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_template_engine_new() {
        let engine = TemplateEngine::new().unwrap();
        assert!(matches!(engine, TemplateEngine { .. }));
    }

    #[test]
    fn test_add_template() {
        let mut engine = TemplateEngine::new().unwrap();
        engine.add_template("test", "Hello, {{ name }}!").unwrap();

        let mut context = HashMap::new();
        context.insert("name", "World");

        let result = engine
            .render("test", Value::from_serialize(&context))
            .unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_render_with_context() {
        let mut engine = TemplateEngine::new().unwrap();
        engine
            .add_template("greeting", "Hello, {{ name }}! Your age is {{ age }}.")
            .unwrap();

        let mut context: HashMap<String, serde_json::Value> = HashMap::new();
        context.insert("name".to_string(), serde_json::json!("Alice"));
        context.insert("age".to_string(), serde_json::json!(30));

        let result = engine
            .render("greeting", Value::from_serialize(&context))
            .unwrap();
        assert_eq!(result, "Hello, Alice! Your age is 30.");
    }

    #[test]
    fn test_render_not_found() {
        let engine = TemplateEngine::new().unwrap();
        let result = engine.render(
            "nonexistent",
            Value::from_serialize(HashMap::<String, String>::new()),
        );
        assert!(result.is_err());
        assert!(matches!(result, Err(PromptError::NotFound(_))));
    }

    #[test]
    fn test_load_templates_from_nonexistent_dir() {
        let mut engine = TemplateEngine::new().unwrap();
        let result = engine.load_templates_from_dir(Path::new("/nonexistent/path"));
        assert!(result.is_ok());
    }
}
