//! Layer type definitions for the 9-layer hierarchy.
//!
//! This module defines the core `Layer` enum representing Jin's multi-layer
//! configuration management system. Each layer represents a different scope
//! of configuration with well-defined storage paths, Git references, and
//! precedence ordering.
//!
//! # Layer Hierarchy (Lowest to Highest Precedence)
//!
//! | Layer | Variant Name | Description | Storage Path | Git Ref |
//! |-------|--------------|-------------|--------------|---------|
//! | 1 | `GlobalBase` | Shared defaults | `jin/global/` | `refs/jin/layers/global` |
//! | 2 | `ModeBase` | Mode defaults | `jin/mode/<mode>/` | `refs/jin/layers/mode/<mode>` |
//! | 3 | `ModeScope` | Scoped mode configs | `jin/mode/<mode>/scope/<scope>/` | `refs/jin/layers/mode/<mode>/scope/<scope>` |
//! | 4 | `ModeScopeProject` | Project overrides for scoped mode | `jin/mode/<mode>/scope/<scope>/project/<project>/` | `refs/jin/layers/mode/<mode>/scope/<scope>/project/<project>` |
//! | 5 | `ModeProject` | Project overrides for mode | `jin/mode/<mode>/project/<project>/` | `refs/jin/layers/mode/<mode>/project/<project>` |
//! | 6 | `ScopeBase` | Untethered scope configs | `jin/scope/<scope>/` | `refs/jin/layers/scope/<scope>` |
//! | 7 | `ProjectBase` | Project-only configs | `jin/project/<project>/` | `refs/jin/layers/project/<project>` |
//! | 8 | `UserLocal` | Machine-only overlays | `~/.jin/local/` | Not versioned (local only) |
//! | 9 | `WorkspaceActive` | Derived merge result | `.jin/workspace/` | Not versioned (derived) |
//!
//! # Precedence
//!
//! Layers are ordered by precedence using the `Ord` trait. Lower layer numbers
//! have lower precedence (Layer 1 < Layer 9). During merge operations,
//! higher-numbered layers override lower-numbered layers.
//!
//! # Examples
//!
//! ```
//! use jin_glm::core::Layer;
//!
//! // Create a layer from CLI flags
//! let layer = Layer::from_flags(Some("claude"), Some("python"), Some("myapp"), false).unwrap();
//! assert_eq!(layer.storage_path("myapp"), std::path::PathBuf::from("jin/mode/claude/scope/python/project/myapp"));
//!
//! // Check if a layer is versioned in Git
//! assert!(Layer::GlobalBase.is_versioned());
//! assert!(!Layer::UserLocal.is_versioned());
//! ```

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// The 9-layer hierarchy for Jin configuration management.
///
/// Layers are declared in precedence order (1-9) so that derived `Ord` implementations
/// correctly order them for merge operations.
///
/// # Data-Carrying Variants
///
/// Variants that include mode, scope, or project fields store these values as owned
/// `String` data to enable self-contained layer instances that can be passed around
/// without requiring additional context.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Layer {
    // ===== Base Layers (1-2) =====
    /// Layer 1: Global defaults shared across all projects and modes.
    ///
    /// Storage: `jin/global/`
    /// Git Ref: `refs/jin/layers/global`
    GlobalBase,

    /// Layer 2: Mode-specific defaults (e.g., "claude", "cursor", "copilot").
    ///
    /// Storage: `jin/mode/<mode>/`
    /// Git Ref: `refs/jin/layers/mode/<mode>`
    ModeBase { mode: String },

    // ===== Mode-Bound Scope Layers (3-5) =====
    /// Layer 3: Mode + scope combination (e.g., claude + python).
    ///
    /// Storage: `jin/mode/<mode>/scope/<scope>/`
    /// Git Ref: `refs/jin/layers/mode/<mode>/scope/<scope>`
    ModeScope { mode: String, scope: String },

    /// Layer 4: Mode + scope + project (full hierarchy).
    ///
    /// Storage: `jin/mode/<mode>/scope/<scope>/project/<project>/`
    /// Git Ref: `refs/jin/layers/mode/<mode>/scope/<scope>/project/<project>`
    ModeScopeProject {
        mode: String,
        scope: String,
        project: String,
    },

    /// Layer 5: Mode + project (scope-independent project overrides).
    ///
    /// Storage: `jin/mode/<mode>/project/<project>/`
    /// Git Ref: `refs/jin/layers/mode/<mode>/project/<project>`
    ModeProject { mode: String, project: String },

    // ===== Untethered Layers (6-7) =====
    /// Layer 6: Untethered scope (not bound to any mode).
    ///
    /// Storage: `jin/scope/<scope>/`
    /// Git Ref: `refs/jin/layers/scope/<scope>`
    ScopeBase { scope: String },

    /// Layer 7: Project-only configuration (default layer).
    ///
    /// Storage: `jin/project/<project>/`
    /// Git Ref: `refs/jin/layers/project/<project>`
    ProjectBase { project: String },

    // ===== Local Layers (8-9) =====
    /// Layer 8: User-local overlays (machine-specific, not versioned).
    ///
    /// Storage: `~/.jin/local/`
    /// Git Ref: None (local only, not stored in Git)
    UserLocal,

    /// Layer 9: Active workspace (derived result, never source of truth).
    ///
    /// Storage: `.jin/workspace/`
    /// Git Ref: None (derived from other layers, not stored)
    WorkspaceActive,
}

