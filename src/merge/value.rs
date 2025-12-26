//! Unified value type for merge operations across multiple formats.
//!
//! This module defines the `MergeValue` enum, which represents any value that can
//! appear in Jin's supported configuration formats (JSON, YAML, TOML, INI) and
//! provides deep merge operations following PRD §11.1 rules.

use crate::core::error::{JinError, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Type alias for the object/map variant storage.
pub type ObjectMap = IndexMap<String, MergeValue>;

/// Strategy for merging arrays during deep merge operations.
///
/// Different configuration scenarios require different array merge behaviors.
/// RFC 7396 specifies "replace" as the default for unkeyed arrays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArrayMergeStrategy {
    /// New array completely replaces old array (RFC 7396 default behavior)
    #[default]
    Replace,

    /// Merge arrays by matching object elements with `id` or `name` fields
    /// Objects with matching keys are deeply merged, new objects are appended
    MergeByKey,

    /// Concatenate arrays - append new elements to the end
    Concatenate,
}

/// Configuration options for deep merge behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MergeConfig {
    /// Strategy for merging arrays
    pub array_strategy: ArrayMergeStrategy,

    /// Maximum recursion depth to prevent stack overflow
    pub max_depth: usize,
}

impl Default for MergeConfig {
    fn default() -> Self {
        Self {
            array_strategy: ArrayMergeStrategy::Replace,
            max_depth: 100,
        }
    }
}

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub enum MergeValue {
    // ===== Primitive Variants =====
    /// Null value - deletes keys during merge operations
    #[default]
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
    Object(ObjectMap),
}

// ===== FROM JSON PARSING =====

impl MergeValue {
    /// Parse a JSON string into a `MergeValue`.
    ///
    /// # Errors
    ///
    /// Returns `JinError::JsonParse` if the input is not valid JSON.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let json = r#"{"name": "jin", "count": 42}"#;
    /// let value = MergeValue::from_json(json)?;
    /// ```
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

    /// Parse a YAML string into a `MergeValue`.
    ///
    /// # Errors
    ///
    /// Returns `JinError::YamlParse` if the input is not valid YAML.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let yaml = "name: jin\ncount: 42";
    /// let value = MergeValue::from_yaml(yaml)?;
    /// ```
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

    /// Parse a TOML string into a `MergeValue`.
    ///
    /// # Errors
    ///
    /// Returns `JinError::TomlParse` if the input is not valid TOML.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let toml = "[database]\nhost = \"localhost\"\nport = 5432";
    /// let value = MergeValue::from_toml(toml)?;
    /// ```
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

    /// Helper function to convert a TOML value directly to MergeValue.
    fn convert_toml_value_to_merge_value(v: &toml::Value) -> Result<MergeValue> {
        use toml::Value as Toml;

        match v {
            Toml::String(s) => Ok(MergeValue::String(s.clone())),
            Toml::Integer(i) => Ok(MergeValue::Integer(*i)),
            Toml::Float(f) => Ok(MergeValue::Float(*f)),
            Toml::Boolean(b) => Ok(MergeValue::Boolean(*b)),
            Toml::Datetime(dt) => Ok(MergeValue::String(dt.to_string())),
            Toml::Array(arr) => {
                let converted: Result<Vec<_>> = arr
                    .iter()
                    .map(Self::convert_toml_value_to_merge_value)
                    .collect();
                Ok(MergeValue::Array(converted?))
            }
            Toml::Table(table) => {
                let mut map = IndexMap::new();
                for (k, v) in table {
                    map.insert(k.clone(), Self::convert_toml_value_to_merge_value(v)?);
                }
                Ok(MergeValue::Object(map))
            }
        }
    }

    /// Parse an INI string into a `MergeValue`.
    ///
    /// INI files have a flat structure with sections. This parser converts
    /// them into nested objects:
    ///
    /// ```text
    /// [database]
    /// host = localhost
    /// port = 5432
    /// ```
    ///
    /// Becomes:
    ///
    /// ```json
    /// {
    ///   "database": {
    ///     "host": "localhost",
    ///     "port": "5432"
    ///   }
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `JinError::IniParse` if the input is not valid INI.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let ini = "[database]\nhost = localhost\nport = 5432";
    /// let value = MergeValue::from_ini(ini)?;
    /// ```
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

