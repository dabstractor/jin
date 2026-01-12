# Phantom Git Layer Patterns: Research and Implementation Guide

## Executive Summary

This research explores how sophisticated version control tools implement "phantom" or overlay Git systems that operate alongside normal Git workflows without interfering with the user's primary repository. Key patterns include git-worktree's multi-directory approach, git-annex's key-value store architecture, vcsh's multi-repository coexistence model, and git-stash's internal ref management. The research focuses on ref namespace isolation, atomic transactions, and GIT_DIR redirection techniques.

---

## 1. Overlay/Shadow Git Patterns

### 1.1 Git-Worktree: Multi-Working Directory Architecture

Git-worktree is Git's native solution for maintaining multiple concurrent working directories from a single repository.

#### How It Works

**Core Design:**
- Single shared `.git` database with per-worktree metadata
- Each linked worktree gets its own private subdirectory: `$GIT_DIR/worktrees/<id>/`
- Main worktree: `$GIT_DIR = /path/main/.git`
- Linked worktree: `$GIT_DIR = /path/main/.git/worktrees/test-next` (with `$GIT_COMMON_DIR` pointing back to shared data)

**Per-Worktree File Structure:**
```
$GIT_DIR/worktrees/test-next/
├── HEAD                          # Per-worktree (different for each worktree)
├── index                         # Per-worktree (different working state)
├── logs/                         # Per-worktree reflog
├── config.worktree              # Per-worktree config (if enabled)
└── locked                       # Optional: prevents auto-pruning
```

#### Conflict Prevention Mechanisms

1. **Branch Checkout Protection**: By default, git-worktree refuses to create a worktree if:
   - The branch is already checked out in another worktree
   - The path is already assigned to another worktree
   - Can be overridden with `--force` flag

2. **Ref Sharing Strategy**:
   - **Shared across all worktrees**: `refs/heads/*`, `refs/tags/*`, `refs/remotes/*`
   - **Exception (not shared)**: `refs/bisect/*`, `refs/worktree/*`, `refs/rewritten/*`
   - **Per-worktree (isolated)**: Pseudo refs like `HEAD`, `index`, `description`

3. **Worktree-Specific Configuration**:
   - Enable with: `git config extensions.worktreeConfig true`
   - Configuration hierarchy: shared `.git/config` then worktree-specific `.git/worktrees/<id>/config.worktree`

#### Ref Access Across Worktrees

Per-worktree refs are accessible from other worktrees via special path syntax:

```bash
# From any worktree, access main worktree's HEAD
git rev-parse main-worktree/HEAD

# From main worktree, access linked worktree's bisect state
git rev-parse worktrees/foo/refs/bisect/good
```

#### Locking and Pruning

Prevent conflicts with unmounted or portable worktrees:

```bash
# Create a lock file to prevent pruning
git worktree lock --reason "portable device" /path/worktree

# Check for locked worktrees
git worktree list --verbose
```

The locked file in `$GIT_DIR/worktrees/<id>/` prevents:
- Automatic pruning via `gc.pruneworktreesexpire`
- Moving or deleting the worktree directory
- Administrative cleanup

#### Key Advantages

- Minimal overhead (no duplicate object storage)
- Atomic operations on shared refs
- Native Git integration (no custom tooling needed)
- Garbage collection protects all worktree refs automatically

#### Limitations

- All worktrees share the same object database (cannot have conflicting unpacked objects)
- Per-worktree refs are limited to specific namespaces
- No built-in namespace isolation for custom metadata

---

### 1.2 Git-Annex: Key-Value Store for Large Files

Git-annex extends Git to manage large files without storing their contents in the repository.

#### Architecture Overview

**Core Concept**: Separate file metadata (tracked in Git) from file content (stored externally).

**Data Flow:**
1. User runs `git annex add <large-file>`
2. File content is moved to key-value store: `.git/annex/objects/`
3. File is renamed according to checksum (WORM or SHA1 backend)
4. Symlink is created pointing to content location
5. Only the symlink is committed to Git

