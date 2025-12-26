//! Layer router for determining target layer from CLI flags.
//!
//! This module implements the layer routing logic that maps combinations
//! of CLI flags (`--mode`, `--scope`, `--project`, `--global`) to target
//! layers in Jin's 9-layer hierarchy.
//!
//! The routing logic delegates to `Layer::from_flags()` to ensure
//! consistency across the codebase.

use crate::core::{
    error::{JinError, Result},
    Layer,
};

/// Layer router for determining target layer from flags.
///
/// The router implements PRD §9.1 routing table, mapping CLI flag
/// combinations to appropriate target layers. It wraps the existing
/// `Layer::from_flags()` function to provide error handling and
/// project context.
///
/// # Routing Table
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
/// | (none) | Error (no routing target) |
///
/// # Examples
///
/// ```ignore
/// use jin_glm::staging::LayerRouter;
///
/// let router = LayerRouter::new("myapp".to_string());
///
/// // Route to global
/// let layer = router.route(None, None, false, true)?;
/// assert!(matches!(layer, Layer::GlobalBase));
///
/// // Route to mode base
/// let layer = router.route(Some("claude"), None, false, false)?;
/// assert!(matches!(layer, Layer::ModeBase { .. }));
/// ```
pub struct LayerRouter {
    /// Project name (for default/project inference)
    project: String,
}

impl LayerRouter {
    /// Creates a new layer router with the given project name.
    ///
    /// # Arguments
    ///
    /// * `project` - Project name for routing
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let router = LayerRouter::new("myapp".to_string());
    /// ```
    pub fn new(project: String) -> Self {
        Self { project }
    }

    /// Routes CLI flags to the appropriate layer.
    ///
    /// This method implements the routing table from PRD §9.1 by delegating
    /// to `Layer::from_flags()`. Returns an error if no flags are provided.
    ///
    /// # Arguments
    ///
    /// * `mode` - Optional mode name from `--mode` flag
    /// * `scope` - Optional scope name from `--scope` flag
    /// * `project_flag` - Project flag from `--project` (boolean)
    /// * `global` - Global flag from `--global` (boolean)
    ///
    /// # Returns
    ///
    /// Returns `Ok(Layer)` with the routed layer, or `Err(JinError::Message)`
    /// if no routing target is available.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let router = LayerRouter::new("myapp".to_string());
    ///
    /// // Global flag
    /// let layer = router.route(None, None, false, true)?;
    /// assert!(matches!(layer, Layer::GlobalBase));
    ///
    /// // Full hierarchy
    /// let layer = router.route(Some("claude"), Some("python"), true, false)?;
    /// assert!(matches!(layer, Layer::ModeScopeProject { .. }));
    ///
    /// // No flags - error
    /// let result = router.route(None, None, false, false);
    /// assert!(result.is_err());
    /// ```
    pub fn route(
        &self,
        mode: Option<&str>,
        scope: Option<&str>,
        project_flag: bool,
        global: bool,
    ) -> Result<Layer> {
        let project = if project_flag { Some(self.project.as_str()) } else { None };
        Layer::from_flags(mode, scope, project, global).ok_or_else(|| {
            JinError::Message("No routing target (use --mode, --scope, or --project)".to_string())
        })
    }

    /// Returns the project name used by this router.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let router = LayerRouter::new("myapp".to_string());
    /// assert_eq!(router.project(), "myapp");
    /// ```
    pub fn project(&self) -> &str {
        &self.project
    }
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Constructor Tests =====

    #[test]
    fn test_router_new() {
        let router = LayerRouter::new("myapp".to_string());
        assert_eq!(router.project(), "myapp");
    }

    // ===== Routing Tests =====

    #[test]
    fn test_router_global_flag() {
        let router = LayerRouter::new("myapp".to_string());
        let layer = router.route(None, None, false, true).unwrap();
        assert!(matches!(layer, Layer::GlobalBase));
    }

    #[test]
    fn test_router_global_with_other_flags() {
        let router = LayerRouter::new("myapp".to_string());
        // Global should take precedence even with other flags
        let layer = router
            .route(Some("claude"), Some("python"), true, true)
            .unwrap();
        assert!(matches!(layer, Layer::GlobalBase));
    }

    #[test]
    fn test_router_mode_only() {
        let router = LayerRouter::new("myapp".to_string());
        let layer = router.route(Some("claude"), None, false, false).unwrap();
        assert_eq!(
            layer,
            Layer::ModeBase {
                mode: "claude".to_string()
            }
        );
    }

