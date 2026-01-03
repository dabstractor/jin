name: "P1.M1.T1: Create .jinmerge File Format Module"
description: |
  Implement the core data structures and file format for .jinmerge conflict files that show layer-aware conflict markers with full ref paths.

---

## Goal

**Feature Goal**: Create a reusable `src/merge/jinmerge.rs` module with layer-aware conflict file generation and parsing capabilities.

**Deliverable**: A new Rust module that:
- Defines `JinMergeConflict` and related data structures
- Generates `.jinmerge` files with Git-compatible conflict markers containing layer ref paths as labels
- Parses `.jinmerge` files to extract conflict regions and metadata
- Integrates cleanly with existing `src/merge/mod.rs`

**Success Definition**:
- Unit tests pass for all data structure methods
- File generation produces valid Git-conflict-format output with layer-aware labels
- File parsing correctly extracts conflict regions with line numbers and layer refs
- Module compiles without warnings (`cargo check --release`)
- Integration with `apply` command in P1.M1.T2 succeeds without modifications

## User Persona

**Target User**: Jin developers implementing the conflict resolution workflow, and end users who will manually resolve layer conflicts.

**Use Case**: When `jin apply` detects conflicts between layers (e.g., between `mode/claude/scope:javascript/` and `mode/claude/project/ui-dashboard/`), it creates `.jinmerge` files that users can edit to resolve conflicts.

**User Journey**:
1. User runs `jin apply` to merge layers
2. Apply detects conflict in `config.json` between two layers
3. Apply writes `config.json.jinmerge` with conflict markers showing layer ref paths
4. User opens `config.json.jinmerge` and sees:
   ```
   <<<<<<< mode/claude/scope:javascript/
   {"target": "es6", "modules": true}
   =======
   {"target": "es2020", "modules": false, "strict": true}
   >>>>>>> mode/claude/project/ui-dashboard/
   ```
5. User edits to resolve conflict and saves
6. User runs `jin resolve config.json.jinmerge`
7. Apply resumes with resolved content

**Pain Points Addressed**:
- **Generic "ours/theirs" labels** in current diffy output don't show which layers conflict
- **No structured conflict metadata** for tooling integration
- **No round-trippable conflict format** - conflicts must be manually tracked

## Why

- **Layer-aware conflict resolution** is critical for Jin's multi-layer system - users need to know which specific layers are conflicting
- **Git tooling compatibility** allows users to use existing Git conflict resolution tools (editors, merge tools)
- **Structured metadata** enables future UI/tooling enhancements (conflict highlighting, resolution suggestions)
- **PRD requirement** - Section 4.6.3 explicitly requires `.jinmerge` file format with layer ref paths

## What

### Module Structure

**File**: `src/merge/jinmerge.rs`

**Public API**:
```rust
/// Represents a complete .jinmerge file with metadata and conflict regions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JinMergeConflict {
    pub file_path: PathBuf,
    pub conflicts: Vec<JinMergeRegion>,
}

/// Represents a single conflict region with layer-aware labels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JinMergeRegion {
    pub layer1_ref: String,     // Full ref path: "mode/claude/scope:javascript/"
    pub layer1_content: String,  // Content from layer1
    pub layer2_ref: String,     // Full ref path: "mode/claude/project/ui-dashboard/"
    pub layer2_content: String,  // Content from layer2
    pub start_line: usize,       // 1-indexed line number
    pub end_line: usize,         // 1-indexed line number
}

/// Header comment added to .jinmerge files
pub const JINMERGE_HEADER: &str = "# Jin merge conflict. Resolve and run 'jin resolve <file>'";
```

**Key Functions**:
```rust
impl JinMergeConflict {
    /// Create from text merge result and layer ref paths
    pub fn from_text_merge(
        file_path: PathBuf,
        layer1_ref: String,
        layer1_content: String,
        layer2_ref: String,
        layer2_content: String,
    ) -> Self

    /// Write to .jinmerge file with layer-aware markers
    pub fn write_to_file(&self, merge_path: &Path) -> Result<()>

    /// Parse existing .jinmerge file
    pub fn parse_from_file(merge_path: &Path) -> Result<Self>

    /// Count total conflict regions
    pub fn conflict_count(&self) -> usize

    /// Check if file appears to be a .jinmerge file
    pub fn is_jinmerge_file(path: &Path) -> bool
}
```

