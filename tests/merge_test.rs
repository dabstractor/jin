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
    // Float is stored as Float variant, not String - as_str() returns None for Float
    match obj.get("pi") {
        Some(MergeValue::Float(f)) => assert!((f - 3.14).abs() < 0.001),
        _ => panic!("Expected Float for pi value"),
    }
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
    let base = MergeValue::String("hello".to_string());
    let override_val = MergeValue::Integer(42);
    let merged = base.merge(&override_val).expect("Merge should succeed");
    assert_eq!(merged.as_i64(), Some(42));

    // Boolean -> Float
    let base = MergeValue::Boolean(true);
    let override_val = MergeValue::Float(3.15);
    let merged = base.merge(&override_val).expect("Merge should succeed");
    // Float is Float variant, not String
    match merged {
        MergeValue::Float(f) => assert!((f - 3.15).abs() < 0.001),
        _ => panic!("Expected Float variant"),
    }
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

    // f64 - Float is stored as Float variant, not String
    let f: MergeValue = 3.14f64.into();
    // Floats don't return as_str(), they're a separate variant
    assert!(f.as_str().is_none());
    match f {
        MergeValue::Float(val) => assert!((val - 3.14).abs() < 0.001),
        _ => panic!("Expected Float variant"),
    }

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

// ===== ARRAY MERGE STRATEGY TESTS =====

use jin_glm::merge::value::{MergeConfig, ArrayMergeStrategy};

