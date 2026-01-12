# Deep Merge Quick Reference Guide

## Behavior Comparison Matrix

### Common Libraries

```
                    | Lodash     | deepmerge  | webpack-merge | Fastify
--------------------|------------|------------|---------------|----------
Array handling      | Replace    | Concat*    | Concat         | Custom
Null semantics      | Skip null  | Copy null  | Copy null      | Skip*
Circular refs       | NONE       | NONE       | NONE           | Custom
Mutating            | YES        | NO         | NO             | NO
Size (gzipped)      | 11.5 KB    | 723 B      | 8.5 KB         | 1.2 KB
Customize per-field | NO         | Custom fn  | Strategy       | Custom fn
Performance         | Fast       | Fastest    | Good           | Fastest*
Bundle size impact  | High       | MINIMAL    | Low            | MINIMAL
```

---

## Quick Decision Tree

### "Which merge library should I use?"

```
START
├─ Is this for webpack?
│  └─ YES → use webpack-merge
│
├─ Do you need circular ref handling?
│  └─ YES → use merge-deep-ts OR custom WeakMap tracking
│
├─ Is bundle size critical (web app)?
│  └─ YES → use deepmerge (723B)
│
├─ Need TypeScript with auto types?
│  └─ YES → use ts-deepmerge
│
├─ Need highly customizable per-field behavior?
│  └─ YES → use deepmerge with options OR custom implementation
│
├─ Is this Rust/Cargo project?
│  └─ YES → use merge crate with derive OR serde_json + custom logic
│
├─ Working with package.json?
│  └─ YES → use merge-packages (npm-aware)
│
└─ Default → use deepmerge (works great for most cases)
```

---

## Code Snippets by Use Case

### 1. Simple Deep Merge (JavaScript)

```javascript
// Using deepmerge
const merge = require('deepmerge');
const result = merge(baseConfig, overrideConfig);
```

### 2. Merge with Array Concatenation

```javascript
// webpack-merge style (concatenate arrays)
const { merge } = require('webpack-merge');
const result = merge(baseConfig, overrideConfig);
// Arrays automatically concatenated
```

### 3. Merge with Depth Limit (Safe)

```javascript
function mergeWithDepthLimit(target, source, maxDepth = 10) {
  function recurse(t, s, depth) {
    if (depth >= maxDepth) return s;

    if (isPlainObject(t) && isPlainObject(s)) {
      const result = { ...t };
      Object.keys(s).forEach(key => {
        result[key] = isPlainObject(s[key]) && isPlainObject(result[key])
          ? recurse(result[key], s[key], depth + 1)
          : s[key];
      });
      return result;
    }
    return s;
  }

  return recurse(target, source, 0);
}
```

### 4. Merge with Circular Reference Detection

```javascript
function mergeWithCircularCheck(target, source, visited = new WeakMap()) {
  if (visited.has(source)) return visited.get(source);
  visited.set(source, target);

  if (isPlainObject(target) && isPlainObject(source)) {
    const result = { ...target };
    Object.keys(source).forEach(key => {
      if (isPlainObject(source[key]) && isPlainObject(result[key])) {
        result[key] = mergeWithCircularCheck(result[key], source[key], visited);
      } else {
        result[key] = source[key];
      }
    });
    return result;
  }

  return source;
}
```

### 5. Rust: Type-Safe Merge with Derive

```rust
use merge::Merge;

#[derive(Merge)]
struct Config {
    #[merge(strategy = merge::vec::append)]
    plugins: Vec<String>,

    #[merge(strategy = merge::bool::overwrite_false)]
    enabled: bool,

    timeout: u32,
}

let mut base = Config { plugins: vec!["a".into()], enabled: true, timeout: 30 };
base.merge(Config { plugins: vec!["b".into()], enabled: false, timeout: 60 });
```

### 6. Rust: Order-Preserving Merge

