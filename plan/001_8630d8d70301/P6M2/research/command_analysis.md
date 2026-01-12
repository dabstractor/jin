# Jin CLI Command Analysis for Integration Testing

## OVERVIEW

Jin implements a sophisticated 9-layer configuration management system with 25 CLI commands across multiple functional areas:

1. **Initialization & Core Management** (4 commands)
2. **Layer Management** (6 commands: modes, scopes)
3. **Staging & Commits** (2 commands)
4. **Workspace Operations** (3 commands: apply, reset, diff)
5. **History & Inspection** (3 commands: log, status, context, layers, list)
6. **Git Interoperability** (2 commands: import, export)
7. **Maintenance** (1 command: repair)
8. **Remote Synchronization** (5 commands: link, fetch, pull, push, sync)
9. **Utilities** (1 command: completion)

---

## KEY INTEGRATION TEST SCENARIOS

### Scenario 1: Core Workflow - Mode Configuration
**Workflow**: init → mode create → add → commit → apply
```bash
# Precondition: Empty project
1. jin init
2. jin mode create claude
3. jin mode use claude
4. Create test file: .claude/config.json
5. jin add .claude/config.json --mode
6. jin commit -m "Add claude config"
7. jin apply
8. Verify: file exists in workspace at .claude/config.json
9. jin log --layer mode-base
```

**Expected State Changes**:
- .jin/ directory created with context
- Mode ref created: refs/jin/modes/claude
- File blob created in Jin repo
- Layer commit created
- Workspace file written

**Error Scenarios to Test**:
- Mode already exists
- File doesn't exist before add
- No staged changes before commit
- Workspace dirty before apply

---

### Scenario 2: Mode + Scope Workflow
**Workflow**: mode create → scope create → layer routing → apply
```bash
1. jin mode create builder
2. jin scope create env:prod --mode builder
3. jin mode use builder
4. jin scope use env:prod
5. Create file1.json, file2.json
6. jin add file1.json --mode
7. jin add file2.json --mode --scope env:prod
8. jin commit -m "Add configs"
9. jin apply
10. Verify: Both files applied with correct precedence
```

**Expected Behavior**:
- file2.json (mode+scope) overrides file1.json (mode) if same path
- Both files present if different paths
- Merge respects layer precedence

---

### Scenario 3: Reset Operations
**Workflow**: add → reset (soft/mixed/hard)
```bash
1. jin init
2. Create test.txt
3. jin add test.txt
4. jin status → shows staged
5. jin reset --soft
6. Verify: Still in staging
7. jin reset --mixed (default)
8. Verify: Removed from staging, file still in workspace
9. jin add test.txt again
10. jin reset --hard
11. Verify: Removed from staging AND workspace
```

---

### Scenario 4: Git Import/Export Workflow
**Workflow**: import from Git → manage in Jin → export back to Git
```bash
1. jin init in existing Git repo
2. echo "data" > tracked.txt
3. git add tracked.txt
4. git commit -m "Add tracked file"
5. jin import tracked.txt
6. Verify: removed from Git index, added to Jin staging
7. jin commit -m "Move to Jin"
8. jin apply
9. Create new.txt
10. jin add new.txt
11. jin commit -m "Add new file"
12. jin export tracked.txt
13. Verify: File back in Git staging
14. git status → shows staged tracked.txt
15. git commit -m "Restore to Git"
```

**Error Scenarios**:
- Import file not tracked by Git
- Import modified file without --force
- Export file not Jin-tracked

---

### Scenario 5: Remote Sync Workflow
**Workflow**: link → fetch → pull → push → sync
```bash
# Setup: Create bare Git repo for remote
1. git init --bare /tmp/jin-remote.git
2. jin init
3. jin link file:///tmp/jin-remote.git
4. jin mode create dev
5. jin mode use dev
6. Create config.yaml
7. jin add config.yaml --mode
8. jin commit -m "Add dev config"
9. jin push
10. Verify: Remote has refs/jin/layers/mode/dev

# On second machine/directory
11. cd /tmp/project2
12. jin init
13. jin link file:///tmp/jin-remote.git
14. jin fetch
15. Verify: Shows available updates
16. jin pull
17. jin mode use dev
18. jin apply
19. Verify: config.yaml present in workspace

# Full sync shortcut
20. cd /tmp/project3
21. jin init
22. jin link file:///tmp/jin-remote.git
23. jin sync
24. jin mode use dev
25. Verify: All configs synced
```

---

