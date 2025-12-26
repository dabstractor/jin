# Serde Value Types Research

## Overview

This research covers the three main serde crates for handling structured data formats in Rust:
- `serde_json` - JSON format
- `serde_yaml_ng` - YAML format
- `toml` - TOML format

Each crate provides a `Value` enum that can represent any valid document of that format.

## 1. serde_json::Value API

### Type Definition
```rust
use serde_json::Value;

// Value enum variants:
enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<Value>),
    Object(Map<String, Value>),
}
```

### Parsing Examples
```rust
use serde_json::{json, Value};

// Parse from string
let json_str = r#"{ "name": "John", "age": 30 }"#;
let value: Value = serde_json::from_str(json_str)?;

// Parse from bytes
let json_bytes = br#"{"active": true}"#;
let value: Value = serde_json::from_slice(json_bytes)?;

// Create directly
let mut value = json!({
    "name": "John",
    "age": 30,
    "hobbies": ["reading", "coding"]
});
```

### Key API Methods
```rust
// Access values
if let Some(name) = value.get("name").and_then(|v| v.as_str()) {
    println!("Name: {}", name);
}

// Mutable access
if let Some(age) = value.get_mut("age").and_then(|v| v.as_mut_i64()) {
    *age = 31;
}

// JSON Pointer syntax
let name = value.pointer("/name").unwrap();

// Check types
if value.is_object() {
    println!("It's an object");
}
if value.is_array() {
    println!("It's an array");
}

// Convert to specific types
let name: String = value["name"].as_str().unwrap().to_string();
let age: i64 = value["age"].as_i64().unwrap();
```

### Merge Operations
```rust
// Basic merge (manual implementation)
fn merge_json(base: &mut Value, update: &Value) {
    match (base, update) {
        (Value::Object(ref mut base_map), Value::Object(update_map)) => {
            for (key, value) in update_map {
                merge_json(base_map.entry(key.clone()).or_insert(Value::Null), value);
            }
        }
        (_, b) => *base = b.clone(),
    }
}

// Usage
let mut config = json!({"database": {"host": "localhost", "port": 5432}});
let update = json!({"database": {"port": 5433, "ssl": true}});
merge_json(&mut config, &update);

// Simple key replacement (no deep merge)
fn simple_merge(mut base: Value, update: Value) -> Value {
    if let (Value::Object(mut base_obj), Value::Object(update_obj)) = (&mut base, &update) {
        base_obj.extend(update_obj);
        base
    } else {
        update
    }
}
```

## 2. serde_yaml_ng::Value API

### Type Definition
```rust
use serde_yaml_ng::Value;

// Value enum variants:
enum Value {
    Null,
    Bool(bool),
    Number(Number),  // Can be Integer or Float
    String(String),
    Sequence(Vec<Value>),
    Mapping(serde_yaml_ng::Mapping),  // BTreeMap<String, Value>
}
```

### Parsing Examples
```rust
use serde_yaml_ng::Value;

// Parse from string
let yaml_str = r#"
name: John
age: 30
hobbies:
  - reading
  - coding
"#;
let value: Value = serde_yaml_ng::from_str(yaml_str)?;

// Parse from file
use std::fs::File;
use std::io::BufReader;
let file = File::open("config.yaml")?;
let reader = BufReader::new(file);
let value: Value = serde_yaml_ng::from_reader(reader)?;
```

### Key API Methods
```rust
// Access values (similar to JSON)
let name = value["name"].as_str().unwrap();
let age = value["age"].as_i64().unwrap();
let hobbies = value["hobbies"].as_sequence().unwrap();

// Check types
if value.is_mapping() {
    println!("It's a mapping (object)");
}
if value.is_sequence() {
    println!("It's a sequence (array)");
}

// Iteration over mappings
if let Some(mapping) = value.as_mapping() {
    for (key, value) in mapping {
        println!("{}: {:?}", key.as_str().unwrap(), value);
    }
}
```

### YAML-Specific Considerations
- YAML mappings preserve insertion order in newer versions
- Supports anchors and aliases (which need special handling for merging)
- Multiline strings and complex data types

## 3. toml::Value API

### Type Definition
```rust
use toml::Value;

// Value enum variants:
enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Datetime(DateTime),
    Array(Vec<Value>),
    Table(Map<String, Value>),
}
```

### Parsing Examples
```rust
use toml::Value;

// Parse from string
let toml_str = r#"
[database]
host = "localhost"
port = 5432
"#;
let value: Value = toml::from_str(toml_str)?;

// Parse from file
use std::fs;
let content = fs::read_to_string("config.toml")?;
let value: Value = content.parse()?;

// Create directly
let mut value = toml::value::Table::new();
value.insert("host".to_string(), toml::Value::String("localhost".to_string()));
value.insert("port".to_string(), toml::Value::Integer(5432));
```

