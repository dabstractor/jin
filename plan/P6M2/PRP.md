# PRP: P6.M2 - Integration Testing

---

## Goal

**Feature Goal**: Implement a comprehensive end-to-end integration test suite for Jin CLI that validates complete user workflows, Git operations, error handling, and multi-command sequences to ensure production reliability.

**Deliverable**: Complete integration test suite in `tests/` directory covering:
1. Core workflow tests (init → mode → add → commit → apply)
2. Mode/scope workflow tests (layer routing and precedence)
3. Sync workflow tests (fetch → pull → push with local remotes)
4. Error recovery tests (conflicts, corruption, rollback)
5. Atomic operation tests (transaction safety)

**Success Definition**:
- All 25 CLI commands tested through complete workflows
- 70% of tests cover error conditions and recovery
- Tests use real Git with local filesystem remotes
- All tests pass in parallel (isolation verified)
- Test suite completes in <5 minutes
- Zero flaky tests (verified with 10x consecutive runs)
- Tests pass on Linux (primary platform)

## User Persona

**Target User**: Jin developer verifying implementation correctness through automated testing

**Use Case**: Developer makes changes to Jin codebase and needs confidence that:
- All user workflows work end-to-end
- Error conditions are handled gracefully
- State changes are atomic and reversible
- Changes don't break existing functionality
- Code works across platforms

**User Journey**:
1. Developer makes code changes
2. Runs `cargo test` to verify changes
3. Integration tests validate complete workflows
4. CI runs full test matrix (platforms)
5. Tests either pass (deploy) or fail (fix)

**Pain Points Addressed**:
- **Manual testing is slow**: Automated tests run in minutes
- **Incomplete coverage**: Systematic test scenarios ensure nothing missed
- **Regression bugs**: Tests catch breaking changes immediately
- **Platform issues**: CI matrix catches OS-specific problems
- **Unclear failures**: Tests provide specific failure messages

## Why

**Business Value**:
- **Confidence in changes**: Developers can refactor safely
- **Fast feedback**: Issues caught in minutes, not days
- **Quality gate**: Prevents broken code from reaching users
- **Documentation**: Tests serve as executable specification
- **Onboarding**: New contributors understand Jin through tests

**Integration with Existing Features**:
- Uses existing test file `tests/cli_basic.rs` as foundation
- Leverages `assert_cmd`, `predicates`, `tempfile` already in dev-dependencies
- Tests actual CLI binary (`Command::cargo_bin("jin")`)
- Validates against PRD requirements for all commands

**Problems This Solves**:
- **No end-to-end validation**: Current tests only check CLI parsing
- **Error paths untested**: Production bugs from unhandled errors
- **No multi-step workflows**: Commands tested in isolation, not sequences
- **No Git operation testing**: Core functionality (layers, commits) untested
- **No failure recovery testing**: Corruption/conflict scenarios not validated

## What

### User-Visible Behavior

Integration tests validate the same behavior users experience:

**Scenario 1: Simple Mode Workflow**
```bash
jin init
jin mode create claude
jin mode use claude
echo '{"key": "value"}' > .claude/config.json
jin add .claude/config.json --mode
jin commit -m "Add Claude config"
jin apply

# Expected: config.json applied to workspace from mode layer
```

**Scenario 2: Layer Precedence**
```bash
jin add base.json --mode              # Layer 2
jin add base.json --mode --project    # Layer 5 (higher precedence)
jin apply

# Expected: Layer 5 content wins (deep merge if JSON)
```

**Scenario 3: Error Recovery**
```bash
jin add file.txt
# Corrupt staging index
jin repair
jin status

# Expected: Repair detects and fixes corruption
```

**Scenario 4: Remote Sync**
```bash
jin link file:///path/to/remote.git
jin fetch
jin pull
jin mode use dev
jin apply

# Expected: Remote changes merged and applied
```

### Success Criteria

- [ ] All core workflows pass (init, mode, scope, add, commit, apply)
- [ ] Mode/scope layer routing validated (9-layer hierarchy)
- [ ] Remote sync tested with local bare repos (no network)
- [ ] Error scenarios covered (conflicts, corruption, permissions)
- [ ] Atomic operations verified (commits are all-or-nothing)
- [ ] Tests run in parallel without interference
- [ ] Test suite completes in <5 minutes
- [ ] Zero flaky tests (10 consecutive successful runs)

