# Product Requirement Prompt (PRP): Reference Management (P1.M2.T2)

---

## Goal

**Feature Goal**: Implement complete CRUD operations for Jin's custom ref namespace (`refs/jin/...`), including delete, list, iterate, and validation operations for layer references and staging refs.

**Deliverable**: Extended `src/git/repo.rs` module with additional methods:
- `delete_layer_ref()` - Delete a layer reference
- `list_layer_refs()` - List all layer references
- `list_layer_refs_by_pattern()` - List layer refs matching a glob pattern
- `layer_ref_exists()` - Check if a layer reference exists
- `create_staging_ref()` - Create a temporary staging ref
- `delete_staging_ref()` - Delete a staging ref
- Comprehensive unit tests for all new operations

**Success Definition**:
- `cargo build` compiles with zero errors
- All unit tests pass with isolated test repositories
- Layer refs can be created, read, updated, and deleted
- Staging refs can be created and cleaned up
- Reference listing and iteration work correctly
- Integration with existing `JinError` types

## User Persona

**Target User**: AI coding agent implementing Jin's Git reference management layer

**Use Case**: The agent needs to complete the CRUD API for Git references that:
- Enables deletion of layer references (for cleanup/repair operations)
- Lists all layer refs (for `jin modes`, `jin scopes` commands)
- Manages temporary staging refs (for transaction system)
- Validates reference existence

**User Journey**:
1. Agent receives this PRP as context
2. Implements delete operations for layer refs
3. Implements list/iterate operations for discovering refs
4. Implements staging ref management for transactions
5. Adds comprehensive unit tests
6. Validates compilation and test success

**Pain Points Addressed**:
- No manual reference string construction - uses `Layer.git_ref()`
- Consistent error handling with `JinError` integration
- Clear patterns for reference iteration and filtering
- Staging refs follow `refs/jin/staging/<id>` pattern

## Why

- **Transaction System Support**: P1.M3 (Transaction System) needs staging refs for atomic commits
- **CLI Commands**: `jin modes`, `jin scopes`, `jin layers` commands need to list refs
- **Cleanup Operations**: Mode/scope deletion requires ref deletion
- **Recovery**: Stale staging refs need cleanup on recovery
- **Problems this solves**:
  - No way to delete layer refs (stale data accumulation)
  - No way to discover what layers exist
  - No staging ref management for transactions
  - Missing complete CRUD API for refs

## What

Extend `JinRepo` with complete CRUD operations for Git references in Jin's namespace (`refs/jin/...`).

### Success Criteria

- [ ] `delete_layer_ref()` method implemented and tested
- [ ] `list_layer_refs()` returns all layer refs
- [ ] `list_layer_refs_by_pattern()` filters refs by glob pattern
- [ ] `layer_ref_exists()` checks ref existence
- [ ] `create_staging_ref()` creates temporary staging refs
- [ ] `delete_staging_ref()` deletes staging refs
- [ ] All methods convert errors to `JinError` consistently
- [ ] Unit tests cover all public methods
- [ ] `cargo test` passes all tests

---

## All Needed Context

### Context Completeness Check

**Validation**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP provides:
- Exact method specifications with all parameters
- Research documents with code examples for all operations
- Specific patterns from existing codebase to follow
- Complete integration guide with `Layer` and `JinError` types
- Validation commands specific to this project

### Documentation & References

