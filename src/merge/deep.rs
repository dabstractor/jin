//! Deep merge logic for structured configuration
//!
//! Implements RFC 7396 (JSON Merge Patch) semantics with extensions for
//! keyed array merging. Key behaviors:
//! - Null values delete keys (RFC 7396)
//! - Objects merge recursively
//! - Arrays with keyed items (by "id" or "name") merge by key
//! - Other arrays are replaced by the higher-precedence value

use super::MergeValue;
use crate::core::Result;
use indexmap::IndexMap;

/// Configuration for merge operations
#[derive(Debug, Clone)]
pub struct MergeConfig {
    /// Key fields to use for keyed array merge (default: ["id", "name"])
    pub array_key_fields: Vec<String>,
}

impl Default for MergeConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl MergeConfig {
    /// Create config with default settings (key fields: ["id", "name"])
    pub fn new() -> Self {
        Self {
            array_key_fields: vec!["id".to_string(), "name".to_string()],
        }
    }

    /// Create config with custom key fields
    pub fn with_key_fields(fields: Vec<String>) -> Self {
        Self {
            array_key_fields: fields,
        }
    }
}

/// Perform a deep merge of two MergeValues using default configuration.
///
/// This is a convenience wrapper around `deep_merge_with_config` that uses
/// the default key fields ["id", "name"] for keyed array merging.
///
/// # Rules
///
/// - Objects are merged recursively
/// - Arrays with keyed items (by "id" or "name") are merged by key
/// - Other arrays are replaced by the higher-precedence value
/// - Null values delete keys (RFC 7396 semantics)
/// - Scalars are replaced by the higher-precedence value
pub fn deep_merge(base: MergeValue, overlay: MergeValue) -> Result<MergeValue> {
    deep_merge_with_config(base, overlay, &MergeConfig::new())
}

/// Perform a deep merge of two MergeValues with custom configuration.
///
/// # Arguments
///
/// * `base` - The base value (lower precedence)
/// * `overlay` - The overlay value (higher precedence)
/// * `config` - Merge configuration (key fields for array merging)
///
/// # Rules
///
/// - Objects are merged recursively
/// - Arrays with keyed items merge by configured key fields
/// - Other arrays are replaced by the higher-precedence value
/// - Null values delete keys (RFC 7396 semantics)
/// - Type conflicts: overlay wins completely
pub fn deep_merge_with_config(
    base: MergeValue,
    overlay: MergeValue,
    config: &MergeConfig,
) -> Result<MergeValue> {
    match (base, overlay) {
        // Null in overlay = delete the key (RFC 7396)
        (_, MergeValue::Null) => Ok(MergeValue::Null),

        // Both objects: recursive merge
        (MergeValue::Object(mut base_obj), MergeValue::Object(overlay_obj)) => {
            for (key, overlay_val) in overlay_obj {
                if overlay_val.is_null() {
                    // Null removes the key entirely
                    base_obj.shift_remove(&key);
                } else if let Some(base_val) = base_obj.shift_remove(&key) {
                    // Recursively merge existing keys
                    let merged = deep_merge_with_config(base_val, overlay_val, config)?;
                    if !merged.is_null() {
                        base_obj.insert(key, merged);
                    }
                } else {
                    // Add new keys from overlay
                    base_obj.insert(key, overlay_val);
                }
            }
            Ok(MergeValue::Object(base_obj))
        }

        // Both arrays: attempt keyed merge, otherwise replace
        (MergeValue::Array(base_arr), MergeValue::Array(overlay_arr)) => {
            // Empty overlay array replaces entirely
            if overlay_arr.is_empty() {
                return Ok(MergeValue::Array(overlay_arr));
            }

            let result = merge_arrays_with_config(base_arr, overlay_arr, config)?;
            Ok(MergeValue::Array(result))
        }

        // Different types or scalars: overlay wins
        (_, overlay) => Ok(overlay),
    }
}

