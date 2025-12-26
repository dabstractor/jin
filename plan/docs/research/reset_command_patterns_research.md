# CLI Reset Command Patterns Research

## Overview

This document researches how various CLI tools implement reset commands, focusing on argument patterns, user experience, best practices, and implementation details. The research covers Git (the reference implementation), Docker, Kubernetes, and other developer tools, with special attention to Rust CLI implementations.

## 1. Git Reset Command (The Reference)

Git's `reset` command is the de facto standard for reset functionality in version control systems. It serves as the primary reference for understanding reset semantics.

### 1.1 Core Implementation

```bash
# Basic syntax
git reset [--mixed] [--soft] [--hard] [--merge] [--keep] [<commit>] [--] <paths>...

# Common patterns
git reset HEAD~1              # Mixed reset (default)
git reset --soft HEAD~1       # Soft reset
git reset --hard HEAD~1       # Hard reset
git reset HEAD file.txt        # Unstage specific file
```

### 1.2 Argument Patterns

| Option | Behavior | Affects Index | Affects Working Tree | Use Case |
|--------|----------|---------------|---------------------|----------|
| `--soft` (default) | Reset HEAD only | No | No | Undo commit but keep changes staged |
| `--mixed` | Reset HEAD and index | Yes | No | Unstage changes but keep in working tree |
| `--hard` | Reset HEAD, index, and working tree | Yes | Yes | Discard all changes permanently |
| `--merge` | Reset with merge safety checks | Yes | Yes | Safe reset with merge conflict awareness |
| `--keep` | Reset but keep local changes | No | No | Reset HEAD but preserve working tree state |

### 1.3 User Experience Patterns

#### Before Reset Information Display
```bash
$ git status
On branch feature-branch
Changes to be committed:
  modified:   src/main.rs
  modified:   tests/test.rs

Changes not staged for commit:
  modified:   README.md

$ git reset --soft HEAD~1
```

#### Output Messages
```bash
# Soft reset
$ git reset --soft HEAD~1
Unstaged changes after reset:
M   src/main.rs
M   tests/test.rs

# Hard reset (with warning)
$ git reset --hard HEAD~1
warning: changes in the working tree will be discarded
```

### 1.4 Error Handling Patterns

```bash
# Invalid commit reference
fatal: ambiguous argument 'HEAD~999': unknown revision or path not in the working tree

# Mixed with paths (confusing behavior)
$ git reset HEAD --mixed file.txt
# This actually unstages file.txt, doesn't reset HEAD

# Untracked files with hard reset
$ git reset --hard HEAD~1
Untracked working tree files would be overwritten by reset:
  new_file.txt
```

### 1.5 Confirmation Prompts

Git does not typically prompt for confirmation on `reset` operations, but provides clear warnings:

```bash
# When there are untracked files
$ git reset --hard HEAD~1
Untracked working tree files would be overwritten by reset:
  new_file.txt
Discarded 3 files.

# When working tree is dirty
$ git status -s
 M modified.txt
?? new.txt

$ git reset --hard HEAD~1
error: The following untracked working tree files would be overwritten by reset:
  new.txt
Please move or remove them before you can reset.
```

## 2. Docker Reset Patterns

Docker implements several reset-like commands through `docker system` and related commands.

### 2.1 Docker System Prune (Closest to Reset)

```bash
# Comprehensive cleanup
docker system prune -a

# Selective pruning
docker system prune --volumes --all --force
```

#### Argument Patterns
| Flag | Description | Destructiveness |
|------|-------------|----------------|
| `-a`, `--all` | Remove all unused images, not just dangling ones | High |
| `--volumes` | Remove all unused volumes | Very High |
| `--filter` | Filter resources by labels | Medium |
| `--force` | Skip confirmation prompt | N/A |

#### UX Patterns
```bash
# Without force (shows what would be removed)
$ docker system prune
WARNING! This will remove all stopped containers.
Are you sure you want to continue? [y/N]

# With force (no confirmation)
$ docker system prune --force
Deleted Containers:
c42b3193b90f
f86c8b93195e

Deleted Images:
<none>
Deleted Volumes:
<none>

Total reclaimed space: 1.2GB
```