// ===== Constants =====

impl Layer {
    /// Total number of layers in the hierarchy.
    pub const LAYER_COUNT: usize = 9;

    /// Root directory for Jin storage in versioned layers.
    pub const JIN_ROOT: &str = "jin";

    /// Storage path for global base layer.
    pub const GLOBAL_BASE_PATH: &str = "jin/global";

    /// Storage path for user-local layer.
    pub const USER_LOCAL_PATH: &str = "~/.jin/local";

    /// Storage path for workspace active layer.
    pub const WORKSPACE_PATH: &str = ".jin/workspace";

    /// Git ref namespace prefix for all Jin layer refs.
    pub const GIT_REF_PREFIX: &str = "refs/jin/layers";
}

// ===== Storage Path =====

impl Layer {
    /// Returns the storage path for files in this layer.
    ///
    /// The `project` parameter is used for layers that reference a project name.
    /// For variants that already contain a project field, this parameter may
    /// be used for validation or as a fallback.
    ///
    /// # Path Construction
    ///
    /// Paths are constructed using `PathBuf::join()` for cross-platform compatibility.
    /// Local layers (UserLocal, WorkspaceActive) use special path prefixes.
    ///
    /// # Examples
    ///
    /// ```
    /// use jin_glm::core::Layer;
    ///
    /// let layer = Layer::ModeBase { mode: "claude".to_string() };
    /// assert_eq!(layer.storage_path("myapp"), std::path::PathBuf::from("jin/mode/claude"));
    ///
    /// let layer = Layer::UserLocal;
    /// assert_eq!(layer.storage_path("myapp"), std::path::PathBuf::from("~/.jin/local"));
    /// ```
    pub fn storage_path(&self, _project: &str) -> PathBuf {
        match self {
            Layer::GlobalBase => PathBuf::from(Self::GLOBAL_BASE_PATH),

            Layer::ModeBase { mode } => PathBuf::from(Self::JIN_ROOT).join("mode").join(mode),

            Layer::ModeScope { mode, scope } => PathBuf::from(Self::JIN_ROOT)
                .join("mode")
                .join(mode)
                .join("scope")
                .join(scope),

            Layer::ModeScopeProject {
                mode,
                scope,
                project: proj,
            } => PathBuf::from(Self::JIN_ROOT)
                .join("mode")
                .join(mode)
                .join("scope")
                .join(scope)
                .join("project")
                .join(proj),

            Layer::ModeProject {
                mode,
                project: proj,
            } => PathBuf::from(Self::JIN_ROOT)
                .join("mode")
                .join(mode)
                .join("project")
                .join(proj),

            Layer::ScopeBase { scope } => PathBuf::from(Self::JIN_ROOT).join("scope").join(scope),

            Layer::ProjectBase { project: proj } => {
                PathBuf::from(Self::JIN_ROOT).join("project").join(proj)
            }

            Layer::UserLocal => PathBuf::from(Self::USER_LOCAL_PATH),

            Layer::WorkspaceActive => PathBuf::from(Self::WORKSPACE_PATH),
        }
    }
}

// ===== Git Reference =====

