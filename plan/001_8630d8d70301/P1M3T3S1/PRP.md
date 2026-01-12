# PRP: P1.M3.T3.S1 - Create Mode/Scope Switching Workflow Tests

---

## Goal

**Feature Goal**: Create integration tests that verify mode and scope switching workflows automatically clear workspace metadata, enabling seamless `jin apply` operations without manual intervention or "detached workspace" errors.

**Deliverable**: Two new integration test functions added to `tests/mode_scope_workflow.rs`:
1. `test_mode_switch_clears_metadata()` - Verifies mode switching workflow
2. `test_scope_switch_clears_metadata()` - Verifies scope switching workflow

**Success Definition**:
- Tests create two modes/scopes, activate one, apply configuration, then switch
- After switching, `jin apply` works without `--force` (no detached state error)
- Tests verify metadata was cleared by checking the clear message appears
- Tests verify the new mode/scope configuration applies correctly
- All tests pass with `cargo test mode_scope_workflow`

---

## User Persona

**Target User**: Jin developers and maintainers who need confidence that the metadata auto-clear feature (P1.M3.T1.S2 and P1.M3.T2.S1) works correctly in real-world usage scenarios.

**Use Case**: Prevent regression where mode/scope switching would require manual metadata clearing. Tests serve as documentation of expected behavior and catch future changes that break this UX.

**User Journey**:
1. Developer changes code related to mode/scope handling or metadata
2. Developer runs `cargo test mode_scope_workflow`
3. Tests verify the complete workflow still works end-to-end
4. If tests fail, developer knows exactly what broke

**Pain Points Addressed**:
- **Before**: No automated verification that mode/scope switching clears metadata
- **After**: Tests automatically detect if metadata clearing breaks
- **Maintainability**: Tests document the expected behavior clearly

---

## Why

- **Problem**: P1.M3.T1.S2 and P1.M3.T2.S1 added metadata auto-clearing on mode/scope switches, but there are no integration tests to verify this works end-to-end. Without tests, future changes could silently break this feature.

- **Solution**: Add two integration tests that exercise the complete workflow: create mode/scope → apply → switch → apply again. This verifies the metadata clearing prevents "detached workspace" errors.

- **User Experience**: Users can switch modes/scopes seamlessly without thinking about metadata state. Tests ensure this stays true.

- **Integration**: These tests complete the P1.M3 milestone by validating that both mode and scope metadata clearing (from P1.M3.T1.S2 and P1.M3.T2.S1) work correctly in real workflows.

---

## What

### User-Visible Behavior

Tests verify the following workflows work without manual intervention:

**Mode Switching Workflow:**
```bash
# User creates two modes
$ jin mode create development
$ jin mode create production

# User activates development mode and applies configuration
$ jin mode use development
$ echo '{"env": "dev"}' > config.json
$ jin add --mode config.json
$ jin commit -m "Dev config"
$ jin apply
# Workspace now has metadata referencing "development" mode

# User switches to production mode
$ jin mode use production
Cleared workspace metadata (mode changed from 'development' to 'production').
Run 'jin apply' to apply new mode configuration.
# Metadata automatically cleared

# User adds production config and applies - NO ERROR
$ echo '{"env": "prod"}' > config.json
$ jin add --mode config.json
$ jin commit -m "Prod config"
$ jin apply
# Works! No "detached workspace" error
```

**Scope Switching Workflow:**
```bash
# User creates two scopes
$ jin scope create backend
$ jin scope create frontend

# User activates backend scope and applies configuration
$ jin scope use backend
$ echo '{"port": 3000}' > config.json
$ jin add --scope=backend config.json
$ jin commit -m "Backend config"
$ jin apply
# Workspace now has metadata referencing "backend" scope

# User switches to frontend scope
$ jin scope use frontend
Cleared workspace metadata (scope changed from 'backend' to 'frontend').
Run 'jin apply' to apply new scope configuration.
# Metadata automatically cleared

# User adds frontend config and applies - NO ERROR
$ echo '{"port": 8080}' > config.json
$ jin add --scope=frontend config.json
$ jin commit -m "Frontend config"
$ jin apply
# Works! No "detached workspace" error
```

### Technical Requirements

1. **Add to `tests/mode_scope_workflow.rs`**: Two new test functions
2. **Test structure**:
   - Use `TestFixture` for isolated environment
   - Use `#[serial]` attribute (tests modify JIN_DIR)
   - Create two modes/scopes with unique names (use `unique_test_id()`)
   - Activate first mode/scope, add file, commit, apply
   - Switch to second mode/scope, verify clear message appears
   - Add file for second mode/scope, commit, apply successfully
   - Verify second mode/scope content is in workspace

3. **Test patterns to follow**: Existing tests in `tests/mode_scope_workflow.rs`
4. **Use helpers**: `create_mode()`, `create_scope()`, `unique_test_id()` from `common::fixtures`

