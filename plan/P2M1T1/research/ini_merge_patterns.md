# INI File Merging Patterns in Rust

## Overview

This research document explores the `configparser` crate for INI file parsing and merge operations in Rust, with a focus on implementing deterministic merge algorithms for the Jin project.

## 1. configparser Crate API and Usage

### Basic Configuration

The `configparser` crate (version 0.4) is already included in the project dependencies:

```toml
[dependencies]
configparser = "0.4"
```

### Core API Methods

#### Creating and Loading INI Files

```rust
use configparser::ini::Ini;

// Create a new empty INI instance
let mut config = Ini::new();

// Load from file
config.load("config.ini")?;

// Load from string
let config = Ini::new_from_str(
    "[section1]
    key1 = value1
    key2 = value2"
)?;
```

#### Section and Key Operations

```rust
// Get all sections
let sections = config.sections().unwrap_or_default();

// Get a value as string
let value = config.get("section", "key")?;

// Get typed values
let bool_val = config.getbool("section", "key")?;
let int_val = config.getint("section", "key")?;
let uint_val = config.getuint("section", "key")?;

// Set values
config.set("section", "key", "value")?;

// Check if section exists
let has_section = config.section(None).is_some();

// Check if key exists in section
let has_key = config.get("section", "key").is_ok();
```

#### Writing INI Files

```rust
// Write to file
config.write("config.ini")?;

// Write to string
let content = config.to_string()?;
```

### Error Handling

```rust
use configparser::ini::Ini;
use std::io::Error as IoError;

fn load_config(path: &str) -> Result<Ini, Box<dyn std::error::Error>> {
    let mut config = Ini::new();
    config.load(path)
        .map_err(|e| IoError::new(std::io::ErrorKind::Other, e))?;
    Ok(config)
}
```

## 2. Parsing INI Files into Rust Structures

### Basic Structure Mapping

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IniConfig {
    pub version: u8,
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub ssl: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
    pub format: String,
}
```

### Parser Implementation

```rust
use configparser::ini::Ini;
use anyhow::Result;

pub fn parse_ini_to_struct(ini_content: &str) -> Result<IniConfig> {
    let config = Ini::new_from_str(ini_content)?;

    let database = DatabaseConfig {
        host: config.get("database", "host")?.unwrap_or_default(),
        port: config.getint("database", "port")?.unwrap_or(5432),
        username: config.get("database", "username")?.unwrap_or_default(),
        password: config.get("database", "password")?.unwrap_or_default(),
        database: config.get("database", "database")?.unwrap_or_default(),
    };

    let server = ServerConfig {
        host: config.get("server", "host")?.unwrap_or_default(),
        port: config.getint("server", "port")?.unwrap_or(8080),
        workers: config.getuint("server", "workers")?.unwrap_or(4),
        ssl: config.getbool("server", "ssl")?.unwrap_or(false),
    };

    let logging = LoggingConfig {
        level: config.get("logging", "level")?.unwrap_or("info".to_string()),
        file: config.get("logging", "file")?,
        format: config.get("logging", "format")?.unwrap_or("simple".to_string()),
    };

    Ok(IniConfig {
        version: 1,
        database,
        server,
        logging,
    })
}
```

### Validation with Default Values

```rust
impl IniConfig {
    pub fn from_ini(config: &Ini) -> Self {
        Self {
            version: 1,
            database: DatabaseConfig {
                host: config.get("database", "host")
                    .unwrap_or_else(|_| "localhost".to_string()),
                port: config.getint("database", "port")
                    .unwrap_or(5432) as u16,
                username: config.get("database", "username")
                    .unwrap_or_else(|_| "admin".to_string()),
                password: config.get("database", "password")
                    .unwrap_or_else(|_| "".to_string()),
                database: config.get("database", "database")
                    .unwrap_or_else(|_| "mydb".to_string()),
            },
            server: ServerConfig {
                host: config.get("server", "host")
                    .unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: config.getint("server", "port")
                    .unwrap_or(8080) as u16,
                workers: config.getuint("server", "workers")
                    .unwrap_or(4) as usize,
                ssl: config.getbool("server", "ssl")
                    .unwrap_or(false),
            },
            logging: LoggingConfig {
                level: config.get("logging", "level")
                    .unwrap_or_else(|_| "info".to_string()),
                file: config.get("logging", "file")
                    .ok(),
                format: config.get("logging", "format")
                    .unwrap_or_else(|_| "simple".to_string()),
            },
        }
    }
}
```

## 3. INI Merge Operations

### Basic Merge Algorithm

```rust
use configparser::ini::Ini;
use std::collections::HashMap;

