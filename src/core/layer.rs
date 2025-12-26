//! Layer hierarchy implementation for Jin
//!
//! The 9-layer hierarchy is Jin's core innovation for managing configuration
//! precedence. Precedence flows bottom (1) to top (9) - higher overrides lower.

use serde::{Deserialize, Serialize};

/// The 9-layer hierarchy for Jin configuration.
/// Precedence flows bottom (1) to top (9) - higher overrides lower.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    /// Layer 1: Shared defaults (jin/global/)
    GlobalBase,
    /// Layer 2: Mode defaults (jin/mode/<mode>/)
    ModeBase,
    /// Layer 3: Scoped mode configs (jin/mode/<mode>/scope/<scope>/)
    ModeScope,
    /// Layer 4: Project overrides for scoped mode
    ModeScopeProject,
    /// Layer 5: Project overrides for mode (jin/mode/<mode>/project/<project>/)
    ModeProject,
    /// Layer 6: Untethered scope configs (jin/scope/<scope>/)
    ScopeBase,
    /// Layer 7: Project-only configs (jin/project/<project>/)
    ProjectBase,
    /// Layer 8: Machine-only overlays (~/.jin/local/)
    UserLocal,
    /// Layer 9: Derived merge result (.jin/workspace/)
    WorkspaceActive,
}

impl Layer {
    /// Returns the precedence level (1-9, higher overrides lower)
    pub fn precedence(&self) -> u8 {
        match self {
            Layer::GlobalBase => 1,
            Layer::ModeBase => 2,
            Layer::ModeScope => 3,
            Layer::ModeScopeProject => 4,
            Layer::ModeProject => 5,
            Layer::ScopeBase => 6,
            Layer::ProjectBase => 7,
            Layer::UserLocal => 8,
            Layer::WorkspaceActive => 9,
        }
    }

    /// Returns the Git ref path for this layer
    pub fn ref_path(
        &self,
        mode: Option<&str>,
        scope: Option<&str>,
        project: Option<&str>,
    ) -> String {
        match self {
            Layer::GlobalBase => "refs/jin/layers/global".to_string(),
            Layer::ModeBase => {
                format!("refs/jin/layers/mode/{}", mode.unwrap_or("default"))
            }
            Layer::ModeScope => format!(
                "refs/jin/layers/mode/{}/scope/{}",
                mode.unwrap_or("default"),
                scope.unwrap_or("default")
            ),
            Layer::ModeScopeProject => format!(
                "refs/jin/layers/mode/{}/scope/{}/project/{}",
                mode.unwrap_or("default"),
                scope.unwrap_or("default"),
                project.unwrap_or("default")
            ),
            Layer::ModeProject => format!(
                "refs/jin/layers/mode/{}/project/{}",
                mode.unwrap_or("default"),
                project.unwrap_or("default")
            ),
            Layer::ScopeBase => {
                format!("refs/jin/layers/scope/{}", scope.unwrap_or("default"))
            }
            Layer::ProjectBase => {
                format!("refs/jin/layers/project/{}", project.unwrap_or("default"))
            }
            Layer::UserLocal => "refs/jin/layers/local".to_string(),
            Layer::WorkspaceActive => "refs/jin/layers/workspace".to_string(),
        }
    }

    /// Returns the storage directory path for this layer
    pub fn storage_path(
        &self,
        mode: Option<&str>,
        scope: Option<&str>,
        project: Option<&str>,
    ) -> String {
        match self {
            Layer::GlobalBase => "jin/global/".to_string(),
            Layer::ModeBase => format!("jin/mode/{}/", mode.unwrap_or("default")),
            Layer::ModeScope => format!(
                "jin/mode/{}/scope/{}/",
                mode.unwrap_or("default"),
                scope.unwrap_or("default")
            ),
            Layer::ModeScopeProject => format!(
                "jin/mode/{}/scope/{}/project/{}/",
                mode.unwrap_or("default"),
                scope.unwrap_or("default"),
                project.unwrap_or("default")
            ),
            Layer::ModeProject => format!(
                "jin/mode/{}/project/{}/",
                mode.unwrap_or("default"),
                project.unwrap_or("default")
            ),
            Layer::ScopeBase => format!("jin/scope/{}/", scope.unwrap_or("default")),
            Layer::ProjectBase => format!("jin/project/{}/", project.unwrap_or("default")),
            Layer::UserLocal => "~/.jin/local/".to_string(),
            Layer::WorkspaceActive => ".jin/workspace/".to_string(),
        }
    }

    /// Returns all layers in precedence order (lowest to highest)
    pub fn all_in_precedence_order() -> Vec<Layer> {
        vec![
            Layer::GlobalBase,
            Layer::ModeBase,
            Layer::ModeScope,
            Layer::ModeScopeProject,
            Layer::ModeProject,
            Layer::ScopeBase,
            Layer::ProjectBase,
            Layer::UserLocal,
            Layer::WorkspaceActive,
        ]
    }

