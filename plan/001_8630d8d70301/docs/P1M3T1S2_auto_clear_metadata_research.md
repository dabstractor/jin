# Research Summary for P1.M3.T1.S2

## Overview

This document summarizes the research conducted for creating the PRP for P1.M3.T1.S2: "Compare modes and clear metadata if different".

## Research Sources

### 1. Codebase Analysis (Parallel Agents)

#### Agent: File Deletion Patterns
- **File**: `src/git/transaction.rs:335-345` - `delete_at()` function
- **Pattern**: Check `path.exists()` before `std::fs::remove_file()`, use `?` for error propagation
- **Additional examples**:
  - `src/commands/rm.rs:175` - Simple deletion with `?`
  - `src/commands/reset.rs:191` - Error collection pattern
  - `src/test_utils.rs:22-24` - Lock file cleanup with `let _ = ...`

#### Agent: WorkspaceMetadata Structure
- **Key Finding**: No dedicated "mode" field in WorkspaceMetadata
- **Mode storage**: Inferred from `applied_layers: Vec<String>` where mode layers are `"mode/<name>"`
- **Example**: `["global", "mode/claude", "mode/production/scope/backend"]`
- **Path**: `.jin/workspace/last_applied.json` or `$JIN_DIR/workspace/last_applied.json`

#### Agent: User Messaging Patterns
- **Info messages**: Use `println!()` (not `eprintln!()`)
- **Format**: Single quotes around names, actionable next steps
- **Examples**:
  - `"Activated mode '{}'"`
  - `"Stage files with: jin add --mode"`
  - `"Activate with: jin mode use {}"`

#### Agent: Mode Command Structure
- **Function**: `use_mode()` at `src/commands/mode.rs:86-130`
- **P1.M3.T1.S1 changes**: Loads `Option<WorkspaceMetadata>` at lines 119-124
- **Integration point**: Add comparison and clearing logic after metadata loading

#### Agent: Fix Specifications
- **Document**: `plan/docs/fix_specifications.md` - Fix 3: Mode Switching UX
- **Option A (Recommended)**: Auto-clear metadata in `ModeAction::Use` handler
- **Message format**: `"Cleared workspace metadata (mode changed). Run 'jin apply' to apply new mode."`

### 2. External Research (Parallel Agent)

#### Rust File Operations Best Practices
- **std::fs::remove_file()**: [Official docs](https://doc.rust-lang.org/std/fs/fn.remove_file.html)
- **Error handling**: [std::io::ErrorKind](https://doc.rust-lang.org/std/io/enum.ErrorKind.html)
- **Idiomatic pattern**: [Rust Users Forum](https://users.rust-lang.org/t/idiomatic-way-to-ignore-certain-kinds-of-errors/67814)
- **TOCTOU warning**: [std::fs docs](https://doc.rust-lang.org/std/fs/index.html)
- **Path::exists() issues**: [Internals discussion](https://internals.rust-lang.org/t/the-api-of-path-exists-encourages-broken-code/13817)

**Key Insights**:
- Don't check `path.exists()` before deletion (TOCTOU race condition)
- Handle `NotFound` gracefully via match on error kind
- Use `?` operator for error propagation
- `ErrorKind` is non-exhaustive - always include wildcard arm

### 3. Direct File Reading

#### `src/commands/mode.rs` (lines 86-130)
- Current `use_mode()` function with P1.M3.T1.S1 changes
- Metadata loaded as `_metadata` (underscore prefix)
- Ready for P1.M3.T1.S2 to add comparison logic

#### `src/staging/metadata.rs` (complete)
- `WorkspaceMetadata` struct definition
- `default_path()` method with JIN_DIR support
- `load()` method returns `Result<Self>` or `JinError::NotFound`

#### `src/core/error.rs` (lines 1-100)
- `JinError` enum with `#[from] std::io::Error`
- Auto-conversion of `std::io::Error` to `JinError::Io`

### 4. Test Patterns

#### `tests/mode_scope_workflow.rs`
- Test fixture pattern: `TestFixture::new()?`
- `fixture.set_jin_dir()` for JIN_DIR isolation
- Serial test attribute: `#[serial]`
- Pattern: Create modes, switch between them, verify behavior

## Key Findings

### Mode Extraction Logic
```rust
// Extract mode from applied_layers
fn get_metadata_mode(metadata: &WorkspaceMetadata) -> Option<String> {
    metadata.applied_layers
        .iter()
        .find(|layer| layer.starts_with("mode/"))
        .and_then(|layer| {
            layer
                .strip_prefix("mode/")
                .and_then(|s| s.split('/').next())
                .map(String::from)
        })
}
```

### File Deletion Pattern
```rust
// Codebase pattern (from transaction.rs:337)
if path.exists() {
    std::fs::remove_file(&path)?;
}
```

**Note**: External research says don't check exists() first (TOCTOU), but codebase consistently uses this pattern. Follow codebase convention for consistency.

### User Message Format
```rust
println!("Cleared workspace metadata (mode changed from '{}' to '{}').", old_mode, new_mode);
println!("Run 'jin apply' to apply new mode configuration.");
```

## Implementation Plan

1. **Rename**: `_metadata` → `metadata` (removing underscore prefix)
2. **Extract**: Mode from `metadata.applied_layers`
3. **Compare**: Extracted mode vs new mode (`name` parameter)
4. **Delete**: Metadata file if modes differ
5. **Inform**: User about metadata clearing and next steps

## Validation Commands

```bash
# Syntax check
cargo check
cargo fmt -- --check
cargo clippy -- -D warnings

# Unit tests
cargo test commands::mode::
cargo test mode_scope_workflow

# Manual test
cd $(mktemp -d) && git init && jin init
jin mode create mode1 && jin mode create mode2
echo "test" > config.txt && jin add --mode config.txt
jin commit -m "Add config" && jin apply
jin mode use mode2  # Should clear metadata
jin apply  # Should work without --force
```

## Confidence Score: 10/10

- Single file change (~15 lines)
- Well-established patterns throughout codebase
- No new types or dependencies
- Builds on P1.M3.T1.S1 output
- Comprehensive external research validation
