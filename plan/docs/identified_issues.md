# Identified Issues from PRD Testing

## Critical Issues

### 1. Missing `--local` Flag for `jin add`

**Status**: NOT IMPLEMENTED
**PRD Reference**: Layer 8 (UserLocal) should be accessible
**Location**: `src/cli/args.rs` - `AddArgs` struct, `src/staging/router.rs`

**Current State**:
- `AddArgs` has: `mode`, `scope`, `project`, `global` flags
- `AddArgs` does NOT have: `local` flag
- Layer 8 (UserLocal at `~/.jin/local/`) is unreachable via CLI

**Evidence from Test Results**:
```
- `jin add --local`: **FAILED**
  - Error: `unexpected argument '--local' found`
```

**Fix Required**:
1. Add `local: bool` field to `AddArgs`
2. Update `route_to_layer()` in `src/staging/router.rs` to handle `--local`
3. Add constraint: `--local` cannot combine with other layer flags

---

### 2. SIGPIPE Handling in `jin log`

**Status**: BUG - CAUSES PANIC
**Location**: `src/commands/log.rs`

**Current State**:
- When output is piped to commands like `head`, the process panics
- Error: "failed printing to stdout: Broken pipe"

**Evidence from Test Results**:
```
#### `jin log`
- **Status**: ⚠️ PASS with Issues
- **BUG**: Panics with "Broken pipe" when piped to `head`
```

**Fix Required**:
1. Add SIGPIPE signal handler in `main.rs`
2. Or wrap stdout writes with graceful error handling
3. Standard Rust pattern: Ignore SIGPIPE or handle `BrokenPipe` error kind

---

### 3. Mode Switching UX Issue

**Status**: UX ISSUE - REQUIRES MANUAL INTERVENTION
**Location**: `src/commands/mode.rs`, `src/staging/workspace.rs`

**Current State**:
- When switching modes with `jin mode use <new-mode>`, workspace becomes "dirty"
- Jin enters "detached state" and refuses to apply
- Requires manual deletion of `$JIN_DIR/workspace/last_applied.json`

**Evidence from Test Results**:
```
#### Mode Switching Workflow Issue
- When switching modes, the workspace becomes "dirty"
- Expected workflow:
  1. Switch mode: `jin mode use <new-mode>`
  2. Clear workspace metadata: `rm $JIN_DIR/workspace/last_applied.json`
  3. Apply new mode: `jin apply`
- **Recommendation**: Auto-clear workspace metadata on mode/scope switch
```

**Fix Options**:
1. Auto-clear workspace metadata when mode/scope changes
2. Add `jin switch <mode>` command that handles everything
3. Clear `last_applied.json` during `jin mode use` if metadata references different mode

---

### 4. `--project` Flag Without Mode Fails Cryptically

**Status**: WORKS BUT CONFUSING MESSAGE
**Location**: `src/staging/router.rs`

**Current State**:
- `jin add --project` fails with "Configuration error: --project requires --mode flag"
- This is correct behavior but test report suggests confusion about accessing Layer 7 directly

**Evidence from Test Results**:
```
- `jin add --project`: **FAILED**
  - Error: `Configuration error: --project requires --mode flag`
  - This suggests direct access to the `project-base` layer (Layer 7) is not possible via `jin add`
```

**Clarification**:
- Layer 7 (ProjectBase) IS accessible via `jin add <file>` (no flags) - this is the default
- `--project` flag is for Layer 5 (ModeProject) and requires `--mode`
- Documentation may need clarification

---

## Partial Issues

### 5. `jin reset --hard` Behavior in Detached State

**Status**: PARTIAL FAILURE - DESIGN DECISION
**Location**: `src/commands/reset.rs`

**Current State**:
- `jin reset --hard` fails when workspace is in "detached state"
- Error message suggests using `jin apply --force` but that also fails
- Recovery requires manual intervention

**Evidence from Test Results**:
```
- `jin reset --hard`: **Partial Failure**
  - Failed to reset workspace file when in "detached state"
  - Error: `Workspace is in a detached state... Recovery: Run 'jin apply' to restore`
```

**Potential Improvement**:
- `--hard --force` should bypass detached state check
- Or provide clearer recovery instructions

---

## Working Features (Confirmed)

1. **Deep Merge**: JSON, YAML, TOML all merge correctly
2. **Layer System**: 9-layer precedence works as documented
3. **Scope Management**: Full CRUD operations work
4. **Mode Management**: Full CRUD operations work
5. **Git Integration**: `.gitignore` auto-management, import/export
6. **Config System**: Get/set/list operations work
7. **JIN_DIR Isolation**: Fully functional for testing

---

## Recommendations Priority

| Priority | Issue | Effort | Impact |
|----------|-------|--------|--------|
| P1 | Add `--local` flag | Low | Enables Layer 8 access |
| P1 | SIGPIPE handling | Low | Fixes CLI usability |
| P2 | Mode switching UX | Medium | Improves user experience |
| P3 | Reset in detached state | Medium | Edge case improvement |
