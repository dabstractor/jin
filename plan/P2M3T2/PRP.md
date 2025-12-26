# Product Requirement Prompt (PRP): Layer Merge Orchestration

**Task**: P2.M3.T2 - Layer Merge Orchestration

---

## Goal

**Feature Goal**: Implement orchestration logic that merges multiple Jin layers in precedence order to produce a consolidated workspace configuration.

**Deliverable**: A new `LayerMerge` orchestration module in `src/merge/layer.rs` that:
- Collects layer contents from Git tree references in precedence order
- Applies deep merge algorithm sequentially from lowest to highest priority layers
- Handles file format detection (JSON, YAML, TOML, INI, text) per file
- Returns the merged workspace configuration as `MergeValue`

**Success Definition**:
- All 9 layers can be merged in correct precedence order
- Files are parsed by format (JSON/YAML/TOML/INI) and deep-merged correctly
- Text files are handled separately (placeholder for P2.M4 - 3-way text merge)
- `cargo test --all` passes with zero failures
- `cargo clippy --all-targets -- -D warnings` produces no warnings
- Result is deterministic and reproducible

## Why

- **PRD §11.1 Merge Strategy**: Requires deterministic layered merges with precedence order
- **Core Feature**: Layer merging is the heart of Jin's configuration management system
- **User Experience**: Users expect changes from higher layers to override lower layers predictably
- **Integration Point**: This orchestrates P2.M3.T1 (Deep Merge) with P1.M2.T4 (Tree Walking) and Layer system
- **Foundation**: Required before P3 (Staging & Commit Pipeline) can apply changes to workspace

## What

Implement a `LayerMerge` orchestrator that:

### Input

- `JinRepo` reference for accessing Git tree data
- `project` name (for project-inferred layers)
- `mode` (optional, from active context)
- `scope` (optional, from active context)
- Layer set to merge (can be partial or full 9-layer hierarchy)

### Output

- `Result<MergeValue>` containing the merged workspace configuration
- Each file path maps to a `MergeValue` representing the merged content

### Merge Algorithm

1. **Determine active layers** based on project, mode, scope context
2. **Sort layers by precedence** (lower layer number = lower priority)
3. **For each layer in precedence order**:
   - Get layer ref from Git
   - Walk the layer's tree for file entries
   - Parse each file by format to `MergeValue`
   - Deep merge into accumulator using `MergeValue::merge()`
4. **Return final merged result**

### Layer Precedence (PRD §11.2)

| Layer | Description | Precedence |
|-------|-------------|------------|
| GlobalBase | Shared defaults | 1 (lowest) |
| ModeBase | Mode defaults | 2 |
| ModeScope | Scoped mode configs | 3 |
| ModeScopeProject | Project-scoped-mode | 4 |
| ModeProject | Mode-project | 5 |
| ScopeBase | Untethered scope | 6 |
| ProjectBase | Project-only | 7 |
| UserLocal | Machine overlays | 8 |
| WorkspaceActive | Derived result | 9 (highest) |

### Success Criteria

- [ ] `LayerMerge::merge_all()` merges full 9-layer stack correctly
- [ ] `LayerMerge::merge_subset()` merges partial layer set correctly
- [ ] JSON files are deep-merged with RFC 7396 semantics
- [ ] YAML files are deep-merged with RFC 7396 semantics
- [ ] TOML files are deep-merged with RFC 7396 semantics
- [ ] INI files are section-merged
- [ ] Text files return higher layer content (placeholder for 3-way merge)
- [ ] Non-existent layers are gracefully skipped
- [ ] Empty layers are handled correctly
- [ ] All tests pass including edge cases

---

## All Needed Context

### Context Completeness Check

**Question**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: Yes - this PRP provides:
- Exact file locations and line numbers for existing code
- Complete layer system understanding with 9-layer hierarchy
- Tree walking patterns from existing JinRepo implementation
- Deep merge algorithm from P2.M3.T1
- Test patterns and file locations
- PRD requirements with specific section references
- External research patterns for merge orchestration
- Code style and naming conventions

### Documentation & References

