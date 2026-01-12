# Reset Command Research

## Research Sources

This document synthesizes research from:
- Git's `git reset` command (all modes)
- Git's `git restore` vs `git reset` (modern patterns)
- Workspace reset strategies
- Staging area vs working directory reset
- Safe reset practices
- CLI design patterns for destructive operations

## Git Reset - Three Core Modes

Git reset manipulates three "trees" in a specific order:

### --soft: Moves HEAD only
- **Effect:** Changes remain staged ("Changes to be committed")
- **Safety:** Safe - no file modifications
- **Use case:** Undo a commit while keeping changes staged
- **Example:** `git reset --soft HEAD^` to undo last commit

### --mixed (default): Resets HEAD and index
- **Effect:** Changes preserved but unstaged
- **Safety:** Safe - working files preserved
- **Use case:** Undo commits and unstage changes while keeping modifications
- **Example:** `git reset HEAD~2` to undo last 2 commits

### --hard: Resets HEAD, index, and working tree
- **Effect:** All changes permanently destroyed
- **Safety:** DESTRUCTIVE - "the only way to make reset dangerous"
- **Warning:** "One of the very few cases where Git will actually destroy data"
- **Use case:** Discard all changes and revert to previous commit

## Three-Step Reset Process

From the official Git book:

1. **Move HEAD** (--soft stops here)
2. **Update Index** (--mixed stops here - default)
3. **Update Working Directory** (--hard continues)

## Modern Pattern: git restore vs git reset

**Introduced in Git 2.23.0** - Clearer separation of concerns:

**git restore:**
- Purpose: Restore files in working tree
- Scope: File-level operations
- Branch impact: Does NOT update branch pointer
- Modern alternative for: `git checkout -- <file>`

**git reset:**
- Purpose: Update branch, moving tip to add/remove commits
- Scope: Branch-level operations
- Branch impact: Updates the branch pointer

**Best practice:** Use `git restore` for file-level ops, `git reset` for branch-level ops.

### Restore Command Options

**Unstage files (modern):**
```bash
git restore --staged file.txt
```

**Discard working tree changes:**
```bash
git restore file.txt
```

**Restore from specific commit:**
```bash
git restore --source=master~2 Makefile
```

## Workspace Preservation Strategies

### Git Stash - Primary Preservation Tool

Quote: "git stash temporarily shelves changes you've made to your working copy"

**Core stash commands:**
```bash
git stash push                    # Save all changes
git stash push -u                 # Include untracked files
git stash push --keep-index       # Keep staged changes
git stash push --staged           # Stash only staged changes
git stash list                    # View all stashes
git stash pop                     # Apply and remove stash
git stash apply stash@{1}         # Apply specific stash
```

### Discard Strategies

**Soft reset (preserve all changes):**
```bash
git reset --soft HEAD~1  # Changes remain staged
```

**Mixed reset (preserve as unstaged):**
```bash
git reset HEAD~1  # Changes in working directory but unstaged
```

**Hard reset (discard everything):**
```bash
git reset --hard HEAD
```
**Warning:** "There is no way to undo a Git reset hard"

**Clean untracked files:**
```bash
git clean -n      # Dry run (preview)
git clean -f      # Delete untracked files
git clean -fd     # Delete files and directories
git clean -i      # Interactive mode
```

## Safe Reset Practices

### Critical Safety Warnings

**Never reset public commits:**
Quote: "Never use git reset on publicly pushed commits. After publishing a commit, you have to assume that other developers are reliant upon it"

**Data loss risks:**
Quote: "The --hard flag is the only way to make the reset command dangerous, and one of the very few cases where Git will actually destroy data"

### Recovery Mechanisms

**Git Reflog - The Safety Net:**

Quote: "Git keeps a record of all actions in reflog"

**Recovery steps:**
```bash
# Find lost commit
git reflog

# Reset to lost commit
git reset --hard <commit-hash>
git reset --hard ORIG_HEAD  # Alternative
```

**Time limitation:**
- Git garbage collects abandoned commits every ~30 days
- Reflog exists only locally, not on remotes

### Safe Reset Workflow

**Before using --hard:**
```bash
git branch backup-branch  # Create safety branch
git reset --hard <target>
```

**Preference hierarchy (safest first):**
1. Use `--soft` or `--mixed` when in doubt
2. Use `--merge` for undoing merges with local changes
3. Use `--keep` to prevent accidental file loss
4. Only use `--hard` when certain

## CLI Design Patterns for Destructive Operations

### Interactive Confirmation Standards

**When to confirm:**

**Mild danger (small local changes):**
- Optional confirmation if command name is explicit
- Example: Deleting a single file

**Moderate danger (bigger changes):**
Quote: "Bigger local changes, remote changes, or complex bulk modifications that can't be easily undone - you usually want to prompt for confirmation"

**Severe danger (complex/irreversible):**
- Require typing resource name
- Use explicit flags like `--confirm="name-of-thing"`

### Force Flag Patterns

**Standard convention:**
Quote: "Confirm before doing anything dangerous, with a common convention being to prompt for 'y'/'yes' if running interactively, or requiring -f/--force otherwise"

**Dual-mode approach:**
- Interactive terminals: Prompt for confirmation
- Non-interactive/scripts: Require `-f` or `--force`
- Quote: "Never require a prompt - always provide a way of passing input with flags"

### Control-C Handling

**Graceful interruption:**
Quote: "If a user hits Ctrl-C during clean-up operations, skip them"

**Multi-level interrupt (Docker pattern):**
```
Gracefully stopping... (press Ctrl+C again to force)
```

## Git Safety Table

From official documentation:

```
Command                     HEAD    Index   WD      Safe?
────────────────────────────────────────────────────────
reset --soft [commit]       REF     NO      NO      ✅ YES
reset [commit]              REF     YES     NO      ✅ YES
reset --hard [commit]       REF     YES     YES     ❌ NO
checkout <commit>           HEAD    YES     YES     ✅ YES

reset [commit] <paths>      NO      YES     NO      ✅ YES
checkout [commit] <paths>   NO      YES     YES     ❌ NO
```

**Key:** Only `--hard` and `checkout <paths>` can destroy unsaved work

## Key Takeaways for Jin Reset

1. **Three reset modes** - Implement --soft, --mixed (default), --hard
2. **Layer-specific reset** - Support resetting individual layers via flags
3. **Staging vs workspace** - Separate reset of staging area vs workspace files
4. **Safety warnings** - Warn before destructive operations (--hard)
5. **Dry-run support** - Preview what will be reset
6. **Confirmation prompts** - Require confirmation for --hard or multiple layers
7. **Force flag** - Allow `-f/--force` for scripting
8. **Preserve option** - Consider stash-like functionality
9. **Clear error messages** - Explain what was reset and what was preserved
10. **Recovery guidance** - Tell users how to undo if they regret it

## Sources

- Git Reset Documentation: https://git-scm.com/docs/git-reset
- Git Restore Documentation: https://git-scm.com/docs/git-restore
- Git Stash Documentation: https://git-scm.com/docs/git-stash
- Git Reset Demystified: https://git-scm.com/book/en/v2/Git-Tools-Reset-Demystified
- Atlassian Git Reset Tutorial: https://www.atlassian.com/git/tutorials/undoing-changes/git-reset
- CLI Guidelines: https://clig.dev/
