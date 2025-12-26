---
name: "P1.M1.T3: Layer Type Definitions - Implementation PRP"
description: |

---

## Goal

**Feature Goal**: Define the 9-layer hierarchy as Rust types with routing logic that maps CLI flags to target layers.

**Deliverable**: A `Layer` enum system with:
1. Nine layer variants representing the complete hierarchy
2. Storage path resolution for each layer
3. Git ref namespace mapping
4. Routing logic from CLI flags to target layers
5. Precedence ordering for merge operations

**Success Definition**:
- `Layer` enum compiles with all nine variants
- Each variant resolves to correct storage path
- Each variant maps to correct Git ref
- `Layer::from_flags()` correctly routes CLI flag combinations
- Precedence ordering is well-defined via `Ord` trait
- All unit tests pass
- No clippy warnings or rustc errors

## User Persona

**Target User**: CLI users who execute `jin add`, `jin commit`, `jin status` commands with mode/scope/project flags.

**Use Case**: A user runs `jin add config.toml --mode=claude --scope=python --project=myapp` and the system correctly routes the file to Layer 4 (Mode → Scope → Project) with storage path `jin/mode/claude/scope/python/project/myapp/` and Git ref `refs/jin/layers/mode/claude/scope/python/project/myapp`.

**User Journey**:
1. User invokes a jin command with optional `--mode`, `--scope`, `--project`, `--global` flags
2. System parses flags and invokes `Layer::from_flags(mode, scope, project, global)`
3. System receives the target `Layer` enum variant
4. System calls `layer.storage_path(project)` to get the file storage location
5. System calls `layer.git_ref()` to get the Git reference for the layer
6. System performs Git operations using the resolved ref

**Pain Points Addressed**:
- No manual path construction by users
- Consistent layer routing across all commands
- Clear precedence rules for merge operations
- Type-safe layer selection (compile-time guarantees)

## Why

- **Business value**: Enables the core multi-layer configuration management that distinguishes Jin from standard Git
- **Integration**: All CLI commands depend on correct layer routing for file placement and merge operations
- **Problems solved**:
  - Users don't need to remember complex path structures
  - Eliminates routing logic bugs from ad-hoc string manipulation
  - Provides single source of truth for layer definitions

## What

### 9-Layer Hierarchy (Lowest to Highest Precedence)

| Layer | Variant Name | Description | Storage Path | Git Ref |
|-------|--------------|-------------|--------------|---------|
| 1 | `GlobalBase` | Shared defaults | `jin/global/` | `refs/jin/layers/global` |
| 2 | `ModeBase` | Mode defaults | `jin/mode/<mode>/` | `refs/jin/layers/mode/<mode>` |
| 3 | `ModeScope` | Scoped mode configs | `jin/mode/<mode>/scope/<scope>/` | `refs/jin/layers/mode/<mode>/scope/<scope>` |
| 4 | `ModeScopeProject` | Project overrides for scoped mode | `jin/mode/<mode>/scope/<scope>/project/<project>/` | `refs/jin/layers/mode/<mode>/scope/<scope>/project/<project>` |
| 5 | `ModeProject` | Project overrides for mode | `jin/mode/<mode>/project/<project>/` | `refs/jin/layers/mode/<mode>/project/<project>` |
| 6 | `ScopeBase` | Untethered scope configs | `jin/scope/<scope>/` | `refs/jin/layers/scope/<scope>` |
| 7 | `ProjectBase` | Project-only configs | `jin/project/<project>/` | `refs/jin/layers/project/<project>` |
| 8 | `UserLocal` | Machine-only overlays | `~/.jin/local/` | Not versioned (local only) |
| 9 | `WorkspaceActive` | Derived merge result | `.jin/workspace/` | Not versioned (derived) |

### Routing Logic Table

