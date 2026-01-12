# PRP: P1.M2.T2.S1 - Add SIGPIPE Test Instructions to Test Documentation

---

## Goal

**Feature Goal**: Create manual test documentation that enables users to verify SIGPIPE handling works correctly in the jin CLI tool.

**Deliverable**: `tests/manual/SIGPIPE_TEST.md` with step-by-step instructions for manual SIGPIPE testing.

**Success Definition**:
- `tests/manual/SIGPIPE_TEST.md` exists with complete test instructions
- Documentation follows jin's documentation style and conventions
- Instructions are clear and actionable for manual testing
- Test coverage includes the scenarios specified in the contract

---

## User Persona

**Target User**: Developers and contributors who need to verify SIGPIPE handling in the jin CLI tool.

**Use Case**: After implementing the SIGPIPE fix (P1.M2.T1.S1 and P1.M2.T1.S2), users need to manually verify that `jin log` exits gracefully when piped to commands like `head`.

**User Journey**:
1. User builds the release binary
2. User initializes a test project with commits
3. User runs `jin log | head -1` and verifies clean exit
4. User runs comparison tests with `jin log | cat`
5. User confirms no panic messages appear

**Pain Points Addressed**:
- Provides clear, step-by-step verification of SIGPIPE handling
- Documents expected behavior for future maintainers
- Creates a reusable test procedure for regression testing
- Ensures the SIGPIPE fix actually works in practice

---

## Why

- **Verification Required**: SIGPIPE handling code is in place (from P1.M2.T1.S1/S2), but requires manual testing to verify it works correctly
- **Unix Convention Compliance**: Jin should exit silently when piped to commands that close early, matching traditional Unix tools
- **Regression Prevention**: Documentation ensures future changes don't break SIGPIPE handling
- **Developer Onboarding**: Clear test instructions help contributors verify signal handling works
- **User Experience**: Users expect clean pipe behavior; documentation confirms this expectation is met

---

## What

### User-Visible Behavior

This change creates documentation that verifies the following user-visible behavior:

**Expected Behavior** (with SIGPIPE fix):
```bash
$ jin log | head -n 1
commit abc1234 (project-base)
Author: Developer <dev@example.com>
...
$ echo $?
0
```

**Broken Behavior** (without SIGPIPE fix):
```bash
$ jin log | head -n 1
commit abc1234 (project-base)
thread 'main' panicked at 'failed printing to stdout: Broken pipe (os error 32)'
```

### Technical Requirements

1. **Create directory**: `tests/manual/` if it doesn't exist
2. **Create file**: `tests/manual/SIGPIPE_TEST.md` with test instructions
3. **Include sections**:
   - Overview and purpose
   - Prerequisites (build release binary)
   - Test setup (initialize test project)
   - Test scenarios (head, cat, grep, tail)
   - Expected vs unexpected behavior
   - Troubleshooting

4. **Follow documentation conventions**:
   - Use clear, instructional tone
   - Include code blocks with commands
   - Show expected output
   - Explain "what just happened" after key steps
   - Use progressive disclosure (simple to complex)

### Success Criteria

- [ ] `tests/manual/SIGPIPE_TEST.md` file created
- [ ] File follows jin documentation style (from plan/docs/ patterns)
- [ ] All contract test scenarios covered:
  - Build release binary
  - Initialize test project with commits
  - Run `jin log | head -1`
  - Verify no panic, clean exit
  - Run `jin log | cat` for comparison
- [ ] Instructions are clear and actionable
- [ ] File is in valid Markdown format

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context including the previous PRP's outputs, SIGPIPE implementation details, documentation patterns, test scenarios, and comprehensive research references._

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# Contract Definition (exact specifications)
- docfile: tasks.json (P1.M2.T2.S1 context_scope)
  why: Defines the exact contract for this work item
  section: |
    CONTRACT DEFINITION:
    1. RESEARCH NOTE: Manual testing required for SIGPIPE: `jin log | head -1` should exit gracefully.
    2. INPUT: Completed SIGPIPE fix from P1.M2.T1.S1.
    3. LOGIC: Create tests/manual/SIGPIPE_TEST.md with instructions:
       (1) Build release binary;
       (2) Initialize test project with commits;
       (3) Run `jin log | head -1`;
       (4) Verify no panic, clean exit;
       (5) Run `jin log | cat` for comparison.
    4. OUTPUT: Documentation file for manual verification of SIGPIPE handling.
  critical: "The file must be created at tests/manual/SIGPIPE_TEST.md"

