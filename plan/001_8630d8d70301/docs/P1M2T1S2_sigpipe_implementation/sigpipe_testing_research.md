# SIGPIPE Testing Best Practices for Rust CLI Tools

**Research Date**: 2026-01-10
**Focus**: Manual testing documentation patterns for SIGPIPE handling in CLI tools

---

## Executive Summary

This research document compiles best practices for documenting and testing SIGPIPE handling in Rust CLI tools. The findings are organized into actionable patterns that can be applied to the `jin` CLI tool.

**Key Finding**: Rust ignores SIGPIPE by default, which causes CLI tools to panic with "Broken pipe" errors instead of exiting silently like traditional Unix tools. This is a well-documented issue in the Rust ecosystem with established testing patterns.

---

## 1. How Other Rust CLI Tools Document SIGPIPE Testing

### 1.1 uutils/coreutils (Rust implementations of GNU coreutils)

**Repository**: https://github.com/uutils/coreutils

**Issue #8919**: "SIGPIPE handling is missing in many utilities"
https://github.com/uutils/coreutils/issues/8919

**Key Documentation Points**:

#### Testing Pattern from `cat` utility:
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

**Documentation Pattern**:
- Clearly states the problem: "when you pipe output and close it early (like `seq inf | head -n 1`), many utilities panic with error messages instead of exiting silently"
- References POSIX and GNU behavior standards
- Provides code examples from the codebase showing the fix pattern
- Lists what's working vs. what's broken

#### Standard Implementation Pattern:
```rust
use uucore::signals::enable_pipe_errors;

#[uucore::main]
pub fn uumain(args: impl uucore::Args) -> UResult<()> {
    #[cfg(unix)]
    enable_pipe_errors()?;

    // rest of the code
}
```

**Utilities Fixed**: cat, env, tail, tee, timeout, tr, yes (7 utilities)
**Utilities Still Broken**: seq, head, echo, ls, wc, cut, sort, uniq, nl (>17+ utilities)

### 1.2 Rust Language Discussions

**Issue #62569**: "Should Rust still ignore SIGPIPE by default?"
https://github.com/rust-lang/rust/issues/62569

**Key Insights**:
- Rust has ignored SIGPIPE by default since 2014
- This causes `command | head -n1` to panic instead of exiting cleanly
- The issue affects all Rust CLI tools that write to stdout
- BurntSushi (ripgrep author) confirms this is a widespread problem

**Quote from BurntSushi**:
> "I don't think we can change the current default behavior because of backcompat concerns... the current status quo is that nominal command line applications start out of the gate as broken"

**Documentation Pattern**: The issue includes concrete examples:
```bash
# Expected Unix behavior
$ seq 1 10000 | head -n1
1
$ echo ${pipestatus[1]}
141  # Exit code 141 = 128 + 13 (SIGPIPE)

# Rust default behavior (broken)
$ ./myprogram | head -n1
thread 'main' panicked at 'failed printing to stdout: Broken pipe (os error 32)'
```

---

## 2. Standard Patterns for Manual Testing Documentation

### 2.1 Pixelbeat.org - Comprehensive SIGPIPE Guide

**URL**: http://www.pixelbeat.org/programming/sigpipe_handling.html

**Title**: "Don't fear SIGPIPE! - Effective handling of the SIGPIPE informational signal"

**Key Documentation Principles**:

#### 1. Explain the Concept
- "shell pipelines are a functional programming concept, supporting functional composition and lazy evaluation"
- SIGPIPE enables "back pressure in the pipe" to stop producers when consumers are done
- This is a **feature**, not a bug

#### 2. Provide Common Test Cases
```bash
# Basic pipe test
yes | head -n1

# Python subprocess test
python2 -c 'import subprocess; subprocess.call("yes | :", shell=True)'

# OpenSSL example
openssl enc -aes-256-ctr -pass pass:seed -nosalt </dev/zero | head -c1

# xargs example
yes 1234 | xargs -n1 | head -n1
```

#### 3. Document Anti-Patterns
- Python 2 doesn't reset SIGPIPE for subprocesses (fixed in Python 3)
- OpenSSL has inappropriate pipe error handling
- xargs reports "terminated by signal 13" instead of exiting silently
- bash/zsh with `pipefail` have questionable SIGPIPE handling

#### 4. Document Edge Cases
- **Intermittent sources**: `tail -f log | grep -m1 'major error'` may not exit immediately
- **Multiple outputs**: `tee` writes to multiple outputs, needs special handling with `-p` flag
- **Long-lived services**: Servers should handle SIGPIPE explicitly to avoid termination

