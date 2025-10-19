# Jin — Phantom Git Layer System

**Product Requirements Document (PRD)**
**Version:** 2.0
**Status:** Draft (Post-Correction & Expansion)
**Last Updated:** 2025-10-19

---

## 1. Overview

**Jin** is a meta-versioning layer built atop Git to manage *developer-specific and tool-specific configuration* in collaborative projects, without contaminating the main repository. Jin provides an isolated "phantom layer" for tracking ignored/untracked files that represent tooling states (AI configs, editor metadata, MCP servers, etc.).

The system allows developers to define **modes** (per-tool setups) and **scopes** (per-language or context overlays) and manage them across projects automatically.

---

## 2. Goals

* **Zero workflow disruption:** Developers continue using Git normally. Jin operations are opt-in and limited to ignored/untracked files.
* **Shared yet isolated tooling:** Each dev can have tool configurations applied without polluting the repo.
* **Automatic merge logic:** Jin must accurately merge JSON, YAML, TOML, and similar structured files across layers.
* **Single repository architecture:** All configuration data—global, per-mode, per-scope, and per-project—is housed in a single Git repo.
* **Scalability:** Multiple projects and teams can use the same Jin repo, allowing both shared and project-specific customizations.
* **Automatic .gitignore management:** Any file tracked by Jin is automatically added to `.gitignore` to prevent accidental Git tracking.

---

## 3. Key Concepts

| Concept         | Description                                                                                       |
| --------------- | ------------------------------------------------------------------------------------------------- |
| **Mode**        | Represents a specific AI/tooling environment (e.g. `claude`, `cursor`, `zed`).                    |
| **Scope**       | A refinement or contextual specialization of a mode (e.g. `language:javascript`, `infra:docker`). |
| **Project**     | The active Git repository detected from CWD. No explicit flag required.                           |
| **Layer**       | A level in the structured merge/commit hierarchy (see §4).                                        |
| **Phantom Git** | Jin's shadow Git tracking ignored/untracked files and mapping them to the correct layers.         |
| **Active Context** | The currently selected mode and/or scope that determines default targeting for `jin add` operations. |

---

## 4. Layer Architecture

### 4.1 Nine-Layer Hierarchy

Each layer refines or overrides the layer beneath it. Merge and commit precedence flows upward.

| # | Layer                  | Description                                                        | Example Location                                      |
| - | ---------------------- | ------------------------------------------------------------------ | ----------------------------------------------------- |
| 1 | Global Base            | Shared defaults for all projects and tools                         | `jin/global/`                                         |
| 2 | Mode Base              | Base configuration for a mode                                      | `jin/mode/claude/`                                    |
| 3 | Mode → Scope           | Mode-specific scoped configs                                       | `jin/mode/claude/scope/lang:js/`                      |
| 4 | Mode → Scope → Project | Project-specific modifications to a scoped mode                    | `jin/mode/claude/scope/lang:js/project/ui-dashboard/` |
| 5 | Mode → Project         | Project-specific modifications to a mode                           | `jin/mode/claude/project/ui-dashboard/`               |
| 6 | Scope Base             | Cross-mode defaults for a given scope type                         | `jin/scope/lang:js/`                                  |
| 7 | Project Base           | Project-specific global configuration                              | `jin/project/ui-dashboard/`                           |
| 8 | User Local             | Machine-specific overlays (unshared)                               | `~/.jin/local/`                                       |
| 9 | Workspace Active       | The currently checked-out and merged config view (in project tree) | `.jin/workspace/`                                     |

---

## 5. Core API Contract

### 5.1 Staging and Committing

**This is the fundamental API the entire tool is built on:**

- `jin add <files>` stages files (like `git add`)
- `jin commit -m "message"` commits staged files (like `git commit`)
- Files remain staged until committed
- Multiple `jin add` operations can occur before a single `jin commit`

### 5.2 Active Context

Jin maintains an **active context** that determines default targeting for operations:

- `jin mode use <mode>` activates a mode - all subsequent operations know this context
- `jin scope use <scope>` activates a scope - all subsequent operations know this context
- **After activation, no need to specify `--mode` or `--scope` flags unless overriding behavior**
- Context persists until explicitly changed or cleared
- Context is stored per-project in `.jin/context`

### 5.3 Automatic .gitignore Management

**Core Feature:** Any file added to Jin is automatically added to `.gitignore` to prevent Git from tracking it.

- Jin checks `.gitignore` before adding files
- If file is not already ignored, Jin appends it to `.gitignore`
- This prevents jin-tracked files from accidentally being committed to Git
- Jin-tracked files that are added to Git should be detected and handled (see §14 Backlog)