# Previous PRP (dependent work item - what this builds upon)
- docfile: plan/P1M2T1S2/PRP.md
  why: Defines the libc dependency that enables SIGPIPE handling
  section: "Goal", "Deliverable", "Implementation Blueprint"
  critical: |
    - libc = "0.2" added to Cargo.toml (line 35)
    - reset_sigpipe() function in src/main.rs (lines 9-17)
    - reset_sigpipe() called at start of main() (line 28)
  dependency: This PRP's code enables the SIGPIPE behavior we are testing

# SIGPIPE Implementation (what we're testing)
- file: /home/dustin/projects/jin/src/main.rs
  why: Shows the SIGPIPE reset implementation we need to test
  pattern: |
    #[cfg(unix)]
    fn reset_sigpipe() {
        unsafe {
            libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        }
    }

    fn main() {
        reset_sigpipe();  // Called BEFORE any other initialization
        // ... rest of main
    }
  critical: SIGPIPE reset happens at startup, before CLI parsing

# Command Under Test (jin log)
- file: /home/dustin/projects/jin/src/commands/log.rs
  why: The command most affected by SIGPIPE (produces large output)
  pattern: |
    // Uses println! extensively - lines 56, 58, 66, 90, 127-134
    // This is why SIGPIPE handling is critical
    // Without reset_sigpipe(), println! panics on broken pipe
  critical: log command outputs commit history, making it pipe-heavy

# Documentation Style Guide (how to write the test doc)
- docfile: plan/docs/GETTING_STARTED.md
  why: Primary example of jin documentation style
  pattern: |
    - Direct address ("you", "your")
    - Action-oriented ("Install Jin", "Create mode")
    - Code blocks with bash language spec
    - "What just happened?" explanations
    - Verification steps after key operations
    - Quick reference sections at end

# Test Documentation Examples
- docfile: plan/docs/TEST_RESULTS.md
  why: Example of test documentation format in the project
  pattern: |
    - Status indicators (PASS/FAIL with emojis)
    - Command tested
    - Expected behavior
    - Actual behavior
    - Issues discovered
    - Workaround recommendations

# SIGPIPE Testing Research
- docfile: plan/P1M2T1S2/research/testing_patterns.md
  why: Contains SIGPIPE testing patterns and templates
  section: "Pattern 1: Basic Manual Test Documentation", "Pattern 7: Documentation for Specific Commands"
  critical: |
    - Test command format
    - Expected vs unexpected behavior
    - Common test scenarios (head, grep, tail, less)
    - Exit code verification

# SIGPIPE Handling Concepts
- docfile: plan/docs/sigpipe_handling_patterns.md
  why: Background on why SIGPIPE handling is needed
  section: "Why SIGPIPE Causes Panics in Rust", "Testing Approaches for SIGPIPE Handling"
  critical: |
    - Rust ignores SIGPIPE by default
    - Writing to closed pipe causes panic with println!
    - reset_sigpipe() restores traditional Unix behavior

# Build Process
- docfile: plan/docs/RELEASE_PROCESS.md
  why: Documents how to build release binaries for testing
  section: "Step 4: Verify the Release", "Installation Methods for Users"
  critical: |
    - Build command: cargo build --release
    - Binary location: target/release/jin
    - Test with: ./target/release/jin --version

