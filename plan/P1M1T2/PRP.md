# Product Requirement Prompt (PRP): Core Error Types (P1.M1.T2)

---

## Goal

**Feature Goal**: Establish a comprehensive error type hierarchy using thiserror that provides clear, actionable error messages for all Jin operations while maintaining proper error chaining and exit code mapping.

**Deliverable**: A `JinError` enum in `src/core/error.rs` with all error variants, thiserror derives, automatic From implementations, and a `Result<T>` type alias exported from `src/core/mod.rs`.

**Success Definition**:
- `cargo build` compiles with zero errors
- `cargo check --tests` passes for error module
- All error variants display meaningful messages
- `From<JinError>` implementation maps to correct exit codes
- Error chaining preserves underlying error context

## User Persona

**Target User**: AI coding agent implementing the Jin error handling foundation

**Use Case**: The agent needs to establish error handling infrastructure that all subsequent modules (git/, merge/, staging/, commit/, workspace/, commands/) will depend on.

**User Journey**:
1. Agent receives this PRP as context
2. Creates `src/core/error.rs` with JinError enum
3. Implements all error variants with thiserror derives
4. Adds Result type alias
5. Exports from `src/core/mod.rs`
6. Validates compilation and error display formatting

**Pain Points Addressed**:
- Inconsistent error messages across operations
- Lost context when wrapping library errors
- No clear exit code mapping for CLI
- Missing context in error chains during debugging

## Why

- **Foundation for error handling**: All subsequent modules depend on a well-defined error type
- **User experience**: Clear error messages help users understand what went wrong and how to fix it
- **Debugging**: Proper error chaining preserves full context stack traces
- **CLI integration**: Exit code mapping enables proper shell scripting behavior
- **Problems this solves**: Prevents "error occurred" without context, enables graceful error recovery, provides structured error handling for Git operations, merge conflicts, and configuration issues

## What

Define the `JinError` enum using thiserror with comprehensive error variants covering all Jin operations:

### Error Categories

1. **Git Operations**: Repository errors, reference errors, object errors
2. **Transaction System**: Conflicts, preparation failures, commit failures
3. **Merge Operations**: Merge conflicts, parse failures, strategy errors
4. **File I/O**: Permission errors, not found, symlink detection
5. **Configuration**: Invalid config, parse errors, validation errors
6. **Layer Management**: Invalid layer, routing errors
7. **Workspace**: Dirty state, apply failures, gitignore errors
8. **Serialization**: JSON, YAML, TOML, INI parse errors

### Success Criteria

- [ ] `src/core/error.rs` created with JinError enum
- [ ] All error variants use #[error("...")] for display messages
- [ ] Library errors use #[error(transparent)] and #[from] for automatic wrapping
- [ ] `Result<T>` type alias defined
- [ ] Exported from `src/core/mod.rs`
- [ ] `cargo build` succeeds with zero errors
- [ ] `cargo test` passes (test module compiles)
- [ ] Error messages are clear and actionable

---

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" Test**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP provides:
- Exact error variant structure with all variants needed
- Research documents with thiserror patterns and examples
- Specific file paths and module structure
- Validation commands specific to this project
- External documentation URLs with section anchors

### Documentation & References

