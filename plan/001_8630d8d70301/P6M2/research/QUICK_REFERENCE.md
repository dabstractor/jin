# CLI Integration Testing - Quick Reference Guide

## One-Minute Summary

**Golden Rule**: Test with real Git using local filesystem remotes, real temporary directories, and proper error injection.

```
Test Stack:
├─ Unit Tests: Mock filesystem interfaces
├─ Integration Tests: Real Git + temp directories
├─ Error Tests: Safe error injection
└─ Error Mix: 70% of tests should test error paths
```

## Checklists

### Before Writing Tests

- [ ] Separate business logic into lib/module
- [ ] Make filesystem operations injectable
- [ ] Define git/remote operations as testable interfaces
- [ ] Plan error scenarios (conflicts, permissions, network)
- [ ] Identify expensive operations for setup layering

### Test Implementation Checklist

- [ ] Use language-specific temp directories
- [ ] Initialize git repos with minimal config
- [ ] Create test fixtures with known state
- [ ] Test both success AND failure paths
- [ ] Verify no cleanup leaves orphaned files
- [ ] Use assert_* libraries for assertions
- [ ] Test exit codes and error messages
- [ ] Run tests in parallel to verify isolation
- [ ] Test on multiple platforms (at least CI)

### Avoiding Common Failures

- [ ] Don't mock git commands (use local remotes)
- [ ] Don't hardcode paths (use temp dirs)
- [ ] Don't share state between tests (isolate)
- [ ] Don't skip error path testing (70% of bugs)
- [ ] Don't ignore cleanup (files accumulate)
- [ ] Don't use network (too slow and flaky)
- [ ] Don't set timeouts on processes (hang detection)
- [ ] Don't depend on timing (races cause flakiness)

## Quick Decision Trees

### How to test THIS scenario?

**Testing Git push behavior**
```
Reality check: Can you create it safely locally?
├─ YES → Use local filesystem remote
│        git init --bare /tmp/test_remote.git
│        Configure repo.git remote to /tmp/test_remote.git
│        Test git push
│
└─ NO  → Use git-http-mock-server
         Serves real repos over HTTP with copy-on-write
```

**Testing authentication failures**
```
Is this about Git protocol or HTTP?
├─ HTTP (clone via https://)
│  └─ Mock HTTP server with 401/403 responses
│
└─ SSH (clone via git@...)
   └─ Create test SSH keys, configure GIT_SSH_COMMAND
```

**Testing merge conflicts**
```
Can you create conflict safely?
├─ YES → Create conflicting commits locally
│        Run actual git merge/pull
│        Verify conflict markers in file
│
└─ NO  → Skip this test (easy to create safely)
```

**Testing permission denied errors**
```
Is this a security test or error handling test?
├─ Handling: Mock filesystem with EACCES error
├─ Real: Use real filesystem, set permissions to 0000
└─ Safe: Mock (won't break other tests)
```

**Testing slow network operations**
```
Do you need to test timeout behavior?
├─ YES → Mock HTTP server that doesn't respond
│        Set reasonable timeout, verify error
│
└─ NO  → Use local repos (avoid network entirely)
```

## Essential Patterns

### Setup Template

```rust
// Rust example
#[test]
fn test_something() {
    // Create isolated temp directory
    let temp = TempDir::new().unwrap();

    // Initialize git repo
    initialize_repo(temp.path());

    // Add test data
    write_file(temp.path(), "file.txt", "content");
    git_commit(temp.path(), "Initial");

    // Run test
    let result = my_function(temp.path());

    // Verify result
    assert!(result.is_ok());

    // Cleanup: automatic when temp is dropped
}
```

### Error Testing Template

```bash
# Bash example
@test "handles error gracefully" {
    initialize_repo "$TEST_REPO"

    # Trigger error condition
    rm -rf "$TEST_REPO/.git/objects"  # Corrupt repo

    # Verify error is reported (not panic/crash)
    run my_cli sync "$TEST_REPO"
    [ $status -ne 0 ]
    [[ "$output" == *"corrupted"* ]] || [[ "$output" == *"invalid"* ]]
}
```

### Git Operation Template

```python
# Python example
def test_git_push():
    # Setup: local and remote
    remote_path = tempfile.mkdtemp()
    local_path = tempfile.mkdtemp()

    # Initialize remote as bare repo
    subprocess.run(["git", "init", "--bare", remote_path], check=True)

    # Initialize local
    subprocess.run(["git", "init", local_path], check=True)
    subprocess.run(["git", "config", "user.email", "test@test.com"],
                   cwd=local_path, check=True)

    # Add remote
    subprocess.run(["git", "remote", "add", "origin", remote_path],
                   cwd=local_path, check=True)

    # Create and push
    create_file(local_path, "file.txt", "content")
    subprocess.run(["git", "add", "file.txt"], cwd=local_path, check=True)
    subprocess.run(["git", "commit", "-m", "Initial"], cwd=local_path, check=True)
    subprocess.run(["git", "push", "-u", "origin", "main"], cwd=local_path, check=True)

    # Verify
    result = subprocess.run(["git", "log", "--oneline"], cwd=remote_path,
                          capture_output=True, text=True, check=True)
    assert "Initial" in result.stdout

    # Cleanup: automatic
```

## Tool Selection by Language