/// Merge two arrays with configuration.
///
/// If both arrays contain objects with key fields (as defined in config),
/// they are merged by key. Otherwise, overlay replaces base entirely.
///
/// # Order Preservation
///
/// When merging keyed arrays:
/// 1. Base array items maintain their original order
/// 2. Overlay items matching base keys are merged in place
/// 3. New overlay items (not in base) are appended at the end
fn merge_arrays_with_config(
    base: Vec<MergeValue>,
    overlay: Vec<MergeValue>,
    config: &MergeConfig,
) -> Result<Vec<MergeValue>> {
    // Check if arrays have keyed objects
    let base_keyed = extract_array_keys(&base, &config.array_key_fields);
    let overlay_keyed = extract_array_keys(&overlay, &config.array_key_fields);

    if let (Some(base_map), Some(mut overlay_map)) = (base_keyed, overlay_keyed) {
        // Merge by key, preserving base order
        let mut result: Vec<MergeValue> = Vec::new();

        // Process base items in order, merging with overlay if present
        for (key, base_val) in base_map {
            if let Some(overlay_val) = overlay_map.shift_remove(&key) {
                // Merge overlay into base item
                let merged = deep_merge_with_config(base_val, overlay_val, config)?;
                result.push(merged);
            } else {
                // Keep base item as-is
                result.push(base_val);
            }
        }

        // Append remaining overlay items (new keys not in base)
        for (_key, overlay_val) in overlay_map {
            result.push(overlay_val);
        }

        Ok(result)
    } else {
        // No keys found or mixed array, overlay replaces
        Ok(overlay)
    }
}

