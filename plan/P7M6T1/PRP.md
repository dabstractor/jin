name: "P7.M6.T1: Fix Test Environment Issues - Test Isolation & Git Lock File Remediation"
description: |

---

## Goal

**Feature Goal**: Fix test environment isolation issues by implementing proper test fixture isolation, temp directory management, and Git lock file handling to ensure tests run reliably without interference between test runs.

**Deliverable**: Updated test infrastructure with:
- Isolated test fixtures that don't share global state
- Proper temp directory handling with guaranteed cleanup
- Git lock file detection and cleanup mechanisms
- Consistent JIN_DIR usage across all tests

**Success Definition**:
- All tests run reliably with `cargo test`
- Tests can run in parallel without lock file conflicts
- No global state pollution between test runs
- Zero dependency on user's `~/.jin` configuration

## User Persona (if applicable)

**Target User**: Developer running tests (CI/CD systems, contributors, maintainers)

**Use Case**: Running test suite during development, in CI pipelines, or before merging changes

**User Journey**:
1. Developer runs `cargo test` to verify changes
2. Tests execute in isolated environments
3. Tests complete successfully without lock file errors
4. No manual cleanup required between test runs

**Pain Points Addressed**:
- Flaky tests due to global state pollution
- Git lock file conflicts causing test failures
- Tests depending on external `~/.jin` configuration
- Modes/scopes accumulating across test runs

## Why

- Test reliability: Current tests suffer from isolation issues causing flaky behavior
- CI/CD stability: Parallel test execution causes Git lock file conflicts
- Developer productivity: Manual cleanup required between test runs
- Onboarding friction: New contributors must have `~/.jin` initialized

## What

Fix test environment issues through:

### Success Criteria

- [ ] All mode/scope tests use isolated Jin repositories
- [ ] Git lock files are detected and cleaned up automatically
- [ ] Tests don't require pre-initialized `~/.jin` directory
- [ ] Tests can run with `cargo test --test-threads=N` for any N
- [ ] No global state accumulation between test runs

## All Needed Context

### Context Completeness Check

_This PRP passes the "No Prior Knowledge" test - all necessary context, file paths, patterns, and external research is provided below._

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# External Research Resources
- url: https://docs.rs/tempfile/latest/tempfile/
  why: tempfile crate API reference for TempDir, NamedTempFile, and automatic cleanup
  critical: TempDir implements Drop - must keep in scope to prevent premature cleanup

- url: https://doc.rust-lang.org/book/ch11-03-test-organization.html#the-tests-directory
  why: Rust integration test architecture and isolation guarantees
  critical: Integration tests run as separate binaries, providing process-level isolation

- url: https://docs.rs/git2/latest/git2/
  why: git2 crate documentation for Repository operations and lock file behavior
  critical: git2::Repository is Send but NOT Sync - concurrent access requires Mutex

- url: https://github.com/rust-lang/git2-rs/issues/194
  why: Thread safety implications for git2::Repository
  critical: "Use an object from a single thread at a time"

# Existing Codebase Patterns
- file: tests/cli_basic.rs
  why: Example of CORRECT JIN_DIR isolation pattern (lines 34-36, 55, 67-68)
  pattern: env("JIN_DIR", temp.path().join(".jin_global"))
  gotcha: This pattern exists but is NOT consistently used across all tests

- file: tests/common/fixtures.rs
  why: Core test fixture definitions that need modification
  pattern: TestFixture struct with TempDir (lines 15-20)
  gotcha: create_mode() and create_scope() (lines 174-213) modify global ~/.jin without isolation

- file: tests/common/assertions.rs
  why: Assertions that need modification to accept repository path
  pattern: assert_layer_ref_exists() directly references ~/.jin (lines 134-158)
  gotcha: Hardcoded path prevents test isolation

- file: tests/mode_scope_workflow.rs
  why: Primary target for fixes - mode/scope workflow tests
  pattern: Uses create_mode() which pollutes global repository (line 27)
  gotcha: No JIN_DIR isolation, depends on global state

- file: tests/error_scenarios.rs
  why: Error handling tests that also lack isolation
  pattern: Uses setup_test_repo() without Jin isolation
  gotcha: May leave global Jin in inconsistent state

# Project Documentation
- docfile: plan/docs/CODEBASE_PATTERNS.md
  why: Existing project patterns to follow
  section: Test patterns and conventions

