//! Configuration types for Jin operations.
//!
//! This module defines the configuration structures used throughout Jin:
//! - `JinConfig`: Global Jin settings stored at `~/.jin/config.yaml`
//! - `ProjectContext`: Per-project active context stored at `.jin/context`
//!
//! # Configuration Format
//!
//! Both configuration types use YAML format with versioning for future migration:
//! - `JinConfig` stores repository location and default mode/scope
//! - `ProjectContext` stores active mode/scope for a project
//!
//! # Examples
//!
//! ```ignore
//! use jin_glm::core::config::{JinConfig, ProjectContext};
//! use std::path::Path;
//!
//! // Load global configuration
//! let config = JinConfig::load()?;
//! println!("Repository: {}", config.repository.display());
//!
//! // Load project context
//! let ctx = ProjectContext::load(Path::new("/my/project"))?;
//! if let Some(mode) = ctx.mode {
//!     println!("Active mode: {}", mode);
//! }
//! ```

use crate::core::error::{JinError, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ===== Global Jin Configuration =====

/// Global Jin configuration stored at `~/.jin/config.yaml`.
///
/// This struct contains settings that apply across all projects:
/// - `version`: Configuration format version for future migration
/// - `repository`: Path to the bare Git repository storing Jin layers
/// - `default_mode`: Optional default mode when no context is set
/// - `default_scope`: Optional default scope when no context is set
///
/// # YAML Format
///
/// ```yaml
/// version: 1
/// repository: /home/user/.jin/repo
/// default_mode: claude
/// default_scope: javascript
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct JinConfig {
    /// Configuration format version (for future migration)
    pub version: u8,
    /// Path to the bare Git repository storing Jin data
    pub repository: PathBuf,
    /// Default mode when no context is set
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_mode: Option<String>,
    /// Default scope when no context is set
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_scope: Option<String>,
}

impl Default for JinConfig {
    fn default() -> Self {
        let mut repo_path = home_dir().unwrap_or_else(|| PathBuf::from("."));
        repo_path.push(".jin/repo");

        Self {
            version: 1,
            repository: repo_path,
            default_mode: None,
            default_scope: None,
        }
    }
}

impl JinConfig {
    /// Load global Jin configuration from `~/.jin/config.yaml`.
    ///
    /// Returns the default configuration if the file doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns `JinError::InvalidConfig` if the YAML is malformed,
    /// or `JinError::Io` if reading fails for other reasons.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let config = JinConfig::load()?;
    /// ```
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let yaml = std::fs::read_to_string(&config_path)?;
        let config: JinConfig =
            serde_yaml_ng::from_str(&yaml).map_err(|e| JinError::InvalidConfig {
                message: format!("Failed to parse config.yaml: {}", e),
            })?;

        Ok(config)
    }

    /// Save global Jin configuration to `~/.jin/config.yaml`.
    ///
    /// Creates parent directories if they don't exist.
    ///
    /// # Errors
    ///
    /// Returns `JinError::ConfigError` if serialization fails,
    /// or `JinError::Io` if writing fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let config = JinConfig::default();
    /// config.save()?;
    /// ```
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let yaml = serde_yaml_ng::to_string(self).map_err(|e| JinError::ConfigError {
            message: format!("Failed to serialize config: {}", e),
        })?;

        std::fs::write(&config_path, yaml)?;

        Ok(())
    }

    /// Get the path to the global config file.
    ///
    /// Returns `~/.jin/config.yaml` using the home directory.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let path = JinConfig::config_path();
    /// println!("Config file: {}", path.display());
    /// ```
    pub fn config_path() -> PathBuf {
        let mut path = home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".jin/config.yaml");
        path
    }
}

// ===== Per-Project Context =====

/// Per-project active context stored at `.jin/context`.
///
/// This struct contains the active mode and scope for a specific project.
/// It's persisted in the project's `.jin/context` file and used by
/// commands to avoid repeatedly specifying `--mode` and `--scope` flags.
///
/// # YAML Format
///
/// ```yaml
/// version: 1
/// mode: claude
/// scope: language:javascript
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProjectContext {
    /// Context format version (for future migration)
    pub version: u8,
    /// Active mode for this project
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Active scope for this project
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}

impl Default for ProjectContext {
    fn default() -> Self {
        Self {
            version: 1,
            mode: None,
            scope: None,
        }
    }
}

impl ProjectContext {
    /// Load project context from `.jin/context`.
    ///
    /// Returns the default context if the file doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `project_dir` - Path to the project root directory
    ///
    /// # Errors
    ///
    /// Returns `JinError::InvalidConfig` if the YAML is malformed,
    /// or `JinError::Io` if reading fails for other reasons.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let ctx = ProjectContext::load(Path::new("/my/project"))?;
    /// ```
    pub fn load(project_dir: &Path) -> Result<Self> {
        let context_path = Self::context_path(project_dir);

        if !context_path.exists() {
            return Ok(Self::default());
        }

        let yaml = std::fs::read_to_string(&context_path)?;
        let context: ProjectContext =
            serde_yaml_ng::from_str(&yaml).map_err(|e| JinError::InvalidConfig {
                message: format!("Failed to parse .jin/context: {}", e),
            })?;

        Ok(context)
    }

