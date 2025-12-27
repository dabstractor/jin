# PRP: P2.M1 - Merge Value System

---

## Goal

**Feature Goal**: Complete the unified value representation (`MergeValue`) for all parseable configuration formats (JSON, YAML, TOML), enabling seamless conversion between formats and serving as the foundation for Jin's deterministic merge engine.

**Deliverable**: An enhanced `src/merge/value.rs` module providing:
1. Complete bidirectional conversion between `MergeValue` and `serde_yaml::Value`
2. Complete bidirectional conversion between `MergeValue` and `toml::Value` (with TOML constraint handling)
3. Convenient parsing and serialization helper functions for all three formats
4. Format detection based on file extension
5. Comprehensive tests verifying roundtrip fidelity for all formats

**Success Definition**:
- All conversion tests pass: `cargo test merge::value::`
- JSON, YAML, and TOML roundtrips preserve data correctly
- TOML-specific constraints (no null, homogeneous arrays) are handled gracefully
- Format detection correctly identifies file types by extension
- Existing deep_merge tests continue to pass

---

## User Persona

**Target User**: Jin internals (merge engine, commit pipeline, format handlers)

**Use Case**: The MergeValue system is used by:
- `deep_merge()` to perform format-agnostic deep merging of configuration files
- Layer merge orchestration to combine configurations from multiple layers
- Format detection to automatically parse files based on extension
- Serialization to write merged configurations back to their original formats

**User Journey**: Users don't interact directly with MergeValue - they experience seamless merging of their JSON, YAML, and TOML configuration files without format-specific quirks affecting the merge behavior.

**Pain Points Addressed**:
- Eliminates format-specific merge logic by normalizing to a common representation
- Handles TOML constraints gracefully without data loss where possible
- Preserves key ordering (via IndexMap) for consistent, reproducible output

---

## Why

- **PRD Requirement**: Section 11.1 specifies "Deterministic structured merges" for JSON/YAML/TOML
- **Foundation for Merge Engine**: P2.M2 (Format Parsers) and P2.M3 (Deep Merge) depend on complete MergeValue conversions
- **Format Agnostic Processing**: Enables the same merge algorithm regardless of source format
- **Reversibility**: Section 11.1 requires merges to be "Deterministic and reversible"

---

## What

### User-Visible Behavior

After this milestone:
```rust
// Parse any supported format
let json_val = MergeValue::from_json(json_str)?;
let yaml_val = MergeValue::from_yaml(yaml_str)?;
let toml_val = MergeValue::from_toml(toml_str)?;

// Auto-detect format from file extension
let val = MergeValue::from_file("config.yaml")?;

// Deep merge works the same regardless of source format
let merged = deep_merge(yaml_val, toml_val)?;

// Serialize back to any format
let json_out = merged.to_json_string()?;
let yaml_out = merged.to_yaml_string()?;
let toml_out = merged.to_toml_string()?; // Errors on null values
```

### Technical Requirements

1. **YAML Conversion**: Full bidirectional conversion with `serde_yaml::Value`
2. **TOML Conversion**: Bidirectional conversion with proper constraint handling
3. **Null Handling**: TOML has no null - use `TryFrom` for error handling
4. **Number Handling**: Preserve integer vs float distinction across formats
5. **Key Order**: Use `IndexMap` throughout to preserve insertion order
6. **Helper Functions**: Parsing and serialization utilities for each format

### Success Criteria

- [ ] `From<serde_yaml::Value> for MergeValue` implemented
- [ ] `From<MergeValue> for serde_yaml::Value` implemented
- [ ] `From<toml::Value> for MergeValue` implemented
- [ ] `TryFrom<MergeValue> for toml::Value` implemented (handles null constraint)
- [ ] `MergeValue::from_json()`, `from_yaml()`, `from_toml()` helper methods
- [ ] `MergeValue::to_json_string()`, `to_yaml_string()`, `to_toml_string()` methods
- [ ] `MergeValue::from_file()` with format detection by extension
- [ ] All roundtrip tests pass for each format
- [ ] Edge cases documented and tested (TOML nulls, number types)
- [ ] Existing `deep_merge` tests continue to pass

