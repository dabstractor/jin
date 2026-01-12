# Git Log Command Implementation Research

## Overview

This document provides comprehensive research on git log command implementation patterns, including traversal algorithms, Rust git2 library APIs, formatting patterns, filtering strategies, and layer-specific display techniques.

---

## 1. Git Log Traversal Algorithms

### 1.1 Breadth-First vs Depth-First Search (BFS vs DFS)

**Source:** [Git's Database Internals II: Commit History Queries](https://github.blog/2022-08-30-gits-database-internals-ii-commit-history-queries/)

#### Breadth-First Search (Original Approach)
- Explores all recent commits first before determining commit reachability
- Limitation: Does not help if target commit is unreachable (explores unnecessarily)
- Performance: Minimizes walks if target is found, but exhaustive if target is unreachable

#### Depth-First Search (With Generation Numbers)
- Follows first-parent history to navigate commit graph efficiently
- Exploits repository structure: first parent represents pre-merge branch state
- Performance: Reaches generation number cutoff faster than BFS
- Key advantage: Finds merge commits introducing target commits sooner

### 1.2 Generation Numbers as Performance Indexes

**Concept:** Generation numbers function as negative reachability indexes for optimization

**Key Property:**
```
If generation(A) < generation(B), then A cannot reach B
```

**Two Implementation Levels:**

1. **Topological Level (Simple)**
   - Root commits: level 1
   - Each commit: level = 1 + max(parent levels)
   - Provides basic reachability filtering

2. **Corrected Commit Dates (Advanced)**
   - Uses actual timestamps when available
   - Better performance for commits based on older code
   - More sophisticated than topological levels

**Performance Impact Example:**
```
Operation: git tag --contains v5.19~100 (Linux kernel)
Without generation numbers: 7.34 seconds
With generation numbers: 0.04 seconds
```

### 1.3 Commit Walking Strategy

**Pattern:**
1. Start from leaf commits (branches, tags, HEAD)
2. Traverse to parent commits following parent pointers
3. Apply inclusion/exclusion rules during traversal
4. Order commits using specified sorting algorithm (topological, date, reverse)
5. Yield commits matching all filters

---

## 2. Rust git2 Library RevWalk and Log APIs

**Official Documentation:** [git2::Revwalk - Rust](https://docs.rs/git2/latest/git2/struct.Revwalk.html)

### 2.1 RevWalk Core Struct

The `Revwalk` struct enables traversal of the commit graph defined by:
- One or more leaf commits (start points)
- One or more root commits (exclusion points)

**Key Feature:** Automatically resets when iteration completes

### 2.2 RevWalk Methods - Sorting and Configuration

```rust
pub fn set_sorting(&mut self, sort: Sort) -> Result<(), git2::Error>
pub fn simplify_first_parent(&mut self) -> Result<(), git2::Error>
pub fn reset(&mut self) -> Result<(), git2::Error>
```

**Sorting Options:**
- `Sort::TOPOLOGICAL` - Topological order (respects commit graph structure)
- `Sort::TIME` / `Sort::DATE_ORDER` - By commit timestamp (most recent first)
- `Sort::REVERSE` - Reverse chronological order
- Combine with bitwise OR: `TOPOLOGICAL | REVERSE`

**simplify_first_parent():** Limits traversal to only the first parent of each commit (useful for linear history view)

### 2.3 RevWalk Methods - Pushing Commits (Inclusion)

```rust
pub fn push(&mut self, oid: Oid) -> Result<(), git2::Error>
pub fn push_head(&mut self) -> Result<(), git2::Error>
pub fn push_ref(&mut self, reference: &str) -> Result<(), git2::Error>
pub fn push_glob(&mut self, glob: &str) -> Result<(), git2::Error>
pub fn push_range(&mut self, range: &str) -> Result<(), git2::Error>
```

**Semantics:**
- At least one commit must be pushed before walk can start
- `push_head()`: Include current branch HEAD
- `push_ref("refs/heads/main")`: Include specific branch
- `push_glob("refs/tags/*")`: Include all matching refs
- `push_range("origin..HEAD")`: Include range (symmetric difference)

### 2.4 RevWalk Methods - Hiding Commits (Exclusion)

```rust
pub fn hide(&mut self, oid: Oid) -> Result<(), git2::Error>
pub fn hide_head(&mut self) -> Result<(), git2::Error>
pub fn hide_ref(&mut self, reference: &str) -> Result<(), git2::Error>
pub fn hide_glob(&mut self, glob: &str) -> Result<(), git2::Error>
pub fn with_hide_callback<C>(&mut self, callback: C) -> Result<(), git2::Error>
  where C: FnMut(Oid) -> bool + 'static
```

**hide_callback Semantics:**
- Callback returns `true` to exclude commit and ancestors
- Custom filtering logic for selective hiding

### 2.5 RevWalk Iterator Implementation

```rust
impl<'repo> Iterator for Revwalk<'repo> {
    type Item = Result<Oid, git2::Error>;

    fn next(&mut self) -> Option<Self::Item> { ... }
}
```

**Usage Pattern:**
```rust
let mut revwalk = repo.revwalk()?;
revwalk.push_head()?;
revwalk.set_sorting(Sort::TOPOLOGICAL)?;

for oid_result in revwalk {
    let oid = oid_result?;
    let commit = repo.find_commit(oid)?;
    // Process commit
}
```

### 2.6 Repository Methods for Commit Discovery

**Source:** [git2::Repository - Rust](https://docs.rs/git2/latest/git2/struct.Repository.html)

```rust
pub fn revwalk(&self) -> Result<Revwalk, Error>
pub fn find_commit(&self, oid: Oid) -> Result<Commit, Error>
pub fn find_commit_by_prefix(&self, prefix: &str) -> Result<Commit, Error>
pub fn find_annotated_commit(&self, id: Oid) -> Result<AnnotatedCommit, Error>
pub fn revparse(&self, spec: &str) -> Result<Revspec, Error>
pub fn revparse_single(&self, spec: &str) -> Result<Object, Error>
pub fn revparse_ext(&self, spec: &str) -> Result<(Object, Reference), Error>
pub fn graph_ahead_behind(&self, local: Oid, upstream: Oid) -> Result<(usize, usize), Error>
```

**Common Patterns:**
- `revwalk()`: Returns iterator over commits
- `find_commit(oid)`: Load specific commit by OID
- `revparse(spec)`: Parse revision specifications (e.g., "HEAD~5", "main..develop")
- `graph_ahead_behind()`: Count commits ahead/behind upstream

---

## 3. Commit Metadata Extraction

**Source:** [git2::Commit - Rust](https://docs.rs/git2/latest/git2/struct.Commit.html)

### 3.1 Commit Object Methods

```rust
pub fn id(&self) -> Oid                              // SHA1 hash
pub fn tree(&self) -> Result<Tree, Error>           // Associated tree object
pub fn tree_id(&self) -> Oid                        // Tree OID without fetch
pub fn parent_count(&self) -> usize                 // Number of parents
pub fn parent(&self, n: usize) -> Result<Commit>   // Get specific parent
pub fn parent_id(&self, n: usize) -> Oid            // Parent OID without fetch
pub fn parents(&self) -> Parents                     // Iterator over parents
pub fn parent_ids(&self) -> ParentIds               // Iterator over parent OIDs
```

### 3.2 Message Extraction Methods

```rust
pub fn message(&self) -> Option<&str>               // UTF-8 validated message
pub fn message_bytes(&self) -> &[u8]                // Message as bytes
pub fn message_raw(&self) -> &str                   // Unprettified message
pub fn message_raw_bytes(&self) -> &[u8]            // Raw as bytes
pub fn message_encoding(&self) -> Option<&str>      // Encoding name
pub fn summary(&self) -> Option<&str>               // Short summary (first line)
pub fn summary_bytes(&self) -> &[u8]                // Summary as bytes
pub fn body(&self) -> Option<&str>                  // Full body (excluding summary)
pub fn body_bytes(&self) -> &[u8]                   // Body as bytes
```

### 3.3 Author and Committer Extraction

```rust
pub fn author(&self) -> Signature                   // Author signature
pub fn author_with_mailmap(&self, mailmap: &Mailmap) -> Option<Signature>
pub fn committer(&self) -> Signature                // Committer signature
pub fn committer_with_mailmap(&self, mailmap: &Mailmap) -> Option<Signature>
```

**Signature Struct Fields:**
```rust
pub struct Signature<'repo> {
    pub name: Option<&'repo str>,
    pub email: Option<&'repo str>,
    pub when: Time,                // Timestamp and offset
}

pub struct Time {
    pub secs: i64,                 // Seconds since epoch
    pub offset_minutes: i32,       // Timezone offset
}
```

### 3.4 Time Handling

```rust
pub fn time(&self) -> Time                          // Commit timestamp
```

**Time Conversion Pattern:**
```rust
let time = commit.time();
let timestamp = time.secs;                // Unix timestamp
let offset = time.offset_minutes;         // Timezone offset in minutes

// Convert to DateTime (using time crate)
let tm = time::at(time::Timespec::new(timestamp, 0));
println!("{}", tm.rfc822());              // RFC 2822 format
```

---

## 4. Log Formatting and Display Patterns

**Source:** [Git - pretty-formats Documentation](https://git-scm.com/docs/pretty-formats)

### 4.1 Built-in Format Options

| Format | Output Pattern | Use Case |
|--------|---|---|
| `oneline` | `<hash> <title-line>` | Quick overview, branch lists |
| `short` | Hash + author + title | Summary view |
| `medium` | + message date (default) | Standard view |
| `full` | + committer info | Full details |
| `fuller` | Full dates for author & committer | Timestamp precision needed |
| `reference` | `<hash> (<title>, <short-date>)` | References, citations |
| `email` | Email-style format | Patch submission |
| `raw` | Raw commit object | Low-level inspection |

### 4.2 Custom Format Syntax

**Format Strings:**
```bash
# Basic format with separator semantics (newline between entries)
git log --pretty=format:"<format-string>"

# Terminator semantics (newline after each entry)
git log --pretty=tformat:"<format-string>"
```

**Difference:**
- `format:` - Uses separator semantics (no trailing newline on last entry)
- `tformat:` - Uses terminator semantics (newline after each entry)

### 4.3 Format Modifiers

```
%+<placeholder>  - Insert line-feed before expansion if non-empty
%-<placeholder>  - Delete preceding line-feeds if expansion is empty
% <placeholder>  - Insert space before expansion if non-empty
```

**Example:**
```bash
# Header with line-feed separator
git log --pretty=format:"%n%Cred%h%Creset -%C(yellow)%d%Creset%n%s%n%b"
```

### 4.4 Color and Formatting Placeholders

```
%Cred        - Red text
%Cgreen      - Green text
%Cblue       - Blue text
%Creset      - Reset color
%C(<spec>)   - Custom color specification
%m           - Merge mark (< for left, > for right, - for merge)
%w([<w>[,<i1>[,<i2>]]])  - Line wrapping options
%<(<n>)      - Right-align to N columns with truncation
%>(<n>)      - Left-align to N columns
%><(<n>)     - Center padding to N columns
```

### 4.5 Commit Information Placeholders

```
%H    - Full commit hash (40 chars)
%h    - Abbreviated commit hash (7 chars default)
%T    - Tree hash
%t    - Abbreviated tree hash
%P    - Parent hashes
%p    - Abbreviated parent hashes
%s    - Subject/title line
%b    - Body (multiple paragraphs)
%B    - Raw body (unwrapped)
%N    - Commit notes
%e    - Encoding name
%f    - Sanitized subject (filename-safe)
```

### 4.6 Author Information Placeholders

```
%an   - Author name
%aN   - Author name (respects .mailmap)
%ae   - Author email
%aE   - Author email (respects .mailmap)
%al   - Author email local-part
%aL   - Author local-part (.mailmap)
%ad   - Author date (respects --date option)
%aD   - Author date (RFC2822 format)
%ar   - Author date (relative: "2 hours ago")
%at   - Author date (Unix timestamp)
%ai   - Author date (ISO 8601-like: "2022-08-30 15:42:32 +0200")
%aI   - Author date (strict ISO 8601)
%as   - Author date (YYYY-MM-DD)
%ah   - Author date (human style)
```

### 4.7 Committer Information Placeholders

Same as author with `c` prefix instead of `a`:
```
%cn, %cN, %ce, %cE, %cl, %cL, %cd, %cD, %cr, %ct, %ci, %cI, %cs, %ch
```

### 4.8 Reference and Decoration Placeholders

```
%d    - Ref names (with decorations): (HEAD -> main, origin/main)
%D    - Ref names (without parentheses)
%S    - Ref name from command line
%gD   - Full reflog selector
%gd   - Short reflog selector
%gn   - Reflog identity name
%gN   - Reflog identity name (.mailmap)
%ge   - Reflog identity email
%gE   - Reflog identity email (.mailmap)
%gs   - Reflog subject
%(decorate[:<options>])  - Custom ref decorations
%(describe[:<options>])  - Human-readable description
%(trailers[:<options>])  - Commit trailers
```

### 4.9 GPG Signature Placeholders

```
%GG  - Raw verification message
%G?  - Signature status (G=good, B=bad, U=untrusted, X=expired, Y=expired key, R=revoked, E=error, N=no sig)
%GS  - Signer name
%GK  - Key used to sign
%GF  - Key fingerprint
%GP  - Primary key fingerprint
%GT  - Key trust level
```

### 4.10 Example Format Strings

**One-line with date and author:**
```bash
git log --pretty=format:"%h - %an, %ar : %s"
# Output: abc1234 - John Doe, 2 hours ago : Fix login bug
```

**Colorized detailed format:**
```bash
git log --pretty=format:"%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset"
# Output with colors: hash (red) - decorations (yellow) message date (green) author (blue)
```

**Multi-line with body:**
```bash
git log --pretty=format:"%H%n%an <%ae>%n%ad%n%n%B%n"
# Output:
# <full hash>
# <author name> <email>
# <date>
# <commit message with full body>
```

**Graph with custom format:**
```bash
git log --graph --pretty=format:"%h -%d %s (%cr) [%an]"
# Includes ASCII graph on left, hash, refs, message, relative date, author
```

---

## 5. Git Log Filtering and Limiting Output

**Source:** [Git - git-log Documentation](https://git-scm.com/docs/git-log)

### 5.1 Limiting Output Count

```bash
git log -n 10                           # Show last 10 commits
git log --max-count=10                  # Same as above
git log -5                              # Show last 5 commits
git log -1                              # Show only HEAD commit
git log --skip=5 -10                    # Skip 5 commits, show next 10
```

### 5.2 Date-Based Filtering

```bash
git log --since="2 weeks ago"           # After date
git log --after="2022-08-30"            # Same as --since
git log --until="1 week ago"            # Before date
git log --before="2022-09-06"           # Same as --until
git log --since="2022-08-01" --until="2022-08-31"  # Date range
```

### 5.3 Author and Committer Filtering

```bash
git log --author="John"                 # Filter by author name (regex)
git log --author="john@example.com"     # Filter by email
git log --committer="Jane"              # Filter by committer name (regex)
git log --author="John" --committer="Jane"  # Both filters (AND)
```

### 5.4 Message Content Filtering

```bash
git log --grep="bugfix"                 # Search commit message (regex)
git log --grep="feature|enhancement"    # Multiple patterns (OR)
git log --grep="feature" --all-match    # With other filters (AND)
git log --grep="WIP" --invert-grep      # Exclude commits (NOT)
git log --grep="fix" -i                 # Case-insensitive search
```

### 5.5 Revision Range Filtering

```bash
git log origin..HEAD                    # Commits in HEAD not in origin
git log main...feature                  # Commits in main OR feature (not both)
git log --all                           # All branches and tags
git log --branches                      # All local branches
git log --remotes                       # All remote branches
git log --tags                          # All tags
git log --branches --remotes --tags     # All refs
git log --branches="feature*"           # Branches matching pattern
```

### 5.6 Merge and Ancestry Filtering

```bash
git log --merges                        # Show only merge commits
git log --no-merges                     # Exclude merge commits
git log --first-parent                  # Follow first parent only
git log --ancestry-path main..develop   # Commits on path from main to develop
git log --full-history                  # Don't prune history
```

### 5.7 File and Pathspec Filtering

```bash
git log -- src/main.rs                  # Commits affecting file
git log -- src/                         # Commits affecting directory
git log -p -- src/lib.rs                # Show patches for file
git log --follow -- src/old.rs          # Track file renames
git log --name-only -- src/             # Show filenames modified
git log --name-status -- src/           # Show file status (A/M/D)
```

---

## 6. Output Control Flags and Display Options

**Source:** [Git - git-log Documentation](https://git-scm.com/docs/git-log), [Advanced Git Log](https://www.atlassian.com/git/tutorials/git-log)

### 6.1 Graph Visualization

```bash
git log --graph                         # ASCII graph of branch structure
git log --graph --oneline               # Compact graph view
git log --graph --decorate --oneline --all  # Full graph with decorations
```

**Output Example:**
```
* abc1234 (HEAD -> main) Merge pull request #123
|\
| * def5678 (origin/feature) Add new feature
| |
| * ghi9012 Update documentation
|/
* jkl3456 Fix critical bug
```

### 6.2 Diff and Patch Output

```bash
git log -p                              # Show patches (full diffs)
git log --patch                         # Same as -p
git log --stat                          # Show diffstat (file change summary)
git log --shortstat                     # Condensed diffstat
git log -u                              # Show unified diff
git log -W                               # Show function context (whole function)
git log --raw                           # Show raw diff format
git log --name-only                     # Show only filenames changed
git log --name-status                   # Show filenames and status (A/M/D)
```

### 6.3 Color and Formatting Output

```bash
git log --color                         # Enable colors (default in terminal)
git log --color=always                  # Force colors (useful for piping)
git log --color=never                   # Disable colors
git log --color=auto                    # Automatic based on output device
git log --decorate                      # Show ref names
git log --decorate=short                # Abbreviated ref names
git log --decorate=full                 # Full ref paths
git log --decorate=auto                 # Automatic (default)
```

### 6.4 Pagination and Output Control

```bash
git log --no-pager                      # Disable pager, output to stdout
git log -p                              # Automatic paging for diffs
git log --paginate                      # Force paging
git log -z                              # NUL separator instead of newlines
```

**Pager Configuration:**
```bash
# Disable paging globally
git config --global core.pager ""

# Use 'cat' as pager (no interactive scrolling)
git config core.pager "cat"

# Use custom pager
git config core.pager "less -R"

# Temporary disable
git --no-pager log
```

### 6.5 History Simplification

```bash
git log --full-history                  # Don't prune history
git log --dense                         # Show selected commits + context
git log --sparse                        # Show all commits in simplified history
git log --simplify-merges               # Remove needless merges
git log --simplify-by-decoration        # Only show tagged commits
```

### 6.6 Advanced Display Options

```bash
git log --source                        # Show ref name by which commit reached
git log --remotes --no-walk             # Show all remote commits (branches)
git log --no-walk=sorted                # Show commits in sorted order
git log --abbrev=12                     # Use 12-char hash abbreviation
git log --encoding=utf-8                # Set output encoding
git log --format=<format>               # Same as --pretty
git log --reverse                       # Reverse chronological order
```

---

## 7. Log Formatting Example Combinations

**Source:** [A better git log](https://coderwall.com/p/euwpig/a-better-git-log), [Pretty Git Branch Graphs](https://betterstack.com/community/questions/pretty-git-branch-graphs/)

### 7.1 Popular Command Combinations

**ADOG (A Dog) - All Decorated Oneline Graph:**
```bash
git log --all --decorate --oneline --graph
# Memorable acronym: "a dog"
```

**Modern Graph with Colors:**
```bash
git log --graph --pretty=format:"%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset" --abbrev-commit
```

**Compact View with Details:**
```bash
git log --oneline --graph --decorate --all
```

**Full Details with Patches:**
```bash
git log -p --graph --all --decorate
```

**File History:**
```bash
git log -p --follow -- path/to/file
git log --stat -- path/to/file
git log --oneline -- path/to/file
```

**Author Activity Summary:**
```bash
git log --shortlog -sn          # Summary by author
git log --format="%an" | sort | uniq -c | sort -rn  # Commit count per author
```

### 7.2 Alias Configuration

Create convenient aliases in `.gitconfig`:
```bash
git config --global alias.lg "log --graph --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset' --abbrev-commit"

git config --global alias.lga "log --graph --oneline --all --decorate"

git config --global alias.ls "log --oneline --graph"

# Then use:
git lg
git lga
git ls
```

---

## 8. Pagination Strategies

**Sources:** [Mastering Git No Pager](https://gitscripts.com/git-no-pager), [Using git without a pager](https://blog.toshima.ru/2021/11/12/git-without-pager.html)

### 8.1 Pager Control

**Disable Pager Temporarily:**
```bash
git --no-pager log -p           # -P is also valid
git -P log -p
```

**Enable Pager Temporarily:**
```bash
git --paginate log -n 1000      # Force pagination even for small output
```

**Global Pager Configuration:**
```bash
# Disable all pagers
git config --global core.pager ""

# Use cat instead (no interactive scrolling)
git config --global core.pager "cat"

# Use less with color support
git config --global core.pager "less -R"

# Use more (basic pager)
git config --global core.pager "more"
```

**Command-Specific Pager Configuration:**
```bash
# Pager for log command only
git config --global pager.log "less -R"

# Disable pager for diff
git config --global pager.diff ""

# Custom pager for show
git config pager.show "less -R -S"  # -S disables line wrapping
```

### 8.2 Less Pager Navigation

Common less commands for navigating paginated output:
```
Space           - Next page
b              - Previous page
d              - Half page down
u              - Half page up
g              - Go to start
G              - Go to end
/pattern       - Search forward
?pattern       - Search backward
n              - Next search match
N              - Previous search match
q              - Quit
-S             - Toggle line wrapping
```

### 8.3 Pagination for Large Outputs

**Limit Output Before Paging:**
```bash
git log -n 50 | less            # Limit to 50 commits, then page
git log --since="2 weeks ago" | less  # Filter by date first
git log --author="John" -p | less     # Filter by author with patches
```

**Buffering Strategy:**
```bash
# For 1000+ commits, consider output format:
git log --oneline -1000         # Efficient: ~1KB per commit
git log -p -1000                # Large: ~50KB per commit with patches

# Use pagination for diffs:
git log -p --no-pager -n 10     # Show 10 commit diffs without paging
git log -p -n 1000 | less -S    # Page large diff output with no wrapping
```

---

## 9. Layer-Specific Log Display

Layer-specific log display refers to showing commits filtered by logical layers (components, modules, subsystems) in a version control system. This is particularly relevant for monorepos or projects with distinct architectural layers.

### 9.1 Layer Filtering by Path

**Display logs for specific layers/modules:**
```bash
# Backend layer logs
git log --oneline -- api/ backend/ server/

# Frontend layer logs
git log --oneline -- ui/ frontend/ client/

# Database layer logs
git log --oneline -- database/ migrations/ schema/

# Documentation layer
git log --oneline -- docs/ README.md CHANGELOG.md
```

**Combined with graph:**
```bash
git log --graph --oneline -- core/
```

### 9.2 Layer Filtering by Author/Component Tag

**Using commit message patterns:**
```bash
# Infrastructure commits
git log --grep="[infra]" --oneline

# Feature commits
git log --grep="[feature]" --oneline

# Bug fixes
git log --grep="fix:" --oneline

# Documentation
git log --grep="docs:" --oneline

# Combination
git log --all-match --grep="[api]" --grep="fix" --oneline
```

### 9.3 Custom Format for Layer Display

**Layer-aware format with module prefix:**
```bash
git log --pretty=format:"%h [%an] %s" --oneline -- backend/

# Or with custom grouping:
git log --pretty=tformat:"%h - %s" -- api/ | grep -E "^\w+ - \[layer\]"
```

### 9.4 Layer-Specific Statistics

**Commits per layer:**
```bash
# Count commits per module
for dir in api/ backend/ frontend/ database/; do
    echo "$dir: $(git log --oneline -- $dir | wc -l) commits"
done

# Lines changed per layer
for dir in api/ backend/ frontend/ database/; do
    echo "$dir: $(git log --stat -- $dir | grep 'changed' | tail -1)"
done
```

### 9.5 Implementation Pattern for Jin CLI

For a CLI tool like `jin`, layer-specific display could be implemented as:

```rust
// Pseudocode for layer-specific log display
pub struct LayerLogFilter {
    paths: Vec<String>,      // Layer paths (e.g., ["api/", "backend/"])
    tags: Vec<String>,       // Component tags (e.g., ["[auth]", "[db]"])
    format: LogFormat,       // Custom format (one-line, graph, detailed)
    limit: Option<usize>,    // Pagination limit
}

impl LayerLogFilter {
    pub fn apply(&self, revwalk: &mut Revwalk) -> Result<Vec<Commit>> {
        // 1. Configure revwalk with push/hide rules
        // 2. Filter commits by path and tags
        // 3. Format output according to display preferences
        // 4. Handle pagination
    }
}
```

**Layers in jin context:**
- **Core layer**: Main command logic, utilities
- **Configuration layer**: Config files, settings
- **Workspace layer**: Workspace management
- **Store layer**: Data storage, persistence
- **UI layer**: Display, formatting

---

## 10. Code Example: Commit History Traversal with git2-rs

**Source:** [git2-rs examples/log.rs](https://github.com/rust-lang/git2-rs/blob/master/examples/log.rs), [24 Days of Rust - git2](https://zsiciarz.github.io/24daysofrust/book/vol2/day16.html)

### 10.1 Basic Commit Walking

```rust
use git2::{ObjectType, Repository, Sort};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let repo = Repository::open(".")?;
    let mut revwalk = repo.revwalk()?;

    // Start from HEAD
    revwalk.push_head()?;

    // Set sorting (topological order)
    revwalk.set_sorting(Sort::TOPOLOGICAL)?;

    // Iterate through commits
    for oid_result in revwalk {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

        println!("{} - {}", oid, commit.summary().unwrap_or(""));
    }

    Ok(())
}
```

### 10.2 Detailed Commit Output

```rust
use git2::{Repository, Signature, Sort, Time};
use std::str;

fn print_commit(commit: &git2::Commit) {
    // Get commit metadata
    let author = commit.author();
    let message = commit.message().unwrap_or("No message");
    let time = commit.time();

    // Format timestamp
    let date_str = format_time(&time);

    println!("commit {}", commit.id());
    println!("Author: {} <{}>",
             author.name().unwrap_or("Unknown"),
             author.email().unwrap_or(""));
    println!("Date:   {}\n", date_str);
    println!("    {}", message.lines().next().unwrap_or(""));
}

fn format_time(time: &Time) -> String {
    // Convert Unix timestamp to readable format
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = std::time::Duration::new(time.secs as u64, 0);
    let system_time = UNIX_EPOCH + duration;

    format!("{:?}", system_time)
}
```

### 10.3 Filtering Commits

```rust
use git2::{Repository, Sort};

fn show_commits_by_author(
    repo: &Repository,
    author: &str,
    limit: usize,
) -> Result<(), git2::Error> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(Sort::TIME)?;

    let mut count = 0;

    for oid_result in revwalk {
        if count >= limit {
            break;
        }

        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

        // Filter by author
        if let Some(commit_author) = commit.author().name() {
            if commit_author.contains(author) {
                println!("{} - {}",
                         &oid.to_string()[..7],
                         commit.summary().unwrap_or(""));
                count += 1;
            }
        }
    }

    Ok(())
}
```

### 10.4 Graph Visualization

```rust
use git2::{Repository, Sort};
use std::collections::HashMap;

fn show_commit_graph(repo: &Repository) -> Result<(), git2::Error> {
    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;
    revwalk.set_sorting(Sort::TOPOLOGICAL)?;

    // Track depth for indentation
    let mut depths: HashMap<git2::Oid, usize> = HashMap::new();

    for oid_result in revwalk {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;

        // Calculate depth
        let depth = if commit.parent_count() == 0 {
            0
        } else {
            let parent_id = commit.parent_id(0)?;
            depths.get(&parent_id).copied().unwrap_or(0) + 1
        };

        depths.insert(oid, depth);

        // Print with indentation
        let indent = "  ".repeat(depth);
        println!("{}* {}",
                 indent,
                 commit.summary().unwrap_or(""));
    }

    Ok(())
}
```

### 10.5 Working with Ranges and Filtering

```rust
use git2::{Repository, Sort};

fn show_commits_in_range(
    repo: &Repository,
    base: &str,
    tip: &str,
) -> Result<(), git2::Error> {
    let mut revwalk = repo.revwalk()?;

    // Parse revisions
    let base_obj = repo.revparse_single(base)?;
    let tip_obj = repo.revparse_single(tip)?;

    // Include tip, exclude base
    revwalk.push(tip_obj.id())?;
    revwalk.hide(base_obj.id())?;
    revwalk.set_sorting(Sort::TIME)?;

    // Walk commits
    for oid_result in revwalk {
        let oid = oid_result?;
        let commit = repo.find_commit(oid)?;
        println!("{} {}",
                 &oid.to_string()[..7],
                 commit.summary().unwrap_or(""));
    }

    Ok(())
}
```

---

## 11. Key Takeaways and Implementation Guidelines

### 11.1 For Jin Log Implementation

1. **Use RevWalk for Efficient Traversal**
   - Initialize with `repo.revwalk()?`
   - Configure sorting: `set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)`
   - Push starting points: `push_head()`, `push_ref()`, `push_glob()`
   - Hide unwanted commits: `hide()`, `hide_glob()`

2. **Extract Metadata Efficiently**
   ```rust
   let commit = repo.find_commit(oid)?;
   let author = commit.author();
   let time = commit.time();
   let message = commit.message().unwrap_or("");
   ```

3. **Implement Custom Formatting**
   - Use format placeholders from git pretty-formats
   - Support `--oneline`, `--graph`, custom `--format`
   - Color support with ANSI codes

4. **Support Filtering**
   - Author filtering: `--author="pattern"`
   - Date filtering: `--since`, `--until`
   - Path filtering: `-- path/to/file`
   - Message filtering: `--grep="pattern"`

5. **Implement Pagination**
   - Limit output: `-n`, `--max-count`
   - Skip commits: `--skip`
   - Pager integration with `--no-pager`

6. **Layer-Specific Features**
   - Filter by workspace/module paths
   - Support component tags in messages
   - Custom layer display formats

### 11.2 Performance Considerations

- **Generation Numbers**: Use depth-first search for large repositories
- **Pathspec Filtering**: Apply path filters during revwalk iteration
- **Memory**: Stream commits instead of loading all into memory
- **Sorting**: Use `TOPOLOGICAL` for DAG traversal, `TIME` for recency

---

## Documentation URLs Reference

### Core Documentation
- [Revwalk in git2 - Rust](https://docs.rs/git2/latest/git2/struct.Revwalk.html)
- [Commit in git2 - Rust](https://docs.rs/git2/latest/git2/struct.Commit.html)
- [Repository in git2 - Rust](https://docs.rs/git2/latest/git2/struct.Repository.html)
- [git2 - Rust](https://docs.rs/git2)

### Git Command Reference
- [Git - git-log Documentation](https://git-scm.com/docs/git-log)
- [Git - pretty-formats Documentation](https://git-scm.com/docs/pretty-formats)

### Examples and Tutorials
- [git2-rs examples/log.rs](https://github.com/rust-lang/git2-rs/blob/master/examples/log.rs)
- [24 Days of Rust - git2](https://zsiciarz.github.io/24daysofrust/book/vol2/day16.html)
- [Advanced Git Log - Atlassian](https://www.atlassian.com/git/tutorials/git-log)

### Algorithms and Performance
- [Git's Database Internals II: Commit History Queries](https://github.blog/2022-08-30-gits-database-internals-ii-commit-history-queries/)
- [Supercharging the Git Commit Graph III: Generations and Graph Algorithms](https://devblogs.microsoft.com/devops/supercharging-the-git-commit-graph-iii-generations/)

### Practical Guides
- [A better git log](https://coderwall.com/p/euwpig/a-better-git-log)
- [Pretty Git Branch Graphs](https://betterstack.com/community/questions/pretty-git-branch-graphs/)
- [Mastering Git No Pager](https://gitscripts.com/git-no-pager)
- [Using git without a pager](https://blog.toshima.ru/2021/11/12/git-without-pager.html)

---

## Research Completed

This comprehensive research document covers all major aspects of git log implementation patterns, from low-level traversal algorithms to high-level formatting and display options. The information is drawn from official git2-rs documentation, git command documentation, and practical implementation guides.

**Document Version:** 1.0
**Date Created:** December 27, 2025
**Research Focus Areas Covered:** 5/5 (all requested areas included)
