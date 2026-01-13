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
    /// Layer 2: Mode defaults (jin/mode/`<mode>/`)
    ModeBase,
    /// Layer 3: Scoped mode configs (jin/mode/`<mode>`/scope/`<scope>/`)
    ModeScope,
    /// Layer 4: Project overrides for scoped mode
    ModeScopeProject,
    /// Layer 5: Project overrides for mode (jin/mode/`<mode>`/project/`<project>/`)
    ModeProject,
    /// Layer 6: Untethered scope configs (jin/scope/`<scope>/`)
    ScopeBase,
    /// Layer 7: Project-only configs (jin/project/`<project>/`)
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
    ///
    /// Note: Layers that can have child refs use `/_` suffix to avoid Git ref naming conflicts.
    /// Git refs are files, so a ref can't exist at a path that has children.
    /// For example, `refs/jin/layers/mode/claude` can't exist as a file if
    /// `refs/jin/layers/mode/claude/project/foo` exists (which requires `claude` to be a directory).
    /// The `/_` suffix solves this: `refs/jin/layers/mode/claude/_` can coexist with
    /// `refs/jin/layers/mode/claude/project/foo`.
    pub fn ref_path(
        &self,
        mode: Option<&str>,
        scope: Option<&str>,
        project: Option<&str>,
    ) -> String {
        // Sanitize scope name: replace colons with slashes for Git ref compatibility
        // This matches the behavior of scope creation in src/commands/scope.rs
        let scope_sanitized = scope.map(|s| s.replace(':', "/"));
        let scope_ref = scope_sanitized.as_deref().or(scope).unwrap_or("default");

        match self {
            Layer::GlobalBase => "refs/jin/layers/global".to_string(),
            // ModeBase uses /_ suffix because ModeScope, ModeScopeProject, and ModeProject
            // create refs under the mode directory
            Layer::ModeBase => {
                format!("refs/jin/layers/mode/{}/_", mode.unwrap_or("default"))
            }
            // ModeScope uses /_ suffix because ModeScopeProject creates refs under it
            Layer::ModeScope => format!(
                "refs/jin/layers/mode/{}/scope/{}/_",
                mode.unwrap_or("default"),
                scope_ref
            ),
            Layer::ModeScopeProject => format!(
                "refs/jin/layers/mode/{}/scope/{}/project/{}",
                mode.unwrap_or("default"),
                scope_ref,
                project.unwrap_or("default")
            ),
            Layer::ModeProject => format!(
                "refs/jin/layers/mode/{}/project/{}",
                mode.unwrap_or("default"),
                project.unwrap_or("default")
            ),
            Layer::ScopeBase => {
                format!("refs/jin/layers/scope/{}", scope_ref)
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

    /// Parse a layer from a Git ref path.
    ///
    /// Returns `Some(Layer)` if the ref path matches a known layer pattern,
    /// or `None` if the path is invalid or doesn't match any layer.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// assert_eq!(Layer::parse_layer_from_ref_path("refs/jin/layers/global"), Some(Layer::GlobalBase));
    /// assert_eq!(Layer::parse_layer_from_ref_path("refs/jin/layers/mode/development/_"), Some(Layer::ModeBase));
    /// assert_eq!(Layer::parse_layer_from_ref_path("refs/jin/layers/mode/production/scope/api/_"), Some(Layer::ModeScope));
    /// assert_eq!(Layer::parse_layer_from_ref_path("invalid/path"), None);
    /// ```
    pub fn parse_layer_from_ref_path(ref_path: &str) -> Option<Layer> {
        // Split the entire ref path and filter empty segments first
        // This handles extra slashes like "refs/jin/layers//global" correctly
        let all_parts: Vec<&str> = ref_path.split('/').filter(|s| !s.is_empty()).collect();

        // Check if we have enough parts and the prefix is correct
        // Prefix is: refs, jin, layers
        if all_parts.len() < 4 {
            return None;
        }
        if all_parts[0] != "refs" || all_parts[1] != "jin" || all_parts[2] != "layers" {
            return None;
        }

        // Extract the parts after "refs/jin/layers/"
        let parts = &all_parts[3..];

        // Match on slice patterns
        // CRITICAL: More specific patterns must come before less specific ones
        // CRITICAL: "_" in patterns is a literal string (the suffix), not a wildcard
        match parts {
            // Most specific: 6 segments for ModeScopeProject
            ["mode", _, "scope", _, "project", _] => Some(Layer::ModeScopeProject),
            // 5 segments for ModeScope with /_ suffix
            ["mode", _, "scope", _, "_"] => Some(Layer::ModeScope),
            // 4 segments for ModeProject
            ["mode", _, "project", _] => Some(Layer::ModeProject),
            // 3 segments for ModeBase with /_ suffix
            ["mode", _, "_"] => Some(Layer::ModeBase),
            // 2 segments with wildcard
            ["scope", _] => Some(Layer::ScopeBase),
            ["project", _] => Some(Layer::ProjectBase),
            // Single segments
            ["global"] => Some(Layer::GlobalBase),
            ["local"] => Some(Layer::UserLocal),
            ["workspace"] => Some(Layer::WorkspaceActive),
            // Unknown pattern
            _ => None,
        }
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
        // ModeBase uses /_ suffix to avoid conflict with child refs
        assert_eq!(
            Layer::ModeBase.ref_path(Some("claude"), None, None),
            "refs/jin/layers/mode/claude/_"
        );
        // ModeScope uses /_ suffix to avoid conflict with ModeScopeProject refs
        // Note: colons in scope names are sanitized to slashes for Git ref compatibility
        assert_eq!(
            Layer::ModeScope.ref_path(Some("claude"), Some("language:javascript"), None),
            "refs/jin/layers/mode/claude/scope/language/javascript/_"
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

    // Tests for parse_layer_from_ref_path()

    #[test]
    fn test_parse_layer_from_ref_path_global_base() {
        let result = Layer::parse_layer_from_ref_path("refs/jin/layers/global");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), Layer::GlobalBase);
    }

    #[test]
    fn test_parse_layer_from_ref_path_mode_base() {
        let result = Layer::parse_layer_from_ref_path("refs/jin/layers/mode/development/_");
        assert_eq!(result, Some(Layer::ModeBase));

        // Test with different mode names
        assert_eq!(
            Layer::parse_layer_from_ref_path("refs/jin/layers/mode/production/_"),
            Some(Layer::ModeBase)
        );
        assert_eq!(
            Layer::parse_layer_from_ref_path("refs/jin/layers/mode/claude/_"),
            Some(Layer::ModeBase)
        );
    }

    #[test]
    fn test_parse_layer_from_ref_path_mode_scope() {
        let result =
            Layer::parse_layer_from_ref_path("refs/jin/layers/mode/production/scope/api/_");
        assert_eq!(result, Some(Layer::ModeScope));

        // Test with scope name containing colon (replaces slash in ref path)
        assert_eq!(
            Layer::parse_layer_from_ref_path(
                "refs/jin/layers/mode/dev/scope/language:javascript/_"
            ),
            Some(Layer::ModeScope)
        );
    }

    #[test]
    fn test_parse_layer_from_ref_path_mode_scope_project() {
        let result =
            Layer::parse_layer_from_ref_path("refs/jin/layers/mode/dev/scope/api/project/ui");
        assert_eq!(result, Some(Layer::ModeScopeProject));

        // More complex example
        assert_eq!(
            Layer::parse_layer_from_ref_path(
                "refs/jin/layers/mode/production/scope/config/project/backend"
            ),
            Some(Layer::ModeScopeProject)
        );
    }

    #[test]
    fn test_parse_layer_from_ref_path_mode_project() {
        let result = Layer::parse_layer_from_ref_path("refs/jin/layers/mode/dev/project/backend");
        assert_eq!(result, Some(Layer::ModeProject));
    }

    #[test]
    fn test_parse_layer_from_ref_path_scope_base() {
        let result = Layer::parse_layer_from_ref_path("refs/jin/layers/scope/config");
        assert_eq!(result, Some(Layer::ScopeBase));
    }

    #[test]
    fn test_parse_layer_from_ref_path_project_base() {
        let result = Layer::parse_layer_from_ref_path("refs/jin/layers/project/api-server");
        assert_eq!(result, Some(Layer::ProjectBase));
    }

    #[test]
    fn test_parse_layer_from_ref_path_user_local() {
        let result = Layer::parse_layer_from_ref_path("refs/jin/layers/local");
        assert_eq!(result, Some(Layer::UserLocal));
    }

    #[test]
    fn test_parse_layer_from_ref_path_workspace_active() {
        let result = Layer::parse_layer_from_ref_path("refs/jin/layers/workspace");
        assert_eq!(result, Some(Layer::WorkspaceActive));
    }

    // Negative cases - invalid inputs

    #[test]
    fn test_parse_layer_from_ref_path_empty_string() {
        let result = Layer::parse_layer_from_ref_path("");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_layer_from_ref_path_wrong_prefix() {
        assert_eq!(Layer::parse_layer_from_ref_path("refs/other/global"), None);
        assert_eq!(Layer::parse_layer_from_ref_path("jin/layers/global"), None);
        assert_eq!(
            Layer::parse_layer_from_ref_path("refs/git/layers/global"),
            None
        );
    }

    #[test]
    fn test_parse_layer_from_ref_path_partial_match() {
        // Just the prefix, no layer name
        assert_eq!(Layer::parse_layer_from_ref_path("refs/jin/layers"), None);

        // Partial mode path without /_ suffix
        assert_eq!(
            Layer::parse_layer_from_ref_path("refs/jin/layers/mode"),
            None
        );

        assert_eq!(
            Layer::parse_layer_from_ref_path("refs/jin/layers/mode/development"),
            None
        );
    }

    #[test]
    fn test_parse_layer_from_ref_path_invalid_suffix() {
        // ModeBase without /_ suffix should not match
        assert_eq!(
            Layer::parse_layer_from_ref_path("refs/jin/layers/mode/development"),
            None
        );

        // ModeScope without /_ suffix should not match
        assert_eq!(
            Layer::parse_layer_from_ref_path("refs/jin/layers/mode/dev/scope/api"),
            None
        );
    }

    // Edge cases

    #[test]
    fn test_parse_layer_from_ref_path_with_underscore_in_name() {
        // Mode names can have underscores
        assert_eq!(
            Layer::parse_layer_from_ref_path("refs/jin/layers/mode/dev_env/_"),
            Some(Layer::ModeBase)
        );

        // Scope names can have underscores
        assert_eq!(
            Layer::parse_layer_from_ref_path("refs/jin/layers/mode/dev/scope/api_test/_"),
            Some(Layer::ModeScope)
        );

        // Project names can have underscores
        assert_eq!(
            Layer::parse_layer_from_ref_path("refs/jin/layers/mode/dev/project/test_ui"),
            Some(Layer::ModeProject)
        );
    }

    #[test]
    fn test_parse_layer_from_ref_path_extra_slashes() {
        // Double slashes should be filtered out
        assert_eq!(
            Layer::parse_layer_from_ref_path("refs/jin/layers//global"),
            Some(Layer::GlobalBase)
        );

        assert_eq!(
            Layer::parse_layer_from_ref_path("refs/jin//layers/global"),
            Some(Layer::GlobalBase)
        );

        // Triple slashes
        assert_eq!(
            Layer::parse_layer_from_ref_path("refs///jin///layers///global"),
            Some(Layer::GlobalBase)
        );
    }

    #[test]
    fn test_parse_layer_from_ref_path_trailing_slash() {
        // Trailing slash creates empty segment that gets filtered
        assert_eq!(
            Layer::parse_layer_from_ref_path("refs/jin/layers/global/"),
            Some(Layer::GlobalBase)
        );
    }

    #[test]
    fn test_parse_layer_from_ref_path_unknown_layer() {
        // Unknown layer type
        assert_eq!(
            Layer::parse_layer_from_ref_path("refs/jin/layers/unknown"),
            None
        );

        // Invalid mode structure
        assert_eq!(
            Layer::parse_layer_from_ref_path("refs/jin/layers/mode/dev/invalid"),
            None
        );
    }

    #[test]
    fn test_parse_layer_from_ref_path_pattern_order() {
        // Verify that more specific patterns match before less specific ones
        // ModeScopeProject (6 segments) should match, not ModeScope
        assert_eq!(
            Layer::parse_layer_from_ref_path("refs/jin/layers/mode/dev/scope/api/project/ui"),
            Some(Layer::ModeScopeProject)
        );
    }
}
