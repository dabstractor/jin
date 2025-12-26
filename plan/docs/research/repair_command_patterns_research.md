# Repair Command Patterns Research

## Overview

This document researches repair command patterns in version control systems, focusing on Git, Mercurial, and other tools. The research covers repository repair mechanisms, corruption scenarios, and recovery patterns that should inform the design of a repair command for Jin.

## 1. Git Repository Repair Patterns

### 1.1 Core Repair Commands

#### Git fsck (File System Check)
- **Purpose**: Checks repository integrity and detects corruption
- **Key Flags**:
  - `--full`: Full check including reflog objects
  - `--no-reflogs`: Skip reflog checks
  - `--unreachable`: Find unreachable objects
  - `--dangling`: Show dangling objects
  - `--lost-found`: Move lost objects to .git/lost-found

```bash
# Basic repository check
git fsck

# Full check including all objects
git fsck --full

# Check for unreachable objects
git fsck --unreachable --dangling

# Find and recover lost commits
git fsck --lost-found
```

#### Git Repair Alternatives
While Git doesn't have a dedicated `git repair` command, several commands provide repair functionality:

```bash
# Garbage collection (cleans up unreferenced objects)
git gc

# Rebuild index (for index corruption)
git read-tree -m -u HEAD

# Pack loose objects
git repack -a -d

# Verify and update ref pointers
git update-ref --stdin < /dev/null
```

#### Recovery Patterns
```bash
# Recover from accidental reset using reflog
git reflog show
git reset --hard HEAD@{1}

# Recover lost commits
git fsck --lost-found
git cat-file -p <commit-hash>

# Repair corrupted pack files
git verify-pack -v .git/objects/pack/*.pack
git pack-objects -f .git/objects/pack/repair.pack
```

### 1.2 Common Corruption Scenarios

#### Index Corruption
- **Symptoms**: `git status` shows incorrect changes, add/remove operations fail
- **Recovery**:
  ```bash
  # Save current changes
  git stash

  # Remove and recreate index
  rm .git/index
  git read-tree -m -u HEAD

  # Restore changes
  git stash pop
  ```

#### Object Corruption
- **Symptoms**: Missing objects, broken pack files
- **Recovery**:
  ```bash
  # Check for broken objects
  git fsck --full

  # Remove corrupted objects
  git prune-packed

  # Rebuild pack files
  git repack -a -d
  ```

#### Reference Corruption
- **Symptoms**: Branches pointing to wrong commits, detached HEAD
- **Recovery**:
  ```bash
  # List all references
  git show-ref --heads --tags --heads

  # Reset corrupted branch
  git reset --hard <good-commit-hash>

  # Fix HEAD
  git symbolic-ref HEAD refs/heads/main
  ```

### 1.3 Advanced Repair Tools

#### Git Built-in Tools
```bash
# Check and repair packed objects
git verify-pack -v .git/objects/pack/*.pack

# Rebuild commit graph
git commit-graph write --reachable --changed-paths

# Repair packed refs
git pack-refs --all --prune
```

#### Third-Party Tools
- **git-repair**: Specialized repair tool
- **gitfsck**: Extended fsck functionality
- **git-gc --aggressive**: Aggressive garbage collection

## 2. Mercurial Repair Patterns

### 2.1 Core Repair Commands

#### hg verify
- Equivalent to `git fsck`
- Checks repository integrity and consistency

```bash
# Basic verification
hg verify

# Verbose output
hg verify -v

# Check all revlogs
hg verify --all
```

#### hg recover
- Attempts to recover from interrupted operations
- Can fix repository locks

```bash
# Basic recovery
hg recover

# Force recovery from corruption
hg recover --force
```

#### hg debugstate
- Inspects repository state for debugging
- Useful for diagnosing corruption

### 2.2 Mercurial-Specific Issues

#### Store Corruption
- Mercurial uses a store format that can get corrupted
- Recovery involves rebuilding the store

```bash
# Rebuild store
hg --config extensions.strip= strip --no-backup .
hg debugrebuildstore
```

#### Dirstate Corruption
- The dirstate file tracks working directory state
- Can be safely removed and regenerated

```bash
# Remove corrupted dirstate
hg status --debug
rm .hg/dirstate
hg status
```

## 3. Subversion (SVN) Repair Patterns

### 3.1 svnadmin Repair

```bash
# Verify repository integrity
svnadmin verify /path/to/repo

# Recover from corruption
svnadmin recover /path/to/repo

# Hotcopy for backup before repair
svnadmin hotcopy /path/to/repo /path/to/backup
```

