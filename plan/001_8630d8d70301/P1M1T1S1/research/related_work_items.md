# Related Work Items Research for P1.M1.T1.S1 (--local Flag Implementation)

## Overview

This research document provides context for implementing the `--local` flag for the `jin add` command, which is part of P1.M1 (Implement Missing --local Flag for Layer 8 Access). The implementation will enable users to access Layer 8 (UserLocal) for storing machine-specific configuration overlays.

## Parent Work Items and Hierarchical Relationships

### Phase 1: Jin CLI Bug Fixes & Missing Features (P1)
- **Purpose**: Address all critical bugs and missing features identified in the PRD test results
- **Status**: Planned
- **Contains**: 5 milestones focused on fixing identified issues and improving user experience

### Milestone P1.M1: Implement Missing --local Flag for Layer 8 Access
- **Parent**: P1 (Phase 1)
- **Purpose**: Enable access to Layer 8 (UserLocal) via the `jin add --local` command
- **Status**: Planned
- **Contains**: 4 tasks covering implementation, routing, wiring, and testing

### Task P1.M1.T1: Add --local Flag to CLI Arguments
- **Parent**: P1.M1
- **Purpose**: Extend the AddArgs struct to include the --local flag for routing files to Layer 8
- **Status**: Researching (current task)
- **Contains**: 1 subtask (P1.M1.T1.S1)

### Subtask P1.M1.T1.S1: Add local field to AddArgs struct (CURRENT TASK)
- **Parent**: P1.M1.T1
- **Purpose**: Add `local: bool` field to AddArgs struct with clap attribute for CLI parsing
- **Story Points**: 0.5
- **Dependencies**: None
- **Status**: Researching

## Downstream Dependencies and Usage

### 1. Routing Logic (P1.M1.T2)

The `local` field will be consumed by the routing system:

**File**: `src/staging/router.rs`
- **RoutingOptions struct** will be extended with `local: bool` field
- **route_to_layer() function** will:
  - Validate that `--local` cannot combine with other layer flags
  - Route to `Layer::UserLocal` when `--local` is specified
  - Return appropriate error for invalid flag combinations

**Integration Point**:
```rust
// In route_to_layer() function
if options.local {
    if options.mode || options.scope.is_some() || options.project || options.global {
        return Err(JinError::Config("--local cannot be combined with other layer flags".into()));
    }
    return Ok(Layer::UserLocal);
}
```

### 2. Command Wiring (P1.M1.T3)

The `--local` flag will be passed through the command pipeline:

**File**: `src/commands/add.rs`
- **execute() function** will create RoutingOptions with `local: args.local`
- This flag value will flow through to the routing layer

**Integration Point**:
```rust
// In add command execution
let routing_options = RoutingOptions {
    mode: args.mode,
    scope: args.scope,
    project: args.project,
    global: args.global,
    local: args.local,  // NEW FIELD
};
```

### 3. Integration Tests (P1.M1.T4)

Comprehensive tests will verify the `--local` functionality:

**File**: `tests/cli_add_local.rs` (new file)
- Test that `jin add .config --local` routes to Layer 8
- Test that `--local` rejects combinations with other flags
- Test the complete workflow: add, commit, apply, verify

## Architectural Context

### The 9-Layer System (PRD Section 4.1)

| Layer | Name | Precedence | Storage Path | Description |
|-------|------|------------|--------------|-------------|
| 8 | UserLocal | 8 | `~/.jin/local/` | Machine-only overlays |

