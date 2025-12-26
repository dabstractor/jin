# toml-rs Research Notes

## Documentation Links

- **Main documentation**: https://docs.rs/toml/latest/toml/
- **GitHub repository**: https://github.com/toml-rs/toml
- **Value enum**: https://docs.rs/toml/latest/toml/enum.Value.html
- **Error type**: https://docs.rs/toml/latest/toml/de/struct.Error.html

## Value Enum Variants

```rust
enum Value {
    Integer(i64),       // Distinct from Float
    Float(f64),         // Distinct from Integer
    Boolean(bool),
    String(String),
    Array(Vec<Value>),
    Table(BTreeMap<String, Value>),
    Datetime(toml::value::Datetime),  // TOML-specific type
}
```

## Critical Implementation Pattern

```rust
use toml::Value;

fn convert_toml_value(value: &Value) -> MergeValue {
    match value {
        Value::Integer(i) => MergeValue::Integer(*i),
        Value::Float(f) => MergeValue::Float(*f),
        Value::Boolean(b) => MergeValue::Boolean(*b),
        Value::String(s) => MergeValue::String(s.clone()),
        Value::Datetime(dt) => MergeValue::String(dt.to_string()),
        Value::Array(arr) => {
            MergeValue::Array(arr.iter().map(convert_toml_value).collect())
        },
        Value::Table(table) => {
            let mut index_map = IndexMap::new();
            for (k, v) in table {
                index_map.insert(k.clone(), convert_toml_value(v));
            }
            MergeValue::Object(index_map)
        }
    }
}
```

## Common Gotchas

1. **Distinct numeric types**: TOML has separate Integer and Float types
2. **Datetime handling**: Native datetime type must be converted to string
3. **Inline tables vs standard tables**: Both represented as Value::Table
4. **Array of tables**: Requires special handling in TOML syntax
5. **Dotted keys**: `table.key` syntax creates nested tables

## Datetime Handling

```rust
// TOML datetime is a distinct type
Value::Datetime(dt) => MergeValue::String(dt.to_string())
```

## Array of Tables

```toml
[[items]]
name = "item1"
value = 1

[[items]]
name = "item2"
value = 2
```

This parses to a top-level `items` key with an Array value containing Table elements.

## Error Handling

```rust
use toml::de::Error;

fn parse_toml(input: &str) -> Result<Value, Error> {
    input.parse()
}
```

## Key API Methods

- `Value::as_integer` - https://docs.rs/toml/latest/toml/enum.Value.html#method.as_integer
- `Value::as_float` - https://docs.rs/toml/latest/toml/enum.Value.html#method.as_float
- `Value::as_str` - https://docs.rs/toml/latest/toml/enum.Value.html#method.as_str
- `Value::as_bool` - https://docs.rs/toml/latest/toml/enum.Value.html#method.as_bool
- `Value::as_datetime` - https://docs.rs/toml/latest/toml/enum.Value.html#method.as_datetime
