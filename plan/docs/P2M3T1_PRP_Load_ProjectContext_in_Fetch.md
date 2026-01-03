# Product Requirement Prompt: Load ProjectContext in Fetch Command

**Task ID**: P2.M3.T1
**Milestone**: P2.M3 - Active Context Notifications in Fetch
**Status**: Ready for Implementation
**Confidence Score**: 10/10

---

## Goal

**Feature Goal**: Load the active ProjectContext at the start of the fetch command to provide active mode/scope information for subsequent filtering of updates.

**Deliverable**: Modified `fetch::execute()` function that loads ProjectContext after loading JinConfig, with context available for use in P2.M3.T2 (Filter and Highlight Active Context Updates).

**Success Definition**:
- Fetch command loads ProjectContext at startup without errors
- Context loading gracefully handles cases where Jin is not initialized
- Context is available in memory for filtering operations in subsequent tasks
- All existing fetch functionality remains intact

---

## User Persona

**Target User**: Developer using Jin for configuration management across multiple modes/scopes

**Use Case**: User runs `jin fetch` to check for remote configuration updates and wants to see which updates are relevant to their currently active mode/scope

**User Journey**:
1. User has active mode set (e.g., `mode: claude`) and/or active scope (e.g., `scope: language:rust`)
2. User runs `jin fetch` to download remote layer refs
3. Fetch command loads user's active context from `.jin/context`
4. In P2.M3.T2, updates are filtered to highlight those affecting active context

**Pain Points Addressed**:
- Currently fetch shows all updates equally, making it hard to spot relevant ones
- No visibility into which updates affect the user's current working context
- Manual filtering of update list is error-prone

---

## Why

- **User Experience**: Reduces cognitive load by calling attention to relevant updates
- **Integration**: Provides foundation for P2.M3.T2 to implement context-aware update filtering
- **Compliance**: Addresses Gap #6 identified in `plan/architecture/codebase_analysis.md`: "Fetch command doesn't load context"
- **Foundation**: This is Task 1 of Milestone 2.3; subsequent tasks depend on context being available

---

## What

Modify `src/commands/fetch.rs` to load ProjectContext at the start of the execute() function. The context should be loaded after JinConfig but before the fetch operation, with graceful handling for uninitialized projects.

### Success Criteria

- [ ] ProjectContext loaded successfully in fetch::execute()
- [ ] Context loading handles NotInitialized error gracefully (use None/default)
- [ ] Context is passed to report_updates() for use in P2.M3.T2
- [ ] All existing fetch tests pass
- [ ] New test verifies context loading behavior

---

## All Needed Context

### Context Completeness Check

This PRP passes the "No Prior Knowledge" test. An implementer unfamiliar with this codebase has everything needed:
- Exact file paths and line numbers for modifications
- Complete code patterns to follow from existing commands
- ProjectContext structure and loading patterns documented
- Test patterns specified with examples

### Documentation & References

```yaml
# MUST READ - Critical implementation references
- file: src/commands/fetch.rs
  why: The primary file to modify; contains execute() function structure
  pattern: Load configuration pattern at lines 17-21
  gotcha: Context must be loaded AFTER JinConfig, before Git operations

- file: src/core/config.rs
  why: Contains ProjectContext struct definition and load() method
  pattern: ProjectContext::load() pattern at lines 111-120
  section: Lines 89-107 (struct definition), 111-120 (load method)
  gotcha: load() returns Err(JinError::NotInitialized) if .jin/context missing

- file: src/commands/status.rs
  why: Example of loading ProjectContext in a command
  pattern: Lines 67-72 show initialization check and context load
  gotcha: Always check ProjectContext::is_initialized() first

- file: src/commands/mode.rs
  why: Example of graceful fallback to default context
  pattern: Lines 103-109 show match pattern with NotInitialized handling
  gotcha: Some commands use default context instead of failing

- file: tests/sync_workflow.rs
  why: Contains existing fetch integration tests
  pattern: test_fetch_updates_refs() shows test structure
  gotcha: Tests use JIN_DIR environment variable for isolation

- docfile: plan/PRD.md
  why: Overall milestone context for P2.M3
  section: Lines 785-883 describe P2.M3 (Active Context Notifications in Fetch)
  gotcha: This is Task 1 of 3; context loading is prerequisite for T2 filtering

- docfile: plan/architecture/codebase_analysis.md
  why: Documents the specific gap this task addresses
  section: Gap #6: "Fetch command doesn't load context"
  gotcha: Gap analysis notes fetch.rs doesn't currently load ProjectContext
```

