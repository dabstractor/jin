name: "P7.M4.T1 - Create Audit Module"
description: |

---

## Goal

**Feature Goal**: Implement audit logging infrastructure that tracks all Jin commit operations in PRD-compliant JSON format, stored in `.jin/audit/` directory, integrated with CommitPipeline to automatically generate audit entries on each commit.

**Deliverable**: Complete audit logging module (`src/audit/`) with:
- `AuditEntry` type matching PRD specification
- Append-only audit log file operations with atomic writes
- Integration with `CommitPipeline` for automatic audit generation
- Comprehensive unit and integration tests

**Success Definition**:
- Every `jin commit` operation generates a JSON audit entry in `.jin/audit/`
- Audit entries contain all required PRD fields (timestamp, user, project, mode, scope, layer, files, commits, context)
- Audit logs use append-only file operations that are safe from corruption
- All existing tests pass plus new audit-specific tests pass
- Audit file format matches PRD specification exactly

## User Persona (if applicable)

**Target User**: Developers and system administrators who need to track, review, and audit Jin operations for compliance, debugging, and historical analysis.

**Use Case**:
- A team lead needs to understand who made configuration changes and when
- Debugging issues by tracing the history of layer modifications
- Compliance auditing for configuration management
- Forensic analysis when something goes wrong

**User Journey**:
1. Developer runs `jin commit -m "Add new config"`
2. Jin automatically writes an audit entry to `.jin/audit/audit-YYYY-MM-DD.jsonl`
3. Auditor/administrator can review audit logs to see the complete history
4. Logs are human-readable JSON and machine-parsable for analysis tools

**Pain Points Addressed**:
- No visibility into who made what changes when
- Difficult to trace configuration changes over time
- No audit trail for compliance requirements
- Cannot regenerate the history of what happened

## Why

- **Compliance**: Many organizations require audit trails for configuration management
- **Debugging**: Understanding the history of changes helps troubleshoot issues
- **Accountability**: Audit logs provide traceability of who made what changes
- **Integration with existing features**: The PRD (Section 17) specifies audit logging as a required feature
- **Non-blocking**: Audit failures should not prevent commits from succeeding

## What

### User-Visible Behavior

**No direct user interaction required** - audit logging happens automatically on every `jin commit` operation.

**Audit files are created automatically** in `.jin/audit/` directory:
- File naming: `audit-YYYY-MM-DD.jsonl` (one file per day)
- Each line is a complete JSON audit entry
- Files are human-readable and machine-parsable

**Example audit entry** (from PRD Section 17):
```json
{
  "timestamp": "2025-10-19T15:04:02Z",
  "user": "dustin",
  "project": "ui-dashboard",
  "mode": "claude",
  "scope": "language:javascript",
  "layer": 4,
  "files": [".claude/config.json"],
  "base_commit": "abc123",
  "merge_commit": "def456",
  "context": {
    "active_mode": "claude",
    "active_scope": "language:javascript"
  }
}
```

### Technical Requirements

1. **New module**: `src/audit/` with `mod.rs`, `entry.rs`, `logger.rs`
2. **Integration point**: `CommitPipeline::execute()` must generate audit entries
3. **File format**: JSON Lines (one JSON object per line, newline-delimited)
4. **Storage**: `.jin/audit/audit-YYYY-MM-DD.jsonl`
5. **Atomic writes**: Use temp file + rename pattern to prevent corruption
6. **Non-blocking**: Audit failures should log warnings but not prevent commits
7. **Derivable**: Audit logs can be regenerated from Git history if needed

### Success Criteria

- [ ] `src/audit/mod.rs` declares and exports audit types
- [ ] `AuditEntry` struct with all PRD-specified fields
- [ ] `AuditLogger` implements append-only file writing
- [ ] `CommitPipeline` generates and persists audit entries
- [ ] Unit tests for `AuditEntry` serialization
- [ ] Unit tests for `AuditLogger` file operations
- [ ] Integration tests for audit generation during commits
- [ ] All existing tests continue to pass

## All Needed Context

### Context Completeness Check

