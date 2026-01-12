# External Research References

## Rust Testing Best Practices

### tempfile Crate Documentation
- **URL**: https://docs.rs/tempfile/latest/tempfile/
- **Why**: Essential for creating temporary files and directories in tests
- **Key Types**:
  - `NamedTempFile` - Temporary file with automatic cleanup
  - `TempDir` - Temporary directory with automatic cleanup

### Rust Integration Test Organization
- **URL**: https://doc.rust-lang.org/book/ch11-03-test-organization.html
- **Why**: Understanding tests/ directory structure
- **Key Points**:
  - Integration tests go in `tests/` directory at crate root
  - Each file is compiled as a separate test binary
  - Use `mod common;` to share test utilities

### serde_json Testing
- **URL**: https://docs.rs/serde_json/latest/serde_json/
- **Why**: JSON parsing and validation in tests
- **Key Functions**:
  - `serde_json::from_str::<T>()` - Parse JSON string
  - `serde_json::to_string_pretty()` - Pretty-print JSON
  - `serde_json::Value` - Dynamic JSON type

### CLI Testing with assert_cmd
- **URL**: https://docs.rs/assert_cmd/latest/assert_cmd/
- **Why**: Testing CLI commands in integration tests
- **Key Pattern**:
  ```rust
  Command::new(env!("CARGO_BIN_EXE_jin"))
      .args(["add", "file.json", "--mode"])
      .current_dir(project_path)
      .env("JIN_DIR", jin_dir)
      .assert()
      .success();
  ```

### predicates Crate for Assertions
- **URL**: https://docs.rs/predicates/latest/predicates/
- **Why**: Flexible output assertions
- **Example**:
  ```rust
  .stdout(predicate::str::contains("Operation paused"))
  ```

### predicates-str crate
- **URL**: https://docs.rs/predicates-str/latest/predicates_str/
- **Why**: String-specific predicates for assertions
- **Common Usage**: `predicate::str::contains("substring")`

### serial_test Crate
- **URL**: https://docs.rs/serial_test/latest/serial_test/
- **Why**: Ensures tests using JIN_DIR run sequentially
- **Usage**: `#[serial]` attribute on test functions

## Codebase-Specific URLs

### Git Operations in Tests
- **URL**: https://docs.rs/git2/latest/git2/
- **Why**: Creating repositories, finding refs in tests
- **Key Functions**:
  - `git2::Repository::init(path)` - Create a new repo
  - `git2::Repository::open(path)` - Open existing repo
  - `repo.find_reference(ref_path)` - Find a Git ref