/// Extract keys from array items if they all have one of the specified key fields.
///
/// Returns None if:
/// - Any item is not an object
/// - Any object lacks all specified key fields
/// - This ensures consistent behavior (no partial merging of mixed arrays)
fn extract_array_keys(
    arr: &[MergeValue],
    key_fields: &[String],
) -> Option<IndexMap<String, MergeValue>> {
    let mut result = IndexMap::new();

    for item in arr {
        if let MergeValue::Object(obj) = item {
            // Try each key field in order of priority
            let key = key_fields
                .iter()
                .find_map(|field| obj.get(field).and_then(|v| v.as_str()));

            if let Some(k) = key {
                result.insert(k.to_string(), item.clone());
            } else {
                // Item without any key field, can't do keyed merge
                return None;
            }
        } else {
            // Non-object in array, can't do keyed merge
            return None;
        }
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn json_to_merge(json: serde_json::Value) -> MergeValue {
        MergeValue::from(json)
    }

    // ========== MergeConfig Tests ==========

    #[test]
    fn test_merge_config_default() {
        let config = MergeConfig::new();
        assert_eq!(config.array_key_fields, vec!["id", "name"]);
    }

    #[test]
    fn test_merge_config_default_trait() {
        let config = MergeConfig::default();
        assert_eq!(config.array_key_fields, vec!["id", "name"]);
    }

    #[test]
    fn test_merge_config_custom_keys() {
        let config = MergeConfig::with_key_fields(vec!["key".into(), "uuid".into()]);
        assert_eq!(config.array_key_fields, vec!["key", "uuid"]);
    }

    // ========== Null Deletion Tests ==========

    #[test]
    fn test_null_deletes_top_level_key() {
        let base = json_to_merge(serde_json::json!({"keep": 1, "delete": 2}));
        let overlay = json_to_merge(serde_json::json!({"delete": null}));

        let result = deep_merge(base, overlay).unwrap();
        let obj = result.as_object().unwrap();

        assert!(obj.contains_key("keep"));
        assert!(!obj.contains_key("delete"));
        assert_eq!(obj.len(), 1);
    }

    #[test]
    fn test_null_deletes_nested_key() {
        let base = json_to_merge(serde_json::json!({"outer": {"keep": 1, "delete": 2}}));
        let overlay = json_to_merge(serde_json::json!({"outer": {"delete": null}}));

        let result = deep_merge(base, overlay).unwrap();
        let outer = result.as_object().unwrap().get("outer").unwrap();
        let inner = outer.as_object().unwrap();

        assert!(inner.contains_key("keep"));
        assert!(!inner.contains_key("delete"));
    }

    #[test]
    fn test_null_deletes_deeply_nested_key() {
        let base = json_to_merge(serde_json::json!({
            "a": { "b": { "c": { "keep": 1, "delete": 2 } } }
        }));
        let overlay = json_to_merge(serde_json::json!({
            "a": { "b": { "c": { "delete": null } } }
        }));

        let result = deep_merge(base, overlay).unwrap();
        let c = result
            .as_object()
            .unwrap()
            .get("a")
            .unwrap()
            .as_object()
            .unwrap()
            .get("b")
            .unwrap()
            .as_object()
            .unwrap()
            .get("c")
            .unwrap()
            .as_object()
            .unwrap();

        assert!(c.contains_key("keep"));
        assert!(!c.contains_key("delete"));
    }

    #[test]
    fn test_null_deletes_entire_nested_object() {
        let base = json_to_merge(serde_json::json!({
            "keep": 1,
            "nested": { "a": 1, "b": 2 }
        }));
        let overlay = json_to_merge(serde_json::json!({"nested": null}));

        let result = deep_merge(base, overlay).unwrap();
        let obj = result.as_object().unwrap();

        assert!(obj.contains_key("keep"));
        assert!(!obj.contains_key("nested"));
    }

    #[test]
    fn test_overlay_null_at_root_returns_null() {
        let base = json_to_merge(serde_json::json!({"a": 1}));
        let overlay = MergeValue::Null;

        let result = deep_merge(base, overlay).unwrap();
        assert!(result.is_null());
    }

    // ========== Object Merge Tests ==========

    #[test]
    fn test_object_merge_adds_new_keys() {
        let base = json_to_merge(serde_json::json!({"a": 1}));
        let overlay = json_to_merge(serde_json::json!({"b": 2}));

        let result = deep_merge(base, overlay).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("a").unwrap().as_i64(), Some(1));
        assert_eq!(obj.get("b").unwrap().as_i64(), Some(2));
    }

    #[test]
    fn test_object_merge_overlay_wins_on_conflict() {
        let base = json_to_merge(serde_json::json!({"a": 1}));
        let overlay = json_to_merge(serde_json::json!({"a": 2}));

        let result = deep_merge(base, overlay).unwrap();
        assert_eq!(
            result.as_object().unwrap().get("a").unwrap().as_i64(),
            Some(2)
        );
    }

    #[test]
    fn test_object_merge_recursive() {
        let base = json_to_merge(serde_json::json!({
            "outer": {
                "inner1": "a",
                "inner2": "b"
            }
        }));
        let overlay = json_to_merge(serde_json::json!({
            "outer": {
                "inner2": "B",
                "inner3": "c"
            }
        }));

        let result = deep_merge(base, overlay).unwrap();
        let outer = result.as_object().unwrap().get("outer").unwrap();
        let inner = outer.as_object().unwrap();

        assert_eq!(inner.get("inner1").unwrap().as_str(), Some("a"));
        assert_eq!(inner.get("inner2").unwrap().as_str(), Some("B"));
        assert_eq!(inner.get("inner3").unwrap().as_str(), Some("c"));
    }

    #[test]
    fn test_type_conflict_object_to_scalar() {
        let base = json_to_merge(serde_json::json!({"a": {"nested": true}}));
        let overlay = json_to_merge(serde_json::json!({"a": "string"}));

        let result = deep_merge(base, overlay).unwrap();
        assert_eq!(
            result.as_object().unwrap().get("a").unwrap().as_str(),
            Some("string")
        );
    }

    #[test]
    fn test_type_conflict_scalar_to_object() {
        let base = json_to_merge(serde_json::json!({"a": "string"}));
        let overlay = json_to_merge(serde_json::json!({"a": {"nested": true}}));

        let result = deep_merge(base, overlay).unwrap();
        let a = result.as_object().unwrap().get("a").unwrap();
        assert!(a.as_object().is_some());
        assert_eq!(
            a.as_object().unwrap().get("nested").unwrap().as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_type_conflict_array_to_scalar() {
        let base = json_to_merge(serde_json::json!({"a": [1, 2, 3]}));
        let overlay = json_to_merge(serde_json::json!({"a": 42}));

        let result = deep_merge(base, overlay).unwrap();
        assert_eq!(
            result.as_object().unwrap().get("a").unwrap().as_i64(),
            Some(42)
        );
    }

    // ========== Keyed Array Merge Tests ==========

    #[test]
    fn test_keyed_array_merge_by_id() {
        let base = json_to_merge(serde_json::json!([
            {"id": "1", "value": "a"},
            {"id": "2", "value": "b"}
        ]));
        let overlay = json_to_merge(serde_json::json!([{"id": "2", "value": "B"}]));

        let result = deep_merge(base, overlay).unwrap();
        let arr = result.as_array().unwrap();

        assert_eq!(arr.len(), 2);
        let item2 = arr
            .iter()
            .find(|v| v.as_object().unwrap().get("id").unwrap().as_str() == Some("2"))
            .unwrap();
        assert_eq!(
            item2.as_object().unwrap().get("value").unwrap().as_str(),
            Some("B")
        );
    }

    #[test]
    fn test_keyed_array_merge_by_name() {
        let base = json_to_merge(serde_json::json!([
            {"name": "item1", "val": 1},
            {"name": "item2", "val": 2}
        ]));
        let overlay = json_to_merge(serde_json::json!([{"name": "item1", "val": 10}]));

        let result = deep_merge(base, overlay).unwrap();
        let arr = result.as_array().unwrap();

        assert_eq!(arr.len(), 2);
        let item1 = arr
            .iter()
            .find(|v| v.as_object().unwrap().get("name").unwrap().as_str() == Some("item1"))
            .unwrap();
        assert_eq!(
            item1.as_object().unwrap().get("val").unwrap().as_i64(),
            Some(10)
        );
    }

    #[test]
    fn test_keyed_array_appends_new_items() {
        let base = json_to_merge(serde_json::json!([{"id": "1", "v": "a"}]));
        let overlay = json_to_merge(serde_json::json!([{"id": "2", "v": "b"}]));

        let result = deep_merge(base, overlay).unwrap();
        let arr = result.as_array().unwrap();

        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn test_keyed_array_preserves_order() {
        let base = json_to_merge(serde_json::json!([{"id": "1"}, {"id": "2"}, {"id": "3"}]));
        let overlay = json_to_merge(serde_json::json!([{"id": "2", "new": true}, {"id": "4"}]));

        let result = deep_merge(base, overlay).unwrap();
        let arr = result.as_array().unwrap();

        let ids: Vec<_> = arr
            .iter()
            .map(|v| v.as_object().unwrap().get("id").unwrap().as_str().unwrap())
            .collect();
        // Base items first in original order, new items appended
        assert_eq!(ids, vec!["1", "2", "3", "4"]);
    }

    #[test]
    fn test_keyed_array_deep_merge_items() {
        let base = json_to_merge(serde_json::json!([
            {"id": "1", "nested": {"a": 1, "b": 2}}
        ]));
        let overlay = json_to_merge(serde_json::json!([
            {"id": "1", "nested": {"b": 20, "c": 30}}
        ]));

        let result = deep_merge(base, overlay).unwrap();
        let arr = result.as_array().unwrap();
        let item = &arr[0];
        let nested = item.as_object().unwrap().get("nested").unwrap();
        let obj = nested.as_object().unwrap();

        assert_eq!(obj.get("a").unwrap().as_i64(), Some(1));
        assert_eq!(obj.get("b").unwrap().as_i64(), Some(20));
        assert_eq!(obj.get("c").unwrap().as_i64(), Some(30));
    }

    #[test]
    fn test_keyed_array_with_custom_key_fields() {
        let config = MergeConfig::with_key_fields(vec!["key".into(), "uuid".into()]);

        let base = json_to_merge(serde_json::json!([{"key": "1", "v": "a"}]));
        let overlay = json_to_merge(serde_json::json!([{"key": "1", "v": "b"}]));

        let result = deep_merge_with_config(base, overlay, &config).unwrap();
        let arr = result.as_array().unwrap();

        assert_eq!(arr.len(), 1);
        assert_eq!(
            arr[0].as_object().unwrap().get("v").unwrap().as_str(),
            Some("b")
        );
    }

    // ========== Unkeyed Array Tests ==========

    #[test]
    fn test_unkeyed_array_replacement_primitives() {
        let base = json_to_merge(serde_json::json!([1, 2, 3]));
        let overlay = json_to_merge(serde_json::json!([4, 5]));

        let result = deep_merge(base, overlay).unwrap();
        let arr = result.as_array().unwrap();

        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_i64(), Some(4));
        assert_eq!(arr[1].as_i64(), Some(5));
    }

    #[test]
    fn test_unkeyed_array_replacement_strings() {
        let base = json_to_merge(serde_json::json!(["a", "b", "c"]));
        let overlay = json_to_merge(serde_json::json!(["x"]));

        let result = deep_merge(base, overlay).unwrap();
        let arr = result.as_array().unwrap();

        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].as_str(), Some("x"));
    }

    #[test]
    fn test_mixed_array_replacement() {
        // Array with some objects lacking id/name should be replaced
        let base = json_to_merge(serde_json::json!([{"id": "1"}, {"no_id": true}]));
        let overlay = json_to_merge(serde_json::json!([{"id": "2"}]));

        let result = deep_merge(base, overlay).unwrap();
        let arr = result.as_array().unwrap();

        assert_eq!(arr.len(), 1);
        assert_eq!(
            arr[0].as_object().unwrap().get("id").unwrap().as_str(),
            Some("2")
        );
    }

    #[test]
    fn test_mixed_types_in_array_replacement() {
        // Array with mixed types (objects and primitives) should be replaced
        let base = json_to_merge(serde_json::json!([{"id": "1"}, 42]));
        let overlay = json_to_merge(serde_json::json!(["replaced"]));

        let result = deep_merge(base, overlay).unwrap();
        let arr = result.as_array().unwrap();

        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0].as_str(), Some("replaced"));
    }

    // ========== Empty Value Tests ==========

    #[test]
    fn test_empty_overlay_array_replaces() {
        let base = json_to_merge(serde_json::json!([1, 2, 3]));
        let overlay = json_to_merge(serde_json::json!([]));

        let result = deep_merge(base, overlay).unwrap();
        assert!(result.as_array().unwrap().is_empty());
    }

    #[test]
    fn test_empty_overlay_object_merges() {
        let base = json_to_merge(serde_json::json!({"a": 1, "b": 2}));
        let overlay = json_to_merge(serde_json::json!({}));

        let result = deep_merge(base, overlay).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_i64(), Some(1));
        assert_eq!(obj.get("b").unwrap().as_i64(), Some(2));
    }

    #[test]
    fn test_empty_base_object_takes_overlay() {
        let base = json_to_merge(serde_json::json!({}));
        let overlay = json_to_merge(serde_json::json!({"a": 1}));

        let result = deep_merge(base, overlay).unwrap();
        let obj = result.as_object().unwrap();
        assert_eq!(obj.get("a").unwrap().as_i64(), Some(1));
    }

    #[test]
    fn test_empty_base_array_takes_overlay() {
        let base = json_to_merge(serde_json::json!([]));
        let overlay = json_to_merge(serde_json::json!([1, 2]));

        let result = deep_merge(base, overlay).unwrap();
        let arr = result.as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    // ========== Scalar Override Tests ==========

    #[test]
    fn test_scalar_string_override() {
        let base = MergeValue::String("base".into());
        let overlay = MergeValue::String("overlay".into());

        let result = deep_merge(base, overlay).unwrap();
        assert_eq!(result.as_str(), Some("overlay"));
    }

    #[test]
    fn test_scalar_integer_override() {
        let base = MergeValue::Integer(1);
        let overlay = MergeValue::Integer(2);

        let result = deep_merge(base, overlay).unwrap();
        assert_eq!(result.as_i64(), Some(2));
    }

    #[test]
    fn test_scalar_bool_override() {
        let base = MergeValue::Bool(false);
        let overlay = MergeValue::Bool(true);

        let result = deep_merge(base, overlay).unwrap();
        assert_eq!(result.as_bool(), Some(true));
    }

    // ========== Complex Scenarios ==========

    #[test]
    fn test_complex_nested_merge() {
        let base = json_to_merge(serde_json::json!({
            "app": {
                "name": "MyApp",
                "version": "1.0.0",
                "features": [
                    {"id": "auth", "enabled": true},
                    {"id": "logging", "level": "info"}
                ],
                "database": {
                    "host": "localhost",
                    "port": 5432
                }
            }
        }));

        let overlay = json_to_merge(serde_json::json!({
            "app": {
                "version": "1.1.0",
                "features": [
                    {"id": "auth", "enabled": false, "method": "oauth"},
                    {"id": "caching", "enabled": true}
                ],
                "database": {
                    "port": 5433,
                    "ssl": true
                }
            }
        }));

        let result = deep_merge(base, overlay).unwrap();
        let app = result.as_object().unwrap().get("app").unwrap();
        let app_obj = app.as_object().unwrap();

        // Check version override
        assert_eq!(app_obj.get("version").unwrap().as_str(), Some("1.1.0"));

        // Check database merge
        let db = app_obj.get("database").unwrap().as_object().unwrap();
        assert_eq!(db.get("host").unwrap().as_str(), Some("localhost"));
        assert_eq!(db.get("port").unwrap().as_i64(), Some(5433));
        assert_eq!(db.get("ssl").unwrap().as_bool(), Some(true));

        // Check features array merge
        let features = app_obj.get("features").unwrap().as_array().unwrap();
        assert_eq!(features.len(), 3); // auth, logging, caching

        let auth = features
            .iter()
            .find(|f| f.as_object().unwrap().get("id").unwrap().as_str() == Some("auth"))
            .unwrap();
        assert_eq!(
            auth.as_object().unwrap().get("enabled").unwrap().as_bool(),
            Some(false)
        );
        assert_eq!(
            auth.as_object().unwrap().get("method").unwrap().as_str(),
            Some("oauth")
        );
    }

    #[test]
    fn test_multiple_null_deletions() {
        let base = json_to_merge(serde_json::json!({
            "a": 1,
            "b": 2,
            "c": 3,
            "d": 4
        }));
        let overlay = json_to_merge(serde_json::json!({
            "a": null,
            "c": null
        }));

        let result = deep_merge(base, overlay).unwrap();
        let obj = result.as_object().unwrap();

        assert!(!obj.contains_key("a"));
        assert!(obj.contains_key("b"));
        assert!(!obj.contains_key("c"));
        assert!(obj.contains_key("d"));
    }

    // ========== Backward Compatibility Tests ==========

    #[test]
    fn test_deep_merge_backward_compatible() {
        // Ensure deep_merge still works with default config
        let base = json_to_merge(serde_json::json!({"a": 1}));
        let overlay = json_to_merge(serde_json::json!({"b": 2}));

        let result1 = deep_merge(base.clone(), overlay.clone()).unwrap();
        let result2 = deep_merge_with_config(base, overlay, &MergeConfig::new()).unwrap();

        assert_eq!(result1, result2);
    }
}