---

## 6. Layer Routing and Add Behavior

### 6.1 Flag Routing Rules (THE HAPPY PATH)

**This is the critical specification that must never be lost:**

| Command                                                  | Target Layer | Description                                              |
| -------------------------------------------------------- | ------------ | -------------------------------------------------------- |
| `jin add <file>`                                         | Layer 7      | Adds to project base layer (no mode/scope context)      |
| `jin add <file> --mode`                                  | Layer 2      | Adds to currently active mode base layer                 |
| `jin add <file> --mode --project`                        | Layer 5      | Adds to project-specific mode layer                      |
| `jin add <file> --scope=<scope>`                         | Layer 6      | Adds to untethered scope base (no mode)                  |
| `jin add <file> --mode --scope=<scope>`                  | Layer 3      | Adds to mode's scope layer                               |
| `jin add <file> --mode --scope=<scope> --project`        | Layer 4      | Adds to project-specific mode-scope layer                |

**Critical Rules:**

1. **No active mode + `--mode` flag = ERROR** - Must have an active mode set via `jin mode use`
2. **Passing multiple scopes is NOT PERMITTED** - Only one scope can be specified at a time
3. **`--project` is always implied by CWD** - The project is auto-detected from Git origin
4. **Flags modify behavior, active context provides defaults** - Use flags to override default behavior

### 6.2 Active Context Behavior Examples

```bash
# Set active context
jin mode use claude
jin scope use language:javascript

# After context is set:
jin add .claude/config.json
# → Targets: project/ui-dashboard/ (base project, ignores active context without flags)

jin add .claude/config.json --mode
# → Targets: mode/claude/project/ui-dashboard/ (uses active mode, ignores active scope)

jin add .claude/config.json --mode --scope=language:javascript
# → Targets: mode/claude/scope/language:javascript/project/ui-dashboard/

jin add .claude/config.json --scope=language:python
# → Targets: scope/language:python/ (untethered scope, no mode)
```

### 6.3 Error Conditions

- `jin add <file> --mode` when no mode is active → **ERROR: No active mode set. Use `jin mode use <mode>` first.**
- `jin add <file> --scope=a --scope=b` → **ERROR: Multiple scopes not permitted.**
- `jin add <file>` for a file already tracked by Git → **ERROR: File is tracked by Git. Use `jin import` to migrate.**

---

## 7. Mode and Scope Behavior

### 7.1 Mode

A mode defines a broad developer setup—typically per AI or editor.

**Mode Lifecycle:**

```bash
# Create mode (first-time setup)
jin mode create claude

# Activate mode (sets active context)
jin mode use claude

# Add files to mode base
jin add .claude/ CLAUDE.md --mode
jin commit -m "Add Claude mode base files"
# → commits to mode/claude/

# Add files to mode-project
jin add .claude/local-config.json --mode --project
jin commit -m "Add project-specific Claude config"
# → commits to mode/claude/project/ui-dashboard/

# Deactivate mode
jin mode unset

# List available modes
jin modes

# Remove mode
jin mode delete claude
```

### 7.2 Scope

A scope applies language- or domain-specific deltas on top of a mode or standalone.

**Scope Lifecycle:**

```bash
# Create scope for a mode
jin scope create language:javascript --mode=claude

# Activate scope (sets active context)
jin scope use language:javascript

# Add files to mode-scope
jin add .claude/commands --mode --scope=language:javascript
jin commit -m "Add Claude JS scope helpers"
# → commits to mode/claude/scope/language:javascript/

# Add files to untethered scope (no mode)
jin add .editorconfig --scope=language:javascript
jin commit -m "Add JS editorconfig"
# → commits to scope/language:javascript/

# Deactivate scope
jin scope unset

# List available scopes
jin scopes

# Remove scope
jin scope delete language:javascript
```

### 7.3 Multiple Active Contexts

- **One active mode at a time** (enforced)
- **One active scope at a time** (enforced)
- **Both can be active simultaneously** (mode + scope)
- Use `jin status` to see current active contexts

---

## 8. Project-Specific Customization

Each project's unique changes (e.g., AngularJS vs React) are stored automatically in project-specific layers inferred from the Git origin name (e.g. `ui-dashboard`).

**Example:**

```bash
# inside ui-dashboard repo
# Project auto-detected from git remote origin

jin add .vscode/settings.json
# → project/ui-dashboard/

jin mode use claude
jin add .claude/config.json --mode --project
# → mode/claude/project/ui-dashboard/

jin scope use language:javascript
jin add .claude/mcp.json --mode --scope=language:javascript --project
# → mode/claude/scope/language:javascript/project/ui-dashboard/
```

