//! Layer merge orchestration
//!
//! This module handles merging configuration files across Jin's 9-layer
//! hierarchy. Files at higher precedence layers override lower layers,
//! with structured files (JSON, YAML, TOML, INI) being deep-merged
//! according to RFC 7396 semantics.

use crate::core::{JinError, Layer, Result};
use crate::git::{JinRepo, RefOps, TreeOps};
use std::collections::HashSet;
use std::path::PathBuf;

use super::{deep_merge, MergeValue};

/// File format for parsing and serialization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    /// JSON format (.json)
    Json,
    /// YAML format (.yaml, .yml)
    Yaml,
    /// TOML format (.toml)
    Toml,
    /// INI format (.ini, .cfg, .conf)
    Ini,
    /// Plain text (any other extension)
    Text,
}

/// Represents a merged file across multiple layers
#[derive(Debug)]
pub struct MergedFile {
    /// Final merged content
    pub content: MergeValue,
    /// Layers that contributed to this file
    pub source_layers: Vec<Layer>,
    /// Original format (for serialization)
    pub format: FileFormat,
}

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
    /// Files that were merged successfully with their content
    pub merged_files: std::collections::HashMap<PathBuf, MergedFile>,
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
            merged_files: std::collections::HashMap::new(),
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

/// Merge all applicable layers for the given configuration.
///
/// This function collects all unique file paths across all layers,
/// then merges each file according to layer precedence (lowest first).
///
/// # Arguments
///
/// * `config` - The merge configuration containing layers and context
/// * `repo` - The Jin repository to read layer contents from
///
/// # Returns
///
/// * `LayerMergeResult` with merged files and their content, plus conflict/added/removed files
pub fn merge_layers(config: &LayerMergeConfig, repo: &JinRepo) -> Result<LayerMergeResult> {
    let mut result = LayerMergeResult::new();

    // Collect all unique file paths across all layers
    let all_paths = collect_all_file_paths(&config.layers, config, repo)?;

    // Merge each file path
    for path in all_paths {
        match merge_file_across_layers(&path, &config.layers, config, repo) {
            Ok(merged) => {
                result.merged_files.insert(path, merged);
            }
            Err(JinError::MergeConflict { .. }) => {
                result.conflict_files.push(path);
            }
            Err(e) => return Err(e),
        }
    }

    Ok(result)
}

