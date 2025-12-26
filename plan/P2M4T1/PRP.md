# Product Requirement Prompt (PRP): 3-Way Text Merge (P2.M4.T1)

---

## Goal

**Feature Goal**: Implement 3-way text merge with Git-style conflict markers for Jin's layer-based configuration system.

**Deliverable**: A new `src/merge/text.rs` module providing 3-way text merge functionality with automatic conflict detection and marker generation.

**Success Definition**:
- Text files from different Jin layers can be merged using 3-way diff algorithm
- Conflicts are detected and marked with standard Git-style conflict markers
- Layer information is included in conflict markers for easy identification
- The merge engine integrates text merge for `FileFormat::Text` files

## Why

- **PRD §11.1**: Text files must use 3-way diff (not simple replacement)
- **Layer merging**: Multiple Jin layers may have conflicting text edits
- **User experience**: Git-standard conflict markers are familiar to developers
- **Deterministic merging**: 3-way merge preserves changes from both sides when possible

## What

### User-Visible Behavior

When merging text files from multiple Jin layers:
1. Non-conflicting changes from both sides are applied automatically
2. Conflicting changes generate standard conflict markers with layer names
3. Users can manually resolve conflicts and recommit

### Technical Requirements

- Use Myers diff algorithm via `similar` crate (already in dependencies)
- Generate Git-standard conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`)
- Include layer paths in marker names (e.g., `mode/claude/scope/language:javascript/`)
- Handle edge cases: empty files, identical content, all-lines conflict

### Success Criteria

- [ ] `src/merge/text.rs` module created with `TextMerge` struct
- [ ] `three_way_merge(base, left, right, left_layer, right_layer)` function implemented
- [ ] Conflict markers generated with layer information
- [ ] Unit tests for: clean merge, single conflict, multiple conflicts, all-lines conflict
- [ ] Integration with `LayerMerge::merge_all()` for text files
- [ ] All tests pass: `cargo test`

---

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" Test**: If someone knew nothing about this codebase, would they have everything needed?

✓ **Yes** - This PRP provides:
- Exact file paths and module structure
- Specific function signatures and patterns to follow
- Test patterns from existing codebase
- Error handling conventions
- External crate API documentation

### Documentation & References

```yaml
# MUST READ - Include these in your context window
- url: https://docs.rs/similar/2.6.0/similar/
  why: API documentation for TextDiff, ChangeTag, and diff operations
  critical:
    - TextDiff::from_lines() for line-based diffing
    - ChangeTag enum for Insert/Delete/Equal detection
    - iter_changes() for iterating over diff results

- url: https://git-scm.com/docs/git-merge-file
  why: Reference for 3-way merge behavior and conflict marker format
  critical:
    - Standard marker format: <<<<<<<, =======, >>>>>>>
    - Marker name conventions for branch identification

- file: /home/dustin/projects/jin-glm-doover/PRD.md
  why: Product requirements for merge behavior (§11.1, §11.3)
  section: §11.1 (merge rules), §11.3 (conflict resolution)
  critical:
    - Text files use 3-way diff
    - Conflict markers must show layer information
    - Example format with layer names

- file: /home/dustin/projects/jin-glm-doover/src/merge/value.rs
  why: MergeValue enum and merge patterns
  pattern: Result<T> return type, error handling with JinError
  gotcha: Text files are currently stored as MergeValue::String (simple replacement)

- file: /home/dustin/projects/jin-glm-doover/src/merge/layer.rs
  why: LayerMerge orchestrator integration point
  pattern: parse_file_by_format() method, FileFormat::Text handling
  gotcha: Line 348 has comment "// 3-way merge in P2.M4" - this is where to integrate

- file: /home/dustin/projects/jin-glm-doover/src/core/error.rs
  why: JinError types for error handling
  pattern: MergeConflict, MergeFailed variants
  gotcha: Use JinError::MergeConflict for conflict detection

- file: /home/dustin/projects/jin-glm-doover/src/core/layer.rs
  why: Layer enum with Display/Debug for marker names
  pattern: impl Display for Layer produces paths like "mode/claude/scope/language:javascript"
  gotcha: Use layer.to_string() for marker names

- docfile: /home/dustin/projects/jin-glm-doover/plan/P2M4T1/research/three_way_merge_algorithm.md
  why: Algorithm explanation and pseudocode for 3-way merge
  section: Algorithm Steps, Pseudocode