---

## 9. Core Commands

### 9.1 Initialization

```bash
jin init                              # Initialize Jin in current project
jin link <repo-url>                   # Link to shared Jin config repo
```

### 9.2 Staging & Committing

```bash
jin add <files> [--mode] [--scope=<scope>] [--project]
                                      # Stage files to appropriate layer
jin commit -m "message"               # Commit staged files
jin reset [--soft|--mixed|--hard]    # Reset staged/committed changes
```

### 9.3 Mode Management

```bash
jin mode create <mode>                # Create new mode
jin mode use <mode>                   # Activate mode (set context)
jin mode unset                        # Deactivate current mode
jin mode delete <mode>                # Delete mode
jin modes                             # List all available modes
```

### 9.4 Scope Management

```bash
jin scope create <scope> [--mode=<mode>]
                                      # Create new scope (optionally tied to mode)
jin scope use <scope>                 # Activate scope (set context)
jin scope unset                       # Deactivate current scope
jin scope delete <scope>              # Delete scope
jin scopes                            # List all available scopes
```

### 9.5 Synchronization

```bash
jin fetch                             # Fetch updates from remote Jin repo
jin pull                              # Fetch and merge updates (requires clean state)
jin push                              # Push local changes (requires fetch + clean merge first)
jin sync                              # Fetch + merge + apply to workspace
```

**Push Requirements:**
- Must fetch before push
- Must have clean merge state (no conflicts)
- If conflicts exist, user must resolve before push is allowed
- This ensures team coordination and prevents divergent branches

**Update Notifications:**
- If `jin fetch` detects updates to active modes/scopes/projects, inform user
- Format: `Updates available for: mode/claude, scope/language:javascript`
- User can then `jin pull` to merge updates

### 9.6 Status & Inspection

```bash
jin status                            # Show workspace state, active contexts, dirty files
jin diff [layer1] [layer2]            # Show differences between layers or workspace
jin log [layer]                       # Show commit history for layer
jin list                              # List available modes/scopes/projects from remote
jin layers                            # Show current layer composition and merge order
```

### 9.7 Reset Operations

```bash
jin reset                                        # Reset project base (Layer 7)
jin reset --mode                                 # Reset active mode base (Layer 2)
jin reset --mode --project                       # Reset mode-project (Layer 5)
jin reset --mode --scope=<scope>                 # Reset mode-scope (Layer 3)
jin reset --mode --scope=<scope> --project       # Reset mode-scope-project (Layer 4)
```

Supports standard Git reset API:
- `--soft`: Keep changes in staging area
- `--mixed` (default): Unstage changes but keep in workspace
- `--hard`: Discard all changes

---

## 10. Merge Strategy

### 10.1 Structured Merge Engine

Structured merge rules for supported file types:

| Type      | Strategy                                                 |
| --------- | -------------------------------------------------------- |
| JSON      | Key-aware deep merge, comma-safe serialization           |
| YAML/TOML | Key-aware hierarchical merge                             |
| INI       | Section merge                                            |
| Text      | 3-way diff using Git's merge-base from underlying layers |

### 10.2 Merge Priority

Precedence: **(1 lowest)** Global Base → Mode Base → Scope → Project layers → User Local → **(9 highest)** Workspace Active.

Conflicts generate `.jinmerge` files with automatic validation hooks.

### 10.3 Conflict Resolution (Future)

When conflicts occur during merge:
1. Jin pauses the merge operation
2. Creates `.jinmerge` files showing conflicts
3. Displays Git-style conflict markers with layer information
4. User resolves conflicts manually
5. User runs `jin add <resolved-files>` and `jin commit` to complete merge

**Format for conflict display:**
```
Conflict in file: .claude/config.json
Layer 1: mode/claude/scope/language:javascript/
Layer 2: mode/claude/project/ui-dashboard/

<<<<<<< mode/claude/scope/language:javascript/
{ "mcpServers": ["server-a"] }
=======
{ "mcpServers": ["server-b"] }
>>>>>>> mode/claude/project/ui-dashboard/
```

---

## 11. Implementation Details

### 11.1 Git and Environment

* Jin uses its own internal Git repo at `$JIN_DIR` (default: `~/.jin/`).
* Each layer path corresponds to a branch in the Jin repo (`mode/claude/scope/lang:js/project/ui-dashboard`).
* Uses standard `GIT_DIR` redirection for operations without altering normal Git.
* `jin commit` performs an atomic commit to multiple branches as needed, then updates the `.jinmap`.
* Git rerere (reuse recorded resolution) is enabled in Jin's Git space for efficient conflict resolution.