## All Needed Context

### Context Completeness Check

_"If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_

**Answer**: Yes, with the comprehensive research provided in `plan/P6M2/research/`:
- 25+ authoritative sources
- 8,414 lines of research documentation
- Real-world patterns from Cargo, Git, Kubernetes
- Existing test patterns from `tests/cli_basic.rs`
- Complete command analysis in `research/command_analysis.md`

### Documentation & References

```yaml
# MUST READ - Start with these in order
- file: plan/P6M2/research/START_HERE.md
  why: Quick overview of all research findings
  critical: Understand test philosophy (real Git + local repos + temp dirs)

- file: plan/P6M2/research/QUICK_REFERENCE.md
  why: Fast lookup for patterns and decisions
  critical: Decision trees for test scenarios, common templates

- file: plan/P6M2/research/command_analysis.md
  why: Complete analysis of all 25 Jin CLI commands
  critical: Test scenarios for each command, dependencies, workflows

# Core Testing Patterns
- file: plan/P6M2/research/05_e2e_workflow_testing.md
  why: End-to-end multi-step workflow patterns
  pattern: Sequential command testing, state verification
  critical: This is the primary pattern for integration tests

- file: plan/P6M2/research/04_git_integration_testing.md
  why: How to test Git operations properly
  pattern: Local filesystem remotes, real Git behavior
  critical: Use local bare repos, not mocks

- file: plan/P6M2/research/03_tempfile_and_fixtures.md
  why: Setup/teardown patterns with automatic cleanup
  pattern: TempDir usage, fixture organization
  critical: Proper isolation prevents flaky tests

# Error Testing
- file: plan/P6M2/research/06-error-recovery-testing.md
  why: 70% of tests should be error paths
  pattern: Safe error injection, rollback testing
  critical: Test merge conflicts, corruption, permissions

- file: plan/P6M2/research/07-atomic-operations-testing.md
  why: Transaction safety verification
  pattern: Crash safety, consistency checks
  critical: Commit atomicity must be tested

# Existing Code Patterns
- file: tests/cli_basic.rs
  why: Current test structure and patterns
  pattern: Uses assert_cmd, predicates, Command::cargo_bin("jin")
  gotcha: Current tests only verify CLI parsing, not actual functionality

- file: plan/docs/CODEBASE_PATTERNS.md
  why: Jin implementation patterns
  pattern: Error types, layer routing, transaction system
  gotcha: Jin repo is BARE (no working directory at ~/.jin/)

# Tools and Frameworks
- url: https://docs.rs/assert_cmd/latest/assert_cmd/
  why: CLI testing framework (already in dev-dependencies)
  critical: Command::cargo_bin() finds binary, .assert() validates output

- url: https://docs.rs/predicates/latest/predicates/
  why: Output assertions (str::contains, path::exists)
  critical: Use predicate::str::contains() for flexible matching

- url: https://docs.rs/tempfile/latest/tempfile/
  why: Temporary directories with automatic cleanup
  critical: TempDir::new() for isolation, auto-cleanup on drop

# Real-World Examples
- file: plan/P6M2/research/RESEARCH_SUMMARY.md
  why: How Cargo and Git structure their tests
  critical: Cargo separates lib.rs (logic) from main.rs (CLI)

- url: https://github.com/rust-lang/cargo/tree/master/tests
  why: Cargo's integration test structure
  critical: Multiple test files, common fixtures, real operations
```

### Current Codebase Tree

```
jin/
├── Cargo.toml               (dev-dependencies: assert_cmd, predicates, tempfile)
├── src/
│   ├── lib.rs               (Library interface)
│   ├── main.rs              (CLI entry point)
│   ├── cli/                 (Argument parsing)
│   ├── commands/            (25 command implementations)
│   ├── git/                 (Git operations, transaction system)
│   ├── merge/               (Deep merge, layer merge)
│   └── staging/             (Index, workspace, gitignore)
└── tests/
    └── cli_basic.rs         (Basic CLI tests - parsing only)
```