### File Format Specification

**Format**: Human-readable text with Git-compatible conflict markers

**Structure**:
```
# Jin merge conflict. Resolve and run 'jin resolve <file>'
<<<<<<< mode/claude/scope:javascript/
content from layer1
=======
content from layer2
>>>>>>> mode/claude/project/ui-dashboard/

# Multiple conflicts in one file:
# Jin merge conflict. Resolve and run 'jin resolve <file>'
<<<<<<< mode/claude/scope:javascript/
first conflict - layer1
=======
first conflict - layer2
>>>>>>> mode/claude/project/ui-dashboard/

<<<<<<< mode/claude/scope:javascript/
second conflict - layer1
=======
second conflict - layer2
>>>>>>> mode/claude/project/ui-dashboard/
```

**Markers**:
- Start: `<<<<<<< ` (7 `<` followed by space + layer ref)
- Separator: `=======` (7 `=`)
- End: `>>>>>>> ` (7 `>` followed by space + layer ref)

### Success Criteria

- [ ] `JinMergeConflict` struct defined with all required fields
- [ ] `JinMergeRegion` struct defined with layer refs and content
- [ ] `write_to_file()` generates valid Git-conflict-format output
- [ ] `parse_from_file()` correctly extracts all conflict regions
- [ ] Layer ref paths are preserved as labels (not generic "ours"/"theirs")
- [ ] Module compiles without warnings
- [ ] Unit tests cover all public methods
- [ ] Module added to `src/merge/mod.rs` exports

## All Needed Context

### Context Completeness Check

**Question**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: Yes - this PRP provides:
- Exact file paths and code patterns to follow
- Existing data structures to integrate with (`ConflictRegion`, `TextMergeResult`)
- Specific crate dependencies and their usage patterns
- Test patterns matching the codebase conventions
- Known gotchas and constraints

### Documentation & References

```yaml
# MUST READ - Core patterns to follow

- file: /home/dustin/projects/jin/src/merge/text.rs
  why: "TextMergeResult, ConflictRegion, TextMergeConfig patterns"
  pattern: "Enum result types (Clean/Conflict), derive macros, line numbering"
  gotcha: "ConflictRegion uses 1-indexed line numbers for user display"

- file: /home/dustin/projects/jin/src/merge/mod.rs
  why: "Module export pattern - how to expose new types"
  pattern: "pub use statements for re-exports, module documentation"
  gotcha: "Must add pub use statements for JinMergeConflict, JinMergeRegion"

- file: /home/dustin/projects/jin/src/core/error.rs
  why: "JinError enum for error handling patterns"
  pattern: "Use thiserror for error enums, Result<T> type alias"
  gotcha: "Use JinError::Parse for format errors, JinError::Io for file I/O"

- file: /home/dustin/projects/jin/src/core/jinmap.rs
  why: "Atomic write pattern for file persistence"
  pattern: "Write to temp file, then rename - prevents corruption"
  gotcha: "Always use atomic writes for config files"

- file: /home/dustin/projects/jin/src/staging/metadata.rs
  why: "JSON serialization pattern with serde_json"
  pattern: "serde_json::to_string_pretty for human-readable output"
  gotcha: "Save() method uses temp file + rename pattern"

- file: /home/dustin/projects/jin/src/merge/value.rs
  why: "MergeValue enum for structured data representation"
  pattern: " serde(untagged) for enum representation, comprehensive derive macros"
  gotcha: "IndexMap for preserving key order in objects"

# EXTERNAL DEPENDENCIES

- url: https://docs.rs/serde/latest/serde/
  why: "Serialization framework - derive macros for Serialize/Deserialize"
  critical: "All structs must have #[derive(Serialize, Deserialize)] for JSON support"

- url: https://docs.rs/thiserror/latest/thiserror/
  why: "Error handling - JinError uses thiserror::Error derive macro"
  critical: "Use JinError::Parse variant for .jinmerge format errors"

- url: https://git-scm.com/docs/git-merge#_how_conflicts_are_presented
  why: "Git's conflict marker format specification"
  critical: "Must match Git's 7-character marker format exactly"

# PLAN DOCUMENTS

- docfile: /home/dustin/projects/jin/plan/architecture/external_deps.md
  why: "Explains why diffy cannot be used - custom marker generation required"
  section: "Section 1.2 Text Merging: diffy (v0.4)"
  critical: "diffy markers are hardcoded to 'ours'/'theirs' - cannot use for layer refs"

- docfile: /home/dustin/projects/jin/plan/docs/TEXT_MERGE_RESEARCH.md
  why: "3-way merge algorithm details and conflict marker format"
  section: "Conflict Marker Format (Git Standard)"
  critical: "Marker format: <<<<<<< , ======= , >>>>>>>"

- docfile: /home/dustin/projects/jin/plan/SYNTHESIS_SUMMARY.md
  why: "Overall project context and task relationships"
  section: "Milestone 1.1: .jinmerge Conflict Resolution"
```