### Success Criteria

- [ ] Test `test_mode_switch_clears_metadata()` passes
- [ ] Test `test_scope_switch_clears_metadata()` passes
- [ ] Tests verify metadata clear message appears on switch
- [ ] Tests verify `jin apply` works without `--force` after switch
- [ ] Tests use `unique_test_id()` for parallel test safety
- [ ] All existing tests still pass (`cargo test`)

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context including the exact test file to modify, the test patterns to follow, the fixture helpers available, the metadata clearing message format, the apply command behavior, and comprehensive test structure examples._

### Documentation & References

```yaml
# MUST READ - Core Test Implementation Context

# Test file to modify (exact location)
- file: /home/dustin/projects/jin/tests/mode_scope_workflow.rs
  why: This is where all mode/scope workflow integration tests live
  section: "Complete file (lines 1-639)"
  pattern: |
    #[test]
    #[serial]
    fn test_<name>() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = TestFixture::new()?;
        let project_path = fixture.path();
        let jin_dir = fixture.jin_dir.as_ref().unwrap();

        // CRITICAL: Set JIN_DIR BEFORE any Jin operations
        fixture.set_jin_dir();

        jin_init(project_path, None)?;

        // ... test logic ...

        Ok(())
    }

# Test fixture helpers available
- file: /home/dustin/projects/jin/tests/common/fixtures.rs
  why: Provides TestFixture, create_mode, create_scope, unique_test_id helpers
  section: "TestFixture (lines 16-51), create_mode (lines 242-280), create_scope (lines 283-321), unique_test_id (lines 323-339)"
  pattern: |
    let fixture = TestFixture::new()?;
    fixture.set_jin_dir();  // CRITICAL: Call before any Jin operations

    let mode_a = format!("mode_a_{}", unique_test_id());
    create_mode(&mode_a, Some(jin_dir))?;

    let scope_x = format!("scope_x_{}", unique_test_id());
    create_scope(&scope_x, Some(jin_dir))?;

# Previous PRPs (CONTRACT - what P1.M3.T1.S2 and P1.M3.T2.S1 produce)
- file: /home/dustin/projects/jin/plan/P1M3T1S2/PRP.md
  why: Defines the mode metadata clearing behavior we need to test
  section: "Goal", "User Persona", "What"
  critical: |
    P1.M3.T1.S2 added automatic metadata clearing when switching modes.
    Message format: "Cleared workspace metadata (mode changed from '{old}' to '{new}'). Run 'jin apply' to apply new mode configuration."
    This happens in src/commands/mode.rs use_mode() function after line 116.

- file: /home/dustin/projects/jin/plan/P1M3T2S1/PRP.md
  why: Defines the scope metadata clearing behavior we need to test
  section: "Goal", "User Persona", "What"
  critical: |
    P1.M3.T2.S1 added automatic metadata clearing when switching scopes.
    Message format: "Cleared workspace metadata (scope changed from '{old}' to '{new}'). Run 'jin apply' to apply new scope configuration."
    This happens in src/commands/scope.rs use_scope() function after line 184.

# Implementation: Mode metadata clearing
- file: /home/dustin/projects/jin/src/commands/mode.rs
  why: Shows the exact implementation we're testing
  section: "use_mode() function (lines 87-164), specifically lines 118-159"
  pattern: |
    // Load workspace metadata (may not exist yet)
    let metadata = match WorkspaceMetadata::load() {
        Ok(meta) => Some(meta),
        Err(JinError::NotFound(_)) => None,
        Err(e) => return Err(e),
    };

    // Extract mode from metadata if present
    if let Some(meta) = &metadata {
        let metadata_mode = meta.applied_layers
            .iter()
            .find(|layer| layer.starts_with("mode/"))
            .and_then(|layer| layer.strip_prefix("mode/"))
            .and_then(|s| s.split('/').next());

        if let Some(old_mode) = metadata_mode {
            if old_mode != name {
                let metadata_path = WorkspaceMetadata::default_path();
                if metadata_path.exists() {
                    std::fs::remove_file(&metadata_path)?;
                    println!("Cleared workspace metadata (mode changed from '{}' to '{}').", old_mode, name);
                    println!("Run 'jin apply' to apply new mode configuration.");
                }
            }
        }
    }
  critical: |
    When switching from mode A to mode B where A != B:
    1. Metadata is deleted
    2. User sees clear message with both mode names
    3. User can then run jin apply without --force

# Implementation: Scope metadata clearing
- file: /home/dustin/projects/jin/src/commands/scope.rs
  why: Shows the exact implementation we're testing
  section: "use_scope() function (lines 142-233), specifically lines 185-227"
  pattern: |
    // Load workspace metadata (may not exist yet)
    let metadata = match WorkspaceMetadata::load() {
        Ok(meta) => Some(meta),
        Err(JinError::NotFound(_)) => None,
        Err(e) => return Err(e),
    };

    // Extract scope from metadata if present
    if let Some(meta) = &metadata {
        let metadata_scope = meta.applied_layers
            .iter()
            .find(|layer| layer.starts_with("scope/") && !layer.starts_with("mode/"))
            .and_then(|layer| layer.strip_prefix("scope/"))
            .and_then(|s| s.split('/').next());

        if let Some(old_scope) = metadata_scope {
            if old_scope != name {
                let metadata_path = WorkspaceMetadata::default_path();
                if metadata_path.exists() {
                    std::fs::remove_file(&metadata_path)?;
                    println!("Cleared workspace metadata (scope changed from '{}' to '{}').", old_scope, name);
                    println!("Run 'jin apply' to apply new scope configuration.");
                }
            }
        }
    }
  critical: |
    When switching from scope X to scope Y where X != Y:
    1. Metadata is deleted
    2. User sees clear message with both scope names
    3. User can then run jin apply without --force

# Apply command detached state error (what we're preventing)
- file: /home/dustin/projects/jin/src/staging/workspace.rs
  why: Shows the detached state validation that would fail without metadata clearing
  section: "validate_workspace_attached() function (lines 325-399)"
  note: |
    Without metadata clearing, switching modes would cause:
    "Workspace is in a detached state" error
    Because metadata.applied_layers references old mode/scope

# Apply command implementation
- file: /home/dustin/projects/jin/src/commands/apply.rs
  why: Shows how apply handles metadata and creates it on successful apply
  section: "Apply flow and metadata handling"
  note: |
    On successful apply, new metadata is created with:
    - metadata.applied_layers containing current mode/scope
    - metadata.files containing file content hashes

# WorkspaceMetadata structure
- file: /home/dustin/projects/jin/src/staging/metadata.rs
  why: Defines the structure that stores applied_layers
  section: "WorkspaceMetadata struct (lines 17-25), default_path() (lines 91-102)"
  pattern: |
    pub struct WorkspaceMetadata {
        pub timestamp: String,
        pub applied_layers: Vec<String>,  // Contains "mode/{name}" or "scope/{name}"
        pub files: HashMap<PathBuf, String>,
    }

    // Path: $JIN_DIR/workspace/last_applied.json or .jin/workspace/last_applied.json
    pub fn default_path() -> PathBuf {
        if let Ok(jin_dir) = std::env::var("JIN_DIR") {
            return PathBuf::from(jin_dir).join("workspace").join("last_applied.json");
        }
        PathBuf::from(".jin").join("workspace").join("last_applied.json")
    }

# Existing test patterns in mode_scope_workflow.rs
- file: /home/dustin/projects/jin/tests/mode_scope_workflow.rs
  why: Shows established patterns for mode/scope integration tests
  section: "All tests (lines 24-638)"
  pattern: |
    #[test]
    #[serial]
    fn test_layer_routing_mode_base() -> Result<(), Box<dyn std::error::Error>> {
        let fixture = TestFixture::new()?;
        let project_path = fixture.path();
        let jin_dir = fixture.jin_dir.as_ref().unwrap();

        fixture.set_jin_dir();
        jin_init(project_path, None)?;

        let mode_name = format!("test_mode_{}", unique_test_id());
        create_mode(&mode_name, Some(jin_dir))?;

        jin()
            .args(["mode", "use", &mode_name])
            .current_dir(project_path)
            .env("JIN_DIR", jin_dir)
            .assert()
            .success();

        // Create and add file
        fs::write(project_path.join("config.json"), r#"{"layer": "mode-base"}"#)?;

        jin()
            .args(["add", "config.json", "--mode"])
            .current_dir(project_path)
            .env("JIN_DIR", jin_dir)
            .assert()
            .success();

        jin()
            .args(["commit", "-m", "Mode base"])
            .current_dir(project_path)
            .env("JIN_DIR", jin_dir)
            .assert()
            .success();

        jin()
            .arg("apply")
            .current_dir(project_path)
            .env("JIN_DIR", jin_dir)
            .assert()
            .success();

        Ok(())
    }

# Test assertion helpers
- file: /home/dustin/projects/jin/tests/common/assertions.rs
  why: Provides helpers for verifying workspace state
  section: "assert_workspace_file() (lines 10-35), assert_workspace_file_exists() (lines 38-53)"
  pattern: |
    // Verify file content
    assert_workspace_file(project_path, "config.json", r#"{"env": "dev"}"#);

    // Verify file exists
    assert_workspace_file_exists(project_path, "config.json");

# Multi-command workflow test example
- file: /home/dustin/projects/jin/tests/mode_scope_workflow.rs
  why: Shows how to sequence multiple jin commands in a test
  section: "test_multiple_modes_isolated() (lines 569-638)"
  pattern: |
    // Add file to mode A
    jin().args(["mode", "use", &mode_a]).current_dir(project_path).env("JIN_DIR", jin_dir).assert().success();

    fs::write(project_path.join("a.txt"), "mode A content")?;

    jin().args(["add", "a.txt", "--mode"]).current_dir(project_path).env("JIN_DIR", jin_dir).assert().success();

    jin().args(["commit", "-m", "Mode A"]).current_dir(project_path).env("JIN_DIR", jin_dir).assert().success();

    // Switch to mode B
    jin().args(["mode", "use", &mode_b]).current_dir(project_path).env("JIN_DIR", jin_dir).assert().success();

    // ... continue with mode B operations ...

# Error message verification pattern
- pattern: |
    // Verify metadata clear message in stdout
    let result = jin()
        .args(["mode", "use", &mode_b])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert();

    let output = result.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout_str.contains(&format!("Cleared workspace metadata (mode changed from '{}' to '{}')", mode_a, mode_b)),
        "Expected metadata clear message. Got: {}",
        stdout_str
    );

# External Research: Rust Testing Best Practices
- url: https://doc.rust-lang.org/book/ch11-00-testing.html
  why: Official Rust testing documentation
  section: "Writing Tests, Test Organization, Running Tests"
  critical: |
    - Use Result<(), Box<dyn std::error::Error>> return type for integration tests
    - Use #[serial] attribute for tests that modify global state (JIN_DIR)
    - Use unique identifiers for parallel test safety

- url: https://docs.rs/assert_cmd/latest/assert_cmd/
  why: assert_cmd documentation for testing CLI commands
  section: "Command struct, assert() method, success() and failure() methods"
  critical: |
    - .assert().success() for commands that should succeed
    - .assert().failure() for commands that should fail
    - .get_output() to access stdout/stderr for custom assertions
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── tests/
│   ├── mode_scope_workflow.rs           # MODIFY: Add two new test functions
│   └── common/
│       ├── fixtures.rs                  # REFERENCE: TestFixture, create_mode, create_scope, unique_test_id
│       ├── assertions.rs                # REFERENCE: assert_workspace_file, assert_workspace_file_exists
│       ├── git_helpers.rs               # REFERENCE: Git cleanup helpers
│       └── mod.rs                       # REFERENCE: Common test utilities
└── src/
    ├── commands/
    │   ├── mode.rs                      # REFERENCE: Metadata clearing implementation (lines 118-159)
    │   ├── scope.rs                     # REFERENCE: Metadata clearing implementation (lines 185-227)
    │   └── apply.rs                     # REFERENCE: Apply command that uses metadata
    └── staging/
        └── metadata.rs                  # REFERENCE: WorkspaceMetadata structure
```