```yaml
# MUST READ - PRD Merge Rules
- file: /home/dustin/projects/jin-glm-doover/PRD.md
  why: Defines Jin's merge requirements, layer precedence, and scope rules
  section: "§11.1 Structured Merge Rules" (lines 260-273)
  section: "§11.2 Merge Priority" (line 277)
  section: "§10 Scope Precedence Rules" (lines 244-254)
  pattern: "Mode-bound scope > Untethered scope > Mode base"
  gotcha: Untethered scopes apply only if no mode-bound scope of same name exists

# MUST READ - Layer Type Definitions
- file: /home/dustin/projects/jin-glm-doover/src/core/layer.rs
  why: The complete 9-layer hierarchy with storage paths and Git refs
  section: Lines 1-118 contain Layer enum definition and hierarchy
  pattern: Layers have `git_ref()`, `storage_path()`, `mode()`, `scope()`, `project()` methods
  gotcha: Layers 1-7 are versioned, 8-9 (UserLocal, WorkspaceActive) are not

# MUST READ - Deep Merge Implementation
- file: /home/dustin/projects/jin-glm-doover/src/merge/value.rs
  why: The `MergeValue::merge()` method to use for layer merging
  section: Lines 385-512 contain merge methods
  pattern: `pub fn merge(&self, other: &MergeValue) -> Result<Self>`
  gotcha: Null values delete keys, arrays replace by default (RFC 7396)

# MUST READ - Tree Walking Patterns
- file: /home/dustin/projects/jin-glm-doover/src/git/repo.rs
  why: JinRepo has tree walking methods for reading layer contents
  section: Lines 809-912 contain `walk_tree()` and `list_tree_files()` methods
  pattern: `pub fn walk_tree<F>(&self, tree_id: git2::Oid, callback: F) -> Result<()>`
  gotcha: Uses iterative stack to avoid recursion depth issues

# MUST READ - Format Parsers
- file: /home/dustin/projects/jin-glm-doover/src/merge/value.rs
  why: Parse layer files to MergeValue based on format
  section: Lines 127-383 contain format parsers
  pattern: `MergeValue::from_json()`, `from_yaml()`, `from_toml()`, `from_ini()`
  gotcha: INI files are converted to nested objects with sections as keys

# MUST READ - Existing P2.M3.T1 PRP
- file: /home/dustin/projects/jin-glm-doover/plan/P2M3T1/PRP.md
  why: Shows how deep merge was implemented and tested
  section: All sections - for pattern reference
  pattern: Test naming, Result handling, merge algorithm structure

# EXTERNAL - RFC 7396 JSON Merge Patch
- url: https://www.rfc-editor.org/rfc/rfc7396
  why: Standard for JSON merge operations - null deletes keys, arrays replace
  critical: Section 3-4 specify merge semantics

# EXTERNAL - Rust Merge Patterns Research
- file: /home/dustin/projects/jin-glm-doover/rust_merge_patterns_research.md
  why: Rust-specific patterns for merge orchestration
  section: Iterator fold patterns, Result error handling, IndexMap usage
  pattern: `layers.into_iter().try_fold(MergedLayer::default(), ...)`

# EXTERNAL - Layer Merge Orchestration Research
- file: /home/dustin/projects/jin-glm-doover/merge_orchestration_research.md
  why: Industry patterns for multi-layer configuration merging
  section: Precedence-based merging, cascading configuration
  pattern: Sequential layer application with conflict resolution

# REFERENCE - JinRepo Layer Operations
- file: /home/dustin/projects/jin-glm-doover/src/git/repo.rs
  section: Lines 170-443 contain layer reference management
  pattern: `get_layer_ref()`, `layer_ref_exists()`, `list_layer_refs()`
```

### Current Codebase Tree

```bash
src/
├── core/
│   ├── error.rs       # JinError enum, Result type alias
│   ├── layer.rs       # Layer enum definitions (9-layer hierarchy)
│   ├── config.rs      # JinConfig, ProjectContext for active context
│   └── mod.rs
├── git/
│   ├── repo.rs        # JinRepo wrapper, tree walking methods
│   ├── transaction.rs # Transaction system for atomic commits
│   └── mod.rs
├── merge/
│   ├── mod.rs         # Module exports (MergeValue, ArrayMergeStrategy)
│   └── value.rs       # MergeValue enum, format parsers, merge() method
├── workspace/
│   └── mod.rs         # EMPTY - workspace operations go here
├── lib.rs             # Public API exports
└── main.rs            # CLI entry point

tests/
├── merge_test.rs      # Integration tests for merge functionality
└── ...
```