### 11.2 `.jinmap` File

Automatically generated; never edited manually.
Example:

```yaml
version: 1
mappings:
  "mode/claude": [".claude/", "CLAUDE.md"]
  "mode/claude/scope/lang:js": [".claude/commands", ".claude/config.json"]
  "project/ui-dashboard": [".vscode/"]
  "mode/claude/project/ui-dashboard": [".claude/local.json"]
meta:
  last-updated-by: "jin@v2.0"
```

### 11.3 `.jin/context` File

Stores active context per-project:

```yaml
version: 1
mode: claude
scope: language:javascript
last-updated: "2025-10-19T15:04:02Z"
```

### 11.4 No Support For

- **Symlinks:** Not supported. Jin will error if symlinks are detected.
- **Binary files:** Jin is for text-based configuration only. Large binaries are out of scope.
- **Git submodules:** Jin will not track files within submodules. Your project can use submodules, but Jin ignores them.
- **Detached workspace states:** Jin will abort any operation that would create a detached state.

---

## 12. Example Workflow (Happy Path)

```bash
# --- Normal Git world ---
git clone git@github.com:myorg/ui-dashboard.git
cd ui-dashboard

# Initialize Jin for this project
jin init
jin link git@github.com:myorg/jin-config

# Add project-level, unscoped files
jin add .vscode/ project-readme.md
jin commit -m "Add project-level files"
# → commits go to jin/project/ui-dashboard/

# ----------------------------
# Create the claude mode (first-time only)
# ----------------------------
jin mode create claude
# - Sets up the mode branch
# - No merge to workspace yet

# ----------------------------
# Activate the mode
# ----------------------------
jin mode use claude
# - After this, --mode flag will work
# - Active context stored in .jin/context

# ----------------------------
# Add mode-level files
# ----------------------------
claude init
jin add .claude/ CLAUDE.md --mode
jin commit -m "Add Claude mode base files"
# → commits go to jin/mode/claude/
# → files automatically added to .gitignore

# ----------------------------
# Create and activate the JavaScript scope
# ----------------------------
jin scope create language:javascript --mode=claude
jin scope use language:javascript
# - From this point, scope is active in context

# ----------------------------
# Add scope-level files
# ----------------------------
claude mcp add frontend-helper
cp ~/src/PRP-framework/.claude/commands .claude/commands
jin add .claude/commands --mode --scope=language:javascript
jin commit -m "Add JS scope files for Claude"
# → commits go to jin/mode/claude/scope/language:javascript/

# ----------------------------
# Add project-specific override
# ----------------------------
jin add .claude/mcp_config.json --mode --scope=language:javascript --project
jin commit -m "Add project-specific MCP config"
# → commits go to jin/mode/claude/scope/language:javascript/project/ui-dashboard/

# ----------------------------
# Push changes to the master Jin repo
# ----------------------------
jin fetch                              # Fetch latest changes
jin push                               # Push (requires clean merge state)
# → pushes mode and scope branches plus project-specific overrides

# ----------------------------
# Subsequent developer workflow (another machine)
# ----------------------------
git clone git@github.com:myorg/ui-dashboard.git
cd ui-dashboard
jin init
jin link git@github.com:myorg/jin-config

# Fetch and apply configuration
jin sync

# Activate mode and scope for workspace
jin mode use claude
jin scope use language:javascript
# → merges master mode + master scope + project overrides into workspace

# Check status
jin status
# Output:
# Active mode: claude
# Active scope: language:javascript
# Clean workspace
# Files tracked: 15

# Modify project-specific configuration within the scope
jin add .claude/mcp_config.json --mode --scope=language:javascript --project
jin commit -m "Update project-specific MCP server args"
# → automatically committed to correct layer based on flags

# Push updates
jin fetch
jin push
```

---

## 13. File Lifecycle Management

### 13.1 File Removal

When a file is removed from a layer:

```bash
# Remove from staging
jin rm <file>

# Remove and commit
jin rm <file>
jin commit -m "Remove config file"
```

**Behavior:**
- File is marked as deleted in the layer
- On merge, if file exists in lower layers, those versions become visible
- If file only existed in removed layer, it's removed from workspace
- Removal is tracked in Jin's Git history

### 13.2 File Renames

Jin tracks renames using Git's rename detection:

```bash
jin mv old-path new-path
jin commit -m "Rename config file"
```

**Behavior:**
- Git's rename detection tracks the move
- History is preserved
- Merge engine understands renames across layers

### 13.3 File Movement Between Layers

Moving a file from one layer to another:

