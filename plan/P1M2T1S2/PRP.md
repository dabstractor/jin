# PRP: P1.M2.T1.S2 - Add libc Dependency to Cargo.toml

---

## Goal

**Feature Goal**: Enable the libc crate for Unix signal handling by adding it as a dependency in Cargo.toml.

**Deliverable**: Modified `Cargo.toml` with `libc = "0.2"` added to the `[dependencies]` section.

**Success Definition**:
- `libc = "0.2"` is present in Cargo.toml under [dependencies]
- `cargo build` completes successfully
- libc crate is available for import in src/main.rs (via `extern crate libc;`)

---

## User Persona

**Target User**: Jin CLI users who pipe command output to Unix tools (head, tail, grep, etc.)

**Use Case**: User runs `jin log | head -n 10` and expects clean Unix-style behavior

**User Journey**:
1. User executes: `jin log | head -n 10`
2. Without libc: Code cannot compile because signal() functions are unavailable
3. With libc: Code compiles and SIGPIPE handling works correctly

**Pain Points Addressed**:
- Enables compilation of SIGPIPE reset code (from P1.M2.T1.S1)
- Provides access to Unix signal handling functions (signal(), SIGPIPE, SIG_DFL)
- Matches standard Rust CLI tool patterns for signal handling

---

## Why

- **Required for SIGPIPE Handling**: The libc crate provides FFI bindings to C library functions needed for signal handling
- **Standard Rust Pattern**: This is the standard approach used by ripgrep, bat, and other Rust CLI tools
- **Minimal Dependency**: libc is a minimal, well-maintained crate that only provides FFI bindings (no complex logic)
- **Cross-Platform**: Uses `#[cfg(unix)]` attributes to exclude code on non-Unix platforms
- **Well-Supported**: libc 0.2.x is de facto stable despite being pre-1.0, maintained for years
- **No Alternatives Needed**: The `sigpipe` crate exists but is unnecessary for this simple use case

---

## What

### User-Visible Behavior

**This change has no direct user-visible behavior** - it enables compilation of the SIGPIPE reset code added in P1.M2.T1.S1.

### Technical Requirements

1. **Add libc dependency** to `Cargo.toml` [dependencies] section
2. **Version specifier**: Use `"0.2"` (accepts any 0.2.x version, protects against 0.3.0 breaking changes)
3. **Placement**: In the "# System" group, after "# Error handling" group, before "# Serialization" group
4. **Format**: Follow existing dependency format: `libc = "0.2"`

### Success Criteria

- [ ] `libc = "0.2"` added to Cargo.toml [dependencies] section
- [ ] Placed in logical location (System group)
- [ ] Follows existing formatting (no unnecessary features)
- [ ] `cargo build` succeeds with no errors
- [ ] Code in src/main.rs can import libc via `extern crate libc;`

---

## All Needed Context

### Context Completeness Check

_This PRP provides complete context including libc version compatibility documentation, Cargo.toml pattern analysis, existing codebase conventions, and comprehensive research on signal handling patterns._

**IMPORTANT NOTE**: This change may already be implemented in the codebase. The PRP documents the required change for reference and verification purposes.

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# Issue Definition (why this change is needed)
- docfile: plan/docs/identified_issues.md
  why: Defines the SIGPIPE bug that requires libc for signal handling
  section: "### 2. SIGPIPE Handling in `jin log`"
  critical: "Fix Required: 1. Add SIGPIPE signal handler in main.rs"

# Contract Definition (exact specifications)
- docfile: tasks.json (P1.M2.T1.S2 context_scope)
  why: Defines the exact contract for this work item
  section: |
    CONTRACT DEFINITION:
    1. RESEARCH NOTE: Cargo.toml is at project root
    2. INPUT: None - standalone change
    3. LOGIC: Add `libc = "0.2"` under [dependencies] section
    4. OUTPUT: libc crate available for import in src/main.rs
  critical: "This provides the signal() function and SIGPIPE constant for Unix platforms"

# Previous PRP (dependent work item)
- docfile: plan/P1M2T1S1/PRP.md
  why: Defines the SIGPIPE reset code that requires libc dependency
  section: Task 1 (MODIFY Cargo.toml)
  critical: "Task 1: MODIFY Cargo.toml - ADD: libc dependency to [dependencies] section"
  dependency: This PRP's Task 1 adds the dependency that P1.M2.T1.S1 Task 2 requires

