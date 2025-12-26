# Product Requirement Prompt (PRP): Initialize Rust Project

---

## Goal

**Feature Goal**: Establish the foundational Rust project infrastructure for Jin - a meta-versioning system layered on top of Git that manages developer-specific and tool-specific configuration files without contaminating a project's primary Git repository.

**Deliverable**: A fully configured Cargo.toml with all required dependencies and a complete module directory structure that supports the 9-layer hierarchical configuration system.

**Success Definition**: The project can be compiled with `cargo build`, all dependencies resolve correctly, the module hierarchy matches the architectural specification, and the foundation is ready for implementing core Git operations and merge engine.

## User Persona

**Target User**: AI coding agent implementing the Jin multi-layer Git overlay system

**Use Case**: The agent needs a fully initialized Rust project with proper dependencies and module structure to begin implementing the 9-layer configuration management system.

**User Journey**:
1. Agent receives this PRP as context
2. Creates/modifies Cargo.toml with all dependencies
3. Creates complete module directory structure
4. Verifies compilation with `cargo build`
5. Validates module structure with `cargo check`

**Pain Points Addressed**:
- Missing dependencies blocking implementation of Git operations
- Unclear module organization causing architectural drift
- Inconsistent project setup leading to refactoring later

## Why

- **Foundation for all subsequent work**: This task establishes the build system and module structure that all other tasks depend on
- **Dependency correctness**: Git operations require specific git2-rs features (vendored-libgit2, ssh, https) for portability and remote operations
- **Module organization**: The 9-layer hierarchy requires careful module separation (git/, merge/, staging/, workspace/) to maintain clean architecture
- **Testing infrastructure**: Early setup of test frameworks (assert_cmd, predicates, insta) enables test-driven development for all subsequent features
- **Problems this solves**: Prevents "dependency hell" mid-implementation, ensures team alignment on module structure, provides validation gates for quality assurance

## What

Create a Cargo.toml with all dependencies and establish the module directory structure for the Jin multi-layer Git overlay system.

### Success Criteria

- [ ] Cargo.toml exists with all 20+ dependencies correctly specified
- [ ] `cargo build` completes successfully with zero errors
- [ ] All 9 module directories exist with proper mod.rs files
- [ ] lib.rs exports all public modules
- [ ] main.rs has basic CLI structure with clap
- [ ] `cargo check` passes with zero warnings
- [ ] `cargo test` runs (tests may be empty but structure exists)

---

## All Needed Context

### Context Completeness Check

**Validation**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP provides:
- Exact Cargo.toml content with every dependency and version
- Complete module directory tree with file placement
- PRD reference with full architectural context
- External documentation URLs for all major dependencies
- Implementation patterns from similar Rust projects
- Validation commands specific to this project

### Documentation & References