### Current Codebase Tree

```bash
src/
├── merge/
│   ├── mod.rs          # Module exports (ADD: pub use jinmerge::*)
│   ├── deep.rs         # RFC 7396 deep merge
│   ├── layer.rs        # Layer merge orchestration
│   ├── text.rs         # 3-way text merge (TextMergeResult, ConflictRegion)
│   ├── value.rs        # MergeValue enum
│   └── jinmerge.rs     # [NEW] JinMergeConflict, JinMergeRegion
├── core/
│   ├── error.rs        # JinError enum (use for error handling)
│   ├── jinmap.rs       # Atomic write pattern reference
│   └── layer.rs        # Layer enum with ref_path() method
├── commands/
│   └── apply.rs        # Will use JinMergeConflict in P1.M1.T2
└── lib.rs
```

### Desired Codebase Tree

```bash
src/
├── merge/
│   ├── mod.rs          # UPDATED: Add pub use jinmerge::{JinMergeConflict, JinMergeRegion}
│   ├── deep.rs         # (unchanged)
│   ├── layer.rs        # (unchanged)
│   ├── text.rs         # (unchanged)
│   ├── value.rs        # (unchanged)
│   └── jinmerge.rs     # [NEW] Module with JinMergeConflict, JinMergeRegion
│       ├── JinMergeConflict struct
│       ├── JinMergeRegion struct
│       ├── write_to_file() method
│       ├── parse_from_file() method
│       └── is_jinmerge_file() helper
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: Cannot use diffy crate for marker generation
// diffy's markers are hardcoded to "ours"/"theirs" - no API for customization
// Must implement custom marker formatting with layer ref paths

// CRITICAL: Git conflict marker format is EXACTLY 7 characters
<<<<<<<        // 7 < characters
=======         // 7 = characters
>>>>>>>        // 7 > characters
// No more, no less - Git parsers expect exactly 7

// CRITICAL: Line numbers in ConflictRegion are 1-indexed
// This is for user-facing display (line 1 is first line, not 0)
start_line: 1,  // First line of conflict
end_line: 5,    // Inclusive - includes marker line

// CRITICAL: Atomic write pattern prevents corruption
// Always write to temp file, then rename atomically
let temp_path = path.with_extension("tmp");
std::fs::write(&temp_path, content)?;
std::fs::rename(&temp_path, &path)?;

// CRITICAL: Layer ref paths use specific format
// Format: "mode/{mode_name}/scope:{scope_name}/" or similar
// See src/core/layer.rs:Layer::ref_path() for exact format

// CRITICAL: Text content handling
// Preserve trailing newlines - "text\n" vs "text" are semantically different
// Use .lines().collect::<Vec<_>>() for line-by-line parsing

// CRITICAL: Derive macros order matters for serde
// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// Serialize/Deserialize must come after Clone for serde to work correctly

// CRITICAL: Error handling pattern
// Use JinError::Parse with format: "jinmerge" for parsing errors
// Use JinError::Io (from std::io::Error) for file I/O errors
// Return Result<()> from methods that can fail
```

## Implementation Blueprint

### Data Models and Structure