# Main Entry Point (uses libc via extern crate)
- file: /home/dustin/projects/jin/src/main.rs
  why: Shows how libc is used after being added to Cargo.toml
  pattern: |
    #[cfg(unix)]
    extern crate libc;

    #[cfg(unix)]
    fn reset_sigpipe() {
        unsafe {
            libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        }
    }
  critical: libc must be in Cargo.toml before extern crate will compile
  placement: Lines 3-4 (extern crate), lines 8-17 (reset_sigpipe function)

# Cargo.toml (file to modify)
- file: /home/dustin/projects/jin/Cargo.toml
  why: The file where libc dependency will be added
  current_state: Line 35 already has `libc = "0.2"` (may already be implemented)
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

    # System
    libc = "0.2"

    # Serialization
    serde = { version = "1.0", features = ["derive"] }
  critical: Add "libc = "0.2"" to [dependencies] section in "# System" group
  placement: After "anyhow = "1.0"", before "serde", with "# System" comment

# System Architecture Context
- docfile: plan/docs/system_context.md
  why: Provides project context and existing dependencies
  section: "Key Dependencies"
  critical: Shows current dependency patterns: clap 4.5, git2 0.19, serde, thiserror 2.0

# Research: libc Version Patterns
- docfile: plan/P1M2T1S2/research/libc_version_patterns.md
  why: Explains why "0.2" is the correct version specifier
  section: "Version Compatibility - '0.2' Meaning", "Current libc Crate Status"
  critical: |
    - "0.2" accepts any 0.2.x version (e.g., 0.2.155, 0.2.178)
    - Protects against breaking 0.3.0 changes
    - libc 0.2.x is de facto stable despite being pre-1.0
    - Latest version: 0.2.178
  sources:
    - https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html
    - https://doc.rust-lang.org/cargo/reference/semver.html
    - https://docs.rs/libc
    - https://crates.io/crates/libc

# Research: Cargo.toml Patterns
- docfile: plan/P1M2T1S2/research/cargo_toml_patterns.md
  why: Documents Jin's existing Cargo.toml conventions to follow
  section: "Jin Project's Current Patterns Analysis"
  critical: |
    - Functional grouping with comments (e.g., "# System")
    - Alphabetical ordering within groups
    - Major.Minor version format (e.g., "0.2", "4.5", "1.0")
    - Consistent spacing and formatting
  placement: In "# System" group, between "# Error handling" and "# Serialization"

# Research: libc::signal() Function
- docfile: plan/P1M2T1S2/research/libc_signal_function.md
  why: Documents the signal() function that libc provides
  section: "Function Signature", "SIG_DFL Constant"
  critical: |
    - Function: pub unsafe extern "C" fn signal(c_int, sighandler_t) -> sighandler_t
    - SIGPIPE constant: pub const SIGPIPE: c_int = 13
    - SIG_DFL constant: pub const SIG_DFL: sighandler_t = 0
    - Safety: Must wrap in unsafe block (FFI call to C)

# External Research: libc Crate Documentation
- url: https://docs.rs/libc/latest/libc/
  why: Official libc crate documentation
  section: Crate overview and available functions
  critical: "libc - Raw FFI bindings to platform libraries like libc"

- url: https://crates.io/crates/libc
  why: libc crate page on crates.io
  section: Version history and download stats
  critical: "Latest version: 0.2.178, 400M+ downloads"

- url: https://docs.rs/libc/latest/libc/fn.signal.html
  why: signal() function documentation
  section: Function signature and safety requirements
  critical: "pub unsafe extern "C" fn signal(signum: c_int, handler: sighandler_t) -> sighandler_t"

- url: https://github.com/rust-lang/libc
  why: libc crate source repository
  section: Platform support and contribution guidelines
  critical: "Provides all core FFI bindings to standard C libraries"

# External Research: Cargo Dependency Specification
- url: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html
  why: Official Cargo documentation on dependency versioning
  section: "Version specifier formats", "Caret requirements (^)"
  critical: |
    - "0.2" is shorthand for "^0.2.0"
    - Accepts 0.2.x but not 0.3.0
    - Compatible with SemVer for 0.x versions