### Desired Codebase Tree After This Subtask

```bash
jin/
└── tests/
    └── mode_scope_workflow.rs           # MODIFIED: Add two new test functions
        ├── Add test_mode_switch_clears_metadata() after line 638
        │   ├── Creates two modes (mode_a, mode_b)
        │   ├── Activates mode_a, adds file, commits, applies
        │   ├── Switches to mode_b
        │   ├── Verifies clear message appears
        │   ├── Adds mode_b file, commits, applies
        │   ├── Verifies mode_b content is in workspace
        │   └── Returns Result<(), Box<dyn std::error::Error>>
        │
        └── Add test_scope_switch_clears_metadata() after above test
            ├── Creates two scopes (scope_x, scope_y)
            ├── Activates scope_x, adds file, commits, applies
            ├── Switches to scope_y
            ├── Verifies clear message appears
            ├── Adds scope_y file, commits, applies
            ├── Verifies scope_y content is in workspace
            └── Returns Result<(), Box<dyn std::error::Error>>
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: Set JIN_DIR BEFORE any Jin operations
// The fixture.set_jin_dir() call MUST happen before jin_init()
// Otherwise, Jin operations will use global ~/.jin directory

// GOTCHA: Test isolation requires unique names
// Use unique_test_id() to generate unique mode/scope names
// Otherwise parallel tests could conflict on mode/scope names
let mode_a = format!("mode_a_{}", unique_test_id());
let mode_b = format!("mode_b_{}", unique_test_id());

// GOTCHA: #[serial] attribute is REQUIRED
// Tests that set JIN_DIR environment variable must use #[serial]
// Otherwise tests can interfere with each other

// PATTERN: Command execution with isolated JIN_DIR
// Every jin() command must include .env("JIN_DIR", jin_dir)
// Otherwise command will use wrong Jin repository
jin()
    .args(["mode", "use", &mode_name])
    .current_dir(project_path)
    .env("JIN_DIR", jin_dir)  // CRITICAL: Must include for isolation
    .assert()
    .success();

// GOTCHA: Metadata clear message format
// Mode message: "Cleared workspace metadata (mode changed from '{old}' to '{new}'). Run 'jin apply' to apply new mode configuration."
// Scope message: "Cleared workspace metadata (scope changed from '{old}' to '{new}'). Run 'jin apply' to apply new scope configuration."
// Note: "mode" vs "scope" in the message

// GOTCHA: TestFixture lifetime
// Keep fixture in scope for entire test
// When fixture is dropped, temp directory is deleted
// Use let fixture = TestFixture::new()?; not let _fixture = ...

// GOTCHA: Git repository initialization
// jin_init() already calls git2::Repository::init(path)
// Don't call git init manually in tests

// GOTCHA: File content verification
// After switching to mode_b and applying, verify mode_b content is in workspace
// Use assert_workspace_file() or fs::read_to_string() + assert_eq!()

// GOTCHA: Scope name format in apply command
// When using --scope flag, format is: --scope=scope_name
// Note the equals sign (no space)

// GOTCHA: Mode flag vs scope flag
// Mode: --mode (no value needed, uses active mode from context)
// Scope: --scope=scope_name (requires value)

// GOTCHA: unique_test_id() returns String
// Use format!("mode_a_{}", unique_test_id()) not format!("mode_a_{}", unique_test_id())

// GOTCHA: Test function return type
// Use fn test_name() -> Result<(), Box<dyn std::error::Error>>
// Allows using ? operator throughout test

// CRITICAL: After switching modes/scopes, apply must succeed
// This is the main assertion - if apply fails, metadata clearing didn't work
// The test passes if apply succeeds without --force

// PATTERN: Verify metadata clear message
// Get command output and check stdout for the clear message
// let output = result.get_output();
// let stdout_str = String::from_utf8_lossy(&output.stdout);
// assert!(stdout_str.contains("Cleared workspace metadata"));

// GOTCHA: Workspace metadata file location
// For test isolation, JIN_DIR is set to temp dir
// Metadata file will be at: {jin_dir}/workspace/last_applied.json
// No need to check file exists - the clear message is sufficient verification
```