```rust
// File: src/merge/jinmerge.rs

use crate::core::{JinError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Header comment added to all .jinmerge files
pub const JINMERGE_HEADER: &str = "# Jin merge conflict. Resolve and run 'jin resolve <file>'";

/// Marker constants (Git-compatible - exactly 7 characters)
pub const MARKER_START: &str = "<<<<<<< ";
pub const MARKER_SEP: &str = "=======";
pub const MARKER_END: &str = ">>>>>>> ";

/// Represents a single conflict region with layer-aware labels
///
/// This structure captures the two conflicting versions along with their
/// layer ref paths for clear conflict resolution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JinMergeRegion {
    /// Full ref path for first layer (e.g., "mode/claude/scope:javascript/")
    pub layer1_ref: String,
    /// Content from first layer
    pub layer1_content: String,
    /// Full ref path for second layer (e.g., "mode/claude/project/ui-dashboard/")
    pub layer2_ref: String,
    /// Content from second layer
    pub layer2_content: String,
    /// Starting line number (1-indexed, for user display)
    pub start_line: usize,
    /// Ending line number (1-indexed, inclusive)
    pub end_line: usize,
}

/// Represents a complete .jinmerge file
///
/// Contains the original file path and all conflict regions.
/// Can be written to disk (with .jinmerge extension) or parsed from existing files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JinMergeConflict {
    /// Original file path (without .jinmerge extension)
    pub file_path: PathBuf,
    /// All conflict regions in the file
    pub conflicts: Vec<JinMergeRegion>,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/merge/jinmerge.rs with basic structures
  - IMPLEMENT: JinMergeRegion struct with 6 fields (layer1_ref, layer1_content, layer2_ref, layer2_content, start_line, end_line)
  - IMPLEMENT: JinMergeConflict struct with 2 fields (file_path, conflicts)
  - IMPLEMENT: JINMERGE_HEADER, MARKER_START, MARKER_SEP, MARKER_END constants
  - FOLLOW pattern: src/merge/text.rs (ConflictRegion structure)
  - NAMING: PascalCase for types, SCREAMING_SNAKE_CASE for constants
  - DERIVES: Debug, Clone, Serialize, Deserialize, PartialEq for all structs
  - PLACEMENT: New file src/merge/jinmerge.rs

Task 2: IMPLEMENT JinMergeConflict::from_text_merge() constructor
  - IMPLEMENT: from_text_merge() method that takes file_path, layer refs, and content
  - CREATE: Single JinMergeRegion with provided content
  - CALCULATE: start_line = 1, end_line based on content line count
  - FOLLOW pattern: src/merge/text.rs (TextMergeConfig::with_labels())
  - NAMING: from_text_merge for clarity (not new() - specific use case)
  - RETURN: JinMergeConflict instance
  - PLACEMENT: impl block for JinMergeConflict

Task 3: IMPLEMENT JinMergeConflict::write_to_file() method
  - IMPLEMENT: write_to_file(&self, merge_path: &Path) -> Result<()>
  - GENERATE: Content with header + conflict markers using layer refs
  - FORMAT: Each conflict as "<<<<<<< {ref}\n{content}\n=======\n{content}\n>>>>>>> {ref}\n"
  - ADD: JINMERGE_HEADER as first line
  - FOLLOW pattern: src/core/jinmap.rs:save() (atomic write with temp file)
  - NAMING: write_to_file (not save - more descriptive)
  - GOTCHA: Use atomic write pattern (temp file + rename)
  - PLACEMENT: impl block for JinMergeConflict

Task 4: IMPLEMENT JinMergeConflict::parse_from_file() method
  - IMPLEMENT: parse_from_file(merge_path: &Path) -> Result<Self>
  - READ: File content using std::fs::read_to_string()
  - PARSE: Line-by-line to extract conflict regions and layer refs
  - VALIDATE: All three markers present (start, sep, end)
  - EXTRACT: layer1_ref from start marker line (after <<<<<<< )
  - EXTRACT: layer2_ref from end marker line (after >>>>>>> )
  - FOLLOW pattern: src/merge/text.rs:parse_conflicts() (similar parsing logic)
  - NAMING: parse_from_file (not load or read - consistent with write_to_file)
  - ERROR: Return JinError::Parse with format "jinmerge" on malformed markers
  - PLACEMENT: impl block for JinMergeConflict

Task 5: IMPLEMENT helper methods
  - IMPLEMENT: conflict_count(&self) -> usize (returns conflicts.len())
  - IMPLEMENT: is_jinmerge_file(path: &Path) -> bool (checks extension + header)
  - IMPLEMENT: merge_path_for_file(original: &Path) -> PathBuf (adds .jinmerge extension)
  - FOLLOW pattern: src/merge/text.rs (has_conflict_markers function style)
  - NAMING: snake_case for functions, descriptive names
  - PLACEMENT: impl block for JinMergeConflict

Task 6: MODIFY src/merge/mod.rs to export new types
  - ADD: pub mod jinmerge; declaration
  - ADD: pub use jinmerge::{JinMergeConflict, JinMergeRegion, JINMERGE_HEADER};
  - PRESERVE: All existing exports
  - PLACEMENT: Top of file after other pub mod declarations

Task 7: CREATE src/merge/jinmerge.rs unit tests
  - IMPLEMENT: #[cfg(test)] mod tests with comprehensive coverage
  - TEST: from_text_merge() constructor
  - TEST: write_to_file() generates correct format
  - TEST: parse_from_file() extracts all regions
  - TEST: round-trip (write then parse yields same data)
  - TEST: is_jinmerge_file() with various extensions
  - TEST: merge_path_for_file() adds .jinmerge correctly
  - TEST: error cases (missing markers, malformed format)
  - FOLLOW pattern: src/merge/text.rs tests (test structure, assertions)
  - NAMING: test_{method}_{scenario} (e.g., test_write_to_file_single_conflict)
  - COVERAGE: All public methods with positive and negative cases
  - PLACEMENT: End of src/merge/jinmerge.rs file
```