impl Layer {
    /// Returns the Git reference for this layer, if versioned.
    ///
    /// Layers 1-7 are versioned in Git and return `Some(ref)`.
    /// Layers 8-9 (UserLocal, WorkspaceActive) are not versioned and return `None`.
    ///
    /// # Git Ref Format
    ///
    /// All refs follow the pattern: `refs/jin/layers/...`
    ///
    /// # Examples
    ///
    /// ```
    /// use jin_glm::core::Layer;
    ///
    /// assert_eq!(
    ///     Layer::GlobalBase.git_ref(),
    ///     Some("refs/jin/layers/global".to_string())
    /// );
    ///
    /// assert_eq!(Layer::UserLocal.git_ref(), None);
    /// ```
    pub fn git_ref(&self) -> Option<String> {
        match self {
            Layer::GlobalBase => Some(format!("{}/global", Self::GIT_REF_PREFIX)),

            Layer::ModeBase { mode } => Some(format!("{}/mode/{}", Self::GIT_REF_PREFIX, mode)),

            Layer::ModeScope { mode, scope } => Some(format!(
                "{}/mode/{}/scope/{}",
                Self::GIT_REF_PREFIX,
                mode,
                scope
            )),

            Layer::ModeScopeProject {
                mode,
                scope,
                project,
            } => Some(format!(
                "{}/mode/{}/scope/{}/project/{}",
                Self::GIT_REF_PREFIX,
                mode,
                scope,
                project
            )),

            Layer::ModeProject { mode, project } => Some(format!(
                "{}/mode/{}/project/{}",
                Self::GIT_REF_PREFIX,
                mode,
                project
            )),

            Layer::ScopeBase { scope } => Some(format!("{}/scope/{}", Self::GIT_REF_PREFIX, scope)),

            Layer::ProjectBase { project } => {
                Some(format!("{}/project/{}", Self::GIT_REF_PREFIX, project))
            }

            // Layers 8-9 are not versioned
            Layer::UserLocal | Layer::WorkspaceActive => None,
        }
    }
}

// ===== CLI Flag Routing =====

impl Layer {
    /// Routes CLI flags to the appropriate layer variant.
    ///
    /// This method implements the layer routing logic that maps combinations of
    /// CLI flags (`--mode`, `--scope`, `--project`, `--global`) to target layers.
    ///
    /// # Routing Table (Priority Order)
    ///
    /// | Flags | Target Layer |
    /// |-------|--------------|
    /// | `--global` | GlobalBase |
    /// | `--mode --scope --project` | ModeScopeProject |
    /// | `--mode --project` | ModeProject |
    /// | `--mode --scope` | ModeScope |
    /// | `--scope` | ScopeBase |
    /// | `--mode` | ModeBase |
    /// | `--project` | ProjectBase |
    /// | (none) | None (use default/project inference) |
    ///
    /// # Arguments
    ///
    /// * `mode` - Optional mode name from `--mode` flag
    /// * `scope` - Optional scope name from `--scope` flag
    /// * `project` - Optional project name from `--project` flag
    /// * `global` - Global flag from `--global` (boolean)
    ///
    /// # Returns
    ///
    /// * `Some(Layer)` - The routed layer for valid flag combinations
    /// * `None` - No flags provided (caller should use project inference)
    ///
    /// # Examples
    ///
    /// ```
    /// use jin_glm::core::Layer;
    ///
    /// // Global flag takes precedence
    /// let layer = Layer::from_flags(None, None, None, true).unwrap();
    /// assert!(matches!(layer, Layer::GlobalBase));
    ///
    /// // Full hierarchy
    /// let layer = Layer::from_flags(Some("claude"), Some("python"), Some("myapp"), false).unwrap();
    /// assert!(matches!(layer, Layer::ModeScopeProject { .. }));
    ///
    /// // No flags returns None
    /// assert_eq!(Layer::from_flags(None, None, None, false), None);
    /// ```
    pub fn from_flags(
        mode: Option<&str>,
        scope: Option<&str>,
        project: Option<&str>,
        global: bool,
    ) -> Option<Self> {
        match (global, mode, scope, project) {
            // Global flag takes precedence over all other flags
            (true, _, _, _) => Some(Layer::GlobalBase),

            // Full hierarchy (most specific combination)
            (_, Some(m), Some(s), Some(p)) => Some(Layer::ModeScopeProject {
                mode: m.to_string(),
                scope: s.to_string(),
                project: p.to_string(),
            }),

            // Mode + Project (no scope)
            (_, Some(m), None, Some(p)) => Some(Layer::ModeProject {
                mode: m.to_string(),
                project: p.to_string(),
            }),

            // Mode + Scope (no project)
            (_, Some(m), Some(s), None) => Some(Layer::ModeScope {
                mode: m.to_string(),
                scope: s.to_string(),
            }),

            // Scope only (untethered scope)
            (_, None, Some(s), _) => Some(Layer::ScopeBase {
                scope: s.to_string(),
            }),

            // Mode only
            (_, Some(m), None, None) => Some(Layer::ModeBase {
                mode: m.to_string(),
            }),

            // Project only - explicit project flag
            (_, None, None, Some(p)) => Some(Layer::ProjectBase {
                project: p.to_string(),
            }),

            // No flags - return None for default/project inference behavior
            (false, None, None, None) => None,
        }
    }
}

