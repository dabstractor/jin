# Product Requirement Prompt: Filter and Highlight Active Context Updates

## Goal

**Feature Goal**: Implement smart filtering and prominent display of fetch results based on active mode/scope context, so users immediately see updates relevant to their current working context.

**Deliverable**: Enhanced `report_updates()` function in `src/commands/fetch.rs` that:
1. Filters update results into "relevant to active context" vs "other updates"
2. Displays active context updates prominently at the top
3. Shows other updates in a less prominent section below

**Success Definition**:
- Fetch command loads and uses ProjectContext (mode/scope) to categorize updates
- Updates matching active mode/scope are displayed in a highlighted "Updates for your active context" section
- Other updates are shown in a separate "Other updates" section
- Output is clear, scannable, and follows existing Jin CLI patterns
- Integration tests verify the filtering and display logic

## User Persona

**Target User**: Developer using Jin with multiple active modes and scopes

**Use Case**: After running `jin fetch`, the user wants to immediately see if there are updates to the mode/scope they're currently working on, without having to scan through all available updates.

**User Journey**:
1. User has active mode "claude" and scope "language:javascript" set in their project context
2. User runs `jin fetch` to check for remote updates
3. Fetch command detects updates to multiple modes (claude, gpt, development) and scopes
4. Fetch displays:
   - Prominent section showing updates to "claude" mode and "language:javascript" scope
   - Secondary section showing updates to other modes/scopes
5. User can immediately see what's relevant to their current work

**Pain Points Addressed**:
- Users with multiple modes/scopes must scan all updates to find relevant ones
- No visual prioritization of updates based on current context
- Difficult to quickly determine if "my stuff" has updates

## Why

- **User Impact**: Reduces cognitive load by prioritizing relevant information
- **Integration**: Builds on P2.M3.T1 (ProjectContext loading already implemented)
- **Completion**: Finishes Milestone 2.3 (Active Context Notifications in Fetch)

## What

### User-Visible Behavior

When running `jin fetch` with an active mode/scope:

```
Fetching from origin (https://github.com/example/jin-config)...
Received 12 objects (100%)

Updates for your active context (mode: claude, scope: language:javascript):
  - mode/claude (3 file(s))
  - mode/claude/scope/language:javascript (1 file(s))

Other updates:
  - mode/development (5 file(s))
  - mode/gpt (2 file(s))
  - scope/env:production (1 file(s))

Run 'jin pull' to merge updates
```

When there are no active context updates:

```
Fetching from origin (https://github.com/example/jin-config)...
Received 12 objects (100%)

No updates for your active context (mode: claude, scope: language:javascript)

Other updates:
  - mode/development (5 file(s))
  - mode/gpt (2 file(s))

Run 'jin pull' to merge updates
```

When there are no updates at all:

```
Fetching from origin (https://github.com/example/jin-config)...
Received 12 objects (100%)

Already up to date
```

### Success Criteria

- [ ] Updates matching active mode are shown in prominent "Updates for your active context" section
- [ ] Updates matching active scope (with or without mode) are shown in prominent section
- [ ] Updates not matching active context are shown in "Other updates" section
- [ ] When no active updates exist, message clearly states this
- [ ] Output formatting matches existing Jin CLI patterns (section headers, indentation)
- [ ] Works with default context (no active mode/scope)
- [ ] Integration tests cover all scenarios

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" Test**: An AI agent unfamiliar with this codebase would have everything needed because:
- Exact file paths and function signatures provided
- Ref path patterns and filtering logic specified
- Existing code patterns to follow
- Test setup patterns documented
- CLI output conventions specified

### Documentation & References

