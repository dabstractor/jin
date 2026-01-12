# Remote Configuration Repository Patterns & Best Practices

## Executive Summary

This document synthesizes research on how similar tools (dotfiles managers, configuration systems) handle remote repository configuration, conflict resolution, and synchronization workflows. Key findings inform the design of Jin's `jin link` command and related fetch/push/pull operations.

**Key Insight**: Remote configuration systems face a fundamental tension between **shared configuration** (team/mode-level) and **local customization** (project/machine-level). The best solutions use a **hierarchical, explicitly-configured approach** rather than attempting automatic conflict resolution.

---

## 1. Similar Tools & Their Approaches

### 1.1 Chezmoi (Modern Go-based dotfiles manager)

**Overview**: Chezmoi manages dotfiles by cloning a remote repository into `~/.local/share/chezmoi` and applying them to the home directory.

**Remote Configuration:**
- Remote is specified during initialization: `chezmoi init --apply https://github.com/$USERNAME/dotfiles.git`
- Remote URL stored in the chezmoi Git config (standard Git remote)
- Branch tracking uses standard Git HEAD tracking

**Synchronization Workflow:**
1. `chezmoi update` - pulls latest from remote, reviews before applying
2. Optional: `chezmoi diff` before applying to review changes
3. `chezmoi apply` - applies merged configuration to workspace

**Key Design Patterns:**
- **Template-based customization**: Uses Go templates to generate machine-specific configuration
- **Declarative format**: TOML or JSON for specifying what files to manage
- **Auto-apply option**: Can auto-fetch and auto-apply, but defaults to manual review
- **Low friction**: Developers don't think about syncing; it "just works"

**Storage Pattern**:
```
~/.local/share/chezmoi/     # Source of truth (checked out Git repo)
  .chezmoi.toml             # Configuration (in Git)
  home/
    .gitconfig
    .bashrc
```

**Advantages**:
- Templates reduce duplication across machines
- Standard Git workflow
- Clear separation of source (chezmoi dir) vs applied (home dir)

---

### 1.2 YADM (Yet Another Dotfiles Manager)

**Overview**: YADM is a Git wrapper that adds dotfiles management to standard Git operations.

**Remote Configuration:**
- Uses standard Git remotes: `yadm remote add origin <url>`
- Branch tracking: Can specify alternate branch via `-b option` during clone
- Default: follows remote HEAD branch on first clone

**Synchronization Workflow:**
```bash
yadm add <file>
yadm commit -m "message"
yadm push                    # Push to origin
yadm pull                    # Fetch + merge
yadm status                  # Standard Git status
```

**Key Design Patterns:**
- **Git-native**: All operations are standard Git commands (`yadm init`, `yadm add`, etc.)
- **No configuration layers**: All files treated equally; no mode/scope hierarchy
- **Direct Git usage**: Developers can use `git` commands interchangeably with `yadm`

**Storage Pattern**:
```
$HOME/
  .config/
    yadm/
      repo.git/            # Bare repository (source of truth)
  .dotfiles/               # Checked out working tree
    .gitconfig
    .bashrc
```

**Advantages**:
- Minimal learning curve (just Git)
- Full Git feature support (branches, merge strategies, etc.)
- Predictable behavior

**Limitations**:
- No built-in conflict resolution strategy
- All files treated equally; no hierarchical organization
- No templating; requires scripting for machine-specific configs

---

### 1.3 Homesick (Ruby-based symlink-based approach)

**Overview**: Homesick clones dotfiles repositories into `~/.homesick/castles/` and symlinks files into place.

**Remote Configuration:**
- Clones castle (dotfiles repo) on init
- Standard Git remote operations inside the castle
- Workflow: `git remote add origin <url>` then `git push -u origin master`

**Synchronization Workflow:**
```bash
homesick clone <url>        # Clone castle
homesick link <castle>      # Create symlinks
# Edit files (via symlinks)
# Push changes back to origin
```

**Key Design Patterns:**
- **Symlink-based**: Files exist in single place; symlinks created in home directory
- **Multiple castles**: Can manage multiple dotfiles repos simultaneously
- **Git-backed**: Each castle is a Git repository

**Storage Pattern**:
```
~/.homesick/castles/
  my-castle/
    home/                   # Maps to $HOME
      .gitconfig
      .bashrc
    .git/                   # Git repository
```

**Advantages**:
- Files exist in one place; edits immediately reflected
- Can manage multiple castles (team, personal, etc.)
- Symlinks are transparent to applications

