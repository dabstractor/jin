# Product Requirement Prompt (PRP): All Format Parsers (P2.M2.T1)

---

## Goal

**Feature Goal**: Implement complete format parsers for JSON, YAML, TOML, and INI that convert configuration files into the unified `MergeValue` type for merge operations.

**Deliverable**: Format parsing functions integrated into `src/merge/value.rs`:
- `from_json()` - Parse JSON strings into `MergeValue`
- `from_yaml()` - Parse YAML strings into `MergeValue`
- `from_toml()` - Parse TOML strings into `MergeValue`
- `from_ini()` - Parse INI strings into `MergeValue`

**Success Definition**:
- All four format parsing functions compile without errors
- Each parser correctly converts its format to `MergeValue` variants
- Nested structures (objects/arrays) are handled recursively
- Format-specific types are normalized (e.g., TOML datetime → String)
- Comprehensive tests cover happy path, edge cases, and error handling
- `cargo test --package jin --lib merge::value::tests::from_*` passes for all formats

## User Persona

**Target User**: AI coding agent implementing the format parsing layer

**Use Case**: The agent needs to create or validate parsing functions that:
- Convert four common configuration formats to unified `MergeValue` representation
- Handle format-specific types correctly
- Propagate parse errors through `JinError`
- Support nested structures recursively

**User Journey**:
1. Agent receives this PRP as context
2. Reviews existing implementation patterns in `src/merge/value.rs`
3. Implements or validates each format parser
4. Adds comprehensive tests for edge cases
5. Validates compilation and test success

**Pain Points Addressed**:
- Unified interface for multiple configuration formats
- Consistent error handling across all parsers
- Proper handling of format-specific gotchas (YAML null, TOML datetime, etc.)

## Why

- **Foundation for merge operations**: All merge tasks (P2.M3) depend on parsed `MergeValue` instances
- **Multi-format support**: Jin supports JSON, YAML, TOML, INI configuration files
- **Type normalization**: Different formats have different types (e.g., TOML datetime) that need normalization
- **Error propagation**: Parse errors must convert to appropriate `JinError` variants

## What

Implement or validate format parsing functions that convert configuration file contents into `MergeValue` instances.

### Format-Specific Requirements

| Format | Source Type | Target MergeValue | Special Handling |
|--------|-------------|-------------------|------------------|
| JSON | `serde_json::Value` | Direct variant mapping | Number can be i64 or f64 |
| YAML | `serde_yaml_ng::Value` | Mapping→Object, Sequence→Array | Mapping vs Object naming |
| TOML | `toml::Value` | Table→Object, Datetime→String | Distinct Integer/Float variants |
| INI | Flat sections | Nested Object structure | Sections become top-level keys |

### Success Criteria

- [ ] `from_json()` parses all JSON value types (null, bool, number, string, array, object)
- [ ] `from_yaml()` handles YAML-specific types (Mapping, Sequence)
- [ ] `from_toml()` converts datetime to String, handles distinct Integer/Float
- [ ] `from_ini()` converts flat sections to nested objects
- [ ] Nested structures are parsed recursively
- [ ] Parse errors return appropriate `JinError` variants
- [ ] All format parsers pass comprehensive tests
- [ ] No clippy warnings or rustc errors

---

## All Needed Context

### Context Completeness Check

**Validation**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: YES - This PRP provides:
- Complete format parser implementations with code examples
- Research documents for each format's crate API
- Existing implementation patterns from `src/merge/value.rs`
- Error handling patterns from `src/core/error.rs`
- Specific gotchas for each format
- Validation commands specific to this project

### Documentation & References

