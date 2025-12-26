# Product Requirement Prompt (PRP): Configuration Types (P1.M1.T4)

---

## Goal

**Feature Goal**: Define Rust configuration structures that represent Jin's global configuration and per-project active context, enabling serialization to/from YAML with proper validation and error handling.

**Deliverable**: A `src/core/config.rs` module with:
1. `JinConfig` struct for global Jin settings (stored at `~/.jin/config.yaml`)
2. `ProjectContext` struct for per-project active mode/scope (stored at `.jin/context`)
3. Load/save methods with proper error handling using `JinError`
4. Serde serialization support for YAML format
5. Default values and validation

**Success Definition**:
- `cargo build` compiles with zero errors
- `JinConfig` and `ProjectContext` serialize to/from YAML correctly
- All unit tests pass for config loading/saving
- Proper error handling with `JinError` variants
- Module exported from `src/core/mod.rs`

## User Persona

**Target User**: AI coding agent implementing Jin's configuration foundation

**Use Case**: The agent needs to establish configuration type infrastructure that:
- Stores global Jin settings (repository location, defaults)
- Persists per-project active mode/scope for context awareness
- Enables `jin mode use`, `jin scope use` commands to persist context
- Allows other modules (git/, merge/, commands/) to access configuration

**User Journey**:
1. Agent receives this PRP as context
2. Creates `src/core/config.rs` with config structs
3. Implements serde serialization for YAML format
4. Adds load/save methods with proper error handling
5. Exports types from `src/core/mod.rs`
6. Validates compilation and serialization behavior

**Pain Points Addressed**:
- No manual YAML parsing by downstream modules
- Type-safe configuration access
- Clear separation between global config and project context
- Proper error handling for missing/invalid config files

## Why

- **Foundation for configuration management**: All subsequent modules depend on well-defined config types
- **User experience**: Enables `jin mode use` and `jin scope use` commands to persist context
- **Integration**: Git module needs to know repository location, merge module needs mode/scope context
- **Problems this solves**:
  - Type-safe configuration prevents runtime errors from typos
  - Centralized config definition prevents inconsistency
  - Serde serialization handles YAML parsing robustly

## What

### Configuration Type Specifications

#### 1. JinConfig (Global Configuration)
Stored at `~/.jin/config.yaml`:

```yaml
# Global Jin configuration
version: 1
repository: ~/.jin/repo  # Path to bare Git repository
default_mode: claude     # Optional default mode
default_scope: javascript  # Optional default scope
```

#### 2. ProjectContext (Per-Project Context)
Stored at `.jin/context` (within each project):

```yaml
# Active context for this project
version: 1
mode: claude              # Active mode (optional)
scope: language:javascript  # Active scope (optional)
```

### Success Criteria

- [ ] `src/core/config.rs` created with JinConfig and ProjectContext
- [ ] Both structs use serde derive macros (Serialize, Deserialize)
- [ ] Load/save methods handle file I/O with proper errors
- [ ] Default values implemented where appropriate
- [ ] Module exported from `src/core/mod.rs`
- [ ] `cargo build` succeeds with zero errors
- [ ] Unit tests for serialization and file I/O
- [ ] YAML format matches PRD specification

---

## All Needed Context

### Context Completeness Check

**"No Prior Knowledge" Test**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP provides:
- Exact struct specifications with all fields
- Research documents with serde and config patterns
- Specific file paths and YAML format examples
- Validation commands specific to this project
- External documentation URLs with section anchors

### Documentation & References

