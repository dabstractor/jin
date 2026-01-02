# Product Requirement Prompt: Repair Command

**Work Item**: P4.M5.T6 - Repair Command
**Status**: Implementation Complete (Reference PRP)
**Implementation**: `src/commands/repair.rs` (887 lines)

---

## Goal

**Feature Goal**: Implement a robust repair command that detects and fixes Jin repository integrity issues across 7 critical areas of state management.

**Deliverable**: Fully functional `jin repair [--dry-run]` command that:
- Scans all Jin state components for corruption
- Reports issues found with clear descriptions
- Repairs issues atomically with backup before modification
- Supports dry-run mode for preview without changes

**Success Definition**:
```bash
# Dry-run shows what would be fixed
$ jin repair --dry-run
Checking repository structure... ✓
Checking layer references... ✗
Checking staging index... ✓
1 issue found (dry run - no changes made)
  Issue: Invalid ref refs/jin/layers/old-mode
    Would attempt recovery from reflog

# Actual repair fixes issues
$ jin repair
Checking repository structure... ✓
Checking layer references... ✗
  Fixed: Recovered refs/jin/layers/old-mode from reflog
Checking staging index... ✓
Repair complete. 1 issue fixed.
```

## User Persona

**Target User**: Jin users experiencing corruption or inconsistency issues

**Use Cases**:
1. After system crashes or interrupted operations
2. When seeing "corrupted" error messages
3. Before major operations to ensure repository health
4. After manual repository manipulation

**Pain Points Addressed**:
- "My staging is corrupted, what do I do?"
- "Jin says invalid layer reference"
- "I killed the process during commit, is everything broken?"
- "I accidentally deleted .jin/context"

## Why

**Business Value**:
- **Reliability**: Users can recover from common corruption scenarios without manual intervention
- **Trust**: Atomic repairs with backups mean users can safely run repair without fear of data loss
- **Debugging**: Clear reporting helps users understand what went wrong

**Integration with Existing Features**:
- Works with transaction recovery system (`RecoveryManager`)
- Cleans up orphan transaction refs from interrupted commits
- Repairs .jinmap which is used by other commands
- Validates layer refs used by merge engine

**Problems Solved**:
1. **Repository Corruption**: Detects and recreates missing/corrupted bare repository
2. **Invalid Layer Refs**: Recovers from reflog or deletes corrupted refs
3. **Staging Corruption**: Rebuilds corrupted staging index (warns about data loss)
4. **JinMap Issues**: Creates/repairs .jinmap file
5. **Workspace Metadata**: Rebuilds corrupted workspace metadata
6. **Config Corruption**: Repairs global config with backup
7. **Context Issues**: Creates/repairs project context file

## What

User-visible behavior:

```bash
# Usage
jin repair [--dry-run]

# Dry-run mode - shows what would be done
$ jin repair --dry-run
Checking Jin repository integrity...

Checking repository structure... ✓
Checking layer references... ✗
  Issue: Invalid ref refs/jin/layers/deleted-mode (object not found)
    Would attempt recovery from reflog
Checking staging index... ✓ (not present)
Checking .jinmap... ✗
  Issue: .jinmap missing
    Would create default .jinmap
Checking workspace metadata... ✓ (not present)
Checking global configuration... ✓
Checking project context... ✓

2 issues found (dry run - no changes made)

# Actual repair
$ jin repair
Checking Jin repository integrity...

Checking repository structure... ✓
Checking layer references... ✗
  Fixed: Deleted invalid ref refs/jin/layers/deleted-mode
Checking staging index... ✓ (not present)
Checking .jinmap... ✗
  Fixed: .jinmap created
Checking workspace metadata... ✓ (not present)
Checking global configuration... ✓
Checking project context... ✓

Repair complete. 2 issues fixed.
```

### Success Criteria

- [ ] All 7 integrity checks pass on healthy repository
- [ ] Detects all corruption types (missing files, invalid refs, corrupted data)
- [ ] Dry-run shows exact changes without making modifications
- [ ] All repairs create `.corrupted` backup before modification
- [ ] Fatal errors provide clear manual recovery instructions
- [ ] Graceful degradation - continues checking even after some repairs fail

---

## All Needed Context

### Context Completeness Check

