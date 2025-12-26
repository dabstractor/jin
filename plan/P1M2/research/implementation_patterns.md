# Implementation Patterns for Jin's Phantom Git Layer

## Quick Implementation Guide

This document provides concrete, copy-paste-ready code patterns for implementing a phantom Git layer system based on research findings.

---

## Pattern 1: Ref Namespace-Based Layer Storage

**Best for:** Keeping phantom layers completely invisible to user's normal Git operations.

### Setup

```bash
#!/bin/bash
# .git/phantom-init.sh - Initialize phantom layer system

PHANTOM_REF_PREFIX="phantom/layers"

# Function to create a new layer
create_layer() {
  local layer_name="$1"
  local commit_sha="$2"

  # Create layer ref
  git update-ref \
    -m "Create phantom layer: $layer_name" \
    "refs/$PHANTOM_REF_PREFIX/$layer_name/refs/heads/main" \
    "$commit_sha"

  # Lock the ref to prevent accidental pruning
  git reflog expire --expire=never "refs/$PHANTOM_REF_PREFIX/$layer_name/refs/heads/main"

  echo "Created layer: $layer_name"
}

# Function to update layer
update_layer() {
  local layer_name="$1"
  local new_commit="$2"
  local old_commit="$3"

  git update-ref \
    -m "Update layer: $layer_name" \
    -z "refs/$PHANTOM_REF_PREFIX/$layer_name/refs/heads/main" \
    "$new_commit" \
    "$old_commit" || {
    echo "Update failed: $layer_name"
    return 1
  }
}

# Function to read layer
get_layer_head() {
  local layer_name="$1"
  git rev-parse "refs/$PHANTOM_REF_PREFIX/$layer_name/refs/heads/main" 2>/dev/null
}

# Example: Create two layers
create_layer "base" "abc123def456"
create_layer "changes" "def456abc123"
```

### Atomic Multi-Layer Updates

```bash
# .git/phantom-update.sh - Atomic update of multiple layers

update_layers_atomic() {
  local layer_updates="$1"  # Array of "layer_name:new_commit:old_commit"

  # Build transaction
  {
    echo "start"
    while IFS=: read -r layer_name new_commit old_commit; do
      echo "update refs/phantom/layers/$layer_name/refs/heads/main $new_commit $old_commit"
    done < <(echo "$layer_updates")
    echo "prepare"
    echo "commit"
  } | git update-ref --stdin --atomic

  if [ $? -eq 0 ]; then
    echo "All layers updated atomically"
    return 0
  else
    echo "Layer update failed - all changes rolled back"
    return 1
  fi
}

# Usage
cat << 'UPDATES' | update_layers_atomic
base:abc123:def456
changes:def456:ghi789
overlay:ghi789:jkl012
UPDATES
```

### User-Facing Commands (Hidden from `git branch`)

```bash
# Commands that use phantom layers are completely invisible
git log refs/phantom/layers/base/refs/heads/main
git diff refs/phantom/layers/base..refs/phantom/layers/changes
git rev-parse refs/phantom/layers/base/refs/heads/main

# User operations remain isolated
git branch -a                          # Doesn't show phantom refs
git log --all --graph                  # Only shows accessible refs
git checkout main                      # Works normally with user branches
```

---

## Pattern 2: Separate Database for Complete Isolation

**Best for:** Keeping phantom system completely separate from main repository for ease of maintenance and independent GC.

### Setup

```bash
#!/bin/bash
# setup-phantom-database.sh

PHANTOM_DB=".git/phantom.git"

# Initialize phantom database
mkdir -p "$PHANTOM_DB"
git --git-dir="$PHANTOM_DB" init --bare

# Configure phantom database
git --git-dir="$PHANTOM_DB" config gc.reflogExpire "never"
git --git-dir="$PHANTOM_DB" config gc.reflogExpireUnreachable "90.days.ago"

# Optional: Disable features that shouldn't affect user
git --git-dir="$PHANTOM_DB" config core.autoCRLF false
git --git-dir="$PHANTOM_DB" config receive.denyCurrentBranch refuse

echo "Phantom database initialized: $PHANTOM_DB"
```

### Operations on Phantom Database