### Desired Codebase Tree

```
jin/
├── Cargo.toml
├── src/                     (no changes to src/)
└── tests/
    ├── cli_basic.rs         (keep existing)
    ├── common/
    │   ├── mod.rs           (test utilities re-export)
    │   ├── fixtures.rs      (setup_test_repo, create_commit helpers)
    │   └── assertions.rs    (custom assertions for Jin state)
    ├── core_workflow.rs     (init → mode → add → commit → apply)
    ├── mode_scope_workflow.rs  (layer routing and precedence)
    ├── sync_workflow.rs     (fetch → pull → push with local remotes)
    ├── error_scenarios.rs   (conflicts, corruption, permissions)
    └── atomic_operations.rs (transaction safety, rollback)
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: Jin repository is BARE (from plan/docs/CODEBASE_PATTERNS.md:377)
// ~/.jin/ has no working directory - only object database
// Always use workspace::read_file() for reading from project working directory

// CRITICAL: Check ref existence before resolving (CODEBASE_PATTERNS.md:335)
if repo.ref_exists(&ref_path) {
    let oid = repo.resolve_ref(&ref_path)?;  // Safe
} else {
    // Initial commit, no parent
}

// CRITICAL: Tempfile must be kept in scope (research/03_tempfile_and_fixtures.md:78)
struct TestFixture {
    _tempdir: TempDir,  // Underscore prefix, but must be stored
    path: PathBuf,
}
// If tempdir dropped, directory deleted immediately

// CRITICAL: Use local filesystem remotes (research/04_git_integration_testing.md:24)
// Don't mock Git - use real Git with local bare repos
git init --bare /tmp/test_remote.git
git remote add origin /tmp/test_remote.git

// CRITICAL: Git config required for commits (research/03_tempfile_and_fixtures.md:138)
git config user.email "test@example.com"
git config user.name "Test User"

// CRITICAL: Error test ratio (research/QUICK_REFERENCE.md:14)
// 70% of tests should test error paths, not happy paths
```

## Implementation Blueprint

### Test Suite Organization

Create comprehensive integration tests following Cargo's pattern (separating logic from CLI interface):