```yaml
# MUST READ - Core Implementation Files

- file: src/commands/fetch.rs
  why: Main file to modify - contains report_updates() function
  pattern: Line 76-151 shows current update reporting logic
  gotcha: Line 79 has suppress warning for context - remove this when implementing

- file: src/core/config.rs
  why: ProjectContext structure definition with mode/scope fields
  pattern: Lines 89-107 define ProjectContext struct
  gotcha: ProjectContext::load() returns Err(JinError::NotInitialized) when .jin/context doesn't exist

- file: src/core/layer.rs
  why: Layer enum and ref_path patterns for filtering logic
  pattern: Lines 50-86 show ref_path() method with all layer patterns
  critical: Ref path patterns must match exactly for filtering to work

- file: src/commands/pull.rs
  why: Contains parse_ref_path() function for extracting mode/scope from refs
  pattern: Lines 221-260 show exact parsing logic - reuse this pattern
  gotcha: Returns (Layer, Option<String>, Option<String>, Option<String>)

# EXTERNAL RESEARCH - CLI Best Practices

- url: https://clig.dev/#sections
  why: Best practices for sectioned CLI output
  critical: Use blank lines between sections, consistent indentation

- url: https://gist.github.com/JBlond/2fea43a3049b38287e5e9cefc87b2124
  why: Complete ANSI color code reference for styling
  section: Color codes for emphasis (yellow for active context updates)

- url: https://trentm.com/2024/09choosing-readable-ansi-colors-for-clis.html
  why: Color readability guidelines for terminal output
  critical: Yellow (33) is good for highlighting without being too aggressive

# TEST PATTERNS

- file: tests/sync_workflow.rs
  why: Integration test patterns for fetch command
  pattern: Lines 1002-1053 (test_fetch_loads_context) shows context testing setup
  pattern: Lines 61-131 (test_fetch_updates_refs) shows remote update testing
  gotcha: Must set JIN_DIR environment variable before running jin commands

- file: tests/common/fixtures.rs
  why: Test helper functions for creating modes/scopes
  pattern: create_mode(), create_scope() functions for setting up test context
  pattern: setup_jin_with_remote() for remote fixture setup
```

### Current Codebase Tree (Relevant Files Only)

```bash
src/
├── commands/
│   ├── fetch.rs          # MODIFY: report_updates() function
│   ├── mod.rs            # Contains command exports
│   └── pull.rs           # REFERENCE: parse_ref_path() pattern
├── core/
│   ├── config.rs         # REFERENCE: ProjectContext struct
│   ├── layer.rs          # REFERENCE: Layer enum and ref_path patterns
│   └── error.rs          # Error types
└── git/
    ├── refs.rs           # REFERENCE: RefOps trait for ref operations
    └── repo.rs           # JinRepo wrapper

tests/
├── sync_workflow.rs      # MODIFY/ADD: Tests for active context filtering
├── common/
│   ├── fixtures.rs       # Test helpers
│   └── assertions.rs     # Assertion helpers
```

### Desired Codebase Tree (Changes)

```bash
src/commands/fetch.rs     # MODIFIED: Enhanced report_updates()
                          # - NEW: is_ref_relevant_to_context() helper
                          # - NEW: format_update_section() helper
                          # - CHANGED: report_updates() logic

tests/sync_workflow.rs    # MODIFIED: Add new tests
                          # - test_fetch_highlights_active_mode_updates()
                          # - test_fetch_separates_active_and_other_updates()
                          # - test_fetch_with_default_context()
```

### Known Gotchas of our Codebase & Library Quirks

```rust
// CRITICAL: Ref path patterns must match Layer::ref_path() exactly
// Pattern: "refs/jin/layers/mode/{name}" for mode base
// Pattern: "refs/jin/layers/mode/{mode}/scope/{scope}" for mode-scope layers
// Pattern: "refs/jin/layers/scope/{name}" for untethered scopes

// CRITICAL: ProjectContext may be default (all fields None) for uninitialized projects
// Always handle: context.mode.as_deref() instead of context.mode.unwrap()

// CRITICAL: Scope relevance depends on whether mode is active
// If mode is set: scope refs must match "mode/{active_mode}/scope/{active_scope}"
// If no mode: scope refs must match "scope/{active_scope}"

// CRITICAL: Use existing parse_ref_path() from pull.rs as pattern
// Don't reimplement - the logic handles all layer types correctly

// GOTCHA: fetch.rs line 79 has let _context = context; to suppress unused warning
// Remove this when implementing the filtering logic

// PATTERN: Jin CLI uses println!() for main output, eprintln!() for warnings
// Section headers use println!() with clear text

// PATTERN: Test isolation requires JIN_DIR environment variable
// Always call fixture.set_jin_dir() before running jin commands in tests
```

## Implementation Blueprint

### Data Models and Structure

No new data models needed. We'll use existing structures:

```rust
// From src/core/config.rs - Already exists
pub struct ProjectContext {
    pub mode: Option<String>,
    pub scope: Option<String>,
    pub project: Option<String>,
    pub last_updated: Option<String>,
}

// From src/commands/fetch.rs - Already exists, will use
struct UpdateInfo {
    category: String,
    refs: Vec<String>,
}

// From src/commands/fetch.rs - Enhance this function
fn report_updates(jin_repo: &JinRepo, context: &ProjectContext) -> Result<()>
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD is_ref_relevant_to_context() helper function
  - CREATE: Helper function in fetch.rs to check if a ref matches active context
  - ACCEPTS: &str (ref_path), &ProjectContext
  - RETURNS: bool
  - LOGIC:
    * Parse ref_path to extract mode and scope components
    * Match against context.mode and context.scope
    * Handle scope-with-mode vs untethered scope patterns
  - FOLLOW pattern: parse_ref_path() from pull.rs (lines 221-260)
  - NAMING: is_ref_relevant_to_context()
  - PLACEMENT: After UpdateInfo struct (around line 160)

Task 2: MODIFY report_updates() to categorize updates
  - CHANGE: Update HashMap to track "active" vs "other" updates
  - CREATE: Two separate HashMap<String, UpdateInfo> for active and other
  - LOGIC:
    * For each update, call is_ref_relevant_to_context()
    * If true, add to active_updates HashMap
    * If false, add to other_updates HashMap
  - PRESERVE: Existing update detection and categorization logic
  - PLACEMENT: In report_updates() function (lines 76-151)

Task 3: ADD format_update_section() helper function
  - CREATE: Helper to format a section of updates with header
  - ACCEPTS: &str (section title), &HashMap<String, UpdateInfo>
  - RETURNS: String (formatted output)
  - LOGIC:
    * Print section header
    * If empty, print "No updates for..." message
    * If has items, iterate and print each update
  - FOLLOW pattern: Existing output formatting in report_updates()
  - PLACEMENT: After is_ref_relevant_to_context()

Task 4: MODIFY report_updates() output logic
  - REPLACE: Single output section with two sections
  - ACTIVE SECTION: Call format_update_section() with "Updates for your active context..."
  - OTHER SECTION: Call format_update_section() with "Other updates"
  - HEADER: Include active mode/scope in header for clarity
  - PRESERVE: "Run 'jin pull' to merge updates" message
  - PLACEMENT: Lines 134-148 in report_updates()

Task 5: REMOVE unused context suppression
  - REMOVE: Line 79 `let _context = context;`
  - WHY: Context is now actively used for filtering
  - PLACEMENT: report_updates() function

Task 6: CREATE test_fetch_highlights_active_mode_updates()
  - IMPLEMENT: Integration test for active mode highlighting
  - SETUP:
    * Create remote with multiple mode updates
    * Set active mode in ProjectContext
    * Run fetch
  - ASSERT: Output contains "Updates for your active context"
  - ASSERT: Active mode updates shown in prominent section
  - FOLLOW pattern: test_fetch_loads_context() (lines 1002-1053)
  - NAMING: test_fetch_highlights_active_mode_updates
  - PLACEMENT: tests/sync_workflow.rs

Task 7: CREATE test_fetch_separates_active_and_other_updates()
  - IMPLEMENT: Integration test for section separation
  - SETUP:
    * Create remote with updates for active and inactive modes
    * Set active mode
    * Run fetch
  - ASSERT: Active updates in first section
  - ASSERT: Other updates in "Other updates" section
  - ASSERT: Clear visual separation between sections
  - FOLLOW pattern: test_fetch_updates_refs() (lines 61-131)
  - NAMING: test_fetch_separates_active_and_other_updates
  - PLACEMENT: tests/sync_workflow.rs

Task 8: CREATE test_fetch_with_default_context()
  - IMPLEMENT: Integration test for no active mode/scope
  - SETUP:
    * Create remote with updates
    * Remove or don't set ProjectContext (default)
    * Run fetch
  - ASSERT: All updates shown in "Other updates" section
  - ASSERT: No active context section shown
  - FOLLOW pattern: test_fetch_loads_context() second test case
  - NAMING: test_fetch_with_default_context
  - PLACEMENT: tests/sync_workflow.rs

Task 9: UPDATE categorize_layer() if needed
  - REVIEW: Current categorization logic (lines 162-173)
  - VERIFY: Categories match ref filtering patterns
  - MODIFY: Only if categorization doesn't align with filtering logic
  - GOTCHA: May not need changes - verify during implementation
  - PLACEMENT: src/commands/fetch.rs
```

### Implementation Patterns & Key Details