- docfile: plan/docs/WORKFLOWS.md
  why: Understanding mode/scope workflows being tested
  section: Mode and Scope operations
```

### Current Codebase Tree

```bash
tests/
├── common/
│   ├── mod.rs                 # Module declarations
│   ├── assertions.rs          # ❌ Hardcoded ~/.jin paths (lines 134-158)
│   └── fixtures.rs            # ❌ Global state pollution (lines 174-213)
├── mode_scope_workflow.rs     # ❌ No JIN_DIR isolation (line 27)
├── error_scenarios.rs         # ❌ No JIN_DIR isolation
├── cli_basic.rs               # ✅ Has correct isolation pattern (lines 34-36)
├── cli_diff.rs                # ⚠️ Mixed isolation patterns
├── cli_import.rs              # ⚠️ Mixed isolation patterns
├── cli_reset.rs               # ⚠️ Mixed isolation patterns
├── atomic_operations.rs       # ⚠️ Git operations without lock handling
├── cli_list.rs
├── cli_mv.rs
├── core_workflow.rs
└── sync_workflow.rs
```

### Desired Codebase Tree with Changes

```bash
tests/
├── common/
│   ├── mod.rs                 # Module declarations
│   ├── assertions.rs          # ✅ Accept optional repo_path parameter
│   ├── fixtures.rs            # ✅ Isolated fixture creation with lock cleanup
│   └── git_helpers.rs         # ✅ NEW: Git lock file handling utilities
├── mode_scope_workflow.rs     # ✅ Use isolated fixtures
├── error_scenarios.rs         # ✅ Use isolated fixtures
├── cli_basic.rs               # ✅ Consistent JIN_DIR pattern
├── cli_diff.rs                # ✅ Consistent JIN_DIR pattern
├── cli_import.rs              # ✅ Consistent JIN_DIR pattern
├── cli_reset.rs               # ✅ Consistent JIN_DIR pattern
└── [other test files...]      # ✅ All use isolated fixtures
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: git2::Repository is Send but NOT Sync
// Source: https://github.com/rust-lang/git2-rs/issues/194
// Cannot be shared between threads without Mutex wrapper

// CRITICAL: TempDir cleanup happens on Drop
// MUST keep TempDir in scope - if dropped early, directory is deleted
// Example: TestFixture stores _tempdir to prevent premature cleanup

// CRITICAL: Git creates .git/index.lock files during operations
// These must be cleaned up if tests crash or fail mid-operation

// CRITICAL: dirs::home_dir() in assertions.rs creates external dependency
// Tests should NOT depend on user's actual ~/.jin configuration

// CRITICAL: std::process::id() does NOT guarantee uniqueness for parallel tests
// Use thread-local counters or test-specific unique identifiers instead

// CRITICAL: JIN_DIR environment variable MUST be set BEFORE test execution
// Some operations respect it, others don't - inconsistent behavior

