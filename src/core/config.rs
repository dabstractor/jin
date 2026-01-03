//! Configuration types for Jin

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::core::error::{JinError, Result};

fn default_version() -> u32 {
    1
}

/// Global Jin configuration (stored at ~/.jin/config.toml)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JinConfig {
    /// Version of the config schema
    #[serde(default = "default_version")]
    pub version: u32,

    /// Remote repository URL for sync
    pub remote: Option<RemoteConfig>,

    /// User information
    pub user: Option<UserConfig>,
}

/// Remote repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    /// URL of the remote Jin repository
    pub url: String,
    /// Whether to fetch on init
    #[serde(default)]
    pub fetch_on_init: bool,
}

/// User configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    /// User's name
    pub name: Option<String>,
    /// User's email
    pub email: Option<String>,
}

impl JinConfig {
    /// Load config from default location (~/.jin/config.toml)
    pub fn load() -> Result<Self> {
        let path = Self::default_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            toml::from_str(&content)
                .map_err(|e| JinError::Config(format!("Failed to parse config: {}", e)))
        } else {
            Ok(Self::default())
        }
    }

    /// Save config to default location
    pub fn save(&self) -> Result<()> {
        let path = Self::default_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| JinError::Config(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Returns default config path (~/.jin/config.toml or $JIN_DIR/config.toml)
    ///
    /// Respects JIN_DIR environment variable for test isolation.
    /// If JIN_DIR is set, returns $JIN_DIR/config.toml.
    /// Otherwise, returns ~/.jin/config.toml.
    pub fn default_path() -> Result<PathBuf> {
        // Check JIN_DIR environment variable first for test isolation
        if let Ok(jin_dir) = std::env::var("JIN_DIR") {
            return Ok(PathBuf::from(jin_dir).join("config.toml"));
        }

        // Fall back to default ~/.jin location
        dirs::home_dir()
            .map(|h| h.join(".jin").join("config.toml"))
            .ok_or_else(|| JinError::Config("Cannot determine home directory".into()))
    }
}

/// Per-project context (stored at .jin/context)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectContext {
    /// Version of the context schema
    #[serde(default = "default_version")]
    pub version: u32,

    /// Currently active mode
    pub mode: Option<String>,

    /// Currently active scope
    pub scope: Option<String>,

    /// Project name (auto-inferred from Git remote)
    pub project: Option<String>,

    /// Last update timestamp
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
}

impl ProjectContext {
    /// Load context from .jin/context in current directory
    pub fn load() -> Result<Self> {
        let path = Self::default_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            serde_yaml::from_str(&content)
                .map_err(|e| JinError::Config(format!("Failed to parse context: {}", e)))
        } else {
            Err(JinError::NotInitialized)
        }
    }

    /// Save context to .jin/context
    pub fn save(&self) -> Result<()> {
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_yaml::to_string(self)
            .map_err(|e| JinError::Config(format!("Failed to serialize context: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Returns default context path (.jin/context)
    pub fn default_path() -> PathBuf {
        PathBuf::from(".jin").join("context")
    }

    /// Check if Jin is initialized in current directory
    pub fn is_initialized() -> bool {
        Self::default_path()
            .parent()
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    /// Get the active mode, returning an error if not set
    pub fn require_mode(&self) -> Result<&str> {
        self.mode
            .as_deref()
            .ok_or_else(|| JinError::NoActiveContext {
                context_type: "mode".to_string(),
            })
    }

    /// Get the active scope, returning an error if not set
    pub fn require_scope(&self) -> Result<&str> {
        self.scope
            .as_deref()
            .ok_or_else(|| JinError::NoActiveContext {
                context_type: "scope".to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = JinConfig::default();
        assert_eq!(config.version, 0); // Default is 0, loaded defaults to 1
        assert!(config.remote.is_none());
        assert!(config.user.is_none());
    }

    #[test]
    fn test_config_serialization() {
        let config = JinConfig {
            version: 1,
            remote: Some(RemoteConfig {
                url: "git@github.com:org/jin-config".to_string(),
                fetch_on_init: true,
            }),
            user: Some(UserConfig {
                name: Some("Test User".to_string()),
                email: Some("test@example.com".to_string()),
            }),
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("version = 1"));
        assert!(toml_str.contains("[remote]"));
        assert!(toml_str.contains("git@github.com:org/jin-config"));
    }

    #[test]
    fn test_default_context() {
        let ctx = ProjectContext::default();
        assert_eq!(ctx.version, 0);
        assert!(ctx.mode.is_none());
        assert!(ctx.scope.is_none());
    }

    #[test]
    fn test_context_serialization() {
        let ctx = ProjectContext {
            version: 1,
            mode: Some("claude".to_string()),
            scope: Some("language:javascript".to_string()),
            project: Some("ui-dashboard".to_string()),
            last_updated: Some("2025-01-01T00:00:00Z".to_string()),
        };

        let yaml_str = serde_yaml::to_string(&ctx).unwrap();
        assert!(yaml_str.contains("version: 1"));
        assert!(yaml_str.contains("mode: claude"));
        assert!(
            yaml_str.contains("scope: 'language:javascript'")
                || yaml_str.contains("scope: language:javascript")
        );
    }

    #[test]
    fn test_require_mode_error() {
        let ctx = ProjectContext::default();
        let result = ctx.require_mode();
        assert!(result.is_err());
    }

    #[test]
    fn test_require_mode_success() {
        let ctx = ProjectContext {
            mode: Some("claude".to_string()),
            ..Default::default()
        };
        assert_eq!(ctx.require_mode().unwrap(), "claude");
    }
}