    /// Returns true if this layer requires a mode to be active
    pub fn requires_mode(&self) -> bool {
        matches!(
            self,
            Layer::ModeBase | Layer::ModeScope | Layer::ModeScopeProject | Layer::ModeProject
        )
    }

    /// Returns true if this layer requires a scope to be specified
    pub fn requires_scope(&self) -> bool {
        matches!(
            self,
            Layer::ModeScope | Layer::ModeScopeProject | Layer::ScopeBase
        )
    }

    /// Returns true if this layer is project-specific
    pub fn is_project_specific(&self) -> bool {
        matches!(
            self,
            Layer::ModeScopeProject | Layer::ModeProject | Layer::ProjectBase
        )
    }
}

impl std::fmt::Display for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Layer::GlobalBase => "global-base",
            Layer::ModeBase => "mode-base",
            Layer::ModeScope => "mode-scope",
            Layer::ModeScopeProject => "mode-scope-project",
            Layer::ModeProject => "mode-project",
            Layer::ScopeBase => "scope-base",
            Layer::ProjectBase => "project-base",
            Layer::UserLocal => "user-local",
            Layer::WorkspaceActive => "workspace-active",
        };
        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precedence_ordering() {
        let layers = Layer::all_in_precedence_order();
        for i in 0..layers.len() - 1 {
            assert!(
                layers[i].precedence() < layers[i + 1].precedence(),
                "Layer {:?} should have lower precedence than {:?}",
                layers[i],
                layers[i + 1]
            );
        }
    }

    #[test]
    fn test_precedence_values() {
        assert_eq!(Layer::GlobalBase.precedence(), 1);
        assert_eq!(Layer::ModeBase.precedence(), 2);
        assert_eq!(Layer::ModeScope.precedence(), 3);
        assert_eq!(Layer::ModeScopeProject.precedence(), 4);
        assert_eq!(Layer::ModeProject.precedence(), 5);
        assert_eq!(Layer::ScopeBase.precedence(), 6);
        assert_eq!(Layer::ProjectBase.precedence(), 7);
        assert_eq!(Layer::UserLocal.precedence(), 8);
        assert_eq!(Layer::WorkspaceActive.precedence(), 9);
    }

    #[test]
    fn test_ref_paths() {
        assert_eq!(
            Layer::GlobalBase.ref_path(None, None, None),
            "refs/jin/layers/global"
        );
        assert_eq!(
            Layer::ModeBase.ref_path(Some("claude"), None, None),
            "refs/jin/layers/mode/claude"
        );
        assert_eq!(
            Layer::ModeScope.ref_path(Some("claude"), Some("language:javascript"), None),
            "refs/jin/layers/mode/claude/scope/language:javascript"
        );
        assert_eq!(
            Layer::ProjectBase.ref_path(None, None, Some("ui-dashboard")),
            "refs/jin/layers/project/ui-dashboard"
        );
    }

    #[test]
    fn test_storage_paths() {
        assert_eq!(
            Layer::GlobalBase.storage_path(None, None, None),
            "jin/global/"
        );
        assert_eq!(
            Layer::ModeBase.storage_path(Some("claude"), None, None),
            "jin/mode/claude/"
        );
        assert_eq!(
            Layer::ModeScope.storage_path(Some("claude"), Some("language:javascript"), None),
            "jin/mode/claude/scope/language:javascript/"
        );
        assert_eq!(
            Layer::UserLocal.storage_path(None, None, None),
            "~/.jin/local/"
        );
        assert_eq!(
            Layer::WorkspaceActive.storage_path(None, None, None),
            ".jin/workspace/"
        );
    }

    #[test]
    fn test_all_layers_count() {
        assert_eq!(Layer::all_in_precedence_order().len(), 9);
    }

    #[test]
    fn test_requires_mode() {
        assert!(!Layer::GlobalBase.requires_mode());
        assert!(Layer::ModeBase.requires_mode());
        assert!(Layer::ModeScope.requires_mode());
        assert!(Layer::ModeScopeProject.requires_mode());
        assert!(Layer::ModeProject.requires_mode());
        assert!(!Layer::ScopeBase.requires_mode());
        assert!(!Layer::ProjectBase.requires_mode());
    }

    #[test]
    fn test_requires_scope() {
        assert!(!Layer::GlobalBase.requires_scope());
        assert!(!Layer::ModeBase.requires_scope());
        assert!(Layer::ModeScope.requires_scope());
        assert!(Layer::ModeScopeProject.requires_scope());
        assert!(!Layer::ModeProject.requires_scope());
        assert!(Layer::ScopeBase.requires_scope());
    }

    #[test]
    fn test_display() {
        assert_eq!(Layer::GlobalBase.to_string(), "global-base");
        assert_eq!(Layer::WorkspaceActive.to_string(), "workspace-active");
    }
}
