# PRP: P3.M1.T1 - Evaluate Staging Index TODOs

---

## Goal

**Feature Goal**: Investigate and implement "proper loading" and "proper saving" for the `StagingIndex` module to ensure data integrity, crash recovery, and consistency with the rest of the codebase.

**Deliverable**: Enhanced `src/staging/index.rs` with atomic write pattern, proper error handling, and optional corruption recovery.

**Success Definition**:
- `StagingIndex::save()` uses atomic write pattern (temp file + rename)
- `StagingIndex::load()` has proper error handling and optional recovery
- Implementation matches patterns used in `JinMap` and `WorkspaceMetadata`
- All tests pass: `cargo test staging::index::`
- No data corruption on crash/interrupt during save
- Consistent error messages with the rest of the codebase

---

## User Persona

**Target User**: Developer working on the Jin codebase (internal quality improvement)

**Use Case**: During development, it was noticed that `src/staging/index.rs` has TODO comments for "proper loading" and "proper saving". This milestone ensures the staging index implementation follows the same robust patterns used elsewhere in the codebase.

**User Journey**:
1. Developer reviews TODO comments in `src/staging/index.rs`
2. Developer identifies gaps in current implementation
3. Developer implements improvements following codebase patterns
4. Developer verifies improvements with tests
5. Developer marks TODOs as resolved

**Pain Points Addressed**:
- Risk of data corruption if process crashes during save
- Inconsistent error handling compared to other index files
- No recovery mechanism for malformed JSON

---

## Why

- **Code Quality**: Current implementation lacks atomic writes used elsewhere (JinMap, WorkspaceMetadata)
- **Data Integrity**: Direct write to `index.json` can cause corruption on crash/power loss
- **Consistency**: Other index files in the codebase use temp file + rename pattern
- **Reliability**: Staging index is critical for the commit workflow - corruption = lost work

---

## What

### Current TODO Comments

```rust
// Line 34 in src/staging/index.rs
/// TODO: Implement proper loading in later milestone
pub fn load() -> Result<Self>

// Line 50 in src/staging/index.rs
/// TODO: Implement proper saving in later milestone
pub fn save(&self) -> Result<()>
```

### Current Implementation Issues

| Issue | Current | Desired |
|-------|---------|---------|
| **Atomic Writes** | Direct write with `std::fs::write` | Temp file + rename pattern |
| **IO Error Handling** | `?` operator not used on `std::fs::write` | Explicit `.map_err(JinError::Io)?` |
| **Corruption Recovery** | None | Optional recovery for malformed JSON |
| **Consistency** | Different from JinMap/WorkspaceMetadata | Same pattern as other index files |

### Success Criteria

- [ ] `save()` uses temp file in same directory as target
- [ ] `save()` writes to temp file, then renames atomically
- [ ] `load()` has proper error handling for all IO operations
- [ ] Implementation follows JinMap pattern (src/core/jinmap.rs:124-132)
- [ ] All existing tests continue to pass
- [ ] New tests for atomic write behavior
- [ ] TODO comments are removed or updated

---

## All Needed Context

### Context Completeness Check

_This PRP provides everything needed to implement the StagingIndex improvements. An AI agent with access to this PRP and the codebase can implement the feature in one pass._

### Documentation & References

