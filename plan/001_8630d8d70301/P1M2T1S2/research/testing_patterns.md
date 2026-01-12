# SIGPIPE Testing Patterns - Actionable Examples

**Extracted from**: Research on Rust CLI tools and Unix SIGPIPE handling
**Application**: jin CLI tool testing and documentation

---

## Pattern 1: Basic Manual Test Documentation

### Template
```markdown
### Manual Test: Pipe to head

**Command**: `jin log | head -n 1`

**Expected Behavior**:
- Command outputs first line of log
- Exits silently (no error messages)
- No "Broken pipe" errors
- Exit code 0 for pipeline

**Unexpected Behavior**:
- Error: "thread 'main' panicked at 'failed printing to stdout: Broken pipe (os error 32)'"
- Error: "IO error: Broken pipe"
- Any panic or stack trace

**How to Verify**:
```bash
jin log | head -n 1
echo $?  # Should be 0
```
```

### Examples from Research

**From uutils/coreutils Issue #8919**:
> "when you pipe output and close it early (like `seq inf | head -n 1`), many utilities panic with error messages instead of exiting silently like GNU coreutils does"

**From pixelbeat.org**:
> "The standard `yes | head -n1` test demonstrates proper SIGPIPE handling"

---

## Pattern 2: Automated Test Implementation

### Rust Test Pattern (from uutils/coreutils)

```rust
#[test]
fn test_broken_pipe() {
    let mut child = new_ucmd!()
        .args(&["alpha.txt"])
        .set_stdout(Stdio::piped())
        .run_no_wait();

    child.close_stdout();
    child.wait().unwrap().fails_silently();
}
```

### Adapted for jin

```rust
#[test]
fn test_log_broken_pipe() {
    let mut child = Command::new("jin")
        .args(&["log"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn jin");

    // Close stdout immediately to simulate pipe closing
    drop(child.stdout.take());

    let status = child.wait().unwrap();
    // Should exit with SIGPIPE (141) or cleanly
    assert!(status.success() || status.signal() == Some(13));
}
```

### Alternative: Using Test Harness

```rust
#[test]
fn test_pipe_to_head() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("jin log | head -n 1")
        .output()
        .expect("Failed to execute command");

    // Should succeed (pipeline exit code 0)
    assert!(output.status.success());

    // Should not contain error messages
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("Broken pipe"));
    assert!(!stderr.contains("panicked"));
}
```

---

## Pattern 3: README Documentation Section

### Full Example (from best practices)

```markdown
## Unix Pipe Compatibility

`jin` is designed to work seamlessly with Unix pipes. When you pipe output to
commands that close early (like `head`, `tail`, or `grep`), `jin` exits
silently without displaying error messages.

### Common Use Cases

```bash
# View first 10 lines
jin log | head -n 10

# Find first error
jin log | grep -m 1 "ERROR"

# View last 20 lines
jin log | tail -n 20

# Interactive pager
jin log | less

# Complex pipeline
jin log | grep "warning" | sort | uniq | head -n 5
```

### Exit Codes

When a pipe closes, `jin` receives a SIGPIPE signal and exits with code 141.
This is standard Unix behavior:

```bash
$ jin log | head -n 1
[output]
$ echo ${PIPESTATUS[0]}
141
```

The pipeline as a whole returns exit code 0, which is what matters for
shell scripts and command chaining.

### Troubleshooting

If you see "Broken pipe" errors, this indicates a bug. Please report it with:
1. The exact command you ran
2. The version of `jin`
3. Your operating system
```

---

## Pattern 4: Contributing Guide Section

### Template for Developers

```markdown
## Testing Pipe Behavior

All commands that write to stdout must handle SIGPIPE correctly.

### Manual Testing

Before submitting PRs, test your changes with:

```bash
# Basic pipe test
jin [your-command] | head -n 1

# Large output test
jin [your-command] --verbose | head -n 1

