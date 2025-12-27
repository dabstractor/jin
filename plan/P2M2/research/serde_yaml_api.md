# serde_yaml 0.9 API Research

Research Date: 2025-12-27
Source: https://docs.rs/serde_yaml/0.9/serde_yaml/
Crate Version: 0.9.x
License: MIT OR Apache-2.0
Status: No longer maintained (consider yaml-rust2 for new projects)

## Overview

The `serde_yaml` crate provides YAML serialization and deserialization using serde. It supports all YAML 1.2 features and integrates seamlessly with Rust's serde ecosystem.

---

## 1. Value Enum Variants

```rust
pub enum Value {
    /// Represents a YAML null value
    Null,

    /// Represents a YAML boolean
    Bool(bool),

    /// Represents a YAML number (integer or float)
    Number(Number),

    /// Represents a YAML string
    String(String),

    /// Represents a YAML sequence (array/list)
    Sequence(Sequence),

    /// Represents a YAML mapping (object/dict)
    Mapping(Mapping),

    /// Represents a tagged value (!Tag value)
    Tagged(Box<TaggedValue>),
}
```

### Type Checking Methods
```rust
impl Value {
    pub fn is_null(&self) -> bool
    pub fn is_bool(&self) -> bool
    pub fn is_number(&self) -> bool
    pub fn is_i64(&self) -> bool
    pub fn is_u64(&self) -> bool
    pub fn is_f64(&self) -> bool
    pub fn is_string(&self) -> bool
    pub fn is_sequence(&self) -> bool
    pub fn is_mapping(&self) -> bool
}
```

### Accessor Methods
```rust
impl Value {
    pub fn as_null(&self) -> Option<()>
    pub fn as_bool(&self) -> Option<bool>
    pub fn as_i64(&self) -> Option<i64>
    pub fn as_u64(&self) -> Option<u64>
    pub fn as_f64(&self) -> Option<f64>
    pub fn as_str(&self) -> Option<&str>
    pub fn as_sequence(&self) -> Option<&Sequence>
    pub fn as_sequence_mut(&mut self) -> Option<&mut Sequence>
    pub fn as_mapping(&self) -> Option<&Mapping>
    pub fn as_mapping_mut(&mut self) -> Option<&mut Mapping>
}
```

---

## 2. Type Conversions

### To Value (Serialize)
```rust
use serde_yaml::{to_value, Value};
use serde::Serialize;

#[derive(Serialize)]
struct Config {
    name: String,
    count: i32,
}

let config = Config { name: "test".into(), count: 42 };
let value: Value = to_value(&config)?;
```

### From Value (Deserialize)
```rust
use serde_yaml::{from_value, Value};
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    name: String,
    count: i32,
}

let value = Value::Mapping(/* ... */);
let config: Config = from_value(value)?;
```

### Indexing Access
```rust
use serde_yaml::Value;

let yaml = serde_yaml::from_str::<Value>("key: value")?;

// Bracket access (returns Value::Null if missing)
let val = &yaml["key"];

// Safe access with get()
if let Some(val) = yaml.get("key") {
    println!("{}", val.as_str().unwrap());
}

// Mutable access
if let Some(val) = yaml.get_mut("key") {
    *val = Value::String("new_value".to_string());
}
```

---

## 3. Mapping Structure

### Type Definition
```rust
// Mapping is backed by IndexMap for order preservation
pub struct Mapping {
    // Uses indexmap::IndexMap internally
}
```

### Creation
```rust
use serde_yaml::Mapping;

let mut map = Mapping::new();
let map_with_capacity = Mapping::with_capacity(10);
```

### Core Operations
```rust
impl Mapping {
    // Insert/Remove
    pub fn insert(&mut self, k: Value, v: Value) -> Option<Value>
    pub fn remove(&mut self, k: &Value) -> Option<Value>
    pub fn shift_remove(&mut self, k: &Value) -> Option<Value>  // Preserves order
    pub fn clear(&mut self)

    // Access
    pub fn get(&self, k: &Value) -> Option<&Value>
    pub fn get_mut(&mut self, k: &Value) -> Option<&mut Value>
    pub fn contains_key(&self, k: &Value) -> bool

    // Entry API
    pub fn entry(&mut self, k: Value) -> Entry

    // Info
    pub fn len(&self) -> usize
    pub fn is_empty(&self) -> bool
    pub fn capacity(&self) -> usize
}
```