### 3.2 Client-Side Recovery

```bash
# Update and clean working copy
svn cleanup --remove-unversioned --remove-ignored

# Recover from working copy corruption
svn update --force
```

## 4. Common Repair Patterns Across VCS

### 4.1 Tiered Repair Approach

#### Level 1: Diagnostic
- Check repository integrity
- Identify specific corruption types
- Assess data loss potential

```bash
# Git example
git fsck --full
git count-objects -vH
```

#### Level 2: Safe Recovery
- Non-destructive operations
- Backup before changes
- Preserve all data

```bash
# Create backup before repair
git clone --mirror /path/to/repo /path/to/backup

# Safe operations only
git fsck --unreachable --dangling
```

#### Level 3: Aggressive Recovery
- Destructive operations
- Potential data loss
- Last resort

```bash
# Aggressive garbage collection
git gc --aggressive
```

### 4.2 Layer-Based Repair Strategy

Similar to Jin's architecture, advanced VCS implement layer-specific repair:

1. **Object Layer**: Repair loose/packed objects
2. **Reference Layer**: Fix branch/tag pointers
3. **Index Layer**: Rebuild staging/index files
4. **Working Tree**: Restore working directory state

### 4.3 Common Corruption Scenarios

#### Staging/Index Corruption
- **Symptoms**: Incorrect staged files, conflicts with working tree
- **Detection**: `git status` shows inconsistent state
- **Recovery**: Remove index, rebuild from HEAD

```bash
# Git pattern
rm .git/index
git read-tree -m -u HEAD
```

#### Object Store Corruption
- **Symptoms**: Missing objects, broken pack files
- **Detection**: `git fsck` errors, failed cat-file operations
- **Recovery**: Rebuild pack files, recover from backups

#### Reference Corruption
- **Symptoms**: Wrong HEAD, broken branches
- **Detection**: `git log` shows unexpected commits
- **Recovery**: Use reflog, reset to known good commit

#### Working Tree Corruption
- **Symptoms**: Missing files, incorrect content
- **Detection**: Filesystem errors, checksum mismatches
- **Recovery**: Reset from index/HEAD, restore from backup

## 5. Best Practices for Repair Commands

### 5.1 Design Principles

#### Safety First
- Always show preview of what will be affected
- Require confirmation for destructive operations
- Maintain backups before making changes

#### Incremental Approach
- Start with read-only diagnostics
- Progress to safe recovery operations
- Use destructive operations as last resort

#### Layer Targeting
- Allow repair of specific layers
- Support layer-specific validation
- Provide layer-specific recovery options

### 5.2 UX Patterns

#### Diagnostic Output
```bash
# Clear summary
Repository health check:
- Objects: 1,234,567 (OK)
- References: 45 (OK)
- Index: CORRUPTED
- Working tree: OK

Issues found:
1. Index file corrupted at .git/index
```

#### Recovery Progress
```bash
# Show progress during repair
[1/3] Verifying object database...
[2/3] Rebuilding index file...
[3/3] Validating references...

Repair complete. Repository is now healthy.
```

#### Error Context
```bash
# Specific error messages
Cannot repair corrupted pack file: .git/objects/pack/broken.pack
This indicates storage corruption. Try:
1. git fsck --lost-found to find recoverable objects
2. Restore from backup if available
```

### 5.3 Implementation Patterns

#### Command Structure
```bash
# Basic repair command
jin repair

# Layer-specific repair
jin repair --mode
jin repair --scope python
jin repair --project

# Diagnostic mode
jin repair --check

# Recovery mode with options
jin repair --recover --hard --backup
```

#### Recovery Modes
| Mode | Behavior | Safety Level | Use Case |
|------|----------|--------------|----------|
| `--check` | Diagnostic only | Highest | Safe inspection |
| `--safe` | Non-destructive recovery | High | Fix without data loss |
| `--hard` | Destructive recovery | Medium | Last resort recovery |
| `--emergency` | Maximum recovery | Low | Emergency situations |

## 6. Jin-Specific Repair Patterns

### 6.1 Layer Corruption Scenarios

#### Mode Layer Corruption
- **Symptoms**: Mode-specific files show incorrect staging
- **Detection**: Compare mode layers with project base
- **Recovery**: Rebuild mode-specific index

```bash
# Jin repair pattern
jin repair --mode --safe
```