# Research: SIGPIPE Testing Best Practices
- docfile: plan/P1M2T1S2/research/README.md
  why: Index of SIGPIPE research with essential URLs and common test commands
  section: "Common Test Commands (Quick Reference)", "Essential URLs"
  sources:
    - https://github.com/uutils/coreutils/issues/8919
    - http://www.pixelbeat.org/programming/sigpipe_handling.html
    - https://github.com/rust-lang/rust/issues/62569

# External Reference: uutils/coreutils SIGPIPE testing
- url: https://github.com/uutils/coreutils/issues/8919
  why: Most comprehensive discussion of SIGPIPE in Rust CLI tools
  section: Test patterns and code examples
  critical: Shows standard test pattern: `cmd | head -n 1`

# External Reference: Pixelbeat SIGPIPE Guide
- url: http://www.pixelbeat.org/programming/sigpipe_handling.html
  why: Comprehensive guide to SIGPIPE handling
  section: Common test cases
  critical: Standard test: `yes | head -n1` demonstrates proper handling
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── Cargo.toml                          # Contains libc = "0.2" dependency (line 35)
├── src/
│   ├── main.rs                         # SIGPIPE reset implementation
│   │   ├── #[cfg(unix)] extern crate libc;     (line 4)
│   │   ├── reset_sigpipe() function             (lines 9-17)
│   │   └── reset_sigpipe() call in main()      (line 28)
│   └── commands/
│       └── log.rs                      # Command under test (uses println!)
├── tests/                              # NO tests/manual/ EXISTS YET
│   ├── cli_add_local.rs                # Existing automated tests
│   ├── cli_basic.rs
│   └── ...                             # Many test files, no manual test dir
└── plan/
    ├── docs/
    │   ├── GETTING_STARTED.md          # Documentation style guide
    │   ├── TEST_RESULTS.md             # Test documentation example
    │   ├── sigpipe_handling_patterns.md    # SIGPIPE background
    │   └── RELEASE_PROCESS.md          # Build instructions
    ├── P1M2T1S1/
    │   └── PRP.md                      # SIGPIPE reset code PRP
    ├── P1M2T1S2/
    │   └── PRP.md                      # libc dependency PRP
    │   └── research/
    │       ├── README.md               # SIGPIPE research index
    │       └── testing_patterns.md     # Testing patterns
    └── P1M2T2S1/
        └── PRP.md                      # THIS FILE
```

### Desired Codebase Tree After This Subtask

```bash
jin/
└── tests/
    └── manual/
        └── SIGPIPE_TEST.md             # NEW: Manual SIGPIPE test instructions
            ├── Overview and purpose
            ├── Prerequisites
            ├── Build release binary
            ├── Test setup
            ├── Test scenarios
            │   ├── Test 1: Pipe to head
            │   ├── Test 2: Pipe to cat (baseline)
            │   ├── Test 3: Pipe to grep
            │   ├── Test 4: Pipe to tail
            │   └── Test 5: Complex pipeline
            ├── Expected vs unexpected behavior
            └── Troubleshooting
```

### Known Gotchas & Library Quirks

```bash
# CRITICAL: Must use release build, not debug build
# Debug builds may behave differently and are slower
# Use: cargo build --release
# Binary: ./target/release/jin

# CRITICAL: Test requires actual commits in the repository
# Empty repository produces no output, can't verify SIGPIPE
# Must initialize with: jin init && git commit

# GOTCHA: jin log may produce no output if no commits exist
# Verify commits exist before testing pipes
# Use: git log to verify repository has commits

# GOTCHA: Exit code 141 (128 + 13) indicates SIGPIPE signal
# This is NORMAL and EXPECTED behavior for Unix tools
# The shell returns 0 for the overall pipeline
# Check with: echo ${PIPESTATUS[0]} on bash

# GOTCHA: stderr vs stdout distinction
# Broken pipe error goes to stderr
# Normal output goes to stdout
# Test stderr separately: jin log 2>&1 | head -1

# CRITICAL: println! macro panics on write errors in Rust
# This is why SIGPIPE reset is required at startup
# Without reset_sigpipe(), any println! after pipe close causes panic
# With reset_sigpipe(), process exits silently on SIGPIPE

