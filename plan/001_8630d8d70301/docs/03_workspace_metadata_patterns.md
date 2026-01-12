# Research: WorkspaceMetadata Usage Patterns

## File Location
**Path**: `/home/dustin/projects/jin/src/staging/metadata.rs`

## Struct Definition (lines 17-25)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    /// RFC3339 timestamp of when configuration was applied
    pub timestamp: String,
    /// Layer names that were merged and applied
    pub applied_layers: Vec<String>,
    /// Map of file paths to their content hashes (Git blob OID)
    pub files: HashMap<PathBuf, String>,
}
```

## Key Methods

### Loading Functions
```rust
pub fn load() -> Result<Self>
pub fn default_path() -> PathBuf
```

### Creation and Management
```rust
pub fn new() -> Self
impl Default for WorkspaceMetadata
pub fn update_timestamp(&mut self)
pub fn add_file(&mut self, path: PathBuf, content_hash: String)
pub fn remove_file(&mut self, path: &Path)
pub fn save(&self) -> Result<()>
```

## Path Resolution

```rust
pub fn default_path() -> PathBuf {
    // Check JIN_DIR environment variable first for test isolation
    if let Ok(jin_dir) = std::env::var("JIN_DIR") {
        return PathBuf::from(jin_dir)
            .join("workspace")
            .join("last_applied.json");
    }
    PathBuf::from(".jin")
        .join("workspace")
        .join("last_applied.json")
}
```

**Storage Locations**:
- Default: `.jin/workspace/last_applied.json`
- With JIN_DIR: `$JIN_DIR/workspace/last_applied.json`

## Common Loading Patterns

### Pattern 1: Handle missing metadata gracefully
```rust
let metadata = match WorkspaceMetadata::load() {
    Ok(meta) => meta,
    Err(JinError::NotFound(_)) => return Ok(()), // Fresh workspace
    Err(e) => return Err(e),
};
```

### Pattern 2: Optional metadata loading
```rust
let metadata = match WorkspaceMetadata::load() {
    Ok(meta) => Some(meta),
    Err(JinError::NotFound(_)) => None, // Fresh workspace - no metadata yet
    Err(e) => return Err(e),
};
```

## Applied Layer Formats

The `applied_layers` Vec contains strings like:
- `"global"` - No mode/scope
- `"mode/claude"` - Mode layer
- `"mode/production/scope/backend"` - Mode + scope
- `"scope/backend"` - Scope-only layer
- `"scope/frontend/api"` - Scope with sub-path

## Scope Extraction Pattern

```rust
fn get_metadata_scope(metadata: &WorkspaceMetadata) -> Option<String> {
    metadata.applied_layers
        .iter()
        .find(|layer| layer.starts_with("scope/") && !layer.starts_with("scope//"))
        .and_then(|layer| {
            layer
                .strip_prefix("scope/")
                .and_then(|s| s.split('/').next()) // Get first component after "scope/"
                .map(String::from)
        })
}
```

**Examples**:
- `"scope/backend"` → `"backend"`
- `"scope/frontend/api"` → `"frontend"`
- `"mode/production/scope/backend"` → Not matched (starts with "mode/")

## Error Handling

- **Loading returns**: `Result<WorkspaceMetadata>`
- **Not found**: `JinError::NotFound` - treated as fresh workspace
- **Parse errors**: `JinError::Parse` - should propagate
- **Save errors**: `JinError::Io` - file system errors

## Atomic Write Pattern

The `save()` method uses atomic write:
1. Write to `.tmp` file
2. Rename to final location
3. Ensure parent directory exists

## Usage in Commands

### Apply Command
```rust
metadata.applied_layers = config.layers.iter().map(|l| l.to_string()).collect();
for (path, merged_file) in &merged.merged_files {
    let oid = repo.create_blob(content.as_bytes())?;
    metadata.add_file(path.clone(), oid.to_string());
}
metadata.save()?;
```

### Mode Command (P1.M3.T1.S2)
```rust
let metadata_mode = meta.applied_layers
    .iter()
    .find(|layer| layer.starts_with("mode/"))
    .and_then(|layer| layer.strip_prefix("mode/"))
    .and_then(|s| s.split('/').next());
```