```rust
// NO CODE CHANGES to src/ - Jin already has good separation
// All tests go in tests/ directory

// Pattern from research/05_e2e_workflow_testing.md:47
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE tests/common/mod.rs
  - IMPLEMENT: Re-export test utility modules
  - NAMING: pub mod fixtures; pub mod assertions;
  - PLACEMENT: tests/common/ (Cargo convention for shared test code)
  - PATTERN: https://doc.rust-lang.org/book/ch11-03-test-organization.html#submodules-in-integration-tests
  - CODE:
      pub mod fixtures;
      pub mod assertions;

Task 2: CREATE tests/common/fixtures.rs
  - IMPLEMENT: Test setup utilities for Git repos, Jin initialization
  - FOLLOW pattern: plan/P6M2/research/03_tempfile_and_fixtures.md:78-120
  - NAMING: setup_test_repo(), setup_jin_with_remote(), create_commit_in_repo()
  - DEPENDENCIES: tempfile::TempDir, git2::Repository
  - PLACEMENT: tests/common/fixtures.rs
  - FUNCTIONS:
      // Create isolated test repository with Jin initialized
      pub fn setup_test_repo() -> (TempDir, PathBuf);

      // Create test repo with local bare remote configured
      pub fn setup_jin_with_remote() -> (TempDir, PathBuf, PathBuf);

      // Helper to create commits in test repo
      pub fn create_commit_in_repo(repo_path: &Path, file: &str, content: &str, msg: &str);

      // Initialize Jin in test directory
      pub fn jin_init(path: &Path) -> Result<()>;
  - PATTERN: See research/03_tempfile_and_fixtures.md:78 for TempDir storage
  - GOTCHA: Must store TempDir in struct to prevent premature cleanup

Task 3: CREATE tests/common/assertions.rs
  - IMPLEMENT: Custom assertions for Jin-specific state
  - FOLLOW pattern: plan/P6M2/research/02_assert_cmd_and_predicates.md:186-213
  - NAMING: assert_file_in_layer(), assert_layer_ref_exists(), assert_staging_contains()
  - PLACEMENT: tests/common/assertions.rs
  - FUNCTIONS:
      // Assert file exists in specific layer
      pub fn assert_file_in_layer(repo: &Path, layer: &str, file: &str);

      // Assert layer ref exists and points to commit
      pub fn assert_layer_ref_exists(repo: &Path, ref_path: &str);

      // Assert staging index contains file
      pub fn assert_staging_contains(repo: &Path, file: &str);

      // Assert workspace file matches content
      pub fn assert_workspace_file(project: &Path, file: &str, content: &str);

Task 4: CREATE tests/core_workflow.rs
  - IMPLEMENT: Test complete core workflow (init → mode → add → commit → apply)
  - FOLLOW pattern: plan/P6M2/research/05_e2e_workflow_testing.md:47-91
  - TEST CASES:
      #[test] fn test_init_creates_context_and_repo()
      #[test] fn test_mode_create_and_use()
      #[test] fn test_add_files_to_mode_layer()
      #[test] fn test_commit_creates_layer_commit()
      #[test] fn test_apply_merges_to_workspace()
      #[test] fn test_complete_workflow_init_to_apply()
  - DEPENDENCIES: common::fixtures, assert_cmd::Command, predicates
  - PATTERN: Each test uses TempDir for isolation
  - VERIFICATION: File exists in workspace after apply, has correct content

Task 5: CREATE tests/mode_scope_workflow.rs
  - IMPLEMENT: Test layer routing and precedence (9-layer hierarchy)
  - FOLLOW pattern: plan/P6M2/research/command_analysis.md:237-256 (layer routing matrix)
  - TEST CASES:
      #[test] fn test_layer_routing_mode_base()
      #[test] fn test_layer_routing_mode_project()
      #[test] fn test_layer_routing_mode_scope()
      #[test] fn test_layer_routing_mode_scope_project()
      #[test] fn test_layer_precedence_higher_wins()
      #[test] fn test_mode_scope_deep_merge()
  - DEPENDENCIES: common::fixtures, common::assertions
  - PATTERN: Create files in multiple layers, verify merge precedence
  - VERIFICATION: Higher layer content overrides lower layer

Task 6: CREATE tests/sync_workflow.rs
  - IMPLEMENT: Test remote sync with local bare repos (no network)
  - FOLLOW pattern: plan/P6M2/research/04_git_integration_testing.md:18-61
  - TEST CASES:
      #[test] fn test_link_to_local_remote()
      #[test] fn test_fetch_updates_refs()
      #[test] fn test_pull_merges_changes()
      #[test] fn test_push_uploads_commits()
      #[test] fn test_sync_complete_workflow()
  - DEPENDENCIES: common::fixtures::setup_jin_with_remote()
  - PATTERN: Create local bare repo as "remote", test actual git operations
  - CRITICAL: Use file:// URLs or filesystem paths, NOT network
  - VERIFICATION: Commits exist in remote after push, local after pull

Task 7: CREATE tests/error_scenarios.rs
  - IMPLEMENT: Test error conditions and recovery (70% error test ratio)
  - FOLLOW pattern: plan/P6M2/research/06-error-recovery-testing.md:29-160
  - TEST CASES:
      #[test] fn test_handles_merge_conflict()
      #[test] fn test_handles_corrupted_staging_index()
      #[test] fn test_handles_permission_denied()
      #[test] fn test_handles_missing_mode()
      #[test] fn test_handles_invalid_layer_ref()
      #[test] fn test_repair_fixes_corruption()
  - DEPENDENCIES: common::fixtures
  - PATTERN: Create error condition, verify error reported correctly, verify recovery
  - VERIFICATION: Exit code != 0, error message is helpful, state is consistent after error

Task 8: CREATE tests/atomic_operations.rs
  - IMPLEMENT: Test transaction safety and atomicity
  - FOLLOW pattern: plan/P6M2/research/07-atomic-operations-testing.md:29-94
  - TEST CASES:
      #[test] fn test_commit_is_atomic()
      #[test] fn test_failed_commit_rolls_back()
      #[test] fn test_multi_layer_commit_atomic()
      #[test] fn test_crash_safety() // Kill process mid-commit
  - DEPENDENCIES: common::fixtures
  - PATTERN: Attempt operation, verify all-or-nothing semantics
  - VERIFICATION: Either all refs updated or none, no partial state

Task 9: UPDATE Cargo.toml (if needed)
  - VERIFY: Dev-dependencies already include assert_cmd, predicates, tempfile
  - ACTION: No changes needed (already correct)
  - VERIFICATION: cargo test --lib compiles without errors

Task 10: CREATE .github/workflows/test.yml (optional, for CI)
  - IMPLEMENT: CI matrix for multiple platforms
  - FOLLOW pattern: plan/P6M2/research/QUICK_REFERENCE.md:263-280
  - PLACEMENT: .github/workflows/test.yml
  - MATRIX: ubuntu-latest (required), windows-latest (optional), macos-latest (optional)
  - PATTERN: Run cargo test on each platform
```

