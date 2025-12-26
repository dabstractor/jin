# Product Requirement Prompt (PRP): MergeValue Type (P2.M1.T1)

---

## Goal

**Feature Goal**: Define the `MergeValue` enum - a unified value type that represents structured data from multiple formats (JSON, YAML, TOML, INI, text) and enables deep merge operations across Jin's 9-layer configuration hierarchy.

**Deliverable**: A `src/merge/value.rs` module with:
- `MergeValue` enum with variants for all primitive and collection types
- Format parsing functions (from JSON, YAML, TOML, INI strings)
- Deep merge implementation following PRD merge rules
- Order-preserving object/map type using `IndexMap`
- Serde serialization/deserialization support
- Comprehensive unit tests

**Success Definition**:
- `MergeValue` enum compiles with all variants
- Can parse JSON, YAML, TOML, INI strings into `MergeValue`
- Deep merge correctly implements PRD rules (objects merge, arrays override, null deletes key)
- `cargo test --package jin --lib merge::value` passes all tests
- No clippy warnings or rustc errors
- Module exported from `src/merge/mod.rs`

## User Persona

**Target User**: AI coding agent implementing the merge engine foundation

**Use Case**: The agent needs to create a unified value representation that:
- Parses multiple configuration formats (JSON, YAML, TOML, INI)
- Performs deep merges following PRD §11.1 rules
- Preserves ordering from the highest layer
- Integrates with existing `JinError` and `Layer` types

**User Journey**:
1. Agent receives this PRP as context
2. Creates `src/merge/value.rs` with `MergeValue` enum
3. Implements parsing functions for each format
4. Implements deep merge with correct precedence rules
5. Adds comprehensive unit tests
6. Validates compilation and test success

**Pain Points Addressed**:
- No need for separate value types per format
- Consistent merge behavior across all formats
- Order preservation for deterministic output
- Integration with existing error handling

## Why

- **Foundation for all merge operations**: Every subsequent merge task (P2.M2-P2.M4) depends on `MergeValue`
- **Multi-format support**: Jin handles JSON, YAML, TOML, INI, and text files
- **Deterministic merges**: PRD requires specific merge behavior (deep key merge, array strategies)
- **Integration point**: Bridges format parsers with merge algorithms

## What

Create a `MergeValue` enum that represents any structured value from Jin's supported formats and provides deep merge operations following the PRD specification.

### Merge Rules from PRD §11.1

| Type | Behavior |
|------|----------|
| JSON / YAML / TOML | Deep key merge |
| Arrays (keyed) | Merge by `id` or `name` |
| Arrays (unkeyed) | Higher layer replaces |
| `null` | Deletes key |
| Ordering | Preserved from highest layer |
| Comments | Not preserved |
| INI | Section merge |
| Text | 3-way diff (via `similar` crate, future task) |

### Success Criteria

- [ ] `MergeValue` enum with all variants compiles
- [ ] `from_json()` parses JSON strings into `MergeValue`
- [ ] `from_yaml()` parses YAML strings into `MergeValue`
- [ ] `from_toml()` parses TOML strings into `MergeValue`
- [ ] `from_ini()` parses INI strings into `MergeValue`
- [ ] `merge()` implements deep merge rules correctly
- [ ] `null` values delete keys during merge
- [ ] Array merging uses "higher layer replaces" strategy
- [ ] Order preservation using `IndexMap`
- [ ] All tests pass
- [ ] Module exported from `src/merge/mod.rs`

---

## All Needed Context

### Context Completeness Check

**Validation**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP provides:
- Exact `MergeValue` enum specification with all variants
- Research documents with code examples for all parsing patterns
- Specific patterns from existing codebase to follow
- Complete integration guide with `JinError` and `IndexMap`
- Validation commands specific to this project

### Documentation & References

