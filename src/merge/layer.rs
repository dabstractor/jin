//! Layer merge orchestration for Jin's multi-layer configuration system.
//!
//! This module provides the `LayerMerge` orchestrator that merges multiple Jin
//! layers in precedence order to produce a consolidated workspace configuration.
//!
//! # Layer Merge Process
//!
//! 1. Determine active layers based on project, mode, and scope context
//! 2. Sort layers by precedence (lower layer number = lower priority)
//! 3. For each layer in precedence order:
//!    - Get layer ref from Git
//!    - Walk the layer's tree for file entries
//!    - Parse each file by format to `MergeValue`
//!    - Deep merge into accumulator using `MergeValue::merge()`
//! 4. Return final merged result
//!
//! # Layer Precedence (PRD §11.2)
//!
//! | Layer | Precedence |
//! |-------|------------|
//! | GlobalBase | 1 (lowest) |
//! | ModeBase | 2 |
//! | ModeScope | 3 |
//! | ModeScopeProject | 4 |
//! | ModeProject | 5 |
//! | ScopeBase | 6 |
//! | ProjectBase | 7 |
//! | UserLocal | 8 |
//! | WorkspaceActive | 9 (highest) |

use crate::core::error::{JinError, Result};
use crate::core::Layer;
use crate::git::JinRepo;
use crate::merge::value::MergeValue;
use crate::merge::text::TextMerge;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::Path;

// ===== File Format Detection =====

/// File format for parsing and merge behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    /// JSON files - deep merge with RFC 7396 semantics
    Json,
    /// YAML files - deep merge with RFC 7396 semantics
    Yaml,
    /// TOML files - deep merge with RFC 7396 semantics
    Toml,
    /// INI files - section merge (sections as top-level keys)
    Ini,
    /// Text files - higher layer replaces (future: 3-way merge)
    Text,
    /// Unknown format - treat as text
    Unknown,
}

impl FileFormat {
    /// Detect file format from path extension.
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|s| s.to_str()) {
            Some("json") => FileFormat::Json,
            Some("yaml") | Some("yml") => FileFormat::Yaml,
            Some("toml") => FileFormat::Toml,
            Some("ini") => FileFormat::Ini,
            _ => FileFormat::Text, // Default to text for unknown formats
        }
    }

    /// Returns true if this format supports structured merging.
    pub fn is_structured(&self) -> bool {
        matches!(self, FileFormat::Json | FileFormat::Yaml | FileFormat::Toml | FileFormat::Ini)
    }
}

// ===== Merge Context =====

/// Context for a merge operation tracking layer state.
///
/// Accumulates merged results as layers are processed in precedence order.
pub struct MergeContext {
    /// Accumulated merged files: path -> MergeValue
    merged_files: IndexMap<String, MergeValue>,
    /// Layers that were actually merged (skipped empty/non-existent)
    merged_layers: Vec<Layer>,
    /// Files that couldn't be parsed (path -> error message)
    parse_errors: Vec<(String, String)>,
    /// For text files: tracks the previous layer that contributed content
    /// Used as the "base" in 3-way merge (previous -> current)
    text_file_layers: HashMap<String, Layer>,
}

impl MergeContext {
    /// Creates a new empty merge context.
    pub fn new() -> Self {
        Self {
            merged_files: IndexMap::new(),
            merged_layers: Vec::new(),
            parse_errors: Vec::new(),
            text_file_layers: HashMap::new(),
        }
    }

    /// Merges a single file into the context.
    fn merge_file(&mut self, path: String, value: MergeValue) {
        if let Some(existing) = self.merged_files.get_mut(&path) {
            // Deep merge with existing value
            if let Ok(merged) = existing.merge(&value) {
                *existing = merged;
            }
            // If merge fails, keep original (could also accumulate errors)
        } else {
            // New file - add to context
            self.merged_files.insert(path, value);
        }
    }

    /// Records a parsing error for a file.
    fn add_parse_error(&mut self, path: String, error: String) {
        self.parse_errors.push((path, error));
    }

    /// Records that a layer was merged.
    fn add_layer(&mut self, layer: Layer) {
        self.merged_layers.push(layer);
    }