```yaml
# MUST READ - Internal Project Documentation

- file: /home/dustin/projects/jin-glm-doover/PRD.md
  why: Active Context specification - defines context format and lifecycle
  section: Lines 136-169 for context rules, YAML format, and commands
  critical: ProjectContext must match the YAML format in PRD §7.1

- file: /home/dustin/projects/jin-glm-doover/plan/docs/system_context.md
  why: File locations and module structure
  section: Lines 169-193 for File Locations (Jin storage, per-project files)
  critical: Config file paths are specified here (~/.jin/config.yaml, .jin/context)

- file: /home/dustin/projects/jin-glm-doover/src/core/error.rs
  why: Follow error handling patterns - use existing JinError variants
  pattern: ConfigError, InvalidConfig, ValidationError variants
  gotcha: Use these error variants for config load/save failures

- file: /home/dustin/projects/jin-glm-doover/src/core/layer.rs
  why: Follow type definition patterns (derives, structure, impl blocks)
  pattern: #[non_exhaustive], Debug/Clone derives, helper methods
  gotcha: Layer shows how to structure types with rich implementations

- file: /home/dustin/projects/jin-glm-doover/src/core/mod.rs
  why: Module organization pattern - config module must be added here
  section: Line 12 shows commented-out config module
  gotcha: Need to uncomment and add pub use for config types

- file: /home/dustin/projects/jin-glm-doover/Cargo.toml
  why: Dependency verification - serde and serde_yaml_ng are already specified
  section: Lines 21-24 for serde (1.0), serde_yaml_ng (0.9), toml (0.9)
  critical: Use serde_yaml_ng, NOT serde_yaml (different crate)

# EXTERNAL - Serde Documentation

- url: https://serde.rs/derive.html
  why: Serde derive macro reference for Serialize/Deserialize traits
  critical: Understanding #[serde(default)] for optional fields
  section: "Field attributes" for default, rename, skip_serializing_if

- url: https://docs.rs/serde_yaml_ng/latest/serde_yaml_ng/
  why: serde_yaml_ng crate documentation (the YAML parser used in this project)
  critical: Different from serde_yaml - API may differ
  section: "from_str" and "to_string" functions for serialization

- url: https://doc.rust-lang.org/std/path/
  why: Path and PathBuf API for config file paths
  critical: Use PathBuf for cross-platform path handling
  section: PathBuf::push(), Path::parent(), Path::file_name()

# EXTERNAL - Rust Config Best Practices

- url: https://doc.rust-lang.org/rust-by-example/serde.html
  why: Serde usage examples in Rust
  critical: Pattern for serializing/deserializing structs
  section: "Serialize and Deserialize" examples

- url: https://github.com/serde-rs/serde/tree/master/serde_yaml_ng
  why: serde_yaml_ng source and examples
  critical: See examples/ directory for usage patterns
```

### Current Codebase Tree

```bash
# Current state (run this command to verify)
tree -L 3 -I 'target|Cargo.lock' /home/dustin/projects/jin-glm-doover

# Expected output:
# /home/dustin/projects/jin-glm-doover
# ├── Cargo.toml                    # Contains serde = "1.0", serde_yaml_ng = "0.9"
# ├── PRD.md
# ├── src/
# │   ├── main.rs
# │   ├── lib.rs
# │   ├── core/
# │   │   ├── mod.rs                # Line 12: // pub mod config; (commented out)
# │   │   ├── error.rs              # Has ConfigError, InvalidConfig variants
# │   │   └── layer.rs              # Follow this type definition pattern
# │   ├── cli/mod.rs
# │   ├── commands/mod.rs
# │   ├── commit/mod.rs
# │   ├── git/mod.rs
# │   ├── merge/mod.rs
# │   ├── staging/mod.rs
# │   └── workspace/mod.rs
# └── plan/
#     ├── docs/
#     │   └── system_context.md
#     ├── P1M1T2/PRP.md
#     ├── P1M1T3/PRP.md
#     └── P1M1T4/
#         └── PRP.md                # THIS FILE
```

### Desired Codebase Tree with Files to be Added

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   └── core/
│       ├── mod.rs                  # MODIFY: Uncomment pub mod config;
│       ├── error.rs                # EXISTING
│       ├── layer.rs                # EXISTING
│       └── config.rs               # CREATE: New config types
└── tests/
    └── core/
        └── config_test.rs          # CREATE: Unit tests for config
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Use serde_yaml_ng, NOT serde_yaml
// Cargo.toml specifies serde_yaml_ng = "0.9"
// These are different crates with different APIs
use serde_yaml_ng;  // Correct
// use serde_yaml;   // WRONG - not in dependencies