```yaml
# MUST READ - Internal Project Documentation

- file: /home/dustin/projects/jin-glm-doover/PRD.md
  why: Complete merge strategy specification with precedence rules
  section: Lines 258-300 for Merge Strategy (§11)
  critical: Deep key merge rules, null deletes key, array strategies

- file: /home/dustin/projects/jin-glm-doover/src/core/error.rs
  why: Error handling patterns - use existing JinError variants
  pattern: JinError::JsonParse, JinError::YamlParse, JinError::TomlParse, JinError::IniParse
  gotcha: Parse errors already defined with transparent conversion

- file: /home/dustin/projects/jin-glm-doover/src/core/layer.rs
  why: Layer enum with storage_path() for file context during merge
  section: Lines 144-211 for storage_path() implementation
  critical: Understanding layer precedence for merge order

- file: /home/dustin/projects/jin-glm-doover/src/merge/mod.rs
  why: Module must export MergeValue after creation
  section: File shows current module structure (currently placeholder)
  gotcha: Need to add pub mod value; and pub use value::MergeValue;

- file: /home/dustin/projects/jin-glm-doover/Cargo.toml
  why: Verify dependencies are available
  section: Lines 19-34 for dependencies
  critical: serde_json (1.0), serde_yaml_ng (0.9), toml (0.9), configparser (0.4), indexmap (2.7)

# RESEARCH DOCUMENTS - Created for this PRP

- docfile: /home/dustin/projects/jin-glm-doover/plan/P2M1T1/research/rust_enum_patterns.md
  why: Enum design patterns for recursive value types
  section: Lines 1-90 for serde_json::Value pattern analysis, Lines 248-330 for merge operation examples
  critical: Shows exact enum structure, derive macros, recursive patterns

- docfile: /home/dustin/projects/jin-glm-doover/plan/P2M1T1/research/serde_value_types.md
  why: Complete API documentation for serde_json, serde_yaml_ng, toml
  section: Lines 12-106 for serde_json API, Lines 108-168 for serde_yaml_ng API, Lines 174-236 for toml API
  critical: Parsing functions, Value type conversions, merge patterns

- docfile: /home/dustin/projects/jin-glm-doover/plan/P2M1T1/research/ini_merge_patterns.md
  why: configparser crate patterns for INI handling
  section: Lines 7-86 for configparser API, Lines 838-901 for INI parser integration
  critical: INI to structured value conversion patterns

- docfile: /home/dustin/projects/jin-glm-doover/plan/P2M1T1/research/similar_text_merge.md
  why: 3-way text merge using similar crate (for future P2.M4)
  section: Lines 1-88 for basic 3-way merge patterns
  critical: Text merge is separate task, this PRP focuses on structured values

# EXTERNAL - Crate Documentation

- url: https://docs.rs/serde_json/latest/serde_json/enum.Value.html
  why: serde_json::Value reference - pattern to follow for MergeValue
  critical: Shows enum variants, recursive structure, derive macros

- url: https://docs.rs/indexmap/latest/indexmap/
  why: IndexMap for order-preserving objects
  critical: Use IndexMap<String, MergeValue> instead of HashMap

- url: https://docs.rs/serde_yaml_ng/latest/serde_yaml_ng/
  why: serde_yaml_ng Value type and parsing
  critical: YAML-specific handling for Mapping vs Sequence

- url: https://docs.rs/toml/latest/toml/
  why: toml Value type and parsing
  critical: TOML-specific handling for Table vs Array
```

### Current Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover
├── Cargo.toml                      # Has all required dependencies
├── PRD.md                          # Merge strategy specification
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── core/
│   │   ├── mod.rs                  # Exports error, layer, config
│   │   ├── error.rs                # Has parse error variants
│   │   ├── layer.rs                # Layer enum with precedence
│   │   └── config.rs
│   ├── merge/
│   │   └── mod.rs                  # Currently placeholder - needs to export MergeValue
│   ├── git/
│   ├── cli/
│   ├── commands/
│   ├── commit/
│   ├── staging/
│   └── workspace/
└── tests/
    └── integration_test.rs
