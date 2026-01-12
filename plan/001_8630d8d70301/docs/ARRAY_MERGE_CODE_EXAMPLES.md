# Array Merge Strategies - Code Examples

This document contains practical, copy-paste-ready code examples for implementing various array merge strategies.

## Table of Contents

1. [JavaScript Examples](#javascript-examples)
2. [Rust Examples](#rust-examples)
3. [YAML Configuration Examples](#yaml-configuration-examples)
4. [JSON Schema Examples](#json-schema-examples)
5. [Common Patterns](#common-patterns)

---

## JavaScript Examples

### Example 1: Replace Strategy (Simple)

```javascript
const merge = (base, overlay) => {
  return overlay;
};

// Usage
const base = { items: [1, 2, 3] };
const overlay = { items: [4, 5] };
const result = merge(base, overlay);
// Result: { items: [4, 5] }
```

### Example 2: Append Strategy

```javascript
const mergeAppend = (base, overlay) => {
  if (Array.isArray(base) && Array.isArray(overlay)) {
    return [...base, ...overlay];
  }
  if (typeof base === 'object' && typeof overlay === 'object') {
    return { ...base, ...overlay };
  }
  return overlay;
};

// Usage
const base = { items: [1, 2, 3] };
const overlay = { items: [4, 5] };
const result = { items: mergeAppend(base.items, overlay.items) };
// Result: { items: [1, 2, 3, 4, 5] }
```

### Example 3: Union Strategy (Deduplicate)

```javascript
const mergeUnion = (base, overlay) => {
  if (Array.isArray(base) && Array.isArray(overlay)) {
    return [...new Set([...base, ...overlay])];
  }
  return overlay;
};

// Usage
const base = { tags: ['a', 'b', 'c'] };
const overlay = { tags: ['b', 'c', 'd'] };
const result = { tags: mergeUnion(base.tags, overlay.tags) };
// Result: { tags: ['a', 'b', 'c', 'd'] }
```

### Example 4: Deep Merge with Default Strategy

```javascript
const deepMerge = (base, overlay, strategies = {}) => {
  // Null in overlay deletes the key
  if (overlay === null) {
    return null;
  }

  // Both objects - recursive merge
  if (
    typeof base === 'object' &&
    !Array.isArray(base) &&
    typeof overlay === 'object' &&
    !Array.isArray(overlay)
  ) {
    const result = { ...base };
    for (const key in overlay) {
      const strategy = strategies[key] || 'append';
      if (strategy === 'append' && Array.isArray(base[key]) && Array.isArray(overlay[key])) {
        result[key] = [...base[key], ...overlay[key]];
      } else {
        result[key] = deepMerge(base[key], overlay[key], strategies);
      }
    }
    return result;
  }

  // Default: overlay wins
  return overlay;
};

// Usage
const base = {
  name: 'app',
  config: {
    port: 8080,
    plugins: ['a', 'b']
  }
};

const overlay = {
  config: {
    plugins: ['c'],
    debug: true
  }
};

const result = deepMerge(base, overlay, {
  'config.plugins': 'append'
});
// Result: {
//   name: 'app',
//   config: {
//     port: 8080,
//     plugins: ['a', 'b', 'c'],
//     debug: true
//   }
// }
```

### Example 5: Keyed Array Merge (By Object ID)

```javascript
const mergeKeyedArrays = (baseArray, overlayArray, keyField = 'id') => {
  if (!Array.isArray(baseArray) || !Array.isArray(overlayArray)) {
    return overlayArray;
  }

  // Check if all items are objects with key field
  const allHaveKey = (arr) =>
    arr.every((item) => typeof item === 'object' && item !== null && keyField in item);

  if (!allHaveKey(baseArray) || !allHaveKey(overlayArray)) {
    // Can't do keyed merge - replace
    return overlayArray;
  }

  // Create map from base items
  const result = new Map();
  for (const item of baseArray) {
    result.set(item[keyField], item);
  }

  // Merge overlay items
  for (const overlayItem of overlayArray) {
    const key = overlayItem[keyField];
    if (result.has(key)) {
      // Merge with existing
      const baseItem = result.get(key);
      result.set(key, { ...baseItem, ...overlayItem });
    } else {
      // Add new item
      result.set(key, overlayItem);
    }
  }

  // Convert back to array (preserves base order, appends new)
  return Array.from(result.values());
};

// Usage
const base = [
  { id: 'web', port: 8080, replicas: 3 },
  { id: 'api', port: 3000, replicas: 2 }
];

const overlay = [
  { id: 'web', replicas: 5 },
  { id: 'cache', port: 6379, replicas: 1 }
];

const result = mergeKeyedArrays(base, overlay, 'id');
// Result: [
//   { id: 'web', port: 8080, replicas: 5 },
//   { id: 'api', port: 3000, replicas: 2 },
//   { id: 'cache', port: 6379, replicas: 1 }
// ]
```

### Example 6: webpack-merge Style

```javascript
const webpack = require('webpack');
const { mergeWithCustomize, customizeArray, unique } = require('webpack-merge');

const baseConfig = {
  entry: ['./main.js'],
  plugins: [new webpack.HotModuleReplacementPlugin()],
  module: {
    rules: [
      { test: /\.js$/, loader: 'babel-loader' }
    ]
  }
};

const envConfig = {
  entry: ['./polyfill.js'],
  plugins: [new webpack.DefinePlugin({ ENV: 'dev' })],
  module: {
    rules: [
      { test: /\.css$/, loader: 'style-loader!css-loader' }
    ]
  }
};

// Strategy 1: Custom per-field strategies
const config1 = mergeWithCustomize({
  customizeArray: customizeArray({
    'entry': 'prepend',  // Put polyfill first
    'module.rules': 'append'  // Add to rules
  })
})(baseConfig, envConfig);

// Strategy 2: Remove duplicate plugins
const config2 = mergeWithCustomize({
  customizeArray: unique(
    'plugins',
    ['HotModuleReplacementPlugin'],
    (plugin) => plugin.constructor?.name
  )
})(baseConfig, envConfig);

// Strategy 3: Custom function
const config3 = mergeWithCustomize({
  customizeArray(a, b, key) {
    if (key === 'plugins') {
      // Deduplicate plugins by name
      const seen = new Set();
      return [...a, ...b].filter((plugin) => {
        const name = plugin.constructor?.name;
        if (seen.has(name)) return false;
        seen.add(name);
        return true;
      });
    }
    return undefined;  // Use default append
  }
})(baseConfig, envConfig);
```

---

## Rust Examples

### Example 1: Simple Replace Strategy

```rust
fn merge_replace<T: Clone>(base: T, overlay: T) -> T {
    overlay
}

// Usage
let base = vec![1, 2, 3];
let overlay = vec![4, 5];
let result = merge_replace(base, overlay);
// Result: [4, 5]
```

### Example 2: Append Strategy

```rust
fn merge_append<T: Clone>(mut base: Vec<T>, overlay: Vec<T>) -> Vec<T> {
    base.extend(overlay);
    base
}

// Usage
let base = vec![1, 2, 3];
let overlay = vec![4, 5];
let result = merge_append(base, overlay);
// Result: [1, 2, 3, 4, 5]
```

### Example 3: Union Strategy (Deduplication)

```rust
use std::collections::HashSet;

fn merge_union<T: Clone + Eq + std::hash::Hash>(
    base: Vec<T>,
    overlay: Vec<T>,
) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();

    for item in base.into_iter().chain(overlay.into_iter()) {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    result
}

// Usage
let base = vec!["a", "b", "c"];
let overlay = vec!["b", "c", "d"];
let result = merge_union(base, overlay);
// Result: ["a", "b", "c", "d"]
```

### Example 4: Keyed Merge (By ID)

```rust
use std::collections::HashMap;

#[derive(Clone, Debug)]
struct Service {
    id: String,
    port: u16,
    replicas: u32,
}

fn merge_keyed_by_id(
    base: Vec<Service>,
    overlay: Vec<Service>,
) -> Vec<Service> {
    // Check if all items have non-empty IDs
    if base.iter().all(|s| !s.id.is_empty()) &&
       overlay.iter().all(|s| !s.id.is_empty()) {
        // Can do keyed merge
        let mut result: HashMap<String, Service> = HashMap::new();

        // Add base items
        for item in base {
            result.insert(item.id.clone(), item);
        }

        // Merge overlay items
        for item in overlay {
            if let Some(mut base_item) = result.remove(&item.id) {
                // Merge: overlay values override base
                base_item.port = item.port;
                base_item.replicas = item.replicas;
                result.insert(item.id.clone(), base_item);
            } else {
                // New item
                result.insert(item.id.clone(), item);
            }
        }

        // Convert back to vec (order may change)
        result.into_values().collect()
    } else {
        // Can't do keyed merge - replace
        overlay
    }
}

// Usage
let base = vec![
    Service { id: "web".into(), port: 8080, replicas: 3 },
    Service { id: "api".into(), port: 3000, replicas: 2 },
];

let overlay = vec![
    Service { id: "web".into(), port: 8080, replicas: 5 },
    Service { id: "cache".into(), port: 6379, replicas: 1 },
];

let result = merge_keyed_by_id(base, overlay);
// Result: web with replicas=5, api unchanged, cache added
```

### Example 5: Using merge Crate with Derive

```rust
use merge::Merge;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Merge, Debug, Default, Serialize, Deserialize)]
struct AppConfig {
    #[merge(skip)]
    name: String,

    // Append arrays
    #[merge(strategy = merge::vec::append)]
    plugins: Vec<String>,

    // Union arrays (deduplicate)
    #[merge(strategy = merge::vec::unique)]
    tags: Vec<String>,

    // Nested config
    database: DatabaseConfig,

    // Deep merge for maps
    #[merge(strategy = merge::hashmap::deep_merge)]
    environment: HashMap<String, String>,
}

#[derive(Merge, Debug, Default, Serialize, Deserialize)]
struct DatabaseConfig {
    #[merge(skip)]
    name: String,

    host: String,

    #[merge(strategy = merge::option::overwrite_some)]
    port: Option<u16>,
}

// Usage
fn main() {
    let mut base = AppConfig {
        name: "app".into(),
        plugins: vec!["auth".into(), "cache".into()],
        tags: vec!["v1".into(), "stable".into()],
        database: DatabaseConfig {
            name: "main".into(),
            host: "localhost".into(),
            port: Some(5432),
        },
        environment: [("DEBUG".into(), "false".into())]
            .iter()
            .cloned()
            .collect(),
    };

    let overlay = AppConfig {
        plugins: vec!["monitoring".into()],
        tags: vec!["stable".into(), "prod".into()],
        database: DatabaseConfig {
            host: "db.example.com".into(),
            ..Default::default()
        },
        environment: [("DEBUG".into(), "true".into())]
            .iter()
            .cloned()
            .collect(),
        ..Default::default()
    };

    base.merge(overlay);
    // base.plugins: ["auth", "cache", "monitoring"]
    // base.tags: ["v1", "stable", "prod"]
    // base.database.host: "db.example.com"
    // base.environment["DEBUG"]: "true"
}
```

### Example 6: Generic Deep Merge

```rust
use serde_json::{json, Value};

fn deep_merge(base: Value, overlay: Value) -> Value {
    match (&base, &overlay) {
        // Null in overlay deletes
        (_, Value::Null) => Value::Null,

        // Both objects - recursive merge
        (Value::Object(base_obj), Value::Object(overlay_obj)) => {
            let mut result = base_obj.clone();
            for (key, overlay_val) in overlay_obj {
                if overlay_val.is_null() {
                    result.remove(key);
                } else if let Some(base_val) = result.get(key) {
                    result[key] = deep_merge(base_val.clone(), overlay_val.clone());
                } else {
                    result[key] = overlay_val.clone();
                }
            }
            Value::Object(result)
        }

        // Both arrays - replace (configurable)
        (Value::Array(_), Value::Array(_)) => overlay,

        // Default: overlay wins
        _ => overlay,
    }
}

// Usage
fn main() {
    let base = json!({
        "name": "app",
        "config": {
            "port": 8080,
            "plugins": ["a", "b"]
        }
    });

    let overlay = json!({
        "config": {
            "plugins": ["c"],
            "debug": true
        }
    });

    let result = deep_merge(base, overlay);
    // Result: {
    //   "name": "app",
    //   "config": {
    //     "port": 8080,
    //     "plugins": ["c"],  // replaced, not merged
    //     "debug": true
    //   }
    // }
}
```

### Example 7: jin-style Keyed Merge (from actual project)

```rust
use indexmap::IndexMap;

#[derive(Clone, Debug)]
enum MergeValue {
    Null,
    Integer(i64),
    String(String),
    Array(Vec<MergeValue>),
    Object(IndexMap<String, MergeValue>),
}

fn deep_merge(base: MergeValue, overlay: MergeValue) -> MergeValue {
    match (base, overlay) {
        (_, MergeValue::Null) => MergeValue::Null,

        (MergeValue::Object(mut base_obj), MergeValue::Object(overlay_obj)) => {
            for (key, overlay_val) in overlay_obj {
                if overlay_val.is_null() {
                    base_obj.shift_remove(&key);
                } else if let Some(base_val) = base_obj.shift_remove(&key) {
                    let merged = deep_merge(base_val, overlay_val);
                    if !merged.is_null() {
                        base_obj.insert(key, merged);
                    }
                } else {
                    base_obj.insert(key, overlay_val);
                }
            }
            MergeValue::Object(base_obj)
        }

        (MergeValue::Array(base_arr), MergeValue::Array(overlay_arr)) => {
            // Try keyed merge first
            if let (Some(base_map), Some(overlay_map)) =
                (extract_array_keys(&base_arr), extract_array_keys(&overlay_arr))
            {
                let mut result: IndexMap<String, MergeValue> = IndexMap::new();
                for (key, val) in base_map {
                    result.insert(key, val);
                }
                for (key, overlay_val) in overlay_map {
                    if let Some(base_val) = result.shift_remove(&key) {
                        let merged = deep_merge(base_val, overlay_val);
                        result.insert(key, merged);
                    } else {
                        result.insert(key, overlay_val);
                    }
                }
                MergeValue::Array(result.into_values().collect())
            } else {
                // Can't do keyed merge - replace
                MergeValue::Array(overlay_arr)
            }
        }

        (_, overlay) => overlay,
    }
}

fn extract_array_keys(arr: &[MergeValue]) -> Option<IndexMap<String, MergeValue>> {
    let mut result = IndexMap::new();

    for item in arr {
        if let MergeValue::Object(obj) = item {
            let key = obj
                .get("id")
                .and_then(|v| v.as_str())
                .or_else(|| obj.get("name").and_then(|v| v.as_str()));

            if let Some(k) = key {
                result.insert(k.to_string(), item.clone());
            } else {
                return None;  // Missing key
            }
        } else {
            return None;  // Non-object in array
        }
    }

    Some(result)
}
```

---

## YAML Configuration Examples

### Example 1: Simple Append (Default in Many Tools)

```yaml
# base.yaml
services:
  - name: web
    port: 8080

# overlay.yaml
services:
  - name: api
    port: 3000

# Result (append)
services:
  - name: web
    port: 8080
  - name: api
    port: 3000
```

### Example 2: Keyed Merge (Kubernetes Style)

```yaml
# base.yaml
spec:
  containers:
    - name: main
      image: app:1.0
      resources:
        memory: "256Mi"

# patch.yaml (strategic merge patch)
spec:
  containers:
    - name: main
      resources:
        memory: "512Mi"  # Only this is merged

# Result
spec:
  containers:
    - name: main
      image: app:1.0
      resources:
        memory: "512Mi"
```

### Example 3: Null Deletion

```yaml
# base.yaml
config:
  debug: true
  cache_size: 1000
  deprecated_field: old_value

# overlay.yaml
config:
  debug: false
  deprecated_field: null  # Explicit deletion

# Result
config:
  debug: false
  cache_size: 1000
  # deprecated_field removed
```

### Example 4: Map-Based Merge (Alternative to Arrays)

```yaml
# base.yaml
plugins:
  auth:
    enabled: true
    version: "1.0"
  cache:
    enabled: true
    ttl: 3600

# overlay.yaml
plugins:
  auth:
    version: "2.0"  # Merged
  monitoring:
    enabled: true

# Result (deep merge friendly)
plugins:
  auth:
    enabled: true
    version: "2.0"
  cache:
    enabled: true
    ttl: 3600
  monitoring:
    enabled: true
```

### Example 5: Separate Base and Override Lists

```yaml
# values-base.yaml
basePlugins:
  - name: auth
    version: "1.0"
  - name: cache
    version: "2.0"

# values-prod.yaml
prodPlugins:
  - name: monitoring
    version: "1.0"

# template.yaml
plugins: {{ concat .Values.basePlugins .Values.prodPlugins }}

# Result
plugins:
  - name: auth
    version: "1.0"
  - name: cache
    version: "2.0"
  - name: monitoring
    version: "1.0"
```

---

## JSON Schema Examples

### Example 1: Schema with Merge Keys

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "services": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "port": { "type": "integer" },
          "replicas": { "type": "integer" }
        },
        "required": ["name"]
      },
      "x-merge-strategy": "keyed",
      "x-merge-key": "name"
    }
  }
}
```

### Example 2: Schema with Per-Field Strategies

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "plugins": {
      "type": "array",
      "items": { "type": "string" },
      "x-merge-strategy": "append"
    },
    "tags": {
      "type": "array",
      "items": { "type": "string" },
      "x-merge-strategy": "union"
    },
    "overrides": {
      "type": "array",
      "items": { "type": "object" },
      "x-merge-strategy": "replace"
    },
    "config": {
      "type": "object",
      "x-merge-strategy": "deep"
    }
  }
}
```

