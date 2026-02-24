//! Context building for repository scanning.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use tracing::{debug, info, instrument};

use crate::error::{CoreError, Result};
use crate::task::{Context, File};

/// Configuration for context building.
#[derive(Debug, Clone)]
pub struct ContextBuilderConfig {
    /// Patterns to exclude when scanning files.
    pub exclude_patterns: Vec<String>,
    /// Maximum file size to include in context (bytes).
    pub max_file_size: usize,
    /// Maximum number of files to include in context.
    pub max_files: usize,
    /// File extensions to include (empty means all).
    pub include_extensions: Vec<String>,
}

impl Default for ContextBuilderConfig {
    fn default() -> Self {
        Self {
            exclude_patterns: vec![
                "target/".to_string(),
                ".git/".to_string(),
                "node_modules/".to_string(),
                ".trees/".to_string(),
                ".claude/".to_string(),
            ],
            max_file_size: 1_048_576, // 1MB
            max_files: 100,
            include_extensions: vec![],
        }
    }
}

impl ContextBuilderConfig {
    /// Create a new context builder configuration.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            exclude_patterns: vec![],
            max_file_size: 0,
            max_files: 0,
            include_extensions: vec![],
        }
    }

    /// Set the exclude patterns.
    #[must_use]
    pub fn with_exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns = patterns;
        self
    }

    /// Set the maximum file size.
    #[must_use]
    pub const fn with_max_file_size(mut self, size: usize) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set the maximum number of files.
    #[must_use]
    pub const fn with_max_files(mut self, count: usize) -> Self {
        self.max_files = count;
        self
    }

    /// Set the include extensions.
    #[must_use]
    pub fn with_include_extensions(mut self, extensions: Vec<String>) -> Self {
        self.include_extensions = extensions;
        self
    }
}

/// Build context from a repository.
///
/// This function scans the repository and builds a context object containing
/// information about the repository, branch, and files.
///
/// # Arguments
///
/// * `repo_path` - Path to the repository.
/// * `branch` - The branch name.
/// * `config` - Configuration for context building.
///
/// # Returns
///
/// A [`Context`] object containing repository information and files.
///
/// # Errors
///
/// Returns an error if:
/// - The repository path does not exist
/// - The path is not a directory
/// - File reading fails
///
/// # Examples
///
/// ```no_run
/// use gba_core::context_builder::{build_context, ContextBuilderConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), gba_core::CoreError> {
///     let repo_path = std::path::PathBuf::from("/path/to/repo");
///     let context = build_context(
///         &repo_path,
///         "main",
///         &ContextBuilderConfig::default(),
///     ).await?;
///
///     println!("Found {} files", context.files.len());
///     Ok(())
/// }
/// ```
#[instrument(skip(config))]
pub async fn build_context(
    repo_path: &Path,
    branch: &str,
    config: &ContextBuilderConfig,
) -> Result<Context> {
    info!("Building context for repository: {:?}", repo_path);

    // Validate the repository path
    if !repo_path.exists() {
        return Err(CoreError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Repository path does not exist: {}", repo_path.display()),
        )));
    }

    if !repo_path.is_dir() {
        return Err(CoreError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Repository path is not a directory: {}",
                repo_path.display()
            ),
        )));
    }

    // Scan for files
    let files = scan_repository(repo_path, config).await?;

    info!(
        "Built context with {} files from branch: {}",
        files.len(),
        branch
    );

    Ok(Context {
        repository_path: repo_path.to_path_buf(),
        branch: branch.to_string(),
        files,
        metadata: HashMap::new(),
    })
}

/// Scan a repository for files matching the configuration.
///
/// # Arguments
///
/// * `repo_path` - Path to the repository.
/// * `config` - Configuration for file scanning.
///
/// # Returns
///
/// A vector of [`File`] objects.
///
/// # Errors
///
/// Returns an error if file reading fails.
#[instrument(skip(config))]
pub async fn scan_repository(repo_path: &Path, config: &ContextBuilderConfig) -> Result<Vec<File>> {
    debug!("Scanning repository: {:?}", repo_path);

    let mut files = Vec::new();
    let mut file_count = 0;

    // Walk the repository directory
    let entries = walk_directory(repo_path).await?;

    for entry in entries {
        // Check if we've reached the maximum file count
        if file_count >= config.max_files {
            debug!("Reached maximum file count: {}", config.max_files);
            break;
        }

        // Skip excluded patterns
        if should_exclude(&entry, &config.exclude_patterns) {
            debug!("Skipping excluded file: {:?}", entry);
            continue;
        }

        // Skip directories
        if entry.is_dir() {
            continue;
        }

        // Check file extension if specified
        if !config.include_extensions.is_empty() {
            let extension = entry.extension().and_then(|ext| ext.to_str()).unwrap_or("");

            if !config.include_extensions.contains(&extension.to_string()) {
                debug!("Skipping file with excluded extension: {:?}", entry);
                continue;
            }
        }

        // Read the file
        match read_file(&entry, config.max_file_size).await {
            Ok(content) => {
                let relative_path = entry
                    .strip_prefix(repo_path)
                    .unwrap_or(&entry)
                    .to_path_buf();

                let language = detect_language(&entry);
                let file = File {
                    path: relative_path,
                    content,
                    language,
                };

                files.push(file);
                file_count += 1;
            }
            Err(e) => {
                debug!("Failed to read file {:?}: {}", entry, e);
                // Continue with other files
            }
        }
    }

    info!("Scanned {} files", files.len());
    Ok(files)
}