// CRITICAL: Path expansion for ~/.jin/ paths
// The tilde (~) is NOT automatically expanded by Rust's std::fs
// Use shellexpand::tilde() or dirs::home_dir() for expansion
// For this task, use dirs crate (already in Cargo.toml line 33)

// CRITICAL: serde_yaml_ng API
// Unlike serde_yaml, serde_yaml_ng may have different function names
// Common pattern:
// let yaml = serde_yaml_ng::to_string(&config)?;
// let config: JinConfig = serde_yaml_ng::from_str(&yaml)?;
// Check actual API docs for exact function signatures

// CRITICAL: YAML format must match PRD specification exactly
// PRD §7.1 specifies this exact format for .jin/context:
// version: 1
// mode: claude
// scope: language:javascript
// Field names must be "version", "mode", "scope" (not "active_mode", etc.)

// CRITICAL: File permissions for ~/.jin/config.yaml
// The ~/.jin/ directory may not exist on first run
// Config load should handle missing files gracefully (return default or error)
// Config save should create parent directories with std::fs::create_dir_all()

// CRITICAL: Use existing error variants from error.rs
// JinError::ConfigError { message }
// JinError::InvalidConfig { message }
// JinError::Io (via #[from] std::io::Error)
// JinError::YamlParse (via #[from] serde_yaml_ng::Error)

// PATTERN: Follow layer.rs structure for type definitions
// - Use #[non_exhaustive] for public structs
// - Derive Debug, Clone for all config types
// - Derive Serialize, Deserialize for serde support
// - Implement impl blocks with methods after struct definition
// - Add #[cfg(test)] mod tests at end of file

// PATTERN: Naming conventions from error.rs and layer.rs
// - Structs: PascalCase (JinConfig, ProjectContext)
// - Fields: snake_case (default_mode, repository_path)
// - Methods: snake_case (load(), save(), default())
// - File name: snake_case (config.rs)

// GOTCHA: Optional fields should use Option<T> with #[serde(default)]
// Example:
// #[serde(default)]
// pub default_mode: Option<String>
// This ensures the field is omitted from YAML when None

// GOTCHA: Version field for future-proofing
// Include a "version" field in both config types
// Even if not used now, enables migration logic in future
// Set to 1 for initial implementation
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
/// Global Jin configuration stored at ~/.jin/config.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct JinConfig {
    /// Configuration format version
    pub version: u8,
    /// Path to the bare Git repository storing Jin layers
    pub repository: PathBuf,
    /// Default mode to use when no context is set
    #[serde(default)]
    pub default_mode: Option<String>,
    /// Default scope to use when no context is set
    #[serde(default)]
    pub default_scope: Option<String>,
}