### Implementation Patterns & Key Details

```rust
// ============================================================================
// Pattern: Marker Generation with Layer Refs (NOT using diffy)
// ============================================================================

// CRITICAL: Custom implementation required - diffy cannot do layer ref labels
fn format_conflict_region(region: &JinMergeRegion) -> String {
    format!(
        "{}{}\n{}\n{}\n{}\n{}\n{}\n{}\n",
        MARKER_START,
        region.layer1_ref,
        region.layer1_content,
        MARKER_SEP,
        region.layer2_content,
        MARKER_END,
        region.layer2_ref
    )
}

// Example output:
// <<<<<<< mode/claude/scope:javascript/
// {"target": "es6"}
// =======
// {"target": "es2020"}
// >>>>>>> mode/claude/project/ui-dashboard/

// ============================================================================
// Pattern: Line-by-line Conflict Parsing
// ============================================================================

fn parse_conflict_regions(content: &str) -> Result<Vec<JinMergeRegion>> {
    let lines: Vec<&str> = content.lines().collect();
    let mut regions = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        if lines[i].starts_with(MARKER_START.trim()) {
            let start_line = i + 1; // 1-indexed
            let layer1_ref = lines[i][MARKER_START.len()..].trim().to_string();

            // Find separator
            let sep_idx = lines[i..].iter()
                .position(|l| l.starts_with(MARKER_SEP))
                .ok_or_else(|| JinError::Parse {
                    format: "jinmerge".to_string(),
                    message: "Missing separator marker".to_string(),
                })?;
            let sep_idx = i + sep_idx;

            // Find end marker
            let end_idx = lines[sep_idx..].iter()
                .position(|l| l.starts_with(MARKER_END.trim()))
                .ok_or_else(|| JinError::Parse {
                    format: "jinmerge".to_string(),
                    message: "Missing end marker".to_string(),
                })?;
            let end_idx = sep_idx + end_idx;

            let layer2_ref = lines[end_idx][MARKER_END.len()..].trim().to_string();
            let layer1_content = lines[i + 1..sep_idx].join("\n");
            let layer2_content = lines[sep_idx + 1..end_idx].join("\n");

            regions.push(JinMergeRegion {
                layer1_ref,
                layer1_content,
                layer2_ref,
                layer2_content,
                start_line,
                end_line: end_idx + 1,
            });

            i = end_idx + 1;
        } else {
            i += 1;
        }
    }

    Ok(regions)
}

// ============================================================================
// Pattern: Atomic Write (from src/core/jinmap.rs)
// ============================================================================

pub fn write_to_file(&self, merge_path: &Path) -> Result<()> {
    let content = self.to_jinmerge_format()?;
    let temp_path = merge_path.with_extension("jinmerge.tmp");

    // Write to temp file first
    std::fs::write(&temp_path, content).map_err(JinError::Io)?;

    // Atomic rename
    std::fs::rename(&temp_path, merge_path).map_err(JinError::Io)?;

    Ok(())
}

// ============================================================================
// Pattern: Header Line Detection
// ============================================================================

pub fn is_jinmerge_file(path: &Path) -> bool {
    if path.extension().and_then(|s| s.to_str()) != Some("jinmerge") {
        return false;
    }

    // Check first line for header
    match std::fs::read_to_string(path) {
        Ok(content) => {
            content.lines().next()
                .map(|line| line.starts_with("# Jin merge conflict"))
                .unwrap_or(false)
        }
        Err(_) => false,
    }
}

// ============================================================================
// GOTCHA: Layer Ref Path Format
// ============================================================================

// Layer refs must match the format from src/core/layer.rs:Layer::ref_path()
// Examples:
// - "global/"
// - "mode/claude/"
// - "mode/claude/scope:javascript/"
// - "mode/claude/project/ui-dashboard/"
// - "scope:language:rust/"
// - "project/my-project/"

// The ref_path() method signature:
// pub fn ref_path(&self, mode: Option<&str>, scope: Option<&str>, project: Option<&str>) -> String
```