### Key API Methods
```rust
// Access values
let host = value["database"]["host"].as_str().unwrap();
let port = value["database"]["port"].as_integer().unwrap();

// Check types
if value.is_table() {
    println!("It's a table (object)");
}
if value.is_array() {
    println!("It's an array");
}

// Safe unwrapping with Option
if let Some(table) = value.as_table() {
    for (key, value) in table {
        println!("{}: {:?}", key, value);
    }
}
```

### TOML-Specific Considerations
- Strict typing rules enforced by TOML specification
- Datetime support for RFC 3339 formats
- Nested tables and inline arrays

## 4. Format Conversion

### JSON to YAML
```rust
use serde_json::Value as JsonValue;
use serde_yaml_ng::Value as YamlValue;

fn json_to_yaml(json: JsonValue) -> YamlValue {
    serde_yaml_ng::from_value(
        serde_yaml_ng::to_value(json).unwrap()
    ).unwrap()
}
```

### YAML to TOML
```rust
use serde_yaml_ng::Value as YamlValue;
use toml::Value as TomlValue;

fn yaml_to_toml(yaml: YamlValue) -> Result<TomlValue, String> {
    let json_string = serde_json::to_string(&yaml).map_err(|e| e.to_string())?;
    let json_value: JsonValue = serde_json::from_str(&json_string).map_err(|e| e.to_string())?;
    let toml_value: TomlValue = toml::from_str(&serde_json::to_string(&json_value).unwrap()).unwrap();
    Ok(toml_value)
}
```

## 5. Deep Merge Patterns

### Generic Deep Merge Function
```rust
use serde_json::Value;

fn deep_merge(base: &mut Value, update: &Value) {
    match (base, update) {
        // Both objects - merge recursively
        (Value::Object(ref mut base_obj), Value::Object(update_obj)) => {
            for (key, value) in update_obj {
                deep_merge(base_obj.entry(key.clone()).or_insert(Value::Null), value);
            }
        }
        // Both arrays - concatenate or replace based on strategy
        (Value::Array(ref mut base_arr), Value::Array(update_arr)) => {
            // Strategy 1: Concatenate arrays
            base_arr.extend(update_arr.clone());

            // Strategy 2: Replace arrays
            // *base_arr = update_arr.clone();
        }
        // Otherwise - replace base with update
        (_, b) => *base = b.clone(),
    }
}
```

### Advanced Merge with Strategy
```rust
enum MergeStrategy {
    Recursive,      // Deep merge objects, concatenate arrays
    Replace,       // Completely replace
    ArrayConcat,   // Concatenate arrays, deep merge objects
    ArrayOverride, // Override arrays with new values
}

fn merge_with_strategy(base: &mut Value, update: &Value, strategy: MergeStrategy) {
    use serde_json::Value::*;

    match (base, update, strategy) {
        (Object(base_obj), Object(update_obj), MergeStrategy::Recursive) => {
            for (key, value) in update_obj {
                merge_with_strategy(
                    base_obj.entry(key.clone()).or_insert(Null),
                    value,
                    MergeStrategy::Recursive
                );
            }
        }
        (Object(base_obj), Object(update_obj), MergeStrategy::ArrayConcat) => {
            for (key, value) in update_obj {
                if !base_obj.contains_key(&key) {
                    base_obj.insert(key.clone(), value.clone());
                } else {
                    merge_with_strategy(
                        base_obj.get_mut(&key).unwrap(),
                        value,
                        MergeStrategy::ArrayConcat
                    );
                }
            }
        }
        (_, b, _) => *base = b.clone(),
    }
}
```

## 6. Using IndexMap for Ordered JSON

### Why IndexMap?
```rust
use indexmap::IndexMap;
use serde_json::Value;

// Create ordered JSON object
let mut ordered = IndexMap::new();
ordered.insert("z", Value::String("last".to_string()));
ordered.insert("a", Value::String("first".to_string()));

// Convert to Value
let value = Value::Object(ordered.into_iter().collect());
```

### Merge with Order Preservation
```rust
fn ordered_merge(base: &mut IndexMap<String, Value>, update: &IndexMap<String, Value>) {
    for (key, value) in update {
        if base.contains_key(&key) {
            // Recursively merge if both are objects
            if let (Some(Value::Object(base_obj)), Value::Object(update_obj)) =
                (base.get(&key), value.as_object()) {
                ordered_merge(base_obj, update_obj);
            } else {
                base.insert(key.clone(), value.clone());
            }
        } else {
            base.insert(key.clone(), value.clone());
        }
    }
}
```

## 7. Complete Usage Example

