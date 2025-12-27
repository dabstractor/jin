# PRP: P2.M2 - Format Parsers

---

## Goal

**Feature Goal**: Implement complete format parsers for JSON, YAML, TOML, and INI that convert bidirectionally with `MergeValue`, enabling Jin's deterministic merge engine to work uniformly across all configuration file formats.

**Deliverable**: An enhanced `src/merge/value.rs` module and new `src/merge/format.rs` module providing:
1. Bidirectional conversion between `MergeValue` and `serde_yaml::Value`
2. Bidirectional conversion between `MergeValue` and `toml::Value` (with TOML constraint handling)
3. Bidirectional conversion between `MergeValue` and INI format via `rust-ini`
4. Parsing helper methods: `from_json()`, `from_yaml()`, `from_toml()`, `from_ini()`
5. Serialization helper methods: `to_json_string()`, `to_yaml_string()`, `to_toml_string()`, `to_ini_string()`
6. Format detection and auto-parsing via `from_file()` based on file extension
7. Comprehensive unit tests for all format conversions

**Success Definition**:
- All conversion tests pass: `cargo test merge::`
- JSON, YAML, and TOML roundtrips preserve data correctly
- INI files parse to nested MergeValue objects (sections → top-level keys)
- TOML-specific constraints (no null, homogeneous arrays) are handled with clear errors
- Format detection correctly identifies files by extension (.json, .yaml, .yml, .toml, .ini)
- All existing `deep_merge` tests continue to pass
- `cargo check && cargo clippy && cargo test` all pass with zero errors/warnings

---

## User Persona

**Target User**: Jin internals (merge engine, commit pipeline, layer orchestration)

**Use Case**: The format parsers are used by:
- `deep_merge()` to perform format-agnostic deep merging of configuration files
- Layer merge orchestration to combine configurations from multiple layers
- Format detection to automatically parse files based on extension
- Serialization to write merged configurations back to their original formats

**User Journey**: Users don't interact directly with format parsers - they experience seamless merging of their JSON, YAML, TOML, and INI configuration files without format-specific quirks affecting merge behavior.

**Pain Points Addressed**:
- Eliminates format-specific merge logic by normalizing to MergeValue representation
- Handles TOML constraints (no null, homogeneous arrays) gracefully with clear errors
- Handles INI's flat structure by mapping sections to nested objects
- Preserves key ordering (via IndexMap) for consistent, reproducible output

---

## Why

- **PRD Requirement**: Section 11.1 specifies "Deterministic structured merges" for JSON, YAML, TOML, and INI formats
- **Foundation for Merge Engine**: P2.M3 (Deep Merge Algorithm) depends on complete format conversions
- **Format Agnostic Processing**: Enables the same merge algorithm regardless of source format
- **Reversibility**: Section 11.1 requires merges to be "Deterministic and reversible"
- **Developer Tool Support**: INI format is used by `.editorconfig`, Git config, and many other tools

---

## What

### User-Visible Behavior

After this milestone:
```rust
// Parse any supported format
let json_val = MergeValue::from_json(json_str)?;
let yaml_val = MergeValue::from_yaml(yaml_str)?;
let toml_val = MergeValue::from_toml(toml_str)?;
let ini_val = MergeValue::from_ini(ini_str)?;

// Auto-detect format from file extension
let val = MergeValue::from_file("config.yaml")?;
let val = MergeValue::from_file(".editorconfig")?;  // INI format

// Deep merge works the same regardless of source format
let merged = deep_merge(yaml_val, toml_val)?;

// Serialize back to any format
let json_out = merged.to_json_string()?;
let yaml_out = merged.to_yaml_string()?;
let toml_out = merged.to_toml_string()?;  // Errors on null values
let ini_out = merged.to_ini_string()?;    // Errors on nested objects
```

### Technical Requirements

1. **YAML Conversion**: Full bidirectional conversion with `serde_yaml::Value`
2. **TOML Conversion**: Bidirectional conversion with proper constraint handling
3. **INI Conversion**: Sections map to top-level object keys, key-value pairs to nested objects
4. **Null Handling**: TOML uses `TryFrom` to error; INI skips null values
5. **Number Handling**: Preserve integer vs float distinction; INI stores all values as strings
6. **Key Order**: Use `IndexMap` throughout to preserve insertion order
7. **Helper Functions**: Parsing and serialization utilities for each format

### Success Criteria

- [ ] `From<serde_yaml::Value> for MergeValue` implemented
- [ ] `From<MergeValue> for serde_yaml::Value` implemented
- [ ] `From<toml::Value> for MergeValue` implemented
- [ ] `TryFrom<MergeValue> for toml::Value` implemented (handles null constraint)
- [ ] INI parsing converts sections to nested objects
- [ ] INI serialization converts nested objects to sections
- [ ] `MergeValue::from_json()`, `from_yaml()`, `from_toml()`, `from_ini()` helper methods
- [ ] `MergeValue::to_json_string()`, `to_yaml_string()`, `to_toml_string()`, `to_ini_string()` methods
- [ ] `MergeValue::from_file()` with format detection by extension
- [ ] All roundtrip tests pass for each format
- [ ] Edge cases documented and tested (TOML nulls, INI nesting limits, number types)
- [ ] Existing `deep_merge` tests continue to pass

---

## All Needed Context

### Context Completeness Check

_This PRP provides everything needed to implement complete format parsers, including exact type mappings, conversion patterns, edge case handling, and comprehensive test cases. An AI agent with access to this PRP and the codebase can implement the feature in one pass._

### Documentation & References