```yaml
# MUST READ - The File to Modify

- file: src/staging/index.rs
  why: Target file containing the TODOs - needs enhancement
  lines: 130 total (lines 33-62 contain load/save methods)
  pattern: |
    Current load(): Reads JSON directly, returns default if not found
    Current save(): Creates directory, writes JSON directly to file
    Missing: Atomic write pattern, IO error handling on write
  critical: This is the ONLY file to modify for this task
  todo_locations:
    - line: 34
      text: "TODO: Implement proper loading in later milestone"
    - line: 50
      text: "TODO: Implement proper saving in later milestone"

# MUST READ - Patterns to Follow

- file: src/core/jinmap.rs
  why: Reference implementation for atomic write pattern
  lines: 124-132 (save method with temp file + rename)
  pattern: |
    let temp_path = path.with_file_name(format!(
        "{}.tmp",
        path.file_name().unwrap().to_string_lossy()
    ));
    std::fs::write(&temp_path, content).map_err(JinError::Io)?;
    std::fs::rename(&temp_path, &path).map_err(JinError::Io)?;
  gotcha: Uses `with_file_name()` for dotfiles like `.jinmap`
  critical: This is the PRIMARY pattern to follow for atomic writes

- file: src/staging/metadata.rs
  why: Secondary reference for atomic write pattern
  lines: 68-71 (WorkspaceMetadata::save method)
  pattern: |
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, content)?;
    std::fs::rename(&temp_path, &path)?;
  gotcha: Uses `with_extension("tmp")` for regular files
  note: Similar pattern but different temp file naming approach

- file: src/staging/entry.rs
  why: StagedEntry structure used in StagingIndex
  pattern: |
    StagedEntry { path, target_layer, content_hash, mode, operation }
    No changes needed - just for context

# MUST READ - Error Handling

- file: src/core/error.rs
  why: JinError types for error handling
  lines: 102
  pattern: |
    JinError::Io - for I/O errors
    JinError::Parse { format, message } - for JSON parsing errors
  critical: Use `.map_err(JinError::Io)?` for IO operations

# EXTERNAL RESEARCH - Stored in plan/P3M1T1/research/

- url: https://docs.rs/tempfile/latest/tempfile/
  why: Rust tempfile crate for atomic writes (alternative approach)
  note: Codebase uses manual temp file pattern instead of tempfile crate

- url: https://doc.rust-lang.org/std/fs/fn.rename.html
  why: Rust rename() is atomic on same filesystem
  critical: This is why temp file must be in same directory

- url: https://medium.com/@travjohs/rusts-quiet-foot-gun-when-drop-hides-your-write-errors-and-how-to-fix-it-41de57e8ade3
  why: Important article about write error handling
  critical: Explains why explicit error handling is needed

- url: https://serde.rs/error-handling.html
  why: Serde error handling best practices
  note: Current error handling is adequate, optional enhancement
```

### Current Codebase Tree

```bash
jin/
├── src/
│   ├── staging/
│   │   ├── entry.rs          # StagedEntry struct (complete, no changes)
│   │   ├── index.rs          # TARGET FILE - StagingIndex with TODOs
│   │   ├── metadata.rs       # WorkspaceMetadata with atomic writes (reference)
│   │   └── mod.rs            # Module exports
│   ├── core/
│   │   ├── error.rs          # JinError types
│   │   ├── jinmap.rs         # JinMap with atomic writes (reference)
│   │   └── mod.rs
│   └── ...
├── tests/
│   ├── integration/
│   │   └── ...               # Existing integration tests
│   └── ...
└── plan/
    └── P3M1T1/
        ├── PRP.md            # This file
        └── research/         # Research documents directory
```

### Desired Codebase Tree After P3.M1.T1

```bash
jin/
├── src/
│   └── staging/
│       └── index.rs          # ENHANCED (~150 lines):
│           ├── save() - with atomic write pattern
│           ├── load() - with enhanced error handling
│           ├── Optional: recover() - for corrupted JSON
│           └── TODO comments removed/resolved
```

### Known Gotchas & Library Quirks

