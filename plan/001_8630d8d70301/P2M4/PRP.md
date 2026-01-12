# PRP: P2.M4 - Text Merge

**Milestone**: P2.M4
**Phase**: P2 - Merge Engine
**Status**: Ready for Implementation
**Dependencies**: P2.M1 (MergeValue), P2.M2 (Format Parsers), P2.M3 (Deep Merge) - All Complete

---

## Goal

**Feature Goal**: Implement a production-ready 3-way text merge algorithm that handles line-level merging with proper conflict detection and resolution for plain text files in Jin's 9-layer hierarchy.

**Deliverable**: A complete `src/merge/text.rs` module that replaces the current skeleton with:
1. Line-level 3-way merge using the `diffy` crate
2. Multiple conflict region detection and tracking
3. Configurable conflict marker formatting (standard and diff3 styles)
4. Conflict parsing for resolution workflows
5. Integration with layer merge orchestration

**Success Definition**:
- All existing tests pass plus 25+ new tests covering edge cases
- `cargo test merge::text` passes with 100% coverage of public API
- Text files with non-overlapping changes merge cleanly
- Overlapping changes produce properly formatted conflict markers
- `parse_conflicts()` correctly extracts all conflict regions

---

## User Persona

**Target User**: Developer using Jin to manage configuration files across multiple layers

**Use Case**: When a text file (README.md, script.sh, .gitignore, etc.) exists in multiple layers with different content, Jin must merge them intelligently rather than simply overwriting.

**User Journey**:
1. Developer runs `jin apply` to materialize workspace
2. Jin detects text files exist in multiple layers
3. Jin performs 3-way merge: base (lowest layer) + ours (accumulated) + theirs (current layer)
4. If clean: merged content written to workspace
5. If conflicts: conflict markers inserted, file added to `conflict_files` list
6. Developer resolves conflicts manually if needed

**Pain Points Addressed**:
- Current behavior overwrites text files (highest layer wins) - loses changes
- No visibility into what conflicted or why
- Manual merging required for any multi-layer text file

---

## Why

- **Core Requirement**: PRD §11.1 specifies "Plain text: 3-way diff" as merge strategy
- **Completes Merge Engine**: P2.M4 is the final milestone of the Merge Engine phase
- **Enables Text Config**: Many config files are plain text (scripts, dotfiles, READMEs)
- **User Expectation**: Developers expect Git-like merge behavior for text files
- **Foundation for Commands**: Required by `jin apply`, `jin commit`, `jin sync` commands

---

## What

### Core Requirements

1. **Line-Level 3-Way Merge**: Merge text files at line granularity using diff algorithms
2. **Conflict Detection**: Identify overlapping changes that cannot be auto-resolved
3. **Conflict Markers**: Generate Git-compatible conflict markers with layer information
4. **Multiple Regions**: Handle files with multiple separate conflict regions
5. **Conflict Parsing**: Extract conflict regions for resolution UI/tooling
6. **Clean Integration**: Work seamlessly with existing `merge_layers()` orchestration

### Success Criteria

- [ ] `text_merge(base, ours, theirs)` returns `TextMergeResult::Clean` for non-overlapping changes
- [ ] `text_merge(base, ours, theirs)` returns `TextMergeResult::Conflict` for overlapping changes
- [ ] Conflict content includes standard markers: `<<<<<<<`, `=======`, `>>>>>>>`
- [ ] `TextMergeResult::Conflict` includes accurate `conflict_count` and `regions`
- [ ] `parse_conflicts()` extracts all `ConflictRegion` structs from marked content
- [ ] `has_conflict_markers()` correctly identifies conflict presence
- [ ] Empty files and trailing whitespace handled gracefully
- [ ] Performance acceptable for files up to 100KB (common config file sizes)

---

## All Needed Context

### Context Completeness Check

_"If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_ - **Yes, with the context below.**

### Documentation & References