```yaml
# MUST READ - Core Implementation Context

- file: src/merge/value.rs
  why: Current MergeValue implementation with JSON conversion only
  critical: |
    - Already has: Null, Bool, Integer, Float, String, Array, Object variants
    - Already has: From<serde_json::Value> and From<MergeValue> for serde_json::Value
    - Uses IndexMap<String, MergeValue> for Object to preserve key order
    - Has is_null(), is_object(), is_array(), as_object(), as_str() helpers
    - NEED TO ADD: YAML, TOML, INI conversions and helper methods
  pattern: Follow existing From<serde_json::Value> implementation style

- file: src/merge/deep.rs
  why: Consumer of MergeValue - must remain compatible
  critical: |
    - deep_merge(base: MergeValue, overlay: MergeValue) -> Result<MergeValue>
    - Uses is_null(), as_object(), shift_remove() on IndexMap
    - Depends on MergeValue structure being stable
    - NO CHANGES NEEDED - just ensure compatibility

- file: src/merge/mod.rs
  why: Module exports that need updating
  critical: |
    - Currently exports: deep_merge, merge_layers, text_merge, MergeValue
    - ADD: New format helper types/traits if needed
  pattern: Keep exports minimal and focused

- file: src/core/error.rs
  why: JinError::Parse variant for conversion errors
  pattern: |
    JinError::Parse { format: String, message: String }
    - Use for TOML null constraint violations
    - Use for INI nesting violations
    - Use for invalid format detection
    - Include line/column info when available

- file: Cargo.toml
  why: Current dependencies and what needs to be added
  critical: |
    ALREADY PRESENT:
    - serde_json = "1.0" ✓
    - serde_yaml = "0.9" ✓
    - toml = "0.8" ✓
    - indexmap = { version = "2.0", features = ["serde"] } ✓

    NEEDS TO BE ADDED:
    - rust-ini = "0.21"  # For INI file parsing

# EXTERNAL REFERENCES

- docfile: plan/P2M2/research/serde_yaml_api.md
  why: Complete serde_yaml::Value API reference
  critical: |
    - Value::Null, Bool, Number, String, Sequence, Mapping, Tagged variants
    - Number has as_i64(), as_f64() - try integer first
    - Mapping preserves insertion order (uses indexmap internally)
    - Tagged values: extract inner value with tagged.value

- docfile: plan/P2M2/research/toml_crate_api.md
  why: Complete toml::Value API reference
  critical: |
    - Value::String, Integer, Float, Boolean, Datetime, Array, Table
    - NO NULL VARIANT - must use TryFrom with error
    - Datetime: convert to string representation
    - Table is BTreeMap (alphabetical order, not insertion)
    - Arrays must be homogeneous - may error on heterogeneous

- docfile: plan/P2M2/research/rust_ini_crate.md
  why: Complete rust-ini API reference
  critical: |
    - Ini::load_from_str(content) for parsing
    - ini.section(Some("name")) returns Properties
    - All values are strings - no type information
    - General section (no header) via ini.general_section()
    - ini.with_section(Some("name")).set(key, value) for building

- docfile: plan/P2M2/research/ini_format_patterns.md
  why: INI format specification and edge cases
  critical: |
    - Sections become top-level keys in MergeValue::Object
    - Key-value pairs become nested MergeValue::Object values
    - General section (no header) merges into root object
    - Edge cases: equals in values, multiline, duplicate keys
```

### Current Codebase Tree (Relevant Files)

```bash
jin/
├── src/
│   ├── core/
│   │   └── error.rs          # JinError::Parse for format errors
│   └── merge/
│       ├── mod.rs            # Module exports (needs minor update)
│       ├── value.rs          # MergeValue enum (TO BE ENHANCED)
│       ├── deep.rs           # deep_merge() - consumer, no changes
│       ├── layer.rs          # Layer merge orchestration
│       └── text.rs           # 3-way text merge
├── Cargo.toml                # Add rust-ini dependency
└── tests/
    └── integration/
        └── cli_basic.rs      # CLI tests (unaffected)
```

### Desired Codebase Tree After P2.M2

```bash
jin/
├── src/
│   └── merge/
│       ├── mod.rs            # Updated exports
│       └── value.rs          # Enhanced with:
│           ├── MergeValue enum (existing, unchanged)
│           │
│           ├── // YAML Conversions
│           ├── From<serde_yaml::Value>       # NEW
│           ├── From<MergeValue> for yaml     # NEW
│           │
│           ├── // TOML Conversions
│           ├── From<toml::Value>             # NEW
│           ├── TryFrom<MergeValue> for toml  # NEW (handles null)
│           │
│           ├── // INI Conversions
│           ├── fn from_ini_internal(ini: &Ini) -> MergeValue    # NEW
│           ├── fn to_ini_internal(value: &MergeValue) -> Result<Ini>  # NEW
│           │
│           ├── impl MergeValue {
│           │   ├── from_json(s: &str)        # NEW helper
│           │   ├── from_yaml(s: &str)        # NEW helper
│           │   ├── from_toml(s: &str)        # NEW helper
│           │   ├── from_ini(s: &str)         # NEW helper
│           │   ├── from_file(path)           # NEW with auto-detect
│           │   ├── to_json_string()          # NEW helper
│           │   ├── to_json_string_compact()  # NEW helper
│           │   ├── to_yaml_string()          # NEW helper
│           │   ├── to_toml_string()          # NEW helper (TryFrom)
│           │   ├── to_ini_string()           # NEW helper
│           │   ├── is_scalar()               # NEW helper
│           │   ├── as_i64()                  # NEW helper
│           │   ├── as_f64()                  # NEW helper
│           │   ├── as_bool()                 # NEW helper
│           │   └── contains_null()           # NEW helper (recursive)
│           │   }
│           └── #[cfg(test)] mod tests        # Comprehensive tests
├── Cargo.toml                # rust-ini = "0.21" added
└── plan/
    └── P2M2/
        ├── PRP.md            # This file
        └── research/         # Research artifacts
            ├── serde_yaml_api.md
            ├── toml_crate_api.md
            ├── rust_ini_crate.md
            └── ini_format_patterns.md
```

### Known Gotchas & Library Quirks