**Limitations**:
- Symlinks break some tools/workflows
- No built-in hierarchy for shared vs local configs
- Requires careful management of symlink breakage

---

### 1.4 Common Patterns Across Tools

| Aspect | Chezmoi | YADM | Homesick | Jin (Proposed) |
|--------|---------|------|----------|----------------|
| **Config Storage** | Git repo (~/.local/share/chezmoi) | Bare Git repo (git dir) | Git castles (~/.homesick/castles) | Unified remote (origin) |
| **File Linking** | Apply (copy/template) | Git worktree | Symlinks | Apply (merge + copy) |
| **Hierarchy** | Declarative template | Flat files | Multiple castles | 9-layer mode/scope |
| **Branch Strategy** | Follow HEAD | User specifies | Per-castle | Single branch + logical refs |
| **Conflict Resolution** | Manual review before apply | User resolves | Manual | Explicit merge rules |
| **Remote Update** | `update` + `diff` + `apply` | `pull` | Git push/pull | `fetch` + `pull` + `apply` |

---

## 2. Shared Configuration Patterns

### 2.1 The Shared vs Local Spectrum

Most configuration systems occupy a point on this spectrum:

```
PURE SHARED ←──────────────────────────→ PURE LOCAL
  (all team configs)              (all machine-specific)

Chezmoi:    Templates (templated shared → local via vars)
YADM:       Single repo (shared files, manual branching for local)
Homesick:   Multiple castles (separate repos for shared vs local)
Jin:        9-layer hierarchy (explicit layer precedence)
```

### 2.2 Handling Conflicts Between Shared and Local Configs

#### Problem
When team pushes global mode config, and developer has local project override, which wins?

#### Chezmoi's Solution
- **Templates with variables**: Template files are rendered with machine-specific variables
- **`.chezmoi.toml` configuration**: Defines which templates apply to this machine
- **Explicit overrides**: You decide in your `.chezmoi.toml` what to apply

**Example**:
```toml
# ~/.local/share/chezmoi/.chezmoi.toml
[data]
email = "user@example.com"
hostname = "my-machine"
```

```jinja2
# ~/.local/share/chezmoi/.vscode/settings.json.tmpl
{
  "email": "{{ .email }}",
  "hostname": "{{ .hostname }}"
}
```

#### YADM's Solution
- **Conditional file includes**: Files included based on system hostname/OS
- **File splitting**: Keep shared files separate from local overrides
- **Manual branching**: Different branches for different machines/teams

**Example**:
```bash
# Shared file
.bashrc                    # In main branch

# Machine-specific
.bashrc.local              # Not tracked (in .gitignore)
# Source in .bashrc: if [ -f ~/.bashrc.local ]; then source ~/.bashrc.local; fi
```

#### Homesick's Solution
- **Multiple castles**: Maintain separate castles for shared vs local
- **Priority order**: Link order determines precedence
  ```bash
  homesick link shared-castle    # Base configs
  homesick link local-castle     # Overrides (symlinks later ones)
  ```
- **Git branching within castle**: Different branches for different teams

#### Jin's Approach
- **Explicit 9-layer hierarchy**: Each layer has defined precedence
- **Mode/scope structure**: Shared configs organized by audience (mode + scope)
- **Local layer precedence**: User Local (layer 8) overrides everything except Workspace Active
- **No automatic conflict resolution**: Conflicts between layers pause merge and require explicit resolution

**Example - Shared mode vs project override**:
```
Layer 2 (Mode Base):         .claude/config.json (shared team config)
  ↓ merges with
Layer 4 (Mode→Scope→Project): .claude/config.json (project-specific override)
  ↓
Layer 9 (Workspace):          Final merged result

If conflict: pause merge, user resolves in .jinmerge, then `jin add` to continue
```

---

### 2.3 Best Practice: Explicit Hierarchy Over Implicit Merging

**Lesson from Similar Tools**:
- Chezmoi uses **templates** to make customization explicit
- YADM relies on **file naming** (`.local` suffix) for overrides
- Homesick uses **castle priority** for layering
- **All avoid automatic deep merging** of conflicting files

**Recommendation for Jin**:
- ✅ **DO**: Use explicit layer hierarchy; make precedence clear
- ✅ **DO**: Pause on conflicts; require user intervention
- ✅ **DO**: Provide `jin layers` to show composition
- ❌ **DON'T**: Attempt automatic 3-way merge of structured files without user visibility
- ❌ **DON'T**: Allow implicit overrides without clear layer indication

