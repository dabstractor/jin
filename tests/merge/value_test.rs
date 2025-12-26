//! Unit tests for the MergeValue type.
//!
//! This test suite validates all MergeValue functionality including:
//! - Enum variant creation and manipulation
//! - Format parsing (JSON, YAML, TOML, INI)
//! - Deep merge operations following PRD §11.1 rules
//! - Helper methods and type conversions

use jin_glm::merge::value::MergeValue;

// ===== ENUM VARIANT CREATION TESTS =====

#[test]
fn test_enum_variants_create() {
    // Test all 7 variants can be created
    let _null = MergeValue::Null;
    let _bool = MergeValue::Boolean(true);
    let _int = MergeValue::Integer(42);
    let _float = MergeValue::Float(3.14);
    let _string = MergeValue::String("hello".to_string());
    let _array = MergeValue::Array(vec![]);
    let _object = MergeValue::Object(indexmap::IndexMap::new());

    // Verify equality works
    assert_eq!(MergeValue::Null, MergeValue::Null);
    assert_eq!(MergeValue::Boolean(true), MergeValue::Boolean(true));
    assert_eq!(MergeValue::Integer(42), MergeValue::Integer(42));
}

#[test]
fn test_enum_clone() {
    let val = MergeValue::String("test".to_string());
    let cloned = val.clone();
    assert_eq!(val, cloned);
}

// ===== FROM JSON TESTS =====

