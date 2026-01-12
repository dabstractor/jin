# Rust CLI Integration Testing Research Summary

## Overview

This research documents best practices for Rust CLI integration testing, specifically focused on the `jin` project - a phantom Git layer system for developer configuration.

## Research Topics

1. **Command Pattern for Testing CLI Apps** - `01_command_pattern_testing.md`
2. **assert_cmd Crate Usage** - `02_assert_cmd_usage_examples.md`
3. **Isolated Test Environments** - `03_isolated_test_environments.md`
4. **Integration Test Naming Conventions** - `04_integration_test_naming_conventions.md`
5. **Testing Git Operations** - `05_git_operations_testing.md`

## Key Findings

### 1. Command Pattern

**Primary Resource**: https://docs.rs/assert_cmd/latest/assert_cmd/

The command pattern for testing Rust CLIs involves:
- Using `assert_cmd::Command` to spawn subprocesses
- Chaining assertions with `.assert().success().stdout(...)`
- Setting current directory and environment variables
- Using `env!("CARGO_BIN_EXE_<name>")` for binary path

```rust
use assert_cmd::Command;

fn jin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

jin()
    .arg("--help")
    .assert()
    .success()
    .stdout(predicate::str::contains("help text"));
```

### 2. assert_cmd Crate

**Primary Resource**: https://docs.rs/assert_cmd/

**Version Used**: 2.0

Key patterns:
- `Command::new(env!("CARGO_BIN_EXE_jin"))` - Binary invocation
- `.args(["add", "file", "--flag"])` - Multiple arguments
- `.current_dir(path)` - Set working directory
- `.env("KEY", value)` - Set environment variables
- `.assert().success()` or `.assert().failure()` - Exit code assertions
- `.stdout(predicate::str::contains("text"))` - Output validation
- `.get_output()` - Get raw output for inspection

Related crates:
- `predicates` 3.0 - Boolean-valued assertions
- `tempfile` 3.0 - Temporary directories
- `serial_test` 3.0 - Sequential test execution

### 3. Isolated Test Environments

**Primary Resources**:
- https://docs.rs/tempfile/
- https://git-scm.com/docs/gitglossary#Documentation/gitglossary.txt-aiddeflockfilealockfile

Critical patterns:

**Temp Directory Isolation**:
```rust
pub struct TestFixture {
    _tempdir: TempDir,  // Must keep in scope
    pub path: PathBuf,
    pub jin_dir: Option<PathBuf>,
}
```

**Git Lock Cleanup**:
```rust
impl Drop for TestFixture {
    fn drop(&mut self) {
        let _ = cleanup_git_locks(&self.path);
        if let Some(ref jin_dir) = self.jin_dir {
            let _ = cleanup_git_locks(jin_dir);
        }
    }
}
```

**Unique Identifiers**:
```rust
pub fn unique_test_id() -> String {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{}_{}", std::process::id(), count)
}
```

### 4. Integration Test Naming Conventions

**Primary Resource**: https://doc.rust-lang.org/book/ch11-03-test-organization.html

**File Organization**:
- `tests/` directory at crate root
- `tests/common/` for shared utilities
- `tests/cli_<command>.rs` for command-specific tests
- `tests/<feature>_workflow.rs` for multi-step workflows
- `tests/<domain>_<type>.rs` for domain-specific tests

**Function Naming Patterns**:
- `test_<feature>_<condition>_<expected_result>()`
- `test_<command>_<state>_<condition>()`
- `test_<feature>_workflow_<scenario>()`
- `test_<feature>_<edge_case>()`

**Example**: `test_add_local_routes_to_layer_8()`

### 5. Testing Git Operations

**Primary Resources**:
- https://docs.rs/git2/
- https://github.com/rust-lang/git2-rs

**Critical Patterns**:

**Git Repository Initialization**:
```rust
git2::Repository::init(path)?;
git2::Repository::init_bare(remote_path)?;
```

**Lock Cleanup**:
```rust
pub fn cleanup_git_locks(repo_path: &Path) -> Result<()> {
    let lock_files = &["index.lock", "HEAD.lock", "config.lock"];
    for lock_file in lock_files {
        let _ = fs::remove_file(git_dir.join(lock_file));
    }
    Ok(())
}
```

**Multi-Layer Testing**:
- Test cross-layer conflicts
- Verify deep merge behavior
- Validate layer precedence
- Test nested object merging

## Best Practices Summary

1. **Always use temp directories** for test isolation
2. **Implement Drop cleanup** especially for Git locks
3. **Use unique identifiers** combining process ID and atomic counter
4. **Set working directory explicitly** in all tests
5. **Use descriptive test names** following established patterns
6. **Chain assertions** for clarity
7. **Test both success and failure** paths
8. **Use predicates** for flexible output validation
9. **Group related tests** with comment sections
10. **Document test purpose** at file and function level

## Common Pitfalls

1. **Early TempDir drop** - Fixture must own the TempDir
2. **Forgotten Git locks** - Causes "index lock exists" errors
3. **Shared environment variables** - Tests interfere
4. **Non-unique names** - Parallel tests overwrite data
5. **Missing current_dir** - Commands run in wrong directory
6. **Vague test names** - Difficult to maintain

## Testing Checklist

- [ ] Using temp directory for all file operations
- [ ] Isolated environment variables
- [ ] Git repository initialized in temp directory
- [ ] Unique identifiers for names
- [ ] Drop implementation for cleanup
- [ ] current_dir() set on all commands
- [ ] Git lock cleanup in place
- [ ] Descriptive test names
- [ ] Tests grouped by feature
- [ ] Both success and failure paths tested

## External Resources

### Documentation
- [assert_cmd documentation](https://docs.rs/assert_cmd/)
- [predicates crate](https://docs.rs/predicates/)
- [tempfile crate](https://docs.rs/tempfile/)
- [git2 crate](https://docs.rs/git2/)
- [serial_test crate](https://docs.rs/serial_test/)

### Guides
- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Rust Book - Integration Tests](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Rust CLI Working Group](https://cli.rs/)
- [Rust API Guidelines - Testing](https://rust-lang.github.io/api-guidelines/testing.html)

### Repositories
- [assert-rs/assert_cmd](https://github.com/assert-rs/assert_cmd)
- [rust-lang/git2-rs](https://github.com/rust-lang/git2-rs)
- [Stebalien/tempfile](https://github.com/Stebalien/tempfile)

### Git Resources
- [Git Internals](https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain)
- [Git Lock Files](https://git-scm.com/docs/gitglossary#Documentation/gitglossary.txt-aiddeflockfilealockfile)

## Project-Specific Patterns

The jin project demonstrates advanced CLI testing patterns:

- **Multi-layer Git operations**: Testing files stored in different Git repositories
- **Conflict resolution**: Testing merge conflicts across layers
- **Structured file merging**: Testing RFC 7396 JSON Merge Patch semantics
- **Nested object merging**: Testing 3-level deep merging
- **Array key-based merging**: Testing array merging with key fields

These patterns are documented in detail in the individual research files.