# GOTCHA: Platform differences
# SIGPIPE only applies to Unix (Linux, macOS, BSD)
# Windows uses different mechanisms for broken pipes
# Test on Unix platforms only

# GOTCHA: Shell differences for exit codes
# bash: echo ${PIPESTATUS[0]} to get first command's exit code
# zsh: echo ${pipestatus[1]} for first command
# fish: echo $pipestatus[1]
# For compatibility, test visible output (no error messages)

# GOTCHA: head closes pipe immediately after reading requested lines
# This is the classic SIGPIPE trigger
# Other commands: grep -m 1, tail -n 1, less (q to quit)
# Test with multiple pipe consumers

# GOTCHA: Test isolation matters
# Run tests in temporary directory or clean git repo
# Use: mkdir /tmp/jin-sigpipe-test && cd $_
# Avoid testing on active development repository

# CRITICAL: Documentation must be actionable
- Include exact commands to run
- Show expected output
- Explain what to look for
- Provide troubleshooting for common issues
```

---

## Implementation Blueprint

### Data Models and Structure

**No new data models** - This is pure documentation creation.

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE tests/manual/ DIRECTORY
  - CREATE: tests/manual/ directory if it doesn't exist
  - VERIFY: Parent tests/ directory exists
  - PERMISSIONS: Standard directory permissions (0755)
  - DEPENDENCIES: None (first task)

Task 2: CREATE tests/manual/SIGPIPE_TEST.md WITH HEADER SECTIONS
  - CREATE: tests/manual/SIGPIPE_TEST.md file
  - ADD: Title and overview section
  - ADD: Purpose and background section
  - FOLLOW pattern: plan/docs/GETTING_STARTED.md (header style, tone)
  - NAMING: SIGPIPE_TEST.md (uppercase for emphasis, standard for test files)
  - PLACEMENT: tests/manual/SIGPIPE_TEST.md
  - DEPENDENCIES: Task 1 (directory must exist)

Task 3: WRITE PREREQUISITES SECTION
  - ADD: Prerequisites section
  - REQUIREMENTS: Unix-based OS (Linux, macOS, BSD)
  - REQUIREMENTS: Rust toolchain installed
  - REQUIREMENTS: git installed
  - INCLUDE: Links to SIGPIPE background documentation
  - FOLLOW pattern: GETTING_STARTED.md "Prerequisites" section
  - PLACEMENT: After overview, before test instructions
  - DEPENDENCIES: Task 2 (file must exist)

Task 4: WRITE BUILD INSTRUCTIONS
  - ADD: "Build Release Binary" section
  - COMMAND: cargo build --release
  - VERIFY: ./target/release/jin --version
  - EXPLAIN: Why release build is required (performance, correct behavior)
  - FOLLOW pattern: RELEASE_PROCESS.md build instructions
  - PLACEMENT: After prerequisites, before test setup
  - DEPENDENCIES: Task 3

Task 5: WRITE TEST SETUP INSTRUCTIONS
  - ADD: "Test Setup" section
  - STEP 1: Create temporary test directory
  - STEP 2: Initialize git repository
  - STEP 3: Initialize jin with jin init
  - STEP 4: Create test commits (multiple commits for log output)
  - EXPLAIN: Why multiple commits are needed (generates pipe output)
  - FOLLOW pattern: GETTING_STARTED.md step-by-step format
  - PLACEMENT: After build instructions, before test scenarios
  - DEPENDENCIES: Task 4

Task 6: WRITE TEST SCENARIO 1 - PIPE TO HEAD
  - ADD: "Test 1: Pipe to head" section
  - COMMAND: ./target/release/jin log | head -n 1
  - EXPECTED: First log entry, silent exit
  - UNEXPECTED: "Broken pipe" error, panic message
  - VERIFY: echo $? shows 0 (or run without error messages)
  - EXPLAIN: What this tests (head closes pipe early, triggers SIGPIPE)
  - FOLLOW pattern: testing_patterns.md Pattern 1
  - PLACEMENT: First test scenario
  - DEPENDENCIES: Task 5

Task 7: WRITE TEST SCENARIO 2 - PIPE TO CAT (BASELINE)
  - ADD: "Test 2: Pipe to cat (baseline)" section
  - COMMAND: ./target/release/jin log | cat
  - EXPECTED: All log output, normal completion
  - PURPOSE: Establish baseline behavior when pipe doesn't close
  - VERIFY: Full log output displayed
  - EXPLAIN: Comparison test - cat reads all output, no SIGPIPE
  - FOLLOW pattern: testing_patterns.md baseline test concept
  - PLACEMENT: After Test 1
  - DEPENDENCIES: Task 6

Task 8: WRITE ADDITIONAL TEST SCENARIOS
  - ADD: "Test 3: Pipe to grep" section (jin log | grep -m 1 "commit")
  - ADD: "Test 4: Pipe to tail" section (jin log | tail -n 5)
  - ADD: "Test 5: Complex pipeline" section (jin log | head -n 1 | cat)
  - FORMAT: Match Test 1 and Test 2 structure
  - PURPOSE: Test various pipe consumers that close early
  - FOLLOW pattern: testing_patterns.md common test commands
  - PLACEMENT: After Test 2
  - DEPENDENCIES: Task 7

Task 9: WRITE EXPECTED VS UNEXPECTED BEHAVIOR SECTION
  - ADD: "Expected Behavior" section with examples
  - ADD: "Unexpected Behavior (Bugs)" section with examples
  - CONTRAST: Clean exit vs panic with error message
  - INCLUDE: Side-by-side comparison if possible
  - EXPLAIN: What each behavior means
  - FOLLOW pattern: TEST_RESULTS.md status format
  - PLACEMENT: After test scenarios
  - DEPENDENCIES: Task 8

Task 10: WRITE TROUBLESHOOTING SECTION
  - ADD: "Troubleshooting" section
  - ISSUE: "No output from jin log" → No commits in repository
  - ISSUE: "Still seeing Broken pipe errors" → Check release build
  - ISSUE: "Command not found" → Check path to binary
  - ISSUE: "Testing on Windows" → SIGPIPE is Unix-only
  - INCLUDE: Solutions for each issue
  - FOLLOW pattern: GETTING_STARTED.md troubleshooting style
  - PLACEMENT: After expected behavior section, before conclusion
  - DEPENDENCIES: Task 9

Task 11: WRITE SUMMARY AND REFERENCES
  - ADD: "Summary" section with test checklist
  - ADD: "References" section with links
  - INCLUDE: Links to sigpipe_handling_patterns.md
  - INCLUDE: Links to previous PRPs (P1.M2.T1.S1, P1.M2.T1.S2)
  - INCLUDE: External references (uutils/coreutils, pixelbeat)
  - FOLLOW pattern: GETTING_STARTED.md quick reference style
  - PLACEMENT: End of document
  - DEPENDENCIES: Task 10

Task 12: VERIFY DOCUMENTATION COMPLETENESS
  - CHECK: All contract scenarios covered
  - CHECK: Documentation style matches project conventions
  - CHECK: Code blocks are properly formatted
  - CHECK: Commands are copy-paste ready
  - CHECK: File is valid Markdown
  - VALIDATION: Run tests/manual/SIGPIPE_TEST.md instructions manually
  - DEPENDENCIES: Task 11
```