| Flags | Target Layer | Notes |
|-------|--------------|-------|
| `(none)` | Layer 7 (ProjectBase) | Default: project-only |
| `--mode` | Layer 2 (ModeBase) | Mode defaults |
| `--mode --project` | Layer 5 (ModeProject) | Mode + project |
| `--mode --scope` | Layer 3 (ModeScope) | Mode + scope |
| `--mode --scope --project` | Layer 4 (ModeScopeProject) | Full hierarchy |
| `--scope` | Layer 6 (ScopeBase) | Untethered scope |
| `--global` | Layer 1 (GlobalBase) | Global defaults |

### Success Criteria

- [ ] `Layer` enum with all 9 variants compiles without errors
- [ ] `storage_path(&self, project: &str) -> PathBuf` returns correct paths for all variants
- [ ] `git_ref(&self, mode: &str, scope: &str, project: &str) -> String` returns correct refs
- [ ] `from_flags(mode, scope, project, global) -> Option<Layer>` routes correctly
- [ ] `Layer` derives `Ord` with precedence (Layer 1 < Layer 9)
- [ ] All unit tests pass
- [ ] `cargo clippy` shows no warnings
- [ ] Module is exported from `src/core/mod.rs`

---

## All Needed Context

### Context Completeness Check

**No Prior Knowledge Test**: If someone knew nothing about this codebase, would they have everything needed?

The PRP below provides:
- Exact file paths and module structure
- Complete layer specifications with all attributes
- Existing code patterns to follow (error handling, naming, derives)
- External documentation links with specific sections
- Implementation task breakdown with exact function signatures
- Validation commands specific to this project

### Documentation & References

```yaml
# MUST READ - Include these in your context window

# === EXTERNAL DOCUMENTATION ===

- url: https://doc.rust-lang.org/book/ch06-00-enums.html
  why: Rust enum fundamentals and best practices for the Layer enum definition
  critical: Use data-carrying enum variants for layers with mode/scope/project parameters

- url: https://doc.rust-lang.org/std/path/
  why: Path and PathBuf API reference for storage_path() implementation
  critical: Use PathBuf::join() for path construction, never string concatenation

- url: https://doc.rust-lang.org/std/path/struct.Path.html#method.join
  why: Cross-platform path joining pattern for building layer storage paths
  critical: join() replaces absolute paths, use push() for appending

- url: https://serde.rs/enum-representations.html
  why: Serde serialization patterns if Layer needs serialization (future-proofing)
  section: Externally Tagged

- url: https://docs.rs/git2/latest/git2/
  why: Git2 crate reference for ref namespace operations
  critical: Reference string format: "refs/jin/layers/..."

# === CODEBASE REFERENCES ===

- file: src/core/error.rs
  why: Follow error handling patterns - use JinError variants for layer errors
  pattern: Structured error variants with context fields (name, layer)
  gotcha: Layer-specific errors already defined (InvalidLayer, LayerRoutingError, ModeNotFound, ScopeNotFound)

- file: src/core/mod.rs
  why: Module organization pattern - layer module must be added here
  pattern: Comment at top describes module contents, re-exports use pub use
  gotcha: Lines 10-11 show commented-out layer module that needs to be uncommented

# === PROJECT DOCUMENTATION ===

- docfile: plan/docs/system_context.md
  why: Complete layer routing table and Git ref namespace specification
  section: Lines 107-116 for Git ref namespace, 133-146 for layer routing table

- docfile: PRD.md
  why: Full 9-layer hierarchy definition with descriptions and precedence
  section: Lines 64-82 for layer table, entire document for context

# === RESEARCH ARTIFACTS ===

- docfile: plan/P1M1T2/research/git2_error_patterns.md
  why: Git2 error handling patterns and repository operation examples
  section: Lines 234-268 for Path usage patterns
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover
├── Cargo.toml                  # Project dependencies
├── Cargo.lock
├── PRD.md                      # Product Requirements Document
├── tasks.json                  # Task tracking
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs                  # Library root
│   ├── core/
│   │   ├── mod.rs              # Core module exports (ADD LAYER MODULE HERE)
│   │   └── error.rs            # JinError enum (FOLLOW THIS PATTERN)
│   ├── cli/
│   │   └── mod.rs
│   ├── commands/
│   │   └── mod.rs
│   ├── commit/
│   │   └── mod.rs
│   ├── git/
│   │   └── mod.rs
│   ├── merge/
│   │   └── mod.rs
│   ├── staging/
│   │   └── mod.rs
│   └── workspace/
│       └── mod.rs
├── tests/
│   └── integration_test.rs
└── plan/
    ├── docs/
    │   ├── system_context.md
    │   ├── external_deps.md
    │   └── implementation_status.md
    ├── P1M1T2/
    │   ├── PRP.md
    │   └── research/
    └── P1M1T3/
        ├── PRP.md              # THIS FILE
        └── research/
```

