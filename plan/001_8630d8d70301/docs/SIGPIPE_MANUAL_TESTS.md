# SIGPIPE Manual Test

## Overview

This test verifies that the `jin` CLI tool handles SIGPIPE signals correctly when output is piped to commands that close early (like `head`, `tail`, `grep`).

**What is SIGPIPE?**

SIGPIPE is a Unix signal sent to a process when it attempts to write to a pipe that has been closed by the reading end. Traditional Unix tools handle this by exiting silently. However, Rust's default behavior ignores SIGPIPE, which can cause `println!` macros to panic with "Broken pipe" error messages instead of exiting cleanly.

**Why this test matters**

The `jin log` command produces large output when displaying commit history. Users commonly pipe this output to commands like `head`, `grep`, or `tail` that may close the pipe early. Without proper SIGPIPE handling, `jin` would display error messages instead of exiting silently like other Unix tools (`git log`, `cat`, `grep`, etc.).

**What we're testing**

This test verifies that the SIGPIPE fix (implemented in `src/main.rs`) works correctly by:
1. Building a release binary
2. Creating a test repository with commits
3. Running `jin log` through various pipe scenarios
4. Confirming clean exit with no error messages

---

## Prerequisites

Before running this test, ensure you have:

* **Unix-based OS**: Linux, macOS, or BSD (SIGPIPE is Unix-specific)
* **Rust toolchain**: Rust 1.70+ installed
* **Git**: Git installed and configured
* **Basic shell familiarity**: Ability to run commands in a terminal

**Platform Note**: This test applies only to Unix systems. Windows handles broken pipes differently via error codes and is not covered by this test.

---

## Build Release Binary

First, build the `jin` binary in release mode:

```bash
cd /path/to/jin
cargo build --release
```

Verify the binary was built:

```bash
./target/release/jin --version
```

Expected output:

```
jin 0.1.0
```

**What just happened?**

* `cargo build --release` compiles `jin` with optimizations enabled
* The release binary is placed at `target/release/jin`
* Using the release binary (not debug build) is important for correct SIGPIPE behavior

**Why release build?**

Debug builds may behave differently and are significantly slower. The release build matches what users will actually run.

---

## Test Setup

Create a temporary test environment with commits to generate log output:

```bash
# Create a temporary directory for testing
mkdir -p /tmp/jin-sigpipe-test
cd /tmp/jin-sigpipe-test

# Initialize a git repository
git init
git config user.email "test@example.com"
git config user.name "Test User"

# Initialize jin
/path/to/jin/target/release/jin init

# Create test commits (these generate log output for piping)
for i in {1..10}; do
    echo "Test content $i" > "file$i.txt"
    git add "file$i.txt"
    git commit -m "Test commit $i"
done

# Verify commits exist
git log --oneline
```

Expected output from `git log --oneline`:

```
10 (HEAD -> main) Test commit 10
9 Test commit 9
8 Test commit 8
...
```

**What just happened?**

* Created a clean test repository isolated from your development work
* Initialized git with user configuration (required for commits)
* Initialized jin to track layer commits
* Created 10 test commits that will produce output when `jin log` is run

**Why multiple commits?**

`jin log` outputs commit history. Without commits, there's no output to pipe, making it impossible to verify SIGPIPE behavior.

---

## Test Scenarios

### Test 1: Pipe to head

The classic SIGPIPE test - pipe to `head` which closes the pipe after reading one line.

```bash
# Run the test
/path/to/jin/target/release/jin log | head -n 1
```

**Expected Behavior**:

```
commit 10 (HEAD -> main) Test commit 10
Author: Test User <test@example.com>
Date:   Fri Jan 10 14:00:00 2026 +0000

    Test commit 10
```

The command should exit silently after showing the first commit. No error messages.

**Verify with exit code**:

```bash
/path/to/jin/target/release/jin log | head -n 1
echo $?
```

Expected exit code: `0`

**What this tests**

* `head -n 1` reads only the first line(s) then closes its stdin
* `jin` attempts to write more output to the closed pipe
* The OS sends SIGPIPE to `jin`
* With the fix: `jin` exits silently (SIG_DFL behavior)
* Without the fix: `jin` panics with "Broken pipe" error

---

### Test 2: Pipe to cat (baseline)

Establish baseline behavior when pipe doesn't close early.

```bash
# Run the test
/path/to/jin/target/release/jin log | cat
```

**Expected Behavior**:

All 10 commit entries are displayed. Command completes normally with no errors.

**Verify output count**:

```bash
/path/to/jin/target/release/jin log | cat | wc -l
```

Expected: Many lines (full commit history for all 10 commits)

**What this tests**

* `cat` reads all output until EOF, never closing the pipe early
* No SIGPIPE is triggered
* Confirms `jin log` produces correct output
* Serves as comparison for Test 1

---

### Test 3: Pipe to grep

