//! Merge value types for structured configuration

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