```bash
# Remove from current layer
jin rm .claude/config.json --mode
jin commit -m "Remove from mode base"

# Add to new layer
jin add .claude/config.json --mode --project
jin commit -m "Move to mode-project layer"
```

**Considerations:**
- This is a destructive operation (file removed from source layer)
- Requires two commits (removal + addition)
- Consider using `jin mv` for simple path changes within same layer

### 13.4 File Path Collisions

When the same file path exists in multiple layers:

**Merge Behavior:**
- Higher-priority layers override lower-priority layers (see §10.2)
- Structured files (JSON/YAML/TOML) are deep-merged
- Text files use 3-way merge
- Binary files: higher layer wins (no merge)

**Detection:**
- `jin status` shows which layers contribute to each file
- `jin layers` shows the complete layer stack for current workspace
- `jin diff <layer1> <layer2>` shows differences

### 13.5 Important Notes

This section requires iteration and discovery to fully implement. File lifecycle management is **not required in the core feature set** but is documented here for future development.

---

## 14. Validation & Testing

* Layer routing correctness (unit tests for all flag combinations)
* Active context persistence and restoration
* Automatic .gitignore management
* Structured merge consistency across JSON, YAML, TOML
* Proper branch resolution and GIT_DIR scoping
* Accurate `.jinmap` generation and recovery
* Multi-scope checkout consistency and rollback
* Fetch-before-push enforcement
* Conflict detection and user notification

---

## 15. Audit & Metadata

Each commit logs:

```json
{
  "timestamp": "2025-10-19T15:04:02Z",
  "user": "dustin",
  "project": "ui-dashboard",
  "mode": "claude",
  "scope": "language:javascript",
  "layer": 4,
  "files": [".claude/config.json"],
  "base_commit": "abc123",
  "merge_commit": "def456",
  "context": {
    "active_mode": "claude",
    "active_scope": "language:javascript"
  }
}
```

All audit records live in `jin/.audit/` for offline inspection.

---

## 16. Acceptance Criteria

1. ✅ 9-layer hierarchy enforced in merge engine and routing.
2. ✅ Project automatically inferred from CWD Git origin.
3. ✅ `jin add` without flags commits to project base layer.
4. ✅ `jin add --mode` → mode layer; `--mode --scope=<scope>` → mode-scope layer; `--mode --project` → mode-project layer.
5. ✅ Multi-layer structured merges must resolve without human intervention (text files may require manual resolution).
6. ✅ `.jinmap` auto-maintained and consistent after every commit.
7. ✅ Active context (mode/scope) persists across sessions via `.jin/context`.
8. ✅ Any file added to Jin is automatically added to `.gitignore`.
9. ✅ `jin push` requires `jin fetch` with clean merge state first.
10. ✅ `jin status`, `jin diff`, `jin log`, `jin list`, `jin modes`, `jin scopes`, `jin layers` commands implemented.
11. ✅ `jin reset` supports all layer combinations with Git-standard flags.
12. ✅ Update notifications when remote has changes to active contexts.
13. ✅ Multiple scopes cannot be passed simultaneously (error condition).
14. ✅ No symlink support (error condition).
15. ✅ No binary file support (guidance only, not enforced).
16. ✅ No detached workspace states (abort operation).
17. ✅ Audit logs match real commits and layers touched.

---

## 17. Backlog (Prioritized)

### High Priority
1. **Worktree automation** - Git hooks or automation to re-enable Jin state in new worktrees
2. **Git-tracked file detection** - Protocol for detecting when jin-tracked files are added to Git, auto-removal and addition to `.jinignore`

### Medium Priority
3. **Conflict resolution UI/UX** - Detailed specification for `.jinmerge` files and resolution workflow
4. **Layer preview/dry-run** - Show what would happen before applying changes
5. **Migration tooling** - Tools for migrating between Jin versions
6. **Rollback procedures** - Additional rollback capabilities beyond `jin reset`

---

## 18. Out of Scope

* Dependency management between modes/scopes (developers manage their own tool installations)
* Security/secrets management (developers manage via env vars and shell commands)
* Performance optimization for massive scale (tool flexibility allows splitting configs if needed)
* CI/CD integration (Jin is for local development workspace only)
* Telemetry/analytics
* Interactive wizards or GUI
* Browser storage or web-based interfaces

---

## 19. Appendix

**Key Design Principles**

* No developer workflow changes outside Jin commands.
* Project inference removes need for explicit `--project`.
* All data lives in a single repo.
* Merging structured configs is always deterministic and reversible.
* Active context reduces flag verbosity for common operations.
* Automatic .gitignore management prevents accidents.
* Fetch-before-push enforces team coordination.

**Configuration is not for secrets. Configuration is for tooling.**

---