```yaml
# MUST READ - External Documentation
- url: https://docs.rs/diffy/latest/diffy/fn.merge.html
  why: Primary merge function API - shows Ok(merged) vs Err(conflict) return pattern
  critical: "Returns Err(String) with conflict markers when conflicts exist"

- url: https://docs.rs/diffy/latest/diffy/index.html
  why: Full diffy crate documentation including create_patch and apply functions
  critical: "diffy handles line-level diffing with Git-compatible output"

- url: https://blog.jcoglan.com/2017/05/08/merging-with-diff3/
  why: Explains diff3 algorithm that diffy implements
  critical: "3-way merge finds common ancestor to determine who changed what"

- url: https://git-scm.com/docs/git-merge#_how_conflicts_are_presented
  why: Git's conflict marker format specification
  critical: "Standard markers are 7 chars: <<<<<<<, =======, >>>>>>>"

# MUST READ - Internal Patterns
- file: src/merge/text.rs
  why: Current skeleton to replace - preserve existing public API signatures
  pattern: TextMergeResult enum, text_merge() signature, ConflictRegion struct
  gotcha: "parse_conflicts() currently returns Err - must implement properly"

- file: src/merge/layer.rs
  why: Shows how text files are currently handled in merge orchestration
  pattern: FileFormat::Text case in parse_content() wraps as MergeValue::String
  gotcha: "Currently uses deep_merge for text - scalar override, not 3-way merge"

- file: src/merge/deep.rs
  why: Shows RFC 7396 merge pattern and error handling conventions
  pattern: deep_merge() returns Result<MergeValue> with JinError
  gotcha: "Text files should NOT go through deep_merge - use text_merge instead"

- file: src/core/error.rs
  why: JinError types to use for text merge errors
  pattern: JinError::MergeConflict { path: String } for conflict reporting
  gotcha: "Parse errors should use JinError::Parse { format, message }"
```

### Current Codebase Tree

```bash
src/
├── core/
│   ├── error.rs          # JinError enum with MergeConflict variant
│   ├── layer.rs          # Layer enum with 9 layers
│   └── config.rs         # Config types
├── merge/
│   ├── mod.rs            # Public API exports (already exports text_merge)
│   ├── value.rs          # MergeValue enum (1161 lines, complete)
│   ├── deep.rs           # deep_merge() algorithm (735 lines, complete)
│   ├── layer.rs          # merge_layers() orchestration (514 lines, complete)
│   └── text.rs           # <- YOUR TARGET (163 lines skeleton)
├── git/                  # Git operations (complete)
├── staging/              # Staging system (complete)
├── commit/               # Commit pipeline
└── commands/             # CLI commands
```

### Desired Implementation Structure

```bash
src/merge/text.rs         # Enhanced with proper 3-way merge
  ├── TextMergeResult     # Keep existing enum, enhance with regions
  ├── ConflictRegion      # Keep existing struct
  ├── MergeStyle          # NEW: Standard vs Diff3 marker style
  ├── TextMergeConfig     # NEW: Configuration for merge behavior
  ├── text_merge()        # REPLACE: Use diffy for real 3-way merge
  ├── text_merge_with_config() # NEW: Configurable merge
  ├── has_conflict_markers()   # KEEP: Already implemented
  ├── parse_conflicts()   # IMPLEMENT: Parse conflict regions
  └── tests module        # ENHANCE: 25+ tests
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: diffy merge() returns different types for success vs conflict
// Success: Ok(String) - the merged content
// Conflict: Err(String) - content WITH conflict markers already inserted
// You must handle BOTH cases from the Err variant

// GOTCHA: diffy uses "ours" and "theirs" labels, not layer names
// Consider making labels configurable in TextMergeConfig

// GOTCHA: Empty strings are valid inputs - handle gracefully
// Empty base + non-empty ours/theirs = take the changes
// All empty = return empty Clean result

// GOTCHA: Line endings matter
// diffy normalizes to \n internally, but preserve original endings on output
// Consider: should we normalize to \n before merge?

// GOTCHA: Trailing newlines
// "file\n" vs "file" are different - preserve trailing newline if present

// GOTCHA: Current layer.rs wraps text as MergeValue::String
// This bypasses text_merge entirely! Future integration needed.
// For P2.M4: Focus on text_merge() correctness; layer integration is separate.
```

---

## Implementation Blueprint

### Data Models and Structures