### 2.2 Container and Image Reset Patterns

```bash
# Reset specific container
docker container prune --force

# Reset networks
docker network prune

# Reset build cache
docker builder prune
```

## 3. Kubernetes Reset Patterns

Kubernetes implements reset functionality primarily through `kubectl` for cluster and component resets.

### 3.1 Cluster Reset Patterns

```bash
# Reset cluster (dangerous)
kubeadm reset --force

# Drain node before reset
kubectl drain node-name --ignore-daemonsets --delete-emptydir-data
```

#### Argument Patterns
| Flag | Description | Safety Level |
|------|-------------|--------------|
| `--force` | Skip confirmation | Low |
| `--ignore-preflight-errors` | Skip pre-flight checks | Medium |
| `--cri-socket` | Specify CRI socket | High |

#### UX Patterns
```bash
# Reset with confirmation
$ kubeadm reset
[reset] WARNING:Changes made to the host by 'kubeadm init' or 'kubeadm join' will be reverted.
[reset] Are you sure you want to continue? [y/N]: y

# Force reset
$ kubeadm reset --force
[reset] Stopping the kubelet service
[reset] Unmounting mounted directories in "/var/lib/kubelet"
[reset] Detaching mounted volumes
[reset] Deleting contents of directories: [/etc/kubernetes/manifests /var/lib/dockershim /var/lib/etcd]
[reset] Deleting files: [/etc/kubernetes/admin.conf /etc/kubernetes/kubelet.conf /etc/kubernetes/bootstrap-kubelet.conf]
```

### 3.2 Resource Reset Patterns

```bash
# Reset deployments
kubectl rollout restart deployment/my-app

# Reset statefulsets
kubectl rollout restart statefulset/my-db

# Reset configmaps (by recreating)
kubectl create configmap my-config --from-file=config.yaml --dry-run=client -o yaml | kubectl apply -f -
```

## 4. Rust CLI Tool Examples

Several prominent Rust CLI tools implement reset functionality with excellent patterns.

### 4.1 Cargo (Package Manager)

Cargo implements `cargo clean` as a reset-like operation:

```bash
# Clean build artifacts
cargo clean

# Clean specific target
cargo clean --package my-package
```

#### UX Patterns
```bash
$ cargo clean
    Removing target/
```

### 4.2 Rust Analyzer (Language Server)

Rust analyzer uses `cargo check` for validation and can be combined with clean:

```bash
# Clean and check
cargo clean && cargo check
```

### 4.1 Helix Editor (Text Editor)

Helix implements reset through its git integration:

```bash
# In Helix, the reset command follows git patterns
:reset soft
:reset mixed
:reset hard
```

### 4.3 Gitoxide (Git Implementation in Rust)

Gitoxide provides a pure-Rust implementation of git with excellent reset patterns:

```bash
# Using gitoxide
gitoxide reset --soft HEAD~1
gitoxide reset --hard HEAD~1
```

## 5. Common Reset Command Patterns

### 5.1 Argument Pattern Taxonomy

| Pattern | Example Tools | Description |
|---------|---------------|-------------|
| Git-style modes | git, gitoxide | `--soft`, `--mixed`, `--hard` with semantic differences |
| Prune-based | docker, kubectl | `prune` with various filters and flags |
| Clean-based | cargo | `clean` with optional targets |
| Rollback-based | kubectl | `rollback` with version references |
| Force-based | Many tools | `--force` to bypass confirmation |

### 5.2 Layer-Based Reset Patterns

Tools with layered configurations often implement targeted resets:

```bash
# Layer-based reset (Jin pattern)
jin reset --mode                # Reset mode layer only
jin reset --scope python        # Reset scope layer only
jin reset --project             # Reset project layer only
jin reset                       # Reset all layers
```

### 5.3 Path-Based Reset Patterns

Most tools support selective reset by path:

```bash
# Path-specific reset
git reset HEAD file.txt           # Unstage specific file
docker system prune --filter name=app  # Filter by name
cargo clean --package my-lib     # Clean specific package
```

## 6. Best Practices for Reset UX

### 6.1 Before Reset Information Display