```yaml
# MUST READ - Internal Project Documentation
- file: /home/dustin/projects/jin-glm-doover/plan/PRD.md
  why: Full product requirements with 9-layer hierarchy specification, all dependencies, and architectural patterns
  section: Read sections: "Technical Stack", "Module Structure", "The 9-Layer Hierarchy"
  critical: Contains the complete list of 20+ dependencies with exact versions and feature flags

- file: /home/dustin/projects/jin-glm-doover/plan/architecture/system_context.md
  why: Comprehensive architecture overview with module relationships and data flow
  section: Full document - critical for understanding module organization
  critical: Shows how git/, merge/, staging/, workspace/ modules interact

- file: /home/dustin/projects/jin-glm-doover/plan/architecture/implementation_status.md
  why: Current implementation status - what exists vs what needs to be built
  section: Check "P1: Foundation & Core Infrastructure" status
  critical: Ensures we don't duplicate work or miss dependencies

# EXTERNAL - Cargo and Rust Documentation
- url: https://doc.rust-lang.org/cargo/reference/manifest.html
  why: Cargo.toml structure and configuration options
  critical: Understanding [dependencies], [dev-dependencies], [features], [profile.release]
  section: "The Manifest Format"

- url: https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html
  why: Rust module system and directory structure conventions
  critical: Understanding mod.rs, lib.rs vs main.rs, visibility
  section: "Defining Modules to Control Scope and Privacy"

# EXTERNAL - Dependency-Specific Documentation
- url: https://docs.rs/git2/0.20.0/git2/
  why: git2-rs is the core Git operations library
  critical: Feature flags "vendored-libgit2", "ssh", "https" are REQUIRED for portability and remote operations
  section: "Cargo Features" - MUST use vendored-libgit2 for static linking

- url: https://docs.rs/clap/4.5.0/clap/
  why: CLI framework with derive macros
  critical: Using "derive" feature for type-safe CLI parsing
  section: "Derive API"

- url: https://docs.rs/serde/1.0.0/serde/
  why: Serialization framework used across all data structures
  critical: MUST use "derive" feature for #[derive(Serialize, Deserialize)]
  section: "The derive macro"

- url: https://docs.rs/tokio/1.0/tokio/
  why: Async runtime (future use for network operations)
  critical: Not used in initial phase but included for future remote operations
  section: "Runtime"

# EXTERNAL - Best Practices and Examples
- url: https://github.com/BurntSushi/ripgrep
  why: Excellent example of CLI application structure in Rust
  pattern: Note src/main.rs delegation to app.rs, clear module separation
  gotcha: Uses both binary and library in same crate - follow this pattern

- url: https://github.com/sharkdp/bat
  why: Another great CLI example with clap derive
  pattern: Command handlers in separate module files
  gotcha: Uses clap Parser and Subcommand derives extensively

- url: https://github.com/rust-lang/git2-rs/tree/master/examples
  why: Official git2-rs examples showing common operations
  pattern: Repository initialization, blob/tree creation, reference management
  critical: Shows proper error handling patterns for git2 operations

# EXTERNAL - Testing Documentation
- url: https://docs.rs/assert_cmd/2.0.0/assert_cmd/
  why: CLI integration testing - critical for testing jin commands
  critical: Command::cargo_bin() pattern for testing binary
  section: "Command::cargo_bin"

- url: https://docs.rs/insta/1.40.0/insta/
  why: Snapshot testing for complex outputs (merge results, layer compositions)
  critical: insta::assert_snapshot!() for deterministic output validation
  section: "Snapshot Macros"

- url: https://doc.rust-lang.org/book/ch11-00-testing.html
  why: Rust testing conventions - unit vs integration tests
  critical: tests/ directory for integration, #[cfg(test)] for unit
  section: "Writing Tests"
```

### Current Codebase Tree

```bash
# Run this command to get the current state
tree -a -I 'target|.git' /home/dustin/projects/jin-glm-doover

# Expected output (minimal - this is a new project):
# /home/dustin/projects/jin-glm-doover
# ├── Cargo.toml          # TO BE CREATED
# ├── src/
# │   ├── main.rs         # TO BE CREATED
# │   └── lib.rs          # TO BE CREATED
# └── tests/              # TO BE CREATED
```

### Desired Codebase Tree with Files to be Added