---

## All Needed Context

### Context Completeness Check

_This PRP provides everything needed to implement complete MergeValue format conversions, including exact type mappings, conversion patterns, edge case handling, and comprehensive test cases._

### Documentation & References

```yaml
# MUST READ - Core Implementation Context

- file: src/merge/value.rs
  why: Current MergeValue implementation with JSON conversion
  critical: |
    - Already has: Null, Bool, Integer, Float, String, Array, Object variants
    - Already has: From<serde_json::Value> and From<MergeValue> for serde_json::Value
    - Uses IndexMap<String, MergeValue> for Object to preserve key order
    - Has is_null(), is_object(), is_array(), as_object(), as_str() helpers

- file: src/merge/deep.rs
  why: Consumer of MergeValue - must remain compatible
  critical: |
    - deep_merge(base: MergeValue, overlay: MergeValue) -> Result<MergeValue>
    - Uses is_null(), as_object(), shift_remove() on IndexMap
    - Depends on MergeValue structure being stable

- file: src/merge/mod.rs
  why: Module exports that need updating
  critical: |
    - Currently exports: deep_merge, merge_layers, text_merge, MergeValue
    - May need additional exports for new helper functions

- file: src/core/error.rs
  why: JinError::Parse variant for conversion errors
  pattern: |
    JinError::Parse { format: String, message: String }
    - Use for TOML null constraint violations
    - Use for invalid format detection

- file: Cargo.toml
  why: Dependencies already configured
  critical: |
    - serde_json = "1.0" ✓
    - serde_yaml = "0.9" ✓
    - toml = "0.8" ✓
    - indexmap = { version = "2.0", features = ["serde"] } ✓

# EXTERNAL REFERENCES

- url: https://docs.rs/serde_yaml/0.9/serde_yaml/enum.Value.html
  why: serde_yaml::Value enum definition
  critical: |
    - Variants: Null, Bool, Number, String, Sequence, Mapping
    - Number stores via serde_yaml::Number (has as_i64(), as_f64())
    - Mapping uses serde_yaml::Mapping (preserves order like IndexMap)

- url: https://docs.rs/toml/0.8/toml/value/enum.Value.html
  why: toml::Value enum definition
  critical: |
    - Variants: String, Integer, Float, Boolean, Datetime, Array, Table
    - NO NULL VARIANT - this is the key constraint
    - Integer is i64, Float is f64
    - Table uses toml::Table (BTreeMap, not ordered by insertion)

- url: https://docs.rs/indexmap/2.0/indexmap/map/struct.IndexMap.html
  why: IndexMap API for preserving key order
  critical: |
    - Use shift_remove() to preserve order when removing keys
    - iter() returns entries in insertion order
    - Serde feature enables Serialize/Deserialize
```

### Current Codebase Tree (Relevant Files)

```bash
jin/
├── src/
│   ├── core/
│   │   └── error.rs          # JinError::Parse for format errors
│   └── merge/
│       ├── mod.rs            # Module exports
│       ├── value.rs          # MergeValue enum (TO BE ENHANCED)
│       ├── deep.rs           # deep_merge() - consumer of MergeValue
│       ├── layer.rs          # Layer merge orchestration
│       └── text.rs           # 3-way text merge
├── Cargo.toml                # Dependencies already configured
└── tests/
    └── integration/
        └── cli_basic.rs      # CLI tests (unaffected)
```

### Desired Codebase Tree After P2.M1