```rust
// ============================================================
// CRITICAL: TOML has no null type
// ============================================================
// toml::Value has NO Null variant - when converting MergeValue::Null to TOML:
// - Use TryFrom and return Err(JinError::Parse { format: "TOML", ... })
// - This surfaces the error clearly to calling code
// - Caller can use contains_null() to check before conversion

impl TryFrom<MergeValue> for toml::Value {
    type Error = JinError;

    fn try_from(value: MergeValue) -> Result<Self, Self::Error> {
        match value {
            MergeValue::Null => Err(JinError::Parse {
                format: "TOML".to_string(),
                message: "TOML does not support null values".to_string(),
            }),
            // ... other cases
        }
    }
}

// ============================================================
// CRITICAL: TOML arrays must be homogeneous
// ============================================================
// All elements must be same type (all strings, all integers, etc.)
// deep_merge may create heterogeneous arrays
// TOML serialization may fail with clear error message

// ============================================================
// GOTCHA: Number type distinctions
// ============================================================
// - JSON: serde_json::Number (can be i64, u64, or f64)
// - YAML: serde_yaml::Number (same approach)
// - TOML: Separate Integer(i64) and Float(f64) variants
// - INI: All values are strings (no numeric types)
// MergeValue uses separate Integer(i64) and Float(f64) - matches TOML best
// When converting to INI, numbers become their string representation

// ============================================================
// GOTCHA: Key ordering preservation
// ============================================================
// - MergeValue uses IndexMap ✓ (insertion order)
// - serde_yaml::Mapping preserves order ✓
// - toml::Table is BTreeMap (alphabetical order) ✗
// - INI (rust-ini) preserves section and key order ✓
// When converting TO TOML, key order will change to alphabetical

// ============================================================
// PATTERN: Error handling for format conversion
// ============================================================
// Use JinError::Parse { format: "FORMAT_NAME", message: "..." }
// Include line/column info when available from parser errors

// ============================================================
// GOTCHA: YAML Mapping to IndexMap conversion
// ============================================================
// serde_yaml::Mapping is not IndexMap, but can be iterated in order
// Need to iterate and build IndexMap manually
// YAML keys can be non-strings - filter to only string keys

// ============================================================
// GOTCHA: YAML Tagged values
// ============================================================
// serde_yaml 0.9+ has Value::Tagged for custom tags like !include
// Extract the inner value and convert that:
serde_yaml::Value::Tagged(tagged) => MergeValue::from(tagged.value)

// ============================================================
// CRITICAL: INI format limitations
// ============================================================
// 1. Only supports 2 levels of nesting (section -> key-value)
// 2. All values are strings - no type information
// 3. Arrays are not supported - must be JSON-encoded or comma-separated
// 4. Null is not supported - skip null entries or error
// 5. General section (values before any [section]) maps to root level

// When serializing MergeValue to INI:
// - Top-level object keys become sections
// - Nested objects become key-value pairs
// - Deeply nested objects (>2 levels) should error
// - Non-string primitives convert to their string representation
// - Arrays should error or be JSON-encoded

// ============================================================
// PATTERN: INI section to MergeValue mapping
// ============================================================
// INI File:
// [database]
// host = localhost
// port = 5432
//
// Maps to:
// MergeValue::Object({
//     "database": MergeValue::Object({
//         "host": MergeValue::String("localhost"),
//         "port": MergeValue::String("5432"),
//     }),
// })

// ============================================================
// PATTERN: File format detection
// ============================================================
let extension = path.extension()
    .and_then(|e| e.to_str())
    .map(|e| e.to_lowercase());

match extension.as_deref() {
    Some("json") => Self::from_json(&content),
    Some("yaml") | Some("yml") => Self::from_yaml(&content),
    Some("toml") => Self::from_toml(&content),
    Some("ini") | Some("cfg") | Some("conf") => Self::from_ini(&content),
    // Special case: .editorconfig is INI format
    None if path.file_name() == Some(".editorconfig".as_ref()) => Self::from_ini(&content),
    Some(ext) => Err(JinError::Parse { format: ext, message: "Unsupported" }),
    None => Err(JinError::Parse { format: "unknown", message: "No extension" }),
}
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// ================== src/merge/value.rs ADDITIONS ==================

// Add to imports at top of file:
use crate::core::{JinError, Result};
use ini::Ini;
use std::path::Path;

// Existing MergeValue enum (UNCHANGED)
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
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: UPDATE Cargo.toml - Add rust-ini dependency
  - ADD: rust-ini = "0.21" to [dependencies]
  - VERIFY: cargo check passes
  - PLACEMENT: After toml dependency

Task 2: IMPLEMENT YAML conversions in src/merge/value.rs
  - ADD: From<serde_yaml::Value> for MergeValue
    - Handle: Null, Bool, Number, String, Sequence, Mapping, Tagged
    - Number: Try as_i64() first, then as_f64()
    - Tagged: Extract inner value with tagged.value
    - Mapping: Filter to string keys only, build IndexMap
  - ADD: From<MergeValue> for serde_yaml::Value
    - Handle all MergeValue variants
    - Float: Use serde_yaml::Number::from(f)
  - PLACEMENT: After existing From<serde_json::Value> implementation
  - FOLLOW pattern: Existing JSON conversion in same file

Task 3: IMPLEMENT TOML conversions in src/merge/value.rs
  - ADD: From<toml::Value> for MergeValue
    - Handle: String, Integer, Float, Boolean, Datetime, Array, Table
    - Datetime: Convert to MergeValue::String(dt.to_string())
  - ADD: TryFrom<MergeValue> for toml::Value (NOT From - must handle null)
    - MergeValue::Null -> return Err(JinError::Parse)
    - Recurse for Array and Object
    - Propagate errors from nested conversions
  - PLACEMENT: After YAML conversion implementations
  - FOLLOW pattern: TryFrom for fallible conversions

Task 4: IMPLEMENT INI conversions in src/merge/value.rs
  - ADD: fn from_ini_value(ini: &Ini) -> MergeValue
    - General section keys go to root level
    - Named sections become top-level keys with Object values
    - All values are MergeValue::String
  - ADD: fn to_ini_value(value: &MergeValue) -> Result<Ini>
    - Top-level Object keys become sections
    - Nested Object values become key-value pairs
    - Error on: null, arrays, deeper than 2-level nesting
    - Convert non-string primitives to string
  - PLACEMENT: After TOML conversion implementations
  - DOCUMENT: INI format limitations in doc comments

Task 5: IMPLEMENT parsing helper methods in impl MergeValue
  - ADD: pub fn from_json(s: &str) -> Result<Self>
    - Parse via serde_json::from_str, convert with From
    - Wrap errors in JinError::Parse { format: "JSON", ... }
  - ADD: pub fn from_yaml(s: &str) -> Result<Self>
    - Parse via serde_yaml::from_str, convert with From
    - Include line/col info in error if available
  - ADD: pub fn from_toml(s: &str) -> Result<Self>
    - Parse via toml::from_str, convert with From
  - ADD: pub fn from_ini(s: &str) -> Result<Self>
    - Parse via Ini::load_from_str, convert with from_ini_value
  - ADD: pub fn from_file(path: impl AsRef<Path>) -> Result<Self>
    - Read file content
    - Detect format by extension
    - Dispatch to appropriate from_* method
    - Handle special cases (.editorconfig)
  - PLACEMENT: In impl MergeValue block

Task 6: IMPLEMENT serialization helper methods in impl MergeValue
  - ADD: pub fn to_json_string(&self) -> Result<String>
    - Convert to serde_json::Value, use to_string_pretty
  - ADD: pub fn to_json_string_compact(&self) -> Result<String>
    - Convert to serde_json::Value, use to_string (no formatting)
  - ADD: pub fn to_yaml_string(&self) -> Result<String>
    - Convert to serde_yaml::Value, use to_string
  - ADD: pub fn to_toml_string(&self) -> Result<String>
    - Use TryInto for conversion (may fail on null)
    - Use toml::to_string_pretty
  - ADD: pub fn to_ini_string(&self) -> Result<String>
    - Use to_ini_value (may fail on invalid structure)
    - Use Ini::write_to for serialization
  - PLACEMENT: In impl MergeValue block after parsing helpers

Task 7: IMPLEMENT additional type-checking helpers in impl MergeValue
  - ADD: pub fn is_scalar(&self) -> bool
    - Returns true for Null, Bool, Integer, Float, String
  - ADD: pub fn as_i64(&self) -> Option<i64>
  - ADD: pub fn as_f64(&self) -> Option<f64>
    - For Integer, return as f64; for Float, return directly
  - ADD: pub fn as_bool(&self) -> Option<bool>
  - ADD: pub fn contains_null(&self) -> bool
    - Recursive check for any null values
    - Useful before TOML conversion
  - PLACEMENT: In impl MergeValue block with existing helpers

Task 8: IMPLEMENT comprehensive tests in #[cfg(test)] mod tests
  - YAML roundtrip tests (various data types, null, nested)
  - TOML roundtrip tests (no nulls, nested tables, arrays)
  - TOML null error tests (surface clear error)
  - INI roundtrip tests (sections, general section)
  - INI nesting error tests (too deep)
  - Cross-format tests (JSON -> MergeValue -> YAML -> TOML)
  - File parsing tests (with tempfile crate)
  - Edge cases (empty objects/arrays, special characters)
  - FOLLOW pattern: Existing test structure in value.rs

Task 9: UPDATE src/merge/mod.rs exports if needed
  - REVIEW: Check if any new types need exporting
  - PRESERVE: Existing exports (deep_merge, merge_layers, text_merge, MergeValue)
  - VERIFY: All tests pass after export changes
```