    /// Save project context to `.jin/context`.
    ///
    /// Creates the `.jin` directory if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `project_dir` - Path to the project root directory
    ///
    /// # Errors
    ///
    /// Returns `JinError::ConfigError` if serialization fails,
    /// or `JinError::Io` if writing fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut ctx = ProjectContext::default();
    /// ctx.set_mode(Some("claude".to_string()));
    /// ctx.save(Path::new("/my/project"))?;
    /// ```
    pub fn save(&self, project_dir: &Path) -> Result<()> {
        let context_path = Self::context_path(project_dir);

        // Create .jin directory if it doesn't exist
        if let Some(parent) = context_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let yaml = serde_yaml_ng::to_string(self).map_err(|e| JinError::ConfigError {
            message: format!("Failed to serialize context: {}", e),
        })?;

        std::fs::write(&context_path, yaml)?;

        Ok(())
    }

    /// Get the path to the project context file.
    ///
    /// # Arguments
    ///
    /// * `project_dir` - Path to the project root directory
    ///
    /// # Returns
    ///
    /// Path to `.jin/context` within the project directory.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let path = ProjectContext::context_path(Path::new("/my/project"));
    /// println!("Context file: {}", path.display());
    /// ```
    pub fn context_path(project_dir: &Path) -> PathBuf {
        project_dir.join(".jin/context")
    }

    /// Set the active mode.
    ///
    /// # Arguments
    ///
    /// * `mode` - Mode name (e.g., "claude", "cursor"), or `None` to unset
    ///
    /// # Examples
    ///
    /// ```ignore
    /// ctx.set_mode(Some("claude".to_string()));
    /// ctx.set_mode(None); // Unset mode
    /// ```
    pub fn set_mode(&mut self, mode: Option<String>) {
        self.mode = mode;
    }

    /// Set the active scope.
    ///
    /// # Arguments
    ///
    /// * `scope` - Scope name (e.g., "language:javascript"), or `None` to unset
    ///
    /// # Examples
    ///
    /// ```ignore
    /// ctx.set_scope(Some("language:javascript".to_string()));
    /// ctx.set_scope(None); // Unset scope
    /// ```
    pub fn set_scope(&mut self, scope: Option<String>) {
        self.scope = scope;
    }

    /// Clear both mode and scope.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// ctx.clear();
    /// assert!(ctx.mode.is_none());
    /// assert!(ctx.scope.is_none());
    /// ```
    pub fn clear(&mut self) {
        self.mode = None;
        self.scope = None;
    }

    /// Check if context has an active mode.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if ctx.has_mode() {
    ///     println!("Active mode: {}", ctx.mode.as_ref().unwrap());
    /// }
    /// ```
    pub fn has_mode(&self) -> bool {
        self.mode.is_some()
    }

    /// Check if context has an active scope.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if ctx.has_scope() {
    ///     println!("Active scope: {}", ctx.scope.as_ref().unwrap());
    /// }
    /// ```
    pub fn has_scope(&self) -> bool {
        self.scope.is_some()
    }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ===== JinConfig Tests =====

    #[test]
    fn test_jinconfig_default() {
        let config = JinConfig::default();
        assert_eq!(config.version, 1);
        assert!(config.repository.ends_with(".jin/repo"));
        assert!(config.default_mode.is_none());
        assert!(config.default_scope.is_none());
    }

    #[test]
    fn test_jinconfig_serialize() {
        let config = JinConfig {
            version: 1,
            repository: PathBuf::from("/tmp/test"),
            default_mode: Some("claude".to_string()),
            default_scope: Some("javascript".to_string()),
        };

        let yaml = serde_yaml_ng::to_string(&config).unwrap();
        assert!(yaml.contains("version: 1"));
        assert!(yaml.contains("repository: /tmp/test"));
        assert!(yaml.contains("default_mode: claude"));
        assert!(yaml.contains("default_scope: javascript"));
    }

    #[test]
    fn test_jinconfig_serialize_omits_none_fields() {
        let config = JinConfig {
            version: 1,
            repository: PathBuf::from("/tmp/test"),
            default_mode: None,
            default_scope: None,
        };

        let yaml = serde_yaml_ng::to_string(&config).unwrap();
        // None fields should not appear in YAML
        assert!(!yaml.contains("default_mode"));
        assert!(!yaml.contains("default_scope"));
    }

