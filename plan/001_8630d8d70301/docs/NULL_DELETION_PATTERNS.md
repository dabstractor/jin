# Null-Deletion Patterns in Configuration Merging Systems

A comprehensive research document on how popular tools and systems implement null-based deletion in configuration merging, including semantics, alternatives, and implementation patterns.

## Table of Contents

1. [How Popular Tools Implement Null-Deletion](#how-popular-tools-implement-null-deletion)
2. [Semantics of "Null Deletes Key"](#semantics-of-null-deletes-key)
3. [Alternative Approaches](#alternative-approaches)
4. [Implementation Patterns](#implementation-patterns)
5. [Key Findings and Recommendations](#key-findings-and-recommendations)

---

## How Popular Tools Implement Null-Deletion

### 1. Kubernetes Strategic Merge Patch

Kubernetes uses the Strategic Merge Patch format (an extension of JSON Merge Patch) with explicit support for null-based deletion.

#### Null Deletion Syntax

```bash
# Remove a key from a secret by setting it to null
kubectl patch secret test-secret -p '{"data":{"username":null,"password":"WTRueXM3ZjEx"}}'
```

This removes the `username` key entirely while preserving the `password` key.

#### Delete Directive Syntax

Kubernetes provides an explicit `$patch: delete` directive for more complex deletion scenarios:

```yaml
# Delete a specific container from a pod spec
containers:
- name: nginx
  image: nginx-1.0
- $patch: delete
  name: log-tailer  # merge key identifies which element to delete
```

**Key Semantics:**
- Both `null` and `$patch: delete` are semantically equivalent for deleting maps
- For list-of-maps structures, the delete directive removes ALL entries matching the merge key
- The `deleteFromPrimitiveList` directive removes items from primitive lists (strings, numbers, etc.)

#### Known Issues

There have been reports of different behavior between `null` and `$patch: delete` directives in some implementations, raising questions about consistency across different Kubernetes clients.

**References:**
- [Strategic Merge Patch Documentation](https://github.com/kubernetes/community/blob/master/contributors/devel/sig-api-machinery/strategic-merge-patch.md)
- [Kubernetes Issue #89140](https://github.com/kubernetes/kubernetes/issues/89140)
- [Kustomize Issue #386](https://github.com/kubernetes-sigs/kustomize/issues/386)

---

### 2. JSON Merge Patch (RFC 7396)

RFC 7396 defines the JSON Merge Patch format, which standardizes null-based deletion at the protocol level.

#### Core Specification

The RFC 7396 specification is built on a comparison principle:

> "Recipients of a merge patch document determine the exact set of changes being requested by comparing the content of the provided patch against the current content of the target document."

#### Null Deletion Rule

**The fundamental rule:**
- Members present in the patch but absent from the target are **added**
- Existing members in the target with patch values are **replaced**
- **Members set to null in the patch are REMOVED from the target**

#### Example

Given an original document:
```json
{
  "author": {
    "givenName": "John",
    "familyName": "Doe"
  },
  "title": "An interesting article"
}
```

Applying this patch:
```json
{
  "author": {
    "familyName": null
  }
}
```

Results in:
```json
{
  "author": {
    "givenName": "John"
  },
  "title": "An interesting article"
}
```

#### Critical Limitation

**RFC 7396 has a fundamental design limitation:**

> "The merge patch format works best for JSON documents that primarily use objects for their structure and does not make use of explicit null values. The merge patch format is not appropriate for all JSON syntaxes."

This means:
- **You cannot set a value to `null`** - setting null always means "delete this key"
- The format cannot selectively modify array elements
- It cannot patch non-object values (arrays, primitives)
- It is unsuitable for JSON documents that use explicit null values as data

#### Media Type

The official media type for JSON Merge Patch is: `application/merge-patch+json`

**References:**
- [RFC 7396 - JSON Merge Patch](https://datatracker.ietf.org/doc/html/rfc7396)
- [RFC 7396 Full Text](https://www.rfc-editor.org/rfc/rfc7396)

---

### 3. Terraform Override Semantics

Terraform treats null values as "absence" rather than explicit deletions.

#### Null-as-Unset Behavior

In Terraform, setting a value to `null` is equivalent to omitting it entirely:

```hcl
variable "example" {
  type    = string
  default = "original"
}

# Override file: example_override.tf
variable "example" {
  default = null  # Treated as "unset", uses the original default if required
}
```

**Key Behavior:**
- `null` causes Terraform to behave as though the argument was completely omitted
- The argument's default value is used if available
- An error is raised if the argument is mandatory and no default exists
- This is most useful in conditional expressions to dynamically omit arguments

#### Override File Merging

Terraform has special handling for files ending with `_override.tf` or `_override.tf.json`:

```hcl
# main.tf
resource "aws_instance" "example" {
  ami           = "ami-12345678"
  instance_type = "t2.micro"
}

# override.tf - Only specified fields override
resource "aws_instance" "example" {
  instance_type = "t2.large"  # Overrides the original value
}
```

#### Handling Null in Lists

The `compact()` function removes null and empty string values:

```hcl
# Terraform 1.5+
local.filtered = compact([
  "item1",
  null,
  "item2",
  ""
])
# Result: ["item1", "item2"]
```

#### Known Issues

There is a subtle issue with representing null as an unset value. In some cases, null values cause unexpected deprecation warnings when they should be treated identically to unset values.

**References:**
- [Terraform Override Files Documentation](https://developer.hashicorp.com/terraform/language/files/override)
- [Terraform Issue #31730](https://github.com/hashicorp/terraform/issues/31730)
- [DEV Community - Removing Null Values from Lists](https://dev.to/markwragg/how-to-remove-null-or-empty-string-values-from-a-list-in-terraform-1e3b)

---

### 4. Ansible Variable Precedence

Ansible does not have explicit null-deletion semantics in its variable system. Instead, it focuses on variable precedence and merging.

#### Precedence Hierarchy

From lowest to highest precedence:

1. Command line values (not variables)
2. Role defaults
3. Inventory file group variables
4. Inventory `group_vars/`
5. Playbook `group_vars/`
6. Inventory host variables
7. Inventory `host_vars/`
8. Playbook `host_vars/`
9. Host facts and cached `set_facts`
10. Play variables
11. Play `vars_prompt`
12. Play `vars_files`
13. Role variables
14. Block variables
15. Task variables
16. `include_vars`
17. `set_facts` and registered variables
18. Role and `include_role` parameters
19. `include` parameters
20. **Extra variables (-e flag) - HIGHEST PRECEDENCE**

#### Null/Undefined Handling

Ansible does not have explicit null deletion semantics like Kubernetes or JSON Merge Patch. Instead:

- Variables defined at different precedence levels simply override each other
- The highest precedence value is always used
- To override all other variables and "delete" a configuration value, use extra variables:

```bash
ansible-playbook playbook.yml -e "variable_name=null"
```

#### Best Practices

Rather than worrying about null-deletion, Ansible recommends:
- Define each variable in exactly one place
- Use role defaults for easily overridable values
- Use role vars for values that should be more explicit
- Use extra variables (-e) for one-time overrides

**References:**
- [Ansible Variable Precedence Documentation](https://docs.ansible.com/ansible/latest/reference_appendices/general_precedence.html)
- [Ansible Using Variables](https://docs.ansible.com/projects/ansible/latest/playbook_guide/playbooks_variables.html)
- [Spacelift - Ansible Variable Precedence Explained](https://spacelift.io/blog/ansible-variable-precedence)

---

## Semantics of "Null Deletes Key"

### When to Remove the Key Entirely vs Set to Null

This distinction is critical and varies by system:

#### RFC 7396 Perspective

In JSON Merge Patch, there is **no distinction** - null always means delete:

| Operation | Result |
|-----------|--------|
| Set key to null | **Key is completely removed** |
| Set key to "null" (string) | Value becomes the string "null" |
| Set key to explicit null object | Still removes the key |

#### Kubernetes Perspective

Kubernetes makes the distinction clear:

| Operation | Result | Use Case |
|-----------|--------|----------|
| `key: null` | Removes the key from the resource | When you want to delete a field |
| `key: ""` (empty string) | Sets the value to empty string | When you want to keep the field but clear it |
| No key in patch | Leaves the original value unchanged | When you want to preserve the field |
| `$patch: delete` | Removes the key explicitly | For complex deletions with merge keys |

#### Terraform Perspective

Terraform distinguishes between null and absence:

| Operation | Result | Behavior |
|-----------|--------|----------|
| `argument = null` | Treated as unset/omitted | Uses default value if available |
| `argument` omitted | Equivalent to null | Same as above |
| `argument = ""` | Sets to empty string | Empty string is a valid value |

### How to Handle Nested Null Deletion

The semantics of null deletion change when dealing with nested structures.

#### JSON Merge Patch Nested Deletion

RFC 7396 provides recursive semantics through the `MergePatch` function:

```json
// Original document
{
  "author": {
    "givenName": "John",
    "familyName": "Doe",
    "address": {
      "street": "123 Main St",
      "city": "New York"
    }
  }
}

// Patch - delete nested field
{
  "author": {
    "address": {
      "city": null
    }
  }
}

// Result - deeply nested field is removed
{
  "author": {
    "givenName": "John",
    "familyName": "Doe",
    "address": {
      "street": "123 Main St"
    }
  }
}
```

**Key Point:** The merge operation recursively applies the patch to nested objects. Null values at any depth trigger deletion at that specific level.

#### Kubernetes Strategic Merge Patch Nested Deletion

Kubernetes handles nested objects with merge semantics:

```yaml
# Original spec
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: nginx
        image: nginx:1.0
        env:
        - name: VAR1
          value: value1
        - name: VAR2
          value: value2

# Patch - delete nested environment variable
apiVersion: v1
kind: Pod
spec:
  template:
    spec:
      containers:
      - name: nginx
        env:
        - name: VAR2
          $patch: delete

# Result - only VAR1 remains
```

#### Propagation Rules for Nested Objects

When implementing nested null deletion, follow these rules:

1. **Null at any depth removes that key:** `patch.foo.bar = null` removes the `bar` key from the `foo` object
2. **Objects merge recursively:** If patch has `{a: {b: value}}` and target has `{a: {c: other}}`, result is `{a: {b: value, c: other}}`
3. **Null removes the key, not the parent:** Setting `patch.foo.bar = null` does NOT remove the `foo` object, only the `bar` key
4. **Empty objects are preserved:** After deleting all keys from an object, the empty object `{}` remains (unless explicitly deleted)

### Marker Values for Deletion in Non-Null-Supporting Formats

For formats like TOML and INI that don't support null values, alternative approaches are needed.

#### TOML Limitations

TOML explicitly does NOT support null values. This is a deliberate design decision.

**Alternatives for TOML:**
1. **Key Absence:** Simply don't include the key (the standard approach)
2. **Comments:** Comment out keys to indicate they could be set:
   ```toml
   # Configuration specification
   # username = "user"  # Uncomment and set value
   ```
3. **Sentinel Values:** Use special string values to indicate deletion:
   ```toml
   # In application code, treat "__DELETE__" as a marker
   username = "__DELETE__"  # Will be removed during merge
   ```
4. **Separate Metadata:** Use a companion file to specify which keys should be deleted
   ```
   .config.toml (data)
   .config.deletions.txt (which keys to delete)
   ```

#### INI Format Alternatives

INI format also lacks null support (no standardized implementation):

**Approaches:**
1. **Empty value syntax:** Some INI parsers treat empty values as null:
   ```ini
   ; Approach 1: Empty value
   username =

   ; Approach 2: Special marker
   username = NULL
   ```
2. **Section removal:** Delete entire sections instead of individual keys:
   ```ini
   ; Original
   [database]
   host = localhost

   ; To delete, override with no database section
   ```

---

## Alternative Approaches

### 1. Explicit Marker Patterns: `$delete` and `$unset`

Some systems use explicit markers instead of relying on null semantics.

#### Kubernetes `$patch: delete` Directive

Kubernetes uses the `$patch` directive for explicit deletion operations:

```yaml
# Delete specific elements from a list
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: nginx
  - $patch: delete
    name: sidecar  # The merge key that identifies the element to delete

# Delete entire maps
spec:
  strategy:
    type: RollingUpdate
  - $patch: delete
    rollingUpdate: null  # Explicit deletion of the entire rolling update strategy
```

**Advantages:**
- Explicit and unambiguous intent
- Works in YAML where null might be interpreted as "no value set"
- Separates deletion logic from data modeling

**Disadvantages:**
- Requires special parser support
- Not portable to other configuration systems
- More verbose than implicit null deletion

#### Proposed `$delete` and `$unset` Patterns

Some systems propose structured markers:

```json
// Pattern 1: $delete marker
{
  "config": {
    "database": { "$delete": true },
    "cache": { "ttl": 3600 }
  }
}

// Pattern 2: $unset marker
{
  "config": {
    "database": { "$unset": true },
    "cache": { "ttl": 3600 }
  }
}

// Pattern 3: Special key-value pair
{
  "config": {
    "__deletion_targets__": ["database", "cache.old_setting"],
    "cache": { "ttl": 3600 }
  }
}
```

#### MusicBrainz Picard `$delete` Function

In MusicBrainz Picard, `$delete` unsets a variable and marks a tag for deletion:

```
$delete(field_name)  # Remove field and mark tag for deletion
$unset(field_name)   # Just remove the field
```

**References:**
- [MusicBrainz Picard $delete Documentation](https://picard-docs.musicbrainz.org/en/functions/func_delete.html)

---

### 2. YAML Custom Tags: `!delete` and `!null`

YAML's tag system allows custom markers for domain-specific behavior.

#### Implementing Custom YAML Tags

YAML supports custom tags using the `!` prefix:

```yaml
# Using custom !delete tag
config:
  database: !delete
  cache:
    ttl: 3600

# Using custom !unset tag
config:
  database: !unset
  cache:
    ttl: 3600

# Using custom !remove tag
config:
  database: !remove ~
  cache:
    ttl: 3600
```

#### How to Define Custom Tags

**In Python (PyYAML):**

```python
import yaml

class DeleteMarker:
    """Custom YAML tag for marking fields for deletion"""
    def __init__(self):
        pass

    def __repr__(self):
        return 'DELETE'

def delete_constructor(loader, node):
    return DeleteMarker()

# Register the tag
yaml.add_constructor('!delete', delete_constructor)

# Usage
yaml_doc = """
config:
  database: !delete
  cache:
    ttl: 3600
"""

data = yaml.safe_load(yaml_doc)
# data['config']['database'] is now DeleteMarker()
```

**In Rust (using the `yaml` crate):**

```rust
// Custom tag handling would require extending the YAML parser
// The eemeli/yaml library supports custom tags through a tag resolution system

// Define custom tag with explicit tag
let yaml_with_tag = "!<tag:example.com,2019:delete> null";

// Or local tag
let yaml_with_local_tag = "!delete null";
```

#### Libraries Supporting Custom Tags

| Language | Library | Custom Tag Support |
|----------|---------|-------------------|
| Python | PyYAML | Yes, via `yaml.add_constructor()` |
| Rust | yaml | Yes, via custom tag resolution |
| JavaScript | eemeli/yaml | Yes, full custom tag support |
| Symfony | symfony/yaml | Yes, via PARSE_CUSTOM_TAGS flag |

**References:**
- [Eemeli YAML Custom Tags Documentation](https://github.com/eemeli/yaml/blob/main/docs/06_custom_tags.md)
- [YAML Tags Tutorial](https://tutorialreference.com/yaml/yaml-tags/)

---

### 3. Empty String vs Null Semantics

Different interpretations of empty values in configuration systems.

#### String-Based Distinction

| Value | Type | Meaning | Use Case |
|-------|------|---------|----------|
| `null` (JSON) / `~` (YAML) | Null | Absence of value | Delete the key |
| `""` (empty string) | String | Empty but present | Clear but keep field |
| Not present | Absent | Field not specified | Use default |
| `"null"` | String literal | The text "null" | When "null" is a valid value |

#### Kubernetes Pod Spec Example

```yaml
# Original
spec:
  containers:
  - name: app
    image: myapp:1.0
    args: ["--config", "/etc/config"]

# Empty string clears args but keeps the field
spec:
  containers:
  - name: app
    args: null  # Removes the args field entirely

# Empty list clears args
spec:
  containers:
  - name: app
    args: []  # Array is empty but field exists
```

#### Terraform Empty String Behavior

```hcl
variable "endpoint" {
  type = string
}

# Three different states:
# endpoint = null           -> Use default if available, error if required
# endpoint = ""            -> Empty string, a valid value
# endpoint = <not set>     -> Omitted, same as null
```

**References:**
- [Kubernetes Pod Spec Documentation](https://kubernetes.io/docs/reference/generated/kubernetes-api/v1.28/#podspec-v1-core)
- [Terraform Variable Types](https://developer.hashicorp.com/terraform/language/values/variables)

---

## Implementation Patterns

### 1. How to Propagate Deletion Through Nested Objects

#### Recursive Merge with Null Propagation

**Algorithm: Deep Merge with Null Handling**

```
function merge(target, patch):
    if patch is null:
        return undefined (signal to delete)

    if patch is not an object:
        return patch (replace with primitive value)

    if target is not an object:
        target = {} (convert to object for merging)

    for each key in patch:
        value = patch[key]

        if value is null:
            delete target[key]
        else if value is an object and target[key] is an object:
            merge(target[key], value)  // Recurse
        else:
            target[key] = value

    return target
```

#### Rust Implementation Example

Using the `json-patch` crate for RFC 7396 compliance:

```rust
use serde_json::{json, Value};
use json_patch::merge;

// Simple null deletion
fn simple_merge_example() {
    let mut doc = json!({
        "author": {
            "givenName": "John",
            "familyName": "Doe"
        }
    });

    let patch = json!({
        "author": {
            "familyName": null
        }
    });

    merge(&mut doc, &patch);

    // Result: familyName key is removed
    assert_eq!(doc["author"]["familyName"], Value::Null);
    // But the author object still exists
    assert!(doc["author"]["givenName"].is_string());
}
```

**Key Rust Libraries:**

1. **json-patch (Recommended)**
   - Supports RFC 6902 (JSON Patch) and RFC 7396 (JSON Merge Patch)
   - Handles nested null deletion correctly
   - Repository: https://github.com/idubrov/json-patch
   - Crate: https://crates.io/crates/json-patch

2. **serde_merge**
   - Merges serializable types
   - Supports custom merge strategies
   - Good for struct-based configuration

3. **json_value_merge**
   - Direct JSON Value merging
   - Path-based merge operations

#### Deep Merge Pattern (JavaScript/TypeScript reference)

For reference, here's how deep merge handles null propagation:

```javascript
function deepMerge(target, source) {
    for (const key in source) {
        if (Object.prototype.hasOwnProperty.call(source, key)) {
            const sourceValue = source[key];
            const targetValue = target[key];

            if (sourceValue === null) {
                // Null marks deletion
                delete target[key];
            } else if (
                sourceValue !== null &&
                typeof sourceValue === 'object' &&
                !Array.isArray(sourceValue) &&
                targetValue !== null &&
                typeof targetValue === 'object' &&
                !Array.isArray(targetValue)
            ) {
                // Both are objects, recurse
                deepMerge(targetValue, sourceValue);
            } else {
                // Replace or add value
                target[key] = sourceValue;
            }
        }
    }
    return target;
}
```

**Considerations:**
- Null values propagate deletion at their depth level
- Parent objects are not deleted, only keys within them
- Recursive calls maintain the merge path context
- Order matters: process in patch order for deterministic results

### 2. How to Handle Array Element Deletion

Array deletion is more complex than object key deletion.

#### Semantic Options for Array Deletion

| Approach | Semantics | Example |
|----------|-----------|---------|
| Index-based | Delete by position | Remove `array[2]` |
| Value-based | Delete by matching value | Remove all items == "value" |
| Merge-key based | Delete by identifier field | Remove item where `name: "target"` |
| Replace entire array | Replace whole array | `array: []` |
| Mark-and-filter | Use marker for later filtering | `array: [{$delete: true}, ...]` |

#### Kubernetes Array Deletion (Merge-Key Based)

Kubernetes uses merge keys to identify array elements for deletion:

```yaml
# Deployment with multiple containers
spec:
  template:
    spec:
      containers:
      - name: nginx
        image: nginx:1.0
      - name: sidecar
        image: sidecar:1.0
      - name: logger
        image: logger:1.0

# Patch: Delete container by merge key (name)
spec:
  template:
    spec:
      containers:
      - $patch: delete
        name: sidecar
```

**How it works:**
1. The merge key is `name` for containers
2. The patch specifies `$patch: delete` and the merge key value
3. All array elements matching that merge key are deleted

#### RFC 7396 Array Behavior (Replacement Only)

JSON Merge Patch does NOT support selective array element deletion. Arrays are either:
- **Replaced entirely:** If patch has an array, the entire target array is replaced
- **Left untouched:** If patch doesn't mention an array, it's preserved

```json
// Original
{
  "tags": ["tag1", "tag2", "tag3"]
}

// Patch
{
  "tags": ["tag1", "tag3"]
}

// Result - entire array is replaced
{
  "tags": ["tag1", "tag3"]
}
```

This is a limitation of RFC 7396 - it cannot express "remove array[1]" without replacing the whole array.

#### JSON Patch (RFC 6902) Array Deletion

RFC 6902 (JSON Patch) provides more granular control:

```json
[
  { "op": "remove", "path": "/tags/1" }
]
```

This removes the element at index 1.

#### Implementation Pattern for Array Deletion

**Rust approach using merge keys:**

```rust
use serde_json::{json, Value};

fn delete_array_element_by_key(
    array: &mut Vec<Value>,
    merge_key: &str,
    key_value: &str,
) {
    array.retain(|item| {
        if let Some(key_val) = item.get(merge_key) {
            key_val != key_value
        } else {
            true // Keep items without the merge key
        }
    });
}

// Usage
let mut containers = json!([
    { "name": "nginx", "image": "nginx:1.0" },
    { "name": "sidecar", "image": "sidecar:1.0" },
    { "name": "logger", "image": "logger:1.0" }
]);

if let Some(arr) = containers.as_array_mut() {
    delete_array_element_by_key(arr, "name", "sidecar");
    // Result: sidecar container is removed
}
```

**Key Patterns:**
- Use merge keys (unique identifiers) to mark elements for deletion
- Implement `retain()` pattern to keep non-deleted elements
- For ordered arrays, support index-based deletion separately
- Document which arrays support selective deletion vs full replacement

### 3. Error Handling for Invalid Deletion Operations

Proper error handling is critical for safe configuration merging.

#### Common Error Scenarios

| Scenario | Error | Handling |
|----------|-------|----------|
| Delete non-existent key | Warn or ignore | Most systems ignore silently |
| Delete from non-object | Type error | Reject the patch |
| Delete required field | Validation error | Reject after deletion |
| Delete with conflicting merge operations | Merge conflict | Report to user |
| Invalid merge key reference | Key not found | Warn or skip |
| Circular reference in nested deletion | Stack overflow | Detect and fail |

#### Kubernetes Error Handling

Kubernetes provides validation at multiple levels:

```yaml
# Example 1: Invalid patch (trying to delete from array with wrong syntax)
# Error: "spec.containers: Invalid value: 'null': spec.containers in body must be of type array"

# Solution: Use proper merge-key based deletion
spec:
  containers:
  - $patch: delete
    name: container-name

# Example 2: Trying to delete a required field
# Error: "spec.replicas: Required value"
spec:
  replicas: null  # This will fail validation

# Example 3: Delete with conflicting operations
# Error: Multiple conflicting delete operations
spec:
  strategy:
  - $patch: delete
  - $patch: delete  # Conflict!
```

#### Rust Error Handling Pattern

```rust
use std::collections::HashMap;

#[derive(Debug)]
enum MergeError {
    /// Attempted to delete from a non-object
    NotAnObject {
        path: String,
        actual_type: String,
    },
    /// Merge key not found in array element
    MergeKeyNotFound {
        key: String,
        element_index: usize,
    },
    /// Circular reference detected
    CircularReference {
        path: String,
    },
    /// Required field would be deleted
    DeletingRequiredField {
        field: String,
    },
}

type MergeResult<T> = Result<T, MergeError>;

fn merge_with_validation(
    target: &mut serde_json::Value,
    patch: &serde_json::Value,
    required_fields: &[&str],
) -> MergeResult<()> {
    if patch.is_null() {
        // Check if this key is required
        // This is context-dependent, so validation happens at call site
        return Ok(());
    }

    if !patch.is_object() {
        // Non-null, non-object patches replace
        *target = patch.clone();
        return Ok(());
    }

    if !target.is_object() && !target.is_null() {
        return Err(MergeError::NotAnObject {
            path: "current".to_string(),
            actual_type: target.type_str().to_string(),
        });
    }

    let patch_obj = patch.as_object().unwrap();
    let target_obj = target.as_object_mut().unwrap_or(&mut serde_json::Map::new());

    for (key, value) in patch_obj {
        if value.is_null() {
            if required_fields.contains(&key.as_str()) {
                return Err(MergeError::DeletingRequiredField {
                    field: key.clone(),
                });
            }
            target_obj.remove(key);
        } else if value.is_object() && target_obj.get(key).map_or(false, |v| v.is_object()) {
            // Recurse for nested objects
            let nested_target = &mut target_obj[key];
            merge_with_validation(nested_target, value, &[])?;
        } else {
            target_obj.insert(key.clone(), value.clone());
        }
    }

    *target = serde_json::Value::Object(target_obj.clone());
    Ok(())
}
```

#### RFC 7396 Error Conditions

According to RFC 7396, servers should handle these conditions:

1. **Invalid JSON:** Reject with 400 Bad Request
2. **Type conflicts:** Replace the conflicting value
3. **Non-object targets with object patches:** Recurse or replace based on implementation
4. **Circular references:** Not explicitly addressed - implementations should prevent infinite loops

#### Detection Strategy for Circular References

```rust
use std::collections::HashSet;

fn merge_with_cycle_detection(
    target: &mut serde_json::Value,
    patch: &serde_json::Value,
    visited: &mut HashSet<String>,
    path: &str,
) -> MergeResult<()> {
    if visited.contains(path) {
        return Err(MergeError::CircularReference {
            path: path.to_string(),
        });
    }

    visited.insert(path.to_string());

    // Perform merge...

    visited.remove(path);
    Ok(())
}
```

**Best Practices:**
- Validate patches before applying them
- Check for required fields before deletion
- Provide clear error messages with path context
- Implement cycle detection for nested structures
- Fail fast on validation errors (don't apply partial patches)
- Log all deletion operations for audit trails

---

## Key Findings and Recommendations

### Comparative Summary Table

| System | Null Deletes | Alternative | Arrays | Nested | Edge Cases |
|--------|--------------|-------------|--------|--------|-----------|
| **Kubernetes SMP** | Yes (`null` & `$patch: delete`) | Merge keys for arrays | Merge-key based | Full recursion | Inconsistency between null and $patch: delete |
| **RFC 7396** | Yes (only method) | None | Full replacement only | Recursive objects | Cannot set to null; unsuitable for null values as data |
| **Terraform** | As unset (if default available) | `compact()` function | State file only | Blocks only | Subtle null/unset distinction issues |
| **Ansible** | No explicit deletion | Extra vars override | Precedence merge | Precedence merge | No built-in null deletion pattern |

### Implementation Recommendations

#### For Rust Configuration Systems

1. **Use the `json-patch` crate** for RFC 7396 compliance
   - Well-maintained and audited
   - Supports both JSON Patch and JSON Merge Patch
   - MIT/Apache 2.0 dual licensed
   - Example: https://github.com/idubrov/json-patch

2. **Implement custom merge strategies** for domain-specific behavior
   - Define `DeleteMarker` types for configurations that need explicit deletion
   - Use Serde's `#[serde(skip_serializing_if)]` for conditional omission
   - Implement custom `Deserialize` logic for custom tags

3. **Always validate before deletion**
   - Check required fields
   - Validate against schema
   - Maintain audit trail of deletions
   - Use type system to enforce invariants

#### For Configuration Format Choice

| Use Case | Recommended Format | Deletion Pattern |
|----------|-------------------|------------------|
| REST API patches | JSON Merge Patch (RFC 7396) | Null deletion |
| Kubernetes resources | Strategic Merge Patch | Null & `$patch: delete` |
| Infrastructure as Code | YAML with merge keys | Custom tags or separate metadata |
| System configuration | INI or TOML | Key absence or sentinel values |
| Application config | YAML or TOML | Extra override file |

#### Error Handling Strategy

```rust
// Three-level validation approach:

// Level 1: Schema validation
validate_against_schema(&patch)?;

// Level 2: Required field checks
check_required_fields(&target, &patch)?;

// Level 3: Post-merge validation
let mut merged = target.clone();
merge(&mut merged, patch)?;
validate_against_schema(&merged)?;
```

### Known Limitations and Workarounds

#### RFC 7396 Limitations

**Problem:** Cannot set a value to null
**Workaround:** Use a different format (JSON Patch RFC 6902) or separate metadata file

**Problem:** Cannot selectively delete array elements
**Workaround:** Replace entire array or use custom merge logic with merge keys

#### Kubernetes Strategic Merge Patch Limitations

**Problem:** Inconsistent behavior between `null` and `$patch: delete`
**Workaround:** Use the more explicit `$patch: delete` directive; file issues with your client library

**Problem:** Cannot delete multiple items with single patch
**Workaround:** Combine all deletions in a single patch file

#### Terraform null Handling

**Problem:** Subtle distinction between `null` and omitted arguments
**Workaround:** Explicitly document expected defaults; use variable validation blocks

### Future Considerations

1. **Structured Data Types:** Consider using Protocol Buffers or similar for better schema enforcement
2. **Deletion Metadata:** Maintain separate deletion logs for audit compliance
3. **Version Control:** Track configuration merge history separately
4. **Custom Markers:** If implementing custom deletion markers, standardize across your organization
5. **Performance:** For large nested structures, consider lazy evaluation of merges

---

## References

### Standards and RFCs

- [RFC 7396 - JSON Merge Patch](https://datatracker.ietf.org/doc/html/rfc7396)
- [RFC 6902 - JSON Patch](https://tools.ietf.org/html/rfc6902)
- [Kubernetes Strategic Merge Patch](https://github.com/kubernetes/community/blob/master/contributors/devel/sig-api-machinery/strategic-merge-patch.md)

### Rust Libraries

- [json-patch Crate](https://crates.io/crates/json-patch)
  - GitHub: https://github.com/idubrov/json-patch
  - Documentation: https://docs.rs/json-patch/latest/json_patch/

- [serde_merge Crate](https://docs.rs/serde_merge)
  - For merging serializable types with custom strategies

- [config Crate](https://docs.rs/config/latest/config/)
  - Hierarchical configuration management

### Tool Documentation

- [Kubernetes PATCH Documentation](https://kubectl.docs.kubernetes.io/references/kustomize/kustomization/patchesstrategicmerge/)
- [Terraform Override Files](https://developer.hashicorp.com/terraform/language/files/override)
- [Ansible Variable Precedence](https://docs.ansible.com/ansible/latest/reference_appendices/general_precedence.html)
- [Cargo Configuration Merging](https://blog.rust-lang.org/inside-rust/2023/08/24/cargo-config-merging/)

### Related Issues and Discussions

- [Kubernetes Issue #89140 - Strategic Merge Patch Inconsistency](https://github.com/kubernetes/kubernetes/issues/89140)
- [Terraform Issue #31730 - Null as Unset](https://github.com/hashicorp/terraform/issues/31730)
- [Kustomize Issue #386 - Multiple Delete Directives](https://github.com/kubernetes-sigs/kustomize/issues/386)
- [TOML Issue #146 - NULL Support Discussion](https://github.com/toml-lang/toml/issues/146)

### Additional Resources

- [Deep Merge in JavaScript - GitHub Gist](https://gist.github.com/ahtcx/0cd94e62691f539160b32ecda18af3d6)
- [JSON Merge Patch Implementation - Rust Blog](https://gustawdaniel.com/posts/en/json-merge-patch/)
- [TOML vs INI Comparison - TechTarget](https://www.techtarget.com/searchdatacenter/tip/TOML-vs-INI-Comparing-configuration-file-formats)
- [Configuration Merge Behavior - Microsoft Learn](https://learn.microsoft.com/en-us/archive/blogs/markgabarra/configuration-merge-behavior)

---

## Document Information

- **Created:** 2025-12-27
- **Research Focus:** Null-deletion patterns in configuration merging systems
- **Sources:** RFC specifications, official documentation, GitHub issues, and academic references
- **Rust Compatibility:** All code examples are compatible with Rust 2021 edition
- **License:** This research document is provided as-is for educational and reference purposes