**No Prior Knowledge Test**: If someone knew nothing about this codebase, they would need:
- ✅ CLI command structure and patterns
- ✅ Git repository operations via git2
- ✅ Layer reference system and ref paths
- ✅ State file locations and formats (staging, context, jinmap)
- ✅ Transaction system for orphan cleanup
- ✅ Test patterns and fixtures

### Documentation & References

```yaml
# MUST READ - Core implementation reference

- file: src/commands/repair.rs
  why: Complete 887-line reference implementation with all 7 checks
  pattern: Full repair command with dry-run, backup, and recovery logic
  critical: This is the reference implementation - PRP documents its design

- file: src/cli/mod.rs
  why: CLI command registration pattern
  section: Commands enum - shows where Repair(RepairArgs) is registered
  pattern: Subcommand pattern with args struct

- file: src/cli/args.rs:134-138
  why: RepairArgs definition
  pattern: #[derive(Args, Debug)] with dry_run: bool flag
  gotcha: dry_run is a flag, not a value

- file: src/commands/mod.rs:47
  why: Command dispatcher wiring
  pattern: Commands::Repair(args) => repair::execute(args)

- file: src/git/repo.rs
  why: JinRepo wrapper for bare repository operations
  pattern: Repository opening/creation, is_bare() validation
  methods: open(), create(), open_or_create(), is_valid()

- file: src/git/refs.rs
  why: RefOps trait for layer reference operations
  methods: list_refs(), resolve_ref(), delete_ref(), set_ref()
  pattern: References under refs/jin/layers/* namespace

- file: src/git/transaction.rs:590-704
  why: RecoveryManager for orphan transaction cleanup
  critical: Repair command cleans orphan refs left by interrupted commits
  methods: RecoveryManager::auto_recover(), detect(), rollback()

- file: src/staging/index.rs
  why: StagingIndex structure and load/save
  pattern: JSON-based index with backup before repair
  methods: load(), save(), default_path()

- file: src/staging/metadata.rs
  why: WorkspaceMetadata for workspace tracking
  pattern: JSON metadata with backup before repair
  methods: load(), save(), default_path()

- file: src/core/config.rs
  why: JinConfig and ProjectContext structures
  pattern: TOML/YAML config files with error handling
  methods: load(), save(), default_path()

- file: plan/docs/COMMANDS.md:675-708
  why: PRD specification for repair command
  critical: Defines the 7 areas to check and user-facing behavior

- docfile: plan/P4M5T6/research/git-repair-patterns.md
  why: External research on Git repair patterns, reflog recovery, fsck
  section: Reflog Recovery Patterns - shows why we recover from reflog

# TESTING REFERENCES

- file: tests/cli_basic.rs
  why: Integration test patterns for commands
  pattern: tempfile::TempDir, JIN_DIR env var, assert_cmd::Command

- file: tests/common/fixtures.rs
  why: Test fixtures and setup helpers
  pattern: TestFixture, RemoteFixture, setup_test_repo()

- file: tests/common/assertions.rs
  why: Custom assertions for Jin state verification
  pattern: assert_workspace_file(), assert_staging_contains()

- file: src/commands/repair.rs:658-886
  why: 12 unit tests covering all repair scenarios
  pattern: Isolated test setup, corrupted state simulation
```

### Current Codebase Tree

```bash
src/
├── cli/
│   ├── mod.rs              # Commands enum with Repair(RepairArgs)
│   └── args.rs             # RepairArgs struct definition
├── commands/
│   ├── mod.rs              # Command dispatcher (repair::execute)
│   └── repair.rs           # Complete implementation (887 lines, 12 tests)
├── git/
│   ├── mod.rs              # Module exports (JinRepo, RefOps, RecoveryManager)
│   ├── repo.rs             # JinRepo wrapper (open, create, is_bare)
│   ├── refs.rs             # RefOps trait (list_refs, resolve_ref, delete_ref)
│   └── transaction.rs      # RecoveryManager for orphan cleanup
├── staging/
│   ├── index.rs            # StagingIndex (load, save, default_path)
│   └── metadata.rs         # WorkspaceMetadata (load, save, default_path)
└── core/
    └── config.rs           # JinConfig, ProjectContext (load, save, default_path)

tests/
├── cli_basic.rs            # Integration test patterns
├── common/
│   ├── fixtures.rs         # TestFixture, setup_test_repo()
│   └── assertions.rs       # Custom Jin assertions

plan/
└── P4M5T6/
    └── PRP.md              # This document
```

### Implementation Blueprint