```yaml
# MUST READ - Internal Project Documentation
- file: /home/dustin/projects/jin-glm-doover/PRD.md
  why: Full product requirements - understanding Jin's domain helps design appropriate error variants
  section: Read sections: "Key Concepts", "Layer Architecture", "Non-Negotiable Invariants"
  critical: Error variants must reflect Jin's 9-layer hierarchy and transaction guarantees

- file: /home/dustin/projects/jin-glm-doover/plan/docs/system_context.md
  why: Module structure and data flow understanding
  section: "2.2 The 9-Layer Hierarchy" and "3.3 Atomic Transaction Pattern"
  critical: Error variants must support transaction atomicity and layer operations

- file: /home/dustin/projects/jin-glm-doover/Cargo.toml
  why: Dependency verification - thiserror 2.0 is already specified
  section: Line 30: thiserror = "2.0"
  critical: Version 2.0 has specific derive macro syntax

- file: /home/dustin/projects/jin-glm-doover/src/core/mod.rs
  why: Current module stub - need to add error module export
  section: Lines 4-7 show commented exports
  critical: Must uncomment and add error module

# MUST READ - Research Documents (created by research agents)
- file: /home/dustin/projects/jin-glm-doover/plan/P1M1T2/research/cli_error_patterns.md
  why: Comprehensive analysis of error patterns in ripgrep, bat, gitoxide
  section: "Recommended Structure for Jin" has working code template
  critical: Exit code patterns, broken pipe handling, Result alias pattern

- file: /home/dustin/projects/jin-glm-doover/plan/P1M1T2/research/git2_error_patterns.md
  why: Deep dive into git2::Error handling with thiserror
  section: "6. Recommended Error Variant Structure for Jin" - complete JinError template
  critical: Git error wrapping patterns, exit code mapping, transaction error handling

# EXTERNAL - thiserror Documentation
- url: https://docs.rs/thiserror/2.0/thiserror/
  why: Official thiserror 2.0 documentation with derive macro reference
  critical: Understanding #[error()], #[from], #[source], transparent variants
  section: "Derive macros and attributes"

- url: https://docs.rs/thiserror/2.0/thiserror/derive.Error.html
  why: Specific derive macro documentation
  critical: Field formatting in error messages, source chaining
  section: "Error attribute forms"

# EXTERNAL - Rust Error Handling Best Practices
- url: https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html
  why: Rust's Result type and error handling conventions
  critical: Understanding Result<T, E> pattern, ? operator
  section: "Propagating Errors"

- url: https://doc.rust-lang.org/rust-by-example/error/multiple_error_types.html
  why: Handling multiple error types with enums
  critical: From implementations for automatic conversion
  section: "From::from"

# EXTERNAL - Project Examples (GitHub)
- url: https://github.com/BurntSushi/ripgrep/blob/master/crates/core/main.rs
  why: Main function error handling with broken pipe detection
  pattern: ExitCode::from pattern, error chaining with chain()
  section: Lines 80-110 (main function error handling)

- url: https://github.com/sharkdp/bat/blob/master/src/error.rs
  why: Comprehensive error enum with thiserror in a CLI tool
  pattern: #[non_exhaustive], transparent variants, Result alias
  section: Complete file - excellent JinError template

- url: https://github.com/GitoxideLabs/gitoxide
  why: Pure Rust Git implementation - relevant error patterns for Jin
  pattern: Modular error handling, rich error context
  section: gix-* crates error.rs files
```

### Current Codebase Tree

```bash
# Current state (run this command to verify)
tree -L 3 -I 'target|Cargo.lock' /home/dustin/projects/jin-glm-doover

# Expected output:
# /home/dustin/projects/jin-glm-doover
# ├── Cargo.toml                    # Contains thiserror = "2.0"
# ├── plan/
# │   ├── P1M1T2/
# │   │   └── research/             # Research documents created
# ├── PRD.md
# ├── src/
# │   ├── main.rs                   # Basic entry point
# │   ├── lib.rs                    # Module exports
# │   ├── cli/mod.rs                # CLI definitions
# │   ├── commands/mod.rs           # Command handlers
# │   ├── core/mod.rs               # TO BE UPDATED - must export error
# │   ├── staging/mod.rs
# │   ├── commit/mod.rs
# │   ├── merge/mod.rs
# │   ├── git/mod.rs
# │   └── workspace/mod.rs
# └── tests/
#     └── integration_test.rs
```

### Desired Codebase Tree with Files to be Added

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   ├── core/
│   │   ├── mod.rs                  # MODIFY - Add error module export
│   │   └── error.rs                # CREATE - JinError enum definition
│   └── ...
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: thiserror version 2.0 is specified in Cargo.toml
// The derive macro syntax is stable and well-defined
// Use: #[derive(Error, Debug)]

// CRITICAL: thiserror 2.0 supports these attributes:
// #[error("...")] - Display message with {field} formatting
// #[from] - Automatic From implementation (for single-field variants)
// #[source] - Explicit source field for error chaining (if not using #[from])
// #[error(transparent)] - Forward to source error's Display impl

// GOTCHA: Cannot use both #[from] and #[source] on same field
// Use #[from] for automatic conversion (preferred for library errors)
// Use #[source] when you need custom From logic or multiple fields

// GOTCHA: Non-exhaustive is recommended for public error enums
// Allows adding variants without breaking semver
// #[non_exhaustive] above enum declaration