### Example 3: Schema with Custom Merge Key

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "servers": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "hostname": { "type": "string" },
          "ip": { "type": "string" },
          "port": { "type": "integer" }
        },
        "required": ["hostname"]
      },
      "x-merge-strategy": "keyed",
      "x-merge-key": "hostname"
    }
  }
}
```

---

## Common Patterns

### Pattern 1: Environment-Specific Configuration

```javascript
// config.js
const baseConfig = require('./config.base.json');
const envConfig = require(`./config.${process.env.NODE_ENV}.json`);

module.exports = deepMerge(baseConfig, envConfig);
```

### Pattern 2: Layer Merging (Multiple Files)

```rust
fn merge_layers(configs: Vec<Config>) -> Config {
    configs.into_iter().reduce(|acc, current| {
        deep_merge(acc, current)
    }).unwrap_or_default()
}

// Usage
let layers = vec![base_config, env_config, local_config];
let final_config = merge_layers(layers);
```

### Pattern 3: Validated Merge

```rust
fn merge_and_validate<T: Validate>(
    base: T,
    overlay: T,
) -> Result<T, ValidationError> {
    let merged = deep_merge(base, overlay);
    merged.validate()?;
    Ok(merged)
}
```

### Pattern 4: Audit Trail (What Changed)

```javascript
function mergeWithDiff(base, overlay) {
  const diff = { added: {}, modified: {}, removed: {} };
  const result = { ...base };

  for (const key in overlay) {
    if (!(key in base)) {
      diff.added[key] = overlay[key];
    } else if (base[key] !== overlay[key]) {
      diff.modified[key] = { from: base[key], to: overlay[key] };
    }
    result[key] = overlay[key];
  }

  for (const key in base) {
    if (!(key in overlay)) {
      diff.removed[key] = base[key];
    }
  }

  return { result, diff };
}
```

### Pattern 5: Conditional Merge

```javascript
function conditionalMerge(base, overlay, predicate) {
  if (predicate(base, overlay)) {
    return deepMerge(base, overlay);
  } else {
    return base;  // Skip merge
  }
}