```

### Desired Codebase Tree with Files to be Added

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   └── merge/
│       ├── mod.rs                  # MODIFY: Add pub mod value; pub use value::MergeValue;
│       └── value.rs                # CREATE: MergeValue enum implementation
└── tests/
    └── merge/
        └── value_test.rs           # CREATE: Unit tests for MergeValue
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: Use IndexMap, not HashMap, for object/map variant
// IndexMap preserves insertion order, which is required by PRD
use indexmap::IndexMap;
// Good: Object(IndexMap<String, MergeValue>)
// Bad: Object(HashMap<String, MergeValue>)

// CRITICAL: null values DELETE keys during merge (PRD §11.1)
// If higher layer has null for a key, remove that key from result
// This is different from typical JSON merge behavior

// CRITICAL: Array merging - "Higher layer replaces" (PRD §11.1)
// Unkeyed arrays do NOT concatenate - higher layer's array replaces lower's
// Only keyed arrays (with id/name) merge by those keys (future enhancement)

// CRITICAL: Serde YAML NG is different from serde_yaml
// We use serde_yaml_ng, which has slightly different API
// Value type is different (Mapping vs Object)

// CRITICAL: configparser INI format maps to nested objects
// INI sections become top-level keys
// INI key-value pairs become nested objects
// Example: [database] host=localhost -> {"database": {"host": "localhost"}}

// CRITICAL: Number handling differs across formats
// JSON: Single Number type (can be i64 or f64)
// YAML: Integer or Float variants
// TOML: Separate Integer, Float variants
// MergeValue should normalize to Integer(i64) and Float(f64)

// CRITICAL: Error conversion pattern
// All parse errors already have JinError variants with #[from]
// Use ? operator for automatic conversion:
//   let json: serde_json::Value = serde_json::from_str(s)?;
//   // JinError::JsonParse is automatically used on failure

// CRITICAL: Recursive enum does NOT need Box in this case
// Vec<MergeValue> and IndexMap<String, MergeValue> are size-known
// Follow serde_json::Value pattern - direct storage, no Box

// PATTERN: Follow error.rs enum structure:
// - Group variants with comment dividers (// ===== ===== =====)
// - Use #[non_exhaustive] for public enums
// - Implement helper methods in impl block after enum definition
// - Add comprehensive doc comments

// PATTERN: Naming conventions from error.rs and layer.rs:
// - Enum: PascalCase (MergeValue, not merge_value)
// - Variants: PascalCase (Object, Array, String, not object, array, string)
// - Methods: snake_case (from_json, merge, not fromJson, merge)
// - File name: snake_case (value.rs, is already correct)

// GOTCHA: TOML datetime handling
// TOML has a Datetime variant that JSON/YAML don't have
// For MergeValue, serialize TOML datetimes as strings
// Parse them as String variant

// GOTCHA: INI file format limitations
// INI has no native array type - comma-separated values are strings
// INI has no nesting beyond sections
// Parser should convert comma-separated strings to String, not Array

// FUTURE: Text merge uses similar crate (P2.M4 task)
// For this PRP, Text variant is a simple placeholder
// Full 3-way text merge implementation is a separate task
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
/// Unified value type for merge operations across multiple formats.
///
/// `MergeValue` represents any value that can appear in Jin's supported
/// configuration formats (JSON, YAML, TOML, INI) and provides deep merge
/// operations following PRD §11.1 rules.
///
/// # Merge Rules
///
/// - **Objects (Maps)**: Deep key merge - keys from higher layers override
/// - **Arrays**: Higher layer replaces (unkeyed), merge by id/name (keyed, future)
/// - **Null**: Deletes key from result
/// - **Primitives**: Higher layer replaces
///
/// # Variants
///
/// - `Null`: Represents null/nil values - deletes keys during merge
/// - `Boolean`: true or false
/// - `Integer`: Signed 64-bit integers
/// - `Float`: IEEE 754 double precision
/// - `String`: Text values
/// - `Array`: Ordered list of values (uses Vec for stack storage)
/// - `Object`: Key-value map (uses IndexMap for order preservation)
///
/// # Examples
///
/// ```ignore
/// use jin_glm::merge::value::MergeValue;
///
/// // Parse JSON
/// let json = r#"{"name": "jin", "version": 1}"#;
/// let value = MergeValue::from_json(json)?;
///
/// // Deep merge
/// let base = MergeValue::from_json(r#"{"a": {"x": 1}}"#)?;
/// let override = MergeValue::from_json(r#"{"a": {"y": 2}}"#)?;
/// let merged = base.merge(&override)?;
/// // Result: {"a": {"x": 1, "y": 2}}
///
/// // Null deletes keys
/// let base = MergeValue::from_json(r#"{"a": 1, "b": 2}"#)?;
/// let override = MergeValue::from_json(r#"{"a": null}"#)?;
/// let merged = base.merge(&override)?;
/// // Result: {"b": 2}
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MergeValue {
    // ===== Primitive Variants =====
    /// Null value - deletes keys during merge operations
    Null,

    /// Boolean value (true or false)
    Boolean(bool),

    /// Signed 64-bit integer
    Integer(i64),

    /// IEEE 754 double-precision floating point
    Float(f64),

    /// UTF-8 string value
    String(String),

    // ===== Collection Variants =====
    /// Ordered list of values
    /// During merge: higher layer replaces (unkeyed arrays)
    Array(Vec<MergeValue>),

    /// Key-value map with order preservation
    /// During merge: deep key merge, null deletes keys
    Object(IndexMap<String, MergeValue>),
}

// Type alias for convenience
pub type ObjectMap = IndexMap<String, MergeValue>;
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: CREATE src/merge/value.rs
  - IMPLEMENT: MergeValue enum with all 7 variants
  - FOLLOW pattern: serde_json::Value from rust_enum_patterns.md research
  - VARIANTS (in order):
    * Null
    * Boolean(bool)
    * Integer(i64)
    * Float(f64)
    * String(String)
    * Array(Vec<MergeValue>)
    * Object(IndexMap<String, MergeValue>)
  - DERIVES: Debug, Clone, PartialEq, Serialize, Deserialize, #[non_exhaustive]
  - IMPORTS:
    * use serde::{Deserialize, Serialize}
    * use indexmap::IndexMap
  - PLACEMENT: New file src/merge/value.rs

