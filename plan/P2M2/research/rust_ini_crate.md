# rust-ini Crate Research

Research Date: 2025-12-27
Source: https://docs.rs/rust-ini/0.21/ini/
Crate Version: 0.21.3
License: MIT

## Overview

The `rust-ini` crate provides INI file parsing and writing functionality for Rust. It allows reading INI configuration files, modifying them, and writing them back.

## Cargo.toml Entry

```toml
[dependencies]
rust-ini = "0.21"
```

## Core Import

```rust
use ini::Ini;
```

---

## 1. Parsing API

### Load from File
```rust
use ini::Ini;

let conf = Ini::load_from_file("config.ini")?;
```

### Load from String
```rust
use ini::Ini;

let ini_content = r#"
[database]
host = localhost
port = 5432

[logging]
level = info
"#;

let conf = Ini::load_from_str(ini_content)?;
```

### Load from Reader
```rust
use ini::Ini;
use std::io::BufReader;
use std::fs::File;

let file = File::open("config.ini")?;
let reader = BufReader::new(file);
let conf = Ini::read_from(&mut reader)?;
```

---

## 2. Creating and Writing

### Create New INI
```rust
use ini::Ini;

let mut conf = Ini::new();

// Add sections and values
conf.with_section(Some("database"))
    .set("host", "localhost")
    .set("port", "5432");

conf.with_section(Some("logging"))
    .set("level", "info")
    .set("file", "/var/log/app.log");
```

### Write to File
```rust
conf.write_to_file("config.ini")?;
```

### Write to String
```rust
let mut output = String::new();
conf.write_to(&mut output)?;
println!("{}", output);
```

---

## 3. Accessing Values

### Get Single Value
```rust
// Get value from section
let host = conf.section(Some("database"))
    .and_then(|s| s.get("host"));

// Returns Option<&str>
if let Some(host_value) = host {
    println!("Host: {}", host_value);
}
```

### Index Access (Panics on Missing)
```rust
// Direct indexing - panics if missing
let host = &conf["database"]["host"];
```

### Get with Default
```rust
let port = conf.section(Some("database"))
    .and_then(|s| s.get("port"))
    .unwrap_or("5432");
```

### Get General Section (No Header)
```rust
// Values without [section] header
let general_value = conf.general_section().get("key");
```

---

## 4. Iteration Patterns

### Iterate Over All Sections and Keys
```rust
for (section_name, properties) in &conf {
    println!("[{:?}]", section_name);
    for (key, value) in properties.iter() {
        println!("  {} = {}", key, value);
    }
}
```

### Iterate Over Specific Section
```rust
if let Some(section) = conf.section(Some("database")) {
    for (key, value) in section.iter() {
        println!("{} = {}", key, value);
    }
}
```

### Get Section Names
```rust
for section_name in conf.sections() {
    println!("Section: {:?}", section_name);
}
```

---

## 5. Error Handling

### Error Types
```rust
use ini::{Ini, Error};

match Ini::load_from_file("config.ini") {
    Ok(conf) => {
        // Use configuration
    }
    Err(Error::Io(io_err)) => {
        eprintln!("IO error: {}", io_err);
    }
    Err(Error::Parse(parse_err)) => {
        eprintln!("Parse error at line {}, col {}: {}",
            parse_err.line,
            parse_err.col,
            parse_err.msg);
    }
}
```

### Parse Error Details
```rust
pub struct ParseError {
    pub line: usize,
    pub col: usize,
    pub msg: String,
}
```

---

## 6. Data Structure Mapping

### INI to MergeValue Mapping

INI File:
```ini
[database]
host = localhost
port = 5432

[features]
caching = true
```

Maps to nested structure:
```rust
MergeValue::Object({
    "database": MergeValue::Object({
        "host": MergeValue::String("localhost"),
        "port": MergeValue::String("5432"),  // INI values are always strings
    }),
    "features": MergeValue::Object({
        "caching": MergeValue::String("true"),  // Boolean as string
    }),
})
```

### General Section Handling
```ini
; Values before any [section]
global_key = global_value

[section]
key = value
```

Maps to:
```rust
MergeValue::Object({
    // General section keys at root level OR in special "__general__" key
    "global_key": MergeValue::String("global_value"),
    "section": MergeValue::Object({
        "key": MergeValue::String("value"),
    }),
})
```

---

## 7. Configuration Options

### ParseOption
```rust
use ini::{Ini, ParseOption};

let option = ParseOption {
    enabled_quote: true,              // Support "quoted values"
    enabled_escape: true,             // Support \n, \t, \xFF escapes
    enabled_indented_multiline_value: true,  // Multiline with indentation
    enabled_preserve_key_leading_whitespace: false,
};

let conf = Ini::load_from_str_opt(content, option)?;
```

### WriteOption
```rust
use ini::{Ini, WriteOption, EscapePolicy, LineSeparator};

let option = WriteOption {
    escape_policy: EscapePolicy::BasicsUnicode,
    line_separator: LineSeparator::LF,
    kv_separator: "=".to_string(),
};

conf.write_to_file_opt("config.ini", option)?;
```

---

## 8. Important Quirks and Limitations

### All Values Are Strings
```rust
// INI does not have type information
// port = 5432 is stored as "5432" (string)
// enabled = true is stored as "true" (string)

let port_str = conf["database"]["port"];
let port: u16 = port_str.parse().unwrap_or(5432);
```

### Case Sensitivity
- Section names are case-sensitive by default
- Key names are case-sensitive by default

### Comments Are Not Preserved
- Comments (`;` or `#`) are stripped during parsing
- Comments are not written back when saving

### No Duplicate Key Support
- If the same key appears twice, last value wins
- Use `section_all()` for duplicate sections

### Unicode Support
- Full UTF-8 support for section names, keys, and values

### Ordering
- Sections and keys maintain insertion order

---

## 9. Complete Conversion Example

```rust
use ini::Ini;
use indexmap::IndexMap;

// Assume MergeValue is defined elsewhere
enum MergeValue {
    Null,
    String(String),
    Object(IndexMap<String, MergeValue>),
    // ... other variants
}

fn ini_to_merge_value(ini: &Ini) -> MergeValue {
    let mut root = IndexMap::new();

    // Handle general section (no header)
    let general = ini.general_section();
    for (key, value) in general.iter() {
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

fn merge_value_to_ini(value: &MergeValue) -> Option<Ini> {
    if let MergeValue::Object(obj) = value {
        let mut ini = Ini::new();

        for (section_name, section_value) in obj {
            if let MergeValue::Object(section_obj) = section_value {
                for (key, val) in section_obj {
                    if let MergeValue::String(s) = val {
                        ini.with_section(Some(section_name.as_str()))
                            .set(key, s);
                    }
                    // Note: Non-string values would need conversion
                }
            }
        }

        Some(ini)
    } else {
        None  // INI requires root to be an object
    }
}
```

---

## References

- GitHub Repository: https://github.com/zonyitoo/rust-ini
- Documentation: https://docs.rs/rust-ini/0.21/ini/
- Crates.io: https://crates.io/crates/rust-ini