```bash
#!/bin/bash
# phantom-ops.sh - Helper functions for phantom database operations

PHANTOM_DB=".git/phantom.git"

phantom_update_ref() {
  local refname="$1"
  local newvalue="$2"
  local oldvalue="${3:-0000000000000000000000000000000000000000}"

  git --git-dir="$PHANTOM_DB" update-ref \
    -m "Phantom update: $refname" \
    "$refname" "$newvalue" "$oldvalue"
}

phantom_get_ref() {
  local refname="$1"
  git --git-dir="$PHANTOM_DB" rev-parse "$refname" 2>/dev/null
}

phantom_create_commit() {
  local message="$1"
  local tree_sha="$2"
  local parent="${3:-}"

  if [ -z "$parent" ]; then
    git --git-dir="$PHANTOM_DB" commit-tree "$tree_sha" \
      -m "$message"
  else
    git --git-dir="$PHANTOM_DB" commit-tree "$tree_sha" \
      -p "$parent" \
      -m "$message"
  fi
}

phantom_log() {
  local refname="$1"
  git --git-dir="$PHANTOM_DB" log --oneline "$refname"
}

# Example: Create a layer in phantom database
phantom_update_ref "refs/layers/base/refs/heads/main" "abc123"
phantom_log "refs/layers/base/refs/heads/main"
```

### Switching Databases

```bash
# Temporary switch to phantom database
(
  export GIT_DIR=.git/phantom.git
  git status
  git branch -a
  git log --graph --all
)

# Immediately reverts to main database at end of subshell

# Or: Explicit switch with cleanup
main_git_status() {
  unset GIT_DIR GIT_WORK_TREE
  git "$@"
}

phantom_git_status() {
  GIT_DIR=.git/phantom.git git "$@"
}

# Usage
main_git_status status
phantom_git_status status
```

---

## Pattern 3: Per-Worktree Phantom State

**Best for:** Supporting git-worktree with independent layer state per worktree.

### Setup

```bash
#!/bin/bash
# setup-worktree-phantom.sh

# When creating a worktree with phantom support
create_worktree_with_phantom() {
  local worktree_path="$1"
  local branch="$2"

  # Create standard worktree
  git worktree add "$worktree_path" "$branch"

  # Add phantom metadata directory for this worktree
  local phantom_dir="$worktree_path/.phantom"
  mkdir -p "$phantom_dir"

  # Initialize per-worktree phantom state
  echo "phantom_state=ready" > "$phantom_dir/state"
  echo "layers=" > "$phantom_dir/active_layers"

  echo "Created worktree with phantom support: $worktree_path"
}

# Access per-worktree phantom state
get_worktree_phantom_state() {
  local worktree_id="$1"
  cat ".git/worktrees/$worktree_id/phantom/state" 2>/dev/null
}

set_worktree_phantom_state() {
  local worktree_id="$1"
  local state="$2"
  mkdir -p ".git/worktrees/$worktree_id/phantom"
  echo "$state" > ".git/worktrees/$worktree_id/phantom/state"
}
```

### Reading Worktree-Specific Phantom Refs

```bash
# Access per-worktree bisect ref
git rev-parse worktrees/test-next/refs/bisect/bad

# Access main-worktree phantom state
git rev-parse main-worktree/refs/phantom/layers/base

# From a linked worktree, access another worktree's state
cd /path/to/worktree/test-next
git rev-parse worktrees/other-worktree/refs/phantom/snapshot/123
```

---

## Pattern 4: Transaction-Based Atomic Operations

**Best for:** Ensuring consistency when modifying multiple layer references simultaneously.

### Atomic Layer Merge

```bash
#!/bin/bash
# merge-layers-atomic.sh

merge_layers_atomic() {
  local base_layer="$1"
  local merge_layer="$2"
  local result_ref="$3"

  # Resolve layer commits
  local base_commit=$(git rev-parse "refs/phantom/layers/$base_layer/refs/heads/main")
  local merge_commit=$(git rev-parse "refs/phantom/layers/$merge_layer/refs/heads/main")

  # Perform merge
  local merge_commit_sha=$(git commit-tree \
    "$(git rev-parse $base_commit^{tree})" \
    -p "$base_commit" \
    -p "$merge_commit" \
    -m "Merged $merge_layer into $base_layer")

  # Atomic transaction: create merge result and update both references
  {
    echo "start"
    echo "create refs/phantom/merges/temp/result $merge_commit_sha"
    echo "update refs/phantom/layers/$base_layer/refs/heads/main $merge_commit_sha $base_commit"
    echo "update refs/phantom/meta/last-merge-time $(date +%s)"
    echo "prepare"
    echo "commit"
  } | git update-ref --stdin --atomic

  if [ $? -eq 0 ]; then
    echo "Merge succeeded and layers updated atomically"
    git rev-parse refs/phantom/merges/temp/result
    return 0
  else
    echo "Merge transaction failed"
    return 1
  fi
}

# Usage
merge_layers_atomic "base" "changes" "refs/phantom/layers/merged"
```

