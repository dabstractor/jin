# Testing Error Conditions and Recovery Scenarios

## Overview
Robust CLI tools must handle errors gracefully and recover when possible. This document covers strategies for testing error paths, recovery mechanisms, and edge cases in Git-based workflows.

## Testing Philosophy

**Key principle**: Errors are first-class test cases, not afterthoughts.

```
Test coverage should include:
├─ Happy path (10% of tests)
├─ Error conditions (70% of tests)
├─ Recovery scenarios (15% of tests)
└─ Edge cases (5% of tests)
```

## Error Categories

### 1. System-Level Errors

**Type**: Issues caused by the operating system or environment

```
Categories:
├─ Permission denied (EACCES)
├─ Disk full (ENOSPC)
├─ File not found (ENOENT)
├─ Not a directory (ENOTDIR)
├─ Too many files open (EMFILE)
├─ Invalid argument (EINVAL)
└─ Device/resource busy (EBUSY)
```

**Testing approach**:
```rust
#[test]
fn test_handles_permission_denied() {
    let temp = TempDir::new().unwrap();

    // Create directory with no read permissions
    let no_read = temp.path().join("no_read");
    fs::create_dir(&no_read).unwrap();
    #[cfg(unix)]
    {
        use std::fs::Permissions;
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&no_read, Permissions::from_mode(0o000)).unwrap();
    }

    // Verify our code handles this gracefully
    let result = process_directory(&no_read);
    match result {
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            // Good - error is reported correctly
        }
        Ok(_) => panic!("Should have failed with permission denied"),
        Err(e) => panic!("Wrong error type: {}", e),
    }
}
```

### 2. Git-Specific Errors

**Type**: Issues from Git operations or repository state

```
Categories:
├─ Merge conflicts (conflicting changes)
├─ Rejected push (non-fast-forward)
├─ Authentication failures (invalid credentials)
├─ Repository corruption (bad objects)
├─ Detached HEAD state (not on branch)
├─ Dirty working directory (uncommitted changes)
├─ Remote not found
└─ Branch divergence
```

**Testing approach**:
```bash
#!/usr/bin/env bats

setup() {
    export LOCAL_REPO=$(mktemp -d)
    export REMOTE_REPO=$(mktemp -d)

    cd "$REMOTE_REPO"
    git init --bare

    cd "$LOCAL_REPO"
    git init
    git config user.email "test@example.com"
    git config user.name "Test"
    git remote add origin "$REMOTE_REPO"
}

@test "handles merge conflict error" {
    # Create conflicting changes
    echo "local" > file.txt
    git add file.txt
    git commit -m "local change"

    cd "$REMOTE_REPO"
    git clone . "$REMOTE_REPO/temp"
    cd "$REMOTE_REPO/temp"
    echo "remote" > file.txt
    git add file.txt
    git commit -m "remote change"
    git push origin main

    # Try to pull - should fail with merge conflict
    cd "$LOCAL_REPO"
    run git pull origin main
    [ $status -ne 0 ]
    [[ "$output" == *"conflict"* ]] || [[ "$output" == *"Conflict"* ]]
}

@test "handles non-fast-forward rejection" {
    # Create and push initial commit
    echo "initial" > file.txt
    git add file.txt
    git commit -m "initial"
    git push -u origin main

    # Force local history to be different
    git reset --hard HEAD~1
    echo "forced" > file.txt
    git add file.txt
    git commit -m "forced change"

    # Try to push without force - should be rejected
    run git push origin main
    [ $status -ne 0 ]
}
```

### 3. Network Errors

**Type**: Issues from remote operations

```
Categories:
├─ Connection refused (remote down)
├─ Connection timeout (network lag)
├─ DNS resolution failure
├─ SSL/TLS certificate errors
├─ Authentication required (401/403)
├─ Server errors (500+)
└─ Rate limiting
```

**Testing approach**:
```python
import pytest
import subprocess
import time
from unittest.mock import patch

def test_handles_connection_timeout():
    """Verify graceful timeout handling"""
    # Use a non-routable IP that will timeout
    result = subprocess.run(
        ["git", "clone", "http://192.0.2.1/repo.git"],  # TEST-NET-1, guaranteed non-routable
        timeout=5,
        capture_output=True
    )
    assert result.returncode != 0
    assert "timeout" in result.stderr.lower() or "connection" in result.stderr.lower()

def test_handles_404_not_found():
    """Verify handling of non-existent repository"""
    # Try to clone non-existent repo from public server
    result = subprocess.run(
        ["git", "clone", "https://github.com/definitely-does-not-exist-12345/repo.git"],
        timeout=10,
        capture_output=True
    )
    assert result.returncode != 0
    assert "not found" in result.stderr.lower() or "repository does not exist" in result.stderr.lower()
```