```rust
// Keep existing - already correct
pub enum TextMergeResult {
    Clean(String),
    Conflict {
        content: String,           // Merged content with conflict markers
        conflict_count: usize,     // Number of conflict regions
    },
}

// Keep existing - already correct
pub struct ConflictRegion {
    pub start_line: usize,
    pub end_line: usize,
    pub ours: String,
    pub theirs: String,
    pub base: Option<String>,      // For diff3 style
}

// NEW: Configuration for merge behavior
pub struct TextMergeConfig {
    /// Label for "ours" side in conflict markers
    pub ours_label: String,        // Default: "ours"
    /// Label for "theirs" side in conflict markers
    pub theirs_label: String,      // Default: "theirs"
    /// Include base in conflict markers (diff3 style)
    pub show_base: bool,           // Default: false
    /// Label for base in diff3 markers
    pub base_label: String,        // Default: "base"
}

impl Default for TextMergeConfig {
    fn default() -> Self {
        Self {
            ours_label: "ours".to_string(),
            theirs_label: "theirs".to_string(),
            show_base: false,
            base_label: "base".to_string(),
        }
    }
}
```

### Implementation Tasks (Ordered by Dependencies)

```yaml
Task 1: ADD diffy dependency to Cargo.toml
  - ADD: diffy = "0.4" to [dependencies] section
  - WHY: Provides production-ready 3-way merge with conflict detection
  - VERIFY: cargo check succeeds after adding
  - NOTE: diffy is MIT/Apache-2.0 licensed (compatible with MIT)

Task 2: ADD TextMergeConfig struct to src/merge/text.rs
  - IMPLEMENT: TextMergeConfig struct with Default trait
  - FOLLOW pattern: MergeConfig in src/merge/deep.rs (lines 14-26)
  - PLACEMENT: After TextMergeResult enum (around line 17)
  - FIELDS: ours_label, theirs_label, show_base, base_label

Task 3: REPLACE text_merge() implementation in src/merge/text.rs
  - REPLACE: Lines 31-61 with diffy-based implementation
  - PRESERVE: Function signature (base: &str, ours: &str, theirs: &str) -> Result<TextMergeResult>
  - ALGORITHM:
    1. Call diffy::merge(base, ours, theirs)
    2. On Ok(merged): Return TextMergeResult::Clean(merged)
    3. On Err(conflict_content): Parse conflict markers, count regions
    4. Return TextMergeResult::Conflict with content and count
  - GOTCHA: diffy::merge returns Err with conflict markers - this is NOT an error condition

Task 4: ADD text_merge_with_config() function
  - IMPLEMENT: New function taking TextMergeConfig parameter
  - PATTERN: Follow deep_merge_with_config() in src/merge/deep.rs
  - PURPOSE: Allow custom labels for conflict markers
  - INTEGRATION: text_merge() calls text_merge_with_config() with defaults
  - ENHANCEMENT: Replace diffy's default labels with config labels

Task 5: IMPLEMENT parse_conflicts() function
  - REPLACE: Lines 71-74 (current stub returning Err)
  - ALGORITHM:
    1. Find all <<<<<<< markers and their line numbers
    2. For each conflict region, extract ours/theirs/base content
    3. Calculate start_line and end_line for each region
    4. Return Vec<ConflictRegion>
  - RETURN: Result<Vec<ConflictRegion>> (empty vec if no conflicts)
  - EDGE CASE: Handle malformed markers gracefully (return error)

Task 6: ENHANCE tests in src/merge/text.rs
  - KEEP: Existing 5 tests (lines 92-162)
  - ADD: Tests for multi-region conflicts (2+ regions per file)
  - ADD: Tests for empty file handling
  - ADD: Tests for trailing newline preservation
  - ADD: Tests for parse_conflicts() function
  - ADD: Tests for TextMergeConfig with custom labels
  - ADD: Tests for large file performance (100KB)
  - ADD: Tests for non-overlapping changes (clean merge)
  - TOTAL: 25+ tests minimum
  - PATTERN: Follow existing test organization with section headers

Task 7: UPDATE module exports in src/merge/mod.rs
  - ADD: Export TextMergeConfig to public API
  - ADD: Export text_merge_with_config to public API
  - ADD: Export parse_conflicts and ConflictRegion to public API
  - LOCATION: Line 38 area, add to existing text exports
```

### Implementation Patterns & Key Details