# External Research: SemVer Compatibility
- url: https://doc.rust-lang.org/cargo/reference/semver.html
  why: Explains how Cargo interprets version numbers
  section: "Compatibility for pre-1.0 versions"
  critical: |
    - 0.0.x is incompatible with 0.0.y if x != y
    - 0.x.y is incompatible with 0.x.z if y != z
    - For 0.x, updates to 0.(x+1).0 are considered breaking
```

### Current Codebase Tree (Relevant Portion)

```bash
jin/
├── Cargo.toml                          # MODIFY: Add libc dependency
├── src/
│   ├── main.rs                         # Uses libc via extern crate
│   │   ├── #[cfg(unix)] extern crate libc;  (line 4)
│   │   └── reset_sigpipe() function    (lines 8-23)
│   ├── cli/
│   │   └── args.rs                     # CLI argument definitions
│   ├── commands/
│   │   └── log.rs                      # Command affected by SIGPIPE
│   └── lib.rs                          # Library entry point
└── plan/
    ├── docs/
    │   ├── identified_issues.md        # SIGPIPE issue definition
    │   └── system_context.md           # Project architecture
    ├── P1M2T1S1/
    │   └── PRP.md                      # Related PRP (uses libc)
    └── P1M2T1S2/
        └── PRP.md                      # THIS FILE
```

### Desired Codebase Tree After This Subtask

```bash
jin/
└── Cargo.toml                          # MODIFIED: Added libc = "0.2"
    └── [dependencies]
        ├── # Error handling
        │   ├── thiserror = "2.0"
        │   └── anyhow = "1.0"
        ├── # System
        │   └── libc = "0.2"            # ADDED: Unix signal FFI bindings
        └── # Serialization
            └── serde = { version = "1.0", features = ["derive"] }
```

### Known Gotchas & Library Quirks

```toml
# CRITICAL: Use "0.2" not "0.2.155" or specific versions
# "0.2" allows any 0.2.x version (currently up to 0.2.178)
# This protects against breaking 0.3.0 changes while allowing compatible updates

# CRITICAL: libc is Unix-only (no Windows support)
# Windows must use #[cfg(unix)] to exclude libc-related code
# Windows uses different APIs (winapi or windows-sys crates)

# CRITICAL: No features needed for libc
# libc provides FFI bindings only - no features to enable
# Do not add: libc = { version = "0.2", features = [...] }

# GOTCHA: libc 0.2.x is pre-1.0 but de facto stable
# Despite being 0.x, libc 0.2.x has been stable for years
# Major Rust projects use libc = "0.2" without issues

# GOTCHA: extern crate libc; is required for 2021 edition with cfg attribute
# Use: #[cfg(unix)] extern crate libc;
# This ensures libc is only linked on Unix platforms

# GOTCHA: libc::signal() is an unsafe FFI function
# Must wrap in unsafe block: unsafe { libc::signal(...); }
# This is safe when using valid constants (SIGPIPE, SIG_DFL)

# GOTCHA: SIGPIPE constant value varies by platform (usually 13)
# Use libc::SIGPIPE constant, not hardcoded values
# This ensures correct value for each Unix platform

# GOTCHA: SIG_DFL is different from SIG_IGN
# SIG_DFL (0): Default behavior - terminates process on SIGPIPE
# SIG_IGN (1): Ignore signal - returns EPIPE error on write
# We want SIG_DFL to restore traditional Unix behavior

# GOTCHA: Cargo.toml dependency order matters for readability
# Follow Jin's convention: functional groups with comments
# Add "# System" comment above libc = "0.2"
# Place after "# Error handling" group, before "# Serialization" group

# GOTCHA: No need for [target.'cfg(unix)'.dependencies] section
# Adding libc to main [dependencies] is sufficient
# Use #[cfg(unix)] in Rust code to exclude on non-Unix platforms