### Multi-format Configuration Merger
```rust
use serde_json::json;
use serde_yaml_ng::Value as YamlValue;
use toml::Value as TomlValue;

pub struct ConfigMerger;

impl ConfigMerger {
    pub fn merge_json_configs(base: &str, update: &str) -> Result<String, String> {
        let base_value: Value = serde_json::from_str(base)
            .map_err(|e| format!("Failed to parse base config: {}", e))?;
        let mut base_value = base_value;
        let update_value: Value = serde_json::from_str(update)
            .map_err(|e| format!("Failed to parse update config: {}", e))?;

        deep_merge(&mut base_value, &update_value);
        serde_json::to_string_pretty(&base_value)
            .map_err(|e| format!("Failed to serialize merged config: {}", e))
    }

    pub fn convert_yaml_to_json(yaml_str: &str) -> Result<String, String> {
        let yaml_value: YamlValue = serde_yaml_ng::from_str(yaml_str)
            .map_err(|e| format!("Failed to parse YAML: {}", e))?;
        let json_value: Value = serde_json::from_value(yaml_value)
            .map_err(|e| format!("Failed to convert to JSON: {}", e))?;
        serde_json::to_string_pretty(&json_value)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))
    }
}

// Usage
fn main() {
    let base_config = r#"
    {
        "database": {
            "host": "localhost",
            "port": 5432,
            "pool_size": 10
        },
        "features": ["auth", "logging"]
    }
    "#;

    let update_config = r#"
    {
        "database": {
            "port": 5433,
            "ssl": true
        },
        "features": ["auth", "logging", "caching"],
        "new_feature": true
    }
    "#;

    match ConfigMerger::merge_json_configs(base_config, update_config) {
        Ok(merged) => println!("Merged config:\n{}", merged),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## 8. Performance Considerations

### Best Practices
1. **Parse once, use multiple times** - Keep parsed Values around rather than re-parsing
2. **Minimize cloning** - Use `&Value` references where possible
3. **Consider borrowing** - For large structures, consider borrowing patterns
4. **Lazy evaluation** - Only parse sections you need when dealing with large configs

### Benchmarks
- `serde_json` is generally the fastest for JSON operations
- `serde_yaml_ng` may be slower due to YAML's complexity
- `toml` is good for small to medium configurations
- IndexMap adds overhead but preserves order

## 9. Error Handling Patterns

### Robust Error Types
```rust
use serde::{de, Deserialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("YAML parse error: {0}")]
    YamlError(#[from] serde_yaml_ng::Error),

    #[error("TOML parse error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("Merge error at path {path}: {message}")]
    MergeError { path: String, message: String },

    #[error("Type mismatch: expected {expected}, found {actual}")]
    TypeMismatch { expected: String, actual: String },
}

impl ConfigMerger {
    pub fn safe_merge(base: &mut Value, update: &Value, path: &str) -> Result<(), ConfigError> {
        match (base, update) {
            (Value::Object(base_obj), Value::Object(update_obj)) => {
                for (key, value) in update_obj {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    if let Some(base_value) = base_obj.get_mut(&key) {
                        safe_merge(base_value, value, &new_path)?;
                    } else {
                        base_obj.insert(key.clone(), value.clone());
                    }
                }
                Ok(())
            }
            (Value::Array(base_arr), Value::Array(update_arr)) => {
                base_arr.extend(update_arr.clone());
                Ok(())
            }
            (base_val, update_val) => {
                if std::mem::discriminant(base_val) != std::mem::discriminant(update_val) {
                    Err(ConfigError::TypeMismatch {
                        expected: format!("{:?}", std::mem::discriminant(base_val)),
                        actual: format!("{:?}", std::mem::discriminant(update_val)),
                    })
                } else {
                    *base_val = update_val.clone();
                    Ok(())
                }
            }
        }
    }
}
```

## 10. Testing Strategies

### Unit Tests for Merge Functions
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deep_merge() {
        let mut base = json!({
            "database": {
                "host": "localhost",
                "port": 5432,
                "pool": {
                    "min": 2,
                    "max": 10
                }
            },
            "features": ["auth", "logging"]
        });

        let update = json!({
            "database": {
                "port": 5433,
                "ssl": true,
                "pool": {
                    "max": 20
                }
            },
            "features": ["auth", "logging", "caching"]
        });

        deep_merge(&mut base, &update);

        assert_eq!(base["database"]["host"], "localhost");
        assert_eq!(base["database"]["port"], 5433);
        assert_eq!(base["database"]["ssl"], true);
        assert_eq!(base["database"]["pool"]["max"], 20);
        assert_eq!(base["database"]["pool"]["min"], 2);

        let features: Vec<&str> = base["features"].as_array().unwrap()
            .iter().map(|v| v.as_str().unwrap()).collect();
        assert_eq!(features, vec!["auth", "logging", "caching"]);
    }

    #[test]
    fn test_type_merging() {
        let mut base = json!({"value": 42});
        let update = json!({"value": "string"});

        assert!(matches!(safe_merge(&mut base, &update, "value"), Err(ConfigError::TypeMismatch { .. })));
    }
}
```

## Documentation Links

- [serde_json documentation](https://docs.rs/serde_json/)
- [serde_yaml_ng documentation](https://docs.rs/serde_yaml_ng/)
- [toml crate documentation](https://docs.rs/toml/)
- [indexmap documentation](https://docs.rs/indexmap/)
- [serde documentation](https://docs.rs/serde/)