    /// Deep merge this value with another, following PRD §11.1 rules.
    ///
    /// # Merge Rules
    ///
    /// - If `other` is `Null`, returns `Null` (key deletion)
    /// - If both are `Object`, performs deep key merge recursively
    /// - If both are `Array`, returns `other` (higher layer replaces)
    /// - Otherwise, returns `other` (primitive replacement)
    ///
    /// # Errors
    ///
    /// Currently returns `Result` for future conflict detection.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let base = MergeValue::from_json(r#"{"a": {"x": 1}, "b": 2}"#)?;
    /// let override = MergeValue::from_json(r#"{"a": {"y": 2}}"#)?;
    /// let merged = base.merge(&override)?;
    /// // Result: {"a": {"x": 1, "y": 2}, "b": 2}
    ///
    /// // Null deletes keys
    /// let base = MergeValue::from_json(r#"{"a": 1, "b": 2}"#)?;
    /// let override = MergeValue::from_json(r#"{"a": null}"#)?;
    /// let merged = base.merge(&override)?;
    /// // Result: {"b": 2}
    /// ```
    pub fn merge(&self, other: &MergeValue) -> Result<Self> {
        // Default behavior: RFC 7396 semantics (replace arrays, null deletes keys)
        self.merge_with_config(other, &MergeConfig::default())
    }

    /// Deep merge this value with another, using the provided configuration.
    ///
    /// This method extends the basic `merge()` behavior by allowing configuration
    /// of array merge strategies and depth limits.
    ///
    /// # Merge Rules
    ///
    /// - If `other` is `Null`, returns `Null` (key deletion per RFC 7396)
    /// - If both are `Object`, performs deep key merge recursively
    /// - If both are `Array`, behavior depends on `config.array_strategy`:
    ///   - `Replace`: Returns `other` (RFC 7396 default)
    ///   - `MergeByKey`: Merges by matching `id` or `name` fields
    ///   - `Concatenate`: Appends `other` to `self`
    /// - Otherwise, returns `other` (primitive replacement)
    ///
    /// # Errors
    ///
    /// Returns `JinError::Message` if max_depth is exceeded.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use jin_glm::merge::value::{MergeValue, MergeConfig, ArrayMergeStrategy};
    ///
    /// let base = MergeValue::from_json(r#"[{"id": "a", "x": 1}]"#)?;
    /// let patch = MergeValue::from_json(r#"[{"id": "a", "y": 2}]"#)?;
    ///
    /// let config = MergeConfig {
    ///     array_strategy: ArrayMergeStrategy::MergeByKey,
    ///     ..Default::default()
    /// };
    ///
    /// let merged = base.merge_with_config(&patch, &config)?;
    /// // Result: [{"id": "a", "x": 1, "y": 2}]
    /// ```
    pub fn merge_with_config(&self, other: &MergeValue, config: &MergeConfig) -> Result<Self> {
        // Check depth limit first
        if config.max_depth == 0 {
            return Err(JinError::Message("Maximum merge depth exceeded".to_string()));
        }

        let child_config = MergeConfig {
            max_depth: config.max_depth - 1,
            ..*config
        };

        match (self, other) {
            // Rule 1: null deletes key (RFC 7396)
            (_, MergeValue::Null) => Ok(MergeValue::Null),

            // Rule 2: Deep key merge for objects (PRD §11.1)
            (MergeValue::Object(base_map), MergeValue::Object(patch_map)) => {
                let mut merged = base_map.clone();

                for (key, patch_value) in patch_map {
                    if let Some(base_value) = merged.get(key) {
                        // Recursively merge nested values
                        let merged_value = base_value.merge_with_config(patch_value, &child_config)?;

                        // Check for null deletion
                        if matches!(merged_value, MergeValue::Null) {
                            merged.shift_remove(key);
                        } else {
                            merged.insert(key.clone(), merged_value);
                        }
                    } else {
                        // Add new key (unless it's null)
                        if !matches!(patch_value, MergeValue::Null) {
                            merged.insert(key.clone(), patch_value.clone());
                        }
                    }
                }

                Ok(MergeValue::Object(merged))
            }

            // Rule 3: Array merge based on strategy
            (MergeValue::Array(base_arr), MergeValue::Array(patch_arr)) => {
                match config.array_strategy {
                    ArrayMergeStrategy::Replace => Ok(MergeValue::Array(patch_arr.clone())),
                    ArrayMergeStrategy::MergeByKey => {
                        let merged = Self::merge_arrays_by_key(base_arr, patch_arr, &child_config)?;
                        Ok(MergeValue::Array(merged))
                    }
                    ArrayMergeStrategy::Concatenate => {
                        let mut result = base_arr.clone();
                        result.extend(patch_arr.clone());
                        Ok(MergeValue::Array(result))
                    }
                }
            }

            // Rule 4: Primitive replacement
            (_, _) => Ok(other.clone()),
        }
    }

