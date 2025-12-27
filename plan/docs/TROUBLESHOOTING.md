# Jin Troubleshooting Guide

Common issues and solutions for Jin. If you don't find your issue here, please [open an issue](https://github.com/jin/jin/issues).

## Table of Contents

- [Installation Issues](#installation-issues)
- [Layer Confusion](#layer-confusion)
- [Merge Conflicts](#merge-conflicts)
- [Remote Operations](#remote-operations)
- [File Operations](#file-operations)
- [Performance Issues](#performance-issues)
- [State Corruption](#state-corruption)

---

## Installation Issues

### Q: `cargo install jin` fails with compile error

**Symptom:**
```
error: failed to compile `jin v0.1.0`
```

**Possible Causes & Solutions:**

1. **Rust version too old**
   ```bash
   $ rustc --version
   rustc 1.65.0  # Too old

   # Update Rust
   $ rustup update
   $ cargo install jin
   ```

   Jin requires Rust 1.70.0+

2. **Missing system dependencies**

   On Linux, ensure you have build essentials:
   ```bash
   # Ubuntu/Debian
   $ sudo apt install build-essential pkg-config libssl-dev

   # Fedora
   $ sudo dnf install gcc pkg-config openssl-devel

   # Arch
   $ sudo pacman -S base-devel openssl
   ```

3. **Cargo cache corruption**
   ```bash
   $ rm -rf ~/.cargo/registry
   $ cargo install jin
   ```

---

### Q: Command not found after installation

**Symptom:**
```bash
$ jin --version
bash: jin: command not found
```

**Solution:**

Ensure `~/.cargo/bin` is in your PATH:

```bash
# Check if in PATH
$ echo $PATH | grep cargo

# If not, add to your shell config
# For bash:
$ echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
$ source ~/.bashrc

# For zsh:
$ echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
$ source ~/.zshrc

# Verify
$ jin --version
jin 0.1.0
```

---

## Layer Confusion

### Q: Changes not appearing after `jin add`

**Symptom:**
```bash
$ jin add config.json --mode
$ jin apply
# config.json not updated in workspace
```

**Cause**: File was staged but not committed.

**Solution:**
```bash
# Check status
$ jin status
Staged changes: 1 file
  - config.json → mode

# Commit the staged file
$ jin commit -m "Add config"

# Now apply
$ jin apply
Applied 1 file to workspace
```

**Remember**: Jin workflow is `add → commit → apply`, similar to Git's `add → commit`.

---

### Q: Which layer am I committing to?

**Symptom:**
Unsure which layer will receive the commit based on flags used.

**Solution:**

Use this reference table:

| Flags | Target Layer | Precedence |
|-------|--------------|------------|
| (none) | Project Base | 7 |
| `--mode` | Mode Base | 2 |
| `--mode --project` | Mode → Project | 5 |
| `--scope <s>` | Scope Base | 6 |
| `--mode --scope <s>` | Mode → Scope | 3 |
| `--mode --scope <s> --project` | Mode → Scope → Project | 4 |
| `--global` | Global Base | 1 |

**Tip**: Use `jin status` before committing to see where staged files will go:
```bash
$ jin status
Staged changes: 1 file
  - config.json → mode+project
```

See [Layer Routing](LAYER_SYSTEM.md#layer-routing) for complete details.

---

### Q: How do I see layer precedence?

**Symptom:**
Want to understand which layers are active and their order.

**Solution:**
```bash
$ jin layers
Active layers (precedence order, lowest to highest):
  1. global/
  2. mode/claude/
  5. mode/claude/project/jin/
  7. project/jin/
  9. workspace/ (derived)

Files per layer:
  global/: 2 files
  mode/claude/: 5 files
  ...
```

For detailed precedence explanation, see [Layer System](LAYER_SYSTEM.md#precedence-rules).

---

### Q: My override isn't working

**Symptom:**
Added override in higher layer but lower layer value still appears.

**Cause**: Higher layer in numeric order doesn't always mean higher precedence. Check the [precedence table](LAYER_SYSTEM.md#the-9-layer-hierarchy).

**Solution:**

Check actual precedence:
```bash
$ jin layers
Active layers (precedence order, lowest to highest):
  2. mode/dev/
  7. project/myapp/
  ...
```

Note: Project Base (Layer 7) has HIGHER precedence than Mode Base (Layer 2), even though 2 < 7 numerically.

**Correct precedence** (from lowest to highest):
1. Global Base
2. Mode Base
3. Mode → Scope
4. Mode → Scope → Project
5. Mode → Project
6. Scope Base
7. Project Base
8. User Local
9. Workspace Active

---

## Merge Conflicts

### Q: Merge conflict in config file after `jin pull`

**Symptom:**
```bash
$ jin pull
Error: Merge conflict in .dev/config.json
```

**Solution:**

1. View the conflict:
   ```bash
   $ jin diff
   Conflict in .dev/config.json:
   <<<<<<< local
   {"debug": true}
   =======
   {"debug": false, "verbose": true}
   >>>>>>> remote
   ```

2. Manually resolve the conflict:
   ```bash
   $ nano .dev/config.json
   # Edit to desired state:
   {"debug": true, "verbose": true}
   ```

3. Stage and commit the resolution:
   ```bash
   $ jin add .dev/config.json --mode
   $ jin commit -m "Resolve merge conflict"
   ```

---

### Q: File deleted unexpectedly after `jin apply`

**Symptom:**
File that existed is gone after applying layers.

**Cause**: A higher layer set the file or key to `null`, which deletes it.

**Solution:**

1. Check which layer deleted it:
   ```bash
   $ jin diff mode/dev mode/prod
   - config.json: {"key": "value"}
   + config.json: null
   ```

2. Remove the `null` entry from the higher layer:
   ```bash
   $ jin mode use prod
   $ jin diff --staged
   # Find the layer with null value

   $ jin reset --mode --hard
   # Or edit and re-commit without null
   ```

**Remember**: In Jin's deep merge, `null` explicitly deletes a key or file.

---

### Q: Array values not merging as expected

**Symptom:**
Arrays from multiple layers are replacing instead of merging.

**Explanation:**

Jin handles arrays differently based on whether they have keys:

**Unkeyed arrays** (replaced):
```json
// Layer 1
{"items": [1, 2, 3]}

// Layer 2
{"items": [4, 5]}

// Result
{"items": [4, 5]}  // Layer 2 replaces
```

**Keyed arrays** (merged by key):
```json
// Layer 1
{"items": [
  {"id": 1, "name": "foo"},
  {"id": 2, "name": "bar"}
]}

// Layer 2
{"items": [
  {"id": 2, "name": "baz"},
  {"id": 3, "name": "qux"}
]}

// Result
{"items": [
  {"id": 1, "name": "foo"},
  {"id": 2, "name": "baz"},
  {"id": 3, "name": "qux"}
]}
```

**Solution**: If you need array merging, use objects with `id` or `name` keys.

---

## Remote Operations

### Q: `jin fetch` says "No remote configured"

**Symptom:**
```bash
$ jin fetch
Error: No remote repository configured
```

**Solution:**
```bash
# Link to a remote repository first
$ jin link https://github.com/your-team/jin-config
Linked to remote

# Now fetch
$ jin fetch
Fetched updates from remote
```

---

### Q: Push rejected: non-fast-forward

**Symptom:**
```bash
$ jin push
Error: Push rejected (non-fast-forward)
```

**Cause**: Remote has commits you don't have locally.

**Solution:**

1. Pull and merge first:
   ```bash
   $ jin pull
   Merged updates from remote
   ```

2. Then push:
   ```bash
   $ jin push
   Pushed layers to remote
   ```

**Alternative** (force push, use with caution):
```bash
$ jin push --force
Warning: Force pushing to remote
```

---

### Q: Sync fails with authentication error

**Symptom:**
```bash
$ jin sync
Error: Authentication failed for remote repository
```

**Solution:**

1. Verify remote URL:
   ```bash
   $ jin remote -v
   origin  https://github.com/team/jin-config (fetch)
   origin  https://github.com/team/jin-config (push)
   ```

2. Ensure SSH key or credentials are configured:
   ```bash
   # For SSH
   $ ssh -T git@github.com

   # For HTTPS, configure credential helper
   $ git config --global credential.helper cache
   ```

3. Update remote URL if needed:
   ```bash
   $ jin link git@github.com:team/jin-config --force
   ```

---

## File Operations

### Q: `jin add` fails: "file not found"

**Symptom:**
```bash
$ jin add config.json --mode
Error: File not found: config.json
```

**Cause**: File path is relative but Jin can't find it.

**Solution:**

1. Check current directory:
   ```bash
   $ pwd
   /home/user/project

   $ ls config.json
   config.json
   ```

2. Use correct relative or absolute path:
   ```bash
   # Relative to project root
   $ jin add ./config.json --mode

   # Or absolute path
   $ jin add /home/user/project/config.json --mode
   ```

---

### Q: Accidentally committed to Git instead of Jin

**Symptom:**
File that should be Jin-managed is in Git repository.

**Solution:**

1. Remove from Git:
   ```bash
   $ git rm --cached .dev/config.json
   $ git commit -m "Remove from Git, manage with Jin"
   ```

2. Add to `.gitignore`:
   ```bash
   $ echo ".dev/" >> .gitignore
   $ git add .gitignore
   $ git commit -m "Ignore Jin-managed files"
   ```

3. Add to Jin:
   ```bash
   $ jin add .dev/config.json --mode
   $ jin commit -m "Add to Jin"
   $ jin apply
   ```

**Prevention**: Jin automatically adds `.jin/` to `.gitignore`, but you should manually ignore Jin-managed directories.

---

### Q: How do I migrate existing files to Jin?

**Symptom:**
Have existing config files in Git, want to move to Jin.

**Solution:**

```bash
# Step 1: Copy files to preserve originals
$ cp .vscode/settings.json .vscode/settings.json.backup

# Step 2: Remove from Git
$ git rm --cached .vscode/settings.json
$ echo ".vscode/" >> .gitignore
$ git commit -m "Migrate .vscode to Jin"

# Step 3: Add to Jin
$ jin init
$ jin mode create vscode
$ jin mode use vscode
$ jin add .vscode/settings.json --mode
$ jin commit -m "Migrate VS Code settings to Jin"

# Step 4: Apply
$ jin apply
```

---

## Performance Issues

### Q: `jin apply` is slow

**Symptom:**
Applying layers takes more than a few seconds.

**Possible Causes & Solutions:**

1. **Large number of files**

   Check how many files are managed:
   ```bash
   $ jin list
   # Shows all files across layers
   ```

   Consider using `.jinignore` (similar to `.gitignore`) to exclude large directories.

2. **Complex deep merging**

   Very deeply nested JSON/YAML structures take longer to merge. Consider flattening if possible.

3. **Slow disk I/O**

   Jin reads/writes to `~/.jin/` and `.jin/`. Ensure these aren't on slow network drives.

---

## State Corruption

### Q: Jin state is corrupted or inconsistent

**Symptom:**
Errors like "invalid layer reference", "orphaned staging entry", or unexpected behavior.

**Solution:**

Run the repair command:

```bash
# Check what would be repaired
$ jin repair --dry-run
Would repair:
  - Remove orphaned staging entry: old-file.json
  - Fix invalid context reference: mode/deleted-mode
  - Rebuild layer index

# Apply repairs
$ jin repair
Repaired Jin state:
  - Removed 1 orphaned entry
  - Fixed 1 invalid reference
  - Rebuilt layer index

Repair complete
```

If repair doesn't fix the issue:

1. Backup your Jin repository:
   ```bash
   $ cp -r ~/.jin ~/.jin.backup
   ```

2. Verify Git repository integrity:
   ```bash
   $ cd ~/.jin
   $ git fsck
   ```

3. If still broken, consider re-initializing (WARNING: loses history):
   ```bash
   $ rm -rf ~/.jin
   $ jin init
   # Re-add configurations
   ```

---

### Q: `.jin/context.json` is invalid

**Symptom:**
```bash
$ jin apply
Error: Invalid context file
```

**Solution:**

1. Check the context file:
   ```bash
   $ cat .jin/context.json
   ```

2. Fix manually or regenerate:
   ```bash
   # Backup
   $ cp .jin/context.json .jin/context.json.backup

   # Regenerate
   $ rm .jin/context.json
   $ jin mode use <mode-name>
   ```

3. Or edit manually:
   ```json
   {
     "mode": "dev",
     "scope": null,
     "project": "my-project"
   }
   ```

---

## Getting More Help

If your issue isn't covered here:

1. **Check the documentation:**
   - [Getting Started](GETTING_STARTED.md)
   - [Layer System](LAYER_SYSTEM.md)
   - [Commands](COMMANDS.md)
   - [Workflows](WORKFLOWS.md)

2. **Search existing issues:**
   - [GitHub Issues](https://github.com/jin/jin/issues)

3. **Ask for help:**
   - [Open a new issue](https://github.com/jin/jin/issues/new)
   - Provide:
     - Jin version (`jin --version`)
     - Operating system
     - Steps to reproduce
     - Relevant output from `jin status` and `jin layers`

4. **Debug mode:**
   ```bash
   $ RUST_LOG=debug jin <command>
   # Provides verbose output for debugging
   ```