The repair command is **already fully implemented** in `src/commands/repair.rs`. This PRP documents the design and serves as a reference for understanding the implementation.

#### Data Models

No new data models needed. Repair uses existing types:
- `RepairArgs`: CLI argument struct with `dry_run: bool`
- `JinRepo`: Bare repository wrapper
- `StagingIndex`, `WorkspaceMetadata`: State files
- `JinConfig`, `ProjectContext`: Configuration types
- `RecoveryManager`: Transaction cleanup

#### Implementation Tasks (Reference - Already Complete)

```yaml
Task 1: CLI Argument Definition
  IMPLEMENT: RepairArgs struct with dry_run flag
  LOCATION: src/cli/args.rs:134-138
  PATTERN: #[derive(Args, Debug)] with #[arg(long)] pub dry_run: bool
  STATUS: ✅ Complete

Task 2: Command Registration
  IMPLEMENT: Add Repair variant to Commands enum
  LOCATION: src/cli/mod.rs - Commands enum
  PATTERN: Repair(RepairArgs) variant
  DISPATCHER: src/commands/mod.rs:47 - Commands::Repair(args) => repair::execute(args)
  STATUS: ✅ Complete

Task 3: Main Execute Function
  IMPLEMENT: execute() function with 7 integrity checks
  LOCATION: src/commands/repair.rs:16-124
  PATTERN: Print header, run checks, display summary
  SIGNATURE: pub fn execute(args: RepairArgs) -> Result<()>
  STATUS: ✅ Complete

Task 4: Repository Structure Check
  IMPLEMENT: check_repository_structure()
  LOCATION: src/commands/repair.rs:126-182
  VALIDATES: ~/.jin/ is valid bare repository
  REPAIRS: Recreates if missing (fatal error if not bare)
  STATUS: ✅ Complete

Task 5: Layer References Check
  IMPLEMENT: check_layer_refs() with reflog recovery
  LOCATION: src/commands/repair.rs:184-286
  VALIDATES: All refs/jin/layers/* point to valid commits
  REPAIRS: Recover from reflog, delete if unrecoverable
  CRITICAL: recover_ref_from_reflog() attempts recovery from Git reflog
  STATUS: ✅ Complete

Task 6: Staging Index Check
  IMPLEMENT: check_staging_index()
  LOCATION: src/commands/repair.rs:288-348
  VALIDATES: .jin/staging/index.json is parseable
  REPAIRS: Rebuild with backup to .json.corrupted
  WARNING: Staging changes are lost on rebuild
  STATUS: ✅ Complete

Task 7: JinMap Check
  IMPLEMENT: check_jinmap()
  LOCATION: src/commands/repair.rs:350-446
  VALIDATES: .jin/.jinmap exists and is valid YAML
  REPAIRS: Create default or backup+recreate corrupted
  HELPER: create_default_jinmap(), repair_jinmap()
  STATUS: ✅ Complete

Task 8: Workspace Metadata Check
  IMPLEMENT: check_workspace_metadata()
  LOCATION: src/commands/repair.rs:448-506
  VALIDATES: .jin/workspace/last_applied.json is parseable
  REPAIRS: Rebuild with backup to .json.corrupted
  STATUS: ✅ Complete

Task 9: Global Configuration Check
  IMPLEMENT: check_global_config()
  LOCATION: src/commands/repair.rs:508-565
  VALIDATES: ~/.jin/config.toml is parseable
  REPAIRS: Backup and recreate with defaults
  STATUS: ✅ Complete

Task 10: Project Context Check
  IMPLEMENT: check_project_context()
  LOCATION: src/commands/repair.rs:567-656
  VALIDATES: .jin/context exists and is parseable
  REPAIRS: Create default or backup+recreate corrupted
  STATUS: ✅ Complete

Task 11: Unit Tests
  IMPLEMENT: 12 unit tests in repair.rs module
  LOCATION: src/commands/repair.rs:658-886
  COVERAGE: All check functions with dry-run and actual repair modes
  STATUS: ✅ Complete

Task 12: Integration Tests
  IMPLEMENT: Test repair command in isolated environment
  LOCATION: tests/cli_repair.rs (to be added)
  PATTERN: Use TestFixture, corrupt state, run repair, verify fixed
  STATUS: ⚠️ Optional - unit tests provide good coverage
```

#### Implementation Patterns & Key Details