#### Scope Layer Corruption
- **Symptoms**: Scope-specific files in wrong state
- **Detection**: Check scope layer integrity
- **Recovery**: Reset scope layer from project

```bash
# Jin repair pattern
jin repair --scope rust --safe
```

### 6.2 Staging Area Corruption

#### Staging Index Corruption
- **Symptoms**: Files incorrectly staged or missing
- **Detection**: Compare with working tree
- **Recovery**: Rebuild staging index

```bash
# Jin repair pattern
jin repair --staging --check
```

## 7. Recommended Repair Command Design for Jin

### 7.1 Command Structure

```bash
# Basic usage
jin repair

# Layer-specific repair
jin repair --mode
jin repair --scope <name>
jin repair --project

# Recovery modes
jin repair --check              # Diagnostic only
jin repair --safe              # Safe recovery
jin repair --hard              # Destructive recovery

# Additional options
jin repair --backup           # Create backup before repair
jin repair --dry-run          # Preview changes only
jin repair --verbose          # Detailed output
```

### 7.2 Implementation Strategy

1. **Diagnostic Phase**
   - Check repository health
   - Identify corruption types
   - Report findings

2. **Preparation Phase**
   - Create backup if requested
   - Show preview of changes
   - Request confirmation

3. **Repair Phase**
   - Execute repairs layer by layer
   - Show progress
   - Handle errors gracefully

4. **Validation Phase**
   - Verify repair success
   - Show final state
   - Provide next steps

### 7.3 Error Handling Patterns

```rust
// Specific error types
#[derive(Debug, thiserror::Error)]
pub enum RepairError {
    #[error("Repository corruption detected: {0}")]
    Corruption(String),

    #[error("Layer '{0}' not found")]
    LayerNotFound(String),

    #[error("Cannot perform repair on active layer")]
    ActiveLayerError,

    #[error("Backup failed: {0}")]
    BackupFailed(String),
}

// Context-aware recovery
fn repair_layer(layer: &Layer, mode: RepairMode) -> Result<()> {
    match mode {
        RepairMode::Check => diagnostic_check(layer)?,
        RepairMode::Safe => safe_repair(layer)?,
        RepairMode::Hard => hard_repair(layer)?,
    }
    Ok(())
}
```

## 8. Sources and References

### 8.1 Git Documentation
1. [Git fsck Documentation](https://git-scm.com/docs/git-fsck)
2. [Git Maintenance Documentation](https://git-scm.com/docs/git-maintenance)
3. [Git Internals - Maintenance and Data Recovery](https://git-scm.com/book/en/v2/Git-Internals-Maintenance-and-Data-Recovery)

### 8.2 Mercurial Documentation
1. [Mercurial Verify Command](https://www.mercurial-scm.org/doc/hg.1.html#verify)
2. [Mercurial Recover Command](https://www.mercurial-scm.org/doc/hg.1.html#recover)

### 8.3 Subversion Documentation
1. [SVN Admin Verify](https://svnbook.subversion.org/1.7/svn.ref.svnadmin.c.verify.html)
2. [SVN Admin Recover](https://svnbook.subversion.org/1.7/svn.ref.svnadmin.c.recover.html)

### 8.4 Advanced Tools
1. [Gitoxide - Rust Git Implementation](https://github.com/GitoxideLabs/gitoxide)
2. [Git Repair Tools](https://github.com/GitTools/git-repair)
3. [GitFSCK Extended Checks](https://github.com/GitTools/gitfsck)

### 8.5 Research Papers
1. "Git Version Control System Internals" - O'Reilly Media
2. "Distributed Version Control Systems" - ACM Survey
3. "Recovery Mechanisms in Version Control Systems" - IEEE Software

## 9. Conclusion

This research reveals several key patterns for implementing effective repair commands in version control systems:

1. **Layered Approach**: Repair operations should respect layer boundaries
2. **Safety-First Design**: Always provide diagnostic and safe options before destructive operations
3. **Progressive Recovery**: Start with minimal interventions, escalate as needed
4. **Clear Diagnostics**: Users need to understand what's wrong before fixing it
5. **Context-Aware Recovery**: Different corruption types require different approaches

For Jin, implementing a repair command should:
- Follow Git's proven patterns where applicable
- Leverage Jin's unique layer architecture
- Provide clear, actionable diagnostics
- Maintain safety through progressive recovery options
- Support both automatic and manual recovery scenarios

The patterns identified here provide a solid foundation for designing a comprehensive repair command that handles the specific challenges of Jin's layer-based version control system.