---

## Implementation Blueprint

### Data Models and Structure

**No new data models** - Tests use existing structures:
- `TestFixture` from `common::fixtures`
- `WorkspaceMetadata` from `src/staging/metadata.rs`
- Standard `fs`, `Path` types

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD test_mode_switch_clears_metadata() FUNCTION
  - LOCATION: tests/mode_scope_workflow.rs after line 638
  - IMPLEMENT: Complete mode switching workflow test
  - STRUCTURE:
    ```rust
    /// Test that switching modes automatically clears workspace metadata
    ///
    /// Workflow:
    /// 1. Create two modes (mode_a, mode_b)
    /// 2. Activate mode_a, add file, commit, apply
    /// 3. Switch to mode_b (should clear metadata)
    /// 4. Add mode_b file, commit, apply (should work without --force)
    /// 5. Verify mode_b content is in workspace
    #[test]
    #[serial]
    fn test_mode_switch_clears_metadata() -> Result<(), Box<dyn std::error::Error>> {
        // Implementation below
    }
    ```
  - DEPENDENCIES: None

Task 2: IMPLEMENT MODE TEST BODY
  - CREATE: TestFixture and set JIN_DIR
  - INITIALIZE: Jin repository with jin_init()
  - CREATE: Two unique mode names using unique_test_id()
  - CREATE: Both modes using create_mode() helper
  - ACTIVATE: First mode (mode_a) with jin mode use
  - ADD: File for mode_a with jin add --mode
  - COMMIT: Changes with jin commit -m
  - APPLY: Configuration with jin apply
  - SWITCH: To mode_b with jin mode use
  - VERIFY: Metadata clear message appears in stdout
  - ADD: Different file for mode_b
  - COMMIT: Mode_b changes
  - APPLY: Mode_b configuration (MUST succeed without --force)
  - VERIFY: Mode_b content is in workspace
  - DEPENDENCIES: Task 1

