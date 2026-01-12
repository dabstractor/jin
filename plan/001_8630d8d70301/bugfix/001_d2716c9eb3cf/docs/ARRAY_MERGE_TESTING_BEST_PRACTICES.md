# Array Merging Testing Best Practices - External Research

## Overview

This document compiles external best practices, patterns, and documentation URLs for testing array merging functionality in configuration management systems. It focuses on key-based merging, order preservation, new item appending, and edge case testing.

**Research Date:** January 12, 2026
**Scope:** Testing patterns for JSON/YAML array merging operations
**Focus Areas:** Key-based merging, order preservation, edge cases, integration testing

---

## Table of Contents

1. [External Documentation URLs](#external-documentation-urls)
2. [Testing Patterns for Key-Based Merging](#testing-patterns-for-key-based-merging)
3. [Testing Order Preservation](#testing-order-preservation)
4. [Testing New Item Appending](#testing-new-item-appending)
5. [Common Edge Cases to Test](#common-edge-cases-to-test)
6. [Integration Test Patterns](#integration-test-patterns)
7. [Well-Known Library Examples](#well-known-library-examples)
8. [Test Case Templates](#test-case-templates)

---

## External Documentation URLs

### Official Documentation

#### 1. **Kubernetes Strategic Merge Patch**
- **Primary Documentation:** https://github.com/kubernetes/community/blob/master/contributors/devel/sig-api-machinery/strategic-merge-patch.md
- **API Package:** https://pkg.go.dev/k8s.io/apimachinery/pkg/util/strategicpatch
- **Key Topics:**
  - Patch merge keys (`x-kubernetes-patch-merge-key`)
  - Patch strategies (merge vs replace)
  - Special directives (`$patch: replace`, `$patch: delete`)
  - `$setElementOrder` for ordering control
- **Relevance:** Gold standard for key-based array merging in production systems

#### 2. **webpack-merge Documentation**
- **GitHub Repository:** https://github.com/survivejs/webpack-merge
- **NPM Package:** https://www.npmjs.com/package/webpack-merge
- **Documentation:** https://survivejs.com/books/webpack/developing/composing-configuration/
- **Key Topics:**
  - `customizeArray` for per-field strategies
  - Built-in strategies: append, prepend, replace
  - Custom merge functions
  - Wildcard pattern matching
  - Unique/deduplicate strategies
- **Relevance:** Practical examples of flexible array merging strategies

#### 3. **Helm Chart Values Documentation**
- **Values Files Guide:** https://helm.sh/docs/chart_template_guide/values_files/
- **Blog Post on Dynamic Config:** https://armel.soro.io/merging-dynamic-config-data-in-helm-charts/
- **Key Topics:**
  - Known limitations of array merging
  - Workaround patterns (separate keys, map-based approach)
  - `mustMergeOverwrite` function
- **Relevance:** Real-world pitfalls and solutions for array merging

#### 4. **Docker Compose Merge Documentation**
- **Official Docs:** https://docs.docker.com/compose/how-tos/multiple-compose-files/merge/
- **GitHub Issue on Inconsistencies:** https://github.com/docker/compose/issues/9756
- **Key Topics:**
  - Inconsistent array merging across fields
  - Append vs replace behavior by field
  - Force override with `!override` directive
  - Null deletion behavior
- **Relevance:** Examples of inconsistent merge behaviors to avoid

#### 5. **ESLint Configuration Documentation**
- **Configuration Files:** https://eslint.org/docs/latest/use/configure/configuration-files
- **Combine Configs:** https://eslint.org/docs/latest/use/configure/combine-configs/
- **Key Topics:**
  - Extends array merging
  - Rule merging behavior (severity override, options preservation)
  - Override blocks with higher precedence
- **Relevance:** Specialized rule-based merging patterns

### Rust Ecosystem Libraries

#### 6. **merge Crate (Rust)**
- **Crates.io:** https://crates.io/crates/merge
- **Documentation:** https://docs.rs/merge/latest/merge/
- **Lib.rs:** https://lib.rs/crates/merge
- **Key Topics:**
  - Derive-based merge strategies
  - Built-in strategies: `merge::vec::append`, `merge::vec::unique`
  - Strategy attributes for structs
- **Relevance:** Rust-specific implementation patterns and testing

#### 7. **serde-toml-merge**
- **GitHub:** https://github.com/jdrouet/serde-toml-merge
- **Crates.io:** https://crates.io/crates/serde-toml-merge
- **Key Topics:**
  - TOML-specific merging challenges
  - Integration with serde
- **Relevance:** Format-specific testing considerations

#### 8. **deepmerge Crate**
- **Documentation:** https://docs.rs/deepmerge/latest/deepmerge/
- **Key Topics:**
  - Flexible merge algorithms
  - Trait-based customization
- **Relevance:** Alternative merge implementations for comparison testing

### Community Resources

#### 9. **JavaScript Array Merging**
- **Lodash Empty Array Bug:** https://github.com/lodash/lodash/issues/1313
- **Merge and Deduplicate:** https://xjavascript.com/blog/how-to-merge-two-arrays-in-javascript-and-de-duplicate-items-while-preserving-original-order/
- **Key Topics:**
  - Empty array edge case bugs
  - Order preservation techniques
- **Relevance:** Common pitfalls to test for

#### 10. **Testing Best Practices**
- **PHPUnit Array Testing:** https://www.php.net/manual/en/function.array-merge.php
- **Array Merge vs Union:** https://www.geeksforgeeks.org/php/what-is-the-difference-between-array-merge-and-array-array-in-php/
- **Key Topics:**
  - Test patterns for array operations
  - Comparison of merge strategies
- **Relevance:** Cross-language testing patterns

---

## Testing Patterns for Key-Based Merging

### Pattern 1: Basic Key Match and Merge

**Purpose:** Verify that items with matching keys are merged correctly

**Test Template:**
```rust
#[test]
fn test_keyed_array_merge_by_key_field() {
    // Arrange
    let base = json_to_merge(json!([
        {"id": "item1", "value": "a", "metadata": {"version": 1}},
        {"id": "item2", "value": "b", "metadata": {"version": 1}}
    ]));
    let overlay = json_to_merge(json!([
        {"id": "item1", "value": "A", "metadata": {"updated": true}}
    ]));

    // Act
    let result = deep_merge(base, overlay).unwrap();

    // Assert
    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 2, "Should preserve all base items");

    // Find merged item
    let item1 = arr.iter()
        .find(|v| v.as_object().unwrap().get("id").unwrap().as_str() == Some("item1"))
        .unwrap();
    let obj = item1.as_object().unwrap();

    // Verify overlay value takes precedence
    assert_eq!(obj.get("value").unwrap().as_str(), Some("A"));

    // Verify nested merge
    let metadata = obj.get("metadata").unwrap().as_object().unwrap();
    assert_eq!(metadata.get("version").unwrap().as_i64(), Some(1));
    assert_eq!(metadata.get("updated").unwrap().as_bool(), Some(true));
}
```

**Key Assertions:**
- Key matching works correctly
- Overlay values override base values
- Nested objects merge recursively
- Unmatched base items are preserved

**External Reference:** Kubernetes Strategic Merge Patch - Section on patch merge keys

### Pattern 2: Multiple Key Field Fallback

**Purpose:** Test fallback behavior when primary key field is missing

**Test Template:**
```rust
#[test]
fn test_keyed_array_merge_with_key_fallback() {
    // Test with "name" field when "id" is absent
    let base = json_to_merge(json!([
        {"name": "service-a", "port": 8080},
        {"name": "service-b", "port": 3000}
    ]));
    let overlay = json_to_merge(json!([
        {"name": "service-a", "port": 9090}
    ]));

    let result = deep_merge(base, overlay).unwrap();
    let arr = result.as_array().unwrap();

    assert_eq!(arr.len(), 2);
    let service = arr.iter()
        .find(|v| v.as_object().unwrap().get("name").unwrap().as_str() == Some("service-a"))
        .unwrap();
    assert_eq!(service.as_object().unwrap().get("port").unwrap().as_i64(), Some(9090));
}
```

**Key Assertions:**
- Falls back from "id" to "name" correctly
- Merge succeeds with alternative key field

**External Reference:** ARRAY_MERGE_STRATEGIES.md - Section on Key Field Options

### Pattern 3: Custom Key Fields

**Purpose:** Test custom key field specification

**Test Template:**
```rust
#[test]
fn test_keyed_array_merge_with_custom_key() {
    let config = MergeConfig::with_key_fields(vec!["uuid".into(), "key".into()]);

    let base = json_to_merge(json!([
        {"uuid": "123e4567-e89b-12d3-a456-426614174000", "data": "base"}
    ]));
    let overlay = json_to_merge(json!([
        {"uuid": "123e4567-e89b-12d3-a456-426614174000", "data": "overlay"}
    ]));

    let result = deep_merge_with_config(base, overlay, &config).unwrap();
    let arr = result.as_array().unwrap();

    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].as_object().unwrap().get("data").unwrap().as_str(), Some("overlay"));
}
```

**Key Assertions:**
- Custom key fields are respected
- Merge works with non-standard key names

**External Reference:** webpack-merge `customizeArray` documentation

---

## Testing Order Preservation

### Pattern 4: Base Order Preservation with New Items Appended

**Purpose:** Verify that original order is maintained and new items are added at the end

**Test Template:**
```rust
#[test]
fn test_keyed_array_preserves_base_order_appends_new() {
    let base = json_to_merge(json!([
        {"id": "first", "value": 1},
        {"id": "second", "value": 2},
        {"id": "third", "value": 3}
    ]));
    let overlay = json_to_merge(json!([
        {"id": "second", "value": 20},  // Update existing
        {"id": "fourth", "value": 4}    // Add new
    ]));

    let result = deep_merge(base, overlay).unwrap();
    let arr = result.as_array().unwrap();

    // Verify order
    let ids: Vec<_> = arr.iter()
        .map(|v| v.as_object().unwrap().get("id").unwrap().as_str().unwrap())
        .collect();

    assert_eq!(ids, vec!["first", "second", "third", "fourth"],
               "Base order preserved, new items appended");
}
```

**Key Assertions:**
- Base items maintain original relative order
- Updated items stay in original position
- New items appear after all base items

**External Reference:**
- Kubernetes `$setElementOrder` directive
- ARRAY_MERGE_STRATEGIES.md - Section on Ordering After Merge

### Pattern 5: Explicit Order Control

**Purpose:** Test explicit ordering directives (if supported)

**Test Template:**
```rust
#[test]
fn test_keyed_array_with_explicit_order() {
    // Base has items in one order
    let base = json_to_merge(json!([
        {"id": "c", "value": 3},
        {"id": "a", "value": 1},
        {"id": "b", "value": 2}
    ]));
    let overlay = json_to_merge(json!([
        {"id": "a", "value": 10},
        {"id": "b", "value": 20}
    ]));

    let result = deep_merge(base, overlay).unwrap();
    let arr = result.as_array().unwrap();

    // Verify deterministic ordering
    let ids: Vec<_> = arr.iter()
        .map(|v| v.as_object().unwrap().get("id").unwrap().as_str().unwrap())
        .collect();

    // Order should be predictable and consistent
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&"a"));
    assert!(ids.contains(&"b"));
    assert!(ids.contains(&"c"));
}
```

**Key Assertions:**
- Ordering is deterministic
- Order is consistent across multiple runs
- Order is reproducible

**External Reference:** Kubernetes `$setElementOrder` directive examples

---

## Testing New Item Appending

### Pattern 6: New Items from Overlay

**Purpose:** Verify that items only in overlay are added to result

**Test Template:**
```rust
#[test]
fn test_keyed_array_appends_overlay_only_items() {
    let base = json_to_merge(json!([
        {"id": "existing", "value": 1}
    ]));
    let overlay = json_to_merge(json!([
        {"id": "new1", "value": 2},
        {"id": "new2", "value": 3}
    ]));

    let result = deep_merge(base, overlay).unwrap();
    let arr = result.as_array().unwrap();

    assert_eq!(arr.len(), 3);
    assert!(arr.iter().any(|v| {
        v.as_object().unwrap().get("id").unwrap().as_str() == Some("new1")
    }));
    assert!(arr.iter().any(|v| {
        v.as_object().unwrap().get("id").unwrap().as_str() == Some("new2")
    }));
}
```

**Key Assertions:**
- New items are added
- No duplicate keys in result
- All base items preserved

**External Reference:** Docker Compose append behavior documentation

### Pattern 7: Empty Overlay Array Behavior

**Purpose:** Test that empty overlay explicitly clears the array

**Test Template:**
```rust
#[test]
fn test_empty_overlay_array_replaces_base() {
    let base = json_to_merge(json!([1, 2, 3, 4, 5]));
    let overlay = json_to_merge(json!([]));

    let result = deep_merge(base, overlay).unwrap();
    let arr = result.as_array().unwrap();

    assert!(arr.is_empty(), "Empty array should replace base array");
}
```

**Key Assertions:**
- Empty array is not ignored (unlike Lodash bug)
- Represents explicit override to empty
- Distinguishes from null deletion

**External Reference:** Lodash Issue #1313 on empty array bug

---

## Common Edge Cases to Test

### Edge Case 1: Missing Keys (Fallback to Replace)

**Test Template:**
```rust
#[test]
fn test_array_with_missing_keys_falls_back_to_replace() {
    // Array with some items lacking key field
    let base = json_to_merge(json!([
        {"id": "valid1", "value": 1},
        {"value": 2}  // Missing id field
    ]));
    let overlay = json_to_merge(json!([
        {"id": "new", "value": 3}
    ]));

    let result = deep_merge(base, overlay).unwrap();
    let arr = result.as_array().unwrap();

    // Should replace entire array (not keyed merge)
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].as_object().unwrap().get("id").unwrap().as_str(), Some("new"));
}
```

**Key Assertions:**
- Falls back to replace when keys missing
- Doesn't attempt partial keyed merge
- Behavior is documented and consistent

**External Reference:** ARRAY_MERGE_STRATEGIES.md - Section on Handling Missing Keys

### Edge Case 2: Duplicate Keys in Source

**Test Template:**
```rust
#[test]
fn test_duplicate_keys_in_base_array() {
    let base = json_to_merge(json!([
        {"id": "dup", "value": 1},
        {"id": "dup", "value": 2}  // Duplicate key
    ]));
    let overlay = json_to_merge(json!([
        {"id": "dup", "value": 3}
    ]));

    let result = deep_merge(base, overlay).unwrap();
    let arr = result.as_array().unwrap();

    // Verify "last wins" behavior
    let matching: Vec<_> = arr.iter()
        .filter(|v| v.as_object().unwrap().get("id").unwrap().as_str() == Some("dup"))
        .collect();

    assert_eq!(matching.len(), 1, "Should deduplicate by key");
    assert_eq!(matching[0].as_object().unwrap().get("value").unwrap().as_i64(), Some(3));
}
```

**Key Assertions:**
- Duplicate keys are handled
- Last occurrence wins (or first, depending on strategy)
- Result has no duplicate keys

**External Reference:** ARRAY_MERGE_STRATEGIES.md - Section on Handling Duplicate Keys

### Edge Case 3: Mixed Type Arrays

**Test Template:**
```rust
#[test]
fn test_mixed_type_array_falls_back_to_replace() {
    let base = json_to_merge(json!([
        {"id": "1", "value": "object"},
        42,  // Primitive in object array
        "string"
    ]));
    let overlay = json_to_merge(json!([1, 2, 3]));

    let result = deep_merge(base, overlay).unwrap();
    let arr = result.as_array().unwrap();

    // Should replace (can't do keyed merge on mixed types)
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0].as_i64(), Some(1));
}
```

**Key Assertions:**
- Mixed types detected and handled
- Falls back to replace strategy
- No crashes or panics

**External Reference:** ARRAY_MERGE_STRATEGIES.md - Section on Mixed Types in Arrays

### Edge Case 4: Null Values in Arrays

**Test Template:**
```rust
#[test]
fn test_null_values_in_array_handling() {
    let base = json_to_merge(json!([
        {"id": "1", "value": "a"},
        null,  // Null element
        {"id": "2", "value": "b"}
    ]));
    let overlay = json_to_merge(json!([]));

    let result = deep_merge(base, overlay).unwrap();

    // Behavior depends on format support
    // JSON/YAML support null, TOML doesn't
    assert!(result.as_array().unwrap().is_empty());
}
```

**Key Assertions:**
- Null values handled according to format
- No ambiguous behavior
- Format constraints respected

**External Reference:**
- ARRAY_MERGE_STRATEGIES.md - Section on Null Elements in Arrays
- serde-toml-merge format constraints

### Edge Case 5: Empty Base Array

**Test Template:**
```rust
#[test]
fn test_merge_with_empty_base_array() {
    let base = json_to_merge(json!([]));
    let overlay = json_to_merge(json!([
        {"id": "new", "value": 1}
    ]));

    let result = deep_merge(base, overlay).unwrap();
    let arr = result.as_array().unwrap();

    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].as_object().unwrap().get("id").unwrap().as_str(), Some("new"));
}
```

**Key Assertions:**
- Empty base + overlay = overlay
- No special casing needed
- Clean behavior

---

## Integration Test Patterns

### Pattern 8: Multi-Layer Merge Integration

**Purpose:** Test array merging across multiple configuration layers

**Test Template:**
```rust
#[test]
fn test_array_merge_across_multiple_layers() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;

    // Layer 1 (Base): Add initial array
    write_config(&fixture, r#"{"services": [{"name": "db", "port": 5432}]}"#)?;
    jin_cmd().args(["add", "config.json", "--global"]).assert().success();

    // Layer 2 (Mode): Add to array
    write_config(&fixture, r#"{"services": [{"name": "cache", "port": 6379}]}"#)?;
    jin_cmd().args(["add", "config.json", "--mode"]).assert().success();

    // Layer 3 (Project): Update existing and add new
    write_config(&fixture, r#"{"services": [{"name": "db", "port": 5433}, {"name": "api", "port": 8080}]}"#)?;
    jin_cmd().args(["add", "config.json"]).assert().success();

    // Apply merge
    jin_cmd().arg("apply").assert().success();

    // Verify merged result
    let merged = read_config(&fixture)?;
    let services = merged["services"].as_array().unwrap();

    assert_eq!(services.len(), 3);
    assert!(services.iter().any(|s| s["name"] == "db" && s["port"] == 5433));
    assert!(services.iter().any(|s| s["name"] == "cache" && s["port"] == 6379));
    assert!(services.iter().any(|s| s["name"] == "api" && s["port"] == 8080));

    Ok(())
}
```

**Key Assertions:**
- Multiple layers merge correctly
- Higher layers override lower layers
- All items from all layers present
- No data loss

**External Reference:** Helm multi-file values merging

### Pattern 9: Conflict Detection for Arrays

**Purpose:** Test that unmergeable arrays generate conflicts

**Test Template:**
```rust
#[test]
fn test_unmergeable_arrays_create_conflict() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;

    // Base: Object array with keys
    write_config(&fixture, r#"{"items": [{"id": "1", "value": "a"}]}"#)?;
    jin_cmd().args(["add", "config.json"]).assert().success();

    // Overlay: Primitive array (incompatible types)
    write_config(&fixture, r#"{"items": [1, 2, 3]}"#)?;
    jin_cmd().args(["add", "config.json", "--mode"]).assert().success();

    // Apply should detect conflict
    let result = jin_cmd().arg("apply").assert();
    // Expect conflict handling behavior

    Ok(())
}
```

**Key Assertions:**
- Incompatible array types detected
- Conflict files created when appropriate
- User can resolve conflicts
- No silent data loss

**External Reference:** Docker Compose field-specific merge behaviors

### Pattern 10: Format-Specific Array Merging

**Purpose:** Test array merging across different file formats

**Test Template:**
```rust
#[test]
fn test_array_merge_different_formats() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;

    // JSON base
    write_config(&fixture, "config.json", r#"{"ports": [8080, 8081]}"#)?;
    jin_cmd().args(["add", "config.json"]).assert().success();

    // YAML overlay
    write_config(&fixture, "config.yaml", "ports:\n  - 8082\n  - 8083\n")?;
    jin_cmd().args(["add", "config.yaml", "--mode"]).assert().success();

    // Apply and verify
    jin_cmd().arg("apply").assert().success();

    // Result should be format-appropriate
    let merged = read_merged_config(&fixture)?;
    let ports = merged["ports"].as_array().unwrap();
    assert!(ports.len() >= 2);

    Ok(())
}
```

**Key Assertions:**
- Format-specific constraints respected
- TOML null handling correct
- INI nesting limitations handled
- Round-trip preservation

**External Reference:**
- serde-toml-merge format documentation
- ARRAY_MERGE_STRATEGIES.md - Format-Specific Constraints

---

## Well-Known Library Examples

### serde_json (Rust)

**Documentation:** https://docs.rs/serde_json/latest/serde_json/

**Testing Pattern:**
```rust
#[test]
fn test_serde_json_value_merge() {
    use serde_json::json;

    let base = json!({
        "items": [
            {"id": "1", "value": "a"},
            {"id": "2", "value": "b"}
        ]
    });

    let overlay = json!({
        "items": [
            {"id": "1", "value": "A"}
        ]
    });

    // serde_json doesn't have built-in deep merge
    // Implement custom merge or use external crate
    let merged = custom_deep_merge(base, overlay);

    assert_eq!(merged["items"][0]["value"], "A");
    assert_eq!(merged["items"][1]["value"], "b");
}
```

**Key Points:**
- No built-in array merge by key
- Requires custom implementation
- Good reference for manual merge testing

**External Reference:** serde_json documentation

### Lodash merge (JavaScript)

**Documentation:** https://lodash.com/docs/4.17.15#merge

**Known Bug - Empty Arrays:**
```javascript
// Lodash 4.17.15
_.merge({ x: [1,2,3] }, { x: [] })
// Result: { x: [1,2,3] }  // BUG: empty array ignored!

// Expected behavior:
// Result: { x: [] }  // empty array should override
```

**Test to Prevent This Bug:**
```rust
#[test]
fn test_empty_array_override_not_ignored() {
    // This test would catch the Lodash bug
    let base = json_to_merge(json!({"items": [1, 2, 3]}));
    let overlay = json_to_merge(json!({"items": []}));

    let result = deep_merge(base, overlay).unwrap();
    let items = result.as_object().unwrap().get("items").unwrap().as_array().unwrap();

    assert!(items.is_empty(), "Empty array should override, not be ignored");
}
```

**Key Points:**
- Don't repeat the Lodash bug!
- Empty arrays must replace base arrays
- Test this explicitly

**External Reference:** Lodash Issue #1313

### webpack-merge (JavaScript)

**Documentation:** https://github.com/survivejs/webpack-merge

**Testing Pattern from Examples:**
```javascript
describe('webpack-merge array strategies', () => {
  test('customizeArray with append', () => {
    const base = {
      plugins: [new PluginA()]
    };

    const overlay = {
      plugins: [new PluginB()]
    };

    const result = mergeWithCustomize({
      customizeArray: customizeArray({
        'plugins': 'append'
      })
    })(base, overlay);

    expect(result.plugins).toHaveLength(2);
  });

  test('unique strategy for plugins', () => {
    const result = mergeWithCustomize({
      customizeArray: unique(
        'plugins',
        ['HotModuleReplacementPlugin'],
        (plugin) => plugin.constructor?.name
      )
    })({
      plugins: [new webpack.HotModuleReplacementPlugin()]
    }, {
      plugins: [new webpack.HotModuleReplacementPlugin()]
    });

    expect(result.plugins).toHaveLength(1);
  });
});
```

**Key Points:**
- Test per-field strategies
- Test deduplication
- Test custom functions

**External Reference:** webpack-merge README and tests

### Helm (Go)

**Documentation:** https://helm.sh/docs/chart_template_guide/values_files/

**Testing Pattern:**
```go
func TestHelmArrayMerging(t *testing.T) {
    // Helm arrays are replaced, not merged
    base := `
plugins:
  - name: auth
    version: "1.0"
`
    overlay := `
plugins:
  - name: cache
    version: "2.0"
`

    // Result: plugins only contains cache
    // auth from base is LOST
    // This is Helm's known limitation
}
```

**Key Points:**
- Test for data loss scenarios
- Document limitations
- Provide workarounds

**External Reference:** Helm documentation on array limitations

---

## Test Case Templates

### Template 1: Comprehensive Array Merge Test Suite

```rust
#[cfg(test)]
mod keyed_array_merge_tests {
    use super::*;

    mod basic_merging {
        use super::*;

        #[test]
        fn test_merge_by_single_key() {
            // TODO: Implement
        }

        #[test]
        fn test_merge_by_multiple_keys() {
            // TODO: Implement
        }

        #[test]
        fn test_fallback_from_id_to_name() {
            // TODO: Implement
        }
    }

    mod order_preservation {
        use super::*;

        #[test]
        fn test_base_order_maintained() {
            // TODO: Implement
        }

        #[test]
        fn test_new_items_appended() {
            // TODO: Implement
        }

        #[test]
        fn test_updated_items_stay_in_position() {
            // TODO: Implement
        }
    }

    mod appending_behavior {
        use super::*;

        #[test]
        fn test_overlay_only_items_added() {
            // TODO: Implement
        }

        #[test]
        fn test_empty_overlay_clears_array() {
            // TODO: Implement
        }

        #[test]
        fn test_empty_base_with_overlay() {
            // TODO: Implement
        }
    }

    mod edge_cases {
        use super::*;

        #[test]
        fn test_missing_keys_falls_back_to_replace() {
            // TODO: Implement
        }

        #[test]
        fn test_duplicate_keys_deduplicated() {
            // TODO: Implement
        }

        #[test]
        fn test_mixed_types_replace() {
            // TODO: Implement
        }

        #[test]
        fn test_null_values_handled() {
            // TODO: Implement
        }

        #[test]
        fn test_nested_object_merging() {
            // TODO: Implement
        }

        #[test]
        fn test_deeply_nested_arrays() {
            // TODO: Implement
        }
    }

    mod integration {
        use super::*;

        #[test]
        fn test_multi_layer_merge() {
            // TODO: Implement
        }

        #[test]
        fn test_conflict_detection() {
            // TODO: Implement
        }

        #[test]
        fn test_cross_format_merge() {
            // TODO: Implement
        }
    }
}
```

### Template 2: Property-Based Testing

```rust
#[cfg(test)]
mod proptests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_keyed_merge_preserves_all_keys(
            base_items in prop::collection::vec(any::<TestItem>(), 0..10),
            overlay_items in prop::collection::vec(any::<TestItem>(), 0..10)
        ) {
            // Property: No unique keys should be lost
            let base = json_to_merge(json!(base_items));
            let overlay = json_to_merge(json!(overlay_items));

            let result = deep_merge(base, overlay).unwrap();
            let arr = result.as_array().unwrap();

            // Verify all unique keys present
            let base_keys: HashSet<_> = base_items.iter()
                .filter_map(|i| i.id.as_ref())
                .collect();
            let overlay_keys: HashSet<_> = overlay_items.iter()
                .filter_map(|i| i.id.as_ref())
                .collect();
            let result_keys: HashSet<_> = arr.iter()
                .filter_map(|v| v.as_object().unwrap().get("id"))
                .filter_map(|v| v.as_str())
                .collect();

            let expected: HashSet<_> = base_keys.union(&overlay_keys).cloned().collect();
            prop_assert_eq!(result_keys, expected);
        }
    }
}
```

---

## Summary of Best Practices

### 1. **Always Test Key-Based Merging**
- Match items by key field
- Merge matched items recursively
- Add new items from overlay
- Preserve unmatched base items

### 2. **Test Order Preservation**
- Base items maintain relative order
- Updated items stay in original position
- New items appended (or explicitly ordered)

### 3. **Test Appending Behavior**
- New items from overlay are added
- Empty overlay array clears base array
- Empty base array + overlay = overlay

### 4. **Test Edge Cases**
- Missing keys → fallback to replace
- Duplicate keys → deduplicate
- Mixed types → replace
- Null values → format-specific handling
- Empty arrays → explicit override

### 5. **Write Integration Tests**
- Multi-layer merges
- Format-specific constraints
- Conflict detection
- Real-world scenarios

### 6. **Learn from Others**
- Study Kubernetes strategic merge patch
- Review webpack-merge strategies
- Understand Helm's limitations
- Avoid Lodash's empty array bug

### 7. **Document Everything**
- Merge behavior per field
- Fallback strategies
- Format constraints
- Known limitations

---

## Quick Reference URLs

**Primary Sources:**
- [Kubernetes Strategic Merge Patch](https://github.com/kubernetes/community/blob/master/contributors/devel/sig-api-machinery/strategic-merge-patch.md)
- [webpack-merge GitHub](https://github.com/survivejs/webpack-merge)
- [Helm Values Files](https://helm.sh/docs/chart_template_guide/values_files/)
- [Docker Compose Merge](https://docs.docker.com/compose/how-tos/multiple-compose-files/merge/)
- [merge Crate (Rust)](https://docs.rs/merge/latest/merge/)

**Bug Reports:**
- [Lodash Empty Array Bug](https://github.com/lodash/lodash/issues/1313)
- [Docker Compose Array Issues](https://github.com/docker/compose/issues/9756)

**Community:**
- [Helm Dynamic Config Merge](https://armel.soro.io/merging-dynamic-config-data-in-helm-charts/)
- [PHP Array Merge vs Union](https://www.geeksforgeeks.org/php/what-is-the-difference-between-array-merge-and-array-array-in-php/)

---

## Next Steps for jin Project

Based on this research, the jin project should:

1. **Implement comprehensive test suite** using templates above
2. **Add property-based tests** for edge case coverage
3. **Document merge behavior** in user-facing docs
4. **Provide examples** in documentation
5. **Cross-reference** with existing research in ARRAY_MERGE_STRATEGIES.md
6. **Consider adding** explicit order control directives (like Kubernetes)
7. **Test format-specific** constraints thoroughly
8. **Benchmark** performance with large arrays

---

**Document Version:** 1.0
**Last Updated:** January 12, 2026
**Related Documents:**
- ARRAY_MERGE_RESEARCH_INDEX.md
- ARRAY_MERGE_STRATEGIES.md
- ARRAY_MERGE_CODE_EXAMPLES.md
- ARRAY_MERGE_SUMMARY.md
