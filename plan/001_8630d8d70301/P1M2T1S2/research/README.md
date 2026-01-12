# SIGPIPE Testing Research - Index

**Task**: P1.M2.T1.S2 - Add libc Dependency to Cargo.toml
**Research Focus**: SIGPIPE testing best practices for Rust CLI tools
**Date**: 2026-01-10

---

## Research Documents

This directory contains comprehensive research on SIGPIPE testing patterns,
documentation best practices, and actionable examples for the jin CLI tool.

### 1. sigpipe_testing_research.md (Main Document)

**Content**: Complete research findings with sources and references

**Includes**:
- How other Rust CLI tools document SIGPIPE handling
- Standard patterns for manual testing documentation
- Common test scenarios (head, tail, grep, etc.)
- GitHub repositories with SIGPIPE test documentation
- Documentation templates for README, CONTRIBUTING, and code comments
- Testing scripts and CI/CD patterns
- Comprehensive list of sources and URLs

**Use when**: You need complete context, sources, or detailed implementation guidance

**Key sections**:
- Section 1: How Other Rust CLI Tools Document SIGPIPE Testing
- Section 2: Standard Patterns for Manual Testing Documentation
- Section 3: Common Test Scenarios for SIGPIPE
- Section 4: GitHub Repositories with SIGPIPE Test Documentation
- Section 5: Documentation Templates for jin CLI
- Section 6: Actionable Recommendations for jin
- Section 7: Sources and References (15+ sources with URLs)

---

### 2. quick_reference.md (Quick Reference Guide)

**Content**: Condensed reference for daily use

**Includes**:
- Common test commands (copy-paste ready)
- Expected vs. unexpected behavior
- Manual testing checklist
- Automated test pattern
- Implementation pattern
- README documentation template
- Key URLs
- Exit codes reference
- Common pitfalls
- Quick test script

**Use when**: You need quick answers while implementing or testing

**Key features**:
- Copy-paste test commands
- Checkbox checklist for manual testing
- One-page reference format
- Immediate application patterns

---

### 3. testing_patterns.md (Actionable Patterns)

**Content**: Specific patterns extracted from research

**Includes**:
- Pattern 1: Basic Manual Test Documentation
- Pattern 2: Automated Test Implementation
- Pattern 3: README Documentation Section
- Pattern 4: Contributing Guide Section
- Pattern 5: Test Script for CI/CD
- Pattern 6: Integration Test Scenarios
- Pattern 7: Documentation for Specific Commands
- Pattern 8: Error Handling Documentation
- Pattern 9: Release Notes Template
- Pattern 10: Code Comments Template

**Use when**: You need specific templates or patterns to implement

**Key features**:
- Ready-to-use templates
- Copy-paste code examples
- Real-world scenarios
- Implementation checklist

---

## Quick Start Guide

### For Implementation

1. **Read**: `quick_reference.md` (5 minutes)
   - Review the "Implementation Pattern" section
   - Copy the `reset_sigpipe()` function template

2. **Implement**: Add to `src/main.rs`
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
       // ... rest of main
   }
   ```

3. **Test**: Use commands from `quick_reference.md`
   ```bash
   jin log | head -n 1
   jin log | grep -m 1 "error"
   jin log | less
   ```

### For Documentation

1. **README**: Use template from `testing_patterns.md` (Pattern 3)
2. **CONTRIBUTING**: Use template from `testing_patterns.md` (Pattern 4)
3. **Code comments**: Use template from `testing_patterns.md` (Pattern 10)
4. **Release notes**: Use template from `testing_patterns.md` (Pattern 9)

### For Testing

1. **Manual tests**: Use checklist from `quick_reference.md`
2. **Automated tests**: Use pattern from `testing_patterns.md` (Pattern 2)
3. **CI/CD**: Use script from `testing_patterns.md` (Pattern 5)
4. **Integration tests**: Use scenarios from `testing_patterns.md` (Pattern 6)

---

## Key Findings Summary

### The Problem

Rust ignores SIGPIPE by default, causing CLI tools to panic with "Broken pipe"
errors when used in Unix pipelines. This is inconsistent with traditional Unix
tools like `cat`, `grep`, and `tail`.

**Example of broken behavior**:
```bash
$ jin log | head -n 1
thread 'main' panicked at 'failed printing to stdout: Broken pipe (os error 32)'
```

**Expected behavior**:
```bash
$ jin log | head -n 1
[First log entry - silent exit]
```

### The Solution

Reset SIGPIPE to SIG_DFL (default handler) at program start using libc:

```rust
#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}
```

### Best Practices

1. **Manual Testing**: Always test with `head`, `tail`, `grep`, and `less`
2. **Automated Testing**: Include `broken_pipe` tests for all stdout-writing commands
3. **Documentation**: Clearly document expected pipe behavior in README
4. **Error Handling**: Distinguish SIGPIPE from real errors
5. **Platform Specifics**: Use `#[cfg(unix)]` for Unix-specific code

