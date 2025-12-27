# Getting Started with Jin

This guide walks you through your first Jin workflows step-by-step. By the end, you'll understand how to manage configuration across projects and machines using Jin's layer system.

**Time to Complete**: ~15 minutes

## Prerequisites

- Git installed and configured
- Rust 1.70+ installed (for cargo install)
- Familiarity with command-line basics
- An existing Git project (or create one: `git init test-project`)

## 1. Installation

Install Jin via Cargo:
```bash
$ cargo install jin
```

Verify installation:
```bash
$ jin --version
jin 0.1.0
```

## 2. Initialize Jin in Your Project

Navigate to your Git project:
```bash
$ cd your-project/
```

Initialize Jin:
```bash
$ jin init
Jin initialized in current project
```

**What just happened?**
- Jin created a `.jin/` directory with:
  - `context.json` - tracks active mode/scope for this project
  - `staging/` - temporary staging area (like Git's)
- Jin also initialized a bare repository at `~/.jin/` to store layer commits

Verify:
```bash
$ ls -la .jin/
total 8
drwxr-xr-x  3 user  staff    96 Dec 27 10:00 .
drwxr-xr-x  8 user  staff   256 Dec 27 10:00 ..
-rw-r--r--  1 user  staff    45 Dec 27 10:00 context.json
drwxr-xr-x  2 user  staff    64 Dec 27 10:00 staging
```

## 3. Create Your First Mode

Modes represent different development environments (like "development", "production", or tool-specific setups like "claude", "cursor").

Create a "dev" mode:
```bash
$ jin mode create dev
Mode 'dev' created
```

Activate it for this project:
```bash
$ jin mode use dev
Mode 'dev' activated in current project
```

Check active context:
```bash
$ jin context
Active mode: dev
Active scope: (none)
Project: your-project
```

## 4. Add Configuration to Mode Layer

Create a config file:
```bash
$ mkdir .dev
$ echo '{"debug": true, "log_level": "trace"}' > .dev/config.json
```

Stage it to the mode layer:
```bash
$ jin add .dev/config.json --mode
Staged .dev/config.json to mode layer
```

**Why `--mode` flag?**

Without flags, `jin add` targets the project-base layer. The `--mode` flag routes the file to the mode-base layer, making it available across ALL projects using this mode.

Commit the staged file:
```bash
$ jin commit -m "Add dev mode configuration"
Committed to mode/dev layer
Staging area cleared
```

## 5. Apply Configuration to Workspace

The configuration is now committed to the mode layer, but it's not yet in your working directory. Apply it:

```bash
$ jin apply
Applied 1 file to workspace:
  .dev/config.json
```

Verify:
```bash
$ cat .dev/config.json
{"debug": true, "log_level": "trace"}
```

**What just happened?**

Jin merged all applicable layers (in this case, just the mode/dev layer) and wrote the result to your workspace.

## 6. Understanding Layer Precedence

Let's add project-specific configuration that overrides the mode defaults.

Create project override:
```bash
$ echo '{"debug": false}' > .dev/config.json
```

Stage and commit to mode+project layer:
```bash
$ jin add .dev/config.json --mode --project
Staged .dev/config.json to mode+project layer

$ jin commit -m "Override debug setting for this project"
Committed to mode/dev/project/your-project layer
```

Apply:
```bash
$ jin apply
Applied 1 file to workspace:
  .dev/config.json
```

Check the result:
```bash
$ cat .dev/config.json
{"debug": false, "log_level": "trace"}
```

**Notice:**
- `debug` was overridden to `false` (from mode+project layer, higher precedence)
- `log_level` was preserved from mode layer (not overridden)

This is **deep merging** - Jin intelligently merges JSON across layers.

## 7. Working with Multiple Projects

Jin's power emerges when you work across multiple projects.

Navigate to another project:
```bash
$ cd ../another-project
$ git init  # Make it a Git project
$ jin init  # Initialize Jin
```

Use the same dev mode:
```bash
$ jin mode use dev
Mode 'dev' activated
```

Apply configurations:
```bash
$ jin apply
Applied 1 file to workspace:
  .dev/config.json
```

Check the file:
```bash
$ cat .dev/config.json
{"debug": true, "log_level": "trace"}
```

**Notice:** This project gets the mode-level defaults because we haven't created project-specific overrides yet.

## 8. Check Status

See what's in your workspace vs. what's committed:

```bash
$ jin status
Active mode: dev
Active scope: (none)

Workspace state: Clean
Staged changes: None

Layer summary:
  mode/dev: 1 file
```

## 9. Using Scopes for Context

Scopes add another dimension to your configuration (e.g., `language:javascript`, `env:production`).

Create a scope:
```bash
$ jin scope create env:prod
Scope 'env:prod' created

$ jin scope use env:prod
Scope 'env:prod' activated
```

Add production-specific config:
```bash
$ echo '{"log_level": "error"}' > .dev/config.json
$ jin add .dev/config.json --mode --scope
$ jin commit -m "Production logging config"
```

Apply and verify:
```bash
$ jin apply
$ cat .dev/config.json
{"debug": true, "log_level": "error"}
```

The scope layer overrode `log_level` while preserving `debug` from mode layer.

## 10. Remote Synchronization

Share your configurations across machines or with teammates.

Link to a remote repository:
```bash
$ jin link https://github.com/your-team/jin-config
Linked to remote: https://github.com/your-team/jin-config
```

Push your configurations:
```bash
$ jin push
Pushed layers to remote
```

On another machine, pull configurations:
```bash
$ jin pull
Pulled and merged layers from remote
$ jin apply
Applied configurations to workspace
```

## 11. Next Steps

Congratulations! You've completed your first Jin workflows.

**Continue Learning:**

- **Understand the 9-layer system** → See [Layer System Explained](LAYER_SYSTEM.md)
- **Learn all commands** → See [Command Reference](COMMANDS.md)
- **Explore advanced workflows** → See [Common Workflows](WORKFLOWS.md)
  - Remote synchronization across machines
  - Scope-based configurations (env:dev, env:prod)
  - Deep JSON merging examples
- **Troubleshooting** → See [FAQ](TROUBLESHOOTING.md)

## Quick Reference

**Initialize Jin:**
```bash
jin init
```

**Create & use mode:**
```bash
jin mode create <name>
jin mode use <name>
```

**Add files to layers:**
```bash
jin add <file>                    # project base
jin add <file> --mode             # mode base
jin add <file> --mode --project   # mode+project
```

**Commit & apply:**
```bash
jin commit -m "message"
jin apply
```

**Check status:**
```bash
jin status
jin context
```

**Remote sync:**
```bash
jin link <url>
jin push
jin pull
```

For complete documentation, see [README](../README.md).