**Key-Value Backend Options:**
- **WORM** (Write Once, Read Many): Default, assumes file contents don't change
- **SHA1**: Expensive for large files, but tracks content changes
- **URL**: Fetches content from external URLs

#### Location Tracking System

Git-annex maintains a location database showing where each file copy exists:
- Local repository
- Remote repositories (network or USB drives)
- Cloud storage (via special remotes)
- Offline locations (archived drives)

**Location Safety Guarantees:**
- Knows how many copies of each file exist
- Prevents dropping last local copy without remote backup
- Supports "partial checkouts" (download only needed files)

#### Content Addressing

```bash
# File is referenced by content hash, not path
# Multiple names can point to same content
# Deduplication across branches and history

# Symlink resolves to .git/annex/objects/<hash-prefix>/<hash>
ls -la largefile.dat
# -> .git/annex/objects/ab/cd/WORM-s1000--abc123.../
```

#### Mixed Content Support

```bash
# Configure which files are treated as large
git config annex.largefiles "largerthan=100mb or *.iso"

# Both git add and git annex add work automatically
git add .                  # Small files go to git, large to annex
git commit                 # Commits both
```

#### Distributed Nature

- **No central server** (unlike Git LFS)
- **All repositories are peers** for data distribution
- **Partial clones** possible (clone without downloading large files)
- **Offline-friendly** (works with disconnected drives)

---

### 1.3 Git-Stash: Internal Ref Management

Git-stash implements temporary work saving through direct Git object manipulation.

#### Internal Implementation

**Storage Mechanism:**
- Stashes are regular merge commits stored as objects
- Reference: `refs/stash` points to latest stash
- Older stashes accessible via reflog: `stash@{0}`, `stash@{1}`, etc.
- Location: `.git/refs/stash` (file) or `.git/logs/refs/stash` (reflog)

**Commit Structure:**
A stash creates a merge commit with parents:
```
stash@{0}              # The stash commit itself
├── parent 1           # Pre-stash HEAD commit
├── parent 2           # Index snapshot (changes staged for commit)
└── parent 3 (optional) # Untracked files (if stashed with --include-untracked)
```

#### Reflog Integration

```bash
.git/logs/refs/stash contains:
<old-sha> <new-sha> <user> <timestamp> saving WIP on master: abc123 commit message
```

**Advantages:**
- Stashes survive garbage collection (tracked in reflog)
- Default retention: 90 days (gc.reflogExpire)
- Unreachable stash entries: 30 days (gc.reflogExpireUnreachable)

#### Low-Level Commands

```bash
# Create a stash (dangling commit, not stored in refs yet)
git stash create

# Store the dangling commit in refs/stash
git stash store <commit-sha>

# Advanced: Work with raw commit objects
git write-tree            # Create tree from index
git commit-tree           # Create commit with parents
```

#### Design Insight

Unlike branches (which use simple ref updates), stash requires:
- Direct manipulation of Git objects
- Custom parent commit generation
- Reflog interaction for historical tracking

This complexity enables stash's flexibility: temporary, non-intrusive storage that doesn't pollute the branch namespace.

---

### 1.4 Vcsh: Multi-Repository in Single Directory

Vcsh enables managing multiple independent Git repositories within the same directory tree (typically `$HOME`).

#### How It Works

**Core Mechanism:**
- Each vcsh-managed repository stores its Git metadata separately
- Multiple repositories share the same working directory
- Git allows specifying alternative `.git` directory via environment variables

**Repository Organization:**
```
$HOME/
├── .config/vcsh/repo.d/          # Repository storage
│   ├── zsh.git/                  # Zsh configuration repo
│   ├── vim.git/                  # Vim configuration repo
│   └── ssh.git/                  # SSH configuration repo
├── .zshrc                        # Tracked by zsh.git
├── .vimrc                        # Tracked by vim.git
├── .ssh/                         # Tracked by ssh.git
```

#### Multi-Repo Management Pattern

**Basic Workflow:**
```bash
# Initialize a new vcsh repository
vcsh init vim

# Add files to this repository
vcsh vim add ~/.vimrc ~/.vim/

# Commit changes to this repo only
vcsh vim commit -m 'Update Vim config'

# Other repos unaffected
vcsh zsh add ~/.zshrc

# List all vcsh repositories
vcsh list
```