### Implementation Patterns & Key Details

```rust
// ================== YAML Conversion ==================

impl From<serde_yaml::Value> for MergeValue {
    fn from(value: serde_yaml::Value) -> Self {
        match value {
            serde_yaml::Value::Null => MergeValue::Null,
            serde_yaml::Value::Bool(b) => MergeValue::Bool(b),
            serde_yaml::Value::Number(n) => {
                // PATTERN: Try integer first to preserve integer semantics
                if let Some(i) = n.as_i64() {
                    MergeValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    MergeValue::Float(f)
                } else {
                    // Fallback for unsigned integers that don't fit i64
                    MergeValue::Float(n.as_f64().unwrap_or(0.0))
                }
            }
            serde_yaml::Value::String(s) => MergeValue::String(s),
            serde_yaml::Value::Sequence(seq) => {
                MergeValue::Array(seq.into_iter().map(MergeValue::from).collect())
            }
            serde_yaml::Value::Mapping(map) => {
                // PATTERN: Filter to string keys only
                let obj: IndexMap<String, MergeValue> = map
                    .into_iter()
                    .filter_map(|(k, v)| {
                        k.as_str().map(|s| (s.to_string(), MergeValue::from(v)))
                    })
                    .collect();
                MergeValue::Object(obj)
            }
            // PATTERN: Handle tagged values by extracting inner value
            serde_yaml::Value::Tagged(tagged) => MergeValue::from(tagged.value),
        }
    }
}

impl From<MergeValue> for serde_yaml::Value {
    fn from(value: MergeValue) -> Self {
        match value {
            MergeValue::Null => serde_yaml::Value::Null,
            MergeValue::Bool(b) => serde_yaml::Value::Bool(b),
            MergeValue::Integer(i) => serde_yaml::Value::Number(i.into()),
            MergeValue::Float(f) => {
                // GOTCHA: NaN/Infinity need special handling
                serde_yaml::Number::from(f)
                    .map(serde_yaml::Value::Number)
                    .unwrap_or(serde_yaml::Value::Null)
            }
            MergeValue::String(s) => serde_yaml::Value::String(s),
            MergeValue::Array(arr) => {
                serde_yaml::Value::Sequence(
                    arr.into_iter().map(serde_yaml::Value::from).collect()
                )
            }
            MergeValue::Object(obj) => {
                let mut map = serde_yaml::Mapping::new();
                for (k, v) in obj {
                    map.insert(
                        serde_yaml::Value::String(k),
                        serde_yaml::Value::from(v)
                    );
                }
                serde_yaml::Value::Mapping(map)
            }
        }
    }
}

// ================== TOML Conversion ==================

impl From<toml::Value> for MergeValue {
    fn from(value: toml::Value) -> Self {
        match value {
            toml::Value::String(s) => MergeValue::String(s),
            toml::Value::Integer(i) => MergeValue::Integer(i),
            toml::Value::Float(f) => MergeValue::Float(f),
            toml::Value::Boolean(b) => MergeValue::Bool(b),
            // PATTERN: Datetime to string conversion
            toml::Value::Datetime(dt) => MergeValue::String(dt.to_string()),
            toml::Value::Array(arr) => {
                MergeValue::Array(arr.into_iter().map(MergeValue::from).collect())
            }
            toml::Value::Table(table) => {
                // NOTE: toml::Table is BTreeMap, so keys are alphabetically sorted
                let obj: IndexMap<String, MergeValue> = table
                    .into_iter()
                    .map(|(k, v)| (k, MergeValue::from(v)))
                    .collect();
                MergeValue::Object(obj)
            }
        }
    }
}

impl TryFrom<MergeValue> for toml::Value {
    type Error = JinError;

    fn try_from(value: MergeValue) -> std::result::Result<Self, Self::Error> {
        match value {
            // CRITICAL: TOML has no null - error clearly
            MergeValue::Null => Err(JinError::Parse {
                format: "TOML".to_string(),
                message: "TOML does not support null values".to_string(),
            }),
            MergeValue::Bool(b) => Ok(toml::Value::Boolean(b)),
            MergeValue::Integer(i) => Ok(toml::Value::Integer(i)),
            MergeValue::Float(f) => Ok(toml::Value::Float(f)),
            MergeValue::String(s) => Ok(toml::Value::String(s)),
            MergeValue::Array(arr) => {
                // Recursively convert, propagating errors
                let converted: std::result::Result<Vec<toml::Value>, _> = arr
                    .into_iter()
                    .map(toml::Value::try_from)
                    .collect();
                Ok(toml::Value::Array(converted?))
            }
            MergeValue::Object(obj) => {
                let mut table = toml::Table::new();
                for (k, v) in obj {
                    table.insert(k, toml::Value::try_from(v)?);
                }
                Ok(toml::Value::Table(table))
            }
        }
    }
}

// ================== INI Conversion ==================

/// Convert INI file to MergeValue
///
/// INI sections become top-level object keys.
/// Key-value pairs within sections become nested objects.
/// Values before any section (general section) are placed at root level.
fn from_ini_value(ini: &Ini) -> MergeValue {
    let mut root = IndexMap::new();

    // Handle general section (values before any [section] header)
    for (key, value) in ini.general_section().iter() {
        root.insert(key.to_string(), MergeValue::String(value.to_string()));
    }

    // Handle named sections
    for (section_name, properties) in ini.iter() {
        if let Some(name) = section_name {
            let mut section_obj = IndexMap::new();
            for (key, value) in properties.iter() {
                section_obj.insert(key.to_string(), MergeValue::String(value.to_string()));
            }
            root.insert(name.to_string(), MergeValue::Object(section_obj));
        }
    }

    MergeValue::Object(root)
}

/// Convert MergeValue to INI file
///
/// # Errors
///
/// Returns error if:
/// - Value contains null (INI doesn't support null)
/// - Value contains arrays (INI doesn't support arrays)
/// - Value has more than 2 levels of nesting
fn to_ini_value(value: &MergeValue) -> Result<Ini> {
    let obj = value.as_object().ok_or_else(|| JinError::Parse {
        format: "INI".to_string(),
        message: "INI root must be an object".to_string(),
    })?;

    let mut ini = Ini::new();

    for (section_name, section_value) in obj {
        match section_value {
            MergeValue::Object(section_obj) => {
                for (key, val) in section_obj {
                    let string_val = match val {
                        MergeValue::Null => {
                            return Err(JinError::Parse {
                                format: "INI".to_string(),
                                message: "INI does not support null values".to_string(),
                            });
                        }
                        MergeValue::String(s) => s.clone(),
                        MergeValue::Bool(b) => b.to_string(),
                        MergeValue::Integer(i) => i.to_string(),
                        MergeValue::Float(f) => f.to_string(),
                        MergeValue::Array(_) => {
                            return Err(JinError::Parse {
                                format: "INI".to_string(),
                                message: "INI does not support arrays".to_string(),
                            });
                        }
                        MergeValue::Object(_) => {
                            return Err(JinError::Parse {
                                format: "INI".to_string(),
                                message: "INI does not support nested objects beyond 2 levels".to_string(),
                            });
                        }
                    };
                    ini.with_section(Some(section_name.as_str())).set(key, string_val);
                }
            }
            // Root-level non-object values go to general section
            MergeValue::String(s) => {
                ini.with_section(None::<String>).set(section_name, s.clone());
            }
            MergeValue::Bool(b) => {
                ini.with_section(None::<String>).set(section_name, b.to_string());
            }
            MergeValue::Integer(i) => {
                ini.with_section(None::<String>).set(section_name, i.to_string());
            }
            MergeValue::Float(f) => {
                ini.with_section(None::<String>).set(section_name, f.to_string());
            }
            MergeValue::Null => {
                return Err(JinError::Parse {
                    format: "INI".to_string(),
                    message: "INI does not support null values".to_string(),
                });
            }
            MergeValue::Array(_) => {
                return Err(JinError::Parse {
                    format: "INI".to_string(),
                    message: "INI does not support arrays".to_string(),
                });
            }
        }
    }

    Ok(ini)
}

// ================== Helper Methods ==================

impl MergeValue {
    /// Parse a JSON string into a MergeValue
    pub fn from_json(s: &str) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(s).map_err(|e| {
            JinError::Parse {
                format: "JSON".to_string(),
                message: e.to_string(),
            }
        })?;
        Ok(Self::from(value))
    }

    /// Parse a YAML string into a MergeValue
    pub fn from_yaml(s: &str) -> Result<Self> {
        let value: serde_yaml::Value = serde_yaml::from_str(s).map_err(|e| {
            let location_info = e.location()
                .map(|l| format!(" at line {}, column {}", l.line(), l.column()))
                .unwrap_or_default();
            JinError::Parse {
                format: "YAML".to_string(),
                message: format!("{}{}", e, location_info),
            }
        })?;
        Ok(Self::from(value))
    }

    /// Parse a TOML string into a MergeValue
    pub fn from_toml(s: &str) -> Result<Self> {
        let value: toml::Value = toml::from_str(s).map_err(|e| {
            JinError::Parse {
                format: "TOML".to_string(),
                message: e.to_string(),
            }
        })?;
        Ok(Self::from(value))
    }

    /// Parse an INI string into a MergeValue
    pub fn from_ini(s: &str) -> Result<Self> {
        let ini = Ini::load_from_str(s).map_err(|e| {
            JinError::Parse {
                format: "INI".to_string(),
                message: e.to_string(),
            }
        })?;
        Ok(from_ini_value(&ini))
    }

    /// Parse a file, auto-detecting format from extension
    ///
    /// Supported extensions:
    /// - `.json` - JSON format
    /// - `.yaml`, `.yml` - YAML format
    /// - `.toml` - TOML format
    /// - `.ini`, `.cfg`, `.conf` - INI format
    /// - `.editorconfig` - INI format (special case)
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        // Special case: .editorconfig is INI format
        let file_name = path.file_name().and_then(|n| n.to_str());
        if file_name == Some(".editorconfig") {
            return Self::from_ini(&content);
        }

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

    /// Serialize to a pretty-printed JSON string
    pub fn to_json_string(&self) -> Result<String> {
        let json_value: serde_json::Value = self.clone().into();
        serde_json::to_string_pretty(&json_value).map_err(|e| {
            JinError::Parse {
                format: "JSON".to_string(),
                message: e.to_string(),
            }
        })
    }

    /// Serialize to a compact JSON string (no formatting)
    pub fn to_json_string_compact(&self) -> Result<String> {
        let json_value: serde_json::Value = self.clone().into();
        serde_json::to_string(&json_value).map_err(|e| {
            JinError::Parse {
                format: "JSON".to_string(),
                message: e.to_string(),
            }
        })
    }

    /// Serialize to a YAML string
    pub fn to_yaml_string(&self) -> Result<String> {
        let yaml_value: serde_yaml::Value = self.clone().into();
        serde_yaml::to_string(&yaml_value).map_err(|e| {
            JinError::Parse {
                format: "YAML".to_string(),
                message: e.to_string(),
            }
        })
    }

    /// Serialize to a TOML string
    ///
    /// # Errors
    ///
    /// Returns `JinError::Parse` if the value contains null, as TOML
    /// does not support null values.
    pub fn to_toml_string(&self) -> Result<String> {
        let toml_value: toml::Value = self.clone().try_into()?;
        toml::to_string_pretty(&toml_value).map_err(|e| {
            JinError::Parse {
                format: "TOML".to_string(),
                message: e.to_string(),
            }
        })
    }

    /// Serialize to an INI string
    ///
    /// # Errors
    ///
    /// Returns `JinError::Parse` if the value contains:
    /// - Null values (INI doesn't support null)
    /// - Arrays (INI doesn't support arrays)
    /// - Objects nested more than 2 levels deep
    pub fn to_ini_string(&self) -> Result<String> {
        let ini = to_ini_value(self)?;
        let mut output = Vec::new();
        ini.write_to(&mut output).map_err(|e| {
            JinError::Parse {
                format: "INI".to_string(),
                message: e.to_string(),
            }
        })?;
        String::from_utf8(output).map_err(|e| {
            JinError::Parse {
                format: "INI".to_string(),
                message: e.to_string(),
            }
        })
    }

    /// Check if this value is a scalar (not object or array)
    pub fn is_scalar(&self) -> bool {
        !matches!(self, MergeValue::Object(_) | MergeValue::Array(_))
    }

    /// Get as integer
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            MergeValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Get as float (also works for integers)
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            MergeValue::Float(f) => Some(*f),
            MergeValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Get as boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            MergeValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Check if value contains any nulls (recursive)
    ///
    /// Useful before TOML conversion to detect potential failures.
    pub fn contains_null(&self) -> bool {
        match self {
            MergeValue::Null => true,
            MergeValue::Array(arr) => arr.iter().any(|v| v.contains_null()),
            MergeValue::Object(obj) => obj.values().any(|v| v.contains_null()),
            _ => false,
        }
    }
}
```

