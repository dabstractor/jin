# CLI Alias Patterns Research: List Operations and Best Practices

## Executive Summary

This research document examines how popular CLI tools handle alias patterns for listing operations, with a focus on Git, package managers (cargo, npm, pip), and container tools (docker). The findings reveal common patterns, UX considerations, and best practices for implementing user-facing CLI aliases.

---

## 1. Git Alias Patterns

### 1.1 Core "List" Command Patterns

Git's most common alias patterns for listing operations include:

#### Branch Listing Aliases
- `br` or `branches` - Common alias for `git branch`
- `lb` or `lbranches` - List branches with extended information
- `gb` - Global branch listing across all remotes

#### Popular Git Aliases for Listing
```bash
# Basic branch listing
br = branch -a
brs = branch -a

# Detailed branch view
brl = "!f() { git branch -a --format=\"%(refname:short)|%(committerdate:relative)|%(committerdate:short)\" | column -t -s'|'; }; f"

# Branch with last commit info
brm = "!f() { git for-each-ref --format='%(refname:short) | %(committerdate:relative) | %(authorname) | %(subject)' refs/heads | sort -k2 -r; }; f"
```

#### Log Listing Patterns
```bash
# Short log with branch info
lg = log --graph --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr)%Creset' --abbrev-commit --date=relative

# One-line log with branch annotations
lol = log --oneline --graph --decorate --all

# All commits with branch context
la = log --oneline --all --graph --decorate
```

### 1.2 Git Alias Best Practices

1. **Naming Conventions**
   - Short, memorable names: `br`, `lg`, `co`
   - Consistent with existing command names
   - Avoid collision with built-in Git commands

2. **Output Formatting**
   - Preserve machine-parsable output when possible
   - Add human-readable columns for context
   - Use consistent formatting across aliases

3. **Documentation Pattern**
   - Include `git config --global alias.<name> '<command>'` examples
   - Explain the rationale for each alias
   - Show common usage patterns

---

## 2. Package Manager Alias Patterns

### 2.1 Cargo (Rust)

#### Common Alias Patterns
```bash
# List installed crates
crates = list --installed
crates-local = list --local
crates-outdated = list --outdated

# Show package information
pkg-info = tree
pkg-graph = tree --no-deps
```

#### Integration Patterns
- Cargo uses `.cargo/config.toml` for workspace-specific configurations
- Alias-like behavior achieved through wrapper scripts and shell functions
- `cargo-edit` and `cargo-audit` provide extended list functionality

### 2.2 NPM

#### Common Alias Patterns
```bash
# List dependencies
deps = list
dev-deps = list --dev
prod-deps = list --production
```

#### Security-Focused Pattern
Docker-based alias for security isolation:
```bash
alias npm='docker run --rm -it -v ${PWD}:${PWD} --net=host \
  -w ${PWD} node:alpine npm'
```

#### Configuration-Based Aliases
- Use `~/.npmrc` for repository aliases
- `npm config set registry <url>` for alternative package sources
- `npm exec` for tool-specific command shortcuts

### 2.3 Pip (Python)

#### Configuration File Approach
`~/.pip/pip.conf` supports multiple sections for different contexts:
```ini
[global]
index-url = https://pypi.org/simple

[list]
format = columns

[install]
trusted-host = pypi.org
```

#### Common Aliases
```bash
# List packages
pkgs = list
pkg-info = show
pkg-deps = show --requires
```

### 2.4 Docker

#### Image Listing Patterns
```bash
# Common aliases
imgs = images
cont = ps
cont-all = ps -a
vols = volume ls
net = network ls
```

#### Complex Pattern Examples
```bash
# Show container stats with formatting
stats-all = "ps -a --format 'table {{.Names}}\t{{.Status}}\t{{.Image}}\t{{.Ports}}'"

# List all resources
ls-all = "system df && echo '--- Images ---' && images && echo '--- Volumes ---' && volume ls"
```

---

## 3. General CLI List Command Patterns

### 3.1 Common Naming Patterns

#### Short Form Aliases
- `l` or `ls` for list operations
- `ll` for detailed listing (equivalent to `ls -l`)
- `la` for all-inclusive listing (equivalent to `ls -a`)

#### Descriptive Names
- `branches` for `git branch`
- `commits` for `git log`
- `packages` for `npm list`
- `images` for `docker images`

### 3.2 UX Best Practices for List Operations

#### 1. Output Formatting
```bash
# Good: Machine-readable with human options
git log --pretty=format:'%h %s' --oneline

# Good: Table format for tabular data
docker ps --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'

# Good: Columns for complex data
git branch --format='%(refname:short)|%(committerdate:short)|%(committerdate:relative)'
```

#### 2. Pagination Patterns
```bash
# Use `| less` for long output
git log --oneline --all | less

# Use `more` in command
docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}" | more
```

#### 3. Filtering and Search
```bash
# Pattern matching
git branch --list '*feature*'
npm list --depth=0 | grep 'package'

# Range-based filtering
git log --since="2024-01-01" --until="2024-12-31"
```

---

## 4. User Experience Considerations

### 4.1 Discoverability