**Key Insight:** Each `vcsh vim <command>` operation sets:
- `GIT_DIR` to `~/.config/vcsh/repo.d/vim.git/`
- `GIT_WORK_TREE` to `$HOME`

Git then operates on this specific repository only.

#### Integration with Myrepos (mr)

Vcsh is designed to work with `myrepos` (mr) for multi-repository management:

```bash
# mr configuration tracks all vcsh repos
# Can clone setup across machines in minutes
mr update     # Update all vcsh repos at once
mr status     # Status of all repos
```

#### Hook System

```
$XDG_CONFIG_HOME/vcsh/hooks-available/   # Available hooks
$XDG_CONFIG_HOME/vcsh/hooks-enabled/     # Enabled hooks (symlinked)
```

**Hooks**: Execute custom scripts on clone, checkout, update operations.

#### Comparison with Alternatives

| Tool | Approach | Overhead |
|------|----------|----------|
| vcsh | Direct git repos | Minimal |
| chezmoi | Symlinks + templating | Medium |
| stow | GNU Stow symlinks | Medium |
| dvcs-autosync | Auto-sync daemon | High |

---

## 2. Custom Ref Namespaces

### 2.1 Ref Namespace Fundamentals

Git's ref system provides built-in namespace isolation for parallel development workflows.

#### Standard Ref Hierarchy

```
refs/
├── heads/          # User branches
├── tags/           # User tags
├── remotes/        # Remote tracking branches
├── namespaces/     # GIT_NAMESPACE-based isolation
├── bisect/         # Bisect state (per-worktree)
├── worktree/       # Worktree-specific (per-worktree)
├── rewritten/      # Rebase/rewrite history (per-worktree)
└── <custom>/       # Application-defined namespaces
```

#### Using Custom Ref Namespaces

**Environment Variable Method:**
```bash
# All git operations use refs/namespaces/foo/ prefix
export GIT_NAMESPACE=foo
git branch feature-x
# Creates: refs/namespaces/foo/refs/heads/feature-x

# Nested namespaces
export GIT_NAMESPACE=foo/bar
# Creates: refs/namespaces/foo/refs/namespaces/bar/refs/heads/...
```

**Direct Ref Creation:**
```bash
# Create refs in custom namespace without GIT_NAMESPACE
git update-ref refs/layers/layer1/refs/heads/feature abc123

# Access from another repo/namespace
git rev-parse refs/layers/layer1/refs/heads/feature
```

#### DVC Experiments Pattern

DVC (Data Version Control) uses `refs/exps/` namespace for experiment tracking:

```
refs/exps/
├── refs/exps/exec/abc123/   # Current experiment state
├── refs/exps/save/          # Saved experiments
└── refs/exps/temp/          # Temporary experiments
```

**Advantages:**
- Experiments don't appear in `git branch` output
- No pollution of user-visible branches
- Thousands of experiments without affecting team's remotes
- Custom refs aren't transferred to remote by default

---

### 2.2 Safety Considerations for Custom Refs

#### Garbage Collection Protection

Git's garbage collector automatically protects custom refs:

```bash
git gc --auto
```

**GC Behavior:**
- Keeps all objects referenced by `refs/*` (including custom namespaces)
- Also protects: index, remote-tracking branches, reflogs
- Dangling objects older than `gc.pruneExpire` (default 14 days) are deleted
- Unreachable reflog entries older than `gc.reflogExpireUnreachable` (30 days) are deleted

**Implications for Custom Layers:**
- Custom refs are never auto-deleted by gc
- Only manual `git reflog expire` + `git gc --prune=now` removes custom ref objects
- Safe for long-term storage of layer metadata

#### Ref Packing

For efficiency, Git packs frequently-accessed refs:

```bash
# Packed refs stored in single file
.git/packed-refs

# Example format:
abc123def456 refs/layers/layer1/refs/heads/main
def456abc123 refs/layers/layer2/refs/heads/feature
```