pub struct IniMerger;

impl IniMerger {
    /// Merge two INI configurations with override strategy
    pub fn merge(base: &Ini, override_config: &Ini) -> Ini {
        let mut merged = base.clone();

        // Get all sections from both configs
        let base_sections = base.sections().unwrap_or_default();
        let override_sections = override_config.sections().unwrap_or_default();

        // Collect all unique sections
        let all_sections: Vec<String> = base_sections
            .into_iter()
            .chain(override_sections)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Merge each section
        for section in all_sections {
            Self::merge_section(&mut merged, section, base, override_config);
        }

        merged
    }

    fn merge_section(
        merged: &mut Ini,
        section: String,
        base: &Ini,
        override_config: &Ini,
    ) {
        // Get keys from both sections
        let base_keys = Self::get_section_keys(base, &section);
        let override_keys = Self::get_section_keys(override_config, &section);

        // Collect all unique keys
        let all_keys: Vec<String> = base_keys
            .into_iter()
            .chain(override_keys)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Merge each key
        for key in all_keys {
            // Try to get value from override config first
            if let Ok(Some(value)) = override_config.get(&section, &key) {
                merged.set(&section, &key, &value);
            } else {
                // Fall back to base config
                if let Ok(Some(value)) = base.get(&section, &key) {
                    merged.set(&section, &key, &value);
                }
            }
        }
    }

    fn get_section_keys(config: &Ini, section: &str) -> Vec<String> {
        config.section(Some(section))
            .map(|section_map| {
                section_map.keys().cloned().collect()
            })
            .unwrap_or_default()
    }
}
```

### Advanced Merge Strategies

#### 1. Override Strategy (Default)

```rust
/// Later values completely override earlier ones
pub fn merge_override(base: &Ini, layers: &[&Ini]) -> Ini {
    let mut merged = base.clone();

    for layer in layers {
        merged = Self::merge(&merged, layer);
    }

    merged
}
```

#### 2. Deep Merge Strategy

```rust
/// Merge values recursively where possible
pub fn merge_deep(base: &Ini, override_config: &Ini) -> Ini {
    let mut merged = base.clone();

    // Get all sections
    let all_sections: Vec<String> = Self::get_all_sections(base, override_config);

    for section in all_sections {
        Self::merge_section_deep(&mut merged, section, base, override_config);
    }

    merged
}

fn merge_section_deep(
    merged: &mut Ini,
    section: String,
    base: &Ini,
    override_config: &Ini,
) {
    // Special handling for nested structures if needed
    // For simple INI, this is the same as regular merge
    Self::merge_section(merged, section, base, override_config);
}
```

#### 3. List Merge Strategy

```rust
/// Merge comma-separated lists by combining unique values
pub fn merge_lists(base: &Ini, override_config: &Ini) -> Ini {
    let mut merged = base.clone();

    let all_sections: Vec<String> = Self::get_all_sections(base, override_config);

    for section in all_sections {
        Self::merge_section_lists(&mut merged, section, base, override_config);
    }

    merged
}

