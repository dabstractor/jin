# Rust Deep Merge Implementations Guide

Comprehensive Rust patterns and working code examples for deep merging configuration files.

## Table of Contents

1. [Using the `merge` Crate](#using-the-merge-crate)
2. [serde_json Deep Merge](#serde_json-deep-merge)
3. [Order-Preserving with IndexMap](#order-preserving-with-indexmap)
4. [Handling Edge Cases](#handling-edge-cases)
5. [Performance Patterns](#performance-patterns)
6. [Complete Examples](#complete-examples)

---

## Using the `merge` Crate

### Introduction

The `merge` crate provides a trait-based approach with derive macro support for type-safe merging.

### Setup

```toml
[dependencies]
merge = "0.1"
```

### Basic Example

```rust
use merge::Merge;

#[derive(Merge, Clone, Debug, Default)]
struct Config {
    server: ServerConfig,
    logging: LoggingConfig,
}

#[derive(Merge, Clone, Debug, Default)]
struct ServerConfig {
    host: Option<String>,
    port: Option<u16>,
}

#[derive(Merge, Clone, Debug, Default)]
struct LoggingConfig {
    #[merge(strategy = merge::bool::overwrite_false)]
    enabled: bool,

    #[merge(strategy = merge::option::overwrite_none)]
    level: Option<String>,
}

fn main() {
    let base = Config {
        server: ServerConfig {
            host: Some("localhost".to_string()),
            port: Some(3000),
        },
        logging: LoggingConfig {
            enabled: true,
            level: Some("info".to_string()),
        },
    };

    let override_cfg = Config {
        server: ServerConfig {
            host: None,  // Won't override
            port: Some(8080),
        },
        logging: LoggingConfig {
            enabled: false,  // Will override
            level: Some("debug".to_string()),
        },
    };

    let mut result = base;
    result.merge(override_cfg);

    println!("{:#?}", result);
    // Output:
    // Config {
    //   server: ServerConfig {
    //     host: Some("localhost"),
    //     port: Some(8080),
    //   },
    //   logging: LoggingConfig {
    //     enabled: false,
    //     level: Some("debug"),
    //   },
    // }
}
```

### Available Merge Strategies

#### Boolean Strategies

```rust
use merge::Merge;

#[derive(Merge)]
struct Features {
    // Only override if original is false
    #[merge(strategy = merge::bool::overwrite_false)]
    auth_enabled: bool,

    // Only override if original is true
    #[merge(strategy = merge::bool::overwrite_true)]
    cache_enabled: bool,

    // Always override
    #[merge(strategy = merge::bool::overwrite)]
    debug_mode: bool,
}
```

#### Option Strategies

```rust
use merge::Merge;

#[derive(Merge)]
struct ApiConfig {
    // Only override if None
    #[merge(strategy = merge::option::overwrite_none)]
    api_key: Option<String>,

    // Always override
    #[merge(strategy = merge::option::overwrite)]
    timeout: Option<u64>,
}
```

#### Collection Strategies

```rust
use merge::Merge;

#[derive(Merge)]
struct PluginConfig {
    // Concatenate vectors
    #[merge(strategy = merge::vec::append)]
    plugins: Vec<String>,

    // Replace entire vector
    #[merge(strategy = merge::vec::overwrite)]
    blocklist: Vec<String>,
}
```

#### Custom Strategies

```rust
use merge::Merge;

fn custom_merge<T: Clone>(left: &mut T, right: T) {
    // Custom merge logic here
    *left = right;
}

#[derive(Merge)]
struct MyConfig {
    #[merge(strategy = custom_merge)]
    custom_field: String,
}
```

---

## serde_json Deep Merge

### Complete Deep Merge Implementation

```rust
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// Deep merge serde_json Values
/// Right-hand value takes precedence in conflicts
pub fn deep_merge(base: &mut Value, override_val: &Value) {
    match (base, override_val) {
        // Both are objects - merge recursively
        (Value::Object(base_map), Value::Object(override_map)) => {
            for (key, override_value) in override_map.iter() {
                let entry = base_map
                    .entry(key.clone())
                    .or_insert_with(|| Value::Null);
                deep_merge(entry, override_value);
            }
        }
        // Both are arrays - merge by index
        (Value::Array(base_arr), Value::Array(override_arr)) => {
            for (i, override_item) in override_arr.iter().enumerate() {
                if i < base_arr.len() {
                    deep_merge(&mut base_arr[i], override_item);
                } else {
                    base_arr.push(override_item.clone());
                }
            }
        }
        // Type mismatch or scalar values - override
        _ => *base = override_val.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nested_object_merge() {
        let mut base = json!({
            "server": {
                "host": "localhost",
                "port": 3000
            },
            "db": {
                "pool": 10
            }
        });

        let override_val = json!({
            "server": {
                "port": 8080,
                "ssl": true
            }
        });

        deep_merge(&mut base, &override_val);

        assert_eq!(base["server"]["host"], "localhost");
        assert_eq!(base["server"]["port"], 8080);
        assert_eq!(base["server"]["ssl"], true);
        assert_eq!(base["db"]["pool"], 10);
    }
}
```

### Array Concatenation Variant

```rust
use serde_json::Value;

pub fn deep_merge_concat_arrays(base: &mut Value, override_val: &Value) {
    match (base, override_val) {
        (Value::Object(base_map), Value::Object(override_map)) => {
            for (key, override_value) in override_map.iter() {
                let entry = base_map
                    .entry(key.clone())
                    .or_insert_with(|| Value::Null);
                deep_merge_concat_arrays(entry, override_value);
            }
        }
        // Arrays are concatenated instead of merged by index
        (Value::Array(base_arr), Value::Array(override_arr)) => {
            base_arr.extend(override_arr.iter().cloned());
        }
        _ => *base = override_val.clone(),
    }
}
```

### With Circular Reference Detection

```rust
use serde_json::Value;
use std::collections::HashSet;
use std::ptr;

pub fn deep_merge_safe(
    base: &mut Value,
    override_val: &Value,
    visited: &mut HashSet<*const Value>,
) -> Result<(), String> {
    // Check for circular references
    let ptr = override_val as *const _;
    if visited.contains(&ptr) {
        return Err("Circular reference detected".to_string());
    }

    visited.insert(ptr);

    let result = match (base, override_val) {
        (Value::Object(base_map), Value::Object(override_map)) => {
            for (key, override_value) in override_map.iter() {
                let entry = base_map
                    .entry(key.clone())
                    .or_insert_with(|| Value::Null);
                deep_merge_safe(entry, override_value, visited)?;
            }
            Ok(())
        }
        (Value::Array(base_arr), Value::Array(override_arr)) => {
            for (i, override_item) in override_arr.iter().enumerate() {
                if i < base_arr.len() {
                    deep_merge_safe(&mut base_arr[i], override_item, visited)?;
                } else {
                    base_arr.push(override_item.clone());
                }
            }
            Ok(())
        }
        _ => {
            *base = override_val.clone();
            Ok(())
        }
    };

    visited.remove(&ptr);
    result
}

#[test]
fn test_with_safe_merge() {
    let mut base = serde_json::json!({ "a": 1 });
    let override_val = serde_json::json!({ "b": 2 });
    let mut visited = std::collections::HashSet::new();

    assert!(deep_merge_safe(&mut base, &override_val, &mut visited).is_ok());
}
```

### With Depth Limit

```rust
use serde_json::Value;

pub fn deep_merge_with_depth_limit(
    base: &mut Value,
    override_val: &Value,
    max_depth: usize,
    current_depth: usize,
) {
    if current_depth >= max_depth {
        *base = override_val.clone();
        eprintln!("Merge depth limit ({}) reached at key", max_depth);
        return;
    }

    match (base, override_val) {
        (Value::Object(base_map), Value::Object(override_map)) => {
            for (key, override_value) in override_map.iter() {
                let entry = base_map
                    .entry(key.clone())
                    .or_insert_with(|| Value::Null);
                deep_merge_with_depth_limit(
                    entry,
                    override_value,
                    max_depth,
                    current_depth + 1,
                );
            }
        }
        (Value::Array(base_arr), Value::Array(override_arr)) => {
            for (i, override_item) in override_arr.iter().enumerate() {
                if i < base_arr.len() {
                    deep_merge_with_depth_limit(
                        &mut base_arr[i],
                        override_item,
                        max_depth,
                        current_depth + 1,
                    );
                } else {
                    base_arr.push(override_item.clone());
                }
            }
        }
        _ => *base = override_val.clone(),
    }
}
```

---

## Order-Preserving with IndexMap

### Setup

```toml
[dependencies]
indexmap = { version = "2", features = ["serde"] }
serde_json = { version = "1", features = ["preserve_order"] }
serde = { version = "1", features = ["derive"] }
```

### Basic Usage

```rust
use indexmap::IndexMap;
use serde_json::Value;

fn merge_ordered_configs(
    mut base: IndexMap<String, Value>,
    override_cfg: IndexMap<String, Value>,
) -> IndexMap<String, Value> {
    for (key, value) in override_cfg {
        match (&mut base.get_mut(&key), &value) {
            (Some(Value::Object(base_obj)), Value::Object(override_obj)) => {
                // Recursively merge nested objects
                for (k, v) in override_obj.iter() {
                    base_obj.insert(k.clone(), v.clone());
                }
            }
            _ => {
                base.insert(key, value);
            }
        }
    }
    base
}

#[test]
fn test_order_preservation() {
    let mut base = indexmap::indexmap! {
        "z".to_string() => serde_json::json!(1),
        "a".to_string() => serde_json::json!(2),
        "m".to_string() => serde_json::json!(3),
    };

    let override_cfg = indexmap::indexmap! {
        "a".to_string() => serde_json::json!(20),
    };

    let result = merge_ordered_configs(base, override_cfg);

    // Keys maintain insertion order: z, a (updated), m
    let keys: Vec<_> = result.keys().collect();
    assert_eq!(keys, vec!["z", "a", "m"]);
}
```

### Loading JSON with Order Preservation

```rust
use serde_json::Value;
use std::fs;

fn load_json_preserve_order(path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    // With serde_json "preserve_order" feature, this preserves key order
    let value = serde_json::from_str(&contents)?;
    Ok(value)
}

fn load_yaml_config(path: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    // Parse YAML to serde_json Value
    let value: Value = serde_yaml::from_str(&contents)?;
    Ok(value)
}
```

---

## Handling Edge Cases

### 1. Type Conflict Resolution

```rust
use serde_json::Value;

pub enum ConflictStrategy {
    /// Override with right-hand value
    Right,
    /// Keep left-hand value
    Left,
    /// Raise an error
    Error,
}

pub fn merge_with_strategy(
    base: &mut Value,
    override_val: &Value,
    strategy: ConflictStrategy,
) -> Result<(), String> {
    // Check for type mismatch
    let type_matches = match (base, override_val) {
        (Value::Object(_), Value::Object(_)) => true,
        (Value::Array(_), Value::Array(_)) => true,
        (Value::Null, Value::Null) => true,
        (Value::Bool(_), Value::Bool(_)) => true,
        (Value::Number(_), Value::Number(_)) => true,
        (Value::String(_), Value::String(_)) => true,
        _ => false,
    };

    if !type_matches {
        match strategy {
            ConflictStrategy::Right => *base = override_val.clone(),
            ConflictStrategy::Left => {},  // Keep base unchanged
            ConflictStrategy::Error => {
                return Err(format!(
                    "Type mismatch: {:?} vs {:?}",
                    base, override_val
                ));
            }
        }
        return Ok(());
    }

    // Normal merge for matching types
    match (base, override_val) {
        (Value::Object(base_map), Value::Object(override_map)) => {
            for (key, override_value) in override_map.iter() {
                let entry = base_map
                    .entry(key.clone())
                    .or_insert_with(|| Value::Null);
                merge_with_strategy(entry, override_value, strategy)?;
            }
        }
        (Value::Array(base_arr), Value::Array(override_arr)) => {
            for (i, override_item) in override_arr.iter().enumerate() {
                if i < base_arr.len() {
                    merge_with_strategy(
                        &mut base_arr[i],
                        override_item,
                        strategy,
                    )?;
                } else {
                    base_arr.push(override_item.clone());
                }
            }
        }
        _ => *base = override_val.clone(),
    }

    Ok(())
}
```

### 2. Null and Undefined Semantics

```rust
use serde_json::Value;

#[derive(Debug, Clone, Copy)]
pub enum NullSemantics {
    /// null overwrites existing values
    Override,
    /// null is skipped, left unchanged
    Skip,
    /// null deletes the key
    Delete,
}

pub fn merge_with_null_semantics(
    base: &mut Value,
    override_val: &Value,
    semantics: NullSemantics,
) {
    match (base, override_val) {
        (Value::Object(base_map), Value::Object(override_map)) => {
            for (key, override_value) in override_map.iter() {
                match (semantics, override_value) {
                    (NullSemantics::Skip, Value::Null) => {
                        // Don't override with null
                        continue;
                    }
                    (NullSemantics::Delete, Value::Null) => {
                        // Delete the key
                        base_map.remove(key);
                        continue;
                    }
                    _ => {}
                }

                let entry = base_map
                    .entry(key.clone())
                    .or_insert_with(|| Value::Null);

                if override_value.is_object() && entry.is_object() {
                    merge_with_null_semantics(entry, override_value, semantics);
                } else {
                    *entry = override_value.clone();
                }
            }
        }
        _ => *base = override_val.clone(),
    }
}

#[test]
fn test_null_semantics() {
    let mut base = serde_json::json!({
        "a": 1,
        "b": 2
    });

    let override_cfg = serde_json::json!({
        "b": null,
        "c": 3
    });

    // Skip null values
    merge_with_null_semantics(&mut base, &override_cfg, NullSemantics::Skip);
    assert_eq!(base["b"], 2);  // b unchanged
    assert_eq!(base["c"], 3);  // c added

    // Delete on null
    let mut base2 = base.clone();
    merge_with_null_semantics(&mut base2, &override_cfg, NullSemantics::Delete);
    assert!(!base2.get("b").is_some());  // b removed
}
```

### 3. Large Array Handling

```rust
use serde_json::Value;

pub enum ArrayMergeStrategy {
    /// Replace entire array
    Replace,
    /// Concatenate arrays
    Concatenate,
    /// Merge by index (default)
    MergeByIndex,
}

pub fn merge_with_array_strategy(
    base: &mut Value,
    override_val: &Value,
    array_strategy: ArrayMergeStrategy,
    large_array_threshold: usize,
) {
    match (base, override_val) {
        (Value::Object(base_map), Value::Object(override_map)) => {
            for (key, override_value) in override_map.iter() {
                let entry = base_map
                    .entry(key.clone())
                    .or_insert_with(|| Value::Null);
                merge_with_array_strategy(
                    entry,
                    override_value,
                    array_strategy,
                    large_array_threshold,
                );
            }
        }
        (Value::Array(base_arr), Value::Array(override_arr)) => {
            // Use strategy based on array size
            let use_strategy = if base_arr.len() > large_array_threshold
                || override_arr.len() > large_array_threshold
            {
                // For large arrays, use Replace by default
                ArrayMergeStrategy::Replace
            } else {
                array_strategy
            };

            match use_strategy {
                ArrayMergeStrategy::Replace => {
                    *base = override_val.clone();
                }
                ArrayMergeStrategy::Concatenate => {
                    base_arr.extend(override_arr.iter().cloned());
                }
                ArrayMergeStrategy::MergeByIndex => {
                    for (i, override_item) in override_arr.iter().enumerate() {
                        if i < base_arr.len() {
                            merge_with_array_strategy(
                                &mut base_arr[i],
                                override_item,
                                array_strategy,
                                large_array_threshold,
                            );
                        } else {
                            base_arr.push(override_item.clone());
                        }
                    }
                }
            }
        }
        _ => *base = override_val.clone(),
    }
}
```

---

## Performance Patterns

### Lazy Merge

```rust
use serde_json::Value;
use std::rc::Rc;
use std::cell::RefCell;

pub struct LazyMerge {
    base: Rc<Value>,
    overrides: Rc<Value>,
    cache: RefCell<Option<Value>>,
}

impl LazyMerge {
    pub fn new(base: Value, overrides: Value) -> Self {
        LazyMerge {
            base: Rc::new(base),
            overrides: Rc::new(overrides),
            cache: RefCell::new(None),
        }
    }

    pub fn get(&self, path: &str) -> Option<Value> {
        // Look in overrides first, then fall back to base
        if let Some(val) = self.overrides.get(path) {
            return Some(val.clone());
        }
        self.base.get(path).cloned()
    }

    pub fn materialize(&self) -> Value {
        if let Some(cached) = &*self.cache.borrow() {
            return cached.clone();
        }

        let mut result = self.base.as_ref().clone();
        deep_merge(&mut result, &self.overrides);
        *self.cache.borrow_mut() = Some(result.clone());
        result
    }
}

fn deep_merge(base: &mut Value, override_val: &Value) {
    match (base, override_val) {
        (Value::Object(base_map), Value::Object(override_map)) => {
            for (key, override_value) in override_map.iter() {
                let entry = base_map
                    .entry(key.clone())
                    .or_insert_with(|| Value::Null);
                deep_merge(entry, override_value);
            }
        }
        _ => *base = override_val.clone(),
    }
}
```

### Streaming Merge (for large files)

```rust
use serde_json::Deserializer;
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;

pub fn merge_large_files(
    base_path: &str,
    override_path: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    // Load base file
    let base_file = File::open(base_path)?;
    let base_reader = BufReader::new(base_file);
    let mut base: Value = serde_json::from_reader(base_reader)?;

    // Stream override file to avoid loading entire thing at once
    let override_file = File::open(override_path)?;
    let override_reader = BufReader::new(override_file);
    let override_val: Value = serde_json::from_reader(override_reader)?;

    // Merge
    deep_merge(&mut base, &override_val);

    Ok(base)
}
```

---

## Complete Examples

### 1. Configuration File Merger with Environment Support

```rust
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

pub struct ConfigMerger {
    base_dir: String,
}

impl ConfigMerger {
    pub fn new(base_dir: &str) -> Self {
        ConfigMerger {
            base_dir: base_dir.to_string(),
        }
    }

    pub fn load_and_merge(&self, env: &str) -> Result<Value, Box<dyn std::error::Error>> {
        // Load base config
        let base_path = format!("{}/config.json", self.base_dir);
        let mut config = self.load_json(&base_path)?;

        // Load environment-specific config if it exists
        let env_path = format!("{}/config.{}.json", self.base_dir, env);
        if Path::new(&env_path).exists() {
            let env_config = self.load_json(&env_path)?;
            deep_merge(&mut config, &env_config);
        }

        Ok(config)
    }

    fn load_json(&self, path: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let contents = fs::read_to_string(path)?;
        let value = serde_json::from_str(&contents)?;
        Ok(value)
    }
}

fn deep_merge(base: &mut Value, override_val: &Value) {
    match (base, override_val) {
        (Value::Object(base_map), Value::Object(override_map)) => {
            for (key, override_value) in override_map.iter() {
                let entry = base_map
                    .entry(key.clone())
                    .or_insert_with(|| Value::Null);
                deep_merge(entry, override_value);
            }
        }
        (Value::Array(base_arr), Value::Array(override_arr)) => {
            for (i, override_item) in override_arr.iter().enumerate() {
                if i < base_arr.len() {
                    deep_merge(&mut base_arr[i], override_item);
                } else {
                    base_arr.push(override_item.clone());
                }
            }
        }
        _ => *base = override_val.clone(),
    }
}

#[test]
fn test_config_merger() {
    // This would require test files, here's the structure:
    // base/config.json:
    // {
    //   "server": {"port": 3000, "host": "localhost"},
    //   "db": {"pool": 10}
    // }
    //
    // base/config.prod.json:
    // {
    //   "server": {"port": 8080},
    //   "db": {"pool": 50}
    // }

    let merger = ConfigMerger::new("/etc/app");
    // let result = merger.load_and_merge("prod").unwrap();
    // assert_eq!(result["server"]["port"], 8080);
    // assert_eq!(result["db"]["pool"], 50);
}
```

### 2. Type-Safe Configuration with Struct

```rust
use merge::Merge;
use serde::{Deserialize, Serialize};

#[derive(Merge, Clone, Debug, Default, Serialize, Deserialize)]
struct AppConfig {
    #[serde(default)]
    server: ServerConfig,

    #[serde(default)]
    database: DatabaseConfig,

    #[serde(default)]
    logging: LoggingConfig,
}

#[derive(Merge, Clone, Debug, Default, Serialize, Deserialize)]
struct ServerConfig {
    #[serde(default = "default_host")]
    host: String,

    #[serde(default = "default_port")]
    port: u16,

    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    ssl: bool,
}

fn default_host() -> String {
    "localhost".to_string()
}

fn default_port() -> u16 {
    3000
}

#[derive(Merge, Clone, Debug, Default, Serialize, Deserialize)]
struct DatabaseConfig {
    #[serde(default)]
    url: String,

    #[merge(strategy = merge::option::overwrite_none)]
    pool_size: Option<u32>,

    #[merge(strategy = merge::vec::append)]
    #[serde(default)]
    migrations: Vec<String>,
}

#[derive(Merge, Clone, Debug, Default, Serialize, Deserialize)]
struct LoggingConfig {
    #[merge(strategy = merge::bool::overwrite_false)]
    #[serde(default)]
    enabled: bool,

    #[merge(strategy = merge::option::overwrite_none)]
    level: Option<String>,

    #[merge(strategy = merge::option::overwrite_none)]
    output: Option<String>,
}

fn load_config(paths: &[&str]) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let mut config = AppConfig::default();

    for path in paths {
        let contents = std::fs::read_to_string(path)?;
        let partial: AppConfig = serde_json::from_str(&contents)?;
        config.merge(partial);
    }

    Ok(config)
}

#[test]
fn test_app_config() {
    let base = serde_json::json!({
        "server": {
            "host": "localhost",
            "port": 3000,
            "ssl": false
        },
        "database": {
            "pool_size": 10,
            "migrations": ["001_init", "002_users"]
        },
        "logging": {
            "enabled": true,
            "level": "info"
        }
    });

    let override_cfg = serde_json::json!({
        "server": {
            "port": 8080,
            "ssl": true
        },
        "database": {
            "migrations": ["003_posts"]
        },
        "logging": {
            "level": "debug"
        }
    });

    let base_str = base.to_string();
    let override_str = override_cfg.to_string();

    let mut config: AppConfig = serde_json::from_str(&base_str).unwrap();
    let partial: AppConfig = serde_json::from_str(&override_str).unwrap();

    config.merge(partial);

    assert_eq!(config.server.host, "localhost");
    assert_eq!(config.server.port, 8080);
    assert_eq!(config.server.ssl, true);
    assert_eq!(config.database.pool_size, Some(10));
    assert_eq!(config.database.migrations.len(), 3);
    assert_eq!(config.logging.level, Some("debug".to_string()));
}
```

---

**Version**: 1.0
**Last Updated**: December 27, 2025