### Current Codebase Tree

```bash
src/
├── commands/
│   ├── fetch.rs          # PRIMARY FILE TO MODIFY (184 lines)
│   ├── status.rs         # Example of context loading pattern
│   ├── mode.rs           # Example of graceful context handling
│   └── mod.rs            # Command registration
├── core/
│   ├── config.rs         # ProjectContext definition and load()
│   ├── error.rs          # JinError types including NotInitialized
│   └── mod.rs
└── git/
    └── remote.rs         # build_fetch_options() used by fetch

tests/
├── sync_workflow.rs      # Fetch integration tests
├── cli_basic.rs          # Basic fetch smoke test
└── common/
    ├── fixtures.rs       # TestFixture, setup_jin_with_remote()
    └── assertions.rs     # Custom assertion helpers
```

### Desired Codebase Tree with Files to be Added/Modified

```bash
# MODIFIED FILE
src/commands/fetch.rs     # Add ProjectContext loading to execute()
                          # Modify report_updates() signature to accept context
                          # Store context for filtering in P2.M3.T2

# NEW TEST FILE (optional, can add to existing test files)
tests/sync_workflow.rs    # Add test_fetch_loads_context()
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: ProjectContext::load() returns Err(JinError::NotInitialized)
// if .jin/context file doesn't exist. Fetch command MUST handle this gracefully.

// CORRECT: Check initialization first, or use match with fallback
if !ProjectContext::is_initialized() {
    // Use None/empty context - fetch should still work
}

// AVOID: Direct load() without initialization check causes premature failure
let context = ProjectContext::load()?;  // Will fail if not initialized

// CRITICAL: Fetch must work even without active context
// Users can fetch before initializing their project

// CRITICAL: Context file location is .jin/context (YAML format)
// DO NOT hardcode path - use ProjectContext::default_path() or load()

// CRITICAL: Context mode/scope are Option<String> - handle None values
// When no active mode/scope, filtering in P2.M3.T2 will show all updates
```

---

## Implementation Blueprint

### Data Models and Structure

**No new data models required** - ProjectContext already exists in `src/core/config.rs`:

```rust
/// Per-project context (stored at .jin/context)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectContext {
    /// Version of the context schema
    #[serde(default = "default_version")]
    pub version: u32,

    /// Currently active mode
    pub mode: Option<String>,

    /// Currently active scope
    pub scope: Option<String>,

    /// Project name (auto-inferred from Git remote)
    pub project: Option<String>,

    /// Last update timestamp
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY src/commands/fetch.rs - Add ProjectContext import
  - ADD: use crate::core::ProjectContext; to imports section
  - LOCATION: Line 6 (after existing use statements)
  - FOLLOW: Existing import pattern for JinConfig, JinError, Result
  - DEPENDENCIES: None

Task 2: MODIFY src/commands/fetch.rs - Load context in execute()
  - ADD: Context loading after JinConfig load (line 18-21)
  - IMPLEMENT: Match pattern with graceful NotInitialized handling
  - PATTERN: Follow mode.rs lines 103-109 (graceful fallback)
  - LOGIC:
      ```rust
      let context = match ProjectContext::load() {
          Ok(ctx) => ctx,
          Err(JinError::NotInitialized) => ProjectContext::default(),
          Err(e) => return Err(e),
      };
      ```
  - PLACEMENT: After JinConfig load, before JinRepo open (line 23-25)
  - DEPENDENCIES: Task 1

Task 3: MODIFY src/commands/fetch.rs - Pass context to report_updates
  - MODIFY: report_updates() function signature to accept context parameter
  - CURRENT: fn report_updates(jin_repo: &JinRepo) -> Result<()>
  - NEW: fn report_updates(jin_repo: &JinRepo, context: &ProjectContext) -> Result<()>
  - UPDATE: Call site at line 64 to pass &context
  - DEPENDENCIES: Task 2

Task 4: MODIFY src/commands/fetch.rs - Store context for P2.M3.T2
  - ADD: Store context in report_updates() for use in filtering (P2.M3.T2)
  - NOTE: No filtering logic needed yet - just make context available
  - FUTURE: P2.M3.T2 will use context.mode and context.scope for filtering
  - DEPENDENCIES: Task 3

Task 5: CREATE/ADD test for context loading
  - IMPLEMENT: Integration test verifying context is loaded
  - FILE: tests/sync_workflow.rs (add new test function)
  - PATTERN: Follow test_fetch_updates_refs() structure (lines 15-100)
  - NAME: test_fetch_loads_context()
  - VERIFY: Context loads successfully even when not initialized
  - COVERAGE: Success path (context exists), NotInitialized path (no context)
  - DEPENDENCIES: Task 4
```