fn merge_section_lists(
    merged: &mut Ini,
    section: String,
    base: &Ini,
    override_config: &Ini,
) {
    let base_keys = Self::get_section_keys(base, &section);
    let override_keys = Self::get_section_keys(override_config, &section);
    let all_keys: Vec<String> = base_keys
        .into_iter()
        .chain(override_keys)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    for key in all_keys {
        let base_value = base.get(&section, &key).ok().flatten();
        let override_value = override_config.get(&section, &key).ok().flatten();

        if let (Some(base_val), Some(override_val)) = (base_value, override_value) {
            // Check if both values are comma-separated lists
            let base_list: Vec<&str> = base_val.split(',').map(|s| s.trim()).collect();
            let override_list: Vec<&str> = override_val.split(',').map(|s| s.trim()).collect();

            let mut merged_list: Vec<&str> = base_list
                .into_iter()
                .chain(override_list)
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            // Preserve original order from base, then append new items
            let base_set: std::collections::HashSet<_> = base_list.into_iter().collect();
            let new_items: Vec<&str> = override_list
                .into_iter()
                .filter(|item| !base_set.contains(item))
                .collect();

            merged_list.extend(new_items);

            let merged_value = merged_list.join(", ");
            merged.set(&section, &key, &merged_value);
        } else if let Some(value) = override_value {
            merged.set(&section, &key, &value);
        } else if let Some(value) = base_value {
            merged.set(&section, &key, &value);
        }
    }
}
```

## 4. Section Merging Strategies

### Section-Based Merge

```rust
pub enum MergeStrategy {
    Override,        // Later sections completely replace earlier ones
    Merge,           // Merge sections with key overrides
    PreserveEmpty,   // Keep empty sections from base
    RemoveEmpty,     // Remove empty sections in result
}

pub fn merge_sections(
    base: &Ini,
    layers: &[&Ini],
    strategy: MergeStrategy,
) -> Ini {
    let mut merged = Ini::new();

    // Collect all unique sections across all layers
    let mut all_sections = std::collections::HashSet::new();
    for layer in layers {
        if let Ok(sections) = layer.sections() {
            all_sections.extend(sections);
        }
    }
    all_sections.extend(base.sections().unwrap_or_default());

    // Apply merge strategy
    match strategy {
        MergeStrategy::Override => {
            // Use the last occurrence of each section
            for section in all_sections {
                for layer in layers.iter().rev() {
                    if let Ok(Some(value)) = layer.get(&section, "_dummy") {
                        // Section exists in this layer, use all its keys
                        if let Some(section_map) = layer.section(Some(&section)) {
                            for (key, value) in section_map {
                                merged.set(&section, key, value);
                            }
                        }
                        break;
                    }
                }
            }
        }
        MergeStrategy::Merge => {
            // Merge all sections with later values overriding earlier ones
            for section in all_sections {
                let mut section_merged = std::collections::HashMap::new();

                // Collect from base first
                if let Some(section_map) = base.section(Some(&section)) {
                    section_merged.extend(section_map.clone());
                }

                // Then override with each layer
                for layer in layers {
                    if let Some(section_map) = layer.section(Some(&section)) {
                        section_merged.extend(section_map.clone());
                    }
                }

                // Apply to merged config
                for (key, value) in section_merged {
                    merged.set(&section, key, &value);
                }
            }
        }
        MergeStrategy::PreserveEmpty => {
            // Similar to Merge but keeps empty sections
            // Implementation similar to Merge but with empty section preservation
        }
        MergeStrategy::RemoveEmpty => {
            // Similar to Merge but removes sections with no keys
            // Implementation similar to Merge with filtering
        }
    }

    merged
}
```

### Precedence-Based Merge

```rust
/// Merge multiple layers with defined precedence order
pub fn merge_with_precedence(layers: &[Ini]) -> Result<Ini> {
    if layers.is_empty() {
        return Ok(Ini::new());
    }

    // Start with the lowest precedence layer
    let mut merged = layers[0].clone();

    // Apply each subsequent layer in order (higher precedence)
    for layer in &layers[1..] {
        merged = Self::merge_override(&merged, layer);
    }

    Ok(merged)
}
```

## 5. Best Practices for INI Merge Operations

### 1. Type Safety

```rust
pub trait IniValue: Clone {
    fn from_str(s: &str) -> Option<Self>
    where
        Self: Sized;
    fn to_string(&self) -> String;
}