### Scenario 6: Repair Workflow
**Workflow**: detect corruption → repair
```bash
1. jin init
2. jin add test.txt
3. Manually corrupt .jin/staging/index.json (invalid JSON)
4. jin repair --dry-run
5. Verify: Reports corruption detected
6. jin repair
7. Verify: Staging index recreated (loses staged changes)
8. jin repair --dry-run
9. Verify: No issues found
```

---

## COMMAND DEPENDENCIES

### Dependency Graph
```
init (no deps)
  ├── mode create (init optional)
  │   └── mode use (requires mode exists)
  │       └── add --mode (requires active mode)
  ├── scope create (init optional)
  │   └── scope use (requires scope exists)
  │       └── add --scope (requires scope exists)
  ├── add (requires init)
  │   └── commit (requires staged files)
  │       └── apply (requires commits)
  ├── import (requires init + Git repo)
  │   └── commit
  ├── link (init optional)
  │   ├── fetch (requires link)
  │   │   └── pull (requires fetch, implicitly calls it)
  │   └── push (requires link)
  └── sync (requires link, calls fetch+pull+apply)
```

---

## ERROR CONDITIONS TO TEST

### File Validation Errors
- Symlink provided to add/import
- Directory provided to add
- Non-existent file
- Git-tracked file to add
- Non-Git-tracked file to import

### State Errors
- Jin not initialized
- No active mode when --mode flag used
- Mode/scope doesn't exist
- Workspace dirty when pull/apply without --force
- No staged changes when commit
- File already Jin-tracked when import
- File not Jin-tracked when export

### Network/Remote Errors
- Invalid remote URL format
- Remote not configured when fetch/pull/push
- Authentication failure
- Network timeout
- Non-fast-forward push without --force

### Corruption Errors
- Invalid JSON in staging index
- Invalid YAML in context
- Missing layer refs
- Orphaned blobs

---

## STATE FILES TO VERIFY

### After Each Operation, Verify:

| File/Location | Commands that Modify | What to Check |
|---------------|----------------------|---------------|
| .jin/context.json | mode use/unset, scope use/unset | mode/scope fields |
| .jin/staging/index.json | add, import, export, commit, reset | entries array |
| ~/.jin/ (Git repo) | All layer ops | refs, commits, blobs |
| ~/.jin/config.toml | link | remote URL |
| .gitignore | add, import, export, apply | managed block |
| workspace files | apply, reset --hard | file content |

---

## LAYER ROUTING TEST MATRIX

| Flags | Expected Layer | Git Ref |
|-------|----------------|---------|
| (none) | ProjectBase | refs/jin/layers/project/<project> |
| --global | GlobalBase | refs/jin/layers/global |
| --mode | ModeBase | refs/jin/layers/mode/<name> |
| --mode --project | ModeProject | refs/jin/layers/mode/<name>/project/<project> |
| --mode --scope=X | ModeScope | refs/jin/layers/mode/<name>/scope/<X> |
| --mode --scope=X --project | ModeScopeProject | refs/jin/layers/mode/<name>/scope/<X>/project/<project> |
| --scope=X | ScopeBase | refs/jin/layers/scope/<X> |

### Test Cases for Each Routing:
1. Add file with flags
2. Verify blob created
3. Verify staging entry has correct layer
4. Commit
5. Verify commit in correct ref
6. Reset from that layer
7. Verify removed from staging

---

## MERGE PRECEDENCE TESTS

### Test Scenario: File in Multiple Layers
```bash
1. Create base.json with {"key": "global"}
2. jin add base.json --global
3. jin commit -m "Global base"

4. Create base.json with {"key": "mode", "mode_key": "value"}
5. jin mode create test
6. jin mode use test
7. jin add base.json --mode
8. jin commit -m "Mode override"

9. jin apply
10. Read base.json
11. Verify: {"key": "mode", "mode_key": "value"}
    - "key" overridden by mode layer (higher precedence)
    - "mode_key" added by mode layer
```

---

## MULTI-COMMAND SEQUENCES

### Sequence 1: Complete Development Workflow
```
init → mode create → mode use → add → commit → status → log → apply
```

### Sequence 2: Collaboration Workflow
```
link → fetch → pull → apply → (make changes) → add → commit → push
```

### Sequence 3: Error Recovery Workflow
```
add → (realize mistake) → reset → (fix) → add → commit
```

### Sequence 4: Layer Management Workflow
```
mode create → scope create --mode → mode use → scope use →
add → commit → apply → layers → context
```

---

This analysis provides comprehensive coverage for designing an end-to-end integration test suite for Jin.