### Transaction with Validation

```bash
#!/bin/bash
# validate-before-commit.sh

update_with_validation() {
  local layer="$1"
  local new_commit="$2"
  local old_commit="$3"

  # Validate new commit before transaction
  if ! git rev-parse "$new_commit" >/dev/null 2>&1; then
    echo "Invalid commit: $new_commit"
    return 1
  fi

  # Verify layer exists
  if ! git rev-parse "refs/phantom/layers/$layer/refs/heads/main" >/dev/null 2>&1; then
    echo "Layer does not exist: $layer"
    return 1
  fi

  # Atomic update
  {
    echo "start"
    echo "update refs/phantom/layers/$layer/refs/heads/main $new_commit $old_commit"
    echo "update refs/phantom/meta/layer-modified/$layer $(date +%s)"
    echo "prepare"
    # If validation hook exists, it's called here
    echo "commit"
  } | git update-ref --stdin --atomic
}

# Usage
update_with_validation "base" "abc123" "def456"
```

---

## Pattern 5: Reference Transaction Hook for Validation

**Best for:** Implementing cross-system consistency checks when layers are modified.

### Hook Setup

```bash
#!/bin/bash
# .git/hooks/reference-transaction

# This hook is called by git update-ref --stdin
# $1 = state: "prepared", "committed", or "aborted"

state="$1"

case "$state" in
  "prepared")
    # Read proposed updates from stdin
    while read old_oid new_oid refname; do
      # Validate phantom layer updates
      if [[ $refname =~ ^refs/phantom/layers ]]; then
        # Example validation: ensure commit exists
        if ! git rev-parse "$new_oid" >/dev/null 2>&1; then
          echo "ERROR: Invalid commit in $refname: $new_oid"
          exit 1  # Abort transaction
        fi

        # Example: prevent certain operations
        if [[ $refname =~ /readonly ]]; then
          echo "ERROR: Cannot modify readonly layer"
          exit 1  # Abort transaction
        fi
      fi
    done

    # Return 0 to allow transaction to proceed
    exit 0
    ;;

  "committed")
    # Log all successful updates
    while read old_oid new_oid refname; do
      echo "[PHANTOM] Updated $refname: $old_oid -> $new_oid" >> .git/phantom.log
    done
    ;;

  "aborted")
    # Clean up if transaction was aborted
    while read old_oid new_oid refname; do
      echo "[PHANTOM] Aborted update to $refname"
    done
    ;;
esac
```

Make the hook executable:
```bash
chmod +x .git/hooks/reference-transaction
```

---

## Pattern 6: GC Protection for Custom Refs

**Best for:** Ensuring phantom layer commits are never garbage collected.

### Configuration

```bash
#!/bin/bash
# protect-phantom-refs.sh

# Set Git configuration for phantom ref protection
git config gc.reflogExpire "never"
git config gc.reflogExpireUnreachable "90.days.ago"
git config gc.pruneExpire "90.days.ago"

# Mark phantom refs for special treatment
# (Not standard, but can be documented in comments)
git config --add phantom.protectedRefs "refs/phantom/*"
git config --add phantom.protectedRefs "refs/snapshots/*"

# Ensure reflog entries exist for all phantom refs
preserve_phantom_reflogs() {
  git for-each-ref "refs/phantom/" | while read sha rest; do
    refname="${rest##refs/}"
    git reflog expire --expire=never "$refname"
  done
}

preserve_phantom_reflogs
```

### Manual Reflog Maintenance

