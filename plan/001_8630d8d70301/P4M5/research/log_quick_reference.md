# Git Log Quick Reference Guide

## Rust git2 RevWalk API Quick Start

### Basic Setup
```rust
use git2::{Repository, Sort};

let repo = Repository::open(".")?;
let mut revwalk = repo.revwalk()?;
revwalk.push_head()?;
revwalk.set_sorting(Sort::TOPOLOGICAL)?;

for oid_result in revwalk {
    let oid = oid_result?;
    let commit = repo.find_commit(oid)?;
    // Process commit
}
```

### Core RevWalk Methods

| Method | Purpose | Example |
|--------|---------|---------|
| `set_sorting(Sort)` | Configure iteration order | `set_sorting(Sort::TOPOLOGICAL)` |
| `push(Oid)` | Include commit in walk | `push(oid)` |
| `push_head()` | Include HEAD | `push_head()` |
| `push_ref(str)` | Include ref | `push_ref("refs/heads/main")` |
| `push_glob(str)` | Include matching refs | `push_glob("refs/tags/*")` |
| `hide(Oid)` | Exclude commit | `hide(oid)` |
| `hide_head()` | Exclude HEAD | `hide_head()` |
| `hide_ref(str)` | Exclude ref | `hide_ref("origin/main")` |
| `simplify_first_parent()` | Linear history only | `simplify_first_parent()` |
| `reset()` | Clear configuration | `reset()` |

### Sorting Options

```rust
Sort::TOPOLOGICAL          // Respects commit graph structure
Sort::TIME                 // By commit timestamp
Sort::REVERSE              // Reverse chronological
Sort::TOPOLOGICAL | Sort::REVERSE  // Combine with bitwise OR
```

## Commit Metadata Extraction

### Key Methods

```rust
// Identity
commit.id()                          // Oid (SHA1)
commit.author()                      // Signature
commit.committer()                   // Signature

// Messages
commit.message()                     // Option<&str> (cleaned)
commit.summary()                     // Option<&str> (first line)
commit.body()                        // Option<&str> (rest)

// Time
commit.time()                        // Time { secs: i64, offset_minutes: i32 }

// Parents
commit.parent_count()                // usize
commit.parent(n)                     // Result<Commit>
commit.parent_id(n)                  // Oid
commit.parents()                     // Iterator<Commit>

// Tree
commit.tree()                        // Result<Tree>
commit.tree_id()                     // Oid
```

### Signature Extraction

```rust
let author = commit.author();
let name = author.name();              // Option<&str>
let email = author.email();            // Option<&str>
let when = author.when();              // Time struct
```

## Git Log Format Placeholders

### Quick Reference Table

| Placeholder | Description | Example |
|---|---|---|
| `%H` | Full hash | `1a2b3c4d5e6f7g8h9i0j` |
| `%h` | Short hash | `1a2b3c4` |
| `%s` | Subject/title | `Fix login bug` |
| `%b` | Body | Multi-line commit body |
| `%an` | Author name | `John Doe` |
| `%ae` | Author email | `john@example.com` |
| `%ar` | Author date (relative) | `2 hours ago` |
| `%ad` | Author date (formatted) | See `--date` option |
| `%cn` | Committer name | `Jane Smith` |
| `%ce` | Committer email | `jane@example.com` |
| `%d` | Decorations | `(HEAD -> main, origin/main)` |
| `%n` | Newline | Line break |
| `%Cred` | Red color | `\x1b[31m` |
| `%Cgreen` | Green color | `\x1b[32m` |
| `%Creset` | Reset color | `\x1b[0m` |

### Common Format Strings

```bash
# One-line with author
--pretty=format:"%h - %an: %s"

# With timestamp
--pretty=format:"%h [%ar] %s"

# Colorized with decorations
--pretty=format:"%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr)%C(bold blue)<%an>%Creset"

# Multi-line with body
--pretty=format:"%H%n%an%n%ad%n%n%B"

# Hash and message only
--pretty=format:"%h %s"
```

## Git Log Filtering Patterns

### Limiting Output
```bash
git log -n 10                         # Last 10 commits
git log --skip=5 -10                  # Skip 5, show next 10
```

### Date Range
```bash
git log --since="2 weeks ago"
git log --until="2022-08-30"
git log --since="2022-08-01" --until="2022-08-31"
```

### Author/Message
```bash
git log --author="John"               # Author name (regex)
git log --grep="bugfix"               # Search message (regex)
git log --grep="WIP" --invert-grep    # Exclude pattern
```

