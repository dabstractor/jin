//! Layer routing logic for `jin add`

use crate::core::{JinError, Layer, ProjectContext, Result};

/// Options for routing a file to a layer
#[derive(Debug, Default)]
pub struct RoutingOptions {
    /// Target mode layer
    pub mode: bool,
    /// Target scope
    pub scope: Option<String>,
    /// Target project layer
    pub project: bool,
    /// Target global layer
    pub global: bool,
    /// Target user-local layer (Layer 8)
    pub local: bool,
}

/// Determine the target layer for a file based on routing options
///
/// See PRD Section 9.1 for the complete routing table:
/// | Command                                           | Target Layer               |
/// | ------------------------------------------------- | -------------------------- |
/// | `jin add <file>`                                  | Project Base (7)           |
/// | `jin add <file> --mode`                           | Mode Base (2)              |
/// | `jin add <file> --mode --project`                 | Mode → Project (5)         |
/// | `jin add <file> --scope=<scope>`                  | Scope Base (6)             |
/// | `jin add <file> --mode --scope=<scope>`           | Mode → Scope (3)           |
/// | `jin add <file> --mode --scope=<scope> --project` | Mode → Scope → Project (4) |
pub fn route_to_layer(options: &RoutingOptions, context: &ProjectContext) -> Result<Layer> {
    // Global flag takes precedence
    if options.global {
        return Ok(Layer::GlobalBase);
    }

    // Local flag routes to UserLocal layer
    if options.local {
        return Ok(Layer::UserLocal);
    }

    // Check mode flag
    if options.mode {
        // Require active mode
        context.require_mode()?;

        if let Some(ref _scope) = options.scope {
            // Mode + Scope
            if options.project {
                // Mode + Scope + Project
                Ok(Layer::ModeScopeProject)
            } else {
                // Mode + Scope
                Ok(Layer::ModeScope)
            }
        } else if options.project {
            // Mode + Project
            Ok(Layer::ModeProject)
        } else {
            // Mode only
            Ok(Layer::ModeBase)
        }
    } else if let Some(ref _scope) = options.scope {
        // Scope without mode (untethered scope)
        Ok(Layer::ScopeBase)
    } else {
        // Default: Project Base
        Ok(Layer::ProjectBase)
    }
}

/// Validate routing options for consistency
pub fn validate_routing_options(options: &RoutingOptions) -> Result<()> {
    // Can't use both --global and other layer flags
    if options.global && (options.mode || options.scope.is_some() || options.project) {
        return Err(JinError::Config(
            "Cannot combine --global with other layer flags".to_string(),
        ));
    }

    // Can't use --local with other layer flags
    if options.local && (options.mode || options.scope.is_some() || options.project || options.global) {
        return Err(JinError::Config(
            "Cannot combine --local with other layer flags".to_string(),
        ));
    }

    // Can't use --project without --mode
    if options.project && !options.mode {
        return Err(JinError::Config(
            "--project requires --mode flag".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context_with_mode() -> ProjectContext {
        ProjectContext {
            mode: Some("claude".to_string()),
            ..Default::default()
        }
    }

    fn context_with_mode_and_scope() -> ProjectContext {
        ProjectContext {
            mode: Some("claude".to_string()),
            scope: Some("language:javascript".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_route_default() {
        let options = RoutingOptions::default();
        let context = ProjectContext::default();
        let layer = route_to_layer(&options, &context).unwrap();
        assert_eq!(layer, Layer::ProjectBase);
    }

    #[test]
    fn test_route_global() {
        let options = RoutingOptions {
            global: true,
            ..Default::default()
        };
        let context = ProjectContext::default();
        let layer = route_to_layer(&options, &context).unwrap();
        assert_eq!(layer, Layer::GlobalBase);
    }

    #[test]
    fn test_route_local() {
        let options = RoutingOptions {
            local: true,
            ..Default::default()
        };
        let context = ProjectContext::default();
        let layer = route_to_layer(&options, &context).unwrap();
        assert_eq!(layer, Layer::UserLocal);
    }

    #[test]
    fn test_route_mode() {
        let options = RoutingOptions {
            mode: true,
            ..Default::default()
        };
        let context = context_with_mode();
        let layer = route_to_layer(&options, &context).unwrap();
        assert_eq!(layer, Layer::ModeBase);
    }

    #[test]
    fn test_route_mode_without_active_mode_fails() {
        let options = RoutingOptions {
            mode: true,
            ..Default::default()
        };
        let context = ProjectContext::default();
        let result = route_to_layer(&options, &context);
        assert!(result.is_err());
    }

    #[test]
    fn test_route_mode_project() {
        let options = RoutingOptions {
            mode: true,
            project: true,
            ..Default::default()
        };
        let context = context_with_mode();
        let layer = route_to_layer(&options, &context).unwrap();
        assert_eq!(layer, Layer::ModeProject);
    }

    #[test]
    fn test_route_mode_scope() {
        let options = RoutingOptions {
            mode: true,
            scope: Some("language:javascript".to_string()),
            ..Default::default()
        };
        let context = context_with_mode_and_scope();
        let layer = route_to_layer(&options, &context).unwrap();
        assert_eq!(layer, Layer::ModeScope);
    }

    #[test]
    fn test_route_mode_scope_project() {
        let options = RoutingOptions {
            mode: true,
            scope: Some("language:javascript".to_string()),
            project: true,
            ..Default::default()
        };
        let context = context_with_mode_and_scope();
        let layer = route_to_layer(&options, &context).unwrap();
        assert_eq!(layer, Layer::ModeScopeProject);
    }

    #[test]
    fn test_route_scope_untethered() {
        let options = RoutingOptions {
            scope: Some("language:javascript".to_string()),
            ..Default::default()
        };
        let context = ProjectContext::default();
        let layer = route_to_layer(&options, &context).unwrap();
        assert_eq!(layer, Layer::ScopeBase);
    }

    #[test]
    fn test_validate_global_conflict() {
        let options = RoutingOptions {
            global: true,
            mode: true,
            ..Default::default()
        };
        let result = validate_routing_options(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_project_without_mode() {
        let options = RoutingOptions {
            project: true,
            ..Default::default()
        };
        let result = validate_routing_options(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_local_conflict_with_mode() {
        let options = RoutingOptions {
            local: true,
            mode: true,
            ..Default::default()
        };
        let result = validate_routing_options(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_local_conflict_with_scope() {
        let options = RoutingOptions {
            local: true,
            scope: Some("language:javascript".to_string()),
            ..Default::default()
        };
        let result = validate_routing_options(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_local_conflict_with_project() {
        let options = RoutingOptions {
            local: true,
            project: true,
            ..Default::default()
        };
        let result = validate_routing_options(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_local_conflict_with_global() {
        let options = RoutingOptions {
            local: true,
            global: true,
            ..Default::default()
        };
        let result = validate_routing_options(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_local_alone_passes() {
        let options = RoutingOptions {
            local: true,
            ..Default::default()
        };
        let result = validate_routing_options(&options);
        assert!(result.is_ok());
    }
}