impl IniValue for String {
    fn from_str(s: &str) -> Option<Self> {
        Some(s.to_string())
    }
    fn to_string(&self) -> String {
        self.clone()
    }
}

impl IniValue for bool {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => None,
        }
    }
    fn to_string(&self) -> String {
        self.to_string()
    }
}

impl IniValue for i64 {
    fn from_str(s: &str) -> Option<Self> {
        s.parse().ok()
    }
    fn to_string(&self) -> String {
        self.to_string()
    }
}
```

### 2. Validation and Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum IniMergeError {
    #[error("Section not found: {0}")]
    SectionNotFound(String),
    #[error("Key not found: {0} in section {1}")]
    KeyNotFound(String, String),
    #[error("Type conversion failed: {0}")]
    TypeConversionFailed(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct ValidatedIniMerger;

impl ValidatedIniMerger {
    pub fn merge_with_validation(
        base: &Ini,
        override_config: &Ini,
        required_sections: &[String],
        required_keys: &HashMap<String, Vec<String>>,
    ) -> Result<Ini, IniMergeError> {
        // Validate required sections
        for section in required_sections {
            if !Self::section_exists(base, section) && !Self::section_exists(override_config, section) {
                return Err(IniMergeError::SectionNotFound(section.clone()));
            }
        }

        // Validate required keys in their sections
        for (section, keys) in required_keys {
            if !Self::section_exists(base, section) && !Self::section_exists(override_config, section) {
                return Err(IniMergeError::SectionNotFound(section.clone()));
            }

            for key in keys {
                if !Self::key_exists(base, section, key) && !Self::key_exists(override_config, section, key) {
                    return Err(IniMergeError::KeyNotFound(key.clone(), section.clone()));
                }
            }
        }

        // Perform merge
        Ok(Self::merge(base, override_config))
    }

    fn section_exists(config: &Ini, section: &str) -> bool {
        config.section(Some(section)).is_some()
    }

    fn key_exists(config: &Ini, section: &str, key: &str) -> bool {
        config.get(section, key).is_ok()
    }
}
```

### 3. Performance Optimization

```rust
/// Cache-friendly INI merge for large configurations
pub struct CachedIniMerger {
    cache: std::collections::HashMap<String, Ini>,
}

impl CachedIniMerger {
    pub fn new() -> Self {
        Self {
            cache: std::collections::HashMap::new(),
        }
    }

    pub fn merge_cached(&mut self, key: &str, base: &Ini, layers: &[&Ini]) -> Ini {
        if let Some(cached) = self.cache.get(key) {
            return cached.clone();
        }

        let merged = Self::merge_efficient(base, layers);
        self.cache.insert(key.to_string(), merged.clone());
        merged
    }

    /// More efficient merge for large INI files
    fn merge_efficient(base: &Ini, layers: &[&Ini]) -> Ini {
        let mut merged = base.clone();

        // Pre-collect all sections and keys
        let mut all_sections = std::collections::HashSet::new();
        let mut all_keys_by_section = std::collections::HashMap::new();

        // Add base sections and keys
        if let Ok(sections) = base.sections() {
            for section in sections {
                all_sections.insert(section.clone());
                if let Some(keys) = base.section(Some(&section)) {
                    all_keys_by_section.insert(section, keys.keys().cloned().collect());
                }
            }
        }

        // Add layer sections and keys
        for layer in layers {
            if let Ok(sections) = layer.sections() {
                for section in sections {
                    all_sections.insert(section.clone());
                    if let Some(keys) = layer.section(Some(&section)) {
                        all_keys_by_section.entry(section.clone())
                            .or_insert_with(Vec::new)
                            .extend(keys.keys().cloned());
                    }
                }
            }
        }

        // Merge in one pass
        for section in all_sections {
            let keys = all_keys_by_section.get(&section).cloned().unwrap_or_default();

            for key in keys {
                // Check each layer in reverse order
                for layer in layers.iter().rev() {
                    if let Ok(Some(value)) = layer.get(&section, &key) {
                        merged.set(&section, &key, &value);
                        break;
                    }
                }

                // If not found in layers, use base value
                if merged.get(&section, &key).is_err() {
                    if let Ok(Some(value)) = base.get(&section, &key) {
                        merged.set(&section, &key, &value);
                    }
                }
            }
        }

        merged
    }
}
```

