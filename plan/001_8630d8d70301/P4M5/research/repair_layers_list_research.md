# Repair, Layers, and List Commands Research Documentation

## Overview

This document consolidates research on patterns for implementing repair, layers, and list commands in the jin project. It covers Git repository repair strategies, Git2-rs APIs for verification, directory traversal patterns, and formatting libraries for hierarchical data display.

---

## 1. Git fsck (File System Check) Patterns

### Purpose and Core Functionality

`git fsck` verifies the connectivity and validity of objects in the Git database, detecting corruption and unreachable objects. It is a diagnostic read-only tool that reports issues but does not fix them.

**Reference:** [Git - git-fsck Documentation](https://git-scm.com/docs/git-fsck)

### Main Command Options

#### Diagnostic Options
- `--unreachable` - Print objects that exist but aren't reachable from reference nodes
- `--dangling` / `--no-dangling` - Print objects never directly used (default: enabled)
- `--root` - Report root nodes
- `--tags` - Report tags

#### Scope Options
- `--full` - Check all object pools including alternates and packed archives (default)
- `--connectivity-only` - Check only reachability, skip blob validation (faster)
- `--cache` - Include index file objects in unreachability trace
- `--no-reflogs` - Exclude reflog-only commits

#### Output & Behavior
- `--verbose` - Detailed output
- `--strict` - Catch legacy file modes (g+w bit)
- `--lost-found` - Write dangling objects to `.git/lost-found/`
- `--name-objects` - Show reachable object names with paths
- `--progress` / `--no-progress` - Control progress reporting

### Repair Patterns

#### Pattern 1: Find Corruption
```bash
git fsck --strict --verbose
```
Identifies all issues in the repository with detailed output.

#### Pattern 2: Locate Dangling Objects
```bash
git fsck --dangling --lost-found
```
Writes recoverable objects to `.git/lost-found/commit/` or `.git/lost-found/other/`

#### Pattern 3: Quick Connectivity Check
```bash
git fsck --connectivity-only
```
Faster check focusing only on reachability without blob validation.

#### Pattern 4: Comprehensive Audit
```bash
git fsck --full --strict --verbose
```
Most thorough check of repository integrity.

### Configuration for Repair

**fsck.<msg-id>** - Control error severity:
```bash
git config fsck.missingEmail ignore   # Convert error to warning
git config fsck.badFilemode warn      # Adjust severity
```

**fsck.skipList** - Ignore known issues:
```bash
git config fsck.skipList .git/fsck-skip
# Add problematic SHA-1s (one per line) to .git/fsck-skip
```

### Recovery Workflow

1. Detect issues: `git fsck --strict`
2. Save dangling objects: `git fsck --lost-found`
3. Inspect recoverable commits in `.git/lost-found/commit/`
4. Configure tolerance for legacy issues if needed
5. Restore corrupted objects from backups if necessary

---

## 2. Repository Integrity Verification

### Git2-rs Verification APIs

**Reference:** [Repository in git2 - Rust](https://docs.rs/git2/latest/git2/struct.Repository.html)

The git2-rs crate provides the following integrity verification mechanisms:

#### Hash Verification Control
```rust
opts::strict_hash_verification(true)  // Enable verification (default)
opts::strict_hash_verification(false) // Disable for performance
```

**Purpose:** Controls whether libgit2 verifies that objects loaded have the expected hash.

#### Object Creation Verification
```rust
opts::strict_object_creation(true)   // Verify referenced objects exist (default)
opts::strict_object_creation(false)  // Skip verification
```

**Purpose:** Controls whether libgit2 verifies when writing an object that all objects it references are valid.

### Object Database (ODB) Verification

**References:**
- [Odb in git2 - Rust](https://docs.rs/git2/latest/git2/struct.Odb.html)
- [odb APIs (libgit2 main)](https://libgit2.org/docs/reference/main/odb/index.html)

#### Core ODB Operations

**Object Existence Checks:**
```rust
odb.exists(oid)                    // Check if object exists
odb.exists_ext(oid, flags)         // Extended existence check
odb.exists_prefix(short_oid, length)  // Check by abbreviated OID
```

**Object Validation & Lookup:**
```rust
odb.expand_ids(ids)                // Determine if objects can be found by abbreviated ID
odb.read(oid)                      // Read full object
odb.read_header(oid)               // Read object metadata without content
```

**Iteration & Enumeration:**
```rust
odb.foreach(|oid| {
    // Process each object in database
    // Return non-zero to stop iteration
    0
})
```

**Maintenance:**
```rust
odb.refresh()                      // Reload newly added files
```

### Index Operations

**Reference:** [Index in git2 - Rust](https://docs.rs/git2/latest/git2/struct.Index.html)

#### Index Reading and Writing

**Reading:**
```rust
let mut index = repo.index()?;
index.read(false)?;                // Update from disk
index.read_tree(&tree)?;           // Replace with tree contents
```

**Writing:**
```rust
index.write()?;                    // Write to disk atomically
let tree_oid = index.write_tree()?; // Convert to tree object
let tree_oid = index.write_tree_to(&repo)?; // Write to specific repo
```

#### Index Update Operations

```rust
index.add_path(path)?;             // Add single file (relative to repo)
index.add_all(pathspecs)?;         // Add matching files
index.remove_all(pathspecs)?;      // Remove matching entries
index.update_all(pathspecs)?;      // Sync with working directory
```

**Important:** Always call `write()` after modifying the index to persist changes to disk.

---

## 3. Repair Strategies

### Index Repair

**Reference:** [How to Fix Git Index File](https://exploratory.io/note/exploratory/How-to-Fix-Git-Index-File-to-Recover-from-index-file-corrupt-Error-aeO7Nge1)

#### Pattern: Corrupt Index Recovery

1. **Back up the corrupt index** (optional but recommended):
   ```bash
   cp .git/index .git/index.bak
   ```

2. **Remove the corrupt index**:
   ```bash
   rm .git/index
   ```

3. **Rebuild the index** - Choose one approach:
   - Using `git reset --mixed`: Makes index match last commit, leaves worktree alone
   - Using `git read-tree`: Lower-level plumbing command for index restoration

**Implementation in git2-rs:**
```rust
let mut index = repo.index()?;
let head_tree = repo.head()?.peel_to_tree()?;
index.read_tree(&head_tree)?;
index.write()?;
```

**Common Causes:**
- Power loss or system crash during Git operation
- Disk errors or file system issues
- Nested .git directories (check for sub-repositories)

### Object Corruption Handling

**Reference:** [Git Cookbook – Repairing Broken Repositories](https://git.seveas.net/repairing-and-recovering-broken-git-repositories.html)

#### Pattern: Identify and Repair Corrupt Objects

1. **Identify corruption**:
   ```bash
   git fsck --full --strict
   ```

2. **Remove corrupt loose objects** (if identified):
   ```bash
   rm .git/objects/[first-2-chars]/[remaining-38-chars]
   ```

3. **Recover from packfiles** (if corruption in pack):
   ```bash
   git unpack-objects -r < .git/objects/pack/[packfile]
   ```

4. **Restore from remote**:
   ```bash
   git fetch origin
   ```

#### Pattern: Dangling Object Recovery

```bash
git fsck --dangling --lost-found
# Objects written to .git/lost-found/commit/ or .git/lost-found/other/
# Inspect and restore as needed
```

### Reference Recovery

**Reference:** [Git Reflog: Recovery Guide](https://www.thisdot.co/blog/git-reflog-a-guide-to-recovering-lost-commits)

#### Pattern: Using Reflog for Recovery

```bash
git reflog show                    # View all reference changes
git reflog show HEAD               # View HEAD changes specifically
git reset --hard HEAD@{n}          # Restore to specific reflog entry
```

**Key Points:**
- Reflog records every change to HEAD locally
- Keeps dangling commits for 30 days after reflog expires
- Cannot recover commits never in your local repo
- Use after discovering lost commits with `git fsck --full`

---

## 4. Layer Enumeration and Display

### Layer Concepts in jin Project

Layers represent different operational contexts:
- Project layers
- Mode layers
- Scope layers
- Hierarchy/grouping for logical organization

### Enumeration Patterns using git2-rs

#### Pattern: Enumerate All Commits (Layer Timeline)

**Reference:** [git2-rs Revwalk Examples](https://github.com/rust-lang/git2-rs/blob/master/examples/log.rs)

```rust
let mut revwalk = repo.revwalk()?;
revwalk.push_head()?;
revwalk.set_sorting(git2::Sort::TIME)?;

for oid in revwalk {
    let oid = oid?;
    let commit = repo.find_commit(oid)?;
    println!("{} {}", oid, commit.message().unwrap_or(""));
}
```

#### Pattern: Enumerate Branches (Layer Sources)

```rust
let branches = repo.branches(None)?;
for branch_result in branches {
    let (branch, _) = branch_result?;
    let name = branch.name()?.unwrap_or("detached");
    println!("Branch: {}", name);
}
```

#### Pattern: Enumerate References

```rust
let references = repo.references()?;
for ref_result in references {
    let reference = ref_result?;
    if let Some(name) = reference.name() {
        println!("Ref: {}", name);
    }
}
```

#### Pattern: Enumerate Tags (Layer Markers)

```rust
let tag_names = repo.tag_names(None)?;
for tag_name in tag_names.iter() {
    if let Some(name) = tag_name {
        println!("Tag: {}", name);
    }
}
```

---

## 5. Tree Walking for Listing Files

### Directory Traversal with walkdir

**Reference:** [walkdir - Rust](https://docs.rs/walkdir/latest/walkdir/)
**GitHub:** [BurntSushi/walkdir](https://github.com/BurntSushi/walkdir)

#### Basic Directory Traversal

```rust
use walkdir::WalkDir;

for entry in WalkDir::new("path/to/dir")
    .into_iter()
    .filter_map(|e| e.ok())
{
    println!("{}", entry.path().display());
}
```

#### Common Patterns

**Filter Hidden Files (Efficient):**
```rust
for entry in WalkDir::new(".")
    .into_iter()
    .filter_entry(|e| !e.file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false))
    .filter_map(|e| e.ok())
{
    println!("{}", entry.path().display());
}
```

**Control Depth:**
```rust
WalkDir::new(".")
    .max_depth(3)              // Limit depth
    .min_depth(1)              // Skip root directory
```

**Follow Symbolic Links:**
```rust
WalkDir::new(".")
    .follow_links(true)
```

**Sort Entries:**
```rust
WalkDir::new(".")
    .sort_by_file_name()
```

**Stay on Same Filesystem:**
```rust
WalkDir::new(".")
    .same_file_system(true)
```

#### Git Object Walking with git2-rs

**Pattern: Walk Tree Objects**

```rust
fn walk_tree(tree: &git2::Tree, prefix: &str) -> Result<()> {
    for entry in tree.iter() {
        let path = format!("{}{}", prefix, entry.name()?);

        if entry.kind() == Some(git2::ObjectType::Tree) {
            let obj = entry.to_object(&repo)?;
            let subtree = obj.as_tree()?;
            walk_tree(&subtree, &format!("{}/", path))?;
        } else {
            println!("File: {}", path);
        }
    }
    Ok(())
}
```

---

## 6. Formatting Hierarchical Data

### Table Formatting Libraries

#### comfy-table

**Reference:** [comfy-table - Rust](https://docs.rs/comfy-table/latest/comfy_table/)
**GitHub:** [Nukesor/comfy-table](https://github.com/Nukesor/comfy-table)

**Purpose:** Build beautiful terminal tables with automatic content wrapping

**Key Features:**
- Dynamic content arrangement by width
- ANSI color and styling support
- Customizable borders, padding, alignment
- Styling presets (UTF8_FULL, ASCII, etc.)
- No unsafe code

**Basic Usage:**
```rust
use comfy_table::Table;

let mut table = Table::new();
table.set_header(vec!["Header1", "Header2", "Header3"])
     .add_row(vec!["data1", "data2", "data3"]);

println!("{table}");
```

**Styling:**
```rust
let mut table = Table::new();
table.set_header(vec!["Name", "Size", "Type"]);
table.add_row(vec!["file.rs", "1.2 KB", "File"]);

// Access and style columns
table.column_mut(0).set_header("Name (Primary)");
println!("{table}");
```

#### prettytable-rs

**Reference:** [prettytable-rs - crates.io](https://crates.io/crates/prettytable-rs)

**Purpose:** Aligned and formatted table printing

**Features:**
- Styled output with colors and formatting
- Cell alignment (Left, Right, Center)
- Multi-column spanning
- Minimum Rust version: 1.56

**Note:** This library has been superseded by comfy-table for new projects.

#### tabled

**Alternative:** [tabled - Rust](https://lib.rs/crates/tabled)

**Purpose:** Pretty print tables of Rust structs and enums

### Tree Formatting Libraries

#### DisplayTree

**Reference:** [DisplayTree in display_tree - Rust](https://docs.rs/display_tree/latest/display_tree/trait.DisplayTree.html)
**GitHub:** [captain-camel/display_tree](https://github.com/captain-camel/display_tree)

**Purpose:** Simple, automatic, and customizable tree pretty-printing

**Derive Macro Usage:**
```rust
#[derive(DisplayTree)]
struct MyTree {
    name: String,
    #[tree]
    children: Vec<MyTree>,
}

fn main() {
    let tree = MyTree { /* ... */ };
    println!("{}", AsTree::new(&tree));
}
```

**Key Attributes:**
- `#[tree]` - Format field using DisplayTree instead of Display
- `#[ignore_field]` - Exclude from output
- `#[node_label]` - Use field value as parent node label
- `#[field_label]` - Add "label: value" formatting
- `#[node_label = "custom"]` - Custom struct-level label

#### ptree

**Reference:** [ptree - Rust](https://docs.rs/ptree/latest/ptree/)
**Crates.io:** [ptree](https://crates.io/crates/ptree)

**Purpose:** Pretty-print tree-like structures with flexible configuration

**Main APIs:**

```rust
use ptree::{TreeBuilder, print_tree};

// Using TreeBuilder
let mut tree = TreeBuilder::new("root");
tree.add("child1");
tree.add("child2");
print_tree(&tree.build())?;

// Using TreeItem trait
impl TreeItem for MyType {
    fn write_self<W: Write>(&self, w: &mut W, style: &Style) -> io::Result<()> {
        write!(w, "Item: {}", self.name)
    }
    fn children(&self) -> Cow<[Box<dyn TreeItem>]> {
        Cow::from(vec![/* children */])
    }
}
```

**Configuration:**
- `PrintConfig` for custom formatting
- Environment variables: `PTREE_INDENT`, `PTREE_BRANCH_FOREGROUND`
- Config files in platform-specific user config directory
- Support for JSON, YAML, TOML deserialized data

**Output Methods:**
```rust
print_tree(&item)?;                          // To stdout
print_tree_with(&item, config)?;             // Custom config
write_tree(&mut writer, &item)?;             // To custom writer
```

#### text_trees

**Reference:** [text_trees - Rust](https://docs.rs/text_trees)

**Purpose:** Flexible tree structure output in text with ASCII and Unicode characters

#### Custom Implementation Pattern

For simple tree printing, create a helper function:
```rust
fn print_tree(node: &str, children: &[String], indent: &str, is_last: bool) {
    let connector = if is_last { "└── " } else { "├── " };
    println!("{}{}{}", indent, connector, node);

    let next_indent = indent.to_string() + if is_last { "    " } else { "│   " };
    // Recursively print children
}
```

---

## 7. Rust Formatting Libraries Summary

### Table Formatting Comparison

| Library | Features | Notes |
|---------|----------|-------|
| comfy-table | Colors, auto-wrapping, presets | Recommended, no unsafe |
| prettytable-rs | Styling, alignment, spanning | Superseded by comfy-table |
| tabled | Derive macro, struct display | Alternative approach |

### Tree Formatting Comparison

| Library | Mechanism | Use Case |
|---------|-----------|----------|
| DisplayTree | Derive macro + trait | Best for simple hierarchies |
| ptree | TreeItem trait | Flexible, config-driven |
| text_trees | Generic TreeNode | ASCII/Unicode output |
| Custom | Manual recursion | Maximum control |

---

## 8. Implementation Recommendations

### For `repair` Command
1. Use `git fsck --full --strict` pattern for detection
2. Implement index rebuild using git2-rs Index APIs
3. Provide `--lost-found` option to save dangling objects
4. Support configuration with `fsck.skipList`
5. Return structured error information for user recovery

### For `layers` Command
1. Use git2-rs revwalk for commit enumeration
2. Enumerate branches, tags, and references
3. Format output with comfy-table for structured display
4. Support hierarchical display using ptree or DisplayTree
5. Include metadata (commit count, recent updates)

### For `list` Command
1. Use walkdir for filesystem traversal
2. Use git2-rs tree walking for object enumeration
3. Format with comfy-table for structured output
4. Support filtering (depth, patterns, hidden)
5. Show file metadata (size, mode, hash)

### General Patterns
- Always provide `--verbose` option for diagnostic output
- Use structured error types for detailed error reporting
- Support both machine-readable (JSON) and human-readable output
- Implement progress reporting for long operations
- Cache results where appropriate for performance

---

## References

### Git fsck and Repair
- [Git - git-fsck Documentation](https://git-scm.com/docs/git-fsck)
- [Git - Maintenance and Data Recovery](https://git-scm.com/book/en/v2/Git-Internals-Maintenance-and-Data-Recovery)
- [Git Cookbook – Repairing Broken Repositories](https://git.seveas.net/repairing-and-recovering-broken-git-repositories.html)
- [Mastering Git Fsck: Your Guide to Repository Integrity](https://gitscripts.com/git-fsck)

### Git2-rs APIs
- [Repository in git2 - Rust](https://docs.rs/git2/latest/git2/struct.Repository.html)
- [Index in git2 - Rust](https://docs.rs/git2/latest/git2/struct.Index.html)
- [Odb in git2 - Rust](https://docs.rs/git2/latest/git2/struct.Odb.html)
- [git2-rs GitHub Repository](https://github.com/rust-lang/git2-rs)

### Index and Object Operations
- [How to Fix Git Index File](https://exploratory.io/note/exploratory/How-to-Fix-Git-Index-File-to-Recover-from-index-file-corrupt-Error-aeO7Nge1)
- [odb APIs (libgit2 main)](https://libgit2.org/docs/reference/main/odb/index.html)
- [Understanding Git — Index](https://konrad126.medium.com/understanding-git-index-4821a0765cf)

### Reference Recovery
- [Git Reflog: Recovery Guide](https://www.thisdot.co/blog/git-reflog-a-guide-to-recovering-lost-commits)
- [Atlassian Git Refs and Reflog Tutorial](https://www.atlassian.com/git/tutorials/refs-and-the-reflog)

### Directory Traversal
- [walkdir - Rust Documentation](https://docs.rs/walkdir/latest/walkdir/)
- [BurntSushi/walkdir GitHub](https://github.com/BurntSushi/walkdir)
- [Directory Traversal - Rust Cookbook](https://rust-lang-nursery.github.io/rust-cookbook/file/dir.html)

### Table Formatting
- [comfy-table - Rust Documentation](https://docs.rs/comfy-table/latest/comfy_table/)
- [Nukesor/comfy-table GitHub](https://github.com/Nukesor/comfy-table)
- [prettytable-rs - crates.io](https://crates.io/crates/prettytable-rs)
- [tabled - Rust Library](https://lib.rs/crates/tabled)

### Tree Formatting
- [DisplayTree in display_tree - Rust](https://docs.rs/display_tree/latest/display_tree/trait.DisplayTree.html)
- [captain-camel/display_tree GitHub](https://github.com/captain-camel/display_tree)
- [ptree - Rust Documentation](https://docs.rs/ptree/latest/ptree/)
- [ptree - crates.io](https://crates.io/crates/ptree)
- [text_trees - Rust](https://docs.rs/text_trees)

### Git Object Model
- [Git - Git Objects](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects)
- [Git - git-hash-object Documentation](https://git-scm.com/docs/git-hash-object)
- [Types of git objects — Curious git](https://matthew-brett.github.io/curious-git/git_object_types.html)

---

## Document Metadata

- **Created:** 2025-12-27
- **Research Scope:** Repair, layers, and list command patterns
- **Coverage:** Git2-rs APIs, repair strategies, directory traversal, formatting libraries
- **Audience:** jin project developers implementing P4M5 commands