```rust
use indexmap::IndexMap;
use serde_json::Value;

fn merge_configs(base: &mut IndexMap<String, Value>, override_cfg: &IndexMap<String, Value>) {
    for (key, value) in override_cfg {
        if let Some(base_val) = base.get_mut(key) {
            match (base_val, value) {
                (Value::Object(m1), Value::Object(m2)) => {
                    // Recursively merge nested objects
                    for (k, v) in m2 {
                        m1.insert(k.clone(), v.clone());
                    }
                }
                _ => { *base_val = value.clone(); }
            }
        } else {
            base.insert(key.clone(), value.clone());
        }
    }
}
```

### 7. YAML Merge with yq

```bash
# Merge two YAML files (right overwrites left)
yq eval-all '.[0] * .[1]' base.yaml override.yaml

# Concatenate arrays
yq eval '.[0].items += .[1].items' base.yaml override.yaml

# Export to stdout
yq eval '.' merged.yaml
```

### 8. Merge with Null/Undefined Semantics

```javascript
// Option 1: Skip undefined, null deletes key
function mergeWithNullDelete(target, source) {
  const result = { ...target };
  Object.entries(source).forEach(([key, value]) => {
    if (value === null) {
      delete result[key];
    } else if (value !== undefined) {
      result[key] = value;
    }
  });
  return result;
}

// Option 2: Copy all values including null
function mergeIncludingNull(target, source) {
  return { ...target, ...source };
}
```

### 9. Package.json Merge (Node.js)

```javascript
const mergePackageJson = require('merge-package.json');

const merged = mergePackageJson(local, base, remote);
// Handles three-way merge with conflict resolution
```

### 10. JSON Configuration with RFC 7396 (Rust)

```rust
use json_patch::{merge, Patch};
use serde_json::{json, Value};

let mut config = json!({
    "server": { "port": 3000, "ssl": false }
});

let patch = json!({
    "server": { "ssl": true }
});

merge(&mut config, &patch);
// config.server.ssl is now true, port still 3000
```

---

## Handling Edge Cases

### Problem: Type Conflict (object vs string)

```javascript
const base = { api: { timeout: 5000 } };      // object
const override = { api: "disabled" };         // string

// Bad - loses structure:
const result = merge(base, override);  // { api: "disabled" }

// Good - validate first:
function smartMerge(base, override) {
  if (typeof base.api === typeof override.api) {
    return merge(base, override);
  }
  // Type mismatch - handle explicitly
  throw new Error('Type conflict at api');
}
```

### Problem: Very Deep Nesting

```javascript
// Option A: Flatten before merge
const flatten = (obj, prefix = '') => {
  const result = {};
  Object.entries(obj).forEach(([k, v]) => {
    const key = prefix ? `${prefix}.${k}` : k;
    if (typeof v === 'object' && v !== null) {
      Object.assign(result, flatten(v, key));
    } else {
      result[key] = v;
    }
  });
  return result;
};

const flat1 = flatten(base);
const flat2 = flatten(override);
const merged = { ...flat1, ...flat2 };

// Option B: Use iterative approach (no stack overflow)
function iterativeMerge(target, source) {
  const stack = [{ target, source }];
  while (stack.length) {
    const { target, source } = stack.pop();
    Object.entries(source).forEach(([k, v]) => {
      if (typeof v === 'object' && v !== null && isPlainObject(target[k])) {
        stack.push({ target: target[k], source: v });
      } else {
        target[k] = v;
      }
    });
  }
  return target;
}
```

### Problem: Large Arrays (100k+ items)

```javascript
// Don't merge large arrays by default
const options = {
  arrayMerge: (target, source) => {
    // Only merge if both arrays are small
    if (target.length > 10000 || source.length > 10000) {
      return source;  // Replace instead of merge
    }
    return target.concat(source);
  }
};

const result = merge(config1, config2, options);
```

### Problem: Circular References

```javascript
function mergeWithCircularDetection(target, source) {
  const visited = new WeakMap();

  function recurse(t, s) {
    if (visited.has(s)) {
      return visited.get(s);  // Already merged
    }

    if (isPlainObject(t) && isPlainObject(s)) {
      visited.set(s, t);
      Object.keys(s).forEach(key => {
        t[key] = isPlainObject(s[key]) && isPlainObject(t[key])
          ? recurse(t[key], s[key])
          : s[key];
      });
    } else {
      t = s;
    }

    return t;
  }

  return recurse(target, source);
}
```

