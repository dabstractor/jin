# TOML 0.8 Crate API Research

Research Date: 2025-12-27
Source: https://docs.rs/toml/0.8/toml/
Crate Version: 0.8.x
License: MIT OR Apache-2.0

## Overview

The `toml` crate provides serde-compatible TOML parsing and serialization. It implements TOML 1.0.0 specification.

---

## 1. Value Enum Variants

```rust
pub enum Value {
    /// A string value
    String(String),

    /// A 64-bit signed integer
    Integer(i64),

    /// A 64-bit floating-point number
    Float(f64),

    /// A boolean value
    Boolean(bool),

    /// A TOML datetime
    Datetime(Datetime),

    /// An array of values (must be homogeneous)
    Array(Array),

    /// A table (key-value pairs)
    Table(Table),
}
```

**CRITICAL: No Null variant** - TOML does not support null values.

### Type Checking Methods
```rust
impl Value {
    pub fn is_str(&self) -> bool
    pub fn is_integer(&self) -> bool
    pub fn is_float(&self) -> bool
    pub fn is_bool(&self) -> bool
    pub fn is_datetime(&self) -> bool
    pub fn is_array(&self) -> bool
    pub fn is_table(&self) -> bool

    // Type description
    pub fn type_str(&self) -> &'static str
    pub fn same_type(&self, other: &Value) -> bool
}
```

### Accessor Methods
```rust
impl Value {
    pub fn as_str(&self) -> Option<&str>
    pub fn as_integer(&self) -> Option<i64>
    pub fn as_float(&self) -> Option<f64>
    pub fn as_bool(&self) -> Option<bool>
    pub fn as_datetime(&self) -> Option<&Datetime>
    pub fn as_array(&self) -> Option<&Array>
    pub fn as_array_mut(&mut self) -> Option<&mut Array>
    pub fn as_table(&self) -> Option<&Table>
    pub fn as_table_mut(&mut self) -> Option<&mut Table>

    // Indexing
    pub fn get<I>(&self, index: I) -> Option<&Value>
    pub fn get_mut<I>(&mut self, index: I) -> Option<&mut Value>
}
```

---

## 2. Table Type (BTreeMap-based)

### Type Definition
```rust
pub type Table = Map<String, Value>;

// Map is the underlying structure
pub struct Map<K, V> { /* private fields */ }
```

### Creation
```rust
use toml::Table;

let mut table = Table::new();
let table_with_capacity = Table::with_capacity(10);  // requires preserve_order feature
```

### Core Operations
```rust
impl Map<String, Value> {
    // Access
    pub fn get(&self, key: &str) -> Option<&Value>
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value>
    pub fn get_key_value(&self, key: &str) -> Option<(&String, &Value)>
    pub fn contains_key(&self, key: &str) -> bool

    // Modification
    pub fn insert(&mut self, key: String, value: Value) -> Option<Value>
    pub fn remove(&mut self, key: &str) -> Option<Value>
    pub fn clear(&mut self)
    pub fn retain<F>(&mut self, keep: F) where F: FnMut(&String, &mut Value) -> bool

    // Entry API
    pub fn entry(&mut self, key: String) -> Entry<String, Value>

    // Info
    pub fn len(&self) -> usize
    pub fn is_empty(&self) -> bool

    // Iteration
    pub fn iter(&self) -> Iter<String, Value>
    pub fn iter_mut(&mut self) -> IterMut<String, Value>
    pub fn keys(&self) -> Keys<String, Value>
    pub fn values(&self) -> Values<String, Value>
    pub fn values_mut(&mut self) -> ValuesMut<String, Value>
}
```

### Key Ordering Note
- Default: Uses BTreeMap (alphabetical ordering)
- With `preserve_order` feature: Uses IndexMap (insertion order)

---

## 3. Datetime Handling

### Type Definition
```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Datetime {
    pub date: Option<Date>,
    pub time: Option<Time>,
    pub offset: Option<Offset>,
}

pub struct Date {
    year: u16,
    month: u8,
    day: u8,
}

pub struct Time {
    hour: u8,
    minute: u8,
    second: u8,
    nanosecond: u32,
}

pub enum Offset {
    Z,                           // UTC
    Custom { hours: i8, minutes: i8 },
}
```

### Datetime Variants
| date | time | offset | TOML Type | Example |
|------|------|--------|-----------|---------|
| Some | Some | Some | Offset Date-Time | `1979-05-27T07:32:00Z` |
| Some | Some | None | Local Date-Time | `1979-05-27T07:32:00` |
| Some | None | None | Local Date | `1979-05-27` |
| None | Some | None | Local Time | `07:32:00` |

