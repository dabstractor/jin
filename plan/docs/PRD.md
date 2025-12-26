# Jin — Phantom Git Layer System

## 1. Overview

**Jin** is a meta-versioning system layered on top of Git that manages *developer-specific and tool-specific configuration* without contaminating a project's primary Git repository.

Jin introduces a **phantom Git layer** that tracks ignored and untracked files (editor configs, AI tooling state, MCP servers, etc.) and composes them deterministically into the working directory based on explicit rules.

Jin is:

* **Opt-in**
* **Non-disruptive**
* **Deterministic**
* **Reversible**
* **Git-native in behavior, but Git-isolated in storage**

---

## 2. Goals

1. **Zero workflow disruption**

   * Developers continue using Git normally
   * Jin only touches ignored or untracked files

2. **Shared yet isolated tooling**

   * Tooling config can be shared across teams
   * Never pollutes the main repository

3. **Deterministic layered merges**

   * JSON, YAML, TOML, INI merged predictably
   * Text files merged via 3-way diff

4. **Single Jin repository**

   * All global, mode, scope, and project configuration lives in one repo

5. **Scalability**

   * Supports many projects, modes, scopes, and teams

6. **Automatic `.gitignore` safety**

   * Jin-managed files cannot accidentally be committed to Git

---

## 3. Key Concepts

| Concept                    | Description                                                        |
| -------------------------- | ------------------------------------------------------------------ |
| **Mode**                   | A broad tooling environment (e.g. `claude`, `cursor`, `zed`)       |
| **Scope**                  | Contextual refinement (e.g. `language:javascript`, `infra:docker`) |
| **Project**                | Auto-inferred from Git remote origin                               |
| **Layer**                  | One level in Jin's override hierarchy                              |
| **Phantom Git**            | Jin's internal Git tracking ignored files                          |
| **Active Context**         | Current mode + scope affecting defaults                            |
| **Workspace Active Layer** | Derived merge output applied to the working tree                   |

---

## 4. Layer Architecture

### 4.1 The Nine-Layer Hierarchy

Precedence flows **bottom → top** (higher overrides lower):

| # | Layer                  | Description                       | Storage Path                                       |
| - | ---------------------- | --------------------------------- | -------------------------------------------------- |
| 1 | Global Base            | Shared defaults                   | `jin/global/`                                      |
| 2 | Mode Base              | Mode defaults                     | `jin/mode/<mode>/`                                 |
| 3 | Mode → Scope           | Scoped mode configs               | `jin/mode/<mode>/scope/<scope>/`                   |
| 4 | Mode → Scope → Project | Project overrides for scoped mode | `jin/mode/<mode>/scope/<scope>/project/<project>/` |
| 5 | Mode → Project         | Project overrides for mode        | `jin/mode/<mode>/project/<project>/`               |
| 6 | Scope Base             | Untethered scope configs          | `jin/scope/<scope>/`                               |
| 7 | Project Base           | Project-only configs              | `jin/project/<project>/`                           |
| 8 | User Local             | Machine-only overlays             | `~/.jin/local/`                                    |
| 9 | Workspace Active       | Derived merge result              | `.jin/workspace/`                                  |

---

## 5. Git Architecture & Invariants

### 5.1 Logical Branch Model

**Invariant:**
Jin does **not** expose or rely on user-facing Git branches.

* Layers are represented as **logical refs** under `refs/jin/...`
* These refs are never checked out directly
* Users never interact with them via raw Git commands

**Implementation options (allowed):**

* Single branch with structured directories
* Namespaced refs with garbage collection

**Disallowed:**

* One real Git branch per layer
* User-visible checkout of layer refs

---

## 6. Core API Contract

### 6.1 Staging & Committing

```bash
jin add <files>
jin commit -m "message"
```

* Identical mental model to Git
* Files remain staged until committed
* Multiple `jin add` calls may precede a commit

### 6.2 Commit Atomicity

**Invariant:**
`jin commit` is atomic across all affected layers.

**Transactional Model:**