Always show what will be affected:

```bash
# Good: Clear summary
$ git reset --hard HEAD~1
The following changes will be permanently discarded:
  M  src/main.rs (modified)
  M  tests/test.rs (modified)
  ?? new_file.txt (untracked)

Are you sure? [y/N]

# Better: Show diff
$ git diff --cached HEAD~1..HEAD
diff --git a/src/main.rs b/src/main.rs
index abc123..def456 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,5 +1,5 @@
 fn main() {
-    println!("Hello");
+    println!("Hello, World!");
 }
```

### 6.2 Warning Messages for Destructive Operations

```bash
# Clear danger indicators
WARNING: This is a destructive operation that cannot be undone.
The following files will be permanently deleted:
  /path/to/important/file

# Safety checks
$ git reset --hard HEAD~1
error: You have unmerged paths.
  (fix conflicts and mark resolved with "git add/rm")
```

### 6.3 Confirmation Strategies

#### Tiered Confirmation
```bash
# First level: Simple warning
$ docker system prune
WARNING! This will remove all stopped containers.
Continue? [y/N]

# Second level: Detailed summary
$ docker system prune -a
Would remove:
  - 12 containers
  - 8 images
  - 2 volumes
Reclaim 2.3GB
Continue? [y/N]
```

#### Force Flag Pattern
```bash
# Default: Safe with confirmation
command reset

# Explicit: Force without confirmation
command reset --force
```

### 6.4 Error Handling for Invalid Targets

```bash
# Specific error messages
$ git reset HEAD~999
fatal: ambiguous argument 'HEAD~999': unknown revision or path not in the working tree

$ git reset --hard invalid-ref
fatal: invalid reference: invalid-ref

# Context-aware errors
$ git reset --soft HEAD~1
error: You have staged changes that would be discarded by this reset.
Consider 'git reset --mixed' instead.
```

### 6.5 Post-Reset State Display

Always show the result:

```bash
# Good: Simple status
$ git reset --mixed HEAD~1
Unstaged changes after reset:
M   src/main.rs
M   tests/test.rs

# Better: Detailed state
$ git status
On branch main
Changes not staged for commit:
  modified:   src/main.rs
  modified:   tests/test.rs

no changes added to commit (use "git add" and/or "git commit -a")
```

## 7. Jin Reset Implementation Analysis

### 7.1 Current Jin Reset Command Structure

From `src/cli/args.rs`:

```rust
pub struct ResetCommand {
    /// Paths to reset (optional, defaults to all)
    #[arg(value_name = "PATH", num_args(0..))]
    pub paths: Vec<PathBuf>,

    /// Route to mode layer
    #[arg(long)]
    pub mode: bool,

    /// Route to scope layer
    #[arg(long, value_name = "SCOPE")]
    pub scope: Option<String>,

    /// Route to project layer
    #[arg(long)]
    pub project: bool,

    /// Keep changes in staging area
    #[arg(long)]
    pub soft: bool,

    /// Unstage but keep in workspace (default)
    #[arg(long, conflicts_with = "soft", conflicts_with = "hard")]
    pub mixed: bool,

    /// Discard all changes
    #[arg(long, conflicts_with = "soft")]
    pub hard: bool,
}
```

### 7.2 Patterns Followed

1. **Git-style modes**: ✅ Implements `--soft`, `--mixed`, `--hard`
2. **Layer targeting**: ✅ Has `--mode`, `--scope`, `--project` flags
3. **Path specificity**: ✅ Supports optional path arguments
4. **Conflict resolution**: ✅ Uses `conflicts_with` for mutually exclusive flags

### 7.3 Missing Best Practices

1. **No confirmation prompts**: Jin should add interactive confirmation for destructive operations
2. **No pre-reset information display**: Should show what will be affected
3. **Limited error context**: Could provide more specific error messages
4. **No post-reset summary**: Should show the state after reset

## 8. Recommendations for Jin Reset Implementation

### 8.1 Add Interactive Confirmation

