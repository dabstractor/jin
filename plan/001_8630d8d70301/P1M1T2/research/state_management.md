# State Management Patterns in Jin - Research Findings

## 1. Persistent State File Creation and Management

Jin uses a multi-layered state management system with persistent files stored in both user-level and project-level locations:

### User-Level State (`~/.jin/`)
- **Location**: `$JIN_DIR` (default: `~/.jin/`) - can be overridden with `JIN_DIR` environment variable for testing
- **Purpose**: Global configuration and repository management
- **Pattern**: Bare Git repository for storing layer configurations
- **Created by**: `JinRepo::open_or_create()` in `src/git/repo.rs`

### Project-Level State (`.jin/`)
- **Location**: `.jin/` directory in each project
- **Purpose**: Per-project context, mappings, and transaction state
- **Pattern**: Regular directory with multiple files

## 2. State File Locations and Patterns

| File/Directory | Location | Format | Purpose |
|----------------|----------|--------|---------|
| `~/.jin/config.toml` | User home | TOML | Global Jin configuration |
| `~/.jin/` (repo) | User home | Git repo | Bare repository storing all layer refs |
| `.jin/context` | Project root | YAML | Active context (mode, scope, project) |
| `.jin/.jinmap` | Project root | YAML | Layer-to-file mapping metadata |
| `.jin/.transaction_in_progress` | Project root | JSON | Transaction log for crash recovery |
| `.jin/staging/index.json` | Project root | JSON | Staging area index |

## 3. Serialization Formats Used

### TOML Format
- **Files**: `~/.jin/config.toml`
- **Library**: `toml = "0.8"`
- **Usage**: Global configuration with `JinConfig` struct

### YAML Format
- **Files**: `.jin/context`, `.jin/.jinmap`
- **Library**: `serde_yaml = "0.9"`
- **Usage**: Human-readable project context and mapping data

### JSON Format
- **Files**: `.jin/.transaction_in_progress`
- **Library**: Built-in `serde_json`
- **Usage**: Transaction logs with crash recovery capabilities

### Git Native Format
- **Files**: All layer refs in `~/.jin/` (bare Git repo)
- **Usage**: Storing actual layer commits and metadata
- **Pattern**: Uses `git2` library for all Git operations

## 4. Operation State Tracking

### Transaction System (Two-Phase Commit)
Location: `src/git/transaction.rs`

**Key Components:**
- `TransactionState` enum: `Pending`, `Prepared`, `Committed`, `Aborted`
- `TransactionLog`: Persistent state stored in `.jin/.transaction_in_progress`
- `LayerTransaction`: Atomic multi-layer commit with crash recovery
- `RecoveryManager`: Detects and handles incomplete transactions

**State Transitions:**
```
Pending -> Prepared -> Committed
   |          |
   v          v
  Aborted   Aborted
```

## 5. Pause/Resume Patterns

### Existing Patterns:
1. **Transaction Recovery**: The `RecoveryManager` provides basic pause/resume capabilities:
   - Detects incomplete transactions on startup
   - Can rollback or resume based on transaction state
   - Only supports transaction operations, not general operations

2. **No General Pause/Resume**: Currently, there is no general-purpose pause/resume system for:
   - Long-running operations (sync, fetch, push)
   - Partial applies with conflicts
   - Multi-step workflows

## 6. JIN_DIR Configuration and Access

### Configuration Pattern:
```rust
// From src/git/repo.rs
impl JinRepo {
    /// Returns the default Jin repository path (`~/.jin/`).
    ///
    /// Can be overridden with the `JIN_DIR` environment variable for testing.
    pub fn default_path() -> Result<PathBuf> {
        // Check for JIN_DIR environment variable first (for testing)
        if let Ok(jin_dir) = std::env::var("JIN_DIR") {
            return Ok(PathBuf::from(jin_dir));
        }

        dirs::home_dir()
            .map(|h| h.join(".jin"))
            .ok_or_else(|| JinError::Config("Cannot determine home directory".into()))
    }
}
```

## 7. State File Creation Patterns

### Common Pattern:
1. Check if file exists with `path.exists()`
2. If exists, deserialize with proper error handling
3. If doesn't exist, return default or create new
4. Save with atomic write pattern (temp file + rename)

### Example from `JinMap::save()`:
```rust
// Atomic write pattern
let temp_path = path.with_file_name(format!("{}.tmp", ...));
std::fs::write(&temp_path, content)?;
std::fs::rename(&temp_path, &path)?;  // Atomic rename
```

## Key Insights

1. **Git-Centric**: Jin uses Git as its primary persistence layer
2. **Atomic Operations**: All writes use atomic patterns
3. **Crash Recovery**: Transaction system provides robust crash recovery
4. **Human-Readable**: Context and mapping files use YAML
5. **Environment Override**: `JIN_DIR` allows flexible configuration for testing
6. **No General Pause/Resume**: While transaction recovery exists, there's no general pause/resume system