1. Changes staged into a temporary transaction ref
2. All target refs updated only after successful object write
3. On failure, all refs roll back
4. Interrupted transactions are detected and auto-recovered

Partial commits are **impossible**.

---

## 7. Active Context

### 7.1 Context Rules

* One active mode at a time
* One active scope at a time
* Both may be active simultaneously
* Stored per-project in `.jin/context`
* Flags override context

```yaml
version: 1
mode: claude
scope: language:javascript
```

### 7.2 Context Commands

```bash
# Activate mode (sets active context)
jin mode use claude

# Activate scope (sets active context)
jin scope use language:javascript

# Deactivate mode
jin mode unset

# Deactivate scope
jin scope unset
```

After activation, no need to specify `--mode` or `--scope` flags unless overriding behavior.

---

## 8. `.gitignore` Management

### 8.1 Managed Block Invariant

**Invariant:**
Jin only modifies a clearly delimited block:

```gitignore
# --- JIN MANAGED START ---
.claude/
.vscode/settings.json
# --- JIN MANAGED END ---
```

Rules:

* Jin never edits outside this block
* Duplicates auto-deduplicated
* Conflicts auto-resolved inside block
* Removed Jin files remove ignore entries

### 8.2 Automatic Safety

* Any file added to Jin is automatically added to `.gitignore`
* Jin checks `.gitignore` before adding files
* This prevents jin-tracked files from accidentally being committed to Git
* Jin-tracked files added to Git are detected and handled (see §23 Backlog)

---

## 9. Layer Routing & `jin add` Semantics

### 9.1 Routing Table

| Command                                           | Target Layer               |
| ------------------------------------------------- | -------------------------- |
| `jin add <file>`                                  | Project Base (7)           |
| `jin add <file> --mode`                           | Mode Base (2)              |
| `jin add <file> --mode --project`                 | Mode → Project (5)         |
| `jin add <file> --scope=<scope>`                  | Scope Base (6)             |
| `jin add <file> --mode --scope=<scope>`           | Mode → Scope (3)           |
| `jin add <file> --mode --scope=<scope> --project` | Mode → Scope → Project (4) |

### 9.2 Errors

* `--mode` with no active mode → ERROR
* Multiple scopes → ERROR
* Git-tracked file → ERROR (use `jin import`)

### 9.3 Active Context Behavior Examples

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

---

## 10. Scope Precedence Rules

When both exist:

```
Mode-bound scope
  > Untethered scope
    > Mode base
```

Untethered scopes apply **only if no mode-bound scope of same name exists**.

---

## 11. Merge Strategy

### 11.1 Structured Merge Rules

| Type               | Behavior                     |
| ------------------ | ---------------------------- |
| JSON / YAML / TOML | Deep key merge               |
| Arrays (keyed)     | Merge by `id` or `name`      |
| Arrays (unkeyed)   | Higher layer replaces        |
| `null`             | Deletes key                  |
| Ordering           | Preserved from highest layer |
| Comments           | Not preserved                |
| INI                | Section merge                |
| Text               | 3-way diff                   |

**Deterministic and reversible.**

### 11.2 Merge Priority

Precedence: **(1 lowest)** Global Base → Mode Base → Scope → Project layers → User Local → **(9 highest)** Workspace Active.

### 11.3 Conflict Resolution

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

## 12. Workspace Active Layer

**Invariant:**
Workspace Active is **never a source of truth**.

* Direct edits allowed
* No persistence without `jin add`
* `jin status` separates:

  * Workspace dirty
  * Layer dirty

---

## 13. Mode & Scope Lifecycle

### 13.1 Mode

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

### 13.2 Scope

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

### 13.3 Multiple Active Contexts

- **One active mode at a time** (enforced)
- **One active scope at a time** (enforced)
- **Both can be active simultaneously** (mode + scope)
- Use `jin status` to see current active contexts

---

## 14. Synchronization Rules

```bash
jin fetch
jin pull
jin push
jin sync
```

**Push Rules:**

* Fetch required
* Clean merge state required
* Conflicts must be resolved first

**Update Notifications:**