- docfile: /home/dustin/projects/jin-glm-doover/plan/P2M4T1/research/similar_crate_usage.md
  why: Concrete API examples for the similar crate
  section: API Examples, Basic Line Diff

- docfile: /home/dustin/projects/jin-glm-doover/plan/P2M4T1/research/conflict_marker_conventions.md
  why: Git-standard conflict marker format and Jin-specific extensions
  section: Jin-Specific Conflict Markers, Implementation Guidelines
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover/
├── Cargo.toml                    # Dependencies: similar = "2.6"
├── src/
│   ├── merge/
│   │   ├── mod.rs                # Module exports
│   │   ├── value.rs              # MergeValue enum, deep merge
│   │   └── layer.rs              # LayerMerge orchestrator
│   └── core/
│       ├── error.rs              # JinError enum
│       └── layer.rs              # Layer enum with Display
```

### Desired Codebase Tree with Files to be Added

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   └── merge/
│       ├── mod.rs                # MODIFY: Add `pub mod text;` and export
│       ├── value.rs              # (existing)
│       ├── layer.rs              # MODIFY: Import text merge, use for Text format
│       └── text.rs               # CREATE: 3-way text merge implementation
└── tests/
        └── merge_test.rs         # MODIFY: Add text merge tests
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: similar crate uses ChangeTag, not a plain enum
use similar::ChangeTag;  // NOT similar::ChangeTag::Insert (wrong)

// CRITICAL: Layer already implements Display for path strings
// Use layer.to_string() directly - don't reconstruct paths
let marker = format!("{} {}", "<<<<<<<", layer.to_string());

// GOTCHA: Files with no extension are FileFormat::Text
// FileFormat::from_path() returns Text for unknown extensions

// CRITICAL: All merge operations must return Result<MergeValue>
// Error type is JinError (from crate::core::error)

// PATTERN: Test naming uses test_<function>_<scenario>
#[test]
fn test_text_merge_clean_merge() { ... }

// GOTCHA: similar crate iter_changes() returns Change<&str>
// The value() method returns &str, not String

// CRITICAL: Text merge only applies to FileFormat::Text
// Structured formats (JSON, YAML, etc.) use deep merge
```

---

## Implementation Blueprint

### Data Models and Structures

No new data models - using existing `JinError`, `Layer`, `FileFormat`, `MergeValue`.

```rust
// New types to define in src/merge/text.rs:

/// Result of a 3-way text merge operation.
pub enum MergeResult {
    /// Merge completed successfully with no conflicts
    Clean(String),
    /// Merge completed with one or more conflicts
    Conflicted(String),
}

/// 3-way text merge orchestrator.
pub struct TextMerge;
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/merge/text.rs
  - IMPLEMENT: TextMerge struct with three_way_merge() static method
  - SIGNATURE: pub fn three_way_merge(base: &str, left: &str, right: &str,
                                      left_layer: &Layer, right_layer: &Layer) -> Result<MergeResult>
  - ALGORITHM: Use similar::TextDiff for base->left and base->right diffs
  - CONFLICTS: Generate markers with layer.to_string() for names
  - NAMING: PascalCase for types, snake_case for functions
  - PLACEMENT: src/merge/text.rs

Task 2: MODIFY src/merge/mod.rs
  - ADD: pub mod text;
  - ADD: pub use text::{TextMerge, MergeResult};
  - PRESERVE: Existing exports (value, layer)
  - PLACEMENT: src/merge/mod.rs (lines 4-7)

Task 3: MODIFY src/merge/layer.rs
  - ADD: use crate::merge::text::TextMerge;
  - MODIFY: parse_file_by_format() method (around line 334)
  - IMPLEMENT: For FileFormat::Text, call TextMerge::three_way_merge()
    - If existing value is MergeValue::String, use as base
    - Parse new content as left/right
    - Store result back as MergeValue::String
  - PRESERVE: Existing JSON/YAML/TOML/INI parsing
  - PLACEMENT: src/merge/layer.rs

Task 4: CREATE tests for text merge in src/merge/text.rs
  - IMPLEMENT: Unit tests in #[cfg(test)] mod tests
  - NAMING: test_text_merge_<scenario>
  - COVERAGE:
    - Clean merge (no conflicts)
    - Single conflict
    - Multiple conflicts
    - All lines conflict
    - Empty base
    - Identical left and right
    - Empty left or right
  - PLACEMENT: src/merge/text.rs (bottom of file)

Task 5: INTEGRATION TESTS (optional but recommended)
  - CREATE: tests/text_merge_test.rs
  - IMPLEMENT: Integration tests with actual LayerMerge
  - VERIFY: Text files from different layers merge correctly
  - PLACEMENT: tests/text_merge_test.rs
```