```rust
// ============================================================
// CRITICAL: Atomic rename requires same filesystem
// ============================================================
// The temp file MUST be created in the same directory as the target
// to ensure the rename() operation is atomic.
//
// CORRECT:
// let temp_path = path.with_extension("tmp");  // Same directory
//
// WRONG:
// let temp_path = PathBuf::from("/tmp/index.tmp");  // Different filesystem!
// std::fs::rename(&temp_path, &path)?;  // NOT atomic, may fail!

// ============================================================
// GOTCHA: Two different temp file naming patterns in codebase
// ============================================================
// JinMap uses: with_file_name() + ".tmp" suffix
//   let temp_path = path.with_file_name(format!("{}.tmp", name));
//
// WorkspaceMetadata uses: with_extension("tmp")
//   let temp_path = path.with_extension("tmp");
//
// For StagingIndex (.jin/staging/index.json), use:
//   let temp_path = path.with_extension("tmp");
// This produces: .jin/staging/index.json.tmp

// ============================================================
// CRITICAL: Explicit error handling on write operations
// ============================================================
// Current code has a bug - missing .map_err() on std::fs::write:
//
// CURRENT (buggy):
// std::fs::write(path, content)?;
//
// CORRECT (from JinMap):
// std::fs::write(&temp_path, content).map_err(JinError::Io)?;
//
// Why: If write fails, we want a clear IO error, not a generic error

// ============================================================
// GOTCHA: serde_json::from_str already has good error messages
// ============================================================
// Current load() error handling is actually fine:
// serde_json::from_str(&content).map_err(|e| JinError::Parse {
//     format: "JSON".to_string(),
//     message: e.to_string(),
// })
//
// Enhancement options (optional):
// - Add recovery for trailing commas (not needed for serde_json)
// - Add backup/restore mechanism
// - Add version migration support

// ============================================================
// GOTCHA: Directory creation is already correct
// ============================================================
// Current code properly creates parent directory:
// if let Some(parent) = path.parent() {
//     std::fs::create_dir_all(parent)?;
// }
//
// This is CORRECT - no changes needed.

// ============================================================
// TESTING: Crash recovery requires manual testing
// ============================================================
// To truly test atomic writes, you need to simulate a crash:
// 1. Write a test that creates a StagingIndex
// 2. Mock a failure during save (hard to do in unit test)
// 3. Verify original file is unchanged
//
// Alternative: Integration test that:
// 1. Starts a save operation
// 2. Kills process mid-save (requires external process control)
// 3. Verifies file is not corrupted
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// No new data models needed - StagingIndex structure remains the same
// Only the load() and save() methods are enhanced

// Current structure (unchanged):
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StagingIndex {
    entries: HashMap<PathBuf, StagedEntry>,
    #[serde(default = "default_version")]
    version: u32,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ANALYZE current implementation and gap documentation
  - READ: src/staging/index.rs lines 33-62
  - IDENTIFY: Specific differences from JinMap pattern
  - DOCUMENT: What changes are needed for "proper" loading/saving
  - OUTPUT: Create research summary in plan/P3M1T1/research/

Task 2: IMPLEMENT atomic write pattern in save() method
  - MODIFY: src/staging/index.rs, save() method (lines 48-62)
  - FOLLOW: JinMap pattern from src/core/jinmap.rs:124-132
  - IMPLEMENT: |
    1. Create temp path using path.with_extension("tmp")
    2. Write content to temp file with .map_err(JinError::Io)?
    3. Rename temp file to target path with .map_err(JinError::Io)?
  - PRESERVE: Directory creation logic (lines 53-55)
  - PRESERVE: JSON serialization logic (lines 56-59)
  - TEST: Verify atomic write behavior

Task 3: ENHANCE error handling in load() method
  - REVIEW: src/staging/index.rs, load() method (lines 33-46)
  - VERIFY: All IO operations have proper error handling
  - OPTIONAL: Add corruption recovery for malformed JSON
  - DECISION: If corruption recovery is added, document the approach
  - TEST: Test load with valid, missing, and corrupted files

Task 4: ADD or UPDATE tests for new behavior
  - VERIFY: Existing unit tests still pass (lines 131-208)
  - ADD: Test for save() creating temp file then renaming
  - ADD: Test for load() handling corrupted JSON (if recovery added)
  - FOLLOW: Test patterns in tests/integration/atomic_operations.rs
  - RUN: cargo test staging::index::

Task 5: REMOVE or UPDATE TODO comments
  - DECIDE: Remove TODOs if implementation is complete
  - OR: Update TODOs if additional work is identified
  - DOCUMENT: Any remaining work in plan/P3M1T1/research/
  - COMMIT: Clear commit message describing changes

Task 6: VALIDATE with full test suite
  - RUN: cargo test
  - RUN: cargo clippy -- -D warnings
  - RUN: cargo fmt -- --check
  - VERIFY: All tests pass, no new warnings
```

### Implementation Patterns & Key Details