```yaml
# MUST READ - Internal Project Documentation

- file: /home/dustin/projects/jin-glm-doover/PRD.md
  why: Git Architecture specification - refs/jin/ namespace, transaction refs
  section: Lines 84-115 for Logical Branch Model, Lines 103-116 for Git Ref Namespace
  critical: refs/jin/layers/ prefix, refs/jin/staging/<transaction-id> pattern

- file: /home/dustin/projects/jin-glm-doover/plan/docs/system_context.md
  why: Git ref namespace and 9-layer hierarchy
  section: Lines 103-116 for Git Ref Namespace format
  critical: Exact ref format for each layer type, staging refs pattern

- file: /home/dustin/projects/jin-glm-doover/src/git/repo.rs
  why: Existing JinRepo implementation - follow patterns for new methods
  section: Lines 173-302 for layer reference operations (get/set/create_layer_ref)
  critical: Pattern for using layer.git_ref(), error handling, reference operations
  pattern: Follow existing get/set/create_layer_ref structure for new methods

- file: /home/dustin/projects/jin-glm-doover/src/core/error.rs
  why: Error handling patterns - use existing JinError variants
  section: Lines 58-78 for Git Operation Errors
  critical: JinError::RefNotFound, JinError::RefExists, JinError::Git (transparent)

- file: /home/dustin/projects/jin-glm-doover/src/core/layer.rs
  why: Layer enum's git_ref() method provides exact ref format - CRITICAL
  section: Lines 215-279 for git_ref() implementation, Lines 459-475 for is_versioned()
  critical: ALWAYS call layer.git_ref() to get ref names, NEVER hardcode strings

# RESEARCH DOCUMENTS - Created for this PRP

- docfile: /home/dustin/projects/jin-glm-doover/plan/P1M2T2/research/git2_ref_crud.md
  why: Complete git2-rs reference CRUD operations with code examples
  section: DELETE Operations (lines 7-121), LIST/ITERATE Operations (lines 123-247), Reference Validation (lines 249-373)
  critical: Shows exact delete(), references_glob() patterns, error handling

- docfile: /home/dustin/projects/jin-glm-doover/plan/P1M2T2/research/testing_patterns.md
  why: Testing patterns from repo.rs for consistent test writing
  section: TestFixture Pattern (lines 5-48), Test Naming (lines 56-72), Assertion Patterns (lines 74-103)
  critical: Integration with existing tempfile usage, error testing patterns

# EXTERNAL - git2-rs Documentation

- url: https://docs.rs/git2/0.20/git2/struct.Reference.html#method.delete
  why: Reference delete method - core of delete_layer_ref implementation
  critical: reference.delete() for deletion, proper error handling

- url: https://docs.rs/git2/0.20/git2/struct.Repository.html#method.references
  why: Reference iteration - core of list_layer_refs implementation
  critical: repo.references() returns iterator over all refs

- url: https://docs.rs/git2/0.20/git2/struct.Repository.html#method.references_glob
  why: Pattern-based reference listing - core of list_layer_refs_by_pattern
  critical: repo.references_glob("refs/jin/layers/*") for filtered iteration

- url: https://docs.rs/git2/0.20/git2/struct.Repository.html#method.find_reference
  why: Reference lookup - used in layer_ref_exists()
  critical: Returns Err(NotFound) if ref doesn't exist

- url: https://github.com/rust-lang/git2-rs/blob/master/examples.rs
  why: Official examples showing reference operations
  section: Examples for reference iteration, deletion, pattern matching
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover/
├── Cargo.toml                      # Has git2 dependency with features
├── PRD.md
├── src/
│   ├── core/
│   │   ├── error.rs               # Has JinError::Git, RefNotFound, RefExists
│   │   ├── layer.rs               # Has Layer enum with git_ref() method
│   │   └── config.rs
│   └── git/
│       ├── mod.rs                 # Exports JinRepo
│       └── repo.rs                # Has get/set/create_layer_ref - needs extension
└── tests/
    └── integration_test.rs
```

### Desired Codebase Tree with Files to be Modified

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   └── git/
│       └── repo.rs                # MODIFY: Add delete, list, exists, staging methods
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Jin refs are LOGICAL, not branches
// refs/jin/layers/* are NOT user-facing branches
// NEVER:
//   - Use repo.branch() - creates user-facing branches
// ALWAYS:
//   - Use repo.reference() to create/update refs
//   - Use repo.find_reference() to read refs
//   - Use reference.delete() to delete refs

// CRITICAL: Layer enum provides ref names
// ALWAYS use layer.git_ref() to get ref names
// NEVER hardcode "refs/jin/layers/" strings
// Good:
//   let ref_name = layer.git_ref().unwrap();
// Bad:
//   let ref_name = format!("refs/jin/layers/{}", mode);

// CRITICAL: Reference deletion requires mut reference
// git2-rs requires &mut Reference for delete()
// Pattern:
//   let mut reference = repo.find_reference(&ref_name)?;
//   reference.delete()?;

// GOTCHA: references_glob() uses git glob pattern
// Pattern: "refs/jin/layers/*" matches all layer refs
// Pattern: "refs/jin/layers/mode/*" matches mode refs only
// Use "**" for recursive matching