**Implications:**
- Custom refs benefit from packing optimization
- No conflicts with packed vs loose refs
- Modification creates new loose ref that overrides packed ref

#### Avoiding Ref Conflicts

**Reserved Namespace Prefixes to Avoid:**
- `refs/heads/` - User branches
- `refs/tags/` - User tags
- `refs/remotes/` - Remote tracking
- `refs/bisect/` - Bisect state
- `refs/worktree/` - Worktree state

**Recommended Custom Namespaces:**
```
refs/layers/*/          # Layer system
refs/phantom/*/         # Phantom/shadow system
refs/snapshots/*/       # Snapshot tracking
refs/experiments/*/     # Experiment management
refs/internal/*/        # Internal metadata
```

#### Security Limitations of Namespaces

**Important:** Namespaces are NOT effective for read access control.

**Attack Vectors:**
1. **Object ID Stealing**: Attacker creates ref to private object in accessible namespace
2. **Delta Information Leakage**: Attacker claims to have object X, victim reveals Y via delta

**Recommendation:** Store sensitive data in separate repository, not in namespaces.

---

### 2.3 Namespace Isolation from User Branches

**Complete Isolation Pattern:**

```bash
# User operations never see custom refs
git branch -a                    # Lists refs/heads, refs/remotes only
git tag                          # Lists refs/tags only
git rev-parse @                  # Resolves HEAD in current namespace

# Custom refs accessible only with explicit ref path
git rev-parse refs/layers/layer1/HEAD
git log refs/phantom/changes..HEAD

# Can't accidentally delete custom refs
git branch -D refs/phantom/changes    # Error: invalid branch name
git update-ref -d refs/phantom/changes  # Works (direct plumbing)
```

---

## 3. Internal Git Storage Patterns

### 3.1 Storing Multiple Logical Branches Without User-Visible Branches

The key insight: Git distinguishes between **porcelain** (user-facing) and **plumbing** (low-level) commands.

#### Porcelain vs Plumbing Refs

**Porcelain commands** only recognize:
- `refs/heads/*` (branches)
- `refs/tags/*` (tags)
- `refs/remotes/*` (remote tracking)
- Symbolic refs (pointing to other refs)

**Plumbing commands** work with any ref path:
- `refs/layers/*/refs/heads/*` (custom branches)
- `refs/experiments/*` (experiment metadata)
- `refs/snapshots/*` (snapshots)
- Any path in `.git/refs/` or `.git/packed-refs`

#### Architecture Example: Multi-Layer System

```
User View (git branch):
  main
  feature/x
  feature/y

Internal Layer System:
  refs/layers/base/refs/heads/main        # Base layer branch
  refs/layers/changes/refs/heads/changes  # Changes layer branch
  refs/layers/overlay/refs/heads/overlay  # Overlay layer branch
```

**Implementation Strategy:**

```bash
# Create layers as custom refs (user never sees these)
git update-ref refs/layers/base/refs/heads/main abc123
git update-ref refs/layers/changes/refs/heads/main def456
git update-ref refs/layers/overlay/refs/heads/main ghi789

# User's main branch is computed from layers
git update-ref refs/heads/main ghi789  # Points to overlay

# Users can work normally with git branch
git checkout main
git branch -a   # Only shows main, not internal layers

# System can modify layers independently
git update-ref refs/layers/base/refs/heads/main $(git rev-parse main^)
```

#### Commit Orphaning Prevention

**Strategy: Protect layer commits in reflog**

```bash
# When creating/modifying layers, create reflog entries
git reflog expire --expire=never refs/layers/*/refs/heads/*

# Layer commits protected from gc.reflogExpireUnreachable
# (default 30 days) by never expiring reflog entries
```

**Garbage Collection Safety:**
```bash
# Explicit reflog management
git for-each-ref refs/layers/ | while read sha ref; do
  # Ensure reflog entry exists
  git reflog expire --expire=never "$ref"
done

# Now safe to run gc
git gc --aggressive
```

---

### 3.2 Reference Transactions and Atomicity

Git's reference transaction system ensures safe multi-ref updates.