### Desired Codebase Tree with New Files

```bash
src/merge/
├── mod.rs             # MODIFY: export LayerMerge
├── value.rs           # EXISTING: MergeValue, merge(), format parsers
└── layer.rs           # NEW: LayerMerge orchestrator

tests/merge/
└── layer_merge_test.rs  # NEW: LayerMerge orchestration tests
```

**Design Decision**: Keep `LayerMerge` in `src/merge/layer.rs` (not `workspace/`) because:
1. It's core merge logic, not workspace application
2. Follows existing pattern: merge engine components go in `src/merge/`
3. `workspace/` module will use `LayerMerge` for applying results

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Layer 8 (UserLocal) and Layer 9 (WorkspaceActive) are NOT versioned
// They have no Git refs and should be skipped during Git tree reading
// Pattern: match layer.git_ref() { Some(ref) => ..., None => continue }

// CRITICAL: Scope precedence rules from PRD §10
// Mode-bound scope > Untethered scope > Mode base
// Untethered scopes apply ONLY if no mode-bound scope of same name exists
// Pattern: When building layer list, check if mode-bound scope exists first

// CRITICAL: Layer sorting uses Ord trait - declarations are in precedence order
// Layer enum variants are declared in order (1-9), so sorting is automatic
// Pattern: layers.sort() will put GlobalBase first, WorkspaceActive last

// CRITICAL: Tree walking uses iterative stack, not recursion
// JinRepo.walk_tree() uses Vec as stack to avoid stack overflow
// Pattern: while let Some((current_id, base_path)) = stack.pop() { ... }

// CRITICAL: Format detection must be based on file extension
// JSON -> .json, YAML -> .yaml/.yml, TOML -> .toml, INI -> .ini
// Pattern: match path.extension().and_then(|s| s.to_str()) { ... }

// CRITICAL: Empty tree handling - repo.find_tree() on empty tree returns valid Tree
// Empty trees have len() == 0 but are not an error
// Pattern: if tree.len() == 0 { continue } or handle gracefully

// GOTCHA: INI files convert to nested objects, not flat key-value
// Sections become top-level keys with nested values
// Pattern: [database] host=localhost -> {"database": {"host": "localhost"}}

// GOTCHA: Text files cannot be deep-merged like structured formats
// For now: higher layer completely replaces lower layer (P2.M4 will add 3-way merge)
// Pattern: Match file extension, if not structured format -> replace

// GOTCHA: Non-existent layer refs should be gracefully skipped
// get_layer_ref() returns Ok(None) if ref doesn't exist
// Pattern: let tree_id = match repo.get_layer_ref(&layer)? { ... }

// PATTERN: All merge operations return Result<T> for error propagation
// Use ? operator for clean error handling
// PATTERN: Test files use descriptive names: test_<feature>_<scenario>
```

---

## Implementation Blueprint

### Data Models and Structure

**1. LayerMerge Struct** (NEW)

```rust
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
```

**2. MergeContext Struct** (NEW)

```rust
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
}
```

**3. FileFormat Enum** (NEW)

```rust
/// File format for parsing and merging behavior.
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
    /// Detect format from file extension.
    pub fn from_path(path: &Path) -> Self { ... }
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/merge/layer.rs with FileFormat enum
  - IMPLEMENT: FileFormat enum with Json, Yaml, Toml, Ini, Text, Unknown variants
  - ADD: from_path() method for extension-based detection
  - ADD: is_structured() method to distinguish from text files
  - ADD: #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  - PLACEMENT: Top of new file src/merge/layer.rs
  - NAMING: PascalCase for enum, snake_case for methods