### Implementation Patterns & Key Details

```rust
// Pattern 1: Test setup with TempDir isolation
// From research/03_tempfile_and_fixtures.md:78
#[test]
fn test_something() -> Result<(), Box<dyn std::error::Error>> {
    // Create isolated temp directory
    let temp = TempDir::new()?;
    let repo_path = temp.path();

    // Initialize Jin
    Command::cargo_bin("jin")?
        .arg("init")
        .current_dir(repo_path)
        .assert()
        .success();

    // Test operations
    // ...

    // Cleanup: automatic when temp dropped
    Ok(())
}

// Pattern 2: Testing with local filesystem remote
// From research/04_git_integration_testing.md:18
#[test]
fn test_sync_with_local_remote() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let local_path = temp.path().join("local");
    let remote_path = temp.path().join("remote");

    // Create bare remote
    Repository::init_bare(&remote_path)?;

    // Create local and link to remote
    fs::create_dir(&local_path)?;
    Command::cargo_bin("jin")?
        .arg("init")
        .current_dir(&local_path)
        .assert()
        .success();

    Command::cargo_bin("jin")?
        .args(&["link", remote_path.to_str().unwrap()])
        .current_dir(&local_path)
        .assert()
        .success();

    // Now test push/pull with real Git
    Ok(())
}

// Pattern 3: Error injection and recovery testing
// From research/06-error-recovery-testing.md:91
#[test]
fn test_handles_corrupted_index() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    common::fixtures::jin_init(temp.path())?;

    // Create valid state
    fs::write(temp.path().join("file.txt"), "content")?;
    Command::cargo_bin("jin")?
        .args(&["add", "file.txt"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Corrupt staging index
    fs::write(temp.path().join(".jin/staging/index.json"), "invalid json")?;

    // Verify error is handled
    Command::cargo_bin("jin")?
        .arg("status")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("corrupted").or(predicate::str::contains("invalid")));

    // Verify repair fixes it
    Command::cargo_bin("jin")?
        .arg("repair")
        .current_dir(temp.path())
        .assert()
        .success();

    // Verify status works after repair
    Command::cargo_bin("jin")?
        .arg("status")
        .current_dir(temp.path())
        .assert()
        .success();

    Ok(())
}

// Pattern 4: Multi-step workflow testing
// From research/05_e2e_workflow_testing.md:57
#[test]
fn test_complete_mode_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let project_path = temp.path();

    // Step 1: Initialize
    Command::cargo_bin("jin")?
        .arg("init")
        .current_dir(project_path)
        .assert()
        .success();

    // Step 2: Create and use mode
    Command::cargo_bin("jin")?
        .args(&["mode", "create", "testmode"])
        .assert()
        .success();

    Command::cargo_bin("jin")?
        .args(&["mode", "use", "testmode"])
        .current_dir(project_path)
        .assert()
        .success();

    // Step 3: Add file to mode
    fs::create_dir_all(project_path.join(".testmode"))?;
    fs::write(project_path.join(".testmode/config.json"), r#"{"test": true}"#)?;

    Command::cargo_bin("jin")?
        .args(&["add", ".testmode/config.json", "--mode"])
        .current_dir(project_path)
        .assert()
        .success();

    // Step 4: Commit
    Command::cargo_bin("jin")?
        .args(&["commit", "-m", "Add test config"])
        .current_dir(project_path)
        .assert()
        .success();

    // Step 5: Apply
    Command::cargo_bin("jin")?
        .arg("apply")
        .current_dir(project_path)
        .assert()
        .success();

    // Step 6: Verify file in workspace
    let config_path = project_path.join(".testmode/config.json");
    assert!(config_path.exists(), "Config file should exist after apply");
    let content = fs::read_to_string(config_path)?;
    assert!(content.contains(r#""test": true"#), "Config should have correct content");

    Ok(())
}

// Pattern 5: Layer precedence testing
#[test]
fn test_layer_precedence() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    common::fixtures::jin_init(temp.path())?;

    // Create mode
    Command::cargo_bin("jin")?
        .args(&["mode", "create", "test"])
        .assert()
        .success();

    Command::cargo_bin("jin")?
        .args(&["mode", "use", "test"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Add file to mode base (Layer 2)
    fs::write(temp.path().join("config.json"), r#"{"layer": "mode-base"}"#)?;
    Command::cargo_bin("jin")?
        .args(&["add", "config.json", "--mode"])
        .current_dir(temp.path())
        .assert()
        .success();

    Command::cargo_bin("jin")?
        .args(&["commit", "-m", "Mode base"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Add file to mode-project (Layer 5 - higher precedence)
    fs::write(temp.path().join("config.json"), r#"{"layer": "mode-project"}"#)?;
    Command::cargo_bin("jin")?
        .args(&["add", "config.json", "--mode", "--project"])
        .current_dir(temp.path())
        .assert()
        .success();

    Command::cargo_bin("jin")?
        .args(&["commit", "-m", "Mode project"])
        .current_dir(temp.path())
        .assert()
        .success();

    // Apply and verify higher layer wins
    Command::cargo_bin("jin")?
        .arg("apply")
        .current_dir(temp.path())
        .assert()
        .success();

    let content = fs::read_to_string(temp.path().join("config.json"))?;
    assert!(content.contains(r#""layer": "mode-project""#),
            "Mode-project (Layer 5) should override mode-base (Layer 2)");

    Ok(())
}
```