### Desired Codebase Tree (Files to Add)

```bash
├── src/
│   └── core/
│       ├── mod.rs              # MODIFY: Uncomment and add pub mod layer;
│       ├── error.rs            # EXISTING: Contains layer error variants
│       └── layer.rs            # CREATE: New Layer enum implementation
└── tests/
    └── core/
        └── layer_test.rs       # CREATE: Unit tests for Layer
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Path behavior with join() vs push()
// join() REPLACES the path if the argument is absolute
let mut path = PathBuf::from("/tmp");
path.push("/etc");  // path is now "/etc", NOT "/tmp/etc"!
// Use push() only for relative components, or use join() carefully

// CRITICAL: Layer 8 (UserLocal) and Layer 9 (WorkspaceActive) are NOT versioned
// They should return None or empty string from git_ref()
// Their storage paths use special prefixes: ~/.jin/ and .jin/

// CRITICAL: Layer enum must use #[non_exhaustive] to allow future expansion
// This is a public API that other modules will depend on

// CRITICAL: Mode, Scope, Project are String values in variants
// They MUST be included as data in the enum variants, not separate state

// CRITICAL: Precedence order is Layer 1 (lowest) through Layer 9 (highest)
// Derive Ord such that GlobalBase < WorkspaceActive
// Rust derives Ord based on declaration order, so declare variants 1-9 in order

// CRITICAL: The error enum already has these variants - use them:
// JinError::InvalidLayer { name }
// JinError::LayerRoutingError { message }
// JinError::ModeNotFound { mode }
// JinError::ScopeNotFound { scope }

// PATTERN: Follow error.rs pattern for enum structure:
// - Group variants with comment dividers (// ===== ===== =====)
// - Use #[non_exhaustive] for public enums
// - Implement helper methods in impl block after enum definition
// - Add #[cfg(test)] mod tests at end of file

// PATTERN: Naming conventions from error.rs:
// - Enum: PascalCase (Layer, not layer)
// - Variants: PascalCase (GlobalBase, not global_base)
// - Methods: snake_case (storage_path, not storagePath)
// - File name: snake_case (layer.rs, not layer.rs is already correct)
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// Core enum representing the 9-layer hierarchy
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[non_exhaustive]
pub enum Layer {
    // ===== Base Layers (1-2) =====
    GlobalBase,
    ModeBase { mode: String },

    // ===== Mode-Bound Scope Layers (3-5) =====
    ModeScope { mode: String, scope: String },
    ModeScopeProject { mode: String, scope: String, project: String },
    ModeProject { mode: String, project: String },

    // ===== Untethered Layers (6-7) =====
    ScopeBase { scope: String },
    ProjectBase { project: String },

    // ===== Local Layers (8-9) =====
    UserLocal,
    WorkspaceActive,
}

// Associated constants for layer metadata
impl Layer {
    pub const LAYER_COUNT: usize = 9;
    pub const GLOBAL_BASE_PATH: &str = "jin/global";
    pub const JIN_ROOT: &str = "jin";
    pub const USER_LOCAL_PATH: &str = ".jin/local";
    pub const WORKSPACE_PATH: &str = ".jin/workspace";
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/core/layer.rs
  - IMPLEMENT: Layer enum with all 9 variants
  - FOLLOW pattern: src/core/error.rs (enum structure, derive macros, #[non_exhaustive])
  - VARIANTS: Declare in order 1-9 for correct Ord behavior
    - GlobalBase
    - ModeBase { mode: String }
    - ModeScope { mode: String, scope: String }
    - ModeScopeProject { mode: String, scope: String, project: String }
    - ModeProject { mode: String, project: String }
    - ScopeBase { scope: String }
    - ProjectBase { project: String }
    - UserLocal
    - WorkspaceActive
  - DERIVES: Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, #[non_exhaustive]
  - PLACEMENT: New file in src/core/

Task 2: IMPLEMENT storage_path() method
  - IMPLEMENT: pub fn storage_path(&self, project: &str) -> PathBuf
  - PATTERN: Match on self, use PathBuf::from() and .join()
  - RETURN: Storage path for files in this layer
  - SPECIAL CASES:
    - UserLocal -> PathBuf::from("~/.jin/local/")
    - WorkspaceActive -> PathBuf::from(".jin/workspace/")
    - All other layers -> jin/ prefixed paths
  - GOTCHA: Use project parameter for ProjectBase variant (variant has project field but may differ)
  - DEPENDENCIES: Requires Task 1 (Layer enum)

Task 3: IMPLEMENT git_ref() method
  - IMPLEMENT: pub fn git_ref(&self) -> Option<String>
  - PATTERN: Match on self, construct ref strings
  - RETURN: Some(ref) for layers 1-7, None for layers 8-9 (not versioned)
  - FORMAT: "refs/jin/layers/..." with components joined by "/"
  - SPECIAL CASES:
    - UserLocal -> None (not versioned)
    - WorkspaceActive -> None (derived, not stored)
  - DEPENDENCIES: Requires Task 1 (Layer enum)

Task 4: IMPLEMENT from_flags() static method
  - IMPLEMENT: pub fn from_flags(mode: Option<&str>, scope: Option<&str>, project: Option<&str>, global: bool) -> Option<Self>
  - LOGIC: Match on flag combinations following routing table
  - ROUTING TABLE (priority order):
    1. global=true -> GlobalBase
    2. mode + scope + project -> ModeScopeProject
    3. mode + project -> ModeProject
    4. mode + scope -> ModeScope
    5. scope -> ScopeBase
    6. mode -> ModeBase
    7. project -> ProjectBase
    8. none -> return None (caller should use project inference)
  - RETURN: Some(Layer) for valid combinations, None for empty (use default)
  - ERROR: Return Err(JinError::LayerRoutingError) for invalid combinations
  - DEPENDENCIES: Requires Task 1 (Layer enum)

Task 5: IMPLEMENT precedence() method
  - IMPLEMENT: pub fn precedence(&self) -> u8
  - RETURN: Layer number 1-9 based on variant
  - ALTERNATIVE: Remove this method - use derived Ord instead
  - DEPENDENCIES: Requires Task 1 (Layer enum)

Task 6: IMPLEMENT is_versioned() method
  - IMPLEMENT: pub fn is_versioned(&self) -> bool
  - RETURN: true for layers 1-7, false for layers 8-9
  - DEPENDENCIES: Requires Task 1 (Layer enum)

Task 7: IMPLEMENT helper methods
  - IMPLEMENT: pub fn mode(&self) -> Option<&str> - extract mode if present
  - IMPLEMENT: pub fn scope(&self) -> Option<&str> - extract scope if present
  - IMPLEMENT: pub fn project(&self) -> Option<&str> - extract project if present
  - PATTERN: Match on self, return Some(field) or None
  - DEPENDENCIES: Requires Task 1 (Layer enum)

Task 8: MODIFY src/core/mod.rs
  - UNCOMMENT: Line 11 // pub mod layer;
  - ADD: pub use layer::Layer; for convenient importing
  - PRESERVE: All existing exports and comments
  - DEPENDENCIES: Requires Task 1 (layer.rs exists)

Task 9: CREATE tests/core/layer_test.rs
  - IMPLEMENT: Unit tests for all Layer methods
  - TESTS:
    - test_layer_ordering() - verify Ord behavior
    - test_storage_path_all_variants() - verify paths
    - test_git_ref_all_variants() - verify refs
    - test_from_flags_routing() - verify routing table
    - test_helper_methods() - verify mode/scope/project extraction
    - test_is_versioned() - verify versioning behavior
  - FOLLOW pattern: src/core/error.rs test structure
  - PLACEMENT: New file in tests/core/
  - DEPENDENCIES: Requires Tasks 1-7 (all Layer methods)

Task 10: VERIFY and VALIDATE
  - RUN: cargo build --release
  - RUN: cargo clippy -- -D warnings
  - RUN: cargo test --package jin --lib core::layer
  - EXPECTED: Zero errors, zero warnings, all tests pass
  - DEPENDENCIES: Requires all previous tasks
```

