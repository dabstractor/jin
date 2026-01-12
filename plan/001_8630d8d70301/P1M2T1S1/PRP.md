# PRP: P1.M2.T1.S1 - Add SIGPIPE reset in main.rs

---

## Goal

**Feature Goal**: Fix the SIGPIPE panic that occurs when piping `jin` output to commands like `head` by restoring default Unix signal handling behavior.

**Deliverable**: Modified `src/main.rs` with SIGPIPE reset function and `Cargo.toml` with libc dependency added.

**Success Definition**:
- `jin log | head -n 5` exits cleanly without panic messages
- `cargo build` completes successfully with libc dependency
- Code compiles on both Unix and non-Unix platforms (via `#[cfg(unix)]`)

---

## User Persona

**Target User**: Jin CLI users who pipe command output to other Unix tools (head, tail, grep, etc.)

**Use Case**: User runs `jin log | head -n 10` to view only the first 10 log entries

**User Journey**:
1. User executes: `jin log | head -n 10`
2. Without fix: Rust panics with "failed printing to stdout: Broken pipe" error
3. With fix: Process exits silently after head closes the pipe (traditional Unix behavior)

**Pain Points Addressed**:
- Ugly panic messages clutter terminal when piping output
- Jin behaves differently from traditional Unix tools
- Error messages confuse users who expect normal pipe behavior

---

## Why

- **Fixes Critical Bug**: SIGPIPE causes panic when piping to common Unix tools (head, tail, grep -q)
- **Matches Unix Conventions**: Restores traditional Unix behavior where tools exit silently on broken pipe
- **Improves UX**: Users expect clean output when piping, not error messages
- **Minimal Change**: Small, focused fix that doesn't require refactoring existing code
- **Safe Implementation**: Uses `#[cfg(unix)]` to avoid breaking Windows builds
- **Well-Researched Pattern**: This is the standard approach used by ripgrep and other Rust CLI tools

---

## What

### User-Visible Behavior

**Before Implementation**:
```bash
$ jin log | head -n 5
<5 lines of output>
thread 'main' panicked at 'failed printing to stdout: Broken pipe (os error 32)', ...
```

**After Implementation**:
```bash
$ jin log | head -n 5
<5 lines of output>
<clean exit, no error message>
```

### Technical Requirements

1. **Add libc dependency** to `Cargo.toml`
2. **Add SIGPIPE reset function** in `src/main.rs` with `#[cfg(unix)]`
3. **Call reset function** at the start of `main()` before CLI parsing
4. **Platform-specific compilation**: Unix-only, no-op on other platforms

### Success Criteria

- [ ] `libc = "0.2"` added to Cargo.toml dependencies
- [ ] `reset_sigpipe()` function added to `src/main.rs`
- [ ] Function called at start of `main()` before `jin::cli::Cli::parse()`
- [ ] Code uses `#[cfg(unix)]` for platform-specific compilation
- [ ] `cargo build` succeeds on Unix
- [ ] Manual test `jin log | head -n 5` exits without panic

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context including libc API documentation, existing codebase patterns for cfg attributes, testing approaches, and implementation examples from popular Rust CLI tools._

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# Issue Definition
- docfile: plan/docs/identified_issues.md
  why: Defines the SIGPIPE bug being fixed
  section: "### 2. SIGPIPE Handling in `jin log`"
  critical: "When output is piped to commands like `head`, the process panics"
  error: "failed printing to stdout: Broken pipe"

# Main Entry Point (file to modify)
- file: src/main.rs
  why: The file where SIGPIPE reset will be added
  pattern: |
    //! Jin CLI entry point

    use clap::Parser;

    fn main() -> anyhow::Result<()> {
        let cli = jin::cli::Cli::parse();
        jin::run(cli)
    }
  critical: SIGPIPE reset must be called BEFORE Cli::parse()
  placement: Top of file for function, first line of main() for call

# Cargo.toml (file to modify)
- file: Cargo.toml
  why: Add libc dependency for signal handling
  pattern: |
    [dependencies]
    # CLI
    clap = { version = "4.5", features = ["derive", "cargo"] }
    clap_complete = "4.5"

    # Git operations
    git2 = { version = "0.19", default-features = false, features = ["vendored-libgit2"] }

    # Error handling
    thiserror = "2.0"
    anyhow = "1.0"
  critical: Add "libc = "0.2"" to [dependencies] section
  placement: After "anyhow", alphabetically sorted