_If someone knew nothing about this codebase, they would have everything needed to implement this successfully because:_
- PRD specification is included verbatim
- File I/O patterns from existing codebase are referenced
- Module organization patterns are specified
- Integration points are clearly identified
- Test patterns are documented
- Error handling patterns are specified

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- url: https://github.com/tokio-rs/tracing
  why: Understanding Rust structured logging patterns (for reference only, not implementing)
  critical: We use simple JSON serialization, not tracing framework

- url: https://hoop.dev/blog/auditing-accountability-git-best-practices-for-tracking-changes/
  why: Git-based audit logging best practices
  critical: Audit logs should be derived from Git commits, not duplicate storage

- url: https://www.loggly.com/use-cases/json-logging-best-practices/
  why: JSON logging format and structure guidance
  critical: JSON Lines format (one JSON per line) for append-only logs

- file: PRD.md#section-17
  why: Exact audit log format specification from requirements
  pattern: PRD specifies exact JSON schema for audit entries
  gotcha: PRD shows "layer" as integer (1-9), not Layer enum

- file: src/commit/pipeline.rs
  why: Integration point for audit generation during commits
  pattern: CommitPipeline::execute() method is where audit entry should be created
  gotcha: Must capture both success and failure outcomes for audit

- file: src/core/layer.rs
  why: Layer enum definition and precedence values
  pattern: Layer::precedence() returns u8 (1-9) which matches PRD "layer" field
  gotcha: PRD uses integer layer value, not the enum variant name

- file: src/core/config.rs
  why: ProjectContext structure for capturing active mode/scope
  pattern: ProjectContext::load() retrieves current context from .jin/context
  gotcha: Context may not be set - use Option handling

- file: src/staging/metadata.rs
  why: Atomic file write pattern to follow
  pattern: Temp file with .tmp extension, then atomic rename
  gotcha: Must preserve temp file on error for debugging

- file: src/core/error.rs
  why: JinError enum for consistent error handling
  pattern: Add new JinError::Audit variant for audit-specific errors
  gotcha: Audit failures should be non-blocking (log warning, not fail)

- file: src/lib.rs
  why: Module declaration and export pattern
  pattern: Add `pub mod audit;` and re-exports
  gotcha: Follow same pattern as core, git, staging, commit modules

- file: tests/common/fixtures.rs
  why: Test fixture patterns for audit module tests
  pattern: Use TempDir with _tempdir field to prevent cleanup
  gotcha: TempDir must be stored in struct to prevent deletion

- docfile: plan/P7M4T1/research/README.md
  why: Comprehensive research summary for audit module
  section: Codebase Patterns Analysis
```

### Current Codebase Tree

```bash
src/
├── core/
│   ├── mod.rs          # Core types export
│   ├── error.rs        # JinError enum, Result type
│   ├── layer.rs        # Layer enum (1-9 hierarchy)
│   └── config.rs       # ProjectContext, JinConfig
├── git/
│   ├── mod.rs          # Git module exports
│   ├── repo.rs         # JinRepo wrapper
│   ├── refs.rs         # Git reference operations
│   ├── objects.rs      # Git object creation
│   ├── tree.rs         # Tree reading/walking
│   ├── transaction.rs  # Atomic multi-layer transactions
│   └── remote.rs       # Remote operations
├── staging/
│   ├── mod.rs          # Staging module exports
│   ├── index.rs        # StagingIndex with JSON serialization
│   ├── entry.rs        # StagedEntry type
│   ├── workspace.rs    # Workspace file operations
│   ├── router.rs       # Layer routing logic
│   ├── gitignore.rs    # .gitignore management
│   └── metadata.rs     # WorkspaceMetadata with atomic writes
├── commit/
│   ├── mod.rs          # Commit module exports
│   └── pipeline.rs     # CommitPipeline with execute() method
├── merge/
│   ├── mod.rs
│   ├── value.rs
│   ├── deep.rs
│   ├── text.rs
│   └── layer.rs
├── commands/
│   ├── mod.rs
│   ├── commit_cmd.rs   # CLI command that calls CommitPipeline
│   ├── add.rs
│   ├── status.rs
│   └── ...
├── cli/
│   ├── mod.rs
│   └── args.rs
└── lib.rs              # Module declarations and re-exports