* If `jin fetch` detects updates to active modes/scopes/projects, inform user
* Format: `Updates available for: mode/claude, scope/language:javascript`
* User can then `jin pull` to merge updates

---

## 15. Failure Recovery Guarantees

Jin guarantees:

* Safe abort on interruption
* Auto-repair of `.jinmap`
* Idempotent retries
* Explicit unrecoverable errors

---

## 16. `.jinmap`

```yaml
version: 1
mappings:
  "mode/claude": [".claude/", "CLAUDE.md"]
meta:
  generated-by: jin
```

* Auto-generated
* Never user-edited
* Recoverable from Git history

---

## 17. Audit Logs

* Informational, append-only
* Derived from Git commits
* May be regenerated
* Commit hashes included

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

## 18. Core Commands

### 18.1 Initialization

```bash
jin init                              # Initialize Jin in current project
jin link <repo-url>                   # Link to shared Jin config repo
```

### 18.2 Staging & Committing

```bash
jin add <files> [--mode] [--scope=<scope>] [--project]
                                      # Stage files to appropriate layer
jin commit -m "message"               # Commit staged files
jin reset [--soft|--mixed|--hard]    # Reset staged/committed changes
```

### 18.3 Mode Management

```bash
jin mode create <mode>                # Create new mode
jin mode use <mode>                   # Activate mode (set context)
jin mode unset                        # Deactivate current mode
jin mode delete <mode>                # Delete mode
jin modes                             # List all available modes
```

### 18.4 Scope Management

```bash
jin scope create <scope> [--mode=<mode>]
                                      # Create new scope (optionally tied to mode)
jin scope use <scope>                 # Activate scope (set context)
jin scope unset                       # Deactivate current scope
jin scope delete <scope>              # Delete scope
jin scopes                            # List all available scopes
```

### 18.5 Synchronization

```bash
jin fetch                             # Fetch updates from remote Jin repo
jin pull                              # Fetch and merge updates (requires clean state)
jin push                              # Push local changes (requires fetch + clean merge first)
jin sync                              # Fetch + merge + apply to workspace
```

### 18.6 Status & Inspection

```bash
jin status                            # Show workspace state, active contexts, dirty files
jin diff [layer1] [layer2]            # Show differences between layers or workspace
jin log [layer]                       # Show commit history for layer
jin list                              # List available modes/scopes/projects from remote
jin layers                            # Show current layer composition and merge order
```

### 18.7 Reset Operations

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

### 18.8 File Operations

```bash
jin rm <file>                         # Remove file from layer
jin mv <old-path> <new-path>          # Rename/move file within layer
```

---

## 19. Implementation Details

### 19.1 Git and Environment

* Jin uses its own internal Git repo at `$JIN_DIR` (default: `~/.jin/`).
* Layers are represented as logical refs under `refs/jin/...`
* Uses standard `GIT_DIR` redirection for operations without altering normal Git.
* `jin commit` performs an atomic commit to multiple refs as needed, then updates the `.jinmap`.
* Git rerere (reuse recorded resolution) is enabled in Jin's Git space for efficient conflict resolution.

### 19.2 `.jin/context` File

Stores active context per-project:

```yaml
version: 1
mode: claude
scope: language:javascript
last-updated: "2025-10-19T15:04:02Z"
```

### 19.3 Unsupported Features

- **Symlinks:** Not supported. Jin will error if symlinks are detected.
- **Binary files:** Jin is for text-based configuration only. Large binaries are out of scope.
- **Git submodules:** Jin will not track files within submodules. Your project can use submodules, but Jin ignores them.
- **Detached workspace states:** Jin will abort any operation that would create a detached state.

---

## 20. Example Workflow

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
# - Sets up the mode ref
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
# → pushes mode and scope refs plus project-specific overrides

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

## 21. File Lifecycle Management

### 21.1 File Removal

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

### 21.2 File Renames

Jin tracks renames using Git's rename detection:

```bash
jin mv old-path new-path
jin commit -m "Rename config file"
```

**Behavior:**
- Git's rename detection tracks the move
- History is preserved
- Merge engine understands renames across layers

### 21.3 File Movement Between Layers

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