Task 3: ADD test_scope_switch_clears_metadata() FUNCTION
  - LOCATION: tests/mode_scope_workflow.rs after Task 1 function
  - IMPLEMENT: Complete scope switching workflow test
  - STRUCTURE: Mirror mode test but use scopes instead
  - DEPENDENCIES: Task 1

Task 4: IMPLEMENT SCOPE TEST BODY
  - CREATE: TestFixture and set JIN_DIR
  - INITIALIZE: Jin repository
  - CREATE: Two unique scope names
  - CREATE: Both scopes using create_scope() helper
  - ACTIVATE: First scope (scope_x)
  - ADD: File with --scope=scope_x flag
  - COMMIT: Changes
  - APPLY: Configuration
  - SWITCH: To scope_y
  - VERIFY: Metadata clear message appears
  - ADD: Different file for scope_y with --scope=scope_y
  - COMMIT: Scope_y changes
  - APPLY: Scope_y configuration (MUST succeed)
  - VERIFY: Scope_y content is in workspace
  - DEPENDENCIES: Task 3

Task 5: RUN TESTS TO VERIFY
  - RUN: cargo test test_mode_switch_clears_metadata
  - RUN: cargo test test_scope_switch_clears_metadata
  - RUN: cargo test mode_scope_workflow
  - VERIFY: All tests pass
  - DEPENDENCIES: Tasks 1-4