Task 2: IMPLEMENT from_json() parsing function
  - IMPLEMENT: pub fn from_json(input: &str) -> Result<Self>
  - PATTERN: Parse to serde_json::Value, convert to MergeValue
  - CONVERSION RULES:
    * Value::Null -> MergeValue::Null
    * Value::Bool(b) -> MergeValue::Boolean(b)
    * Value::Number(n) -> MergeValue::Integer(n.as_i64()) or Float(n.as_f64())
    * Value::String(s) -> MergeValue::String(s)
    * Value::Array(arr) -> recursive conversion
    * Value::Object(obj) -> recursive conversion to IndexMap
  - ERROR HANDLING: Use ? operator - JinError::JsonParse auto-converts
  - CODE TEMPLATE from serde_value_types.md lines 337-369
  - PLACEMENT: impl MergeValue block

Task 3: IMPLEMENT from_yaml() parsing function
  - IMPLEMENT: pub fn from_yaml(input: &str) -> Result<Self>
  - PATTERN: Parse to serde_yaml_ng::Value, convert to MergeValue
  - CONVERSION RULES: Same as JSON but handle serde_yaml_ng types
    * Value::Null -> MergeValue::Null
    * Value::Bool(b) -> MergeValue::Boolean(b)
    * Value::Number(n) -> Integer or Float based on type
    * Value::String(s) -> MergeValue::String(s)
    * Value::Sequence(seq) -> MergeValue::Array(recursive)
    * Value::Mapping(map) -> MergeValue::Object(recursive to IndexMap)
  - GOTCHA: serde_yaml_ng uses "Mapping" not "Object"
  - ERROR HANDLING: Use ? operator - JinError::YamlParse auto-converts
  - PLACEMENT: impl MergeValue block

Task 4: IMPLEMENT from_toml() parsing function
  - IMPLEMENT: pub fn from_toml(input: &str) -> Result<Self>
  - PATTERN: Parse to toml::Value, convert to MergeValue
  - CONVERSION RULES:
    * Value::String -> MergeValue::String
    * Value::Integer -> MergeValue::Integer
    * Value::Float -> MergeValue::Float
    * Value::Boolean -> MergeValue::Boolean
    * Value::Datetime -> MergeValue::String (serialize as ISO string)
    * Value::Array -> MergeValue::Array(recursive)
    * Value::Table -> MergeValue::Object(recursive to IndexMap)
  - GOTCHA: TOML Datetime has no JSON equivalent - convert to String
  - ERROR HANDLING: Use ? operator - JinError::TomlParse auto-converts
  - PLACEMENT: impl MergeValue block

Task 5: IMPLEMENT from_ini() parsing function
  - IMPLEMENT: pub fn from_ini(input: &str) -> Result<Self>
  - PATTERN: Parse with configparser, convert nested structure to MergeValue
  - STRUCTURE MAPPING:
    * INI sections become top-level Object keys
    * INI key-value pairs become nested Object entries
    * All values initially String (type coercion on access)
  - ALGORITHM:
    1. Parse INI with Ini::new_from_str(input)
    2. Create outer IndexMap for sections
    3. For each section, create inner IndexMap for keys
    4. For each key-value, add String variant to inner map
    5. Return Object(outer_map)
  - ERROR HANDLING: Convert configparser errors to JinError::IniParse
  - CODE TEMPLATE from ini_merge_patterns.md lines 838-901
  - PLACEMENT: impl MergeValue block

Task 6: IMPLEMENT merge() method - deep merge algorithm
  - IMPLEMENT: pub fn merge(&self, other: &MergeValue) -> Result<MergeValue>
  - ALGORITHM (PRD §11.1 rules):
    1. If other is Null -> return Null (key deletion)
    2. If both are Object -> deep merge keys recursively
    3. If both are Array -> return other (higher replaces)
    4. Otherwise -> return other (primitive replacement)
  - OBJECT MERGE:
    * Start with clone of self
    * For each (key, other_value) in other:
      - If key exists in self AND both are Object -> recursive merge
      - Otherwise -> insert/replace key with other_value
      - If other_value is Null -> remove key from result
  - NULL HANDLING:
    * When merging Object with Object, null values delete keys
    * This is the special "delete key" behavior from PRD
  - ERROR HANDLING: Return JinError::MergeConflict on type mismatches
  - CODE TEMPLATE from rust_enum_patterns.md lines 288-330
  - PLACEMENT: impl MergeValue block