```rust
// PATTERN: Check function structure
fn check_<component>(
    args: &RepairArgs,
    issues_found: &mut Vec<String>,
    issues_fixed: &mut Vec<String>,
) {
    print!("Checking <component>... ");

    // Validate
    match validation_logic() {
        Ok(_) => println!("✓"),
        Err(_) => {
            println!("✗");
            let issue = "<description>".to_string();
            issues_found.push(issue.clone());

            if !args.dry_run {
                // Repair
                match repair_logic() {
                    Ok(()) => {
                        let fix = "<fix description>".to_string();
                        issues_fixed.push(fix.clone());
                        println!("  Fixed: {}", fix);
                    }
                    Err(e) => {
                        println!("  Failed to repair: {}", e);
                    }
                }
            } else {
                println!("  Issue: {}", issue);
                println!("    Would <repair action>");
            }
        }
    }
}

// CRITICAL: Reflog recovery pattern
fn recover_ref_from_reflog(repo: &JinRepo, ref_name: &str) -> Result<bool> {
    let reflog = repo.inner().reflog(ref_name)?;

    // Find most recent valid commit
    for i in 0..reflog.len() {
        if let Some(entry) = reflog.get(i) {
            let oid = entry.id_new();

            if let Ok(obj) = repo.inner().find_object(oid, None) {
                if obj.kind() == Some(git2::ObjectType::Commit) {
                    // Found valid commit, restore ref
                    repo.set_ref(ref_name, oid, &format!("Recovered from reflog entry {}", i))?;
                    return Ok(true);
                }
            }
        }
    }

    Ok(false) // No valid reflog entry
}

// PATTERN: Backup before repair
fn rebuild_staging_index(index_path: &PathBuf) -> Result<()> {
    // Backup corrupted file
    let backup_path = index_path.with_extension("json.corrupted");
    if index_path.exists() {
        std::fs::rename(index_path, backup_path)?;
    }

    // Create new empty index
    let index = StagingIndex::new();
    index.save()?;
    Ok(())
}

// PATTERN: Dry-run handling
if !args.dry_run {
    // Actually repair
} else {
    // Just report what would be done
    println!("  Would <action>");
}

// GOTCHA: Repository is fatal error if not bare
if !repo.inner().is_bare() {
    eprintln!("Error: Repository at ~/.jin exists but is not a bare repository");
    eprintln!("Manual intervention required.");
    return Err(JinError::Other("Repository is not bare".to_string()));
}

// PATTERN: Graceful degradation
// Continue checking even if one check fails
let repo_result = check_repository_structure(&args, &mut issues_found, &mut issues_fixed);
let repo = repo_result.ok(); // Use Option, continue if None

if let Some(ref repo) = repo {
    check_layer_refs(&args, repo, &mut issues_found, &mut issues_fixed);
}
// Always continue with other checks
```

#### Integration Points

```yaml
TRANSACTION_SYSTEM:
  - integration: "Clean orphan refs from incomplete transactions"
  - module: "src/git/transaction.rs - RecoveryManager"
  - pattern: "Repair cleans refs/refs/jin/transactions/* refs"

GIT_OPERATIONS:
  - dependency: "git2 crate for repository operations"
  - reflog: "git2::Reflog for ref recovery"
  - validation: "git2::ObjectType::Commit check"

STAGING_SYSTEM:
  - state_file: ".jin/staging/index.json"
  - backup_pattern: "*.json.corrupted"

CONFIG_SYSTEM:
  - global: "~/.jin/config.toml"
  - project: ".jin/context"
  - backup_pattern: "*.corrupted"

LAYER_SYSTEM:
  - refs: "refs/jin/layers/*"
  - reflog: "Automatic recovery from Git reflog"
```

---

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# Run after making changes
cargo check --bin jin
cargo clippy --bin jin -- -D warnings
cargo fmt --check

# Expected: Zero errors, zero warnings
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run repair command unit tests
cargo test --bin jin repair::tests -- --nocapture

# Specific test examples
cargo test test_execute_dry_run
cargo test test_check_staging_index_corrupted
cargo test test_recover_ref_from_reflog
cargo test test_rebuild_staging_index

# Expected: All 12 unit tests pass
```

### Level 3: Integration Testing (System Validation)

```bash
# Manual integration test script
#!/bin/bash
set -e

# Setup
TEST_DIR=$(mktemp -d)
export JIN_DIR="$TEST_DIR/.jin_global"
cd "$TEST_DIR"