# Existing cfg(unix) patterns in codebase
- file: src/staging/workspace.rs (lines 107-117)
  why: Example of platform-specific code pattern to follow
  pattern: |
    #[cfg(unix)]
    pub fn get_file_mode(path: &Path) -> u32 {
        use std::os::unix::fs::PermissionsExt;
        match std::fs::metadata(path) {
            Ok(meta) if meta.permissions().mode() & 0o111 != 0 => 0o100755,
            _ => 0o100644,
        }
    }

    #[cfg(not(unix))]
    pub fn get_file_mode(_path: &Path) -> u32 {
        0o100644
    }
  gotcha: Always provide fallback for non-Unix platforms

# External Research: libc crate documentation
- url: https://docs.rs/libc/latest/libc/fn.signal.html
  why: Official documentation for libc::signal() function
  section: Function signature and usage
  critical: "pub unsafe extern "C" fn signal(signum: c_int, handler: sighandler_t) -> sighandler_t"

- url: https://docs.rs/libc/latest/libc/constant.SIGPIPE.html
  why: SIGPIPE constant documentation
  section: Signal number value
  critical: "pub const SIGPIPE: c_int = 13"

- url: https://docs.rs/libc/latest/libc/constant.SIG_DFL.html
  why: SIG_DFL constant for default signal handling
  section: Handler constant
  critical: "pub const SIG_DFL: sighandler_t"

# External Research: Why SIGPIPE causes panics
- url: https://github.com/rust-lang/rust/issues/62569
  why: Explains why Rust ignores SIGPIPE by default
  section: "Back in 2014, the Rust startup code started ignoring SIGPIPE by default"
  critical: Rust sets SIGPIPE to SIG_IGN before main(), causing write errors instead of termination

# External Research: Testing approaches
- url: https://blog.logrocket.com/guide-signal-handling-rust/
  why: Comprehensive guide on signal handling in Rust
  section: "Handling SIGPIPE"
  critical: Manual testing with pipes: `cargo run | head -n 5`

- url: https://rust-cli.github.io/book/in-depth/signals.html
  why: Rust CLI Book chapter on signal handling
  section: "SIGPIPE"
  critical: Standard pattern for SIGPIPE reset in CLI applications

# Research Artifacts
- docfile: plan/docs/libc_sigpipe.md
  why: Comprehensive research on libc crate usage for SIGPIPE
  section: "Basic Usage Pattern", "Platform-Specific Considerations"
  critical: Complete code examples and best practices

- docfile: plan/docs/sigpipe_handling_patterns.md
  why: Research on SIGPIPE handling patterns in Rust CLI tools
  section: "Common Solutions Used by Popular Rust CLI Tools", "Testing Approaches"
  critical: Real-world examples from ripgrep, bat, and other tools

- docfile: plan/docs/codebase_signal_patterns.md
  why: Existing platform-specific code patterns in Jin codebase
  section: "cfg Attribute Usage", "Unix-Specific Code Patterns"
  critical: Template to follow for cfg(unix) implementation
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── src/
│   ├── main.rs                      # MODIFY: Add SIGPIPE reset
│   ├── cli/
│   │   └── args.rs                  # CLI argument definitions
│   ├── commands/
│   │   ├── log.rs                   # Command affected by SIGPIPE
│   │   ├── apply.rs                 # Has cfg(unix) pattern (line 379)
│   │   └── add.rs                   # Has cfg(unix) tests (line 259)
│   ├── staging/
│   │   └── workspace.rs             # Has cfg(unix) patterns (lines 107-117)
│   └── lib.rs                       # Library entry point
├── Cargo.toml                       # MODIFY: Add libc dependency
├── tests/
│   └── ...                          # Integration tests
└── plan/
    ├── docs/
    │   ├── identified_issues.md     # SIGPIPE issue definition
    │   ├── libc_sigpipe.md          # libc crate research (moved from P1M2T1S1/research/)
    │   ├── sigpipe_handling_patterns.md  # SIGPIPE patterns research (moved from P1M2T1S1/research/)
    │   └── codebase_signal_patterns.md  # Existing patterns in codebase (moved from P1M2T1S1/research/)
    └── P1M2T1S1/
        └── PRP.md                   # THIS FILE