#### Transaction States

```
START
  │
  └── prepare ──┐
                 ├── (all locks acquired successfully)
                 │
                 └── commit ──────> (all refs updated, transaction ends)
                 │
                 └── (conflict/error)
                      │
                      └── aborted ──> (locks released, no changes)
```

#### Git-Update-Ref Transaction Commands

The `git update-ref --stdin` command provides explicit transaction control:

```bash
# Start transaction
start

# Queue reference updates
update refs/layers/base/refs/heads/main abc123 def456
update refs/layers/changes/refs/heads/main def456 ghi789
create refs/layers/new-layer xyz789

# Prepare: acquire all locks and validate
prepare

# On success, commit all together
commit

# On failure, abort and release locks
abort
```

**Atomic Semantics:**
- If all refs can be locked with matching old-oids simultaneously → all succeed
- If any ref has wrong old-oid or can't be locked → entire transaction aborts
- No partial updates: either all changes applied or none

**Important Caveat:**
While each individual ref is updated atomically, a concurrent reader may see partial updates (not all refs modified yet). For strong consistency, use:
- Reference-transaction hook (verify all updates succeeded)
- External coordination mechanism
- Reftable backend (Git 2.51.0+, orders of magnitude faster)

#### Reference-Transaction Hook

Available as of Git 2.5.0+, the `reference-transaction` hook observes all ref updates:

```bash
# .git/hooks/reference-transaction (executable)
#!/bin/bash
state="$1"  # prepared, committed, or aborted

# Read ref updates from stdin
while read old_oid new_oid refname; do
  case "$state" in
    prepared)
      # Validate proposed updates, return non-zero to abort
      validate_update "$refname" "$old_oid" "$new_oid"
      ;;
    committed)
      # Transaction succeeded (can't abort here)
      log_transaction "$refname" "$new_oid"
      ;;
    aborted)
      # Transaction failed
      cleanup_transaction "$refname"
      ;;
  esac
done
```

**Use Cases:**
- Distributed consistency (vote across replicas)
- Audit logging of all ref changes
- Custom validation rules
- Triggering external systems

**Limitations:**
- Committed/aborted hooks can't prevent transaction (info-only)
- Prepared hook can abort, but called only when transaction ready
- Hook stdout/stderr captured, but only prepared state exit code matters

---

### 3.3 Dangling Objects and Retention

#### Object Lifecycle

```
Object created
  │
  ├── Referenced by ref
  │     │
  │     └─(reachable) GC never deletes
  │
  └── Unreachable (no ref)
        │
        ├─(in reflog) Protected 30+ days
        │
        ├─(no reflog) Loose objects after 14 days
        │
        └─(gc --prune=now) Immediately deleted
```

#### GC Configuration for Custom Refs

**Protect layer commits:**

```bash
# Disable automatic pruning of unreachable objects
git config gc.pruneExpire "never"

# Or: Keep unreachable objects for 90 days (default 14)
git config gc.pruneExpire "90.days.ago"

# Reflog retention for custom refs
git config gc.reflogExpire "never"
git config gc.reflogExpireUnreachable "90.days.ago"
```

#### Manually Ensuring Retention

```bash
# Refresh reflog for all layer refs
git for-each-ref refs/layers/ | while read sha rest; do
  # Update reflog timestamp to now (prevents expiry)
  git reflog expire --expire=never "${rest##refs/}"
done

# Refresh for specific ref
git reflog expire --expire=never refs/layers/base/refs/heads/main
```

#### Finding Dangling Objects

```bash
# List all dangling commits in object database
git fsck --lost-found

# Show dangling commits reachable from layer refs
git log --all --graph --decorate --oneline

# Find specific dangling commit
git rev-list --all | while read sha; do
  git cat-file -t "$sha" | grep -q commit && \
  git merge-base --is-ancestor "$sha" refs/layers//* || \
  echo "dangling: $sha"
done
```

---

## 4. Best Practices for Parallel Git Repositories

### 4.1 Managing Separate Git Database

#### GIT_DIR and GIT_WORK_TREE Environment Variables

