//! Merge value types for structured configuration

use crate::core::{JinError, Result};
use indexmap::IndexMap;
use ini::Ini;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Represents a value that can be merged
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MergeValue {
    /// Null value (used to delete keys)
    Null,
    /// Boolean value
    Bool(bool),
    /// Integer value
    Integer(i64),
    /// Floating-point value
    Float(f64),
    /// String value
    String(String),
    /// Array of values
    Array(Vec<MergeValue>),
    /// Object/map of values (ordered to preserve key order)
    Object(IndexMap<String, MergeValue>),
}

impl MergeValue {
    /// Check if this value is null
    pub fn is_null(&self) -> bool {
        matches!(self, MergeValue::Null)
    }

    /// Check if this value is an object
    pub fn is_object(&self) -> bool {
        matches!(self, MergeValue::Object(_))
    }

    /// Check if this value is an array
    pub fn is_array(&self) -> bool {
        matches!(self, MergeValue::Array(_))
    }

    /// Get as object reference
    pub fn as_object(&self) -> Option<&IndexMap<String, MergeValue>> {
        match self {
            MergeValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Get as mutable object reference
    pub fn as_object_mut(&mut self) -> Option<&mut IndexMap<String, MergeValue>> {
        match self {
            MergeValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Get as array reference
    pub fn as_array(&self) -> Option<&Vec<MergeValue>> {
        match self {
            MergeValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get as string reference
    pub fn as_str(&self) -> Option<&str> {
        match self {
            MergeValue::String(s) => Some(s),
            _ => None,
        }
    }

    // ================== Parsing Helpers ==================

    /// Parse a JSON string into a MergeValue
    pub fn from_json(s: &str) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(s).map_err(|e| JinError::Parse {
            format: "JSON".to_string(),
            message: e.to_string(),
        })?;
        Ok(Self::from(value))
    }

    /// Parse a YAML string into a MergeValue
    pub fn from_yaml(s: &str) -> Result<Self> {
        let value: serde_yaml::Value = serde_yaml::from_str(s).map_err(|e| {
            let location_info = e
                .location()
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
        let value: toml::Value = toml::from_str(s).map_err(|e| JinError::Parse {
            format: "TOML".to_string(),
            message: e.to_string(),
        })?;
        Ok(Self::from(value))
    }

    /// Parse an INI string into a MergeValue
    pub fn from_ini(s: &str) -> Result<Self> {
        let ini = Ini::load_from_str(s).map_err(|e| JinError::Parse {
            format: "INI".to_string(),
            message: e.to_string(),
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

    // ================== Serialization Helpers ==================

    /// Serialize to a pretty-printed JSON string
    pub fn to_json_string(&self) -> Result<String> {
        let json_value: serde_json::Value = self.clone().into();
        serde_json::to_string_pretty(&json_value).map_err(|e| JinError::Parse {
            format: "JSON".to_string(),
            message: e.to_string(),
        })
    }

    /// Serialize to a compact JSON string (no formatting)
    pub fn to_json_string_compact(&self) -> Result<String> {
        let json_value: serde_json::Value = self.clone().into();
        serde_json::to_string(&json_value).map_err(|e| JinError::Parse {
            format: "JSON".to_string(),
            message: e.to_string(),
        })
    }

    /// Serialize to a YAML string
    pub fn to_yaml_string(&self) -> Result<String> {
        let yaml_value: serde_yaml::Value = self.clone().into();
        serde_yaml::to_string(&yaml_value).map_err(|e| JinError::Parse {
            format: "YAML".to_string(),
            message: e.to_string(),
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
        toml::to_string_pretty(&toml_value).map_err(|e| JinError::Parse {
            format: "TOML".to_string(),
            message: e.to_string(),
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
        ini.write_to(&mut output).map_err(|e| JinError::Parse {
            format: "INI".to_string(),
            message: e.to_string(),
        })?;
        String::from_utf8(output).map_err(|e| JinError::Parse {
            format: "INI".to_string(),
            message: e.to_string(),
        })
    }

    // ================== Type-Checking Helpers ==================

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

impl From<serde_json::Value> for MergeValue {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => MergeValue::Null,
            serde_json::Value::Bool(b) => MergeValue::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    MergeValue::Integer(i)
                } else if let Some(f) = n.as_f64() {
                    MergeValue::Float(f)
                } else {
                    MergeValue::Float(0.0)
                }
            }
            serde_json::Value::String(s) => MergeValue::String(s),
            serde_json::Value::Array(arr) => {
                MergeValue::Array(arr.into_iter().map(MergeValue::from).collect())
            }
            serde_json::Value::Object(obj) => MergeValue::Object(
                obj.into_iter()
                    .map(|(k, v)| (k, MergeValue::from(v)))
                    .collect(),
            ),
        }
    }
}

impl From<MergeValue> for serde_json::Value {
    fn from(value: MergeValue) -> Self {
        match value {
            MergeValue::Null => serde_json::Value::Null,
            MergeValue::Bool(b) => serde_json::Value::Bool(b),
            MergeValue::Integer(i) => serde_json::Value::Number(i.into()),
            MergeValue::Float(f) => serde_json::Number::from_f64(f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            MergeValue::String(s) => serde_json::Value::String(s),
            MergeValue::Array(arr) => {
                serde_json::Value::Array(arr.into_iter().map(serde_json::Value::from).collect())
            }
            MergeValue::Object(obj) => serde_json::Value::Object(
                obj.into_iter()
                    .map(|(k, v)| (k, serde_json::Value::from(v)))
                    .collect(),
            ),
        }
    }
}

// ================== YAML Conversions ==================

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
                    // Fallback for unsigned integers that don't fit i64
                    MergeValue::Float(n.as_f64().unwrap_or(0.0))
                }
            }
            serde_yaml::Value::String(s) => MergeValue::String(s),
            serde_yaml::Value::Sequence(seq) => {
                MergeValue::Array(seq.into_iter().map(MergeValue::from).collect())
            }
            serde_yaml::Value::Mapping(map) => {
                // Filter to string keys only - YAML allows non-string keys
                let obj: IndexMap<String, MergeValue> = map
                    .into_iter()
                    .filter_map(|(k, v)| k.as_str().map(|s| (s.to_string(), MergeValue::from(v))))
                    .collect();
                MergeValue::Object(obj)
            }
            // Handle tagged values by extracting inner value
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
                // Handle NaN/Infinity - they serialize as .nan/.inf in YAML
                if f.is_nan() || f.is_infinite() {
                    // Use string representation for special floats
                    if f.is_nan() {
                        serde_yaml::Value::Number(serde_yaml::Number::from(f64::NAN))
                    } else if f.is_sign_positive() {
                        serde_yaml::Value::Number(serde_yaml::Number::from(f64::INFINITY))
                    } else {
                        serde_yaml::Value::Number(serde_yaml::Number::from(f64::NEG_INFINITY))
                    }
                } else {
                    serde_yaml::Value::Number(serde_yaml::Number::from(f))
                }
            }
            MergeValue::String(s) => serde_yaml::Value::String(s),
            MergeValue::Array(arr) => {
                serde_yaml::Value::Sequence(arr.into_iter().map(serde_yaml::Value::from).collect())
            }
            MergeValue::Object(obj) => {
                let mut map = serde_yaml::Mapping::new();
                for (k, v) in obj {
                    map.insert(serde_yaml::Value::String(k), serde_yaml::Value::from(v));
                }
                serde_yaml::Value::Mapping(map)
            }
        }
    }
}

// ================== TOML Conversions ==================

impl From<toml::Value> for MergeValue {
    fn from(value: toml::Value) -> Self {
        match value {
            toml::Value::String(s) => MergeValue::String(s),
            toml::Value::Integer(i) => MergeValue::Integer(i),
            toml::Value::Float(f) => MergeValue::Float(f),
            toml::Value::Boolean(b) => MergeValue::Bool(b),
            // Datetime converts to string representation
            toml::Value::Datetime(dt) => MergeValue::String(dt.to_string()),
            toml::Value::Array(arr) => {
                MergeValue::Array(arr.into_iter().map(MergeValue::from).collect())
            }
            toml::Value::Table(table) => {
                // Note: toml::Table is BTreeMap, so keys come alphabetically sorted
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
        fn convert(value: MergeValue) -> std::result::Result<toml::Value, JinError> {
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
                    let converted: std::result::Result<Vec<toml::Value>, JinError> =
                        arr.into_iter().map(convert).collect();
                    Ok(toml::Value::Array(converted?))
                }
                MergeValue::Object(obj) => {
                    let mut table = toml::Table::new();
                    for (k, v) in obj {
                        table.insert(k, convert(v)?);
                    }
                    Ok(toml::Value::Table(table))
                }
            }
        }
        convert(value)
    }
}

// ================== INI Conversions ==================

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
                                message: "INI does not support nested objects beyond 2 levels"
                                    .to_string(),
                            });
                        }
                    };
                    ini.with_section(Some(section_name.as_str()))
                        .set(key, string_val);
                }
            }
            // Root-level non-object values go to general section
            MergeValue::String(s) => {
                ini.with_section(None::<String>)
                    .set(section_name, s.clone());
            }
            MergeValue::Bool(b) => {
                ini.with_section(None::<String>)
                    .set(section_name, b.to_string());
            }
            MergeValue::Integer(i) => {
                ini.with_section(None::<String>)
                    .set(section_name, i.to_string());
            }
            MergeValue::Float(f) => {
                ini.with_section(None::<String>)
                    .set(section_name, f.to_string());
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

#[cfg(test)]
mod tests {
    use super::*;

    // ========== Existing Tests ==========

    #[test]
    fn test_merge_value_null() {
        let val = MergeValue::Null;
        assert!(val.is_null());
        assert!(!val.is_object());
    }

    #[test]
    fn test_merge_value_object() {
        let mut obj = IndexMap::new();
        obj.insert("key".to_string(), MergeValue::String("value".to_string()));
        let val = MergeValue::Object(obj);

        assert!(val.is_object());
        assert!(val.as_object().is_some());
    }

    #[test]
    fn test_json_roundtrip() {
        let json = serde_json::json!({
            "name": "test",
            "count": 42,
            "active": true,
            "items": ["a", "b", "c"]
        });

        let merge_val = MergeValue::from(json.clone());
        let back: serde_json::Value = merge_val.into();

        assert_eq!(json, back);
    }

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
    fn test_yaml_conversion_from_value() {
        let yaml_val = serde_yaml::Value::Null;
        let merge_val = MergeValue::from(yaml_val);
        assert!(merge_val.is_null());

        let yaml_val = serde_yaml::Value::Bool(true);
        let merge_val = MergeValue::from(yaml_val);
        assert_eq!(merge_val.as_bool(), Some(true));

        let yaml_val = serde_yaml::Value::String("hello".to_string());
        let merge_val = MergeValue::from(yaml_val);
        assert_eq!(merge_val.as_str(), Some("hello"));
    }

    #[test]
    fn test_yaml_nested_objects() {
        let yaml = r#"
outer:
  inner:
    deep: value
    count: 42
"#;
        let val = MergeValue::from_yaml(yaml).unwrap();
        let outer = val.as_object().unwrap().get("outer").unwrap();
        let inner = outer.as_object().unwrap().get("inner").unwrap();
        let deep = inner.as_object().unwrap().get("deep").unwrap();
        assert_eq!(deep.as_str(), Some("value"));
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

    #[test]
    fn test_toml_conversion_tryfrom() {
        let merge_val = MergeValue::Integer(42);
        let toml_val: toml::Value = merge_val.try_into().unwrap();
        assert_eq!(toml_val.as_integer(), Some(42));

        let merge_val = MergeValue::String("hello".to_string());
        let toml_val: toml::Value = merge_val.try_into().unwrap();
        assert_eq!(toml_val.as_str(), Some("hello"));
    }

    #[test]
    fn test_toml_nested_tables() {
        let toml = r#"
[database]
host = "localhost"
port = 5432

[logging]
level = "info"
"#;
        let val = MergeValue::from_toml(toml).unwrap();
        let db = val.as_object().unwrap().get("database").unwrap();
        let host = db.as_object().unwrap().get("host").unwrap();
        assert_eq!(host.as_str(), Some("localhost"));
    }

    // ========== INI Tests ==========

    #[test]
    fn test_ini_roundtrip_basic() {
        let ini = r#"
[database]
host=localhost
port=5432

[logging]
level=info
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
key=value
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
global_key=global_value

[section]
key=value
"#;
        let val = MergeValue::from_ini(ini).unwrap();
        let obj = val.as_object().unwrap();
        // General section values at root level
        assert_eq!(
            obj.get("global_key").unwrap().as_str(),
            Some("global_value")
        );
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
        // YAML should parse back correctly
        let reparsed = MergeValue::from_yaml(&yaml).unwrap();
        assert_eq!(val, reparsed);
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

    #[test]
    fn test_cross_format_data_preservation() {
        // Create a value, convert through multiple formats
        let original = MergeValue::from(serde_json::json!({
            "name": "test",
            "count": 42,
            "enabled": true,
            "nested": {
                "key": "value"
            }
        }));

        // JSON -> YAML -> JSON
        let yaml = original.to_yaml_string().unwrap();
        let from_yaml = MergeValue::from_yaml(&yaml).unwrap();
        assert_eq!(original, from_yaml);

        // JSON -> TOML -> JSON
        let toml = original.to_toml_string().unwrap();
        let from_toml = MergeValue::from_toml(&toml).unwrap();
        assert_eq!(original, from_toml);
    }

    // ========== File Detection Tests ==========

    #[test]
    fn test_file_extension_detection() {
        use std::io::Write;
        use tempfile::NamedTempFile;

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
        writeln!(ini_file, "[section]\ntest=true").unwrap();
        let val = MergeValue::from_file(ini_file.path()).unwrap();
        assert!(val.as_object().unwrap().contains_key("section"));
    }

    #[test]
    fn test_yml_extension() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut yml_file = NamedTempFile::with_suffix(".yml").unwrap();
        writeln!(yml_file, "test: true").unwrap();
        let val = MergeValue::from_file(yml_file.path()).unwrap();
        assert!(val.as_object().unwrap().contains_key("test"));
    }

    #[test]
    fn test_unsupported_extension() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut txt_file = NamedTempFile::with_suffix(".txt").unwrap();
        writeln!(txt_file, "test").unwrap();
        let result = MergeValue::from_file(txt_file.path());
        assert!(result.is_err());
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
    fn test_contains_null_in_array() {
        let arr = MergeValue::from(serde_json::json!([1, null, 3]));
        assert!(arr.contains_null());

        let arr_no_null = MergeValue::from(serde_json::json!([1, 2, 3]));
        assert!(!arr_no_null.contains_null());
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

    // ========== Parsing Helper Tests ==========

    #[test]
    fn test_from_json_helper() {
        let json = r#"{"key": "value", "num": 42}"#;
        let val = MergeValue::from_json(json).unwrap();
        assert_eq!(
            val.as_object().unwrap().get("key").unwrap().as_str(),
            Some("value")
        );
        assert_eq!(
            val.as_object().unwrap().get("num").unwrap().as_i64(),
            Some(42)
        );
    }

    #[test]
    fn test_from_json_invalid() {
        let result = MergeValue::from_json("{invalid json}");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_yaml_helper() {
        let yaml = "key: value\nnum: 42";
        let val = MergeValue::from_yaml(yaml).unwrap();
        assert_eq!(
            val.as_object().unwrap().get("key").unwrap().as_str(),
            Some("value")
        );
        assert_eq!(
            val.as_object().unwrap().get("num").unwrap().as_i64(),
            Some(42)
        );
    }

    #[test]
    fn test_from_yaml_invalid() {
        let result = MergeValue::from_yaml("invalid: [yaml: content");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_toml_helper() {
        let val = MergeValue::from_toml("key = \"value\"\nnum = 42").unwrap();
        assert_eq!(
            val.as_object().unwrap().get("key").unwrap().as_str(),
            Some("value")
        );
        assert_eq!(
            val.as_object().unwrap().get("num").unwrap().as_i64(),
            Some(42)
        );
    }

    #[test]
    fn test_from_toml_invalid() {
        let result = MergeValue::from_toml("invalid = [toml");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_ini_helper() {
        let ini = "[section]\nkey=value";
        let val = MergeValue::from_ini(ini).unwrap();
        let section = val.as_object().unwrap().get("section").unwrap();
        assert_eq!(
            section.as_object().unwrap().get("key").unwrap().as_str(),
            Some("value")
        );
    }

    // ========== Serialization Helper Tests ==========

    #[test]
    fn test_to_json_string() {
        let val = MergeValue::from(serde_json::json!({"key": "value"}));
        let json = val.to_json_string().unwrap();
        assert!(json.contains("\"key\""));
        assert!(json.contains("\"value\""));
    }

    #[test]
    fn test_to_json_string_compact() {
        let val = MergeValue::from(serde_json::json!({"key": "value"}));
        let json = val.to_json_string_compact().unwrap();
        // Compact should have no newlines
        assert!(!json.contains('\n'));
    }

    #[test]
    fn test_to_yaml_string() {
        let val = MergeValue::from(serde_json::json!({"key": "value"}));
        let yaml = val.to_yaml_string().unwrap();
        assert!(yaml.contains("key:"));
    }

    #[test]
    fn test_to_toml_string() {
        let val = MergeValue::from(serde_json::json!({"key": "value"}));
        let toml = val.to_toml_string().unwrap();
        assert!(toml.contains("key = \"value\""));
    }

    #[test]
    fn test_to_ini_string() {
        let val = MergeValue::from(serde_json::json!({
            "section": {
                "key": "value"
            }
        }));
        let ini = val.to_ini_string().unwrap();
        assert!(ini.contains("[section]"));
        assert!(ini.contains("key=value"));
    }

    // ========== Edge Case Tests ==========

    #[test]
    fn test_empty_object() {
        let val = MergeValue::Object(IndexMap::new());
        assert_eq!(val.to_json_string().unwrap(), "{}");
        let yaml = val.to_yaml_string().unwrap();
        assert!(yaml.contains("{}") || yaml.is_empty() || yaml.trim() == "{}");
    }

    #[test]
    fn test_empty_array() {
        let val = MergeValue::Array(vec![]);
        assert_eq!(val.to_json_string().unwrap(), "[]");
    }

    #[test]
    fn test_special_characters_in_strings() {
        let val = MergeValue::String("hello\nworld\ttab".to_string());
        let json = val.to_json_string().unwrap();
        // JSON should escape special characters
        assert!(json.contains("\\n") || json.contains("\\t"));
    }

    #[test]
    fn test_integer_vs_float_preservation() {
        // Integer should stay integer
        let val = MergeValue::Integer(42);
        let json = val.to_json_string().unwrap();
        assert_eq!(json.trim(), "42");

        // Float should stay float
        let val = MergeValue::Float(3.14);
        let json = val.to_json_string().unwrap();
        assert!(json.contains("3.14"));
    }

    #[test]
    fn test_ini_root_level_scalars() {
        let val = MergeValue::from(serde_json::json!({
            "global_key": "global_value"
        }));
        let ini = val.to_ini_string().unwrap();
        // Scalar at root level should be in general section
        assert!(ini.contains("global_key=global_value"));
    }

    #[test]
    fn test_ini_number_conversion() {
        let val = MergeValue::from(serde_json::json!({
            "section": {
                "port": 5432,
                "ratio": 3.14,
                "enabled": true
            }
        }));
        let ini = val.to_ini_string().unwrap();
        // Numbers and booleans should be converted to strings
        assert!(ini.contains("port=5432"));
        assert!(ini.contains("ratio=3.14"));
        assert!(ini.contains("enabled=true"));
    }
}