```bash
jin/
├── src/
│   └── merge/
│       ├── mod.rs            # Updated exports
│       └── value.rs          # Enhanced with:
│           ├── MergeValue enum (existing)
│           ├── From<serde_yaml::Value>       # NEW
│           ├── From<MergeValue> for yaml     # NEW
│           ├── From<toml::Value>             # NEW
│           ├── TryFrom<MergeValue> for toml  # NEW
│           ├── impl MergeValue {
│           │   ├── from_json()               # NEW helper
│           │   ├── from_yaml()               # NEW helper
│           │   ├── from_toml()               # NEW helper
│           │   ├── from_file()               # NEW with auto-detect
│           │   ├── to_json_string()          # NEW helper
│           │   ├── to_yaml_string()          # NEW helper
│           │   └── to_toml_string()          # NEW helper (TryFrom)
│           │   }
│           └── comprehensive tests           # NEW tests
└── plan/
    └── P2M1/
        ├── PRP.md            # This file
        └── research/         # Research artifacts
```

### Known Gotchas & Library Quirks

```rust
// CRITICAL: TOML has no null type
// toml::Value has NO Null variant - when converting MergeValue::Null to TOML:
// - Option 1: Return Err() from TryFrom (chosen approach)
// - Option 2: Skip the key entirely (lossy)
// - Option 3: Use empty string (lossy, breaks semantics)

// CRITICAL: TOML arrays must be homogeneous
// All elements must be same type (all strings, all integers, etc.)
// Our deep_merge may create heterogeneous arrays - TOML serialization may fail
// Use TryFrom to surface these errors clearly

// GOTCHA: Number type distinctions
// - JSON: serde_json::Number (can be i64, u64, or f64)
// - YAML: serde_yaml::Number (same approach)
// - TOML: Separate Integer(i64) and Float(f64) variants
// MergeValue uses separate Integer(i64) and Float(f64) - matches TOML best

// GOTCHA: Key ordering preservation
// - MergeValue uses IndexMap ✓ (insertion order)
// - serde_yaml::Mapping preserves order ✓
// - toml::Table is BTreeMap (alphabetical order) ✗
// When converting to TOML, key order will change to alphabetical

// PATTERN: Error handling for format conversion
// Use JinError::Parse { format: "TOML", message: "..." } for conversion errors
// This allows calling code to understand which format caused the issue

// PATTERN: YAML Mapping to IndexMap conversion
// serde_yaml::Mapping is not IndexMap, but can be iterated in order
// Need to iterate and build IndexMap manually
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// ================== src/merge/value.rs ADDITIONS ==================

use crate::core::{JinError, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

// Existing MergeValue enum (unchanged)
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

// ================== YAML Conversion ==================

impl From<serde_yaml::Value> for MergeValue {
    fn from(value: serde_yaml::Value) -> Self {
        match value {
            serde_yaml::Value::Null => MergeValue::Null,
            serde_yaml::Value::Bool(b) => MergeValue::Bool(b),
            serde_yaml::Value::Number(n) => {
                // Try integer first, then float
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
                let obj: IndexMap<String, MergeValue> = map
                    .into_iter()
                    .filter_map(|(k, v)| {
                        // YAML keys can be non-strings; we only support string keys
                        k.as_str().map(|s| (s.to_string(), MergeValue::from(v)))
                    })
                    .collect();
                MergeValue::Object(obj)
            }
            // Handle tagged values by extracting the value
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
            toml::Value::Datetime(dt) => {
                // Convert datetime to string representation
                MergeValue::String(dt.to_string())
            }
            toml::Value::Array(arr) => {
                MergeValue::Array(arr.into_iter().map(MergeValue::from).collect())
            }
            toml::Value::Table(table) => {
                // Note: toml::Table is BTreeMap, order will be alphabetical
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
            MergeValue::Null => {
                Err(JinError::Parse {
                    format: "TOML".to_string(),
                    message: "TOML does not support null values".to_string(),
                })
            }
            MergeValue::Bool(b) => Ok(toml::Value::Boolean(b)),
            MergeValue::Integer(i) => Ok(toml::Value::Integer(i)),
            MergeValue::Float(f) => Ok(toml::Value::Float(f)),
            MergeValue::String(s) => Ok(toml::Value::String(s)),
            MergeValue::Array(arr) => {
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

// ================== Helper Methods ==================

impl MergeValue {
    // ---- Parsing Helpers ----

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
            JinError::Parse {
                format: "YAML".to_string(),
                message: e.to_string(),
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

    /// Parse a file, auto-detecting format from extension
    ///
    /// Supported extensions:
    /// - `.json` - JSON format
    /// - `.yaml`, `.yml` - YAML format
    /// - `.toml` - TOML format
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

    // ---- Serialization Helpers ----

    /// Serialize to a JSON string
    pub fn to_json_string(&self) -> Result<String> {
        let json_value: serde_json::Value = self.clone().into();
        serde_json::to_string_pretty(&json_value).map_err(|e| {
            JinError::Parse {
                format: "JSON".to_string(),
                message: e.to_string(),
            }
        })
    }

    /// Serialize to a compact JSON string (no pretty printing)
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

    // ---- Type Checking Helpers (existing, may need additions) ----

    /// Check if this value is a scalar (not object or array)
    pub fn is_scalar(&self) -> bool {
        !matches!(self, MergeValue::Object(_) | MergeValue::Array(_))
    }

    /// Get as integer reference
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            MergeValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Get as float reference
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            MergeValue::Float(f) => Some(*f),
            MergeValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Get as boolean reference
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            MergeValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Check if value contains any nulls (recursive)
    /// Useful before TOML conversion
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

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: ADD serde_yaml::Value conversion to src/merge/value.rs
  - IMPLEMENT: From<serde_yaml::Value> for MergeValue
  - IMPLEMENT: From<MergeValue> for serde_yaml::Value
  - HANDLE: Number type (try i64 first, then f64)
  - HANDLE: Tagged values (extract inner value)
  - HANDLE: Non-string keys (filter out, YAML allows any type as key)
  - PLACEMENT: After existing From<serde_json::Value> implementation
  - TESTS: YAML-specific roundtrip tests

Task 2: ADD toml::Value conversion to src/merge/value.rs
  - IMPLEMENT: From<toml::Value> for MergeValue
  - IMPLEMENT: TryFrom<MergeValue> for toml::Value (NOT From - must handle null)
  - HANDLE: Datetime by converting to string
  - HANDLE: Null by returning JinError::Parse
  - PLACEMENT: After YAML conversion implementations
  - TESTS: TOML-specific tests including null error case

Task 3: ADD parsing helper methods to MergeValue
  - IMPLEMENT: from_json(s: &str) -> Result<Self>
  - IMPLEMENT: from_yaml(s: &str) -> Result<Self>
  - IMPLEMENT: from_toml(s: &str) -> Result<Self>
  - IMPLEMENT: from_file(path: impl AsRef<Path>) -> Result<Self>
  - ERROR HANDLING: Use JinError::Parse with format name
  - TESTS: Parsing success and error cases

Task 4: ADD serialization helper methods to MergeValue
  - IMPLEMENT: to_json_string(&self) -> Result<String>
  - IMPLEMENT: to_json_string_compact(&self) -> Result<String>
  - IMPLEMENT: to_yaml_string(&self) -> Result<String>
  - IMPLEMENT: to_toml_string(&self) -> Result<String>
  - ERROR HANDLING: TOML must use try_into for null handling
  - TESTS: Serialization output verification

Task 5: ADD additional type-checking helpers
  - IMPLEMENT: is_scalar(&self) -> bool
  - IMPLEMENT: as_i64(&self) -> Option<i64>
  - IMPLEMENT: as_f64(&self) -> Option<f64>
  - IMPLEMENT: as_bool(&self) -> Option<bool>
  - IMPLEMENT: contains_null(&self) -> bool (recursive check)
  - PLACEMENT: In impl MergeValue block with existing helpers
  - TESTS: Helper method tests

Task 6: ADD comprehensive tests
  - FILE: Tests inline in value.rs (after existing tests section)
  - TEST CATEGORIES:
    - YAML roundtrip tests (various data types)
    - TOML roundtrip tests (no nulls)
    - TOML null error tests
    - Cross-format tests (JSON -> MergeValue -> YAML)
    - File parsing tests (with tempfile)
    - Edge cases (empty objects/arrays, nested structures)
  - PATTERN: Use serde_json::json! for test data construction
  - VERIFY: Existing deep_merge tests still pass

Task 7: UPDATE src/merge/mod.rs exports if needed
  - REVIEW: Check if any new types need exporting
  - PRESERVE: Existing exports (deep_merge, merge_layers, text_merge, MergeValue)
  - VERIFY: All tests pass after export changes
```