// GOTCHA: Mode and scope creation writes to global Jin repository by default
// Must use JIN_DIR override or create isolated repositories explicitly
```

## Implementation Blueprint

### Data Models and Structure

No new data models required. This task modifies test infrastructure only.

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE tests/common/git_helpers.rs
  - IMPLEMENT: Git lock file detection and cleanup utilities
  - FUNCTIONS:
    * cleanup_git_locks(repo_path: &Path) -> Result<()>
    * with_lock_retry<T, F>(repo_path: &Path, f: F) -> Result<T> where F: Fn() -> Result<T>
    * GitTestEnv wrapper struct with automatic cleanup on Drop
  - PATTERN: Follow existing fixture.rs structure conventions
  - NAMING: snake_case for functions, CamelCase for structs
  - DEPENDENCIES: None (new module)

Task 2: MODIFY tests/common/fixtures.rs
  - MODIFY: TestFixture struct to include optional JIN_DIR
    ADD: jin_dir: Option<PathBuf> field
    MODIFY: new() to accept isolated Jin directory
  - MODIFY: RemoteFixture struct with lock cleanup
    ADD: cleanup_git_locks() call in Drop implementation
  - MODIFY: create_mode() and create_scope() functions
    ADD: jin_dir parameter (Option<PathBuf>)
    SET: JIN_DIR environment variable when provided
    USE: Unique naming with test-specific identifiers
  - MODIFY: All fixture functions to support Jin isolation
    ADD: jin_dir parameter to setup_test_repo(), setup_jin_with_remote()
  - DEPENDENCIES: Task 1 (git_helpers module)

Task 3: MODIFY tests/common/assertions.rs
  - MODIFY: assert_layer_ref_exists() function
    ADD: Optional jin_repo_path parameter
    DEFAULT: None for backward compatibility (uses ~/.jin if None)
    REMOVE: Hardcoded dirs::home_dir().join(".jin")
    USE: Provided path or fall back to environment
  - MODIFY: assert_context_mode() function
    ADD: Optional project_path parameter for context file location
  - MODIFY: assert_context_scope() function
    ADD: Optional project_path parameter for context file location
  - DEPENDENCIES: Task 2 (fixture modifications)

Task 4: MODIFY tests/mode_scope_workflow.rs
  - MODIFY: All test functions to use isolated Jin directory
    ADD: let jin_dir = temp.path().join(".jin_global");
    ADD: std::env::set_var("JIN_DIR", &jin_dir);
    PASS: jin_dir to create_mode() and create_scope() calls
  - MODIFY: Mode/scope creation calls
    FROM: create_mode("test_mode")
    TO: create_mode("test_mode", Some(&jin_dir))
  - UPDATE: Unique naming strategy
    FROM: format!("test_mode_{}", std::process::id())
    TO: format!("test_mode_{}_{thread_id}", unique_test_id())
  - DEPENDENCIES: Task 2, Task 3

Task 5: MODIFY tests/error_scenarios.rs
  - MODIFY: All test functions to use JIN_DIR isolation
    ADD: JIN_DIR environment variable setup
    PASS: jin_dir to fixture functions
  - ADD: Cleanup after error scenarios
    ENSURE: Global state is restored after test
  - DEPENDENCIES: Task 2, Task 3

Task 6: UPDATE tests/cli_diff.rs, tests/cli_import.rs, tests/cli_reset.rs
  - MODIFY: Ensure consistent JIN_DIR usage across all CLI tests
    STANDARDIZE: Pattern from cli_basic.rs (lines 34-36)
    ADD: .env("JIN_DIR", ...) to all jin() command calls
  - DEPENDENCIES: Task 2

Task 7: UPDATE remaining test files (cli_list.rs, cli_mv.rs, core_workflow.rs, sync_workflow.rs, atomic_operations.rs)
  - MODIFY: Apply isolation patterns consistently
    ADD: JIN_DIR environment variable setup where missing
    ADD: Git lock cleanup for operations that create repositories
  - DEPENDENCIES: Task 2

Task 8: CREATE tests/common/mod.rs updates
  - ADD: pub mod git_helpers; declaration
  - EXPORT: New git helper functions for use in tests
  - DEPENDENCIES: Task 1

Task 9: VERIFY all tests pass with isolation
  - RUN: cargo test
  - RUN: cargo test --test-threads=1 (sequential)
  - RUN: cargo test --test-threads=4 (parallel)
  - VERIFY: No lock file errors
  - VERIFY: Tests pass without ~/.jin initialization
  - DEPENDENCIES: All previous tasks
```

### Implementation Patterns & Key Details