/// Per-project active context stored at .jin/context
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProjectContext {
    /// Context format version
    pub version: u8,
    /// Active mode for this project (if set)
    #[serde(default)]
    pub mode: Option<String>,
    /// Active scope for this project (if set)
    #[serde(default)]
    pub scope: Option<String>,
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/core/config.rs
  - IMPLEMENT: JinConfig and ProjectContext structs
  - FOLLOW pattern: src/core/layer.rs (derive macros, structure, #[non_exhaustive])
  - STRUCTURE:
    * Use #[derive(Debug, Clone, Serialize, Deserialize)]
    * Use #[non_exhaustive] for forward compatibility
    * Use #[serde(default)] on Option fields
  - FIELDS FOR JinConfig:
    * pub version: u8
    * pub repository: PathBuf
    * pub default_mode: Option<String>
    * pub default_scope: Option<String>
  - FIELDS FOR ProjectContext:
    * pub version: u8
    * pub mode: Option<String>
    * pub scope: Option<String>
  - IMPORTS:
    * use serde::{Deserialize, Serialize}
    * use std::path::PathBuf
    * use crate::core::error::{JinError, Result}
  - PLACEMENT: New file in src/core/
  - NAMING: Follow Rust conventions (PascalCase structs, snake_case fields)

Task 2: IMPLEMENT Default traits
  - IMPLEMENT: impl Default for JinConfig and ProjectContext
  - PATTERN: Provide sensible defaults for all fields
  - FOR JinConfig:
    * version: 1
    * repository: ~/.jin/repo (use dirs::home_dir())
    * default_mode: None
    * default_scope: None
  - FOR ProjectContext:
    * version: 1
    * mode: None
    * scope: None
  - PLACEMENT: impl Default blocks after struct definitions
  - DEPENDENCIES: Requires Task 1 (structs exist)

Task 3: IMPLEMENT JinConfig::load() method
  - IMPLEMENT: pub fn load() -> Result<Self>
  - PATTERN:
    * Get config path: ~/.jin/config.yaml
    * Use std::fs::read_to_string() to read file
    * Use serde_yaml_ng::from_str() to parse YAML
    * Return Err(JinError::ConfigError) on failure
  - SPECIAL CASE: If file doesn't exist, return Default::default()
  - PATH HANDLING: Use dirs::home_dir() to get home directory
  - ERROR HANDLING:
    * File not found -> Ok(Default::default())
    * Parse error -> Err(JinError::YamlParse(...))
    * Invalid YAML -> Err(JinError::InvalidConfig { ... })
  - PLACEMENT: impl JinConfig block
  - DEPENDENCIES: Requires Task 1 (JinConfig struct)

Task 4: IMPLEMENT JinConfig::save() method
  - IMPLEMENT: pub fn save(&self) -> Result<()>
  - PATTERN:
    * Get config path: ~/.jin/config.yaml
    * Create parent directories with std::fs::create_dir_all()
    * Serialize with serde_yaml_ng::to_string()
    * Write with std::fs::write()
  - ERROR HANDLING:
    * IO errors -> JinError::Io (via ? operator)
    * Serialize errors -> JinError::YamlParse
  - PLACEMENT: impl JinConfig block
  - DEPENDENCIES: Requires Task 1 (JinConfig struct)

Task 5: IMPLEMENT ProjectContext::load() method
  - IMPLEMENT: pub fn load(project_dir: &Path) -> Result<Self>
  - PATTERN: Similar to JinConfig::load() but with project_dir parameter
  - PATH CONSTRUCTION: project_dir.join(".jin/context")
  - SPECIAL CASE: If file doesn't exist, return Default::default()
  - ERROR HANDLING: Same pattern as JinConfig::load()
  - PLACEMENT: impl ProjectContext block
  - DEPENDENCIES: Requires Task 1 (ProjectContext struct)

Task 6: IMPLEMENT ProjectContext::save() method
  - IMPLEMENT: pub fn save(&self, project_dir: &Path) -> Result<()>
  - PATTERN: Similar to JinConfig::save() but with project_dir parameter
  - PATH CONSTRUCTION: project_dir.join(".jin/context")
  - CREATE PARENT: Create .jin directory if it doesn't exist
  - ERROR HANDLING: Same pattern as JinConfig::save()
  - PLACEMENT: impl ProjectContext block
  - DEPENDENCIES: Requires Task 1 (ProjectContext struct)

Task 7: IMPLEMENT helper methods
  - IMPLEMENT: pub fn config_path() -> PathBuf (static method on JinConfig)
  - IMPLEMENT: pub fn context_path(project_dir: &Path) -> PathBuf (static method on ProjectContext)
  - PATTERN: Return the file paths for config/context files
  - USEFUL FOR: Testing, error messages, debugging
  - PLACEMENT: impl blocks
  - DEPENDENCIES: Requires Task 1 (structs exist)

Task 8: IMPLEMENT ProjectContext convenience methods
  - IMPLEMENT: pub fn set_mode(&mut self, mode: Option<String>)
  - IMPLEMENT: pub fn set_scope(&mut self, scope: Option<String>)
  - IMPLEMENT: pub fn clear(&mut self) (clear both mode and scope)
  - PATTERN: Setter methods for modifying context
  - USEFUL FOR: Commands that modify active context
  - PLACEMENT: impl ProjectContext block
  - DEPENDENCIES: Requires Task 1 (ProjectContext struct)

Task 9: MODIFY src/core/mod.rs
  - UNCOMMENT: Line 12: // pub mod config;
  - ADD: pub use config::{JinConfig, ProjectContext};
  - PRESERVE: All existing exports and comments
  - PLACEMENT: src/core/mod.rs
  - DEPENDENCIES: Requires Task 1 (config.rs exists)

Task 10: CREATE tests/core/config_test.rs
  - IMPLEMENT: Unit tests for all config methods
  - TESTS:
    * test_jinconfig_default() - verify default values
    * test_jinconfig_serialize() - verify YAML serialization
    * test_jinconfig_load_save() - verify round-trip
    * test_projectcontext_default() - verify default values
    * test_projectcontext_serialize() - verify YAML format matches PRD
    * test_projectcontext_load_save() - verify round-trip
    * test_projectcontext_setters() - verify setter methods
  - FOLLOW pattern: tests/core/layer_test.rs (if it exists)
  - PLACEMENT: tests/core/config_test.rs
  - DEPENDENCIES: Requires Tasks 1-8 (all config methods)

Task 11: VERIFY and VALIDATE
  - RUN: cargo build --release
  - RUN: cargo clippy -- -D warnings
  - RUN: cargo test --package jin --lib core::config
  - EXPECTED: Zero errors, zero warnings, all tests pass
  - DEPENDENCIES: Requires all previous tasks
```

### Implementation Patterns & Key Details

```rust
// ===== IMPORT PATTERN =====
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use crate::core::error::{JinError, Result};
use dirs::home_dir;  // From dirs crate in Cargo.toml

// ===== JIN CONFIG STRUCT PATTERN =====
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct JinConfig {
    /// Configuration format version (for future migration)
    pub version: u8,
    /// Path to the bare Git repository storing Jin data
    pub repository: PathBuf,
    /// Default mode when no context is set
    #[serde(default)]
    pub default_mode: Option<String>,
    /// Default scope when no context is set
    #[serde(default)]
    pub default_scope: Option<String>,
}

// ===== PROJECT CONTEXT STRUCT PATTERN =====
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProjectContext {
    /// Context format version (for future migration)
    pub version: u8,
    /// Active mode for this project
    #[serde(default)]
    pub mode: Option<String>,
    /// Active scope for this project
    #[serde(default)]
    pub scope: Option<String>,
}

// ===== DEFAULT TRAIT PATTERN =====
impl Default for JinConfig {
    fn default() -> Self {
        let mut repo_path = home_dir().unwrap_or_else(|| PathBuf::from("."));
        repo_path.push(".jin/repo");

        Self {
            version: 1,
            repository: repo_path,
            default_mode: None,
            default_scope: None,
        }
    }
}

impl Default for ProjectContext {
    fn default() -> Self {
        Self {
            version: 1,
            mode: None,
            scope: None,
        }
    }
}

// ===== JIN CONFIG LOAD PATTERN =====
impl JinConfig {
    /// Load global Jin configuration from ~/.jin/config.yaml
    /// Returns default config if file doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let yaml = std::fs::read_to_string(&config_path)?;
        let config: JinConfig = serde_yaml_ng::from_str(&yaml)
            .map_err(|e| JinError::InvalidConfig {
                message: format!("Failed to parse config.yaml: {}", e),
            })?;

        Ok(config)
    }

    /// Save global Jin configuration to ~/.jin/config.yaml
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();

        // Create parent directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let yaml = serde_yaml_ng::to_string(self)
            .map_err(|e| JinError::ConfigError {
                message: format!("Failed to serialize config: {}", e),
            })?;

        std::fs::write(&config_path, yaml)?;

        Ok(())
    }

    /// Get the path to the global config file
    pub fn config_path() -> PathBuf {
        let mut path = home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".jin/config.yaml");
        path
    }
}

