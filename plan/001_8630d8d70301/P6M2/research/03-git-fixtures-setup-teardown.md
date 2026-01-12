# Setup and Teardown Patterns for Git Repositories in Tests

## Overview
Proper setup and teardown of test fixtures is critical for reliable, isolated Git repository testing. This document covers patterns for initializing test repositories and cleaning up resources.

## Fixture Lifecycle Pattern

```
Test Lifecycle:
┌─────────────────────────────────────┐
│  Setup Phase                        │
│  ├─ Create temp directory           │
│  ├─ Initialize repository           │
│  ├─ Configure git user              │
│  ├─ Create initial commits/branches │
│  └─ Prepare test state              │
├─────────────────────────────────────┤
│  Test Execution                     │
│  ├─ Run test code                   │
│  └─ Verify state changes            │
├─────────────────────────────────────┤
│  Teardown Phase                     │
│  ├─ Verify cleanup occurs           │
│  ├─ Remove temp directory           │
│  ├─ Close file handles              │
│  └─ Release resources               │
└─────────────────────────────────────┘
```

## Key Principles

1. **Isolation**: Each test operates on independent repository
2. **Determinism**: Same setup produces same state every time
3. **Cleanup**: Resources always cleaned regardless of test outcome
4. **Idempotency**: Setup code can safely run multiple times
5. **Documentation**: Setup code clearly shows test prerequisites

## Temporary Directory Patterns

### 1. Language-Specific Temp Directory Utilities

**Go Pattern** (`testing.TempDir()`):
```go
func TestGitOperations(t *testing.T) {
    dir := t.TempDir()  // Auto-cleanup after test
    repo := InitializeGitRepo(dir)

    // Test code
    // Cleanup happens automatically
}
```

**Advantages**:
- Automatic cleanup handled by test framework
- Works across platforms (Windows, Linux, macOS)
- No manual file deletion code needed
- Nested temp directories supported

**Rust Pattern** (using `tempfile` crate):
```rust
#[test]
fn test_git_operations() {
    let temp_dir = TempDir::new().unwrap();
    let repo = initialize_git_repo(temp_dir.path());

    // Test code
    // Auto-cleanup when temp_dir is dropped
}
```

**Python Pattern** (using `pytest` fixtures):
```python
@pytest.fixture
def temp_repo(tmp_path):
    # Setup
    repo = initialize_git_repo(tmp_path)
    yield repo
    # Teardown happens automatically after yield
```

### 2. Manual Cleanup with Guarantees

**Pattern**: Cleanup in `finally` or `defer` blocks

```
Try-Finally pattern:
try {
    // Setup
    tempDir = createTempDir()
    repo = initializeRepo(tempDir)

    // Test code
    performTest()
} finally {
    // Cleanup always runs, even if test fails
    removeTempDir(tempDir)
}
```

**Benefits**:
- Handles test failures gracefully
- Cleanup always executes
- Exceptions don't prevent cleanup
- Works across different test frameworks

## Repository Initialization Patterns

### 1. Minimal Git Setup

**Pattern**: Create repository with minimal configuration

```bash
#!/usr/bin/env bash

setup_git_repo() {
    local repo_dir=$1

    cd "$repo_dir"

    # Initialize repository
    git init

    # Configure user (required for commits)
    git config user.email "test@example.com"
    git config user.name "Test User"

    # Create initial commit (many tests expect this)
    touch .gitkeep
    git add .gitkeep
    git commit -m "Initial commit"
}
```

**Configuration items**:
- `user.email`: Required for commits
- `user.name`: Required for commits
- `core.filemode`: Set for consistent behavior on Windows
- `core.safecrlf`: Control line ending handling

### 2. Create Test Fixtures with Specific State

**Pattern 1: Files in repository**

```bash
create_repo_with_files() {
    local repo_dir=$1

    # Initialize
    git init "$repo_dir"
    cd "$repo_dir"
    git config user.email "test@example.com"
    git config user.name "Test User"

    # Create file structure
    mkdir -p src/lib
    echo "fn main() {}" > src/lib/mod.rs
    echo "pub fn hello() {}" > src/lib/hello.rs

    # Commit
    git add .
    git commit -m "Add library files"
}
```

**Pattern 2: Multiple commits**

```bash
create_repo_with_history() {
    local repo_dir=$1

    initialize_repo "$repo_dir"

    # Create commit 1
    echo "version = 1" > version.txt
    git add version.txt
    git commit -m "Version 1"

    # Create commit 2
    echo "version = 2" > version.txt
    git add version.txt
    git commit -m "Version 2"

    # Create branch
    git checkout -b feature/branch
    echo "feature" > feature.txt
    git add feature.txt
    git commit -m "Add feature"
}
```

**Pattern 3: GitLab-style custom fixtures**

```ruby
# Ruby with GitLab testing utilities
let(:project) do
  create(
    :project, :custom_repo,
    files: {
      'README.md'          => 'Project description',
      'src/main.rs'        => 'fn main() {}',
      'tests/test.rs'      => '#[test] fn test() {}',
      'Cargo.toml'         => '[package] name = "myapp"'
    }
  )
end
```

### 3. Remote Repository Setup

**Pattern**: Create bare repository as test remote