// Usage
const result = conditionalMerge(base, overlay, (b, o) => {
  // Only merge if overlay is "complete"
  return Object.keys(o).length > Object.keys(b).length / 2;
});
```

---

## Testing Examples

### JavaScript Tests

```javascript
describe('mergeKeyedArrays', () => {
  test('merges items by id', () => {
    const base = [{ id: 'a', v: 1 }];
    const overlay = [{ id: 'a', v: 2 }];
    expect(mergeKeyedArrays(base, overlay, 'id')).toEqual([{ id: 'a', v: 2 }]);
  });

  test('preserves base items not in overlay', () => {
    const base = [{ id: 'a', v: 1 }, { id: 'b', v: 2 }];
    const overlay = [{ id: 'a', v: 3 }];
    const result = mergeKeyedArrays(base, overlay, 'id');
    expect(result).toContainEqual({ id: 'b', v: 2 });
  });

  test('adds new items from overlay', () => {
    const base = [{ id: 'a', v: 1 }];
    const overlay = [{ id: 'b', v: 2 }];
    const result = mergeKeyedArrays(base, overlay, 'id');
    expect(result).toContainEqual({ id: 'b', v: 2 });
  });

  test('falls back to replace if missing keys', () => {
    const base = [{ id: 'a', v: 1 }, { v: 2 }];  // Second item lacks id
    const overlay = [{ id: 'c', v: 3 }];
    expect(mergeKeyedArrays(base, overlay, 'id')).toEqual(overlay);
  });

  test('handles empty arrays', () => {
    expect(mergeKeyedArrays([], [], 'id')).toEqual([]);
    expect(mergeKeyedArrays([{ id: 'a', v: 1 }], [], 'id')).toEqual([]);
  });
});
```

### Rust Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_keyed_by_id() {
        let base = vec![
            Service { id: "web".into(), port: 8080, replicas: 3 },
        ];
        let overlay = vec![
            Service { id: "web".into(), port: 8080, replicas: 5 },
        ];
        let result = merge_keyed_by_id(base, overlay);
        assert_eq!(result[0].replicas, 5);
    }

    #[test]
    fn test_merge_keyed_preserves_unmodified() {
        let base = vec![
            Service { id: "web".into(), port: 8080, replicas: 3 },
            Service { id: "api".into(), port: 3000, replicas: 2 },
        ];
        let overlay = vec![
            Service { id: "web".into(), port: 8080, replicas: 5 },
        ];
        let result = merge_keyed_by_id(base, overlay);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_merge_keyed_adds_new() {
        let base = vec![
            Service { id: "web".into(), port: 8080, replicas: 3 },
        ];
        let overlay = vec![
            Service { id: "cache".into(), port: 6379, replicas: 1 },
        ];
        let result = merge_keyed_by_id(base, overlay);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_merge_falls_back_if_missing_id() {
        let base = vec![
            Service { id: "web".into(), port: 8080, replicas: 3 },
            Service { id: String::new(), port: 9000, replicas: 1 },  // No ID
        ];
        let overlay = vec![
            Service { id: "api".into(), port: 3000, replicas: 2 },
        ];
        let result = merge_keyed_by_id(base, overlay);
        // Should replace entire array
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "api");
    }
}
```

---

## Summary

These examples cover the most common array merge patterns:

- **Replace**: Simplest, overlay entirely replaces base
- **Append**: Additive, both arrays combined
- **Union**: Deduplication, unique elements only
- **Keyed**: Complex object arrays, merge by identifier
- **Deep**: Recursive merging for nested structures

Choose based on your use case and configuration complexity.
