# Jin Command Reference

Complete reference for all Jin commands, organized by category.

## Table of Contents

- [Core Commands](#core-commands)
- [Mode Management](#mode-management)
- [Scope Management](#scope-management)
- [Workspace Operations](#workspace-operations)
- [Inspection & Analysis](#inspection--analysis)
- [Remote Sync](#remote-sync)
- [Utility](#utility)

---

## Core Commands

### jin init

**Purpose**: Initialize Jin in the current project

**Usage**: `jin init`

**Description**:
Creates a `.jin/` directory in the current project with:
- `context.json` - tracks active mode and scope
- `staging/` - temporary staging area for uncommitted changes
- Initializes the global Jin repository at `~/.jin/` if it doesn't exist

**Examples**:
```bash
$ cd my-project/
$ jin init
Jin initialized in current project
```

**See Also**: `jin status`, `jin context`

---

### jin add

**Purpose**: Stage files to the appropriate layer based on flags

**Usage**: `jin add <files>... [flags]`

**Arguments**:
- `<files>` - One or more file paths to stage

**Flags**:
- `--mode` - Target mode layer (Layer 2)
- `--scope <scope>` - Target scope layer (Layer 6) or combined with mode
- `--project` - Combine with mode/scope to target project-specific layer
- `--global` - Target global layer (Layer 1)

**Description**:
Stages files to Jin's staging area. The combination of flags determines which layer the files will be committed to. Without flags, files are staged to the Project Base layer (Layer 7).

**Examples**:

```bash
# Stage to project base (default)
$ jin add config.json
Staged config.json to project layer

# Stage to mode base
$ jin add .dev/settings.json --mode
Staged .dev/settings.json to mode layer

# Stage to mode+project layer
$ jin add .dev/settings.json --mode --project
Staged .dev/settings.json to mode+project layer

# Stage to global layer
$ jin add defaults.json --global
Staged defaults.json to global layer

# Stage multiple files
$ jin add file1.json file2.yaml --mode
Staged 2 files to mode layer
```

**See Also**: `jin commit`, `jin status`, [Layer Routing](LAYER_SYSTEM.md#layer-routing)

---

### jin commit

**Purpose**: Atomically commit staged files to their target layers

**Usage**: `jin commit -m <message> [flags]`

**Flags**:
- `-m, --message <message>` - Commit message (required)
- `--dry-run` - Show what would be committed without committing

**Description**:
Commits all staged files to their respective target layers atomically. If any part of the commit fails, the entire operation is rolled back.

**Examples**:

```bash
# Commit staged changes
$ jin commit -m "Add development configuration"
Committed to mode/dev layer
Staging area cleared

# Preview commit without executing
$ jin commit -m "Test commit" --dry-run
Would commit to mode/dev layer:
  - .dev/config.json
  - .dev/settings.yaml
```

**See Also**: `jin add`, `jin status`, `jin reset`

---

### jin status

**Purpose**: Show workspace state and active contexts

**Usage**: `jin status`

**Description**:
Displays:
- Active mode and scope
- Current project
- Workspace state (clean/dirty)
- Staged changes
- Layer summary (files per layer)

**Examples**:

```bash
$ jin status
Active mode: dev
Active scope: env:prod
Project: my-project

Workspace state: Clean
Staged changes: 2 files
  - config.json → mode+project
  - settings.yaml → mode

Layer summary:
  global: 1 file
  mode/dev: 3 files
  mode/dev/project/my-project: 1 file
```

**See Also**: `jin context`, `jin layers`, `jin diff`

---

### jin context

**Purpose**: Show or set the active context (mode and scope)

**Usage**: `jin context`

**Description**:
Displays the current active mode and scope for the project.

**Examples**:

```bash
$ jin context
Active mode: claude
Active scope: language:rust
Project: jin
```

**See Also**: `jin mode`, `jin scope`, `jin status`

---

## Mode Management

Modes represent broad development environments or tool configurations (e.g., `claude`, `cursor`, `dev`, `prod`).

### jin mode create

**Purpose**: Create a new mode

**Usage**: `jin mode create <name>`

**Arguments**:
- `<name>` - Name of the mode to create

**Examples**:

```bash
$ jin mode create claude
Mode 'claude' created

$ jin mode create dev
Mode 'dev' created
```

**See Also**: `jin mode use`, `jin mode list`

---

### jin mode use

**Purpose**: Activate a mode for the current project

**Usage**: `jin mode use <name>`

**Arguments**:
- `<name>` - Name of the mode to activate

**Description**:
Sets the active mode for the current project. This affects which layers are merged during `jin apply`.

**Examples**:

```bash
$ jin mode use claude
Mode 'claude' activated

$ jin apply
Applied 5 files from mode/claude layer
```

**See Also**: `jin mode create`, `jin mode unset`, `jin apply`

---

### jin mode list

**Purpose**: List all available modes

**Usage**: `jin mode list`

**Examples**:

```bash
$ jin mode list
Available modes:
  * claude (active)
    cursor
    dev
    prod
```

**See Also**: `jin mode create`, `jin mode show`

---

### jin mode delete

**Purpose**: Delete a mode and all its configurations

**Usage**: `jin mode delete <name>`

**Arguments**:
- `<name>` - Name of the mode to delete

**Examples**:

```bash
$ jin mode delete old-mode
Mode 'old-mode' deleted
```

**Warning**: This permanently deletes all configurations in the mode layer.

**See Also**: `jin mode create`, `jin mode list`

---

### jin mode show

**Purpose**: Show the currently active mode

**Usage**: `jin mode show`

**Examples**:

```bash
$ jin mode show
Active mode: claude
```

**See Also**: `jin mode list`, `jin context`

---

### jin mode unset

**Purpose**: Deactivate the current mode

**Usage**: `jin mode unset`

**Description**:
Deactivates the mode for the current project. The mode layer will no longer be included during `jin apply`.

**Examples**:

```bash
$ jin mode unset
Mode deactivated

$ jin apply
Applied 2 files (no mode layer active)
```

**See Also**: `jin mode use`, `jin apply`

---

## Scope Management

Scopes provide contextual refinement within modes (e.g., `env:dev`, `language:rust`, `feature:auth`).

### jin scope create

**Purpose**: Create a new scope

**Usage**: `jin scope create <name> [--mode <mode>]`

**Arguments**:
- `<name>` - Name of the scope to create

**Flags**:
- `--mode <mode>` - Associate with a specific mode

**Examples**:

```bash
$ jin scope create env:prod
Scope 'env:prod' created

$ jin scope create language:rust --mode claude
Scope 'language:rust' created and associated with mode 'claude'
```

**See Also**: `jin scope use`, `jin scope list`

---

### jin scope use

**Purpose**: Activate a scope for the current project

**Usage**: `jin scope use <name>`

**Arguments**:
- `<name>` - Name of the scope to activate

**Examples**:

```bash
$ jin scope use env:prod
Scope 'env:prod' activated

$ jin apply
Applied configurations with scope/env:prod layer
```

**See Also**: `jin scope create`, `jin scope unset`

---

### jin scope list

**Purpose**: List all available scopes

**Usage**: `jin scope list`

**Examples**:

```bash
$ jin scope list
Available scopes:
  * env:prod (active)
    env:dev
    language:rust
    language:javascript
```

**See Also**: `jin scope create`, `jin scope show`

---

### jin scope delete

**Purpose**: Delete a scope and all its configurations

**Usage**: `jin scope delete <name>`

**Arguments**:
- `<name>` - Name of the scope to delete

**Examples**:

```bash
$ jin scope delete old-scope
Scope 'old-scope' deleted
```

**Warning**: This permanently deletes all configurations in the scope layer.

**See Also**: `jin scope create`, `jin scope list`

---

### jin scope show

**Purpose**: Show the currently active scope

**Usage**: `jin scope show`

**Examples**:

```bash
$ jin scope show
Active scope: env:prod
```

**See Also**: `jin scope list`, `jin context`

---

### jin scope unset

**Purpose**: Deactivate the current scope

**Usage**: `jin scope unset`

**Examples**:

```bash
$ jin scope unset
Scope deactivated
```

**See Also**: `jin scope use`, `jin apply`

---

## Workspace Operations

### jin apply

**Purpose**: Apply merged layers to the workspace

**Usage**: `jin apply [flags]`

**Flags**:
- `--force` - Force apply even if workspace is dirty
- `--dry-run` - Show what would be applied without applying

**Description**:
Merges all active layers according to precedence rules and writes the result to the workspace. This is the command that makes Jin configurations active in your working directory.

**Examples**:

```bash
# Apply configurations
$ jin apply
Applied 5 files to workspace:
  .dev/config.json
  .dev/settings.yaml
  .vscode/settings.json
  .claude/config.json
  .cursor/settings.json

# Preview what would be applied
$ jin apply --dry-run
Would apply 5 files:
  .dev/config.json (merged from 3 layers)
  .dev/settings.yaml (merged from 2 layers)
  ...

# Force apply over dirty workspace
$ jin apply --force
Warning: Overwriting uncommitted changes
Applied 5 files to workspace
```

**See Also**: `jin status`, `jin diff`, `jin reset`

---

### jin reset

**Purpose**: Reset staged or committed changes

**Usage**: `jin reset [flags]`

**Flags**:
- `--soft` - Keep changes in staging
- `--mixed` - Unstage but keep in workspace (default)
- `--hard` - Discard all changes
- `--mode` - Reset mode layer
- `--scope <scope>` - Reset scope layer
- `--project` - Reset project layer

**Description**:
Resets changes in staging or specific layers. Similar to Git's reset command.

**Examples**:

```bash
# Unstage all changes (keep in workspace)
$ jin reset
Unstaged 2 files

# Discard all staged changes
$ jin reset --hard
Discarded 2 staged files

# Reset mode layer
$ jin reset --mode --hard
Reset mode/dev layer

# Keep staged changes
$ jin reset --soft
Reset commit pointer (staged files preserved)
```

**See Also**: `jin add`, `jin commit`, `jin apply`

---

## Inspection & Analysis

### jin diff

**Purpose**: Show differences between layers or workspace

**Usage**: `jin diff [layer1] [layer2] [flags]`

**Arguments**:
- `[layer1]` - First layer to compare (optional)
- `[layer2]` - Second layer to compare (optional)

**Flags**:
- `--staged` - Show staged changes

**Description**:
Shows differences between layers, workspace, or staged changes.

**Examples**:

```bash
# Show staged changes
$ jin diff --staged
diff --git a/config.json b/config.json
+++ config.json (staged)
@@ -1,3 +1,4 @@
 {
   "debug": true,
+  "log_level": "trace"
 }

# Compare two layers
$ jin diff mode/dev mode/prod
diff --git a/config.json b/config.json
...

# Show workspace vs. last commit
$ jin diff
Modified: config.json
  - debug: false → true
```

**See Also**: `jin status`, `jin log`

---

### jin log

**Purpose**: Show commit history for a layer

**Usage**: `jin log [--layer <layer>] [--count <n>]`

**Flags**:
- `--layer <layer>` - Show history for specific layer
- `--count <n>` - Number of entries to show (default: 10)

**Examples**:

```bash
# Show recent commits
$ jin log
commit a1b2c3d (mode/dev)
Author: You <you@example.com>
Date:   2025-12-27 10:30:00

    Add debug configuration

commit d4e5f6g (mode/dev)
Author: You <you@example.com>
Date:   2025-12-26 15:20:00

    Initial dev mode setup

# Show specific layer history
$ jin log --layer mode/dev --count 5
(shows last 5 commits for mode/dev layer)
```

**See Also**: `jin diff`, `jin status`

---

### jin layers

**Purpose**: Show current layer composition and active layers

**Usage**: `jin layers`

**Description**:
Displays which layers are active and how they're composed for the current project context.

**Examples**:

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
  mode/claude/project/jin/: 1 file
  project/jin/: 3 files
```

**See Also**: `jin status`, `jin context`, [Layer System](LAYER_SYSTEM.md)

---

### jin list

**Purpose**: List available modes, scopes, and projects

**Usage**: `jin list`

**Examples**:

```bash
$ jin list
Modes:
  - claude
  - cursor
  - dev
  - prod

Scopes:
  - env:dev
  - env:prod
  - language:rust
  - language:javascript

Projects:
  - jin
  - my-app
  - my-lib
```

**See Also**: `jin mode list`, `jin scope list`

---

### jin repair

**Purpose**: Repair Jin state and fix inconsistencies

**Usage**: `jin repair [--dry-run]`

**Flags**:
- `--dry-run` - Show what would be repaired without repairing

**Description**:
Scans Jin's internal state and fixes:
- Orphaned staging entries
- Missing layer references
- Corrupted context files
- Invalid layer trees

**Examples**:

```bash
# Check what would be repaired
$ jin repair --dry-run
Would repair:
  - Remove orphaned staging entry: config.json
  - Fix invalid context reference: mode/old-mode

# Repair issues
$ jin repair
Repaired Jin state:
  - Removed 1 orphaned staging entry
  - Fixed 1 context reference
```

**See Also**: `jin status`

---

## Remote Sync

### jin link

**Purpose**: Link to a shared Jin configuration repository

**Usage**: `jin link <url> [--force]`

**Arguments**:
- `<url>` - Remote repository URL (Git)

**Flags**:
- `--force` - Force update existing remote

**Description**:
Configures a remote Git repository for sharing Jin configurations across machines or with teammates.

**Examples**:

```bash
$ jin link https://github.com/team/jin-config
Linked to remote: https://github.com/team/jin-config

$ jin link https://github.com/team/new-config --force
Updated remote to: https://github.com/team/new-config
```

**See Also**: `jin push`, `jin pull`, `jin fetch`

---

### jin fetch

**Purpose**: Fetch updates from the remote repository

**Usage**: `jin fetch`

**Description**:
Downloads layer updates from the remote repository without merging them. Similar to `git fetch`.

**Examples**:

```bash
$ jin fetch
Fetching from remote...
Fetched updates for:
  - mode/claude
  - mode/dev
  - global
```

**See Also**: `jin pull`, `jin link`

---

### jin pull

**Purpose**: Fetch and merge updates from remote

**Usage**: `jin pull`

**Description**:
Fetches updates from remote and merges them into local layers. Equivalent to `jin fetch` followed by merge.

**Examples**:

```bash
$ jin pull
Pulling from remote...
Merged updates:
  - mode/claude: 2 files updated
  - global: 1 file updated

Run `jin apply` to update workspace
```

**See Also**: `jin fetch`, `jin apply`, `jin sync`

---

### jin push

**Purpose**: Push local changes to remote

**Usage**: `jin push [--force]`

**Flags**:
- `--force` - Force push (overwrite remote)

**Description**:
Uploads local layer changes to the remote repository.

**Examples**:

```bash
$ jin push
Pushing to remote...
Pushed layers:
  - mode/claude
  - mode/dev
  - global

$ jin push --force
Warning: Force pushing to remote
Pushed layers (force)
```

**See Also**: `jin link`, `jin pull`

---

### jin sync

**Purpose**: Synchronize with remote (fetch + merge + apply)

**Usage**: `jin sync`

**Description**:
Convenience command that performs:
1. `jin fetch` - Download updates
2. Merge updates into local layers
3. `jin apply` - Update workspace

**Examples**:

```bash
$ jin sync
Syncing with remote...
Fetched updates from remote
Merged 3 layers
Applied 5 files to workspace

Workspace synchronized
```

**See Also**: `jin pull`, `jin push`, `jin apply`

---

## Utility

### jin completion

**Purpose**: Generate shell completion scripts

**Usage**: `jin completion <shell>`

**Arguments**:
- `<shell>` - Shell type (bash, zsh, fish, powershell)

**Description**:
Outputs completion script to stdout. Redirect to a file and source it to enable tab completion in your shell.

**Examples**:

```bash
# Bash
$ jin completion bash > /usr/local/share/bash-completion/completions/jin
$ source /usr/local/share/bash-completion/completions/jin

# Zsh
$ jin completion zsh > ~/.zsh/completions/_jin
$ source ~/.zsh/completions/_jin

# Fish
$ jin completion fish > ~/.config/fish/completions/jin.fish

# PowerShell
$ jin completion powershell > $PROFILE\..\Completions\jin_completion.ps1
```

**See Also**: Shell-specific completion documentation

---

## Command Quick Reference

| Category | Commands |
|----------|----------|
| **Core** | `init`, `add`, `commit`, `status`, `context` |
| **Mode** | `mode create`, `mode use`, `mode list`, `mode delete`, `mode show`, `mode unset` |
| **Scope** | `scope create`, `scope use`, `scope list`, `scope delete`, `scope show`, `scope unset` |
| **Workspace** | `apply`, `reset` |
| **Inspection** | `diff`, `log`, `layers`, `list`, `repair` |
| **Remote** | `link`, `fetch`, `pull`, `push`, `sync` |
| **Utility** | `completion` |

For workflow examples showing these commands in action, see [Common Workflows](WORKFLOWS.md).