---

## 3. Security Considerations for Shared Configs

### 3.1 What NOT to Store in Remote Config Repos

**Never store**:
- Passwords or authentication tokens
- Private keys (SSH, GPG, etc.)
- API keys
- Database credentials
- Secrets of any kind

**Why**: Private Git repositories are high-value targets; an attacker who gains access gets all secrets.

### 3.2 Recommended Approach: Environment Variables + Secure Vaults

**Best Practices** (from OWASP and industry research):

1. **Use environment variables for secrets**:
   ```bash
   # In config file (checked in)
   export CLAUDE_API_KEY="${CLAUDE_API_KEY:-default}"

   # At runtime, set via environment
   export CLAUDE_API_KEY="abc123xyz"
   ```

2. **Use centralized secret management** for teams:
   - HashiCorp Vault
   - AWS Secrets Manager
   - GitHub Secrets
   - LastPass/1Password team vaults

3. **Automate secret rotation**:
   - Never hardcode credentials
   - Rotate regularly (30-90 days)
   - Monitor usage and access patterns

4. **Document the pattern**:
   ```markdown
   # .claude/config.json (in repo, no secrets)
   {
     "apiKeyEnvVar": "CLAUDE_API_KEY",
     "modelEnvVar": "CLAUDE_MODEL"
   }

   # User's .bashrc (not in repo)
   export CLAUDE_API_KEY="..."
   export CLAUDE_MODEL="claude-opus"
   ```

### 3.3 Jin-Specific Recommendations

**For shared modes/scopes**:
- ✅ Store **non-secret configuration** (paths, tool settings, mode definitions)
- ❌ Never store **secrets** (keys, tokens, passwords)
- ✅ Use environment variable placeholders in templates
- ✅ Document required environment variables in README

**For local projects**:
- ✅ Project-level configs (Layer 7) can reference secrets via env vars
- ✅ User Local (Layer 8) can override with machine-specific settings
- ❌ Don't add secrets to Layer 8; use shell sourcing instead

**Example secure pattern**:
```yaml
# jin/mode/claude/config.yaml (shared, no secrets)
apiEndpoint: ${CLAUDE_ENDPOINT:-https://api.anthropic.com}
model: claude-3-sonnet

# .jin/context (local, no secrets)
mode: claude
scope: language:javascript

# ~/.bashrc (user's machine, has secrets)
export CLAUDE_API_KEY="..."
export CLAUDE_ENDPOINT="..."
```

---

## 4. Workflow Patterns & Recommendations

### 4.1 Typical User Journey: init → link → fetch → pull

```bash
# 1. Initialize Jin in current project
git clone git@github.com:myorg/my-project
cd my-project
jin init
# Creates: .jin/context, .jin/staging, .jin/workspace

# 2. Link to shared remote repository
jin link git@github.com:myorg/jin-config
# Stores remote URL in ~/.jin/config.toml

# 3. Fetch updates from remote
jin fetch
# Downloads mode/scope/project data from remote
# Shows: "Updates available for: mode/claude, scope/language:javascript"

# 4. Pull (fetch + merge) updates
jin pull
# Merges fetched remote data into layers
# May pause on conflicts requiring manual resolution

# 5. Apply merged config to workspace
jin apply
# Applies all merged layers to .jin/workspace/
# Files ready for use

# 6. Make local changes
jin add .claude/config.json
jin commit -m "Customize MCP servers for this project"
# Commits to Layer 7 (Project Base)

# 7. Push changes back to remote
jin fetch          # Fetch first (required)
jin push           # Push mode/scope/project refs
# Updates remote with local project overrides
```

### 4.2 Should Link Auto-Fetch or Require Explicit Fetch?

**Analysis of Similar Tools**:

| Tool | `init` Behavior | Reasoning |
|------|-----------------|-----------|
| **Chezmoi** | `init --apply` auto-fetches + applies | Low friction; get started immediately |
| **YADM** | `yadm clone` auto-clones; then manual fetch | Developer controls initial state |
| **Homesick** | `homesick clone` auto-clones | Explicit clone semantics |

**Recommendation for Jin**:

```bash
# Option A: Auto-fetch (low friction)
jin link <url>      # Automatically fetches + shows available modes/scopes
# Shows: "Available modes: claude, cursor"
# Shows: "Available scopes: language:javascript, language:python"

# Option B: Manual fetch (explicit control)
jin link <url>      # Only stores URL
jin fetch            # User explicitly pulls from remote
```

**Recommendation**: **Option A (auto-fetch)** is better because:
- ✅ Reduces steps for new developers
- ✅ Provides immediate visibility into available modes/scopes
- ✅ Still allows user to skip merge if they want (`jin fetch` without `jin pull`)
- ✅ Matches Chezmoi's successful pattern

---

### 4.3 Re-linking to a Different Remote

**Problem**: Developer switches teams or moves configuration to new host

**Solution**:
```bash
# Current state
~/.jin/config.toml contains: url = "git@github.com:oldteam/jin-config"

# Re-link to new remote
jin link git@github.com:newteam/jin-config
# Prompts: "Replace existing remote? (y/n)"
# Updates ~/.jin/config.toml
# Next: jin fetch to get new config
```

**Implementation notes**:
- Only update `~/.jin/config.toml`, not per-project `.jin/context`
- Project context remains; only remote source changes
- Requires fresh fetch to avoid stale cached data

---

### 4.4 Multiple Projects Linked to Same Remote

**Scenario**: Entire team uses same remote; developers work across multiple projects

```bash
# Project A
cd ~/projects/project-a
jin init
jin link git@github.com:myorg/jin-config

# Project B
cd ~/projects/project-b
jin init
jin link git@github.com:myorg/jin-config

# Both reference the same ~/.jin/config.toml (global)
# Each has its own .jin/context (per-project mode/scope)
jin mode use claude              # Affects project-b only
jin scope use language:python    # Affects project-b only

# They can use same modes but different scopes
```

**Benefits**:
- ✅ Single source of truth for modes/scopes
- ✅ Different projects can activate different scopes
- ✅ Reduces duplication of configuration

**Implementation**:
- Remote URL in `~/.jin/config.toml` (global, shared)
- Active mode/scope in `.jin/context` (per-project, local)

---

## 5. Configuration Schema: Where to Store What

### 5.1 Global vs Per-Project Configuration

**Current Jin Config Structure**:

```
~/.jin/
  config.toml              # Global configuration
  local/                   # User-local overrides (Layer 8)
    .vscode/settings.json
    .claude/config.json
  [git internals]

.jin/                      # Per-project directory
  context                  # Per-project active mode/scope
  staging/                 # Staged changes
  workspace/               # Merged + applied files
  [git internals]
```

### 5.2 What Goes Where?

#### ~/.jin/config.toml (Global, One Per Machine)

**Should contain**:
- Remote URL and fetch behavior settings
- User info (name, email for commits)
- Global defaults (preferred editor, tool version)
- Machine-wide tool paths

**Should NOT contain**:
- Project-specific settings (those go in `.jin/context` or `.jin/layers`)
- Secrets (use environment variables)
- User Local customizations (those go in `~/.jin/local/`)

**Example**:
```toml
version = 1

[remote]
url = "git@github.com:myorg/jin-config"
fetch_on_init = true
fetch_on_enter = false          # New: auto-fetch on `jin` in new session?

[user]
name = "Jane Developer"
email = "jane@example.com"

[defaults]
editor = "code"
shell = "zsh"
```

#### .jin/context (Per-Project, One Per Project)

**Should contain**:
- Project name (auto-inferred from Git origin)
- Currently active mode
- Currently active scope
- Last update timestamp

**Should NOT contain**:
- Remote URL (use global)
- User info (use global)
- File paths (those are in `.jinmap`)

**Example**:
```yaml
version: 1
project: ui-dashboard
mode: claude
scope: language:javascript
last_updated: "2025-01-15T10:30:00Z"
```

#### .jinmap (Per-Project, Auto-Generated)

**Should contain**:
- Mapping of logical layers to actual files
- Layer-to-file associations for recovery

**Should NOT contain**:
- Remote info
- Context info
- Secrets

**Example**:
```yaml
version: 1
mappings:
  "mode/claude": [".claude/", "CLAUDE.md"]
  "mode/claude/scope/language:javascript": [".claude/commands"]
  "project/ui-dashboard": [".vscode/"]
meta:
  generated-by: jin
  generated-at: "2025-01-15T10:30:00Z"
```

### 5.3 Information In Remote URL vs Local Config