```

### Desired Codebase Tree After This Subtask

```bash
jin/
├── src/
│   └── main.rs                      # MODIFIED: Added reset_sigpipe() function
│       ├── #[cfg(unix)] extern crate libc;
│       ├── #[cfg(unix)] fn reset_sigpipe() { ... }
│       └── main() calls reset_sigpipe() first
└── Cargo.toml                       # MODIFIED: Added libc = "0.2"
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: SIGPIPE reset MUST be called before any I/O operations
// Calling after println! or logging may be too late

// CRITICAL: libc::signal() is unsafe - must wrap in unsafe block
// This is safe because we're using valid constants (SIGPIPE, SIG_DFL)

// CRITICAL: Must use #[cfg(unix)] to prevent compilation on Windows
// libc::SIGPIPE and libc::SIG_DFL may not be defined on Windows

// CRITICAL: extern crate libc; declaration must be #[cfg(unix)]
// Only needed on Unix platforms

// GOTCHA: Rust ignores SIGPIPE by default (sets to SIG_IGN before main)
// This is why we need to explicitly reset to SIG_DFL

// GOTCHA: println! panics on BrokenPipe error when SIGPIPE is ignored
// With SIG_DFL, the process terminates silently instead

// GOTCHA: libc dependency version - use "0.2" not "^0.2" or specific version
// Follows Cargo.toml convention of using bare "0.2"

// GOTCHA: No extern crate libc needed in 2021 edition with use statements
// But still needed for cfg-based extern crate declaration

// GOTCHA: signal() returns previous handler - can check for SIG_ERR
// For simple use case, return value can be ignored

// GOTCHA: Signal handlers are process-wide
// Affects entire process, not just current thread

// GOTCHA: The reset_sigpipe() call must be the VERY FIRST thing in main()
// Before CLI parsing, before logging, before any other initialization
```

---

## Implementation Blueprint

### Data Models and Structure

**No new data models** - This is a straightforward signal handler setup.

**Changes to existing files**:
1. `src/main.rs` - Add SIGPIPE reset function and call it
2. `Cargo.toml` - Add libc dependency

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: MODIFY Cargo.toml
  - ADD: libc dependency to [dependencies] section
  - VERSION: "0.2" (follows existing pattern with other dependencies)
  - PLACEMENT: After "anyhow = "1.0"", before "chrono"
  - PATTERN: Follow existing dependency format
  - BEFORE: |
    [dependencies]
    # Error handling
    thiserror = "2.0"
    anyhow = "1.0"

    # Serialization
    serde = { version = "1.0", features = ["derive"] }
  - AFTER: |
    [dependencies]
    # Error handling
    thiserror = "2.0"
    anyhow = "1.0"

    # System
    libc = "0.2"

    # Serialization
    serde = { version = "1.0", features = ["derive"] }
  - DEPENDENCIES: None (first task)

Task 2: ADD extern crate declaration to main.rs
  - ADD: `#[cfg(unix)] extern crate libc;` at top of file
  - PLACEMENT: After module doc comment, before `use clap::Parser;`
  - PATTERN: Follow cfg(unix) pattern from src/staging/workspace.rs
  - DEPENDENCIES: Task 1 (libc must be in Cargo.toml)

Task 3: ADD reset_sigpipe() function to main.rs
  - ADD: Function with cfg(unix) attribute
  - SIGNATURE: `fn reset_sigpipe()` (no parameters, no return value)
  - BODY: `unsafe { libc::signal(libc::SIGPIPE, libc::SIG_DFL); }`
  - PLACEMENT: After extern crate, before main() function
  - PATTERN: Follow cfg(unix) pattern from src/staging/workspace.rs:107
  - DEPENDENCIES: Task 2 (extern crate libc must be declared)