tests/
├── common/
│   ├── mod.rs          # Test utilities
│   ├── fixtures.rs     # TestFixture, RemoteFixture
│   └── assertions.rs   # Custom assertion helpers
├── cli_basic.rs        # Basic CLI integration tests
└── ...
```

### Desired Codebase Tree with Files to be Added

```bash
src/
├── audit/              # NEW: Audit logging module
│   ├── mod.rs          # NEW: Module exports (entry, logger types)
│   ├── entry.rs        # NEW: AuditEntry type definition
│   └── logger.rs       # NEW: AuditLogger for file operations

# Modified files:
├── commit/
│   └── pipeline.rs     # MODIFY: Add audit logging to execute()
└── lib.rs              # MODIFY: Add pub mod audit;

tests/
└── audit_tests.rs      # NEW: Audit module integration tests

.jin/                    # Jin working directory
└── audit/               # NEW: Audit log storage directory
    └── audit-YYYY-MM-DD.jsonl  # NEW: Daily audit log files (JSON Lines format)
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: PRD specifies "layer" as integer (1-9), not Layer enum
// When serializing AuditEntry, use Layer::precedence() which returns u8
#[serde(skip_serializing_if = "Option::is_none")]
pub layer: Option<u8>,  // Not Layer enum!

// CRITICAL: Atomic write pattern - temp file MUST be in same filesystem as target
// Use .with_extension("tmp") pattern from src/staging/metadata.rs
let temp_path = path.with_extension("tmp");
std::fs::write(&temp_path, content)?;
std::fs::rename(&temp_path, path)?;

// CRITICAL: TempDir MUST be stored in test fixture struct
// If TempDir is dropped, directory is deleted immediately
pub struct TestFixture {
    _tempdir: TempDir,  // Underscore prefix = "I'm keeping this for drop"
    pub path: PathBuf,
}

// CRITICAL: Audit failures should NOT prevent commits
// Audit is informational - log warnings but don't fail the operation
if let Err(e) = audit_logger.log_entry(&entry) {
    eprintln!("Warning: Failed to write audit log: {}", e);
    // Continue anyway - commit succeeded
}

// CRITICAL: JSON Lines format - one JSON object per line
// Use serde_json::to_string() (not to_string_pretty()) for single-line JSON
let json_line = serde_json::to_string(&entry)?;
writeln!(writer, "{}", json_line)?;

// CRITICAL: Use BufWriter for file I/O performance
// Direct std::fs::write is fine for small files, but BufWriter for many writes
use std::io::BufWriter;
let file = std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(&path)?;
let mut writer = BufWriter::new(file);

// CRITICAL: Timestamp format must match PRD exactly
// Use chrono::Utc::now().to_rfc3339() for "2025-10-19T15:04:02Z" format
use chrono::Utc;
let timestamp = Utc::now().to_rfc3339();

// CRITICAL: Username retrieval - handle errors gracefully
// Git config may not be set, use fallback
let user = match std::process::Command::new("git")
    .args(["config", "user.email"])
    .output()
{
    Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
    Err(_) => "unknown".to_string(),
};
```

## Implementation Blueprint

### Data Models and Structure

Create the core audit types that match the PRD specification:

```rust
// src/audit/entry.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Audit context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditContext {
    /// Currently active mode at time of commit
    pub active_mode: Option<String>,
    /// Currently active scope at time of commit
    pub active_scope: Option<String>,
}

/// Single audit log entry matching PRD Section 17 specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// ISO 8601 timestamp (e.g., "2025-10-19T15:04:02Z")
    pub timestamp: String,
    /// User identity (from Git config or system)
    pub user: String,
    /// Project name (inferred from Git remote or context)
    pub project: Option<String>,
    /// Mode context (if applicable)
    pub mode: Option<String>,
    /// Scope context (if applicable)
    pub scope: Option<String>,
    /// Layer number (1-9) from Layer::precedence()
    pub layer: Option<u8>,
    /// List of files affected by this operation
    pub files: Vec<String>,
    /// Base commit hash (parent commit)
    pub base_commit: Option<String>,
    /// Merge commit hash (newly created commit)
    pub merge_commit: Option<String>,
    /// Additional context
    pub context: AuditContext,
}

