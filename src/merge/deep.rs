//! Deep merge logic for structured configuration

use super::MergeValue;
use crate::core::Result;
use indexmap::IndexMap;

/// Perform a deep merge of two MergeValues
///
/// Rules:
/// - Objects are merged recursively
/// - Arrays with keyed items (by "id" or "name") are merged
/// - Other arrays are replaced by the higher-precedence value
/// - Null values delete keys
/// - Scalars are replaced by the higher-precedence value
pub fn deep_merge(base: MergeValue, overlay: MergeValue) -> Result<MergeValue> {
    match (base, overlay) {
        // Null in overlay deletes the key
        (_, MergeValue::Null) => Ok(MergeValue::Null),

        // Both objects: recursive merge
        (MergeValue::Object(mut base_obj), MergeValue::Object(overlay_obj)) => {
            for (key, overlay_val) in overlay_obj {
                if overlay_val.is_null() {
                    // Null removes the key
                    base_obj.shift_remove(&key);
                } else if let Some(base_val) = base_obj.shift_remove(&key) {
                    // Recursively merge existing keys
                    let merged = deep_merge(base_val, overlay_val)?;
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
            let result = merge_arrays(base_arr, overlay_arr)?;
            Ok(MergeValue::Array(result))
        }

        // Different types or scalars: overlay wins
        (_, overlay) => Ok(overlay),
    }
}

/// Merge two arrays, attempting to merge by "id" or "name" keys if present
fn merge_arrays(base: Vec<MergeValue>, overlay: Vec<MergeValue>) -> Result<Vec<MergeValue>> {
    // Check if arrays have keyed objects
    let base_keyed = extract_array_keys(&base);
    let overlay_keyed = extract_array_keys(&overlay);

    if base_keyed.is_some() && overlay_keyed.is_some() {
        // Merge by key
        let base_map = base_keyed.unwrap();
        let overlay_map = overlay_keyed.unwrap();

        let mut result: IndexMap<String, MergeValue> = IndexMap::new();

        // Add all base items
        for (key, val) in base_map {
            result.insert(key, val);
        }

        // Merge or add overlay items
        for (key, overlay_val) in overlay_map {
            if let Some(base_val) = result.shift_remove(&key) {
                let merged = deep_merge(base_val, overlay_val)?;
                result.insert(key, merged);
            } else {
                result.insert(key, overlay_val);
            }
        }

        Ok(result.into_values().collect())
    } else {
        // No keys found, overlay replaces
        Ok(overlay)
    }
}

/// Extract keys from array items if they have "id" or "name" fields
fn extract_array_keys(arr: &[MergeValue]) -> Option<IndexMap<String, MergeValue>> {
    let mut result = IndexMap::new();

    for item in arr {
        if let MergeValue::Object(obj) = item {
            // Try "id" first, then "name"
            let key = obj
                .get("id")
                .and_then(|v| v.as_str())
                .or_else(|| obj.get("name").and_then(|v| v.as_str()));

            if let Some(k) = key {
                result.insert(k.to_string(), item.clone());
            } else {
                // Item without key, can't do keyed merge
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

    #[test]
    fn test_deep_merge_objects() {
        let base = json_to_merge(serde_json::json!({
            "a": 1,
            "b": 2
        }));
        let overlay = json_to_merge(serde_json::json!({
            "b": 3,
            "c": 4
        }));

        let result = deep_merge(base, overlay).unwrap();
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("a").unwrap().as_str(), None); // It's an integer
        assert_eq!(obj.len(), 3);
    }

    #[test]
    fn test_deep_merge_null_deletes() {
        let base = json_to_merge(serde_json::json!({
            "keep": 1,
            "delete": 2
        }));
        let overlay = json_to_merge(serde_json::json!({
            "delete": null
        }));

        let result = deep_merge(base, overlay).unwrap();
        let obj = result.as_object().unwrap();

        assert!(obj.contains_key("keep"));
        assert!(!obj.contains_key("delete"));
    }

    #[test]
    fn test_deep_merge_nested() {
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

        assert_eq!(inner.len(), 3);
    }
}