// ===== Helper Methods =====

impl Layer {
    /// Extracts the mode name from this layer, if present.
    ///
    /// Returns `Some(mode)` for layers that contain a mode field,
    /// `None` for all other layers.
    ///
    /// # Examples
    ///
    /// ```
    /// use jin_glm::core::Layer;
    ///
    /// let layer = Layer::ModeBase { mode: "claude".to_string() };
    /// assert_eq!(layer.mode(), Some("claude"));
    ///
    /// let layer = Layer::ScopeBase { scope: "python".to_string() };
    /// assert_eq!(layer.mode(), None);
    /// ```
    pub fn mode(&self) -> Option<&str> {
        match self {
            Layer::ModeBase { mode }
            | Layer::ModeScope { mode, .. }
            | Layer::ModeScopeProject { mode, .. }
            | Layer::ModeProject { mode, .. } => Some(mode),
            _ => None,
        }
    }

    /// Extracts the scope name from this layer, if present.
    ///
    /// Returns `Some(scope)` for layers that contain a scope field,
    /// `None` for all other layers.
    ///
    /// # Examples
    ///
    /// ```
    /// use jin_glm::core::Layer;
    ///
    /// let layer = Layer::ModeScope { mode: "claude".to_string(), scope: "python".to_string() };
    /// assert_eq!(layer.scope(), Some("python"));
    ///
    /// let layer = Layer::ModeBase { mode: "claude".to_string() };
    /// assert_eq!(layer.scope(), None);
    /// ```
    pub fn scope(&self) -> Option<&str> {
        match self {
            Layer::ModeScope { scope, .. } | Layer::ModeScopeProject { scope, .. } => Some(scope),
            Layer::ScopeBase { scope } => Some(scope),
            _ => None,
        }
    }

    /// Extracts the project name from this layer, if present.
    ///
    /// Returns `Some(project)` for layers that contain a project field,
    /// `None` for all other layers.
    ///
    /// # Examples
    ///
    /// ```
    /// use jin_glm::core::Layer;
    ///
    /// let layer = Layer::ProjectBase { project: "myapp".to_string() };
    /// assert_eq!(layer.project(), Some("myapp"));
    ///
    /// let layer = Layer::ModeBase { mode: "claude".to_string() };
    /// assert_eq!(layer.project(), None);
    /// ```
    pub fn project(&self) -> Option<&str> {
        match self {
            Layer::ProjectBase { project }
            | Layer::ModeScopeProject { project, .. }
            | Layer::ModeProject { project, .. } => Some(project),
            _ => None,
        }
    }

    /// Returns `true` if this layer is versioned in Git.
    ///
    /// Layers 1-7 are versioned and have Git refs.
    /// Layers 8-9 (UserLocal, WorkspaceActive) are not versioned.
    ///
    /// # Examples
    ///
    /// ```
    /// use jin_glm::core::Layer;
    ///
    /// assert!(Layer::GlobalBase.is_versioned());
    /// assert!(Layer::ProjectBase { project: "x".to_string() }.is_versioned());
    /// assert!(!Layer::UserLocal.is_versioned());
    /// assert!(!Layer::WorkspaceActive.is_versioned());
    /// ```
    pub fn is_versioned(&self) -> bool {
        !matches!(self, Layer::UserLocal | Layer::WorkspaceActive)
    }
}

// ===== DISPLAY IMPLEMENTATION =====