Task 2: CREATE MergeContext struct
  - IMPLEMENT: MergeContext struct with merged_files, merged_layers, parse_errors fields
  - ADD: new() constructor creating empty context
  - ADD: merge_file() method to merge a single file into context
  - ADD: get_result() method to return final merged IndexMap
  - ADD: add_parse_error() method to record parsing failures
  - PLACEMENT: In src/merge/layer.rs after FileFormat
  - PATTERN: Use IndexMap for ordered file paths

Task 3: IMPLEMENT LayerMerge struct
  - IMPLEMENT: LayerMerge struct with repo, project, mode, scope fields
  - ADD: new() constructor
  - ADD: with_project(), with_mode(), with_scope() builder methods
  - PLACEMENT: In src/merge/layer.rs after MergeContext
  - PATTERN: Builder-style methods for fluent configuration

Task 4: IMPLEMENT active layer determination
  - IMPLEMENT: determine_active_layers() method
  - ALGORITHM:
    1. Start with base layers: GlobalBase, ProjectBase { project }
    2. If mode set: add ModeBase { mode }, ModeProject { mode, project }
    3. If scope set:
       - Check if mode-bound scope exists (mode + scope)
       - If mode set and mode-bound scope exists: add ModeScope { mode, scope }
       - If mode set and mode-bound scope exists: add ModeScopeProject { mode, scope, project }
       - If no mode-bound scope exists: add ScopeBase { scope } (untethered)
    4. Add UserLocal at end (not read from Git, from filesystem)
  - RETURN: Vec<Layer> sorted by precedence (using Layer's Ord)
  - PLACEMENT: In src/merge/layer.rs as impl LayerMerge
  - GOTCHA: Must implement scope precedence rule correctly

Task 5: IMPLEMENT layer tree reading
  - IMPLEMENT: read_layer_files() method
  - ALGORITHM:
    1. Get layer ref using repo.get_layer_ref()
    2. If ref doesn't exist (Ok(None)), return empty HashMap
    3. Get target tree OID from ref
    4. Find tree by OID
    5. Walk tree using repo.walk_tree()
    6. For each blob entry: read blob content, store in HashMap
  - ERROR HANDLING: Return JinError for Git read failures
  - PLACEMENT: In src/merge/layer.rs as impl LayerMerge
  - PATTERN: Use existing repo.walk_tree() and repo.find_blob()

Task 6: IMPLEMENT file parsing by format
  - IMPLEMENT: parse_file_by_format() method
  - ALGORITHM:
    1. Detect FileFormat from path extension
    2. Match format:
       - Json: MergeValue::from_json(content)
       - Yaml: MergeValue::from_yaml(content)
       - Toml: MergeValue::from_toml(content)
       - Ini: MergeValue::from_ini(content)
       - Text/Unknown: Store as String (placeholder for 3-way merge)
    3. Return Result<MergeValue>
  - ERROR HANDLING: Return JinError::ParseError for invalid content
  - PLACEMENT: In src/merge/layer.rs as impl LayerMerge
  - PATTERN: Use existing MergeValue format parsers

Task 7: IMPLEMENT merge_all() orchestration method
  - IMPLEMENT: pub fn merge_all(&self) -> Result<IndexMap<String, MergeValue>>
  - ALGORITHM:
    1. Determine active layers using determine_active_layers()
    2. Create empty MergeContext
    3. For each layer in precedence order:
       a. Skip UserLocal and WorkspaceActive (no Git refs)
       b. Read layer files using read_layer_files()
       c. For each (path, content) in layer files:
          - Parse to MergeValue using parse_file_by_format()
          - Merge into context using context.merge_file(path, value)
    4. Return context.get_result()
  - ERROR HANDLING: Propagate Git and parse errors
  - PLACEMENT: In src/merge/layer.rs as impl LayerMerge
  - PATTERN: Sequential iteration, accumulating into MergeContext

Task 8: IMPLEMENT merge_subset() for partial layers
  - IMPLEMENT: pub fn merge_subset(&self, layers: &[Layer]) -> Result<IndexMap<String, MergeValue>>
  - ALGORITHM: Same as merge_all() but use provided layer list instead of determining
  - VALIDATION: Verify all layers are versioned (error on UserLocal/WorkspaceActive)
  - PLACEMENT: In src/merge/layer.rs as impl LayerMerge
  - USE CASE: Testing, partial merge operations

Task 9: EXPORT LayerMerge in module
  - MODIFY: src/merge/mod.rs
  - ADD: pub mod layer;
  - ADD: pub use layer::{LayerMerge, FileFormat, MergeContext};
  - ENSURE: Public API includes new types

Task 10: ADD comprehensive unit tests
  - CREATE: tests/merge/layer_merge_test.rs
  - TEST: merge_all() with empty layers
  - TEST: merge_all() with single layer
  - TEST: merge_all() with multiple layers (precedence verification)
  - TEST: merge_all() with all 9 layers
  - TEST: Format detection for all supported formats
  - TEST: JSON deep merge across layers
  - TEST: YAML deep merge across layers
  - TEST: TOML deep merge across layers
  - TEST: INI section merge across layers
  - TEST: Text file replacement (higher wins)
  - TEST: Non-existent layer handling
  - TEST: Scope precedence rules
  - TEST: Mode-only layer set
  - TEST: Scope-only layer set
  - TEST: Mode + scope combination
  - TEST: Parse error handling
  - FOLLOW: Existing test patterns from tests/merge_test.rs
  - NAMING: test_layer_merge_<scenario>
  - PLACEMENT: New file tests/merge/layer_merge_test.rs
```

### Implementation Patterns & Key Details

```rust
// ===== PATTERN: File Format Detection =====

impl FileFormat {
    /// Detect file format from path extension.
    pub fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|s| s.to_str()) {
            Some("json") => FileFormat::Json,
            Some("yaml") | Some("yml") => FileFormat::Yaml,
            Some("toml") => FileFormat::Toml,
            Some("ini") => FileFormat::Ini,
            _ => FileFormat::Text,  // Default to text for unknown formats
        }
    }

    /// Returns true if this format supports structured merging.
    pub fn is_structured(&self) -> bool {
        matches!(self, FileFormat::Json | FileFormat::Yaml | FileFormat::Toml | FileFormat::Ini)
    }
}