### 21.4 File Path Collisions

When the same file path exists in multiple layers:

**Merge Behavior:**
- Higher-priority layers override lower-priority layers (see §11.2)
- Structured files (JSON/YAML/TOML) are deep-merged
- Text files use 3-way merge
- Binary files: higher layer wins (no merge)

**Detection:**
- `jin status` shows which layers contribute to each file
- `jin layers` shows the complete layer stack for current workspace
- `jin diff <layer1> <layer2>` shows differences

---

## 22. Validation & Testing

* Layer routing correctness (unit tests for all flag combinations)
* Active context persistence and restoration
* Automatic .gitignore management with managed block
* Structured merge consistency across JSON, YAML, TOML
* Proper ref resolution and GIT_DIR scoping
* Accurate `.jinmap` generation and recovery
* Multi-scope checkout consistency and rollback
* Fetch-before-push enforcement
* Conflict detection and user notification
* Commit atomicity verification
* Failure recovery testing
* Scope precedence validation

---

## 23. Backlog

### High Priority
1. **Worktree automation** - Git hooks or automation to re-enable Jin state in new worktrees
2. **Git-tracked file detection** - Protocol for detecting when jin-tracked files are added to Git, auto-removal and addition to `.jinignore`

### Medium Priority
3. **Conflict resolution UI/UX** - Detailed specification for `.jinmerge` files and resolution workflow
4. **Layer preview/dry-run** - Show what would happen before applying changes
5. **Migration tooling** - Tools for migrating between Jin versions
6. **Rollback procedures** - Additional rollback capabilities beyond `jin reset`

---

## 24. Out of Scope

* Dependency management between modes/scopes (developers manage their own tool installations)
* Security/secrets management (developers manage via env vars and shell commands)
* Performance optimization for massive scale (tool flexibility allows splitting configs if needed)
* CI/CD integration (Jin is for local development workspace only)
* Telemetry/analytics
* Interactive wizards or GUI
* Browser storage or web-based interfaces

---

## 25. Non-Negotiable Invariants

1. No user-facing Git branch explosion
2. Atomic multi-layer commits
3. Deterministic structured merges
4. Workspace is never source of truth
5. `.gitignore` is safely sandboxed
6. Scope precedence is explicit
7. Failure is recoverable
8. Jin enforces correctness, not social policy

---

## 26. Acceptance Criteria

1. 9-layer hierarchy enforced in merge engine and routing.
2. Project automatically inferred from CWD Git origin.
3. `jin add` without flags commits to project base layer.
4. `jin add --mode` → mode layer; `--mode --scope=<scope>` → mode-scope layer; `--mode --project` → mode-project layer.
5. Multi-layer structured merges must resolve without human intervention (text files may require manual resolution).
6. `.jinmap` auto-maintained and consistent after every commit.
7. Active context (mode/scope) persists across sessions via `.jin/context`.
8. Any file added to Jin is automatically added to `.gitignore` managed block.
9. `jin push` requires `jin fetch` with clean merge state first.
10. `jin status`, `jin diff`, `jin log`, `jin list`, `jin modes`, `jin scopes`, `jin layers` commands implemented.
11. `jin reset` supports all layer combinations with Git-standard flags.
12. Update notifications when remote has changes to active contexts.
13. Multiple scopes cannot be passed simultaneously (error condition).
14. No symlink support (error condition).
15. No binary file support (guidance only, not enforced).
16. No detached workspace states (abort operation).
17. Audit logs match real commits and layers touched.
18. Logical refs used instead of user-facing branches.
19. Commit atomicity guaranteed across all affected layers.

---

## 27. Appendix

**Key Design Principles**

* No developer workflow changes outside Jin commands.
* Project inference removes need for explicit `--project`.
* All data lives in a single repo.
* Merging structured configs is always deterministic and reversible.
* Active context reduces flag verbosity for common operations.
* Automatic .gitignore management prevents accidents.
* Fetch-before-push enforces team coordination.
* Logical refs prevent branch explosion.

**Configuration is not for secrets. Configuration is for tooling.**
