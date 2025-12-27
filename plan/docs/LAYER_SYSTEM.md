# Understanding Jin's Layer System

Jin's power comes from its 9-layer precedence system. This guide explains how layers work, how they stack, and how Jin resolves conflicts when multiple layers define the same configuration.

## Table of Contents

1. [What Are Layers?](#what-are-layers)
2. [The 9-Layer Hierarchy](#the-9-layer-hierarchy)
3. [Precedence Rules](#precedence-rules)
4. [Simple Example: Two Layers](#simple-example-two-layers)
5. [Complex Example: Multi-Layer Precedence](#complex-example-multi-layer-precedence)
6. [Conflict Resolution](#conflict-resolution)
7. [Layer Routing (Which Flags Target Which Layers)](#layer-routing)
8. [Deep Merge Behavior](#deep-merge-behavior)

---

## What Are Layers?

Think of layers like transparent sheets stacked on top of each other. Each sheet can have different configurations, and when you look through the stack, what you see is the combination of all sheets, with sheets higher in the stack blocking or modifying what's below.

**In Jin:**
- Each layer can contain configuration files (JSON, YAML, TOML, text)
- Layers stack from bottom (lowest precedence) to top (highest precedence)
- Higher layers override lower layers when the same setting exists in multiple layers
- Files are deep-merged intelligently (not just replaced)

**Analogy**: CSS Cascade

If you're familiar with CSS, Jin's layer system works similarly to CSS specificity - more specific rules override more general ones.

---

## The 9-Layer Hierarchy

Jin defines 9 layers with strict precedence ordering:

```
┌─────────────────────────────────────────────┐
│  9. Workspace Active (Derived, Highest)     │ ← Highest precedence
├─────────────────────────────────────────────┤
│  8. User Local (~/.jin/local/)              │
├─────────────────────────────────────────────┤
│  7. Project Base (project/<project>/)       │
├─────────────────────────────────────────────┤
│  6. Scope Base (scope/<scope>/)             │
├─────────────────────────────────────────────┤
│  5. Mode → Project                          │
├─────────────────────────────────────────────┤
│  4. Mode → Scope → Project                  │
├─────────────────────────────────────────────┤
│  3. Mode → Scope                            │
├─────────────────────────────────────────────┤
│  2. Mode Base (mode/<mode>/)                │
├─────────────────────────────────────────────┤
│  1. Global Base (global/) (Lowest)          │ ← Lowest precedence
└─────────────────────────────────────────────┘

         Precedence Flow: Bottom → Top
        (Higher layers override lower layers)
```

**Key Principle**: Higher number = higher precedence

---

## Precedence Rules

When the same configuration key exists in multiple layers:

1. **Layer 9 (Workspace Active) wins** - always (it's the merged result)
2. **Layer 8 (User Local) wins** - machine-specific overrides
3. **Layer 7 (Project Base) wins** - project-specific configs
4. **Layer 6 (Scope Base) wins** - scope overrides
5. **Layer 5 (Mode → Project) wins** - mode+project combination
6. **Layer 4 (Mode → Scope → Project) wins** - all three combined
7. **Layer 3 (Mode → Scope) wins** - mode+scope
8. **Layer 2 (Mode Base) wins** - mode defaults
9. **Layer 1 (Global Base)** - fallback defaults

**Specificity Principle** (from Ansible):

"The more specific wins against the more general"

---

## Simple Example: Two Layers

Let's start with just two layers: **Global Base** and **Mode Base**.

**Scenario**: You want default settings globally, but different settings when in "dev" mode.

### Step 1: Set Global Defaults

```bash
$ echo '{"timeout": 30, "retries": 3}' > defaults.json
$ jin add defaults.json --global
$ jin commit -m "Global defaults"
```

This stores `defaults.json` in **Layer 1 (Global Base)**.

### Step 2: Override in Dev Mode

```bash
$ jin mode create dev
$ jin mode use dev
$ echo '{"timeout": 5}' > defaults.json
$ jin add defaults.json --mode
$ jin commit -m "Dev mode: faster timeout"
```

This stores `defaults.json` in **Layer 2 (Mode Base)**.

### Step 3: Apply and See Result

```bash
$ jin apply
$ cat defaults.json
{
  "timeout": 5,
  "retries": 3
}
```

**What happened:**
- `timeout` was overridden to `5` (from Mode Base, higher precedence)
- `retries` was preserved from Global Base (not overridden)
- Jin performed a **deep merge**, not a replacement

**Without Mode Active:**
```bash
$ jin mode unset
$ jin apply
$ cat defaults.json
{
  "timeout": 30,
  "retries": 3
}
```

Global defaults are used when no mode is active.

---

## Complex Example: Multi-Layer Precedence

Let's use 4 layers to show how Jin resolves conflicts.

**Layers:**
1. **Global Base**: Default settings for all projects and modes
2. **Mode Base**: "claude" mode defaults
3. **Project Base**: Project-specific settings
4. **Mode → Project**: Claude mode settings for this specific project

### Setup

```bash
# Layer 1: Global defaults
echo '{"ai": "assistant", "model": "default", "temp": 0.7}' > config.json
jin add config.json --global
jin commit -m "Global: Default AI settings"

# Layer 2: Mode defaults (Claude mode)
jin mode create claude
jin mode use claude
echo '{"model": "claude-3-5-sonnet", "temp": 1.0}' > config.json
jin add config.json --mode
jin commit -m "Claude mode: Use Claude model"

# Layer 7: Project-specific
echo '{"temp": 0.5}' > config.json
jin add config.json --project
jin commit -m "Project: Lower temperature"

# Layer 5: Mode + Project
echo '{"model": "claude-3-opus"}' > config.json
jin add config.json --mode --project
jin commit -m "This project: Use Opus in Claude mode"
```

### Result After Apply

```bash
$ jin apply
$ cat config.json
{
  "ai": "assistant",
  "model": "claude-3-opus",
  "temp": 0.5
}
```

**Breakdown:**
- `ai`: From Global Base (Layer 1) - not overridden anywhere
- `model`: From Mode → Project (Layer 5) - highest layer defining this key
- `temp`: From Project Base (Layer 7) - higher than Mode Base (Layer 2)

**Precedence Table:**

| Key | Global (1) | Mode (2) | Project (7) | Mode+Project (5) | **Result** |
|-----|-----------|----------|-------------|------------------|------------|
| ai | assistant | - | - | - | **assistant** (Global) |
| model | default | claude-3-5-sonnet | - | claude-3-opus | **claude-3-opus** (M+P) |
| temp | 0.7 | 1.0 | 0.5 | - | **0.5** (Project) |

---

## Conflict Resolution

When multiple layers define the same configuration key, Jin uses **deep merge** for structured files.

### Conflict Resolution Table

| Layer 1 | Layer 2 | Layer 3 | **Result** |
|---------|---------|---------|------------|
| `{"a": 1}` | `{"b": 2}` | - | `{"a": 1, "b": 2}` |
| `{"a": 1}` | `{"a": 2}` | - | `{"a": 2}` (L2 wins) |
| `{"a": 1}` | `{"a": 2}` | `{"a": 3}` | `{"a": 3}` (L3 wins, highest) |
| `{"obj": {"x": 1}}` | `{"obj": {"y": 2}}` | - | `{"obj": {"x": 1, "y": 2}}` (deep merge) |
| `{"arr": [1, 2]}` | `{"arr": [3, 4]}` | - | `{"arr": [3, 4]}` (L2 replaces, unkeyed array) |
| `{"key": "val"}` | `{"key": null}` | - | `{}` (null deletes key) |

**Key Principles:**
- **Deep Merge**: Nested objects are merged recursively
- **Highest Wins**: When same key exists at multiple layers, highest layer value wins
- **Null Deletes**: Setting a key to `null` in a higher layer removes it
- **Arrays**: Unkeyed arrays are replaced, keyed arrays (with `id` or `name`) are merged by key

---

## Layer Routing

The flags you use with `jin add` determine which layer the file goes to:

| Command | Target Layer | Precedence |
|---------|--------------|------------|
| `jin add <file>` | Project Base | 7 |
| `jin add <file> --mode` | Mode Base | 2 |
| `jin add <file> --mode --project` | Mode → Project | 5 |
| `jin add <file> --scope <scope>` | Scope Base | 6 |
| `jin add <file> --mode --scope <scope>` | Mode → Scope | 3 |
| `jin add <file> --mode --scope <scope> --project` | Mode → Scope → Project | 4 |
| `jin add <file> --global` | Global Base | 1 |

**Active Context Behavior:**

When you activate a mode and/or scope with `jin mode use` and `jin scope use`, it affects which layers are applied during `jin apply`, but NOT which layer files are added to unless you use the corresponding flags.

**Example:**
```bash
$ jin mode use claude
$ jin add config.json          # Goes to Project Base (Layer 7), NOT Mode Base
$ jin add config.json --mode   # Goes to Mode Base (Layer 2) because of --mode flag
```

---

## Deep Merge Behavior

Jin performs intelligent deep merging for structured file formats (JSON, YAML, TOML).

### Example: Nested Object Merge

**Layer 1 (Global):**
```json
{
  "server": {
    "host": "0.0.0.0",
    "port": 8080,
    "ssl": false
  },
  "logging": {
    "level": "info",
    "format": "json"
  }
}
```

**Layer 2 (Mode):**
```json
{
  "server": {
    "port": 3000,
    "ssl": true
  },
  "logging": {
    "level": "debug"
  }
}
```

**Result After Merge:**
```json
{
  "server": {
    "host": "0.0.0.0",        // Preserved from Global
    "port": 3000,             // Overridden by Mode
    "ssl": true               // Overridden by Mode
  },
  "logging": {
    "level": "debug",         // Overridden by Mode
    "format": "json"          // Preserved from Global
  }
}
```

**Key Insight**: Jin merges **keys**, not entire objects. Only the specific keys that conflict are overridden.

### File Type Behavior

| File Type | Merge Strategy |
|-----------|----------------|
| JSON, YAML, TOML | Deep merge by key |
| Text files (.txt, .md) | 3-way diff merge (like Git) |
| Binary files | Higher layer replaces (no merge) |

---

## Common Patterns

### Pattern 1: Global Defaults with Mode Overrides

Use Case: Set sensible defaults globally, override per mode.

```bash
# Set global defaults
jin add defaults.json --global

# Override in specific modes
jin mode use dev
jin add dev-overrides.json --mode

jin mode use prod
jin add prod-overrides.json --mode
```

### Pattern 2: Team Shared + Personal Overrides

Use Case: Share base configuration with team, personalize locally.

```bash
# Team shared (committed to jin remote repository)
jin add .vscode/settings.json --mode

# Personal local (never synced to remote)
jin add .vscode/personal.json --local  # Layer 8, highest user precedence
```

### Pattern 3: Project-Specific Customization

Use Case: Use mode defaults across projects, customize for one project.

```bash
# Mode defaults apply to all projects
jin mode use claude
jin add .claude/base-config.json --mode

# This project needs different settings
jin add .claude/project-config.json --mode --project  # Overrides mode defaults
```

---

## Summary

**Key Takeaways:**

1. **9 layers** with strict precedence: higher layers override lower layers
2. **Deep merge**: Structured files merge intelligently by key, not replaced wholesale
3. **Flags determine layer**: `--mode`, `--scope`, `--project` flags control where files go
4. **Specificity wins**: More specific layers (mode+scope+project) beat general ones (global)
5. **Null deletes**: Use `null` to remove keys from higher layers
6. **Text files use 3-way merge**: Like Git, for non-structured files

**Next Steps:**

- **See all commands** → [Command Reference](COMMANDS.md)
- **Real-world examples** → [Common Workflows](WORKFLOWS.md)
- **Troubleshooting** → [FAQ](TROUBLESHOOTING.md)