    #[test]
    fn test_router_scope_only() {
        let router = LayerRouter::new("myapp".to_string());
        let layer = router.route(None, Some("python"), false, false).unwrap();
        assert_eq!(
            layer,
            Layer::ScopeBase {
                scope: "python".to_string()
            }
        );
    }

    #[test]
    fn test_router_project_only() {
        let router = LayerRouter::new("myapp".to_string());
        let layer = router.route(None, None, true, false).unwrap();
        assert_eq!(
            layer,
            Layer::ProjectBase {
                project: "myapp".to_string()
            }
        );
    }

    #[test]
    fn test_router_mode_and_scope() {
        let router = LayerRouter::new("myapp".to_string());
        let layer = router
            .route(Some("claude"), Some("python"), false, false)
            .unwrap();
        assert_eq!(
            layer,
            Layer::ModeScope {
                mode: "claude".to_string(),
                scope: "python".to_string()
            }
        );
    }

    #[test]
    fn test_router_mode_and_project() {
        let router = LayerRouter::new("myapp".to_string());
        let layer = router.route(Some("claude"), None, true, false).unwrap();
        assert_eq!(
            layer,
            Layer::ModeProject {
                mode: "claude".to_string(),
                project: "myapp".to_string()
            }
        );
    }

    #[test]
    fn test_router_full_hierarchy() {
        let router = LayerRouter::new("myapp".to_string());
        let layer = router
            .route(Some("claude"), Some("python"), true, false)
            .unwrap();
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
    fn test_router_no_flags_errors() {
        let router = LayerRouter::new("myapp".to_string());
        let result = router.route(None, None, false, false);
        assert!(result.is_err());
        if let Err(JinError::Message(msg)) = result {
            assert!(msg.contains("No routing target"));
        } else {
            panic!("Expected JinError::Message");
        }
    }

    #[test]
    fn test_router_scope_and_project_no_mode() {
        let router = LayerRouter::new("myapp".to_string());
        // scope + project without mode routes to ScopeBase
        let layer = router.route(None, Some("python"), true, false).unwrap();
        assert_eq!(
            layer,
            Layer::ScopeBase {
                scope: "python".to_string()
            }
        );
    }

    // ===== PRD Routing Table Compliance Tests =====

    #[test]
    fn test_prd_routing_table_add_only() {
        let router = LayerRouter::new("myapp".to_string());
        // "jin add <file>" -> Project Base
        let layer = router.route(None, None, true, false).unwrap();
        assert!(matches!(layer, Layer::ProjectBase { .. }));
    }

    #[test]
    fn test_prd_routing_table_add_mode() {
        let router = LayerRouter::new("myapp".to_string());
        // "jin add <file> --mode" -> Mode Base
        let layer = router.route(Some("claude"), None, false, false).unwrap();
        assert!(matches!(layer, Layer::ModeBase { .. }));
    }

    #[test]
    fn test_prd_routing_table_add_mode_project() {
        let router = LayerRouter::new("myapp".to_string());
        // "jin add <file> --mode --project" -> Mode -> Project
        let layer = router.route(Some("claude"), None, true, false).unwrap();
        assert!(matches!(layer, Layer::ModeProject { .. }));
    }

    #[test]
    fn test_prd_routing_table_add_scope() {
        let router = LayerRouter::new("myapp".to_string());
        // "jin add <file> --scope" -> Scope Base
        let layer = router.route(None, Some("python"), false, false).unwrap();
        assert!(matches!(layer, Layer::ScopeBase { .. }));
    }

    #[test]
    fn test_prd_routing_table_add_mode_scope() {
        let router = LayerRouter::new("myapp".to_string());
        // "jin add <file> --mode --scope" -> Mode -> Scope
        let layer = router
            .route(Some("claude"), Some("python"), false, false)
            .unwrap();
        assert!(matches!(layer, Layer::ModeScope { .. }));
    }

    #[test]
    fn test_prd_routing_table_add_mode_scope_project() {
        let router = LayerRouter::new("myapp".to_string());
        // "jin add <file> --mode --scope --project" -> Mode -> Scope -> Project
        let layer = router
            .route(Some("claude"), Some("python"), true, false)
            .unwrap();
        assert!(matches!(layer, Layer::ModeScopeProject { .. }));
    }
}