### Implementation Patterns & Key Details

```rust
// PATTERN 1: Context loading with graceful fallback
// Location: fetch.rs execute() function, after JinConfig load
let context = match ProjectContext::load() {
    Ok(ctx) => ctx,
    Err(JinError::NotInitialized) => ProjectContext::default(),
    Err(e) => return Err(e),
};

// GOTCHA: Do NOT use let context = ProjectContext::load()?
// This would fail if .jin/context doesn't exist

// PATTERN 2: Pass context to report_updates
// Location: fetch.rs line 64, modify call site
report_updates(&jin_repo, &context)?;

// PATTERN 3: Store context in report_updates for P2.M3.T2
// Location: fetch.rs report_updates() function signature
fn report_updates(jin_repo: &JinRepo, context: &ProjectContext) -> Result<()> {
    // In P2.M3.T2, will use context.mode and context.scope to filter updates
    // For now, just accept the parameter - no filtering logic yet
    let _context = context; // Suppress unused warning temporarily

    // ... existing update reporting logic ...
}

// CRITICAL: Context fields are Option<String> - always handle None
// When context.mode is None, P2.M3.T2 will show all mode updates
// When context.scope is None, P2.M3.T2 will show all scope updates
```

### Integration Points

```yaml
NO CONFIG CHANGES: No configuration files modified

NO ROUTE CHANGES: No new CLI routes or arguments

NO DATABASE CHANGES: No database or migration

MODIFIED FUNCTION SIGNATURES:
  - src/commands/fetch.rs::report_updates() gains &ProjectContext parameter
  - Call site at execute() line 64 updated to pass &context

FUTURE INTEGRATIONS (P2.M3.T2):
  - report_updates() will filter updates based on context.mode/context.scope
  - Display will separate active context updates from other updates
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after modifying fetch.rs - fix before proceeding
cargo check --bin jin 2>&1 | head -50

# Check for compilation errors specifically in fetch module
cargo check --bin jin 2>&1 | grep -A5 "fetch\|error"

# Format check
cargo fmt --check src/commands/fetch.rs

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
# Common errors to watch for:
# - "use of undeclared type: ProjectContext" -> missing import
# - "mismatched types: expected JinRepo, found ProjectContext" -> wrong parameter order
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run fetch command unit tests (the categorize_layer tests)
cargo test --lib fetch::tests::test_categorize_layer

# Run all command unit tests to ensure no regressions
cargo test --lib commands::

# Expected: All tests pass. Watch for:
# - "unused variable: context" warnings (use let _context = context to suppress)
# - Type mismatches in function signatures
```

### Level 3: Integration Testing (System Validation)