```rust
// Pattern 1: Main merge function using diffy
use diffy::merge;

pub fn text_merge(base: &str, ours: &str, theirs: &str) -> Result<TextMergeResult> {
    text_merge_with_config(base, ours, theirs, &TextMergeConfig::default())
}

pub fn text_merge_with_config(
    base: &str,
    ours: &str,
    theirs: &str,
    config: &TextMergeConfig,
) -> Result<TextMergeResult> {
    // CRITICAL: diffy's merge() signature
    // Ok(String) = clean merge result
    // Err(String) = content with conflict markers (NOT an error!)

    match merge(base, ours, theirs) {
        Ok(merged) => Ok(TextMergeResult::Clean(merged)),
        Err(conflict_content) => {
            // diffy puts its own markers - optionally rewrite with custom labels
            let content = if config.ours_label != "ours" || config.theirs_label != "theirs" {
                rewrite_conflict_labels(&conflict_content, config)
            } else {
                conflict_content
            };

            let conflict_count = count_conflict_regions(&content);

            Ok(TextMergeResult::Conflict {
                content,
                conflict_count,
            })
        }
    }
}

// Pattern 2: Count conflict regions
fn count_conflict_regions(content: &str) -> usize {
    content.matches("<<<<<<<").count()
}

// Pattern 3: Parse conflict regions with line tracking
pub fn parse_conflicts(content: &str) -> Result<Vec<ConflictRegion>> {
    if !has_conflict_markers(content) {
        return Ok(Vec::new());
    }

    let mut regions = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        if lines[i].starts_with("<<<<<<<") {
            let start_line = i + 1; // 1-indexed for user display

            // Find ======= separator
            let mut sep_idx = i + 1;
            while sep_idx < lines.len() && !lines[sep_idx].starts_with("=======") {
                sep_idx += 1;
            }

            // Find >>>>>>> end marker
            let mut end_idx = sep_idx + 1;
            while end_idx < lines.len() && !lines[end_idx].starts_with(">>>>>>>") {
                end_idx += 1;
            }

            if sep_idx >= lines.len() || end_idx >= lines.len() {
                return Err(JinError::Parse {
                    format: "conflict".to_string(),
                    message: "Malformed conflict markers".to_string(),
                });
            }

            let ours = lines[i+1..sep_idx].join("\n");
            let theirs = lines[sep_idx+1..end_idx].join("\n");

            regions.push(ConflictRegion {
                start_line,
                end_line: end_idx + 1, // 1-indexed
                ours,
                theirs,
                base: None, // Standard format doesn't include base
            });

            i = end_idx + 1;
        } else {
            i += 1;
        }
    }

    Ok(regions)
}
```

### Integration Points

```yaml
DEPENDENCY:
  - add to: Cargo.toml [dependencies] section
  - line: "diffy = \"0.4\""
  - verify: cargo check passes

MODULE_EXPORTS:
  - modify: src/merge/mod.rs
  - add: "pub use text::{TextMergeConfig, text_merge_with_config, parse_conflicts, ConflictRegion};"
  - preserve: existing "pub use text::text_merge;" export

FUTURE_INTEGRATION (not part of P2.M4):
  - location: src/merge/layer.rs merge_file_across_layers()
  - change: For FileFormat::Text, call text_merge() instead of deep_merge()
  - note: This is P3 scope - don't implement in P2.M4
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification - fix before proceeding
cargo check                          # Compilation check
cargo clippy -- -D warnings          # Lint with warnings as errors
cargo fmt -- --check                  # Format check

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test the text merge module specifically
cargo test merge::text -- --nocapture

# Test all merge-related modules
cargo test merge:: -- --nocapture

# Full test suite
cargo test

# Expected: All tests pass. 25+ tests in text module.
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the binary
cargo build

# Run the full test suite including integration tests
cargo test --all

# Verify diffy dependency is working
cargo tree -p diffy

# Run with verbose output for debugging
RUST_BACKTRACE=1 cargo test merge::text -- --nocapture

# Expected: All tests pass, diffy appears in dependency tree
```

### Level 4: Manual Verification