---

## Performance Tips

### Use Lazy Evaluation When Possible

```javascript
class LazyConfig {
  constructor(base, overrides) {
    this.base = base;
    this.overrides = overrides;
    this._cache = null;
  }

  get(path) {
    return this.overrides[path] ?? this.base[path];
  }

  materialize() {
    if (!this._cache) {
      this._cache = merge(this.base, this.overrides);
    }
    return this._cache;
  }
}

// Only materialize when needed!
```

### Avoid Repeated Merging

```javascript
// Bad - merges same objects multiple times
const result = merge(merge(merge(base, a), b), c);

// Good - batch merge
function batchMerge(base, ...sources) {
  return sources.reduce((acc, src) => merge(acc, src), base);
}
const result = batchMerge(base, a, b, c);
```

### Choose Right Data Structure

| Use Case | Best Choice |
|----------|-------------|
| Small config (<100 keys) | Object/HashMap |
| Need insertion order | IndexMap (Rust) / Object (JS) |
| Frequent lookups | HashMap |
| Iteration order matters | BTreeMap (Rust) / IndexMap |
| Very large config | Lazy evaluation + IndexMap |

---

## Common Pitfalls & Solutions

| Pitfall | Problem | Solution |
|---------|---------|----------|
| Forgot merge is mutating | Original changed | Use `_.cloneDeep` first or use non-mutating lib |
| Null overrides base value | Data loss | Define null semantics explicitly |
| Array gets replaced | Expected concatenation | Use webpack-merge or deepmerge with options |
| Stack overflow on deep nesting | Recursion limit hit | Use iterative approach or depth limit |
| Circular refs cause infinite loop | Application hangs | Use WeakMap tracking |
| Type conflict crashes | Different types in merge | Add type checking before merge |
| Performance degradation | Merging large arrays | Use lazy evaluation or array-replace strategy |

---

## Testing Checklist

```
Test Cases                          Status
─────────────────────────────────────────────
Empty objects {}                    □
Single level merge                  □
Multi-level nested merge            □
Array concatenation                 □
Array with objects                  □
Null values                         □
Undefined values                    □
Type conflicts (object vs string)   □
Circular reference                  □
Deep nesting (20+ levels)          □
Large arrays (10k+ items)          □
Special types (Date, RegExp)       □
Unicode/special characters         □
Numeric string keys                □
Prototype pollution check          □
```

---

## Quick Troubleshooting

**Q: My arrays are being replaced instead of merged**
- A: Using lodash? It replaces arrays by index. Use deepmerge or webpack-merge instead.

**Q: Null values are disappearing**
- A: Lodash skips null by design. Define your own merge strategy or use deepmerge.

**Q: Stack overflow with deep config**
- A: Use iterative merge or set depth limit. See "Very Deep Nesting" section.

**Q: Performance is slow**
- A: Use lazy evaluation, avoid merging in loops, use depth limit for untrusted input.

**Q: Circular references cause infinite loop**
- A: Use WeakMap tracking or merge-deep-ts which handles this automatically.

**Q: Key order is lost after merge**
- A: Use IndexMap (Rust), or choose library that preserves order (most modern ones do).

---

## Language-Specific Recommendations

### JavaScript/Node.js
- **Best All-Around**: `deepmerge` (tiny, customizable)
- **For Webpack**: `webpack-merge` (purpose-built)
- **For Bundle Size**: `deepmerge` (723B)
- **For Advanced Features**: `@fastify/deepmerge` (fastest, many options)

### Python
- **Best**: `deepmerge` PyPI package
- **For Config Files**: `jsonmerge` with OrderedDict support

### Go
- **Best**: `TwiN/deepmerge` (handles YAML and JSON)

### Rust
- **Best Ergonomic**: `merge` crate with derive macro
- **Best Flexible**: `serde_json` + custom recursion
- **Order Preservation**: `indexmap` + `serde`
- **RFC 7396**: `json-patch` crate

### YAML/JSON Tooling
- **Best CLI**: `yq` (YAML and JSON)
- **Alternative CLI**: `jq` (JSON only, more powerful)

---

**Last Updated**: December 27, 2025
**Version**: 1.0