Two critical environment variables control Git's paths:

- **GIT_DIR**: Location of `.git` directory (repository metadata)
- **GIT_WORK_TREE**: Working directory (where files are checked out)

**Default Behavior:**
```bash
# If .git is a file (linked worktree):
GIT_WORK_TREE = .git's parent
GIT_DIR = (path from .git content)

# If .git is a directory:
GIT_DIR = .git (current dir)
GIT_WORK_TREE = GIT_DIR's parent
```

#### Separate Database Pattern

**Use Case: Phantom layer system alongside user's repo**

```bash
# Main repository (user's)
.git/                           # Standard .git directory

# Phantom layer system (separate database)
.git/phantom.git/              # Separate database
  ├── objects/
  ├── refs/layers/
  └── HEAD

# Set for phantom operations
export GIT_DIR=.git/phantom.git
export GIT_WORK_TREE=.

# Now all git operations use phantom database
git rev-list refs/layers/base  # Uses phantom.git

# Revert to main repository
unset GIT_DIR GIT_WORK_TREE
```

#### Shadow Repository Pattern

The git-shadow project demonstrates this approach:

```
Project directory:
├── .git/                       # Main repository
├── .shadow/                    # Shadow repository directory
│   └── objects/
│       └── <by-commit-id>/    # One repo per commit
│           └── .git/
└── shadow-file (temporary)    # Buffer for current session
```

**Workflow:**
1. Buffer file contents as user edits
2. Periodically commit buffers to shadow repo
3. Shadow repo organized by original commit ID
4. Analysis: correlate changes with commits

---

### 4.2 GIT_DIR Redirection Techniques

#### Runtime Redirection

```bash
# Explicit GIT_DIR for single command
GIT_DIR=.git/phantom.git git branch -a

# For scripts operating on multiple repos
for repo in repos/*; do
  GIT_DIR="$repo/.git" git status
done
```

#### Persistent Redirection in .git File

Worktrees use `.git` as a plain file instead of directory:

```bash
# Contents of .git (worktree's root)
gitdir: /path/main/.git/worktrees/test-next
worktreeDir: /path/test-next
```

**To create phantom database link:**

```bash
# Rename existing .git to preserve it
mv .git .git.main

# Create .git file pointing to phantom database
cat > .git << EOF
gitdir: .git/phantom.git
EOF

# Now git operations use phantom database by default
git status

# Access main repository when needed
GIT_DIR=.git.main git status
```

#### Layered Configuration

```bash
# Global: use main repository
GIT_DIR=.git git status

# Per-operation: use phantom database
GIT_DIR=.git/phantom.git git log refs/layers/base

# Script: iterate all databases
for db in .git .git/phantom.git .git/archive.git; do
  echo "=== $db ==="
  GIT_DIR="$db" git rev-parse HEAD
done
```

---

### 4.3 Avoiding Interference with User Workflow

#### Ref Namespace Isolation

**Keep phantom refs completely invisible:**

```bash
# User operations
git branch -a                    # Only sees refs/heads, refs/remotes
git log --all --graph           # Only follows accessible branches

# Phantom refs in custom namespace never appear
refs/phantom/*
refs/layers/*
refs/snapshots/*

# Accessed only with explicit ref paths
git log refs/phantom/layer1..main
git rev-parse refs/layers/base
```

#### Hook System Separation

**Don't interfere with user's git hooks:**

```bash
# Standard hooks location (user-controlled)
.git/hooks/

# Phantom system hooks (separate location)
.git/phantom.git/hooks/

# Or use environment variable
CORE_HOOKSDIR=.git/phantom-hooks/

# Never trigger phantom hooks on user operations
# Never modify user's .git/hooks/
```

#### Index and HEAD Isolation

**Phantom database has its own state:**

```bash
# Main repository state
.git/HEAD                       # User's current branch
.git/index                      # User's staging area

# Phantom system state
.git/phantom.git/HEAD           # Phantom HEAD (isolated)
.git/phantom.git/index          # Phantom index (isolated)

# Modifications to phantom state don't affect user
GIT_DIR=.git/phantom.git git add .  # Modifies phantom index only
git status                           # Shows user's index (unchanged)
```

