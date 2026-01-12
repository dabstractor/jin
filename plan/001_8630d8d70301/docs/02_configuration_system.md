# Research: Jin Configuration System

## Configuration Architecture

Jin has two configuration systems:

### 1. Global Configuration (`~/.jin/config.toml`)
- **Format**: TOML
- **Location**: `$JIN_DIR/config.toml` (default: `~/.jin/config.toml`)
- **Struct**: `JinConfig`

### 2. Project Context (`.jin/context`)
- **Format**: YAML
- **Location**: `.jin/context` (in project directory)
- **Struct**: `ProjectContext`

## Key Configuration: JIN_DIR

### What is JIN_DIR?
- Environment variable controlling where Jin stores its internal Git repository
- Default: `~/.jin/`
- Override via `export JIN_DIR=/custom/path`

### Critical Characteristic
**JIN_DIR must be set BEFORE any Jin commands run** - it is read at process startup.

### File Location Resolution
```rust
// From src/core/config.rs:75-85
pub fn default_path() -> Result<PathBuf> {
    // Check JIN_DIR environment variable first
    if let Ok(jin_dir) = std::env::var("JIN_DIR") {
        return Ok(PathBuf::from(jin_dir).join("config.toml"));
    }

    // Fall back to default ~/.jin location
    dirs::home_dir()
        .map(|h| h.join(".jin").join("config.toml"))
        .ok_or_else(|| JinError::Config("Cannot determine home directory".into()))
}
```

## Configuration Types

### JinConfig (Global)
**File**: `src/core/config.rs:13-24`

```rust
pub struct JinConfig {
    pub version: u32,
    pub remote: Option<RemoteConfig>,
    pub user: Option<UserConfig>,
}

pub struct RemoteConfig {
    pub url: String,
    pub fetch_on_init: bool,
}

pub struct UserConfig {
    pub name: String,
    pub email: String,
}
```

### ProjectContext (Project-level)
**File**: `src/core/config.rs:89-107`

```rust
pub struct ProjectContext {
    pub version: u32,
    pub mode: Option<String>,
    pub scope: Option<String>,
    pub project: Option<String>,
    pub last_updated: Option<String>,
}
```

## Configuration Access Patterns

### Loading
```rust
// Load global config
let config = JinConfig::load()?;

// Load project context
let context = ProjectContext::load()?;
```

### Saving
```rust
// Modify and save
let mut config = JinConfig::load()?;
config.remote = Some(RemoteConfig { url: "...".to_string(), fetch_on_init: true });
config.save()?;
```

### Validation
```rust
// Check if initialized
if !ProjectContext::is_initialized() {
    return Err(JinError::NotInitialized);
}

// Get required values
let mode = context.require_mode()?;
```

## Gotchas and Constraints

1. **Test Isolation**: Every test sets `JIN_DIR` to a temporary directory
2. **Process-Global**: `JIN_DIR` is set via `std::env::set_var()`
3. **File Formats**: Global config = TOML, project context = YAML
4. **No Atomic Writes**: No atomic write guarantees for config files
5. **No Validation**: No schema validation for config files
6. **No Migration**: No version migration support

## Key Files

- `src/core/config.rs` - Configuration types and logic
- `src/core/error.rs` - Configuration error types
- `src/commands/init.rs` - Initialization logic
- `src/commands/context.rs` - Context command

## Sources
- `/home/dustin/projects/jin/src/core/config.rs`
- `/home/dustin/projects/jin/README.md` (lines 177-298)