```rust
// ================== CURRENT IMPLEMENTATION (lines 48-62) ==================
// TODO: Implement proper saving in later milestone
pub fn save(&self) -> Result<()> {
    let path = Self::default_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(self).map_err(|e| JinError::Parse {
        format: "JSON".to_string(),
        message: e.to_string(),
    })?;
    std::fs::write(path, content)?;  // BUG: Missing .map_err(JinError::Io)?
    Ok(())
}

// ================== DESIRED IMPLEMENTATION ==================
// Follows JinMap pattern from src/core/jinmap.rs:124-132
pub fn save(&self) -> Result<()> {
    let path = Self::default_path();

    // Create parent directory if needed (existing code - keep)
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(JinError::Io)?;
    }

    // Serialize to JSON (existing code - keep)
    let content = serde_json::to_string_pretty(self).map_err(|e| JinError::Parse {
        format: "JSON".to_string(),
        message: e.to_string(),
    })?;

    // === NEW: Atomic write pattern ===
    // Create temp file in same directory (ensures same filesystem)
    let temp_path = path.with_extension("tmp");

    // Write to temp file with explicit error handling
    std::fs::write(&temp_path, content).map_err(JinError::Io)?;

    // Atomic rename (overwrites existing file)
    std::fs::rename(&temp_path, &path).map_err(JinError::Io)?;

    Ok(())
}

// ================== CURRENT LOAD (lines 33-46) ==================
// TODO: Implement proper loading in later milestone
pub fn load() -> Result<Self> {
    let path = Self::default_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path)?;  // Missing error handling
        serde_json::from_str(&content).map_err(|e| JinError::Parse {
            format: "JSON".to_string",
            message: e.to_string(),
        })
    } else {
        Ok(Self::new())
    }
}

// ================== ENHANCED LOAD (minimal changes) ==================
pub fn load() -> Result<Self> {
    let path = Self::default_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path).map_err(JinError::Io)?;
        serde_json::from_str(&content).map_err(|e| JinError::Parse {
            format: "JSON".to_string(),
            message: e.to_string(),
        })
    } else {
        Ok(Self::new())
    }
}

// ================== OPTIONAL: Corruption Recovery ==================
// Only implement if analysis determines it's needed
pub fn load_with_recovery() -> Result<Self> {
    let path = Self::default_path();
    if !path.exists() {
        return Ok(Self::new());
    }

    let content = std::fs::read_to_string(&path).map_err(JinError::Io)?;

    // Try normal deserialization first
    match serde_json::from_str::<Self>(&content) {
        Ok(index) => Ok(index),
        Err(parse_err) => {
            // Attempt recovery for common corruption scenarios
            // This is optional and depends on analysis findings
            Err(JinError::Parse {
                format: "JSON".to_string(),
                message: format!("Failed to load staging index: {}", parse_err),
            })
        }
    }
}
```

### Integration Points

```yaml
NO NEW DEPENDENCIES NEEDED:
  - All required dependencies already in Cargo.toml
  - serde_json for JSON serialization
  - std::fs for file operations
  - No external crates needed

FILES TO MODIFY:
  - src/staging/index.rs (ONLY file to modify)

FILES TO READ (for reference only):
  - src/core/jinmap.rs (atomic write pattern reference)
  - src/staging/metadata.rs (secondary reference)
  - src/core/error.rs (error type reference)

FILES THAT USE StagingIndex (no changes needed):
  - src/commands/commit_cmd.rs
  - src/commands/status.rs
  - src/staging/mod.rs
  - tests/integration/*.rs

EXPORTS (no changes needed):
  - StagingIndex is already exported via src/staging/mod.rs
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after modifying save() method
cargo check                           # Type checking
cargo fmt -- --check                  # Format check
cargo clippy -- -D warnings           # Lint check

# Expected: Zero errors, zero warnings
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run StagingIndex tests
cargo test staging::index::           # All index tests

# Run specific tests
cargo test staging::index::test_staging_index_new
cargo test staging::index::test_staging_index_add_remove
cargo test staging::index::test_staging_index_entries_for_layer

# Expected: All existing tests pass
```

### Level 3: Manual Testing (Atomic Write Verification)

```bash
# Manual test to verify atomic write behavior
cd /tmp && mkdir jin-atomic-test && cd jin-atomic-test

# Setup
mkdir -p .jin/staging
echo '{"entries":{},"version":1}' > .jin/staging/index.json

# Create a test program that uses StagingIndex
cat > test_atomic.rs << 'EOF'
use std::path::PathBuf;
use jin::staging::StagingIndex;

fn main() {
    let mut index = StagingIndex::load().unwrap();
    // ... add entries ...
    index.save().unwrap();
}
EOF

# Run test and verify:
# 1. .jin/staging/index.json.tmp is created during save
# 2. .jin/staging/index.json.tmp is renamed to index.json
# 3. No intermediate corrupted state

# Cleanup
cd /tmp && rm -rf jin-atomic-test
```

