# Array Merge Testing - Quick Reference for PRP

## Essential URLs for Array Merge Testing

### Primary Documentation (Must Reference)

1. **Kubernetes Strategic Merge Patch**
   - URL: https://github.com/kubernetes/community/blob/master/contributors/devel/sig-api-machinery/strategic-merge-patch.md
   - API: https://pkg.go.dev/k8s.io/apimachinery/pkg/util/strategicpatch
   - **Why:** Gold standard for key-based array merging in production
   - **Key Concepts:** Patch merge keys, $patch directives, $setElementOrder
   - **Test Patterns:**
     - Merge by id/name fields
     - Delete directives ($patch: delete)
     - Replace directives ($patch: replace)
     - Order control ($setElementOrder)

2. **webpack-merge**
   - URL: https://github.com/survivejs/webpack-merge
   - Docs: https://survivejs.com/books/webpack/developing/composing-configuration/
   - **Why:** Flexible per-field array merge strategies
   - **Key Concepts:** customizeArray, unique/append/prepend strategies
   - **Test Patterns:**
     - Per-field strategy specification
     - Wildcard pattern matching
     - Custom merge functions
     - Deduplicate by key

3. **Helm Chart Values**
   - URL: https://helm.sh/docs/chart_template_guide/values_files/
   - Blog: https://armel.soro.io/merging-dynamic-config-data-in-helm-charts/
   - **Why:** Documents array merge limitations and workarounds
   - **Key Concepts:** Arrays replaced not merged, workarounds
   - **Test Patterns:**
     - Test for data loss
     - Map-based alternatives
     - Separate base/override lists

4. **Docker Compose Merge**
   - URL: https://docs.docker.com/compose/how-tos/multiple-compose-files/merge/
   - Issue: https://github.com/docker/compose/issues/9756
   - **Why:** Examples of inconsistent merge behaviors
   - **Key Concepts:** Some fields append, others replace
   - **Test Patterns:**
     - Field-specific behavior tests
     - Append vs replace verification

5. **Lodash Bug Report**
   - URL: https://github.com/lodash/lodash/issues/1313
   - **Why:** Critical bug to avoid in your implementation
   - **Bug:** Empty arrays are ignored instead of replacing
   - **Test Pattern:**
     ```rust
     #[test]
     fn test_empty_array_override_not_ignored() {
         let base = json_to_merge(json!({"items": [1, 2, 3]}));
         let overlay = json_to_merge(json!({"items": []}));
         let result = deep_merge(base, overlay).unwrap();
         assert!(result["items"].as_array().unwrap().is_empty());
     }
     ```

### Rust Ecosystem

6. **merge Crate**
   - URL: https://docs.rs/merge/latest/merge/
   - **Why:** Rust-specific merge strategies and derive macros
   - **Key Concepts:** Strategy attributes, vec::append, vec::unique
   - **Test Patterns:**
     - Derive-based testing
     - Strategy per field
     - HashMap deep merge

7. **serde-toml-merge**
   - URL: https://github.com/jdrouet/serde-toml-merge
   - **Why:** TOML-specific merge challenges
   - **Key Concepts:** TOML null limitations
   - **Test Patterns:**
     - Format-specific constraint testing
     - Null handling verification

---

## Test Categories Required

### 1. Key-Based Merging Tests
```rust
// Test: Merge by id field
fn test_keyed_array_merge_by_id()
// Test: Merge by name field (fallback)
fn test_keyed_array_merge_by_name()
// Test: Custom key fields
fn test_keyed_array_with_custom_key()
// Test: Deep merge of matched items
fn test_keyed_array_deep_merge_items()
```

### 2. Order Preservation Tests
```rust
// Test: Base order maintained, new items appended
fn test_keyed_array_preserves_order()
// Test: Updated items stay in position
fn test_updated_items_maintain_position()
// Test: Deterministic ordering
fn test_ordering_is_deterministic()
```

### 3. Appending Behavior Tests
```rust
// Test: New items from overlay added
fn test_keyed_array_appends_new_items()
// Test: Empty array replaces (not ignored)
fn test_empty_overlay_array_replaces()
// Test: Empty base + overlay = overlay
fn test_empty_base_with_overlay()
```