### Implementation Patterns & Key Details

```rust
// PATTERN: YAML Number handling
// serde_yaml::Number provides as_i64() and as_f64() methods
// Try integer first to preserve integer semantics
if let Some(i) = n.as_i64() {
    MergeValue::Integer(i)
} else if let Some(f) = n.as_f64() {
    MergeValue::Float(f)
} else {
    MergeValue::Float(0.0) // Fallback
}

// PATTERN: YAML Tagged values
// serde_yaml 0.9+ has Value::Tagged for custom tags like !include
// Extract the inner value and convert that
serde_yaml::Value::Tagged(tagged) => MergeValue::from(tagged.value)

// PATTERN: TOML Datetime handling
// TOML has a native Datetime type; we convert to string
// This is lossy but preserves the data for display/storage
toml::Value::Datetime(dt) => MergeValue::String(dt.to_string())

// PATTERN: TOML null handling with TryFrom
// TryFrom allows returning Result instead of panicking
impl TryFrom<MergeValue> for toml::Value {
    type Error = JinError;

    fn try_from(value: MergeValue) -> std::result::Result<Self, Self::Error> {
        match value {
            MergeValue::Null => Err(JinError::Parse {
                format: "TOML".to_string(),
                message: "TOML does not support null values".to_string(),
            }),
            // ... other cases
        }
    }
}

// PATTERN: File format detection
let extension = path.extension()
    .and_then(|e| e.to_str())
    .map(|e| e.to_lowercase());

match extension.as_deref() {
    Some("json") => Self::from_json(&content),
    Some("yaml") | Some("yml") => Self::from_yaml(&content),
    Some("toml") => Self::from_toml(&content),
    Some(ext) => Err(JinError::Parse { ... }),
    None => Err(JinError::Parse { ... }),
}

// PATTERN: Test data construction with serde_json::json!
// Even for YAML/TOML tests, construct MergeValue via JSON for readability
let test_val = MergeValue::from(serde_json::json!({
    "name": "test",
    "count": 42,
    "items": ["a", "b", "c"]
}));
```