    #[test]
    fn test_jinconfig_roundtrip() {
        let original = JinConfig {
            version: 1,
            repository: PathBuf::from("/tmp/test"),
            default_mode: Some("cursor".to_string()),
            default_scope: None,
        };

        let yaml = serde_yaml_ng::to_string(&original).unwrap();
        let deserialized: JinConfig = serde_yaml_ng::from_str(&yaml).unwrap();

        assert_eq!(original.version, deserialized.version);
        assert_eq!(original.repository, deserialized.repository);
        assert_eq!(original.default_mode, deserialized.default_mode);
        assert_eq!(original.default_scope, deserialized.default_scope);
    }

    #[test]
    fn test_jinconfig_config_path() {
        let path = JinConfig::config_path();
        assert!(path.ends_with(".jin/config.yaml"));
    }

    // ===== ProjectContext Tests =====

    #[test]
    fn test_projectcontext_default() {
        let ctx = ProjectContext::default();
        assert_eq!(ctx.version, 1);
        assert!(ctx.mode.is_none());
        assert!(ctx.scope.is_none());
    }

    #[test]
    fn test_projectcontext_serialize_matches_prd() {
        let ctx = ProjectContext {
            version: 1,
            mode: Some("claude".to_string()),
            scope: Some("language:javascript".to_string()),
        };

        let yaml = serde_yaml_ng::to_string(&ctx).unwrap();

        // Verify format matches PRD §7.1 exactly
        assert!(yaml.contains("version: 1"));
        assert!(yaml.contains("mode: claude"));
        assert!(yaml.contains("scope: language:javascript"));
    }

    #[test]
    fn test_projectcontext_serialize_omits_none_fields() {
        let ctx = ProjectContext {
            version: 1,
            mode: None,
            scope: None,
        };

        let yaml = serde_yaml_ng::to_string(&ctx).unwrap();
        // None fields should not appear in YAML
        assert!(!yaml.contains("mode"));
        assert!(!yaml.contains("scope"));
    }

    #[test]
    fn test_projectcontext_context_path() {
        let project_dir = Path::new("/my/project");
        let path = ProjectContext::context_path(project_dir);
        assert_eq!(path, PathBuf::from("/my/project/.jin/context"));
    }

    #[test]
    fn test_projectcontext_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        let original = ProjectContext {
            version: 1,
            mode: Some("zed".to_string()),
            scope: Some("language:rust".to_string()),
        };

        original.save(project_dir).unwrap();
        let loaded = ProjectContext::load(project_dir).unwrap();

        assert_eq!(original.version, loaded.version);
        assert_eq!(original.mode, loaded.mode);
        assert_eq!(original.scope, loaded.scope);
    }

    #[test]
    fn test_projectcontext_load_missing_returns_default() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        // Don't create context file
        let loaded = ProjectContext::load(project_dir).unwrap();

        assert_eq!(loaded, ProjectContext::default());
    }

    #[test]
    fn test_projectcontext_save_creates_jin_directory() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        let ctx = ProjectContext {
            version: 1,
            mode: Some("claude".to_string()),
            scope: None,
        };

        ctx.save(project_dir).unwrap();

        // Verify .jin directory was created
        let jin_dir = project_dir.join(".jin");
        assert!(jin_dir.exists());
        assert!(jin_dir.is_dir());

        // Verify context file was created
        let context_file = project_dir.join(".jin/context");
        assert!(context_file.exists());
    }

    #[test]
    fn test_projectcontext_setters() {
        let mut ctx = ProjectContext::default();

        ctx.set_mode(Some("claude".to_string()));
        assert_eq!(ctx.mode, Some("claude".to_string()));
        assert!(ctx.has_mode());

        ctx.set_scope(Some("python".to_string()));
        assert_eq!(ctx.scope, Some("python".to_string()));
        assert!(ctx.has_scope());

        ctx.clear();
        assert!(ctx.mode.is_none());
        assert!(ctx.scope.is_none());
        assert!(!ctx.has_mode());
        assert!(!ctx.has_scope());
    }

    #[test]
    fn test_projectcontext_set_mode_to_none() {
        let mut ctx = ProjectContext {
            version: 1,
            mode: Some("claude".to_string()),
            scope: Some("python".to_string()),
        };

        ctx.set_mode(None);
        assert!(ctx.mode.is_none());
        assert!(ctx.has_scope()); // Scope should still be set
    }

    #[test]
    fn test_projectcontext_set_scope_to_none() {
        let mut ctx = ProjectContext {
            version: 1,
            mode: Some("claude".to_string()),
            scope: Some("python".to_string()),
        };

        ctx.set_scope(None);
        assert!(ctx.scope.is_none());
        assert!(ctx.has_mode()); // Mode should still be set
    }

    // ===== YAML Format Validation Tests =====

    #[test]
    fn test_yaml_format_version_first() {
        let ctx = ProjectContext {
            version: 1,
            mode: Some("claude".to_string()),
            scope: Some("language:javascript".to_string()),
        };

        let yaml = serde_yaml_ng::to_string(&ctx).unwrap();
        let lines: Vec<&str> = yaml.lines().collect();

        // version should be first line
        assert!(lines[0].contains("version: 1"));
    }
}