### 4. Configuration Schema Validation

```rust
#[derive(Debug, Clone)]
pub struct IniSchema {
    pub sections: std::collections::HashMap<String, SectionSchema>,
}

#[derive(Debug, Clone)]
pub struct SectionSchema {
    pub keys: std::collections::HashMap<String, KeySchema>,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct KeySchema {
    pub key_type: KeyType,
    pub required: bool,
    pub default: Option<String>,
    pub validator: Option<Box<dyn Fn(&str) -> bool>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyType {
    String,
    Integer,
    Boolean,
    Float,
    List,
}

pub struct SchemaValidator;

impl SchemaValidator {
    pub fn validate_ini(config: &Ini, schema: &IniSchema) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check sections
        if let Ok(sections) = config.sections() {
            for section in sections {
                if let Some(section_schema) = schema.sections.get(&section) {
                    Self::validate_section(config, &section, section_schema, &mut errors);
                } else if schema.sections.contains_key("_") {
                    // Wildcard section validation
                    if let Some(wildcard_schema) = schema.sections.get("_") {
                        Self::validate_section(config, &section, wildcard_schema, &mut errors);
                    }
                }
            }
        }

        // Check required sections
        for (section_name, schema) in &schema.sections {
            if schema.required && !config.section(Some(section_name)).is_some() {
                errors.push(format!("Required section missing: {}", section_name));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_section(
        config: &Ini,
        section: &str,
        schema: &SectionSchema,
        errors: &mut Vec<String>,
    ) {
        if let Some(section_map) = config.section(Some(section)) {
            for (key, value) in section_map {
                if let Some(key_schema) = schema.keys.get(&key) {
                    Self::validate_key_value(section, &key, value, key_schema, errors);
                }
            }

            // Check required keys
            for (key_name, key_schema) in &schema.keys {
                if key_schema.required && !section_map.contains_key(key_name) {
                    errors.push(format!(
                        "Required key missing: {} in section {}",
                        key_name, section
                    ));
                }
            }
        }
    }

    fn validate_key_value(
        section: &str,
        key: &str,
        value: &str,
        schema: &KeySchema,
        errors: &mut Vec<String>,
    ) {
        // Type validation
        match schema.key_type {
            KeyType::Integer => {
                if value.parse::<i64>().is_err() {
                    errors.push(format!(
                        "Invalid integer value for {}.{}: {}",
                        section, key, value
                    ));
                }
            }
            KeyType::Boolean => {
                if !["true", "false", "1", "0", "yes", "no", "on", "off"]
                    .contains(&value.to_lowercase().as_str())
                {
                    errors.push(format!(
                        "Invalid boolean value for {}.{}: {}",
                        section, key, value
                    ));
                }
            }
            KeyType::Float => {
                if value.parse::<f64>().is_err() {
                    errors.push(format!(
                        "Invalid float value for {}.{}: {}",
                        section, key, value
                    ));
                }
            }
            KeyType::List => {
                if !value.contains(',') {
                    errors.push(format!(
                        "Expected comma-separated list for {}.{}: {}",
                        section, key, value
                    ));
                }
            }
            KeyType::String => {} // No validation needed
        }

        // Custom validation
        if let Some(validator) = &schema.validator {
            if !validator(value) {
                errors.push(format!(
                    "Validation failed for {}.{}: {}",
                    section, key, value
                ));
            }
        }
    }
}
```