    /// Helper method to merge arrays by key field (`id` or `name`).
    ///
    /// This algorithm:
    /// 1. Separates object and non-object elements
    /// 2. Builds indexes by `id` or `name` field for objects
    /// 3. Merges objects with matching keys
    /// 4. Preserves non-object elements
    /// 5. Maintains order: base elements first, then new patch elements
    fn merge_arrays_by_key(
        base: &[MergeValue],
        patch: &[MergeValue],
        config: &MergeConfig,
    ) -> Result<Vec<MergeValue>> {
        use std::collections::{HashMap, HashSet};

        // Helper to extract key from an object element
        fn get_key(value: &MergeValue) -> Option<String> {
            if let MergeValue::Object(obj) = value {
                // Check "id" field first, then "name"
                obj.get("id").and_then(|v| v.as_str())
                    .or_else(|| obj.get("name").and_then(|v| v.as_str()))
                    .map(|k| k.to_string())
            } else {
                None
            }
        }

        // Build index of patch array elements by key
        let mut patch_by_key: HashMap<String, MergeValue> = HashMap::new();
        let mut patch_has_key: HashSet<String> = HashSet::new();
        let mut patch_non_objects: Vec<MergeValue> = Vec::new();

        for elem in patch {
            match get_key(elem) {
                Some(key) => {
                    patch_by_key.insert(key.clone(), elem.clone());
                    patch_has_key.insert(key);
                }
                None => { patch_non_objects.push(elem.clone()); }
            }
        }

        // Process base array in order, merging keyed elements
        let mut result = Vec::new();
        let mut merged_keys: HashSet<String> = HashSet::new();

        for elem in base {
            match get_key(elem) {
                Some(key) => {
                    // Keyed element - check if patch has matching key
                    if let Some(patch_elem) = patch_by_key.get(&key) {
                        // Merge with patch element
                        result.push(elem.merge_with_config(patch_elem, config)?);
                        merged_keys.insert(key);
                    } else {
                        // No matching patch element - include as-is
                        result.push(elem.clone());
                    }
                }
                None => {
                    // Non-object element - include as-is
                    result.push(elem.clone());
                }
            }
        }

        // Add new keyed elements from patch (not in base)
        let mut new_keys: Vec<String> = patch_has_key.iter()
            .filter(|k| !merged_keys.contains(*k))
            .cloned()
            .collect();
        new_keys.sort(); // For deterministic output

        for key in new_keys {
            if let Some(elem) = patch_by_key.get(&key) {
                if !matches!(elem, MergeValue::Null) {
                    result.push(elem.clone());
                }
            }
        }

        // Append non-object elements from patch
        for elem in patch_non_objects {
            result.push(elem);
        }

        Ok(result)
    }

    // ===== TYPE CHECK METHODS =====

    /// Returns `true` if this value is `Null`.
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, MergeValue::Null)
    }

    /// Returns `true` if this value is an `Object`.
    #[inline]
    pub fn is_object(&self) -> bool {
        matches!(self, MergeValue::Object(_))
    }

    /// Returns `true` if this value is an `Array`.
    #[inline]
    pub fn is_array(&self) -> bool {
        matches!(self, MergeValue::Array(_))
    }

    // ===== TYPE CONVERSION METHODS =====

    /// Returns a reference to the inner `ObjectMap` if this is an `Object`.
    #[inline]
    pub fn as_object(&self) -> Option<&ObjectMap> {
        match self {
            MergeValue::Object(map) => Some(map),
            _ => None,
        }
    }

    /// Returns a reference to the inner `Vec` if this is an `Array`.
    #[inline]
    pub fn as_array(&self) -> Option<&[MergeValue]> {
        match self {
            MergeValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Returns a reference to the inner `String` if this is a `String`.
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            MergeValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the inner `i64` if this is an `Integer`.
    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            MergeValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Returns the inner `bool` if this is a `Boolean`.
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            MergeValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }
}

