# Product Requirement Prompt (PRP): Deep Merge Implementation

**Task**: P2.M3.T1 - Deep Merge Implementation with RFC 7396 Semantics and Array Strategies

---

## Goal

**Feature Goal**: Implement RFC 7396-compliant recursive deep merge with configurable array merge strategies including keyed array merging by `id` or `name` fields.

**Deliverable**: Enhanced `MergeValue` type with `ArrayMergeStrategy` enum and configurable merge behavior that supports both unkeyed array replacement (RFC 7396 default) and keyed array merging (PRD §11.1 requirement).

**Success Definition**:
- All existing tests continue to pass (backward compatibility)
- New tests verify keyed array merging by `id` and `name` fields
- RFC 7396 null-deletion semantics work correctly at all nesting levels
- `cargo test --all` passes with zero failures
- `cargo clippy --all-targets -- -D warnings` produces no warnings

## Why

- **RFC 7396 Compliance**: JSON Merge Patch is the industry standard for configuration merging, ensuring predictable behavior
- **PRD §11.1 Requirements**: The PRD explicitly requires "Arrays (keyed) merge by `id` or `name`" which is not yet implemented
- **Layer System Foundation**: Deep merge is the core algorithm that enables Jin's 9-layer configuration hierarchy
- **Deterministic Merges**: Predictable merge behavior is critical for user trust and system reliability
- **Array Strategy Flexibility**: Different configuration scenarios require different array merge behaviors

## What

Implement configurable deep merge with the following behaviors:

### Merge Rules (PRD §11.1 + RFC 7396)

| Value Type | Merge Behavior |
|------------|----------------|
| Objects (Maps) | Deep key merge - recursively merge keys from higher layer |
| Arrays (unkeyed) | Higher layer replaces (RFC 7396 default) |
| Arrays (keyed) | Merge by matching `id` or `name` fields |
| `null` | Deletes key from result (RFC 7396) |
| Primitives | Higher layer replaces |

### Array Merge Strategies

1. **Replace**: New array completely replaces old array (RFC 7396 default)
2. **MergeByKey**: Merge arrays by matching `id` or `name` fields within object elements
3. **Concatenate**: Combine arrays (append new to end)

### Success Criteria

- [ ] `MergeValue::merge()` maintains backward compatibility
- [ ] `ArrayMergeStrategy` enum defined with Replace, MergeByKey, Concatenate variants
- [ ] Keyed array merge correctly matches objects by `id` field
- [ ] Keyed array merge correctly matches objects by `name` field
- [ ] Null deletion works at any nesting level
- [ ] Order preservation maintained for objects (IndexMap)
- [ ] All tests pass including edge cases (empty arrays, null values, deep nesting)

---

## All Needed Context

### Context Completeness Check

Before implementing, validate that the following context is understood:

**Question**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: Yes - this PRP provides:
- Exact file locations and line numbers for existing code
- Complete enum definitions to follow
- Test patterns and file locations
- RFC 7396 specification with specific examples
- PRD requirements with section references
- Code style and naming conventions

### Documentation & References