# GOTCHA: Alternative crates exist but are unnecessary
# sigpipe crate: Adds SIGPIPE reset automatically (overkill for our needs)
# nix crate: Safer Unix bindings (heavier dependency)
# signal-hook crate: More complex signal handling (unnecessary here)
# libc is the minimal, standard choice for this use case
```

---

## Implementation Blueprint

### Data Models and Structure

**No new data models** - This is a simple dependency addition.

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: VERIFY CURRENT STATE OF CARGO.TOML
  - CHECK: Read Cargo.toml to see if libc = "0.2" already exists
  - IF PRESENT: Skip to Task 3 (verification)
  - IF ABSENT: Proceed to Task 2
  - LOCATION: Line 35 in current Cargo.toml
  - DEPENDENCIES: None (first task)

Task 2: ADD LIBC DEPENDENCY TO CARGO.TOML
  - ADD: "# System" comment section if not present
  - ADD: libc = "0.2" under [dependencies] section
  - PLACEMENT: After "# Error handling" group, before "# Serialization" group
  - FORMAT: Follow existing pattern: `libc = "0.2"`
  - PATTERN: Match surrounding dependencies (thiserror, anyhow, serde)
  - DEPENDENCIES: Task 1 (verification that it's not already present)
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

Task 3: VERIFY COMPILATION
  - COMMAND: cargo build
  - EXPECTED: Successful build with no errors
  - VERIFY: No "error: unresolved extern crate `libc`" messages
  - IF FAILS: Check that libc = "0.2" is properly formatted in Cargo.toml
  - DEPENDENCIES: Task 2 (libc must be in Cargo.toml)

Task 4: VERIFY LIBC IS IMPORTABLE
  - CHECK: src/main.rs has `#[cfg(unix)] extern crate libc;`
  - VERIFY: Code compiles without "unresolved extern crate" errors
  - TEST: cargo build succeeds
  - DEPENDENCIES: Task 3 (build must succeed)

Task 5: VERIFY CARGO LOCK UPDATE
  - CHECK: Cargo.lock file includes libc package entry
  - VERIFY: Version is in 0.2.x range (e.g., 0.2.155, 0.2.178)
  - RUN: cargo build to generate/update Cargo.lock
  - DEPENDENCIES: Task 3 (build must succeed)
```

### Implementation Patterns & Key Details

```toml
# ================== COMPLETE IMPLEMENTATION ==================
# Location: Cargo.toml, [dependencies] section

# BEFORE (lines 22-42):
[dependencies]
# CLI
clap = { version = "4.5", features = ["derive", "cargo"] }
clap_complete = "4.5"

# Git operations
git2 = { version = "0.19", default-features = false, features = ["vendored-libgit2"] }

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
toml = "0.8"
rust-ini = "0.21"

# Data structures
indexmap = { version = "2.0", features = ["serde"] }

# Text merging
diffy = "0.4"

# Utilities
dirs = "5.0"
chrono = { version = "0.4", features = ["serde"] }
regex = "1.10"

# AFTER (lines 22-45):
[dependencies]
# CLI
clap = { version = "4.5", features = ["derive", "cargo"] }
clap_complete = "4.5"

# Git operations
git2 = { version = "0.19", default-features = false, features = ["vendored-libgit2"] }

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# System
libc = "0.2"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
toml = "0.8"
rust-ini = "0.21"

# Data structures
indexmap = { version = "2.0", features = ["serde"] }

# Text merging
diffy = "0.4"

# Utilities
dirs = "5.0"
chrono = { version = "0.4", features = ["serde"] }
regex = "1.10"

# ================== EXPLANATION ==================
#
# Why "# System" comment?
# - Follows Jin's convention of functional grouping
# - Makes it clear this is a system-level FFI dependency
# - Improves readability for other developers
#
# Why "0.2" not "0.2.155"?
# - Allows compatible updates (0.2.155 → 0.2.178)
# - Protects against breaking 0.3.0 changes
# - Follows Rust ecosystem best practices
# - Matches Jin's other dependencies (clap "4.5", git2 "0.19")
#
# Why this placement?
# - After "Error handling" (anyhow)
# - Before "Serialization" (serde)
# - Logical flow: low-level dependencies first
# - Maintains alphabetical ordering within groups
#
# Why no features?
# - libc provides FFI bindings only
# - No features to enable
# - Keep dependency simple and minimal
#
# Why not in [target.'cfg(unix)'.dependencies]?
# - Unnecessary complexity
# - #[cfg(unix)] in Rust code is sufficient
# - Cargo handles platform-specific linking automatically
```

### Integration Points

```yaml
CARGO_TOML:
  - file: Cargo.toml
  - section: [dependencies]
  - modification: Add "# System" group and "libc = "0.2"" line
  - location: After line 32 (anyhow), before line 34 (serde)

