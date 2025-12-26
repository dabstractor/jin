# configparser Research Notes

## Documentation Links

- **crates.io**: https://crates.io/crates/configparser
- **GitHub repository**: https://github.com/mahkoh/configparser-rs
- **docs.rs**: https://docs.rs/configparser/

## API Overview

```rust
use configparser::ini::Ini;

let mut config = Ini::new();
config.load("config.ini")?;
// or
config.read(ini_content)?;
```

## Key Methods

- `Ini::new()` - Create new INI parser
- `load(&mut self, filename: &str)` - Load from file
- `read(&mut self, data: &str)` - Read from string
- `get(&self, section: &str, key: &str) -> Option<String>` - Get value
- `set(&mut self, section: &str, key: &str, value: Option<&str>)` - Set value
- `has_section(&self, section: &str) -> bool` - Check section exists
- `sections(&self) -> &HashMap<String, HashMap<String, String>>` - Get all sections

## Critical Implementation Pattern

```rust
use configparser::ini::Ini;

fn parse_ini_to_merge_value(input: &str) -> Result<MergeValue, String> {
    let mut config = Ini::new();
    config.read(input).map_err(|e| e.to_string())?;

    let mut result = IndexMap::new();

    // Get all sections
    let sections_map = config.sections();

    for (section_name, keys) in sections_map {
        let mut section_map = IndexMap::new();
        for (key, value) in keys {
            // INI values are always strings
            section_map.insert(
                key.clone(),
                MergeValue::String(value.clone())
            );
        }
        result.insert(section_name.clone(), MergeValue::Object(section_map));
    }

    Ok(MergeValue::Object(result))
}
```

## INI Structure to MergeValue Mapping

```ini
[section1]
key1 = value1
key2 = value2

[section2]
key1 = value3
```

Converts to:

```json
{
    "section1": {
        "key1": "value1",
        "key2": "value2"
    },
    "section2": {
        "key1": "value3"
    }
}
```

## Common Gotchas

1. **All values are strings** - Must parse numeric/boolean types manually if needed
2. **Keys before any section** - configparser handles these differently (global section)
3. **Multi-line values** - Supported but with whitespace considerations
4. **Comments** - Both `;` and `#` are supported
5. **No native type support** - Every value is a string

## Global Keys (Before Any Section)

```ini
global_key = value1

[section]
key = value2
```

Handle with special "global" key or top-level keys:

```rust
// Check for keys without sections
// May need to handle as a special "global" section or top-level keys
```

## Error Handling

```rust
use configparser::ini::Ini;

let mut config = Ini::new();
match config.read(ini_content) {
    Ok(_) => println!("INI parsed successfully"),
    Err(e) => eprintln!("Failed to parse INI: {}", e),
}
```

## Iteration Patterns

```rust
// Iterate all sections and keys
for (section_name, keys) in config.sections().iter() {
    println!("[{}]", section_name);
    for (key, value) in keys.iter() {
        println!("{} = {}", key, value);
    }
}
```