### Integration Points

```yaml
EXISTING_CODE:
  - pattern: tests/cli_basic.rs uses Command::cargo_bin("jin")
  - keep: All existing basic tests (CLI parsing validation)
  - extend: Add integration tests in separate files

DEV_DEPENDENCIES:
  - already_present: assert_cmd = "2.0"
  - already_present: predicates = "3.0"
  - already_present: tempfile = "3.0"
  - add if needed: git2 = "0.19" (if not in main dependencies)

NO_CODE_CHANGES:
  - src/: No changes required
  - Jin already has good lib/binary separation
  - Commands already implemented and working
```

## Validation Loop

### Level 1: Syntax & Build (Immediate Feedback)

```bash
# After creating each test file
cargo test --test <test_file_name> --no-run

# Expected: Compiles without errors
# If errors: Fix compilation before proceeding
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run individual test files
cargo test --test core_workflow -- --nocapture
cargo test --test mode_scope_workflow -- --nocapture
cargo test --test sync_workflow -- --nocapture
cargo test --test error_scenarios -- --nocapture
cargo test --test atomic_operations -- --nocapture

# Expected: All tests in file pass
# If failing: Debug root cause, fix implementation or test
```

### Level 3: Integration Test Suite (Full Validation)

```bash
# Run all integration tests
cargo test --tests

# Run in parallel (default) to verify isolation
cargo test --tests -- --test-threads=$(nproc)

# Run sequentially if debugging
cargo test --tests -- --test-threads=1

# Expected: All integration tests pass
# If failing: Fix isolation issues or implementation bugs
```

### Level 4: Reliability Validation (Flakiness Detection)