impl AuditEntry {
    /// Create a new audit entry from commit information
    pub fn from_commit(
        user: String,
        project: Option<String>,
        mode: Option<String>,
        scope: Option<String>,
        layer: Option<u8>,
        files: Vec<String>,
        base_commit: Option<String>,
        merge_commit: String,
    ) -> Self {
        Self {
            timestamp: Utc::now().to_rfc3339(),
            user,
            project,
            mode,
            scope,
            layer,
            files,
            base_commit,
            merge_commit: Some(merge_commit),
            context: AuditContext {
                active_mode: mode.clone(),
                active_scope: scope.clone(),
            },
        }
    }
}

// src/audit/logger.rs
use crate::core::{JinError, Result};
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

/// Audit logger for writing audit entries to disk
pub struct AuditLogger {
    /// Base directory for audit files
    audit_dir: PathBuf,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(audit_dir: PathBuf) -> Result<Self> {
        // Ensure audit directory exists
        std::fs::create_dir_all(&audit_dir)
            .map_err(|e| JinError::Io(e))?;
        Ok(Self { audit_dir })
    }

    /// Get the audit file path for today
    fn today_path(&self) -> PathBuf {
        let date = Utc::now().format("%Y-%m-%d");
        self.audit_dir.join(format!("audit-{}.jsonl", date))
    }

    /// Write an audit entry to today's log file
    pub fn log_entry(&self, entry: &AuditEntry) -> Result<()> {
        let path = self.today_path();

        // Open file in append mode, create if not exists
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| JinError::Io(e))?;

        let mut writer = BufWriter::new(file);

        // Serialize as single-line JSON (JSON Lines format)
        let json_line = serde_json::to_string(entry)
            .map_err(|e| JinError::Parse {
                format: "JSON".to_string(),
                message: e.to_string(),
            })?;

        // Write line with newline
        use std::io::Write;
        writeln!(writer, "{}", json_line)
            .map_err(|e| JinError::Io(e))?;

        // Flush to ensure write is complete
        writer.flush().map_err(|e| JinError::Io(e))?;

        Ok(())
    }

    /// Create audit logger from project context
    pub fn from_project() -> Result<Self> {
        let audit_dir = PathBuf::from(".jin").join("audit");
        Self::new(audit_dir)
    }
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/audit/mod.rs
  - IMPLEMENT: Module declaration and exports
  - FOLLOW pattern: src/staging/mod.rs (pub mod declarations, pub use re-exports)
  - NAMING: mod.rs for module root
  - PLACEMENT: src/audit/ directory (new)
  - EXPORT: pub use entry::AuditEntry, pub use logger::AuditLogger

Task 2: CREATE src/audit/entry.rs
  - IMPLEMENT: AuditEntry, AuditContext structs with serde Serialize/Deserialize
  - FOLLOW pattern: src/staging/entry.rs (struct definition with derives)
  - NAMING: PascalCase for types, snake_case for fields
  - FIELD types: String for timestamp/user, Option<String> for optional fields, Vec<String> for files
  - SERIALIZATION: Use serde_json for JSON output
  - PLACEMENT: src/audit/entry.rs
  - GOTCHA: "layer" field is Option<u8> (from Layer::precedence()), not Layer enum

Task 3: CREATE src/audit/logger.rs
  - IMPLEMENT: AuditLogger struct with log_entry() method
  - FOLLOW pattern: src/staging/metadata.rs file operations (create_dir_all, BufWriter)
  - NAMING: AuditLogger struct, log_entry() method
  - FILE FORMAT: JSON Lines (one JSON per line, use serde_json::to_string not to_string_pretty)
  - FILE NAMING: audit-YYYY-MM-DD.jsonl in .jin/audit/
  - ERROR HANDLING: Audit failures return JinError but caller should log warning only
  - PLACEMENT: src/audit/logger.rs
  - DEPENDENCIES: Uses AuditEntry from Task 2

