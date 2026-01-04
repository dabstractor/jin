# Research: External Config Command Patterns

## Best Practices for Config Commands in Rust CLIs

## 1. Standard Precedence Model

**Highest to Lowest Priority**:
1. CLI arguments
2. Environment variables
3. Configuration files

### Reference
- [Rain's Rust CLI recommendations](https://rust-cli-recommendations.sunshowers.io/hierarchical-config.html)

## 2. clap v4 Nested Subcommands

```rust
#[derive(Subcommand)]
enum Commands {
    /// Get configuration values
    Get { key: String },
    /// Set configuration values
    Set { key: String, value: String },
    /// List all configuration
    List,
}
```

### Reference
- [clap v4 Documentation](https://docs.rs/clap/latest/)

## 3. Config File Format: TOML

TOML is the recommended format in the Rust ecosystem.

### Why TOML?
- Human-readable and writable
- Standard in Rust (Cargo uses it)
- Good support via `toml` crate

### Reference
- [TOML crate](https://crates.io/crates/toml)

## 4. Common Config Command Operations

### Get Operation
```rust
fn handle_get(key: &str) -> Result<()> {
    let config = Config::load()?;
    let value = config.get(key)?;
    println!("{} = {}", key, value);
    Ok(())
}
```

### Set Operation
```rust
fn handle_set(key: &str, value: &str) -> Result<()> {
    let mut config = Config::load()?;
    config.set(key, value)?;
    config.save()?;
    println!("Set {} = {}", key, value);
    Ok(())
}
```

### List Operation
```rust
fn handle_list() -> Result<()> {
    let config = Config::load()?;
    for (key, value) in config.iter() {
        println!("{} = {}", key, value);
    }
    Ok(())
}
```

## 5. Cargo Config Pattern

Cargo's config system is a good reference:

```bash
cargo config get build.jobs
cargo config set build.jobs 4
cargo config list
```

### Reference
- [Cargo Configuration Reference](https://doc.rust-lang.org/cargo/reference/config.html)

## 6. Environment Variable Handling

**For JIN_DIR specifically**:
- JIN_DIR is a process-global environment variable
- Must be set BEFORE Jin starts
- Cannot be changed via config file

**Pattern**:
```rust
// Display current JIN_DIR
if let Ok(jin_dir) = std::env::var("JIN_DIR") {
    println!("JIN_DIR: {}", jin_dir);
} else {
    println!("JIN_DIR: (using default ~/.jin)");
}
```

## 7. Best Practices

1. **Provide clear error messages** when config parsing fails
2. **Handle missing config files gracefully** - don't fail if config doesn't exist
3. **Support both get/set operations** for all config keys
4. **Document all config keys** in `--help` output
5. **Validate config values** on set

## 8. Common Gotchas

1. **Don't modify shell profiles** - users do this themselves
2. **JIN_DIR is special** - it's an env var, not a config file setting
3. **Test isolation** - each test needs its own JIN_DIR
4. **Atomic writes** - consider using atomic file writes for config changes

## Sources

- [clap v4 Documentation](https://docs.rs/clap/latest/)
- [Cargo Config Reference](https://doc.rust-lang.org/cargo/reference/config.html)
- [Rain's CLI recommendations](https://rust-cli-recommendations.sunshowers.io/hierarchical-config.html)
- [TOML crate](https://crates.io/crates/toml)