### Level 4: Integration Testing

```bash
# Full test suite to ensure no regressions
cargo test

# Specific integration tests that use StagingIndex
cargo test commit::
cargo test status::

# Expected: All tests pass
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo clippy -- -D warnings` shows no warnings
- [ ] `cargo test staging::index::` all tests pass
- [ ] `cargo test` all tests pass (no regressions)

### Feature Validation

- [ ] `save()` uses atomic write pattern (temp file + rename)
- [ ] Temp file is created in same directory as target
- [ ] All IO operations have explicit `.map_err(JinError::Io)?`
- [ ] `load()` has proper error handling
- [ ] Implementation matches JinMap pattern

### Code Quality Validation

- [ ] Code follows existing patterns in codebase
- [ ] Error messages are consistent with JinError types
- [ ] No unwrap() in library code
- [ ] Doc comments updated if needed
- [ ] TODO comments removed or resolved

### Decision Documentation

- [ ] Document if corruption recovery was implemented and why
- [ ] Document any remaining work or future improvements
- [ ] Update plan/P3M1T1/research/ with findings

---

## Anti-Patterns to Avoid

- ❌ Don't create temp file in different directory (breaks atomic rename)
- ❌ Don't skip `.map_err(JinError::Io)?` on IO operations
- ❌ Don't change the StagingIndex data structure
- ❌ Don't modify methods other than `load()` and `save()`
- ❌ Don't add new dependencies (use std::fs)
- ❌ Don't remove existing functionality
- ❌ Don't change the JSON file format
- ❌ Don't modify TODOs without resolving them

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification:**
- Clear, focused scope: only enhance `load()` and `save()` methods
- Reference implementations available in codebase (JinMap, WorkspaceMetadata)
- No new data structures or dependencies needed
- Existing comprehensive test coverage
- Research provides specific patterns to follow
- Changes are localized to a single file
- Low risk: improvements to existing working code

**No significant risks identified.**

---

## Research Artifacts

### Summary of Findings

| Aspect | Finding | Action |
|--------|---------|--------|
| **Atomic Writes** | Not implemented | Add temp file + rename pattern |
| **IO Error Handling** | Missing on `std::fs::write` | Add `.map_err(JinError::Io)?` |
| **Load Error Handling** | Mostly complete | Add `.map_err(JinError::Io)?` on read |
| **Corruption Recovery** | Not implemented | Optional: evaluate need |
| **Consistency** | Differs from JinMap/WorkspaceMetadata | Follow JinMap pattern |

### Key Code Patterns Discovered

**JinMap Atomic Write Pattern** (src/core/jinmap.rs:124-132):
```rust
let temp_path = path.with_file_name(format!("{}.tmp", path.file_name().unwrap().to_string_lossy()));
std::fs::write(&temp_path, content).map_err(JinError::Io)?;
std::fs::rename(&temp_path, &path).map_err(JinError::Io)?;
```

**WorkspaceMetadata Atomic Write Pattern** (src/staging/metadata.rs:68-71):
```rust
let temp_path = path.with_extension("tmp");
std::fs::write(&temp_path, content)?;
std::fs::rename(&temp_path, &path)?;
```

### External References

- [Rust std::fs::rename documentation](https://doc.rust-lang.org/std/fs/fn.rename.html) - Atomic on same filesystem
- [Serde error handling](https://serde.rs/error-handling.html) - Best practices for JSON parsing
- [tempfile crate](https://docs.rs/tempfile/latest/tempfile/) - Alternative approach (not used in codebase)

---

## Appendix: Comparison Table

| Feature | Current StagingIndex | JinMap | WorkspaceMetadata | Target |
|---------|---------------------|--------|-------------------|--------|
| **Atomic Write** | No | Yes | Yes | Yes |
| **Temp File Pattern** | N/A | `.with_file_name() + .tmp` | `.with_extension("tmp")` | `.with_extension("tmp")` |
| **IO Error Handling** | Partial | Full | Partial | Full |
| **JSON Format** | Pretty | YAML | Pretty | Pretty (unchanged) |
| **Returns Default if Missing** | Yes | Yes | No | Yes (unchanged) |