# Grep test
jin [your-command] | grep -m 1 "pattern"
```

**Expected**: Silent exit with no error messages.

### Automated Testing

Add a `broken_pipe` test for any command producing output:

```rust
#[test]
fn test_your_command_broken_pipe() {
    use std::process::{Command, Stdio};

    let mut child = Command::new("jin")
        .args(&["your-command"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn");

    drop(child.stdout.take());
    let status = child.wait().unwrap();

    // Should exit cleanly (may have SIGPIPE signal)
    assert!(status.success() || status.signal() == Some(13));
}
```

### Code Requirements

1. Use `writeln!` instead of `println!` for output
2. Check `io::Result` from write operations
3. Match `io::ErrorKind::BrokenPipe` specifically
4. Use `#[cfg(unix)]` for Unix-specific code

Example:
```rust
use std::io::{self, Write};

fn output_data(data: &str) -> io::Result<()> {
    match writeln!(stdout(), "{}", data) {
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
            // Silent exit for broken pipe
            std::process::exit(0);
        }
        Err(e) => Err(e),
        Ok(()) => Ok(()),
    }
}
```
```

---

## Pattern 5: Test Script for CI/CD

### Shell Script Pattern

```bash
#!/bin/bash
# test_sigpipe.sh - Automated SIGPIPE testing

set -e

echo "Testing SIGPIPE handling..."

# Test 1: Basic pipe to head
echo -n "Test 1 (head -n 1): "
if jin log | head -n 1 > /dev/null 2>&1; then
    echo "✓ PASS"
else
    echo "✗ FAIL"
    exit 1
fi

# Test 2: Pipe to grep with match limit
echo -n "Test 2 (grep -m 1): "
if jin log | grep -m 1 "test" > /dev/null 2>&1; then
    echo "✓ PASS"
else
    echo "✗ FAIL"
    exit 1
fi

# Test 3: Large output
echo -n "Test 3 (large output): "
if jin --verbose 2>&1 | head -n 1 > /dev/null 2>&1; then
    echo "✓ PASS"
else
    echo "✗ FAIL"
    exit 1
fi

# Test 4: Complex pipeline
echo -n "Test 4 (complex pipeline): "
if jin log | grep "x" | head -n 1 > /dev/null 2>&1; then
    echo "✓ PASS"
else
    echo "✗ FAIL"
    exit 1
fi

# Test 5: Verify no error messages in stderr
echo -n "Test 5 (no error messages): "
OUTPUT=$(jin log | head -n 1 2>&1)
if echo "$OUTPUT" | grep -qi "error"; then
    echo "✗ FAIL (found error messages)"
    exit 1
else
    echo "✓ PASS"
fi

echo ""
echo "All SIGPIPE tests passed!"
```

### GitHub Actions Pattern

```yaml
name: Test SIGPIPE Handling

on: [push, pull_request]

jobs:
  test-sigpipe:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build jin
        run: cargo build --release

      - name: Test pipe to head
        run: |
          ./target/release/jin log | head -n 1
          echo "Exit code: $?"

      - name: Test pipe to grep
        run: |
          ./target/release/jin log | grep -m 1 "test" || true

      - name: Test large output
        run: |
          ./target/release/jin --verbose 2>&1 | head -n 1

      - name: Verify no broken pipe errors
        run: |
          ! ./target/release/jin log 2>&1 | head -n 1 | grep -i "broken pipe"
```

---

## Pattern 6: Integration Test Scenarios

### Scenario 1: Log Processing Pipeline

```bash
#!/bin/bash
# Test real-world log processing scenario

# Generate test log
jin init test-project
cd test-project

# Create log entries
jin log --level INFO "Test message 1"
jin log --level ERROR "Test error 1"
jin log --level INFO "Test message 2"
jin log --level ERROR "Test error 2"

# Test: Find first error (should exit cleanly)
jin log | grep -m 1 "ERROR" | head -n 1

# Test: Show last 10 entries
jin log | tail -n 10

# Test: Count errors (should exit cleanly)
ERROR_COUNT=$(jin log | grep -c "ERROR")
echo "Found $ERROR_COUNT errors"

# Test: Format and limit output
jin log | jin format | head -n 5

echo "All log pipeline tests passed!"
```

### Scenario 2: Large Output Handling

```bash
#!/bin/bash
# Test handling of output larger than pipe buffer

# PIPE_BUF is typically 64KB on Linux
# Generate output larger than this

# Test 1: Output much larger than PIPE_BUF
for i in {1..10000}; do
    jin log --level INFO "This is a test log message number $i with some additional text to make it longer"
done | head -n 1

if [ $? -eq 0 ]; then
    echo "✓ Large output test passed"
else
    echo "✗ Large output test failed"
    exit 1
fi

# Test 2: Very long single lines
jin log --level INFO "$(python3 -c 'print("x" * 100000)')" | head -c 100

if [ $? -eq 0 ]; then
    echo "✓ Long line test passed"
else
    echo "✗ Long line test failed"
    exit 1
fi
```

---

## Pattern 7: Documentation for Specific Commands

### Command-Specific Testing

```markdown
### jin log

**Purpose**: Display log entries

**Pipe Behavior**: Compatible with all Unix pipe operations

**Examples**:
```bash
# View first 20 log entries
jin log | head -n 20

# Find and display first error
jin log | grep -m 1 "ERROR" | jin format

# View last 50 entries
jin log | tail -n 50

# Filter and sort
jin log | grep "WARNING" | sort | uniq

# Interactive pager
jin log | less
```

**Testing**:
```bash
# Test 1: Basic pipe
jin log | head -n 1
# Expected: First log entry, silent exit

# Test 2: Filter pipe
jin log | grep -m 1 "ERROR"
# Expected: First error entry, silent exit

# Test 3: Large output
jin log --all | head -n 1
# Expected: First entry, silent exit even with large buffer
```
```

---

## Pattern 8: Error Handling Documentation

### Distinguishing SIGPIPE from Real Errors

```markdown
## Error Handling

`jin` distinguishes between pipe closure (SIGPIPE) and actual errors.

### SIGPIPE (Normal Pipe Behavior)

When a pipe closes, `jin` exits silently:

```bash
$ jin log | head -n 1
[First log entry]
$ echo $?
0
```

This is **normal** and **expected** behavior.

### Real Errors (Should Display)

Actual errors will still be displayed:

```bash
$ jin log --file /nonexistent/file
Error: Unable to open log file: No such file or directory (os error 2)
$ echo $?
1
```

### Troubleshooting

If you see "Broken pipe" errors, this indicates a bug. Please report:

1. Command that produced the error
2. Full error message
3. Version: `jin --version`
4. OS: `uname -a`
```

---

## Pattern 9: Release Notes Template

```markdown
## SIGPIPE Handling

Fixed pipe handling to match Unix conventions. Previously, piping output
to commands like `head` or `grep` would result in "Broken pipe" errors.

### Before

```bash
$ jin log | head -n 1
thread 'main' panicked at 'failed printing to stdout: Broken pipe (os error 32)'
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

### After

```bash
$ jin log | head -n 1
[First log entry]
```

### Impact

This change affects all commands that write to stdout and are commonly used
in pipelines. The behavior now matches traditional Unix tools like `cat`,
`grep`, and `tail`.

### Migration

No migration needed. Existing pipelines will work correctly without changes.

### Testing

To verify the fix:

```bash
# Should exit silently
jin log | head -n 1

# Should exit silently
jin log | grep -m 1 "pattern"

# Should exit silently
jin log | less
```

### Related

- Issue: #[issue-number]
- Commit: [commit-hash]
```

---

## Pattern 10: Code Comments Template

```rust
// ============================================================================
// SIGPIPE HANDLING
// ============================================================================

// On Unix systems, writing to a closed pipe normally sends a SIGPIPE signal,
// which terminates the process. However, Rust ignores SIGPIPE by default,
// which causes "Broken pipe" errors instead of clean termination.

// We reset SIGPIPE to SIG_DFL (default behavior) to match Unix conventions.
// This allows jin to exit silently when a pipe closes, which is the expected
// behavior for command-line tools used in pipelines.

// Example:
//   $ jin log | head -n 1
//   [First log entry - clean exit]
//
// Without this fix:
//   $ jin log | head -n 1
//   thread 'main' panicked at 'failed printing to stdout: Broken pipe (os error 32)'

// References:
// - https://github.com/rust-lang/rust/issues/62569
// - https://github.com/uutils/coreutils/issues/8919
// - http://www.pixelbeat.org/programming/sigpipe_handling.html

#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}
```

---

## Summary: Implementation Checklist

### Phase 1: Code Changes
- [ ] Add `libc` dependency to Cargo.toml
- [ ] Implement `reset_sigpipe()` function
- [ ] Call `reset_sigpipe()` at start of `main()`
- [ ] Ensure `#[cfg(unix)]` is used appropriately
- [ ] Test compilation on Unix and Windows

### Phase 2: Automated Tests
- [ ] Add `test_broken_pipe()` test for each command
- [ ] Add integration tests for common pipe scenarios
- [ ] Add tests for large output handling
- [ ] Add tests for complex pipelines
- [ ] Ensure tests pass consistently

### Phase 3: Documentation
- [ ] Update README with pipe behavior section
- [ ] Update CONTRIBUTING.md with testing guidelines
- [ ] Add examples to command documentation
- [ ] Create troubleshooting guide
- [ ] Update CHANGELOG with SIGPIPE fix

### Phase 4: Verification
- [ ] Manual testing with all common pipe scenarios
- [ ] Testing with real-world use cases
- [ ] Testing on multiple Unix systems (Linux, macOS, BSD)
- [ ] Verify no regressions in other functionality
- [ ] Verify real errors still display correctly

---

**Document Version**: 1.0
**Last Updated**: 2026-01-10