// GOTCHA: find_reference() returns Err if not found
// Use Result conversion for exists checks:
//   Ok(ref) => exists,
//   Err(e) if e.code() == NotFound => doesn't exist

// CRITICAL: Staging refs follow specific pattern
// refs/jin/staging/<transaction-id>
// These are temporary refs for transaction commits
// Should be cleaned up after transaction completes

// PATTERN: Follow existing method structure
// get_layer_ref() -> delete_layer_ref()
// Both use layer.git_ref(), similar error handling
// Keep consistent with existing patterns

// PATTERN: Iterator returns references, collect to Vec
// repo.references()? returns References iterator
// Use .map(|r| r.unwrap()).collect() to get Vec<Reference>
// Or filter with .filter() before collecting
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// No new data models needed - extending existing JinRepo

// New methods to add to JinRepo:
impl JinRepo {
    // DELETE operations
    pub fn delete_layer_ref(&self, layer: &Layer) -> Result<()>;

    // LIST/ITERATE operations
    pub fn list_layer_refs(&self) -> Result<Vec<(Layer, git2::Oid)>>;
    pub fn list_layer_refs_by_pattern(&self, pattern: &str) -> Result<Vec<String>>;

    // VALIDATION operations
    pub fn layer_ref_exists(&self, layer: &Layer) -> bool;