### Implementation Patterns & Key Details

```markdown
# ================== DOCUMENTATION STRUCTURE ==================
# File: tests/manual/SIGPIPE_TEST.md

# SIGPIPE Manual Test

## Overview

[Brief explanation of what SIGPIPE is and why this test exists]

## Prerequisites

[Unix system, Rust toolchain, etc.]

## Build Release Binary

```bash
cargo build --release
./target/release/jin --version
```

**What just happened?**
[Explanation of release build and why it's required]

## Test Setup

[Step-by-step setup of test repository]

## Test Scenarios

### Test 1: Pipe to head

[Command, expected output, verification]

### Test 2: Pipe to cat (baseline)

[Command, expected output, comparison]

[... more tests ...]

## Expected vs Unexpected Behavior

[Clear comparison of good vs bad behavior]

## Troubleshooting

[Common issues and solutions]

## References

[Links to related documentation]

# ================== TONE AND STYLE NOTES ==================
#
# Use direct address: "you", "your"
# Use imperative verbs: "Build", "Run", "Verify"
# Include "What just happened?" explanations after key steps
# Show expected output in code blocks
# Use ✅/❌ emojis for pass/fail indicators if helpful
# Keep instructions copy-paste ready
# Explain WHY, not just WHAT
# Reference related documentation with links
```

