# P3.M1 Staging System - Research Summary

## Research Sources

### Git Staging Patterns
- Git Index Format Documentation: https://git-scm.com/docs/index-format
- git2-rs Documentation: https://docs.rs/git2
- Git Objects Documentation: https://git-scm.com/book/en/v2/Git-Internals-Git-Objects

### Layer Routing Patterns
- Kubernetes Kubeconfig Management
- Terraform Workspace System
- Docker Compose Override Merging
- Cobra + Viper Configuration Precedence

### Rust Implementation Patterns
- git2 crate for blob creation and OID handling
- serde_json for staging index persistence
- thiserror for custom error types
- tempfile for test isolation
- indexmap for ordered collections

## Key Insights

### 1. Git Blob Creation
- Use `repo.blob(content)` to create blobs from memory
- Jin's bare repository at `~/.jin/` stores objects
- Read file content from WORKSPACE, store in JIN REPO
- Content hash is 40-character hex string (SHA-1)

### 2. Layer Routing
- PRD Section 9.1 defines complete routing table
- Existing `router.rs` implements routing logic fully
- RoutingOptions + ProjectContext → Layer determination
- Validation prevents invalid flag combinations

### 3. Staging Index
- JSON-based persistence at `.jin/staging/index.json`
- HashMap<PathBuf, StagedEntry> storage
- Consider IndexMap for deterministic ordering
- Atomic writes using temp file + rename pattern

### 4. .gitignore Management
- PRD Section 8.1 defines managed block format
- Markers: `# --- JIN MANAGED START/END ---`
- Never edit outside managed block
- Auto-deduplicate entries

### 5. File Validation
- Check file exists before staging
- Reject symlinks (PRD Section 19.3)
- Reject Git-tracked files (use `jin import` instead)
- Detect executable bit for file mode

## Implementation Patterns

### Blob Creation Pattern
```rust
let content = std::fs::read(workspace_path)?;  // Read from workspace
let oid = jin_repo.create_blob(&content)?;     // Store in Jin's bare repo
let hash = oid.to_string();                    // 40-char hex string
```

### Git Tracking Detection Pattern
```rust
let project_repo = git2::Repository::discover(".")?;
let index = project_repo.index()?;
let is_tracked = index.get_path(rel_path, 0).is_some();
```

### File Mode Detection Pattern
```rust
#[cfg(unix)]
fn get_file_mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    match std::fs::metadata(path) {
        Ok(m) if m.permissions().mode() & 0o111 != 0 => 0o100755,
        _ => 0o100644,
    }
}
```

## Critical Gotchas

1. **Jin repo is BARE** - no working directory, must read from workspace
2. **Check Git in PROJECT repo** - not Jin repo
3. **Platform differences** - Windows has no executable bit
4. **Path normalization** - case-sensitive on all platforms for consistency
5. **Directory staging** - stage individual files, not directories

## Test Patterns

- Use `tempfile::TempDir` for isolated tests
- Create test Git repos with `git2::Repository::init`
- Mock file system operations for edge cases
- Test all routing combinations from PRD Section 9.1
