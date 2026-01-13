# PRP Template: Run Full Test Suite and Compare Against Baseline

## Objective
Execute the complete Jin test suite, capture all results, and compare against the established baseline to verify test fixes and overall project health.

## Context
- **Project**: Jin CLI
- **Location**: /home/dustin/projects/jin
- **Baseline Metrics**:
  - Total tests: ~650 (acceptable range: 600-700)
  - Expected failures before fixes: ~8-12
  - Test structure: 621 unit tests + integration tests
- **Test Types**: Unit tests (lib.rs), integration tests (tests/*.rs), doc tests

## Prerequisites
- Rust toolchain installed (1.70+)
- Project dependencies available
- Write access to project directory for test output files

## Instructions

### Step 1: Navigate to Project Directory
```bash
cd /home/dustin/projects/jin
pwd  # Verify: /home/dustin/projects/jin
```

### Step 2: Run Full Test Suite with Complete Result Capture
```bash
# Run all tests in workspace, continue on failure, capture output
cargo test --workspace --no-fail-fast 2>&1 | tee test_results_$(date +%Y%m%d_%H%M%S).txt

# Save the exit code
TEST_EXIT_CODE=$?

# Note: Exit codes:
#   0 = All tests passed
#   1 = One or more tests failed
#   101 = Test executable failed to compile
```

**Expected Output Pattern:**
```
   Compiling jin v0.1.0 (/home/dustin/projects/jin)
    Finished `test` profile [unoptimized + debuginfo] target(s) in X.XXs

     Running unittests src/lib.rs (target/debug/deps/jin-XXXXX)

running 621 tests
test module::test_name ... ok
test module::test_name ... FAILED
# ... more test output ...

test result: FAILED. 620 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out

     Running tests/integration_test.rs (target/debug/deps/integration_test-XXXXX)

running 15 tests
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Step 3: Extract and Display Test Summary
```bash
# Extract summary lines from output
grep -E "(running|test result:|failures:)" test_results_*.txt > test_summary.txt
cat test_summary.txt
```

### Step 4: Calculate Aggregate Metrics
```bash
# Parse all test results and calculate totals
awk '
  /test result:/ {
    # Remove commas and semicolons
    gsub(/[,;]/, "")

    # Sum counts across all test binaries
    passed += $3
    failed += $5
    ignored += $7
    measured += $9
    filtered += $11
  }
  END {
    total = passed + failed + ignored

    print "=== TEST RESULTS SUMMARY ==="
    print "Total tests run: " total
    print "Passed: " passed
    print "Failed: " failed
    print "Ignored: " ignored
    print "Measured: " measured
    print "Filtered out: " filtered
    print ""
    print "Exit code: " '"$TEST_EXIT_CODE"'
  }
' test_summary.txt
```

**Example Output:**
```
=== TEST RESULTS SUMMARY ===
Total tests run: 651
Passed: 643
Failed: 8
Ignored: 0
Measured: 0
Filtered out: 0

Exit code: 1
```

### Step 5: Compare Against Baseline
```bash
# Validate against expected baseline
awk -v total="$TOTAL_TESTS" -v failed="$FAILED_TESTS" '
  BEGIN {
    expected_total = 650
    min_acceptable = 600
    max_acceptable = 700
    expected_failed_min = 8
    expected_failed_max = 12

    print "=== BASELINE COMPARISON ==="

    if (total < min_acceptable || total > max_acceptable) {
      print "WARNING: Test count (" total ") outside acceptable range (" min_acceptable "-" max_acceptable ")"
    } else {
      print "OK: Test count (" total ") within acceptable range"
    }

    if (failed < expected_failed_min) {
      print "EXCELLENT: Failed tests (" failed ") below baseline minimum (" expected_failed_min ")"
      print "This may indicate successful test fixes!"
    } else if (failed > expected_failed_max) {
      print "CONCERN: Failed tests (" failed ") above baseline maximum (" expected_failed_max ")"
    } else {
      print "INFO: Failed tests (" failed ") within expected range (" expected_failed_min "-" expected_failed_max ")"
    }
  }
'
```

### Step 6: Extract Failure Details (if any failed)
```bash
# If there are failures, extract details
if [ "$FAILED_TESTS" -gt 0 ]; then
  echo ""
  echo "=== FAILURE DETAILS ==="
  echo ""

  # Extract test failure names
  grep -A 5 "^failures:" test_results_*.txt | grep "^    " | sort -u

  echo ""
  echo "Full failure output available in test_results_*.txt"
  echo ""
  echo "To view full failure details:"
  echo "  grep -A 20 '^----' test_results_*.txt"
fi
```

### Step 7: Generate Final Report
```bash
# Create comprehensive report
cat > TEST_REPORT.md << 'EOF'
# Test Execution Report

**Date**: $(date)
**Workspace**: /home/dustin/projects/jin
**Command**: cargo test --workspace --no-fail-fast

## Summary

- **Total Tests**: {{TOTAL_TESTS}}
- **Passed**: {{PASSED_TESTS}}
- **Failed**: {{FAILED_TESTS}}
- **Ignored**: {{IGNORED_TESTS}}
- **Exit Code**: {{TEST_EXIT_CODE}}

## Baseline Comparison

- **Expected Total**: ~650 (range: 600-700)
- **Expected Failures**: 8-12 (before fixes)
- **Status**: {{STATUS}}

## Detailed Results

See test_results_*.txt for full output.

## Failure Details

{{FAILURE_DETAILS}}

## Recommendations

{{RECOMMENDATIONS}}
EOF

# Replace placeholders
sed -i "s/{{TOTAL_TESTS}}/$TOTAL_TESTS/g" TEST_REPORT.md
sed -i "s/{{PASSED_TESTS}}/$PASSED_TESTS/g" TEST_REPORT.md
sed -i "s/{{FAILED_TESTS}}/$FAILED_TESTS/g" TEST_REPORT.md
sed -i "s/{{IGNORED_TESTS}}/$IGNORED_TESTS/g" TEST_REPORT.md
sed -i "s/{{TEST_EXIT_CODE}}/$TEST_EXIT_CODE/g" TEST_REPORT.md

# Determine status
if [ "$FAILED_TESTS" -eq 0 ]; then
  sed -i 's/{{STATUS}}/EXCELLENT - All tests passing!/g' TEST_REPORT.md
  sed -i 's/{{RECOMMENDATIONS}}/No action needed. All tests pass./g' TEST_REPORT.md
elif [ "$FAILED_TESTS" -lt 8 ]; then
  sed -i 's/{{STATUS}}/GOOD - Fewer failures than baseline/g' TEST_REPORT.md
  sed -i 's/{{RECOMMENDATIONS}}/Test fixes appear successful. Review remaining failures./g' TEST_REPORT.md
elif [ "$FAILED_TESTS" -le 12 ]; then
  sed -i 's/{{STATUS}}/ACCEPTABLE - Within baseline range/g' TEST_REPORT.md
  sed -i 's/{{RECOMMENDATIONS}}/Test count within expected range. Continue test fix efforts./g' TEST_REPORT.md
else
  sed -i 's/{{STATUS}}/CONCERN - More failures than baseline/g' TEST_REPORT.md
  sed -i 's/{{RECOMMENDATIONS}}/Review new test failures. May have introduced regressions./g' TEST_REPORT.md
fi

# Add failure details if present
if [ "$FAILED_TESTS" -gt 0 ]; then
  grep -A 5 "^failures:" test_results_*.txt | grep "^    " | sort -u | sed 's/^/- /' | sed -i 's/{{FAILURE_DETAILS}}/This file was generated automatically./r /dev/stdin' TEST_REPORT.md
else
  sed -i 's/{{FAILURE_DETAILS}}/No failures./g' TEST_REPORT.md
fi

cat TEST_REPORT.md
```

### Step 8: Verify Exit Code
```bash
# Report final status
echo ""
echo "=== FINAL STATUS ==="

if [ $TEST_EXIT_CODE -eq 0 ]; then
  echo "SUCCESS: All tests passed!"
  echo "Exit code: 0"
elif [ $TEST_EXIT_CODE -eq 1 ]; then
  echo "WARNING: Some tests failed"
  echo "Exit code: 1"
  echo "Review failure details above and in test_results_*.txt"
elif [ $TEST_EXIT_CODE -eq 101 ]; then
  echo "ERROR: Test compilation failed"
  echo "Exit code: 101"
  echo "Review compilation errors in test_results_*.txt"
else
  echo "UNKNOWN: Unexpected exit code"
  echo "Exit code: $TEST_EXIT_CODE"
fi
```

## Success Criteria

### Minimum Acceptable Results
- [ ] All test binaries executed (lib + integration tests)
- [ ] Total test count between 600-700
- [ ] Results captured to file
- [ ] Summary extracted and displayed
- [ ] Exit code correctly reported

### Ideal Results
- [ ] All tests passing (0 failed)
- [ ] Test count matches baseline (~650)
- [ ] No new test failures introduced
- [ ] Report generated successfully

### Failure Indicators
- Total tests < 600 or > 700 (significant change in test suite)
- Compilation errors (exit code 101)
- More than 12 failing tests (regression)
- Test execution interrupted (user cancel, timeout)

## Troubleshooting

### Issue: Test count significantly different from baseline
**Possible causes:**
- Tests added/removed from codebase
- Integration test files added/removed
- Doc tests enabled/disabled

**Action:** Verify recent changes to test files

### Issue: Compilation fails
**Possible causes:**
- Syntax errors in code
- Missing dependencies
- Rust version incompatibility

**Action:**
```bash
# Check compilation errors
cargo build --workspace --verbose

# Check Rust version
rustc --version
```

### Issue: More failures than baseline
**Possible causes:**
- Recent code changes introduced bugs
- Test expectations updated
- Environment-specific failures

**Action:**
```bash
# Review failure details
grep -A 20 "^----" test_results_*.txt

# Run tests sequentially for clearer output
cargo test --workspace --no-fail-fast -- --test-threads=1
```

### Issue: Intermittent test failures
**Possible causes:**
- Parallel test execution conflicts
- File system race conditions
- Timing-dependent tests

**Action:**
```bash
# Run sequentially
cargo test --workspace --no-fail-fast -- --test-threads=1

# Run multiple times to check consistency
for i in {1..3}; do
  echo "Run $i:"
  cargo test --workspace --no-fail-fast
done
```

## Additional Commands for Debugging

### Run specific failing test
```bash
cargo test --workspace --exact test_name -- --nocapture --test-threads=1
```

### Run only integration tests
```bash
cargo test --workspace --test '*'
```

### Run only unit tests
```bash
cargo test --workspace --lib
```

### Run with backtrace
```bash
RUST_BACKTRACE=1 cargo test --workspace --no-fail-fast
```

### List all tests
```bash
cargo test --workspace -- --list
```

## Output Files

After execution, the following files will be generated:

1. **test_results_TIMESTAMP.txt** - Complete test output
2. **test_summary.txt** - Extracted summary lines
3. **TEST_REPORT.md** - Human-readable report

## Related Documentation

- [Comprehensive Research Guide](/home/dustin/projects/jin/plan/001_8630d8d70301/docs/cargo_test_research_prp_guide.md)
- [Quick Reference Card](/home/dustin/projects/jin/plan/001_8630d8d70301/docs/cargo_test_prp_quick_reference.md)
- [Official Cargo Test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)

## Notes

- Always use `--workspace` to ensure all packages are tested
- Always use `--no-fail-fast` to see all failures in one run
- Exit codes are important for CI/CD integration
- Multiple test binaries generate separate summary lines - aggregate for totals
- Baseline metrics should be updated as the project evolves

---

**Template Version**: 1.0
**Last Updated**: January 12, 2026
**Baseline Established**: January 12, 2026
**Next Baseline Review**: After significant test changes
