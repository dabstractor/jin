//! Layer merge orchestration

use crate::core::{Layer, Result};
use std::path::PathBuf;

/// Configuration for a layer merge operation
#[derive(Debug)]
pub struct LayerMergeConfig {
    /// Layers to merge, in precedence order (lowest first)
    pub layers: Vec<Layer>,
    /// Active mode, if any
    pub mode: Option<String>,
    /// Active scope, if any
    pub scope: Option<String>,
    /// Project name
    pub project: Option<String>,
}

/// Result of a layer merge operation
#[derive(Debug)]
pub struct LayerMergeResult {
    /// Files that were merged successfully
    pub merged_files: Vec<PathBuf>,
    /// Files that have conflicts
    pub conflict_files: Vec<PathBuf>,
    /// Files that were added (only in higher layer)
    pub added_files: Vec<PathBuf>,
    /// Files that were removed (deleted in higher layer)
    pub removed_files: Vec<PathBuf>,
}

impl Default for LayerMergeResult {
    fn default() -> Self {
        Self::new()
    }
}

impl LayerMergeResult {
    /// Create a new empty merge result
    pub fn new() -> Self {
        Self {
            merged_files: Vec::new(),
            conflict_files: Vec::new(),
            added_files: Vec::new(),
            removed_files: Vec::new(),
        }
    }

    /// Check if the merge was clean (no conflicts)
    pub fn is_clean(&self) -> bool {
        self.conflict_files.is_empty()
    }
}

/// Merge multiple layers into the workspace
///
/// TODO: Implement proper layer merging in later milestone
///
/// # Arguments
/// * `config` - The merge configuration
///
/// # Returns
/// * `LayerMergeResult` with the merge outcome
pub fn merge_layers(_config: &LayerMergeConfig) -> Result<LayerMergeResult> {
    // TODO: Implement layer merging
    // 1. For each layer in precedence order:
    //    a. Get the tree of files for that layer
    //    b. For each file, merge with the accumulated result
    // 2. Write merged files to workspace
    // 3. Return merge result

    Ok(LayerMergeResult::new())
}

/// Get the list of layers that apply given the current context
pub fn get_applicable_layers(
    mode: Option<&str>,
    scope: Option<&str>,
    _project: Option<&str>,
) -> Vec<Layer> {
    let mut layers = vec![Layer::GlobalBase];

    if let Some(_mode) = mode {
        layers.push(Layer::ModeBase);

        if let Some(_scope) = scope {
            layers.push(Layer::ModeScope);
            layers.push(Layer::ModeScopeProject);
        }

        layers.push(Layer::ModeProject);
    }

    if scope.is_some() {
        layers.push(Layer::ScopeBase);
    }

    layers.push(Layer::ProjectBase);
    layers.push(Layer::UserLocal);
    layers.push(Layer::WorkspaceActive);

    layers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_result_new() {
        let result = LayerMergeResult::new();
        assert!(result.is_clean());
        assert!(result.merged_files.is_empty());
    }

    #[test]
    fn test_applicable_layers_no_context() {
        let layers = get_applicable_layers(None, None, None);
        assert!(layers.contains(&Layer::GlobalBase));
        assert!(layers.contains(&Layer::ProjectBase));
        assert!(layers.contains(&Layer::UserLocal));
        assert!(!layers.contains(&Layer::ModeBase));
    }

    #[test]
    fn test_applicable_layers_with_mode() {
        let layers = get_applicable_layers(Some("claude"), None, None);
        assert!(layers.contains(&Layer::GlobalBase));
        assert!(layers.contains(&Layer::ModeBase));
        assert!(layers.contains(&Layer::ModeProject));
        assert!(!layers.contains(&Layer::ModeScope));
    }

    #[test]
    fn test_applicable_layers_with_mode_and_scope() {
        let layers = get_applicable_layers(Some("claude"), Some("language:javascript"), None);
        assert!(layers.contains(&Layer::GlobalBase));
        assert!(layers.contains(&Layer::ModeBase));
        assert!(layers.contains(&Layer::ModeScope));
        assert!(layers.contains(&Layer::ModeScopeProject));
        assert!(layers.contains(&Layer::ScopeBase));
    }
}