### String Conversion
```rust
// Datetime implements Display for RFC 3339 format
let dt: Datetime = /* ... */;
let s = dt.to_string();  // "1979-05-27T07:32:00Z"

// Parsing from string
let dt: Datetime = "1979-05-27T07:32:00Z".parse()?;
```

### Converting to MergeValue
```rust
// Since MergeValue has no Datetime, convert to String
toml::Value::Datetime(dt) => MergeValue::String(dt.to_string())
```

---

## 4. Error Types

### Error Structure
```rust
pub struct Error {
    inner: Box<Inner>,
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {}
impl std::fmt::Debug for Error {}
```

### Error Handling Pattern
```rust
use toml::{from_str, Table};

let toml_str = "invalid = [1, \"mixed\", 2.5]";  // Mixed types - error!

match from_str::<Table>(toml_str) {
    Ok(table) => println!("Parsed successfully"),
    Err(e) => {
        eprintln!("Parse error: {}", e);
        // Error message includes line/column info
    }
}
```

### Common Error Messages
```
"expected newline, found an identifier at line 1 column 5"
"the type of this value should be [string] at line 2 column 10"
"duplicate key 'name' at line 3 column 1"
"the type of this value should be [integer] at line 5 column 9"
```

### Converting to JinError
```rust
fn parse_toml(content: &str) -> Result<MergeValue, JinError> {
    let value: toml::Value = toml::from_str(content)
        .map_err(|e| JinError::Parse {
            format: "TOML".to_string(),
            message: e.to_string(),
        })?;

    Ok(MergeValue::from(value))
}
```

---

## 5. Serialization

### Functions
```rust
// Compact output
pub fn to_string<T: Serialize>(value: &T) -> Result<String, Error>

// Pretty output (requires 'display' feature)
pub fn to_string_pretty<T: Serialize>(value: &T) -> Result<String, Error>
```

### Usage
```rust
use toml::{to_string_pretty, Table, Value};

let mut table = Table::new();
table.insert("name".into(), Value::String("test".into()));
table.insert("count".into(), Value::Integer(42));

let output = to_string_pretty(&table)?;
// Output:
// name = "test"
// count = 42
```

### Nested Tables
```rust
let mut config = Table::new();
let mut database = Table::new();
database.insert("host".into(), Value::String("localhost".into()));
database.insert("port".into(), Value::Integer(5432));
config.insert("database".into(), Value::Table(database));

let output = to_string_pretty(&config)?;
// Output:
// [database]
// host = "localhost"
// port = 5432
```

---

## 6. Key Differences from JSON/YAML

### No Null Type
```rust
// TOML has NO null - must handle in TryFrom
impl TryFrom<MergeValue> for toml::Value {
    type Error = JinError;

    fn try_from(value: MergeValue) -> Result<Self, Self::Error> {
        match value {
            MergeValue::Null => Err(JinError::Parse {
                format: "TOML".to_string(),
                message: "TOML does not support null values".to_string(),
            }),
            // ... other conversions
        }
    }
}
```

### Homogeneous Arrays Only
```toml
# VALID - all same type
integers = [1, 2, 3]
strings = ["a", "b", "c"]

# INVALID - mixed types
mixed = [1, "string", 2.5]  # ERROR!
```

### Native Datetime Type
```toml
# TOML has native datetime support
created = 1979-05-27T07:32:00Z
birthday = 1990-06-15
alarm = 07:30:00
```

---

## 7. Complete Conversion Example

```rust
use toml;
use indexmap::IndexMap;
use crate::core::JinError;

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

// Convert toml::Value to MergeValue (always succeeds)
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
                // Note: toml::Table is BTreeMap, so keys are alphabetically sorted
                let obj: IndexMap<String, MergeValue> = table
                    .into_iter()
                    .map(|(k, v)| (k, MergeValue::from(v)))
                    .collect();
                MergeValue::Object(obj)
            }
        }
    }
}

// Convert MergeValue to toml::Value (may fail on null)
impl TryFrom<MergeValue> for toml::Value {
    type Error = JinError;

    fn try_from(value: MergeValue) -> Result<Self, Self::Error> {
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
                let converted: Result<Vec<toml::Value>, _> = arr
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
```

---

## 8. Feature Flags

```toml
[dependencies]
# Default features
toml = "0.8"

# All features
toml = { version = "0.8", features = ["parse", "display", "preserve_order"] }
```

| Feature | Description |
|---------|-------------|
| `parse` | Enable `from_str()` deserialization |
| `display` | Enable `to_string_pretty()` |
| `preserve_order` | Use IndexMap for insertion order |

---

## References

- Documentation: https://docs.rs/toml/0.8/toml/
- GitHub: https://github.com/toml-rs/toml
- TOML Specification: https://toml.io/en/v1.0.0
