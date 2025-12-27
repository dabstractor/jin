# Jin - Phantom Git Layer System

**Manage developer-specific and tool-specific configuration without contaminating your project's Git repository.**

---

## What is Jin?

Jin is a meta-versioning system layered on top of Git that manages developer-specific and tool-specific configuration (like `.claude/`, `.cursor/`, `.vscode/` settings) across multiple projects without polluting your primary Git repository. Think of it as "Git for your ignored files" with a powerful 9-layer precedence system.

**Key Benefits:**
- **Non-disruptive**: Works alongside Git, only touches ignored/untracked files
- **Deterministic**: Structured files (JSON/YAML/TOML) merge predictably across layers
- **Team-friendly**: Share base configurations while preserving local customizations
- **Remote sync**: Collaborate on tool configurations via shared repository

## Quick Start

### Installation

Install via Cargo:
```bash
cargo install jin
```

Or build from source:
```bash
git clone https://github.com/jin/jin
cd jin && cargo build --release
cp target/release/jin /usr/local/bin/
```

### Your First Jin Workflow

Initialize Jin in your project:
```bash
$ jin init
Jin initialized in current project
```

Create and activate a development mode:
```bash
$ jin mode create dev
Mode 'dev' created

$ jin mode use dev
Mode 'dev' activated
```

Add configuration to mode layer:
```bash
$ echo '{"debug": true}' > .dev/config.json
$ jin add .dev/config.json --mode
Staged .dev/config.json to mode layer

$ jin commit -m "Add dev configuration"
Committed to mode/dev layer
```

Apply configuration to workspace:
```bash
$ jin apply
Applied 1 file to workspace:
  .dev/config.json
```

That's it! Your configuration is now managed by Jin.

### Next Steps

- **New to Jin?** → Read the [Getting Started Guide](docs/GETTING_STARTED.md)
- **Want to understand layers?** → See [Layer System Explained](docs/LAYER_SYSTEM.md)
- **Need command reference?** → See [Commands](docs/COMMANDS.md)
- **Looking for examples?** → See [Common Workflows](docs/WORKFLOWS.md)
- **Troubleshooting?** → See [FAQ](docs/TROUBLESHOOTING.md)

## Installation

### Via Cargo (Recommended)

If you have Rust installed:
```bash
cargo install jin
```

### Via Homebrew (macOS)
```bash
brew install jin  # (if packaged)
```

### Build from Source

Prerequisites: Rust 1.70.0+ ([install from rustup.rs](https://rustup.rs/))

```bash
git clone https://github.com/jin/jin
cd jin
cargo build --release
cargo test  # Verify build
```

Binary will be at `target/release/jin`. Add to your PATH.

## Command Overview

Jin provides 32 commands organized by function:

**Core Commands**: `init`, `add`, `commit`, `status`, `context`

**Mode Management**: `mode create|use|list|delete|show|unset`

**Scope Management**: `scope create|use|list|delete|show|unset`

**Workspace Operations**: `apply`, `reset`

**Remote Sync**: `link`, `fetch`, `pull`, `push`, `sync`

**Inspection**: `diff`, `log`, `layers`, `list`, `repair`

**Utility**: `completion`

For complete reference, see [Command Documentation](docs/COMMANDS.md).

## Documentation

- [Getting Started Guide](docs/GETTING_STARTED.md) - Step-by-step tutorial
- [Layer System Explained](docs/LAYER_SYSTEM.md) - Understanding Jin's precedence system
- [Command Reference](docs/COMMANDS.md) - All 32 commands with examples
- [Common Workflows](docs/WORKFLOWS.md) - Real-world usage patterns
- [Troubleshooting](docs/TROUBLESHOOTING.md) - Common issues and solutions

## Why Jin?

Traditional approaches to developer configuration management have problems:

**Problem 1: Git Pollution**

Committing `.vscode/`, `.claude/`, `.cursor/` settings to Git pollutes the repository with tool-specific configuration that doesn't belong there.

**Problem 2: Ignored Files Lost**

Adding them to `.gitignore` means they're not versioned and can't be shared across machines or with teammates.

**Problem 3: Manual Synchronization**

Developers resort to copy-pasting config files between machines, leading to drift and inconsistency.

**Jin's Solution:**

Jin creates a separate "phantom" Git layer that versions ignored files independently from your main repository, with a 9-layer precedence system for sharing base configurations while allowing local overrides.

## How Jin Works

Jin introduces a **phantom Git layer** that:
1. Only touches files that are ignored or untracked by your main Git repository
2. Stores configurations in a separate Git repository at `~/.jin/`
3. Merges configurations from multiple layers based on active mode and scope
4. Applies the merged result to your workspace deterministically

### Layering System

Jin uses a 9-layer hierarchy where higher layers override lower layers:

```
Layer 9: Workspace Active (merged result, highest precedence)
Layer 8: User Local (machine-specific)
Layer 7: Project Base (project-specific)
Layer 6: Scope Base (scope defaults)
Layer 5: Mode → Project
Layer 4: Mode → Scope → Project
Layer 3: Mode → Scope
Layer 2: Mode Base (mode defaults)
Layer 1: Global Base (shared defaults, lowest precedence)
```

For detailed explanation, see [Layer System Documentation](docs/LAYER_SYSTEM.md).

## Example Use Cases

### Developer Configuration

Share VS Code, Cursor, or Claude settings across projects while preserving personal preferences:

```bash
# Share base settings with team
jin add .vscode/settings.json --mode

# Personal machine-specific overrides
jin add .vscode/personal.json --local
```

### Environment-Specific Configuration

Different configs for development vs. production:

```bash
# Development mode
jin mode create dev
jin mode use dev
jin add .env --mode

# Production mode
jin mode create prod
jin mode use prod
jin add .env --mode
```

### Tool-Specific Workflows

Separate configurations for different AI tools:

```bash
# Claude mode
jin mode create claude
jin add .claude/ --mode

# Cursor mode
jin mode create cursor
jin add .cursor/ --mode
```

## Features

- **9-Layer Precedence System**: Global → Mode → Scope → Project → Local with deterministic merging
- **Deep Merge for Structured Files**: JSON, YAML, TOML files merge intelligently by key
- **3-Way Merge for Text Files**: Conflict resolution like Git for text files
- **Mode & Scope Management**: Organize configurations by development environment and context
- **Remote Synchronization**: Share configurations via Git-based remote repository
- **Atomic Operations**: All commits are atomic and reversible
- **Automatic `.gitignore` Safety**: Jin-managed files are auto-ignored in main Git repo
- **Shell Completion**: Bash, Zsh, Fish, PowerShell support

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - See [LICENSE](LICENSE) for details

---

**Questions?** See [Troubleshooting](docs/TROUBLESHOOTING.md) or [open an issue](https://github.com/jin/jin/issues).