```

### Implementation Patterns & Key Details

```rust
// ================== MODE SWITCH TEST IMPLEMENTATION ==================

/// Test that switching modes automatically clears workspace metadata
///
/// Workflow:
/// 1. Create two modes (mode_a, mode_b)
/// 2. Activate mode_a, add file, commit, apply
/// 3. Switch to mode_b (should clear metadata)
/// 4. Add mode_b file, commit, apply (should work without --force)
/// 5. Verify mode_b content is in workspace
#[test]
#[serial]
fn test_mode_switch_clears_metadata() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test fixture
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    // CRITICAL: Set JIN_DIR BEFORE any Jin operations
    fixture.set_jin_dir();

    // Initialize Jin repository
    jin_init(project_path, None)?;

    // Create two unique mode names
    let mode_a = format!("mode_a_{}", unique_test_id());
    let mode_b = format!("mode_b_{}", unique_test_id());

    // Create both modes
    create_mode(&mode_a, Some(jin_dir))?;
    create_mode(&mode_b, Some(jin_dir))?;

    // === STEP 1: Activate mode_a and apply configuration ===
    jin()
        .args(["mode", "use", &mode_a])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Add a file for mode_a
    fs::write(
        project_path.join("config.json"),
        r#"{"mode": "mode_a"}"#,
    )?;

    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Mode A config"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Apply mode_a configuration
    jin()
        .arg("apply")
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify mode_a content is in workspace
    assert_workspace_file(project_path, "config.json", r#"{"mode": "mode_a"}"#);

    // === STEP 2: Switch to mode_b ===
    let result = jin()
        .args(["mode", "use", &mode_b])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert();

    // Verify metadata clear message appears
    let output = result.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout_str.contains(&format!(
            "Cleared workspace metadata (mode changed from '{}' to '{}')",
            mode_a, mode_b
        )),
        "Expected metadata clear message. Got: {}",
        stdout_str
    );

    // === STEP 3: Add mode_b configuration and apply ===
    // Note: File content changed to mode_b
    fs::write(
        project_path.join("config.json"),
        r#"{"mode": "mode_b"}"#,
    )?;

    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Mode B config"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // CRITICAL: This apply must succeed without --force
    // If metadata wasn't cleared, this would fail with "detached workspace" error
    jin()
        .arg("apply")
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify mode_b content is in workspace
    assert_workspace_file(project_path, "config.json", r#"{"mode": "mode_b"}"#);

    Ok(())
}

// ================== SCOPE SWITCH TEST IMPLEMENTATION ==================