**Key Points**:
- Layer 8 has the second highest precedence (only WorkspaceActive is higher)
- Stores machine-specific configuration that overrides all other layers
- Files are stored in `~/.jin/local/` (user's home directory)
- Designed for local development environment overrides

### Current Layer Routing System (PRD Section 9.1)

The current `jin add` routing table:
| Command | Target Layer |
|---------|--------------|
| `jin add <file>` | ProjectBase (7) |
| `jin add <file> --mode` | ModeBase (2) |
| `jin add <file> --mode --project` | ModeProject (5) |
| `jin add <file> --scope=<scope>` | ScopeBase (6) |
| `jin add <file> --mode --scope=<scope>` | ModeScope (3) |
| `jin add <file> --mode --scope=<scope> --project` | ModeScopeProject (4) |
| `jin add <file> --global` | GlobalBase (1) |

**Missing**: `jin add <file> --local` → UserLocal (8)

### Current AddArgs Struct (CLI Layer)

**File**: `src/cli/args.rs`
```rust
pub struct AddArgs {
    /// Files to stage
    pub files: Vec<String>,

    /// Target mode layer
    #[arg(long)]
    pub mode: bool,

    /// Target scope layer
    #[arg(long)]
    pub scope: Option<String>,

    /// Target project layer
    #[arg(long)]
    pub project: bool,

    /// Target global layer
    #[arg(long)]
    pub global: bool,
    // TODO: Add --local flag here
}
```

## Technical Implementation Context

### 1. Validation Rules

The `--local` flag implementation must enforce these rules:
- **Exclusivity**: `--local` cannot be combined with any other layer flags (`--mode`, `--scope`, `--project`, `--global`)
- **Independence**: `--local` does NOT require an active mode or scope
- **Layer Target**: Always routes to `Layer::UserLocal` (Layer 8)

### 2. Error Handling Patterns

Following existing patterns in the codebase:
```rust
// Similar to --global validation
if options.local && (options.mode || options.scope.is_some() || options.project || options.global) {
    return Err(JinError::Config("--local cannot be combined with other layer flags".into()));
}
```

### 3. File Path Resolution

When routed to Layer 8, files should be stored at:
- **Storage Path**: `~/.jin/local/<file_path>`
- **Git Ref**: `refs/jin/layers/local`
- **Working Directory**: `.jin/workspace/<file_path>` (after merge)

## Dependencies on Other Subtasks

### Current Dependencies (P1.M1.T1.S1)
- **None** - This is the foundational subtask that other dependencies rely on

### Downstream Dependencies (P1.M1.T1.S1 → ...)
1. **P1.M1.T2.S1**: RoutingOptions struct modification
   - Depends on AddArgs having the `local` field

2. **P1.M1.T2.S2**: Validation logic in route_to_layer()
   - Depends on RoutingOptions having the `local` field

3. **P1.M1.T2.S3**: Routing case for Layer::UserLocal
   - Depends on validation being implemented

4. **P1.M1.T3.S1**: Command wiring in add command
   - Depends on routing implementation being complete

5. **P1.M1.T4.S1**: Integration tests
   - Depends on complete implementation through the entire pipeline

### Cross-Milestone Dependencies

#### P1.M3 (Mode Switching UX)
- **Indirect dependency**: UserLocal layer files might need special handling during mode switches
- **Consideration**: Machine-specific files should persist across mode changes

#### P1.M4 (Reset in Detached State)
- **Indirect dependency**: Layer 8 files should be preserved during reset operations
- **Consideration**: UserLocal files are machine-specific and should survive most resets

#### P1.M5 (Documentation Updates)
- **Direct dependency**: The layer routing table in help text will need to include `--local` → Layer 8
- **Update required**: `jin add --help` will show the new routing option

## Integration Points with Existing Systems

### 1. Git Integration

- **Repository**: Uses the same Jin Git repository at `~/.jin/`
- **Ref Path**: Files committed to Layer 8 will be stored under `refs/jin/layers/local`
- **Atomic Operations**: Follows the same two-phase commit pattern as other layers

### 2. Merge System

- **Precedence**: Layer 8 has higher precedence than all layers except WorkspaceActive
- **Merge Behavior**: UserLocal files will override all lower layers during merge
- **Conflict Resolution**: If UserLocal files conflict with workspace files, UserLocal wins

### 3. .gitignore Management

- **Automatic Addition**: Files added with `--local` will be automatically added to `.gitignore`
- **Managed Block**: Follows the same managed block pattern as other Jin files

### 4. Context System

- **Mode Independence**: No active mode required for `--local` operations
- **Scope Independence**: No active scope required for `--local` operations
- **Project Inference**: Still uses the current project context for path construction

## Testing Strategy Context

### 1. Unit Tests

The `--local` functionality will impact:
- **CLI parsing**: Test `AddArgs` struct with `--local` flag
- **Routing logic**: Test `route_to_layer()` with local flag combinations
- **Layer validation**: Test that Layer::UserLocal is correctly targeted

### 2. Integration Tests

From P1.M1.T4.S1 requirements:
```rust
// Test case examples
test_add_local_routes_to_layer_8          // Verify correct routing
test_add_local_rejects_mode_flag           // Verify flag exclusivity
test_add_local_rejects_global_flag        // Verify flag exclusivity
test_add_local_commit_apply_workflow     // End-to-end workflow
```

### 3. Common Test Infrastructure

Will leverage existing test infrastructure:
- **TestFixture**: Provides isolated Jin environment
- **assert_cmd**: CLI command testing
- **predicates**: Output verification
- **tempfile**: Temporary file management

## Potential Future Extensions

### 1. UserLocal-Only Commands

Future enhancements might include:
- `jin config --local` for setting local-only configuration
- `jin list --local` for listing local files only
- `jin reset --local` for resetting local files only

### 2. Local Sync Patterns

Potential for local-specific sync behaviors:
- Local files might never be pushed to remote (machine-specific)
- Or selective sync with user preference
- Separate backup strategies for local configurations

### 3. Environment Integration

Future integration with:
- Machine detection (different files per machine)
- Environment variable injection from local config
- Local override files for development vs production

## Implementation Risks and Considerations

### 1. Cross-Platform Path Issues

- **Home Directory**: Use `dirs` crate to reliably find `~/.jin/local/`
- **Path Separators**: Ensure forward slashes work on all platforms
- **Permission Issues**: Handle cases where home directory is not writable

### 2. User Confusion

- **Documentation**: Clear help text explaining when to use `--local`
- **Layer Understanding**: Users might confuse Layer 8 with project-specific layers
- **Workflow Integration**: Examples showing typical `--local` use cases

### 3. Backup and Recovery

- **Local Files**: Machine-specific files might need special backup handling
- **Migration**: Moving between machines might require local file migration
- **Cleanup**: Mechanism for cleaning up old local configurations

## Conclusion

The `--local` flag implementation (P1.M1.T1.S1) is a foundational change that enables access to Layer 8 of the Jin layer system. It has clear dependencies and impacts multiple downstream subtasks within P1.M1. The implementation follows established patterns in the codebase for CLI argument parsing, validation, and routing. The feature completes the layer routing table and provides users with the ability to store machine-specific configuration overlays that can override all other layers except the active workspace.

This implementation is part of a larger effort to address identified gaps between the PRD specification and the actual implementation, specifically focusing on enabling access to all 9 layers of the Jin system through the CLI interface.