Test with `grep` closing pipe after finding first match.

```bash
# Run the test
/path/to/jin/target/release/jin log | grep -m 1 "Test commit"
```

**Expected Behavior**:

Shows the first matching commit, then exits silently.

**Verify no errors**:

```bash
/path/to/jin/target/release/jin log | grep -m 1 "Test commit" 2>&1 | grep -i "broken pipe"
```

Expected: No output (grep finds nothing)

**What this tests**

* `grep -m 1` exits after first match, closing the pipe
* `jin` continues writing, triggering SIGPIPE
* Confirms clean exit for grep scenario (common real-world use case)

---

### Test 4: Pipe to tail

Test with `tail` reading from the end of output.

```bash
# Run the test
/path/to/jin/target/release/jin log | tail -n 5
```

**Expected Behavior**:

Shows the last 5 lines of log output. Exits silently.

**Verify**:

```bash
/path/to/jin/target/release/jin log | tail -n 5
echo $?
```

Expected exit code: `0`

**What this tests**

* `tail -n 5` may buffer and close pipe early depending on implementation
* Another common pipe scenario that should exit cleanly

---

### Test 5: Complex Pipeline

Test with multiple pipe commands.

```bash
# Run the test
/path/to/jin/target/release/jin log | head -n 1 | cat
```

**Expected Behavior**:

Shows first commit, exits silently with no errors.

**What this tests**

* Complex pipelines with multiple consumers
* Verifies SIGPIPE handling works through chained commands

---

### Test 6: Large Output Stress Test

Generate enough output to exceed pipe buffer size.

```bash
# Create many more commits for large output
for i in {11..100}; do
    echo "Test content $i" > "file$i.txt"
    git add "file$i.txt"
    git commit -m "Test commit $i"
done

# Test with large output
/path/to/jin/target/release/jin log | head -n 1
```

**Expected Behavior**:

Shows first commit, exits silently. No panic even with large buffered output.

**What this tests**

* Output larger than PIPE_BUF (typically 64KB on Linux)
* Ensures SIGPIPE handling works regardless of output size

---

## Expected vs Unexpected Behavior

### Expected Behavior (SIGPIPE Fix Working)

```bash
$ /path/to/jin/target/release/jin log | head -n 1
commit 100 (HEAD -> main) Test commit 100
Author: Test User <test@example.com>
Date:   Fri Jan 10 14:00:00 2026 +0000

    Test commit 100
$
```

**Characteristics**:
* Clean output
* Silent exit (no error messages)
* Exit code 0 for the pipeline
* Matches behavior of traditional Unix tools

### Unexpected Behavior (SIGPIPE Fix Not Working)

```bash
$ /path/to/jin/target/release/jin log | head -n 1
commit 100 (HEAD -> main) Test commit 100
Author: Test User <test@example.com>
Date:   Fri Jan 10 14:00:00 2026 +0000

    Test commit 100
thread 'main' panicked at src/commands/log.rs:90:5:
failed printing to stdout: Broken pipe (os error 32)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**Characteristics**:
* Panic message displayed
* "Broken pipe (os error 32)" error
* Stack trace shown
* Ugly output that concerns users
**Does not match Unix tool behavior**

If you see this behavior, the SIGPIPE fix is not working correctly. Check that:
1. The code in `src/main.rs` includes `reset_sigpipe()` function
2. `reset_sigpipe()` is called at the start of `main()` before CLI parsing
3. The `libc = "0.2"` dependency is in `Cargo.toml`
4. You're running the release binary, not debug build

---

## Troubleshooting

### "No output from jin log"

**Problem**: Running `jin log | head -n 1` produces no output.

**Cause**: Test repository has no commits.

**Solution**:
```bash
# Verify commits exist
git log --oneline

# If empty, create test commits
echo "test" > test.txt
git add test.txt
git commit -m "Test commit"
```

---

### "Still seeing Broken pipe errors"

**Problem**: Panics with "Broken pipe" error when piping.

**Possible Causes**:

1. **Not using release binary**:
   ```bash
   # Wrong (debug build may behave differently)
   cargo run | head -n 1

   # Correct (use release binary)
   ./target/release/jin log | head -n 1
   ```

2. **SIGPIPE code not present** - Check `src/main.rs`:
   ```bash
   grep -A 5 "reset_sigpipe" src/main.rs
   ```
   Should show:
   ```rust
   #[cfg(unix)]
   fn reset_sigpipe() {
       unsafe {
           libc::signal(libc::SIGPIPE, libc::SIG_DFL);
       }
   }
   ```

3. **reset_sigpipe() not called** - Check that it's called before CLI parsing:
   ```bash
   grep -A 10 "fn main" src/main.rs
   ```
   Should show `reset_sigpipe();` as first line in `main()`.

4. **libc dependency missing** - Check `Cargo.toml`:
   ```bash
   grep "libc" Cargo.toml
   ```
   Should show `libc = "0.2"` in dependencies.

---

### "Command not found"

**Problem**: `jin: command not found`

**Cause**: Path to binary incorrect or not built.

**Solution**:
```bash
# Build if not already built
cd /path/to/jin
cargo build --release