## 6. Integration with Jin Project

### INI Parser for Jin Layers

```rust
use crate::core::layer::Layer;
use crate::merge::value::MergeValue;
use configparser::ini::Ini;
use anyhow::Result;

pub struct IniParser;

impl IniParser {
    pub fn parse(content: &str) -> Result<MergeValue> {
        let config = Ini::new_from_str(content)?;
        let mut map = indexmap::IndexMap::new();

        if let Ok(sections) = config.sections() {
            for section in sections {
                let mut section_map = indexmap::IndexMap::new();

                if let Some(keys) = config.section(Some(&section)) {
                    for (key, value) in keys {
                        section_map.insert(key.to_string(), MergeValue::String(value.clone()));
                    }
                }

                map.insert(section.to_string(), MergeValue::Object(section_map));
            }
        }

        Ok(MergeValue::Object(map))
    }

    pub fn serialize(value: &MergeValue) -> Result<String> {
        let mut config = Ini::new();

        if let MergeValue::Object(obj) = value {
            for (section, section_value) in obj {
                if let MergeValue::Object(section_obj) = section_value {
                    for (key, value) in section_obj {
                        let value_str = match value {
                            MergeValue::String(s) => s.clone(),
                            MergeValue::Integer(i) => i.to_string(),
                            MergeValue::Boolean(b) => b.to_string(),
                            MergeValue::Float(f) => f.to_string(),
                            MergeValue::Array(arr) => {
                                let values: Vec<String> = arr.iter()
                                    .map(|v| match v {
                                        MergeValue::String(s) => s.clone(),
                                        _ => v.to_string(),
                                    })
                                    .collect();
                                values.join(", ")
                            }
                            MergeValue::Null => String::new(),
                        };
                        config.set(section, key, &value_str);
                    }
                }
            }
        }

        Ok(config.to_string()?)
    }
}
```

### INI Layer Merger Implementation

```rust
pub struct IniLayerMerger {
    parser: IniParser,
}

impl IniLayerMerger {
    pub fn new() -> Self {
        Self {
            parser: IniParser,
        }
    }

    pub fn merge_layers(
        &self,
        layers: &[(Layer, String)],
        base_content: Option<&str>,
    ) -> Result<String> {
        // Parse base content if exists
        let mut base_ini = if let Some(content) = base_content {
            Ini::new_from_str(content)?
        } else {
            Ini::new()
        };

        // Parse and merge each layer
        for (layer, content) in layers {
            let layer_ini = Ini::new_from_str(content)?;
            base_ini = self.merge_ini_configs(&base_ini, &layer_ini, layer);
        }

        // Convert back to string
        Ok(base_ini.to_string()?)
    }

    fn merge_ini_configs(
        &self,
        base: &Ini,
        layer: &Ini,
        layer_info: &Layer,
    ) -> Ini {
        // Use merge strategy based on layer precedence
        // Higher precedence layers override lower ones
        let mut merged = base.clone();

        let sections = layer.sections().unwrap_or_default();

        for section in sections {
            if let Some(section_map) = layer.section(Some(&section)) {
                for (key, value) in section_map {
                    // Only set if key doesn't exist in base or if layer has higher precedence
                    if base.get(&section, key).is_err() || layer_info.has_higher_precedence() {
                        merged.set(&section, key, value);
                    }
                }
            }
        }

        merged
    }
}
```

## References

- [configparser crate on crates.io](https://crates.io/crates/configparser)
- [configparser documentation on docs.rs](https://docs.rs/configparser)
- [Rust INI file parsing examples](https://github.com/rustsec/rustsec/tree/main/configparser)
- [Configuration file merge algorithms](https://en.wikipedia.org/wiki/Merge_(version_control))
- [JSON Merge Patch RFC 7396](https://tools.ietf.org/html/rfc7396) (can be adapted for INI)