    /// Returns the final merged result.
    pub fn get_result(self) -> IndexMap<String, MergeValue> {
        self.merged_files
    }

    /// Returns the layers that were merged.
    pub fn merged_layers(&self) -> &[Layer] {
        &self.merged_layers
    }

    /// Returns the parse errors.
    pub fn parse_errors(&self) -> &[(String, String)] {
        &self.parse_errors
    }
}

impl Default for MergeContext {
    fn default() -> Self {
        Self::new()
    }
}

// ===== Layer Merge Orchestrator =====

/// Orchestrates merging of multiple Jin layers in precedence order.
///
/// This struct handles the core merge orchestration logic:
/// - Collecting layers in precedence order
/// - Reading layer contents from Git trees
/// - Parsing files by format to MergeValue
/// - Applying deep merge algorithm sequentially
/// - Returning consolidated workspace configuration
pub struct LayerMerge<'a> {
    /// The Jin repository for reading layer data
    repo: &'a JinRepo,
    /// Project name for layer resolution
    project: String,
    /// Active mode (optional)
    mode: Option<String>,
    /// Active scope (optional)
    scope: Option<String>,
}

impl<'a> LayerMerge<'a> {
    /// Creates a new layer merge orchestrator.
    ///
    /// # Arguments
    ///
    /// * `repo` - The Jin repository for reading layer data
    /// * `project` - The project name for layer resolution
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let merger = LayerMerge::new(&repo, "myproject");
    /// ```
    pub fn new(repo: &'a JinRepo, project: impl Into<String>) -> Self {
        Self {
            repo,
            project: project.into(),
            mode: None,
            scope: None,
        }
    }

    /// Sets the active mode.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let merger = LayerMerge::new(&repo, "myproject")
    ///     .with_mode("claude");
    /// ```
    pub fn with_mode(mut self, mode: impl Into<String>) -> Self {
        self.mode = Some(mode.into());
        self
    }

    /// Sets the active scope.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let merger = LayerMerge::new(&repo, "myproject")
    ///     .with_scope("language:javascript");
    /// ```
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// Determines the active layers based on project, mode, and scope context.
    ///
    /// Implements PRD §10 scope precedence rules:
    /// - Mode-bound scope > Untethered scope > Mode base
    /// - Untethered scopes apply ONLY if no mode-bound scope of same name exists
    ///
    /// # Returns
    ///
    /// A vector of layers sorted by precedence (using Layer's Ord).
    pub fn determine_active_layers(&self) -> Vec<Layer> {
        let mut layers = vec![
            Layer::GlobalBase,
            Layer::ProjectBase {
                project: self.project.clone(),
            },
        ];

        if let Some(ref mode) = self.mode {
            layers.push(Layer::ModeBase {
                mode: mode.clone(),
            });
            layers.push(Layer::ModeProject {
                mode: mode.clone(),
                project: self.project.clone(),
            });
        }

        if let Some(ref scope) = self.scope {
            // Check if mode-bound scope should be used
            if let Some(ref mode) = self.mode {
                // Mode-bound scope variants
                layers.push(Layer::ModeScope {
                    mode: mode.clone(),
                    scope: scope.clone(),
                });
                layers.push(Layer::ModeScopeProject {
                    mode: mode.clone(),
                    scope: scope.clone(),
                    project: self.project.clone(),
                });
                // NOTE: No untethered scope added when mode-bound exists
            } else {
                // Untethered scope (only if no mode)
                layers.push(Layer::ScopeBase {
                    scope: scope.clone(),
                });
            }
        }

        // UserLocal is added at the end but not read from Git
        // WorkspaceActive is the output layer, never a source

        layers.sort(); // Sort by Layer's Ord (precedence order)
        layers
    }