```rust
// Before destructive operations
if cmd.hard || cmd.soft {
    println!("WARNING: This is a destructive operation that cannot be undone.");
    println!("The following layers will be affected:");

    if cmd.mode {
        println!("  - Mode layer");
    }
    // ... other layers

    if !cmd.paths.is_empty() {
        println!("Files to be reset:");
        for path in &cmd.paths {
            println!("  - {}", path.display());
        }
    }

    if !interactive_confirm("Continue? [y/N]: ") {
        return Ok(());
    }
}
```

### 8.2 Pre-Reset Information Display

```rust
fn show_reset_preview(cmd: &ResetCommand, repo: &JinRepo) -> Result<()> {
    let target_layer = determine_target_layer(cmd, repo)?;
    let staged_files = get_staged_files(&target_layer)?;
    let working_changes = get_working_changes(&target_layer)?;

    println!("Reset preview for layer: {}", target_layer.display());

    if !staged_files.is_empty() {
        println!("Staged files to be reset:");
        for file in &staged_files {
            println!("  - {}", file.display());
        }
    }

    if cmd.hard && !working_changes.is_empty() {
        println!("Working tree changes to be discarded:");
        for file in &working_changes {
            println!("  - {}", file.display());
        }
    }

    Ok(())
}
```

### 8.3 Enhanced Error Messages

```rust
// Specific error for invalid commit references
fn validate_reset_target(target: &str) -> Result<()> {
    if !is_valid_ref(repo, target) {
        bail!("Invalid reset target: '{}'\nUse 'jin log' to see available commits", target);
    }
    Ok(())
}

// Context-aware error for staged changes
if cmd.soft && has_staged_changes(&target_layer) {
    bail!("Cannot use --soft reset with staged changes.\nUse --mixed to unstage changes first.");
}
```

### 8.4 Post-Reset Summary

```rust
fn show_reset_summary(cmd: &ResetCommand, result: &ResetResult) {
    println!("Reset completed successfully");

    if !cmd.paths.is_empty() {
        println!("{} files reset in {}", cmd.paths.len(), result.layer.display());
    } else {
        println!("Entire layer reset: {}", result.layer.display());
    }

    if result.staged_count > 0 {
        println!("{} changes unstage", result.staged_count);
    }

    if result.discarded_count > 0 {
        println!("{} changes discarded", result.discarded_count);
    }

    if let Some(new_head) = result.new_head {
        println!("HEAD is now at: {}", new_head);
    }
}
```

### 8.5 Dry-Run Support

```rust
pub struct ResetCommand {
    // ... existing fields

    /// Preview changes without applying
    #[arg(long)]
    pub dry_run: bool,
}

// Implementation
fn execute_reset(cmd: &ResetCommand) -> Result<()> {
    if cmd.dry_run {
        return show_reset_preview(cmd, repo);
    }

    // ... actual reset logic
}
```

## 9. Conclusion

Git's reset command provides the most comprehensive and well-thought-out patterns for reset functionality. Key takeaways for Jin's implementation:

1. **Follow Git semantics**: The `--soft`/`--mixed`/`--hard` pattern is widely understood and should be maintained
2. **Layer targeting is innovative**: Jin's layer-specific reset is a unique and valuable feature
3. **Safety first**: Always provide clear warnings and confirmation for destructive operations
4. **Show, don't tell**: Display what will be affected before performing the operation
5. **Comprehensive error handling**: Provide specific, actionable error messages
6. **Post-operation summary**: Show the result state after completion

By combining Git's proven patterns with Jin's innovative layer system, the reset command can provide both familiarity and unique value to users.

## Sources

1. [Git Documentation - git-reset](https://git-scm.com/docs/git-reset)
2. [Docker Documentation - docker system prune](https://docs.docker.com/engine/reference/commandline/system_prune/)
3. [Kubernetes Documentation - kubeadm reset](https://kubernetes.io/docs/reference/setup-tools/kubeadm/kubeadm-reset/)
4. [Cargo Documentation - cargo clean](https://doc.rust-lang.org/cargo/commands/cargo-clean.html)
5. [Clap v4 Documentation](https://docs.rs/clap/4.5/clap/)
6. [Gitoxide Repository](https://github.com/GitoxideLabs/gitoxide)
7. [Helix Editor Documentation](https://helix-editor.com/)