```bash
#!/bin/bash
# maintain-phantom-gc-safety.sh

# Before running git gc, refresh all phantom ref reflogs
refresh_phantom_reflogs() {
  echo "Refreshing phantom ref reflogs..."

  git for-each-ref "refs/phantom/" "refs/snapshots/" | while read sha rest; do
    git reflog expire --expire=never "$rest"
  done

  # Also refresh metadata refs
  for ref in refs/phantom/meta/*; do
    git reflog expire --expire=never "$ref" 2>/dev/null
  done

  echo "Phantom reflogs refreshed - safe for gc"
}

# Run gc
safe_gc() {
  refresh_phantom_reflogs
  git gc --aggressive
  echo "GC completed, phantom refs protected"
}

# Usage
safe_gc
```

---

## Pattern 7: Layer Snapshot and History

**Best for:** Recording layer state over time for auditing and recovery.

### Snapshot Mechanism

```bash
#!/bin/bash
# snapshot-layers.sh

snapshot_all_layers() {
  local snapshot_id="$1"
  local description="$2"

  if [ -z "$snapshot_id" ]; then
    snapshot_id=$(date +%s)-$(git rev-parse --short HEAD)
  fi

  echo "Creating snapshot: $snapshot_id"

  # Capture state of all layers
  {
    echo "start"

    # For each layer, record current state
    git for-each-ref "refs/phantom/layers/" | while read sha rest; do
      layer="${rest#refs/phantom/layers/}"
      layer="${layer%/refs/heads/main}"

      # Create snapshot ref
      echo "create refs/phantom/snapshots/$snapshot_id/$layer $sha"
    done

    # Record snapshot metadata
    echo "create refs/phantom/snapshots/$snapshot_id/metadata/created-at $(date +%s)"
    echo "create refs/phantom/snapshots/$snapshot_id/metadata/description $description"

    echo "prepare"
    echo "commit"
  } | git update-ref --stdin --atomic

  if [ $? -eq 0 ]; then
    echo "Snapshot created: refs/phantom/snapshots/$snapshot_id"
    return 0
  else
    echo "Snapshot creation failed"
    return 1
  fi
}

# Usage
snapshot_all_layers "" "Before major merge"
snapshot_all_layers "stable-v1" "Release candidate"

# List snapshots
list_snapshots() {
  git for-each-ref "refs/phantom/snapshots/" --format='%(refname)' | \
    awk -F/ '{print $(NF-1)}' | sort -u
}
```

### Restore from Snapshot

```bash
#!/bin/bash
# restore-layers-from-snapshot.sh

restore_snapshot() {
  local snapshot_id="$1"

  echo "Restoring snapshot: $snapshot_id"

  {
    echo "start"

    # For each layer in snapshot, restore to main layers
    git for-each-ref "refs/phantom/snapshots/$snapshot_id/" | \
    grep -v metadata | \
    while read sha rest; do
      layer="${rest#refs/phantom/snapshots/$snapshot_id/}"

      old_sha=$(git rev-parse "refs/phantom/layers/$layer" 2>/dev/null || \
                echo "0000000000000000000000000000000000000000")

      echo "update refs/phantom/layers/$layer $sha $old_sha"
    done

    echo "prepare"
    echo "commit"
  } | git update-ref --stdin --atomic
}

# Usage
restore_snapshot "1703001234-abc123d"
```

---

## Pattern 8: Danger-Free Development with Phantom Layers

**Best for:** Testing layer merge operations without affecting user's branches.

### Experimental Merge in Phantom Space

```bash
#!/bin/bash
# experimental-merge.sh

# Test merging without touching user branches
test_layer_merge() {
  local base_layer="$1"
  local merge_layer="$2"

  # Get current state
  local base_commit=$(git rev-parse "refs/phantom/layers/$base_layer/refs/heads/main")
  local merge_commit=$(git rev-parse "refs/phantom/layers/$merge_layer/refs/heads/main")

  echo "Testing merge of $merge_layer into $base_layer"
  echo "Base: $base_commit"
  echo "Merge: $merge_commit"

  # Create temporary worktree for merge test
  local temp_tree=$(mktemp -d)
  trap "rm -rf $temp_tree; git worktree remove $temp_tree 2>/dev/null" EXIT

  git worktree add "$temp_tree" "$base_commit"
  cd "$temp_tree"

  # Perform test merge
  if git merge --no-commit "$merge_commit"; then
    echo "Merge would succeed"

    # Can inspect conflicts, run tests, etc.
    # ...

    cd - && rm -rf "$temp_tree"
    return 0
  else
    echo "Merge has conflicts:"
    git status
    cd - && rm -rf "$temp_tree"
    return 1
  fi
}

# Usage
test_layer_merge "base" "changes" && echo "Safe to apply" || echo "Fix conflicts first"
```

