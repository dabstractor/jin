# serde_yaml_ng Research Notes

## Documentation Links

- **Main documentation**: https://docs.rs/serde_yaml_ng/latest/serde_yaml_ng/
- **crates.io**: https://crates.io/crates/serde_yaml_ng
- **Value enum**: https://docs.rs/serde_yaml_ng/latest/serde_yaml_ng/enum.Value.html
- **Error type**: https://docs.rs/serde_yaml_ng/latest/serde_yaml_ng/struct.Error.html

## Key Differences from serde_yaml

- Better support for YAML-specific features
- Improved error messages with position information
- More efficient parsing for large documents
- Better handling of anchors and aliases

## Value Enum Variants

```rust
enum Value {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Sequence(Vec<Value>),      // Not "Array" like JSON
    Mapping(BTreeMap<String, Value>),  // Not "Object" like JSON
}
```

## Critical Implementation Pattern

```rust
use serde_yaml_ng::Value;

fn convert_yaml_value(value: &Value) -> MergeValue {
    match value {
        Value::Null => MergeValue::Null,
        Value::Bool(b) => MergeValue::Boolean(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                MergeValue::Integer(i)
            } else if let Some(f) = n.as_f64() {
                MergeValue::Float(f)
            } else {
                MergeValue::Null
            }
        },
        Value::String(s) => MergeValue::String(s.clone()),
        Value::Sequence(seq) => {
            MergeValue::Array(seq.iter().map(convert_yaml_value).collect())
        },
        Value::Mapping(map) => {
            let mut index_map = IndexMap::new();
            for (k, v) in map {
                if let Some(key_str) = k.as_str() {
                    index_map.insert(key_str.to_string(), convert_yaml_value(v));
                }
            }
            MergeValue::Object(index_map)
        }
    }
}
```

## Common Gotchas

1. **YAML null has multiple representations**: `null`, `~`, `""`
2. **Timestamps parsed as strings**: YAML dates are not automatically parsed
3. **Multi-document YAML**: Requires manual splitting by `---` delimiter
4. **Merge operator `<<`**: Not automatically handled, needs custom logic
5. **Key types**: Mapping keys can be any type, not just strings

## Multi-Document Handling

```rust
fn parse_multi_document(content: &str) -> Vec<Value> {
    content
        .split("---\n")
        .filter(|doc| !doc.trim().is_empty())
        .filter_map(|doc| serde_yaml_ng::from_str(doc).ok())
        .collect()
}
```

## Anchor/Alias Resolution

Anchors and aliases are automatically resolved during parsing by serde_yaml_ng - no special handling needed.

```yaml
base: &base
  host: localhost
  port: 5432

database:
  <<: *base  # This merge operator is NOT auto-handled
  name: mydb
```

## Error Handling

```rust
use serde_yaml_ng::{Error, Value};

fn parse_yaml(input: &str) -> Result<Value, Error> {
    serde_yaml_ng::from_str(input)
}
```