Task 7: IMPLEMENT helper methods
  - IMPLEMENT: pub fn is_null(&self) -> bool
  - IMPLEMENT: pub fn is_object(&self) -> bool
  - IMPLEMENT: pub fn is_array(&self) -> bool
  - IMPLEMENT: pub fn as_object(&self) -> Option<&ObjectMap>
  - IMPLEMENT: pub fn as_array(&self) -> Option<&[MergeValue]>
  - IMPLEMENT: pub fn as_str(&self) -> Option<&str>
  - IMPLEMENT: pub fn as_i64(&self) -> Option<i64>
  - IMPLEMENT: pub fn as_bool(&self) -> Option<bool>
  - PATTERN: Match on self, return Some(value) or None
  - PLACEMENT: impl MergeValue block

Task 8: IMPLEMENT Default trait
  - IMPLEMENT: impl Default for MergeValue
  - RETURN: MergeValue::Null as default
  - PLACEMENT: After enum definition

Task 9: IMPLEMENT From conversions for primitives
  - IMPLEMENT: impl From<bool> for MergeValue
  - IMPLEMENT: impl From<i64> for MergeValue
  - IMPLEMENT: impl From<f64> for MergeValue
  - IMPLEMENT: impl From<String> for MergeValue
  - IMPLEMENT: impl From<&str> for MergeValue
  - PATTERN: Return appropriate variant
  - PLACEMENT: After impl MergeValue block

Task 10: MODIFY src/merge/mod.rs
  - ADD: pub mod value;
  - ADD: pub use value::MergeValue;
  - PRESERVE: Any existing content or comments
  - FINAL FILE:
    pub mod value;
    pub use value::MergeValue;
  - PLACEMENT: src/merge/mod.rs
  - DEPENDENCIES: Task 1 (value.rs must exist)

Task 11: CREATE tests/merge/value_test.rs
  - IMPLEMENT: Unit tests for all MergeValue methods
  - TESTS:
    * test_enum_variants_create() - verify all variants can be created
    * test_from_json_valid() - parse JSON string
    * test_from_json_invalid() - handle parse errors
    * test_from_yaml_valid() - parse YAML string
    * test_from_toml_valid() - parse TOML string
    * test_from_ini_valid() - parse INI string
    * test_merge_objects_deep() - recursive object merge
    * test_merge_null_deletes_key() - null removes keys
    * test_merge_arrays_replace() - higher array replaces
    * test_merge_primitives_replace() - higher primitive wins
    * test_merge_empty_with_object() - empty + object = object
    * test_helper_methods() - test all as_* and is_* methods
    * test_from_conversions() - test From impls
  - FOLLOW: Pattern from error.rs test structure (lines 337-475)
  - USE: Result<()> return for tests
  - PLACEMENT: tests/merge/value_test.rs (create tests/merge/ directory first)
  - DEPENDENCIES: Tasks 1-9
```

### Implementation Patterns & Key Details

```rust
// ===== ENUM DEFINITION PATTERN =====
// Follow serde_json::Value structure with IndexMap for order preservation
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use crate::core::error::{JinError, Result};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum MergeValue {
    // ===== Primitive Variants =====
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),

    // ===== Collection Variants =====
    Array(Vec<MergeValue>),
    Object(IndexMap<String, MergeValue>),
}

// ===== TYPE ALIAS FOR CLARITY =====
pub type ObjectMap = IndexMap<String, MergeValue>;

// ===== FROM JSON PATTERN =====
impl MergeValue {
    pub fn from_json(input: &str) -> Result<Self> {
        use serde_json::Value as Json;

        let json: Json = serde_json::from_str(input)?;

        // Convert serde_json::Value to MergeValue
        match json {
            Json::Null => Ok(MergeValue::Null),
            Json::Bool(b) => Ok(MergeValue::Boolean(b)),
            Json::Number(n) => {
                if n.is_i64() {
                    Ok(MergeValue::Integer(n.as_i64().unwrap()))
                } else if n.is_f64() {
                    Ok(MergeValue::Float(n.as_f64().unwrap()))
                } else {
                    Err(JinError::Message("Unsupported number type".to_string()))
                }
            }
            Json::String(s) => Ok(MergeValue::String(s)),
            Json::Array(arr) => {
                let converted: Result<Vec<_>> = arr
                    .into_iter()
                    .map(|v| -> Result<MergeValue> {
                        Ok(MergeValue::from_json(&serde_json::to_string(&v)?)?)
                    })
                    .collect();
                Ok(MergeValue::Array(converted?))
            }
            Json::Object(obj) => {
                let mut map = IndexMap::new();
                for (k, v) in obj {
                    map.insert(k, MergeValue::from_json(&serde_json::to_string(&v)?)?);
                }
                Ok(MergeValue::Object(map))
            }
        }
    }
}