---

## Pattern 9: Debugging Phantom Layers

**Best for:** Inspecting layer state and history for troubleshooting.

### Inspection Tools

```bash
#!/bin/bash
# inspect-phantom.sh

# Show all phantom layers and their current state
show_phantom_layers() {
  echo "=== Phantom Layers ==="
  git for-each-ref "refs/phantom/layers/" --format='
    Layer: %(refname:strip=4)
    Commit: %(objectname:short)
    Author: %(authorname)
    Date: %(creatordate:short)
    Msg: %(contents:subject)
    ---' | sed 's/^  //'
}

# Show recent changes to phantom layers
show_phantom_changes() {
  echo "=== Recent Phantom Changes ==="
  git reflog show --all | grep "phantom"
}

# Diff between phantom layers
diff_phantom_layers() {
  local layer1="$1"
  local layer2="$2"

  echo "=== Diff: $layer1 -> $layer2 ==="
  git diff \
    "refs/phantom/layers/$layer1/refs/heads/main" \
    "refs/phantom/layers/$layer2/refs/heads/main"
}

# Check layer consistency
validate_phantom_layers() {
  echo "=== Validating Phantom Layers ==="

  git for-each-ref "refs/phantom/layers/" | while read sha rest; do
    layer="${rest#refs/phantom/layers/}"
    layer="${layer%/refs/heads/main}"

    # Verify commit exists
    if git rev-parse "$sha" >/dev/null 2>&1; then
      echo "OK: $layer ($sha)"
    else
      echo "ERROR: $layer - dangling ref! ($sha)"
    fi
  done
}

# Show phantom refs in git fsck
show_phantom_fsck() {
  echo "=== Phantom Objects Status ==="
  git fsck --full 2>&1 | grep -E "^(dangling|error)" || echo "All objects OK"

  # Count phantom refs
  local count=$(git for-each-ref "refs/phantom/" | wc -l)
  echo "Total phantom refs: $count"
}

# Usage
show_phantom_layers
show_phantom_changes
diff_phantom_layers "base" "changes"
validate_phantom_layers
show_phantom_fsck
```

---

## Pattern 10: Clean Integration with User Workflow

**Best for:** Ensuring phantom system doesn't interfere with user's normal Git operations.

### User-Safe Operations

```bash
#!/bin/bash
# user-safe-operations.sh

# These operations must NOT affect user branches/tags/commits

# User's git branch should never show phantom refs
assert_phantom_hidden() {
  if git branch -a | grep "phantom"; then
    echo "ERROR: Phantom refs visible to user!"
    return 1
  fi
  echo "OK: Phantom refs hidden from user"
}

# User's reflog should not contain phantom operations
assert_phantom_reflog_clean() {
  if git reflog | grep "refs/phantom"; then
    echo "WARNING: Phantom operations in user reflog"
    # This is not necessarily an error, but informational
  fi
}

# Verify user's staged changes are not affected
verify_user_index_safe() {
  # Phantom operations should not modify .git/index
  # or .git/HEAD (user's current branch)

  local user_head=$(git rev-parse HEAD)
  local user_index_size=$(stat -f%z .git/index 2>/dev/null || stat -c%s .git/index)

  echo "User HEAD: $user_head"
  echo "User index size: $user_index_size"
}

# User's remotes should not be affected
verify_user_remotes_safe() {
  echo "=== User Remotes (unchanged by phantom ops) ==="
  git remote -v
}

# Usage in tests
assert_phantom_hidden
assert_phantom_reflog_clean
verify_user_index_safe
verify_user_remotes_safe
```

---

## Checklist for Implementation

- [ ] Initialize phantom ref namespace with `refs/phantom/*`
- [ ] Set up atomic transaction support with `git update-ref --stdin --atomic`
- [ ] Configure GC safety with reflog expire settings
- [ ] Create reference-transaction hook for validation
- [ ] Implement snapshot/restore mechanism
- [ ] Add inspection/debugging tools
- [ ] Verify user workflow isolation
- [ ] Test with git-worktree
- [ ] Document phantom ref namespace conventions
- [ ] Add error handling and rollback procedures

