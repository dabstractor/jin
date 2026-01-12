# Git Diff Implementation Patterns Research

**Date**: 2025-12-27
**Research Focus**: Git diff algorithms, Rust implementations, layer diff patterns, structured diff formats, CLI best practices

---

## Table of Contents

1. [Git Diff Algorithms and Implementations](#git-diff-algorithms-and-implementations)
2. [Rust Implementations of Diff Functionality](#rust-implementations-of-diff-functionality)
3. [Layer Diff Patterns](#layer-diff-patterns)
4. [Structured Diff for JSON/YAML/TOML](#structured-diff-for-jsonyamltoml)
5. [Diff Output Formatting](#diff-output-formatting)
6. [Tree Comparison Patterns](#tree-comparison-patterns)
7. [Error Handling Best Practices](#error-handling-best-practices)

---

## Git Diff Algorithms and Implementations

### Overview

Git uses the `xdiff` module for text diffing, which implements multiple algorithms. The implementation was switched from using the system `diff` executable to a built-in approach in March 2006. The current code uses a simplified version of libxdiff with Eugene W. Myers's algorithm as the default.

**Source**: [Git Source Code Review: Diff Algorithms](https://www.fabiensanglard.net/git_code_review/diff.php)

### Supported Diff Algorithms

Git supports four primary diff algorithms, configurable via the `--diff-algorithm` option:

#### 1. Myers Algorithm (Default)

**Description**: The basic greedy diff algorithm based on finding the shortest edit script (SES).

**How It Works**:
- Models the editing process on a grid where rightward = deletion, downward = insertion, diagonal = match
- Employs greedy, breadth-first search to explore all paths simultaneously at each depth level
- Prefers deletions before insertions and maximizes diagonal (matching) segments
- Time complexity: O(N*M) where N and M are file sizes
- Space complexity in linear-space variant: O(N + M)

**Best For**: General purpose diffing with good quality and performance tradeoff

**Implementation Pattern**:
```
algorithm: greedy-lcs
approach: breadth-first graph search
preference: deletion > insertion > insertion
diagonal: free (matching) moves
```

**Sources**:
- [The Myers Diff Algorithm: Part 1](https://blog.jcoglan.com/2017/02/12/the-myers-diff-algorithm-part-1/)
- [Myers Diff in Linear Space: Theory](https://blog.jcoglan.com/2017/03/22/myers-diff-in-linear-space-theory/)
- [How Git-diff Algorithm Works](https://medium.com/@gabrielschade/how-git-diff-works-a-sample-with-f-af3e3737963)

#### 2. Patience Algorithm

**Description**: Produces more intuitive output than Myers by preserving unique lines.

**Key Difference**: Instead of minimizing the number of +/- lines first, it tries to preserve lines that are unique in the sequences.

**Best For**: Code with many unique identifiers and function signatures where preserving context is important

**Source**: [When to Use Each of the Git Diff Algorithms](https://luppeng.wordpress.com/2020/10/10/when-to-use-each-of-the-git-diff-algorithms/)

#### 3. Histogram Algorithm

**Description**: Extends the patience algorithm to support "low-occurrence common elements".

**Advantages**:
- Faster than patience algorithm
- Developed by the JGit project
- Better results than Myers for certain code patterns
- Git 1.7.7+

**Research Finding**: In 14 Java projects, histogram diff showed different results than Myers in 1.7-8.2% of commits, and for bug-introducing change identification, 6-13.3% of bug-fix commits had different results, with histogram being more suitable than Myers.

**Best For**: Large repositories where performance and code readability matter

**Source**: [How Different Are Different Diff Algorithms in Git?](https://arxiv.org/abs/1902.02467)

#### 4. Minimal Algorithm

**Description**: Extended Myers variant that iterates to ensure the smallest possible diff.

**Trade-off**: Spends extra computation time for guaranteed minimal diff output

**Best For**: Cases where the absolute smallest diff is required, even at cost of performance

### Binary Diff (Deltas)

For binary files, Git uses a different approach:

**Algorithm**: Rabin fingerprinting with sliding window

**Advantage**: Detects short additions at the beginning of binary files (e.g., compressed data) while keeping deltas tiny

**Pattern**:
```
rabin_fingerprinting:
  - sliding_window: tracks content chunks
  - advantage: compact representation
  - use_case: binary files, compressed data
```

### Command-Line Interface

Git diff supports several target comparison modes:

```bash
# Working tree vs index (unstaged changes)
git diff [options] [--] [<path>...]

# Staged changes vs HEAD
git diff [options] --cached [<commit>] [--] [<path>...]

# Working tree vs specific commit
git diff [options] [--merge-base] <commit> [--] [<path>...]

# Two commits
git diff [options] <commit>..<commit> [--] [<path>...]
git diff [options] <commit>...<commit> [--] [<path>...]
```

**Source**: [Git - git-diff Documentation](https://git-scm.com/docs/git-diff)

#### Key Options Reference

**Output Format**:
- `-p`, `--patch`: Generate patch text (default)
- `-s`, `--no-patch`: Suppress diff output
- `--stat`: Show diffstat with file change summary
- `--numstat`: Machine-readable format with line counts
- `--name-only`: Only changed file names
- `--name-status`: File names and status (A/M/D/R/C)

**Context and Whitespace**:
- `-U<n>`, `--unified=<n>`: Lines of context (default: 3)
- `-w`, `--ignore-all-space`: Ignore all whitespace
- `-b`, `--ignore-space-change`: Ignore whitespace amount changes
- `--ignore-space-at-eol`: Ignore trailing whitespace

**Rename and Copy Detection**:
- `-M[<n>]`, `--find-renames[=<n>]`: Detect renames (threshold default: 50%)
- `-C[<n>]`, `--find-copies[=<n>]`: Detect copies
- `--find-copies-harder`: Inspect unmodified files for copies
- `-B[<n>]`, `--break-rewrites`: Break complete rewrites into delete/create

**Filtering and Search**:
- `--diff-filter=[ACDMRTUXB]`: Filter by status
- `-S<string>`: Find changes adding/removing string
- `-G<regex>`: Find changes where patch matches regex
- `--check`: Warn about whitespace errors

**Algorithm Selection**:
- `--diff-algorithm={myers|minimal|patience|histogram}`: Choose algorithm

### git diff-tree (Plumbing)

**Purpose**: Low-level tree comparison tool that directly compares blob content and mode of two tree objects.

**Output Format** (raw by default):
```
:100644 100644 bcd1234 0123456 M file0
```

**Components**:
- Source mode (6 octal digits)
- Destination mode (6 octal digits)
- Source SHA1 hash
- Destination SHA1 hash
- Status letter (A/C/D/M/R/T/U/X)
- Optional similarity/dissimilarity score
- Pathname(s)

**Status Letters**:
- A: Addition
- C: Copy
- D: Deletion
- M: Modification
- R: Rename
- T: Type change
- U: Unmerged (merge conflict)
- X: Unknown

**Source**: [Git - git-diff-tree Documentation](https://git-scm.com/docs/git-diff-tree)

---

## Rust Implementations of Diff Functionality

### git2 Library (libgit2 Bindings)

The git2 crate provides Rust bindings to libgit2, a pure C implementation of Git core functionality.

**Cargo.toml**:
```toml
[dependencies]
git2 = "0.48"  # Latest as of research date
```

**Official Documentation**: [docs.rs/git2](https://docs.rs/git2/latest/git2/struct.Diff.html)

#### Core Diff Types

**Diff struct**: Contains all individual file deltas. Opaque structure allocated by diff generator functions.

```rust
pub struct Diff<'repo> { /* opaque */ }
```

**Characteristics**:
- Implements `Send` (thread-safe for transfer)
- Does NOT implement `Sync` (not concurrent-safe)
- Automatically manages memory via `Drop` implementation

**Source**: [git2::Diff - Rust Docs](https://docs.rs/git2/latest/git2/struct.Diff.html)

#### Diff Generation Methods

All diff creation functions on Repository:

##### 1. Tree to Tree
```rust
pub fn diff_tree_to_tree(
    &self,
    old_tree: Option<&Tree>,
    new_tree: Option<&Tree>,
    opts: Option<&mut DiffOptions>
) -> Result<Diff, Error>
```
**Equivalent to**: `git diff <old-tree> <new-tree>`

##### 2. Tree to Index
```rust
pub fn diff_tree_to_index(
    &self,
    tree: Option<&Tree>,
    index: Option<&Index>,
    opts: Option<&mut DiffOptions>
) -> Result<Diff, Error>
```
**Equivalent to**: `git diff --cached <treeish>`

##### 3. Index to Working Directory
```rust
pub fn diff_index_to_workdir(
    &self,
    index: Option<&Index>,
    opts: Option<&mut DiffOptions>
) -> Result<Diff, Error>
```
**Equivalent to**: `git diff` (default)

##### 4. Tree to Working Directory
```rust
pub fn diff_tree_to_workdir(
    &self,
    tree: Option<&Tree>,
    opts: Option<&mut DiffOptions>
) -> Result<Diff, Error>
```
**Note**: Ignores index entirely, may not match expected git behavior for staged deletions

##### 5. Tree to Working Directory (with Index)
```rust
pub fn diff_tree_to_workdir_with_index(
    &self,
    tree: Option<&Tree>,
    opts: Option<&mut DiffOptions>
) -> Result<Diff, Error>
```
**Recommended**: Uses index to account for staged changes, emulates standard git diff behavior

**Critical Pattern**: Use `diff_tree_to_workdir_with_index` when you need standard git behavior; `diff_tree_to_workdir` ignores staged changes.

**Source**: [libgit2 Diff API Reference](https://libgit2.org/docs/reference/main/diff/index.html)

#### DiffOptions Structure

Controls diff execution behavior:

```rust
pub struct DiffOptions { /* opaque */ }
```

**Methods**:
```rust
pub fn new() -> DiffOptions
pub fn include_untracked(&mut self, include: bool)
pub fn include_unmodified(&mut self, include: bool)
pub fn include_ignored(&mut self, include: bool)
pub fn include_typechange(&mut self, include: bool)
pub fn include_typechange_trees(&mut self, include: bool)
pub fn ignore_filemode(&mut self, ignore: bool)
pub fn ignore_submodules(&mut self, ignore: bool)
pub fn ignore_case(&mut self, ignore: bool)
pub fn include_casechange(&mut self, include: bool)
pub fn show_untracked_content(&mut self, show: bool)
pub fn show_binary(&mut self, show: bool)
pub fn context_lines(&mut self, lines: u32)
pub fn interhunk_lines(&mut self, lines: u32)
pub fn oid_abbrev(&mut self, abbrev: u16)
pub fn max_size(&mut self, size: i64)
pub fn old_prefix(&mut self, prefix: &str) -> Result<(), Error>
pub fn new_prefix(&mut self, prefix: &str) -> Result<(), Error>
pub fn pathspec(&mut self, pathspec: &str) -> Result<(), Error>
pub fn find_renames(&mut self, threshold: Option<u16>)
pub fn find_copies(&mut self, threshold: Option<u16>)
pub fn find_copies_from_unmodified(&mut self)
pub fn find_rewrites(&mut self, threshold: Option<u16>)
pub fn break_rewrites(&mut self, threshold: Option<u16>)
pub fn fail_on_conflict(&mut self, fail: bool)
pub fn skip_binary_check(&mut self, skip: bool)
pub fn line_prefix(&mut self, prefix: &str) -> Result<(), Error>
```

**Source**: [DiffOptions in git2 - Rust](https://docs.rs/git2/latest/git2/struct.DiffOptions.html)

#### Core Methods

**Iteration**:
```rust
pub fn foreach<F>(&self, cb: F, rlc: F, rhc: F, lc: F) -> Result<(), Error>
where
    F: FnMut(DiffDelta, f32) -> bool,
    rlc: FnMut(DiffDelta, DiffHunk) -> bool,
    rhc: FnMut(DiffDelta, DiffHunk) -> bool,
    lc: FnMut(DiffDelta, DiffHunk, DiffLine) -> bool
```

**Analysis**:
```rust
pub fn deltas(&self) -> DeltasIterator
pub fn get_delta(&self, delta_idx: usize) -> Option<DiffDelta>
pub fn stats(&self) -> Result<DiffStats, Error>
pub fn find_similar(&mut self, opts: Option<&DiffFindOptions>) -> Result<(), Error>
pub fn merge(&mut self, other: &Diff) -> Result<(), Error>
pub fn patchid(&self) -> Result<Oid, Error>
```

**Output**:
```rust
pub fn print<F>(&self, format: DiffFormat, mut print_cb: F) -> Result<(), Error>
where
    F: FnMut(DiffDelta, DiffHunk, DiffLine) -> bool
pub fn format_patch<F>(&self, delta_idx: usize, hunk_idx: usize, line_idx: usize, mut line_cb: F) -> Result<(), Error>
where
    F: FnMut(&[u8]) -> bool
```

**Source**: [git2::Diff - Rust](https://docs.rs/git2/latest/git2/struct.Diff.html)

#### Example Implementation

**Complete Example**: [git2-rs/examples/diff.rs](https://github.com/rust-lang/git2-rs/blob/master/examples/diff.rs)

**Key Patterns from Example**:

```rust
// Argument parsing structure
struct Args {
    arg_object: Vec<String>,
    arg_files: Vec<String>,
    flag_cached: bool,
    flag_stat: bool,
    flag_patch: bool,
    flag_color: Option<String>,
    flag_no_index: bool,
    // ... more options
}

// Helper: resolve string to blob
fn resolve_blob(repo: &Repository, spec: &str) -> Result<Option<Blob>, Error> {
    let obj = repo.revparse_single(spec)?;
    let tree = obj.peel(ObjectType::Blob)?;
    Ok(tree.into_blob().ok())
}

// Helper: convert to tree object
fn tree_to_treeish(repo: &Repository, arg: &str) -> Result<Object, Error> {
    let obj = repo.revparse_single(arg)?;
    // Optionally peel to tree
    Ok(obj)
}

// Color output helper
fn line_color(line: git2::DiffLineType) -> &'static str {
    match line {
        git2::DiffLineType::Addition => "\x1b[32m",    // Green
        git2::DiffLineType::Deletion => "\x1b[31m",    // Red
        git2::DiffLineType::FileHeader => "\x1b[1m",   // Bold
        git2::DiffLineType::HunkHeader => "\x1b[36m",  // Cyan
        _ => "",
    }
}

// Main diff generation
let diff = match (old_tree, new_tree) {
    (Some(old), Some(new)) => repo.diff_tree_to_tree(Some(&old), Some(&new), Some(opts))?,
    (Some(old), None) => repo.diff_tree_to_workdir_with_index(Some(&old), Some(opts))?,
    (None, Some(new)) => repo.diff_tree_to_tree(None, Some(&new), Some(opts))?,
    (None, None) => repo.diff_index_to_workdir(None, Some(opts))?,
};

// Rename detection
if find_renames {
    diff.find_similar(None)?;
}

// Iterate through deltas
for delta in diff.deltas() {
    println!("{}", delta.status());
    // Process delta...
}
```

#### Error Handling Pattern in git2

The library uses `Result<T, Error>` for error propagation:

```rust
pub fn diff_tree_to_tree(
    &self,
    old_tree: Option<&Tree>,
    new_tree: Option<&Tree>,
    opts: Option<&mut DiffOptions>
) -> Result<Diff, Error>
```

**Common Error Sources**:
- Invalid tree objects
- Invalid index
- Working directory access issues
- Invalid diff options

**Callback-based Error Handling**:
When using diff callbacks (foreach, print), returning `false` from the callback terminates iteration and returns an error from the function.

```rust
diff.foreach(
    |delta, _| {
        // Return false to signal error and stop iteration
        if should_stop_processing {
            return false;
        }
        true
    },
    |_delta, _hunk| true,
    |_delta, _hunk, _line| true,
)?;
```

### Alternative Rust Crates for Diff

#### 1. similar - High-Level Diffing Library

**Crate**: [mitsuhiko/similar](https://github.com/mitsuhiko/similar)

**Algorithms**:
- Myers' diff
- Patience diff
- Hunt-McIlroy / Hunt-Szymanski LCS diff

**Granularity Levels**:
- Line-level
- Character and grapheme-level
- Word-level

**Input/Output**:
- Text and byte diffing
- Arbitrary comparable sequences
- Unified diff output format

**Dependencies**: None
**MSRV**: Rust 1.60+
**License**: Apache-2.0

**Use Case**: When you need pure Rust diffing without git2 dependency, or for snapshot testing (created for insta framework)

**Source**: [similar - Rust Documentation](https://github.com/mitsuhiko/similar)

#### 2. Structured Data Diff Crates

**JSON Diff Crates**:

1. **json-structural-diff**: Structural JSON diff library
   - Crate: `json-structural-diff`
   - Focus: JSON-specific diff operations

2. **sjdiff**: Structural JSON Diff Library
   - GitHub: [amanbolat/sjdiff](https://github.com/amanbolat/sjdiff)
   - Docs: [docs.rs/sjdiff](https://docs.rs/sjdiff)
   - Features: Compares two JSON values, produces structural differences

3. **serde_json_diff**: Machine-readable JSON diffs
   - Modules for comparing serde_json::Map and Vec values
   - Crate: [serde_json_diff](https://crates.io/crates/serde_json_diff/)

4. **assert-json-diff**: JSON comparison for testing
   - Focus: Good output formatting
   - Crate: [assert-json-diff](https://crates.io/crates/assert-json-diff)

5. **json_diff_ng**: Deep-sorting and key exclusion
   - CLI included
   - Crate: [json_diff_ng](https://crates.io/crates/json_diff_ng)

**Multi-Format Diff Crates**:

1. **diff-struct**: General data structure diffing
   - Trait-based approach for custom diffing
   - Derive macro support
   - Crate: [diff-struct](https://crates.io/crates/diff-struct)
   - Good for YAML/TOML via serialization

**Source**: [Crates.io: JSON Diff Packages](https://crates.io/keywords/json-diff)

---

## Layer Diff Patterns

### Configuration Layer Comparison

Configuration systems often use layered architectures where later layers override earlier ones. Diffing layers requires understanding this hierarchy.

#### Cisco Contextual Configuration Diff Approach

**Pattern**: Hierarchical structure-aware comparison

**Components**:
1. **Hierarchical Analysis**: Understands configuration file structure (e.g., interface blocks, routing configuration)
2. **Alignment Logic**: Reorders and aligns files to face matching parts
3. **Order Sensitivity**: Tracks location changes for order-sensitive configuration lines

**Output Format**:
- `-` : Line exists in file1 but not in file2
- `+` : Line exists in file2 but not in file1
- `!` : Order-sensitive line with different position (with descriptive comment)

**Advantage**: Produces more meaningful diffs than line-based comparison for hierarchical config files

**Source**: [Cisco Configuration Diff Documentation](https://www.cisco.com/c/en/us/td/docs/ios-xml/ios/config-mgmt/configuration/15-sy/config-mgmt-15-sy-book/cm-config-diff.html)

#### Property-Based vs Line-Based Comparison

**Traditional Line-Based**: Compares only text lines
- Advantage: Simple, works everywhere
- Disadvantage: May not recognize semantic equivalence

**Property-Based**: Parses config and compares actual property values
- Advantage: Understands configuration semantics
- Disadvantage: Requires format-specific parsing

**Example Tool**: [nerdynick/config-diff-tool](https://github.com/nerdynick/config-diff-tool)

**Pattern for Implementation**:
```
layer-diff-strategy:
  1. parse_file_to_structure()
  2. resolve_inheritance()
  3. compare_property_values()
  4. track_order_changes()
  5. generate_semantic_diff()
```

#### Layered Architecture Pattern

sops-diff uses a layered architecture where:
- Each layer has distinct responsibilities
- Later layers override earlier layers
- Comparison must track override chain

**Implementation Pattern**:
```rust
struct Layer {
    name: String,
    properties: HashMap<String, Value>,
    overrides: Vec<String>,
}

fn compare_layers(base: &Layer, override_layer: &Layer) -> LayerDiff {
    // Only show overridden properties
}
```

**Source**: [sops-diff Architecture](https://deepwiki.com/saltydogtechnology/sops-diff/9-architecture-and-internals)

### Recommended Layer Diff Pattern for jin

**Multi-layer comparison approach**:

1. **Load all configuration layers** from workspace
2. **Resolve inheritance chain** (which layer overrides which)
3. **Compare at layer boundaries**:
   - What changed between layer N and layer N+1?
   - What is the cumulative effect?
4. **Generate layer-aware diff** showing:
   - Which layer introduced each change
   - Override chain visualization
   - Effective configuration at each stage

---

## Structured Diff for JSON/YAML/TOML

### Comparison Strategies

#### 1. String-Based Diff (Simplest)

```rust
use similar::TextDiff;

let old_json = serde_json::to_string_pretty(&old_value)?;
let new_json = serde_json::to_string_pretty(&new_value)?;
let diff = TextDiff::from_lines(&old_json, &new_json);
```

**Advantage**: Works with any format
**Disadvantage**: Produces line-level diffs, not semantic diffs; format-specific

#### 2. Structural JSON Diff (Recommended)

```rust
use sjdiff::JsonDiff;

let diff = JsonDiff::from_values(&old_value, &new_value)?;

// Results show:
// - Added keys
// - Removed keys
// - Modified values
// - Structural changes
```

**Advantage**:
- Semantic understanding of JSON structure
- Format-aware comparison
- Can show path changes (e.g., "user.name" changed)

**Disadvantage**:
- JSON-specific
- Need separate implementations for YAML/TOML

**Source**: [sjdiff Documentation](https://docs.rs/sjdiff)

#### 3. Format-Agnostic Diff via Serialization

```rust
use serde_json::Value;
use diff_struct::DiffableValue;

impl<T: Serialize + Deserialize> Diffable for T {
    type Delta = ValueDiff;

    fn diff(&self, other: &Self) -> ValueDiff {
        let v1: Value = serde_json::to_value(self)?;
        let v2: Value = serde_json::to_value(other)?;
        structural_diff(&v1, &v2)
    }
}
```

**Pattern**: Normalize to common format (JSON), perform semantic diff, format output for original type

### Output Format Patterns

#### JSON Structural Diff Output

```json
{
  "added": {
    "new_field": "value"
  },
  "removed": {
    "old_field": "old_value"
  },
  "modified": {
    "existing_field": {
      "from": "old_value",
      "to": "new_value"
    }
  }
}
```

#### Unified Diff for Config Files

```diff
--- config.yaml.old
+++ config.yaml.new
@@ -5,7 +5,7 @@
 server:
   host: localhost
-  port: 3000
+  port: 8080
   ssl:
     enabled: true
-    cert: /etc/ssl/old.crt
+    cert: /etc/ssl/new.crt
```

### Implementation Pattern

```rust
fn diff_config<T: Serialize + Deserialize>(
    old: &T,
    new: &T,
    format: ConfigFormat,
) -> Result<ConfigDiff, Error> {
    let old_value = serde_json::to_value(old)?;
    let new_value = serde_json::to_value(new)?;

    let diff = compute_structural_diff(&old_value, &new_value);

    match format {
        ConfigFormat::Json => {
            // Return as JSON diff structure
            Ok(ConfigDiff::Structured(diff))
        }
        ConfigFormat::Yaml | ConfigFormat::Toml => {
            // Convert to text diff with formatting
            let old_text = format_to_string(&old_value, format)?;
            let new_text = format_to_string(&new_value, format)?;
            let text_diff = TextDiff::from_lines(&old_text, &new_text);
            Ok(ConfigDiff::Text(text_diff))
        }
    }
}
```

---

## Diff Output Formatting

### Unified Diff Format (RFC 3881)

**Standard Format**:
```
--- a/file1
+++ b/file1
@@ -98,20 +98,12 @@
 context line
-removed line
+added line
 context line
```

**Components**:
- `---` : Original file marker
- `+++` : Modified file marker
- `@@` : Hunk header with line ranges `@@ -old_start,count +new_start,count @@`
- ` ` (space): Context line
- `-` : Removed line
- `+` : Added line

**Context Lines**: Default 3 lines before/after change. Configurable via `-U<n>` option

**Source**: [Git Diff Documentation](https://git-scm.com/docs/git-diff)

### Context Output Formats

#### Unified Diff (Standard)
```
@@ -10,7 +10,7 @@
 context
-old
+new
 context
```

#### Context Diff (RCS-style)
```
*** 10,16 ****
 context
-old
 context
--- 10,16 ----
 context
+new
 context
```

#### Side-by-Side Diff

**Tools**:
- `diff -y` / `diff --side-by-side`: Built-in option
- `sdiff`: Dedicated postprocessor
- `icdiff`: Enhanced with color and highlighting
- `vimdiff`: Visual diff in Vim
- `meld`: Graphical tool

**Output Example**:
```
Line 1          |  Line 1
-old line       |  +new line
Line 3          |  Line 3
```

**Options for diff command**:
- `-y`, `--side-by-side`: Enable side-by-side
- `-W<num>`, `--width=<num>`: Set column width (default: 130)
- `-l`, `--left-column`: Print only left column for common lines

**Source**: [diff(1) - Linux Manual](https://man7.org/linux/man-pages/man1/diff.1.html)

### Color Output Formatting

**Standard ANSI Color Codes**:

```rust
// Color mapping for diff lines
const COLOR_REMOVAL: &str = "\x1b[31m";    // Red
const COLOR_ADDITION: &str = "\x1b[32m";   // Green
const COLOR_CONTEXT: &str = "\x1b[0m";     // Default
const COLOR_FILE_HEADER: &str = "\x1b[1m"; // Bold
const COLOR_HUNK_HEADER: &str = "\x1b[36m"; // Cyan
const COLOR_RESET: &str = "\x1b[0m";

fn format_colored_diff(line: &DiffLine) -> String {
    match line.origin() {
        '+' => format!("{}{}\x1b[0m", COLOR_ADDITION, line.content()),
        '-' => format!("{}{}\x1b[0m", COLOR_REMOVAL, line.content()),
        'F' | 'H' => format!("{}{}\x1b[0m", COLOR_FILE_HEADER, line.content()),
        '@' => format!("{}{}\x1b[0m", COLOR_HUNK_HEADER, line.content()),
        _ => line.content().to_string(),
    }
}
```

**Git Options**:
- `--color[=<when>]`: When to use color (always/never/auto)
- `--color-words[=<regex>]`: Highlight word-level changes
- `--color-moved[=<mode>]`: Color moved code blocks (dimmed/plain/highlight)

**CLI Best Practice**: Auto-detect when output is terminal, default to color

### Word-Level and Character-Level Diff

**Unified Diff Variant**: `--color-words`

```diff
- old word removed
+ new word added
```

Instead of:
```diff
- old word removed
+ new word added
```

**Implementation Pattern**:
```rust
fn word_diff(old_line: &str, new_line: &str) -> Vec<DiffSegment> {
    let old_words = tokenize_into_words(old_line);
    let new_words = tokenize_into_words(new_line);

    // Apply Myers algorithm to words instead of characters
    let diff = TextDiff::from_words(&old_words, &new_words);

    diff.iter_all_changes()
        .collect()
}
```

**Performance Consideration**: Word-diff is slower but more readable for large changes

---

## Tree Comparison Patterns

### Git Object Models

**Git Objects**:
- **Blob**: File content (immutable)
- **Tree**: Directory listing with file mode, name, blob/tree references
- **Commit**: Points to tree, parent commits, metadata
- **Tag**: Reference to other objects

### Comparison Scenarios

#### 1. Tree to Tree Comparison

**Pattern**: Direct structural comparison

```rust
let diff = repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), opts)?;

for delta in diff.deltas() {
    println!("{:?}: {}", delta.status(), delta.old_file().path());
}
```

**Use Cases**:
- Compare any two commits: `git diff <commit1> <commit2>`
- Compare branches: `git diff main..feature`
- Compare trees in general

**Output**: Files added, deleted, modified, renamed, copied

#### 2. Tree to Index (Staged Changes)

**Pattern**: Pre-commit staging area comparison

```rust
let index = repo.index()?;
let head = repo.head()?.peel_to_tree()?;

let diff = repo.diff_tree_to_index(Some(&head), Some(&index), opts)?;
```

**Equivalent to**: `git diff --cached` or `git diff --staged`

**Use Case**: Verify what will be committed before git commit

#### 3. Index to Working Directory

**Pattern**: Unstaged changes comparison

```rust
let index = repo.index()?;

let diff = repo.diff_index_to_workdir(Some(&index), opts)?;
```

**Equivalent to**: `git diff` (default)

**Use Case**: See what changed in working directory but not yet staged

#### 4. Tree to Working Directory

**Pattern**: Complete change set including untracked files

**Important**: Choose based on whether you want to consider staged changes

**Option A - With Index (Recommended)**:
```rust
let head = repo.head()?.peel_to_tree()?;

let diff = repo.diff_tree_to_workdir_with_index(Some(&head), opts)?;
```

**Behavior**: Considers index state, matches `git diff HEAD` behavior
- Shows as deleted: files with staged delete
- Shows as modified: even if file restored to working directory

**Option B - Without Index**:
```rust
let head = repo.head()?.peel_to_tree()?;

let diff = repo.diff_tree_to_workdir(Some(&head), opts)?;
```

**Behavior**: Ignores index entirely, shows actual working directory state
- May show file as modified even if deletion is staged
- Better for "what's literally different" analysis

**Critical Pattern**: Use `diff_tree_to_workdir_with_index` unless you specifically need to ignore staged changes.

**Source**: [libgit2 Diff Tree to Workdir](https://libgit2.org/docs/reference/v1.0.0/diff/git_diff_tree_to_workdir.html)

### Rename and Copy Detection

**Pattern**: Post-processing with find_similar

```rust
let mut diff = repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), opts)?;

// Enable rename detection
diff.find_similar(None)?;

// Now deltas include rename info
for delta in diff.deltas() {
    if delta.status() == git2::Delta::Renamed {
        let similarity = delta.similarity(); // 0-100%
        println!("Renamed: {} -> {} ({:?}% similar)",
            delta.old_file().path().display(),
            delta.new_file().path().display(),
            similarity);
    }
}
```

**Options**:
```rust
let mut find_opts = DiffFindOptions::new();
find_opts.rename_threshold(Some(50)); // Default: 50%
find_opts.copy_threshold(Some(50));
find_opts.rename_from_rewrite_threshold(Some(50));

diff.find_similar(Some(&find_opts))?;
```

**Algorithm**: Content-based similarity (blob hash comparison)

---

## Error Handling Best Practices

### Rust Error Handling Approaches

#### 1. Custom Error Enum (thiserror - for Libraries)

**Best For**: Library code where callers need to distinguish error types

**Pattern**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiffError {
    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    #[error("tree not found: {0}")]
    TreeNotFound(String),

    #[error("invalid options: {0}")]
    InvalidOptions(String),

    #[error("diff computation failed: {0}")]
    ComputationFailed(String),
}

pub fn compute_diff(repo: &Repository, ...) -> Result<Diff, DiffError> {
    let tree = repo.find_tree(oid)
        .map_err(|_| DiffError::TreeNotFound(oid.to_string()))?;

    repo.diff_tree_to_tree(Some(&tree), None, opts)
        .map_err(DiffError::from)
}
```

**Advantage**:
- Callers can match on specific error types
- Excellent for library APIs
- Integrates well with error propagation operator `?`

**Source**: [Error Handling Best Practices](https://medium.com/@Murtza/error-handling-best-practices-in-rust-a-comprehensive-guide-to-building-resilient-applications-46bdf6fa6d9d)

#### 2. Opaque Error (anyhow - for Binaries)

**Best For**: Binary/CLI applications where callers just report error to user

**Pattern**:
```rust
use anyhow::{Result, Context};

fn main() -> Result<()> {
    let repo = Repository::open(".")
        .context("Failed to open repository")?;

    let diff = repo.diff_tree_to_tree(Some(&old), Some(&new), None)
        .context("Failed to compute diff")?;

    print_diff(&diff)
        .context("Failed to print diff")?;

    Ok(())
}
```

**Advantage**:
- Simple, focuses on error messages
- Good error chain for debugging
- No need to define custom error types

#### 3. snafu (Hybrid Approach)

**Best For**: Complex applications needing both typed errors and good context

**Pattern**:
```rust
use snafu::{Snafu, ResultExt};

#[derive(Snafu, Debug)]
pub enum Error {
    #[snafu(display("Tree not found: {}", tree_id))]
    TreeNotFound { tree_id: String, backtrace: Backtrace },

    #[snafu(display("Git operation failed: {}", source))]
    Git { source: git2::Error },
}

fn compute_diff(repo: &Repository, tree_id: &str) -> Result<Diff, Error> {
    let tree = repo.find_tree_oid(tree_id)
        .context(TreeNotFoundSnafu { tree_id })?;

    repo.diff_tree_to_tree(Some(&tree), None, None)
        .context(GitSnafu)
}
```

### Error Context in Diff Operations

**Critical Error Points**:

1. **Repository Access**
   ```rust
   repo.diff_tree_to_tree(...).context("Failed to compute tree diff")?
   ```

2. **Object Resolution**
   ```rust
   repo.find_tree(oid)
       .context(format!("Tree {} not found", oid))?
   ```

3. **Option Configuration**
   ```rust
   opts.old_prefix(prefix)
       .context("Invalid old prefix")?
   ```

4. **Delta Iteration**
   ```rust
   diff.foreach(|delta, _| {
       // Handle errors in callback
       process_delta(delta).ok(); // silently continue, or
       // Return false to stop processing and return error
       if !should_process(delta) {
           return false;
       }
       true
   })?
   ```

**Pattern for Layered Errors**:
```rust
pub type Result<T> = std::result::Result<T, DiffError>;

#[derive(Error, Debug)]
#[error("diff operation failed")]
pub struct DiffError {
    #[source]
    source: Box<dyn std::error::Error + Send + Sync>,
    context: String,
}

impl DiffError {
    fn with_context(source: impl std::error::Error, context: impl Into<String>) -> Self {
        DiffError {
            source: Box::new(source),
            context: context.into(),
        }
    }
}
```

### Panic vs Result

**Never Panic in Libraries**:
```rust
// BAD - Libraries should never panic
pub fn diff_trees(repo: &Repository, ...) -> Diff {
    repo.diff_tree_to_tree(...).unwrap()
}

// GOOD - Return Result
pub fn diff_trees(repo: &Repository, ...) -> Result<Diff, Error> {
    repo.diff_tree_to_tree(...)
}
```

**Acceptable Panics in Binaries**:
```rust
// CLI applications can panic on invariant violations
fn main() {
    let repo = Repository::open(".")
        .expect("Must run in git repository");
}
```

**Sources**:
- [Error Handling in GreptimeDB](https://greptime.com/blogs/2024-05-07-error-rust)
- [A Deep Dive into Error Handling](https://lpalmieri.com/posts/error-handling-rust/)
- [Designing Error Types](https://mmapped.blog/posts/12-rust-error-handling)
- [Rust Error Handling - Dev Community](https://dev.to/nathan20/how-to-handle-errors-in-rust-a-comprehensive-guide-1cco)

---

## Best Practices Summary

### For CLI Tool Implementation (jin)

#### 1. Diff Generation
```rust
// 1. Resolve objects
let old_tree = repo.head()?.peel_to_tree()?;
let new_tree = repo.index()?.write_tree()?;

// 2. Configure diff options
let mut opts = DiffOptions::new();
opts.include_untracked(include_untracked);
opts.find_renames(Some(threshold));

// 3. Generate appropriate diff
let diff = repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), Some(&opts))?;

// 4. Optionally refine
diff.find_similar(None)?;
```

#### 2. Output Formatting
```rust
// Support multiple formats
match output_format {
    OutputFormat::Patch => print_unified_diff(&diff),
    OutputFormat::Stat => print_diffstat(&diff),
    OutputFormat::NameOnly => print_names(&diff),
    OutputFormat::NameStatus => print_status(&diff),
    OutputFormat::SideBySide => print_side_by_side(&diff),
}

// Always support color
if atty::is(Stream::Stdout) {
    apply_colors(&output);
}
```

#### 3. Algorithm Selection
```rust
// Let users choose algorithm
match args.algorithm {
    "myers" => set_algorithm_myers(),
    "patience" => set_algorithm_patience(),
    "histogram" => set_algorithm_histogram(),
    "minimal" => set_algorithm_minimal(),
    _ => set_default(),
}
```

#### 4. Error Handling
```rust
#[derive(Error, Debug)]
pub enum DiffError {
    #[error("git error: {0}")]
    Git(#[from] git2::Error),
    #[error("invalid options: {0}")]
    InvalidOptions(String),
}

// Use ? operator for propagation
fn diff_layers(workspace: &Workspace) -> Result<LayerDiff, DiffError> {
    // Operations...
}
```

#### 5. Layer Diff Strategy
```rust
// For jin's multi-layer configuration
fn diff_configuration_layers(workspace: &Workspace) -> Result<ConfigDiff> {
    let base_config = workspace.load_base_layer()?;
    let scope_config = workspace.load_scope_layer()?;
    let mode_config = workspace.load_mode_layer()?;

    // Compare hierarchically
    let base_to_scope = compare_configs(&base_config, &scope_config)?;
    let scope_to_mode = compare_configs(&scope_config, &mode_config)?;

    Ok(ConfigDiff {
        base_to_scope,
        scope_to_mode,
        effective: merge_layers(vec![base_config, scope_config, mode_config]),
    })
}
```

---

## References and Resources

### Official Documentation
- [Git - git-diff Documentation](https://git-scm.com/docs/git-diff)
- [Git - git-diff-tree Documentation](https://git-scm.com/docs/git-diff-tree)
- [libgit2 Diff API Reference](https://libgit2.org/docs/reference/main/diff/index.html)
- [git2-rs Documentation](https://docs.rs/git2/latest/git2/)

### Algorithm Research
- [The Myers Diff Algorithm: Part 1](https://blog.jcoglan.com/2017/02/12/the-myers-diff-algorithm-part-1/)
- [Myers Diff in Linear Space: Theory](https://blog.jcoglan.com/2017/03/22/myers-diff-in-linear-space-theory/)
- [How Different Are Different Diff Algorithms in Git?](https://arxiv.org/abs/1902.02467)
- [Git Source Code Review: Diff Algorithms](https://www.fabiensanglard.net/git_code_review/diff.php)

### Rust Implementations
- [git2-rs Examples](https://github.com/rust-lang/git2-rs/blob/master/examples/diff.rs)
- [similar Crate](https://github.com/mitsuhiko/similar)
- [sjdiff - Structural JSON Diff](https://docs.rs/sjdiff)

### Error Handling
- [Error Handling Best Practices](https://medium.com/@Murtza/error-handling-best-practices-in-rust-a-comprehensive-guide-to-building-resilient-applications-46bdf6fa6d9d)
- [Error Handling in GreptimeDB](https://greptime.com/blogs/2024-05-07-error-rust)
- [thiserror Documentation](https://docs.rs/thiserror)
- [anyhow Documentation](https://docs.rs/anyhow)

### Configuration Diff Patterns
- [Cisco Configuration Diff Utility](https://www.cisco.com/c/en/us/td/docs/ios-xml/ios/config-mgmt/configuration/15-sy/config-mgmt-15-sy-book/cm-config-diff.html)
- [config-diff-tool](https://github.com/nerdynick/config-diff-tool)