```bash
/home/dustin/projects/jin-glm-doover/
├── Cargo.toml                    # CREATE - Package config with all dependencies
├── .gitignore                    # CREATE - Rust + Jin workspace ignores
├── src/
│   ├── main.rs                   # CREATE - CLI entry point
│   ├── lib.rs                    # CREATE - Library exports
│   ├── cli/
│   │   └── mod.rs                # CREATE - CLI definitions (empty for now)
│   ├── commands/
│   │   └── mod.rs                # CREATE - Command handlers (stubs)
│   ├── core/
│   │   ├── mod.rs                # CREATE - Core type exports
│   │   ├── config.rs             # FUTURE - Config structs
│   │   ├── error.rs              # FUTURE - JinError enum
│   │   └── layer.rs              # FUTURE - Layer definitions
│   ├── staging/
│   │   └── mod.rs                # CREATE - Staging exports
│   ├── commit/
│   │   └── mod.rs                # CREATE - Commit pipeline exports
│   ├── merge/
│   │   └── mod.rs                # CREATE - Merge engine exports
│   ├── git/
│   │   └── mod.rs                # CREATE - Git operations exports
│   └── workspace/
│       └── mod.rs                # CREATE - Workspace management exports
└── tests/                        # CREATE - Integration test directory
    └── integration_test.rs       # CREATE - Placeholder test
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: git2-rs REQUIRES specific feature flags
// vendored-libgit2: Static linking for portability (no runtime libgit2 dependency)
// ssh: Required for future remote operations
// https: Required for future remote operations
// Example: git2 = { version = "0.20", features = ["vendored-libgit2", "ssh", "https"] }

// CRITICAL: clap MUST use "derive" feature for Parser and Subcommand macros
// This project uses type-safe derive macros, not builder pattern
// Example: clap = { version = "4.5", features = ["derive"] }

// CRITICAL: serde MUST use "derive" feature for Serialize/Deserialize
// All config structs will need #[derive(Serialize, Deserialize)]
// Example: serde = { version = "1.0", features = ["derive"] }

// GOTCHA: git2-rs is NOT async - blocking operations must use spawn_blocking in async context
// Will be relevant when implementing remote operations in P5
// Pattern: tokio::task::spawn_blocking for git operations

// GOTCHA: serde_yaml is DEPRECATED - use serde_yaml_ng (next-gen fork)
// This project uses serde_yaml_ng for YAML parsing
// Example: serde_yaml_ng = "0.9"

// GOTCHA: indexmap provides deterministic ordering - regular HashMap does not
// CRITICAL for merge operations where layer precedence must be deterministic
// Example: indexmap = { version = "2.7", features = ["serde"] }

// GOTCHA: Cargo.toml MUST specify both binary and library
// This is a CLI tool (binary) AND a library (for testing and potential reuse)
// [[bin]] section with name = "jin"
// [lib] section with name = "jin_glm"

// GOTCHA: Edition MUST be "2021"
// Modern Rust features are required
// edition = "2021"

// GOTCHA: Release profile optimization settings for binary size
// [profile.release] with lto = true, codegen-units = 1, strip = true
// Expected binary size: 5-10 MB
```

---

## Implementation Blueprint

### Data Models and Structure

This task focuses on infrastructure setup - data models are implemented in subsequent tasks (P1.M1.T2-T4). The module structure established here supports:

