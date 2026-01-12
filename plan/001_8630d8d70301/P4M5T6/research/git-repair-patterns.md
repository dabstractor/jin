# Git Repository Repair and Recovery Patterns Research

## Overview

This document researches Git repository repair and recovery patterns, covering corruption scenarios, reflog recovery, integrity checking, and repair operations. The research is based on official Git documentation, community resources, and real-world case studies.

## 1. Common Git Corruption Scenarios

### Corruption Causes
1. **Incomplete Operations**
   - Interrupted clones or fetches
   - Half-written packfiles during network operations
   - Premature termination of `git gc` or `git repack`

2. **Filesystem Issues**
   - Disk corruption or hardware failures
   - Storage quota exceeded during operations
   - Improperly terminated filesystem operations

3. **Repository Structure Issues**
   - Missing or corrupted `.git/` directory components
   - Broken packfiles or index files
   - Corrupted loose objects

4. **Import/Migration Issues**
   - Incomplete imports from other version control systems
   - Malformed history during repository conversion

### Common Error Patterns
- Missing objects (`error: object file .git/objects/xx/xxxx... is empty`)
- Packfile corruption (`Packfile checksum mismatch`)
- Index corruption (`invalid index entry`)
- Reference corruption (`error: refname not found`)

## 2. Git Reflog Recovery Patterns

### What is Git Reflog?
The Git reflog (reference log) maintains a record of where HEAD and references have been. It's stored in `.git/logs/` and can be used to recover lost commits and branch states.

### Key Reflog Recovery Commands

```bash
# View reflog history
git reflog

# View reflog with log format
git log -g

# View specific reflog entry
git log -g HEAD@{2}

# Create recovery branch from reflog
git branch recovery-branch HEAD@{n}

# Reset to specific reflog entry
git reset --hard HEAD@{n}
```

### Recovery Scenarios

#### Scenario 1: Accidental Hard Reset
```bash
# After hard reset that lost commits
git reflog                    # Find lost commit
git branch recovery HEAD@{n}  # Create recovery branch
git merge recovery            # Recover changes
```

#### Scenario 2: Deleted Branch Recovery
```bash
# List all reflog entries for branches
git reflog --all

# Recover deleted branch
git checkout -b branch-name HEAD@{n}
```

#### Scenario 3: Messed Up Rebase Recovery
```bash
# Find pre-rebase state in reflog
git reflog | grep "rebase finished"

# Reset to pre-rebase state
git reset --hard HEAD@{n}

# Or recover lost commits
git cherry-pick <commit-hash>
```

### Reflog Best Practices
1. **Regular Backup**: Periodically commit or backup important reflog states
2. **Avoid Manual Deletion**: Never manually edit `.git/logs/`
3. **Aware of GC**: `git gc` may prune old reflog entries
4. **Use `--date=iso`**: Better timestamp readability in reflog

## 3. Repository Integrity Checking with `git fsck`

### Basic `git fsck` Usage
```bash
# Basic integrity check
git fsck

# Full check including all objects
git fsck --full

# Check with detailed output
git fsck --verbose

# Check specific objects
git fsck <object-hash>
```

### Common `git fsck` Error Types

#### Dangling Objects
```
dangling commit ab1afef80fac8e34258ff41fc1b867c702daa24b
dangling blob d670460b4b4aece5915caf5c68d12f560a9fe3e4
dangling tree aea790b9a58f6cf6f2804eeac9f0abbe9631e4c9
```
These objects exist but aren't referenced by anything. They may be recoverable.

#### Corruption Errors
```
error: object file .git/objects/xx/xxxx... is empty
error: inflate: data corruption
error: packfile checksum mismatch
```
These indicate serious corruption requiring intervention.

#### Reference Errors
```
error: refs/heads/master: invalid object name ab1afef
error: bad reflog entry SHA1
```
Issues with references or reflog entries.

### Recovery with `git fsck`
```bash
# Find dangling commits
git fsck --full | grep "dangling commit"

# Create branch from dangling commit
git branch recovery <commit-hash>

# Check for specific error types
git fsck --full --lost-found
```

## 4. Repair Operations in Git Utilities