// ===== FROM YAML PATTERN =====
impl MergeValue {
    pub fn from_yaml(input: &str) -> Result<Self> {
        use serde_yaml_ng::Value as Yaml;

        // Parse as intermediate JSON-like format
        let yaml: Yaml = serde_yaml_ng::from_str(input)?;

        // Convert to MergeValue (similar pattern to JSON)
        match yaml {
            Yaml::Null => Ok(MergeValue::Null),
            Yaml::Bool(b) => Ok(MergeValue::Boolean(b)),
            Yaml::Number(n) => {
                // Try integer first, then float
                if let Some(i) = n.as_i64() {
                    Ok(MergeValue::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(MergeValue::Float(f))
                } else {
                    Err(JinError::Message("Unsupported number type".to_string()))
                }
            }
            Yaml::String(s) => Ok(MergeValue::String(s)),
            Yaml::Sequence(seq) => {
                let converted: Result<Vec<_>> = seq
                    .into_iter()
                    .map(|v| -> Result<MergeValue> {
                        // Convert each value - simplified approach
                        Ok(MergeValue::from_yaml(&serde_yaml_ng::to_string(&v)?)?)
                    })
                    .collect();
                Ok(MergeValue::Array(converted?))
            }
            Yaml::Mapping(map) => {
                let mut index_map = IndexMap::new();
                for (k, v) in map {
                    if let Some(key_str) = k.as_str() {
                        let value = MergeValue::from_yaml(&serde_yaml_ng::to_string(&v)?)?;
                        index_map.insert(key_str.to_string(), value);
                    }
                }
                Ok(MergeValue::Object(index_map))
            }
            // Handle other YAML-specific variants by converting to string
            _ => Ok(MergeValue::String(serde_yaml_ng::to_string(&yaml)?)),
        }
    }
}

// ===== FROM TOML PATTERN =====
impl MergeValue {
    pub fn from_toml(input: &str) -> Result<Self> {
        use toml::Value as Toml;

        let toml: Toml = toml::from_str(input)?;

        match toml {
            Toml::String(s) => Ok(MergeValue::String(s)),
            Toml::Integer(i) => Ok(MergeValue::Integer(i)),
            Toml::Float(f) => Ok(MergeValue::Float(f)),
            Toml::Boolean(b) => Ok(MergeValue::Boolean(b)),
            Toml::Datetime(dt) => Ok(MergeValue::String(dt.to_string())),
            Toml::Array(arr) => {
                let converted: Result<Vec<_>> = arr
                    .into_iter()
                    .map(|v| -> Result<MergeValue> {
                        Ok(MergeValue::from_toml(&toml::to_string(&v)?)?)
                    })
                    .collect();
                Ok(MergeValue::Array(converted?))
            }
            Toml::Table(table) => {
                let mut map = IndexMap::new();
                for (k, v) in table {
                    map.insert(k, MergeValue::from_toml(&toml::to_string(&v)?)?);
                }
                Ok(MergeValue::Object(map))
            }
        }
    }
}

// ===== FROM INI PATTERN =====
impl MergeValue {
    pub fn from_ini(input: &str) -> Result<Self> {
        use configparser::ini::Ini;

        let config = Ini::new_from_str(input)
            .map_err(|e| JinError::IniParse(e.to_string()))?;

        let mut outer_map = IndexMap::new();

        // Get all sections
        let sections = config.sections().map_err(|e| JinError::IniParse(e.to_string()))?;

        for section in sections {
            let mut inner_map = IndexMap::new();

            // Get all keys in this section
            if let Some(section_map) = config.section(Some(&section)) {
                for (key, value_opt) in section_map {
                    if let Some(value) = value_opt {
                        inner_map.insert(key.clone(), MergeValue::String(value));
                    }
                }
            }

            outer_map.insert(section, MergeValue::Object(inner_map));
        }

        Ok(MergeValue::Object(outer_map))
    }
}

// ===== MERGE METHOD PATTERN =====
// Implements PRD §11.1 rules: deep key merge, null deletes key, higher replaces
impl MergeValue {
    pub fn merge(&self, other: &MergeValue) -> Result<Self> {
        match (self, other) {
            // Rule: null deletes key (PRD §11.1)
            (_, MergeValue::Null) => Ok(MergeValue::Null),

            // Rule: Deep key merge for objects (PRD §11.1)
            (MergeValue::Object(base_map), MergeValue::Object(override_map)) => {
                let mut merged = base_map.clone();

                for (key, override_value) in override_map {
                    if let Some(base_value) = merged.get(key) {
                        // Recursively merge nested objects
                        match (base_value, override_value) {
                            (MergeValue::Object(_), MergeValue::Object(_)) => {
                                let merged_value = base_value.merge(override_value)?;
                                merged.insert(key.clone(), merged_value);
                            }
                            _ => {
                                // Null deletes key
                                if matches!(override_value, MergeValue::Null) {
                                    merged.remove(key);
                                } else {
                                    merged.insert(key.clone(), override_value.clone());
                                }
                            }
                        }
                    } else {
                        // Add new key (unless it's null)
                        if !matches!(override_value, MergeValue::Null) {
                            merged.insert(key.clone(), override_value.clone());
                        }
                    }
                }

                Ok(MergeValue::Object(merged))
            }

            // Rule: Arrays - higher layer replaces (PRD §11.1)
            (MergeValue::Array(_), MergeValue::Array(_)) => {
                Ok(other.clone())
            }

            // Rule: Primitives - higher layer replaces
            (_, _) => Ok(other.clone()),
        }
    }
}

// ===== HELPER METHODS PATTERN =====
impl MergeValue {
    pub fn is_null(&self) -> bool {
        matches!(self, MergeValue::Null)
    }