### Integration Points

```yaml
MERGE_MODULE:
  - modify: src/merge/mod.rs
  - add: "pub mod jinmerge;"
  - add: "pub use jinmerge::{JinMergeConflict, JinMergeRegion, JINMERGE_HEADER};"
  - after: "pub mod text;" declaration

FUTURE_APPLY_COMMAND:
  - will_use: JinMergeConflict::from_text_merge() in P1.M1.T2
  - will_use: JinMergeConflict::write_to_file() in P1.M1.T2
  - will_use: JinMergeConflict::is_jinmerge_file() in P1.M1.T2

FUTURE_RESOLVE_COMMAND:
  - will_use: JinMergeConflict::parse_from_file() in P1.M1.T3
  - will_use: JinMergeConflict::conflict_count() in P1.M1.T3
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after completing implementation - fix before proceeding
cargo check --release                          # Full compilation check
cargo check --release -p jin 2>&1 | grep jinmerge  # Check for warnings in new module

# Expected: Zero errors, zero warnings. If errors exist:
# 1. Read error message carefully
# 2. Check derive macros order (Serialize/Deserialize after Clone)
# 3. Verify imports (use crate::core::{JinError, Result})
# 4. Check for missing use statements

# Format check
cargo fmt --check                              # Verify formatting
cargo fmt                                      # Auto-format if needed

# Expected: No formatting errors. Run cargo fmt to fix.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run tests for jinmerge module specifically
cargo test --lib jinmerge -- --nocapture       # Run jinmerge tests with output

# Run all merge module tests
cargo test --lib merge::                       # Verify no regressions in merge module

# Run tests with output for debugging
cargo test --lib jinmerge::tests::test_write_to_file -- --nocapture --show-output

# Expected: All tests pass. If failing:
# 1. Check test output for specific assertion failures
# 2. Verify marker format (exactly 7 characters)
# 3. Check line number calculations (1-indexed)
# 4. Validate atomic write pattern (temp file exists then renamed)
```

### Level 3: Integration Testing (System Validation)

```bash
# Create test .jinmerge file manually
cat > /tmp/test.json.jinmerge << 'EOF'
# Jin merge conflict. Resolve and run 'jin resolve <file>'
<<<<<<< mode/claude/scope:javascript/
{"target": "es6"}
=======
{"target": "es2020"}
>>>>>>> mode/claude/project/ui-dashboard/
EOF

# Test parsing via Rust (one-liner)
cargo run --example parse_jinmerge -- /tmp/test.json.jinmerge 2>/dev/null || echo "Example not needed - unit tests cover this"

# Verify file is detected as .jinmerge
test -f /tmp/test.json.jinmerge && echo "File exists" || echo "File creation failed"

# Expected: File parses correctly, is_jinmerge_file() returns true
```

