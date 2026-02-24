//! Configuration management for GBA CLI.
//!
//! This module handles loading and managing GBA project configuration.

use gba_core::config::ProjectConfig;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, instrument};

/// Result type alias for configuration operations.
pub type Result<T> = std::result::Result<T, ConfigLoadError>;

/// Error types for configuration loading.
#[derive(Debug, Error)]
pub enum ConfigLoadError {
    /// Configuration file not found.
    #[error("Configuration file not found: {0}")]
    NotFound(PathBuf),

    /// Error loading configuration.
    #[error("Failed to load configuration: {0}")]
    LoadError(#[from] gba_core::config::ConfigError),

    /// Invalid project path.
    #[error("Invalid project path: {0}")]
    InvalidPath(PathBuf),

    /// Not a GBA project (no .gba directory).
    #[error("Not a GBA project: {0} (missing .gba directory)")]
    NotGbaProject(PathBuf),
}

/// Configuration manager for GBA CLI.
#[derive(Debug)]
pub struct ConfigManager {
    /// Project path.
    project_path: PathBuf,
    /// Loaded configuration.
    config: ProjectConfig,
}

impl ConfigManager {
    /// Get the default configuration file path for a project.
    ///
    /// # Arguments
    ///
    /// * `project_path` - Path to the project directory.
    #[must_use]
    pub fn config_file_path(project_path: &Path) -> PathBuf {
        project_path.join(".gba").join("config.yml")
    }

    /// Check if a directory is a valid GBA project.
    ///
    /// # Arguments
    ///
    /// * `project_path` - Path to check.
    ///
    /// # Returns
    ///
    /// `true` if the directory contains a .gba directory, `false` otherwise.
    #[must_use]
    pub fn is_gba_project(project_path: &Path) -> bool {
        project_path.join(".gba").is_dir()
    }

    /// Load configuration from a project directory.
    ///
    /// # Arguments
    ///
    /// * `project_path` - Path to the project directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be loaded.
    #[instrument(skip(project_path))]
    pub fn load(project_path: &Path) -> Result<Self> {
        if !project_path.exists() {
            return Err(ConfigLoadError::InvalidPath(project_path.to_path_buf()));
        }

        if !Self::is_gba_project(project_path) {
            return Err(ConfigLoadError::NotGbaProject(project_path.to_path_buf()));
        }

        let config_path = Self::config_file_path(project_path);
        if !config_path.exists() {
            return Err(ConfigLoadError::NotFound(config_path));
        }

        debug!("Loading configuration from {}", config_path.display());
        let config = ProjectConfig::load_from_file(&config_path)?;

        Ok(Self {
            project_path: project_path.to_path_buf(),
            config,
        })
    }

    /// Get the project path.
    #[must_use]
    pub fn project_path(&self) -> &Path {
        &self.project_path
    }

    /// Get the configuration.
    #[must_use]
    pub const fn config(&self) -> &ProjectConfig {
        &self.config
    }

    /// Get the templates directory path.
    #[must_use]
    pub fn templates_dir(&self) -> PathBuf {
        self.project_path.join(&self.config.prompts.directory)
    }

    /// Get the features directory path.
    #[must_use]
    pub fn features_dir(&self) -> PathBuf {
        self.project_path.join(".gba").join("features")
    }

    /// Get the worktree directory path.
    #[must_use]
    #[allow(dead_code)]
    pub fn worktree_dir(&self) -> PathBuf {
        self.project_path.join(&self.config.worktree.directory)
    }

    /// Get the state file path for a feature.
    ///
    /// # Arguments
    ///
    /// * `feature_id` - The feature identifier.
    #[must_use]
    pub fn feature_state_path(&self, feature_id: &str) -> PathBuf {
        self.features_dir().join(feature_id).join("state.yml")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_file_path() {
        let path = Path::new("/test/project");
        let config_path = ConfigManager::config_file_path(path);
        assert_eq!(config_path, PathBuf::from("/test/project/.gba/config.yml"));
    }

    #[test]
    fn test_is_gba_project_false() {
        let temp_dir = std::env::temp_dir().join("gba-test-no-gba");
        std::fs::create_dir_all(&temp_dir).unwrap();
        assert!(!ConfigManager::is_gba_project(&temp_dir));
        std::fs::remove_dir_all(temp_dir).ok();
    }
}