Task 4: CALL reset_sigpipe() in main()
  - ADD: Function call as first line of main()
  - SYNTAX: `reset_sigpipe();` (also wrapped in #[cfg(unix)])
  - PLACEMENT: Before `let cli = jin::cli::Cli::parse();`
  - CRITICAL: Must be before any CLI parsing or I/O operations
  - DEPENDENCIES: Task 3 (function must be defined)

Task 5: VERIFY COMPILATION
  - COMMAND: cargo build
  - EXPECTED: Successful build with no errors
  - IF FAILS: Check libc dependency version, extern crate syntax
  - DEPENDENCIES: Tasks 1-4 complete

Task 6: MANUAL TESTING
  - COMMAND: cargo run -- log | head -n 5
  - EXPECTED: Clean exit without panic message
  - VERIFY: No "failed printing to stdout" error appears
  - DEPENDENCIES: Task 5 (build must succeed)
```

### Implementation Patterns & Key Details

```rust
// ================== COMPLETE IMPLEMENTATION ==================
// Location: src/main.rs

//! Jin CLI entry point

// STEP 1: Declare libc crate for Unix platforms
#[cfg(unix)]
extern crate libc;

use clap::Parser;

// STEP 2: Define SIGPIPE reset function (Unix only)
#[cfg(unix)]
fn reset_sigpipe() {
    // SAFETY: This is safe because:
    // - SIGPIPE is a valid signal number on all Unix systems
    // - SIG_DFL is a valid handler constant
    // - The call has no other side effects that depend on signal state
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

// STEP 3: For non-Unix platforms, provide no-op implementation
#[cfg(not(unix))]
fn reset_sigpipe() {
    // SIGPIPE doesn't exist on non-Unix platforms
    // Windows handles broken pipes differently via error codes
}

fn main() -> anyhow::Result<()> {
    // STEP 4: Reset SIGPIPE BEFORE any other initialization
    // This must be called before CLI parsing to catch all stdout writes
    reset_sigpipe();

    // STEP 5: Rest of existing main() function
    let cli = jin::cli::Cli::parse();
    jin::run(cli)
}

// ================== EXPLANATION ==================
//
// Why extern crate libc?
// - Required for FFI bindings to C library functions
// - The #[cfg(unix)] ensures it only compiles on Unix platforms
//
// Why unsafe block?
// - All FFI calls to C are inherently unsafe
// - The compiler cannot verify the safety of C function calls
// - This specific call is safe because we use valid constants
//
// Why SIG_DFL (default) instead of SIG_IGN (ignore)?
// - Rust's default is SIG_IGN, which causes the panic we're fixing
// - SIG_DFL restores traditional Unix behavior: terminate on broken pipe
// - This matches the behavior of traditional Unix tools (grep, cat, etc.)
//
// Why call at start of main()?
// - Must be set before any stdout writes occur
// - CLI parsing might trigger writes, so we call it first
// - Signal handlers are process-wide, so once is enough
//
// Why provide no-op for non-Unix?
// - Ensures code compiles on all platforms
// - Windows handles broken pipes differently (via error codes, not signals)
// - The function call is harmless on non-Unix platforms
```

### Integration Points

```yaml
CARGO_TOML:
  - file: Cargo.toml
  - section: [dependencies]
  - addition: |
    # System
    libc = "0.2"

MAIN_RS:
  - file: src/main.rs
  - changes:
    - Add extern crate declaration (after doc comment)
    - Add reset_sigpipe() function (before main())
    - Call reset_sigpipe() (first line in main())

NO_OTHER_CHANGES:
  - No changes to src/commands/log.rs
  - No changes to src/lib.rs
  - No changes to any other files
  - This is a localized fix to the entry point
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after making changes - fix before proceeding
cargo build                          # Compile the project
cargo check                          # Quick compile check

# Expected: Zero compilation errors. If errors exist, READ output and fix.
# Common issues:
# - Missing extern crate libc (error: unresolved extern crate)
# - Wrong libc version (error: failed to select a version)
# - Missing unsafe block (error: call to unsafe function requires unsafe block)
```

### Level 2: Manual Testing (Functional Validation)

```bash
# Test SIGPIPE handling with pipe to head
cargo run -- log | head -n 5

# Expected: Clean exit with only 5 lines of log output
# Should NOT see: "thread 'main' panicked at 'failed printing to stdout"

# Test with other pipe scenarios
cargo run -- log | head -n 1
cargo run -- log | tail -n 5

# Test that normal operation still works
cargo run -- log

# Expected: All commands exit cleanly without panic messages
```

### Level 3: Cross-Platform Validation

```bash
# Unix/Linux/macOS
cargo build                          # Should succeed with libc
cargo test                           # All tests should pass

# Windows (if available)
cargo build                          # Should succeed (cfg(unix) excludes code)
cargo test                           # All tests should pass

# Expected: Clean builds on all platforms
```

### Level 4: Integration Testing (System Validation)

```bash
# Test all jin commands that produce output
cargo run -- status | head -n 5
cargo run -- list | head -n 5
cargo run -- config --list | head -n 5

# Test pipe chains
cargo run -- log | grep "commit" | head -n 3

# Test with output redirection
cargo run -- log > /tmp/output.txt
head -n 5 /tmp/output.txt

# Expected: All commands work correctly with pipes
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `Cargo.toml` includes `libc = "0.2"` in dependencies
- [ ] `src/main.rs` has `#[cfg(unix)] extern crate libc;` declaration
- [ ] `src/main.rs` has `#[cfg(unix)] fn reset_sigpipe()` function
- [ ] `src/main.rs` has `#[cfg(not(unix))] fn reset_sigpipe()` fallback
- [ ] `main()` calls `reset_sigpipe()` as first statement
- [ ] `cargo build` succeeds with zero errors
- [ ] Code follows existing cfg(unix) patterns from codebase

### Feature Validation

- [ ] `jin log | head -n 5` exits without panic messages
- [ ] Only first N lines appear, no error output
- [ ] Traditional Unix behavior restored (silent exit on broken pipe)
- [ ] No regressions in normal jin log output
- [ ] All pipe scenarios work correctly

### Code Quality Validation

- [ ] Function placement follows codebase conventions
- [ ] cfg(unix) usage matches existing patterns
- [ ] unsafe block properly scoped
- [ ] No unsafe code on non-Unix platforms
- [ ] Changes are minimal and focused

### Documentation & Deployment

- [ ] Code is self-documenting with clear function name
- [ ] No additional documentation needed (transparent to users)

---

## Anti-Patterns to Avoid

- **Don't** skip the `#[cfg(unix)]` attribute - code will fail to compile on Windows
- **Don't** forget the `unsafe` block - libc::signal() requires it
- **Don't** call reset_sigpipe() after CLI parsing - must be first thing in main()
- **Don't** use a specific libc version like "0.2.155" - use "0.2" for compatibility
- **Don't** add error handling for libc::signal() return value - not needed for this use case
- **Don't** put reset_sigpipe() in a different module - keep it in main.rs for simplicity
- **Don't** try to handle SIGPIPE in lib.rs - this is a CLI entry point concern only
- **Don't** add tests for signal handling - manual testing with pipes is sufficient
- **Don't** modify src/commands/log.rs - the fix is at the entry point only
- **Don't** use the sigpipe crate - libc dependency is sufficient and standard

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification**:
- **Clear Specification**: Contract specifies exact code to add
- **Minimal Changes**: Only 2 files modified (main.rs, Cargo.toml)
- **Well-Researched**: Comprehensive research on libc, SIGPIPE, and cfg patterns
- **Existing Patterns**: Codebase has proven cfg(unix) patterns to follow
- **Standard Solution**: This is the standard approach used by ripgrep and other CLI tools
- **Simple Testing**: Manual test with pipe is straightforward
- **No Complex Logic**: Just a function declaration and call
- **Platform-Safe**: cfg(unix) prevents Windows build issues

**Zero Risk Factors**:
- Changes are isolated to entry point
- No interaction with existing business logic
- Well-documented libc API
- Existing codebase patterns to follow

**Current Status**: Ready for implementation - all context and patterns are clear

---

## Research Artifacts Location

Research documentation stored at: `plan/docs/`

**Key File References**:
- `src/main.rs` - Entry point to modify (currently 8 lines)
- `Cargo.toml` - Dependencies file (add libc = "0.2")
- `src/staging/workspace.rs:107-117` - cfg(unix) pattern reference
- `plan/docs/identified_issues.md` - SIGPIPE issue definition
- `plan/docs/libc_sigpipe.md` - libc crate API research
- `plan/docs/sigpipe_handling_patterns.md` - SIGPIPE handling patterns
- `plan/docs/codebase_signal_patterns.md` - Existing codebase patterns

**External References** (from research):
- [libc::signal() documentation](https://docs.rs/libc/latest/libc/fn.signal.html)
- [Should Rust still ignore SIGPIPE by default?](https://github.com/rust-lang/rust/issues/62569)
- [Rust CLI Book - Signal Handling](https://rust-cli.github.io/book/in-depth/signals.html)