```bash
# Create a simple test to verify 3-way merge works
# In a Rust playground or test file:

use diffy::merge;

fn main() {
    let base = "line1\nline2\nline3\n";
    let ours = "line1\nOUR CHANGE\nline3\n";
    let theirs = "line1\nline2\nTHEIR ADDITION\nline3\n";

    match merge(base, ours, theirs) {
        Ok(merged) => println!("Clean merge:\n{}", merged),
        Err(conflict) => println!("Conflict:\n{}", conflict),
    }
}

# Expected: Clean merge with both changes applied (non-overlapping)
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` passes with no errors
- [ ] `cargo clippy -- -D warnings` passes with no warnings
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo test` passes (all existing + new tests)
- [ ] `cargo test merge::text` shows 25+ tests passing
- [ ] `diffy = "0.4"` appears in Cargo.toml

### Feature Validation

- [ ] `text_merge(base, ours, theirs)` returns Clean for non-overlapping changes
- [ ] `text_merge(base, ours, theirs)` returns Conflict for overlapping changes
- [ ] Conflict markers match Git format: `<<<<<<<`, `=======`, `>>>>>>>`
- [ ] `conflict_count` accurately reflects number of conflict regions
- [ ] `parse_conflicts()` extracts all ConflictRegion structs correctly
- [ ] `has_conflict_markers()` returns true/false correctly
- [ ] Empty file inputs handled without panic
- [ ] Trailing newlines preserved correctly

### Code Quality Validation

- [ ] Follows existing patterns from `src/merge/deep.rs`
- [ ] Test organization matches existing style (section headers)
- [ ] Error types use `JinError::Parse` or `JinError::MergeConflict`
- [ ] Public API exported in `src/merge/mod.rs`
- [ ] No unwrap() calls - use proper Result handling
- [ ] Function documentation matches existing style

### API Compatibility

- [ ] `text_merge()` signature unchanged: `(base: &str, ours: &str, theirs: &str) -> Result<TextMergeResult>`
- [ ] `TextMergeResult` enum structure unchanged
- [ ] `ConflictRegion` struct fields unchanged
- [ ] `has_conflict_markers()` signature unchanged
- [ ] New additions are additive (no breaking changes)

---

## Anti-Patterns to Avoid

- **Don't** implement custom diff algorithm - use `diffy` crate
- **Don't** treat diffy's `Err(conflict_content)` as an error - it's expected behavior
- **Don't** modify `layer.rs` integration in this milestone (future scope)
- **Don't** add diff3 base display without config flag (optional feature)
- **Don't** skip tests for edge cases (empty files, large files, special chars)
- **Don't** use `unwrap()` - proper error handling with `?` operator
- **Don't** add unnecessary abstractions - keep implementation focused
- **Don't** change existing test structure - add new tests following pattern

---

## Research References

### Library Documentation
- [diffy crate - docs.rs](https://docs.rs/diffy/latest/diffy/) - Primary library
- [diffy merge function](https://docs.rs/diffy/latest/diffy/fn.merge.html) - Core API
- [similar crate - docs.rs](https://docs.rs/similar/latest/similar/) - Alternative (not used)

### Algorithm Background
- [Merging with diff3](https://blog.jcoglan.com/2017/05/08/merging-with-diff3/) - Algorithm explanation
- [The Magic of 3-Way Merge](https://blog.git-init.com/the-magic-of-3-way-merge/) - Conceptual overview
- [Git Merge Documentation](https://git-scm.com/docs/git-merge) - Conflict marker format

### Codebase Patterns
- `src/merge/deep.rs:14-26` - MergeConfig pattern to follow
- `src/merge/deep.rs:28-45` - Function signature pattern
- `src/merge/value.rs:91-162` - Test organization pattern
- `src/core/error.rs:1-50` - Error type definitions

---

## Confidence Score

**Implementation Confidence**: 9/10

**Reasoning**:
- `diffy` crate is mature, well-documented, and handles the hard parts
- Current skeleton provides clear API to preserve
- Existing patterns in codebase are clear to follow
- All dependencies (P2.M1-P2.M3) are complete
- Only complexity is conflict marker parsing, which is well-defined

**Risk Factors**:
- Edge cases with unusual line endings (mitigated by tests)
- Large file performance (mitigated by diffy's efficiency)
- Integration with layer.rs (explicitly out of scope for P2.M4)

---

## Summary

P2.M4 Text Merge is a well-scoped milestone that completes Jin's merge engine. By leveraging the `diffy` crate, the implementation focuses on integration and API design rather than algorithm complexity. The existing skeleton provides a clear contract to fulfill, and the extensive test suite will ensure correctness.

**Key deliverables**:
1. Replace text.rs skeleton with diffy-based implementation
2. Add TextMergeConfig for customization
3. Implement parse_conflicts() for conflict extraction
4. Add 25+ comprehensive tests
5. Export new types in module API

**Out of scope** (future milestones):
- Integration with layer.rs merge orchestration
- CLI commands for conflict resolution
- Interactive conflict resolution UI