```rust
// ============================================================================
// Pattern 1: Git Lock File Cleanup (tests/common/git_helpers.rs)
// ============================================================================

use std::path::Path;
use std::fs;

/// Cleans up stale Git lock files in a repository
pub fn cleanup_git_locks(repo_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let git_dir = repo_path.join(".git");

    // Clean index.lock
    let index_lock = git_dir.join("index.lock");
    if index_lock.exists() {
        fs::remove_file(&index_lock)?;
    }

    // Clean other common lock files
    for lock_file in &["HEAD.lock", "refs/heads/main.lock"] {
        let lock_path = git_dir.join(lock_file);
        if lock_path.exists() {
            let _ = fs::remove_file(&lock_path); // Ignore errors
        }
    }

    Ok(())
}

/// Wrapper for test environments with automatic Git lock cleanup
pub struct GitTestEnv {
    temp_dir: tempfile::TempDir,
    repo_path: std::path::PathBuf,
}

impl GitTestEnv {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = tempfile::TempDir::new()?;
        let repo_path = temp_dir.path().to_path_buf();
        Ok(Self { temp_dir, repo_path })
    }

    pub fn path(&self) -> &Path {
        &self.repo_path
    }
}

impl Drop for GitTestEnv {
    fn drop(&mut self) {
        // CRITICAL: Clean up locks before temp dir is deleted
        let _ = cleanup_git_locks(&self.repo_path);
    }
}

// ============================================================================
// Pattern 2: Isolated Fixture with JIN_DIR (tests/common/fixtures.rs)
// ============================================================================

use std::path::PathBuf;

pub struct TestFixture {
    _tempdir: tempfile::TempDir,
    pub path: PathBuf,
    pub jin_dir: Option<PathBuf>,  // NEW: Isolated Jin directory
}

impl TestFixture {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = tempfile::TempDir::new()?;
        let path = tempdir.path().to_path_buf();
        let jin_dir = Some(path.join(".jin_global"));  // NEW: Isolated Jin

        Ok(Self {
            _tempdir: tempdir,
            path,
            jin_dir,
        })
    }

    /// Sets JIN_DIR environment variable for this fixture
    pub fn set_jin_dir(&self) {
        if let Some(ref jin_dir) = self.jin_dir {
            std::env::set_var("JIN_DIR", jin_dir);
        }
    }
}

// ============================================================================
// Pattern 3: Isolated Mode Creation (tests/common/fixtures.rs)
// ============================================================================

/// Creates a mode in an isolated Jin repository
pub fn create_mode(
    mode_name: &str,
    jin_dir: Option<&PathBuf>
) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = jin();

    // CRITICAL: Set JIN_DIR before command execution
    if let Some(jin_dir) = jin_dir {
        cmd.env("JIN_DIR", jin_dir);
    }

    cmd.args(["mode", "create", mode_name])
        .assert()
        .success();

    Ok(())
}

// ============================================================================
// Pattern 4: Unique Test ID Generator (tests/common/fixtures.rs)
// ============================================================================

use std::sync::atomic::{AtomicUsize, Ordering};

/// Generates unique test identifiers
/// GOTCHA: std::process::id() is not sufficient for parallel tests
pub fn unique_test_id() -> String {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    let thread_id = std::thread::current().id();
    format!("{}_{}_{:?}", std::process::id(), count, thread_id)
}

/// Usage in tests:
/// let mode_name = format!("test_mode_{}", unique_test_id());

// ============================================================================
// Pattern 5: Updated Assertions (tests/common/assertions.rs)
// ============================================================================

/// Asserts that a layer ref exists in the Jin repository
/// NEW: Accepts optional repository path for isolation
pub fn assert_layer_ref_exists(ref_path: &str, jin_repo_path: Option<&Path>) {
    let repo_path = match jin_repo_path {
        Some(path) => path.clone(),
        None => {
            // Fallback to environment variable or home directory
            if let Ok(jin_dir) = std::env::var("JIN_DIR") {
                PathBuf::from(jin_dir)
            } else {
                dirs::home_dir().expect("Failed to get home directory").join(".jin")
            }
        }
    };

    let repo = git2::Repository::open(&repo_path)
        .expect("Failed to open Jin repository");

    repo.find_reference(ref_path)
        .expect(&format!("Layer ref '{}' not found", ref_path));
}

// ============================================================================
// Pattern 6: Test Usage (tests/mode_scope_workflow.rs)
// ============================================================================

#[test]
fn test_mode_layer_routing() -> Result<(), Box<dyn std::error::Error>> {
    // Create isolated fixture
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    // CRITICAL: Set JIN_DIR BEFORE any Jin operations
    fixture.set_jin_dir();

    // Initialize project
    jin_init(project_path)?;

    // Create mode with isolated Jin
    let mode_name = format!("test_mode_{}", unique_test_id());
    create_mode(&mode_name, Some(jin_dir))?;

    // Use mode
    jin()
        .env("JIN_DIR", jin_dir)  // CRITICAL: Always set JIN_DIR
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .assert()
        .success();

    // Add file and verify
    // ... test code ...

    Ok(())
}
```

### Integration Points