### 2.2 Manual Testing Scenarios from StackOverflow

**URL**: https://stackoverflow.com/questions/33020759/piping-to-head-results-in-broken-pipe-in-shell-script-called-from-python

**Common Test Commands**:
```bash
# Infinite output with head
yes | head -n 1

# Sequence generation with head
seq 1 100000 | head -n 5

# Large file with head
cat largefile.log | head -n 10

# Random data with head
openssl rand -base64 10000000 | head -n1
```

**Exit Code Documentation**:
- Exit code 141 indicates SIGPIPE (128 + 13)
- This is the expected behavior for Unix tools
- Tools should exit silently, not print errors

---

## 3. Common Test Scenarios for SIGPIPE

### 3.1 Basic Pipe Scenarios

#### Test 1: Piping to `head`
```bash
# Should exit silently after first line
jin log | head -n 1

# Should exit silently after 10 lines
jin log | head -n 10

# Should exit silently after 100 lines
jin log | head -n 100
```

**Expected Behavior**:
- No error messages
- Exit code 0 (for the pipeline overall)
- The piped command (jin) exits with SIGPIPE (code 141), but this is silent

#### Test 2: Piping to `head -c` (byte limit)
```bash
# Should exit silently after 100 bytes
jin log | head -c 100

# Should exit silently after 1KB
jin log | head -c 1024
```

**Expected Behavior**:
- No error messages
- Clean exit

#### Test 3: Piping to `grep -m` (match count limit)
```bash
# Should exit after first match
jin log | grep -m 1 "error"

# Should exit after 5 matches
jin log | grep -m 5 "warning"
```

**Expected Behavior**:
- No error messages
- Clean exit once match count is reached

#### Test 4: Piping to `tail`
```bash
# Should exit cleanly
jin log | tail -n 20

# Continuous monitoring
tail -f jin.log | jin process
```

**Expected Behavior**:
- For simple tail: clean exit
- For tail -f: may need special handling (intermittent source problem)

#### Test 5: Chained Pipes
```bash
# Multiple commands in pipeline
jin log | grep "error" | head -n 10

# Complex pipeline
jin log | sort | uniq | head -n 5

# Pipeline with processing
jin log | jin format | head -n 100
```

**Expected Behavior**:
- All commands should exit silently on broken pipe
- No error messages from any command in the pipeline

### 3.2 Error Detection Scenarios

#### Test 6: Verify Real Errors Still Show
```bash
# This should show an error (not SIGPIPE related)
jin non-existent-command 2>&1

# File read errors should still be reported
jin log --file /nonexistent/file 2>&1
```

**Expected Behavior**:
- Real errors should still be displayed
- SIGPIPE handling shouldn't suppress legitimate errors

#### Test 7: Buffer Overflow Test
```bash
# Generate output larger than pipe buffer (typically 64KB)
jin --generate-large-output | head -n 1

# Test with very large output
jin --massive-output | head -c 100
```

**Expected Behavior**:
- Should handle writes larger than PIPE_BUF
- Should exit silently when pipe closes

### 3.3 Interactive Scenarios

#### Test 8: Piping to Pager
```bash
# Less pager
jin log | less

# More pager
jin log | more

# Custom pager
jin log | cat
```

**Expected Behavior**:
- Should work with pagers that close early
- No broken pipe errors when user quits pager

#### Test 9: Redirection and Tees
```bash
# Tee to multiple outputs
jin log | tee output.txt | head -n 10

# Redirect to file (no SIGPIPE)
jin log > output.txt

# Redirect stderr
jin log 2> errors.txt | head -n 10
```

**Expected Behavior**:
- Tee should handle SIGPIPE gracefully (may need `-p` flag)
- File redirections should not trigger SIGPIPE

---

## 4. GitHub Repositories with SIGPIPE Test Documentation

### 4.1 Real-World Examples

#### curl Project
**Issue #14344**: "Starting with 8.9.1, SIGPIPE leaks in some cases"
https://github.com/curl/curl/issues/14344

- Shows that even mature projects encounter SIGPIPE issues
- Documents test suite failures due to SIGPIPE
- Example of regression testing for SIGPIPE behavior

#### git-cliff Project
**Issue #407**: "Errors out with `ERROR git_cliff > IO error: Broken pipe`"
https://github.com/orhun/git-cliff/issues/407

