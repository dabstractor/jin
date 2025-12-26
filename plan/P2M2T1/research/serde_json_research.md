# serde_json Research Notes

## Documentation Links

- **Main documentation**: https://docs.rs/serde_json/
- **GitHub repository**: https://github.com/serde-rs/json
- **Serde book**: https://serde.rs/json.html
- **Value enum**: https://docs.rs/serde_json/latest/serde_json/enum.Value.html

## Key API Methods

### Value Type Methods
- `Value::is_null` - https://docs.rs/serde_json/latest/serde_json/enum.Value.html#method.is_null
- `Value::is_number` - https://docs.rs/serde_json/latest/serde_json/enum.Value.html#method.is_number
- `Value::is_string` - https://docs.rs/serde_json/latest/serde_json/enum.Value.html#method.is_string
- `Value::is_array` - https://docs.rs/serde_json/latest/serde_json/enum.Value.html#method.is_array
- `Value::is_object` - https://docs.rs/serde_json/latest/serde_json/enum.Value.html#method.is_object

### Number Type Methods
- `Number::is_i64` - https://docs.rs/serde_json/latest/serde_json/struct.Number.html#method.is_i64
- `Number::is_u64` - https://docs.rs/serde_json/latest/serde_json/struct.Number.html#method.is_u64
- `Number::is_f64` - https://docs.rs/serde_json/latest/serde_json/struct.Number.html#method.is_f64
- `Number::as_i64` - https://docs.rs/serde_json/latest/serde_json/struct.Number.html#method.as_i64
- `Number::as_f64` - https://docs.rs/serde_json/latest/serde_json/struct.Number.html#method.as_f64

## Critical Implementation Pattern

```rust
use serde_json::Value;

fn convert_json_value(value: &Value) -> MergeValue {
    match value {
        Value::Null => MergeValue::Null,
        Value::Bool(b) => MergeValue::Boolean(*b),
        Value::Number(n) => {
            if n.is_i64() {
                MergeValue::Integer(n.as_i64().unwrap())
            } else if n.is_f64() {
                MergeValue::Float(n.as_f64().unwrap())
            } else {
                // Handle u64 and other cases
                MergeValue::Integer(n.as_i64().unwrap_or(0))
            }
        },
        Value::String(s) => MergeValue::String(s.clone()),
        Value::Array(arr) => {
            MergeValue::Array(arr.iter().map(convert_json_value).collect())
        },
        Value::Object(obj) => {
            let mut map = IndexMap::new();
            for (k, v) in obj {
                map.insert(k.clone(), convert_json_value(v));
            }
            MergeValue::Object(map)
        }
    }
}
```

## Common Gotchas

1. **JSON has only "number" type** - No distinction between int/float, must check explicitly
2. **Large integers** - u64 values larger than i64::MAX need special handling
3. **Precision loss** - Float values may lose precision when converting

## Error Handling

```rust
use serde_json::{Error, Value};

fn parse_json(input: &str) -> Result<Value, Error> {
    serde_json::from_str(input)
}
```

## Related Crates

- `serde_path_to_error` - Better error reporting with path information