### 4. Edge Case Tests
```rust
// Test: Missing keys → fallback to replace
fn test_array_with_missing_keys_falls_back_to_replace()
// Test: Duplicate keys → deduplicate
fn test_duplicate_keys_in_base_array()
// Test: Mixed types → replace
fn test_mixed_type_array_falls_back_to_replace()
// Test: Null values handled
fn test_null_values_in_array_handling()
```

### 5. Integration Tests
```rust
// Test: Multi-layer merge
fn test_array_merge_across_multiple_layers()
// Test: Conflict detection
fn test_unmergeable_arrays_create_conflict()
// Test: Cross-format merge
fn test_array_merge_different_formats()
```

---

## Critical Test Patterns (Copy These)

### Pattern 1: Key Match and Merge
```rust
#[test]
fn test_keyed_array_merge_by_key_field() {
    let base = json_to_merge(json!([
        {"id": "item1", "value": "a"},
        {"id": "item2", "value": "b"}
    ]));
    let overlay = json_to_merge(json!([
        {"id": "item1", "value": "A"}  // Update existing
    ]));

    let result = deep_merge(base, overlay).unwrap();
    let arr = result.as_array().unwrap();

    assert_eq!(arr.len(), 2);  // All items preserved

    let item1 = arr.iter()
        .find(|v| v.as_object().unwrap().get("id").unwrap().as_str() == Some("item1"))
        .unwrap();

    // Overlay value wins
    assert_eq!(item1.as_object().unwrap().get("value").unwrap().as_str(), Some("A"));

    // item2 preserved from base
    assert!(arr.iter().any(|v| v.as_object().unwrap().get("id").unwrap().as_str() == Some("item2")));
}
```

### Pattern 2: Order Preservation
```rust
#[test]
fn test_keyed_array_preserves_base_order_appends_new() {
    let base = json_to_merge(json!([
        {"id": "first"}, {"id": "second"}, {"id": "third"}
    ]));
    let overlay = json_to_merge(json!([
        {"id": "second"},  // Update existing
        {"id": "fourth"}   // Add new
    ]));

    let result = deep_merge(base, overlay).unwrap();
    let arr = result.as_array().unwrap();

    let ids: Vec<_> = arr.iter()
        .map(|v| v.as_object().unwrap().get("id").unwrap().as_str().unwrap())
        .collect();

    // Base order preserved, new items appended
    assert_eq!(ids, vec!["first", "second", "third", "fourth"]);
}
```

### Pattern 3: Empty Array Override (Critical - Lodash Bug)
```rust
#[test]
fn test_empty_array_override_not_ignored() {
    // This test would catch the Lodash bug where empty arrays are ignored
    let base = json_to_merge(json!([1, 2, 3]));
    let overlay = json_to_merge(json!([]));

    let result = deep_merge(base, overlay).unwrap();
    let arr = result.as_array().unwrap();

    assert!(arr.is_empty(), "Empty array should override, not be ignored");
}
```

### Pattern 4: Missing Keys Fallback
```rust
#[test]
fn test_array_with_missing_keys_falls_back_to_replace() {
    let base = json_to_merge(json!([
        {"id": "valid1", "value": 1},
        {"value": 2}  // Missing id field
    ]));
    let overlay = json_to_merge(json!([{"id": "new", "value": 3}]));

    let result = deep_merge(base, overlay).unwrap();
    let arr = result.as_array().unwrap();

    // Should replace entire array (not keyed merge)
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].as_object().unwrap().get("id").unwrap().as_str(), Some("new"));
}
```