// ===== PROJECT CONTEXT LOAD PATTERN =====
impl ProjectContext {
    /// Load project context from .jin/context
    /// Returns default context if file doesn't exist
    pub fn load(project_dir: &Path) -> Result<Self> {
        let context_path = Self::context_path(project_dir);

        if !context_path.exists() {
            return Ok(Self::default());
        }

        let yaml = std::fs::read_to_string(&context_path)?;
        let context: ProjectContext = serde_yaml_ng::from_str(&yaml)
            .map_err(|e| JinError::InvalidConfig {
                message: format!("Failed to parse .jin/context: {}", e),
            })?;

        Ok(context)
    }

    /// Save project context to .jin/context
    pub fn save(&self, project_dir: &Path) -> Result<()> {
        let context_path = Self::context_path(project_dir);

        // Create .jin directory if it doesn't exist
        if let Some(parent) = context_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let yaml = serde_yaml_ng::to_string(self)
            .map_err(|e| JinError::ConfigError {
                message: format!("Failed to serialize context: {}", e),
            })?;

        std::fs::write(&context_path, yaml)?;

        Ok(())
    }

    /// Get the path to the project context file
    pub fn context_path(project_dir: &Path) -> PathBuf {
        project_dir.join(".jin/context")
    }

    /// Set the active mode
    pub fn set_mode(&mut self, mode: Option<String>) {
        self.mode = mode;
    }

