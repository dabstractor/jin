# External Dependencies Analysis

## 1. Core Dependencies

### 1.1 Git Integration: git2 0.20

**Purpose**: All Git operations (refs, objects, commits, transactions)

**Features Used**:
- `vendored-libgit2`: Statically link libgit2 for portability
- `ssh`: SSH transport for remote operations
- `https`: HTTPS transport for remote operations

**Key APIs**:
```rust
Repository::open(), Repository::init()
Repository::reference(), Repository::find_reference()
Repository::blob(), Repository::treebuilder()
Repository::commit()
Repository::transaction() // For atomic ref updates
```

**Considerations**:
- Vendored build adds compile time but ensures compatibility
- SSH/HTTPS features required for P5 remote sync

### 1.2 CLI Framework: clap 4.5

**Purpose**: Command-line argument parsing with derive macros

**Features Used**:
- `derive`: Automatic parser generation from structs

**Key Patterns**:
```rust
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init { ... },
    Add { files: Vec<PathBuf> },
    // ...
}
```

**Shell Completion**: Use `clap_complete` crate for P6.M1

### 1.3 Serialization: serde 1.0

**Purpose**: Unified serialization framework

**Features Used**:
- `derive`: Derive Serialize/Deserialize

**Usage Pattern**:
```rust
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub version: u8,
    pub mode: Option<String>,
}
```

## 2. Format-Specific Parsers

### 2.1 JSON: serde_json 1.0

**Purpose**: JSON parsing and generation

**Key APIs**:
```rust
serde_json::from_slice::<Value>(&bytes)
serde_json::to_vec_pretty(&value)
```

### 2.2 YAML: serde_yaml_ng 0.9

**Purpose**: YAML parsing and generation

**Note**: Using `serde_yaml_ng` (next-gen fork) instead of deprecated `serde_yaml`

**Key APIs**:
```rust
serde_yaml_ng::from_slice::<Value>(&bytes)
serde_yaml_ng::to_string(&value)
```

### 2.3 TOML: toml 0.9

**Purpose**: TOML parsing and generation

**Key APIs**:
```rust
toml::from_str::<Value>(&str)
toml::to_string_pretty(&value)
```

### 2.4 INI: configparser 0.4

**Purpose**: INI file parsing (section-based)

**Key APIs**:
```rust
let mut config = Ini::new();
config.read(content);
config.sections() // Get all sections
config.get(section, key)
```

## 3. Utility Dependencies

### 3.1 Diff/Merge: similar 2.6

**Purpose**: Text diff and 3-way merge

**Key APIs**:
```rust
similar::TextDiff::from_lines(base, other)
// 3-way merge requires manual implementation
```

**Note**: 3-way merge is implemented in `src/merge/text.rs` using similar's diff

### 3.2 Ordered Maps: indexmap 2.7

**Purpose**: Preserve key insertion order (critical for deterministic merge)

**Features Used**:
- `serde`: Serialize/deserialize IndexMap

**Key Type**:
```rust
use indexmap::IndexMap;
// MergeValue::Object(IndexMap<String, MergeValue>)
```

### 3.3 UUIDs: uuid 1.19

**Purpose**: Transaction IDs

**Features Used**:
- `v4`: Random UUIDs

**Usage**:
```rust
use uuid::Uuid;
let id = Uuid::new_v4();
```

### 3.4 Timestamps: chrono 0.4

**Purpose**: Audit timestamps, context last-updated

**Features Used**:
- `serde`: Serialize timestamps

**Usage**:
```rust
use chrono::{DateTime, Utc};
let now: DateTime<Utc> = Utc::now();
```

### 3.5 Error Handling

**thiserror 2.0**: Define error types with derive macro
```rust
#[derive(Error, Debug)]
pub enum JinError {
    #[error("not initialized")]
    NotInitialized,
}
```

**anyhow 1.0**: Available for ad-hoc error handling (less used)

### 3.6 File Operations

**tempfile 3.12**: Atomic file writes
```rust
let temp = NamedTempFile::new()?;
// write to temp
temp.persist(final_path)?;
```

**walkdir 2.5**: Directory traversal
```rust
for entry in WalkDir::new(path) { ... }
```

**dirs 5.0**: Home directory detection
```rust
dirs::home_dir() // Returns Option<PathBuf>
```

## 4. Test Dependencies

### 4.1 CLI Testing: assert_cmd 2.0

**Purpose**: Integration test CLI invocations

```rust
use assert_cmd::Command;

Command::cargo_bin("jin")
    .arg("init")
    .assert()
    .success();
```

### 4.2 Assertions: predicates 3.1

**Purpose**: Fluent assertions for test output

```rust
use predicates::prelude::*;

cmd.assert()
    .stdout(predicate::str::contains("initialized"));
```

### 4.3 Snapshots: insta 1.40

**Purpose**: Snapshot testing for complex outputs

```rust
use insta::assert_snapshot;

assert_snapshot!(merged_output);
```

## 5. Dependency Versions (Cargo.toml)

```toml
[dependencies]
git2 = { version = "0.20", features = ["vendored-libgit2", "ssh", "https"] }
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.9"
serde_yaml_ng = "0.9"
configparser = "0.4"
similar = "2.6"
walkdir = "2.5"
indexmap = { version = "2.7", features = ["serde"] }
uuid = { version = "1.19", features = ["v4"] }
anyhow = "1.0"
thiserror = "2.0"
tempfile = "3.12"
dirs = "5.0"
chrono = { version = "0.4", features = ["serde"] }

[dev-dependencies]
assert_cmd = "2.0"
predicates = "3.1"
insta = "1.40"
```

## 6. Future Dependencies (P5/P6)

### 6.1 Shell Completion: clap_complete

```toml
clap_complete = "4.5"
```

### 6.2 Credential Handling

For remote SSH/HTTPS auth, git2's credential callbacks may need:
```rust
// git2 has built-in credential handling via Credentials type
```

### 6.3 Progress Indicators (optional)

```toml
indicatif = "0.17"  # Progress bars
console = "0.15"    # Terminal colors
```

## 7. Build Considerations

### 7.1 Compile Time

- `git2` with vendored libgit2: ~2-3 minutes on first build
- Total clean build: ~4-5 minutes

### 7.2 Binary Size

Release profile with LTO and stripping:
```toml
[profile.release]
lto = true
codegen-units = 1
strip = true
```

Expected binary size: 5-10 MB

### 7.3 Cross-Compilation

Vendored libgit2 simplifies cross-compilation but may need:
- OpenSSL headers for some targets
- CMake for libgit2 build