- Real bug report from a Rust CLI tool
- Shows user-facing impact of broken SIGPIPE handling
- Includes fix for broken pipe errors

#### uutils/coreutils
**Repository**: https://github.com/uutils/coreutils
**Key Files**:
- `src/uu/cat/src/cat.rs` - Fixed with direct libc calls (commit 4406b40)
- `src/uu/tail/src/tail.rs` - Uses `enable_pipe_errors()` (line 49)
- `src/uu/tee/src/tee.rs` - Uses `enable_pipe_errors()` (line 166)
- `src/uu/tr/src/tr.rs` - Uses `enable_pipe_errors()` (line 42)
- `tests/by-util/test_cat.rs` - Test at lines 122-135

### 4.2 Code Documentation Patterns

#### Pattern 1: Function Documentation
```rust
/// Enables pipe errors by resetting SIGPIPE handler to default.
///
/// This allows the process to be killed by SIGPIPE when writing to
/// a closed pipe, which is the expected Unix behavior for command-line
/// tools.
///
/// # Example
/// ```
/// #[cfg(unix)]
/// enable_pipe_errors()?;
/// ```
///
/// # Testing
/// Test with: `command | head -n 1`
/// Expected: Silent exit (no error messages)
#[cfg(unix)]
fn enable_pipe_errors() -> Result<()> {
    // Implementation
}
```

#### Pattern 2: Module Documentation
```rust
//! # Unix Signal Handling
//!
//! This module handles Unix signals for proper CLI tool behavior.
//!
//! ## SIGPIPE Handling
//!
//! By default, Rust ignores SIGPIPE, which causes "Broken pipe" errors
//! when piping output to commands like `head`, `tail`, or `grep`.
//!
//! We reset SIGPIPE to SIG_DFL to match Unix conventions:
//!
//! ```bash
//! $ jin log | head -n 1
//! # Expected: silent exit (no error message)
//! # Without fix: "thread 'main' panicked at 'Broken pipe (os error 32)'"
//! ```
//!
//! ## Testing
//!
//! Test SIGPIPE handling with:
//! - `jin log | head -n 1`
//! - `jin log | head -n 10`
//! - `jin log | grep -m 1 "pattern"`
//! - `jin log | less` (then quit immediately)
```

#### Pattern 3: README Documentation
```markdown
## Pipe Behavior

`jin` follows Unix conventions for pipe handling. When output is piped to
another command that closes early (e.g., `head`, `grep -m`, `less`), `jin`
will exit silently without error messages.

### Examples

```bash
# These commands exit silently when the pipe closes
jin log | head -n 10
jin log | grep -m 1 "error"
jin log | less

# Exit code 141 indicates SIGPIPE (normal for piped commands)
jin log | head -n 1
echo ${PIPESTATUS[0]}  # Outputs: 141
```

### Testing

To verify pipe handling works correctly:

```bash
# Should exit silently (no error messages)
jin log | head -n 1

# Should not show "Broken pipe" errors
jin --verbose | head -n 5
```
```

---

## 5. Documentation Templates for jin CLI

### 5.1 README Section Template

```markdown
## Signal Handling

### SIGPIPE Support

`jin` properly handles SIGPIPE signals when used in Unix pipelines. This means
that when you pipe output to a command that closes early (like `head`, `tail`,
or `grep`), `jin` will exit silently instead of displaying "Broken pipe" errors.

#### Examples

```bash
# All of these exit cleanly without errors
jin log | head -n 10
jin log | tail -n 20
jin log | grep -m 1 "ERROR"
jin log | less

# Complex pipelines work correctly
jin log | grep "error" | sort | uniq | head -n 5
```

#### Testing

To test SIGPIPE handling:

```bash
# Basic test - should exit silently
jin log | head -n 1

# Large output test - should handle buffer overflow
jin --verbose | head -n 1

# Verify no error messages
jin log 2>&1 | head -n 5 | grep -i "error"
# Should return nothing (no errors)
```

#### Implementation Notes

- On Unix systems, `jin` resets SIGPIPE to the default handler
- This matches the behavior of traditional Unix tools (GNU coreutils)
- Exit code 141 indicates SIGPIPE (128 + 13), which is normal for pipelines
- On Windows, SIGPIPE is not applicable
```

### 5.2 Contributing Guide Section Template

```markdown
## Testing SIGPIPE Handling

When adding features that write to stdout, ensure SIGPIPE is handled correctly.

### Manual Testing Checklist

