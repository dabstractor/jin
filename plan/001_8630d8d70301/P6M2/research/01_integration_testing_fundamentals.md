# Rust Integration Testing Fundamentals

## Overview

Integration tests are entirely external to your library and use your code in the same way any other external code would, using only the public interface and potentially exercising multiple modules per test. Their purpose is to test whether many parts of your library work together correctly.

**Key Statistics (2024 Survey):**
- 68% of Rust developers spend more time debugging integration issues than writing new features
- This emphasizes the importance of treating tests as architecture, not afterthoughts

## Project Structure

Cargo looks for integration tests in a `tests` directory at the top level of your project, next to `src`.

### Recommended Directory Structure

```
<crate_root>
├── Cargo.toml
├── src/
│   ├── lib.rs        # Library code (primary testing target)
│   └── main.rs       # Minimal binary wrapper
└── tests/
    ├── integration_test.rs
    └── common/
        └── mod.rs    # Shared test utilities
```

### Why `lib.rs` and `main.rs` Separation?

Integration tests **only work with library crates** (with `src/lib.rs`). Binary-only crates cannot be tested via integration tests.

**Best Practice:**
```rust
// src/lib.rs - Contains all library logic (testable)
pub fn find_matches(content: &str, pattern: &str, mut writer: impl std::io::Write) {
    for line in content.lines() {
        if line.contains(pattern) {
            writeln!(writer, "{}", line).ok();
        }
    }
}

// src/main.rs - Minimal code that calls library functions
use std::io;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string("input.txt")?;
    lib::find_matches(&content, "pattern", &mut io::stdout());
    Ok(())
}
```

This structure allows integration tests to verify the library, ensuring the small amount of code in `main.rs` will work correctly.

## Integration Test Characteristics

### Compilation and Execution

- **Separate Compilation:** Each file in the `tests` directory is compiled as its own separate crate
- **External Testing:** Tests are entirely external to your library and use only the public API
- **Multiple Modules:** Integration tests can exercise multiple modules per test
- **Test Isolation:** Each integration test file has its own test section

### Running Integration Tests

```bash
# Run all tests
cargo test

# Run a specific integration test file
cargo test --test integration_test

# Run a specific test function
cargo test --test integration_test test_function_name

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel (default) or sequentially
cargo test -- --test-threads=1
```

## Key Principles

### 1. **Treat Tests as First-Class Citizens**

- Allocate appropriate time and resources to integration testing
- Plan integration tests alongside feature implementation
- Review test architecture as part of code review

### 2. **Test User-Visible Behavior**

Focus integration tests on:
- Complete user workflows
- Error handling from a user perspective
- Output format and structure
- File system interactions
- Exit codes and signals

Avoid testing:
- Auto-generated help text formatting (unless it's critical)
- Implementation details
- Private functions

### 3. **Proper Code Organization**

- Keep business logic in library code
- Minimize code in binary wrappers
- Use public interfaces for integration tests
- Leverage crates for code organization

## Best Practices Summary

| Practice | Benefit |
|----------|---------|
| Separate lib.rs and main.rs | Enables comprehensive integration testing |
| Use tests/ directory | Clear organization and standard convention |
| Test complete workflows | Catches integration issues early |
| Use shared test modules | Reduces duplication and maintenance |
| Test only public API | Ensures true integration testing |
| Parallel test execution | Faster test feedback |

## References

- [Test Organization - The Rust Programming Language](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Integration Testing by Example](https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html)
- [Testing - Command Line Applications in Rust](https://rust-cli.github.io/book/tutorial/testing.html)
- [Rust Testing Best Practices: Unit to Integration | Medium](https://medium.com/@asma.shaikh_19478/rust-testing-best-practices-unit-to-integration-965b39a8212f)
- [Integration Testing Rust Binaries](https://www.unwoundstack.com/blog/integration-testing-rust-binaries.html)