### Integration Points

```yaml
DOCUMENTATION_STRUCTURE:
  - file: tests/manual/SIGPIPE_TEST.md
  - format: Markdown
  - style: Follow plan/docs/GETTING_STARTED.md conventions
  - sections: Overview, Prerequisites, Build, Setup, Tests, Troubleshooting, References

BUILD_PROCESS:
  - command: cargo build --release
  - output: target/release/jin
  - verification: ./target/release/jin --version
  - reference: plan/docs/RELEASE_PROCESS.md

SIGPIPE_IMPLEMENTATION:
  - code: src/main.rs reset_sigpipe() function
  - dependency: libc = "0.2" in Cargo.toml
  - platform: Unix-only (Linux, macOS, BSD)
  - reference: plan/P1M2T1S1/PRP.md, plan/P1M2T1S2/PRP.md

TEST_COMMANDS:
  - primary: jin log | head -n 1
  - baseline: jin log | cat
  - additional: jin log | grep -m 1, jin log | tail -n 5
  - reference: plan/P1M2T1S2/research/testing_patterns.md

RESEARCH_REFERENCES:
  - internal: plan/docs/sigpipe_handling_patterns.md
  - internal: plan/P1M2T1S2/research/README.md
  - external: https://github.com/uutils/coreutils/issues/8919
  - external: http://www.pixelbeat.org/programming/sigpipe_handling.html
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after file creation - verify formatting
# Check Markdown syntax
markdownlint tests/manual/SIGPIPE_TEST.md 2>/dev/null || echo "markdownlint not available, skipping"

# Check for broken links (if markdown-link-check available)
markdown-link-check tests/manual/SIGPIPE_TEST.md 2>/dev/null || echo "link checker not available, skipping"

# Verify file is readable
cat tests/manual/SIGPIPE_TEST.md | head -20

# Expected: Clean Markdown with no syntax errors
# Common issues:
# - Unclosed code blocks
# - Broken internal links
# - Malformed tables
```

### Level 2: Content Validation (Documentation Quality)

```bash
# Verify all contract scenarios are covered
grep -q "Build release binary" tests/manual/SIGPIPE_TEST.md
grep -q "head -n 1" tests/manual/SIGPIPE_TEST.md
grep -q "jin log | cat" tests/manual/SIGPIPE_TEST.md

# Expected: All grep commands succeed (find the content)
# If any fail: Content is missing from documentation

# Verify documentation style conventions
grep -q "## Overview" tests/manual/SIGPIPE_TEST.md
grep -q "## Prerequisites" tests/manual/SIGPIPE_TEST.md
grep -q "```bash" tests/manual/SIGPIPE_TEST.md

# Expected: Standard sections and code blocks present
# If missing: Follow GETTING_STARTED.md structure more closely

# Check code block language specification
grep -E '```(bash|sh|rust)' tests/manual/SIGPIPE_TEST.md | wc -l

# Expected: At least 5 code blocks with language specified
# If low: Add language spec to code blocks for syntax highlighting
```

### Level 3: Manual Testing (Instruction Verification)

```bash
# Follow the instructions in the documentation to verify they work

# Step 1: Build release binary (from doc)
cd /tmp/jin-sigpipe-test-verification
cargo build --release

