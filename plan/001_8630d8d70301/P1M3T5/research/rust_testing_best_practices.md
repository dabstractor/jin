# Rust Integration Testing Best Practices - Research Summary

## Key Resources

### 1. Testing CLI Applications
- **[Rust CLI Book - Testing](https://rust-cli.github.io/book/tutorial/testing.html)** - Definitive guide for CLI testing
  - Use `assert_cmd` crate for testing CLIs
  - Test observable behaviors, not internal logic
  - Place tests in `tests/` directory
  - Invoke binary like a real user would

### 2. Temporary Directory Testing
- **[Testing in Rust: Temporary Files](http://www.andrewra.dev/2019/03/01/testing-in-rust-temporary-files/)** - tempfile patterns
  - Use `tempfile` crate (not deprecated `tempdir`)
  - Create custom Fixture structs for better organization
  - Avoid changing CWD - use absolute paths instead
  - Automatic cleanup when TempDir goes out of scope

### 3. Integration Testing Organization
- **[Rust By Example - Integration Testing](https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html)**
  - Separate integration tests in `tests/` directory
  - Use `tests/common/` for test utilities
  - Group related tests in modules

### 4. Assertion Patterns
- **[assert_cmd Documentation](https://docs.rs/assert_cmd)** - CLI testing assertions
- **[predicates crate](https://docs.rs/predicates)** - Powerful output matching
  ```rust
  Command::cargo_bin("my-cli")
      .unwrap()
      .args(&["init"])
      .assert()
      .success()
      .stdout(predicate::str::contains("Initialized"));
  ```

### 5. Error Testing
- Test error conditions explicitly
- Use `Result` return types in tests
- Match specific error kinds when possible

## Common Pitfalls to Avoid

1. Don't test implementation details - focus on observable behavior
2. Avoid flaky tests - use proper temporary directories and cleanup
3. Don't depend on network - mock external services
4. Avoid global state - each test should be isolated
5. Don't forget cleanup - use RAII patterns for temporary resources
6. Don't ignore error cases - test both success and failure paths

## Recommended Tools

- `assert_cmd` - For testing CLI binaries
- `predicates` - For powerful output matching
- `tempfile` - For temporary file/directory management
- `git2` - For programmatic Git operations in tests
- `serial_test` - For sequential test execution when needed