    pub fn is_object(&self) -> bool {
        matches!(self, MergeValue::Object(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, MergeValue::Array(_))
    }

    pub fn as_object(&self) -> Option<&ObjectMap> {
        match self {
            MergeValue::Object(map) => Some(map),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[MergeValue]> {
        match self {
            MergeValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            MergeValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            MergeValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            MergeValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

// ===== DEFAULT TRAIT PATTERN =====
impl Default for MergeValue {
    fn default() -> Self {
        MergeValue::Null
    }
}

// ===== FROM CONVERSIONS PATTERN =====
impl From<bool> for MergeValue {
    fn from(value: bool) -> Self {
        MergeValue::Boolean(value)
    }
}

impl From<i64> for MergeValue {
    fn from(value: i64) -> Self {
        MergeValue::Integer(value)
    }
}

impl From<f64> for MergeValue {
    fn from(value: f64) -> Self {
        MergeValue::Float(value)
    }
}

impl From<String> for MergeValue {
    fn from(value: String) -> Self {
        MergeValue::String(value)
    }
}

impl From<&str> for MergeValue {
    fn from(value: &str) -> Self {
        MergeValue::String(value.to_string())
    }
}
```

### Integration Points

```yaml
ERROR_HANDLING:
  - use: src/core/error.rs
  - patterns:
    * JinError::JsonParse(#[from] serde_json::Error) - automatic
    * JinError::YamlParse(#[from] serde_yaml_ng::Error) - automatic
    * JinError::TomlParse(#[from] toml::de::Error) - automatic
    * JinError::IniParse(String) - manual conversion for configparser
    * JinError::MergeConflict { file_path } - for merge conflicts (future)

MODULE_EXPORTS:
  - modify: src/merge/mod.rs
  - add: pub mod value;
  - add: pub use value::MergeValue;
  - result: crate::merge::MergeValue is accessible

TYPE_IMPORTS:
  - use: indexmap::IndexMap from Cargo.toml dependency
  - use: serde::{Deserialize, Serialize} for derives
  - use: crate::core::error::{JinError, Result}

TESTING:
  - create: tests/merge/value_test.rs
  - use: assert_eq!, assert! macros for assertions
  - pattern: Result<()> return for tests

FUTURE_INTEGRATION:
  - P2.M2: Format Parsers will use from_* functions
  - P2.M3: Deep Merge Algorithm will use merge() method
  - P2.M4: Text Merge will add Text variant or separate handling
  - P3: Staging System will use MergeValue for file comparisons
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after creating value.rs - fix before proceeding
cargo check --package jin                    # Check compilation
cargo clippy --package jin -- -D warnings    # Lint with warnings as errors
cargo fmt --check                            # Verify formatting

# Format the code
cargo fmt

# Expected: Zero errors. If errors exist, READ output and fix before proceeding.

# Common issues to watch for:
# - "unused_imports" -> remove unused imports (use, IndexMap, serde)
# - "dead_code" -> public methods are used by tests, mark pub
# - Pattern matching errors -> ensure all enum variants are covered
# - Type mismatch errors -> check serde_json Value vs MergeValue conversions
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test MergeValue module specifically
cargo test --package jin --lib merge::value --verbose

# Run all merge module tests
cargo test --package jin --lib merge:: --verbose

# Run with output
cargo test --package jin --lib merge::value -- --nocapture

# Expected: All tests pass. Look for:
# - test_from_json_valid: JSON parsing works
# - test_from_yaml_valid: YAML parsing works
# - test_from_toml_valid: TOML parsing works
# - test_from_ini_valid: INI parsing works
# - test_merge_objects_deep: Recursive object merge
# - test_merge_null_deletes_key: Null removes keys
# - test_merge_arrays_replace: Higher array replaces
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the full project
cargo build --release

# Test format detection and parsing
# Add a temporary main() test or integration test
cargo test --package jin test_merge_value_integration -- --exact

# Verify merge behavior matches PRD rules
cargo test --package jin test_prd_merge_rules -- --exact

# Expected: Clean build, all parsing works, merge follows PRD rules
```

### Level 4: Domain-Specific Validation

```bash
# Verify order preservation (IndexMap usage)
cargo test --package jin test_order_preservation -- --exact
# Asserts: Keys maintain insertion order from highest layer

# Verify null deletes key behavior
cargo test --package jin test_null_deletes_key -- --exact
# Asserts: {"a": 1, "b": 2} + {"a": null} = {"b": 2}

# Verify array replacement (not concatenation)
cargo test --package jin test_array_replacement -- --exact
# Asserts: [1, 2] + [3, 4] = [3, 4], not [1, 2, 3, 4]

# Verify deep object merge
cargo test --package jin test_deep_object_merge -- --exact
# Asserts: {"a": {"x": 1}} + {"a": {"y": 2}} = {"a": {"x": 1, "y": 2}}

# Expected: All PRD §11.1 rules correctly implemented
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --package jin --lib`
- [ ] No linting errors: `cargo clippy --package jin -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] Documentation comments on all public methods
- [ ] All enum variants have #[doc] comments

### Feature Validation

- [ ] `MergeValue` enum with all 7 variants compiles
- [ ] `from_json()` parses JSON strings correctly
- [ ] `from_yaml()` parses YAML strings correctly
- [ ] `from_toml()` parses TOML strings correctly
- [ ] `from_ini()` parses INI strings correctly
- [ ] `merge()` implements deep key merge
- [ ] `null` values delete keys during merge
- [ ] Arrays use "higher replaces" strategy
- [ ] Order preservation with `IndexMap`
- [ ] Helper methods (as_*, is_*) work correctly

### Code Quality Validation

- [ ] Follows existing codebase patterns (error.rs enum structure)
- [ ] File placement matches desired tree structure
- [ ] Module exported from `src/merge/mod.rs`
- [ ] No #[allow] attributes except for justified cases
- [ ] All public methods have doc comments
- [ ] Test coverage for all public methods

### Documentation & Deployment

- [ ] Module-level doc comment explains MergeValue purpose
- [ ] Each variant has doc comment explaining merge behavior
- [ ] Complex methods have usage examples in doc comments
- [ ] Gotchas documented (null deletes keys, array replacement)

---

## Anti-Patterns to Avoid

- ❌ Don't use `HashMap` instead of `IndexMap` - order must be preserved
- ❌ Don't concatenate arrays during merge - PRD specifies "higher replaces"
- ❌ Don't treat null as regular value - null must delete keys
- ❌ Don't use `Box<MergeValue>` in Vec/Map - direct storage is fine
- ❌ Don't skip recursive merge for nested objects - must be deep
- ❌ Don't forget to handle all serde_yaml_ng variants (Mapping not Object)
- ❌ Don't convert INI arrays to MergeValue::Array - keep as comma-separated strings
- ❌ Don't skip the `#[non_exhaustive]` attribute on public enums
- ❌ Don't use different number types (use i64 and f64 consistently)
- ❌ Don't ignore TOML datetime - convert to String

---

## Appendix: Quick Reference

### MergeValue API Summary

```rust
// Constructors (from formats)
pub fn from_json(input: &str) -> Result<Self>
pub fn from_yaml(input: &str) -> Result<Self>
pub fn from_toml(input: &str) -> Result<Self>
pub fn from_ini(input: &str) -> Result<Self>

// Merge operation
pub fn merge(&self, other: &MergeValue) -> Result<MergeValue>

// Type checks
pub fn is_null(&self) -> bool
pub fn is_object(&self) -> bool
pub fn is_array(&self) -> bool

// Type conversions (return None if wrong type)
pub fn as_object(&self) -> Option<&ObjectMap>
pub fn as_array(&self) -> Option<&[MergeValue]>
pub fn as_str(&self) -> Option<&str>
pub fn as_i64(&self) -> Option<i64>
pub fn as_bool(&self) -> Option<bool>
```

### Merge Rules Reference (PRD §11.1)

| Left Type | Right Type | Result |
|-----------|------------|--------|
| Any | `Null` | `Null` (deletes key) |
| `Object` | `Object` | Deep merge (recursive) |
| `Array` | `Array` | Right (higher replaces) |
| Any | Any | Right (higher replaces) |

### Format-Specific Notes

| Format | Special Handling |
|--------|------------------|
| JSON | Number can be i64 or f64 |
| YAML | "Mapping" not "Object", "Sequence" not "Array" |
| TOML | Datetime -> String conversion |
| INI | Sections -> nested Objects, all values -> String |

---

**PRP Version**: 1.0
**Last Updated**: 2025-12-26
**Confidence Score**: 9/10 - High confidence in one-pass implementation success