Task 4: MODIFY src/lib.rs
  - ADD: pub mod audit; at top level
  - FOLLOW pattern: Existing module declarations (pub mod core;, pub mod git;, etc.)
  - RE-EXPORT: pub use audit::{AuditEntry, AuditLogger}; for convenience
  - PRESERVE: All existing module declarations and exports
  - PLACEMENT: After existing pub mod declarations, in alphabetical order

Task 5: MODIFY src/commit/pipeline.rs
  - ADD: use crate::audit::{AuditEntry, AuditLogger};
  - MODIFY: execute() method to create and log audit entry after successful commit
  - CAPTURE: user from Git config or "unknown", affected layers, file paths, commit hashes
  - INTEGRATION: Create AuditEntry after tx.commit() succeeds, call AuditLogger::log_entry()
  - ERROR HANDLING: Wrap audit logging in if let Err(e) = ... { eprintln!("Warning: {}", e); }
  - PRESERVE: All existing commit logic - audit is non-blocking
  - GOTCHA: Must capture both base_commit and merge_commit from layer_commits
  - GOTCHA: Use Layer::precedence() for layer number (u8), not enum variant

Task 6: CREATE src/audit/tests.rs (or embed in entry.rs/logger.rs)
  - IMPLEMENT: Unit tests for AuditEntry serialization
  - IMPLEMENT: Unit tests for AuditLogger file operations
  - FOLLOW pattern: src/commit/pipeline.rs #[cfg(test)] module
  - TEST: AuditEntry serializes to correct JSON format matching PRD
  - TEST: AuditLogger creates directory and writes to file
  - TEST: AuditLogger appends to existing file
  - FIXTURE: Use TempDir for isolated test directories
  - COVERAGE: All public methods with positive and negative cases
  - PLACEMENT: #[cfg(test)] mod tests { ... } within entry.rs and logger.rs

Task 7: CREATE tests/audit_tests.rs
  - IMPLEMENT: Integration tests for audit during commit operations
  - FOLLOW pattern: tests/cli_basic.rs (TestFixture setup, assert_cmd usage)
  - TEST: jin commit creates audit file in .jin/audit/
  - TEST: Audit entry contains correct fields (timestamp, user, files, commits)
  - TEST: Multiple commits append to same daily file
  - TEST: Audit file format is valid JSON Lines
  - FIXTURE: Use tests/common/fixtures.rs TestFixture
  - CLEANUP: TempDir auto-cleanup after test
  - PLACEMENT: tests/audit_tests.rs

Task 8: MODIFY src/core/error.rs (OPTIONAL)
  - ADD: JinError::Audit variant if needed for specific audit errors
  - FOLLOW pattern: Existing error variants with String message
  - OR: Use existing JinError::Io and JinError::Other for audit failures
  - DECISION: Only add if audit-specific error context is needed
  - PLACEMENT: src/core/error.rs in JinError enum
```

### Implementation Patterns & Key Details

```rust
// Pattern 1: Audit Entry Creation in CommitPipeline
// In src/commit/pipeline.rs, after tx.commit()? succeeds:

// After successful commit
self.staging.clear();
self.staging.save()?;

// Create audit entry
if let Err(e) = self.log_audit(&layer_commits, &config.message, &context) {
    eprintln!("Warning: Failed to write audit log: {}", e);
}