### Pattern 5: Integration Test
```rust
#[test]
fn test_array_merge_across_multiple_layers() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;

    // Layer 1: Base array
    write_config(&fixture, r#"{"services": [{"name": "db", "port": 5432}]}"#)?;
    jin_cmd().args(["add", "config.json", "--global"]).assert().success();

    // Layer 2: Add to array
    write_config(&fixture, r#"{"services": [{"name": "cache", "port": 6379}]}"#)?;
    jin_cmd().args(["add", "config.json", "--mode"]).assert().success();

    // Layer 3: Update existing and add new
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

---

## Edge Cases to Test (Checklist)

- [ ] Empty base array + non-empty overlay
- [ ] Non-empty base + empty overlay (CRITICAL - Lodash bug)
- [ ] Both arrays empty
- [ ] Single item arrays
- [ ] Large arrays (performance)
- [ ] Missing key field in some items
- [ ] All items missing key field
- [ ] Duplicate keys in base array
- [ ] Duplicate keys in overlay array
- [ ] Same key in both base and overlay
- [ ] Keys with different casing
- [ ] Keys with special characters
- [ ] Mixed types in array (objects + primitives)
- [ ] Null values in array
- [ ] Nested objects within array items
- [ ] Nested arrays within array items
- [ ] Deeply nested structures (3+ levels)
- [ ] Unicode in key fields
- [ ] Very long key strings
- [ ] Whitespace in key fields

---

## External Best Practices Summary

### From Kubernetes:
- Use patch merge keys for object arrays
- Support delete directives
- Support replace directives
- Provide order control mechanisms
- Maintain backward compatibility

### From webpack-merge:
- Allow per-field strategy specification
- Support custom merge functions
- Provide built-in strategies (append, prepend, replace, unique)
- Use wildcard patterns for concise configuration

### From Helm:
- Document limitations clearly
- Provide workaround patterns
- Consider map-based alternatives
- Warn about data loss scenarios

### From Docker Compose:
- Be consistent with merge behavior
- Document field-specific behavior
- Support force override directives
- Handle null deletion

### From Lodash (Anti-Pattern):
- Don't ignore empty arrays
- Empty arrays should replace base arrays
- Test this edge case explicitly

---

## PRP References to Include

When creating your PRP, reference these URLs in the "External Research" section:

```markdown
## External Research

### Primary References
- [Kubernetes Strategic Merge Patch](https://github.com/kubernetes/community/blob/master/contributors/devel/sig-api-machinery/strategic-merge-patch.md) - Key-based array merging patterns
- [webpack-merge Documentation](https://github.com/survivejs/webpack-merge) - Per-field merge strategies
- [Helm Values Files](https://helm.sh/docs/chart_template_guide/values_files/) - Array merge limitations and workarounds
- [Docker Compose Merge](https://docs.docker.com/compose/how-tos/multiple-compose-files/merge/) - Field-specific merge behaviors

### Bug Reports to Avoid
- [Lodash Empty Array Bug](https://github.com/lodash/lodash/issues/1313) - Empty arrays must replace, not be ignored

### Rust Ecosystem
- [merge Crate](https://docs.rs/merge/latest/merge/) - Derive-based merge strategies
- [serde-toml-merge](https://github.com/jdrouet/serde-toml-merge) - Format-specific constraints

### Test Pattern Sources
- [Jin Project Tests](https://github.com/your-repo/jin/tree/main/tests) - Existing test patterns
- [ARRAY_MERGE_TESTING_BEST_PRACTICES.md](./ARRAY_MERGE_TESTING_BEST_PRACTICES.md) - Comprehensive test patterns
```

---

## Quick Start: Copy These Tests

1. **Basic key merge:**
   - `test_keyed_array_merge_by_id`
   - `test_keyed_array_merge_by_name`
   - `test_keyed_array_deep_merge_items`

2. **Order preservation:**
   - `test_keyed_array_preserves_order`
   - `test_keyed_array_preserves_base_order_appends_new`

3. **Appending:**
   - `test_keyed_array_appends_new_items`
   - `test_empty_overlay_array_replaces` (CRITICAL)

4. **Edge cases:**
   - `test_array_with_missing_keys_falls_back_to_replace`
   - `test_duplicate_keys_in_base_array`
   - `test_mixed_type_array_falls_back_to_replace`

5. **Integration:**
   - `test_array_merge_across_multiple_layers`
   - `test_nested_object_deep_merge`

---

## Success Criteria

Your array merge tests should verify:

1. **Correctness:**
   - Items matched by key are merged
   - Overlay values override base values
   - Nested objects merge recursively

2. **Preservation:**
   - Base order maintained
   - Unmatched base items preserved
   - No data loss

3. **Appending:**
   - New items added from overlay
   - Empty arrays replace (not ignored)

4. **Edge Cases:**
   - Missing keys → fallback to replace
   - Duplicate keys → deduplicated
   - Mixed types → replace

5. **Integration:**
   - Multiple layers merge correctly
   - Format constraints respected
   - Conflicts detected when appropriate

---

**Document Version:** 1.0
**Created:** January 12, 2026
**Purpose:** Quick reference for PRP creation with external URLs and test patterns
**Related:** ARRAY_MERGE_TESTING_BEST_PRACTICES.md (comprehensive version)