### Revision Ranges
```bash
git log origin..HEAD                  # In HEAD, not in origin
git log main...feature                # In either, not both
git log --all                         # All branches and tags
git log --first-parent                # Follow first parent only
```

### Path Filtering
```bash
git log -- src/main.rs                # Commits affecting file
git log -- src/                       # Commits affecting directory
git log --follow -- renamed_file.rs   # Follow renames
```

## Git Log Display Patterns

### Graph Visualization
```bash
# Basic graph
git log --graph --oneline

# Full graph with decorations
git log --graph --oneline --all --decorate

# Colorized graph
git log --graph --pretty=format:"%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr)%Creset" --abbrev-commit
```

### Diff Output
```bash
git log -p                            # Show patches
git log --stat                        # File change summary
git log --shortstat                   # Condensed stats
git log --name-only                   # Only filenames
git log --name-status                 # Filenames with status (A/M/D)
```

### Common Aliases (add to .gitconfig)

```bash
# Compact graph
git config --global alias.lg "log --graph --oneline --all --decorate"

# Detailed graph
git config --global alias.lga "log --graph --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset' --abbrev-commit"

# Simple list
git config --global alias.ls "log --oneline"

# With patches
git config --global alias.lp "log -p --graph --oneline"
```

## Pager Configuration

### Global Settings
```bash
# Disable paging
git config --global core.pager ""

# Use cat (no scrolling)
git config --global core.pager "cat"

# Use less with color
git config --global core.pager "less -R"

# Disable line wrapping in less
git config --global core.pager "less -S"
```

### One-off Disabling
```bash
git --no-pager log -p
git -P log -p
```

## Layer-Specific Log Patterns

### Path-Based Filtering
```bash
# Backend layer
git log --oneline -- api/ backend/ server/

# Frontend layer
git log --oneline -- ui/ frontend/ client/

# Database layer
git log --oneline -- database/ migrations/

# Combined with graph
git log --graph --oneline -- core/
```

### Tag-Based Filtering
```bash
# Infrastructure commits
git log --grep="[infra]" --oneline

# Feature commits
git log --grep="[feature]" --oneline

# Bug fixes
git log --grep="fix:" --oneline

# Multiple tags (OR)
git log --grep="[api]" --grep="[db]" --oneline
```

### Statistics
```bash
# Commits per author
git log --format="%an" | sort | uniq -c | sort -rn

# Commits per module
for dir in api/ backend/ frontend/; do
    echo "$dir: $(git log --oneline -- $dir | wc -l)"
done

# Lines changed per layer
for dir in api/ backend/ frontend/; do
    echo "$dir: $(git log --stat -- $dir | grep 'changed' | tail -1)"
done
```

## Performance Tips

### For Large Repositories
```rust
// Use depth-first search (generation numbers available in Git 2.36+)
revwalk.set_sorting(Sort::TOPOLOGICAL)?;

// Only walk needed ancestors
revwalk.hide_glob("refs/remotes/*")?;  // Exclude remote branches

// Stream commits instead of collecting all
for oid_result in revwalk {  // Iterator, not Vec
    let commit = repo.find_commit(oid_result?)?;
    // Process one at a time
}
```

### Format Strings
```bash
# Efficient: ~1KB per commit
git log --oneline -1000

# Memory-intensive: ~50KB per commit
git log -p -1000

# For large outputs, filter first
git log --author="John" -p | head -n 100
```

## Code Patterns for Jin Implementation

### Filter by Author
```rust
let mut revwalk = repo.revwalk()?;
revwalk.push_head()?;

for oid_result in revwalk {
    let commit = repo.find_commit(oid_result?)?;
    if commit.author().name().map_or(false, |name| name.contains("John")) {
        println!("{} - {}", &oid_result?.to_string()[..7], commit.summary()?);
    }
}
```

### Format Output
```rust
fn format_commit(commit: &git2::Commit) -> String {
    let hash = &commit.id().to_string()[..7];
    let author = commit.author().name().unwrap_or("Unknown");
    let message = commit.summary().unwrap_or("No message");
    format!("{} - {}: {}", hash, author, message)
}
```

### Pagination
```rust
let mut revwalk = repo.revwalk()?;
revwalk.push_head()?;

let mut count = 0;
let limit = 20;

for oid_result in revwalk {
    if count >= limit {
        println!("... (showing {} of many commits)", limit);
        break;
    }
    let commit = repo.find_commit(oid_result?)?;
    println!("{}", format_commit(&commit));
    count += 1;
}
```

---

**Quick Reference Version 1.0**
**For detailed information, see: log_research.md**