    /// Reads all files from a layer's Git tree.
    ///
    /// # Arguments
    ///
    /// * `layer` - The layer to read
    ///
    /// # Returns
    ///
    /// A HashMap mapping file paths to their content as bytes.
    ///
    /// # Errors
    ///
    /// Returns `JinError` for Git read failures.
    fn read_layer_files(&self, layer: &Layer) -> Result<HashMap<String, Vec<u8>>> {
        // Get layer ref (returns None if doesn't exist)
        let reference = match self.repo.get_layer_ref(layer)? {
            Some(ref_) => ref_,
            None => return Ok(HashMap::new()), // Empty layer
        };

        // Get commit OID from reference
        let commit_oid = reference.target().ok_or_else(|| JinError::Message(format!(
            "Layer {:?} has no target OID",
            layer
        )))?;

        // Get the commit object to find its tree
        let commit = self.repo.inner.find_commit(commit_oid).map_err(JinError::from)?;
        let tree_id = commit.tree_id();

        // Handle empty trees gracefully
        let tree = self.repo.find_tree(tree_id)?;
        if tree.is_empty() {
            return Ok(HashMap::new());
        }

        // Collect all files
        let mut files = HashMap::new();
        self.repo.walk_tree(tree_id, |path, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob) {
                let blob = self.repo.find_blob(entry.id())?;
                files.insert(path.to_string(), blob.content().to_vec());
            }
            Ok(())
        })?;

        Ok(files)
    }

    /// Parses file content to MergeValue based on format.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path for format detection
    /// * `content` - The file content as bytes
    ///
    /// # Returns
    ///
    /// The parsed MergeValue.
    ///
    /// # Errors
    ///
    /// Returns `JinError::Message` for invalid content.
    fn parse_file_by_format(&self, path: &str, content: &[u8]) -> Result<MergeValue> {
        let path_obj = Path::new(path);
        let format = FileFormat::from_path(path_obj);
        let content_str = std::str::from_utf8(content).map_err(|_| JinError::Message(format!(
            "File {} is not valid UTF-8",
            path
        )))?;

        match format {
            FileFormat::Json => MergeValue::from_json(content_str),
            FileFormat::Yaml => MergeValue::from_yaml(content_str),
            FileFormat::Toml => MergeValue::from_toml(content_str),
            FileFormat::Ini => MergeValue::from_ini(content_str),
            FileFormat::Text | FileFormat::Unknown => {
                // Store as string for now (3-way merge in P2.M4)
                Ok(MergeValue::String(content_str.to_string()))
            }
        }
    }

    /// Merges all active layers in precedence order.
    ///
    /// # Returns
    ///
    /// An IndexMap mapping file paths to their merged MergeValue.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let merger = LayerMerge::new(&repo, "myproject")
    ///     .with_mode("claude");
    /// let result = merger.merge_all()?;
    /// ```
    pub fn merge_all(&self) -> Result<IndexMap<String, MergeValue>> {
        let layers = self.determine_active_layers();
        let mut context = MergeContext::new();

        for layer in &layers {
            // Skip UserLocal (not in Git) and WorkspaceActive (output layer)
            if matches!(layer, Layer::UserLocal | Layer::WorkspaceActive) {
                continue;
            }

            // Read layer files
            let files = self.read_layer_files(layer)?;

            // If layer is empty, skip it
            if files.is_empty() {
                continue;
            }

            // Merge each file
            for (path, content) in files {
                let path_obj = Path::new(&path);
                let format = FileFormat::from_path(path_obj);

                match self.parse_file_by_format(&path, &content) {
                    Ok(value) => {
                        // For text files, use simple replacement (higher priority wins)
                        // For structured formats, use deep merge
                        if matches!(format, FileFormat::Text | FileFormat::Unknown) {
                            // Text files: simple replacement (newer layers override)
                            context.merge_file(path, value);
                        } else {
                            // Structured formats: deep merge
                            context.merge_file(path, value);
                        }
                    }
                    Err(e) => {
                        context.add_parse_error(path, e.to_string());
                    }
                }
            }

            context.add_layer(layer.clone());
        }

        Ok(context.get_result())
    }

    /// Merges a specific subset of layers.
    ///
    /// # Arguments
    ///
    /// * `layers` - The layers to merge (must all be versioned)
    ///
    /// # Returns
    ///
    /// An IndexMap mapping file paths to their merged MergeValue.
    ///
    /// # Errors
    ///
    /// Returns `JinError::InvalidLayer` if any layer is UserLocal or WorkspaceActive.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let layers = vec![
    ///     Layer::GlobalBase,
    ///     Layer::ProjectBase { project: "myproject".to_string() },
    /// ];
    /// let merger = LayerMerge::new(&repo, "myproject");
    /// let result = merger.merge_subset(&layers)?;
    /// ```
    pub fn merge_subset(&self, layers: &[Layer]) -> Result<IndexMap<String, MergeValue>> {
        // Verify all layers are versioned
        for layer in layers {
            if !layer.is_versioned() {
                return Err(JinError::InvalidLayer {
                    name: format!("{:?}", layer),
                });
            }
        }

        let mut context = MergeContext::new();

        for layer in layers {
            // Read layer files
            let files = self.read_layer_files(layer)?;

            // If layer is empty, skip it
            if files.is_empty() {
                continue;
            }

            // Merge each file
            for (path, content) in files {
                match self.parse_file_by_format(&path, &content) {
                    Ok(value) => {
                        context.merge_file(path, value);
                    }
                    Err(e) => {
                        context.add_parse_error(path, e.to_string());
                    }
                }
            }

            context.add_layer(layer.clone());
        }

        Ok(context.get_result())
    }
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ===== FileFormat Tests =====

    #[test]
    fn test_file_format_from_path() {
        assert_eq!(FileFormat::from_path(Path::new("config.json")), FileFormat::Json);
        assert_eq!(FileFormat::from_path(Path::new("settings.yaml")), FileFormat::Yaml);
        assert_eq!(FileFormat::from_path(Path::new("config.yml")), FileFormat::Yaml);
        assert_eq!(FileFormat::from_path(Path::new("app.toml")), FileFormat::Toml);
        assert_eq!(FileFormat::from_path(Path::new("setup.ini")), FileFormat::Ini);
        assert_eq!(FileFormat::from_path(Path::new("README.md")), FileFormat::Text);
        assert_eq!(FileFormat::from_path(Path::new("data.unknown")), FileFormat::Text);
        assert_eq!(FileFormat::from_path(Path::new("noextension")), FileFormat::Text);
    }

    #[test]
    fn test_file_format_is_structured() {
        assert!(FileFormat::Json.is_structured());
        assert!(FileFormat::Yaml.is_structured());
        assert!(FileFormat::Toml.is_structured());
        assert!(FileFormat::Ini.is_structured());
        assert!(!FileFormat::Text.is_structured());
        assert!(!FileFormat::Unknown.is_structured());
    }

    // ===== MergeContext Tests =====

    #[test]
    fn test_merge_context_new() {
        let ctx = MergeContext::new();
        assert!(ctx.merged_files.is_empty());
        assert!(ctx.merged_layers().is_empty());
        assert!(ctx.parse_errors().is_empty());
    }

    #[test]
    fn test_merge_context_merge_file_new() {
        let mut ctx = MergeContext::new();
        ctx.merge_file("config.json".to_string(), MergeValue::Integer(42));

        assert_eq!(ctx.merged_files.len(), 1);
        assert_eq!(ctx.merged_files.get("config.json"), Some(&MergeValue::Integer(42)));
    }

    #[test]
    fn test_merge_context_merge_file_deep_merge() {
        let mut ctx = MergeContext::new();

        // Add first file
        let base = MergeValue::from_json(r#"{"a": {"x": 1}, "b": 2}"#).unwrap();
        ctx.merge_file("config.json".to_string(), base);

        // Merge second file
        let override_val = MergeValue::from_json(r#"{"a": {"y": 2}}"#).unwrap();
        ctx.merge_file("config.json".to_string(), override_val);

        // Check deep merge occurred
        let result = ctx.merged_files.get("config.json").unwrap();
        let obj = result.as_object().unwrap();
        let a_obj = obj.get("a").unwrap().as_object().unwrap();
        assert_eq!(a_obj.get("x").and_then(|v| v.as_i64()), Some(1));
        assert_eq!(a_obj.get("y").and_then(|v| v.as_i64()), Some(2));
        assert_eq!(obj.get("b").and_then(|v| v.as_i64()), Some(2));
    }

    #[test]
    fn test_merge_context_add_parse_error() {
        let mut ctx = MergeContext::new();
        ctx.add_parse_error("config.json".to_string(), "Invalid JSON".to_string());

        assert_eq!(ctx.parse_errors().len(), 1);
        assert_eq!(
            ctx.parse_errors()[0],
            ("config.json".to_string(), "Invalid JSON".to_string())
        );
    }

    #[test]
    fn test_merge_context_add_layer() {
        let mut ctx = MergeContext::new();
        ctx.add_layer(Layer::GlobalBase);
        ctx.add_layer(Layer::ProjectBase {
            project: "test".to_string(),
        });

        assert_eq!(ctx.merged_layers().len(), 2);
    }

    #[test]
    fn test_merge_context_get_result() {
        let mut ctx = MergeContext::new();
        ctx.merge_file("a.txt".to_string(), MergeValue::Integer(1));
        ctx.merge_file("b.txt".to_string(), MergeValue::Integer(2));

        let result = ctx.get_result();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_merge_context_default() {
        let ctx = MergeContext::default();
        assert!(ctx.merged_files.is_empty());
        assert!(ctx.merged_layers().is_empty());
        assert!(ctx.parse_errors().is_empty());
    }

    // ===== LayerMerge Tests =====

    #[test]
    fn test_layer_merge_new() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject");

        assert_eq!(merger.project, "testproject");
        assert!(merger.mode.is_none());
        assert!(merger.scope.is_none());
    }

    #[test]
    fn test_layer_merge_with_mode() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject").with_mode("claude");

        assert_eq!(merger.mode, Some("claude".to_string()));
    }

    #[test]
    fn test_layer_merge_with_scope() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject").with_scope("language:javascript");

        assert_eq!(merger.scope, Some("language:javascript".to_string()));
    }

    #[test]
    fn test_layer_merge_with_mode_and_scope() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject")
            .with_mode("claude")
            .with_scope("language:javascript");

        assert_eq!(merger.mode, Some("claude".to_string()));
        assert_eq!(merger.scope, Some("language:javascript".to_string()));
    }

    // ===== determine_active_layers Tests =====

    #[test]
    fn test_determine_active_layers_base_only() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject");

        let layers = merger.determine_active_layers();

        // Should have GlobalBase and ProjectBase
        assert_eq!(layers.len(), 2);
        assert!(layers.contains(&Layer::GlobalBase));
        assert!(layers.contains(&Layer::ProjectBase {
            project: "testproject".to_string()
        }));
    }

    #[test]
    fn test_determine_active_layers_with_mode() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject").with_mode("claude");

        let layers = merger.determine_active_layers();

        // Should have GlobalBase, ProjectBase, ModeBase, ModeProject
        assert_eq!(layers.len(), 4);
        assert!(layers.contains(&Layer::GlobalBase));
        assert!(layers.contains(&Layer::ProjectBase {
            project: "testproject".to_string()
        }));
        assert!(layers.contains(&Layer::ModeBase {
            mode: "claude".to_string()
        }));
        assert!(layers.contains(&Layer::ModeProject {
            mode: "claude".to_string(),
            project: "testproject".to_string()
        }));
    }

    #[test]
    fn test_determine_active_layers_with_scope() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject").with_scope("python");

        let layers = merger.determine_active_layers();

        // Should have GlobalBase, ProjectBase, ScopeBase (untethered)
        assert_eq!(layers.len(), 3);
        assert!(layers.contains(&Layer::GlobalBase));
        assert!(layers.contains(&Layer::ProjectBase {
            project: "testproject".to_string()
        }));
        assert!(layers.contains(&Layer::ScopeBase {
            scope: "python".to_string()
        }));
    }

    #[test]
    fn test_determine_active_layers_with_mode_and_scope() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject")
            .with_mode("claude")
            .with_scope("javascript");

        let layers = merger.determine_active_layers();

        // Should have mode-bound scope, NOT untethered scope
        assert!(layers.contains(&Layer::ModeScope {
            mode: "claude".to_string(),
            scope: "javascript".to_string()
        }));
        assert!(layers.contains(&Layer::ModeScopeProject {
            mode: "claude".to_string(),
            scope: "javascript".to_string(),
            project: "testproject".to_string()
        }));

        // Should NOT have untethered ScopeBase
        assert!(!layers.contains(&Layer::ScopeBase {
            scope: "javascript".to_string()
        }));
    }

    #[test]
    fn test_determine_active_layers_precedence_ordering() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject")
            .with_mode("claude")
            .with_scope("javascript");

        let layers = merger.determine_active_layers();

        // Verify ordering: GlobalBase should be first (lowest precedence)
        assert_eq!(layers[0], Layer::GlobalBase);

        // Each subsequent layer should have higher precedence
        for i in 1..layers.len() {
            assert!(layers[i - 1] < layers[i]);
        }
    }

    // ===== read_layer_files Tests =====

    #[test]
    fn test_read_layer_files_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject");

        let files = merger.read_layer_files(&Layer::GlobalBase).unwrap();
        assert!(files.is_empty());
    }

    // ===== parse_file_by_format Tests =====

    #[test]
    fn test_parse_file_by_format_json() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject");

        let content = br#"{"key": "value", "number": 42}"#;
        let result = merger.parse_file_by_format("config.json", content).unwrap();

        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("key").and_then(|v| v.as_str()), Some("value"));
        assert_eq!(obj.get("number").and_then(|v| v.as_i64()), Some(42));
    }

    #[test]
    fn test_parse_file_by_format_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject");

        let content = b"key: value\nnumber: 42";
        let result = merger.parse_file_by_format("config.yaml", content).unwrap();

        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("key").and_then(|v| v.as_str()), Some("value"));
        assert_eq!(obj.get("number").and_then(|v| v.as_i64()), Some(42));
    }

    #[test]
    fn test_parse_file_by_format_toml() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject");

        let content = b"key = \"value\"\nnumber = 42";
        let result = merger.parse_file_by_format("config.toml", content).unwrap();

        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("key").and_then(|v| v.as_str()), Some("value"));
        assert_eq!(obj.get("number").and_then(|v| v.as_i64()), Some(42));
    }

    #[test]
    fn test_parse_file_by_format_ini() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject");

        let content = b"[section]\nkey = value\nnumber = 42";
        let result = merger.parse_file_by_format("config.ini", content).unwrap();

        assert!(result.is_object());
        let obj = result.as_object().unwrap();
        let section = obj.get("section").unwrap().as_object().unwrap();
        assert_eq!(section.get("key").and_then(|v| v.as_str()), Some("value"));
        assert_eq!(section.get("number").and_then(|v| v.as_str()), Some("42"));
    }

    #[test]
    fn test_parse_file_by_format_text() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject");

        let content = b"This is plain text content";
        let result = merger.parse_file_by_format("README.md", content).unwrap();

        assert_eq!(result, MergeValue::String("This is plain text content".to_string()));
    }

    #[test]
    fn test_parse_file_by_format_invalid_utf8() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject");

        let content = &[0xFF, 0xFE, 0xFD]; // Invalid UTF-8
        let result = merger.parse_file_by_format("config.json", content);

        assert!(matches!(result, Err(JinError::Message(_))));
    }

    // ===== merge_all Tests =====

    #[test]
    fn test_merge_all_empty_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject");

        let result = merger.merge_all().unwrap();
        assert!(result.is_empty());
    }

    // ===== merge_subset Tests =====

    #[test]
    fn test_merge_subset_unversioned_layer_error() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject");

        let layers = vec![Layer::UserLocal];
        let result = merger.merge_subset(&layers);

        assert!(matches!(result, Err(JinError::InvalidLayer { .. })));
    }

    #[test]
    fn test_merge_subset_workspaceactive_error() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject");

        let layers = vec![Layer::WorkspaceActive];
        let result = merger.merge_subset(&layers);

        assert!(matches!(result, Err(JinError::InvalidLayer { .. })));
    }

    #[test]
    fn test_merge_subset_empty_layers() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let merger = LayerMerge::new(&repo, "testproject");

        let layers = vec![Layer::GlobalBase];
        let result = merger.merge_subset(&layers).unwrap();

        assert!(result.is_empty());
    }
}