# Test 1: Repair on healthy repo
jin init
jin repair --dry-run | grep "No issues found"
jin repair | grep "No issues found"

# Test 2: Repair corrupted staging
echo "invalid json" > .jin/staging/index.json
jin repair --dry-run | grep "corrupted"
jin repair | grep "rebuilt"
jin repair | grep "No issues found"

# Test 3: Repair invalid ref
# (Requires creating a ref pointing to non-existent commit)

# Cleanup
rm -rf "$TEST_DIR"
```

### Level 4: Creative & Domain-Specific Validation

```bash
# Corruption scenario testing

# Scenario 1: Interrupted commit leaves orphan transaction
jin init
echo "test" > config.json
jin add config.json
# Kill process during commit
kill -9 $(pgrep -f "jin commit")
jin repair  # Should clean orphan transaction refs

# Scenario 2: Corrupted reflog recovery
# Create ref, delete commit, verify reflog recovery

# Scenario 3: Missing repository
rm -rf ~/.jin
jin repair  # Should recreate repository

# Scenario 4: Corrupted .jinmap YAML
echo "invalid: [yaml" > .jin/.jinmap
jin repair  # Should backup and recreate
```

---

## Final Validation Checklist

### Technical Validation

- [ ] All 7 integrity checks implemented
- [ ] Dry-run mode works without modifications
- [ ] All repairs create `.corrupted` backup
- [ ] Reflog recovery attempts before deletion
- [ ] Fatal errors provide manual recovery steps
- [ ] All unit tests pass (12 tests)
- [ ] Zero clippy warnings
- [ ] Proper error handling with `JinError` types

### Feature Validation

- [ ] Dry-run shows exact issues without changes
- [ ] Actual repair fixes all detected issues
- [ ] Graceful degradation (continues after failures)
- [ ] Clear user feedback with ✓/✗ indicators
- [ ] Summary shows issues found vs fixed
- [ ] Backup files created with `.corrupted` extension

### Code Quality Validation

- [ ] Follows existing CLI command patterns
- [ ] Consistent error handling across all checks
- [ ] Proper use of `JinRepo`, `RefOps`, and `RecoveryManager`
- [ ] All helper functions are private
- [ ] Integration with transaction system for orphan cleanup

### Documentation & Deployment

- [ ] COMMANDS.md documentation matches implementation
- [ ] Help text is clear
- [ ] Dry-run behavior is documented
- [ ] Manual recovery steps provided for fatal errors

---

## Anti-Patterns to Avoid

- ❌ **Don't skip backup before repair** - Always create `.corrupted` backup
- ❌ **Don't stop on first error** - Continue checking all components
- ❌ **Don't ignore reflog recovery** - Attempt recovery before deletion
- ❌ **Don't hardcode paths** - Use `default_path()` methods
- ❌ **Don't use unwrap() in repair logic** - Handle errors gracefully
- ❌ **Don't recreate non-bare repos** - Fatal error, manual recovery required
- ❌ **Don't repair without dry-run option** - Users need preview capability

---

## Implementation Status

**This PRP documents a completed implementation.**

- ✅ Command implementation: 887 lines in `src/commands/repair.rs`
- ✅ CLI arguments: `RepairArgs` with `--dry-run` flag
- ✅ Command registration: Wired in `mod.rs`
- ✅ Unit tests: 12 tests covering all scenarios
- ✅ Documentation: COMMANDS.md specification

**Key Design Decisions Made**:

1. **Seven distinct checks** - Each component verified independently
2. **Backup-first approach** - All repairs create `.corrupted` backups
3. **Reflog recovery** - Attempt recovery from Git reflog before deleting refs
4. **Graceful degradation** - Continue checking even if some repairs fail
5. **Dry-run mode** - Preview without making changes
6. **Clear reporting** - ✓/✗ indicators with detailed issue descriptions

**Confidence Score**: 10/10 - Implementation is complete, tested, and documented.

---

## References

- **Implementation**: `src/commands/repair.rs`
- **PRD Specification**: `plan/docs/COMMANDS.md:675-708`
- **External Research**: `plan/P4M5T6/research/git-repair-patterns.md`
- **Related Systems**:
  - Transaction recovery: `src/git/transaction.rs`
  - Layer references: `src/git/refs.rs`
  - State management: `src/staging/`, `src/core/config.rs`
