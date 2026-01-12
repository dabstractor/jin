# Rust CLI Integration Testing - Quick Reference with URLs

## External Documentation URLs

### Core Testing Crates
- **assert_cmd**: https://docs.rs/assert_cmd/latest/assert_cmd/
- **predicates**: https://docs.rs/predicates/latest/predicates/
- **tempfile**: https://docs.rs/tempfile/latest/tempfile/
- **git2**: https://docs.rs/git2/latest/git2/
- **serial_test**: https://docs.rs/serial_test/latest/serial_test/

### Official Rust Documentation
- **Rust Book - Testing**: https://doc.rust-lang.org/book/ch11-00-testing.html
- **Integration Tests**: https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests
- **Rust CLI Working Group**: https://cli.rs/

### Repositories
- **assert-rs/assert_cmd**: https://github.com/assert-rs/assert_cmd
- **rust-lang/git2-rs**: https://github.com/rust-lang/git2-rs
- **Stebalien/tempfile**: https://github.com/Stebalien/tempfile
- **palfrey/serial_test**: https://github.com/palfrey/serial_test

### Git Resources
- **Git Internals**: https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain
- **Git Lock Files**: https://git-scm.com/docs/gitglossary#Documentation/gitglossary.txt-aiddeflockfilealockfile

### Standards and Guidelines
- **Rust API Guidelines - Testing**: https://rust-lang.github.io/api-guidelines/testing.html
- **Naming Conventions RFC**: https://rust-lang.github.io/rfcs/0430-finalizing-naming-conventions.html

## Key Insights

### 1. Command Pattern
- Use `Command::new(env!("CARGO_BIN_EXE_<name>"))` for binary invocation
- Chain assertions: `.assert().success().stdout(predicate::str::contains("text"))`
- Set working directory with `.current_dir(path)`
- Set environment variables with `.env("KEY", value)`

### 2. Test Isolation
- Always use `tempfile::TempDir` for temporary directories
- Implement `Drop` for automatic cleanup
- Clean up Git locks before temp dir deletion
- Use unique identifiers: `format!("test_{}_{}", std::process::id(), counter)`

### 3. Git Testing Patterns
- Initialize Git repos in temp directories
- Use bare repositories for remotes
- Clean up lock files: `index.lock`, `HEAD.lock`, `config.lock`
- Test multi-layer operations with cross-layer conflicts

### 4. Naming Conventions
- `test_<feature>_<condition>_<expected_result>()`
- `test_<command>_<state>_<condition>()`
- `test_<feature>_workflow_<scenario>()`
- Files: `tests/cli_<command>.rs`, `tests/<feature>_workflow.rs`

### 5. Best Practices
- Keep TempDir in scope (use leading underscore for unused field)
- Use `#[serial]` for tests modifying global state
- Test both success and failure paths
- Use descriptive names and documentation comments
- Group related tests with comment sections

## Common Patterns from jin Project

### Binary Invocation Helper
```rust
fn jin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}
```

### Test Fixture with Cleanup
```rust
pub struct TestFixture {
    _tempdir: TempDir,  // Must keep in scope
    pub path: PathBuf,
    pub jin_dir: Option<PathBuf>,
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        let _ = cleanup_git_locks(&self.path);
    }
}
```

### Unique Test Identifier
```rust
pub fn unique_test_id() -> String {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let count = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{}_{}", std::process::id(), count)
}
```

### Git Lock Cleanup
```rust
pub fn cleanup_git_locks(repo_path: &Path) -> Result<()> {
    let git_dir = repo_path.join(".git");
    if !git_dir.exists() {
        return Ok(());
    }

    for lock_file in &["index.lock", "HEAD.lock", "config.lock"] {
        let _ = fs::remove_file(git_dir.join(lock_file));
    }
    Ok(())
}
```

## Testing Checklist

- [ ] Using temp directory for all file operations
- [ ] Isolated environment variables (JIN_DIR, HOME, etc.)
- [ ] Git repository initialized in temp directory
- [ ] Unique identifiers for names (modes, scopes, etc.)
- [ ] Drop implementation for cleanup
- [ ] current_dir() set on all commands
- [ ] Git lock cleanup in place
- [ ] Descriptive test names
- [ ] Tests grouped by feature
- [ ] Both success and failure paths tested

## Project-Specific Patterns

The jin project demonstrates:
- **Multi-layer Git operations**: Files stored in different Git repositories
- **Conflict resolution**: Merge conflicts across layers
- **Structured file merging**: RFC 7396 JSON Merge Patch semantics
- **Nested object merging**: 3-level deep merging
- **Array key-based merging**: Array merging with key fields

These are advanced patterns for CLI tools that manage version control layers.