### Iteration
```rust
impl Mapping {
    pub fn iter(&self) -> Iter<'_>
    pub fn iter_mut(&mut self) -> IterMut<'_>
    pub fn keys(&self) -> Keys<'_>
    pub fn values(&self) -> Values<'_>
    pub fn values_mut(&mut self) -> ValuesMut<'_>
    pub fn into_keys(self) -> IntoKeys
    pub fn into_values(self) -> IntoValues
}
```

### Usage Example
```rust
use serde_yaml::{Mapping, Value};

let mut map = Mapping::new();

// Insert with Value keys
map.insert(
    Value::String("name".to_string()),
    Value::String("test".to_string())
);

// Iterate
for (key, value) in map.iter() {
    if let (Value::String(k), Value::String(v)) = (key, value) {
        println!("{} = {}", k, v);
    }
}
```

---

## 4. Number Handling

### Number Type
```rust
pub struct Number {
    // Internal representation
}
```

### Creation
```rust
use serde_yaml::Number;

// From integers
let n: Number = 42.into();
let n: Number = 42i64.into();
let n: Number = 42u64.into();

// From floats
let n: Number = 3.14f64.into();

// From string (parsing)
let n: Number = "42".parse()?;
let n: Number = "3.14".parse()?;
```

### Type Checking
```rust
impl Number {
    pub fn is_i64(&self) -> bool
    pub fn is_u64(&self) -> bool
    pub fn is_f64(&self) -> bool
    pub fn is_nan(&self) -> bool
    pub fn is_infinite(&self) -> bool
    pub fn is_finite(&self) -> bool
}
```

### Extraction
```rust
impl Number {
    pub fn as_i64(&self) -> Option<i64>
    pub fn as_u64(&self) -> Option<u64>
    pub fn as_f64(&self) -> Option<f64>
}
```

### Usage in MergeValue Conversion
```rust
use serde_yaml::Value;

fn convert_number(n: &serde_yaml::Number) -> MergeValue {
    // Try integer first to preserve integer semantics
    if let Some(i) = n.as_i64() {
        MergeValue::Integer(i)
    } else if let Some(f) = n.as_f64() {
        MergeValue::Float(f)
    } else {
        // Fallback for unsigned integers that don't fit i64
        MergeValue::Float(n.as_f64().unwrap_or(0.0))
    }
}
```

---

## 5. Tagged Values Handling

### TaggedValue Structure
```rust
pub struct TaggedValue {
    pub tag: Tag,
    pub value: Value,
}

pub struct Tag(String);
```

YAML supports custom tags like `!Foo value` for enums and custom types.

### Example YAML with Tags
```yaml
# Tagged values in YAML
value: !CustomType
  field: data

# Enum representation
enum_val: !VariantName
  inner_field: value
```

### Handling in Conversion
```rust
use serde_yaml::Value;

fn convert_yaml_value(value: serde_yaml::Value) -> MergeValue {
    match value {
        serde_yaml::Value::Tagged(tagged) => {
            // Extract the inner value, discarding the tag
            // For Jin's purposes, we treat tagged values as their inner value
            convert_yaml_value(tagged.value)
        }
        // ... other variants
    }
}
```

### Creating Tagged Values
```rust
use serde_yaml::{Value, value::{TaggedValue, Tag}};

let tagged = Value::Tagged(Box::new(TaggedValue {
    tag: Tag::new("!CustomType"),
    value: Value::String("data".to_string()),
}));
```

---

## 6. Error Handling

### Error Type
```rust
pub struct Error {
    // Opaque error type
}

impl Error {
    pub fn location(&self) -> Option<Location>
}

pub struct Location {
    // Line and column info
}

impl Location {
    pub fn line(&self) -> usize
    pub fn column(&self) -> usize
}
```