1. **Help Integration**
   - All aliases should work with `--help` flag
   - Show alias definitions in help output
   - Include examples in documentation

2. **Intuitive Naming**
   - Follow established conventions (e.g., `ls` for list)
   - Use abbreviations that are easily guessable
   - Avoid cryptic names that require memorization

### 4.2 Consistency Patterns

1. **Verb-Noun Structure**
   - `list-branches`, `show-packages`, `view-images`
   - `ls`, `ll`, `la` for list variations

2. **Option Consistency**
   - `--all` for comprehensive view
   - `--format` for output customization
   - `--filter` or `--grep` for searching

### 4.3 Performance Considerations

1. **Lazy Loading**
   - Avoid expensive operations in aliases
   - Use flags to control performance impact

2. **Caching**
   - Cache expensive list operations when possible
   - Provide options to bypass cache

---

## 5. Documentation Patterns

### 5.1 Git Alias Documentation

```bash
# Example from popular Git aliases
#
# Description: List all branches with last commit info
# Usage: git brl
# Configuration:
#   git config --global brl "!f() { git for-each-ref --format='%(refname:short) | %(committerdate:relative) | %(authorname) | %(subject)' refs/heads | sort -k2 -r; }; f"
#
# The format shows:
#   - Branch name
#   - Relative time since last commit
#   - Commit author name
#   - Commit subject (title)
```

### 5.2 Package Manager Documentation

#### NPM Example
```json
{
  "name": "my-project",
  "scripts": {
    "list-deps": "npm list --depth=0",
    "list-deps-tree": "npm list",
    "list-outdated": "npm outdated"
  }
}
```

#### Pip Configuration
```ini
# ~/.pip/pip.conf
[list]
format = columns
verbose

[global]
timeout = 60
retries = 3
```

### 5.3 Docker Alias Documentation

```bash
# ~/.docker/alias
# imgs - List images with size and ID
# Usage: docker imgs
imgs="images --format 'table {{.Repository}}\t{{.Tag}}\t{{.ID}}\t{{.Size}}'"

# cont-all - List all containers including stopped
# Usage: docker cont-all
cont-all="ps -a --format 'table {{.Names}}\t{{.Status}}\t{{.Image}}'"
```

---

## 6. Recommended Patterns for Jin

Based on the research, here are recommended patterns for implementing list aliases in Jin:

### 6.1 Primary Alias Suggestions
```bash
# Core list operations
ls = list              # Main list command
ll = list --long       # Detailed view with more info
la = list --all        # Show all items including hidden/hidden
lt = list --tree       # Tree-like view for hierarchical data

# Layer-specific aliases
layers = list --type=layer    # List only layers
layers-all = list --all --type=layer  # List all layers including hidden

# Commit-related listings
commits = list --type=commit   # List commits
commits-graph = list --type=commit --graph  # Graph view
```

### 6.2 Best Practices Implementation
1. **Consistent Output Format**
   - Use tabular format for structured data
   - Provide machine-readable output options
   - Include human-readable columns by default

2. **Help Integration**
   - All aliases should support `--help`
   - Show alias expansion in help
   - Include examples for common use cases

3. **Performance Considerations**
   - Avoid expensive operations by default
   - Provide flags for detailed views
   - Support filtering to reduce output

4. **Documentation Strategy**
   - Inline documentation with examples
   - Show actual command being executed
   - Include usage tips and common patterns

---

## 7. Common Pitfalls to Avoid

### 7.1 Breaking Changes
- Never change the output format of existing aliases
- Maintain backward compatibility with flags
- Deprecate carefully with warnings

### 7.2 Output Parsing
- Avoid output formats that break parsing
- Use stable delimiters in tabular output
- Provide machine-readable alternatives

### 7.3 Namespace Collision
- Avoid aliasing core system commands (like `ls`)
- Use unique prefixes for tool-specific aliases
- Document potential conflicts

---

## Sources

1. [Git Aliases for Faster and Productive Git Workflow](https://snyk.io/blog/10-git-aliases-for-faster-and-productive-git-workflow/)
2. [10 Levels of Git Aliases: Advanced and Beyond](https://www.eficode.com/blog/10-levels-of-git-aliases-advanced-and-beyond)
3. [Git Aliases and Shortcuts for Daily Use](https://some-natalie.dev/blog/git-aliases/)
4. [UX Patterns for CLI Tools](https://lucasfcosta.com/blog/ux-patterns-cli-tools)
5. [Command Line Interface Guidelines](https://clig.dev/)
6. [Atlassian's 10 Design Principles for Delightful CLIs](https://www.atlassian.com/blog/it-teams/10-design-principles-for-delightful-clis)
7. [Must Have Git Aliases: Advanced Examples](https://www.durdn.com/blog/2012/11/22/must-have-git-aliases-advanced-examples/)
8. [Graphite Git List Branches Guide](https://graphite.com/guides/git-list-branches)
9. [ThoughtWorks CLI Design Guidelines](https://www.thoughtworks.com/insights/blog/engineering-effectiveness/elevate-developer-experiences-cli-design-guidelines)
10. [The Ultimate CLI Cheat-sheets Collection](https://jinaldesai.com/the-ultimate-cli-cheat-sheets-collection/)