```yaml
# MUST READ - Internal Project Documentation

- file: /home/dustin/projects/jin-glm-doover/src/merge/value.rs
  why: EXISTING IMPLEMENTATION - Format parsers already exist (lines 90-346)
  section: Lines 90-138 for JSON, 140-192 for YAML, 194-232 for TOML, 261-346 for INI
  critical: Complete reference implementations for all format parsers

- file: /home/dustin/projects/jin-glm-doover/src/core/error.rs
  why: Error handling - use existing JinError parse variants
  pattern: JinError::JsonParse, JinError::YamlParse, JinError::TomlParse, JinError::IniParse
  section: Lines 38-52 for parse error variants
  critical: All parse errors have transparent conversion via #[from]

- file: /home/dustin/projects/jin-glm-doover/Cargo.toml
  why: Verify dependencies are available
  section: Lines 21-24 for serde and format parsers
  critical: serde_json (1.0), serde_yaml_ng (0.9), toml (0.9), configparser (0.4)

- file: /home/dustin/projects/jin-glm-doover/PRD.md
  why: Merge strategy specification
  section: Lines 258-300 for Merge Strategy (§11)
  critical: Understanding how parsed values will be merged

# RESEARCH DOCUMENTS - Created for this PRP

- docfile: /home/dustin/projects/jin-glm-doover/plan/P2M2T1/research/serde_json_research.md
  why: serde_json crate API and patterns
  critical: Value enum variants, number handling (is_i64/is_f64), recursive conversion

- docfile: /home/dustin/projects/jin-glm-doover/plan/P2M2T1/research/serde_yaml_ng_research.md
  why: serde_yaml_ng crate API and differences from serde_yaml
  critical: Mapping vs Object, Sequence vs Array, anchor/alias handling

- docfile: /home/dustin/projects/jin-glm-doover/plan/P2M2T1/research/toml_research.md
  why: toml crate API with datetime handling
  critical: Distinct Integer/Float variants, Datetime conversion, Table handling

- docfile: /home/dustin/projects/jin-glm-doover/plan/P2M2T1/research/configparser_research.md
  why: configparser crate API for INI files
  critical: Section structure, all values are strings, global section handling

# EXTERNAL - Crate Documentation

- url: https://docs.rs/serde_json/latest/serde_json/enum.Value.html
  why: serde_json::Value reference for JSON parser implementation
  critical: Shows enum variants and conversion patterns

- url: https://docs.rs/serde_yaml_ng/latest/serde_yaml_ng/enum.Value.html
  why: serde_yaml_ng::Value reference for YAML parser
  critical: Different naming (Mapping, Sequence) from JSON

- url: https://docs.rs/toml/latest/toml/enum.Value.html
  why: toml::Value reference for TOML parser
  critical: Datetime variant, distinct Integer/Float types

- url: https://docs.rs/configparser/latest/configparser/
  why: configparser API reference for INI parser
  critical: Ini::new(), load(), get(), sections() methods
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
│   │   ├── error.rs                # Has parse error variants (lines 38-52)
│   │   ├── layer.rs                # Layer enum with precedence
│   │   └── config.rs
│   ├── merge/
│   │   ├── mod.rs                  # Exports MergeValue
│   │   └── value.rs                # HAS: from_json, from_yaml, from_toml, from_ini
│   ├── git/
│   ├── cli/
│   ├── commands/
│   ├── commit/
│   ├── staging/
│   └── workspace/
└── tests/
    └── merge/
        └── value_test.rs           # Integration tests for MergeValue
```

### Desired Codebase Tree

```bash
/home/dustin/projects/jin-glm-doover/
├── src/
│   └── merge/
│       └── value.rs                # EXISTING: Has all format parsers (lines 90-346)
                                    # VALIDATE: Correct implementation of all parsers
└── tests/
    └── merge/
        └── value_test.rs           # EXISTING: Tests for format parsers (lines 540-674)
                                    # VERIFY: Comprehensive coverage
```

### Known Gotchas of Our Codebase & Library Quirks