---

## Essential URLs

### Core Documentation

- **uutils/coreutils Issue #8919**: https://github.com/uutils/coreutils/issues/8919
  - Most comprehensive discussion of SIGPIPE in Rust CLI tools
  - Includes code examples and test patterns

- **Rust Issue #62569**: https://github.com/rust-lang/rust/issues/62569
  - Discussion of Rust's SIGPIPE handling
  - Context for why the fix is needed

- **Pixelbeat SIGPIPE Guide**: http://www.pixelbeat.org/programming/sigpipe_handling.html
  - Comprehensive guide to SIGPIPE handling
  - Common test cases and anti-patterns

### Testing Resources

- **Rust CLI Book**: https://rust-cli.github.io/book/tutorial/testing.html
  - General testing patterns for CLI tools

- **uutils/coreutils test_cat.rs**: https://github.com/uutils/coreutils/blob/main/tests/by-util/test_cat.rs
  - Example test implementation (lines 122-135)

### Community Discussions

- **Unix StackExchange**: https://unix.stackexchange.com/questions/528844/how-to-portably-test-for-a-sigpipe-failure
  - Testing SIGPIPE portably

- **StackOverflow**: https://stackoverflow.com/questions/108183/how-to-prevent-sigpipes-or-handle-them-properly
  - General SIGPIPE handling discussion

---

## Common Test Commands (Quick Reference)

```bash
# Basic tests
jin log | head -n 1
jin log | head -n 10
jin log | grep -m 1 "error"
jin log | tail -n 20
jin log | less

# Large output tests
jin --verbose 2>&1 | head -n 1

# Complex pipelines
jin log | grep "error" | sort | uniq | head -n 5
```

**Expected**: Silent exit with no error messages
**Unexpected**: "Broken pipe" errors, panics, or stack traces

---

## Implementation Checklist

- [ ] Add `libc` dependency to Cargo.toml
- [ ] Implement `reset_sigpipe()` function
- [ ] Call `reset_sigpipe()` in `main()`
- [ ] Add automated tests for broken pipe
- [ ] Test manually with common pipe scenarios
- [ ] Update README with pipe behavior section
- [ ] Update CONTRIBUTING with testing guidelines
- [ ] Document in CHANGELOG

---

## Document Structure

```
P1M2T1S2/
├── research/
│   ├── README.md                    (This file - index and quick start)
│   ├── sigpipe_testing_research.md  (Complete research with sources)
│   ├── quick_reference.md           (One-page quick reference)
│   └── testing_patterns.md          (Actionable patterns and templates)
└── PRP.md                           (Project requirements document)
```

---

## Next Steps

1. **Review**: Read `quick_reference.md` for overview
2. **Implement**: Add SIGPIPE reset code using pattern from `testing_patterns.md`
3. **Test**: Run manual tests from `quick_reference.md`
4. **Document**: Update README using template from `testing_patterns.md`
5. **Verify**: All tests pass and no broken pipe errors appear

---

## Questions?

Refer to:
- **Implementation details**: `sigpipe_testing_research.md` Section 6
- **Testing patterns**: `testing_patterns.md` Patterns 1-6
- **Documentation templates**: `testing_patterns.md` Patterns 3-4, 7-10
- **Quick reference**: `quick_reference.md`

---

**Research completed**: 2026-01-10
**Researcher**: Claude (Anthropic)
**Task**: P1.M2.T1.S2 - Add libc Dependency to Cargo.toml