# Use full path to binary
/path/to/jin/target/release/jin log | head -n 1

# Or add to PATH
export PATH="/path/to/jin/target/release:$PATH"
jin log | head -n 1
```

---

### "Testing on Windows"

**Problem**: Test instructions don't work on Windows.

**Cause**: SIGPIPE is Unix-specific. Windows handles broken pipes differently.

**Solution**:
* These tests are for Unix systems only (Linux, macOS, BSD)
* Windows users can skip this test
* Windows uses ERROR_BROKEN_PIPE (error code 109) instead of signals

---

### "Exit code 141 instead of 0"

**Problem**: Pipeline exit code is 141 instead of 0.

**Explanation**: Exit code 141 = 128 + 13 (SIGPIPE signal number). This is **normal** for Unix tools.

**Why this happens**:
* The left command (`jin log`) receives SIGPIPE and exits with signal 13
* Shell reports this as exit code 141 (128 + 13)
* The overall pipeline still returns 0

**Verification**:
```bash
# In bash, check individual command exit codes
/path/to/jin/target/release/jin log | head -n 1
echo ${PIPESTATUS[@]}
```

Expected output: `141 0` (jin exited due to SIGPIPE, head succeeded)

**This is expected behavior** and matches traditional Unix tools.

---

## Summary

### Test Checklist

Run this checklist to verify SIGPIPE handling:

- [ ] Build release binary: `cargo build --release`
- [ ] Initialize test repository with commits
- [ ] Test 1: `jin log | head -n 1` exits silently
- [ ] Test 2: `jin log | cat` shows full output
- [ ] Test 3: `jin log | grep -m 1 "pattern"` exits silently
- [ ] Test 4: `jin log | tail -n 5` exits silently
- [ ] Test 5: `jin log | head -n 1 | cat` exits silently
- [ ] Verify no "Broken pipe" errors in any test
- [ ] Verify exit code 0 for pipelines

### What to Remember

* **Use release binary**: `./target/release/jin`, not `cargo run`
* **Need commits**: Test repository must have commits to generate log output
* **Silent exit is success**: No error messages means SIGPIPE handling works
* **Exit code 141 is normal**: This indicates SIGPIPE was received (expected)
* **Unix-only test**: SIGPIPE doesn't apply to Windows

### Clean Up

After testing, remove the temporary test directory:

```bash
rm -rf /tmp/jin-sigpipe-test
```

---

## References

### Internal Documentation

* **SIGPIPE Background**: [plan/docs/sigpipe_handling_patterns.md](../../plan/docs/sigpipe_handling_patterns.md) - Comprehensive guide to SIGPIPE handling in Rust CLI tools
* **Implementation PRP**: [plan/P1M2T1S1/PRP.md](../../plan/P1M2T1S1/PRP.md) - SIGPIPE reset code implementation
* **libc Dependency PRP**: [plan/P1M2T1S2/PRP.md](../../plan/P1M2T1S2/PRP.md) - libc dependency addition
* **Testing Patterns**: [plan/P1M2T1S2/research/testing_patterns.md](../../plan/P1M2T1S2/research/testing_patterns.md) - SIGPIPE testing templates and examples

### External Resources

* **[uutils/coreutils Issue #8919](https://github.com/uutils/coreutils/issues/8919)** - Discussion of SIGPIPE handling in Rust CLI tools
* **[Pixelbeat SIGPIPE Guide](http://www.pixelbeat.org/programming/sigpipe_handling.html)** - Comprehensive SIGPIPE handling guide with test patterns
* **[Rust Issue #62569](https://github.com/rust-lang/rust/issues/62569)** - Discussion of whether Rust should keep ignoring SIGPIPE by default
* **[Rust unix_sigpipe Feature](https://doc.rust-lang.org/beta/unstable-book/language-features/unix-sigpipe.html)** - Documentation of unstable feature for SIGPIPE handling

### Code Implementation

* **SIGPIPE Reset Function**: `src/main.rs` (lines 9-17) - `reset_sigpipe()` function
* **Main Entry Point**: `src/main.rs` (line 28) - `reset_sigpipe()` call before CLI parsing
* **Command Under Test**: `src/commands/log.rs` - `jin log` command that produces pipe-heavy output

### Related Tests

* **Automated Tests**: `tests/cli_basic.rs` - Basic CLI workflow tests
* **Test Documentation**: `plan/docs/TEST_RESULTS.md` - Test results and known issues

---

**Document Version**: 1.0
**Last Updated**: 2026-01-10
**Test Status**: Ready for manual verification