### Built-in Repair Tools

#### `git gc` (Garbage Collection)
```bash
# Automatic garbage collection
git gc

# Force garbage collection
git gc --aggressive

# Auto gc with custom limits
git config gc.auto 7000
git config gc.autopacklimit 50
```

#### `git prune`
```bash
# Prune unreachable objects older than 2 weeks
git prune

# Prune specific timestamp
git prune --expire="2 weeks ago"

# Prune packed objects
git prune-packed
```

#### `git repack`
```bash
# Repack all objects
git repack -a -d

# Create packs of maximum size
git repack -a --max-pack-size=100M
```

### Third-Party Repair Tools

#### `git-repair`
```bash
# Automated repair tool
git-repair

# Check before repair
git-repair --check
```

#### Clone-Based Recovery
```bash
# Clone to new repository
git clone --mirror /path/to/broken-repo new-repo

# Add as alternate
git config core.alternates /path/to/broken-repo/objects

# Lift objects from alternate
git repack -a -d
```

### Manual Repair Patterns

#### Pattern 1: Object Recovery from Alternate
```bash
# Add known good repository as alternate
git config core.alternates /path/to/good-repo/objects

# Fetch missing objects
git fetch -p

# Remove alternate after recovery
git config --unset core.alternates
```

#### Pattern 2: Packfile Repair
```bash
# Verify and recreate packfiles
git verify-pack -v .git/objects/pack/pack-*.idx
git repack -a -f
```

#### Pattern 3: Reference Reconstruction
```bash
# Recover packed-refs
git pack-refs --all

# Verify and rebuild HEAD
git symbolic-ref HEAD refs/heads/main
```

## 5. Atomic Repair Operations Best Practices

### Principles of Atomic Operations
1. **All-or-Nothing**: Either complete the entire operation or fail completely
2. **No Partial Updates**: Prevent intermediate states that could corrupt the repository
3. **Rollback Capability**: Ability to revert to previous state if operation fails

### Atomic Patterns

#### Pattern 1: Backup Before Repair
```bash
# Create timestamped backup
git clone --mirror /path/to/repo /backup/repo-$(date +%Y%m%d-%H%M%S)

# Or create packfile backup
git repack -a -d
git prune --expire=now
```

#### Pattern 2: Staged Repair
```bash
# Create test branch first
git checkout -b test-branch

# Perform repair operations
git fsck --full
git gc --aggressive

# Verify before merging
git checkout main
git merge test-branch --no-ff
```

#### Pattern 3: Transaction-like Operations
```bash
# Use detached HEAD for repair work
git checkout --detach

# Perform repair
git fsck --full
git repack -a -d

# Only if successful, update main branch
git checkout main
git merge --ff-only HEAD
```

### Safety Considerations
1. **Always Work on Copies**: Never repair original repositories directly
2. **Verify After Each Step**: Check repository integrity after each operation
3. **Restrict Access**: Prevent other users from accessing during repair
4. **Document Changes**: Keep log of all repair operations performed

## 6. Bare Repository Specific Repair Considerations

### Special Considerations for Bare Repositories
1. **No Working Directory**: Cannot checkout files for verification
2. **Central Storage**: Often shared among multiple users
3. **No .git Directory**: Repository root is the .git directory itself
4. **Network Access**: Often requires network connectivity for recovery

### Bare Repository Repair Patterns

#### Pattern 1: Remote-Based Recovery
```bash
# Use remote clone to recover
git clone --mirror ssh://user@host:/path/to/bare-repo recovery-repo

# Push back to original
cd recovery-repo
git push --mirror ssh://user@host:/path/to/original-bare-repo
```

#### Pattern 2: Fetch-Only Recovery
```bash
# Try to fetch from remotes to restore objects
git remote add temp-origin /path/to/remote-repo
git fetch -p temp-origin
git remote remove temp-origin
```

#### Pattern 3: Packfile-Only Repair
```bash
# Focus on packfile integrity
git verify-pack -v *.idx

# Rebuild packfiles
git repack -a -f

# Update packed-refs
git pack-refs --all --prune
```

