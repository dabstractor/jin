# Jin System Context and Architecture

## 1. Project Overview

**Jin** is a meta-versioning system layered on top of Git that manages developer-specific and tool-specific configuration without contaminating a project's primary Git repository.

### 1.1 Current Implementation Status

The project has a substantial Rust implementation on the `glm` branch with:
- **Complete**: P1 (Foundation), P2 (Merge Engine), P3 (Staging)
- **Partial**: P4 (CLI Commands - M1-M2 complete, M3-M5 need implementation)
- **Planned**: P5 (Synchronization), P6 (Polish)

### 1.2 Technology Stack (Validated)

| Component | Technology | Version | Purpose |
|-----------|------------|---------|---------|
| Language | Rust | 2021 Edition | Core implementation |
| CLI Framework | clap | 4.5 | Command-line argument parsing |
| Git Integration | git2 | 0.20 | libgit2 bindings (vendored) |
| JSON | serde_json | 1.0 | JSON parse/serialize |
| YAML | serde_yaml_ng | 0.9 | YAML parse/serialize |
| TOML | toml | 0.9 | TOML parse/serialize |
| INI | configparser | 0.4 | INI file handling |
| Diff/Merge | similar | 2.6 | 3-way text merge |
| Ordering | indexmap | 2.7 | Deterministic key ordering |
| Error Handling | thiserror/anyhow | 2.0/1.0 | Error types |
| Testing | assert_cmd, insta | 2.0, 1.40 | Integration/snapshot tests |

## 2. Architectural Components

### 2.1 Module Structure

```
src/
├── main.rs              # CLI entry point
├── lib.rs               # Library exports
├── cli/                 # Command-line interface (clap derive)
│   └── mod.rs
├── commands/            # Command implementations
│   ├── mod.rs
│   ├── context.rs       # Active context management
│   ├── diff.rs          # Layer diff viewing
│   ├── export.rs        # Export from Jin to Git
│   ├── import_cmd.rs    # Import from Git to Jin
│   ├── log.rs           # Commit history
│   ├── mode.rs          # Mode lifecycle
│   ├── repair.rs        # State recovery
│   └── scope.rs         # Scope lifecycle
├── core/                # Domain types
│   ├── mod.rs
│   ├── config.rs        # JinConfig, ProjectContext
│   ├── error.rs         # JinError enum
│   └── layer.rs         # 9-layer hierarchy
├── staging/             # Staging system
│   ├── mod.rs
│   ├── entry.rs         # StagedEntry type
│   ├── index.rs         # Persistent staging index
│   └── router.rs        # Flag-to-layer routing
├── commit/              # Commit pipeline
│   ├── mod.rs
│   ├── pipeline.rs      # Atomic commit orchestration
│   ├── audit.rs         # Audit trail
│   ├── jinmap.rs        # .jinmap management
│   └── validate.rs      # Pre-commit validation
├── merge/               # Merge engine
│   ├── mod.rs
│   ├── value.rs         # Universal MergeValue type
│   ├── deep.rs          # Deep merge algorithm
│   ├── layer.rs         # Layer-wise merging
│   ├── array.rs         # Array merge strategies
│   ├── text.rs          # 3-way text merge
│   └── config.rs        # Merge configuration
├── git/                 # Git operations
│   ├── mod.rs
│   ├── repo.rs          # JinRepo wrapper
│   ├── objects.rs       # Blob/tree/commit creation
│   ├── refs.rs          # Reference management
│   └── transaction.rs   # Atomic transactions
└── workspace/           # Workspace management
    ├── mod.rs
    ├── apply.rs         # Layer application
    ├── gitignore.rs     # Managed .gitignore block
    └── jinmap_mgr.rs    # .jinmap file manager
```

### 2.2 The 9-Layer Hierarchy

```
Layer 9: WorkspaceActive         ← Derived (never source of truth)
Layer 8: UserLocal               ← ~/.jin/local/
Layer 7: ProjectBase             ← jin/project/<project>/
Layer 6: ScopeBase               ← jin/scope/<scope>/
Layer 5: ModeProject             ← jin/mode/<mode>/project/<project>/
Layer 4: ModeScopeProject        ← jin/mode/<mode>/scope/<scope>/project/<project>/
Layer 3: ModeScope               ← jin/mode/<mode>/scope/<scope>/
Layer 2: ModeBase                ← jin/mode/<mode>/
Layer 1: GlobalBase              ← jin/global/
```

