# Cargo Test Research: Best Practices, Output Interpretation, and Result Capture

## Executive Summary

This document provides comprehensive research on `cargo test` best practices, output interpretation, and test result capture strategies for Rust projects, specifically tailored for creating a PRP (Problem Resolution Protocol) for the Jin CLI project.

**Current Project Baseline:**
- Total tests: ~650 tests across workspace
- Current status: ~8-12 failing tests (needs verification)
- Test structure: 621 unit tests + integration tests

---

## Table of Contents

1. [Official Documentation URLs](#1-official-documentation-urls)
2. [Cargo Test Command Reference](#2-cargo-test-command-reference)
3. [Test Output Interpretation](#3-test-output-interpretation)
4. [Test Result Capture Strategies](#4-test-result-capture-strategies)
5. [Best Practices for Full Test Suites](#5-best-practices-for-full-test-suites)
6. [Common Gotchas and Pitfalls](#6-common-gotchas-and-pitfalls)
7. [PRP Implementation Guide](#7-prp-implementation-guide)
8. [Real-World Examples from Jin Project](#8-real-world-examples-from-jin-project)

---

## 1. Official Documentation URLs

### Core Rust Testing Documentation

| Resource | URL | Key Sections |
|----------|-----|--------------|
| **Cargo Test Command** | https://doc.rust-lang.org/cargo/commands/cargo-test.html | All flags, options, exit codes |
| **The Rust Book - Testing** | https://doc.rust-lang.org/book/ch11-00-testing.html | How to write and run tests |
| **Test Organization** | https://doc.rust-lang.org/book/ch11-03-test-organization.html | Unit vs integration tests |
| **Rust by Example - Testing** | https://doc.rust-lang.org/rust-by-example/testing.html | Test examples and patterns |
| **Rust Reference - Attributes** | https://doc.rust-lang.org/reference/attributes/testing.html | Test attributes (`#[test]`, `#[ignore]`) |
| **CLI Book - Testing** | https://rust-cli.github.io/book/tutorial/testing.html | CLI-specific testing |

### Section Anchors for Quick Reference

```bash
# Cargo test options
https://doc.rust-lang.org/cargo/commands/cargo-test.html#options

# Test filtering
https://doc.rust-lang.org/cargo/commands/cargo-test.html#test-selection

# Output formats
https://doc.rust-lang.org/cargo/commands/cargo-test.html#output-options

# Test attributes
https://doc.rust-lang.org/reference/attributes/testing.html
```

---

## 2. Cargo Test Command Reference

### Essential Commands for Full Test Suite Execution

#### Basic Full Test Run
```bash
# Run all tests in workspace (stops on first failure)
cargo test --workspace

# Run all tests, continue even if tests fail
cargo test --workspace --no-fail-fast

# Run all tests with verbose output
cargo test --workspace --verbose
```

#### Test Result Capture Commands
```bash
# Capture all output to file
cargo test --workspace > test_output.txt 2>&1

# Capture with exit code preservation
cargo test --workspace 2>&1 | tee test_output.txt

# Capture only summary lines
cargo test --workspace 2>&1 | grep -E "(running|test result:|failures:)"
```

#### JSON Format Output (Rust 1.64+)
```bash
# JSON output for parsing
cargo test --workspace -- --format json

# JSON with timing information
cargo test --workspace -- --format json --report-time
```

### Key Flags Explained

| Flag | Purpose | When to Use |
|------|---------|-------------|
| `--workspace` / `--all` | Test all packages in workspace | **Default for full suite** |
| `--no-fail-fast` | Run all tests even if some fail | **Essential for complete results** |
| `--verbose` / `-v` | Detailed compilation output | Debugging compilation issues |
| `--quiet` / `-q` | Minimal output (one char per test) | Quick progress check |
| `--no-run` | Compile but don't run tests | Verify compilation only |
| `--message-format` | Output format (human, json, etc.) | Automated parsing |
| `-- --list` | List all tests without running | Test inventory |
| `-- --test-threads=N` | Control parallelism | Debugging, resource limits |

### Test Binary Options (after `--`)

```bash
# Show output during tests
cargo test --workspace -- --nocapture

# Run tests sequentially
cargo test --workspace -- --test-threads=1

# Run ignored tests
cargo test --workspace -- --ignored

# Show test execution time
cargo test --workspace -- --report-time

# Format options
cargo test --workspace -- --format=json
cargo test --workspace -- --format=terse
```

---

## 3. Test Output Interpretation

### Standard Test Output Format

```
running 621 tests
test test_name_one ... ok
test test_name_two ... ok
test test_name_three ... FAILED

test result: FAILED. 620 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

### Output Components Breakdown

#### 1. Test Execution Header
```
running 621 tests
```
- **Meaning**: Number of tests in this test binary
- **Note**: Multiple test binaries run separately (lib, each integration test file)

#### 2. Individual Test Results
```
test module::test_name ... ok          # Passed
test module::test_name ... FAILED      # Failed
test module::test_name ... ignored     # Ignored (#[ignore])
```

#### 3. Summary Line
```
test result: <STATUS>. <PASSED> passed; <FAILED> failed; <IGNORED> ignored; <MEASURED> measured; <FILTERED> filtered out
```

**Status Values:**
- `ok` = All tests passed
- `FAILED` = At least one test failed

**Count Categories:**
- `passed` = Successful tests
- `failed` = Failed tests
- `ignored` = Tests marked with `#[ignore]`
- `measured` = Benchmark tests
- `filtered out` = Tests not matching filter pattern

#### 4. Failure Details
```
failures:

---- test_name stdout ----
thread 'test_name' panicked at 'assertion failed: `(left == right)`
  left: `"expected"`,
 right: `"actual"`', src/file.rs:42:9
stack backtrace:
   0: rust_begin_unwind
             at /rustc/library/std/src/panicking.rs:584:5
   ...

failures:
    test_name

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

### Multiple Test Binary Output

```
   Compiling jin v0.1.0 (/path/to/jin)
    Finished dev [unoptimized + debuginfo] target(s) in 2.45s

     Running unittests src/lib.rs (target/debug/deps/jin-XXXXX)

running 621 tests
test result: FAILED. 620 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out

     Running tests/cli_basic.rs (target/debug/deps/cli_basic-XXXXX)

running 15 tests
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running tests/cli_add_local.rs (target/debug/deps/cli_add_local-XXXXX)

running 9 tests
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | All tests passed |
| `1` | One or more tests failed |
| `101` | Test executable failed to compile |

---

## 4. Test Result Capture Strategies

### Strategy 1: Simple File Capture

```bash
# Capture all output
cargo test --workspace > test_results.txt 2>&1

# Check exit code
echo $?  # 0 = success, 1 = failure
```

**Pros:** Simple, preserves all output
**Cons:** Requires parsing to extract summary

### Strategy 2: Summary-Only Capture

```bash
# Capture only summary lines
cargo test --workspace 2>&1 | grep -E "(running|test result:|failures:)" > test_summary.txt

# Example output:
# running 621 tests
# test result: FAILED. 620 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

**Pros:** Easy to parse, shows key metrics
**Cons:** Missing failure details

### Strategy 3: Structured JSON Capture

```bash
# Capture JSON output (Rust 1.64+)
cargo test --workspace -- --format json > test_results.json

# Parse with jq
jq -r 'select(.type == "test") | select(.event == "failed") | .name' test_results.json
```

**JSON Event Types:**
- `{"type": "test", "event": "started", "name": "..."}`
- `{"type": "test", "event": "ok", "name": "..."}`
- `{"type": "test", "event": "failed", "name": "..."}`
- `{"type": "test", "event": "ignored", "name": "..."}`

**Pros:** Machine-readable, parseable
**Cons:** Verbose, requires JSON parsing

### Strategy 4: Tee Capture (Live + File)

```bash
# Show output live and save to file
cargo test --workspace 2>&1 | tee test_results.txt

# With exit code preservation
cargo test --workspace 2>&1 | tee test_results.txt; exit ${PIPESTATUS[0]}
```

**Pros:** See output in real-time, preserve for later
**Cons:** None, this is the recommended approach

### Strategy 5: Automated Parsing Script

```bash
#!/bin/bash
# parse_test_results.sh

OUTPUT_FILE="test_results.txt"
SUMMARY_FILE="test_summary.txt"

# Run tests and capture output
cargo test --workspace --no-fail-fast 2>&1 | tee "$OUTPUT_FILE"

# Extract summary
echo "=== TEST SUMMARY ===" > "$SUMMARY_FILE"
grep -E "(running|test result:)" "$OUTPUT_FILE" >> "$SUMMARY_FILE"

# Count totals
TOTAL_PASSED=$(grep -oP '\d+(?= passed)' "$OUTPUT_FILE" | awk '{s+=$1} END {print s}')
TOTAL_FAILED=$(grep -oP '\d+(?= failed)' "$OUTPUT_FILE" | awk '{s+=$1} END {print s}')
TOTAL_IGNORED=$(grep -oP '\d+(?= ignored)' "$OUTPUT_FILE" | awk '{s+=$1} END {print s}')

echo "" >> "$SUMMARY_FILE"
echo "Total Tests: $((TOTAL_PASSED + TOTAL_FAILED + TOTAL_IGNORED))" >> "$SUMMARY_FILE"
echo "Passed: $TOTAL_PASSED" >> "$SUMMARY_FILE"
echo "Failed: $TOTAL_FAILED" >> "$SUMMARY_FILE"
echo "Ignored: $TOTAL_IGNORED" >> "$SUMMARY_FILE"

# Exit with error if any tests failed
[ "$TOTAL_FAILED" -eq 0 ]
```

### Strategy 6: Baseline Comparison

```bash
#!/bin/bash
# compare_to_baseline.sh

BASELINE_FILE="test_baseline.txt"
CURRENT_FILE="test_results.txt"

# Run tests
cargo test --workspace --no-fail-fast 2>&1 | tee "$CURRENT_FILE"

# Compare with baseline
if [ -f "$BASELINE_FILE" ]; then
    echo "=== BASELINE COMPARISON ==="
    diff -u "$BASELINE_FILE" "$CURRENT_FILE" || true
fi

# Extract current metrics
CURRENT_PASSED=$(grep -oP '\d+(?= passed)' "$CURRENT_FILE" | awk '{s+=$1} END {print s}')
CURRENT_FAILED=$(grep -oP '\d+(?= failed)' "$CURRENT_FILE" | awk '{s+=$1} END {print s}')

echo "Current: $CURRENT_PASSED passed, $CURRENT_FAILED failed"

# Check if within expected range
EXPECTED_TOTAL=650
EXPECTED_FAILED_RANGE="8-12"

CURRENT_TOTAL=$((CURRENT_PASSED + CURRENT_FAILED))
if [ "$CURRENT_TOTAL" -lt "$((EXPECTED_TOTAL - 50))" ] || [ "$CURRENT_TOTAL" -gt "$((EXPECTED_TOTAL + 50))" ]; then
    echo "WARNING: Test count ($CURRENT_TOTAL) differs significantly from baseline (~$EXPECTED_TOTAL)"
fi
```

---

## 5. Best Practices for Full Test Suites

### 1. Always Use `--workspace` for Full Coverage

```bash
# GOOD - Tests all packages
cargo test --workspace

# AVOID - Only tests current package
cargo test

# DEPRECATED - Use --workspace instead
cargo test --all
```

### 2. Use `--no-fail-fast` for Complete Results

```bash
# GOOD - Shows all failures
cargo test --workspace --no-fail-fast

# AVOID - Stops on first failure
cargo test --workspace
```

**Why:** With `--no-fail-fast`, you see ALL failing tests in one run, not just the first one.

### 3. Capture Output for Analysis

```bash
# Recommended pattern
cargo test --workspace --no-fail-fast 2>&1 | tee test_results_$(date +%Y%m%d_%H%M%S).txt
```

### 4. Run Tests Sequentially for Debugging

```bash
# For debugging intermittent failures
cargo test --workspace -- --test-threads=1
```

### 5. Use Consistent Test Order

```bash
# Use exact test names for reproducibility
cargo test --workspace --exact --list

# Run specific test with exact name
cargo test --workspace --exact test_name
```

### 6. Monitor Test Execution Time

```bash
# Show slow tests
cargo test --workspace -- --report-time

# Export timing data
cargo test --workspace -- --format json --report-time
```

### 7. Clean Build Periodically

```bash
# Ensure clean build for accurate results
cargo clean
cargo test --workspace
```

### 8. Use Feature Flags Appropriately

```bash
# Test with all features
cargo test --workspace --all-features

# Test with specific features
cargo test --workspace --features "test-feature-1,test-feature-2"

# Test without default features
cargo test --workspace --no-default-features
```

### 9. Parallel vs Sequential Execution

```bash
# Default: Parallel (fast, but may have race conditions)
cargo test --workspace

# Sequential: Slower, but easier to debug
cargo test --workspace -- --test-threads=1

# Controlled parallelism
cargo test --workspace -- --test-threads=4
```

### 10. CI/CD Best Practices

```bash
# CI pattern: Complete results with artifacts
cargo test --workspace --no-fail-fast --verbose 2>&1 | tee ci_test_results.txt

# Capture exit code for CI
cargo test --workspace --no-fail-fast; TEST_EXIT_CODE=$?

# Upload test results as CI artifact
# (CI-specific configuration)
```

---

## 6. Common Gotchas and Pitfalls

### Gotcha 1: Test Binary Segmentation

**Issue:** Tests run in separate binaries (lib, each integration test file), so you get multiple summary lines.

**Example:**
```
running 621 tests
test result: FAILED. 620 passed; 1 failed

running 15 tests
test result: ok. 15 passed

running 9 tests
test result: ok. 9 passed
```

**Solution:** Aggregate all summary lines for total count.

```bash
# Sum all passed/failed counts
cargo test --workspace 2>&1 | grep "test result:" | \
  awk '{
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

### Gotcha 2: `--all` vs `--workspace`

**Issue:** `--all` is deprecated, use `--workspace` instead.

**Solution:**
```bash
# Use this
cargo test --workspace

# Not this
cargo test --all
```

### Gotcha 3: Exit Codes with Pipes

**Issue:** `cargo test | tee file.txt` always returns 0 because of the pipe.

**Solution:**
```bash
# Use PIPESTATUS
cargo test --workspace 2>&1 | tee test_results.txt
exit ${PIPESTATUS[0]}

# Or use a temporary file
cargo test --workspace > test_results.txt 2>&1
EXIT_CODE=$?
cat test_results.txt
exit $EXIT_CODE
```

### Gotcha 4: Test Output Truncation

**Issue:** Long test output gets truncated.

**Solution:**
```bash
# Increase output buffer
cargo test --workspace -- --nocapture --test-threads=1

# Or capture to file
cargo test --workspace > test_results.txt 2>&1
```

### Gotcha 5: Parallel Test Execution Issues

**Issue:** Tests interfere with each other when running in parallel.

**Solution:**
```bash
# Run sequentially for debugging
cargo test --workspace -- --test-threads=1

# Or use serial_test crate
# #[test]
# #[serial]
# fn test_with_shared_state() { }
```

### Gotcha 6: Filter Pattern Matching

**Issue:** `cargo test test_name` matches any test CONTAINING "test_name", not exact matches.

**Solution:**
```bash
# Exact match
cargo test --workspace -- --exact test_name

# Partial match
cargo test --workspace test_name
```

### Gotcha 7: Ignored Tests Not Run

**Issue:** Tests marked `#[ignore]` are skipped by default.

**Solution:**
```bash
# Run ignored tests
cargo test --workspace -- --ignored

# Run all tests including ignored
cargo test --workspace -- --ignored
```

### Gotcha 8: Workspace vs Package Tests

**Issue:** `cargo test` only tests the current package in a workspace.

**Solution:**
```bash
# Always use --workspace for full coverage
cargo test --workspace

# Or specify specific packages
cargo test --workspace -p package1 -p package2
```

### Gotcha 9: Compilation Warnings Masked

**Issue:** Test output may hide compilation warnings.

**Solution:**
```bash
# Show compilation output
cargo test --workspace --verbose

# Or check compilation separately
cargo build --workspace --verbose
```

### Gotcha 10: Test Count Inflation

**Issue:** Doc tests, unit tests, and integration tests all count separately.

**Example:**
```rust
/// This is a doc test
/// ```
/// assert_eq!(2 + 2, 4);
/// ```
fn example() {}
```

**Solution:**
```bash
# Run only specific test types
cargo test --workspace --lib          # Unit tests only
cargo test --workspace --test '*'     # Integration tests only
cargo test --workspace --doc          # Doc tests only
```

---

## 7. PRP Implementation Guide

### PRP Template for Test Execution

```markdown
# Test Execution PRP

## Objective
Run full test suite, capture results, compare against baseline.

## Commands

### Step 1: Initial Test Run
```bash
# Run all tests, capture output
cargo test --workspace --no-fail-fast 2>&1 | tee test_results_initial.txt
```

### Step 2: Parse Results
```bash
# Extract summary
grep -E "(running|test result:)" test_results_initial.txt > test_summary.txt

# Calculate totals
awk '
  /test result:/ {
    passed += $3
    failed += $5
    ignored += $7
  }
  END {
    print "TOTAL_TESTS=" (passed + failed + ignored)
    print "PASSED=" passed
    print "FAILED=" failed
    print "IGNORED=" ignored
  }
' test_summary.txt > test_metrics.txt
```

### Step 3: Compare Against Baseline
```bash
# Expected: ~650 total tests, ~8-12 failing
source test_metrics.txt

if [ "$TOTAL_TESTS" -lt 600 ] || [ "$TOTAL_TESTS" -gt 700 ]; then
  echo "WARNING: Test count ($TOTAL_TESTS) outside expected range (600-700)"
fi

if [ "$FAILED" -lt 8 ] || [ "$FAILED" -gt 12 ]; then
  echo "INFO: Failed test count ($FAILED) differs from baseline (8-12)"
fi
```

### Step 4: Exit Code Check
```bash
# Exit with error if tests failed
if [ "$FAILED" -gt 0 ]; then
  echo "ERROR: $FAILED tests failed"
  exit 1
fi
```

## Expected Output Format
```
running 621 tests
test result: FAILED. 620 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out

running 15 tests
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Success Criteria
- Total tests: ~650 (±50)
- All test binaries executed
- Results captured to file
- Exit code reflects test status
```

### Automated PRP Script

```bash
#!/bin/bash
set -e

# Configuration
WORKSPACE_DIR="/home/dustin/projects/jin"
OUTPUT_DIR="$WORKSPACE_DIR/test_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_FILE="$OUTPUT_DIR/test_results_$TIMESTAMP.txt"
SUMMARY_FILE="$OUTPUT_DIR/test_summary_$TIMESTAMP.txt"
METRICS_FILE="$OUTPUT_DIR/test_metrics_$TIMESTAMP.txt"

# Expected baseline
EXPECTED_TOTAL=650
EXPECTED_FAILED_MIN=8
EXPECTED_FAILED_MAX=12

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "=== Running Test Suite ==="
echo "Timestamp: $TIMESTAMP"
echo "Workspace: $WORKSPACE_DIR"
echo ""

# Change to workspace directory
cd "$WORKSPACE_DIR"

# Run tests
echo "Running: cargo test --workspace --no-fail-fast"
cargo test --workspace --no-fail-fast 2>&1 | tee "$OUTPUT_FILE"
TEST_EXIT_CODE=${PIPESTATUS[0]}

echo ""
echo "=== Extracting Summary ==="
grep -E "(running|test result:|failures:)" "$OUTPUT_FILE" > "$SUMMARY_FILE"
cat "$SUMMARY_FILE"

echo ""
echo "=== Calculating Metrics ==="
awk -v min="$EXPECTED_FAILED_MIN" -v max="$EXPECTED_FAILED_MAX" '
  /test result:/ {
    gsub(/[,;]/, "")
    passed += $3
    failed += $5
    ignored += $7
    measured += $9
    filtered += $11
  }
  END {
    total = passed + failed + ignored
    print "TOTAL_TESTS=" total
    print "PASSED=" passed
    print "FAILED=" failed
    print "IGNORED=" ignored
    print "MEASURED=" measured
    print "FILTERED=" filtered

    # Validate against baseline
    if (total < 600 || total > 700) {
      print "WARNING: Test count " total " outside expected range (600-700)"
    }

    if (failed < min || failed > max) {
      print "INFO: Failed count " failed " outside baseline range (" min "-" max ")"
    }
  }
' "$SUMMARY_FILE" | tee "$METRICS_FILE"

echo ""
echo "=== Test Complete ==="
echo "Exit Code: $TEST_EXIT_CODE"
echo "Output: $OUTPUT_FILE"
echo "Summary: $SUMMARY_FILE"
echo "Metrics: $METRICS_FILE"

# Exit with test exit code
exit $TEST_EXIT_CODE
```

---

## 8. Real-World Examples from Jin Project

### Current Test Status (as of research)

```bash
$ cargo test --workspace
   Compiling jin v0.1.0 (/home/dustin/projects/jin)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.09s

     Running unittests src/lib.rs (target/debug/deps/jin-8f5c46eea4d762d5)

running 621 tests
test audit::entry::tests::test_audit_context_skips_none ... ok
test audit::entry::tests::test_audit_context_serialization ... ok
# ... 619 more tests ...

failures:

---- core::layer::tests::test_ref_paths stdout ----

thread 'core::layer::tests::test_ref_paths' (3405805) panicked at src/core/layer.rs:289:9:
assertion `left == right` failed
  left: "refs/jin/layers/mode/claude/scope/language/javascript/_"
 right: "refs/jin/layers/mode/claude/scope/language:javascript/_"

failures:
    core::layer::tests::test_ref_paths

test result: FAILED. 620 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
```

### Test Breakdown by Category

| Test Binary | Tests | Status |
|-------------|-------|--------|
| Unit tests (lib) | 621 | 620 passed, 1 failed |
| Integration tests | ~30 | Varies by file |
| **Total** | **~650** | **~8-12 failing** |

### Example Failure Analysis

```bash
# Extract failure details
grep -A 20 "^----" test_results.txt

# Output:
# ---- core::layer::tests::test_ref_paths stdout ----
# thread 'core::layer::tests::test_ref_paths' panicked at src/core/layer.rs:289:9
# assertion `left == right` failed
#   left: "refs/jin/layers/mode/claude/scope/language/javascript/_"
#  right: "refs/jin/layers/mode/claude/scope/language:javascript/_"
```

### Common Failure Patterns in Jin

1. **Ref path format issues** (layer reference paths)
2. **File system path handling** (Windows vs Unix)
3. **Git lock conflicts** (parallel test execution)
4. **Mode/scope validation** (context mismatch)

### Recommended Test Commands for Jin

```bash
# Full test suite with complete results
cargo test --workspace --no-fail-fast 2>&1 | tee jin_test_results.txt

# Run only unit tests
cargo test --workspace --lib

# Run only integration tests
cargo test --workspace --test '*'

# Run specific integration test file
cargo test --workspace --test cli_basic

# Run with backtrace for debugging
RUST_BACKTRACE=1 cargo test --workspace --no-fail-fast

# Run sequentially (avoid git lock issues)
cargo test --workspace -- --test-threads=1
```

---

## Appendix A: Quick Reference Commands

### Essential Commands

```bash
# Run all tests
cargo test --workspace

# Run all tests, don't stop on failure
cargo test --workspace --no-fail-fast

# Capture output to file
cargo test --workspace 2>&1 | tee test_results.txt

# Extract summary only
cargo test --workspace 2>&1 | grep -E "(running|test result:|failures:)"

# Run with verbose output
cargo test --workspace --verbose

# Run with backtrace
RUST_BACKTRACE=1 cargo test --workspace

# Run sequentially
cargo test --workspace -- --test-threads=1

# List all tests
cargo test --workspace -- --list

# Run specific test
cargo test --workspace --exact test_name

# Run ignored tests
cargo test --workspace -- --ignored
```

### Parsing Commands

```bash
# Count total tests
cargo test --workspace 2>&1 | grep -oP '\d+(?= passed)' | awk '{s+=$1} END {print s}'

# Show only failures
cargo test --workspace 2>&1 | grep -A 20 "^----"

# Extract test names
cargo test --workspace -- --list | grep "^[a-z]"

# Compare two test runs
diff <(grep "test result:" run1.txt) <(grep "test result:" run2.txt)
```

---

## Appendix B: Exit Code Handling

### Bash Script Pattern

```bash
#!/bin/bash

# Run tests and preserve exit code
cargo test --workspace --no-fail-fast
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
  echo "All tests passed!"
elif [ $EXIT_CODE -eq 1 ]; then
  echo "Some tests failed!"
else
  echo "Test compilation failed!"
fi

exit $EXIT_CODE
```

### Python Script Pattern

```python
import subprocess
import re

result = subprocess.run(
    ["cargo", "test", "--workspace", "--no-fail-fast"],
    capture_output=True,
    text=True
)

output = result.stdout
exit_code = result.returncode

# Parse results
for line in output.split('\n'):
    if 'test result:' in line:
        # Parse: test result: ok. 620 passed; 1 failed; 0 ignored
        match = re.search(r'(\d+) passed; (\d+) failed; (\d+) ignored', line)
        if match:
            passed, failed, ignored = match.groups()
            print(f"Passed: {passed}, Failed: {failed}, Ignored: {ignored}")

sys.exit(exit_code)
```

---

## Summary

This research document provides:

1. **Official URLs** with section anchors for detailed reference
2. **Complete command reference** for `cargo test` with explanations
3. **Output interpretation guide** with real examples
4. **Six capture strategies** from simple to automated
5. **Best practices** for running full test suites
6. **Ten common gotchas** with solutions
7. **PRP implementation guide** with ready-to-use scripts
8. **Real-world examples** from the Jin project

**Key Takeaways for AI PRP Implementation:**

1. Always use `cargo test --workspace --no-fail-fast` for complete results
2. Capture output with `2>&1 | tee file.txt` for live viewing + preservation
3. Parse summary lines with `grep -E "(running|test result:|failures:)"`
4. Aggregate counts across multiple test binaries
5. Compare against baseline: ~650 total tests, ~8-12 failing
6. Preserve exit codes for CI/CD integration

**Baseline Metrics for Jin Project:**
- Total tests: ~650 (range: 600-700)
- Expected failures: ~8-12 (before fixes)
- Test structure: 621 unit tests + integration tests
- Common issues: Ref paths, file system paths, git locks

---

**Last Updated:** January 12, 2026
**Project:** Jin CLI
**Rust Edition:** 2021
**Cargo Version:** 1.70+