```yaml
# MUST READ - RFC 7396 Specification
- url: https://www.rfc-editor.org/rfc/rfc7396
  why: Defines JSON Merge Patch semantics - null deletes keys, arrays replace
  critical: Section 3 specifies "If the patch is not an Object, the target is replaced by the patch"
  critical: Section 4.2 shows example: `{"a": null}` deletes key "a"

# MUST READ - PRD Merge Rules
- file: /home/dustin/projects/jin-glm-doover/PRD.md
  why: Defines Jin's merge requirements including keyed array merging
  section: "§11.1 Structured Merge Rules" (lines 260-273)
  pattern: "Arrays (keyed) merge by `id` or `name`" must be implemented
  gotcha: Arrays (unkeyed) should use replace strategy, not merge

# MUST READ - Existing Merge Implementation
- file: /home/dustin/projects/jin-glm-doover/src/merge/value.rs
  why: Current merge() method to enhance - maintains backward compatibility
  section: Lines 375-418 contain the current merge implementation
  pattern: Uses match (self, other) with recursive object merging
  gotcha: Current array behavior is always replace - needs to become configurable

# MUST READ - Test File Location and Patterns
- file: /home/dustin/projects/jin-glm-doover/tests/merge/value_test.rs
  why: Where to add new array strategy tests
  pattern: Tests use `test_<feature>_<scenario>` naming convention
  pattern: Test structure: setup -> action -> assertions with descriptive messages
  section: Lines 234-355 contain existing merge tests

# MUST READ - MergeValue Enum Definition
- file: /home/dustin/projects/jin-glm-doover/src/merge/value.rs
  why: The enum we're enhancing - understand all 7 variants
  section: Lines 60-86 contain the complete MergeValue enum
  pattern: Object variant uses IndexMap for order preservation
  gotcha: Null variant has special semantic meaning (deletion, not value)

# MUST READ - Error Type Definitions
- file: /home/dustin/projects/jin-glm-doover/src/core/error.rs
  why: Understanding JinError variants for error handling
  pattern: Use JinError::Message for custom error strings
  gotcha: Merge operations return Result<T> for future conflict detection

# REFERENCE - RFC 7396 Official Example
- url: https://www.rfc-editor.org/rfc/rfc7396#section-3
  why: Shows the canonical JSON Merge Patch example with null deletion
  example: |
    Target:  {"title": "Goodbye!", "author": {"given": "John"}}
    Patch:   {"title": "Hello!", "author": null}
    Result:  {"title": "Hello!"}
    # Note: "author" key is deleted because patch value is null

# REFERENCE - Rust Merge Crate Patterns
- url: https://docs.rs/merge/latest/merge/
  why: Reference for how to structure trait-based merge operations
  pattern: Consider using Merge trait for extensibility (optional, not required)

# REFERENCE - IndexMap Documentation
- url: https://docs.rs/indexmap/latest/indexmap/
  why: Understanding order preservation behavior
  pattern: shift_remove() preserves order when deleting keys
  gotcha: IndexMap maintains insertion order, not sorted order
```

### Current Codebase Tree

```bash
src/
├── core/
│   ├── error.rs       # JinError enum, Result type alias
│   ├── layer.rs       # Layer enum definitions
│   ├── config.rs      # Configuration types
│   └── mod.rs
├── merge/
│   ├── mod.rs         # Module exports
│   └── value.rs       # MergeValue enum, format parsers, merge() method
├── git/
│   ├── repo.rs        # JinRepo wrapper
│   ├── transaction.rs # Transaction system
│   └── mod.rs
├── lib.rs             # Public API exports
└── main.rs            # CLI entry point

tests/
└── merge/
    └── value_test.rs  # Comprehensive MergeValue tests
```

### Desired Codebase Tree with New Files

```bash
src/merge/
├── mod.rs             # ADD: export ArrayMergeStrategy
├── value.rs           # MODIFY: add ArrayMergeStrategy, enhance merge()
└── strategy.rs        # NEW: ArrayMergeStrategy enum and merge logic (optional - can be in value.rs)

tests/merge/
└── value_test.rs      # MODIFY: add array strategy tests
```

**Design Decision**: Keep `ArrayMergeStrategy` and implementation in `value.rs` to avoid over-abstraction for a single use case. The merge module should remain focused.

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: MergeValue::Null has semantic meaning (key deletion), not a regular value
// Example: Merging {"a": 1} with {"a": null} results in {} (key deleted)
// Pattern: if matches!(override_value, MergeValue::Null) { merged.shift_remove(key); }

// CRITICAL: IndexMap preserves insertion order, not sorted order
// When merging, new keys are added at the end to preserve original ordering
// Use shift_remove() instead of remove() to maintain order during deletion

// CRITICAL: The current merge() method signature must not break
// Current: pub fn merge(&self, other: &MergeValue) -> Result<Self>
// Enhancement: Add merge_with_strategy() for configurable behavior

// CRITICAL: Float values are stored as MergeValue::Float, not String
// From JSON: 3.14 becomes MergeValue::Float(3.14)
// From YAML: 3.14 becomes MergeValue::Float(3.14)

// GOTCHA: Array elements must be objects to have "id" or "name" keys
// Non-object array elements cannot participate in keyed merge
// Strategy: Filter to object elements, merge those, append non-objects

// GOTCHA: Recursive merge can cause stack overflow on deeply nested structures
// Current implementation has no depth limit
// Consider: Add depth limit parameter or use iterative approach