- [ ] `jin [command] | head -n 1` exits silently
- [ ] `jin [command] | head -n 10` exits silently
- [ ] `jin [command] | grep -m 1 "pattern"` exits silently
- [ ] `jin [command] | less` exits when pager closes
- [ ] `jin [command] | head -c 100` exits silently (byte limit)
- [ ] Large output test: `jin [command] --verbose | head -n 1`
- [ ] Complex pipeline: `jin [command] | grep "x" | head -n 5`

### Automated Testing

Use the `broken_pipe()` test pattern from uutils/coreutils:

```rust
#[test]
fn test_broken_pipe() {
    let mut child = new_ucmd!()
        .args(&["your-command"])
        .set_stdout(Stdio::piped())
        .run_no_wait();

    child.close_stdout();
    child.wait().unwrap().fails_silently();
}
```

### Common Pitfalls

1. **Using `println!` without error handling**: Use `writeln!` and check for `BrokenPipe` errors
2. **Ignoring write errors**: Always check `io::Result` from write operations
3. **Buffer issues**: Large output may exceed PIPE_BUF (typically 64KB)
4. **Subprocesses**: Ensure subprocesses inherit correct SIGPIPE behavior
```

### 5.3 Release Notes Template

```markdown
## Pipe Handling Improvements

Fixed SIGPIPE handling to match Unix conventions. Previously, piping output
to commands like `head` or `grep` would result in "Broken pipe" errors.
Now `jin` exits silently when the pipe closes, matching the behavior of
traditional Unix tools.

### Before

```bash
$ jin log | head -n 1
thread 'main' panicked at 'failed printing to stdout: Broken pipe (os error 32)'
```

### After

```bash
$ jin log | head -n 1
[First line of output - clean exit, no error]
```

This change affects all commands that write to stdout and are commonly used
in pipelines.
```

---

## 6. Actionable Recommendations for jin

### 6.1 Immediate Actions

1. **Add Manual Testing Section to README**
   - Document expected SIGPIPE behavior
   - Provide example test commands
   - Show expected vs. unexpected output

2. **Add Automated Tests**
   - Implement `broken_pipe()` test for each command that outputs to stdout
   - Test with various pipe scenarios (head, grep, tail, less)
   - Test with large output (buffer overflow)

3. **Add Integration Tests**
   - Test real-world pipelines
   - Test with common Unix tools (head, tail, grep, awk, sed)
   - Test exit codes

### 6.2 Documentation Structure

```
docs/
├── sigpipe_handling.md         # Technical documentation
├── testing_guide.md            # Testing procedures
└── pipe_behavior_examples.md   # User-facing examples

README.md                       # Add "Pipe Behavior" section
CONTRIBUTING.md                 # Add SIGPIPE testing checklist
CHANGELOG.md                    # Document SIGPIPE fixes
```

### 6.3 Testing Commands for Manual Testing

Create a test script:

```bash
#!/bin/bash
# test_sigpipe.sh - Manual SIGPIPE testing for jin

echo "Testing SIGPIPE handling..."
echo

# Test 1: Basic head test
echo "Test 1: jin log | head -n 1"
jin log | head -n 1
if [ $? -eq 0 ]; then
    echo "✓ PASS: Exited cleanly"
else
    echo "✗ FAIL: Non-zero exit code"
fi
echo

# Test 2: Large output
echo "Test 2: jin --verbose | head -n 1"
jin --verbose 2>&1 | head -n 1
if [ $? -eq 0 ]; then
    echo "✓ PASS: Large output handled"
else
    echo "✗ FAIL: Large output caused error"
fi
echo

# Test 3: Grep with match limit
echo "Test 3: jin log | grep -m 1 test"
jin log | grep -m 1 "test"
if [ $? -eq 0 ]; then
    echo "✓ PASS: Grep -m handled"
else
    echo "✗ FAIL: Grep -m caused error"
fi
echo

# Test 4: Complex pipeline
echo "Test 4: jin log | head -n 5 | sort"
jin log | head -n 5 | sort > /dev/null
if [ $? -eq 0 ]; then
    echo "✓ PASS: Complex pipeline handled"
else
    echo "✗ FAIL: Complex pipeline caused error"
fi
echo