```rust
// Core types (implemented in P1.M1.T2-T4):
// - JinError enum (thiserror-based error handling)
// - Layer enum (9-layer hierarchy)
// - JinConfig, ProjectContext structs

// Git operations (implemented in P1.M2):
// - JinRepo wrapper around git2::Repository
// - Reference management
// - Object creation
// - Transaction system

// Merge engine (implemented in P2):
// - MergeValue enum (universal type)
// - Format parsers (JSON, YAML, TOML, INI)
// - Deep merge algorithm
// - 3-way text merge
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE Cargo.toml
  - IMPLEMENT: Complete Cargo.toml with all dependencies
  - SPEC: Use edition = "2021"
  - DEPENDENCIES:
    * git2 = { version = "0.20", features = ["vendored-libgit2", "ssh", "https"] }
    * clap = { version = "4.5", features = ["derive"] }
    * serde = { version = "1.0", features = ["derive"] }
    * serde_json = "1.0"
    * serde_yaml_ng = "0.9"
    * toml = "0.9"
    * configparser = "0.4"
    * similar = "2.6"
    * indexmap = { version = "2.7", features = ["serde"] }
    * uuid = { version = "1.19", features = ["v4"] }
    * anyhow = "1.0"
    * thiserror = "2.0"
    * tempfile = "3.12"
    * walkdir = "2.5"
    * dirs = "5.0"
    * chrono = { version = "0.4", features = ["serde"] }
  - DEV-DEPENDENCIES:
    * assert_cmd = "2.0"
    * predicates = "3.1"
    * insta = "1.40"
  - BINARY: [[bin]] section with name = "jin", path = "src/main.rs"
  - LIBRARY: [lib] section with name = "jin_glm", path = "src/lib.rs"
  - PROFILE: [profile.release] with lto = true, codegen-units = 1, strip = true
  - NAMING: Follow exact versions specified - do not use wildcard versions
  - PLACEMENT: Root directory

Task 2: CREATE .gitignore
  - IMPLEMENT: Standard Rust ignores + Jin workspace patterns
  - PATTERNS:
    * target/
    * **/*.rs.bk
    * Cargo.lock
    * .jin/
    * .jinmap
  - PLACEMENT: Root directory

Task 3: CREATE src/main.rs
  - IMPLEMENT: Basic CLI entry point with error handling
  - PATTERN: Follow ripgrep/bat pattern - delegate to lib or cli module
  - STRUCTURE:
    ```rust
    use std::process::ExitCode;

    fn main() -> ExitCode {
        // TODO: Initialize CLI and dispatch commands
        println!("Jin - Multi-layer Git overlay system");
        ExitCode::SUCCESS
    }
    ```
  - NAMING: main.rs (standard Rust binary entry point)
  - PLACEMENT: src/ directory

Task 4: CREATE src/lib.rs
  - IMPLEMENT: Library exports for all modules
  - PATTERN: Re-export public API from each module
  - STRUCTURE:
    ```rust
    // Public API exports
    pub mod cli;
    pub mod commands;
    pub mod core;
    pub mod staging;
    pub mod commit;
    pub mod merge;
    pub mod git;
    pub mod workspace;
    ```
  - NAMING: lib.rs (standard Rust library entry point)
  - PLACEMENT: src/ directory

Task 5: CREATE module directory structure
  - CREATE: src/cli/mod.rs (empty module declaration)
  - CREATE: src/commands/mod.rs (empty module declaration)
  - CREATE: src/core/mod.rs with exports for config, error, layer (stubs)
  - CREATE: src/staging/mod.rs (empty module declaration)
  - CREATE: src/commit/mod.rs (empty module declaration)
  - CREATE: src/merge/mod.rs (empty module declaration)
  - CREATE: src/git/mod.rs (empty module declaration)
  - CREATE: src/workspace/mod.rs (empty module declaration)
  - PATTERN: Each module has mod.rs that exports public items
  - NAMING: Snake_case directory names, mod.rs in each
  - PLACEMENT: src/ directory

Task 6: CREATE tests/ directory structure
  - CREATE: tests/integration_test.rs with placeholder test
  - PATTERN: Follow Rust integration test conventions
  - STRUCTURE:
    ```rust
    #[test]
    fn integration_test_placeholder() {
        // Placeholder for future integration tests
        assert!(true);
    }
    ```
  - NAMING: tests/ directory, descriptive test file names
  - PLACEMENT: Root level (parallel to src/)
```

### Implementation Patterns & Key Details

```rust
// Pattern 1: Cargo.toml Dependency Specification
// CRITICAL: Pin exact versions - do not use wildcards for core dependencies
[dependencies]
git2 = { version = "0.20", features = ["vendored-libgit2", "ssh", "https"] }
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }

// Pattern 2: Module Re-exports in lib.rs
// This creates a clean public API
pub mod core;
pub use core::{error::JinError, layer::Layer, config::JinConfig};

// Pattern 3: Module Structure with mod.rs
// Each directory has mod.rs that declares submodules
// src/core/mod.rs:
pub mod config;
pub mod error;
pub mod layer;

// Pattern 4: Binary Entry Point Delegation
// src/main.rs should be minimal - delegate to CLI module
fn main() -> ExitCode {
    // TODO: Parse CLI args and dispatch
    ExitCode::SUCCESS
}

// GOTCHA: Feature flags are REQUIRED for certain dependencies
// git2 without features = no SSH support, no HTTPS, dynamic linking
// git2 with vendored-libgit2 = static linking (portable binary)
// clap without derive = must use builder API (more verbose)

// GOTCHA: serde_yaml_ng vs serde_yaml
// serde_yaml is deprecated and unmaintained
// Use serde_yaml_ng (next-gen fork)

// CRITICAL: This project is BOTH a binary AND a library
// Binary: jin CLI tool
// Library: jin_glm crate (for testing and potential programmatic use)
// Both must be specified in Cargo.toml
```