```bash
# Run tests 10x consecutively to detect flakiness
for i in {1..10}; do
    echo "Run $i"
    cargo test --tests || exit 1
done

# Expected: All 10 runs pass without failures
# If flaky: Fix timing issues, improve isolation, remove sleeps
```

### Level 5: Performance Validation

```bash
# Time full test suite
time cargo test --tests

# Expected: Completes in <5 minutes
# If slow: Profile slow tests, use local repos not network, optimize setup
```

### Level 6: Coverage Validation (Optional)

```bash
# Install cargo-tarpaulin if not present
# cargo install cargo-tarpaulin

# Run coverage analysis
cargo tarpaulin --out Html --output-dir coverage

# Expected: >70% coverage for CLI commands
# Open coverage/index.html to view report
```

## Final Validation Checklist

### Technical Validation

- [ ] All tests compile: `cargo test --tests --no-run`
- [ ] All tests pass: `cargo test --tests`
- [ ] No flaky tests: 10 consecutive runs successful
- [ ] Tests run in parallel without interference: `--test-threads=$(nproc)`
- [ ] Test suite completes in <5 minutes
- [ ] No linting errors: `cargo clippy --tests`

### Feature Validation

- [ ] Core workflow tested: init → mode → add → commit → apply
- [ ] Mode/scope routing tested: All 9 layers
- [ ] Sync workflow tested: link → fetch → pull → push
- [ ] Error scenarios tested: conflicts, corruption, permissions
- [ ] Atomic operations tested: commit atomicity, rollback
- [ ] All 25 CLI commands have integration test coverage

### Code Quality Validation

- [ ] Test code follows Rust conventions
- [ ] Tests use TempDir for isolation (no shared state)
- [ ] Tests use local filesystem remotes (no network)
- [ ] Tests include helpful assertions and error messages
- [ ] Test names clearly describe what is being tested
- [ ] Common utilities extracted to fixtures.rs

### Documentation

- [ ] Tests serve as executable specification of behavior
- [ ] Test names document expected behavior
- [ ] Error messages help debug failures

---

## Anti-Patterns to Avoid

Based on extensive research (plan/P6M2/research/):

- ❌ Don't mock Git commands - use real Git with local repos
- ❌ Don't use network - use local filesystem remotes
- ❌ Don't share state between tests - use TempDir per test
- ❌ Don't hardcode paths - use temp directories
- ❌ Don't skip error testing - 70% of bugs are in error paths
- ❌ Don't ignore cleanup - tempfile handles it automatically
- ❌ Don't use timing/sleeps - causes flaky tests
- ❌ Don't test only happy paths - error paths are critical

## Confidence Score

**Confidence Score: 9/10** for one-pass implementation success

**Reasoning**:
1. ✓ Comprehensive research (8,414 lines, 25+ sources)
2. ✓ Real-world patterns validated (Cargo, Git, Kubernetes)
3. ✓ Existing test infrastructure in place (assert_cmd, predicates, tempfile)
4. ✓ Clear implementation tasks with dependencies
5. ✓ Complete code patterns provided
6. ✓ Validation gates at multiple levels
7. ✓ Anti-patterns documented to avoid
8. ✓ Command analysis completed (all 25 commands)
9. ⚠ Partial uncertainty: Actual jin command implementations may have undocumented behavior
10. ⚠ Some tests may need adjustments based on actual command output format

**Mitigation**: Start with core_workflow.rs tests first, validate against actual command behavior, adjust patterns as needed for remaining tests.

---

## Research Documentation

All research findings are documented in `plan/P6M2/research/`:

- **START_HERE.md**: Quick overview
- **QUICK_REFERENCE.md**: Fast lookup and decision trees
- **command_analysis.md**: All 25 Jin commands analyzed
- **INDEX.md**: Complete research index
- Plus 15+ detailed research documents on specific topics

Total research: 8,414 lines across 21 files
Sources: 25+ authoritative references
Real-world validation: Cargo, Git, Kubernetes, Pika

---

**PRP Created**: December 27, 2025
**Research Completed**: Comprehensive (8,414 lines)
**Implementation Readiness**: High (9/10 confidence)
**Expected Implementation Time**: 2-4 hours for experienced Rust developer