### Implementation Patterns & Key Details

```rust
// ===== STORAGE PATH PATTERN =====
// Use PathBuf::join() for constructing paths
impl Layer {
    pub fn storage_path(&self, project: &str) -> PathBuf {
        match self {
            Layer::GlobalBase => {
                PathBuf::from("jin/global")
            }
            Layer::ModeBase { mode } => {
                PathBuf::from("jin").join("mode").join(mode)
            }
            Layer::ModeScope { mode, scope } => {
                PathBuf::from("jin").join("mode").join(mode).join("scope").join(scope)
            }
            Layer::ModeScopeProject { mode, scope, project: proj } => {
                PathBuf::from("jin")
                    .join("mode")
                    .join(mode)
                    .join("scope")
                    .join(scope)
                    .join("project")
                    .join(proj)
            }
            Layer::ModeProject { mode, project: proj } => {
                PathBuf::from("jin")
                    .join("mode")
                    .join(mode)
                    .join("project")
                    .join(proj)
            }
            Layer::ScopeBase { scope } => {
                PathBuf::from("jin").join("scope").join(scope)
            }
            Layer::ProjectBase { project: proj } => {
                PathBuf::from("jin").join("project").join(proj)
            }
            Layer::UserLocal => {
                PathBuf::from("~/.jin/local")
            }
            Layer::WorkspaceActive => {
                PathBuf::from(".jin/workspace")
            }
        }
    }
}

// ===== GIT REF PATTERN =====
// Return Option<String> - None for non-versioned layers
impl Layer {
    pub fn git_ref(&self) -> Option<String> {
        match self {
            Layer::GlobalBase => {
                Some("refs/jin/layers/global".to_string())
            }
            Layer::ModeBase { mode } => {
                Some(format!("refs/jin/layers/mode/{}", mode))
            }
            Layer::ModeScope { mode, scope } => {
                Some(format!("refs/jin/layers/mode/{}/scope/{}", mode, scope))
            }
            Layer::ModeScopeProject { mode, scope, project } => {
                Some(format!(
                    "refs/jin/layers/mode/{}/scope/{}/project/{}",
                    mode, scope, project
                ))
            }
            Layer::ModeProject { mode, project } => {
                Some(format!("refs/jin/layers/mode/{}/project/{}", mode, project))
            }
            Layer::ScopeBase { scope } => {
                Some(format!("refs/jin/layers/scope/{}", scope))
            }
            Layer::ProjectBase { project } => {
                Some(format!("refs/jin/layers/project/{}", project))
            }
            // Layers 8-9 are not versioned
            Layer::UserLocal | Layer::WorkspaceActive => None,
        }
    }
}

// ===== FROM_FLAGS ROUTING PATTERN =====
// Match in priority order (most specific first)
impl Layer {
    pub fn from_flags(
        mode: Option<&str>,
        scope: Option<&str>,
        project: Option<&str>,
        global: bool,
    ) -> Option<Layer> {
        match (global, mode, scope, project) {
            // Global flag takes precedence
            (true, _, _, _) => Some(Layer::GlobalBase),

            // Full hierarchy (most specific)
            (_, Some(m), Some(s), Some(p)) => {
                Some(Layer::ModeScopeProject {
                    mode: m.to_string(),
                    scope: s.to_string(),
                    project: p.to_string(),
                })
            }

            // Mode + Project (no scope)
            (_, Some(m), None, Some(p)) => Some(Layer::ModeProject {
                mode: m.to_string(),
                project: p.to_string(),
            }),

            // Mode + Scope (no project)
            (_, Some(m), Some(s), None) => Some(Layer::ModeScope {
                mode: m.to_string(),
                scope: s.to_string(),
            }),

            // Scope only
            (_, None, Some(s), _) => Some(Layer::ScopeBase {
                scope: s.to_string(),
            }),

            // Mode only
            (_, Some(m), None, None) => Some(Layer::ModeBase {
                mode: m.to_string(),
            }),

            // Project only - this is the default, return None
            // to allow caller to use project inference
            (_, None, None, Some(_)) => Some(Layer::ProjectBase {
                project: project.unwrap().to_string(),
            }),

            // No flags - return None for default behavior
            (false, None, None, None) => None,
        }
    }
}

// ===== HELPER METHODS PATTERN =====
impl Layer {
    pub fn mode(&self) -> Option<&str> {
        match self {
            Layer::ModeBase { mode }
            | Layer::ModeScope { mode, .. }
            | Layer::ModeScopeProject { mode, .. }
            | Layer::ModeProject { mode, .. } => Some(mode),
            _ => None,
        }
    }

    pub fn scope(&self) -> Option<&str> {
        match self {
            Layer::ModeScope { scope, .. } | Layer::ModeScopeProject { scope, .. } => {
                Some(scope)
            }
            Layer::ScopeBase { scope } => Some(scope),
            _ => None,
        }
    }

    pub fn project(&self) -> Option<&str> {
        match self {
            Layer::ProjectBase { project }
            | Layer::ModeScopeProject { project, .. }
            | Layer::ModeProject { project, .. } => Some(project),
            _ => None,
        }
    }

    pub fn is_versioned(&self) -> bool {
        !matches!(self, Layer::UserLocal | Layer::WorkspaceActive)
    }
}
```