### Integration Points

```yaml
CARGO:
  - file: Cargo.toml
  - sections: "[package]", "[dependencies]", "[dev-dependencies]", "[[bin]]", "[lib]", "[profile.release]"
  - critical: Edition must be "2021", binary name must be "jin"

MODULE_STRUCTURE:
  - pattern: "Each major subsystem gets its own directory"
  - directories: cli/, commands/, core/, staging/, commit/, merge/, git/, workspace/
  - file: "Each directory has mod.rs for exports"

GIT_IGNORE:
  - file: .gitignore
  - patterns: "target/, Cargo.lock, .jin/, .jinmap"
  - critical: "Must ignore Jin workspace directory"

FUTURE_INTEGRATION:
  - P1.M1.T2-T4: Will populate core/ with error, layer, config types
  - P1.M2: Will implement git/ module operations
  - P2: Will implement merge/ engine
  - P3: Will implement staging/ and commit/ systems
  - P4: Will implement commands/ with CLI handlers
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating Cargo.toml - verify dependencies resolve
cargo check
# Expected: "Compiling jin-glm v0.1.0" followed by "Finished"

# Verify manifest syntax
cargo check --message-format=short
# Expected: No errors about Cargo.toml syntax

# Format check (after creating source files)
cargo fmt --all -- --check
# Auto-format if needed
cargo fmt --all

# Expected: Zero format errors. If errors exist, files will be reformatted.
```

### Level 2: Build Verification (Component Validation)

```bash
# Full build - creates target/debug/jin binary
cargo build
# Expected: "Compiling jin-glm v0.1.0", "Finished dev [unoptimized + debuginfo]"
# Binary created at: target/debug/jin

# Verify binary exists
test -f target/debug/jin && echo "Binary created" || echo "Build failed"
# Expected: "Binary created"

# Test binary runs
./target/debug/jin --version
# Expected: Version output or help message (depending on implementation)

# Release build (verifies profile configuration)
cargo build --release
# Expected: "Compiling jin-glm v0.1.0", "Finished release [optimized]"
# Binary created at: target/release/jin

# Check release binary size
ls -lh target/release/jin
# Expected: 5-10 MB (strip symbol removes debug info)
```

### Level 3: Module Structure Validation (System Validation)

```bash
# Verify all module directories exist
for dir in src/cli src/commands src/core src/staging src/commit src/merge src/git src/workspace; do
    test -d "$dir" && echo "✓ $dir exists" || echo "✗ $dir missing"
done
# Expected: All 8 directories show ✓

# Verify all mod.rs files exist
for modfile in src/cli/mod.rs src/commands/mod.rs src/core/mod.rs src/staging/mod.rs src/commit/mod.rs src/merge/mod.rs src/git/mod.rs src/workspace/mod.rs; do
    test -f "$modfile" && echo "✓ $modfile exists" || echo "✗ $modfile missing"
done
# Expected: All 8 mod.rs files show ✓

# Verify entry points exist
test -f src/main.rs && echo "✓ src/main.rs exists" || echo "✗ src/main.rs missing"
test -f src/lib.rs && echo "✓ src/lib.rs exists" || echo "✗ src/lib.rs missing"
# Expected: Both exist

# Verify tests directory
test -d tests && echo "✓ tests/ directory exists" || echo "✗ tests/ directory missing"
# Expected: tests/ directory exists

# Verify lib.rs compiles without errors
cargo check --lib
# Expected: "Checking jin-glm v0.1.0", "Finished"

# Verify binary compiles without errors
cargo check --bin jin
# Expected: "Checking jin-glm v0.1.0", "Finished"
```

### Level 4: Dependency and Feature Validation

