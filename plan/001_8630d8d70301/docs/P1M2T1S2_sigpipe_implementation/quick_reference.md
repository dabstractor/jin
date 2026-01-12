# SIGPIPE Testing Quick Reference

**For**: jin CLI Tool Development
**Purpose**: Quick reference for SIGPIPE testing and documentation

---

## Common Test Commands

### Basic Pipe Tests
```bash
# Test 1: Single line with head
jin log | head -n 1

# Test 2: Multiple lines with head
jin log | head -n 10

# Test 3: Byte limit with head
jin log | head -c 100

# Test 4: Match limit with grep
jin log | grep -m 1 "error"

# Test 5: Tail command
jin log | tail -n 20

# Test 6: Pager (quit immediately)
jin log | less
```

### Large Output Tests
```bash
# Test 7: Verbose output with early pipe close
jin --verbose 2>&1 | head -n 1

# Test 8: Large data generation
jin --generate-lots-of-output | head -c 1024
```

### Complex Pipeline Tests
```bash
# Test 9: Chain multiple commands
jin log | grep "error" | sort | uniq | head -n 5

# Test 10: Pipe to tee
jin log | tee output.txt | head -n 10
```

---

## Expected Behavior

### What SHOULD Happen
- ✓ Silent exit (no error messages)
- ✓ No panic messages
- ✓ No "Broken pipe" errors
- ✓ Pipeline exit code 0
- ✓ Individual command exit code 141 (SIGPIPE = 128 + 13)

### What SHOULD NOT Happen
- ✗ Error: "thread 'main' panicked at 'failed printing to stdout: Broken pipe (os error 32)'"
- ✗ Error: "failed printing to stdout: Broken pipe"
- ✗ Error: "IO error: Broken pipe (os error 32)"
- ✗ Stack traces or backtraces

---

## Manual Testing Checklist

```
[ ] jin log | head -n 1 exits cleanly
[ ] jin log | head -n 10 exits cleanly
[ ] jin log | grep -m 1 "pattern" exits cleanly
[ ] jin log | tail -n 20 exits cleanly
[ ] jin log | less exits when pager closes
[ ] jin --verbose | head -n 1 handles large output
[ ] jin log | grep "x" | head -n 5 works in complex pipeline
[ ] No "Broken pipe" error messages in any test
[ ] Exit code 141 for SIGPIPE is acceptable
[ ] Real errors (not SIGPIPE) still display correctly
```

---

## Automated Test Pattern

From uutils/coreutils:

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

---

## Implementation Pattern

```rust
#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

fn main() {
    #[cfg(unix)]
    reset_sigpipe();

    // Rest of main()
}
```

---

## Documentation Template for README

```markdown
## Pipe Behavior

`jin` follows Unix conventions for pipe handling. When you pipe output to
commands that close early (like `head`, `tail`, or `grep`), `jin` will exit
silently without error messages.

### Examples

```bash
# These commands exit cleanly when the pipe closes
jin log | head -n 10
jin log | grep -m 1 "error"
jin log | less
```

### Testing

```bash
# Should exit silently (no error messages)
jin log | head -n 1
```
```

---

## Key URLs

- **uutils/coreutils Issue**: https://github.com/uutils/coreutils/issues/8919
- **Rust SIGPIPE Discussion**: https://github.com/rust-lang/rust/issues/62569
- **SIGPIPE Handling Guide**: http://www.pixelbeat.org/programming/sigpipe_handling.html
- **Rust CLI Testing**: https://rust-cli.github.io/book/tutorial/testing.html

---

## Exit Codes Reference

- **0**: Success (pipeline as a whole)
- **141**: SIGPIPE (128 + 13) - expected when pipe closes
- **1**: General error
- **130**: SIGINT (Ctrl+C)
- **Other**: Check command-specific documentation

---

## Common Pitfalls

1. **Using `println!`**: Doesn't handle errors, use `writeln!` instead
2. **Ignoring write errors**: Always check `io::Result` from writes
3. **Buffer size issues**: Large output may exceed PIPE_BUF (64KB)
4. **Forgetting `#[cfg(unix)]`**: SIGPIPE is Unix-only
5. **Not testing with real pipes**: Automated tests must use actual pipes

---

## Quick Test Script

```bash
#!/bin/bash
echo "Testing SIGPIPE handling..."

# Test 1
echo -n "Test 1 (head -n 1): "
jin log | head -n 1 > /dev/null 2>&1 && echo "PASS" || echo "FAIL"

# Test 2
echo -n "Test 2 (grep -m 1): "
jin log | grep -m 1 "test" > /dev/null 2>&1 && echo "PASS" || echo "FAIL"

# Test 3
echo -n "Test 3 (large output): "
jin --verbose 2>&1 | head -n 1 > /dev/null 2>&1 && echo "PASS" || echo "FAIL"

echo "Done!"
```

---

**See Also**: `sigpipe_testing_research.md` for comprehensive documentation