```yaml
ENVIRONMENT:
  - variable: "JIN_DIR"
    purpose: Override default ~/.jin location for test isolation
    pattern: "env(\"JIN_DIR\", temp.path().join(\".jin_global\"))"

TEST_COMMON:
  - module: tests/common/fixtures.rs
    exports: TestFixture, RemoteFixture, create_mode, create_scope, unique_test_id
  - module: tests/common/git_helpers.rs (NEW)
    exports: cleanup_git_locks, GitTestEnv
  - module: tests/common/assertions.rs
    exports: assert_layer_ref_exists, assert_context_mode, assert_context_scope

GIT_OPERATIONS:
  - crate: git2
    type: Repository operations
    pattern: Always cleanup locks after operations
    gotcha: Repository is NOT thread-safe

TEST_EXECUTION:
  - command: cargo test
  - command: cargo test --test-threads=1
  - command: cargo test --test-threads=N
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file creation - fix before proceeding
cargo check --tests                    # Check test code compilation
cargo clippy --tests -- -D warnings   # Lint checking for tests
cargo fmt                              # Format all code

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test the new git_helpers module
cargo test --test git_helpers

# Test fixture modifications
cargo test --test common

# Test mode/scope workflow with isolation
cargo test --test mode_scope_workflow

# Full test suite
cargo test --all

# Expected: All tests pass. If failing, debug root cause and fix implementation.
```

### Level 3: Integration Testing (System Validation)

```bash
# Sequential test execution (no lock conflicts)
cargo test --test-threads=1

# Parallel test execution (tests isolation)
cargo test --test-threads=4

# Clean build test (no cached artifacts)
cargo clean
cargo test

# Test without ~/.jin initialization
# Temporarily rename ~/.jin if it exists
mv ~/.jin ~/.jin.backup 2>/dev/null || true
cargo test
mv ~/.jin.backup ~/.jin 2>/dev/null || true

# Expected:
# - All test execution modes pass
# - No "index.lock" errors
# - Tests pass without pre-initialized ~/.jin
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Stress test: Run tests multiple times to catch race conditions
for i in {1..10}; do
    echo "Run $i:"
    cargo test --test-threads=4 || exit 1
done

# Test isolation verification
# Run specific tests in different orders to detect dependencies
cargo test mode_scope_workflow::test_mode_layer_routing
cargo test error_scenarios::test_permission_error
cargo test mode_scope_workflow::test_mode_layer_routing  # Again

# Git lock file cleanup verification
# Create a test that intentionally leaves a lock file
# Verify subsequent tests can still run

# Performance: Verify isolation doesn't significantly slow tests
time cargo test

# Expected:
# - All iterations pass (no flaky tests)
# - Tests pass regardless of execution order
# - Lock files are properly cleaned up
# - Test execution time is reasonable
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --all`
- [ ] Tests pass sequentially: `cargo test --test-threads=1`
- [ ] Tests pass in parallel: `cargo test --test-threads=4`
- [ ] No linting errors: `cargo clippy --tests -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check`

### Feature Validation

- [ ] Tests pass without ~/.jin initialization
- [ ] No Git lock file errors in any test execution mode
- [ ] Mode/scope tests use isolated Jin repositories
- [ ] Global state doesn't accumulate between test runs
- [ ] Tests can run in any order without interference

### Code Quality Validation

- [ ] New git_helpers.rs module follows existing patterns
- [ ] Modified fixtures.rs maintains backward compatibility
- [ ] Assertions accept optional repository path parameter
- [ ] All test files use consistent JIN_DIR isolation pattern
- [ ] Unique test ID generation prevents naming conflicts

### Documentation & Deployment

- [ ] Code changes are self-documenting with clear names
- [ ] Comments explain non-obvious patterns (JIN_DIR, lock cleanup)
- [ ] No breaking changes to public test utility APIs
- [ ] CI/CD pipeline should work without modifications

---

## Anti-Patterns to Avoid

- ❌ Don't use `std::process::id()` alone for unique names (not thread-safe enough)
- ❌ Don't hardcode `~/.jin` paths in assertions
- ❌ Don't create modes/scopes without JIN_DIR isolation
- ❌ Don't let TempDir go out of scope prematurely
- ❌ Don't ignore Git lock file cleanup in Drop implementations
- ❌ Don't use `git2::Repository` from multiple threads without Mutex
- ❌ Don't assume tests run sequentially (plan for parallel execution)
- ❌ Don't leave global state (modes, scopes) after test completion
- ❌ Don't create new patterns when existing ones work (follow cli_basic.rs pattern)
- ❌ Don't skip cleanup because "TempDir will handle it" (locks need explicit cleanup)