/// Collect all unique file paths across all applicable layers.
///
/// Iterates through each layer, resolves its Git ref, and lists all files
/// in its tree. Returns a set of unique paths.
fn collect_all_file_paths(
    layers: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<HashSet<PathBuf>> {
    let mut paths = HashSet::new();

    for layer in layers {
        let ref_path = layer.ref_path(
            config.mode.as_deref(),
            config.scope.as_deref(),
            config.project.as_deref(),
        );

        // CRITICAL: Check ref_exists() before resolve_ref()
        if repo.ref_exists(&ref_path) {
            if let Ok(commit_oid) = repo.resolve_ref(&ref_path) {
                let commit = repo.inner().find_commit(commit_oid)?;
                let tree_oid = commit.tree_id();

                for file_path in repo.list_tree_files(tree_oid)? {
                    paths.insert(PathBuf::from(file_path));
                }
            }
        }
        // Layer ref doesn't exist = no files in this layer (skip gracefully)
    }

    Ok(paths)
}

/// Merge a single file across multiple layers.
///
/// Reads the file content from each layer that contains it,
/// parses according to file format, and deep-merges in precedence order.
fn merge_file_across_layers(
    path: &std::path::Path,
    layers: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<MergedFile> {
    let mut accumulated: Option<MergeValue> = None;
    let mut source_layers = Vec::new();
    let mut format = FileFormat::Text;

    // Process layers in precedence order (lowest first)
    for layer in layers {
        let ref_path = layer.ref_path(
            config.mode.as_deref(),
            config.scope.as_deref(),
            config.project.as_deref(),
        );

        // CRITICAL: Check ref_exists() before resolve_ref()
        if !repo.ref_exists(&ref_path) {
            continue;
        }

        if let Ok(commit_oid) = repo.resolve_ref(&ref_path) {
            let commit = repo.inner().find_commit(commit_oid)?;
            let tree_oid = commit.tree_id();

            if let Ok(content) = repo.read_file_from_tree(tree_oid, path) {
                let content_str = String::from_utf8_lossy(&content);

                format = detect_format(path);
                let layer_value = parse_content(&content_str, format)?;

                source_layers.push(*layer);

                accumulated = Some(match accumulated {
                    Some(base) => deep_merge(base, layer_value)?,
                    None => layer_value,
                });
            }
        }
    }

    match accumulated {
        Some(content) => Ok(MergedFile {
            content,
            source_layers,
            format,
        }),
        None => Err(JinError::NotFound(path.display().to_string())),
    }
}

/// Detect file format from path extension.
///
/// Returns the appropriate FileFormat based on the file extension.
/// Unknown extensions default to Text.
pub fn detect_format(path: &std::path::Path) -> FileFormat {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext.to_lowercase().as_str() {
        "json" => FileFormat::Json,
        "yaml" | "yml" => FileFormat::Yaml,
        "toml" => FileFormat::Toml,
        "ini" | "cfg" | "conf" => FileFormat::Ini,
        _ => FileFormat::Text,
    }
}

/// Parse content string according to file format.
///
/// Returns a MergeValue representation of the content.
/// Text files are wrapped as MergeValue::String.
pub fn parse_content(content: &str, format: FileFormat) -> Result<MergeValue> {
    match format {
        FileFormat::Json => MergeValue::from_json(content),
        FileFormat::Yaml => MergeValue::from_yaml(content),
        FileFormat::Toml => MergeValue::from_toml(content),
        FileFormat::Ini => MergeValue::from_ini(content),
        FileFormat::Text => Ok(MergeValue::String(content.to_string())),
    }
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

/// Find which layers contain a specific file.
///
/// Iterates through the provided layers in precedence order and checks
/// if each layer's Git tree contains the specified file. Layers that
/// don't exist yet are gracefully skipped.
///
/// # Arguments
///
/// * `file_path` - Path to the file to search for (relative to repo root)
/// * `layers` - Layers to search, in precedence order
/// * `config` - Merge configuration with mode/scope/project context
/// * `repo` - Jin repository for Git operations
///
/// # Returns
///
/// * `Ok(Vec<Layer>)` - Layers containing the file, in input order
/// * `Err(JinError)` - Git operation failure
///
/// # Examples
///
/// ```ignore
/// use jin::merge::{find_layers_containing_file, LayerMergeConfig};
/// use jin::core::Layer;
/// use std::path::Path;
///
/// let config = LayerMergeConfig { /* ... */ };
/// let layers = vec![Layer::GlobalBase, Layer::ModeBase];
/// let containing = find_layers_containing_file(
///     Path::new("config.json"),
///     &layers,
///     &config,
///     &repo
/// )?;
/// ```
pub fn find_layers_containing_file(
    file_path: &std::path::Path,
    layers: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<Vec<Layer>> {
    let mut containing_layers = Vec::new();

    for layer in layers {
        let ref_path = layer.ref_path(
            config.mode.as_deref(),
            config.scope.as_deref(),
            config.project.as_deref(),
        );

        // CRITICAL: Check ref_exists() before resolve_ref()
        // Layer refs may not exist yet - skip gracefully
        if !repo.ref_exists(&ref_path) {
            continue;
        }

        // Resolve the commit for this layer
        let commit_oid = repo.resolve_ref(&ref_path);
        if let Ok(commit_oid) = commit_oid {
            let commit = repo.inner().find_commit(commit_oid)?;
            let tree_oid = commit.tree_id();

            // Check if file exists in this layer's tree
            // get_tree_entry() returns Err if file not found
            if repo.get_tree_entry(tree_oid, file_path).is_ok() {
                containing_layers.push(*layer);
            }
        }
        // If resolve_ref fails, skip this layer (may not be initialized)
    }

    Ok(containing_layers)
}

/// Check if a file has different content across multiple layers.
///
/// Compares file content across all provided layers to detect conflicts.
/// For structured files (JSON, YAML, TOML, INI), content is parsed to
/// `MergeValue` for semantic comparison. For text files, raw content
/// strings are compared directly.
///
/// # Arguments
///
/// * `file_path` - Path to the file to check (relative to repo root)
/// * `layers_with_file` - Layers containing this file (from find_layers_containing_file)
/// * `config` - Merge configuration with mode/scope/project context
/// * `repo` - Jin repository for Git operations
///
/// # Returns
///
/// * `Ok(false)` - 0-1 layers OR all layers have identical content (no conflict)
/// * `Ok(true)` - 2+ layers with differing content (conflict detected)
/// * `Err(JinError)` - Git operation failure or parse error
///
/// # Examples
///
/// ```ignore
/// use jin::merge::{find_layers_containing_file, has_different_content_across_layers};
/// use jin::core::Layer;
/// use std::path::Path;
///
/// let layers = vec![Layer::GlobalBase, Layer::ModeBase];
/// let config = LayerMergeConfig { /* ... */ };
/// let containing = find_layers_containing_file(Path::new("config.json"), &layers, &config, &repo)?;
///
/// if containing.len() > 1 {
///     let has_conflict = has_different_content_across_layers(
///         Path::new("config.json"),
///         &containing,
///         &config,
///         &repo
///     )?;
///     if has_conflict {
///         println!("Conflict detected!");
///     }
/// }
/// ```
pub fn has_different_content_across_layers(
    file_path: &std::path::Path,
    layers_with_file: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<bool> {
    // Early exit: no conflict possible with fewer than 2 layers
    if layers_with_file.len() <= 1 {
        return Ok(false);
    }

    let format = detect_format(file_path);

    // For text files, compare raw strings (not MergeValue)
    if format == FileFormat::Text {
        return has_different_text_content(file_path, layers_with_file, config, repo);
    }

    // For structured files, parse and compare MergeValue
    has_different_structured_content(file_path, layers_with_file, config, repo, format)
}

/// Helper: Compare text file content across layers (raw string comparison)
fn has_different_text_content(
    file_path: &std::path::Path,
    layers_with_file: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
) -> Result<bool> {
    // Read content from first layer
    let first_layer = &layers_with_file[0];
    let first_ref_path = first_layer.ref_path(
        config.mode.as_deref(),
        config.scope.as_deref(),
        config.project.as_deref(),
    );

    let first_commit_oid = repo.resolve_ref(&first_ref_path)?;
    let first_commit = repo.inner().find_commit(first_commit_oid)?;
    let first_tree_oid = first_commit.tree_id();

    let first_content_bytes = repo.read_file_from_tree(first_tree_oid, file_path)?;
    let first_content = String::from_utf8_lossy(&first_content_bytes);

    // Compare with each subsequent layer
    for layer in &layers_with_file[1..] {
        let ref_path = layer.ref_path(
            config.mode.as_deref(),
            config.scope.as_deref(),
            config.project.as_deref(),
        );

        let commit_oid = repo.resolve_ref(&ref_path)?;
        let commit = repo.inner().find_commit(commit_oid)?;
        let tree_oid = commit.tree_id();

        let content_bytes = repo.read_file_from_tree(tree_oid, file_path)?;
        let content = String::from_utf8_lossy(&content_bytes);

        if content != first_content {
            return Ok(true); // Different content detected
        }
    }

    Ok(false) // All layers have identical content
}

/// Helper: Compare structured file content across layers (MergeValue comparison)
fn has_different_structured_content(
    file_path: &std::path::Path,
    layers_with_file: &[Layer],
    config: &LayerMergeConfig,
    repo: &JinRepo,
    format: FileFormat,
) -> Result<bool> {
    // Read and parse content from first layer
    let first_layer = &layers_with_file[0];
    let first_ref_path = first_layer.ref_path(
        config.mode.as_deref(),
        config.scope.as_deref(),
        config.project.as_deref(),
    );

    let first_commit_oid = repo.resolve_ref(&first_ref_path)?;
    let first_commit = repo.inner().find_commit(first_commit_oid)?;
    let first_tree_oid = first_commit.tree_id();

    let first_content_bytes = repo.read_file_from_tree(first_tree_oid, file_path)?;
    let first_content_str = String::from_utf8_lossy(&first_content_bytes);
    let first_value = parse_content(&first_content_str, format)?;

    // Compare with each subsequent layer
    for layer in &layers_with_file[1..] {
        let ref_path = layer.ref_path(
            config.mode.as_deref(),
            config.scope.as_deref(),
            config.project.as_deref(),
        );

        let commit_oid = repo.resolve_ref(&ref_path)?;
        let commit = repo.inner().find_commit(commit_oid)?;
        let tree_oid = commit.tree_id();

        let content_bytes = repo.read_file_from_tree(tree_oid, file_path)?;
        let content_str = String::from_utf8_lossy(&content_bytes);
        let value = parse_content(&content_str, format)?;

        if value != first_value {
            return Ok(true); // Different content detected
        }
    }

    Ok(false) // All layers have identical content
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::objects::ObjectOps;
    use std::path::Path;
    use tempfile;

    // ========== FileFormat & MergedFile Tests ==========

    #[test]
    fn test_file_format_equality() {
        assert_eq!(FileFormat::Json, FileFormat::Json);
        assert_ne!(FileFormat::Json, FileFormat::Yaml);
    }

    #[test]
    fn test_file_format_clone() {
        let format = FileFormat::Toml;
        let cloned = format;
        assert_eq!(format, cloned);
    }

    // ========== LayerMergeResult Tests ==========

    #[test]
    fn test_merge_result_new() {
        let result = LayerMergeResult::new();
        assert!(result.is_clean());
        assert!(result.merged_files.is_empty());
        assert!(result.conflict_files.is_empty());
        assert!(result.added_files.is_empty());
        assert!(result.removed_files.is_empty());
    }

    #[test]
    fn test_merge_result_default() {
        let result = LayerMergeResult::default();
        assert!(result.is_clean());
    }

    #[test]
    fn test_merge_result_is_clean_with_conflicts() {
        let mut result = LayerMergeResult::new();
        result.conflict_files.push(PathBuf::from("conflict.json"));
        assert!(!result.is_clean());
    }

    // ========== detect_format Tests ==========

    #[test]
    fn test_detect_format_json() {
        assert_eq!(
            detect_format(&PathBuf::from("config.json")),
            FileFormat::Json
        );
        assert_eq!(
            detect_format(&PathBuf::from("path/to/config.json")),
            FileFormat::Json
        );
        assert_eq!(
            detect_format(&PathBuf::from("CONFIG.JSON")),
            FileFormat::Json
        );
    }

    #[test]
    fn test_detect_format_yaml() {
        assert_eq!(
            detect_format(&PathBuf::from("config.yaml")),
            FileFormat::Yaml
        );
        assert_eq!(
            detect_format(&PathBuf::from("config.yml")),
            FileFormat::Yaml
        );
        assert_eq!(
            detect_format(&PathBuf::from("CONFIG.YML")),
            FileFormat::Yaml
        );
    }

    #[test]
    fn test_detect_format_toml() {
        assert_eq!(
            detect_format(&PathBuf::from("config.toml")),
            FileFormat::Toml
        );
        assert_eq!(
            detect_format(&PathBuf::from("Cargo.toml")),
            FileFormat::Toml
        );
    }

    #[test]
    fn test_detect_format_ini() {
        assert_eq!(detect_format(&PathBuf::from("config.ini")), FileFormat::Ini);
        assert_eq!(
            detect_format(&PathBuf::from("settings.cfg")),
            FileFormat::Ini
        );
        assert_eq!(detect_format(&PathBuf::from("app.conf")), FileFormat::Ini);
    }

    #[test]
    fn test_detect_format_text() {
        assert_eq!(detect_format(&PathBuf::from("README.md")), FileFormat::Text);
        assert_eq!(detect_format(&PathBuf::from("script.sh")), FileFormat::Text);
        assert_eq!(detect_format(&PathBuf::from("notes.txt")), FileFormat::Text);
        assert_eq!(
            detect_format(&PathBuf::from("noextension")),
            FileFormat::Text
        );
    }

    // ========== parse_content Tests ==========

    #[test]
    fn test_parse_content_json() {
        let json = r#"{"key": "value", "num": 42}"#;
        let result = parse_content(json, FileFormat::Json).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("key").unwrap().as_str(), Some("value"));
        assert_eq!(obj.get("num").unwrap().as_i64(), Some(42));
    }

    #[test]
    fn test_parse_content_yaml() {
        let yaml = "key: value\nnum: 42";
        let result = parse_content(yaml, FileFormat::Yaml).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("key").unwrap().as_str(), Some("value"));
        assert_eq!(obj.get("num").unwrap().as_i64(), Some(42));
    }

    #[test]
    fn test_parse_content_toml() {
        let toml = "key = \"value\"\nnum = 42";
        let result = parse_content(toml, FileFormat::Toml).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("key").unwrap().as_str(), Some("value"));
        assert_eq!(obj.get("num").unwrap().as_i64(), Some(42));
    }

    #[test]
    fn test_parse_content_ini() {
        let ini = "[section]\nkey=value";
        let result = parse_content(ini, FileFormat::Ini).unwrap();
        let section = result.as_object().unwrap().get("section").unwrap();
        assert_eq!(
            section.as_object().unwrap().get("key").unwrap().as_str(),
            Some("value")
        );
    }

    #[test]
    fn test_parse_content_text() {
        let text = "Hello, World!\nThis is plain text.";
        let result = parse_content(text, FileFormat::Text).unwrap();
        assert_eq!(result.as_str(), Some(text));
    }

    #[test]
    fn test_parse_content_json_invalid() {
        let invalid = "{not valid json";
        let result = parse_content(invalid, FileFormat::Json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_content_yaml_with_null() {
        let yaml = "key: null";
        let result = parse_content(yaml, FileFormat::Yaml).unwrap();
        let obj = result.as_object().unwrap();
        assert!(obj.get("key").unwrap().is_null());
    }

    // ========== get_applicable_layers Tests ==========

    #[test]
    fn test_applicable_layers_no_context() {
        let layers = get_applicable_layers(None, None, None);
        assert!(layers.contains(&Layer::GlobalBase));
        assert!(layers.contains(&Layer::ProjectBase));
        assert!(layers.contains(&Layer::UserLocal));
        assert!(layers.contains(&Layer::WorkspaceActive));
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

    #[test]
    fn test_applicable_layers_scope_without_mode() {
        let layers = get_applicable_layers(None, Some("language:rust"), None);
        assert!(layers.contains(&Layer::ScopeBase));
        assert!(!layers.contains(&Layer::ModeBase));
        assert!(!layers.contains(&Layer::ModeScope));
    }

    // ========== Layer Precedence Tests ==========

    #[test]
    fn test_layer_precedence_order() {
        let layers = Layer::all_in_precedence_order();
        assert_eq!(layers[0], Layer::GlobalBase);
        assert_eq!(layers[8], Layer::WorkspaceActive);

        for i in 0..layers.len() - 1 {
            assert!(layers[i].precedence() < layers[i + 1].precedence());
        }
    }

    #[test]
    fn test_layer_precedence_values() {
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

    // ========== find_layers_containing_file Tests ==========

    // Helper function to create a test repository for layer tests
    fn create_layer_test_repo() -> (tempfile::TempDir, JinRepo) {
        let temp = tempfile::TempDir::new().unwrap();
        let repo_path = temp.path().join(".jin");
        let repo = JinRepo::create_at(&repo_path).unwrap();
        (temp, repo)
    }

    // Helper to create a commit with a file and set a ref
    fn create_layer_with_file(
        repo: &JinRepo,
        ref_name: &str,
        file_path: &str,
        content: &[u8],
    ) -> Result<()> {
        let blob_oid = repo.create_blob(content)?;
        let tree_oid = repo.create_tree_from_paths(&[(file_path.to_string(), blob_oid)])?;

        let sig = git2::Signature::now("test", "test@test.com")?;
        let tree = repo.inner().find_tree(tree_oid)?;
        let commit_oid = repo
            .inner()
            .commit(None, &sig, &sig, "test commit", &tree, &[])?;

        repo.set_ref(ref_name, commit_oid, "test layer")?;
        Ok(())
    }

    #[test]
    fn test_find_layers_single_layer_containing_file() {
        let (_temp, repo) = create_layer_test_repo();

        // Create a layer with a file
        create_layer_with_file(
            &repo,
            "refs/jin/layers/global",
            "config.json",
            br#"{"key":"global"}"#,
        )
        .unwrap();

        let layers = vec![Layer::GlobalBase];
        let config = LayerMergeConfig {
            layers,
            mode: None,
            scope: None,
            project: None,
        };

        let result =
            find_layers_containing_file(Path::new("config.json"), &config.layers, &config, &repo)
                .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Layer::GlobalBase);
    }

    #[test]
    fn test_find_layers_multiple_layers_containing_file() {
        let (_temp, repo) = create_layer_test_repo();

        // Create two layers with the same file
        create_layer_with_file(
            &repo,
            "refs/jin/layers/global",
            "config.json",
            br#"{"key":"global"}"#,
        )
        .unwrap();

        create_layer_with_file(
            &repo,
            "refs/jin/layers/mode/dev/_",
            "config.json",
            br#"{"key":"mode"}"#,
        )
        .unwrap();

        let layers = vec![Layer::GlobalBase, Layer::ModeBase];
        let config = LayerMergeConfig {
            layers,
            mode: Some("dev".to_string()),
            scope: None,
            project: None,
        };

        let result =
            find_layers_containing_file(Path::new("config.json"), &config.layers, &config, &repo)
                .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], Layer::GlobalBase);
        assert_eq!(result[1], Layer::ModeBase);
    }

    #[test]
    fn test_find_layers_file_not_in_any_layer() {
        let (_temp, repo) = create_layer_test_repo();

        // Create a layer with a different file
        create_layer_with_file(
            &repo,
            "refs/jin/layers/global",
            "other.json",
            br#"{"key":"value"}"#,
        )
        .unwrap();

        let layers = vec![Layer::GlobalBase];
        let config = LayerMergeConfig {
            layers,
            mode: None,
            scope: None,
            project: None,
        };

        let result =
            find_layers_containing_file(Path::new("config.json"), &config.layers, &config, &repo)
                .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_layers_nonexistent_file_path() {
        let (_temp, repo) = create_layer_test_repo();

        let layers = vec![Layer::GlobalBase];
        let config = LayerMergeConfig {
            layers,
            mode: None,
            scope: None,
            project: None,
        };

        // Non-existent file should return empty vec, not error
        let result = find_layers_containing_file(
            Path::new("does/not/exist.json"),
            &config.layers,
            &config,
            &repo,
        )
        .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_layers_empty_layer_list() {
        let (_temp, repo) = create_layer_test_repo();

        let layers: Vec<Layer> = vec![];
        let config = LayerMergeConfig {
            layers,
            mode: None,
            scope: None,
            project: None,
        };

        let result =
            find_layers_containing_file(Path::new("config.json"), &config.layers, &config, &repo)
                .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_layers_nonexistent_layer_refs_skipped() {
        let (_temp, repo) = create_layer_test_repo();

        // Create only the GlobalBase layer
        create_layer_with_file(
            &repo,
            "refs/jin/layers/global",
            "config.json",
            br#"{"key":"global"}"#,
        )
        .unwrap();

        // Include layers that don't exist yet
        let layers = vec![Layer::GlobalBase, Layer::ModeBase, Layer::ProjectBase];
        let config = LayerMergeConfig {
            layers,
            mode: Some("dev".to_string()),
            scope: None,
            project: Some("myproject".to_string()),
        };

        let result =
            find_layers_containing_file(Path::new("config.json"), &config.layers, &config, &repo)
                .unwrap();
        // Only GlobalBase should be returned; non-existent layers are skipped
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Layer::GlobalBase);
    }

    #[test]
    fn test_find_layers_precedence_order_maintained() {
        let (_temp, repo) = create_layer_test_repo();

        // Create multiple layers with the same file in reverse precedence order
        create_layer_with_file(
            &repo,
            "refs/jin/layers/project/myproject",
            "config.json",
            br#"{"key":"project"}"#,
        )
        .unwrap();

        create_layer_with_file(
            &repo,
            "refs/jin/layers/global",
            "config.json",
            br#"{"key":"global"}"#,
        )
        .unwrap();

        // Pass layers in specific order
        let layers = vec![Layer::ProjectBase, Layer::GlobalBase];
        let config = LayerMergeConfig {
            layers,
            mode: None,
            scope: None,
            project: Some("myproject".to_string()),
        };

        let result =
            find_layers_containing_file(Path::new("config.json"), &config.layers, &config, &repo)
                .unwrap();
        assert_eq!(result.len(), 2);
        // Order should match input order (ProjectBase first, then GlobalBase)
        assert_eq!(result[0], Layer::ProjectBase);
        assert_eq!(result[1], Layer::GlobalBase);
    }

    #[test]
    fn test_find_layers_nested_directory_files() {
        let (_temp, repo) = create_layer_test_repo();

        // Create a layer with a nested file
        create_layer_with_file(
            &repo,
            "refs/jin/layers/global",
            "src/config/app.json",
            br#"{"key":"value"}"#,
        )
        .unwrap();

        let layers = vec![Layer::GlobalBase];
        let config = LayerMergeConfig {
            layers,
            mode: None,
            scope: None,
            project: None,
        };

        let result = find_layers_containing_file(
            Path::new("src/config/app.json"),
            &config.layers,
            &config,
            &repo,
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Layer::GlobalBase);
    }

    #[test]
    fn test_find_layers_mode_scope_with_context() {
        let (_temp, repo) = create_layer_test_repo();

        // Create ModeScope layer with proper ref path
        create_layer_with_file(
            &repo,
            "refs/jin/layers/mode/dev/scope/frontend/_",
            "config.json",
            br#"{"key":"mode-scope"}"#,
        )
        .unwrap();

        let layers = vec![Layer::ModeScope];
        let config = LayerMergeConfig {
            layers,
            mode: Some("dev".to_string()),
            scope: Some("frontend".to_string()),
            project: None,
        };

        let result =
            find_layers_containing_file(Path::new("config.json"), &config.layers, &config, &repo)
                .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], Layer::ModeScope);
    }

    // ========== has_different_content_across_layers Tests ==========

    #[test]
    fn test_has_different_content_single_layer() {
        let (_temp, repo) = create_layer_test_repo();
        let config = LayerMergeConfig {
            layers: vec![Layer::GlobalBase],
            mode: None,
            scope: None,
            project: None,
        };

        // Single layer - should return false
        let layers = vec![Layer::GlobalBase];
        let result =
            has_different_content_across_layers(Path::new("config.json"), &layers, &config, &repo);

        assert!(result.is_ok());
        assert!(!result.unwrap()); // No conflict with single layer
    }

    #[test]
    fn test_has_different_content_same_structured() {
        let (_temp, repo) = create_layer_test_repo();
        let config = LayerMergeConfig {
            layers: vec![Layer::GlobalBase, Layer::ModeBase],
            mode: Some("test".to_string()),
            scope: None,
            project: None,
        };

        let content = br#"{"port": 8080, "debug": true}"#;

        // Both layers have identical content
        create_layer_with_file(&repo, "refs/jin/layers/global", "config.json", content).unwrap();
        create_layer_with_file(&repo, "refs/jin/layers/mode/test/_", "config.json", content)
            .unwrap();

        let layers = vec![Layer::GlobalBase, Layer::ModeBase];
        let result =
            has_different_content_across_layers(Path::new("config.json"), &layers, &config, &repo);

        assert!(result.is_ok());
        assert!(!result.unwrap()); // No conflict - same content
    }

    #[test]
    fn test_has_different_content_different_structured() {
        let (_temp, repo) = create_layer_test_repo();
        let config = LayerMergeConfig {
            layers: vec![Layer::GlobalBase, Layer::ModeBase],
            mode: Some("test".to_string()),
            scope: None,
            project: None,
        };

        let global_content = br#"{"port": 8080}"#;
        let mode_content = br#"{"port": 9090}"#;

        // Layers have different content
        create_layer_with_file(
            &repo,
            "refs/jin/layers/global",
            "config.json",
            global_content,
        )
        .unwrap();
        create_layer_with_file(
            &repo,
            "refs/jin/layers/mode/test/_",
            "config.json",
            mode_content,
        )
        .unwrap();

        let layers = vec![Layer::GlobalBase, Layer::ModeBase];
        let result =
            has_different_content_across_layers(Path::new("config.json"), &layers, &config, &repo);

        assert!(result.is_ok());
        assert!(result.unwrap()); // Conflict detected
    }

    #[test]
    fn test_has_different_content_same_text() {
        let (_temp, repo) = create_layer_test_repo();
        let config = LayerMergeConfig {
            layers: vec![Layer::GlobalBase, Layer::ModeBase],
            mode: Some("test".to_string()),
            scope: None,
            project: None,
        };

        let content = b"hello world\nline two\n";

        // Both layers have identical text content
        create_layer_with_file(&repo, "refs/jin/layers/global", "README.txt", content).unwrap();
        create_layer_with_file(&repo, "refs/jin/layers/mode/test/_", "README.txt", content)
            .unwrap();

        let layers = vec![Layer::GlobalBase, Layer::ModeBase];
        let result =
            has_different_content_across_layers(Path::new("README.txt"), &layers, &config, &repo);

        assert!(result.is_ok());
        assert!(!result.unwrap()); // No conflict - same content
    }

    #[test]
    fn test_has_different_content_different_text() {
        let (_temp, repo) = create_layer_test_repo();
        let config = LayerMergeConfig {
            layers: vec![Layer::GlobalBase, Layer::ModeBase],
            mode: Some("test".to_string()),
            scope: None,
            project: None,
        };

        let global_content = b"hello world\n";
        let mode_content = b"goodbye world\n";

        // Layers have different text content
        create_layer_with_file(
            &repo,
            "refs/jin/layers/global",
            "README.txt",
            global_content,
        )
        .unwrap();
        create_layer_with_file(
            &repo,
            "refs/jin/layers/mode/test/_",
            "README.txt",
            mode_content,
        )
        .unwrap();

        let layers = vec![Layer::GlobalBase, Layer::ModeBase];
        let result =
            has_different_content_across_layers(Path::new("README.txt"), &layers, &config, &repo);

        assert!(result.is_ok());
        assert!(result.unwrap()); // Conflict detected
    }

    #[test]
    fn test_has_different_content_three_layers_all_same() {
        let (_temp, repo) = create_layer_test_repo();
        let config = LayerMergeConfig {
            layers: vec![Layer::GlobalBase, Layer::ModeBase, Layer::ModeScope],
            mode: Some("test".to_string()),
            scope: Some("web".to_string()),
            project: None,
        };

        let content = br#"{"value": 42}"#;

        // All three layers have identical content
        create_layer_with_file(&repo, "refs/jin/layers/global", "config.json", content).unwrap();
        create_layer_with_file(&repo, "refs/jin/layers/mode/test/_", "config.json", content)
            .unwrap();
        create_layer_with_file(
            &repo,
            "refs/jin/layers/mode/test/scope/web/_",
            "config.json",
            content,
        )
        .unwrap();

        let layers = vec![Layer::GlobalBase, Layer::ModeBase, Layer::ModeScope];
        let result =
            has_different_content_across_layers(Path::new("config.json"), &layers, &config, &repo);

        assert!(result.is_ok());
        assert!(!result.unwrap()); // No conflict - all same
    }

    #[test]
    fn test_has_different_content_three_layers_one_different() {
        let (_temp, repo) = create_layer_test_repo();
        let config = LayerMergeConfig {
            layers: vec![Layer::GlobalBase, Layer::ModeBase, Layer::ModeScope],
            mode: Some("test".to_string()),
            scope: Some("web".to_string()),
            project: None,
        };

        let global_content = br#"{"value": 1}"#;
        let mode_content = br#"{"value": 2}"#;
        let scope_content = br#"{"value": 2}"#;

        // Global differs, mode and scope are same
        create_layer_with_file(
            &repo,
            "refs/jin/layers/global",
            "config.json",
            global_content,
        )
        .unwrap();
        create_layer_with_file(
            &repo,
            "refs/jin/layers/mode/test/_",
            "config.json",
            mode_content,
        )
        .unwrap();
        create_layer_with_file(
            &repo,
            "refs/jin/layers/mode/test/scope/web/_",
            "config.json",
            scope_content,
        )
        .unwrap();

        let layers = vec![Layer::GlobalBase, Layer::ModeBase, Layer::ModeScope];
        let result =
            has_different_content_across_layers(Path::new("config.json"), &layers, &config, &repo);

        assert!(result.is_ok());
        assert!(result.unwrap()); // Conflict detected
    }

    #[test]
    fn test_has_different_content_yaml_format() {
        let (_temp, repo) = create_layer_test_repo();
        let config = LayerMergeConfig {
            layers: vec![Layer::GlobalBase, Layer::ModeBase],
            mode: Some("test".to_string()),
            scope: None,
            project: None,
        };

        let global_content = b"port: 8080\n";
        let mode_content = b"port: 9090\n";

        create_layer_with_file(
            &repo,
            "refs/jin/layers/global",
            "config.yaml",
            global_content,
        )
        .unwrap();
        create_layer_with_file(
            &repo,
            "refs/jin/layers/mode/test/_",
            "config.yaml",
            mode_content,
        )
        .unwrap();

        let layers = vec![Layer::GlobalBase, Layer::ModeBase];
        let result =
            has_different_content_across_layers(Path::new("config.yaml"), &layers, &config, &repo);

        assert!(result.is_ok());
        assert!(result.unwrap()); // Conflict detected
    }

    #[test]
    fn test_has_different_content_toml_format() {
        let (_temp, repo) = create_layer_test_repo();
        let config = LayerMergeConfig {
            layers: vec![Layer::GlobalBase, Layer::ModeBase],
            mode: Some("test".to_string()),
            scope: None,
            project: None,
        };

        let global_content = br#"port = 8080"#;
        let mode_content = br#"port = 9090"#;

        create_layer_with_file(
            &repo,
            "refs/jin/layers/global",
            "config.toml",
            global_content,
        )
        .unwrap();
        create_layer_with_file(
            &repo,
            "refs/jin/layers/mode/test/_",
            "config.toml",
            mode_content,
        )
        .unwrap();

        let layers = vec![Layer::GlobalBase, Layer::ModeBase];
        let result =
            has_different_content_across_layers(Path::new("config.toml"), &layers, &config, &repo);

        assert!(result.is_ok());
        assert!(result.unwrap()); // Conflict detected
    }

    #[test]
    fn test_has_different_content_ini_format() {
        let (_temp, repo) = create_layer_test_repo();
        let config = LayerMergeConfig {
            layers: vec![Layer::GlobalBase, Layer::ModeBase],
            mode: Some("test".to_string()),
            scope: None,
            project: None,
        };

        let global_content = b"[section]\nport=8080\n";
        let mode_content = b"[section]\nport=9090\n";

        create_layer_with_file(
            &repo,
            "refs/jin/layers/global",
            "config.ini",
            global_content,
        )
        .unwrap();
        create_layer_with_file(
            &repo,
            "refs/jin/layers/mode/test/_",
            "config.ini",
            mode_content,
        )
        .unwrap();

        let layers = vec![Layer::GlobalBase, Layer::ModeBase];
        let result =
            has_different_content_across_layers(Path::new("config.ini"), &layers, &config, &repo);

        assert!(result.is_ok());
        assert!(result.unwrap()); // Conflict detected
    }

    #[test]
    fn test_has_different_content_semantic_json_whitespace() {
        let (_temp, repo) = create_layer_test_repo();
        let config = LayerMergeConfig {
            layers: vec![Layer::GlobalBase, Layer::ModeBase],
            mode: Some("test".to_string()),
            scope: None,
            project: None,
        };

        // Semantically identical JSON, different formatting
        let global_content = br#"{"name":"test","value":42}"#;
        let mode_content = br#"{
  "name": "test",
  "value": 42
}"#;

        create_layer_with_file(
            &repo,
            "refs/jin/layers/global",
            "config.json",
            global_content,
        )
        .unwrap();
        create_layer_with_file(
            &repo,
            "refs/jin/layers/mode/test/_",
            "config.json",
            mode_content,
        )
        .unwrap();

        let layers = vec![Layer::GlobalBase, Layer::ModeBase];
        let result =
            has_different_content_across_layers(Path::new("config.json"), &layers, &config, &repo);

        assert!(result.is_ok());
        assert!(!result.unwrap()); // No conflict - semantically same
    }

    #[test]
    fn test_has_different_content_text_exact_match_required() {
        let (_temp, repo) = create_layer_test_repo();
        let config = LayerMergeConfig {
            layers: vec![Layer::GlobalBase, Layer::ModeBase],
            mode: Some("test".to_string()),
            scope: None,
            project: None,
        };

        // Text files: different whitespace = different content
        let global_content = b"hello\nworld\n";
        let mode_content = b"hello\n  world\n"; // Extra spaces

        create_layer_with_file(
            &repo,
            "refs/jin/layers/global",
            "README.txt",
            global_content,
        )
        .unwrap();
        create_layer_with_file(
            &repo,
            "refs/jin/layers/mode/test/_",
            "README.txt",
            mode_content,
        )
        .unwrap();

        let layers = vec![Layer::GlobalBase, Layer::ModeBase];
        let result =
            has_different_content_across_layers(Path::new("README.txt"), &layers, &config, &repo);

        assert!(result.is_ok());
        assert!(result.unwrap()); // Conflict - text requires exact match
    }

    #[test]
    fn test_has_different_content_empty_layer_list() {
        let (_temp, repo) = create_layer_test_repo();
        let config = LayerMergeConfig {
            layers: vec![],
            mode: None,
            scope: None,
            project: None,
        };

        let layers: Vec<Layer> = vec![];
        let result =
            has_different_content_across_layers(Path::new("config.json"), &layers, &config, &repo);

        assert!(result.is_ok());
        assert!(!result.unwrap()); // No conflict with empty list
    }
}