```rust
// CRITICAL: JSON number type handling
// JSON has only "number" type - must check is_i64() vs is_f64()
// Pattern: n.is_i64() ? MergeValue::Integer(n.as_i64()) : MergeValue::Float(n.as_f64())

// CRITICAL: YAML uses different naming than JSON
// serde_yaml_ng uses "Mapping" not "Object", "Sequence" not "Array"
// Must handle: Yaml::Mapping, Yaml::Sequence in match arms

// CRITICAL: YAML null has multiple representations
// null, ~, and "" can all represent null
// Our parser treats only explicit null as MergeValue::Null

// CRITICAL: TOML has distinct Integer and Float variants
// Unlike JSON (single Number), TOML has Value::Integer(i64) and Value::Float(f64)
// No type checking needed - direct mapping possible

// CRITICAL: TOML datetime has no JSON equivalent
// TOML Value::Datetime must be converted to MergeValue::String
// Pattern: Value::Datetime(dt) => MergeValue::String(dt.to_string())

// CRITICAL: INI has flat structure - must convert to nested
// INI: [section] key=value becomes JSON: {"section": {"key": "value"}}
// Custom parser required - configparser crate returns flat HashMap

// CRITICAL: INI all values are strings
// INI does not have native number, boolean, or array types
// All values parse as MergeValue::String

// CRITICAL: INI global keys (before any section)
// Keys appearing before first [section] need special handling
// Common pattern: create "global" section or treat as top-level keys

// CRITICAL: Error conversion uses #[from] attribute
// JinError::JsonParse(#[from] serde_json::Error)
// Using ? operator automatically converts parse errors

// CRITICAL: Recursive conversion for nested structures
// Arrays and Objects require recursive parsing
// Pattern: map(|v| MergeValue::from_format(&format::to_string(&v)?)).collect()

// CRITICAL: Order preservation requirement
// Use IndexMap for all object/map variants
// Preserves insertion order from highest layer (PRD requirement)

// GOTCHA: serde_json to_string on nested values
// When recursively converting, re-serialize and re-parse
// Pattern: MergeValue::from_json(&serde_json::to_string(&v)?)
// This handles nested variants correctly but is inefficient

// GOTCHA: YAML anchor/alias resolution
// serde_yaml_ng automatically resolves anchors during parsing
// No special handling needed for merge keys (<<) - they remain in structure

// GOTCHA: TOML inline tables vs dotted keys
// table.key = value and [table] key = value are equivalent
// toml crate normalizes these - no special handling needed
```

---

## Implementation Blueprint

### Data Models and Structure

```rust
// Format parsers are methods on MergeValue enum
// No new data structures needed - parsers return existing MergeValue variants

impl MergeValue {
    /// Parse JSON string -> MergeValue
    pub fn from_json(input: &str) -> Result<Self>;

    /// Parse YAML string -> MergeValue
    pub fn from_yaml(input: &str) -> Result<Self>;

    /// Parse TOML string -> MergeValue
    pub fn from_toml(input: &str) -> Result<Self>;

    /// Parse INI string -> MergeValue
    pub fn from_ini(input: &str) -> Result<Self>;
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
# NOTE: These tasks are ALREADY COMPLETE in src/merge/value.rs
# This section serves as validation checklist and implementation reference

Task 1: VALIDATE from_json() implementation
  - LOCATION: src/merge/value.rs lines 103-138
  - VERIFY: Correct serde_json::Value to MergeValue conversion
  - CHECK: Number type handling (is_i64 vs is_f64)
  - CHECK: Recursive conversion for arrays and objects
  - ERROR: Uses ? operator for automatic JinError::JsonParse conversion
  - TEST: test_from_json_valid, test_from_json_invalid, test_from_json_nested, test_from_json_array

Task 2: VALIDATE from_yaml() implementation
  - LOCATION: src/merge/value.rs lines 152-192
  - VERIFY: Correct serde_yaml_ng::Value to MergeValue conversion
  - CHECK: Handles Mapping (not Object) and Sequence (not Array)
  - CHECK: Recursive conversion for nested structures
  - CHECK: YAML-specific variants fall back to String
  - ERROR: Uses ? operator for automatic JinError::YamlParse conversion
  - TEST: test_from_yaml_valid, test_from_yaml_array

Task 3: VALIDATE from_toml() implementation
  - LOCATION: src/merge/value.rs lines 206-232
  - VERIFY: Correct toml::Value to MergeValue conversion
  - CHECK: Datetime converted to String (dt.to_string())
  - CHECK: Distinct Integer/Float variants handled correctly
  - CHECK: Helper function convert_toml_value_to_merge_value for recursion
  - ERROR: Uses ? operator for automatic JinError::TomlParse conversion
  - TEST: test_from_toml_valid, test_from_toml_table

Task 4: VALIDATE from_ini() implementation
  - LOCATION: src/merge/value.rs lines 293-346
  - VERIFY: Correct flat INI to nested Object conversion
  - CHECK: Sections become top-level Object keys
  - CHECK: Key-value pairs become nested Object entries
  - CHECK: All values stored as String variant
  - CHECK: Global section handling (before first [section])
  - CHECK: Comment handling (# and ;)
  - ERROR: Manual conversion to JinError::IniParse(String)
  - TEST: test_from_ini_valid

Task 5: VERIFY comprehensive test coverage
  - LOCATION: src/merge/value.rs lines 522-805 (embedded tests)
  - LOCATION: tests/merge/value_test.rs (integration tests)
  - CHECK: Each format parser has happy path test
  - CHECK: Each format parser has error test (invalid input)
  - CHECK: Nested structure tests for recursive parsing
  - CHECK: Edge cases: empty files, null values, complex nesting
  - RUN: cargo test --package jin --lib merge::value::tests
```

