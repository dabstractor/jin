# P7.M4.T1: Audit Module Research

## Overview

This research supports the implementation of the Audit Module for the Jin project. The audit system tracks all significant operations for compliance, debugging, and historical analysis.

## PRD Requirements Summary

From PRD Section 17: Audit Logs

### Key Characteristics
1. **Informational, append-only** - Logs are write-only, not modified after creation
2. **Derived from Git commits** - Audit entries are generated from actual Git operations
3. **May be regenerated** - Audit logs can be rebuilt from Git history
4. **Commit hashes included** - Full traceability to Git commits

### Required Format (JSON)
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

### Storage Location
All audit records live in `jin/.audit/` for offline inspection.

## Codebase Patterns Analysis

### Module Organization
- **Recommended location**: `src/audit/`
- Follows the same pattern as `src/core/`, `src/git/`, `src/staging/`, `src/commit/`
- Module should be declared in `src/lib.rs` with selective re-exports

### Error Handling Patterns
- Uses `JinError` enum defined in `src/core/error.rs`
- New error variants should follow the pattern:
  ```rust
  #[error("Audit operation failed: {0}")]
  Audit(String),
  ```
- Result type alias: `pub type Result<T> = std::result::Result<T, JinError>;`

### File I/O Patterns
- **JSON serialization**: `serde_json` for pretty-printed output
- **Atomic writes**: Use temp file + rename pattern for safety
- **Directory creation**: `std::fs::create_dir_all()` with parent check
- **Path handling**: `PathBuf` for all path operations

Example pattern from `src/staging/metadata.rs`:
```rust
pub fn save(&self) -> Result<()> {
    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, content)?;
    std::fs::rename(&temp_path, &path)?;
    Ok(())
}
```

### Testing Patterns
- **Unit tests**: Embedded in `#[cfg(test)]` modules within source files
- **Integration tests**: Located in `tests/` directory
- **Test framework**: Standard Rust `#[test]` with:
  - `assert_cmd = "2.0"` for CLI testing
  - `predicates = "3.0"` for output assertions
  - `tempfile = "3.0"` for isolated test directories
- **Common fixtures**: `tests/common/fixtures.rs` provides `TestFixture` struct

### Integration Points

#### CommitPipeline (`src/commit/pipeline.rs`)
The audit system must integrate with `CommitPipeline::execute()` to capture:
- Committed layers
- File count
- Commit hashes for each layer
- Staging entries that were committed

#### Layer (`src/core/layer.rs`)
Need to capture:
- Layer enum values (1-9)
- Layer precedence
- Layer display names

#### ProjectContext (`src/core/config.rs`)
Need to capture:
- Active mode
- Active scope
- Project name

## External Research Findings

### Recommended Audit Format
**JSON** is the industry standard for audit logging due to:
- Machine readability and structured querying
- Integration with observability platforms
- Support for complex nested structures

### Rust Ecosystem Recommendations

#### Core Dependencies
- `serde` + `serde_json`: Already in project, use for JSON serialization
- `chrono`: For timestamp handling (ISO 8601 with UTC timezone)
- `uuid`: For unique audit event IDs (consider adding)

#### Performance Considerations
- Append-only file operations
- Non-blocking I/O for high-frequency events
- Buffered writes for better throughput
- Time-based rotation (hourly/daily)

### Security & Compliance
- Ensure audit logs are tamper-evident
- Store logs separately from application data
- Implement proper file permissions
- Consider cryptographic signatures for integrity

## Implementation Strategy

### Phase 1: Core Types (P7.M4.T1.S1)
Define `AuditEntry` struct with PRD-compliant fields

### Phase 2: File Operations (P7.M4.T1.S2)
Implement append-only audit log file writing

### Phase 3: Integration (P7.M4.T1.S3)
Integrate with `CommitPipeline` to generate audit entries on commit

## Key Design Decisions

1. **Storage Location**: `.jin/audit/` (not `jin/.audit/` - note the dot prefix)
2. **File Format**: JSON lines (one JSON object per line) for easy parsing
3. **File Naming**: `audit-YYYY-MM-DD.jsonl` for daily rotation
4. **Write Strategy**: Append-only with atomic operations
5. **Error Handling**: Non-blocking - audit failures should not prevent commits

## Sources

### External
- [Rust Tracing Framework](https://github.com/tokio-rs/tracing)
- [Auditing & Accountability Git Best Practices](https://hoop.dev/blog/auditing-accountability-git-best-practices-for-tracking-changes/)
- [JSON Logging Best Practices - Loggly](https://www.loggly.com/use-cases/json-logging-best-practices/)
- [Audit Logging Best Practices - SonarSource](https://www.sonarsource.com/resources/library/audit-logging/)

### Internal
- PRD Section 17: Audit Logs
- `src/commit/pipeline.rs` - CommitPipeline implementation
- `src/core/layer.rs` - Layer enum definition
- `src/core/config.rs` - ProjectContext implementation
- `src/staging/metadata.rs` - File I/O patterns
- `tests/common/fixtures.rs` - Test fixture patterns