```bash
# Test 1: Fetch with initialized context
cd /tmp/test_fetch_context
rm -rf .git .jin 2>/dev/null
git init
jin init
jin mode create test-mode
jin mode use test-mode
jin link <remote_url> 2>&1 | head -1
jin fetch 2>&1 | head -20

# Expected: Fetch succeeds, context is loaded internally

# Test 2: Fetch without initialized context
cd /tmp/test_fetch_no_context
rm -rf .git .jin 2>/dev/null
git init
jin init
jin link <remote_url> 2>&1 | head -1
jin fetch 2>&1 | head -20

# Expected: Fetch succeeds, uses default context (no crash)

# Test 3: Run existing fetch integration tests
cargo test --test sync_workflow test_fetch_updates_refs

# Expected: Existing tests still pass

# Test 4: Run basic fetch smoke test
cargo test --test cli_basic test_fetch_subcommand

# Expected: Fetch fails without remote (existing behavior preserved)
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Manual verification: Add debug logging to confirm context loads
# Temporarily add to fetch.rs execute():
#   println!("DEBUG: Loaded context - mode: {:?}, scope: {:?}", context.mode, context.scope);

# Then run:
jin fetch | grep DEBUG

# Expected output examples:
# - With active mode: "DEBUG: Loaded context - mode: Some(\"claude\"), scope: None"
# - Without context: "DEBUG: Loaded context - mode: None, scope: None"

# Verify context fields are accessible for P2.M3.T2
# In report_updates(), temporarily add:
#   if let Some(mode) = &context.mode {
#       println!("DEBUG: Active mode in report_updates: {}", mode);
#   }

# Run: jin fetch (with active mode set)
# Expected: Debug output shows mode name

# IMPORTANT: Remove debug logging before committing
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] `cargo check --bin jin` passes with no errors
- [ ] `cargo test --lib commands::` passes all tests
- [ ] `cargo test --test sync_workflow` passes fetch tests
- [ ] `cargo fmt --check src/commands/fetch.rs` shows no formatting issues
- [ ] No unused variable warnings (or properly suppressed with `let _`)

### Feature Validation

- [ ] Fetch loads ProjectContext when `.jin/context` exists
- [ ] Fetch handles NotInitialized gracefully (uses default context)
- [ ] Fetch still works when project is not initialized
- [ ] report_updates() receives context parameter correctly
- [ ] Existing fetch functionality unchanged (backward compatible)

### Code Quality Validation

- [ ] Follows Pattern A from status.rs (initialization check) OR Pattern B from mode.rs (graceful fallback)
- [ ] Import statement added: `use crate::core::ProjectContext;`
- [ ] Context loading placed after JinConfig load, before JinRepo open
- [ ] Function signature updated: `report_updates(jin_repo, context)`
- [ ] No hardcoding of `.jin/context` path (uses ProjectContext::load())

### Documentation & Deployment

- [ ] Code is self-documenting with clear variable names
- [ ] No new environment variables or configuration needed
- [ ] No user-visible behavior changes in this task (context loading is internal)
- [ ] Ready for P2.M3.T2 to use loaded context for filtering

---

## Anti-Patterns to Avoid

- ❌ Don't fail fetch if context doesn't exist - use graceful fallback
- ❌ Don't hardcode `.jin/context` path - use ProjectContext::load()
- ❌ Don't add filtering logic in this task - that's P2.M3.T2
- ❌ Don't change user-visible output yet - this is internal preparation
- ❌ Don't skip passing context to report_updates() - P2.M3.T2 needs it
- ❌ Don't use `let context = ProjectContext::load()?` without NotInitialized handling
- ❌ Don't add new dependencies or imports beyond ProjectContext
- ❌ Don't modify JinConfig or other core types

---

## Task Dependencies

**Predecessor Tasks**: None (this is the first task in P2.M3)

**Successor Tasks**:
- P2.M3.T2.S1 (Compare updated refs with active context) - depends on context being loaded
- P2.M3.T2.S2 (Display prominent notification for active updates) - uses context.mode/scope
- P2.M3.T2.S3 (Display other updates section) - uses context for filtering

**Related Work**:
- P1.M2.T1 (Add Automatic Fetch to Push) - fetch command is called by push
- P2.M2.T1 (Query JinMap for Layer Contents) - different layer lookup mechanism

---

## Research Summary

This PRP is based on comprehensive research including:

1. **Codebase Analysis**: Full review of fetch.rs (184 lines), status.rs context pattern, mode.rs graceful fallback
2. **Pattern Research**: Analyzed 18+ commands loading ProjectContext to identify standard patterns
3. **Test Research**: Reviewed sync_workflow.rs for fetch test patterns using TestFixture
4. **Architecture Review**: Consulted tasks.json for exact task scope (P2.M3.T1.S1)

**Key Findings**:
- Fetch command currently does NOT load ProjectContext (confirmed gap)
- Two patterns exist: Pattern A (fail fast if not initialized) and Pattern B (graceful fallback)
- For fetch, Pattern B is correct - fetch should work even without initialized project
- ProjectContext structure already defined in src/core/config.rs with clear semantics

**Confidence Score: 10/10** - All necessary context provided, patterns documented, gotchas identified.