    // STAGING REF operations
    pub fn create_staging_ref(&self, transaction_id: &str, oid: git2::Oid) -> Result<git2::Reference>;
    pub fn delete_staging_ref(&self, transaction_id: &str) -> Result<()>;
    pub fn staging_ref_exists(&self, transaction_id: &str) -> bool;
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: IMPLEMENT delete_layer_ref()
  - ADD: pub fn delete_layer_ref(&self, layer: &Layer) -> Result<()>
  - PATTERN: Follow set_layer_ref() structure, use layer.git_ref()
  - CODE TEMPLATE:
    pub fn delete_layer_ref(&self, layer: &Layer) -> Result<()> {
        let ref_name = layer.git_ref()
            .ok_or_else(|| JinError::InvalidLayer {
                name: format!("{:?}", layer),
            })?;

        let mut reference = self.inner.find_reference(&ref_name)
            .map_err(|e| match e.code() {
                git2::ErrorCode::NotFound => JinError::RefNotFound {
                    name: ref_name.clone(),
                    layer: format!("{:?}", layer),
                },
                _ => JinError::from(e),
            })?;

        reference.delete()?;
        Ok(())
    }
  - ERROR HANDLING: Convert NotFound to RefNotFound with context
  - PLACEMENT: impl JinRepo block, after create_layer_ref()
  - DEPENDENCIES: Existing get_layer_ref(), JinError types

Task 2: IMPLEMENT layer_ref_exists()
  - ADD: pub fn layer_ref_exists(&self, layer: &Layer) -> bool
  - PATTERN: Similar to get_layer_ref() but returns bool
  - CODE TEMPLATE:
    pub fn layer_ref_exists(&self, layer: &Layer) -> bool {
        let ref_name = match layer.git_ref() {
            Some(name) => name,
            None => return false, // UserLocal/WorkspaceActive never exist as refs
        };

        self.inner.find_reference(&ref_name).is_ok()
    }
  - SEMANTICS: Returns false for UserLocal/WorkspaceActive layers
  - PLACEMENT: impl JinRepo block, after delete_layer_ref()
  - DEPENDENCIES: Layer.git_ref()

Task 3: IMPLEMENT list_layer_refs()
  - ADD: pub fn list_layer_refs(&self) -> Result<Vec<(Layer, git2::Oid)>>
  - PATTERN: Use repo.references_glob("refs/jin/layers/*") for efficiency
  - CODE TEMPLATE:
    pub fn list_layer_refs(&self) -> Result<Vec<(Layer, git2::Oid)>> {
        let mut refs = Vec::new();

        for reference in self.inner.references_glob("refs/jin/layers/*")? {
            let reference = reference?;
            if let (Some(name), Some(oid)) = (reference.name(), reference.target()) {
                // Parse ref name back to Layer
                if let Some(layer) = Self::ref_name_to_layer(name) {
                    refs.push((layer, oid));
                }
            }
        }

        Ok(refs)
    }
  - HELPER: Add fn ref_name_to_layer(name: &str) -> Option<Layer> helper
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: references_glob(), Layer parsing logic

Task 4: IMPLEMENT ref_name_to_layer() helper
  - ADD: fn ref_name_to_layer(ref_name: &str) -> Option<Layer> (private)
  - PATTERN: Parse ref_name string back to Layer variant
  - CODE TEMPLATE:
    fn ref_name_to_layer(ref_name: &str) -> Option<Layer> {
        if !ref_name.starts_with("refs/jin/layers/") {
            return None;
        }

        let path = ref_name.strip_prefix("refs/jin/layers/")?;

        // Parse based on path structure
        let parts: Vec<&str> = path.split('/').collect();
        match parts.as_slice() {
            ["global"] => Some(Layer::GlobalBase),
            ["mode", mode] => Some(Layer::ModeBase { mode: mode.to_string() }),
            ["mode", mode, "scope", scope] => Some(Layer::ModeScope {
                mode: mode.to_string(),
                scope: scope.to_string(),
            }),
            ["mode", mode, "scope", scope, "project", project] => Some(Layer::ModeScopeProject {
                mode: mode.to_string(),
                scope: scope.to_string(),
                project: project.to_string(),
            }),
            ["mode", mode, "project", project] => Some(Layer::ModeProject {
                mode: mode.to_string(),
                project: project.to_string(),
            }),
            ["scope", scope] => Some(Layer::ScopeBase { scope: scope.to_string() }),
            ["project", project] => Some(Layer::ProjectBase {
                project: project.to_string(),
            }),
            _ => None,
        }
    }
  - PLACEMENT: impl JinRepo block, private helper
  - DEPENDENCIES: Layer enum structure

Task 5: IMPLEMENT list_layer_refs_by_pattern()
  - ADD: pub fn list_layer_refs_by_pattern(&self, pattern: &str) -> Result<Vec<String>>
  - PATTERN: Use repo.references_glob() with custom pattern
  - CODE TEMPLATE:
    pub fn list_layer_refs_by_pattern(&self, pattern: &str) -> Result<Vec<String>> {
        let mut refs = Vec::new();

        for reference in self.inner.references_glob(pattern)? {
            let reference = reference?;
            if let Some(name) = reference.name() {
                refs.push(name.to_string());
            }
        }

        Ok(refs)
    }
  - USE CASE: List all mode refs with "refs/jin/layers/mode/*"
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: references_glob()

Task 6: IMPLEMENT create_staging_ref()
  - ADD: pub fn create_staging_ref(&self, transaction_id: &str, oid: git2::Oid) -> Result<git2::Reference>
  - PATTERN: Use refs/jin/staging/<transaction-id> format
  - CODE TEMPLATE:
    pub fn create_staging_ref(&self, transaction_id: &str, oid: git2::Oid) -> Result<git2::Reference> {
        let ref_name = format!("refs/jin/staging/{}", transaction_id);

        self.inner.reference(
            &ref_name,
            oid,
            false, // force=false - fail if exists
            &format!("Staging ref for transaction: {}", transaction_id),
        ).map_err(|e| match e.code() {
            git2::ErrorCode::Exists => JinError::RefExists {
                name: ref_name.clone(),
                layer: "staging".to_string(),
            },
            _ => JinError::from(e),
        })
    }
  - CRITICAL: Use force=false to detect conflicting transactions
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: repo.reference()

Task 7: IMPLEMENT delete_staging_ref()
  - ADD: pub fn delete_staging_ref(&self, transaction_id: &str) -> Result<()>
  - PATTERN: Similar to delete_layer_ref() but with staging ref format
  - CODE TEMPLATE:
    pub fn delete_staging_ref(&self, transaction_id: &str) -> Result<()> {
        let ref_name = format!("refs/jin/staging/{}", transaction_id);

        let mut reference = self.inner.find_reference(&ref_name)
            .map_err(|e| match e.code() {
                git2::ErrorCode::NotFound => JinError::RefNotFound {
                    name: ref_name.clone(),
                    layer: "staging".to_string(),
                },
                _ => JinError::from(e),
            })?;

        reference.delete()?;
        Ok(())
    }
  - USE CASE: Cleanup after transaction completes
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: Similar to delete_layer_ref()

Task 8: IMPLEMENT staging_ref_exists()
  - ADD: pub fn staging_ref_exists(&self, transaction_id: &str) -> bool
  - PATTERN: Similar to layer_ref_exists() but for staging refs
  - CODE TEMPLATE:
    pub fn staging_ref_exists(&self, transaction_id: &str) -> bool {
        let ref_name = format!("refs/jin/staging/{}", transaction_id);
        self.inner.find_reference(&ref_name).is_ok()
    }
  - USE CASE: Transaction recovery detection
  - PLACEMENT: impl JinRepo block
  - DEPENDENCIES: None

Task 9: ADD unit tests for delete operations
  - ADD: Tests for delete_layer_ref()
  - TESTS:
    * test_jinrepo_delete_layer_ref()
    * test_jinrepo_delete_layer_ref_not_found_errors()
    * test_jinrepo_delete_unversioned_layer_errors()
  - FOLLOW: Pattern from testing_patterns.md lines 123-154
  - PLACEMENT: tests module in repo.rs

Task 10: ADD unit tests for list operations
  - ADD: Tests for list_layer_refs(), list_layer_refs_by_pattern()
  - TESTS:
    * test_jinrepo_list_layer_refs_empty()
    * test_jinrepo_list_layer_refs_multiple()
    * test_jinrepo_list_layer_refs_by_pattern()
    * test_jinrepo_ref_name_to_layer_parsing()
  - FOLLOW: Pattern from testing_patterns.md
  - PLACEMENT: tests module in repo.rs

Task 11: ADD unit tests for exists operations
  - ADD: Tests for layer_ref_exists(), staging_ref_exists()
  - TESTS:
    * test_jinrepo_layer_ref_exists_true()
    * test_jinrepo_layer_ref_exists_false()
    * test_jinrepo_staging_ref_exists_true()
    * test_jinrepo_staging_ref_exists_false()
  - FOLLOW: Pattern from testing_patterns.md
  - PLACEMENT: tests module in repo.rs

Task 12: ADD unit tests for staging ref operations
  - ADD: Tests for create_staging_ref(), delete_staging_ref()
  - TESTS:
    * test_jinrepo_create_staging_ref()
    * test_jinrepo_create_staging_ref_fails_if_exists()
    * test_jinrepo_delete_staging_ref()
    * test_jinrepo_delete_staging_ref_not_found_errors()
  - FOLLOW: Pattern from testing_patterns.md
  - PLACEMENT: tests module in repo.rs
```

### Implementation Patterns & Key Details

```rust
// ===== DELETE PATTERN =====
// Always get mut reference, handle NotFound error specifically
impl JinRepo {
    pub fn delete_layer_ref(&self, layer: &Layer) -> Result<()> {
        let ref_name = layer.git_ref()
            .ok_or_else(|| JinError::InvalidLayer {
                name: format!("{:?}", layer),
            })?;

        // CRITICAL: Must be mut for delete()
        let mut reference = self.inner.find_reference(&ref_name)
            .map_err(|e| match e.code() {
                git2::ErrorCode::NotFound => JinError::RefNotFound {
                    name: ref_name.clone(),
                    layer: format!("{:?}", layer),
                },
                _ => JinError::from(e),
            })?;

        reference.delete()?;
        Ok(())
    }
}

// ===== LIST PATTERN =====
// Use references_glob() for efficiency, collect to Vec
impl JinRepo {
    pub fn list_layer_refs(&self) -> Result<Vec<(Layer, git2::Oid)>> {
        let mut refs = Vec::new();

        for reference in self.inner.references_glob("refs/jin/layers/*")? {
            let reference = reference?;
            if let (Some(name), Some(oid)) = (reference.name(), reference.target()) {
                if let Some(layer) = Self::ref_name_to_layer(name) {
                    refs.push((layer, oid));
                }
            }
        }

        Ok(refs)
    }
}

// ===== PARSE REF NAME TO LAYER PATTERN =====
// Parse ref path back to Layer variant
impl JinRepo {
    fn ref_name_to_layer(ref_name: &str) -> Option<Layer> {
        if !ref_name.starts_with("refs/jin/layers/") {
            return None;
        }

        let path = ref_name.strip_prefix("refs/jin/layers/")?;
        let parts: Vec<&str> = path.split('/').collect();

        match parts.as_slice() {
            ["global"] => Some(Layer::GlobalBase),
            ["mode", mode] => Some(Layer::ModeBase { mode: mode.to_string() }),
            ["scope", scope] => Some(Layer::ScopeBase { scope: scope.to_string() }),
            ["project", project] => Some(Layer::ProjectBase {
                project: project.to_string(),
            }),
            // ... other patterns
            _ => None,
        }
    }
}

// ===== EXISTS PATTERN =====
// Simple check using find_reference().is_ok()
impl JinRepo {
    pub fn layer_ref_exists(&self, layer: &Layer) -> bool {
        let ref_name = match layer.git_ref() {
            Some(name) => name,
            None => return false,
        };

        self.inner.find_reference(&ref_name).is_ok()
    }
}

// ===== STAGING REF PATTERN =====
// Use refs/jin/staging/<transaction-id> format
impl JinRepo {
    pub fn create_staging_ref(&self, transaction_id: &str, oid: git2::Oid) -> Result<git2::Reference> {
        let ref_name = format!("refs/jin/staging/{}", transaction_id);

        self.inner.reference(
            &ref_name,
            oid,
            false, // force=false for safety
            &format!("Staging ref for transaction: {}", transaction_id),
        ).map_err(|e| match e.code() {
            git2::ErrorCode::Exists => JinError::RefExists {
                name: ref_name.clone(),
                layer: "staging".to_string(),
            },
            _ => JinError::from(e),
        })
    }