// PATTERN: All parsers return Result<MergeValue> for consistent error handling
// PATTERN: Error types use #[error(transparent)] for wrapping library errors
// PATTERN: Tests use descriptive assertion messages for debugging
```

---

## Implementation Blueprint

### Data Models and Structure

**1. ArrayMergeStrategy Enum** (NEW)

```rust
/// Strategy for merging arrays during deep merge operations.
///
/// Different configuration scenarios require different array merge behaviors.
/// RFC 7396 specifies "replace" as the default for unkeyed arrays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrayMergeStrategy {
    /// New array completely replaces old array (RFC 7396 default behavior)
    #[default]
    Replace,

    /// Merge arrays by matching object elements with `id` or `name` fields
    /// Objects with matching keys are deeply merged, new objects are appended
    MergeByKey,

    /// Concatenate arrays - append new elements to the end
    Concatenate,
}
```

**2. MergeConfig Struct** (NEW)

```rust
/// Configuration options for deep merge behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MergeConfig {
    /// Strategy for merging arrays
    pub array_strategy: ArrayMergeStrategy,

    /// Maximum recursion depth to prevent stack overflow
    pub max_depth: usize,
}

impl Default for MergeConfig {
    fn default() -> Self {
        Self {
            array_strategy: ArrayMergeStrategy::Replace,
            max_depth: 100,
        }
    }
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD ArrayMergeStrategy enum to src/merge/value.rs
  - IMPLEMENT: ArrayMergeStrategy enum with Replace, MergeByKey, Concatenate variants
  - ADD: #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
  - DOCUMENT: Each variant with doc comments explaining behavior
  - PLACEMENT: Add after ObjectMap type alias (after line 12), before MergeValue enum
  - NAMING: PascalCase for enum, CamelCase for variants

Task 2: ADD MergeConfig struct to src/merge/value.rs
  - IMPLEMENT: MergeConfig struct with array_strategy and max_depth fields
  - ADD: Default impl that sets Replace strategy and max_depth of 100
  - DOCUMENT: Field purpose and default values
  - PLACEMENT: Add after ArrayMergeStrategy enum, before MergeValue enum
  - NAMING: PascalCase for struct, snake_case for fields

Task 3: IMPLEMENT merge_arrays_by_key helper method
  - IMPLEMENT: fn merge_arrays_by_key(&self, base: &[MergeValue], patch: &[MergeValue]) -> Result<Vec<MergeValue>>
  - ALGORITHM:
    1. Build index of base array elements by "id" or "name" field (check "id" first)
    2. Build index of patch array elements by "id" or "name" field
    3. For each key in union of both indices:
       - If in both: deep merge the objects
       - If only in base: include as-is (unless deleted by null)
       - If only in patch: include as-is
    4. Preserve order: base elements first, then new patch elements
  - FILTER: Only include Object variants that have the key field
  - NON-OBJECTS: Append non-object elements from base, then from patch
  - PLACEMENT: Add as private method in impl MergeValue block (after merge() method)
  - ERROR HANDLING: Return JinError::Message for mismatched types

Task 4: IMPLEMENT merge_with_config method
  - IMPLEMENT: pub fn merge_with_config(&self, other: &MergeValue, config: &MergeConfig) -> Result<Self>
  - ALGORITHM:
    1. Check max_depth, return error if exceeded
    2. Match on (self, other) with these cases:
       - (_, Null): Return Null (key deletion)
       - (Object, Object): Recursively merge using IndexMap
       - (Array, Array): Apply config.array_strategy
         - Replace: Return other.clone()
         - Concatenate: base + other
         - MergeByKey: Call merge_arrays_by_key()
       - (_, _): Return other.clone() (primitive replacement)
  - PRESERVE: Existing merge() behavior (call merge_with_config with default config)
  - PLACEMENT: Add after existing merge() method in impl MergeValue block
  - RECURSION: Pass reduced max_depth to recursive calls

Task 5: UPDATE existing merge() method to use merge_with_config
  - MODIFY: pub fn merge(&self, other: &MergeValue) -> Result<Self>
  - IMPLEMENT: Call merge_with_config(self, other, &MergeConfig::default())
  - PRESERVE: All existing behavior and semantics
  - TEST: All existing tests must pass without modification
  - PLACEMENT: Modify existing method at lines 375-418

Task 6: EXPORT ArrayMergeStrategy in module
  - MODIFY: src/merge/mod.rs to export ArrayMergeStrategy
  - ADD: pub use value::{MergeValue, ArrayMergeStrategy};
  - ENSURE: Public API includes the new type

Task 7: ADD unit tests for array strategies
  - CREATE: Tests for Replace strategy (default behavior)
  - CREATE: Tests for MergeByKey with "id" field matching
  - CREATE: Tests for MergeByKey with "name" field matching
  - CREATE: Tests for MergeByKey with nested object merging
  - CREATE: Tests for Concatenate strategy
  - CREATE: Tests for max_depth error handling
  - CREATE: Tests for non-object array elements (should be preserved)
  - FOLLOW: Existing test patterns in tests/merge/value_test.rs
  - NAMING: test_merge_array_<strategy>_<scenario>
  - PLACEMENT: Add to tests/merge/value_test.rs after existing merge tests (after line 355)

Task 8: ADD unit tests for edge cases
  - TEST: Empty arrays with all strategies
  - TEST: Arrays with null elements
  - TEST: Arrays with mixed object and primitive elements
  - TEST: Deeply nested merging (10+ levels)
  - TEST: Max depth limit exceeded
  - TEST: Keyed arrays where some objects lack the key field
  - PLACEMENT: Add to tests/merge/value_test.rs
```

### Implementation Patterns & Key Details

```rust
// ===== PATTERN: Keyed Array Merge =====

// This is the core algorithm for merging arrays by a key field.
// It handles object arrays where each element has an "id" or "name" field.

fn merge_arrays_by_key(&self, base: &[MergeValue], patch: &[MergeValue]) -> Result<Vec<MergeValue>> {
    use std::collections::HashMap;

    // Helper to extract key from an object element
    let get_key = |value: &MergeValue| -> Option<(String, &MergeValue)> {
        if let MergeValue::Object(obj) = value {
            // Check "id" field first, then "name"
            let key = obj.get("id").and_then(|v| v.as_str())
                .or_else(|| obj.get("name").and_then(|v| v.as_str()));
            key.map(|k| (k.to_string(), value))
        } else {
            None
        }
    };

    // Build index of base array elements by key
    let mut base_by_key: HashMap<String, &MergeValue> = HashMap::new();
    let mut base_non_objects: Vec<&MergeValue> = Vec::new();

    for elem in base {
        match get_key(elem) {
            Some((key, value)) => { base_by_key.insert(key, value); }
            None => { base_non_objects.push(elem); }
        }
    }

    // Build index of patch array elements by key
    let mut patch_by_key: HashMap<String, &MergeValue> = HashMap::new();
    let mut patch_non_objects: Vec<&MergeValue> = Vec::new();

    for elem in patch {
        match get_key(elem) {
            Some((key, value)) => { patch_by_key.insert(key, value); }
            None => { patch_non_objects.push(elem); }
        }
    }

    // Collect all keys from both arrays
    let mut all_keys: Vec<String> = base_by_key.keys().chain(patch_by_key.keys())
        .cloned().collect::<std::collections::HashSet<_>>()
        .into_iter().collect();

    // Sort to ensure deterministic output (could also preserve insertion order)
    all_keys.sort();

    let mut result = Vec::new();

    // Merge non-object elements from base first
    for elem in base_non_objects {
        result.push(elem.clone());
    }

    // Merge keyed elements
    for key in all_keys {
        match (base_by_key.get(&key), patch_by_key.get(&key)) {
            (Some(base_val), Some(patch_val)) => {
                // Both exist - deep merge
                result.push(base_val.merge_with_config(patch_val, &MergeConfig {
                    array_strategy: ArrayMergeStrategy::MergeByKey,
                    max_depth: self.max_depth - 1,
                })?);
            }
            (Some(base_val), None) => {
                // Only in base - check if not deleted
                result.push((*base_val).clone());
            }
            (None, Some(patch_val)) => {
                // Only in patch - add new element
                if !matches!(patch_val, MergeValue::Null) {
                    result.push((*patch_val).clone());
                }
            }
            _ => {}
        }
    }

    // Append non-object elements from patch
    for elem in patch_non_objects {
        result.push(elem.clone());
    }

    Ok(result)
}

// ===== PATTERN: merge_with_config Structure =====

pub fn merge_with_config(&self, other: &MergeValue, config: &MergeConfig) -> Result<Self> {
    // Check depth limit first
    if config.max_depth == 0 {
        return Err(JinError::Message("Maximum merge depth exceeded".to_string()));
    }

    let child_config = MergeConfig {
        max_depth: config.max_depth - 1,
        ..*config
    };

    match (self, other) {
        // Rule 1: null deletes key (RFC 7396)
        (_, MergeValue::Null) => Ok(MergeValue::Null),

        // Rule 2: Deep key merge for objects (PRD §11.1)
        (MergeValue::Object(base_map), MergeValue::Object(patch_map)) => {
            let mut merged = base_map.clone();

            for (key, patch_value) in patch_map {
                if let Some(base_value) = merged.get(key) {
                    // Recursively merge nested values
                    let merged_value = base_value.merge_with_config(patch_value, &child_config)?;

                    // Check for null deletion
                    if matches!(merged_value, MergeValue::Null) {
                        merged.shift_remove(key);
                    } else {
                        merged.insert(key.clone(), merged_value);
                    }
                } else {
                    // Add new key (unless it's null)
                    if !matches!(patch_value, MergeValue::Null) {
                        merged.insert(key.clone(), patch_value.clone());
                    }
                }
            }

            Ok(MergeValue::Object(merged))
        }

        // Rule 3: Array merge based on strategy
        (MergeValue::Array(base_arr), MergeValue::Array(patch_arr)) => {
            match config.array_strategy {
                ArrayMergeStrategy::Replace => Ok(MergeValue::Array(patch_arr.clone())),
                ArrayMergeStrategy::MergeByKey => {
                    let merged = self.merge_arrays_by_key(base_arr, patch_arr)?;
                    Ok(MergeValue::Array(merged))
                }
                ArrayMergeStrategy::Concatenate => {
                    let mut result = base_arr.clone();
                    result.extend(patch_arr.clone());
                    Ok(MergeValue::Array(result))
                }
            }
        }

        // Rule 4: Primitive replacement
        (_, _) => Ok(other.clone()),
    }
}

// ===== PATTERN: Backward-Compatible merge() =====

pub fn merge(&self, other: &MergeValue) -> Result<Self> {
    // Default behavior: RFC 7396 semantics (replace arrays, null deletes keys)
    self.merge_with_config(other, &MergeConfig::default())
}

// ===== GOTCHA: IndexMap Order Preservation =====

// When deleting keys during merge, use shift_remove() instead of remove()
// to preserve the order of remaining keys.

if matches!(merged_value, MergeValue::Null) {
    merged.shift_remove(key);  // Preserves order of other keys
}

// ===== GOTCHA: Recursion Depth Check =====

// Always check max_depth at the START of merge_with_config, not after matching.
// This prevents stack overflow before any recursive calls.

if config.max_depth == 0 {
    return Err(JinError::Message("Maximum merge depth exceeded".to_string()));
}

// ===== PATTERN: Error Handling =====

// Use JinError::Message for custom errors in merge operations.
// The Result type is used for future conflict detection.

return Err(JinError::Message("Maximum merge depth exceeded".to_string()));
```

### Integration Points

```yaml
NO DATABASE CHANGES: Merge is pure computation, no persistence involved

NO CONFIG CHANGES: MergeConfig is a local parameter, not global config

NO ROUTE CHANGES: This is a library module, not exposed via HTTP

MODULE EXPORTS:
  - file: src/merge/mod.rs
  - add: pub use value::{MergeValue, ArrayMergeStrategy};
  - ensures: Public API includes new types

TEST COVERAGE:
  - file: tests/merge/value_test.rs
  - add: ~15-20 new tests for array strategies
  - add: ~5-10 new tests for edge cases
  - ensures: Comprehensive coverage of merge behaviors
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Check after each file modification - fix before proceeding
cargo check                          # Fast compilation check
cargo clippy --all-targets -- -D warnings  # Lint checking with warnings as errors

# Format check
cargo fmt -- --check                 # Verify formatting without making changes

# Run together (recommended)
cargo check && cargo clippy --all-targets -- -D warnings && cargo fmt -- --check

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.
# Common issues:
# - Must use #[allow(clippy::...)] for intentional patterns
# - Unused code warnings indicate incomplete implementation
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test only the merge module
cargo test --lib merge

# Test with output
cargo test --lib merge -- --nocapture

# Run specific test
cargo test test_merge_array_by_key

# Full test suite for merge module
cargo test --package jin_glm --lib merge

# Coverage (optional, if cargo-coverage is installed)
cargo tarpaulin --out Html --output-dir coverage

# Expected: All tests pass. If failing, debug root cause and fix implementation.
# Test file location: tests/merge/value_test.rs
```

### Level 3: Integration Testing (System Validation)

```bash
# Run all tests
cargo test --all

# Run tests in sequence (not parallel)
cargo test -- --test-threads=1

# Run with logging
RUST_LOG=debug cargo test --all

# Check for test compilation
cargo test --no-run

# Expected: All integration tests pass, no conflicts with existing functionality
```

### Level 4: RFC 7396 Compliance Validation

```bash
# Run specific RFC 7396 compliance tests
cargo test test_merge_rfc7396

# Verify null deletion behavior
cargo test test_merge_null_deletes_key
cargo test test_merge_null_in_nested_object_deletes

# Verify array replacement (RFC 7396 default)
cargo test test_merge_arrays_replace

# Verify deep object merge
cargo test test_merge_objects_deep
cargo test test_merge_three_levels_deep

# Expected: All RFC 7396 tests pass with correct behavior
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] `cargo test --all` passes with zero failures
- [ ] `cargo clippy --all-targets -- -D warnings` produces no warnings
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo check` completes without errors
- [ ] No new `unsafe` code introduced
- [ ] All public APIs have documentation

### Feature Validation

- [ ] `ArrayMergeStrategy::Replace` works correctly (default)
- [ ] `ArrayMergeStrategy::MergeByKey` merges by `id` field
- [ ] `ArrayMergeStrategy::MergeByKey` merges by `name` field when no `id`
- [ ] `ArrayMergeStrategy::Concatenate` appends arrays
- [ ] Null deletion works at all nesting levels
- [ ] Existing `merge()` method maintains backward compatibility
- [ ] Max depth limit prevents stack overflow
- [ ] Non-object array elements are preserved
- [ ] Order is preserved for object keys (IndexMap)

### Code Quality Validation

- [ ] Follows existing codebase patterns (match statements, Result returns)
- [ ] File placement matches desired codebase tree
- [ ] Naming conventions followed (PascalCase types, snake_case methods)
- [ ] Error handling uses JinError::Message for custom errors
- [ ] Public APIs have doc comments with examples
- [ ] Tests follow existing patterns (descriptive names, clear assertions)

### Documentation & Deployment

- [ ] `ArrayMergeStrategy` enum has complete doc comments
- [ ] `MergeConfig` struct has complete doc comments
- [ ] `merge_with_config` method has usage examples
- [ ] New tests have descriptive names and assertion messages
- [ ] RFC 7396 compliance documented in code comments

---

## Anti-Patterns to Avoid

- **Don't break existing merge() behavior** - All existing tests must pass without modification
- **Don't use unsafe code** - Rust's safety guarantees are essential
- **Don't ignore depth limits** - Unbounded recursion causes stack overflow
- **Don't use remove() instead of shift_remove()** - Order preservation is required
- **Don't forget to handle non-object array elements** - Primitives in arrays must be preserved
- **Don't assume all objects have "id" or "name"** - Must handle missing keys gracefully
- **Don't clone unnecessarily** - Consider borrowing where possible for performance
- **Don't ignore the Result type** - Merge operations can fail, handle errors properly
- **Don't add #[allow(...)] without justification** - Clippy warnings indicate real issues
- **Don't skip tests for edge cases** - Empty arrays, null values, deep nesting all need tests
- **Don't hardcode strategy selection** - Use config parameter to make it configurable
- **Don't forget to export new types** - Module must export ArrayMergeStrategy

---

## Example Test Cases

### Test 1: Keyed Array Merge by "id"

```rust
#[test]
fn test_merge_array_by_key_with_id() {
    let base = MergeValue::from_json(r#"
        [
            {"id": "server-a", "port": 8080},
            {"id": "server-b", "port": 8081}
        ]
    "#).expect("JSON parsing should succeed");

    let patch = MergeValue::from_json(r#"
        [
            {"id": "server-a", "port": 9090},
            {"id": "server-c", "port": 8082}
        ]
    "#).expect("JSON parsing should succeed");

    let config = MergeConfig {
        array_strategy: ArrayMergeStrategy::MergeByKey,
        ..Default::default()
    };

    let merged = base.merge_with_config(&patch, &config).expect("Merge should succeed");

    let arr = merged.as_array().expect("Result should be array");
    assert_eq!(arr.len(), 3, "Should have 3 elements");

    // server-a should have updated port
    let server_a = &arr[0];
    assert_eq!(server_a.as_object().unwrap().get("id").unwrap().as_str(), Some("server-a"));
    assert_eq!(server_a.as_object().unwrap().get("port").unwrap().as_i64(), Some(9090));

    // server-b should be unchanged
    let server_b = &arr[1];
    assert_eq!(server_b.as_object().unwrap().get("id").unwrap().as_str(), Some("server-b"));
    assert_eq!(server_b.as_object().unwrap().get("port").unwrap().as_i64(), Some(8081));

    // server-c should be added
    let server_c = &arr[2];
    assert_eq!(server_c.as_object().unwrap().get("id").unwrap().as_str(), Some("server-c"));
}
```

### Test 2: Backward Compatibility

```rust
#[test]
fn test_merge_backward_compatibility() {
    // This test must pass without any changes
    let base = MergeValue::from_json(r#"{"a": {"x": 1}, "b": 2}"#).expect("JSON parsing should succeed");
    let override_val = MergeValue::from_json(r#"{"a": {"y": 2}}"#).expect("JSON parsing should succeed");

    // Using original merge() method (default config)
    let merged = base.merge(&override_val).expect("Merge should succeed");

    let obj = merged.as_object().expect("Result should be object");
    let a_obj = obj.get("a").expect("Should have 'a' key").as_object().expect("'a' should be object");

    assert_eq!(a_obj.get("x").and_then(|v| v.as_i64()), Some(1), "Original value preserved");
    assert_eq!(a_obj.get("y").and_then(|v| v.as_i64()), Some(2), "New value added");
    assert_eq!(obj.get("b").and_then(|v| v.as_i64()), Some(2), "Unrelated key preserved");
}
```

### Test 3: RFC 7396 Null Deletion

```rust
#[test]
fn test_merge_rfc7396_null_deletion() {
    // Example from RFC 7396 Section 3
    let target = MergeValue::from_json(r#"
        {"title": "Goodbye!", "author": {"given": "John", "family": "Doe"}, "tags": ["example"]}
    "#).expect("JSON parsing should succeed");

    let patch = MergeValue::from_json(r#"
        {"title": "Hello!", "author": null, "tags": ["sample"]}
    "#).expect("JSON parsing should succeed");

    let merged = target.merge(&patch).expect("Merge should succeed");

    let obj = merged.as_object().expect("Result should be object");

    // title should be updated
    assert_eq!(obj.get("title").and_then(|v| v.as_str()), Some("Hello!"));

    // author key should be deleted (null in patch)
    assert!(!obj.contains_key("author"), "author key should be deleted");

    // tags array should be replaced (RFC 7396)
    let tags = obj.get("tags").and_then(|v| v.as_array()).expect("Should have tags");
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].as_str(), Some("sample"));
}
```

---

## Success Metrics

**Confidence Score**: 9/10 for one-pass implementation success

**Rationale**:
- Existing codebase has clear patterns to follow
- RFC 7396 specification is unambiguous
- Test patterns are well-established
- Dependencies are minimal (no new crates needed)
- Backward compatibility requirement prevents breaking changes

**Risk Mitigation**:
- Maintain existing `merge()` signature for backward compatibility
- Add new `merge_with_config()` for extended functionality
- Comprehensive test coverage ensures correctness
- Depth limit prevents stack overflow

**Validation**: The completed PRP provides sufficient context for an AI agent unfamiliar with the codebase to implement the feature successfully using only the PRP content and codebase access.