### Integration Points

```yaml
DEPENDENCIES:
  - ADD to Cargo.toml: rust-ini = "0.21"
  - EXISTING (no changes): serde_json, serde_yaml, toml, indexmap

MERGE ENGINE:
  - deep.rs uses MergeValue for merge operations (no changes needed)
  - layer.rs will use format detection for layer files (future work)

CORE:
  - error.rs provides JinError::Parse for format errors (already exists)

TESTING:
  - Inline #[cfg(test)] mod tests in value.rs
  - Use tempfile for file parsing tests (already in dev-dependencies)
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after each file modification - fix before proceeding
cargo check                           # Type checking - MUST pass
cargo fmt -- --check                  # Format check
cargo clippy -- -D warnings           # Lint check

# Expected: Zero errors, zero warnings
```

### Level 2: Build Validation

```bash
# Full build test
cargo build                           # Debug build

# Expected: Clean build with no warnings
```

### Level 3: Unit Tests (Component Validation)

```bash
# Run all merge module tests
cargo test merge::                    # All merge tests

# Run specific test categories
cargo test merge::value::test_yaml    # YAML-specific tests
cargo test merge::value::test_toml    # TOML-specific tests
cargo test merge::value::test_ini     # INI-specific tests
cargo test merge::value::test_roundtrip  # Roundtrip tests
cargo test merge::value::test_cross_format  # Cross-format tests

# Verify deep_merge still works
cargo test merge::deep::              # Deep merge tests

# Run with output for debugging
cargo test merge:: -- --nocapture

# Expected: All tests pass
```