// Add helper method to CommitPipeline:
impl CommitPipeline {
    fn log_audit(
        &self,
        layer_commits: &[(Layer, git2::Oid)],
        message: &str,
        context: &ProjectContext,
    ) -> Result<()> {
        use crate::audit::{AuditEntry, AuditLogger};
        use std::process::Command;

        // Get user from Git config
        let user = Command::new("git")
            .args(["config", "user.email"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        // Collect all files from staging
        let files: Vec<String> = self.staging
            .entries()
            .iter()
            .map(|e| e.path.display().to_string())
            .collect();

        // For each layer commit, create audit entry
        let logger = AuditLogger::from_project()?;

        for (layer, commit_oid) in layer_commits {
            let entry = AuditEntry {
                timestamp: chrono::Utc::now().to_rfc3339(),
                user: user.clone(),
                project: context.project.clone(),
                mode: context.mode.clone(),
                scope: context.scope.clone(),
                layer: Some(layer.precedence()),
                files: files.clone(),
                base_commit: None, // TODO: Get parent commit
                merge_commit: Some(commit_oid.to_string()),
                context: crate::audit::AuditContext {
                    active_mode: context.mode.clone(),
                    active_scope: context.scope.clone(),
                },
            };

            logger.log_entry(&entry)?;
        }

        Ok(())
    }
}

// Pattern 2: File I/O with BufWriter for performance
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};

let file = OpenOptions::new()
    .create(true)
    .append(true)
    .open(&path)?;

let mut writer = BufWriter::new(file);
let json_line = serde_json::to_string(&entry)?;
writeln!(writer, "{}", json_line)?;
writer.flush()?;

// Pattern 3: Test fixture pattern for audit tests
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_audit_test_fixture() -> (TempDir, AuditLogger) {
        let temp = TempDir::new().unwrap();
        let audit_dir = temp.path().join("audit");
        let logger = AuditLogger::new(audit_dir).unwrap();
        (temp, logger)
    }

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditEntry {
            timestamp: "2025-10-19T15:04:02Z".to_string(),
            user: "test@example.com".to_string(),
            // ... other fields
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"timestamp\":\"2025-10-19T15:04:02Z\""));
    }

    #[test]
    fn test_audit_logger_writes_file() {
        let (_temp, logger) = create_audit_test_fixture();

        let entry = AuditEntry { /* ... */ };
        logger.log_entry(&entry).unwrap();

        let audit_file = logger.today_path();
        assert!(audit_file.exists());
    }
}
```

### Integration Points

```yaml
COMMITPIPELINE:
  - modify: src/commit/pipeline.rs
  - location: In execute() method, after tx.commit()? and staging.clear()
  - pattern: Add audit logging as non-blocking post-commit operation
  - capture: user (git config), layers, files, commit hashes, context
  - error: Print warning but don't fail commit

LIB_RS:
  - modify: src/lib.rs
  - add: pub mod audit;
  - add: pub use audit::{AuditEntry, AuditLogger};
  - location: After existing module declarations

ERROR_RS:
  - modify: src/core/error.rs (optional)
  - add: JinError::Audit(String) variant if needed
  - or: Use existing JinError::Io, JinError::Other variants

CARGO_TOML:
  - verify: chrono dependency exists (or add it)
  - verify: serde, serde_json dependencies exist (they should)
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check --message-format=short       # Fast compilation check
cargo clippy --all-targets -- -D warnings  # Linting with warnings as errors

# Project-wide validation after all files created
cargo check                              # Full compilation check
cargo clippy                              # Full linting check
cargo fmt --check                         # Verify formatting

# Format code if needed
cargo fmt                                 # Auto-format all code

# Expected: Zero errors, zero warnings. Fix any clippy suggestions.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test audit module specifically
cargo test --lib audit -- --nocapture     # Run audit unit tests with output

# Test all lib code
cargo test --lib                          # Run all unit tests in src/

# Run with output for debugging
cargo test --lib audit_entry_serialization -- --nocapture
cargo test --lib audit_logger_writes_file -- --nocapture

# Run tests with backtrace for failures
RUST_BACKTRACE=1 cargo test --lib audit

# Expected: All unit tests pass. AuditEntry serialization matches PRD format exactly.
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the jin binary
cargo build --release                     # Build release binary

# Initialize a test project
cd /tmp && mkdir test_audit && cd test_audit
/home/dustin/projects/jin/target/release/jin init

# Make a commit to trigger audit
echo "test" > test.txt
/home/dustin/projects/jin/target/release/jin add test.txt
/home/dustin/projects/jin/target/release/jin commit -m "Test commit"

# Verify audit file was created
ls -la .jin/audit/
cat .jin/audit/audit-*.jsonl

# Verify audit content is valid JSON
cat .jin/audit/audit-*.jsonl | jq .