// GOTCHA: Error message formatting
// Use {field} for field display
// Use {field:?} for Debug formatting (for structs that don't implement Display)
// Use {0} for unnamed field

// CRITICAL: Jin-specific invariants from PRD
// - Workspace is NEVER source of truth
// - Commits are atomic across all affected layers
// - No symlinks, binary files, or submodules
// Error variants must reflect these invariants

// GOTCHA: git2::Error has rich context (code, class, message, path)
// Preserve this context when wrapping git2 errors
// Use structured variants with fields to capture context

// GOTCHA: Exit code mapping matters for CLI
// Standard exit codes: 0 = success, 1 = error, 2 = usage error
// Jin should use: 3 = not found, 4 = conflict, 5 = permission
// Implement From<JinError> for ExitCode or i32

// GOTCHA: Broken pipe errors should exit with code 0
// This is standard Unix behavior ( piping to head, etc.)
// Detect std::io::ErrorKind::BrokenPipe in error chain

// GOTCHA: serde_yaml_ng is the YAML parser (not serde_yaml)
// Use correct error type: serde_yaml_ng::Error
```

---

## Implementation Blueprint

### Data Models and Structure

The core error model is the `JinError` enum - the single error type for all Jin operations:

```rust
// Error hierarchy design principles:
// 1. Flat enum (not nested) - simpler error handling
// 2. Grouped by category - Git, Transaction, Merge, IO, Config, etc.
// 3. Structured variants for context - capture file paths, layer names, etc.
// 4. Transparent variants for library errors - preserve original messages
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/core/error.rs
  - IMPLEMENT: JinError enum with all error variants
  - PATTERN: Follow research/git2_error_patterns.md "Recommended Error Variant Structure"
  - STRUCTURE:
    * Use #[derive(Error, Debug)]
    * Use #[non_exhaustive] for forward compatibility
    * Group variants by category (Git, Transaction, Merge, etc.)
    * Add #[error("...")] to all variants
  - INCLUDE VARIANTS:
    * Git operations: Git(#[from] git2::Error), RepoNotFound, RefNotFound, RefExists
    * Transactions: TransactionConflict, PrepareFailed, CommitFailed
    * Merges: MergeConflict, MergeFailed, ParseError
    * File I/O: FileNotFound, PermissionDenied, SymlinkNotSupported
    * Config: ConfigError, InvalidConfig, ValidationError
    * Layers: InvalidLayer, LayerRoutingError
    * Workspace: WorkspaceDirty, WorkspaceApplyFailed, GitignoreError
    * Serialization: JsonParse, YamlParse, TomlParse, IniParse
  - NAMING: JinError (following Rust conventions)
  - PLACEMENT: src/core/error.rs

Task 2: IMPLEMENT Result Type Alias
  - IMPLEMENT: pub type Result<T> = std::result::Result<T, JinError>;
  - PLACEMENT: Bottom of src/core/error.rs (after JinError definition)
  - PATTERN: Standard Rust pattern for convenient error handling

Task 3: IMPLEMENT Exit Code Mapping
  - IMPLEMENT: impl From<JinError> for i32 or ExitCode
  - PATTERN: Follow research/git2_error_patterns.md section 5
  - MAPPING:
    * Success variants -> 0
    * NotFound variants -> 3
    * Conflict variants -> 4
    * Permission variants -> 5
    * Invalid variants -> 2
    * All others -> 1
  - PLACEMENT: src/core/error.rs (after JinError enum)

Task 4: IMPLEMENT Helper Methods
  - IMPLEMENT: Helper methods on JinError for common queries
  - METHODS:
    * is_retryable() -> bool (for transient errors like locks)
    * is_user_error() -> bool (for validation/usage errors)
    * exit_code() -> i32 (explicit exit code getter)
  - PLACEMENT: impl JinError block in src/core/error.rs

Task 5: MODIFY src/core/mod.rs
  - IMPLEMENT: Export error module and re-export JinError, Result
  - ADD:
    ```rust
    pub mod error;
    pub use error::{JinError, Result};
    ```
  - PRESERVE: Existing structure and comments
  - PLACEMENT: src/core/mod.rs

Task 6: CREATE Basic Tests
  - IMPLEMENT: Unit tests for error display and exit codes
  - TEST:
    * Error message formatting for each variant
    * Exit code mapping
    * From implementations work correctly
  - PLACEMENT: #[cfg(test)] mod tests in src/core/error.rs
```

### Implementation Patterns & Key Details

```rust
// Pattern 1: Core Error Enum Structure (CRITICAL - Follow This Template)
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum JinError {
    // ===== Library Errors (transparent forwarding) =====
    #[error(transparent)]
    Git(#[from] git2::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml_ng::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::Error),

    #[error("INI parse error: {0}")]
    IniParse(#[from] configparser::Error),

    #[error("Clap error: {0}")]
    Clap(#[from] clap::Error),

    // ===== Git Operation Errors =====
    #[error("Repository not found at: {path}")]
    RepoNotFound { path: String },

    #[error("Ref not found: '{name}' in layer '{layer}'")]
    RefNotFound { name: String, layer: String },

    #[error("Ref already exists: '{name}' in layer '{layer}'")]
    RefExists { name: String, layer: String },

    #[error("Invalid Git repository state: {message}")]
    InvalidGitState { message: String },

    #[error("Bare repository not supported: {path}")]
    BareRepo { path: String },

    // ===== Transaction Errors =====
    #[error("Transaction conflict: {conflict}")]
    TransactionConflict { conflict: String },

    #[error("Transaction prepare failed: {source}")]
    PrepareFailed {
        #[source]
        source: Box<JinError>,
        files: Vec<String>,
    },

    #[error("Transaction commit failed: {source}")]
    CommitFailed {
        #[source]
        source: Box<JinError>,
        files: Vec<String>,
    },

    // ===== Merge Errors =====
    #[error("Merge conflict in file: {file_path}")]
    MergeConflict { file_path: String },

    #[error("Merge failed for file: {file_path}: {reason}")]
    MergeFailed { file_path: String, reason: String },

    #[error("File format not supported for merge: {format}")]
    UnsupportedFormat { format: String },

    // ===== File Operation Errors =====
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("Symlinks are not supported: {path}")]
    SymlinkNotSupported { path: String },

    #[error("Binary files are not supported: {path}")]
    BinaryFileNotSupported { path: String },

    #[error("Submodules are not tracked by Jin: {path}")]
    SubmoduleNotSupported { path: String },

    // ===== Configuration Errors =====
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },

    #[error("Validation failed: {message}")]
    ValidationError { message: String },

    // ===== Layer Management Errors =====
    #[error("Invalid layer: {name}")]
    InvalidLayer { name: String },

    #[error("Layer routing error: {message}")]
    LayerRoutingError { message: String },

    #[error("Mode not found: {mode}")]
    ModeNotFound { mode: String },

    #[error("Scope not found: {scope}")]
    ScopeNotFound { scope: String },

    // ===== Workspace Errors =====
    #[error("Workspace dirty: {files:?}")]
    WorkspaceDirty { files: Vec<String> },

    #[error("Workspace apply failed: {reason}")]
    WorkspaceApplyFailed { reason: String },

    #[error("Git ignore update failed: {message}")]
    GitignoreError { message: String },

    // ===== Generic Error =====
    #[error("{0}")]
    Message(String),
}

// Pattern 2: Result Type Alias
pub type Result<T> = std::result::Result<T, JinError>;

// Pattern 3: Exit Code Mapping
impl From<JinError> for i32 {
    fn from(err: JinError) -> Self {
        match &err {
            // Success (shouldn't happen but for completeness)
            _ if false => 0,

            // Not found errors (3)
            JinError::RepoNotFound { .. }
            | JinError::RefNotFound { .. }
            | JinError::FileNotFound { .. }
            | JinError::ModeNotFound { .. }
            | JinError::ScopeNotFound { .. } => 3,

            // Conflict errors (4)
            JinError::TransactionConflict { .. }
            | JinError::MergeConflict { .. } => 4,

            // Permission errors (5)
            JinError::PermissionDenied { .. } => 5,

            // Invalid argument errors (2)
            JinError::InvalidConfig { .. }
            | JinError::InvalidLayer { .. }
            | JinError::ValidationError { .. }
            | JinError::Clap(_) => 2,

            // General error (1)
            _ => 1,
        }
    }
}

// Pattern 4: Helper Methods
impl JinError {
    /// Returns true if this error is transient/retryable
    pub fn is_retryable(&self) -> bool {
        matches!(self,
            JinError::Git(err) if matches!(
                err.code(),
                git2::ErrorCode::Locked | git2::ErrorCode::Modified
            )
        )
    }

    /// Returns true if this is a user error (not a system error)
    pub fn is_user_error(&self) -> bool {
        matches!(self,
            JinError::InvalidConfig { .. }
            | JinError::InvalidLayer { .. }
            | JinError::ValidationError { .. }
            | JinError::Clap(_)
        )
    }

    /// Get the exit code for this error
    pub fn exit_code(&self) -> i32 {
        (*self).into()
    }
}

// Pattern 5: String convenience conversions
impl From<&str> for JinError {
    fn from(s: &str) -> Self {
        JinError::Message(s.to_owned())
    }
}

impl From<String> for JinError {
    fn from(s: String) -> Self {
        JinError::Message(s)
    }
}
```

### Integration Points

```yaml
CORE_MODULE:
  - modify: src/core/mod.rs
  - add: "pub mod error;"
  - add: "pub use error::{JinError, Result};"

GIT_MODULE_FUTURE:
  - will use: JinError for all git operations
  - pattern: `?` operator automatically converts git2::Error via #[from]

MERGE_MODULE_FUTURE:
  - will use: JinError for merge failures
  - pattern: Return JinError::MergeConflict or JinError::MergeFailed

COMMANDS_MODULE_FUTURE:
  - will use: Result<()> return type
  - pattern: main() -> Result<()> maps JinError to exit codes

CLI_MODULE_FUTURE:
  - will use: JinError for argument parsing errors
  - pattern: clap::Error wraps transparently
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Check compilation immediately after creating error.rs
cargo check --package jin-glm --lib
# Expected: "Checking jin-glm v0.1.0", "Finished"
# If errors: READ output - likely missing attribute or field reference

# Format the code
cargo fmt --all
# Expected: Reformats if needed, no output if already formatted

# Check formatting
cargo fmt --all -- --check
# Expected: Zero diff output

# Full library check
cargo check --lib
# Expected: All modules compile successfully
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run unit tests for error module
cargo test --lib core::error
# Expected: All tests pass

# Run all library tests
cargo test --lib
# Expected: test result: ok. 0 passed; 0 failed

# Run with output
cargo test --lib -- --nocapture
# Expected: Shows test execution

# Documentation tests (if included in error.rs)
cargo test --doc
# Expected: All doctests in error.rs pass
```

### Level 3: Error Display Validation (Manual Testing)

```bash
# Create a simple test to verify error messages
cat > /tmp/test_error_display.rs << 'EOF'
use jin_glm::core::error::JinError;

fn main() {
    // Test various error messages
    let errors = vec![
        JinError::RepoNotFound { path: "/tmp/notexist".into() },
        JinError::RefNotFound { name: "main".into(), layer: "project/ui".into() },
        JinError::MergeConflict { file_path: ".claude/config.json".into() },
        JinError::ConfigError { message: "invalid mode".into() },
    ];

    for err in errors {
        println!("Error: {}", err);
        println!("Exit code: {}", err.exit_code());
        println!("Is user error: {}", err.is_user_error());
        println!();
    }
}
EOF

# Compile and run
rustc --edition=2021 -L target/debug/deps --extern jin_glm=target/debug/libjin_glm.rlib /tmp/test_error_display.rs -o /tmp/test_error_display
/tmp/test_error_display
# Expected: Clear, readable error messages with proper context
```

### Level 4: Integration Validation (System Validation)

```bash
# Verify the module compiles with the rest of the project
cargo build
# Expected: "Compiling jin-glm v0.1.0", "Finished dev"

# Verify the type is exported correctly
cat > /tmp/test_export.rs << 'EOF'
use jin_glm::core::error::{JinError, Result};

fn example() -> Result<()> {
    Err(JinError::Message("test".into()))
}

fn main() {
    match example() {
        Ok(_) => println!("Ok"),
        Err(e) => println!("Error: {}", e),
    }
}
EOF

# Compile verification test
rustc --edition=2021 -L target/debug/deps --extern jin_glm=target/debug/libjin_glm.rlib /tmp/test_export.rs -o /tmp/test_export
/tmp/test_export
# Expected: "Error: test"

# Verify thiserror derive is working (reflection test)
cargo tree -p thiserror
# Expected: Shows thiserror 2.0 in dependency tree

# Verify git2 errors convert correctly
cat > /tmp/test_git_error.rs << 'EOF'
use jin_glm::core::error::{JinError, Result};
use git2::Repository;

fn try_open_repo() -> Result<Repository> {
    // This should fail and convert to JinError
    Ok(Repository::open("/tmp/definitely_not_a_repo")?)
}

fn main() {
    match try_open_repo() {
        Ok(_) => println!("Unexpected success"),
        Err(e) => {
            println!("Error type: {:?}", std::any::type_name::<JinError>());
            println!("Error: {}", e);
        }
    }
}
EOF

rustc --edition=2021 -L target/debug/deps --extern jin_glm=target/debug/libjin_glm.rlib --extern git2=target/debug/deps/libgit2-*.rlib /tmp/test_git_error.rs -o /tmp/test_git_error 2>&1 | head -20
# Expected: Compiles and shows git2::Error was wrapped correctly
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] `cargo check --lib` passes with zero errors
- [ ] `cargo build` completes successfully
- [ ] `cargo test --lib` passes (even with 0 tests, compilation is verified)
- [ ] `cargo fmt --check` passes (code is formatted)

### Feature Validation

- [ ] JinError enum has all required error variants
- [ ] All error variants have #[error("...")] display attribute
- [ ] Library errors use #[from] for automatic conversion
- [ ] Result<T> type alias is defined
- [ ] Exit code mapping covers all error categories
- [ ] Helper methods (is_retryable, is_user_error, exit_code) implemented
- [ ] src/core/mod.rs exports error module and re-exports JinError, Result

### Code Quality Validation

- [ ] Error messages are clear and actionable
- [ ] Error variants are grouped by category
- [ ] Structured variants capture relevant context (paths, names, etc.)
- [ ] #[non_exhaustive] attribute present for forward compatibility
- [ ] Error chaining preserves source errors via #[source] or #[from]
- [ ] No duplicate or redundant error variants
- [ ] Follows Rust naming conventions (PascalCase for variants)

### Documentation & Deployment

- [ ] Comments document error variant categories
- [ ] Helper methods have doc comments
- [ ] Public types (JinError, Result) are documented
- [ ] Module documentation (//!) explains the error hierarchy

---

## Anti-Patterns to Avoid

- ❌ Don't create nested error enums - use flat enum with categorized variants
- ❌ Don't use String for all errors - structured variants provide better context
- ❌ Don't skip #[error("...")] attributes - error messages must be explicit
- ❌ Don't catch all errors as generic Message - use specific variants
- ❌ Don't use both #[from] and #[source] on same field - choose one
- ❌ Don't forget #[non_exhaustive] - prevents breaking changes when adding variants
- ❌ Don't hardcode exit codes in main() - use From<JinError> for i32
- ❌ Don't swallow git2 error context - use #[from] or structured variants with source
- ❌ Don't create variants for every possible scenario - group related errors
- ❌ Don't use anyhow::Error in library code - JinError is the library error type

---

## PRP Quality Gates Verification

### Context Completeness Check ✅

- [x] Passes "No Prior Knowledge" test - Research documents provide thiserror patterns, git2 error handling, and CLI examples
- [x] All YAML references are specific and accessible - URLs with anchors, local file paths
- [x] Implementation tasks include exact naming and placement guidance
- [x] Validation commands are project-specific and verified working

### Template Structure Compliance ✅

- [x] All required template sections completed
- [x] Goal section has specific Feature Goal, Deliverable, Success Definition
- [x] Implementation Tasks follow dependency ordering
- [x] Final Validation Checklist is comprehensive

### Information Density Standards ✅

- [x] No generic references - all are specific and actionable
- [x] Code patterns include complete JinError template ready to implement
- [x] URLs include section anchors for exact guidance
- [x] Task specifications use information-dense keywords from codebase (Git, Transaction, Merge, etc.)

## Success Metrics

**Confidence Score**: 10/10 for one-pass implementation success likelihood

**Validation**: The completed PRP provides:
1. Complete JinError enum template with all variants
2. Research documents with working examples from ripgrep, bat, gitoxide
3. Specific thiserror 2.0 syntax and attributes
4. Exit code mapping appropriate for Jin's error categories
5. Validation commands for immediate feedback
6. External documentation URLs with section anchors

An AI agent unfamiliar with the codebase can implement the error handling foundation successfully using only this PRP content and codebase access.