### Level 4: Integration Testing

```bash
# Full test suite
cargo test

# Create test files and verify parsing
mkdir -p /tmp/jin-test
echo '{"key": "value"}' > /tmp/jin-test/test.json
echo 'key: value' > /tmp/jin-test/test.yaml
echo 'key = "value"' > /tmp/jin-test/test.toml
echo -e '[section]\nkey = value' > /tmp/jin-test/test.ini

# Verify all formats parse (via test code)
cargo test test_file_extension        # File detection tests

# Clean up
rm -rf /tmp/jin-test

# Expected: All formats parse correctly, file detection works
```

### Level 5: Compatibility Verification

```bash
# Ensure existing functionality still works
cargo test merge::deep::test_deep_merge
cargo test merge::deep::test_null_deletes
cargo test merge::deep::test_nested

# Run full test suite
cargo test

# Expected: All existing tests pass, no regressions
```

---

## Final Validation Checklist

### Technical Validation

- [ ] `cargo check` completes with 0 errors
- [ ] `cargo fmt -- --check` shows no formatting issues
- [ ] `cargo clippy -- -D warnings` shows no warnings
- [ ] `cargo build` succeeds
- [ ] `cargo test merge::value::` all tests pass
- [ ] `cargo test merge::deep::` all existing tests still pass
- [ ] `cargo test` all tests pass

### Feature Validation

- [ ] `From<serde_yaml::Value> for MergeValue` handles all YAML variants
- [ ] `From<MergeValue> for serde_yaml::Value` produces valid YAML
- [ ] `From<toml::Value> for MergeValue` handles all TOML variants
- [ ] `TryFrom<MergeValue> for toml::Value` errors on null values
- [ ] INI parsing converts sections to nested objects
- [ ] INI serialization converts nested objects to sections
- [ ] `MergeValue::from_json()` parses JSON strings correctly
- [ ] `MergeValue::from_yaml()` parses YAML strings correctly
- [ ] `MergeValue::from_toml()` parses TOML strings correctly
- [ ] `MergeValue::from_ini()` parses INI strings correctly
- [ ] `MergeValue::from_file()` auto-detects format by extension
- [ ] `to_json_string()` produces valid, pretty JSON
- [ ] `to_yaml_string()` produces valid YAML
- [ ] `to_toml_string()` errors appropriately on null values
- [ ] `to_ini_string()` errors on unsupported structures
- [ ] JSON -> MergeValue -> JSON roundtrip preserves data
- [ ] YAML -> MergeValue -> YAML roundtrip preserves data
- [ ] TOML -> MergeValue -> TOML roundtrip preserves data (no nulls)
- [ ] INI -> MergeValue -> INI roundtrip preserves data
- [ ] Cross-format conversion works (JSON -> YAML, etc.)