echo "SIGPIPE testing complete!"
```

### 6.4 Code Comments

Add to `src/main.rs`:

```rust
// SIGPIPE Handling
//
// On Unix systems, we reset SIGPIPE to SIG_DFL to enable proper pipe behavior.
// This allows jin to exit silently when a pipe closes (e.g., when piping to
// `head`, `tail`, or `grep`), matching the behavior of traditional Unix tools.
//
// Without this fix, piping output would result in:
//   "thread 'main' panicked at 'failed printing to stdout: Broken pipe (os error 32)'"
//
// With this fix, the expected behavior is:
//   $ jin log | head -n 1
//   [First line of output - clean exit, no error]
//
// See: https://github.com/rust-lang/rust/issues/62569
// See: https://github.com/uutils/coreutils/issues/8919
#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}
```

---

## 7. Sources and References

### Primary Sources

1. **uutils/coreutils Issue #8919** - "SIGPIPE handling is missing in many utilities"
   https://github.com/uutils/coreutils/issues/8919

2. **Rust Issue #62569** - "Should Rust still ignore SIGPIPE by default?"
   https://github.com/rust-lang/rust/issues/62569

3. **Rust Issue #97889** - "Tracking Issue for unix_sigpipe"
   https://github.com/rust-lang/rust/issues/97889

4. **Pixelbeat.org** - "Effective handling of the SIGPIPE informational signal"
   http://www.pixelbeat.org/programming/sigpipe_handling.html

5. **Unix StackExchange** - "How to portably test for a SIGPIPE failure"
   https://unix.stackexchange.com/questions/528844/how-to-portably-test-for-a-sigpipe-failure

6. **StackOverflow** - "How to prevent SIGPIPEs (or handle them properly)"
   https://stackoverflow.com/questions/108183/how-to-prevent-sigpipes-or-handle-them-properly

### Additional Resources

7. **Rust Tracking Issue #131436** - "Gracefully handling broken pipes"
   https://github.com/rust-lang/rust/issues/131436

8. **git-cliff Issue #407** - "IO error: Broken pipe"
   https://github.com/orhun/git-cliff/issues/407

9. **curl Issue #14344** - "SIGPIPE leaks in some cases"
   https://github.com/curl/curl/issues/14344

10. **Rust CLI Book** - "Testing Command Line Applications in Rust"
    https://rust-cli.github.io/book/tutorial/testing.html

11. **sigpipe crate** - Crate for resetting SIGPIPE behavior
    https://lib.rs/crates/sigpipe

12. **Unix SIGPIPE Guide** - PMhahn's SIGPIPE documentation
    https://pmhahn.github.io/SIGPIPE/

### Testing Resources

13. **uutils/coreutils test_cat.rs** - Test example at lines 122-135
    https://github.com/uutils/coreutils/blob/main/tests/by-util/test_cat.rs

14. **Lobsters discussion** - "Unix shell pipelines have two usage patterns"
    https://lobste.rs/s/sfovod/unix_shell_pipelines_have_two_usage

15. **GitHub Actions Issue #2684** - "Action runner ignores SIGPIPE"
    https://github.com/actions/runner/issues/2684

---

## Appendix: Quick Reference

### Common Test Commands

```bash
# Basic pipe tests
jin command | head -n 1
jin command | head -n 10
jin command | tail -n 20
jin command | grep -m 1 "pattern"
jin command | less

# Large output tests
jin command --verbose | head -n 1
jin command --massive-output | head -c 100

# Complex pipelines
jin command | grep "x" | sort | uniq | head -n 5
jin command | tee output.txt | head -n 10

# Exit code tests
jin command | head -n 1; echo ${PIPESTATUS[0]}
# Expected: 141 (or 0 for the overall pipeline)
```

### Expected Behavior Checklist

- ✓ No "Broken pipe" error messages
- ✓ No panic messages
- ✓ Silent exit when pipe closes
- ✓ Exit code 141 for SIGPIPE (128 + 13)
- ✓ Pipeline returns exit code 0
- ✓ Real errors still displayed appropriately
- ✓ Works with all common Unix tools (head, tail, grep, less, etc.)

### Implementation Checklist

- ✓ Add `libc` dependency to Cargo.toml
- ✓ Reset SIGPIPE to SIG_DFL in main()
- ✓ Use `#[cfg(unix)]` for platform-specific code
- ✓ Handle `io::ErrorKind::BrokenPipe` in write operations
- ✓ Add automated tests for broken pipe scenarios
- ✓ Add manual testing documentation
- ✓ Update README with pipe behavior examples
- ✓ Update CHANGELOG with SIGPIPE fixes

---

**Document Version**: 1.0
**Last Updated**: 2026-01-10
**Maintainer**: Research for P1.M2.T1.S2 - SIGPIPE Testing Documentation
