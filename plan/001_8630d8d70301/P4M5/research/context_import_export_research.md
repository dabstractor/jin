# Context, Import, and Export Commands Research

## Executive Summary

This document consolidates comprehensive research on patterns for implementing context display, import, and export commands in version control systems. The research focuses on:

1. **Context Display Patterns** - showing active state within the jin system
2. **Git Import Workflows** - transferring files from git to jin tracking
3. **Export Workflows** - moving files from jin back to git
4. **File Migration Patterns** - safe transfers between tracking systems
5. **Metadata Preservation** - maintaining file attributes and history

## Table of Contents

1. [Context Display Patterns](#context-display-patterns)
2. [Git Porcelain and Plumbing Commands](#git-porcelain-and-plumbing-commands)
3. [File Tracking and Untracking Workflows](#file-tracking-and-untracking-workflows)
4. [Programmatic Index Manipulation](#programmatic-index-manipulation)
5. [File Import/Export Mechanisms](#fileimportexport-mechanisms)
6. [Atomic Operations and Error Recovery](#atomic-operations-and-error-recovery)
7. [Metadata Preservation Strategies](#metadata-preservation-strategies)
8. [Rust Patterns for File System Operations](#rust-patterns-for-file-system-operations)
9. [Implementation Recommendations](#implementation-recommendations)

---

## 1. Context Display Patterns

### 1.1 Active Mode Display

The jin system requires displaying the currently active mode and layer composition, similar to git's branch visualization.

#### Pattern: Symbolic References

**Source**: [Git - git-symbolic-ref Documentation](https://git-scm.com/docs/git-symbolic-ref)

Git uses symbolic references (symrefs) to track the current branch context:

```bash
# Read current HEAD reference
git symbolic-ref HEAD
# Output: ref: refs/heads/main

# Get just the branch name
git symbolic-ref --short HEAD
# Output: main

# Detect detached HEAD state (returns error)
git symbolic-ref HEAD 2>/dev/null || echo "detached HEAD"
```

**Application for jin context display:**

```rust
// Pseudo-code pattern for context display
fn display_context() -> Result<ContextInfo> {
    let current_mode = read_symbolic_ref(".jin/mode")?;
    let current_scope = read_symbolic_ref(".jin/scope")?;
    let layer_stack = read_layer_composition(".jin/layers")?;

    println!("Active Mode: {}", current_mode.short_name());
    println!("Scope: {}", current_scope.short_name());
    println!("Layers:");
    for (idx, layer) in layer_stack.iter().enumerate() {
        println!("  [{}] {} ({})", idx, layer.name, layer.version);
    }
}
```

#### Pattern: Configuration Hierarchy

**Source**: [Git - git-config Documentation](https://git-scm.com/docs/git-config)

Git's config system uses a hierarchical scope system that jin can mirror:

```
Command (highest priority)
    ↓
Worktree
    ↓
Local (.jin/config)
    ↓
Global (~/.jin/config)
    ↓
System (/etc/jin/config) (lowest priority)
```

**Config Scope Priority:**

| Scope | Location | Use Case |
|-------|----------|----------|
| Command | `JIN_CONFIG_*` env vars, `-c` flags | Temporary overrides |
| Worktree | `.jin/config.worktree` | Multi-worktree configs |
| Local | `.jin/config` | Project-specific settings |
| Global | `~/.jin/config` | User-wide defaults |
| System | `/etc/jin/config` | Machine-wide settings |

**Implementation pattern:**

```rust
fn get_config_value(key: &str) -> Result<String> {
    // Check scopes in priority order
    if let Ok(val) = env::var(&format!("JIN_CONFIG_{}", key.to_uppercase())) {
        return Ok(val);
    }

    if let Ok(val) = read_worktree_config(key)? {
        return Ok(val);
    }

    if let Ok(val) = read_local_config(key)? {
        return Ok(val);
    }

    if let Ok(val) = read_global_config(key)? {
        return Ok(val);
    }

    read_system_config(key)
}
```

### 1.2 Active Mode and Scope Display

**Source**: [Understanding the Three Levels of Git Config](https://medium.com/@yadavprakhar1809/understanding-the-three-levels-of-git-config-local-global-and-system-e95c26aac8ee)

Display patterns should show:
- Current active mode (from symbolic ref)
- Current scope level (local/global/system)
- Scope hierarchy for transparency
- Configuration value sources

```rust
struct ContextDisplay {
    active_mode: String,           // Current mode name
    mode_ref_path: String,         // .jin/mode symbolic ref
    active_scope: ScopeLevel,      // Local, Global, System, etc.
    layer_composition: Vec<Layer>, // Stack of active layers
    config_source: ConfigSource,   // Where config came from
}

impl ContextDisplay {
    fn show_full_context(&self) -> String {
        format!(
            "Mode: {} ({})\nScope: {}\nLayers: {}\nConfig: {}",
            self.active_mode,
            self.mode_ref_path,
            self.active_scope,
            self.layer_composition.len(),
            self.config_source
        )
    }
}
```

### 1.3 Layer Composition Visualization

Display the stack of active layers with their metadata:

```
Active Context:
  Mode:     feature-branch
  Scope:    Local (.jin/config)

  Layer Stack:
    [0] base          (v1.0.0) - stable, 3 files
    [1] feature-new   (v1.1.0) - 5 files
    [2] experimental  (dev)    - 2 files
```

---

## 2. Git Porcelain and Plumbing Commands

### 2.1 Porcelain vs. Plumbing Architecture

**Source**: [Git - Plumbing and Porcelain](https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain)

Git's architecture separates user-friendly commands (porcelain) from low-level operations (plumbing):

#### Porcelain Commands (High-Level)
- `git add` - Stage files for commit
- `git commit` - Create a commit
- `git checkout` - Switch branches
- `git push` / `git pull` - Synchronize with remotes

#### Plumbing Commands (Low-Level)

| Command | Purpose |
|---------|---------|
| `git hash-object` | Create blob object from file content |
| `git update-index` | Directly manipulate staging area |
| `git write-tree` | Create tree object from index |
| `git commit-tree` | Create commit object from tree |
| `git cat-file` | Inspect Git objects |
| `git ls-files` | List files in index |
| `git check-ignore` | Check if paths match ignore rules |

**Application for jin:**

```rust
// Porcelain-level API (user-friendly)
pub fn jin_add_files(paths: &[&str]) -> Result<()> {
    // High-level wrapper around plumbing operations
}

// Plumbing-level API (scriptable)
pub fn jin_hash_object(content: &[u8]) -> Result<Hash> {
    // Low-level content hashing
}

pub fn jin_update_layer_index(mode: &str, files: &[(String, Hash)]) -> Result<()> {
    // Direct index manipulation
}
```

### 2.2 Object Model for File Operations

**Source**: [Git - Git Objects](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects)

Git's object model consists of:

```
Blob (file content)
  ↓
Tree (directory listing with blobs/trees)
  ↓
Commit (snapshot with metadata)
  ↓
Tag (annotated reference to commit)
```

**For jin file import/export:**

```rust
// Content-addressable storage pattern
struct JinObject {
    hash: Sha1Hash,      // Content-based ID
    object_type: Type,   // blob, tree, commit
    size: usize,
    data: Vec<u8>,
}

enum Type {
    Blob(Vec<u8>),           // Raw file content
    Tree(Vec<TreeEntry>),    // Directory listing
    Commit(CommitMetadata),  // Version snapshot
}

struct TreeEntry {
    mode: FileMode,   // 100644 (file), 100755 (exec), 120000 (symlink)
    name: String,
    hash: Sha1Hash,   // Points to blob or tree
}
```

### 2.3 Fast Import/Export Format

**Source**: [Git - git-fast-import Documentation](https://git-scm.com/docs/git-fast-import)

Git's `fast-import` format enables efficient bulk data transfer:

```
commit refs/heads/main
author Name <email@example.com> 1234567890 -0500
committer Name <email@example.com> 1234567890 -0500
data 12
Initial commit

M 100644 4b825dc642cb6eb9a060e54bf8d69288fbee4904 README.md
M 100755 5f4c8a44f7e2f1c5c3b8a9d6e7f8g9h0i1j2k3l4 script.sh
```

**Key commands for migration:**

```bash
# Export repository history
git fast-export --all > repo.dump

# Import into new repository
git init new-repo
cd new-repo
git fast-import < repo.dump

# Incremental update with marks for resume capability
git fast-export --all --export-marks=marks.txt > repo.dump
git fast-import --import-marks=marks.txt < incremental.dump
```

**For jin export workflow:**

```rust
struct FastImportCommand {
    command: String,      // "commit", "blob", "reset", etc.
    data: HashMap<String, String>,
}

impl FastImportCommand {
    fn blob(content: &[u8]) -> Self {
        FastImportCommand {
            command: "blob".to_string(),
            data: [
                ("data".to_string(), format!("{}", content.len())),
                ("content".to_string(), String::from_utf8_lossy(content).to_string()),
            ].iter().cloned().collect(),
        }
    }

    fn serialize(&self) -> String {
        format!("{} LF\ndata {} LF\n{}",
            self.command,
            self.data.get("data").unwrap_or(&"0".to_string()),
            self.data.get("content").unwrap_or(&"".to_string())
        )
    }
}
```

---

## 3. File Tracking and Untracking Workflows

### 3.1 Git File State Machine

**Source**: [Git - Recording Changes to the Repository](https://git-scm.com/book/en/v2/Git-Basics-Recording-Changes-to-the-Repository)

Files in git exist in states:

```
Untracked → Staged → Committed
    ↑          ↓
    └─ Modified (unstaged changes)
```

**For jin file import:**

```
Git-tracked → Jin-staged → Jin-committed
     ↑             ↓
     └─ Modified (unstaged in jin)
```

### 3.2 Removing Files from Git Tracking

**Source**: [How to Stop Tracking File in Git](https://www.delftstack.com/howto/git/git-stop-tracking-file/)

Safe removal pattern:

```bash
# Remove from staging without deleting local copy
git rm --cached filename.txt

# Remove from tracking (all matching pattern)
git rm --cached '*.log'

# Update .gitignore to prevent re-tracking
echo "*.log" >> .gitignore

# Commit the changes
git commit -m "Stop tracking filename.txt"
```

**For jin import from git:**

```rust
fn import_from_git(git_path: &str, jin_mode: &str) -> Result<()> {
    // 1. Verify file exists and is tracked in git
    let is_tracked = check_git_tracking(git_path)?;

    if !is_tracked {
        return Err("File not tracked in git".into());
    }

    // 2. Read current content and metadata
    let content = read_file(git_path)?;
    let metadata = get_file_metadata(git_path)?;

    // 3. Hash and store in jin
    let hash = hash_object(&content)?;
    store_in_jin(jin_mode, &hash, &content)?;

    // 4. Remove from git tracking
    git_remove_cached(git_path)?;

    // 5. Add to .gitignore
    add_to_gitignore(git_path)?;

    // 6. Update jin layer index
    update_jin_index(jin_mode, git_path, &hash, metadata)?;

    // 7. Commit change
    git_commit(&format!("Import {} into jin layer: {}", git_path, jin_mode))?;

    Ok(())
}
```

### 3.3 Large File Migration (LFS Pattern)

**Source**: [Git LFS Tracking, Migration and Un-Migration](https://mslinn.com/git/5300-git-lfs-patterns-tracking.html)

Git LFS migration pattern can inform jin's approach:

```bash
# Enable LFS tracking for file type
git lfs track "*.mp4"

# Migrate existing files
git lfs migrate import --include="*.mp4"

# Export (convert LFS pointers back to files)
git lfs migrate export --include="*.mp4"
```

**Applies to jin for large file handling:**

```rust
struct MigrationConfig {
    include_patterns: Vec<String>,  // Files to migrate
    exclude_patterns: Vec<String>,
    preserve_history: bool,         // Keep git history
}

fn migrate_to_jin(config: MigrationConfig) -> Result<MigrationReport> {
    let mut report = MigrationReport::new();

    for pattern in &config.include_patterns {
        let files = find_matching_files(pattern)?;

        for file in files {
            match import_file_to_jin(&file) {
                Ok(_) => report.success_count += 1,
                Err(e) => report.add_error(&file, e),
            }
        }
    }

    Ok(report)
}
```

---

## 4. Programmatic Index Manipulation

### 4.1 Git Index Structure

**Source**: [Git - git-ls-files Documentation](https://git-scm.com/docs/git-ls-files) and [Understanding the Git Index](https://shafiul.github.io/gitbook/1_the_git_index.html)

The Git index (staging area) is a binary file in `.git/index` that tracks:

```rust
struct IndexEntry {
    ctime: SystemTime,          // Creation time
    mtime: SystemTime,          // Modification time
    dev: u32,                   // Device ID
    ino: u32,                   // Inode number
    mode: FileMode,             // 100644, 100755, 120000
    uid: u32,                   // User ID
    gid: u32,                   // Group ID
    size: u32,                  // File size
    hash: Sha1Hash,             // Object ID (blob hash)
    path: String,               // File path
    stage: Stage,               // 0=normal, 1=base, 2=ours, 3=theirs
}
```

### 4.2 Reading Index Information

**Pattern for inspecting jin layer index:**

```bash
# List all tracked files in current layer
jin ls-files --cached

# Show file with metadata
jin ls-files --stage
# Output: [stage] <mode> <object> <stage_num> <file>
# Example: 100644 4b825dc642cb6 0 README.md

# List modified files
jin ls-files --modified

# List deleted files
jin ls-files --deleted

# List untracked files
jin ls-files --others

# List with custom format
jin ls-files --format='%(objectname) %(path)'
```

### 4.3 Updating Index Programmatically

**Source**: [git-update-index Documentation](https://git-scm.com/docs/git-update-index) and [Linux Command Library - git-update-index](https://linuxcommandlibrary.com/man/git-update-index)

Low-level index manipulation:

```bash
# Add file to index with specific mode and hash
git update-index --add --cacheinfo 100644 \
    4b825dc642cb6eb9a060e54bf8d69288fbee4904 \
    README.md

# Add file from working directory
git update-index --add filename.txt

# Remove file from index
git update-index --remove filename.txt

# Mark file as unchanged (assumes-unchanged)
git update-index --assume-unchanged sensitive.key

# Opposite - stop assuming unchanged
git update-index --no-assume-unchanged sensitive.key

# Update specific file stats without changing content
git update-index --really-refresh filename.txt
```

**For jin layer management:**

```rust
struct LayerIndex {
    entries: HashMap<PathBuf, IndexEntry>,
}

impl LayerIndex {
    fn add_cached(
        &mut self,
        mode: FileMode,
        hash: Sha1Hash,
        path: PathBuf,
    ) -> Result<()> {
        self.entries.insert(
            path.clone(),
            IndexEntry {
                mode,
                hash,
                path: path.to_string_lossy().to_string(),
                stage: Stage::Normal,
                ctime: SystemTime::now(),
                mtime: SystemTime::now(),
                size: 0,
                dev: 0,
                ino: 0,
                uid: 0,
                gid: 0,
            },
        );
        Ok(())
    }

    fn add_from_file(&mut self, path: &Path) -> Result<()> {
        let content = std::fs::read(path)?;
        let hash = hash_content(&content)?;
        let metadata = std::fs::metadata(path)?;

        self.entries.insert(
            path.to_path_buf(),
            IndexEntry {
                mode: get_file_mode(&metadata),
                hash,
                path: path.to_string_lossy().to_string(),
                stage: Stage::Normal,
                ctime: metadata.created()?,
                mtime: metadata.modified()?,
                size: metadata.len() as u32,
                dev: 0,
                ino: 0,
                uid: 0,
                gid: 0,
            },
        );
        Ok(())
    }

    fn remove(&mut self, path: &Path) -> Result<()> {
        self.entries.remove(path);
        Ok(())
    }
}
```

### 4.4 Hash Objects Directly

**Pattern for content addressing:**

```bash
# Hash file content without storing
git hash-object filename.txt

# Hash stdin content
echo "test content" | git hash-object --stdin

# Store in git object database
git hash-object -w filename.txt
```

**For jin:**

```rust
fn hash_object(content: &[u8]) -> Result<Sha1Hash> {
    let mut hasher = Sha1::new();

    // Git format: "blob <size>\0<content>"
    hasher.update(format!("blob {}\0", content.len()).as_bytes());
    hasher.update(content);

    Ok(Sha1Hash::from(hasher.finalize()))
}

fn store_object(hash: &Sha1Hash, content: &[u8]) -> Result<()> {
    let object_dir = format!(".jin/objects/{}", &hash.to_string()[..2]);
    fs::create_dir_all(&object_dir)?;

    let object_file = format!("{}/{}", object_dir, &hash.to_string()[2..]);

    // Store with compression
    let compressed = compress_zlib(content)?;
    fs::write(object_file, compressed)?;

    Ok(())
}
```

---

## 5. File Import/Export Mechanisms

### 5.1 Import Workflow (Git → Jin)

**Full workflow for safe file migration:**

```rust
struct ImportOperation {
    source_path: PathBuf,
    target_layer: String,
    preserve_history: bool,
    atomic: bool,
}

impl ImportOperation {
    fn execute(self) -> Result<ImportResult> {
        // Phase 1: Pre-flight checks
        self.validate_source()?;
        self.validate_target_layer()?;
        self.check_conflicts()?;

        // Phase 2: Preparation (no side effects)
        let content = fs::read(&self.source_path)?;
        let metadata = fs::metadata(&self.source_path)?;
        let hash = hash_object(&content)?;

        // Phase 3: Atomic transaction
        let txn = ImportTransaction::begin()?;

        match (|| {
            // Store object
            txn.store_object(&hash, &content)?;

            // Update layer index
            txn.update_layer_index(
                &self.target_layer,
                &self.source_path,
                &hash,
                &metadata,
            )?;

            // Write transaction log
            txn.log_operation(&ImportLog {
                timestamp: SystemTime::now(),
                source: self.source_path.clone(),
                target_layer: self.target_layer.clone(),
                hash: hash.clone(),
                size: content.len(),
            })?;

            Ok(hash)
        })() {
            Ok(hash) => {
                txn.commit()?;

                // Phase 4: Post-import cleanup (git side)
                self.remove_from_git()?;
                self.add_to_gitignore()?;

                Ok(ImportResult {
                    source_path: self.source_path,
                    target_layer: self.target_layer,
                    hash,
                    size: content.len(),
                    success: true,
                })
            }
            Err(e) => {
                txn.rollback()?;
                Err(e.into())
            }
        }
    }
}
```

### 5.2 Export Workflow (Jin → Git)

**Pattern for reversing import:**

```rust
struct ExportOperation {
    source_hash: Sha1Hash,
    source_layer: String,
    target_path: PathBuf,
    restore_git_history: bool,
}

impl ExportOperation {
    fn execute(self) -> Result<ExportResult> {
        // Phase 1: Validation
        let object = self.retrieve_object()?;
        self.validate_target_path()?;

        // Phase 2: Preparation
        let metadata = self.retrieve_metadata()?;

        // Phase 3: Atomic transaction
        let txn = ExportTransaction::begin()?;

        match (|| {
            // Write content to working directory
            fs::create_dir_all(self.target_path.parent().unwrap())?;
            fs::write(&self.target_path, &object.data)?;

            // Restore file metadata
            self.restore_metadata(&self.target_path, &metadata)?;

            // Add to git tracking
            txn.git_add(&self.target_path)?;

            // Log operation
            txn.log_operation(&ExportLog {
                timestamp: SystemTime::now(),
                source_hash: self.source_hash.clone(),
                source_layer: self.source_layer.clone(),
                target_path: self.target_path.clone(),
                size: object.data.len(),
            })?;

            Ok(())
        })() {
            Ok(_) => {
                txn.commit()?;

                // Remove from .gitignore if present
                self.remove_from_gitignore()?;

                Ok(ExportResult {
                    source_hash: self.source_hash,
                    target_path: self.target_path,
                    size: object.data.len(),
                    success: true,
                })
            }
            Err(e) => {
                txn.rollback()?;
                Err(e.into())
            }
        }
    }
}
```

### 5.3 Batch Operations

**Pattern for multiple file migrations:**

```rust
struct BatchMigration {
    operations: Vec<MigrationOp>,
}

impl BatchMigration {
    fn execute(self) -> Result<BatchReport> {
        let mut report = BatchReport::new();

        // Validate all operations first
        for op in &self.operations {
            op.validate()?;
        }

        // Execute with per-operation atomicity
        for op in self.operations {
            match op.execute() {
                Ok(result) => {
                    report.add_success(result);
                }
                Err(e) => {
                    report.add_error(op, e);
                    // Continue with remaining operations
                    // Later: implement optional --fail-fast
                }
            }
        }

        Ok(report)
    }
}
```

---

## 6. Atomic Operations and Error Recovery

### 6.1 Write-Ahead Logging Pattern

**Source**: [Write-Ahead Logging](https://sqlite.org/wal.html) and [ARIES Recovery](https://dl.acm.org/doi/10.1145/128765.128770)

Implement safe file operations with recovery capability:

```rust
struct TransactionLog {
    log_file: PathBuf,
    entries: Vec<LogEntry>,
}

#[derive(Serialize, Deserialize)]
struct LogEntry {
    id: u64,
    timestamp: SystemTime,
    operation: OperationType,
    status: LogStatus,
    details: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
enum LogStatus {
    Started,
    Committed,
    RolledBack,
    Failed(String),
}

impl TransactionLog {
    fn record_start(&mut self, op: &OperationType) -> Result<u64> {
        let entry = LogEntry {
            id: self.next_id(),
            timestamp: SystemTime::now(),
            operation: op.clone(),
            status: LogStatus::Started,
            details: serde_json::json!({}),
        };

        self.entries.push(entry.clone());
        self.flush_to_disk()?;

        Ok(entry.id)
    }

    fn record_commit(&mut self, id: u64) -> Result<()> {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.status = LogStatus::Committed;
            self.flush_to_disk()?;
        }
        Ok(())
    }

    fn record_rollback(&mut self, id: u64) -> Result<()> {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.id == id) {
            entry.status = LogStatus::RolledBack;
            self.flush_to_disk()?;
        }
        Ok(())
    }

    fn recover_failed_operations(&self) -> Result<Vec<LogEntry>> {
        let failed = self.entries.iter()
            .filter(|e| matches!(e.status, LogStatus::Started | LogStatus::Failed(_)))
            .cloned()
            .collect();

        Ok(failed)
    }
}
```

### 6.2 Atomic File Operations

**Source**: [atomicwrites GitHub](https://github.com/untitaker/rust-atomicwrites) and [atomic-file crate](https://crates.io/crates/atomic-file)

Pattern for safe file writes:

```rust
use std::fs::File;
use std::io::Write;

struct AtomicFileWrite {
    temp_path: PathBuf,
    target_path: PathBuf,
}

impl AtomicFileWrite {
    fn new(target_path: PathBuf) -> Result<Self> {
        let temp_path = target_path.with_extension(".tmp");

        Ok(AtomicFileWrite {
            temp_path,
            target_path,
        })
    }

    fn write<F>(&self, write_fn: F) -> Result<()>
    where
        F: FnOnce(&mut File) -> std::io::Result<()>,
    {
        // Step 1: Write to temporary file
        let mut temp_file = File::create(&self.temp_path)?;
        write_fn(&mut temp_file)?;
        temp_file.sync_all()?;  // Ensure data is on disk

        // Step 2: Atomic rename
        // On POSIX: atomic replace
        // On Windows: two-phase (delete then rename)
        #[cfg(unix)]
        std::fs::rename(&self.temp_path, &self.target_path)?;

        #[cfg(windows)]
        {
            if self.target_path.exists() {
                std::fs::remove_file(&self.target_path)?;
            }
            std::fs::rename(&self.temp_path, &self.target_path)?;
        }

        Ok(())
    }
}

// Usage
fn save_layer_index(layer: &str, index: &LayerIndex) -> Result<()> {
    let target_path = PathBuf::from(format!(".jin/layers/{}/index", layer));
    let atomic = AtomicFileWrite::new(target_path)?;

    atomic.write(|file| {
        let serialized = serde_json::to_vec(index)?;
        file.write_all(&serialized)
    })?;

    Ok(())
}
```

### 6.3 Transaction Pattern

**Source**: [Rust Error Handling - The Rust Programming Language](https://doc.rust-lang.org/book/ch09-00-error-handling.html)

```rust
struct Transaction {
    id: u64,
    log_entry: Option<u64>,
    temp_files: Vec<PathBuf>,
    changes: Vec<Change>,
}

impl Transaction {
    fn begin() -> Result<Self> {
        let log_entry = TransactionLog::record_start(&Operation::FileOp)?;

        Ok(Transaction {
            id: generate_uuid(),
            log_entry: Some(log_entry),
            temp_files: Vec::new(),
            changes: Vec::new(),
        })
    }

    fn add_change(&mut self, change: Change) {
        self.changes.push(change);
    }

    fn commit(mut self) -> Result<()> {
        // Apply all changes
        for change in self.changes {
            change.apply()?;
        }

        // Log commitment
        if let Some(log_id) = self.log_entry {
            TransactionLog::record_commit(log_id)?;
        }

        // Cleanup temp files
        self.cleanup()?;

        Ok(())
    }

    fn rollback(mut self) -> Result<()> {
        // Reverse changes (if implemented)
        for change in &self.changes {
            change.revert()?;
        }

        // Log rollback
        if let Some(log_id) = self.log_entry {
            TransactionLog::record_rollback(log_id)?;
        }

        // Cleanup temp files
        self.cleanup()?;

        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        for temp_file in &self.temp_files {
            if temp_file.exists() {
                std::fs::remove_file(temp_file)?;
            }
        }
        Ok(())
    }
}

// Drop implementation for automatic rollback
impl Drop for Transaction {
    fn drop(&mut self) {
        // Auto-rollback if not explicitly committed
        // This is safe due to Rust's RAII
        if self.log_entry.is_some() {
            let _ = self.rollback();
        }
    }
}
```

### 6.4 Error Recovery Patterns

**Source**: [Rust Error Propagation](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)

Use the `?` operator for clean error propagation:

```rust
fn import_operation(path: &Path) -> Result<ImportResult> {
    // Error propagates up on ?
    let content = fs::read(path)?;
    let metadata = fs::metadata(path)?;
    let hash = hash_object(&content)?;

    let mut txn = Transaction::begin()?;

    store_object(&hash, &content)?;
    txn.add_change(Change::StoreObject {
        hash: hash.clone(),
        content: content.clone(),
    });

    update_index(&hash, path, &metadata)?;
    txn.add_change(Change::UpdateIndex {
        path: path.to_path_buf(),
        hash: hash.clone(),
    });

    // Auto-commit on success, auto-rollback on error
    txn.commit()?;

    Ok(ImportResult { hash, size: content.len() })
}
```

---

## 7. Metadata Preservation Strategies

### 7.1 File Metadata Tracking

**Source**: [Git File Permissions Documentation](https://www.baeldung.com/linux/git-ignore-file-mode) and [Metadata Migration](https://www.cloudfuze.com/metadata-migration)

Important file attributes to preserve:

```rust
#[derive(Clone, Serialize, Deserialize)]
struct FileMetadata {
    // Permissions
    mode: u32,                  // Unix permissions (0o755, 0o644, etc.)
    executable: bool,          // Is file executable?
    symlink_target: Option<String>,  // If symlink, where does it point?

    // Timestamps
    created_at: SystemTime,
    modified_at: SystemTime,
    accessed_at: SystemTime,

    // File info
    size: u64,
    hash: Sha1Hash,            // Content hash

    // Optional extended attributes
    owner: Option<String>,     // Original owner
    group: Option<String>,     // Original group

    // Version control metadata
    git_mode: GitFileMode,     // 100644, 100755, 120000
    tracked_since: SystemTime, // When first tracked
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
enum GitFileMode {
    RegularFile,      // 100644
    Executable,       // 100755
    SymbolicLink,     // 120000
}

impl GitFileMode {
    fn as_octal(&self) -> u32 {
        match self {
            GitFileMode::RegularFile => 0o100644,
            GitFileMode::Executable => 0o100755,
            GitFileMode::SymbolicLink => 0o120000,
        }
    }
}
```

### 7.2 Extracting Metadata from Git

**Pattern for reading file metadata during import:**

```rust
fn extract_git_metadata(git_path: &Path) -> Result<FileMetadata> {
    let fs_metadata = fs::metadata(git_path)?;
    let git_mode = get_git_file_mode(git_path)?;
    let hash = get_git_object_hash(git_path)?;

    Ok(FileMetadata {
        mode: {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs_metadata.permissions().mode()
            }
            #[cfg(not(unix))]
            {
                0o644  // Default for non-Unix
            }
        },
        executable: {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs_metadata.permissions().mode() & 0o111 != 0
            }
            #[cfg(not(unix))]
            {
                false
            }
        },
        symlink_target: if fs_metadata.is_symlink() {
            Some(fs::read_link(git_path)?.to_string_lossy().to_string())
        } else {
            None
        },
        created_at: fs_metadata.created()?,
        modified_at: fs_metadata.modified()?,
        accessed_at: fs_metadata.accessed()?,
        size: fs_metadata.len(),
        hash: hash.clone(),
        git_mode,
        tracked_since: SystemTime::now(),
        owner: None,    // Optional: extract with nix crate
        group: None,    // Optional: extract with nix crate
    })
}

fn get_git_file_mode(path: &Path) -> Result<GitFileMode> {
    // Query git index for file mode
    let output = Command::new("git")
        .args(&["ls-files", "--stage", path.to_str().unwrap()])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;

    // Parse: "100644 hash 0 path"
    for line in stdout.lines() {
        if let Some(mode_str) = line.split_whitespace().next() {
            return match mode_str {
                "100644" => Ok(GitFileMode::RegularFile),
                "100755" => Ok(GitFileMode::Executable),
                "120000" => Ok(GitFileMode::SymbolicLink),
                _ => Err("Unknown git file mode".into()),
            };
        }
    }

    Err("File not found in git index".into())
}
```

### 7.3 Restoring Metadata After Export

**Pattern for applying metadata when exporting to git:**

```rust
fn restore_metadata(path: &Path, metadata: &FileMetadata) -> Result<()> {
    // Restore timestamps
    filetime::set_file_mtime(
        path,
        filetime::FileTime::from_system_time(metadata.modified_at),
    )?;

    // Restore permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(metadata.mode);
        fs::set_permissions(path, perms)?;
    }

    // Restore symlink target (would need to recreate symlink)
    if let Some(ref target) = metadata.symlink_target {
        // Note: Must delete and recreate symlink
        if path.exists() {
            fs::remove_file(path)?;
        }
        std::os::unix::fs::symlink(target, path)?;
    }

    Ok(())
}
```

### 7.4 PREMIS Metadata Standard

**Pattern for capturing provenance information:**

```rust
#[derive(Serialize, Deserialize)]
struct PremisMetadata {
    // Digital object identification
    object_id: String,
    object_type: String,

    // Provenance
    original_location: String,
    import_timestamp: SystemTime,
    imported_by: String,
    import_method: String,  // "git-import", "direct-upload", etc.

    // Authenticity
    checksum_before: String,
    checksum_after: String,
    checksum_algorithm: String,  // "sha1", "sha256"

    // Transformation events
    events: Vec<TransformationEvent>,
}

#[derive(Serialize, Deserialize)]
struct TransformationEvent {
    event_type: String,     // "migration", "compression", etc.
    event_date: SystemTime,
    event_outcome: String,  // "success", "failure"
    details: String,
}
```

---

## 8. Rust Patterns for File System Operations

### 8.1 Safe File System API Design

**Source**: [std::fs Documentation](https://doc.rust-lang.org/std/fs/)

```rust
// Create directory safely
fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

// Write file atomically
fn write_file(path: &Path, content: &[u8]) -> Result<()> {
    let temp_path = path.with_extension(".tmp");

    fs::write(&temp_path, content)?;

    // Atomic rename on POSIX, safe replace on Windows
    #[cfg(unix)]
    fs::rename(&temp_path, path)?;

    #[cfg(windows)]
    {
        if path.exists() {
            fs::remove_file(path)?;
        }
        fs::rename(&temp_path, path)?;
    }

    Ok(())
}

// Read file with error context
fn read_file(path: &Path) -> Result<Vec<u8>> {
    fs::read(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e).into())
}

// Safe directory traversal
fn walk_directory<F>(path: &Path, callback: F) -> Result<()>
where
    F: Fn(&Path) -> Result<()>,
{
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            callback(&path)?;
        } else if path.is_dir() {
            walk_directory(&path, &callback)?;
        }
    }

    Ok(())
}
```

### 8.2 Error Handling Patterns

**Source**: [Mastering Rust Error Handling](https://medium.com/@Murtza/error-handling-best-practices-in-rust-a-comprehensive-guide-to-building-resilient-applications-46bdf6fa6d9d)

```rust
#[derive(Debug)]
pub enum JinError {
    IoError(std::io::Error),
    SerializationError(serde_json::Error),
    GitError(String),
    InvalidOperation(String),
    TransactionFailed(String),
}

impl From<std::io::Error> for JinError {
    fn from(err: std::io::Error) -> Self {
        JinError::IoError(err)
    }
}

impl From<serde_json::Error> for JinError {
    fn from(err: serde_json::Error) -> Self {
        JinError::SerializationError(err)
    }
}

impl From<String> for JinError {
    fn from(err: String) -> Self {
        JinError::InvalidOperation(err)
    }
}

// Usage with ? operator
fn operation() -> Result<(), JinError> {
    let content = fs::read("file.txt")?;  // Converts io::Error
    let data = serde_json::from_slice(&content)?;  // Converts serde Error

    Ok(())
}

// Custom error context
fn contextualize<T, E: std::fmt::Display>(
    result: Result<T, E>,
    context: &str,
) -> Result<T, String> {
    result.map_err(|e| format!("{}: {}", context, e))
}

// Usage
let content = contextualize(
    fs::read("config.json"),
    "Failed to read configuration",
)?;
```

### 8.3 Result Type Extensions

```rust
trait ResultExt<T> {
    fn context(self, msg: &str) -> Result<T>;
    fn recover_with<F: FnOnce() -> T>(self, f: F) -> T;
}

impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn context(self, msg: &str) -> Result<T> {
        self.map_err(|e| format!("{}: {}", msg, e).into())
    }

    fn recover_with<F: FnOnce() -> T>(self, f: F) -> T {
        self.unwrap_or_else(|_| f())
    }
}

// Usage
let config = parse_config("config.json")
    .context("Failed to parse configuration")?;

let fallback = get_setting("key")
    .recover_with(|| default_value());
```

---

## 9. Implementation Recommendations

### 9.1 Context Display Command

```bash
jin context [--short] [--format json|text]

# Output:
# Active Mode: feature-branch (refs/modes/feature-branch)
# Scope:       local (.jin/config)
# Layers:      [base] [feature-new] [experimental]

# With --short:
# feature-branch | local | 3 layers

# With --format json:
# {"mode": "feature-branch", "scope": "local", "layers": [...]}
```

### 9.2 Import Command

```bash
jin import <file> [--to-layer LAYER] [--preserve-history]

# Workflow:
# 1. Validate file is tracked in git
# 2. Read content and metadata
# 3. Hash content
# 4. Store in jin objects
# 5. Update layer index (atomic)
# 6. Remove from git tracking
# 7. Add to .gitignore
# 8. Commit changes

# Batch import:
jin import --from-git '**/*.mp4' --to-layer media --preserve-history
```

### 9.3 Export Command

```bash
jin export <hash> [--to-path PATH] [--restore-git-history]

# Workflow:
# 1. Verify hash exists in jin
# 2. Retrieve content and metadata
# 3. Write to filesystem (atomic)
# 4. Restore file metadata
# 5. Add to git tracking
# 6. Remove from .gitignore
# 7. Commit changes

# Export all from layer:
jin export --from-layer media --to-git
```

### 9.4 Configuration Files

**`.jin/config` (Local scope):**
```toml
[core]
    mode = "feature-branch"
    scope = "local"

[layers]
    active = ["base", "feature-new"]
    composition = "merge"

[import]
    default_layer = "working"
    preserve_history = true

[export]
    restore_permissions = true
    restore_timestamps = true
```

**`~/.jin/config` (Global scope):**
```toml
[user]
    name = "Developer"
    email = "dev@example.com"

[core]
    editor = "vim"
    pager = "less"

[import]
    default_preserve_history = true
    verify_checksums = true
```

### 9.5 Error Recovery Checklist

- [ ] Write-ahead logging for all mutations
- [ ] Atomic file operations with temp files
- [ ] Transaction pattern with auto-rollback
- [ ] Graceful error propagation with `?` operator
- [ ] Detailed error messages with context
- [ ] Recovery procedures for interrupted operations
- [ ] Consistency verification after failures
- [ ] Cleanup of temporary files on error

---

## References

### Git Architecture and Commands

1. [Git - Plumbing and Porcelain](https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain) - Comprehensive overview of Git's architecture separating low-level and user-friendly commands

2. [Git - Recording Changes to the Repository](https://git-scm.com/book/en/v2/Git-Basics-Recording-Changes-to-the-Repository) - File tracking lifecycle and staging mechanics

3. [Git - Git Objects](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects) - Content-addressable storage model with blobs, trees, and commits

4. [Git - git-symbolic-ref Documentation](https://git-scm.com/docs/git-symbolic-ref) - Reading and writing symbolic references for active branch tracking

5. [Git - git-config Documentation](https://git-scm.com/docs/git-config) - Configuration scope hierarchy and priority

6. [Git - git-ls-files Documentation](https://git-scm.com/docs/git-ls-files) - Index inspection with metadata and formatting options

7. [Git - git-fast-import Documentation](https://git-scm.com/docs/git-fast-import) - High-performance data import format for repository migration

### File Operations and Workflows

8. [How to Stop Tracking File in Git](https://www.delftstack.com/howto/git/git-stop-tracking-file/) - Safe file untracking patterns using `git rm --cached`

9. [Git LFS Tracking, Migration and Un-Migration](https://mslinn.com/git/5300-git-lfs-patterns-tracking.html) - Large file migration patterns applicable to jin

10. [Git File Permissions](https://www.baeldung.com/linux/git-ignore-file-mode) - Tracking and ignoring permission changes

### Rust Implementation Patterns

11. [atomicwrites GitHub Repository](https://github.com/untitaker/rust-atomicwrites) - Atomic file write implementation for POSIX and Windows

12. [atomic-file crate](https://crates.io/crates/atomic-file) - ACID-compliant file operations with versioning

13. [Rust Error Handling](https://doc.rust-lang.org/book/ch09-00-error-handling.html) - Result/Option types and error propagation

14. [Rust Error Propagation with `?` Operator](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html) - Clean error handling patterns

15. [Transaction Management in Rust: SQLx and Diesel](https://softwarepatternslexicon.com/patterns-rust/11/10/) - Transaction patterns with automatic rollback

### Database and Transaction Patterns

16. [Write-Ahead Logging](https://sqlite.org/wal.html) - WAL pattern for atomicity and crash recovery

17. [ARIES Recovery Algorithm](https://dl.acm.org/doi/10.1145/128765.128770) - Fine-grained recovery with partial rollback support

18. [Atomicity in Databases](https://www.datacamp.com/tutorial/atomicity) - ACID properties and atomic transaction implementation

### Metadata and Preservation

19. [Metadata Migration Strategies](https://www.cloudfuze.com/metadata-migration) - Best practices for preserving metadata during transfer

20. [PREMIS Digital Preservation Metadata](https://www.iri.com/blog/iri/iri-workbench/submission-version-control/) - Capturing provenance and authenticity information

---

## Document Metadata

- **Created**: 2025-12-27
- **Version**: 1.0
- **Focus Areas**: Context display, import/export workflows, atomic operations, metadata preservation
- **Research Coverage**: 20+ authoritative sources
- **Key Concepts**: 100+ technical patterns and implementations

---

## Related Documentation

- P4M5 Plan: Comprehensive mode, scope, and layer management
- P4M4 Research: CLI framework and documentation patterns
- Git Documentation: Official Git manual and Pro Git book
- Rust Programming Language: Official documentation and error handling guides