/// Test that switching scopes automatically clears workspace metadata
///
/// Workflow:
/// 1. Create two scopes (scope_x, scope_y)
/// 2. Activate scope_x, add file, commit, apply
/// 3. Switch to scope_y (should clear metadata)
/// 4. Add scope_y file, commit, apply (should work without --force)
/// 5. Verify scope_y content is in workspace
#[test]
#[serial]
fn test_scope_switch_clears_metadata() -> Result<(), Box<dyn std::error::Error>> {
    // Setup test fixture
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    // CRITICAL: Set JIN_DIR BEFORE any Jin operations
    fixture.set_jin_dir();

    // Initialize Jin repository
    jin_init(project_path, None)?;

    // Create two unique scope names
    let scope_x = format!("scope_x_{}", unique_test_id());
    let scope_y = format!("scope_y_{}", unique_test_id());

    // Create both scopes
    create_scope(&scope_x, Some(jin_dir))?;
    create_scope(&scope_y, Some(jin_dir))?;

    // === STEP 1: Activate scope_x and apply configuration ===
    jin()
        .args(["scope", "use", &scope_x])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Add a file for scope_x (note: --scope= flag format)
    fs::write(
        project_path.join("config.json"),
        r#"{"scope": "scope_x"}"#,
    )?;

    jin()
        .args(["add", "config.json", &format!("--scope={}", scope_x)])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Scope X config"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Apply scope_x configuration
    jin()
        .arg("apply")
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify scope_x content is in workspace
    assert_workspace_file(project_path, "config.json", r#"{"scope": "scope_x"}"#);

    // === STEP 2: Switch to scope_y ===
    let result = jin()
        .args(["scope", "use", &scope_y])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert();

    // Verify metadata clear message appears
    let output = result.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout_str.contains(&format!(
            "Cleared workspace metadata (scope changed from '{}' to '{}')",
            scope_x, scope_y
        )),
        "Expected metadata clear message. Got: {}",
        stdout_str
    );

    // === STEP 3: Add scope_y configuration and apply ===
    // Note: File content changed to scope_y
    fs::write(
        project_path.join("config.json"),
        r#"{"scope": "scope_y"}"#,
    )?;

    jin()
        .args(["add", "config.json", &format!("--scope={}", scope_y)])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Scope Y config"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // CRITICAL: This apply must succeed without --force
    // If metadata wasn't cleared, this would fail with "detached workspace" error
    jin()
        .arg("apply")
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify scope_y content is in workspace
    assert_workspace_file(project_path, "config.json", r#"{"scope": "scope_y"}"#);

    Ok(())
}

// ================== PATTERN EXPLANATION ==================
//
// Test structure mirrors existing tests in mode_scope_workflow.rs:
// - Use #[test] #[serial] attributes
// - Return Result<(), Box<dyn std::error::Error>>
// - Use TestFixture with set_jin_dir() for isolation
// - Use unique_test_id() for mode/scope names
// - Sequence multiple jin commands with .env("JIN_DIR", jin_dir)
//
// Key assertions:
// 1. Metadata clear message appears on switch
// 2. Second apply succeeds without --force (critical!)
// 3. Second mode/scope content is in workspace
//
// Differences from mode test:
// - Use create_scope() instead of create_mode()
// - Use jin scope use instead of jin mode use
// - Use --scope=scope_name flag (equals sign, no space)
// - Message says "scope" instead of "mode"
```

### Integration Points

```yaml
TEST_FILE:
  - file: tests/mode_scope_workflow.rs
  - add_at: After line 638 (end of existing tests)
  - imports: Already imports all needed helpers (common::*, fs, serial_test::serial)

FIXTURE_HELPERS:
  - TestFixture::new() - Creates isolated test environment
  - fixture.set_jin_dir() - Sets JIN_DIR environment variable
  - create_mode(name, jin_dir) - Creates a mode
  - create_scope(name, jin_dir) - Creates a scope
  - unique_test_id() - Generates unique test identifier
  - jin_init(path, jin_dir) - Initializes Jin repository

ASSERTION_HELPERS:
  - assert_workspace_file(path, file, content) - Verifies file exists with exact content
  - jin() - Returns assert_cmd::Command for jin binary

COMMAND_PATTERN:
  - Always include .env("JIN_DIR", jin_dir) for isolation
  - Use .current_dir(project_path) to set working directory
  - Use .assert().success() for commands that should succeed
  - Use .get_output() to access stdout/stderr for custom assertions

METADATA_CLEARING_IMPLEMENTATION:
  - mode_metadata_clearing: src/commands/mode.rs lines 118-159
  - scope_metadata_clearing: src/commands/scope.rs lines 185-227
  - Both delete metadata file and print clear message

APPLY_COMMAND_BEHAVIOR:
  - Without metadata: Creates fresh workspace state
  - With metadata matching context: Updates workspace
  - With metadata from different mode/scope: Would cause detached error (if not cleared)
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after adding test functions - fix before proceeding
cargo check --tests                    # Type checking - MUST pass
cargo fmt -- --check                   # Format check

# Expected: Zero errors
# If errors exist, READ output and fix before proceeding
```

### Level 2: Unit Test Compilation

```bash
# Verify test compiles
cargo test --no-run test_mode_switch_clears_metadata
cargo test --no-run test_scope_switch_clears_metadata

# Expected: Tests compile successfully
# If compilation fails, check imports and syntax
```

### Level 3: Test Execution (Component Validation)

```bash
# Run new tests individually
cargo test test_mode_switch_clears_metadata -- --nocapture
cargo test test_scope_switch_clears_metadata -- --nocapture

# Run all mode_scope_workflow tests
cargo test mode_scope_workflow -- --nocapture

# Expected: All tests pass
# Key assertions that must pass:
# 1. Metadata clear message appears in stdout
# 2. Second jin apply succeeds without --force flag
# 3. Second mode/scope content verified in workspace
```

### Level 4: Full Test Suite (System Validation)

```bash
# Run full test suite to ensure no regressions
cargo test

# Expected: All tests pass
# Focus on: mode_scope_workflow, commands::mode, commands::scope tests

# Verify tests are serial-safe (run multiple times)
cargo test mode_scope_workflow -- --test-threads=1
cargo test test_mode_switch_clears_metadata -- --test-threads=1
```

### Level 5: Manual Verification (Optional)

```bash
# Manual verification (in temporary directory)
cd $(mktemp -d)
export JIN_DIR=$(pwd)/.jin_global
git init
jin init

# Create and test mode switching
jin mode create test_mode_a
jin mode create test_mode_b

# Apply mode A
jin mode use test_mode_a
echo '{"mode": "a"}' > config.json
jin add --mode config.json
jin commit -m "Mode A"
jin apply
cat config.json  # Should show {"mode": "a"}

# Switch to mode B
jin mode use test_mode_b
# Should see: "Cleared workspace metadata (mode changed from 'test_mode_a' to 'test_mode_b')"

# Apply mode B - should work!
echo '{"mode": "b"}' > config.json
jin add --mode config.json
jin commit -m "Mode B"
jin apply
cat config.json  # Should show {"mode": "b"}

# Expected: All commands succeed, no detached state errors
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check --tests` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo test test_mode_switch_clears_metadata` passes
- [ ] `cargo test test_scope_switch_clears_metadata` passes
- [ ] `cargo test mode_scope_workflow` all tests pass
- [ ] `cargo test` all tests pass (no regressions)

### Feature Validation

- [ ] Mode test verifies metadata clear message appears on switch
- [ ] Mode test verifies second apply succeeds without `--force`
- [ ] Mode test verifies second mode content is in workspace
- [ ] Scope test verifies metadata clear message appears on switch
- [ ] Scope test verifies second apply succeeds without `--force`
- [ ] Scope test verifies second scope content is in workspace
- [ ] Tests use `unique_test_id()` for parallel safety
- [ ] Tests use `#[serial]` attribute for JIN_DIR modification

### Code Quality Validation

- [ ] Test structure matches existing tests in mode_scope_workflow.rs
- [ ] Uses TestFixture pattern correctly
- [ ] All jin() commands include `.env("JIN_DIR", jin_dir)`
- [ ] Fixture is kept in scope (not dropped early)
- [ ] Uses `Result<(), Box<dyn std::error::Error>>` return type
- [ ] Test names are descriptive and follow naming convention
- [ ] Comments explain the workflow steps

### Documentation & Deployment

- [ ] Test function has doc comment explaining workflow
- [ ] Assertions have clear failure messages
- [ ] Test can be understood as documentation of expected behavior
- [ ] No test isolation issues (uses unique_test_id)

---

## Anti-Patterns to Avoid

- **Don't** forget to call `fixture.set_jin_dir()` before `jin_init()`
- **Don't** skip `.env("JIN_DIR", jin_dir)` on jin() commands
- **Don't** use static mode/scope names (use `unique_test_id()`)
- **Don't** forget `#[serial]` attribute on tests that modify JIN_DIR
- **Don't** drop fixture early (keep in scope for entire test)
- **Don't** use `--scope scope_name` format (should be `--scope=scope_name`)
- **Don't** use `--mode mode_name` format (should be just `--mode`)
- **Don't** skip the final apply verification (this is the critical test!)
- **Don't** use `let _fixture =` pattern (keeps fixture but less clear)
- **Don't** verify metadata file directly (clear message is sufficient)
- **Don't** skip content verification after second apply
- **Don't** use mode/scope names that could collide (always use unique_test_id)

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification**:
- **Single-file change**: Only `tests/mode_scope_workflow.rs` needs modification
- **Well-established patterns**: Existing tests in same file show exact patterns to follow
- **Clear test structure**: Workflow is straightforward (create → apply → switch → apply)
- **No new dependencies**: Uses existing fixtures and assertions
- **Isolated tests**: Each test is independent with unique mode/scope names
- **Clear success criteria**: Tests pass if apply succeeds after switch
- **Comprehensive examples**: Mode test and scope test mirror each other
- **Established helpers**: `create_mode()`, `create_scope()`, `unique_test_id()` available

**Zero Risk Factors**:
- Adding tests cannot break existing code
- Tests are isolated in separate functions
- Uses same patterns as existing tests in file
- No modifications to production code
- Test failures only indicate issues with test implementation

**Current Status**: Ready for implementation - all context gathered, patterns identified, test structure is clear

---

## Research Artifacts Location

Research documentation referenced throughout this PRP:

**Primary Research** (from this PRP creation):
- `plan/P1M3T3S1/research/` - Directory for all research findings
  - Agent research files stored here for reference

**Background Documentation**:
- `tests/mode_scope_workflow.rs` - Test file to modify (existing patterns)
- `tests/common/fixtures.rs` - TestFixture and helper functions
- `tests/common/assertions.rs` - Assertion helpers
- `src/commands/mode.rs` - Mode metadata clearing implementation
- `src/commands/scope.rs` - Scope metadata clearing implementation
- `src/staging/metadata.rs` - WorkspaceMetadata structure
- `src/commands/apply.rs` - Apply command behavior

**Pattern References**:
- `test_layer_routing_mode_base()` (lines 24-72) - Mode test pattern
- `test_layer_routing_mode_scope()` (lines 131-191) - Scope test pattern
- `test_multiple_modes_isolated()` (lines 569-638) - Multi-mode workflow pattern

**Related PRPs**:
- `plan/P1M3T1S2/PRP.md` - Mode metadata clearing implementation
- `plan/P1M3T2S1/PRP.md` - Scope metadata clearing implementation

**External Research**:
- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html) - Official Rust testing guide
- [assert_cmd documentation](https://docs.rs/assert_cmd/latest/assert_cmd/) - CLI testing library
