# Apply Command Research

## Purpose

The `jin apply` command merges active layers (1-7) based on the current mode/scope context and writes the consolidated configuration to the workspace directory (`.jin/workspace/`, Layer 9: WorkspaceActive).

## Key Files & Patterns

### Command Pattern: `src/commands/add.rs`
- Uses `std::env::current_dir()` for workspace root
- Loads `ProjectContext` for active mode/scope
- Uses `detect_project_name()` helper for project name
- Follows Result<> error handling pattern

### Merge Engine: `src/merge/layer.rs`
- `LayerMerge::new(&repo, project)` - Create merger
- `.with_mode(mode)` - Set active mode
- `.with_scope(scope)` - Set active scope
- `.merge_all()` - Returns `IndexMap<String, MergeValue>` of merged files
- `.determine_active_layers()` - Returns sorted Vec<Layer> based on context

### Layer Definitions: `src/core/layer.rs`
- `Layer::WORKSPACE_PATH` = ".jin/workspace"
- `Layer::is_versioned()` - WorkspaceActive is NOT versioned
- Layer precedence: 1 (GlobalBase) to 7 (ProjectBase)

### MergeValue Serialization: `src/merge/value.rs`
- `MergeValue` represents any JSON/YAML/TOML/INI/Text value
- Need to serialize back to files based on format
- `FileFormat::from_path()` for format detection

## CLI Definition

`src/cli/args.rs:278-288`:
```rust
pub struct ApplyCommand {
    /// Skip dirty check and force apply
    #[arg(long)]
    pub force: bool,

    /// Show plan without applying
    #[arg(long)]
    pub dry_run: bool,
}
```

## Implementation Requirements

1. **Load context** - Get active mode/scope from `ProjectContext`
2. **Detect project** - Get project name from git origin or directory
3. **Open JinRepo** - Use `JinRepo::open_or_create()`
4. **Create LayerMerge** - With project, mode, scope
5. **Merge all layers** - Call `merge_all()` to get `IndexMap<String, MergeValue>`
6. **Serialize to workspace** - Write each MergeValue to `.jin/workspace/<path>`
7. **Handle flags**:
   - `--dry-run`: Show what would change without writing
   - `--force`: Skip dirty workspace check

## Workspace State

The workspace (`.jin/workspace/`) is a **derived** layer - the output of merging layers 1-7. It is never a source for commits; users commit to layers 1-7 via `jin add` and `jin commit`.

## File Format Serialization

For each merged file, need to:
1. Detect format from file extension (`.json`, `.yaml`, `.toml`, `.ini`, other)
2. Serialize `MergeValue` to appropriate format
3. Create directory structure under `.jin/workspace/`
4. Write file content

## Gotchas

1. **MergeValue serialization** - Not all formats have explicit to_* methods; may need to work via serde_json
2. **Directory creation** - Need `std::fs::create_dir_all()` for nested paths
3. **Relative paths** - Paths in merge result are relative to project root
4. **WorkspaceActive is NOT versioned** - Never call `repo.get_layer_ref(&Layer::WorkspaceActive)`