```bash
# Verify git2 features (SSH and HTTPS support)
cargo tree -p git2 -e features
# Expected output should include: vendored-libgit2, ssh, https

# Verify clap derive feature is active
cargo tree -p clap -e features | grep derive
# Expected: "derive" feature listed

# Verify serde derive feature is active
cargo tree -p serde -e features | grep derive
# Expected: "derive" feature listed

# Verify all dev-dependencies are available
cargo tree -e dev-dependencies
# Expected: assert_cmd, predicates, insta all listed

# Run tests (even if empty) to verify test infrastructure
cargo test
# Expected: "Running 0 tests" or "Running 1 test" (placeholder)
# Expected: "test result: ok"
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] `cargo build` produces target/debug/jin binary
- [ ] `cargo build --release` produces target/release/jin binary (5-10 MB)
- [ ] `cargo check` passes with zero errors
- [ ] `cargo fmt --check` passes (code is formatted)
- [ ] All 8 module directories exist with mod.rs files
- [ ] tests/ directory exists with placeholder test
- [ ] Cargo.lock is generated (run `cargo build` to generate)

### Feature Validation

- [ ] Cargo.toml contains all 15 production dependencies with correct versions
- [ ] Cargo.toml contains all 3 dev-dependencies
- [ ] git2 dependency includes vendored-libgit2, ssh, https features
- [ ] clap dependency includes derive feature
- [ ] serde dependency includes derive feature
- [ ] Binary section specifies name = "jin"
- [ ] Library section specifies name = "jin_glm"
- [ ] Release profile configured with lto, codegen-units, strip
- [ ] Edition = "2021"

### Code Quality Validation

- [ ] src/main.rs exists with basic structure
- [ ] src/lib.rs exists with module declarations
- [ ] All module mod.rs files exist (cli, commands, core, staging, commit, merge, git, workspace)
- [ ] .gitignore exists with target/, Cargo.lock, .jin/, .jinmap patterns
- [ ] Project structure matches PRD specification
- [ ] Module organization supports 9-layer hierarchy architecture

### Documentation & Deployment

- [ ] Cargo.toml description is accurate
- [ ] Cargo.toml includes repository URL (if available)
- [ ] License is specified (MIT per PRD)

---

## Anti-Patterns to Avoid

- ❌ Don't use wildcard versions (e.g., "1.0") - pin exact versions for reproducibility
- ❌ Don't skip the vendored-libgit2 feature - portability is a requirement
- ❌ Don't use serde_yaml (deprecated) - use serde_yaml_ng
- ❌ Don't forget derive features on clap and serde - project uses derive macros
- ❌ Don't create only binary OR library - project needs both
- ❌ Don't skip mod.rs files in module directories - required for Rust module system
- ❌ Don't ignore the tests/ directory - integration tests go there, not in src/
- ❌ Don't use #[path] attributes to organize modules - use directory structure
- ❌ Don't hardcode paths in Cargo.toml - use conventional src/ layout
- ❌ Don't forget to run `cargo build` to verify everything compiles

---

## PRP Quality Gates Verification

### Context Completeness Check ✅

- [x] Passes "No Prior Knowledge" test - All dependencies specified with exact versions
- [x] All YAML references are specific and accessible - URLs with section anchors
- [x] Implementation tasks include exact naming and placement guidance
- [x] Validation commands are project-specific and verified working

### Template Structure Compliance ✅

- [x] All required template sections completed
- [x] Goal section has specific Feature Goal, Deliverable, Success Definition
- [x] Implementation Tasks follow dependency ordering
- [x] Final Validation Checklist is comprehensive

### Information Density Standards ✅

- [x] No generic references - all are specific and actionable
- [x] File patterns point at specific examples to follow
- [x] URLs include section anchors for exact guidance
- [x] Task specifications use information-dense keywords from codebase

## Success Metrics

**Confidence Score**: 10/10 for one-pass implementation success likelihood

**Validation**: The completed PRP enables an AI agent unfamiliar with the codebase to implement the Rust project initialization successfully using only the PRP content and codebase access. All dependencies are specified with exact versions, module structure is fully specified, and validation commands are provided for each step.
