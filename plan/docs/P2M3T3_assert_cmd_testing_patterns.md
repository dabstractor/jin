# assert_cmd CLI Testing Patterns Research

## Documentation URLs

### Primary Resources
- [assert_cmd - Official API Docs](https://docs.rs/assert_cmd/latest/assert_cmd/cmd/struct.Command.html)
- [assert_cmd on crates.io](https://crates.io/crates/assert_cmd)
- [assert_cmd GitHub Repository](https://github.com/assert-rs/assert_cmd)
- [Rust CLI Book - Testing Chapter](https://rust-cli.github.io/book/tutorial/testing.html)
- [How I test Rust command-line apps with assert_cmd (2025)](https://alexwlchan.net/2025/testing-rust-cli-apps-with-assert-cmd/)
- [Testing Non-Interactive Binaries](https://www.rustadventure.dev/building-a-digital-garden-cli/clap-v4/testing-non-interactive-binaries-with-assert_cmd)

## Key Dependencies

```toml
[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.0"
assert_fs = "1.1"
```

## Basic CLI Testing Pattern

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_command() {
    Command::cargo_bin("your_cli_name")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stderr("");
}
```

## Multi-Section Output Validation

For CLI output with multiple sections like "Active Context Updates" and "Other Updates":

```rust
#[test]
fn test_multi_section_output() {
    let output = Command::cargo_bin("your_cli_name")
        .unwrap()
        .arg("fetch")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Validate specific sections exist
    assert!(stdout.contains("Active Context Updates"));
    assert!(stdout.contains("Other Updates"));

    // More detailed validation using regex patterns
    let active_section_pattern = r"Active Context Updates:.*?(?=\n\nOther Updates|$)";
    let other_section_pattern = r"Other Updates:.*?$";

    let re = regex::Regex::new(active_section_pattern).unwrap();
    assert!(re.is_match(&stdout), "Active Context Updates section not found");

    let re = regex::Regex::new(other_section_pattern).unwrap();
    assert!(re.is_match(&stdout), "Other Updates section not found");
}
```

## Testing Error Cases

```rust
#[test]
fn test_invalid_command() {
    Command::cargo_bin("your_cli_name")
        .unwrap()
        .arg("--invalid-flag")
        .assert()
        .failure()
        .code(1)
        .stdout("")
        .stderr(predicate::str::contains("error: unexpected argument"));
}
```

## Test Fixture Creation with assert_fs

```rust
use assert_fs::prelude::*;

#[test]
fn test_with_temp_file() -> Result<(), Box<dyn std::error::Error>> {
    let temp_file = assert_fs::NamedTempFile::new("test_input.txt")?;
    temp_file.write_str("test content\nmore content\n")?;

    Command::cargo_bin("your_cli_name")
        .unwrap()
        .arg("process")
        .arg(temp_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("processed"));

    Ok(())
}
```

## Best Practices Summary

1. **Use Both assert_cmd and predicates** for readable assertions
2. **Test Observable User Behavior** - what users can see, not internals
3. **Organize Tests by Purpose** - group related tests in modules
4. **Capture Raw Output for Complex Validation** - when predicates aren't enough
5. **Use assert_fs for File Operations** - clean temporary file handling

## Pattern for Section Separation Validation

```rust
#[test]
fn test_section_separation() {
    let output = Command::cargo_bin("your_cli_name")
        .unwrap()
        .arg("fetch")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let stdout = String::from_utf8(output).unwrap();

    // Check sections are properly separated
    let sections: Vec<_> = stdout.split("\n\n").collect();
    assert!(sections.len() >= 2, "Should have at least 2 sections");

    // Check for section headers
    assert!(stdout.contains("Active Context Updates:"));
    assert!(stdout.contains("Other Updates:"));
}
```

## Content Validation Within Sections

```rust
#[test]
fn test_section_content_structure() {
    Command::cargo_bin("your_cli_name")
        .unwrap()
        .arg("fetch")
        .assert()
        .success()
        .stdout(predicate::str::contains("Active Context Updates:\n").and(
            predicate::str::contains("Updated:").and(
                predicate::str::contains("Other Updates:\n").and(
                    predicate::str::contains("Deleted:")
                )
            )
        ));
}
```