```rust
// PATTERN 1: Ref relevance checking (Task 1)
// Reuses parse_ref_path logic from pull.rs

/// Check if a ref path is relevant to the active context
///
/// A ref is relevant if:
/// - It matches the active mode (e.g., "mode/claude" when mode is "claude")
/// - It matches the active scope with mode (e.g., "mode/claude/scope/js" when mode="claude", scope="js")
/// - It matches the active scope without mode (e.g., "scope/js" when mode=None, scope="js")
/// - Global refs are always relevant
fn is_ref_relevant_to_context(ref_path: &str, context: &ProjectContext) -> bool {
    // Strip prefix to get layer path
    let layer_path = match ref_path.strip_prefix("refs/jin/layers/") {
        Some(path) => path,
        None => return false,
    };

    // Global is always relevant
    if layer_path == "global" {
        return true;
    }

    // Parse the path components
    let parts: Vec<&str> = layer_path.split('/').collect();

    match parts.as_slice() {
        // Mode refs: Check if matches active mode
        ["mode", mode, ..] => {
            context.mode.as_deref() == Some(*mode)
        }

        // Untethered scope refs: Only relevant if no active mode
        ["scope", scope, ..] => {
            context.mode.is_none() && context.scope.as_deref() == Some(*scope)
        }

        // Other patterns: Not relevant to context
        _ => false,
    }
}

// PATTERN 2: Update section formatting (Task 3)
fn format_update_section(
    title: &str,
    updates: &HashMap<String, UpdateInfo>,
    jin_repo: &JinRepo,
) -> Result<()> {
    if updates.is_empty() {
        // No updates in this section
        if title.contains("active context") {
            // Only show "no updates" message for active context
            println!("{}", title);
        }
        return Ok(());
    }

    // Print section header
    println!();
    println!("{}", title);

    // Sort and display updates
    let mut categories: Vec<_> = updates.keys().collect();
    categories.sort();

    for category in categories {
        let info = &updates[category];
        println!("  - {} ({} file(s))", info.refs[0], info.refs.len());
    }

    Ok(())
}

// PATTERN 3: Modified report_updates structure (Task 2 & 4)
fn report_updates(jin_repo: &JinRepo, context: &ProjectContext) -> Result<()> {
    // ... existing ref collection logic ...

    // Split into active and other updates
    let mut active_updates: HashMap<String, UpdateInfo> = HashMap::new();
    let mut other_updates: HashMap<String, UpdateInfo> = HashMap::new();

    for (category, info) in updates {
        // Check if any ref in this category is relevant
        let is_relevant = info.refs.iter()
            .any(|ref_path| is_ref_relevant_to_context(ref_path, context));

        if is_relevant {
            active_updates.insert(category, info);
        } else {
            other_updates.insert(category, info);
        }
    }

    // Build section title with context info
    let active_title = if let (Some(mode), Some(scope)) = (&context.mode, &context.scope) {
        format!("Updates for your active context (mode: {}, scope: {}):", mode, scope)
    } else if let Some(mode) = &context.mode {
        format!("Updates for your active context (mode: {}):", mode)
    } else if let Some(scope) = &context.scope {
        format!("Updates for your active context (scope: {}):", scope)
    } else {
        "Updates for your active context:".to_string()
    };

    // Display active updates
    format_update_section(&active_title, &active_updates, jin_repo)?;

    // Display other updates
    if !other_updates.is_empty() {
        format_update_section("Other updates:", &other_updates, jin_repo)?;
    }

    // Show next steps if any updates exist
    if !active_updates.is_empty() || !other_updates.is_empty() {
        println!("\nRun 'jin pull' to merge updates");
    }

    Ok(())
}

// GOTCHA: Scope filtering is complex
// - If active mode is "claude" and active scope is "js"
//   - Relevant: "mode/claude/scope/js/*"
//   - NOT relevant: "mode/gpt/scope/js/*" (wrong mode)
//   - NOT relevant: "scope/js/*" (untethered, no mode)
//
// - If no active mode but active scope is "js"
//   - Relevant: "scope/js/*" (untethered scope)
//   - NOT relevant: "mode/*/scope/js/*" (any mode-scope)
```

### Integration Points

