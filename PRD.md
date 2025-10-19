# Jin — Phantom Git Layer System

**Product Requirements Document (PRD)**
**Version:** 1.0
**Status:** Draft (Post-Correction)
**Last Updated:** 2025-10-19

---

## 1. Overview

**Jin** is a meta-versioning layer built atop Git to manage *developer-specific and tool-specific configuration* in collaborative projects, without contaminating the main repository. Jin provides an isolated “phantom layer” for tracking ignored/untracked files that represent tooling states (AI configs, editor metadata, MCP servers, etc.).

The system allows developers to define **modes** (per-tool setups) and **scopes** (per-language or context overlays) and manage them across projects automatically.

---

## 2. Goals

* **Zero workflow disruption:** Developers continue using Git normally. Jin operations are opt-in and limited to ignored/untracked files.
* **Shared yet isolated tooling:** Each dev can have tool configurations applied without polluting the repo.
* **Automatic merge logic:** Jin must accurately merge JSON, YAML, TOML, and similar structured files across layers.
* **Single repository architecture:** All configuration data—global, per-mode, per-scope, and per-project—is housed in a single Git repo.
* **Scalability:** Multiple projects and teams can use the same Jin repo, allowing both shared and project-specific customizations.

---

## 3. Key Concepts

| Concept         | Description                                                                                       |
| --------------- | ------------------------------------------------------------------------------------------------- |
| **Mode**        | Represents a specific AI/tooling environment (e.g. `claude`, `cursor`, `zed`).                    |
| **Scope**       | A refinement or contextual specialization of a mode (e.g. `language:javascript`, `infra:docker`). |
| **Project**     | The active Git repository detected from CWD. No explicit flag required.                           |
| **Layer**       | A level in the structured merge/commit hierarchy (see §4).                                        |
| **Phantom Git** | Jin’s shadow Git tracking ignored/untracked files and mapping them to the correct layers.         |

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

## 5. Layer Routing and Add Behavior

### 5.1 Flag Routing Rules

| Command State                                                        | Target Layer | Description                            |
| -------------------------------------------------------------------- | ------------ | -------------------------------------- |
| `jin add` *(no flags)*                                               | Layer 7      | Adds to project base layer.            |
| `jin add --mode=<mode>`                                              | Layer 2      | Adds to mode base layer.               |
| `jin add --mode=<mode> --project` *(implied by cwd)*                 | Layer 5      | Adds to project-specific mode layer.   |
| `jin add --mode=<mode> --scope=<scope>`                              | Layer 3      | Adds to mode’s scope layer.            |
| `jin add --mode=<mode> --scope=<scope> --project` *(implied by cwd)* | Layer 4      | Adds to project-specific scoped layer. |
| (Local config only)                                                  | Layer 8      | Machine-only overlays.                 |

Only one scope may be *written* at a time, but multiple scopes may be *checked out* simultaneously.

---

## 6. Mode and Scope Behavior

### 6.1 Mode

A mode defines a broad developer setup—typically per AI or editor.
Example:

```bash
jin mode create claude
jin add --mode=claude .claude/ CLAUDE.md
jin commit -m "Add Claude mode base files"
```

### 6.2 Scope

A scope applies language- or domain-specific deltas on top of a mode.
Example:

```bash
jin scope create language:javascript --mode=claude
jin add --mode=claude --scope=language:javascript .claude/commands
jin commit -m "Add Claude JS scope helpers"
```

---

## 7. Project-Specific Customization

Each project’s unique changes (e.g., AngularJS vs React) are stored automatically in project-specific layers inferred from the Git origin name (e.g. `ui-dashboard`).

Example:

```bash
# inside ui-dashboard repo
jin add .claude/config.json   # → project/ui-dashboard/
jin add --mode=claude .claude/config.json  # → mode/claude/project/ui-dashboard/
jin add --mode=claude --scope=lang:js .claude/config.json  # → mode/claude/scope/lang:js/project/ui-dashboard/
```

---

## 8. Merge Strategy

### 8.1 Structured Merge Engine

Structured merge rules for supported file types:

| Type      | Strategy                                                 |
| --------- | -------------------------------------------------------- |
| JSON      | Key-aware deep merge, comma-safe serialization           |
| YAML/TOML | Key-aware hierarchical merge                             |
| INI       | Section merge                                            |
| Text      | 3-way diff using Git’s merge-base from underlying layers |

### 8.2 Merge Priority

Precedence: **(1 lowest)** Global Base → Mode Base → Scope → Project layers → User Local → **(9 highest)** Workspace Active.

Conflicts generate `.jinmerge` files with automatic validation hooks.

---

## 9. Implementation Details

### 9.1 Git and Environment

* Jin uses its own internal Git repo at `$JIN_DIR` (default: `~/.jin/`).
* Each layer path corresponds to a branch in the Jin repo (`mode/claude/scope/lang:js/project/ui-dashboard`).
* Uses standard `GIT_DIR` redirection for operations without altering normal Git.
* `jin commit` performs an atomic commit to multiple branches as needed, then updates the `.jinmap`.

### 9.2 `.jinmap` File

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
  last-updated-by: "jin@v1.0"
```

---

## 10. Example Workflow (Happy Path)

```bash
# Normal Git world
git clone git@github.com:myorg/ui-dashboard.git
cd ui-dashboard

# Initialize Jin
jin init
jin link git@github.com/myorg/jin-config

# Create Claude mode and JS scope
jin mode create claude
jin scope create language:javascript --mode=claude

# Add base Claude configuration
claude init
jin add --mode=claude .claude/ CLAUDE.md
jin commit -m "Add Claude mode base files"

# Add language-specific overlay
claude mcp add chrome-devtools-mcp
jin add --mode=claude --scope=language:javascript .claude/
jin commit -m "Add Claude JS MCP servers"

# Sync all Jin data
jin push
jin sync
```

---

## 11. Validation & Testing

* Layer routing correctness (unit tests for all flag combinations)
* Structured merge consistency across JSON, YAML, TOML
* Proper branch resolution and GIT_DIR scoping
* Accurate `.jinmap` generation and recovery
* Multi-scope checkout consistency and rollback

---

## 12. Audit & Metadata

Each commit logs:

```json
{
  "timestamp": "2025-10-19T15:04:02Z",
  "user": "dustin",
  "project": "ui-dashboard",
  "mode": "claude",
  "scope": "language:javascript",
  "layer": 3,
  "files": [".claude/config.json"],
  "base_commit": "abc123",
  "merge_commit": "def456"
}
```

All audit records live in `jin/.audit/` for offline inspection.

---

## 13. Acceptance Criteria

1. 9-layer hierarchy enforced in merge engine and routing.
2. Project automatically inferred from CWD Git origin.
3. `jin add` without flags commits to project base layer.
4. `jin add --mode` → mode layer; `--mode --scope` → mode-scope layer; add project context if inferred.
5. Multi-layer structured merges must resolve without human intervention.
6. `.jinmap` auto-maintained and consistent after every commit.
7. `jin push` syncs all dirty branches.
8. Audit logs match real commits and layers touched.

---

## 14. Appendix

**Key Design Principles**

* No developer workflow changes outside Jin commands.
* Project inference removes need for explicit `--project`.
* All data lives in a single repo.
* Merging structured configs is always deterministic and reversible.

---