#[test]
fn test_merge_array_replace_strategy_default() {
    // Default merge() uses Replace strategy (RFC 7396)
    let base = MergeValue::from_json(r#"[1, 2, 3]"#).expect("JSON parsing should succeed");
    let patch = MergeValue::from_json(r#"[4, 5, 6]"#).expect("JSON parsing should succeed");
    let merged = base.merge(&patch).expect("Merge should succeed");

    let arr = merged.as_array().expect("Result should be array");
    assert_eq!(arr.len(), 3, "Array should be replaced");
    assert_eq!(arr[0].as_i64(), Some(4), "First element from patch");
    assert_eq!(arr[1].as_i64(), Some(5), "Second element from patch");
    assert_eq!(arr[2].as_i64(), Some(6), "Third element from patch");
}

#[test]
fn test_merge_array_by_key_with_id() {
    let base = MergeValue::from_json(r#"
        [
            {"id": "server-a", "port": 8080},
            {"id": "server-b", "port": 8081}
        ]
    "#).expect("JSON parsing should succeed");

    let patch = MergeValue::from_json(r#"
        [
            {"id": "server-a", "port": 9090},
            {"id": "server-c", "port": 8082}
        ]
    "#).expect("JSON parsing should succeed");

    let config = MergeConfig {
        array_strategy: ArrayMergeStrategy::MergeByKey,
        ..Default::default()
    };

    let merged = base.merge_with_config(&patch, &config).expect("Merge should succeed");
    let arr = merged.as_array().expect("Result should be array");

    assert_eq!(arr.len(), 3, "Should have 3 elements");

    // server-a should have updated port
    let server_a = &arr[0];
    assert_eq!(server_a.as_object().unwrap().get("id").unwrap().as_str(), Some("server-a"));
    assert_eq!(server_a.as_object().unwrap().get("port").unwrap().as_i64(), Some(9090));

    // server-b should be unchanged
    let server_b = &arr[1];
    assert_eq!(server_b.as_object().unwrap().get("id").unwrap().as_str(), Some("server-b"));
    assert_eq!(server_b.as_object().unwrap().get("port").unwrap().as_i64(), Some(8081));

    // server-c should be added
    let server_c = &arr[2];
    assert_eq!(server_c.as_object().unwrap().get("id").unwrap().as_str(), Some("server-c"));
    assert_eq!(server_c.as_object().unwrap().get("port").unwrap().as_i64(), Some(8082));
}

#[test]
fn test_merge_array_by_key_with_name() {
    let base = MergeValue::from_json(r#"
        [
            {"name": "database", "host": "localhost"},
            {"name": "cache", "host": "redis"}
        ]
    "#).expect("JSON parsing should succeed");

    let patch = MergeValue::from_json(r#"
        [
            {"name": "database", "port": 5432},
            {"name": "queue", "host": "rabbitmq"}
        ]
    "#).expect("JSON parsing should succeed");

    let config = MergeConfig {
        array_strategy: ArrayMergeStrategy::MergeByKey,
        ..Default::default()
    };

    let merged = base.merge_with_config(&patch, &config).expect("Merge should succeed");
    let arr = merged.as_array().expect("Result should be array");

    assert_eq!(arr.len(), 3, "Should have 3 elements");

    // Elements from base maintain their relative order
    // database (from base position 0, merged)
    let database = &arr[0];
    assert_eq!(database.as_object().unwrap().get("name").unwrap().as_str(), Some("database"));
    assert_eq!(database.as_object().unwrap().get("host").unwrap().as_str(), Some("localhost"));
    assert_eq!(database.as_object().unwrap().get("port").unwrap().as_i64(), Some(5432));

    // cache (from base position 1, unchanged)
    let cache = &arr[1];
    assert_eq!(cache.as_object().unwrap().get("name").unwrap().as_str(), Some("cache"));

    // queue (new from patch, added after base elements, sorted among new keys)
    let queue = &arr[2];
    assert_eq!(queue.as_object().unwrap().get("name").unwrap().as_str(), Some("queue"));
}

#[test]
fn test_merge_array_by_key_with_nested_object_merging() {
    let base = MergeValue::from_json(r#"
        [
            {"id": "svc1", "config": {"timeout": 100}}
        ]
    "#).expect("JSON parsing should succeed");

    let patch = MergeValue::from_json(r#"
        [
            {"id": "svc1", "config": {"retries": 3}}
        ]
    "#).expect("JSON parsing should succeed");

    let config = MergeConfig {
        array_strategy: ArrayMergeStrategy::MergeByKey,
        ..Default::default()
    };

    let merged = base.merge_with_config(&patch, &config).expect("Merge should succeed");
    let arr = merged.as_array().expect("Result should be array");

    assert_eq!(arr.len(), 1, "Should have 1 element");

    let svc = &arr[0];
    let config_obj = svc.as_object().unwrap().get("config").unwrap().as_object().unwrap();
    assert_eq!(config_obj.get("timeout").unwrap().as_i64(), Some(100));
    assert_eq!(config_obj.get("retries").unwrap().as_i64(), Some(3));
}

#[test]
fn test_merge_array_concatenate_strategy() {
    let base = MergeValue::from_json(r#"[1, 2, 3]"#).expect("JSON parsing should succeed");
    let patch = MergeValue::from_json(r#"[4, 5, 6]"#).expect("JSON parsing should succeed");

    let config = MergeConfig {
        array_strategy: ArrayMergeStrategy::Concatenate,
        ..Default::default()
    };

    let merged = base.merge_with_config(&patch, &config).expect("Merge should succeed");
    let arr = merged.as_array().expect("Result should be array");

    assert_eq!(arr.len(), 6, "Should have 6 elements");
    assert_eq!(arr[0].as_i64(), Some(1));
    assert_eq!(arr[1].as_i64(), Some(2));
    assert_eq!(arr[2].as_i64(), Some(3));
    assert_eq!(arr[3].as_i64(), Some(4));
    assert_eq!(arr[4].as_i64(), Some(5));
    assert_eq!(arr[5].as_i64(), Some(6));
}

#[test]
fn test_merge_array_by_key_with_non_object_elements() {
    let base = MergeValue::from_json(r#"
        [
            "string-element",
            {"id": "obj1", "value": 1},
            42
        ]
    "#).expect("JSON parsing should succeed");

    let patch = MergeValue::from_json(r#"
        [
            {"id": "obj1", "value": 2},
            "another-string"
        ]
    "#).expect("JSON parsing should succeed");

    let config = MergeConfig {
        array_strategy: ArrayMergeStrategy::MergeByKey,
        ..Default::default()
    };

    let merged = base.merge_with_config(&patch, &config).expect("Merge should succeed");
    let arr = merged.as_array().expect("Result should be array");

    assert_eq!(arr.len(), 4, "Should have 4 elements");

    // Non-object from base first
    assert_eq!(arr[0].as_str(), Some("string-element"));

    // Merged object
    assert_eq!(arr[1].as_object().unwrap().get("id").unwrap().as_str(), Some("obj1"));
    assert_eq!(arr[1].as_object().unwrap().get("value").unwrap().as_i64(), Some(2));

    // Non-object from base
    assert_eq!(arr[2].as_i64(), Some(42));

    // Non-object from patch
    assert_eq!(arr[3].as_str(), Some("another-string"));
}

#[test]
fn test_merge_array_empty_arrays() {
    let base = MergeValue::from_json(r#"[]"#).expect("JSON parsing should succeed");
    let patch = MergeValue::from_json(r#"[{"id": "a", "x": 1}]"#).expect("JSON parsing should succeed");

    let config = MergeConfig {
        array_strategy: ArrayMergeStrategy::MergeByKey,
        ..Default::default()
    };

    let merged = base.merge_with_config(&patch, &config).expect("Merge should succeed");
    let arr = merged.as_array().expect("Result should be array");

    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0].as_object().unwrap().get("id").unwrap().as_str(), Some("a"));
}

#[test]
fn test_merge_array_max_depth_limit() {
    let base = MergeValue::from_json(r#"{"a": {"b": {"c": 1}}}"#).expect("JSON parsing should succeed");
    let patch = MergeValue::from_json(r#"{"a": {"b": {"c": 2}}}"#).expect("JSON parsing should succeed");

    let config = MergeConfig {
        array_strategy: ArrayMergeStrategy::Replace,
        max_depth: 4,
    };

    // Should succeed - depth is just enough
    let _merged = base.merge_with_config(&patch, &config).expect("Merge should succeed");

    // Now with insufficient depth
    let config = MergeConfig {
        array_strategy: ArrayMergeStrategy::Replace,
        max_depth: 3,
    };

    let result = base.merge_with_config(&patch, &config);
    assert!(result.is_err(), "Should fail with insufficient depth");
    assert!(result.unwrap_err().to_string().contains("Maximum merge depth exceeded"));
}

#[test]
fn test_merge_array_by_key_objects_without_key_field() {
    let base = MergeValue::from_json(r#"
        [
            {"id": "obj1", "value": 1},
            {"nokey": "should-be-preserved"}
        ]
    "#).expect("JSON parsing should succeed");

    let patch = MergeValue::from_json(r#"
        [
            {"id": "obj1", "value": 2},
            {"another": "nokey-object"}
        ]
    "#).expect("JSON parsing should succeed");

    let config = MergeConfig {
        array_strategy: ArrayMergeStrategy::MergeByKey,
        ..Default::default()
    };

    let merged = base.merge_with_config(&patch, &config).expect("Merge should succeed");
    let arr = merged.as_array().expect("Result should be array");

    assert_eq!(arr.len(), 3);

    // Merged object
    assert_eq!(arr[0].as_object().unwrap().get("id").unwrap().as_str(), Some("obj1"));

    // Non-keyed object from base
    assert_eq!(arr[1].as_object().unwrap().get("nokey").unwrap().as_str(), Some("should-be-preserved"));

    // Non-keyed object from patch
    assert_eq!(arr[2].as_object().unwrap().get("another").unwrap().as_str(), Some("nokey-object"));
}

#[test]
fn test_merge_backward_compatibility() {
    // All existing tests must still pass
    let base = MergeValue::from_json(r#"{"a": {"x": 1}, "b": 2}"#).expect("JSON parsing should succeed");
    let patch = MergeValue::from_json(r#"{"a": {"y": 2}}"#).expect("JSON parsing should succeed");

    let merged = base.merge(&patch).expect("Merge should succeed");

    let obj = merged.as_object().expect("Result should be object");
    let a_obj = obj.get("a").expect("Should have 'a' key").as_object().expect("'a' should be object");

    assert_eq!(a_obj.get("x").and_then(|v| v.as_i64()), Some(1), "Original value preserved");
    assert_eq!(a_obj.get("y").and_then(|v| v.as_i64()), Some(2), "New value added");
    assert_eq!(obj.get("b").and_then(|v| v.as_i64()), Some(2), "Unrelated key preserved");
}

#[test]
fn test_merge_rfc7396_null_deletion() {
    // Example from RFC 7396 Section 3
    let target = MergeValue::from_json(r#"
        {"title": "Goodbye!", "author": {"given": "John", "family": "Doe"}, "tags": ["example"]}
    "#).expect("JSON parsing should succeed");

    let patch = MergeValue::from_json(r#"
        {"title": "Hello!", "author": null, "tags": ["sample"]}
    "#).expect("JSON parsing should succeed");

    let merged = target.merge(&patch).expect("Merge should succeed");

    let obj = merged.as_object().expect("Result should be object");

    // title should be updated
    assert_eq!(obj.get("title").and_then(|v| v.as_str()), Some("Hello!"));

    // author key should be deleted (null in patch)
    assert!(!obj.contains_key("author"), "author key should be deleted");

    // tags array should be replaced (RFC 7396)
    let tags = obj.get("tags").and_then(|v| v.as_array()).expect("Should have tags");
    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].as_str(), Some("sample"));
}

#[test]
fn test_merge_array_by_key_with_null_values() {
    let base = MergeValue::from_json(r#"
        [
            {"id": "a", "x": 1},
            {"id": "b", "y": 2}
        ]
    "#).expect("JSON parsing should succeed");

    let patch = MergeValue::from_json(r#"
        [
            {"id": "a", "x": null}
        ]
    "#).expect("JSON parsing should succeed");

    let config = MergeConfig {
        array_strategy: ArrayMergeStrategy::MergeByKey,
        ..Default::default()
    };

    let merged = base.merge_with_config(&patch, &config).expect("Merge should succeed");
    let arr = merged.as_array().expect("Result should be array");

    // The null value in the object should not delete the entire object
    // Only the key with null value should be affected
    assert_eq!(arr.len(), 2);

    // Object 'a' should still exist, but 'x' might be null depending on merge behavior
    let obj_a = &arr[0];
    assert_eq!(obj_a.as_object().unwrap().get("id").unwrap().as_str(), Some("a"));

    // Object 'b' should be preserved
    let obj_b = &arr[1];
    assert_eq!(obj_b.as_object().unwrap().get("id").unwrap().as_str(), Some("b"));
}

#[test]
fn test_merge_config_default() {
    let config = MergeConfig::default();
    assert_eq!(config.array_strategy, ArrayMergeStrategy::Replace);
    assert_eq!(config.max_depth, 100);
}

#[test]
fn test_merge_array_strategy_equality() {
    assert_eq!(ArrayMergeStrategy::Replace, ArrayMergeStrategy::Replace);
    assert_eq!(ArrayMergeStrategy::MergeByKey, ArrayMergeStrategy::MergeByKey);
    assert_eq!(ArrayMergeStrategy::Concatenate, ArrayMergeStrategy::Concatenate);

    assert_ne!(ArrayMergeStrategy::Replace, ArrayMergeStrategy::MergeByKey);
    assert_ne!(ArrayMergeStrategy::MergeByKey, ArrayMergeStrategy::Concatenate);
}