#### Performance Isolation

**Prevent phantom operations from slowing user:**

```bash
# Phantom gc doesn't affect main repo performance
GIT_DIR=.git/phantom.git git gc --aggressive

# Use separate reflog expiration
git -C .git/phantom.git reflog expire --expire=now --all

# Separate repack strategy
git -C .git/phantom.git repack -Ad
```

#### Disabled Features in Phantom Mode

```bash
# Disable operations that would modify user state
GIT_DIR=.git/phantom.git git config --local receive.denyCurrentBranch refuse

# Prevent accidental user impact
GIT_DIR=.git/phantom.git git config --local core.autoCRLF false
GIT_DIR=.git/phantom.git git config --local core.safeCRLF false
```

---

## 5. Implementation Recommendations for Jin

### 5.1 Recommended Architecture

Based on this research, a **hybrid approach** combining multiple patterns:

**Layer 1: Ref Namespace (Phantom Refs)**
```
User branches:          refs/heads/*
Phantom layers:         refs/phantom/layers/*/refs/heads/*
Snapshot history:       refs/phantom/snapshots/*/refs/heads/*
Merge state:           refs/phantom/merges/*/refs/heads/*
```

**Layer 2: Separate Database (Optional for Efficiency)**
```
.git/phantom.git/       # Could hold phantom refs for isolation
                        # Or use as journal/log database
```

**Layer 3: Per-Worktree Metadata**
```
.git/worktrees/<id>/phantom/    # Per-worktree shadow state
                                # If using worktree-aware Jin
```

### 5.2 Atomic Operations

Use `git update-ref --stdin --atomic` for multi-ref updates:

```bash
git update-ref --stdin --atomic << EOF
start
update refs/phantom/layers/base/refs/heads/main abc123 def456
update refs/phantom/layers/changes/refs/heads/main def456 ghi789
update refs/phantom/meta/base-commit abc123
update refs/phantom/meta/state "layers-updated"
prepare
commit
EOF
```

### 5.3 GC Safety

Configure Git to protect phantom refs:

```bash
# In .git/config (main repo)
[refs]
  # Protect custom phantom refs from gc
  preserveObjects = true

[gc]
  # Generous unreachable object retention
  reflogExpire = 180 days
  reflogExpireUnreachable = 90 days
  pruneExpire = 90 days

# Or: per-reference retention
[refs "refs/phantom/*"]
  # Keep phantom refs forever in gc
  preserveObjects = true
```

### 5.4 Worktree Compatibility

If Jin needs to work with git-worktree:

```bash
# Each worktree has private location for phantom state
.git/worktrees/<id>/phantom/layers/      # Per-worktree layer state
.git/worktrees/<id>/phantom/state        # Per-worktree Jin state

# Main worktree shared phantom database
.git/phantom.git/                        # Shared layer definitions
.git/refs/phantom/layers/                # Shared layer refs
```

### 5.5 Ref Naming Convention

**Recommended structure for clarity and safety:**

```
refs/phantom/layers/{layer-id}/refs/heads/{branch}
refs/phantom/layers/{layer-id}/refs/tags/{tag}
refs/phantom/snapshots/{snapshot-id}
refs/phantom/merges/{merge-id}/state
refs/phantom/merges/{merge-id}/base
refs/phantom/merges/{merge-id}/branch
refs/phantom/meta/version
refs/phantom/meta/state
```

---

## 6. Comparison Table: Overlay Approaches

| Approach | Storage | Isolation | User Visibility | Gc Safety | Atomicity |
|----------|---------|-----------|-----------------|-----------|-----------|
| **Ref Namespaces** | Shared objects | Ref namespace | Hidden | Auto-protected | Yes (--atomic) |
| **Separate DB** | Isolated objects | Complete | Hidden | Manual | Yes (--atomic) |
| **git-worktree** | Shared objects | Per-worktree refs | Hidden | Auto-protected | Yes (per-ref) |
| **git-annex** | External store | Content hash | Symlinks visible | Auto-protected | Via committing |
| **vcsh** | Separate repos | Per-repo index | Files visible | Auto-protected | Per-repo gc |