# Make multiple commits and verify appending
echo "test2" > test2.txt
/home/dustin/projects/jin/target/release/jin add test2.txt
/home/dustin/projects/jin/target/release/jin commit -m "Second commit"

# Count lines in audit file (should be 2 entries)
wc -l .jin/audit/audit-*.jsonl

# Expected: Audit file exists, contains valid JSON, entries append correctly.
```

### Level 4: Full Test Suite & Regression Testing

```bash
# Run all tests to ensure no regressions
cargo test --all                          # Run all tests (unit + integration)

# Run integration tests specifically
cargo test --test cli_basic               # Run basic CLI tests
cargo test --test audit_tests             # Run audit integration tests

# Verify specific test categories
cargo test --test 'cli_*'                 # All CLI integration tests
cargo test --test 'core_workflow'         # Core workflow tests

# Run tests with timing
cargo test --all -- --nocapture --test-threads=1

# Expected: All tests pass, no regressions in existing functionality.
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] `cargo test --all` passes with all tests green
- [ ] `cargo clippy --all-targets` produces zero warnings
- [ ] `cargo fmt --check` shows no formatting issues
- [ ] Audit file format matches PRD specification exactly (compare JSON structure)
- [ ] Audit entries contain all required fields from PRD

### Feature Validation

- [ ] Audit file created at `.jin/audit/audit-YYYY-MM-DD.jsonl`
- [ ] Audit entry contains timestamp, user, project, mode, scope, layer, files, commits
- [ ] Layer value is integer (1-9) from `Layer::precedence()`, not enum
- [ ] Multiple commits append to same daily file
- [ ] Audit failure doesn't prevent commit from succeeding
- [ ] Audit file is valid JSON Lines format (one JSON object per line)

### Code Quality Validation

- [ ] Module follows existing patterns (mod.rs structure, exports)
- [ ] Error handling matches JinError patterns
- [ ] File I/O uses atomic operations (create_dir_all, BufWriter)
- [ ] Tests follow existing patterns (TestFixture, TempDir)
- [ ] No new dependencies added unless necessary (verify chrono is in Cargo.toml)

### Documentation & Deployment

- [ ] Code is self-documenting with clear type names
- [ ] Public types have doc comments (///)
- [ ] Complex logic has inline comments (//)
- [ ] Module-level documentation (//!) in mod.rs

### Integration Validation

- [ ] `CommitPipeline::execute()` generates audit entries
- [ ] Audit logging is non-blocking (warns but doesn't fail)
- [ ] Audit logger integrates with project context (mode, scope, project)
- [ ] Existing tests continue to pass (no regressions)

---

## Anti-Patterns to Avoid

- **Don't** use `serde_json::to_string_pretty()` - use `to_string()` for single-line JSON
- **Don't** fail the commit if audit logging fails - log warning and continue
- **Don't** store Layer enum in audit - use `Layer::precedence()` for integer value
- **Don't** create a new audit file for each entry - use daily files with append
- **Don't** forget to flush BufWriter - data may not be written otherwise
- **Don't** drop TempDir in tests - store it in struct to prevent cleanup
- **Don't** add new dependencies without checking if they're already available
- **Don't** use complex async I/O - simple buffered writes are sufficient
- **Don't** hardcode paths - use `.jin/audit/` consistently
- **Don't** make audit logging required - it's informational, not critical

---

## Success Metrics

**Confidence Score**: 9/10 for one-pass implementation success

**Validation Factors**:
- PRD specification is clear and complete
- Integration point (CommitPipeline) is well-defined
- File I/O patterns from existing codebase are well-documented
- Test patterns are established and consistent
- Error handling patterns are consistent across codebase

**Remaining Risk**: Minimal - the main risk is ensuring audit format matches PRD exactly, especially the "layer" integer value.

**Dependencies**: Verify `chrono` crate is in Cargo.toml (if not, add `chrono = "0.4"`)

**Next Steps After Implementation**:
- Consider adding `jin audit` command to view/query audit logs (future work)
- Consider log rotation/retention policies (future work)
- Consider audit log analysis tools (future work)