### Implementation Patterns & Key Details

```rust
// ===== CONFLICT MARKER CONSTANTS =====
// From research/conflict_marker_conventions.md

/// Start marker for conflicts (7 `<` symbols)
pub const CONFLICT_START: &str = "<<<<<<<";

/// Separator marker for conflicts (7 `=` symbols)
pub const CONFLICT_SEPARATOR: &str = "=======";

/// End marker for conflicts (7 `>` symbols)
pub const CONFLICT_END: &str = ">>>>>>>";

// ===== THREE-WAY MERGE ALGORITHM =====
// From research/three_way_merge_algorithm.md

use similar::{TextDiff, ChangeTag};
use crate::core::{Layer, JinError};

impl TextMerge {
    pub fn three_way_merge(
        base: &str,
        left: &str,
        right: &str,
        left_layer: &Layer,
        right_layer: &Layer,
    ) -> Result<MergeResult> {
        // Step 1: Compute diffs
        let base_to_left = TextDiff::from_lines(base, left);
        let base_to_right = TextDiff::from_lines(base, right);

        // Step 2: Collect changes
        let left_changes: Vec<_> = base_to_left.iter_changes(None).collect();
        let right_changes: Vec<_> = base_to_right.iter_changes(None).collect();

        // Step 3: Merge and detect conflicts
        let mut result = Vec::new();
        let mut has_conflicts = false;

        // ... merge logic ...

        if has_conflicts {
            Ok(MergeResult::Conflicted(result.join("\n")))
        } else {
            Ok(MergeResult::Clean(result.join("\n")))
        }
    }
}

// ===== MARKER FORMATTING =====

fn format_marker_start(layer: &Layer) -> String {
    format!("{} {}", CONFLICT_START, layer.to_string())
}

fn format_marker_end(layer: &Layer) -> String {
    format!("{} {}", CONFLICT_END, layer.to_string())
}

// ===== CHANGE DETECTION PATTERN =====

// similar crate uses ChangeTag enum:
use similar::ChangeTag;

match change.tag() {
    ChangeTag::Delete => { /* handle deleted line */ }
    ChangeTag::Insert => { /* handle inserted line */ }
    ChangeTag::Equal => { /* line unchanged */ }
}

// ===== GOTCHA: Handling line endings =====

// similar crate includes newlines in change.value()
// Be careful not to double-newline when reconstructing

for change in diff.iter_changes(None) {
    let value = change.value();
    // value already contains \n if present
    result.push_str(value);
}
```

### Integration Points

```yaml
LAYER MERGE:
  - file: src/merge/layer.rs
  - method: parse_file_by_format()
  - pattern: "Match FileFormat::Text case"
  - integration:
    ```rust
    FileFormat::Text => {
        // Check if we have an existing value to use as base
        let base = if let Some(MergeValue::String(existing)) =
            context.merged_files.get(path) {
            Some(existing.as_str())
        } else {
            Some("")  // No base - empty string
        };

        // For now, store as string (full 3-way merge in LayerMerge)
        Ok(MergeValue::String(content_str.to_string()))
    }
    ```

ERROR HANDLING:
  - use: JinError::MergeConflict for detected conflicts
  - use: JinError::MergeFailed for algorithm errors
  - return: Result<MergeResult> type

EXPORTS:
  - file: src/merge/mod.rs
  - add: pub use text::{TextMerge, MergeResult};
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check --color=always                   # Check compilation
cargo clippy --color=always -W clippy::all   # Lint checks

# Project-wide validation
cargo check --all-targets
cargo clippy --all-targets -W clippy::all

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test text merge module specifically
cargo test --lib merge::text -- --nocapture --test-threads=1

# Run with output
cargo test --lib text_merge -- --nocapture

# Full merge module tests
cargo test --lib merge -- --test-threads=1

# Expected: All tests pass. Review test output for correctness.
```

### Level 3: Integration Testing (System Validation)