#[test]
fn test_from_json_valid() {
    let json = r#"{"name": "jin", "count": 42, "active": true, "pi": 3.14}"#;
    let value = MergeValue::from_json(json).expect("JSON parsing should succeed");

    assert!(value.is_object());
    let obj = value.as_object().expect("Should be an object");
    assert_eq!(obj.len(), 4);

    assert_eq!(obj.get("name").and_then(|v| v.as_str()), Some("jin"));
    assert_eq!(obj.get("count").and_then(|v| v.as_i64()), Some(42));
    assert_eq!(obj.get("active").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(obj.get("pi").and_then(|v| v.as_str()), Some("3.14"));
}

#[test]
fn test_from_json_invalid() {
    let json = "invalid json {{{";
    let result = MergeValue::from_json(json);
    assert!(result.is_err(), "Should fail to parse invalid JSON");
}

#[test]
fn test_from_json_nested() {
    let json = r#"{"outer": {"inner": {"value": 42}}}"#;
    let value = MergeValue::from_json(json).expect("JSON parsing should succeed");

    let outer = value.as_object().expect("Should have outer object");
    let inner_obj = outer.get("outer").expect("Should have inner key").as_object().expect("Should be object");
    let inner = inner_obj.get("inner").expect("Should have deepest key").as_object().expect("Should be object");

    assert_eq!(inner.get("value").and_then(|v| v.as_i64()), Some(42));
}

#[test]
fn test_from_json_array() {
    let json = r#"[1, 2, 3, "four", true]"#;
    let value = MergeValue::from_json(json).expect("JSON parsing should succeed");

    let arr = value.as_array().expect("Should be an array");
    assert_eq!(arr.len(), 5);
    assert_eq!(arr[0].as_i64(), Some(1));
    assert_eq!(arr[1].as_i64(), Some(2));
    assert_eq!(arr[2].as_i64(), Some(3));
    assert_eq!(arr[3].as_str(), Some("four"));
    assert_eq!(arr[4].as_bool(), Some(true));
}

#[test]
fn test_from_json_null() {
    let json = r#"{"value": null}"#;
    let value = MergeValue::from_json(json).expect("JSON parsing should succeed");

    let obj = value.as_object().expect("Should be an object");
    assert!(obj.get("value").expect("Should have key").is_null());
}

// ===== FROM YAML TESTS =====

#[test]
fn test_from_yaml_valid() {
    let yaml = "name: jin\ncount: 42\nactive: true\npi: 3.14";
    let value = MergeValue::from_yaml(yaml).expect("YAML parsing should succeed");

    assert!(value.is_object());
    let obj = value.as_object().expect("Should be an object");
    assert_eq!(obj.len(), 4);

    assert_eq!(obj.get("name").and_then(|v| v.as_str()), Some("jin"));
    assert_eq!(obj.get("count").and_then(|v| v.as_i64()), Some(42));
    assert_eq!(obj.get("active").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn test_from_yaml_array() {
    let yaml = "- one\n- two\n- three";
    let value = MergeValue::from_yaml(yaml).expect("YAML parsing should succeed");

    let arr = value.as_array().expect("Should be an array");
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0].as_str(), Some("one"));
    assert_eq!(arr[1].as_str(), Some("two"));
    assert_eq!(arr[2].as_str(), Some("three"));
}

#[test]
fn test_from_yaml_nested() {
    let yaml = "outer:\n  inner:\n    value: 42";
    let value = MergeValue::from_yaml(yaml).expect("YAML parsing should succeed");

    let outer = value.as_object().expect("Should have outer object");
    let inner_obj = outer.get("outer").expect("Should have outer key").as_object().expect("Should be object");
    let inner = inner_obj.get("inner").expect("Should have inner key").as_object().expect("Should be object");

    assert_eq!(inner.get("value").and_then(|v| v.as_i64()), Some(42));
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
    let value = MergeValue::from_toml(toml).expect("TOML parsing should succeed");

    assert!(value.is_object());
    let obj = value.as_object().expect("Should be an object");
    assert_eq!(obj.len(), 4);

    assert_eq!(obj.get("name").and_then(|v| v.as_str()), Some("jin"));
    assert_eq!(obj.get("count").and_then(|v| v.as_i64()), Some(42));
    assert_eq!(obj.get("active").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn test_from_toml_table() {
    let toml = r#"
        [database]
        host = "localhost"
        port = 5432

        [server]
        port = 8080
    "#;
    let value = MergeValue::from_toml(toml).expect("TOML parsing should succeed");

    let obj = value.as_object().expect("Should be an object");
    assert_eq!(obj.len(), 2);

    let db = obj.get("database").expect("Should have database key").as_object().expect("Should be object");
    assert_eq!(db.get("host").and_then(|v| v.as_str()), Some("localhost"));
    assert_eq!(db.get("port").and_then(|v| v.as_i64()), Some(5432));

    let server = obj.get("server").expect("Should have server key").as_object().expect("Should be object");
    assert_eq!(server.get("port").and_then(|v| v.as_i64()), Some(8080));
}

#[test]
fn test_from_toml_datetime() {
    let toml = r#"created = 2025-12-26T12:00:00Z"#;
    let value = MergeValue::from_toml(toml).expect("TOML parsing should succeed");

    let obj = value.as_object().expect("Should be an object");
    // Datetime is converted to String
    assert!(obj.get("created").and_then(|v| v.as_str()).is_some());
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
    let value = MergeValue::from_ini(ini).expect("INI parsing should succeed");

    let obj = value.as_object().expect("Should be an object");
    assert_eq!(obj.len(), 2);

    let db = obj.get("database").expect("Should have database key").as_object().expect("Should be object");
    assert_eq!(db.get("host").and_then(|v| v.as_str()), Some("localhost"));
    assert_eq!(db.get("port").and_then(|v| v.as_str()), Some("5432"));

    let server = obj.get("server").expect("Should have server key").as_object().expect("Should be object");
    assert_eq!(server.get("port").and_then(|v| v.as_str()), Some("8080"));
}

#[test]
fn test_from_ini_empty_sections() {
    let ini = r#"
        [empty]
        [filled]
        key = value
    "#;
    let value = MergeValue::from_ini(ini).expect("INI parsing should succeed");

    let obj = value.as_object().expect("Should be an object");
    assert_eq!(obj.len(), 2);

    let empty = obj.get("empty").expect("Should have empty key").as_object().expect("Should be object");
    assert_eq!(empty.len(), 0);

    let filled = obj.get("filled").expect("Should have filled key").as_object().expect("Should be object");
    assert_eq!(filled.len(), 1);
}

// ===== MERGE TESTS =====

#[test]
fn test_merge_objects_deep() {
    let base = MergeValue::from_json(r#"{"a": {"x": 1}, "b": 2}"#).expect("JSON parsing should succeed");
    let override_val = MergeValue::from_json(r#"{"a": {"y": 2}}"#).expect("JSON parsing should succeed");
    let merged = base.merge(&override_val).expect("Merge should succeed");

    let obj = merged.as_object().expect("Result should be object");

    // Check deep merge occurred
    let a_obj = obj.get("a").expect("Should have 'a' key").as_object().expect("'a' should be object");
    assert_eq!(a_obj.get("x").and_then(|v| v.as_i64()), Some(1), "Original value preserved");
    assert_eq!(a_obj.get("y").and_then(|v| v.as_i64()), Some(2), "New value added");

    // Check unrelated key preserved
    assert_eq!(obj.get("b").and_then(|v| v.as_i64()), Some(2), "Unrelated key preserved");
}

#[test]
fn test_merge_null_deletes_key() {
    let base = MergeValue::from_json(r#"{"a": 1, "b": 2, "c": 3}"#).expect("JSON parsing should succeed");
    let override_val = MergeValue::from_json(r#"{"a": null, "b": 20}"#).expect("JSON parsing should succeed");
    let merged = base.merge(&override_val).expect("Merge should succeed");

    let obj = merged.as_object().expect("Result should be object");

    assert!(!obj.contains_key("a"), "Key 'a' should be deleted by null");
    assert_eq!(obj.get("b").and_then(|v| v.as_i64()), Some(20), "Key 'b' should be updated");
    assert_eq!(obj.get("c").and_then(|v| v.as_i64()), Some(3), "Key 'c' should be preserved");
}

#[test]
fn test_merge_arrays_replace() {
    let base = MergeValue::from_json(r#"[1, 2, 3]"#).expect("JSON parsing should succeed");
    let override_val = MergeValue::from_json(r#"[4, 5, 6]"#).expect("JSON parsing should succeed");
    let merged = base.merge(&override_val).expect("Merge should succeed");

    let arr = merged.as_array().expect("Result should be array");
    assert_eq!(arr.len(), 3, "Array should be replaced, not concatenated");
    assert_eq!(arr[0].as_i64(), Some(4), "First element should be from override");
    assert_eq!(arr[1].as_i64(), Some(5), "Second element should be from override");
    assert_eq!(arr[2].as_i64(), Some(6), "Third element should be from override");
}

#[test]
fn test_merge_primitives_replace() {
    let base = MergeValue::Integer(10);
    let override_val = MergeValue::Integer(20);
    let merged = base.merge(&override_val).expect("Merge should succeed");

    assert_eq!(merged.as_i64(), Some(20), "Override value should win");
}

#[test]
fn test_merge_empty_with_object() {
    let base = MergeValue::Object(indexmap::IndexMap::new());
    let override_val = MergeValue::from_json(r#"{"a": 1}"#).expect("JSON parsing should succeed");
    let merged = base.merge(&override_val).expect("Merge should succeed");

    let obj = merged.as_object().expect("Result should be object");
    assert_eq!(obj.len(), 1, "Should have one key");
    assert_eq!(obj.get("a").and_then(|v| v.as_i64()), Some(1), "Should have the value");
}

#[test]
fn test_merge_null_in_nested_object_deletes() {
    let base = MergeValue::from_json(r#"{"outer": {"inner": "value", "delete_me": "gone"}}"#)
        .expect("JSON parsing should succeed");
    let override_val = MergeValue::from_json(r#"{"outer": {"delete_me": null}}"#)
        .expect("JSON parsing should succeed");
    let merged = base.merge(&override_val).expect("Merge should succeed");

    let outer = merged.as_object().expect("Result should be object")
        .get("outer").expect("Should have outer key").as_object().expect("Outer should be object");

    assert_eq!(outer.get("inner").and_then(|v| v.as_str()), Some("value"), "Sibling key preserved");
    assert!(!outer.contains_key("delete_me"), "Nested key deleted by null");
}

#[test]
fn test_merge_three_levels_deep() {
    let base = MergeValue::from_json(r#"{"a": {"b": {"c": 1, "d": 2}}}"#).expect("JSON parsing should succeed");
    let override_val = MergeValue::from_json(r#"{"a": {"b": {"c": 10, "e": 3}}}"#).expect("JSON parsing should succeed");
    let merged = base.merge(&override_val).expect("Merge should succeed");

    let a = merged.as_object().expect("Result should be object")
        .get("a").expect("Should have 'a' key").as_object().expect("'a' should be object");
    let b = a.get("b").expect("Should have 'b' key").as_object().expect("'b' should be object");

    assert_eq!(b.get("c").and_then(|v| v.as_i64()), Some(10), "Deep value updated");
    assert_eq!(b.get("d").and_then(|v| v.as_i64()), Some(2), "Deep sibling preserved");
    assert_eq!(b.get("e").and_then(|v| v.as_i64()), Some(3), "New deep key added");
}

#[test]
fn test_merge_adds_new_keys() {
    let base = MergeValue::from_json(r#"{"a": 1}"#).expect("JSON parsing should succeed");
    let override_val = MergeValue::from_json(r#"{"b": 2, "c": 3}"#).expect("JSON parsing should succeed");
    let merged = base.merge(&override_val).expect("Merge should succeed");

    let obj = merged.as_object().expect("Result should be object");

    assert_eq!(obj.get("a").and_then(|v| v.as_i64()), Some(1), "Original key preserved");
    assert_eq!(obj.get("b").and_then(|v| v.as_i64()), Some(2), "New key added");
    assert_eq!(obj.get("c").and_then(|v| v.as_i64()), Some(3), "New key added");
}

#[test]
fn test_merge_primitive_types() {
    // String -> Integer
    let base = MergeValue::String("hello");
    let override_val = MergeValue::Integer(42);
    let merged = base.merge(&override_val).expect("Merge should succeed");
    assert_eq!(merged.as_i64(), Some(42));

    // Boolean -> Float
    let base = MergeValue::Boolean(true);
    let override_val = MergeValue::Float(3.14);
    let merged = base.merge(&override_val).expect("Merge should succeed");
    assert_eq!(merged.as_str(), Some("3.14"));
}

// ===== HELPER METHOD TESTS =====

#[test]
fn test_helper_methods() {
    // Null
    let null_val = MergeValue::Null;
    assert!(null_val.is_null(), "Should be null");
    assert!(!null_val.is_object(), "Null should not be object");
    assert!(!null_val.is_array(), "Null should not be array");

    // String
    let str_val = MergeValue::String("hello".to_string());
    assert!(!str_val.is_null(), "String should not be null");
    assert!(!str_val.is_object(), "String should not be object");
    assert!(!str_val.is_array(), "String should not be array");
    assert_eq!(str_val.as_str(), Some("hello"), "Should get string value");
    assert_eq!(str_val.as_i64(), None, "Should not get i64 from string");
    assert_eq!(str_val.as_bool(), None, "Should not get bool from string");

    // Integer
    let int_val = MergeValue::Integer(42);
    assert_eq!(int_val.as_i64(), Some(42), "Should get integer value");
    assert_eq!(int_val.as_str(), None, "Should not get string from integer");

    // Boolean
    let bool_val = MergeValue::Boolean(true);
    assert_eq!(bool_val.as_bool(), Some(true), "Should get boolean value");

    // Array
    let arr_val = MergeValue::Array(vec![MergeValue::Integer(1), MergeValue::Integer(2)]);
    assert!(arr_val.is_array(), "Should be array");
    assert!(!arr_val.is_object(), "Array should not be object");
    assert_eq!(arr_val.as_array().map(|a| a.len()), Some(2), "Should get array length");

    // Object
    let mut obj_map = indexmap::IndexMap::new();
    obj_map.insert("key".to_string(), MergeValue::String("value".to_string()));
    let obj_val = MergeValue::Object(obj_map);
    assert!(obj_val.is_object(), "Should be object");
    assert!(!obj_val.is_array(), "Object should not be array");
    assert_eq!(obj_val.as_object().map(|o| o.len()), Some(1), "Should get object size");
}

// ===== FROM CONVERSION TESTS =====

#[test]
fn test_from_conversions() {
    // bool
    let b: MergeValue = true.into();
    assert_eq!(b.as_bool(), Some(true));

    // i64
    let i: MergeValue = 42i64.into();
    assert_eq!(i.as_i64(), Some(42));

    // f64
    let f: MergeValue = 3.14f64.into();
    assert_eq!(f.as_str(), Some("3.14"));

    // String
    let s: MergeValue = String::from("hello").into();
    assert_eq!(s.as_str(), Some("hello"));

    // &str
    let ref_str: MergeValue = "world".into();
    assert_eq!(ref_str.as_str(), Some("world"));
}

// ===== DEFAULT TEST =====

#[test]
fn test_default() {
    let val = MergeValue::default();
    assert!(val.is_null(), "Default should be Null");
}

// ===== ORDER PRESERVATION TEST =====

#[test]
fn test_order_preservation() {
    use indexmap::IndexMap;

    let mut map = IndexMap::new();
    map.insert("z".to_string(), MergeValue::Integer(1));
    map.insert("a".to_string(), MergeValue::Integer(2));
    map.insert("m".to_string(), MergeValue::Integer(3));

    let obj = MergeValue::Object(map);
    let obj_ref = obj.as_object().unwrap();

    let keys: Vec<&str> = obj_ref.keys().map(|k| k.as_str()).collect();
    assert_eq!(keys, vec!["z", "a", "m"], "Order should be preserved");
}