### Integration Points

```yaml
MERGE ENGINE:
  - deep.rs uses MergeValue for merge operations (no changes needed)
  - layer.rs will use format detection for layer files (future work)

CORE:
  - error.rs provides JinError::Parse for format errors (already exists)

TESTING:
  - Inline #[cfg(test)] mod tests in value.rs
  - Use tempfile for file parsing tests (already in dev-dependencies)

DEPENDENCIES (already in Cargo.toml):
  - serde_json = "1.0"
  - serde_yaml = "0.9"
  - toml = "0.8"
  - indexmap = { version = "2.0", features = ["serde"] }
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

# Expected: Clean build
```

### Level 3: Unit Tests (Component Validation)

```bash
# Run MergeValue tests
cargo test merge::value::             # All value.rs tests
cargo test test_yaml                  # YAML-specific tests
cargo test test_toml                  # TOML-specific tests
cargo test test_roundtrip             # Roundtrip tests

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

# Manual verification - create test files and parse them
echo '{"key": "value"}' > /tmp/test.json
echo 'key: value' > /tmp/test.yaml
echo 'key = "value"' > /tmp/test.toml

# Run a quick Rust script to test (via cargo test or playground)

# Expected: All formats parse correctly
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
- [ ] `MergeValue::from_json()` parses JSON strings correctly
- [ ] `MergeValue::from_yaml()` parses YAML strings correctly
- [ ] `MergeValue::from_toml()` parses TOML strings correctly
- [ ] `MergeValue::from_file()` auto-detects format by extension
- [ ] `to_json_string()` produces valid, pretty JSON
- [ ] `to_yaml_string()` produces valid YAML
- [ ] `to_toml_string()` errors appropriately on null values
- [ ] JSON -> MergeValue -> JSON roundtrip preserves data
- [ ] YAML -> MergeValue -> YAML roundtrip preserves data
- [ ] TOML -> MergeValue -> TOML roundtrip preserves data (no nulls)
- [ ] Cross-format conversion works (JSON -> YAML, etc.)

### Code Quality Validation

- [ ] All new methods have doc comments
- [ ] Error handling uses JinError::Parse consistently
- [ ] No unwrap() in library code (only in tests)
- [ ] Uses IndexMap throughout for key ordering
- [ ] Tests cover edge cases (empty, nested, null handling)
- [ ] Follows existing code patterns in value.rs

---

## Anti-Patterns to Avoid

- ❌ Don't use `From` for TOML output - use `TryFrom` to handle null constraint
- ❌ Don't use `HashMap` for object conversion - use `IndexMap` for order
- ❌ Don't silently drop null values in TOML - return an error
- ❌ Don't assume all YAML keys are strings - filter non-string keys
- ❌ Don't use `unwrap()` in library code - use `?` and proper errors
- ❌ Don't forget to test deeply nested structures
- ❌ Don't skip testing the contains_null() helper before TOML conversion
- ❌ Don't hardcode file extensions - use case-insensitive comparison

---

## Confidence Score

**Rating: 9/10** for one-pass implementation success

**Justification:**
- Existing MergeValue structure is already well-designed for this task
- JSON conversion provides clear pattern to follow for YAML/TOML
- All dependencies already in Cargo.toml
- TOML constraints are well-documented with clear handling strategy
- Comprehensive test cases defined
- No architectural changes needed - purely additive

**Remaining Risks:**
- YAML tagged values may have edge cases not covered
- TOML heterogeneous array errors may surface during deep_merge
- Large file parsing performance not tested

---

## Research Artifacts Location

Research documentation stored at: `plan/P2M1/research/`

Key external references:
- serde_yaml Value documentation: https://docs.rs/serde_yaml/0.9/serde_yaml/enum.Value.html
- toml Value documentation: https://docs.rs/toml/0.8/toml/value/enum.Value.html
- IndexMap documentation: https://docs.rs/indexmap/2.0/indexmap/map/struct.IndexMap.html
- TOML specification (no null): https://toml.io/en/v1.0.0

---

## Appendix: Format Comparison Matrix

| Feature | JSON | YAML | TOML | MergeValue |
|---------|------|------|------|------------|
| Null | `null` | `null` / `~` | ❌ N/A | `Null` |
| Boolean | `true`/`false` | `true`/`false` | `true`/`false` | `Bool(bool)` |
| Integer | Number | Number | `Integer` | `Integer(i64)` |
| Float | Number | Number | `Float` | `Float(f64)` |
| String | `"string"` | `"string"` / `string` | `"string"` | `String(String)` |
| Array | `[...]` | `- ...` | `[...]` | `Array(Vec)` |
| Object | `{...}` | mapping | `[table]` | `Object(IndexMap)` |
| Datetime | ❌ (string) | ❌ (string) | native | `String` (converted) |
| Key Order | ❌ | ✅ | alphabetical | ✅ (IndexMap) |

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

    // ========== Cross-Format Tests ==========

    #[test]
    fn test_json_to_yaml() {
        let json = r#"{"name": "test", "items": [1, 2, 3]}"#;
        let val = MergeValue::from_json(json).unwrap();
        let yaml = val.to_yaml_string().unwrap();
        assert!(yaml.contains("name: test"));
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
    }
}
```