## Recovery Scenario Testing

### 1. Rollback Recovery

**Pattern**: Verify ability to undo failed operations

```rust
#[test]
fn test_rollback_on_apply_failure() {
    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();

    // Setup repo with initial state
    git_init(repo_path);
    create_file(repo_path, "file.txt", "original content");
    git_commit(repo_path, "Initial");

    // Save state before operation
    let initial_commit = git_current_commit(repo_path);

    // Attempt operation that will fail (apply bad patch)
    let patch_content = "--- file.txt\n+++ file.txt\n@@ -1 +1 @@\n-original\n+modified\n";
    let result = apply_patch(repo_path, patch_content);

    // Should fail
    assert!(result.is_err());

    // Verify state was rolled back
    assert_eq!(git_current_commit(repo_path), initial_commit);
    assert_eq!(read_file(repo_path, "file.txt"), "original content");
}
```

### 2. Incremental Recovery

**Pattern**: Resume operations from checkpoint

```go
type Operation struct {
    id    string
    steps []Step
}

type Step struct {
    id     string
    action func() error
}

func (op *Operation) Execute(checkpoint string) error {
    // Start from checkpoint, not from beginning
    startIdx := op.findCheckpoint(checkpoint)

    for i := startIdx; i < len(op.steps); i++ {
        step := op.steps[i]
        if err := step.action(); err != nil {
            // Save checkpoint before failing
            op.saveCheckpoint(step.id)
            return err
        }
    }
    return nil
}

func TestIncrementalRecovery(t *testing.T) {
    // Create operation with multiple steps
    op := &Operation{
        id: "test-op",
        steps: []Step{
            {id: "step1", action: func() error { return nil }},
            {id: "step2", action: func() error { return errors.New("fail") }},
            {id: "step3", action: func() error { return nil }},
        },
    }

    // First attempt fails at step2
    err := op.Execute("")
    assert.Error(t, err)

    // Resume from checkpoint - should skip step1 and restart at step2
    err = op.Execute("step2")
    // Verify step2 is retried
}
```

### 3. Cleanup and Verification

**Pattern**: Verify recovery doesn't leave orphaned state

```bash
@test "cleanup after failed operation leaves no orphans" {
    # Start operation
    my_cli sync --repo "$TEST_REPO" --remote-fail &
    PID=$!

    # Give it time to partially complete
    sleep 0.5

    # Kill it mid-operation
    kill $PID
    wait $PID 2>/dev/null || true

    # Verify no orphaned lock files
    [ ! -f "$TEST_REPO/.lock" ]
    [ ! -f "$TEST_REPO/.tmp-*" ]

    # Verify repository is still valid
    run git -C "$TEST_REPO" status
    [ $status -eq 0 ]
}
```

## Error Injection Techniques

### 1. Mock Injection

**Pattern**: Inject errors at specific points

```rust
pub trait GitProvider {
    fn push(&self) -> Result<()>;
    fn pull(&self) -> Result<()>;
}

pub struct RealGit;

impl GitProvider for RealGit {
    fn push(&self) -> Result<()> {
        // Real implementation
    }
}

pub struct FailingGit {
    fail_on: String,
}

impl GitProvider for FailingGit {
    fn push(&self) -> Result<()> {
        if self.fail_on == "push" {
            Err("Push failed".into())
        } else {
            // Delegate to real implementation
            RealGit.push()
        }
    }
}

#[test]
fn test_handles_push_failure() {
    let git = FailingGit {
        fail_on: "push".to_string(),
    };

    let result = sync_with_remote(&git);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Push failed"));
}
```

### 2. Conditional Failure

**Pattern**: Fail only for specific conditions

```go
type ConditionalFailFS struct {
    underlying fs.FS
    failOn    map[string]bool
}

func (cf *ConditionalFailFS) ReadFile(name string) ([]byte, error) {
    if cf.failOn[name] {
        return nil, syscall.EACCES
    }
    return fs.ReadFile(cf.underlying, name)
}

func TestReadFileWithPermission(t *testing.T) {
    mockFS := &ConditionalFailFS{
        underlying: os.DirFS("."),
        failOn: map[string]bool{
            "secret.key": true,  // This file fails with permission denied
        },
    }

    data, err := mockFS.ReadFile("secret.key")
    assert.Error(t, err)
    assert.Nil(t, data)
}
```

### 3. Failure Injection via Environment

**Pattern**: Control failures through environment variables or configs

```bash
#!/usr/bin/env bash

# In test:
export JIN_TEST_FAIL_GIT_CLONE=true
export JIN_TEST_FAIL_ON_FILE="Cargo.toml"

# Code in application:
if [[ "${JIN_TEST_FAIL_GIT_CLONE:-false}" == "true" ]]; then
    echo "Test failure: git clone" >&2
    exit 1
fi
```