```yaml
MODIFY: src/commands/fetch.rs
  - function: report_updates()
  - changes: Add filtering logic, split output into sections
  - preserve: All existing ref detection and update logic
  - lines: 76-151 (current), will expand

MODIFY: tests/sync_workflow.rs
  - add: 3 new integration tests
  - pattern: Follow existing test_fetch_loads_context structure
  - setup: Use RemoteFixture, create modes/scopes, set context
  - assertions: Use predicates::prelude::* for output checking

NO CHANGES NEEDED:
  - src/core/config.rs (ProjectContext already loaded)
  - src/core/layer.rs (Layer patterns already defined)
  - src/commands/pull.rs (only referenced for pattern)
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification
cargo check --bin jin                # Fast compilation check
cargo clippy --bin jin -- -D warnings # Linting with warnings as errors

# Auto-fix formatting
cargo fmt --                         # Format all Rust code

# Expected: Zero errors, zero warnings. Fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run fetch-specific unit tests (in fetch.rs)
cargo test --bin jin fetch::tests --verbose

# Run all command tests
cargo test --bin jin commands::tests --verbose

# Run unit tests only (not integration)
cargo test --bin jin --lib

# Expected: All unit tests pass. categorize_layer tests should still pass.
```

### Level 3: Integration Testing (System Validation)

```bash
# Test the specific feature
cargo test --test sync_workflow test_fetch_highlights_active_mode_updates --verbose --exact
cargo test --test sync_workflow test_fetch_separates_active_and_other_updates --verbose --exact
cargo test --test sync_workflow test_fetch_with_default_context --verbose --exact

# Run all sync workflow tests to ensure no regressions
cargo test --test sync_workflow --verbose

# Expected: All new tests pass, all existing tests still pass
```

### Level 4: Manual Testing (Real-World Validation)

```bash
# Setup test environment with active context
cd /tmp/test_jin_active_context
mkdir -p .jin
echo "version: 1
mode: claude
scope: language:javascript" > .jin/context

# Initialize Jin and link to remote
jin init
jin link /path/to/remote

# Create some mode/scope layers in remote
# (Use existing test patterns from sync_workflow.rs)

# Run fetch and verify output
jin fetch

# Expected output should show:
# 1. "Updates for your active context (mode: claude, scope: language:javascript):"
# 2. Active updates listed under this header
# 3. "Other updates:" section with non-relevant updates
# 4. Clear visual separation between sections

# Test with no active context
rm .jin/context
jin fetch

# Expected: All updates shown in "Other updates:" section only
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] `cargo test --bin jin --lib` passes (unit tests)
- [ ] `cargo test --test sync_workflow` passes (integration tests)
- [ ] `cargo clippy --bin jin -- -D warnings` produces no warnings
- [ ] `cargo fmt --check` reports no formatting issues

### Feature Validation

- [ ] Active mode updates are shown in prominent section
- [ ] Active scope updates (with mode) are shown in prominent section
- [ ] Active scope updates (without mode) are shown in prominent section
- [ ] Other updates are shown in separate "Other updates" section
- [ ] Section headers clearly indicate active mode/scope
- [ ] Output matches existing Jin CLI formatting patterns
- [ ] Works correctly with default context (no active mode/scope)
- [ ] All success criteria from "What" section met

### Code Quality Validation

- [ ] Follows existing codebase patterns (parse_ref_path, categorize_layer)
- [ ] Reuses existing ProjectContext loading (no new loading logic)
- [ ] Error handling matches existing patterns (using JinError types)
- [ ] Function placement matches file structure (helpers before main logic)
- [ ] Test setup follows RemoteFixture pattern from sync_workflow.rs

### Edge Cases Handled

- [ ] No updates at all (shows "Already up to date")
- [ ] Updates exist but none are relevant (shows "No updates for your active context")
- [ ] All updates are relevant (no "Other updates" section)
- [ ] Only mode active (no scope)
- [ ] Only scope active (no mode)
- [ ] Neither mode nor scope active (default context)
- [ ] Multiple refs in same category (correctly shows file count)

---

## Anti-Patterns to Avoid

- ❌ Don't create new ref parsing logic - reuse parse_ref_path() pattern from pull.rs
- ❌ Don't hardcode ref path patterns - match them exactly from Layer::ref_path()
- ❌ Don't use unwrap() on context.mode/scope - use as_deref() for Option handling
- ❌ Don't skip testing with default context - must work when ProjectContext is uninitialized
- ❌ Don't create new test helpers - use existing RemoteFixture and fixtures.rs patterns
- ❌ Don't add color codes to output - use plain text like other Jin commands
- ❌ Don't suppress the context variable - it's now actively used
- ❌ Don't over-complicate scope filtering - scope relevance depends on mode presence