### Integration Points

```yaml
ERROR_HANDLING:
  - use: src/core/error.rs variants
  - patterns:
    - JinError::InvalidLayer { name } for unknown layer names
    - JinError::LayerRoutingError { message } for routing failures
    - JinError::ModeNotFound { mode } for missing modes
    - JinError::ScopeNotFound { scope } for missing scopes

MODULE_EXPORTS:
  - modify: src/core/mod.rs
  - add: pub mod layer;
  - add: pub use layer::Layer;

TESTING:
  - create: tests/core/layer_test.rs
  - run: cargo test --package jin --lib core::layer

FUTURE_INTEGRATION:
  - CLI commands will use Layer::from_flags()
  - Git operations will use Layer::git_ref()
  - File operations will use Layer::storage_path()
  - Merge engine will use Layer precedence (Ord)
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating layer.rs - fix before proceeding
cargo check --package jin                    # Check compilation
cargo clippy --package jin -- -D warnings    # Lint with warnings as errors
cargo fmt --check                            # Verify formatting

# Format the code
cargo fmt

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.

# Common issues to watch for:
# - "dead_code" warnings -> add #[allow(dead_code)] or pub to methods
# - "unused_variables" -> prefix with underscore or remove
# - Pattern matching errors -> ensure all enum variants are covered
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test Layer module specifically
cargo test --package jin --lib core::layer --verbose

# Test all core functionality
cargo test --package jin --lib core:: --verbose

# Run with output
cargo test --package jin --lib core::layer -- --nocapture

# Coverage check (if tarpaulin is installed)
cargo tarpaulin --out Html --output-dir coverage/ --exclude-files '*/tests/*'

# Expected: All tests pass. Look for:
# - test_storage_path_*: Verify correct path construction
# - test_git_ref_*: Verify correct ref strings
# - test_from_flags_*: Verify routing logic
# - test_precedence: Verify Layer 1 < Layer 9
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the full project
cargo build --release

# Verify module exports work
cargo run --bin jin -- --help

# Test that Layer can be imported from other modules
# (Add a temporary smoke test in main.rs)

# Expected: Clean build, Layer is accessible from cli/commands modules
```