/// Walk a directory recursively and return all entries.
///
/// # Arguments
///
/// * `path` - Path to the directory.
///
/// # Returns
///
/// A vector of [`PathBuf`] entries.
///
/// # Errors
///
/// Returns an error if directory reading fails.
pub async fn walk_directory(path: &Path) -> Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    let mut stack = vec![path.to_path_buf()];

    while let Some(current_path) = stack.pop() {
        let mut dir_entries = tokio::fs::read_dir(&current_path).await.map_err(|e| {
            CoreError::Io(std::io::Error::other(format!(
                "Failed to read directory {}: {}",
                current_path.display(),
                e
            )))
        })?;

        while let Some(entry) = dir_entries.next_entry().await.map_err(|e| {
            CoreError::Io(std::io::Error::other(format!(
                "Failed to read directory entry: {}",
                e
            )))
        })? {
            let entry_path = entry.path();

            if entry_path.is_dir() {
                // Add to stack for processing later
                stack.push(entry_path);
            } else {
                entries.push(entry_path);
            }
        }
    }

    Ok(entries)
}

/// Check if a path should be excluded based on patterns.
///
/// # Arguments
///
/// * `path` - The path to check.
/// * `exclude_patterns` - List of exclude patterns.
///
/// # Returns
///
/// `true` if the path should be excluded, `false` otherwise.
#[must_use]
pub fn should_exclude(path: &Path, exclude_patterns: &[String]) -> bool {
    for pattern in exclude_patterns {
        // Check if the path starts with the pattern
        if let Some(path_str) = path.to_str()
            && (path_str.starts_with(pattern) || path_str.contains(pattern))
        {
            return true;
        }

        // Check if any parent directory matches a pattern
        for ancestor in path.ancestors() {
            if let Some(ancestor_str) = ancestor.to_str()
                && (ancestor_str.ends_with(pattern.trim_end_matches('/'))
                    || ancestor_str.contains(pattern))
            {
                return true;
            }
        }
    }

    false
}

/// Read a file, limiting the content to the maximum size.
///
/// # Arguments
///
/// * `path` - Path to the file.
/// * `max_size` - Maximum size to read in bytes.
///
/// # Returns
///
/// The file content as a string.
///
/// # Errors
///
/// Returns an error if file reading fails.
#[instrument(skip(max_size))]
pub async fn read_file(path: &Path, max_size: usize) -> Result<String> {
    // First, check the file size
    let metadata = tokio::fs::metadata(path).await.map_err(CoreError::Io)?;

    let file_size = metadata.len() as usize;
    if file_size > max_size {
        return Err(CoreError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("File size {} exceeds maximum size {}", file_size, max_size),
        )));
    }

    // Read the file content
    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(CoreError::Io)?;

    Ok(content)
}