### Implementation Patterns & Key Details

```rust
// ===== FROM JSON PATTERN (lines 103-138) =====
pub fn from_json(input: &str) -> Result<Self> {
    use serde_json::Value as Json;

    let json: Json = serde_json::from_str(input)?;

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
                    MergeValue::from_json(&serde_json::to_string(&v)?)
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

// ===== FROM YAML PATTERN (lines 152-192) =====
pub fn from_yaml(input: &str) -> Result<Self> {
    use serde_yaml_ng::Value as Yaml;

    let yaml: Yaml = serde_yaml_ng::from_str(input)?;

    match yaml {
        Yaml::Null => Ok(MergeValue::Null),
        Yaml::Bool(b) => Ok(MergeValue::Boolean(b)),
        Yaml::Number(n) => {
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
                    MergeValue::from_yaml(&serde_yaml_ng::to_string(&v)?)
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

// ===== FROM TOML PATTERN (lines 206-232) =====
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
                .map(|v| Self::convert_toml_value_to_merge_value(&v))
                .collect();
            Ok(MergeValue::Array(converted?))
        }
        Toml::Table(table) => {
            let mut map = IndexMap::new();
            for (k, v) in table {
                map.insert(k, Self::convert_toml_value_to_merge_value(&v)?);
            }
            Ok(MergeValue::Object(map))
        }
    }
}

// Helper for direct TOML value conversion (avoids re-serialization)
fn convert_toml_value_to_merge_value(v: &toml::Value) -> Result<MergeValue> {
    // Similar structure to from_toml but takes reference
    // See src/merge/value.rs lines 235-259
}

// ===== FROM INI PATTERN (lines 293-346) =====
pub fn from_ini(input: &str) -> Result<Self> {
    let mut outer_map = IndexMap::new();
    let mut current_section: String = String::new();
    let mut current_map: Option<ObjectMap> = None;

    for line in input.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            // Save previous section if exists
            if let Some(map) = current_map.take() {
                if !current_section.is_empty() {
                    outer_map.insert(current_section, MergeValue::Object(map));
                }
            }

            // Start new section
            current_section = line[1..line.len() - 1].to_string();
            current_map = Some(IndexMap::new());
            continue;
        }

        // Key-value pair
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().to_string();

            // Create a section if none exists (global section)
            if current_map.is_none() {
                current_section = "global".to_string();
                current_map = Some(IndexMap::new());
            }

            if let Some(ref mut map) = current_map {
                map.insert(key, MergeValue::String(value));
            }
        }
    }

    // Save last section
    if let Some(map) = current_map {
        if !current_section.is_empty() {
            outer_map.insert(current_section, MergeValue::Object(map));
        }
    }

    Ok(MergeValue::Object(outer_map))
}
```