## Specific Error Test Cases

### Merge Conflict Handling

```bash
@test "resolves conflicts with ours strategy" {
    # Setup: create conflicting changes
    create_conflicting_commits "$LOCAL_REPO" "$REMOTE_REPO"

    # Execute sync with conflict resolution
    run my_cli sync --repo "$LOCAL_REPO" --conflict-strategy ours
    [ $status -eq 0 ]

    # Verify: local version was kept
    [ "$(git -C "$LOCAL_REPO" show HEAD:file.txt)" == "local content" ]
}

@test "aborts on unresolvable conflicts" {
    # Setup: create truly conflicting changes
    create_conflicting_commits "$LOCAL_REPO" "$REMOTE_REPO"

    # Execute without strategy
    run my_cli sync --repo "$LOCAL_REPO"
    [ $status -ne 0 ]
    [[ "$output" == *"conflict"* ]]

    # Verify: working directory is unchanged (abort preserved state)
    run git -C "$LOCAL_REPO" status --porcelain
    [ -z "$output" ]
}
```

### Corrupted Repository Handling

```bash
@test "detects corrupted repository" {
    # Create valid repo
    initialize_repo "$TEST_REPO"

    # Corrupt it: remove object file
    objects_dir="$TEST_REPO/.git/objects"
    rm -f "$objects_dir"/??/*

    # Verify corruption detection
    run my_cli verify "$TEST_REPO"
    [ $status -ne 0 ]
}

@test "attempts repair of corrupted repository" {
    # Create and corrupt repo
    initialize_repo "$TEST_REPO"
    corrupt_repo "$TEST_REPO"

    # Attempt repair
    run my_cli repair "$TEST_REPO"

    if [ $status -eq 0 ]; then
        # If repair succeeds, verify repo is valid
        run git -C "$TEST_REPO" fsck
        [ $status -eq 0 ]
    else
        # If repair fails, error message should be clear
        [[ "$output" == *"cannot repair"* ]] || [[ "$output" == *"corrupted"* ]]
    fi
}
```

## Exit Code Conventions

**Establish clear exit codes for different error types**:

```
0  - Success
1  - General error
2  - Misuse of command (wrong arguments)
3  - Repository error (not a git repo, corruption)
4  - Network error (no internet, timeout)
5  - Authentication error (invalid credentials)
6  - Authorization error (permission denied)
7  - Conflict (merge conflict, non-fast-forward)
8  - Not found (file, branch, remote)
```

**Test exit codes explicitly**:
```bash
@test "returns 3 for invalid repository" {
    run my_cli status /nonexistent/path
    [ $status -eq 3 ]
}

@test "returns 7 for merge conflict" {
    # Setup conflict scenario
    create_conflicting_state "$TEST_REPO"
    run my_cli merge-sync "$TEST_REPO"
    [ $status -eq 7 ]
}
```

## Error Message Quality

**Testing error messages**:
```rust
#[test]
fn test_error_messages_are_helpful() {
    let temp = TempDir::new().unwrap();

    // Non-existent repository
    let err = process_repo(&temp.path().join("nonexistent"));
    assert!(err.is_err());

    let msg = err.unwrap_err().to_string();
    assert!(msg.contains("not a git repository"));
    assert!(msg.contains(temp.path().join("nonexistent").display().to_string()));
    assert!(msg.contains("did you mean"));  // Suggestion
}
```

## Best Practices

1. **Test error paths as thoroughly as happy paths**
2. **Use realistic error injection**: Don't mock trivially
3. **Verify recovery leaves clean state**: No orphaned files
4. **Test exit codes**: Each error type should have code
5. **Error messages must be helpful**: Include context
6. **Test cleanup on failure**: Finally blocks work?
7. **Document error scenarios**: What can go wrong?
8. **Incremental operations need checkpoints**: Resume capability
9. **Rollback must be atomic**: All-or-nothing
10. **Verify error conditions can't occur**: Fix root causes

## References

- [Integration Testing Best Practices - Hypertest](https://www.hypertest.co/integration-testing/integration-testing-best-practices)
- [Testing: Types and Approaches - TestGrid](https://testgrid.io/blog/integration-testing-types-approaches/)
- [Disaster Recovery Testing - GitProtect](https://gitprotect.io/blog/become-the-master-of-disaster-disaster-recovery-testing-for-devops/)
- [Git Data Recovery - Git Book](https://git-scm.com/book/en/v2/Git-Internals-Maintenance-and-Data-Recovery)
- [Troubleshooting Git - GitLab Docs](https://docs.gitlab.com/topics/git/troubleshooting_git/)