---

## 7. Sources and Further Reading

### Official Git Documentation

- [Git - git-worktree Documentation](https://git-scm.com/docs/git-worktree)
- [Git - gitnamespaces Documentation](https://git-scm.com/docs/gitnamespaces)
- [Git - git-update-ref Documentation](https://git-scm.com/docs/git-update-ref)
- [Git - githooks Documentation](https://git-scm.com/docs/githooks)
- [Git - git-gc Documentation](https://git-scm.com/docs/git-gc)
- [Git Internals - Plumbing and Porcelain](https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain)

### Tool Documentation

- [git-annex Home](https://git-annex.branchable.com/)
- [git-annex - Large Files Management](https://lwn.net/Articles/419241/)
- [vcsh - Version Control System for $HOME](https://github.com/RichiH/vcsh)
- [vcsh Ubuntu Manpage](https://manpages.ubuntu.com/manpages/trusty/man1/vcsh.1.html)

### Technical Deep Dives

- [Inside Git Stash](https://www.codeproject.com/Articles/5378745/Inside-Git-Stash)
- [Git from the Bottom Up - Stashing](https://jwiegley.github.io/git-from-the-bottom-up/4-Stashing-and-the-reflog.html)
- [Mastering Git Worktree](https://mskadu.medium.com/mastering-git-worktree-a-developers-guide-to-multiple-working-directories-c30f834f79a5)
- [DVC - Custom References for ML Experiments](https://dvc.org/blog/experiment-refs)

### Advanced Patterns

- [git-shadow: Recording Coding Activity](https://github.com/jfoote/git-shadow)
- [ShadowGit: Parallel Repository Tracking](https://docs.shadowgit.com/)
- [Git Plumbing Commands](https://mindmajix.com/git-plumbing-commands)
- [Reference Transactions in Git](https://github.com/git/git/commit/675415976704459edaf8fb39a176be2be0f403d8)

---

## Appendix A: Quick Reference Commands

```bash
# Create custom ref (plumbing)
git update-ref refs/phantom/layers/base/refs/heads/main <commit-sha>

# Atomic multi-ref update
git update-ref --stdin --atomic << EOF
start
update refs/phantom/layer1 abc123 def456
update refs/phantom/layer2 def456 ghi789
prepare
commit
EOF

# Access custom ref
git rev-parse refs/phantom/layers/base/refs/heads/main
git log refs/phantom/layers/base..HEAD

# Create ref with reflog protection
git update-ref -m "Layer 1 base" refs/phantom/layers/base abc123
git reflog expire --expire=never refs/phantom/layers/base

# Inspect transaction
git update-ref --dry-run refs/phantom/test abc123

# Using namespace
GIT_NAMESPACE=phantom git branch  # Shows branches in phantom namespace
export GIT_NAMESPACE=phantom
git branch feature                # Creates refs/namespaces/phantom/refs/heads/feature

# Worktree-aware phantom state
GIT_DIR=.git/worktrees/test-next/phantom.git git status
```

---

## Appendix B: Architecture Decision Tree

**When to use each pattern:**

1. **Need to hide from `git branch` output?**
   - YES → Use ref namespaces (`refs/phantom/*`)
   - NO → Can use branches, tags, or remote refs

2. **Need per-worktree isolation?**
   - YES → Store state in `.git/worktrees/<id>/phantom/`
   - NO → Shared `.git/phantom.git` or `.git/refs/phantom/`

3. **Need atomic multi-ref updates?**
   - YES → Use `git update-ref --stdin --atomic`
   - NO → Single ref updates with `git update-ref`

4. **Need complete object store isolation?**
   - YES → Separate `.git/phantom.git` database
   - NO → Share objects in main `.git`, use ref namespaces

5. **Need to track large files/binary content?**
   - YES → Use git-annex pattern (key-value store)
   - NO → Regular refs and commits

6. **Need distributed repository access?**
   - YES → Consider separate repos (vcsh pattern)
   - NO → Single unified database