**Precedence**: Higher numbers override lower numbers during merge.

### 2.3 Git Ref Namespace

All Jin data is stored in a separate Git repository at `~/.jin/` using custom refs:

```
refs/jin/layers/global                              # Layer 1
refs/jin/layers/mode/<mode>                         # Layer 2
refs/jin/layers/mode/<mode>/scope/<scope>           # Layer 3
refs/jin/layers/mode/<mode>/scope/<scope>/project/<project>  # Layer 4
refs/jin/layers/mode/<mode>/project/<project>       # Layer 5
refs/jin/layers/scope/<scope>                       # Layer 6
refs/jin/layers/project/<project>                   # Layer 7
refs/jin/staging/<transaction-id>                   # Temp transaction refs
```

## 3. Key Patterns

### 3.1 Command Handler Pattern

```rust
pub struct CommandName {
    // CLI args as fields
}

impl CommandName {
    pub fn new(...) -> Self { ... }
    pub fn execute(&self) -> Result<()> { ... }
}
```

### 3.2 Layer Routing Pattern

The router maps CLI flags to target layers:

| Flags | Target Layer |
|-------|--------------|
| (none) | Layer 7 (ProjectBase) |
| `--mode` | Layer 2 (ModeBase) |
| `--mode --project` | Layer 5 (ModeProject) |
| `--mode --scope` | Layer 3 (ModeScope) |
| `--mode --scope --project` | Layer 4 (ModeScopeProject) |
| `--scope` | Layer 6 (ScopeBase) |
| `--global` | Layer 1 (GlobalBase) |

### 3.3 Atomic Transaction Pattern

```
1. Begin transaction (create staging ref)
2. Build trees for each affected layer
3. Create commits pointing to trees
4. Prepare: lock refs, validate old OIDs
5. Commit: atomically update all refs
6. Cleanup: remove staging ref
```

### 3.4 Merge Strategy

1. Collect all layers in precedence order (1→9)
2. For each file, determine format (JSON/YAML/TOML/INI/text)
3. Parse structured files into MergeValue
4. Apply deep merge: higher layer overrides, null deletes keys
5. Arrays: keyed arrays merge by id/name, unkeyed arrays replace
6. Text files: 3-way merge with conflict markers
7. Serialize back to original format

## 4. File Locations

### 4.1 Global Jin Storage

```
~/.jin/
├── config.yaml          # Global configuration
├── local/               # User local overlays (Layer 8)
└── repo/                # Bare Git repository
    ├── refs/jin/...     # Layer refs
    └── objects/...      # Git objects
```

### 4.2 Per-Project Files

```
<project>/
├── .jin/
│   ├── context          # Active mode/scope
│   ├── workspace/       # Applied files
│   └── staging/
│       └── index.json   # Staged changes
├── .jinmap              # Layer-to-file mapping
└── .gitignore           # Contains Jin managed block
```

## 5. Implementation Notes

### 5.1 What's Complete

- **Core Types**: Layer enum, JinError, Result alias
- **Git Operations**: JinRepo, refs, objects, transactions
- **Merge Engine**: MergeValue, deep merge, array strategies, text 3-way merge
- **Staging**: StagedEntry, StagingIndex, router
- **Commit Pipeline**: Validation, .jinmap, audit
- **CLI Framework**: All commands defined in clap

### 5.2 What Needs Implementation

The command handlers in `src/commands/` are mostly stubs that return errors. These need to be connected to the existing infrastructure:

1. **InitCommand**: Create .jin directory, link to Jin repo
2. **AddCommand**: Use staging router, add to StagingIndex
3. **CommitCommand**: Use CommitPipeline
4. **StatusCommand**: Read StagingIndex and workspace state
5. **Mode/Scope commands**: Already partially implemented
6. **Apply**: Implemented via `apply_workspace()`
7. **Remote sync**: fetch/pull/push/sync (P5)

### 5.3 Key Invariants

1. WorkspaceActive is NEVER a source of truth
2. Commits are atomic across all affected layers
3. .gitignore managed block is never modified outside markers
4. Scope precedence: mode-bound > untethered > mode base
5. No symlinks, binary files, or submodules
6. Merge is deterministic: same inputs = same outputs
