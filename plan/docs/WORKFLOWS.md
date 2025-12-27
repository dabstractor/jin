# Jin Workflows

Complete workflow examples showing Jin's power through real-world scenarios. Each workflow is tested and verified.

## Table of Contents

1. [Basic Project Setup](#1-basic-project-setup)
2. [Multi-Layer Configuration](#2-multi-layer-configuration)
3. [Remote Sync & Collaboration](#3-remote-sync--collaboration)
4. [Deep Configuration Merge](#4-deep-configuration-merge)
5. [Scope-Based Configuration](#5-scope-based-configuration)
6. [Multi-Project Mode Sharing](#6-multi-project-mode-sharing)
7. [Error Recovery](#7-error-recovery)

---

## 1. Basic Project Setup

**Use Case**: Initialize Jin in a new project and set up basic development configuration.

### Steps

```bash
# Step 1: Initialize Jin in your project
$ cd my-project/
$ jin init
Jin initialized in current project

# Step 2: Create a development mode
$ jin mode create dev
Mode 'dev' created

# Step 3: Activate the mode for this project
$ jin mode use dev
Mode 'dev' activated

# Step 4: Create configuration files
$ mkdir .dev
$ echo '{"debug": true, "log_level": "trace"}' > .dev/config.json
$ echo '{"theme": "dark", "fontSize": 14}' > .dev/editor.json

# Step 5: Add files to mode layer
$ jin add .dev/config.json --mode
$ jin add .dev/editor.json --mode
Staged 2 files to mode layer

# Step 6: Commit changes
$ jin commit -m "Initial dev mode configuration"
Committed to mode/dev layer

# Step 7: Apply to workspace
$ jin apply
Applied 2 files to workspace:
  .dev/config.json
  .dev/editor.json

# Step 8: Verify files exist
$ cat .dev/config.json
{"debug": true, "log_level": "trace"}
```

**Result**: Development configuration is now managed by Jin and can be shared across all projects using the `dev` mode.

---

## 2. Multi-Layer Configuration

**Use Case**: Set up global defaults, mode-specific settings, and project-specific overrides demonstrating the layer precedence system.

### Steps

```bash
# Step 1: Set global defaults
$ echo '{"timeout": 30, "retries": 3, "verbose": false}' > defaults.json
$ jin add defaults.json --global
$ jin commit -m "Global default settings"
Committed to global layer

# Step 2: Create and configure Claude mode
$ jin mode create claude
$ jin mode use claude
$ echo '{"model": "claude-3-5-sonnet", "temperature": 1.0}' > ai-config.json
$ jin add ai-config.json --mode
$ jin commit -m "Claude mode AI configuration"
Committed to mode/claude layer

# Step 3: Override timeout for this specific project
$ echo '{"timeout": 5}' > defaults.json
$ jin add defaults.json --project
$ jin commit -m "Faster timeout for this project"
Committed to project/my-project layer

# Step 4: Add project-specific Claude settings
$ echo '{"model": "claude-3-opus"}' > ai-config.json
$ jin add ai-config.json --mode --project
$ jin commit -m "Use Opus for this project"
Committed to mode/claude/project/my-project layer

# Step 5: Apply all layers
$ jin apply
Applied 2 files from 4 layers

# Step 6: Verify merged configuration
$ cat defaults.json
{
  "timeout": 5,
  "retries": 3,
  "verbose": false
}

$ cat ai-config.json
{
  "model": "claude-3-opus",
  "temperature": 1.0
}
```

**Layer Breakdown**:
- `timeout`: 5 (from project layer, overrides global's 30)
- `retries`: 3 (from global layer, not overridden)
- `verbose`: false (from global layer, not overridden)
- `model`: "claude-3-opus" (from mode+project layer, overrides mode's "claude-3-5-sonnet")
- `temperature`: 1.0 (from mode layer)

---

## 3. Remote Sync & Collaboration

**Use Case**: Share configurations with teammates via a remote Git repository.

### Setup on First Machine

```bash
# Step 1: Create a remote repository for Jin configs
# (On GitHub/GitLab, create a new repository: team/jin-config)

# Step 2: Link local Jin to remote
$ jin link https://github.com/team/jin-config
Linked to remote: https://github.com/team/jin-config

# Step 3: Create shared team configurations
$ jin mode create team
$ jin mode use team

$ mkdir .team
$ echo '{"shared": true, "team_id": "alpha"}' > .team/config.json
$ jin add .team/config.json --mode
$ jin commit -m "Team shared configuration"

# Step 4: Add global defaults for the team
$ echo '{"company": "Acme Inc", "timezone": "UTC"}' > global-config.json
$ jin add global-config.json --global
$ jin commit -m "Global company settings"

# Step 5: Push to remote
$ jin push
Pushing to remote...
Pushed layers:
  - global
  - mode/team
Done
```

### Setup on Second Machine

```bash
# Step 1: Initialize Jin and link to remote
$ cd my-project/
$ jin init
$ jin link https://github.com/team/jin-config

# Step 2: Pull configurations from remote
$ jin pull
Pulling from remote...
Merged updates:
  - global: 1 file
  - mode/team: 1 file

# Step 3: Use the team mode
$ jin mode use team

# Step 4: Apply configurations
$ jin apply
Applied 2 files to workspace:
  global-config.json
  .team/config.json

# Step 5: Verify configurations
$ cat .team/config.json
{"shared": true, "team_id": "alpha"}
```

### Syncing Updates

```bash
# On any machine, after making changes:
$ jin sync
Syncing with remote...
Fetched updates
Merged 2 layers
Applied 3 files to workspace

Workspace synchronized
```

---

## 4. Deep Configuration Merge

**Use Case**: Demonstrate deep merging of nested JSON structures across layers.

### Steps

```bash
# Step 1: Set up base server configuration globally
$ cat > server.json << 'EOF'
{
  "server": {
    "host": "0.0.0.0",
    "port": 8080,
    "ssl": {
      "enabled": false,
      "cert": null
    }
  },
  "database": {
    "host": "localhost",
    "port": 5432,
    "name": "app_db"
  },
  "logging": {
    "level": "info",
    "format": "json",
    "outputs": ["stdout"]
  }
}
EOF

$ jin add server.json --global
$ jin commit -m "Base server configuration"

# Step 2: Create production mode with overrides
$ jin mode create prod
$ jin mode use prod

$ cat > server.json << 'EOF'
{
  "server": {
    "port": 443,
    "ssl": {
      "enabled": true,
      "cert": "/etc/ssl/cert.pem"
    }
  },
  "database": {
    "host": "prod-db.example.com"
  },
  "logging": {
    "level": "warn",
    "outputs": ["stdout", "file"]
  }
}
EOF

$ jin add server.json --mode
$ jin commit -m "Production server overrides"

# Step 3: Add project-specific database settings
$ cat > server.json << 'EOF'
{
  "database": {
    "name": "myproject_prod"
  },
  "logging": {
    "level": "error"
  }
}
EOF

$ jin add server.json --mode --project
$ jin commit -m "Project-specific database name"

# Step 4: Apply and see the deep merge result
$ jin apply
Applied 1 file from 3 layers

$ cat server.json
{
  "server": {
    "host": "0.0.0.0",
    "port": 443,
    "ssl": {
      "enabled": true,
      "cert": "/etc/ssl/cert.pem"
    }
  },
  "database": {
    "host": "prod-db.example.com",
    "port": 5432,
    "name": "myproject_prod"
  },
  "logging": {
    "level": "error",
    "format": "json",
    "outputs": ["stdout", "file"]
  }
}
```

**Merge Analysis**:
- `server.host`: "0.0.0.0" (from global - not overridden)
- `server.port`: 443 (from mode - overrides global's 8080)
- `server.ssl.enabled`: true (from mode - overrides global's false)
- `server.ssl.cert`: "/etc/ssl/cert.pem" (from mode - overrides global's null)
- `database.host`: "prod-db.example.com" (from mode - overrides global's localhost)
- `database.port`: 5432 (from global - not overridden)
- `database.name`: "myproject_prod" (from mode+project - overrides mode's implicit value)
- `logging.level`: "error" (from mode+project - overrides mode's "warn")
- `logging.format`: "json" (from global - not overridden)
- `logging.outputs`: ["stdout", "file"] (from mode - replaces global's array)

---

## 5. Scope-Based Configuration

**Use Case**: Use scopes to manage environment-specific (dev/staging/prod) and language-specific configurations.

### Steps

```bash
# Step 1: Create a mode for AI tooling
$ jin mode create claude
$ jin mode use claude

# Step 2: Set up base Claude configuration
$ echo '{"model": "claude-3-5-sonnet", "max_tokens": 4096}' > .claude/config.json
$ jin add .claude/config.json --mode
$ jin commit -m "Base Claude configuration"

# Step 3: Create environment-specific scopes
$ jin scope create env:dev --mode claude
$ jin scope create env:staging --mode claude
$ jin scope create env:prod --mode claude

# Step 4: Configure dev environment
$ jin scope use env:dev
$ echo '{"temperature": 1.0, "max_tokens": 8192}' > .claude/config.json
$ jin add .claude/config.json --mode --scope
$ jin commit -m "Dev environment: higher creativity and token limit"

# Step 5: Configure production environment
$ jin scope use env:prod
$ echo '{"temperature": 0.3, "max_tokens": 2048}' > .claude/config.json
$ jin add .claude/config.json --mode --scope
$ jin commit -m "Production: conservative settings"

# Step 6: Switch between environments
$ jin scope use env:dev
$ jin apply
$ cat .claude/config.json
{
  "model": "claude-3-5-sonnet",
  "temperature": 1.0,
  "max_tokens": 8192
}

$ jin scope use env:prod
$ jin apply
$ cat .claude/config.json
{
  "model": "claude-3-5-sonnet",
  "temperature": 0.3,
  "max_tokens": 2048
}

# Step 7: Create language-specific scope
$ jin scope create language:rust --mode claude
$ jin scope use language:rust

$ echo '{"system_prompt": "You are a Rust expert"}' > .claude/config.json
$ jin add .claude/config.json --mode --scope
$ jin commit -m "Rust-specific system prompt"

$ jin apply
$ cat .claude/config.json
{
  "model": "claude-3-5-sonnet",
  "max_tokens": 4096,
  "system_prompt": "You are a Rust expert"
}
```

**Use Multiple Scopes**: You can combine environment and language scopes for fine-grained control.

---

## 6. Multi-Project Mode Sharing

**Use Case**: Share mode configurations across multiple projects while maintaining project-specific overrides.

### Steps

```bash
# Step 1: Create a shared mode for VS Code settings
$ jin mode create vscode
$ jin mode use vscode

# Step 2: Configure base VS Code settings
$ mkdir .vscode
$ cat > .vscode/settings.json << 'EOF'
{
  "editor.fontSize": 14,
  "editor.tabSize": 2,
  "editor.formatOnSave": true,
  "files.autoSave": "onFocusChange"
}
EOF

$ jin add .vscode/settings.json --mode
$ jin commit -m "Base VS Code settings"

# Step 3: Navigate to first project
$ cd ~/projects/frontend-app
$ jin init
$ jin mode use vscode
$ jin apply

$ cat .vscode/settings.json
{
  "editor.fontSize": 14,
  "editor.tabSize": 2,
  "editor.formatOnSave": true,
  "files.autoSave": "onFocusChange"
}

# Step 4: Add JavaScript-specific override for this project
$ cat > .vscode/settings.json << 'EOF'
{
  "editor.tabSize": 4,
  "[javascript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  }
}
EOF

$ jin add .vscode/settings.json --mode --project
$ jin commit -m "JavaScript project overrides"
$ jin apply

$ cat .vscode/settings.json
{
  "editor.fontSize": 14,
  "editor.tabSize": 4,
  "editor.formatOnSave": true,
  "files.autoSave": "onFocusChange",
  "[javascript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  }
}

# Step 5: Navigate to second project (Rust)
$ cd ~/projects/rust-cli
$ jin init
$ jin mode use vscode
$ jin apply

$ cat .vscode/settings.json
{
  "editor.fontSize": 14,
  "editor.tabSize": 2,
  "editor.formatOnSave": true,
  "files.autoSave": "onFocusChange"
}

# Step 6: Add Rust-specific settings
$ cat > .vscode/settings.json << 'EOF'
{
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "rust-analyzer.checkOnSave.command": "clippy"
}
EOF

$ jin add .vscode/settings.json --mode --project
$ jin commit -m "Rust project settings"
$ jin apply

$ cat .vscode/settings.json
{
  "editor.fontSize": 14,
  "editor.tabSize": 2,
  "editor.formatOnSave": true,
  "files.autoSave": "onFocusChange",
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  },
  "rust-analyzer.checkOnSave.command": "clippy"
}
```

**Result**: Both projects share the base VS Code settings from the mode layer, but each has language-specific customizations in their respective mode+project layers.

---

## 7. Error Recovery

**Use Case**: Recover from common errors and fix Jin state issues.

### Scenario 1: Accidental Staging

```bash
# Accidentally staged wrong file
$ jin add sensitive-data.json --mode
Staged sensitive-data.json to mode layer

# Undo staging
$ jin reset
Unstaged 1 file
```

### Scenario 2: Wrong Layer

```bash
# Committed to wrong layer
$ jin add config.json --mode
$ jin commit -m "Config"

# Check what was committed
$ jin log --layer mode/dev
commit a1b2c3d
    Config

# Remove from mode layer and re-add to project layer
$ jin reset --mode --hard
Reset mode/dev layer

$ jin add config.json --project
$ jin commit -m "Config (project-specific)"
```

### Scenario 3: Merge Conflict

```bash
# After pulling from remote with conflicts
$ jin pull
Pulling from remote...
Error: Merge conflict in .dev/config.json

# View differences
$ jin diff
Conflict in .dev/config.json:
<<<<<<< local
{"debug": true}
=======
{"debug": false, "verbose": true}
>>>>>>> remote

# Resolve manually
$ cat > .dev/config.json << 'EOF'
{"debug": true, "verbose": true}
EOF

$ jin add .dev/config.json --mode
$ jin commit -m "Resolve merge conflict"
```

### Scenario 4: Corrupted State

```bash
# Jin state is corrupted
$ jin apply
Error: Invalid layer reference

# Run repair
$ jin repair --dry-run
Would repair:
  - Remove orphaned staging entry: old-file.json
  - Fix invalid context reference: mode/deleted-mode

$ jin repair
Repaired Jin state:
  - Removed 1 orphaned entry
  - Fixed 1 reference

$ jin apply
Applied 3 files to workspace
```

### Scenario 5: Undo Apply

```bash
# Applied wrong configuration
$ jin apply
Applied 5 files to workspace

# Reset workspace to clean state
$ jin reset --hard
Workspace reset to last known good state

# Or reset to specific layer state
$ jin mode unset
$ jin apply
Applied 2 files (no mode active)
```

---

## Workflow Quick Reference

| Workflow | Key Commands |
|----------|--------------|
| **Basic Setup** | `init → mode create → mode use → add --mode → commit → apply` |
| **Multi-Layer** | `add --global → add --mode → add --project → apply` |
| **Remote Sync** | `link → pull → mode use → apply` or `push` to share |
| **Deep Merge** | Use nested JSON across multiple layers, `apply` to merge |
| **Scopes** | `scope create → scope use → add --mode --scope → apply` |
| **Multi-Project** | `mode use` in each project, override with `--mode --project` |
| **Recovery** | `reset`, `repair`, `diff` to fix issues |

For detailed command reference, see [Commands](COMMANDS.md).