/// Detect the programming language of a file based on its extension.
///
/// # Arguments
///
/// * `path` - Path to the file.
///
/// # Returns
///
/// The detected language name, or "unknown" if the language could not be detected.
#[must_use]
pub fn detect_language(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "rs" => "rust".to_string(),
            "js" => "javascript".to_string(),
            "ts" => "typescript".to_string(),
            "py" => "python".to_string(),
            "java" => "java".to_string(),
            "c" | "h" => "c".to_string(),
            "cpp" | "hpp" | "cc" | "cxx" => "cpp".to_string(),
            "go" => "go".to_string(),
            "rb" => "ruby".to_string(),
            "php" => "php".to_string(),
            "swift" => "swift".to_string(),
            "kt" | "kts" => "kotlin".to_string(),
            "scala" => "scala".to_string(),
            "cs" => "csharp".to_string(),
            "fs" | "fsi" | "fsx" => "fsharp".to_string(),
            "html" => "html".to_string(),
            "css" => "css".to_string(),
            "scss" | "sass" => "scss".to_string(),
            "json" => "json".to_string(),
            "yaml" | "yml" => "yaml".to_string(),
            "toml" => "toml".to_string(),
            "md" => "markdown".to_string(),
            "txt" => "text".to_string(),
            "sh" => "shell".to_string(),
            "bash" => "bash".to_string(),
            "zsh" => "zsh".to_string(),
            "fish" => "fish".to_string(),
            "sql" => "sql".to_string(),
            "xml" => "xml".to_string(),
            "graphql" | "gql" => "graphql".to_string(),
            "dockerfile" => "dockerfile".to_string(),
            _ => "unknown".to_string(),
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Build a minimal context with only repository information.
///
/// This function creates a context without scanning files, which is useful
/// for operations that don't need file contents.
///
/// # Arguments
///
/// * `repo_path` - Path to the repository.
/// * `branch` - The branch name.
///
/// # Returns
///
/// A [`Context`] object with repository information but no files.
///
/// # Examples
///
/// ```no_run
/// use gba_core::context_builder::build_minimal_context;
///
/// #[tokio::main]
/// async fn main() -> Result<(), gba_core::CoreError> {
///     let context = build_minimal_context(
///         std::path::PathBuf::from("/path/to/repo"),
///         "main",
///     ).await?;
///
///     println!("Repository: {:?}", context.repository_path);
///     Ok(())
/// }
/// ```
#[instrument]
pub async fn build_minimal_context(
    repo_path: PathBuf,
    branch: impl Into<String> + std::fmt::Debug,
) -> Result<Context> {
    let branch = branch.into();

    info!("Building minimal context for repository: {:?}", repo_path);

    Ok(Context {
        repository_path: repo_path,
        branch,
        files: Vec::new(),
        metadata: HashMap::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_builder_config_default() {
        let config = ContextBuilderConfig::default();
        assert_eq!(config.max_files, 100);
        assert_eq!(config.max_file_size, 1_048_576);
        assert!(config.exclude_patterns.contains(&"target/".to_string()));
    }

    #[test]
    fn test_context_builder_config_with_params() {
        let config = ContextBuilderConfig::new()
            .with_max_files(50)
            .with_max_file_size(512_000)
            .with_exclude_patterns(vec!["test/".to_string()])
            .with_include_extensions(vec!["rs".to_string()]);

        assert_eq!(config.max_files, 50);
        assert_eq!(config.max_file_size, 512_000);
        assert!(config.exclude_patterns.contains(&"test/".to_string()));
        assert!(config.include_extensions.contains(&"rs".to_string()));
    }

    #[test]
    fn test_detect_language() {
        let tests = vec![
            ("test.rs", "rust"),
            ("test.js", "javascript"),
            ("test.ts", "typescript"),
            ("test.py", "python"),
            ("test.go", "go"),
            ("test.java", "java"),
            ("test.cpp", "cpp"),
            ("test.md", "markdown"),
            ("test.yaml", "yaml"),
            ("test.yml", "yaml"),
            ("test.json", "json"),
            ("Dockerfile", "unknown"),
        ];

        for (filename, expected) in tests {
            let path = PathBuf::from(filename);
            assert_eq!(detect_language(&path), expected);
        }
    }

    #[test]
    fn test_should_exclude() {
        let patterns = vec![
            "target/".to_string(),
            ".git/".to_string(),
            "node_modules/".to_string(),
        ];

        let excluded_paths = vec![
            PathBuf::from("/repo/target/main.rs"),
            PathBuf::from("/repo/.git/config"),
            PathBuf::from("/repo/node_modules/package.json"),
            PathBuf::from("/repo/src/target/debug/main.rs"),
        ];

        let included_paths = vec![
            PathBuf::from("/repo/src/main.rs"),
            PathBuf::from("/repo/README.md"),
            PathBuf::from("/repo/Cargo.toml"),
        ];

        for path in excluded_paths {
            assert!(
                should_exclude(&path, &patterns),
                "Expected to exclude: {:?}",
                path
            );
        }

        for path in included_paths {
            assert!(
                !should_exclude(&path, &patterns),
                "Expected not to exclude: {:?}",
                path
            );
        }
    }

    #[tokio::test]
    async fn test_build_minimal_context() {
        let context = build_minimal_context(PathBuf::from("/repo"), "main")
            .await
            .unwrap();

        assert_eq!(context.repository_path, PathBuf::from("/repo"));
        assert_eq!(context.branch, "main");
        assert!(context.files.is_empty());
    }
}