```bash
setup_test_remote() {
    local remote_dir=$1

    # Create bare repository (no working directory)
    git init --bare "$remote_dir"

    # Optional: Set git config for custom behavior
    # git config -f "$remote_dir/config" receive.denyNonFastforwards false
}

setup_local_with_remote() {
    local repo_dir=$1
    local remote_dir=$2

    # Create local repo
    git init "$repo_dir"
    cd "$repo_dir"
    git config user.email "test@example.com"
    git config user.name "Test User"

    # Add remote pointing to local bare repo
    git remote add origin "$remote_dir"

    # Create and push
    echo "content" > file.txt
    git add file.txt
    git commit -m "Initial"
    git push -u origin main
}
```

## Pytest Fixtures for Git Testing

### Basic Fixture Pattern

```python
import pytest
import tempfile
import os
import subprocess

@pytest.fixture
def git_repo(tmp_path):
    """Fixture providing initialized Git repository"""
    repo_path = tmp_path / "test_repo"
    repo_path.mkdir()

    # Setup
    os.chdir(repo_path)
    subprocess.run(["git", "init"], check=True)
    subprocess.run(["git", "config", "user.email", "test@example.com"], check=True)
    subprocess.run(["git", "config", "user.name", "Test User"], check=True)

    yield repo_path

    # Teardown happens automatically
```

### Fixture with Remote

```python
@pytest.fixture
def git_repo_with_remote(tmp_path):
    """Fixture with local and remote repositories"""
    repo_path = tmp_path / "local"
    remote_path = tmp_path / "remote"

    # Setup remote
    remote_path.mkdir()
    subprocess.run(["git", "init", "--bare", str(remote_path)], check=True)

    # Setup local
    repo_path.mkdir()
    os.chdir(repo_path)
    subprocess.run(["git", "init"], check=True)
    subprocess.run(["git", "config", "user.email", "test@example.com"], check=True)
    subprocess.run(["git", "config", "user.name", "Test User"], check=True)
    subprocess.run(["git", "remote", "add", "origin", str(remote_path)], check=True)

    # Create initial commit
    (repo_path / "README.md").write_text("Initial commit")
    subprocess.run(["git", "add", "README.md"], check=True)
    subprocess.run(["git", "commit", "-m", "Initial"], check=True)
    subprocess.run(["git", "push", "-u", "origin", "main"], check=True)

    yield {
        "local": repo_path,
        "remote": remote_path
    }

    # Cleanup automatic
```

### Scoped Fixtures

```python
@pytest.fixture(scope="function")  # New repo per test
def git_repo_function_scoped(tmp_path):
    pass

@pytest.fixture(scope="module")    # Shared across module
def git_repo_module_scoped(tmp_path):
    pass

@pytest.fixture(scope="session")   # Shared across session
def git_repo_session_scoped(tmp_path):
    pass
```

## Bats Test Framework Patterns

### Basic Setup/Teardown

```bash
#!/usr/bin/env bats

setup() {
    # Create temp directory
    export TEST_REPO="${BATS_TMPDIR}/git_test_$$"
    mkdir -p "$TEST_REPO"
    cd "$TEST_REPO"

    # Initialize repo
    git init
    git config user.email "test@example.com"
    git config user.name "Test User"
}

teardown() {
    # Cleanup
    rm -rf "$TEST_REPO"
}

@test "verify commit creation" {
    echo "test" > file.txt
    git add file.txt
    git commit -m "test"

    [ "$(git log --oneline | wc -l)" -eq 1 ]
}
```

### Multi-Repository Setup

```bash
setup() {
    export LOCAL_REPO="${BATS_TMPDIR}/local_$$"
    export REMOTE_REPO="${BATS_TMPDIR}/remote_$$"

    # Setup remote
    mkdir -p "$REMOTE_REPO"
    cd "$REMOTE_REPO"
    git init --bare

    # Setup local
    mkdir -p "$LOCAL_REPO"
    cd "$LOCAL_REPO"
    git init
    git config user.email "test@example.com"
    git config user.name "Test User"
    git remote add origin "$REMOTE_REPO"
}

teardown() {
    rm -rf "$LOCAL_REPO" "$REMOTE_REPO"
}
```

## Common Setup Mistakes and Solutions

| Problem | Cause | Solution |
|---------|-------|----------|
| **Commits fail** | Missing git config | Set user.email and user.name |
| **Tests interfere** | Shared state | Use `t.TempDir()` per test |
| **Cleanup fails** | No error handling | Use try-finally or fixtures with cleanup |
| **Slow tests** | Large fixtures | Reuse fixtures at higher scope |
| **Flaky tests** | Timing issues | Ensure setup fully completes before test |
| **Platform issues** | CRLF line endings | Configure `core.safecrlf` or `core.autocrlf` |
| **Permission errors** | File locking | Close file handles in teardown |

## Best Practices Checklist

- [ ] Use language-specific temp directory utilities
- [ ] Always clean up in finally blocks or equivalent
- [ ] Document what state setup() creates
- [ ] Make setup idempotent (can run multiple times safely)
- [ ] Use minimal git configuration
- [ ] Test cleanup itself (verify files are deleted)
- [ ] Use higher scope for expensive setups
- [ ] Test isolation first (no cross-test interference)
- [ ] Handle both success and failure paths
- [ ] Make error messages clear in setup failures

## References

- [Pytest: Fixtures - setup and teardown](https://docs.pytest.org/en/stable/how-to/fixtures.html)
- [GitLab: Testing best practices](https://docs.gitlab.com/ee/development/testing_guide/best_practices.html)
- [Bats: Testing Bash scripts](https://github.com/bats-core/bats-core)
- [Go: Testing package documentation](https://golang.org/pkg/testing/)
- [Python: unittest.TestCase.setUp/tearDown](https://docs.python.org/3/library/unittest.html)