SRC_MAIN_RS:
  - file: src/main.rs
  - usage: Imports libc via #[cfg(unix)] extern crate libc;
  - dependency: Requires libc in Cargo.toml to compile

CARGO_LOCK:
  - file: Cargo.lock
  - effect: Automatically updated with libc package entry
  - version: Will resolve to latest 0.2.x compatible version

NO_OTHER_CHANGES:
  - No changes to any source files
  - No changes to build configuration
  - No changes to tests
  - This is a pure dependency addition
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after adding dependency - fix before proceeding
cargo check                          # Quick compile check (faster than full build)
cargo build                          # Full build to verify dependency resolution

# Expected: Zero compilation errors.
# Common issues to check:
# - If "error: unresolved extern crate `libc`" → dependency not added correctly
# - If "error: failed to select a version for libc" → version specifier wrong
# - If "error: could not find libc in registry" → network or registry issue

# Verify Cargo.toml syntax
cargo toml validate                  # If cargo-toml tool is available

# Expected: Clean validation with no syntax errors
```

### Level 2: Dependency Verification (Component Validation)

```bash
# Verify libc is properly registered
cargo tree | grep libc                # Check dependency tree includes libc

# Expected output: libc v0.2.x (where x is latest, e.g., 155, 178)
# If missing: Dependency not added correctly

# Verify specific version
cargo tree -p libc                    # Show libc dependency details

# Expected: Shows libc version and features (should be none)

# Check Cargo.lock was updated
grep -A 5 'name = "libc"' Cargo.lock

# Expected: Entry showing libc package with checksum and version
```

### Level 3: Integration Testing (System Validation)

```bash
# Verify src/main.rs can import libc
cargo build 2>&1 | grep -i "unresolved extern crate"

# Expected: No output (no unresolved crate errors)
# If errors appear: Check extern crate syntax in src/main.rs

# Test compilation on Unix (if on Unix/Linux/macOS)
cargo build --release

# Expected: Clean release build with libc linked

# Test compilation detection
cargo rustc -- --print cfg | grep unix

# Expected: "unix" present if on Unix platform
# This confirms #[cfg(unix)] attributes will work correctly

# Verify no Windows-specific issues (if cross-compiling)
cargo build --target x86_64-pc-windows-msvc 2>&1 | head -20

# Expected: May have warnings but should not fail due to libc
# (#[cfg(unix)] should exclude libc on Windows)
```

### Level 4: Functional Validation (End-to-End)

```bash
# Test that SIGPIPE reset code compiles
cargo build 2>&1 | grep -E "(signal|SIGPIPE|unsafe)"

# Expected: No errors related to signal() or SIGPIPE
# If errors: Check that libc = "0.2" is in Cargo.toml

# Test basic jin functionality still works
cargo run -- --help

# Expected: Jin CLI help output displays correctly
# This verifies libc dependency didn't break anything

# Test specific command that uses SIGPIPE handling
cargo run -- log | head -n 1

# Expected: Clean exit without panic messages
# (This also validates P1.M2.T1.S1 implementation)

# Verify dependency didn't introduce version conflicts
cargo build 2>&1 | grep -i "conflict"

# Expected: No dependency conflict errors
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `Cargo.toml` includes `libc = "0.2"` in [dependencies] section
- [ ] libc dependency is under "# System" comment group
- [ ] Placement is after "# Error handling", before "# Serialization"
- [ ] `cargo build` succeeds with zero errors
- [ ] `cargo tree` shows libc in dependency tree
- [ ] `Cargo.lock` includes libc package entry
- [ ] No "unresolved extern crate" errors in build output
- [ ] No dependency conflicts reported

### Feature Validation