// ===== PATTERN: Active Layer Determination =====

impl<'a> LayerMerge<'a> {
    /// Determines the active layers based on project, mode, and scope context.
    fn determine_active_layers(&self) -> Vec<Layer> {
        let mut layers = vec![
            Layer::GlobalBase,
            Layer::ProjectBase { project: self.project.clone() },
        ];

        if let Some(ref mode) = self.mode {
            layers.push(Layer::ModeBase { mode: mode.clone() });
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
}

// ===== PATTERN: Layer File Reading =====

impl<'a> LayerMerge<'a> {
    /// Reads all files from a layer's Git tree.
    fn read_layer_files(&self, layer: &Layer) -> Result<HashMap<String, Vec<u8>>> {
        use std::collections::HashMap;

        // Get layer ref (returns None if doesn't exist)
        let reference = match self.repo.get_layer_ref(layer)? {
            Some(ref_) => ref_,
            None => return Ok(HashMap::new()), // Empty layer
        };

        // Get tree OID
        let tree_id = reference.target().ok_or_else(|| JinError::Message(
            format!("Layer {:?} has no target OID", layer)
        ))?;

        // Find the tree
        let tree = self.repo.find_tree(tree_id)?;

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
}

// ===== PATTERN: File Parsing =====

impl<'a> LayerMerge<'a> {
    /// Parses file content to MergeValue based on format.
    fn parse_file_by_format(&self, path: &str, content: &[u8]) -> Result<MergeValue> {
        let path_obj = Path::new(path);
        let format = FileFormat::from_path(path_obj);
        let content_str = std::str::from_utf8(content)
            .map_err(|_| JinError::ParseError {
                file: path.to_string(),
                message: "File is not valid UTF-8".to_string(),
            })?;

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
}

// ===== PATTERN: MergeContext Accumulation =====

impl MergeContext {
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

    /// Returns the final merged result.
    fn get_result(self) -> IndexMap<String, MergeValue> {
        self.merged_files
    }
}

// ===== PATTERN: Full Orchestration =====

impl<'a> LayerMerge<'a> {
    /// Merges all active layers in precedence order.
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

            context.merged_layers.push(layer.clone());
        }

        Ok(context.get_result())
    }
}

// ===== GOTCHA: Scope Precedence Rules =====

// PRD §10: Mode-bound scope > Untethered scope > Mode base
// Untethered scopes apply ONLY if no mode-bound scope of same name exists

fn determine_active_layers(&self) -> Vec<Layer> {
    let mut layers = vec![
        Layer::GlobalBase,
        Layer::ProjectBase { project: self.project.clone() },
    ];

    if let Some(ref mode) = self.mode {
        layers.push(Layer::ModeBase { mode: mode.clone() });
        layers.push(Layer::ModeProject {
            mode: mode.clone(),
            project: self.project.clone(),
        });
    }

    if let Some(ref scope) = self.scope {
        if let Some(ref mode) = self.mode {
            // Mode-bound scope (both mode and scope are active)
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
            // Untethered scope (only when no mode)
            layers.push(Layer::ScopeBase {
                scope: scope.clone(),
            });
        }
    }

    layers.sort(); // Layers have Ord, automatic precedence ordering
    layers
}

// ===== GOTCHA: Empty Layer Handling =====

fn read_layer_files(&self, layer: &Layer) -> Result<HashMap<String, Vec<u8>>> {
    let reference = match self.repo.get_layer_ref(layer)? {
        Some(ref_) => ref_,
        None => return Ok(HashMap::new()), // Gracefully handle non-existent layers
    };

    let tree_id = reference.target().ok_or_else(|| JinError::Message(
        format!("Layer {:?} reference has no target", layer)
    ))?;

    let tree = self.repo.find_tree(tree_id)?;

    if tree.len() == 0 {
        return Ok(HashMap::new()); // Empty tree is valid
    }

    // ... rest of tree walking
}

// ===== PATTERN: Test Structure =====

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::JinRepo;
    use tempfile::TempDir;

    #[test]
    fn test_layer_merge_precedence() {
        // Set up repo with multiple layers
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();

        // Create layer trees with overlapping files
        // ... (setup code)

        let merger = LayerMerge::new(&repo)
            .with_project("test-project")
            .with_mode("claude");

        let result = merger.merge_all().unwrap();

        // Verify higher layer values override lower layer values
        assert_eq!(result.get("config.json").unwrap().as_object().unwrap()
            .get("key").unwrap().as_str(), Some("higher-layer-value"));
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(FileFormat::from_path(Path::new("config.json")), FileFormat::Json);
        assert_eq!(FileFormat::from_path(Path::new("settings.yaml")), FileFormat::Yaml);
        assert_eq!(FileFormat::from_path(Path::new("config.yml")), FileFormat::Yaml);
        assert_eq!(FileFormat::from_path(Path::new("app.toml")), FileFormat::Toml);
        assert_eq!(FileFormat::from_path(Path::new("setup.ini")), FileFormat::Ini);
        assert_eq!(FileFormat::from_path(Path::new("README.md")), FileFormat::Text);
    }
}
```

### Integration Points

```yaml
NO DATABASE CHANGES: Layer merge operates on Git data only

NO CONFIG CHANGES: LayerMerge parameters are passed via builder methods

NO ROUTE CHANGES: This is library code, not HTTP-exposed

MODULE EXPORTS:
  - file: src/merge/mod.rs
  - add: pub mod layer;
  - add: pub use layer::{LayerMerge, FileFormat, MergeContext};
  - ensures: Public API includes new types

WORKSPACE MODULE (future):
  - file: src/workspace/mod.rs
  - will use: LayerMerge to get merged configuration
  - will apply: Write merged results to .jin/workspace/

TEST COVERAGE:
  - file: tests/merge/layer_merge_test.rs
  - add: ~20+ tests for orchestration behavior
  - ensures: All merge scenarios are covered
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Check after each file modification
cargo check                          # Fast compilation check
cargo clippy --all-targets -- -D warnings  # Lint checking

# Format check
cargo fmt -- --check

# Run together
cargo check && cargo clippy --all-targets -- -D warnings && cargo fmt -- --check

# Expected: Zero errors
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test layer merge module
cargo test --lib merge::layer

# Test with output
cargo test --lib merge::layer -- --nocapture

# Run specific test
cargo test test_layer_merge_precedence

# Full merge module tests
cargo test --package jin_glm --lib merge

# Expected: All tests pass
```

### Level 3: Integration Testing (System Validation)

```bash
# Run all tests
cargo test --all

# Integration tests
cargo test --test integration_test

# Merge integration tests
cargo test merge_test

# Expected: All integration tests pass
```

### Level 4: Layer Merge Validation

```bash
# Test specific merge scenarios
cargo test test_layer_merge_all_layers
cargo test test_layer_merge_precedence
cargo test test_layer_merge_scope_precedence

# Test format-specific merges
cargo test test_json_deep_merge_across_layers
cargo test test_yaml_deep_merge_across_layers
cargo test test_toml_deep_merge_across_layers
cargo test test_ini_section_merge_across_layers

# Test edge cases
cargo test test_layer_merge_empty_layer
cargo test test_layer_merge_nonexistent_layer
cargo test test_layer_merge_parse_error_handling

# Expected: All layer merge tests pass
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] `cargo test --all` passes with zero failures
- [ ] `cargo clippy --all-targets -- -D warnings` produces no warnings
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo check` completes without errors

### Feature Validation

- [ ] `merge_all()` correctly merges all 9 layers in precedence order
- [ ] `merge_subset()` correctly merges partial layer sets
- [ ] JSON files are deep-merged using RFC 7396 semantics
- [ ] YAML files are deep-merged using RFC 7396 semantics
- [ ] TOML files are deep-merged using RFC 7396 semantics
- [ ] INI files are section-merged correctly
- [ ] Text files use higher-layer replacement (placeholder)
- [ ] Non-existent layers are gracefully skipped
- [ ] Empty layers are handled correctly
- [ ] Scope precedence rules are implemented correctly

### Code Quality Validation

- [ ] Follows existing codebase patterns (Result returns, error handling)
- [ ] File placement matches desired codebase tree
- [ ] Naming conventions followed (PascalCase types, snake_case methods)
- [ ] Error handling uses appropriate JinError variants
- [ ] Public APIs have doc comments
- [ ] Tests follow existing patterns (descriptive names, clear assertions)

### Documentation & Deployment

- [ ] `LayerMerge` struct has complete doc comments
- [ ] `FileFormat` enum has usage examples
- [ ] `MergeContext` struct is documented
- [ ] New tests have descriptive names and assertion messages
- [ ] Layer precedence rules are documented in code comments

---

## Anti-Patterns to Avoid

- **Don't skip non-existent layers** - Must handle gracefully, not error
- **Don't ignore scope precedence** - Mode-bound scope > untethered scope
- **Don't forget UserLocal/WorkspaceActive** - These layers have no Git refs
- **Don't assume all files are structured** - Text files need special handling
- **Don't use recursion for tree walking** - Use existing iterative walk_tree()
- **Don't clone excessively** - Use references where possible
- **Don't ignore parse errors** - Record and propagate appropriately
- **Don't hardcode layer list** - Use determine_active_layers() method
- **Don't forget to sort layers** - Use Layer's Ord for precedence
- **Don't skip empty trees** - Empty trees are valid, not errors
- **Don't mix up mode-bound and untethered scopes** - They're mutually exclusive
- **Don't forget to export new types** - Update mod.rs with pub use

---

## Success Metrics

**Confidence Score**: 9/10 for one-pass implementation success

**Rationale**:
- All dependencies are already implemented (MergeValue, Layer, JinRepo, tree walking)
- Clear algorithm with well-defined steps
- Existing patterns to follow throughout codebase
- External research provides implementation guidance
- Test patterns are well-established

**Risk Mitigation**:
- Incremental implementation with clear task breakdown
- Comprehensive test coverage at each step
- Graceful error handling for edge cases
- Follows existing code patterns closely

**Validation**: The completed PRP provides sufficient context for an AI agent unfamiliar with the codebase to implement the feature successfully using only the PRP content and codebase access.