# Step 2: Verify binary exists
test -f ./target/release/jin && echo "Binary built successfully"

# Step 3: Initialize test project (from doc)
mkdir test-repo && cd test-repo
git init
jin init

# Step 4: Create test commits (from doc)
echo "test1" > test.txt
git add test.txt
git commit -m "Test commit 1"
echo "test2" > test2.txt
git add test2.txt
git commit -m "Test commit 2"

# Step 5: Run Test 1 - Pipe to head (from doc)
../../target/release/jin log | head -n 1

# Expected: First log entry displayed, no error messages
# If panic appears: SIGPIPE fix not working (check P1.M2.T1.S1/S2)

# Step 6: Verify no error messages
../../target/release/jin log | head -n 1 2>&1 | grep -i "broken pipe"
# Expected: No output (grep returns exit code 1)
# If found: SIGPIPE handling not working

# Step 7: Run Test 2 - Pipe to cat (baseline)
../../target/release/jin log | cat | wc -l
# Expected: Multiple lines of output (all commits shown)

# Expected: All tests pass with clean output
# If failures: Debug which step failed and update documentation
```

### Level 4: Integration Verification (System Validation)

```bash
# Verify documentation integrates with project structure

# Check file is in correct location
test -f tests/manual/SIGPIPE_TEST.md && echo "File location correct"

# Verify links to related documentation work
# Extract and test links from References section
grep -E 'https?://' tests/manual/SIGPIPE_TEST.md | head -5

# Check references to internal docs exist
test -f plan/docs/sigpipe_handling_patterns.md && echo "Reference exists"
test -f plan/P1M2T1S1/PRP.md && echo "Previous PRP exists"
test -f plan/P1M2T1S2/PRP.md && echo "Dependent PRP exists"

# Verify the test actually tests what it claims
cd /tmp/jin-sigpipe-final-test
cargo build --release
mkdir test && cd test
git init
jin init
for i in {1..10}; do echo "test $i" > "file$i.txt"; git add "file$i.txt"; git commit -m "Commit $i"; done

# Run the documented test commands
../target/release/jin log | head -n 1 > /dev/null 2>&1
EXIT_CODE=$?
if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ Test 1 (pipe to head) passed"
else
    echo "❌ Test 1 failed with exit code $EXIT_CODE"
fi

../target/release/jin log | cat > /dev/null 2>&1
EXIT_CODE=$?
if [ $EXIT_CODE -eq 0 ]; then
    echo "✅ Test 2 (pipe to cat) passed"
else
    echo "❌ Test 2 failed with exit code $EXIT_CODE"
fi

# Expected: All tests pass, documentation is accurate
# If failures: Either documentation is wrong or code has bug
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `tests/manual/SIGPIPE_TEST.md` file created at correct path
- [ ] File is valid Markdown with no syntax errors
- [ ] All code blocks have language specification (```bash, ```rust)
- [ ] All internal links point to existing files
- [ ] All external URLs are accessible
- [ ] File follows project documentation conventions

### Contract Validation

- [ ] "Build release binary" instructions present
- [ ] "Initialize test project with commits" instructions present
- [ ] "Run `jin log | head -1`" test scenario present
- [ ] "Verify no panic, clean exit" verification present
- [ ] "Run `jin log | cat`" comparison test present

### Feature Validation

- [ ] Multiple test scenarios covered (head, cat, grep, tail)
- [ ] Expected behavior clearly documented
- [ ] Unexpected behavior (bugs) documented with examples
- [ ] Troubleshooting section addresses common issues
- [ ] Prerequisites section includes platform requirements (Unix-only)

### Code Quality Validation

- [ ] Documentation tone matches project style (direct, instructional)
- [ ] Step-by-step instructions are clear and actionable
- [ ] "What just happened?" explanations included for key steps
- [ ] Code examples are copy-paste ready
- [ ] References section links to related documentation

### Documentation & Deployment