// ===== FROM CONVERSIONS FOR PRIMITIVES =====

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

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;

    // ===== ENUM VARIANT TESTS =====

    #[test]
    fn test_enum_variants_create() {
        // All variants can be created
        let _ = MergeValue::Null;
        let _ = MergeValue::Boolean(true);
        let _ = MergeValue::Integer(42);
        let _ = MergeValue::Float(3.15);
        let _ = MergeValue::String("hello".to_string());
        let _ = MergeValue::Array(vec![]);
        let _ = MergeValue::Object(IndexMap::new());
    }

    // ===== FROM JSON TESTS =====

    #[test]
    fn test_from_json_valid() {
        let json = r#"{"name": "jin", "count": 42, "active": true, "pi": 3.15}"#;
        let value = MergeValue::from_json(json).unwrap();

        assert!(value.is_object());
        let obj = value.as_object().unwrap();
        assert_eq!(obj.len(), 4);

        assert_eq!(obj.get("name").and_then(|v| v.as_str()), Some("jin"));
        assert_eq!(obj.get("count").and_then(|v| v.as_i64()), Some(42));
        assert_eq!(obj.get("active").and_then(|v| v.as_bool()), Some(true));
        // Float is stored as Float variant, not String
        match obj.get("pi") {
            Some(MergeValue::Float(f)) => assert!((f - 3.15).abs() < 0.001),
            _ => panic!("Expected Float for pi value"),
        }
    }

    #[test]
    fn test_from_json_invalid() {
        let json = "invalid json";
        assert!(MergeValue::from_json(json).is_err());
    }

    #[test]
    fn test_from_json_nested() {
        let json = r#"{"outer": {"inner": {"value": 42}}}"#;
        let value = MergeValue::from_json(json).unwrap();

        let outer = value.as_object().unwrap();
        let inner_obj = outer.get("outer").unwrap().as_object().unwrap();
        let inner = inner_obj.get("inner").unwrap().as_object().unwrap();

        assert_eq!(inner.get("value").and_then(|v| v.as_i64()), Some(42));
    }

    #[test]
    fn test_from_json_array() {
        let json = r#"[1, 2, 3, "four"]"#;
        let value = MergeValue::from_json(json).unwrap();

        let arr = value.as_array().unwrap();
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0].as_i64(), Some(1));
        assert_eq!(arr[3].as_str(), Some("four"));
    }

    // ===== FROM YAML TESTS =====

    #[test]
    fn test_from_yaml_valid() {
        let yaml = "name: jin\ncount: 42\nactive: true\npi: 3.14";
        let value = MergeValue::from_yaml(yaml).unwrap();

        assert!(value.is_object());
        let obj = value.as_object().unwrap();
        assert_eq!(obj.len(), 4);

        assert_eq!(obj.get("name").and_then(|v| v.as_str()), Some("jin"));
        assert_eq!(obj.get("count").and_then(|v| v.as_i64()), Some(42));
    }

    #[test]
    fn test_from_yaml_array() {
        let yaml = "- one\n- two\n- three";
        let value = MergeValue::from_yaml(yaml).unwrap();

        let arr = value.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_str(), Some("one"));
    }

    // ===== FROM TOML TESTS =====

    #[test]
    fn test_from_toml_valid() {
        let toml = r#"
            name = "jin"
            count = 42
            active = true
            pi = 3.14
        "#;
        let value = MergeValue::from_toml(toml).unwrap();

        assert!(value.is_object());
        let obj = value.as_object().unwrap();
        assert_eq!(obj.len(), 4);

        assert_eq!(obj.get("name").and_then(|v| v.as_str()), Some("jin"));
        assert_eq!(obj.get("count").and_then(|v| v.as_i64()), Some(42));
    }

    #[test]
    fn test_from_toml_table() {
        let toml = r#"
            [database]
            host = "localhost"
            port = 5432
        "#;
        let value = MergeValue::from_toml(toml).unwrap();

        let obj = value.as_object().unwrap();
        let db = obj.get("database").unwrap().as_object().unwrap();

        assert_eq!(db.get("host").and_then(|v| v.as_str()), Some("localhost"));
        assert_eq!(db.get("port").and_then(|v| v.as_i64()), Some(5432));
    }

    // ===== FROM INI TESTS =====

    #[test]
    fn test_from_ini_valid() {
        let ini = r#"
            [database]
            host = localhost
            port = 5432

            [server]
            port = 8080
        "#;
        let value = MergeValue::from_ini(ini).unwrap();

        let obj = value.as_object().unwrap();
        assert_eq!(obj.len(), 2);

        let db = obj.get("database").unwrap().as_object().unwrap();
        assert_eq!(db.get("host").and_then(|v| v.as_str()), Some("localhost"));
        assert_eq!(db.get("port").and_then(|v| v.as_str()), Some("5432"));

        let server = obj.get("server").unwrap().as_object().unwrap();
        assert_eq!(server.get("port").and_then(|v| v.as_str()), Some("8080"));
    }

    // ===== MERGE TESTS =====

    #[test]
    fn test_merge_objects_deep() {
        let base = MergeValue::from_json(r#"{"a": {"x": 1}, "b": 2}"#).unwrap();
        let override_val = MergeValue::from_json(r#"{"a": {"y": 2}}"#).unwrap();
        let merged = base.merge(&override_val).unwrap();

        let obj = merged.as_object().unwrap();

        // Check deep merge occurred
        let a_obj = obj.get("a").unwrap().as_object().unwrap();
        assert_eq!(a_obj.get("x").and_then(|v| v.as_i64()), Some(1));
        assert_eq!(a_obj.get("y").and_then(|v| v.as_i64()), Some(2));

        // Check unrelated key preserved
        assert_eq!(obj.get("b").and_then(|v| v.as_i64()), Some(2));
    }

    #[test]
    fn test_merge_null_deletes_key() {
        let base = MergeValue::from_json(r#"{"a": 1, "b": 2, "c": 3}"#).unwrap();
        let override_val = MergeValue::from_json(r#"{"a": null, "b": 20}"#).unwrap();
        let merged = base.merge(&override_val).unwrap();

        let obj = merged.as_object().unwrap();
        assert!(!obj.contains_key("a"), "Key 'a' should be deleted");
        assert_eq!(obj.get("b").and_then(|v| v.as_i64()), Some(20));
        assert_eq!(obj.get("c").and_then(|v| v.as_i64()), Some(3));
    }

    #[test]
    fn test_merge_arrays_replace() {
        let base = MergeValue::from_json(r#"[1, 2, 3]"#).unwrap();
        let override_val = MergeValue::from_json(r#"[4, 5, 6]"#).unwrap();
        let merged = base.merge(&override_val).unwrap();

        let arr = merged.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_i64(), Some(4));
        assert_eq!(arr[1].as_i64(), Some(5));
        assert_eq!(arr[2].as_i64(), Some(6));
    }

    #[test]
    fn test_merge_primitives_replace() {
        let base = MergeValue::Integer(10);
        let override_val = MergeValue::Integer(20);
        let merged = base.merge(&override_val).unwrap();

        assert_eq!(merged.as_i64(), Some(20));
    }

    #[test]
    fn test_merge_empty_with_object() {
        let base = MergeValue::Object(IndexMap::new());
        let override_val = MergeValue::from_json(r#"{"a": 1}"#).unwrap();
        let merged = base.merge(&override_val).unwrap();

        let obj = merged.as_object().unwrap();
        assert_eq!(obj.len(), 1);
        assert_eq!(obj.get("a").and_then(|v| v.as_i64()), Some(1));
    }

    #[test]
    fn test_merge_null_in_nested_object_deletes() {
        let base =
            MergeValue::from_json(r#"{"outer": {"inner": "value", "delete_me": "gone"}}"#).unwrap();
        let override_val = MergeValue::from_json(r#"{"outer": {"delete_me": null}}"#).unwrap();
        let merged = base.merge(&override_val).unwrap();

        let outer = merged
            .as_object()
            .unwrap()
            .get("outer")
            .unwrap()
            .as_object()
            .unwrap();
        assert_eq!(outer.get("inner").and_then(|v| v.as_str()), Some("value"));
        assert!(!outer.contains_key("delete_me"));
    }

    // ===== HELPER METHOD TESTS =====

    #[test]
    fn test_helper_methods() {
        let null_val = MergeValue::Null;
        assert!(null_val.is_null());
        assert!(!null_val.is_object());
        assert!(!null_val.is_array());

        let str_val = MergeValue::String("hello".to_string());
        assert!(!str_val.is_null());
        assert!(!str_val.is_object());
        assert_eq!(str_val.as_str(), Some("hello"));

        let int_val = MergeValue::Integer(42);
        assert_eq!(int_val.as_i64(), Some(42));

        let bool_val = MergeValue::Boolean(true);
        assert_eq!(bool_val.as_bool(), Some(true));

        let arr_val = MergeValue::Array(vec![MergeValue::Integer(1)]);
        assert!(arr_val.is_array());
        assert_eq!(arr_val.as_array().map(|a| a.len()), Some(1));

        let obj_val = MergeValue::Object(IndexMap::new());
        assert!(obj_val.is_object());
        assert_eq!(obj_val.as_object().map(|o| o.len()), Some(0));
    }

    // ===== FROM CONVERSION TESTS =====

    #[test]
    fn test_from_conversions() {
        let _b: MergeValue = true.into();
        let _i: MergeValue = 42i64.into();
        let _f: MergeValue = 3.15f64.into();
        let _s: MergeValue = String::from("hello").into();
        let _ref: MergeValue = "world".into();
    }

    // ===== DEFAULT TEST =====

    #[test]
    fn test_default() {
        let val = MergeValue::default();
        assert!(val.is_null());
    }
}