### Level 4: Manual Validation

```bash
# Test 1: Create .jinmerge file manually and verify format
cat > /tmp/manual_test.json.jinmerge << 'EOF'
# Jin merge conflict. Resolve and run 'jin resolve <file>'
<<<<<<< mode/claude/scope:javascript/
line1
line2
=======
lineA
lineB
>>>>>>> mode/claude/project/ui-dashboard/
EOF

# Verify marker lengths
echo "Start marker: $(head -2 /tmp/manual_test.json.jinmerge | tail -1 | wc -c)"
echo "Expected: 7 + ref length"

# Test 2: Verify header detection
head -1 /tmp/manual_test.json.jinmerge | grep -q "# Jin merge conflict" && echo "Header OK" || echo "Header missing"

# Test 3: Check extension handling
echo "/tmp/test.json.jinmerge" | grep -q "\.jinmerge$" && echo "Extension OK" || echo "Extension wrong"

# Expected: All validations pass
```

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check --release` completes with zero errors
- [ ] `cargo clippy --release -p jin` produces zero warnings
- [ ] `cargo fmt --check` passes (code is formatted)
- [ ] `cargo test --lib jinmerge` passes all tests
- [ ] `cargo test --lib merge::` passes all merge module tests
- [ ] No new warnings added to compilation output

### Feature Validation

- [ ] `JinMergeConflict` struct compiles with all 6 required derives
- [ ] `JinMergeRegion` struct compiles with all 6 required derives
- [ ] `write_to_file()` generates Git-compatible conflict markers
- [ ] `parse_from_file()` correctly extracts layer refs from markers
- [ ] `is_jinmerge_file()` returns true only for valid .jinmerge files
- [ ] `conflict_count()` returns accurate count
- [ ] Constants `JINMERGE_HEADER`, `MARKER_START`, `MARKER_SEP`, `MARKER_END` defined

### Code Quality Validation

- [ ] All public methods have doc comments (`///`)
- [ ] Error handling uses `JinError::Parse` for format errors
- [ ] File I/O uses atomic write pattern (temp file + rename)
- [ ] Line numbers are 1-indexed (matching `ConflictRegion` pattern)
- [ ] Layer ref format matches `Layer::ref_path()` output
- [ ] Module added to `src/merge/mod.rs` with proper exports

### Documentation & Deployment

- [ ] Module-level documentation (`//!`) explains purpose
- [ ] Public API documentation includes examples
- [ ] Constants have documentation explaining their purpose
- [ ] Error messages are descriptive and actionable

---

## Anti-Patterns to Avoid

- **Don't use diffy crate for marker generation** - diffy's markers are hardcoded to "ours"/"theirs", must implement custom formatting
- **Don't hardcode layer ref paths** - always accept as parameters or extract from existing layer refs
- **Don't skip atomic write pattern** - always write to temp file first, then rename
- **Don't use 0-indexed line numbers** - ConflictRegion uses 1-indexed for user display
- **Don't forget newline handling** - preserve trailing newlines, they're semantically significant
- **Don't use generic error types** - use `JinError::Parse { format: "jinmerge", ... }` for format errors
- **Don't skip the header comment** - all .jinmerge files must start with `JINMERGE_HEADER`
- **Don't use incorrect marker lengths** - Git markers are EXACTLY 7 characters each
- **Don't order derives incorrectly** - `Serialize, Deserialize` must come after `Clone`
- **Don't forget pub use exports** - add to `src/merge/mod.rs` for external access

---

## Confidence Score

**Score: 9/10**

**Rationale**:
- Comprehensive codebase analysis with exact file paths and patterns
- Clear specification of data structures matching existing patterns
- Specific gotchas identified (diffy limitations, marker format, atomic writes)
- Test patterns well-defined with existing examples
- Error handling follows established `JinError` patterns
- Integration points clearly defined

**Risk Mitigation**:
- The -1 is due to potential edge cases in conflict parsing (nested markers, malformed input)
- Mitigation: Comprehensive unit tests with edge cases will catch these issues early