### Code Quality Validation

- [ ] All new methods have doc comments
- [ ] Error handling uses JinError::Parse consistently
- [ ] No unwrap() in library code (only in tests)
- [ ] Uses IndexMap throughout for key ordering
- [ ] Tests cover edge cases (empty, nested, null handling, INI limits)
- [ ] Follows existing code patterns in value.rs

---

## Anti-Patterns to Avoid

- ❌ Don't use `From` for TOML output - use `TryFrom` to handle null constraint
- ❌ Don't use `HashMap` for object conversion - use `IndexMap` for order
- ❌ Don't silently drop null values - return clear errors
- ❌ Don't assume all YAML keys are strings - filter non-string keys
- ❌ Don't use `unwrap()` in library code - use `?` and proper errors
- ❌ Don't forget to test deeply nested structures
- ❌ Don't skip testing the `contains_null()` helper before TOML conversion
- ❌ Don't hardcode file extensions - use case-insensitive comparison
- ❌ Don't allow INI to have more than 2 levels of nesting - error clearly
- ❌ Don't try to support arrays in INI - error with explanation
- ❌ Don't forget to handle YAML tagged values (extract inner value)
- ❌ Don't forget TOML Datetime → String conversion

---

## Confidence Score

**Rating: 9/10** for one-pass implementation success

**Justification:**
- Existing MergeValue structure is well-designed for multi-format support
- JSON conversion provides clear pattern to follow for YAML/TOML
- All serde-based dependencies already in Cargo.toml (only rust-ini needed)
- TOML and YAML constraints are well-documented with clear handling strategy
- INI format limitations are clearly defined with error strategies
- Comprehensive test cases defined covering all edge cases
- No architectural changes needed - purely additive to existing code
- Research documents provide complete API references

**Remaining Risks:**
- YAML tagged values may have edge cases not covered by simple extraction
- TOML heterogeneous array errors may surface during deep_merge (handled by returning error)
- INI files with unusual formats (Windows INI vs Unix) may have dialect issues
- Large file parsing performance not tested (unlikely to be an issue for config files)

---

## Research Artifacts Location

Research documentation stored at: `plan/P2M2/research/`

| File | Description |
|------|-------------|
| `serde_yaml_api.md` | Complete serde_yaml 0.9 API reference |
| `toml_crate_api.md` | Complete toml 0.8 API reference |
| `rust_ini_crate.md` | Complete rust-ini 0.21 API reference |
| `ini_format_patterns.md` | INI format specification and edge cases |
| `ini_quick_reference.md` | Quick lookup guide for INI |
| `ini_code_examples.md` | Production code implementations |

Key external references:
- serde_yaml documentation: https://docs.rs/serde_yaml/0.9/serde_yaml/
- toml documentation: https://docs.rs/toml/0.8/toml/
- rust-ini documentation: https://docs.rs/rust-ini/0.21/ini/
- TOML specification: https://toml.io/en/v1.0.0

---

## Appendix: Format Comparison Matrix

| Feature | JSON | YAML | TOML | INI | MergeValue |
|---------|------|------|------|-----|------------|
| Null | `null` | `null`/`~` | ❌ N/A | ❌ N/A | `Null` |
| Boolean | `true`/`false` | `true`/`false` | `true`/`false` | "true"/"false" | `Bool(bool)` |
| Integer | Number | Number | `Integer` | String | `Integer(i64)` |
| Float | Number | Number | `Float` | String | `Float(f64)` |
| String | `"string"` | `"string"` | `"string"` | `value` | `String(String)` |
| Array | `[...]` | `- ...` | `[...]` | ❌ N/A | `Array(Vec)` |
| Object | `{...}` | mapping | `[table]` | `[section]` | `Object(IndexMap)` |
| Datetime | ❌ (string) | ❌ (string) | native | ❌ (string) | `String` |
| Key Order | ❌ | ✅ | alphabetical | ✅ | ✅ (IndexMap) |
| Nesting | unlimited | unlimited | unlimited | 2 levels max | unlimited |

---