### Bare Repository Maintenance
```bash
# Regular maintenance for bare repos
git gc --aggressive
git prune --expire=now
git update-server-info

# Check permissions
chmod -R g+rwX .git/
```

## 7. Comprehensive Repair Workflow

### Step-by-Step Recovery Process

1. **Assessment Phase**
   ```bash
   # Create backup
   git clone --mirror /path/to/repo /backup/repo-$(date +%Y%m%d)

   # Diagnose issues
   git fsck --full
   git count-objects -v
   ```

2. **Recovery Phase**
   ```bash
   # Try basic repairs
   git gc
   git prune --expire=now

   # Check for recoverable objects
   git fsck --full --lost-found
   ```

3. **Verification Phase**
   ```bash
   # Run comprehensive checks
   git fsck --full
   git log --graph --all
   git branch -a
   ```

4. **Cleanup Phase**
   ```bash
   # Remove backup references
   rm -rf .git/refs/original
   rm -rf .git/logs/

   # Final pack
   git repack -a -d
   ```

### Emergency Recovery Script
```bash
#!/bin/bash
# Emergency Git repository recovery

REPO_DIR="$1"
BACKUP_DIR="${REPO_DIR}-backup-$(date +%Y%m%d-%H%M%S)"

echo "Creating backup at $BACKUP_DIR..."
git clone --mirror "$REPO_DIR" "$BACKUP_DIR"

echo "Running fsck..."
cd "$REPO_DIR"
git fsck --full

echo "Attempting recovery..."
git gc --aggressive
git prune --expire=now
git repack -a -d

echo "Verifying repository..."
git fsck --full

if [ $? -eq 0 ]; then
    echo "Recovery successful!"
else
    echo "Recovery failed. Restoring from backup..."
    rm -rf "$REPO_DIR"
    mv "$BACKUP_DIR" "$REPO_DIR"
fi
```

## 8. Resources and References

### Official Documentation
- [Git - Maintenance and Data Recovery](https://git-scm.com/book/en/v2/Git-Internals-Maintenance-and-Data-Recovery)
- [git-fsck Documentation](https://git-scm.com/docs/git-fsck)

### Community Resources
- [Recovering lost commits with git reflog - Graphite](https://graphite.com/guides/recovering-lost-commits-git-reflog)
- [Git Reflogs: A Guide to Rescuing Your Lost Work - CodeMiner42](https://blog.codeminer42.com/git-reflogs-a-guide-to-rescuing-your-lost-work/)
- [How to recover lost commits in Git - GeeksforGeeks](https://www.geeksforgeeks.org/git/recovering-lost-commits-in-git/)

### Specific Guides
- [Recover A Corrupt Git Bare Repository - REWOO Blog](https://rewoo.wordpress.com/2012/02/14/recover-a-corrupt-git-bare-repository/)
- [Git - Recovering from bare Remote Repository corruptions](https://www.rahulsingla.com/blog/2024/01/git-recovering-from-bare-remote-repository-corruptions/)
- [Repairing a corrupt Git repo using a clone](https://edofic.com/posts/2016-02-24-git-repair/)

### Stack Overflow Discussions
- [Repair corrupted Git repository](https://stackoverflow.com/questions/8271263/repair-corrupted-git-repository)
- [How can I recover a lost commit in Git?](https://stackoverflow.com/questions/10099258/how-can-i-recover-a-lost-commit-in-git)
- [What can I do with Git corruption due to a missing object?](https://stackoverflow.com/questions/4929674/what-can-i-do-with-git-corruption-due-to-a-missing-object)

## 9. Prevention Strategies

### Regular Maintenance
- Schedule regular `git gc` operations
- Monitor repository size with `git count-objects`
- Keep backups of important repositories

### Best Practices
- Avoid force operations when possible
- Use `--dry-run` before destructive operations
- Keep reflog entries longer by adjusting `gc.reflogExpire`

### Monitoring
- Set up scripts to regularly run `git fsck`
- Monitor repository size growth
- Check for unusual error patterns

---

This research document provides a comprehensive overview of Git repository repair and recovery patterns, combining official documentation with practical examples and community wisdom. Always remember to test recovery procedures in safe environments before applying them to critical repositories.