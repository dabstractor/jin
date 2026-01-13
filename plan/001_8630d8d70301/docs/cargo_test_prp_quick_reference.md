# Cargo Test PRP Quick Reference

## Purpose
Quick reference for creating AI PRPs that run and interpret cargo test results.

## Essential Commands

### Run All Tests (Recommended)
```bash
cargo test --workspace --no-fail-fast 2>&1 | tee test_results.txt
```

### Run Tests and Capture Summary
```bash
cargo test --workspace --no-fail-fast 2>&1 | tee test_results.txt | grep -E "(running|test result:|failures:)"
```

### Parse Test Metrics
```bash
cargo test --workspace --no-fail-fast 2>&1 | awk '
  /test result:/ {
    gsub(/[,;]/, "")
    passed += $3
    failed += $5
    ignored += $7
  }
  END {
    print "Total: " (passed + failed + ignored)
    print "Passed: " passed
    print "Failed: " failed
    print "Ignored: " ignored
  }'
```

## Output Format

### Success Example
```
running 15 tests
test test_name ... ok
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Failure Example
```
running 15 tests
test test_name ... FAILED

failures:

---- test_name stdout ----
thread 'test_name' panicked at 'assertion failed', src/file.rs:42:9

failures:
    test_name

test result: FAILED. 14 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

## Summary Line Format
```
test result: <STATUS>. <PASSED> passed; <FAILED> failed; <IGNORED> ignored; <MEASURED> measured; <FILTERED> filtered out
```

## Key Flags

| Flag | Purpose |
|------|---------|
| `--workspace` | Test all packages in workspace |
| `--no-fail-fast` | Run all tests even if some fail |
| `--verbose` | Show detailed compilation output |
| `-- --nocapture` | Show test output during execution |
| `-- --test-threads=1` | Run tests sequentially |
| `-- --list` | List all tests without running |

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | All tests passed |
| `1` | One or more tests failed |
| `101` | Test executable failed to compile |

## Jin Project Baseline

- **Total tests**: ~650 (±50)
- **Expected failures**: ~8-12 (before fixes)
- **Test structure**: 621 unit tests + integration tests
- **Common issues**: Ref paths, file system paths, git locks

## PRP Template

```markdown
## Test Execution

### Run Full Test Suite
```bash
cd /home/dustin/projects/jin
cargo test --workspace --no-fail-fast 2>&1 | tee test_results.txt
```

### Extract Summary
```bash
grep -E "(running|test result:|failures:)" test_results.txt > test_summary.txt
cat test_summary.txt
```

### Calculate Metrics
```bash
awk '
  /test result:/ {
    gsub(/[,;]/, "")
    passed += $3
    failed += $5
    ignored += $7
  }
  END {
    print "Total: " (passed + failed + ignored)
    print "Passed: " passed
    print "Failed: " failed
    print "Ignored: " ignored
  }
' test_summary.txt
```

### Compare Against Baseline
Expected: ~650 total tests, ~8-12 failing

If total tests < 600 or > 700: WARNING
If failed tests < 8 or > 12: INFO (may indicate progress)
```

## Common Gotchas

1. **Multiple test binaries**: Each file generates separate summary line
   - Solution: Aggregate all summary lines

2. **Pipe masks exit code**: `cargo test | tee` returns 0
   - Solution: Use `${PIPESTATUS[0]}` or save output first

3. **Tests stop on first failure** (default)
   - Solution: Always use `--no-fail-fast`

4. **Parallel tests interfere**
   - Solution: Use `-- --test-threads=1` for debugging

5. **`--all` is deprecated**
   - Solution: Use `--workspace` instead

## Official Documentation

- [Cargo Test Command](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [The Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)

## Quick Commands

```bash
# Run all tests
cargo test --workspace

# Run all tests, don't stop on failure
cargo test --workspace --no-fail-fast

# Capture to file
cargo test --workspace > test_results.txt 2>&1

# Capture and show live
cargo test --workspace 2>&1 | tee test_results.txt

# Show only failures
cargo test --workspace 2>&1 | grep -A 20 "^----"

# Run sequentially
cargo test --workspace -- --test-threads=1

# Run with backtrace
RUST_BACKTRACE=1 cargo test --workspace

# List all tests
cargo test --workspace -- --list
```

## Parsing Examples

### Extract test counts
```bash
grep "test result:" test_results.txt | \
  awk '{passed+=$3; failed+=$5} END {print passed+failed, passed, failed}'
```

### Show only failed test names
```bash
grep -A 5 "^failures:" test_results.txt | grep "^    "
```

### Compare two runs
```bash
diff <(grep "test result:" run1.txt) <(grep "test result:" run2.txt)
```

### Count total tests across all binaries
```bash
cargo test --workspace 2>&1 | grep -oP '\d+(?= passed)' | awk '{s+=$1} END {print s}'
```

---

**Last Updated**: January 12, 2026
**See Also**: `cargo_test_research_prp_guide.md` for comprehensive guide