## Appendix: Test Case Examples

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ========== YAML Tests ==========

    #[test]
    fn test_yaml_roundtrip_basic() {
        let yaml = r#"
name: test
count: 42
enabled: true
ratio: 3.14
items:
  - one
  - two
"#;
        let val = MergeValue::from_yaml(yaml).unwrap();
        let back = val.to_yaml_string().unwrap();
        let reparsed = MergeValue::from_yaml(&back).unwrap();
        assert_eq!(val, reparsed);
    }

    #[test]
    fn test_yaml_null_handling() {
        let yaml = "value: null";
        let val = MergeValue::from_yaml(yaml).unwrap();
        assert!(val.as_object().unwrap().get("value").unwrap().is_null());
    }

    #[test]
    fn test_yaml_tagged_value() {
        let yaml = "value: !CustomTag data";
        let val = MergeValue::from_yaml(yaml).unwrap();
        // Tagged value should be extracted as its inner value
        assert!(val.as_object().unwrap().contains_key("value"));
    }

    // ========== TOML Tests ==========

    #[test]
    fn test_toml_roundtrip_basic() {
        let toml = r#"
name = "test"
count = 42
enabled = true
ratio = 3.14
items = ["one", "two"]
"#;
        let val = MergeValue::from_toml(toml).unwrap();
        let back = val.to_toml_string().unwrap();
        let reparsed = MergeValue::from_toml(&back).unwrap();
        assert_eq!(val, reparsed);
    }

    #[test]
    fn test_toml_null_error() {
        let val = MergeValue::Null;
        let result = val.to_toml_string();
        assert!(result.is_err());
        if let Err(JinError::Parse { format, .. }) = result {
            assert_eq!(format, "TOML");
        }
    }

    #[test]
    fn test_toml_nested_null_error() {
        let val = MergeValue::from(serde_json::json!({
            "outer": {
                "inner": null
            }
        }));
        assert!(val.contains_null());
        let result = val.to_toml_string();
        assert!(result.is_err());
    }

    #[test]
    fn test_toml_datetime_to_string() {
        let toml = r#"created = 1979-05-27T07:32:00Z"#;
        let val = MergeValue::from_toml(toml).unwrap();
        let created = val.as_object().unwrap().get("created").unwrap();
        assert!(created.as_str().is_some());
        assert!(created.as_str().unwrap().contains("1979"));
    }

    // ========== INI Tests ==========

    #[test]
    fn test_ini_roundtrip_basic() {
        let ini = r#"
[database]
host = localhost
port = 5432

[logging]
level = info
"#;
        let val = MergeValue::from_ini(ini).unwrap();
        let back = val.to_ini_string().unwrap();
        let reparsed = MergeValue::from_ini(&back).unwrap();

        // Compare structure (INI may reorder)
        let orig_db = val.as_object().unwrap().get("database").unwrap();
        let new_db = reparsed.as_object().unwrap().get("database").unwrap();
        assert_eq!(orig_db, new_db);
    }

    #[test]
    fn test_ini_section_to_nested_object() {
        let ini = r#"
[section]
key = value
"#;
        let val = MergeValue::from_ini(ini).unwrap();
        let obj = val.as_object().unwrap();
        assert!(obj.contains_key("section"));
        let section = obj.get("section").unwrap().as_object().unwrap();
        assert_eq!(section.get("key").unwrap().as_str(), Some("value"));
    }

    #[test]
    fn test_ini_general_section() {
        let ini = r#"
global_key = global_value

[section]
key = value
"#;
        let val = MergeValue::from_ini(ini).unwrap();
        let obj = val.as_object().unwrap();
        // General section values at root level
        assert_eq!(obj.get("global_key").unwrap().as_str(), Some("global_value"));
    }

    #[test]
    fn test_ini_null_error() {
        let val = MergeValue::from(serde_json::json!({
            "section": {
                "key": null
            }
        }));
        let result = val.to_ini_string();
        assert!(result.is_err());
    }

    #[test]
    fn test_ini_array_error() {
        let val = MergeValue::from(serde_json::json!({
            "section": {
                "items": [1, 2, 3]
            }
        }));
        let result = val.to_ini_string();
        assert!(result.is_err());
    }

    #[test]
    fn test_ini_deep_nesting_error() {
        let val = MergeValue::from(serde_json::json!({
            "level1": {
                "level2": {
                    "level3": "too deep"
                }
            }
        }));
        let result = val.to_ini_string();
        assert!(result.is_err());
    }

    // ========== Cross-Format Tests ==========

    #[test]
    fn test_json_to_yaml() {
        let json = r#"{"name": "test", "items": [1, 2, 3]}"#;
        let val = MergeValue::from_json(json).unwrap();
        let yaml = val.to_yaml_string().unwrap();
        assert!(yaml.contains("name: test") || yaml.contains("name: \"test\""));
    }

    #[test]
    fn test_yaml_to_toml_no_null() {
        let yaml = r#"
name: test
count: 42
"#;
        let val = MergeValue::from_yaml(yaml).unwrap();
        let toml = val.to_toml_string().unwrap();
        assert!(toml.contains("name = \"test\""));
    }

    #[test]
    fn test_json_to_ini_simple() {
        let json = r#"{"section": {"key": "value"}}"#;
        let val = MergeValue::from_json(json).unwrap();
        let ini = val.to_ini_string().unwrap();
        assert!(ini.contains("[section]"));
        assert!(ini.contains("key=value") || ini.contains("key = value"));
    }

    // ========== File Detection Tests ==========

    #[test]
    fn test_file_extension_detection() {
        use tempfile::NamedTempFile;
        use std::io::Write;

        // Test JSON
        let mut json_file = NamedTempFile::with_suffix(".json").unwrap();
        writeln!(json_file, r#"{{"test": true}}"#).unwrap();
        let val = MergeValue::from_file(json_file.path()).unwrap();
        assert!(val.as_object().unwrap().contains_key("test"));

        // Test YAML
        let mut yaml_file = NamedTempFile::with_suffix(".yaml").unwrap();
        writeln!(yaml_file, "test: true").unwrap();
        let val = MergeValue::from_file(yaml_file.path()).unwrap();
        assert!(val.as_object().unwrap().contains_key("test"));

        // Test TOML
        let mut toml_file = NamedTempFile::with_suffix(".toml").unwrap();
        writeln!(toml_file, "test = true").unwrap();
        let val = MergeValue::from_file(toml_file.path()).unwrap();
        assert!(val.as_object().unwrap().contains_key("test"));

        // Test INI
        let mut ini_file = NamedTempFile::with_suffix(".ini").unwrap();
        writeln!(ini_file, "[section]\ntest = true").unwrap();
        let val = MergeValue::from_file(ini_file.path()).unwrap();
        assert!(val.as_object().unwrap().contains_key("section"));
    }

    // ========== Helper Method Tests ==========

    #[test]
    fn test_is_scalar() {
        assert!(MergeValue::Null.is_scalar());
        assert!(MergeValue::Bool(true).is_scalar());
        assert!(MergeValue::Integer(42).is_scalar());
        assert!(MergeValue::Float(3.14).is_scalar());
        assert!(MergeValue::String("test".into()).is_scalar());
        assert!(!MergeValue::Array(vec![]).is_scalar());
        assert!(!MergeValue::Object(IndexMap::new()).is_scalar());
    }

    #[test]
    fn test_contains_null() {
        assert!(MergeValue::Null.contains_null());
        assert!(!MergeValue::Integer(42).contains_null());

        let nested = MergeValue::from(serde_json::json!({
            "a": { "b": null }
        }));
        assert!(nested.contains_null());

        let no_null = MergeValue::from(serde_json::json!({
            "a": { "b": "value" }
        }));
        assert!(!no_null.contains_null());
    }

    #[test]
    fn test_as_numeric() {
        assert_eq!(MergeValue::Integer(42).as_i64(), Some(42));
        assert_eq!(MergeValue::Integer(42).as_f64(), Some(42.0));
        assert_eq!(MergeValue::Float(3.14).as_f64(), Some(3.14));
        assert_eq!(MergeValue::Float(3.14).as_i64(), None);
        assert_eq!(MergeValue::String("test".into()).as_i64(), None);
    }

    #[test]
    fn test_as_bool() {
        assert_eq!(MergeValue::Bool(true).as_bool(), Some(true));
        assert_eq!(MergeValue::Bool(false).as_bool(), Some(false));
        assert_eq!(MergeValue::Integer(1).as_bool(), None);
    }
}
```