### Integration Points

```yaml
ERROR_HANDLING:
  - use: src/core/error.rs
  - patterns:
    * JinError::JsonParse(#[from] serde_json::Error) - automatic conversion
    * JinError::YamlParse(#[from] serde_yaml_ng::Error) - automatic conversion
    * JinError::TomlParse(#[from] toml::de::Error) - automatic conversion
    * JinError::IniParse(String) - manual string conversion
  - usage: ? operator automatically converts parse errors

MERGE_INTEGRATION:
  - use: src/merge/value.rs merge() method (lines 375-418)
  - purpose: Parsed values are merged using PRD rules
  - dependency: Format parsers create MergeValue instances for merging

TYPE_ALIASES:
  - type ObjectMap = IndexMap<String, MergeValue>
  - used by: All format parsers for object/map variants
  - preserves: Insertion order from highest layer

DEPENDENCIES:
  - serde_json = "1.0" (Cargo.toml line 22)
  - serde_yaml_ng = "0.9" (Cargo.toml line 23)
  - toml = "0.9" (Cargo.toml line 24)
  - configparser = "0.4" (Cargo.toml line 25)
  - indexmap = { version = "2.7", features = ["serde"] }
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run to verify format parsers compile
cargo check --package jin

# Run linter
cargo clippy --package jin -- -D warnings

# Check formatting
cargo fmt --check

# Expected: Zero errors. Format parsers should compile without issues.

# Common issues to watch for:
# - unused_imports -> remove unused crate imports
# - dead_code -> parser methods are used by tests and public API
# - Pattern matching errors -> ensure all Value variants are covered
```

### Level 2: Unit Tests (Component Validation)

```bash
# Test all format parsers
cargo test --package jin --lib merge::value::tests --verbose

# Test specific format parsers
cargo test --package jin --lib merge::value::tests::test_from_json --verbose
cargo test --package jin --lib merge::value::tests::test_from_yaml --verbose
cargo test --package jin --lib merge::value::tests::test_from_toml --verbose
cargo test --package jin --lib merge::value::tests::test_from_ini --verbose

# Run with output for debugging
cargo test --package jin --lib merge::value::tests -- --nocapture

# Expected: All tests pass.
# - test_from_json_valid: JSON parsing works
# - test_from_json_invalid: Parse error handling
# - test_from_yaml_valid: YAML parsing works
# - test_from_yaml_array: YAML sequence handling
# - test_from_toml_valid: TOML parsing works
# - test_from_toml_table: TOML table nesting
# - test_from_ini_valid: INI section parsing
```

### Level 3: Integration Testing (System Validation)

```bash
# Build the full project
cargo build --release

# Test format parsing through merge operations
cargo test --package jin test_merge_with_parsed_formats -- --exact

# Verify all formats can be merged together
cargo test --package jin test_cross_format_merge -- --exact

# Expected: Clean build, all formats parse correctly, cross-format merge works
```

### Level 4: Domain-Specific Validation

