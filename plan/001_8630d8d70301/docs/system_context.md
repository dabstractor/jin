# Jin System Architecture Context

## Project Overview

**Jin** is a phantom Git layer system for developer configuration management. It's a Rust-based CLI tool that manages developer-specific and tool-specific configuration without contaminating a project's primary Git repository.

## Core Architecture

### The 9-Layer System

Jin implements a deterministic 9-layer precedence system stored in Git refs:

| Layer | Name | Precedence | Git Ref Path | Storage Path | Description |
|-------|------|------------|--------------|--------------|-------------|
| 1 | GlobalBase | 1 | `refs/jin/layers/global` | `jin/global/` | Shared defaults across all projects |
| 2 | ModeBase | 2 | `refs/jin/layers/mode/<mode>/_` | `jin/mode/<mode>/` | Mode-specific defaults |
| 3 | ModeScope | 3 | `refs/jin/layers/mode/<mode>/scope/<scope>/_` | `jin/mode/<mode>/scope/<scope>/` | Scoped mode configs |
| 4 | ModeScopeProject | 4 | (nested under mode/scope) | `jin/mode/<mode>/scope/<scope>/project/<project>/` | Project overrides for scoped mode |
| 5 | ModeProject | 5 | `refs/jin/layers/mode/<mode>/project/<project>` | `jin/mode/<mode>/project/<project>/` | Project overrides for mode |
| 6 | ScopeBase | 6 | `refs/jin/layers/scope/<scope>` | `jin/scope/<scope>/` | Untethered scope configs |
| 7 | ProjectBase | 7 | `refs/jin/layers/project/<project>` | `jin/project/<project>/` | Project-only configs |
| 8 | UserLocal | 8 | `refs/jin/layers/local` | `~/.jin/local/` | Machine-only overlays |
| 9 | WorkspaceActive | 9 | `refs/jin/layers/workspace` | `.jin/workspace/` | Derived merge result |

**Note**: The `/_` suffix is used for layers that have child refs to avoid Git ref naming conflicts.

### Module Structure

```
src/
├── cli/                 # CLI argument parsing (clap)
│   ├── mod.rs          # Cli struct and Commands enum
│   └── args.rs         # Command-specific argument structs
├── commands/           # Command implementations (28 total)
│   ├── add.rs          # Stage files to layers
│   ├── apply.rs        # Apply merged layers to workspace
│   ├── commit_cmd.rs   # Commit staged files
│   ├── mode.rs         # Mode management
│   ├── scope.rs        # Scope management
│   ├── reset.rs        # Reset staged/workspace changes
│   ├── log.rs          # Show commit history
│   └── ...             # Other commands
├── core/               # Core data structures
│   ├── layer.rs        # Layer enum with precedence
│   ├── config.rs       # JinConfig, ProjectContext
│   ├── error.rs        # JinError unified errors
│   └── jinmap.rs       # Layer-to-file mapping
├── git/                # Git integration
│   ├── repo.rs         # JinRepo wrapper
│   ├── refs.rs         # RefOps for layer refs
│   ├── objects.rs      # ObjectOps for blobs/trees
│   ├── transaction.rs  # Atomic multi-ref updates
│   └── remote.rs       # Remote sync operations
├── merge/              # Merge engine
│   ├── value.rs        # MergeValue universal representation
│   ├── deep.rs         # RFC 7396 deep merge
│   ├── layer.rs        # Multi-layer merge orchestration
│   ├── text.rs         # 3-way text merge
│   └── jinmerge.rs     # Conflict file format
├── staging/            # Staging area
│   ├── entry.rs        # StagedEntry, StagedOperation
│   ├── index.rs        # StagingIndex
│   ├── router.rs       # route_to_layer() logic
│   ├── workspace.rs    # Workspace state management
│   └── gitignore.rs    # .gitignore managed block
├── commit/             # Commit pipeline
│   └── pipeline.rs     # Atomic commit orchestration
└── audit/              # Audit logging
    ├── entry.rs        # AuditEntry structure
    └── logger.rs       # AuditLogger
```

### Key Data Structures

#### Layer Enum (`src/core/layer.rs`)
```rust
pub enum Layer {
    GlobalBase, ModeBase, ModeScope, ModeScopeProject,
    ModeProject, ScopeBase, ProjectBase, UserLocal, WorkspaceActive,
}
```

Key methods: `precedence()`, `ref_path()`, `storage_path()`, `requires_mode()`, `requires_scope()`, `is_project_specific()`

#### ProjectContext (`src/core/config.rs`)
```rust
pub struct ProjectContext {
    pub mode: Option<String>,
    pub scope: Option<String>,
    pub project: Option<String>,
    pub last_updated: Option<String>,
}
```
Stored at `.jin/context` in YAML format.

#### StagedEntry (`src/staging/entry.rs`)
```rust
pub struct StagedEntry {
    pub path: PathBuf,
    pub target_layer: Layer,
    pub content_hash: String,  // Git blob OID
    pub mode: u32,
    pub operation: StagedOperation,
}
```

#### MergeValue (`src/merge/value.rs`)
```rust
pub enum MergeValue {
    Null, Bool(bool), Integer(i64), Float(f64), String(String),
    Array(Vec<MergeValue>), Object(IndexMap<String, MergeValue>),
}
```

### Merge System

**Supported Formats**: JSON, YAML, TOML, INI, Plain Text

**Deep Merge Rules** (RFC 7396):
1. Null values delete keys
2. Objects merge recursively
3. Keyed arrays (with "id" or "name" fields) merge by key
4. Unkeyed arrays are replaced entirely
5. Scalars are overridden by higher precedence

### Workspace States

1. **CLEAN**: Workspace files match last applied configuration
2. **DIRTY**: User modified files outside of Jin operations
3. **DETACHED**: Workspace is disconnected from valid layer commits

### Layer Routing (`jin add` flags)

| Command | Target Layer |
|---------|--------------|
| `jin add <file>` | ProjectBase (7) |
| `jin add <file> --mode` | ModeBase (2) |
| `jin add <file> --mode --project` | ModeProject (5) |
| `jin add <file> --scope=<scope>` | ScopeBase (6) |
| `jin add <file> --mode --scope=<scope>` | ModeScope (3) |
| `jin add <file> --mode --scope=<scope> --project` | ModeScopeProject (4) |
| `jin add <file> --global` | GlobalBase (1) |

**Note**: `--local` flag is NOT implemented (Layer 8 inaccessible via CLI).

### Testing Infrastructure

- **Location**: `/home/dustin/projects/jin/tests/`
- **Framework**: `assert_cmd`, `predicates`, `tempfile`, `serial_test`
- **Isolation**: Uses `JIN_DIR` environment variable
- **Common utilities**: `tests/common/mod.rs`, `tests/common/fixtures.rs`

### Key Dependencies

- **clap 4.5**: CLI parsing
- **git2 0.19**: Git operations
- **serde/serde_json/serde_yaml/toml**: Serialization
- **thiserror 2.0**: Error handling
- **indexmap 2.0**: Ordered maps
- **diffy 0.4**: Text diffing