    pub fn delete_staging_ref(&self, transaction_id: &str) -> Result<()> {
        let ref_name = format!("refs/jin/staging/{}", transaction_id);

        let mut reference = self.inner.find_reference(&ref_name)
            .map_err(|e| match e.code() {
                git2::ErrorCode::NotFound => JinError::RefNotFound {
                    name: ref_name.clone(),
                    layer: "staging".to_string(),
                },
                _ => JinError::from(e),
            })?;

        reference.delete()?;
        Ok(())
    }
}
```

### Integration Points

```yaml
ERROR_HANDLING:
  - use: src/core/error.rs
  - patterns:
    * JinError::RefNotFound { name, layer } - for missing refs
    * JinError::RefExists { name, layer } - for duplicate refs
    * JinError::InvalidLayer { name } - for UserLocal/WorkspaceActive
    * JinError::Git (transparent) - automatic via #[from]

LAYER_INTEGRATION:
  - use: src/core/layer.rs
  - method: layer.git_ref() returns Option<String>
  - parsing: ref_name_to_layer() parses ref name back to Layer

TRANSACTION_SYSTEM (FUTURE):
  - P1.M3: Transaction System will use create_staging_ref()
  - Staging refs hold pre-commit state
  - Delete after successful commit

CLI_COMMANDS (FUTURE):
  - P4.M5: `jin modes` will use list_layer_refs_by_pattern("refs/jin/layers/mode/*")
  - P4.M5: `jin scopes` will use list_layer_refs_by_pattern("refs/jin/layers/scope/*")
  - P4.M4: `jin reset` will use delete_layer_ref()
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after modifying repo.rs - fix before proceeding
cargo check --package jin                    # Check compilation
cargo clippy --package jin -- -D warnings    # Lint with warnings as errors
cargo fmt --check                            # Verify formatting

# Format the code
cargo fmt

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test JinRepo module specifically
cargo test --package jin --lib git::repo --verbose

# Run specific test categories
cargo test --package jin --lib git::repo::tests::test_delete --verbose
cargo test --package jin --lib git::repo::tests::test_list --verbose
cargo test --package jin --lib git::repo::tests::test_staging --verbose

# Expected: All tests pass. Look for:
# - test_jinrepo_delete_layer_ref: Verifies deletion works
# - test_jinrepo_list_layer_refs: Lists all layer refs correctly
# - test_jinrepo_ref_name_to_layer_parsing: Parses ref names to Layer
# - test_jinrepo_create_staging_ref: Creates staging refs
```

### Level 3: Integration Testing (System Validation)

```bash
# Test actual reference operations with real git2
cargo test --package jin --lib git::repo --verbose

# Manual verification of reference creation/deletion
# After test creates refs, verify cleanup worked
# No refs should remain after test completes

# Expected:
# - Refs are created correctly under refs/jin/layers/
# - Refs are deleted successfully
# - Staging refs are created and cleaned up
```

### Level 4: Domain-Specific Validation

```bash
# Verify Layer.git_ref() round-trip
cargo test --package jin test_jinrepo_ref_name_to_layer_parsing -- --exact
# Asserts: Ref names parse back to correct Layer variants

# Verify staging ref isolation
cargo test --package jin test_jinrepo_create_staging_ref -- --exact
# Asserts: Staging refs don't interfere with layer refs

# Verify pattern matching
cargo test --package jin test_jinrepo_list_layer_refs_by_pattern -- --exact
# Asserts: Pattern filtering works correctly

# Expected: All Jin-specific requirements met
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --package jin --lib`
- [ ] No linting errors: `cargo clippy --package jin -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Documentation comments on all new public methods

### Feature Validation

- [ ] `delete_layer_ref()` deletes layer refs successfully
- [ ] `layer_ref_exists()` returns correct boolean
- [ ] `list_layer_refs()` returns all layer refs with Layer/Oid tuples
- [ ] `list_layer_refs_by_pattern()` filters refs correctly
- [ ] `ref_name_to_layer()` parses all ref formats correctly
- [ ] `create_staging_ref()` creates refs under refs/jin/staging/
- [ ] `delete_staging_ref()` deletes staging refs
- [ ] `staging_ref_exists()` checks staging ref existence

### Code Quality Validation

- [ ] Follows existing repo.rs patterns
- [ ] Uses `layer.git_ref()` consistently (no hardcoded strings)
- [ ] Error handling matches existing patterns
- [ ] Test coverage for all public methods
- [ ] Tests follow testing_patterns.md conventions

### Documentation & Deployment

- [ ] All public methods have doc comments
- [ ] Examples in doc comments where helpful
- [ ] Gotchas documented (mut reference for delete, ref name parsing)

---

## Anti-Patterns to Avoid

- Don't hardcode "refs/jin/layers/" strings - use `layer.git_ref()`
- Don't use `repo.branch()` - Jin refs are not branches
- Don't forget `&mut` for reference.delete() - compile error
- Don't ignore NotFound errors - convert to `JinError::RefNotFound`
- Don't use force=true for create_staging_ref - should detect conflicts
- Don't parse ref names with string slicing - use proper strip_prefix and split
- Don't skip testing ref_name_to_layer() - complex parsing logic
- Don't forget to handle UserLocal/WorkspaceActive (git_ref() returns None)
- Don't use staging refs for layer data - separate namespaces
- Don't leave stale staging refs - must be cleaned up

---

## Appendix: Quick Reference

### New Methods Summary

```rust
// DELETE operations
pub fn delete_layer_ref(&self, layer: &Layer) -> Result<()>
pub fn delete_staging_ref(&self, transaction_id: &str) -> Result<()>

// LIST/ITERATE operations
pub fn list_layer_refs(&self) -> Result<Vec<(Layer, git2::Oid)>>
pub fn list_layer_refs_by_pattern(&self, pattern: &str) -> Result<Vec<String>>

// VALIDATION operations
pub fn layer_ref_exists(&self, layer: &Layer) -> bool
pub fn staging_ref_exists(&self, transaction_id: &str) -> bool

// STAGING REF operations
pub fn create_staging_ref(&self, transaction_id: &str, oid: git2::Oid) -> Result<git2::Reference>

// PRIVATE helper
fn ref_name_to_layer(ref_name: &str) -> Option<Layer>
```

### Ref Namespace Summary

| Ref Pattern | Example | Layer Variant |
|-------------|---------|---------------|
| `refs/jin/layers/global` | - | GlobalBase |
| `refs/jin/layers/mode/{mode}` | `refs/jin/layers/mode/claude` | ModeBase |
| `refs/jin/layers/mode/{mode}/scope/{scope}` | `refs/jin/layers/mode/claude/scope/python` | ModeScope |
| `refs/jin/layers/scope/{scope}` | `refs/jin/layers/scope/python` | ScopeBase |
| `refs/jin/layers/project/{project}` | `refs/jin/layers/project/myapp` | ProjectBase |
| `refs/jin/staging/{transaction-id}` | `refs/jin/staging/abc123` | (staging) |

---

**PRP Version**: 1.0
**Last Updated**: 2025-12-26
**Confidence Score**: 9/10 - High confidence in one-pass implementation success