| Information | Remote URL | Local Config | Notes |
|-------------|-----------|--------------|-------|
| **Repository URL** | ✅ Yes | ❌ No | Source of truth for all projects |
| **Branch to track** | ✅ Yes (implicit: origin/master) | ⚠️ Sometimes | Could make this configurable |
| **Fetch behavior** | ✅ Yes (fetch_on_init) | ❌ No | Global setting |
| **Active mode** | ❌ No | ✅ Yes | Per-project |
| **Active scope** | ❌ No | ✅ Yes | Per-project |
| **Project name** | ❌ No | ✅ Yes | Auto-inferred, cached |
| **User info** | ❌ No | ✅ Yes | Global (name/email for commits) |

### 5.4 Branch/Ref Strategy for Remote Tracking

**Problem**: Should `jin fetch` track a specific branch or always use `origin/master`?

**Options**:

#### Option A: Single Master Branch (Recommended)
All modes, scopes, projects in single `master` branch as directories:
```
origin/master
  jin/
    global/
    mode/
      claude/
      cursor/
    scope/
      language:javascript/
    project/
      project-a/
```

**Pros**: Simple, no branch management, clear hierarchy
**Cons**: Can't have separate release cycles

#### Option B: Configurable Branch Tracking
Allow specifying branch in remote config:
```toml
[remote]
url = "git@github.com:myorg/jin-config"
branch = "stable"  # Track stable instead of master
```

**Pros**: Flexibility for different release tracks
**Cons**: Added complexity, per-project branch overrides?

**Recommendation**: **Option A** initially; add Option B if needed later

---

## 6. Key Insights & Recommendations for Jin

### 6.1 Remote Configuration Best Practices

1. **Store Remote URL Globally** (`~/.jin/config.toml`)
   - One source of truth across all projects
   - Auto-discovered when running `jin fetch`
   - Can re-link to different remote if needed

2. **Use Explicit Layer Precedence, Not Implicit Merging**
   - Clear 9-layer hierarchy prevents surprise conflicts
   - Pause on conflicts; require explicit resolution
   - Show layer composition via `jin layers` before applying

3. **Separate Shared (Remote) from Local (User)**
   - Remote: contains modes/scopes (shared by team)
   - User Local (Layer 8): machine-specific overrides
   - Project Local (Layer 7): project-specific customizations
   - Clear separation prevents surprises

4. **Never Store Secrets; Use Environment Variables**
   - Remote repo is readable by multiple developers
   - Store API keys, passwords outside Jin
   - Document required environment variables clearly

5. **Auto-Fetch on Link, But Require Explicit Pull**
   - `jin link <url>` automatically fetches available modes/scopes
   - Shows what's available to activate
   - User still controls when to pull updates
   - Prevents silent updates breaking workspace

### 6.2 Recommended Workflow for Teams

```bash
# --- Team Setup (once) ---
# Create shared repository
git init --bare jin-config.git

# Initialize with modes/scopes
git clone jin-config.git ~/.jin-shared
mkdir -p ~/.jin-shared/jin/mode/claude/
echo "..." > ~/.jin-shared/jin/mode/claude/config.json
cd ~/.jin-shared && git add . && git commit && git push
rm -rf ~/.jin-shared

# --- Developer Setup (new machine) ---
cd ~/projects/my-project
git clone git@github.com:myorg/my-project
jin init
jin link git@github.com:myorg/jin-config
# Auto-fetches; shows available modes/scopes

# --- Activate for this project ---
jin mode use claude           # From what was fetched
jin scope use language:javascript
jin pull                      # Merge into layers
jin apply                     # Apply to workspace

# --- Make changes ---
jin add .claude/settings.json --mode
jin commit -m "Update Claude settings"
jin push                      # Push back to team config

# --- On other machine, collaborator gets updates ---
jin fetch                     # Fetches team's latest
jin pull                      # Merges in
```

### 6.3 Implementation Checklist for P4M6

- [ ] **jin link <url>** command:
  - Stores URL in `~/.jin/config.toml`
  - Auto-fetches modes/scopes from remote
  - Shows available modes/scopes to user
  - Prompts if re-linking to different URL

