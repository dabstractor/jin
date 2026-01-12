# Array Merge Strategies - Quick Summary

## Key Takeaways

This research covers array merging in configuration file systems across multiple real-world tools and best practices for implementation.

## The Five Core Strategies

| Strategy | Behavior | Use Case | Example Tool |
|----------|----------|----------|--------------|
| **Replace** | Overlay array replaces base entirely | Simple overrides | Kubernetes (default), Docker Compose (command) |
| **Append** | Overlay items added to end of base | Incremental configuration | webpack-merge (default), Docker Compose (ports) |
| **Prepend** | Overlay items inserted at start | Priority/precedence | webpack-merge (customizable), ESLint (extends) |
| **Union** | Unique elements only (deduplication) | De-duplication | webpack-merge (unique strategy) |
| **Keyed Merge** | Objects merged by id/name field | Complex object merging | Kubernetes SMP, jin project |

## Real-World Implementations

### Kubernetes Strategic Merge Patch
- **Key Feature:** `x-kubernetes-patch-merge-key` and `x-kubernetes-patch-strategy` directives
- **Supports:** Delete, replace, ordering directives
- **Best for:** Pod/container configuration with complex arrays
- **Reference:** [GitHub strategic-merge-patch.md](https://github.com/kubernetes/community/blob/master/contributors/devel/sig-api-machinery/strategic-merge-patch.md)

### webpack-merge customizeArray
- **Key Feature:** Per-field strategy specification via `customizeArray()`
- **Supports:** append, prepend, replace, custom functions, wildcards
- **Best for:** Webpack configuration composition
- **Reference:** [GitHub webpack-merge](https://github.com/survivejs/webpack-merge)

### Helm Chart Merging
- **Current Behavior:** Deep merge for objects, replace for arrays (known limitation)
- **Workaround:** Use separate keys (basePlugins + prodPlugins) or map-based config
- **Reference:** [Helm Values Files](https://helm.sh/docs/chart_template_guide/values_files/)

### Docker Compose Override
- **Behavior:** Append for most arrays (ports, expose, etc.), replace for specific fields (command, entrypoint)
- **Tools:** Use `!override` directive or `null` to force behavior
- **Known Issue:** Inconsistent merging across different array types
- **Reference:** [Docker Merge Documentation](https://docs.docker.com/compose/how-tos/multiple-compose-files/merge/)

### ESLint Configuration
- **Extends Array:** Inheritance chain with special rule merging
- **Special Behavior:** Rule severity from later config, options preserved from earlier
- **Override Blocks:** Higher precedence than regular config
- **Reference:** [ESLint Configuration](https://eslint.org/docs/latest/use/configure/configuration-files)

## Keyed Array Merging (Most Complex)

**When to Use:**
- Arrays of objects with unique identifiers
- Need element-level merge instead of array-level
- Kubernetes pods, services, environment variables

**Key Challenges:**
1. **Identifying Keys:** Use "id" field, fallback to "name"
2. **Missing Keys:** Fall back to replace strategy if any item lacks key
3. **Duplicate Keys:** Decide: last wins, first wins, merge, or error
4. **Ordering:** Preserve base order + append new, or explicit sort
5. **Deletion:** Use null markers, delete directives, or exclude lists

**Example (jin project):**
```rust
fn merge_arrays(base: Vec<MergeValue>, overlay: Vec<MergeValue>) -> Result<Vec<MergeValue>> {
    let base_keyed = extract_array_keys(&base);
    let overlay_keyed = extract_array_keys(&overlay);

    if let (Some(base_map), Some(overlay_map)) = (base_keyed, overlay_keyed) {
        // Merge by key - base + overlay items
        // Matching keys: recursive merge
        // New keys: add from overlay
    } else {
        // Can't do keyed merge - replace with overlay
        Ok(overlay)
    }
}
```

## Configuration Patterns

### Pattern 1: Per-Field Strategy Annotation
```rust
#[derive(Merge)]
struct Config {
    #[merge(strategy = merge::vec::append)]
    plugins: Vec<Plugin>,

    #[merge(strategy = merge::vec::keyed_by_id)]
    services: Vec<Service>,
}
```

### Pattern 2: Schema-Driven Rules
```json
{
  "services": {
    "type": "array",
    "x-merge-strategy": "keyed",
    "x-merge-key": "name"
  }
}
```

### Pattern 3: External Configuration
```yaml
mergeRules:
  keyed_array:
    - path: [database, servers]
      key: hostname
```

### Pattern 4: Custom Function
```javascript
customizeArray(a, b, key) {
  if (key === 'plugins') return _.uniq([...a, ...b]);
  return undefined;  // use default
}
```

## Edge Cases to Handle

### Empty Arrays
- **Issue:** Should empty overlay array delete base array?
- **Solution:** Treat as explicit override (yes, it should replace)
- **Note:** Lodash has bug here - ignores empty arrays

### Null Elements in Arrays
- **Issue:** Null as value vs. null as deletion marker
- **Solutions:**
  - Use explicit delete directive (`$patch: delete`)
  - Validate schema to prevent nulls in arrays
  - Handle format-specific constraints (TOML has no null)

### Mixed Types in Arrays
- **Issue:** Array with integers, strings, objects
- **Solutions:**
  - Require homogeneous arrays (strict)
  - Use tagged unions for type safety
  - Coerce to string representation

### Missing Keys in Objects
- **Issue:** Some array items have id/name, others don't
- **Conservative Solution:** Fall back to replace (used in jin)
- **Aggressive Solutions:** Generate keys, separate keyed/unkeyed

## Rust Libraries for Merging

1. **merge crate** - Derive macro with strategy attributes
   ```toml
   [dependencies]
   merge = "0.1"
   ```

2. **serde-toml-merge** - Specific for TOML values
   ```toml
   [dependencies]
   serde-toml-merge = "0.3"
   ```

3. **deepmerge** - Flexible, policy-driven merging
4. **schematic** - Complete config system with validation
5. **config** - Hierarchical configuration management

## Best Practices

1. **Always specify strategy explicitly** - Don't rely on defaults
2. **Use keyed merge for object arrays** - Preserves fine-grained control
3. **Document merge behavior** - In schema or comments
4. **Handle format constraints** - TOML null limitation
5. **Test edge cases** - Empty arrays, missing keys, duplicates
6. **Validate after merge** - Catch errors early
7. **Preserve order deterministically** - For reproducibility
8. **Use null for deletion** - Explicit, unambiguous removal

## Related Resources

- [Full Research Document](./ARRAY_MERGE_STRATEGIES.md) - 1360 lines, comprehensive coverage
- [jin Project Deep Merge Implementation](/home/dustin/projects/jin/src/merge/deep.rs)
- [jin MergeValue Type](/home/dustin/projects/jin/src/merge/value.rs)

## Comparison Table

| Aspect | Kubernetes | webpack-merge | Helm | Docker | ESLint |
|--------|-----------|---------------|------|--------|--------|
| **Object Merge** | Deep | Deep | Deep | Deep | Merge (special for rules) |
| **Array Default** | Replace | Append | Replace | Append* | Replace |
| **Keyed Arrays** | Yes (patchMergeKey) | No | No | No | No |
| **Delete Support** | Yes ($patch) | No | null | null | No |
| **Ordering Control** | Yes ($setElementOrder) | No | No | No | No |
| **Extend/Inherit** | No | No | No | No | Yes (extends) |

## Next Steps

For jin project:
1. Consider documenting merge strategy for each config section
2. Add per-field strategy annotations if schema is formalized
3. Handle edge cases in tests (empty arrays, missing keys)
4. Consider supporting explicit deletion markers like Kubernetes
5. Add ordering control if needed (`$setElementOrder` style)