```bash
# Run integration tests
cargo test --test merge_test -- --nocapture

# Full test suite
cargo test --all -- --test-threads=1

# Expected: All tests pass, including integration tests.
```

### Level 4: Manual Validation (Creative & Domain-Specific)

```bash
# Manual test: Create conflicting text files
mkdir -p /tmp/jin-merge-test
cd /tmp/jin-merge-test

# Initialize a Jin repo
cargo run -- init

# Create test layers with conflicting text files
# ... (manual testing scenario)

# Expected: Text files merge with proper conflict markers showing layer names
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --all`
- [ ] No clippy warnings: `cargo clippy --all-targets -W clippy::all`
- [ ] No compilation errors: `cargo check --all-targets`

### Feature Validation

- [ ] Text files from different layers can be merged
- [ ] Non-conflicting changes auto-merge correctly
- [ ] Conflicts generate Git-standard markers with layer names
- [ ] Layer names in markers match Layer::to_string() format
- [ ] Empty files handled gracefully
- [ ] Identical content handled correctly

### Code Quality Validation

- [ ] Follows existing codebase patterns (JinError, Result types)
- [ ] File placement matches desired codebase tree
- [ ] Public functions have doc comments
- [ ] Tests cover all scenarios (clean, single conflict, multiple conflicts)
- [ ] Module exports properly in src/merge/mod.rs

### Integration Validation

- [ ] TextMerge is exported from merge module
- [ ] LayerMerge::parse_file_by_format() can use text merge
- [ ] Error handling matches existing patterns

---

## Anti-Patterns to Avoid

- **Don't** create new error types - use `JinError::MergeConflict`
- **Don't** implement your own diff algorithm - use the `similar` crate
- **Don't** skip tests - this is critical functionality for data integrity
- **Don't** use non-standard conflict markers - follow Git conventions
- **Don't** forget to handle edge cases (empty files, all-lines conflict)
- **Don't** ignore the layer information in conflict markers
- **Don't** hardcode marker strings - use constants
- **Don't** double-newline when reconstructing merged text

---

## Success Metrics

**Confidence Score**: 9/10 for one-pass implementation success

**Rationale**:
- Comprehensive research provided (algorithm, API, conventions)
- Existing codebase patterns clearly documented
- External crate (`similar`) already in dependencies
- Test patterns established in existing modules
- Integration points explicitly specified

**Confidence Impact**: The `similar` crate's well-documented API and established patterns in `value.rs` provide clear guidance. The only risk area is the merge algorithm complexity, which is mitigated by the research documentation provided.

---

## Appendix: Quick Reference

### MergeResult Enum

```rust
pub enum MergeResult {
    Clean(String),      // No conflicts - merged text
    Conflicted(String), // Has conflicts - text with markers
}

impl MergeResult {
    /// Returns true if this result has conflicts
    pub fn has_conflicts(&self) -> bool {
        matches!(self, MergeResult::Conflicted(_))
    }

    /// Returns the merged text
    pub fn into_text(self) -> String {
        match self {
            MergeResult::Clean(s) => s,
            MergeResult::Conflicted(s) => s,
        }
    }
}
```

### Example Usage

```rust
use jin_glm::merge::{TextMerge, MergeResult};
use jin_glm::core::Layer;

let base = "line 1\nline 2\nline 3";
let left = "line 1 modified\nline 2\nline 3";
let right = "line 1\nline 2 modified\nline 3";

let left_layer = Layer::ModeBase { mode: "claude".to_string() };
let right_layer = Layer::ProjectBase { project: "myproject".to_string() };

let result = TextMerge::three_way_merge(base, left, right, &left_layer, &right_layer)?;

assert!(!result.has_conflicts());
assert_eq!(result.into_text(), "line 1 modified\nline 2 modified\nline 3");
```

### Conflict Example

```rust
let base = "line 1\nline 2";
let left = "changed by left\nline 2";
let right = "changed by right\nline 2";

let result = TextMerge::three_way_merge(base, left, right, &left_layer, &right_layer)?;

assert!(result.has_conflicts());
let text = result.into_text();
assert!(text.contains("<<<<<<< mode/claude"));
assert!(text.contains("changed by left"));
assert!(text.contains("======="));
assert!(text.contains("changed by right"));
assert!(text.contains(">>>>>>> project/myproject"));
```