- [ ] Links to sigpipe_handling_patterns.md included
- [ ] Links to previous PRPs (P1.M2.T1.S1, P1.M2.T1.S2) included
- [ ] External research references (uutils/coreutils, pixelbeat) included
- [ ] Test instructions can be followed by someone unfamiliar with SIGPIPE
- [ ] File is discoverable (in logical location: tests/manual/)

---

## Anti-Patterns to Avoid

- **Don't** create the file in the wrong location (must be `tests/manual/SIGPIPE_TEST.md`)
- **Don't** skip the "Build release binary" step (debug builds behave differently)
- **Don't** forget to create test commits (empty repo produces no log output)
- **Don't** use cargo run instead of target/release/jin binary
- **Don't** skip the baseline test with cat (important comparison)
- **Don't** include Windows-specific workarounds (SIGPIPE is Unix-only)
- **Don't** make the instructions too verbose (keep them copy-paste ready)
- **Don't** forget to explain WHY each step matters
- **Don't** omit troubleshooting for common issues
- **Don't** link to non-existent documentation
- **Don't** use technical jargon without explanation
- **Don't** assume the reader knows what SIGPIPE is

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification**:
- **Simple deliverable**: Single Markdown file creation
- **Well-researched**: Comprehensive SIGPIPE testing patterns from external sources
- **Clear contract**: All test scenarios explicitly specified in tasks.json
- **Established patterns**: Documentation style from GETTING_STARTED.md to follow
- **Existing implementation**: SIGPIPE code already in place (just needs test doc)
- **No code changes**: Pure documentation, no risk of breaking existing functionality
- **Testable outcome**: Can verify documentation works by following instructions

**Zero Risk Factors**:
- Creating documentation cannot break existing code
- File location is new (tests/manual/), no conflicts
- Instructions are based on proven SIGPIPE test patterns
- Can validate by running documented tests

**Current Status**: Ready for implementation - all context gathered, patterns identified, template structure defined

---

## Research Artifacts Location

Research documentation referenced throughout this PRP:

**Primary Research** (from P1.M2.T1.S2):
- `plan/P1M2T1S2/research/README.md` - SIGPIPE research index and quick start
- `plan/P1M2T1S2/research/testing_patterns.md` - Actionable testing patterns

**Background Documentation**:
- `plan/docs/sigpipe_handling_patterns.md` - SIGPIPE handling in Rust CLI tools
- `plan/docs/GETTING_STARTED.md` - Documentation style guide
- `plan/docs/TEST_RESULTS.md` - Test documentation format examples
- `plan/docs/RELEASE_PROCESS.md` - Build instructions

**Previous PRPs** (context for what we're testing):
- `plan/P1M2T1S1/PRP.md` - SIGPIPE reset code implementation
- `plan/P1M2T1S2/PRP.md` - libc dependency addition

**External References**:
- [uutils/coreutils Issue #8919](https://github.com/uutils/coreutils/issues/8919) - SIGPIPE in Rust
- [Pixelbeat SIGPIPE Guide](http://www.pixelbeat.org/programming/sigpipe_handling.html) - Testing patterns
- [Rust Issue #62569](https://github.com/rust-lang/rust/issues/62569) - SIGPIPE discussion

---

## Implementation Status Note

**Ready to implement**: This PRP provides complete context for creating `tests/manual/SIGPIPE_TEST.md`.

**Dependencies already satisfied**:
- SIGPIPE reset code: Implemented in `src/main.rs` (P1.M2.T1.S1)
- libc dependency: Added to `Cargo.toml` (P1.M2.T1.S2)

**What this creates**:
- New directory: `tests/manual/`
- New file: `tests/manual/SIGPIPE_TEST.md`

**Implementation order**:
1. Create `tests/manual/` directory
2. Create `tests/manual/SIGPIPE_TEST.md` following template structure
3. Follow implementation tasks for section-by-section creation
4. Validate with Level 1-4 checks
5. Manually verify by following documented instructions

**Post-implementation verification**:
Follow the instructions in `tests/manual/SIGPIPE_TEST.md` to confirm the SIGPIPE fix works as expected.