impl std::fmt::Display for Layer {
    /// Formats the layer as a path string for use in conflict markers and logging.
    ///
    /// The format follows the layer hierarchy pattern:
    /// - `global` for GlobalBase
    /// - `mode/<mode>` for ModeBase
    /// - `mode/<mode>/scope/<scope>` for ModeScope
    /// - `mode/<mode>/scope/<scope>/project/<project>` for ModeScopeProject
    /// - `mode/<mode>/project/<project>` for ModeProject
    /// - `scope/<scope>` for ScopeBase
    /// - `project/<project>` for ProjectBase
    /// - `user-local` for UserLocal
    /// - `workspace-active` for WorkspaceActive
    ///
    /// # Examples
    ///
    /// ```
    /// use jin_glm::core::Layer;
    ///
    /// assert_eq!(Layer::GlobalBase.to_string(), "global");
    /// assert_eq!(Layer::ModeBase { mode: "claude".to_string() }.to_string(), "mode/claude");
    /// assert_eq!(Layer::ProjectBase { project: "myproject".to_string() }.to_string(), "project/myproject");
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Layer::GlobalBase => write!(f, "global"),
            Layer::ModeBase { mode } => write!(f, "mode/{}", mode),
            Layer::ModeScope { mode, scope } => write!(f, "mode/{}/scope/{}", mode, scope),
            Layer::ModeScopeProject {
                mode,
                scope,
                project,
            } => write!(f, "mode/{}/scope/{}/project/{}", mode, scope, project),
            Layer::ModeProject { mode, project } => {
                write!(f, "mode/{}/project/{}", mode, project)
            }
            Layer::ScopeBase { scope } => write!(f, "scope/{}", scope),
            Layer::ProjectBase { project } => write!(f, "project/{}", project),
            Layer::UserLocal => write!(f, "user-local"),
            Layer::WorkspaceActive => write!(f, "workspace-active"),
        }
    }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Layer Ordering Tests =====

    #[test]
    fn test_layer_ordering() {
        // Verify precedence: GlobalBase < WorkspaceActive
        assert!(Layer::GlobalBase < Layer::WorkspaceActive);

        // Verify cross-category ordering
        assert!(
            Layer::ModeBase {
                mode: "x".to_string()
            } < Layer::ScopeBase {
                scope: "y".to_string()
            }
        );

        // Verify specific precedence relationships
        assert!(Layer::GlobalBase < Layer::ModeBase { mode: "m".into() });
        assert!(
            Layer::ModeBase { mode: "m".into() }
                < Layer::ModeScope {
                    mode: "m".into(),
                    scope: "s".into()
                }
        );
        assert!(
            Layer::ModeScope {
                mode: "m".into(),
                scope: "s".into()
            } < Layer::ModeScopeProject {
                mode: "m".into(),
                scope: "s".into(),
                project: "p".into()
            }
        );
    }

    // ===== Storage Path Tests =====

    #[test]
    fn test_storage_path_global_base() {
        let layer = Layer::GlobalBase;
        assert_eq!(layer.storage_path("myproject"), PathBuf::from("jin/global"));
    }

    #[test]
    fn test_storage_path_mode_base() {
        let layer = Layer::ModeBase {
            mode: "claude".to_string(),
        };
        assert_eq!(
            layer.storage_path("myproject"),
            PathBuf::from("jin/mode/claude")
        );
    }

    #[test]
    fn test_storage_path_mode_scope() {
        let layer = Layer::ModeScope {
            mode: "claude".to_string(),
            scope: "python".to_string(),
        };
        assert_eq!(
            layer.storage_path("myproject"),
            PathBuf::from("jin/mode/claude/scope/python")
        );
    }

    #[test]
    fn test_storage_path_mode_scope_project() {
        let layer = Layer::ModeScopeProject {
            mode: "claude".to_string(),
            scope: "python".to_string(),
            project: "myapp".to_string(),
        };
        assert_eq!(
            layer.storage_path("myproject"),
            PathBuf::from("jin/mode/claude/scope/python/project/myapp")
        );
    }

    #[test]
    fn test_storage_path_mode_project() {
        let layer = Layer::ModeProject {
            mode: "claude".to_string(),
            project: "myapp".to_string(),
        };
        assert_eq!(
            layer.storage_path("myproject"),
            PathBuf::from("jin/mode/claude/project/myapp")
        );
    }

    #[test]
    fn test_storage_path_scope_base() {
        let layer = Layer::ScopeBase {
            scope: "python".to_string(),
        };
        assert_eq!(
            layer.storage_path("myproject"),
            PathBuf::from("jin/scope/python")
        );
    }

    #[test]
    fn test_storage_path_project_base() {
        let layer = Layer::ProjectBase {
            project: "myapp".to_string(),
        };
        assert_eq!(
            layer.storage_path("myproject"),
            PathBuf::from("jin/project/myapp")
        );
    }

    #[test]
    fn test_storage_path_user_local() {
        let layer = Layer::UserLocal;
        assert_eq!(
            layer.storage_path("myproject"),
            PathBuf::from("~/.jin/local")
        );
    }

    #[test]
    fn test_storage_path_workspace_active() {
        let layer = Layer::WorkspaceActive;
        assert_eq!(
            layer.storage_path("myproject"),
            PathBuf::from(".jin/workspace")
        );
    }

    // ===== Git Ref Tests =====

    #[test]
    fn test_git_ref_global_base() {
        assert_eq!(
            Layer::GlobalBase.git_ref(),
            Some("refs/jin/layers/global".to_string())
        );
    }

    #[test]
    fn test_git_ref_mode_base() {
        assert_eq!(
            Layer::ModeBase {
                mode: "claude".to_string()
            }
            .git_ref(),
            Some("refs/jin/layers/mode/claude".to_string())
        );
    }

    #[test]
    fn test_git_ref_mode_scope() {
        assert_eq!(
            Layer::ModeScope {
                mode: "claude".to_string(),
                scope: "python".to_string()
            }
            .git_ref(),
            Some("refs/jin/layers/mode/claude/scope/python".to_string())
        );
    }

    #[test]
    fn test_git_ref_mode_scope_project() {
        assert_eq!(
            Layer::ModeScopeProject {
                mode: "claude".to_string(),
                scope: "python".to_string(),
                project: "myapp".to_string()
            }
            .git_ref(),
            Some("refs/jin/layers/mode/claude/scope/python/project/myapp".to_string())
        );
    }

    #[test]
    fn test_git_ref_mode_project() {
        assert_eq!(
            Layer::ModeProject {
                mode: "claude".to_string(),
                project: "myapp".to_string()
            }
            .git_ref(),
            Some("refs/jin/layers/mode/claude/project/myapp".to_string())
        );
    }

    #[test]
    fn test_git_ref_scope_base() {
        assert_eq!(
            Layer::ScopeBase {
                scope: "python".to_string()
            }
            .git_ref(),
            Some("refs/jin/layers/scope/python".to_string())
        );
    }

    #[test]
    fn test_git_ref_project_base() {
        assert_eq!(
            Layer::ProjectBase {
                project: "myapp".to_string()
            }
            .git_ref(),
            Some("refs/jin/layers/project/myapp".to_string())
        );
    }

    #[test]
    fn test_git_ref_non_versioned_layers() {
        assert_eq!(Layer::UserLocal.git_ref(), None);
        assert_eq!(Layer::WorkspaceActive.git_ref(), None);
    }

    // ===== from_flags Routing Tests =====

    #[test]
    fn test_from_flags_global() {
        let layer = Layer::from_flags(None, None, None, true).unwrap();
        assert!(matches!(layer, Layer::GlobalBase));
    }

    #[test]
    fn test_from_flags_global_with_other_flags() {
        // Global flag should take precedence even with other flags
        let layer = Layer::from_flags(Some("claude"), Some("python"), Some("myapp"), true).unwrap();
        assert!(matches!(layer, Layer::GlobalBase));
    }

    #[test]
    fn test_from_flags_mode_only() {
        let layer = Layer::from_flags(Some("claude"), None, None, false).unwrap();
        assert_eq!(
            layer,
            Layer::ModeBase {
                mode: "claude".to_string()
            }
        );
    }

    #[test]
    fn test_from_flags_scope_only() {
        let layer = Layer::from_flags(None, Some("python"), None, false).unwrap();
        assert_eq!(
            layer,
            Layer::ScopeBase {
                scope: "python".to_string()
            }
        );
    }

    #[test]
    fn test_from_flags_project_only() {
        let layer = Layer::from_flags(None, None, Some("myapp"), false).unwrap();
        assert_eq!(
            layer,
            Layer::ProjectBase {
                project: "myapp".to_string()
            }
        );
    }

    #[test]
    fn test_from_flags_mode_and_scope() {
        let layer = Layer::from_flags(Some("claude"), Some("python"), None, false).unwrap();
        assert_eq!(
            layer,
            Layer::ModeScope {
                mode: "claude".to_string(),
                scope: "python".to_string()
            }
        );
    }

    #[test]
    fn test_from_flags_mode_and_project() {
        let layer = Layer::from_flags(Some("claude"), None, Some("myapp"), false).unwrap();
        assert_eq!(
            layer,
            Layer::ModeProject {
                mode: "claude".to_string(),
                project: "myapp".to_string()
            }
        );
    }

    #[test]
    fn test_from_flags_full_hierarchy() {
        let layer =
            Layer::from_flags(Some("claude"), Some("python"), Some("myapp"), false).unwrap();
        assert_eq!(
            layer,
            Layer::ModeScopeProject {
                mode: "claude".to_string(),
                scope: "python".to_string(),
                project: "myapp".to_string()
            }
        );
    }

    #[test]
    fn test_from_flags_no_flags() {
        // No flags should return None for default behavior
        assert_eq!(Layer::from_flags(None, None, None, false), None);
    }

    #[test]
    fn test_from_flags_scope_and_project() {
        // scope + project without mode should route to ScopeBase
        let layer = Layer::from_flags(None, Some("python"), Some("myapp"), false).unwrap();
        assert_eq!(
            layer,
            Layer::ScopeBase {
                scope: "python".to_string()
            }
        );
    }

    // ===== Helper Method Tests =====

    #[test]
    fn test_helper_methods_mode() {
        let layer = Layer::ModeBase {
            mode: "claude".to_string(),
        };
        assert_eq!(layer.mode(), Some("claude"));
        assert_eq!(layer.scope(), None);
        assert_eq!(layer.project(), None);
    }

    #[test]
    fn test_helper_methods_mode_scope() {
        let layer = Layer::ModeScope {
            mode: "claude".to_string(),
            scope: "python".to_string(),
        };
        assert_eq!(layer.mode(), Some("claude"));
        assert_eq!(layer.scope(), Some("python"));
        assert_eq!(layer.project(), None);
    }

    #[test]
    fn test_helper_methods_mode_scope_project() {
        let layer = Layer::ModeScopeProject {
            mode: "claude".to_string(),
            scope: "python".to_string(),
            project: "myapp".to_string(),
        };
        assert_eq!(layer.mode(), Some("claude"));
        assert_eq!(layer.scope(), Some("python"));
        assert_eq!(layer.project(), Some("myapp"));
    }

    #[test]
    fn test_helper_methods_mode_project() {
        let layer = Layer::ModeProject {
            mode: "claude".to_string(),
            project: "myapp".to_string(),
        };
        assert_eq!(layer.mode(), Some("claude"));
        assert_eq!(layer.scope(), None);
        assert_eq!(layer.project(), Some("myapp"));
    }

    #[test]
    fn test_helper_methods_scope_base() {
        let layer = Layer::ScopeBase {
            scope: "python".to_string(),
        };
        assert_eq!(layer.mode(), None);
        assert_eq!(layer.scope(), Some("python"));
        assert_eq!(layer.project(), None);
    }

    #[test]
    fn test_helper_methods_project_base() {
        let layer = Layer::ProjectBase {
            project: "myapp".to_string(),
        };
        assert_eq!(layer.mode(), None);
        assert_eq!(layer.scope(), None);
        assert_eq!(layer.project(), Some("myapp"));
    }

    #[test]
    fn test_helper_methods_global_base() {
        let layer = Layer::GlobalBase;
        assert_eq!(layer.mode(), None);
        assert_eq!(layer.scope(), None);
        assert_eq!(layer.project(), None);
    }

    #[test]
    fn test_helper_methods_user_local() {
        let layer = Layer::UserLocal;
        assert_eq!(layer.mode(), None);
        assert_eq!(layer.scope(), None);
        assert_eq!(layer.project(), None);
    }

    #[test]
    fn test_helper_methods_workspace_active() {
        let layer = Layer::WorkspaceActive;
        assert_eq!(layer.mode(), None);
        assert_eq!(layer.scope(), None);
        assert_eq!(layer.project(), None);
    }

    // ===== is_versioned Tests =====

    #[test]
    fn test_is_versioned_global_base() {
        assert!(Layer::GlobalBase.is_versioned());
    }

    #[test]
    fn test_is_versioned_mode_layers() {
        assert!(Layer::ModeBase { mode: "x".into() }.is_versioned());
        assert!(Layer::ModeScope {
            mode: "x".into(),
            scope: "y".into()
        }
        .is_versioned());
        assert!(Layer::ModeScopeProject {
            mode: "x".into(),
            scope: "y".into(),
            project: "z".into()
        }
        .is_versioned());
        assert!(Layer::ModeProject {
            mode: "x".into(),
            project: "y".into()
        }
        .is_versioned());
    }

    #[test]
    fn test_is_versioned_untethered_layers() {
        assert!(Layer::ScopeBase { scope: "x".into() }.is_versioned());
        assert!(Layer::ProjectBase {
            project: "x".into()
        }
        .is_versioned());
    }

    #[test]
    fn test_is_versioned_local_layers() {
        assert!(!Layer::UserLocal.is_versioned());
        assert!(!Layer::WorkspaceActive.is_versioned());
    }

    // ===== Constant Tests =====

    #[test]
    fn test_constants() {
        assert_eq!(Layer::LAYER_COUNT, 9);
        assert_eq!(Layer::JIN_ROOT, "jin");
        assert_eq!(Layer::GLOBAL_BASE_PATH, "jin/global");
        assert_eq!(Layer::USER_LOCAL_PATH, "~/.jin/local");
        assert_eq!(Layer::WORKSPACE_PATH, ".jin/workspace");
        assert_eq!(Layer::GIT_REF_PREFIX, "refs/jin/layers");
    }

    // ===== Display Tests =====

    #[test]
    fn test_display_global_base() {
        assert_eq!(Layer::GlobalBase.to_string(), "global");
    }

    #[test]
    fn test_display_mode_base() {
        let layer = Layer::ModeBase {
            mode: "claude".to_string(),
        };
        assert_eq!(layer.to_string(), "mode/claude");
    }

    #[test]
    fn test_display_mode_scope() {
        let layer = Layer::ModeScope {
            mode: "claude".to_string(),
            scope: "javascript".to_string(),
        };
        assert_eq!(layer.to_string(), "mode/claude/scope/javascript");
    }

    #[test]
    fn test_display_mode_scope_project() {
        let layer = Layer::ModeScopeProject {
            mode: "claude".to_string(),
            scope: "javascript".to_string(),
            project: "ui-dashboard".to_string(),
        };
        assert_eq!(
            layer.to_string(),
            "mode/claude/scope/javascript/project/ui-dashboard"
        );
    }

    #[test]
    fn test_display_mode_project() {
        let layer = Layer::ModeProject {
            mode: "claude".to_string(),
            project: "ui-dashboard".to_string(),
        };
        assert_eq!(layer.to_string(), "mode/claude/project/ui-dashboard");
    }

    #[test]
    fn test_display_scope_base() {
        let layer = Layer::ScopeBase {
            scope: "javascript".to_string(),
        };
        assert_eq!(layer.to_string(), "scope/javascript");
    }

    #[test]
    fn test_display_project_base() {
        let layer = Layer::ProjectBase {
            project: "myproject".to_string(),
        };
        assert_eq!(layer.to_string(), "project/myproject");
    }

    #[test]
    fn test_display_user_local() {
        assert_eq!(Layer::UserLocal.to_string(), "user-local");
    }

    #[test]
    fn test_display_workspace_active() {
        assert_eq!(Layer::WorkspaceActive.to_string(), "workspace-active");
    }

    // ===== Clone and Equality Tests =====

    #[test]
    fn test_layer_clone() {
        let layer = Layer::ModeScopeProject {
            mode: "claude".to_string(),
            scope: "python".to_string(),
            project: "myapp".to_string(),
        };
        let cloned = layer.clone();
        assert_eq!(layer, cloned);
    }

    #[test]
    fn test_layer_equality() {
        let layer1 = Layer::ModeBase {
            mode: "claude".to_string(),
        };
        let layer2 = Layer::ModeBase {
            mode: "claude".to_string(),
        };
        assert_eq!(layer1, layer2);

        let layer3 = Layer::ModeBase {
            mode: "cursor".to_string(),
        };
        assert_ne!(layer1, layer3);
    }

    #[test]
    fn test_layer_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Layer::GlobalBase);
        set.insert(Layer::ModeBase {
            mode: "claude".to_string(),
        });
        set.insert(Layer::GlobalBase); // Duplicate, won't be added again
        assert_eq!(set.len(), 2);
    }
}