### Error Handling Pattern
```rust
use serde_yaml::{from_str, Value};

let yaml_str = "invalid: [yaml: content";

match from_str::<Value>(yaml_str) {
    Ok(value) => {
        // Process value
    }
    Err(e) => {
        eprintln!("YAML error: {}", e);
        if let Some(loc) = e.location() {
            eprintln!("  at line {}, column {}", loc.line(), loc.column());
        }
    }
}
```

### Converting to JinError
```rust
use serde_yaml;
use crate::core::JinError;

fn parse_yaml(content: &str) -> Result<MergeValue, JinError> {
    let value: serde_yaml::Value = serde_yaml::from_str(content)
        .map_err(|e| {
            let location_info = e.location()
                .map(|l| format!(" at line {}, column {}", l.line(), l.column()))
                .unwrap_or_default();

            JinError::Parse {
                format: "YAML".to_string(),
                message: format!("{}{}", e, location_info),
            }
        })?;

    Ok(MergeValue::from(value))
}
```

---

## 7. Module Functions

### Serialization
```rust
// To string
pub fn to_string<T: Serialize>(value: &T) -> Result<String, Error>

// To writer
pub fn to_writer<W: Write, T: Serialize>(writer: W, value: &T) -> Result<(), Error>

// To Value
pub fn to_value<T: Serialize>(value: &T) -> Result<Value, Error>
```

### Deserialization
```rust
// From string
pub fn from_str<'de, T: Deserialize<'de>>(s: &'de str) -> Result<T, Error>

// From reader
pub fn from_reader<R: Read, T: DeserializeOwned>(reader: R) -> Result<T, Error>

// From slice
pub fn from_slice<'de, T: Deserialize<'de>>(slice: &'de [u8]) -> Result<T, Error>

// From Value
pub fn from_value<T: DeserializeOwned>(value: Value) -> Result<T, Error>
```

---

## 8. Complete Conversion Example

```rust
use serde_yaml;
use indexmap::IndexMap;

// MergeValue enum (from jin)
pub enum MergeValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<MergeValue>),
    Object(IndexMap<String, MergeValue>),
}

// Convert serde_yaml::Value to MergeValue
impl From<serde_yaml::Value> for MergeValue {
    fn from(value: serde_yaml::Value) -> Self {
        match value {
            serde_yaml::Value::Null => MergeValue::Null,
            serde_yaml::Value::Bool(b) => MergeValue::Bool(b),
            serde_yaml::Value::Number(n) => {
                // Try integer first to preserve integer semantics
                if let Some(i) = n.as_i64() {
                    MergeValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    MergeValue::Float(f)
                } else {
                    // Fallback for unsigned integers
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
                        // YAML keys can be non-strings; only support string keys
                        k.as_str().map(|s| (s.to_string(), MergeValue::from(v)))
                    })
                    .collect();
                MergeValue::Object(obj)
            }
            // Handle tagged values by extracting inner value
            serde_yaml::Value::Tagged(tagged) => MergeValue::from(tagged.value),
        }
    }
}

// Convert MergeValue to serde_yaml::Value
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
```

---

## 9. Important Notes

### Maintenance Status
The `serde_yaml` crate is no longer actively maintained. For new projects, consider:
- `yaml-rust2` - Fork of yaml-rust with active maintenance
- Continue using `serde_yaml` for stability in existing projects

### Key Characteristics
- Uses `unsafe-libyaml` for YAML parsing
- Maintains insertion order for mappings (uses indexmap)
- Supports YAML merge key (`<<`) via `apply_merge()` method
- Full serde integration

### Gotchas
1. **Non-string keys**: YAML allows any value as a key, but Jin only supports string keys
2. **Tagged values**: Must be handled explicitly (extract inner value)
3. **Float edge cases**: NaN, Infinity require special handling
4. **Mapping order**: Preserved via indexmap, unlike JSON's HashMap

---

## References

- Documentation: https://docs.rs/serde_yaml/0.9/serde_yaml/
- GitHub: https://github.com/dtolnay/serde-yaml
- YAML 1.2 Spec: https://yaml.org/spec/1.2.2/