### Level 4: Domain-Specific Validation

```bash
# Verify routing matches PRD specification
cargo test --package jin test_routing_table -- --exact

# Verify Git ref namespace matches system_context.md
cargo test --package jin test_git_ref_namespace -- --exact

# Verify layer precedence for merge operations
cargo test --package jin test_layer_precedence -- --exact

# Manual verification - print layer metadata
# Add a temporary main() test that prints all layer info
# Run and visually verify against PRD table

# Expected: All outputs match the 9-layer table in PRD.md exactly
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --package jin --lib`
- [ ] No linting errors: `cargo clippy --package jin -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Documentation comments on all public methods
- [ ] All enum variants have #[doc] comments

### Feature Validation

- [ ] All 9 layer variants defined and compile
- [ ] storage_path() returns correct paths for all variants
- [ ] git_ref() returns correct refs for layers 1-7, None for 8-9
- [ ] from_flags() routes correctly per routing table
- [ ] Layer ordering matches precedence (Layer 1 < Layer 9)
- [ ] Helper methods (mode, scope, project) extract correctly

### Code Quality Validation

- [ ] Follows error.rs enum pattern
- [ ] File placement matches desired tree structure
- [ ] Module exported from src/core/mod.rs
- [ ] No #[allow] attributes except for justified cases
- [ ] All public methods have doc comments
- [ ] Test coverage for all public methods

### Documentation & Deployment

- [ ] Module-level doc comment explains the 9-layer system
- [ ] Each variant has a doc comment explaining its purpose
- [ ] Complex methods have usage examples in doc comments
- [ ] Gotchas documented (e.g., join() behavior on absolute paths)

---

## Anti-Patterns to Avoid

- ❌ Don't use String concatenation for paths - use `PathBuf::join()`
- ❌ Don't hardcode the full path in each variant - construct dynamically
- ❌ Don't forget to declare variants in order 1-9 for correct Ord behavior
- ❌ Don't skip the #[non_exhaustive] attribute on public enums
- ❌ Don't use Option<String> for mode/scope/project - use String directly
- ❌ Don't implement Display manually unless needed - Debug is sufficient
- ❌ Don't make from_flags() return Result - use Option for simplicity
- ❌ Don't panic in storage_path() or git_ref() - all paths are valid
- ❌ Don't duplicate path construction logic - use helper methods
- ❌ Don't skip unit tests for edge cases (empty strings, special characters)

---

## Test Cases to Implement

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_ordering() {
        // Verify precedence: GlobalBase < WorkspaceActive
        assert!(Layer::GlobalBase < Layer::WorkspaceActive);
        assert!(Layer::ModeBase { mode: "x".into() } < Layer::ScopeBase { scope: "y".into() });
    }

    #[test]
    fn test_storage_path_global_base() {
        let layer = Layer::GlobalBase;
        assert_eq!(layer.storage_path("myproject"), PathBuf::from("jin/global"));
    }

    #[test]
    fn test_storage_path_mode_base() {
        let layer = Layer::ModeBase { mode: "claude".into() };
        assert_eq!(layer.storage_path("myproject"), PathBuf::from("jin/mode/claude"));
    }

    #[test]
    fn test_storage_path_mode_scope() {
        let layer = Layer::ModeScope { mode: "claude".into(), scope: "python".into() };
        assert_eq!(layer.storage_path("myproject"), PathBuf::from("jin/mode/claude/scope/python"));
    }

    #[test]
    fn test_storage_path_user_local() {
        let layer = Layer::UserLocal;
        assert_eq!(layer.storage_path("myproject"), PathBuf::from("~/.jin/local"));
    }

    #[test]
    fn test_git_ref_versioned_layers() {
        assert_eq!(Layer::GlobalBase.git_ref(), Some("refs/jin/layers/global".into()));
        assert_eq!(
            Layer::ModeBase { mode: "claude".into() }.git_ref(),
            Some("refs/jin/layers/mode/claude".into())
        );
    }

    #[test]
    fn test_git_ref_non_versioned_layers() {
        assert_eq!(Layer::UserLocal.git_ref(), None);
        assert_eq!(Layer::WorkspaceActive.git_ref(), None);
    }

    #[test]
    fn test_from_flags_global() {
        let layer = Layer::from_flags(None, None, None, true).unwrap();
        assert!(matches!(layer, Layer::GlobalBase));
    }

    #[test]
    fn test_from_flags_mode_only() {
        let layer = Layer::from_flags(Some("claude"), None, None, false).unwrap();
        assert_eq!(layer, Layer::ModeBase { mode: "claude".into() });
    }

    #[test]
    fn test_from_flags_full_hierarchy() {
        let layer = Layer::from_flags(Some("claude"), Some("python"), Some("myapp"), false).unwrap();
        assert_eq!(
            layer,
            Layer::ModeScopeProject {
                mode: "claude".into(),
                scope: "python".into(),
                project: "myapp".into()
            }
        );
    }

    #[test]
    fn test_from_flags_no_flags() {
        // No flags should return None for default behavior
        assert_eq!(Layer::from_flags(None, None, None, false), None);
    }

    #[test]
    fn test_helper_methods_mode() {
        let layer = Layer::ModeBase { mode: "claude".into() };
        assert_eq!(layer.mode(), Some("claude"));
        assert_eq!(layer.scope(), None);
        assert_eq!(layer.project(), None);
    }

    #[test]
    fn test_helper_methods_mode_scope_project() {
        let layer = Layer::ModeScopeProject {
            mode: "claude".into(),
            scope: "python".into(),
            project: "myapp".into(),
        };
        assert_eq!(layer.mode(), Some("claude"));
        assert_eq!(layer.scope(), Some("python"));
        assert_eq!(layer.project(), Some("myapp"));
    }

    #[test]
    fn test_is_versioned() {
        assert!(Layer::GlobalBase.is_versioned());
        assert!(Layer::ProjectBase { project: "x".into() }.is_versioned());
        assert!(!Layer::UserLocal.is_versioned());
        assert!(!Layer::WorkspaceActive.is_versioned());
    }
}
```

---

## Appendix: Quick Reference

### Layer Routing Decision Tree

```
from_flags(mode, scope, project, global):
├── if global: return GlobalBase
├── if mode && scope && project: return ModeScopeProject
├── if mode && project: return ModeProject
├── if mode && scope: return ModeScope
├── if scope: return ScopeBase
├── if mode: return ModeBase
├── if project: return ProjectBase
└── return None (use default/project inference)
```

### Precedence Order (for Merge)

```
Layer 1: GlobalBase       (lowest)
Layer 2: ModeBase
Layer 3: ModeScope
Layer 4: ModeScopeProject
Layer 5: ModeProject
Layer 6: ScopeBase
Layer 7: ProjectBase
Layer 8: UserLocal
Layer 9: WorkspaceActive  (highest)
```

### Git Ref Namespace Pattern

```
refs/jin/layers/
├── global
├── mode/<mode>/
│   ├── scope/<scope>/
│   │   └── project/<project>
│   └── project/<project>
├── scope/<scope>/
└── project/<project>
```

---

**PRP Version**: 1.0
**Last Updated**: 2025-12-26
**Confidence Score**: 9/10 - High confidence in one-pass implementation success