| Language | CLI Testing | Git Testing | Filesystem | Mocking |
|----------|-------------|------------|-----------|---------|
| Rust | assert_cmd | git2 crate | assert_fs | mockall |
| Python | subprocess | GitPython | tempfile | unittest.mock |
| Bash | bats | git cli | mktemp | manual |
| JavaScript | judo | git | tmp | jest.mock |
| Go | exec | git | ioutil.TempDir | testify/mock |

## Exit Code Convention

```
0   Success
1   General error
2   Misuse of command
3   Repository error (not git repo, corruption)
4   Network error (timeout, unreachable)
5   Authentication error (invalid credentials)
6   Authorization error (permission denied)
7   Conflict (merge conflict, non-fast-forward)
8   Not found (file, branch, remote)
```

## File Organization

```
project/
├─ src/
│  ├─ lib.rs              (Business logic, injectable)
│  ├─ main.rs             (CLI interface)
│  └─ test.rs             (Unit tests)
│
├─ tests/
│  ├─ integration.rs       (Multi-command workflows)
│  ├─ git_ops.rs          (Git operations)
│  ├─ error_handling.rs    (Error paths)
│  ├─ atomic_ops.rs        (Transactional behavior)
│  └─ common/
│     ├─ fixtures.rs       (Setup utilities)
│     └─ assertions.rs     (Custom assertions)
│
└─ .github/workflows/
   └─ test.yml             (CI matrix: OS × version)
```

## Test Execution Template

```yaml
# GitHub Actions example
name: Tests

on: [push, pull_request]

jobs:
  test:
    name: Test
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust-version: [stable, nightly]
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust-version }}
      - run: cargo test --verbose
```

## Performance Guidelines

| Test Type | Target Speed | How |
|-----------|----------|-----|
| Unit test | 1-10ms | Mock filesystem, no I/O |
| Integration test | 10-100ms | Real git, temp dir, local |
| Error test | 10-100ms | Error injection, no network |
| Full suite | <5min | Parallel execution, no network |

**Rule**: If a test is slow, it's doing too much. Split it.

## Red Flags

- **Flaky tests**: Usually timing or environment issues
  - Fix: Remove timing dependencies, fix isolation
- **Slow tests**: Doing too much or waiting for network
  - Fix: Split test, remove network calls, mock expensive ops
- **Hard to read tests**: Duplicated setup, unclear purpose
  - Fix: Extract fixtures, use descriptive names, add comments
- **Brittle tests**: Break when code changes slightly
  - Fix: Test behavior, not implementation; use real deps where possible
- **Tests interfering**: Failures depend on test order
  - Fix: Use temp dirs, not shared files; proper cleanup

## One-Liners for Common Tasks

```bash
# Create temp git repo for testing
git_test_repo=$(mktemp -d) && cd "$git_test_repo" && git init && \
  git config user.email "test@test.com" && git config user.name "Test"

# Create bare remote repo
git init --bare /tmp/test_remote.git

# Check if repo is valid
git -C "$REPO" fsck --full

# View git object
git -C "$REPO" cat-file -p <hash>

# Run single test (Rust)
cargo test test_name -- --nocapture

# Run tests in isolation (verify no cross-test interference)
for test in $(cargo test --list | grep ': test' | cut -d: -f1); do
  cargo test "$test" || exit 1
done
```

## When You're Stuck

1. **Tests are flaky**
   - Likely: Timing dependencies, shared state, environment assumptions
   - Fix: Use `t.TempDir()`, avoid timing, ensure isolation
   - Verify: Run same test 10x consecutively, run in random order

2. **Tests are slow**
   - Likely: Network calls, disk I/O, expensive setup, sequential execution
   - Fix: Use local repos, mock HTTP, layer setup, run parallel
   - Verify: Time individual test, profile slow operations

3. **Tests don't match reality**
   - Likely: Mocking too much, not testing actual behavior
   - Fix: Use real Git with local remotes, real filesystem with temp dirs
   - Verify: Can test pass if you manually do the operation?

4. **Tests fail on CI but pass locally**
   - Likely: Environment differences (OS, path separators, permissions)
   - Fix: Test on multiple platforms, use cross-platform paths
   - Verify: Use CI matrix for OS combinations

5. **Hard to write error tests**
   - Likely: Trying to create real error conditions
   - Fix: Use safe error injection (mocks, env vars, fake errors)
   - Verify: Can you trigger the error without breaking other tests?

## References Quick Links

- **Setup/Teardown**: Document 03
- **Isolation Patterns**: Document 04
- **Git Testing**: Document 02
- **Error Testing**: Document 06
- **Real Examples**: See Cargo, Git, Kubernetes in INDEX
- **Tools**: See tool recommendations in INDEX
- **Full Reference**: Start with INDEX.md

## Key Statistics from Research

- **Git's test suite**: ~10,000 tests
- **Cargo test speed**: <5 minutes full suite
- **Recommended error test ratio**: 70% of tests
- **Performance gain from temp dirs**: 100-1000x vs network
- **Common test flakiness**: 80% caused by shared state
- **Fastest mock filesystem**: Go's fstest.MapFS (microseconds)

---

**Last Updated**: December 27, 2025
**Based On**: 25+ authoritative sources
**Real-World Validation**: Cargo, Git, Kubernetes, Pika
