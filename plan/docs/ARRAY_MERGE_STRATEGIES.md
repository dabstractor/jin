# Array Merge Strategies in Configuration File Merging

A comprehensive research document on array merge strategies used in real-world configuration management systems.

## Table of Contents

1. [Common Array Merge Strategies](#common-array-merge-strategies)
2. [Real-World Tool Implementations](#real-world-tool-implementations)
3. [Keyed Array Merging Patterns](#keyed-array-merging-patterns)
4. [Configuration Patterns](#configuration-patterns)
5. [Edge Cases](#edge-cases)
6. [Rust Implementation Examples](#rust-implementation-examples)
7. [References](#references)

---

## Common Array Merge Strategies

### 1. Replace (Overlay Replaces Base)

The simplest strategy where the entire array from the higher-precedence configuration completely replaces the base array.

**Characteristics:**
- Base array is discarded entirely
- Overlay array becomes the result
- No combination of elements
- Useful for complete configuration overrides

**Example:**
```yaml
# base.yaml
items: [a, b, c]

# overlay.yaml
items: [x, y]

# Result (replace strategy)
items: [x, y]
```

**Tools using this:** Docker Compose (default for many fields), Kubernetes (default for untagged lists)

---

### 2. Append (Overlay Array Appended to Base)

Elements from the overlay array are added to the end of the base array.

**Characteristics:**
- Base array elements preserved
- Overlay elements added at the end
- Duplicates preserved (may need deduplication)
- Order: base elements first, then overlay
- Memory: O(n+m) where n and m are array lengths

**Example:**
```yaml
# base.yaml
items: [a, b, c]

# overlay.yaml
items: [d, e]

# Result (append strategy)
items: [a, b, c, d, e]
```

**Tools using this:** webpack-merge (default), Helm (requires workaround)

---

### 3. Prepend (Overlay Array Prepended to Base)

Elements from the overlay array are inserted at the beginning of the base array.

**Characteristics:**
- Overlay elements placed first
- Base array elements follow
- Used when overlay has higher precedence
- Useful for rules/plugins that should execute first

**Example:**
```yaml
# base.yaml
rules: [rule-b, rule-c]

# overlay.yaml
rules: [rule-a]

# Result (prepend strategy)
rules: [rule-a, rule-b, rule-c]
```

**Tools using this:** webpack-merge (via customizeArray), ESLint (extends chain)

---

### 4. Union (Deduplicated Merge)

Combines arrays while removing duplicate elements, preserving the first occurrence.

**Characteristics:**
- All unique elements included
- Duplicates removed (first occurrence kept)
- Order may vary depending on implementation
- Hash-map based implementations: O(n+m) time
- Requires equality comparison for elements

**Example:**
```yaml
# base.yaml
tags: [a, b, c]

# overlay.yaml
tags: [b, c, d]

# Result (union strategy)
tags: [a, b, c, d]
```

**Tools using this:** webpack-merge (via unique strategy)

---

### 5. Keyed Merge (By ID/Name Field)

For arrays of objects, merge is performed by a key field ("id" or "name"), allowing element-level merge operations.

**Characteristics:**
- Each array item has a unique identifier
- Base and overlay items matched by key
- Matching items merged recursively
- Non-matching items added to result
- Preserves structure and metadata of objects

**Example:**
```yaml
# base.yaml
services:
  - name: web
    port: 8080
    replicas: 3
  - name: api
    port: 3000
    replicas: 2

# overlay.yaml
services:
  - name: web
    replicas: 5
  - name: cache
    port: 6379
    replicas: 1

# Result (keyed merge by name)
services:
  - name: web
    port: 8080      # from base
    replicas: 5     # from overlay (merged)
  - name: api
    port: 3000
    replicas: 2
  - name: cache
    port: 6379
    replicas: 1
```

**Tools using this:** Kubernetes (strategic merge patch), jin configuration system

---

## Real-World Tool Implementations

### Kubernetes Strategic Merge Patch

**Overview:**
Kubernetes uses strategic merge patch for resource updates, solving the problem that standard JSON merge patch always replaces arrays.

**Key Features:**

1. **Patch Merge Keys**
   - Defined via `x-kubernetes-patch-merge-key` OpenAPI extension
   - Specifies which field uniquely identifies array items
   - Common merge keys: `name`, `id`, `ip`

   ```go
   type Container struct {
       Name  string `json:"name" patchMergeKey:"name" patchStrategy:"merge"`
       Env   []EnvVar `json:"env" patchMergeKey:"name" patchStrategy:"merge"`
   }
   ```

2. **Patch Strategies**
   - `replace`: Entire array is replaced (default)
   - `merge`: Array merged by patch merge key as unordered set

3. **Special Directives**

   **Replace Directive** - Override merge with replacement:
   ```json
   {
     "spec": {
       "containers": [
         {
           "$patch": "replace",
           "name": "nginx",
           "image": "nginx:latest"
         }
       ]
     }
   }
   ```

   **Delete Directive** - Remove array elements:
   ```json
   {
     "spec": {
       "containers": [
         {
           "$patch": "delete",
           "name": "old-container"
         }
       ]
     }
   }
   ```

   **setElementOrder** - Control ordering:
   ```json
   {
     "$setElementOrder/containers": [
       {"name": "init"},
       {"name": "main"},
       {"name": "sidecar"}
     ]
   }
   ```

4. **Backward Compatibility**
   - Patches valid in previous versions remain valid
   - New directives don't break old patches
   - Non-strategic patches still work on untagged fields

**Reference:** [Kubernetes Strategic Merge Patch](https://github.com/kubernetes/community/blob/master/contributors/devel/sig-api-machinery/strategic-merge-patch.md)

---

### webpack-merge customizeArray

**Overview:**
webpack-merge provides flexible array merging strategies for webpack configuration composition.

**Key Features:**

1. **Built-in Strategies**
   ```javascript
   const { mergeWithCustomize, customizeArray } = require('webpack-merge');

   // Append (default)
   const config = mergeWithCustomize({
     customizeArray: customizeArray({
       'entry': 'append'  // default behavior
     })
   })(base, overlay);

   // Prepend
   const config = mergeWithCustomize({
     customizeArray: customizeArray({
       'entry.*': 'prepend'  // wildcard pattern
     })
   })(base, overlay);

   // Replace
   const config = mergeWithCustomize({
     customizeArray: customizeArray({
       'module.rules': 'replace'
     })
   })(base, overlay);
   ```

2. **Custom Function Pattern**
   ```javascript
   const { mergeWithCustomize } = require('webpack-merge');

   const config = mergeWithCustomize({
     customizeArray(a, b, key) {
       if (key === 'extensions') {
         // Union: deduplicate array
         return _.uniq([...a, ...b]);
       }
       // Fall back to default (append)
       return undefined;
     }
   })(object1, object2);
   ```

3. **Unique Strategy** - Remove duplicate plugins
   ```javascript
   const { mergeWithCustomize, unique } = require("webpack-merge");

   const config = mergeWithCustomize({
     customizeArray: unique(
       "plugins",
       ["HotModuleReplacementPlugin"],
       (plugin) => plugin.constructor?.name
     )
   })({ plugins: [...] }, { plugins: [...] });
   ```

4. **Wildcard Pattern Matching**
   - `'entry.*'` matches any field under entry
   - Enables fine-grained strategy control
   - Reduces repetitive configuration

**Reference:** [webpack-merge GitHub](https://github.com/survivejs/webpack-merge)

---

### Helm Chart Value Merging

**Overview:**
Helm merges multiple values files with a deep merge strategy for objects but complete replacement for arrays.

**Key Features:**

1. **Default Behavior**
   - Objects merged by key (later keys override)
   - Arrays completely replaced by later values
   - Null values delete keys from merge

   ```bash
   helm install myapp ./chart \
     -f base-values.yaml \
     -f env-values.yaml \
     -f secrets.yaml
   # Later files have higher precedence for array replacement
   ```

2. **Known Limitation**
   - Arrays from multiple files don't merge
   - Causes unexpected loss of base array elements
   - Common pain point for users

3. **Workaround Patterns**

   **Separate Keys Approach:**
   ```yaml
   # values-base.yaml
   basePlugins:
     - name: auth
     - name: cache

   # values-prod.yaml
   prodPlugins:
     - name: monitoring

   # template.yaml
   plugins: {{ concat .Values.basePlugins .Values.prodPlugins | toJson }}
   ```

   **Map-based Approach** (merge-friendly):
   ```yaml
   # Use maps instead of arrays for merge-friendly configuration
   pluginsMap:
     auth:
       enabled: true
       version: 1.0
     cache:
       enabled: true
       version: 2.0
   ```

4. **mustMergeOverwrite Function**
   ```yaml
   merged: {{ mustMergeOverwrite $base $override | toJson }}
   ```

**Reference:** [Helm Values Files Documentation](https://helm.sh/docs/chart_template_guide/values_files/)

---

### Docker Compose Override Merging

**Overview:**
Docker Compose merges multiple compose files with append for arrays by default but replacement for certain fields.

**Key Features:**

1. **Default Append Behavior**
   - ports: appended
   - expose: appended
   - external_links: appended
   - dns, dns_search: appended

   ```yaml
   # docker-compose.yaml
   services:
     web:
       ports:
         - "8080:8080"

   # docker-compose.override.yaml
   services:
     web:
       ports:
         - "3000:3000"

   # Result: ports: [8080:8080, 3000:3000]
   ```

2. **Replace Behavior** (for specific fields)
   - command
   - entrypoint
   - healthcheck.test

   ```yaml
   # docker-compose.yaml
   services:
     web:
       command: ["npm", "start"]

   # docker-compose.override.yaml
   services:
     web:
       command: ["npm", "dev"]

   # Result: command: ["npm", "dev"] (replaced, not merged)
   ```

3. **Force Override with Directives**
   ```yaml
   services:
     web:
       env_file: !override
         - .env.prod
   ```

4. **Null Deletion**
   ```yaml
   # Remove build config entirely
   services:
     web:
       build: null  # Deletes the build key
   ```

5. **Configuration Verification**
   ```bash
   docker compose config  # View merged configuration
   ```

**Known Issues:**
- Inconsistent array merging across different fields
- Some arrays merge, others replace
- Different behavior between v1 and v2

**Reference:** [Docker Compose Merge Documentation](https://docs.docker.com/compose/how-tos/multiple-compose-files/merge/)

---

### ESLint Config Merging

**Overview:**
ESLint uses configuration extension and override blocks with special rule merging behavior.

**Key Features:**

1. **Extends Array**
   ```javascript
   module.exports = {
     extends: [
       'eslint:recommended',
       'plugin:react/recommended',
       './custom-config.js'
     ],
     rules: {
       'no-console': 'warn'
     }
   };
   ```

2. **Rule Merging Behavior**
   - Last config wins for severity level
   - Options from earlier configs preserved
   - Only rules section has special merging

   ```javascript
   // extends[0]
   '@typescript-eslint/no-empty-function': [
     'error',
     { allow: ['arrowFunctions', 'functions', 'methods'] }
   ]

   // extends[1]
   '@typescript-eslint/no-empty-function': 'warn'

   // Result
   '@typescript-eslint/no-empty-function': [
     'warn',  // severity from extends[1]
     { allow: ['arrowFunctions', 'functions', 'methods'] }  // options preserved
   ]
   ```

3. **Other Properties** (non-rules)
   - Use Object.assign() semantics
   - Last value wins (replace)
   - No special merging

4. **Override Blocks**
   - Higher precedence than regular config
   - Applied in order (last wins)
   - Support glob patterns

   ```javascript
   module.exports = {
     rules: {
       'no-console': 'error'
     },
     overrides: [
       {
         files: ['*.test.js'],
         rules: {
           'no-console': 'warn'  // Higher precedence
         }
       }
     ]
   };
   ```

**Reference:** [ESLint Configuration Files](https://eslint.org/docs/latest/use/configure/configuration-files)

---

## Keyed Array Merging Patterns

### Identifying Array Items

**Key Field Options:**
1. **ID Field** - Numeric or UUID identifier
   - Most reliable
   - Globally unique
   - Standard in many systems

2. **Name Field** - String identifier
   - Human-readable
   - Scope-relative uniqueness
   - Recommended fallback

3. **Custom Key Field**
   - User-defined identifier
   - Specified via schema/config
   - System-specific (e.g., hostname)

### Handling Missing Keys

**Scenario:** Array has some items with keys and some without

**Approaches:**

1. **Fall Back to Replace Strategy**
   - If any item lacks key, replace entire array
   - Conservative but safe
   - Example: jin implementation

   ```rust
   fn extract_array_keys(arr: &[MergeValue]) -> Option<IndexMap<String, MergeValue>> {
       for item in arr {
           // Must be object with id or name
           if let MergeValue::Object(obj) = item {
               let key = obj.get("id")
                   .and_then(|v| v.as_str())
                   .or_else(|| obj.get("name").and_then(|v| v.as_str()));

               if key.is_none() {
                   // Missing key - can't do keyed merge
                   return None;
               }
           } else {
               // Non-object in array
               return None;
           }
       }
       Some(result)
   }
   ```

2. **Generate Keys**
   - Auto-generate IDs for missing items
   - Use index-based keys
   - Risky for reproducibility

3. **Separate Keyed and Unkeyed**
   - Keyed items merged by key
   - Unkeyed items appended
   - Requires hybrid merge logic

### Handling Duplicate Keys

**Scenario:** Same key appears multiple times in base or overlay

**Approaches:**

1. **Last One Wins**
   - Most recent value kept
   - Previous duplicates discarded
   - Simple implementation

   ```rust
   for (key, item) in array_items {
       result.insert(key, item);  // Overwrites previous
   }
   ```

2. **First One Wins**
   - Initial value preserved
   - Later duplicates ignored
   - Defensive approach

   ```rust
   for (key, item) in array_items {
       result.entry(key).or_insert(item);
   }
   ```

3. **Error on Duplicate**
   - Strict validation
   - Forces unique constraints
   - Prevents subtle bugs

4. **Deep Merge Duplicates**
   - Recursively merge duplicate items
   - Preserves all information
   - Most complex

### Ordering After Merge

**Strategies:**

1. **Preserve Base Order, Append New**
   - Base items in original order
   - New items appended
   - Stable and predictable

2. **Preserve Overlay Order**
   - Follow overlay array order
   - Overlay has precedence
   - Used in Kubernetes

3. **Explicit Ordering Directive**
   - User specifies final order
   - `$setElementOrder` in Kubernetes
   - Maximum control

   ```json
   {
     "$setElementOrder/containers": [
       {"name": "init"},
       {"name": "main"},
       {"name": "sidecar"}
     ]
   }
   ```

4. **Deterministic Alphabetical**
   - Sort by key field
   - Reproducible across runs
   - Used in some configuration systems

### Deletion Markers for Array Elements

**Problem:** How to remove elements from base array in overlay?

**Solutions:**

1. **Null Marker (Implicit)**
   ```yaml
   # overlay.yaml
   items:
     - name: old-service
       # null or absence indicates deletion
   ```

2. **Delete Directive**
   - Kubernetes `$patch: delete`
   - Explicit deletion marker

   ```json
   {
     "containers": [
       {
         "$patch": "delete",
         "name": "old-container"
       }
     ]
   }
   ```

3. **Exclude List**
   - Separate array of items to remove
   - Non-destructive representation

   ```yaml
   services:
     add:
       - name: new-service
     remove:
       - name: old-service
   ```

4. **Filtering Function**
   - Custom predicate for filtering
   - Complex merge logic

   ```rust
   overlay_items
       .filter(|item| !is_deleted(item))
       .map(|item| merge_with_base(item))
   ```

---

## Configuration Patterns

### Per-Field Strategy Annotation

**Goal:** Specify merge strategy for individual fields in schema/configuration

**Pattern 1: Struct Attributes (Rust)**
```rust
#[derive(Merge)]
struct Config {
    #[merge(strategy = merge::vec::append)]
    plugins: Vec<Plugin>,

    #[merge(strategy = merge::vec::prepend)]
    rules: Vec<Rule>,

    #[merge(strategy = merge::bool::overwrite_false)]
    enabled: bool,
}
```

**Pattern 2: Schema Annotations (JSON Schema)**
```json
{
  "type": "object",
  "properties": {
    "plugins": {
      "type": "array",
      "mergeStrategy": "append"
    },
    "rules": {
      "type": "array",
      "mergeStrategy": "keyed",
      "mergeKey": "name"
    }
  }
}
```

**Pattern 3: External Configuration**
```yaml
# merge-config.yaml
mergeRules:
  keyed_array:
    - path: [database, servers]
      attribute: items
      key: hostname
    - path: [middleware]
      attribute: handlers
      key: name
  strategies:
    - path: [plugins]
      strategy: append
    - path: [overrides]
      strategy: replace
```

**Pattern 4: webpack-merge Style**
```javascript
mergeWithCustomize({
  customizeArray: customizeArray({
    'entry': 'prepend',
    'module.rules': 'append',
    'plugins': 'replace'
  })
})
```

### Global Default Strategy

**Purpose:** Define fallback behavior when field-specific strategy not specified

**Common Defaults:**

| Tool | Objects | Arrays | Scalars |
|------|---------|--------|---------|
| Kubernetes | Merge | Replace | Replace |
| Helm | Merge | Replace | Replace |
| Docker Compose | Merge | Append* | Replace |
| webpack-merge | Merge | Append | Replace |
| ESLint | Merge | Replace | Merge (rules only) |
| jin | Merge | Keyed if possible, else Replace | Replace |

**Implementation Pattern:**
```rust
fn merge_values(
    base: MergeValue,
    overlay: MergeValue,
    strategy: Option<MergeStrategy>
) -> Result<MergeValue> {
    let strategy = strategy.unwrap_or_default();

    match (&base, &overlay) {
        (MergeValue::Object(b), MergeValue::Object(o)) => {
            // Merge objects (always recursive)
            merge_objects(b, o)
        }
        (MergeValue::Array(b), MergeValue::Array(o)) => {
            // Apply specified array strategy
            match strategy {
                MergeStrategy::Append => [b, o].concat(),
                MergeStrategy::Keyed => merge_arrays_keyed(b, o),
                MergeStrategy::Replace => o.clone(),
            }
        }
        _ => Ok(overlay)  // Scalars: overlay wins
    }
}
```

### Schema-Driven Merge Rules

**Concept:** Use formal schema to define merge behavior for entire configuration

**Advantages:**
- Single source of truth
- Type-safe
- Can be validated
- Documentation included

**Example: JSON Schema with Merge Extensions**
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
          "port": { "type": "integer" }
        }
      },
      "x-merge-strategy": "keyed",
      "x-merge-key": "name"
    },
    "environment": {
      "type": "object",
      "x-merge-strategy": "deep"
    },
    "version": {
      "type": "string",
      "x-merge-strategy": "replace"
    }
  }
}
```

**Example: Rust Trait-Based Approach**
```rust
trait MergeConfig: Serialize + Deserialize<'static> {
    fn merge_strategy(&self) -> MergeStrategy;
    fn merge(&mut self, other: Self) -> Result<()>;
}

impl MergeConfig for AppConfig {
    fn merge_strategy(&self) -> MergeStrategy {
        MergeStrategy::Deep {
            array_strategy: ArrayMergeStrategy::KeyedByName,
        }
    }

    fn merge(&mut self, other: Self) -> Result<()> {
        // Custom merge logic
        Ok(())
    }
}
```

---

## Edge Cases

### Empty Arrays

**Issue:** How to handle empty arrays in merge?

**Behavior Variations:**

1. **Lodash Default (Problematic)**
   ```javascript
   _.merge({ x: [1,2,3] }, { x: [] })
   // Result: { x: [1,2,3] } - empty array IGNORED
   // Expected: { x: [] } - empty array should override
   ```

2. **Intended Behavior**
   - Empty array should replace base array
   - Represents explicit override to empty
   - Null vs. empty distinction important

3. **Implementation Consideration**
   ```rust
   match (base, overlay) {
       // Empty overlay explicitly replaces
       (_, MergeValue::Array(arr)) if arr.is_empty() => {
           Ok(MergeValue::Array(arr))
       }
       // Non-empty overlay merges normally
       (MergeValue::Array(b), MergeValue::Array(o)) => {
           merge_arrays(b, o)
       }
   }
   ```

### Null Elements in Arrays

**Issue:** How to handle null values within arrays?

**Scenarios:**

1. **Null to Delete**
   ```json
   {
     "items": [
       { "id": "a", "value": 1 },
       null,  // Does null mean delete?
       { "id": "b", "value": 2 }
     ]
   }
   ```

2. **Null as Value**
   - Some formats support null as valid value
   - YAML/JSON yes, TOML no
   - Ambiguity problem

3. **Solutions**
   - Explicit delete marker (`$patch: delete`)
   - Schema validation to prevent nulls in arrays
   - Format-specific handling
   - Pre-processing to clean nulls

4. **Format-Specific Constraints**
   ```rust
   // TOML doesn't support null
   fn to_toml_value(val: MergeValue) -> Result<toml::Value> {
       match val {
           MergeValue::Null => Err(JinError::Parse {
               format: "TOML".into(),
               message: "TOML does not support null values".into()
           }),
           // ... other cases
       }
   }
   ```

### Mixed Types in Arrays

**Issue:** Array containing different types of elements

**Example:**
```json
{
  "mixed": [1, "string", { "obj": true }, [1, 2, 3], null]
}
```

**Problems:**
- Type consistency
- Merge ambiguity
- Schema validation

**Handling:**

1. **Strict Type Checking**
   - Require homogeneous arrays
   - Fail on mixed types
   - Safest approach

2. **Permissive Merge**
   - Handle mixed types gracefully
   - Preserve type during merge
   - Validate after merge

3. **Type Coercion**
   - Convert to common type
   - String representation
   - Loss of type information

4. **Tagged Union Approach**
   ```rust
   #[derive(Serialize, Deserialize)]
   #[serde(untagged)]
   enum ArrayElement {
       Integer(i64),
       String(String),
       Object(IndexMap<String, Value>),
   }
   ```

---

## Rust Implementation Examples

### Example 1: Keyed Array Merge (from jin project)

The jin project implements keyed array merging in `/home/dustin/projects/jin/src/merge/deep.rs`:

```rust
/// Merge two arrays, attempting to merge by "id" or "name" keys if present
fn merge_arrays(
    base: Vec<MergeValue>,
    overlay: Vec<MergeValue>
) -> Result<Vec<MergeValue>> {
    // Check if arrays have keyed objects
    let base_keyed = extract_array_keys(&base);
    let overlay_keyed = extract_array_keys(&overlay);

    if let (Some(base_map), Some(overlay_map)) = (base_keyed, overlay_keyed) {
        // Both arrays are keyed - merge by key
        let mut result: IndexMap<String, MergeValue> = IndexMap::new();

        // Add all base items
        for (key, val) in base_map {
            result.insert(key, val);
        }

        // Merge or add overlay items
        for (key, overlay_val) in overlay_map {
            if let Some(base_val) = result.shift_remove(&key) {
                // Key exists in both - merge recursively
                let merged = deep_merge(base_val, overlay_val)?;
                result.insert(key, merged);
            } else {
                // New key only in overlay
                result.insert(key, overlay_val);
            }
        }

        // Preserve order by converting back to vector
        Ok(result.into_values().collect())
    } else {
        // Arrays don't have consistent keys - replace with overlay
        Ok(overlay)
    }
}

/// Extract keys from array items if they have "id" or "name" fields
fn extract_array_keys(arr: &[MergeValue]) -> Option<IndexMap<String, MergeValue>> {
    let mut result = IndexMap::new();

    for item in arr {
        if let MergeValue::Object(obj) = item {
            // Try "id" first, then "name"
            let key = obj
                .get("id")
                .and_then(|v| v.as_str())
                .or_else(|| obj.get("name").and_then(|v| v.as_str()));

            if let Some(k) = key {
                result.insert(k.to_string(), item.clone());
            } else {
                // Item without key, can't do keyed merge
                return None;
            }
        } else {
            // Non-object in array, can't do keyed merge
            return None;
        }
    }

    Some(result)
}
```

**Key Design Decisions:**
1. Use `IndexMap` to preserve insertion order
2. Try "id" field first, fall back to "name"
3. Return `Option` - if any item lacks key, return None and fall back to replace
4. Recursive `deep_merge` for matching keys
5. Maintain order from result map (preserves base order, appends new items)

---

### Example 2: Multi-Format MergeValue Type

From `/home/dustin/projects/jin/src/merge/value.rs`:

```rust
/// Represents a value that can be merged
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MergeValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<MergeValue>),
    Object(IndexMap<String, MergeValue>),
}

impl MergeValue {
    // Format detection and conversion
    pub fn from_json(s: &str) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(s)
            .map_err(|e| JinError::Parse {
                format: "JSON".to_string(),
                message: e.to_string(),
            })?;
        Ok(Self::from(value))
    }

    pub fn from_yaml(s: &str) -> Result<Self> { /* ... */ }
    pub fn from_toml(s: &str) -> Result<Self> { /* ... */ }
    pub fn from_ini(s: &str) -> Result<Self> { /* ... */ }

    // File auto-detection
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        match extension.as_deref() {
            Some("json") => Self::from_json(&content),
            Some("yaml") | Some("yml") => Self::from_yaml(&content),
            Some("toml") => Self::from_toml(&content),
            Some("ini") | Some("cfg") | Some("conf") => Self::from_ini(&content),
            Some(ext) => Err(JinError::Parse {
                format: ext.to_string(),
                message: format!("Unsupported file extension: .{}", ext),
            }),
            None => Err(JinError::Parse {
                format: "unknown".to_string(),
                message: "File has no extension".to_string(),
            }),
        }
    }
}

// Conversion implementations
impl From<serde_json::Value> for MergeValue { /* ... */ }
impl From<MergeValue> for serde_json::Value { /* ... */ }
impl From<serde_yaml::Value> for MergeValue { /* ... */ }
impl From<MergeValue> for serde_yaml::Value { /* ... */ }
impl From<toml::Value> for MergeValue { /* ... */ }
impl TryFrom<MergeValue> for toml::Value { /* ... */ }
```

**Advantages:**
1. Format-agnostic representation
2. Automatic format detection
3. Lossless round-trip conversion (mostly)
4. Handles format-specific constraints (e.g., TOML null)

---

### Example 3: Using merge Crate with Derive Macros

**Example from Rust ecosystem:**

```rust
use merge::Merge;

#[derive(Merge, Debug, Default, Serialize, Deserialize)]
struct AppConfig {
    // Default merge (replace)
    #[merge(skip)]
    name: String,

    // Append strategy for plugins
    #[merge(strategy = merge::vec::append)]
    plugins: Vec<Plugin>,

    // Union strategy for tags
    #[merge(strategy = merge::vec::unique)]
    tags: Vec<String>,

    // Nested config merge
    database: DatabaseConfig,

    // Deep merge for maps
    #[merge(strategy = merge::hashmap::deep_merge)]
    environment: HashMap<String, String>,
}

#[derive(Merge, Debug, Default, Serialize, Deserialize)]
struct DatabaseConfig {
    #[merge(skip)]
    name: String,

    #[merge(strategy = merge::num::saturating_add)]
    max_connections: u32,

    #[merge(skip)]
    #[merge(strategy = merge::option::overwrite_some)]
    ssl_cert: Option<String>,
}

// Usage
let mut base = AppConfig::default();
let overlay = AppConfig { /* ... */ };

base.merge(overlay);
```

---

### Example 4: Null Deletion Pattern

```rust
/// Perform a deep merge of two MergeValues
pub fn deep_merge(base: MergeValue, overlay: MergeValue) -> Result<MergeValue> {
    match (base, overlay) {
        // Null in overlay deletes the key
        (_, MergeValue::Null) => Ok(MergeValue::Null),

        // Both objects: recursive merge
        (MergeValue::Object(mut base_obj), MergeValue::Object(overlay_obj)) => {
            for (key, overlay_val) in overlay_obj {
                if overlay_val.is_null() {
                    // Null removes the key
                    base_obj.shift_remove(&key);
                } else if let Some(base_val) = base_obj.shift_remove(&key) {
                    // Recursively merge existing keys
                    let merged = deep_merge(base_val, overlay_val)?;
                    if !merged.is_null() {
                        base_obj.insert(key, merged);
                    }
                } else {
                    // Add new keys from overlay
                    base_obj.insert(key, overlay_val);
                }
            }
            Ok(MergeValue::Object(base_obj))
        }

        // Other cases: overlay wins
        (_, overlay) => Ok(overlay),
    }
}
```

**Key Features:**
1. Null values explicitly delete keys
2. Recursive merging for objects
3. Type-aware handling (scalars replaced, objects merged)

---

## References

### Official Documentation

1. **Kubernetes Strategic Merge Patch**
   - [GitHub: strategic-merge-patch.md](https://github.com/kubernetes/community/blob/master/contributors/devel/sig-api-machinery/strategic-merge-patch.md)
   - [Kubernetes API Machinery Package](https://pkg.go.dev/k8s.io/apimachinery/pkg/util/strategicpatch)

2. **webpack-merge**
   - [GitHub Repository](https://github.com/survivejs/webpack-merge)
   - [npm Package](https://www.npmjs.com/package/webpack-merge)
   - [SurviveJS Documentation](https://survivejs.com/books/webpack/developing/composing-configuration/)

3. **Helm Chart Values**
   - [Helm Values Files Guide](https://helm.sh/docs/chart_template_guide/values_files/)
   - [Merging Dynamic Configuration in Helm Charts](https://armel.soro.io/merging-dynamic-config-data-in-helm-charts/)

4. **Docker Compose Merge**
   - [Docker Compose Merge Documentation](https://docs.docker.com/compose/how-tos/multiple-compose-files/merge/)
   - [GitHub Issue: Array Merge Inconsistencies](https://github.com/docker/compose/issues/9756)

5. **ESLint Configuration**
   - [Configuration Files Documentation](https://eslint.org/docs/latest/use/configure/configuration-files)
   - [Combine Configs Guide](https://eslint.org/docs/latest/use/configure/combine-configs/)

### Rust Libraries

1. **merge Crate**
   - [Crates.io](https://crates.io/crates/merge)
   - [Docs.rs](https://docs.rs/merge/latest/merge/)
   - [Lib.rs](https://lib.rs/crates/merge)

2. **serde-toml-merge**
   - [GitHub Repository](https://github.com/jdrouet/serde-toml-merge)
   - [Crates.io](https://crates.io/crates/serde-toml-merge)

3. **deepmerge**
   - [Docs.rs](https://docs.rs/deepmerge/latest/deepmerge/)

4. **schematic**
   - [Crates.io](https://crates.io/crates/schematic/)
   - [Lib.rs](https://lib.rs/crates/schematic)

### Related Tools

1. **jsonmerge (Python)**
   - [PyPI Package](https://pypi.org/project/jsonmerge/)
   - [Documentation](https://github.com/avian2/jsonmerge)

2. **yq (YAML Processor)**
   - [Multiply/Merge Operators](https://mikefarah.gitbook.io/yq/operators/multiply-merge)

3. **Cargo Configuration**
   - [Cargo Configuration System](https://doc.rust-lang.org/cargo/reference/config.html)
   - [Inside Rust Blog: Array Merge Changes](https://blog.rust-lang.org/inside-rust/2023/08/24/cargo-config-merging/)

### Research Sources

1. **JavaScript Array Merging**
   - [Lodash Issue: Empty Array Merge](https://github.com/lodash/lodash/issues/1313)
   - [How to Merge Arrays and Deduplicate](https://xjavascript.com/blog/how-to-merge-two-arrays-in-javascript-and-de-duplicate-items-while-preserving-original-order/)

2. **PHP Array Operations**
   - [PHP array_merge() Manual](https://www.php.net/manual/en/function.array-merge.php)
   - [Array Merge vs Union Operator](https://www.geeksforgeeks.org/php/what-is-the-difference-between-array-merge-and-array-array-in-php/)

3. **Keyed Array Merging**
   - [JavaScript Merge by Key](https://www.geeksforgeeks.org/how-to-merge-multiple-array-of-object-by-id-in-javascript/)
   - [YAML Merge by ID](https://github.com/sjramblings/yaml-merge)

---

## Conclusion

Array merge strategies are essential for configuration management in layered systems. The choice of strategy depends on:

1. **Use Case**: Simple overrides vs. incremental configuration building
2. **Data Structure**: Primitive arrays vs. object arrays with identifiers
3. **Constraints**: Format limitations (TOML no null), schema requirements
4. **Precedence**: Layer ordering and conflict resolution
5. **Usability**: Explicit control vs. convention-based defaults

**Best Practices:**

- Use **keyed merge** for arrays of objects with identifiers
- Provide **explicit strategy specification** (don't rely on guessing)
- Document **merge behavior clearly** in schema/comments
- Support **null-based deletion** for explicit removal
- Test **edge cases** thoroughly (empty arrays, missing keys, duplicates)
- Consider **format constraints** (TOML limitations, INI nesting)
- Validate **after merge** to catch errors early

The jin project's implementation provides a good reference for practical keyed array merging in Rust with automatic format detection and format-specific constraint handling.