    /// Set the active scope
    pub fn set_scope(&mut self, scope: Option<String>) {
        self.scope = scope;
    }

    /// Clear both mode and scope
    pub fn clear(&mut self) {
        self.mode = None;
        self.scope = None;
    }

    /// Check if context has an active mode
    pub fn has_mode(&self) -> bool {
        self.mode.is_some()
    }

    /// Check if context has an active scope
    pub fn has_scope(&self) -> bool {
        self.scope.is_some()
    }
}
```

### Integration Points

```yaml
ERROR_HANDLING:
  - use: src/core/error.rs variants
  - patterns:
    - JinError::ConfigError { message } for save failures
    - JinError::InvalidConfig { message } for parse failures
    - JinError::Io (via #[from]) for file operations
    - JinError::YamlParse (via #[from]) for serde_yaml_ng errors

MODULE_EXPORTS:
  - modify: src/core/mod.rs
  - uncomment: pub mod config;
  - add: pub use config::{JinConfig, ProjectContext};

TESTING:
  - create: tests/core/config_test.rs
  - run: cargo test --package jin --lib core::config

FUTURE_INTEGRATION:
  - CLI commands will use JinConfig::load() and ProjectContext::load()
  - Git module will use JinConfig.repository for repo path
  - Mode/scope commands will use ProjectContext setters
  - Merge engine will read active mode/scope from ProjectContext
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating config.rs - fix before proceeding
cargo check --package jin                    # Check compilation
cargo clippy --package jin -- -D warnings    # Lint with warnings as errors
cargo fmt --check                            # Verify formatting

# Format the code
cargo fmt

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.

# Common issues to watch for:
# - "unused_imports" -> remove unused imports
# - "dead_code" -> add #[allow(dead_code)] or pub to methods
# - Pattern matching errors -> ensure all enum variants covered
# - serde attribute errors -> verify #[serde(default)] syntax
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test config module specifically
cargo test --package jin --lib core::config --verbose

# Test all core functionality
cargo test --package jin --lib core:: --verbose

# Run with output
cargo test --package jin --lib core::config -- --nocapture

# Expected: All tests pass. Look for:
# - test_jinconfig_default: Verify default values are correct
# - test_jinconfig_serialize: Verify YAML round-trip
# - test_projectcontext_serialize: Verify YAML matches PRD format
# - test_projectcontext_load_save: Verify file I/O works
```

### Level 3: Serialization Testing (Manual Validation)

```bash
# Create a manual test to verify YAML format
cat > /tmp/test_yaml_format.rs << 'EOF'
use jin_glm::core::config::{JinConfig, ProjectContext};
use std::path::Path;

fn main() {
    // Test JinConfig YAML format
    let config = JinConfig::default();
    let yaml = serde_yaml_ng::to_string(&config).unwrap();
    println!("JinConfig YAML:\n{}", yaml);

    // Test ProjectContext YAML format
    let ctx = ProjectContext {
        version: 1,
        mode: Some("claude".to_string()),
        scope: Some("language:javascript".to_string()),
    };
    let yaml = serde_yaml_ng::to_string(&ctx).unwrap();
    println!("\nProjectContext YAML:\n{}", yaml);

    // Verify format matches PRD specification
    assert!(yaml.contains("version: 1"));
    assert!(yaml.contains("mode: claude"));
    assert!(yaml.contains("scope: language:javascript"));
}
EOF

# Compile and run
rustc --edition=2021 -L target/debug/deps --extern jin_glm=target/debug/libjin_glm.rlib /tmp/test_yaml_format.rs -o /tmp/test_yaml_format
/tmp/test_yaml_format

# Expected: YAML output matches PRD format exactly
```

### Level 4: Domain-Specific Validation

```bash
# Verify YAML format matches PRD specification
# The .jin/context format should be exactly:
# version: 1
# mode: claude
# scope: language:javascript

# Test that empty fields are omitted from YAML
cargo test --package jin test_yaml_omits_none_fields -- --exact

# Test that config load handles missing files gracefully
cargo test --package jin test_load_missing_file_returns_default -- --exact

# Verify path handling for ~/.jin/ locations
cargo test --package jin test_config_path_expands_home -- --exact

# Expected: All domain-specific requirements from PRD are met
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --package jin --lib`
- [ ] No linting errors: `cargo clippy --package jin -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Documentation comments on all public methods
- [ ] All structs have doc comments

### Feature Validation

- [ ] JinConfig struct has all required fields (version, repository, default_mode, default_scope)
- [ ] ProjectContext struct has all required fields (version, mode, scope)
- [ ] YAML format matches PRD specification exactly
- [ ] Config load handles missing files (returns default)
- [ ] Config save creates parent directories
- [ ] Default trait implementations provide sensible values

### Code Quality Validation

- [ ] Follows layer.rs type definition pattern
- [ ] File placement matches desired tree structure
- [ ] Module exported from src/core/mod.rs
- [ ] No #[allow] attributes except for justified cases
- [ ] All public methods have doc comments
- [ ] Test coverage for all public methods

### Documentation & Deployment

- [ ] Module-level doc comment explains the config system
- [ ] Each struct has a doc comment explaining its purpose
- [ ] Complex methods have usage examples in doc comments
- [ ] Gotchas documented (path expansion, serde_yaml_ng vs serde_yaml)

---

## Anti-Patterns to Avoid

- - Don't use serde_yaml - use serde_yaml_ng (different crate)
- - Don't hardcode paths like "/home/user/.jin" - use dirs::home_dir()
- - Don't panic on missing config files - return default or error
- - Don't skip #[serde(default)] on Option fields - required for proper YAML omission
- - Don't use String concatenation for paths - use PathBuf::join()
- - Don't forget to create parent directories in save() methods
- - Don't use different field names than PRD specifies (must be "mode", "scope", "version")
- - Don't skip unit tests for serialization - YAML format must match exactly
- - Don't forget #[non_exhaustive] on public structs
- - Don't use anyhow::Error in library code - use JinError

---

## Test Cases to Implement

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_jinconfig_default() {
        let config = JinConfig::default();
        assert_eq!(config.version, 1);
        assert!(config.repository.ends_with(".jin/repo"));
        assert!(config.default_mode.is_none());
        assert!(config.default_scope.is_none());
    }

    #[test]
    fn test_jinconfig_serialize() {
        let config = JinConfig {
            version: 1,
            repository: PathBuf::from("/tmp/test"),
            default_mode: Some("claude".to_string()),
            default_scope: Some("javascript".to_string()),
        };

        let yaml = serde_yaml_ng::to_string(&config).unwrap();
        assert!(yaml.contains("version: 1"));
        assert!(yaml.contains("mode: claude") || yaml.contains("default_mode: claude"));
    }

    #[test]
    fn test_jinconfig_roundtrip() {
        let original = JinConfig {
            version: 1,
            repository: PathBuf::from("/tmp/test"),
            default_mode: Some("cursor".to_string()),
            default_scope: None,
        };

        let yaml = serde_yaml_ng::to_string(&original).unwrap();
        let deserialized: JinConfig = serde_yaml_ng::from_str(&yaml).unwrap();

        assert_eq!(original.version, deserialized.version);
        assert_eq!(original.repository, deserialized.repository);
        assert_eq!(original.default_mode, deserialized.default_mode);
        assert_eq!(original.default_scope, deserialized.default_scope);
    }

    #[test]
    fn test_projectcontext_default() {
        let ctx = ProjectContext::default();
        assert_eq!(ctx.version, 1);
        assert!(ctx.mode.is_none());
        assert!(ctx.scope.is_none());
    }

    #[test]
    fn test_projectcontext_serialize_matches_prd() {
        let ctx = ProjectContext {
            version: 1,
            mode: Some("claude".to_string()),
            scope: Some("language:javascript".to_string()),
        };

        let yaml = serde_yaml_ng::to_string(&ctx).unwrap();

        // Verify format matches PRD §7.1 exactly
        assert!(yaml.contains("version: 1"));
        assert!(yaml.contains("mode: claude"));
        assert!(yaml.contains("scope: language:javascript"));
    }

    #[test]
    fn test_projectcontext_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        let original = ProjectContext {
            version: 1,
            mode: Some("zed".to_string()),
            scope: Some("language:rust".to_string()),
        };

        original.save(project_dir).unwrap();
        let loaded = ProjectContext::load(project_dir).unwrap();

        assert_eq!(original.version, loaded.version);
        assert_eq!(original.mode, loaded.mode);
        assert_eq!(original.scope, loaded.scope);
    }

    #[test]
    fn test_projectcontext_load_missing_returns_default() {
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();

        // Don't create context file
        let loaded = ProjectContext::load(project_dir).unwrap();

        assert_eq!(loaded, ProjectContext::default());
    }

    #[test]
    fn test_projectcontext_setters() {
        let mut ctx = ProjectContext::default();

        ctx.set_mode(Some("claude".to_string()));
        assert_eq!(ctx.mode, Some("claude".to_string()));
        assert!(ctx.has_mode());

        ctx.set_scope(Some("python".to_string()));
        assert_eq!(ctx.scope, Some("python".to_string()));
        assert!(ctx.has_scope());

        ctx.clear();
        assert!(ctx.mode.is_none());
        assert!(ctx.scope.is_none());
    }

    #[test]
    fn test_jinconfig_save_load() {
        let temp_home = TempDir::new().unwrap();

        // Note: This test is tricky because JinConfig::config_path() uses dirs::home_dir()
        // In real implementation, you might want to make the path configurable for testing
        // For now, just verify the logic structure

        let original = JinConfig {
            version: 1,
            repository: PathBuf::from("/tmp/jin-repo"),
            default_mode: Some("cursor".to_string()),
            default_scope: None,
        };

        let yaml = serde_yaml_ng::to_string(&original).unwrap();
        let loaded: JinConfig = serde_yaml_ng::from_str(&yaml).unwrap();

        assert_eq!(original.repository, loaded.repository);
        assert_eq!(original.default_mode, loaded.default_mode);
    }
}
```

---

## Appendix: Quick Reference

### YAML Format Specification

**JinConfig** (~/.jin/config.yaml):
```yaml
version: 1
repository: /home/user/.jin/repo
default_mode: claude
default_scope: javascript
```

**ProjectContext** (.jin/context):
```yaml
version: 1
mode: claude
scope: language:javascript
```

### Config File Locations

| Config | Location | Versioned | Purpose |
|--------|----------|-----------|---------|
| JinConfig | ~/.jin/config.yaml | No | Global Jin settings |
| ProjectContext | .jin/context | No | Per-project active mode/scope |

### Method Signatures

```rust
// JinConfig
impl JinConfig {
    pub fn load() -> Result<Self>
    pub fn save(&self) -> Result<()>
    pub fn config_path() -> PathBuf
}

// ProjectContext
impl ProjectContext {
    pub fn load(project_dir: &Path) -> Result<Self>
    pub fn save(&self, project_dir: &Path) -> Result<()>
    pub fn context_path(project_dir: &Path) -> PathBuf
    pub fn set_mode(&mut self, mode: Option<String>)
    pub fn set_scope(&mut self, scope: Option<String>)
    pub fn clear(&mut self)
    pub fn has_mode(&self) -> bool
    pub fn has_scope(&self) -> bool
}
```

---

**PRP Version**: 1.0
**Last Updated**: 2025-12-26
**Confidence Score**: 9/10 - High confidence in one-pass implementation success