- [ ] src/main.rs `extern crate libc;` compiles without errors
- [ ] `#[cfg(unix)]` attributes work correctly (no compile errors on Unix)
- [ ] SIGPIPE reset code compiles (if P1.M2.T1.S1 is implemented)
- [ ] `jin log | head -n 5` works without panics (integration test with P1.M2.T1.S1)
- [ ] No regressions in existing jin functionality

### Code Quality Validation

- [ ] Dependency follows Jin's Cargo.toml conventions
- [ ] Functional grouping with comments maintained
- [ ] No unnecessary features added to libc dependency
- [ ] Version specifier follows ecosystem best practices ("0.2")
- [ ] Changes are minimal and focused

### Documentation & Deployment

- [ ] No additional documentation needed (transparent to users)
- [ ] Change is backwards compatible (no breaking changes)
- [ ] Dependency is well-maintained and widely used

---

## Anti-Patterns to Avoid

- **Don't** use a specific version like "0.2.155" - use "0.2" for compatibility
- **Don't** add features to libc dependency - libc has no features to enable
- **Don't** use `^0.2` - "0.2" is already shorthand for "^0.2.0"
- **Don't** put libc in `[target.'cfg(unix)'.dependencies]` - unnecessary complexity
- **Don't** forget to add "# System" comment - breaks functional grouping convention
- **Don't** place libc randomly - maintain logical ordering (after Error, before Serialization)
- **Don't** use the `sigpipe` crate instead - libc is sufficient and standard
- **Don't** add `libc = { version = "0.2", default-features = false }` - libc has no features
- **Don't** panic if dependency already exists - verify state before adding
- **Don't** modify other dependencies - keep changes focused on libc only

---

## Confidence Score

**Rating: 10/10** for one-pass implementation success

**Justification**:
- **Extremely Simple**: Single line addition to Cargo.toml
- **Well-Researched**: Comprehensive research on libc, Cargo.toml patterns, and versioning
- **Existing Pattern**: Follows Jin's established Cargo.toml conventions
- **Standard Practice**: This is how all Rust projects add libc dependency
- **Zero Complexity**: No features, no build configuration, no code changes
- **Already Implemented**: Change may already be present in codebase
- **Clear Validation**: Simple `cargo build` verification
- **No Dependencies**: No other tasks must complete first

**Zero Risk Factors**:
- Adding a dependency cannot break existing code
- libc is a stable, widely-used crate
- No code modifications required
- No platform-specific issues (cfg(unix) handles this)

**Current Status**: Ready for implementation - may already be complete in codebase

---

## Research Artifacts Location

Research documentation stored at: `plan/P1M2T1S2/research/`

**Key Research Files**:
- `plan/P1M2T1S2/research/libc_version_patterns.md` - libc version compatibility and semver
- `plan/P1M2T1S2/research/cargo_toml_patterns.md` - Cargo.toml organization and patterns
- `plan/P1M2T1S2/research/libc_signal_function.md` - signal() function documentation

**Key File References**:
- `Cargo.toml:35` - Location where libc = "0.2" should be (or already is)
- `src/main.rs:4` - Location of `extern crate libc;` declaration
- `src/main.rs:8-23` - Location of `reset_sigpipe()` function (uses libc)
- `plan/docs/identified_issues.md` - SIGPIPE issue definition
- `plan/P1M2T1S1/PRP.md` - Related PRP that depends on this change

**External References** (from research):
- [libc crate documentation](https://docs.rs/libc)
- [libc on crates.io](https://crates.io/crates/libc)
- [Cargo dependency specification](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
- [SemVer in Cargo](https://doc.rust-lang.org/cargo/reference/semver.html)

---

## Implementation Status Note

**IMPORTANT**: Based on codebase analysis at PRP creation time:
- `Cargo.toml` line 35 already contains `libc = "0.2"`
- `src/main.rs` already has `extern crate libc;` and `reset_sigpipe()` function
- This suggests both P1.M2.T1.S1 and P1.M2.T1.S2 may already be implemented

This PRP serves as:
1. **Documentation** of what the change should be
2. **Verification** that the change is correct
3. **Reference** for future similar dependency additions
4. **Template** for PRP creation of simple dependency additions

If implementing from scratch, follow the Implementation Tasks section. If already implemented, use the Validation Loop to verify correctness.