```bash
# Verify JSON number type handling
cargo test --package jin test_json_number_types -- --exact
# Asserts: Integer numbers -> MergeValue::Integer, Floats -> MergeValue::Float

# Verify YAML anchor/alias resolution
cargo test --package jin test_yaml_anchor_resolution -- --exact
# Asserts: Anchors and aliases are resolved during parsing

# Verify TOML datetime conversion
cargo test --package jin test_toml_datetime_conversion -- --exact
# Asserts: TOML datetime values -> MergeValue::String

# Verify INI nested structure
cargo test --package jin test_ini_section_nesting -- --exact
# Asserts: [section] key=value -> {"section": {"key": "value"}}

# Verify recursive parsing for nested structures
cargo test --package jin test_recursive_parsing -- --exact
# Asserts: Nested objects/arrays in all formats are fully parsed

# Expected: All format-specific behaviors are correct
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] All tests pass: `cargo test --package jin --lib merge::value::tests`
- [ ] No linting errors: `cargo clippy --package jin -- -D warnings`
- [ ] No formatting issues: `cargo fmt --check`
- [ ] All format parsers have doc comments
- [ ] Error handling uses correct `JinError` variants

### Feature Validation

- [ ] `from_json()` handles all JSON value types correctly
- [ ] `from_yaml()` handles YAML Mapping/Sequence correctly
- [ ] `from_toml()` converts datetime to String correctly
- [ ] `from_ini()` converts flat structure to nested objects correctly
- [ ] Nested structures are parsed recursively in all formats
- [ ] Parse errors return appropriate `JinError` variants
- [ ] Number type handling is correct (JSON i64/f64 check)
- [ ] Order preservation with `IndexMap` for all object variants

### Code Quality Validation

- [ ] Follows existing codebase patterns (error.rs, layer.rs)
- [ ] No code duplication (helper functions for recursion)
- [ ] Efficient conversion (avoid unnecessary re-serialization)
- [ ] Proper error propagation with `?` operator
- [ ] Comprehensive test coverage (happy path + edge cases)

### Documentation & Deployment

- [ ] Each parser has doc comment with usage example
- [ ] Format-specific gotchas are documented
- [ ] Error conditions are documented
- [ ] Integration with merge() method is clear

---

## Anti-Patterns to Avoid

- Don't use `HashMap` instead of `IndexMap` - order must be preserved
- Don't forget to handle YAML Mapping vs Object naming difference
- Don't treat TOML datetime as anything other than String
- Don't skip recursion for nested structures - must parse completely
- Don't forget INI global section handling (keys before first [section])
- Don't treat INI values as typed - all are strings
- Don't use Box<MergeValue> in collections - direct storage is fine
- Don't ignore YAML-specific variants - convert to String as fallback
- Don't skip error conversion - use `?` operator for automatic conversion

---

## Appendix: Quick Reference

### Format Parser API

```rust
// Parse JSON string
MergeValue::from_json(input: &str) -> Result<MergeValue>

// Parse YAML string
MergeValue::from_yaml(input: &str) -> Result<MergeValue>

// Parse TOML string
MergeValue::from_toml(input: &str) -> Result<MergeValue>

// Parse INI string
MergeValue::from_ini(input: &str) -> Result<MergeValue>
```

### Format Type Mapping

| Format | Source Type | Target MergeValue |
|--------|-------------|-------------------|
| JSON | Value::Null | Null |
| JSON | Value::Bool | Boolean |
| JSON | Value::Number | Integer (if is_i64) or Float (if is_f64) |
| JSON | Value::String | String |
| JSON | Value::Array | Array (recursive) |
| JSON | Value::Object | Object (recursive) |

| Format | Source Type | Target MergeValue |
|--------|-------------|-------------------|
| YAML | Value::Null | Null |
| YAML | Value::Bool | Boolean |
| YAML | Value::Number | Integer or Float |
| YAML | Value::String | String |
| YAML | Value::Sequence | Array (recursive) |
| YAML | Value::Mapping | Object (recursive) |
| YAML | Other variants | String (fallback) |

| Format | Source Type | Target MergeValue |
|--------|-------------|-------------------|
| TOML | Value::String | String |
| TOML | Value::Integer | Integer |
| TOML | Value::Float | Float |
| TOML | Value::Boolean | Boolean |
| TOML | Value::Datetime | String (dt.to_string()) |
| TOML | Value::Array | Array (recursive) |
| TOML | Value::Table | Object (recursive) |

| Format | Source Structure | Target MergeValue |
|--------|-----------------|-------------------|
| INI | [section] key=value | Object({"section": Object({"key": String})}) |
| INI | key=value (global) | Object({"global": Object({"key": String})}) |
| INI | # comment | (skipped) |
| INI | ; comment | (skipped) |

---

**PRP Version**: 1.0
**Last Updated**: 2025-12-26
**Confidence Score**: 10/10 - Implementation is complete and tested
**Status**: Format parsers are fully implemented in `src/merge/value.rs` (lines 90-346)