- [ ] **jin fetch** command:
  - Reads remote URL from `~/.jin/config.toml`
  - Fetches all mode/scope/project refs from remote
  - Shows what's available or what's new
  - No merge yet (that's `jin pull`)

- [ ] **jin pull** command:
  - Requires prior `jin fetch`
  - Merges fetched data into local layers
  - Pauses on conflicts; shows `.jinmerge` files
  - User resolves and runs `jin add` + `jin commit`

- [ ] **jin push** command:
  - Requires prior `jin fetch` (to avoid divergence)
  - Requires clean merge state
  - Pushes local project/mode/scope changes to remote
  - Updates remote refs

- [ ] **Configuration updates**:
  - [ ] `RemoteConfig` in `src/core/config.rs` - Done (has `url`, `fetch_on_init`)
  - [ ] Per-project remote branch tracking (optional, v2)
  - [ ] Fetch behavior options (`fetch_on_init`, `fetch_on_enter`)

---

## 7. References

### Tools Researched

- **Chezmoi**: [https://www.chezmoi.io/](https://www.chezmoi.io/)
  - [Setup Guide](https://www.chezmoi.io/user-guide/setup/)
  - [Quick Start](https://www.chezmoi.io/quick-start/)

- **YADM**: [https://yadm.io/](https://yadm.io/)
  - [GitHub Repository](https://github.com/yadm-dev/yadm)
  - [Ubuntu Manual](https://manpages.ubuntu.com/manpages/focal/man1/yadm.1.html)

- **Homesick**: [https://github.com/technicalpickles/homesick](https://github.com/technicalpickles/homesick)
  - [RubyDoc Documentation](https://www.rubydoc.info/gems/homesick)

### Dotfiles Management Patterns

- [Dotfiles utilities reference](https://dotfiles.github.io/utilities/)
- [Atlassian: How to Store Dotfiles - Bare Repository](https://www.atlassian.com/git/tutorials/dotfiles)

### Git Workflows & Merge Strategies

- [Git Advanced Merging](https://git-scm.com/book/en/v2/Git-Tools-Advanced-Merging)
- [Atlassian: Resolving Merge Conflicts](https://www.atlassian.com/git/tutorials/using-branches/merge-conflicts)
- [Git Merge Strategies & Conflict Resolution](https://unstop.com/blog/merge-in-git)

### Security & Secrets Management

- [OWASP Secrets Management Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html)
- [GitGuardian: API Keys Security & Secrets Management](https://blog.gitguardian.com/secrets-api-management/)
- [BeyondTrust: Secrets Management Best Practices](https://www.beyondtrust.com/resources/glossary/secrets-management)
- [DigitalOcean: Managing Secrets Safely with Version Control](https://www.digitalocean.com/community/tutorials/an-introduction-to-managing-secrets-safely-with-version-control-systems)
- [Akeyless: Essential Guide to Secrets Management](https://www.akeyless.io/blog/the-essential-guide-to-secrets-management/)

### Git Worktree & Multi-Branch Workflows

- [Git Worktree Documentation](https://git-scm.com/docs/git-worktree)
- [DataCamp: Git Worktree Tutorial](https://www.datacamp.com/tutorial/git-worktree-tutorial)
- [GitKraken: How to Use Git Worktree](https://www.gitkraken.com/learn/git/git-worktree)
- [Git Tower: Working with Multiple Branches Using Git Worktree](https://www.git-tower.com/learn/git/faq/git-worktree)

---

## Appendix: Comparison Matrix

| Aspect | Chezmoi | YADM | Homesick | Jin (Proposed) |
|--------|---------|------|----------|----------------|
| **Remote Storage** | Git repo | Bare Git | Git castles | Unified Git repo |
| **Config Language** | Templates (Go) | Files + hooks | Files + symlinks | Structured (JSON/YAML) |
| **Hierarchy** | Templates + vars | Flat | Multiple castles | 9-layer explicit |
| **Shared vs Local** | Template vars | File naming | Castle priority | Layer precedence |
| **Conflict Resolution** | Manual review | Manual | Manual | Explicit pause + resolve |
| **Initial Fetch** | Auto (`init --apply`) | Auto (`clone`) | Auto (`clone`) | Auto (recommended) |
| **Per-Project Scope** | No (home dir focused) | No (home dir focused) | No (home dir focused) | Yes (project-specific) |
| **Team Scalability** | Single repo | Single repo | Multiple castles | Single repo + modes |

---

## Document Metadata

- **Created**: 2025-01-15
- **Phase**: P4M6 - Remote Configuration Repository Patterns
- **Status**: Research Complete - Ready for Implementation
- **Next Steps**: Reference this document when implementing `jin link`, `jin fetch`, `jin pull`, `jin push` commands
