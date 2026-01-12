# Context, Import, Export Research - Quick Summary

## Document Location
**File**: `/home/dustin/projects/jin/plan/P4M5/research/context_import_export_research.md`

**Size**: 42 KB, 1588 lines

**Status**: Complete with 20+ reference sources

---

## What Was Researched

### 1. Context Display Patterns
How to show the currently active mode, scope, and layer composition (like git shows the active branch).

**Key Patterns**:
- Symbolic references for tracking active state
- Configuration hierarchy (command → worktree → local → global → system)
- Layer stack visualization
- Configuration value source tracking

**Git Commands**:
- `git symbolic-ref HEAD` - get current branch reference
- `git symbolic-ref --short HEAD` - get branch name only
- `git config --show-origin` - show where config comes from

**Implementation**: Rust structure for context display with scope tracking

---

### 2. Git Porcelain and Plumbing Commands
Understanding git's architecture for file operations and how jin can use similar patterns.

**Porcelain Commands** (user-friendly):
- `git add`, `git commit`, `git checkout`, `git push`, `git pull`

**Plumbing Commands** (low-level, scriptable):
- `git hash-object` - create blob from content
- `git update-index` - directly manipulate staging area
- `git write-tree` - create tree from index
- `git commit-tree` - create commit from tree
- `git cat-file` - inspect git objects
- `git ls-files` - list files in index
- `git fast-import` / `git fast-export` - bulk data transfer

**Object Model**: Blobs (content) → Trees (directories) → Commits (snapshots) → Tags (references)

---

### 3. File Tracking and Untracking Workflows
How to safely move files between git and jin tracking systems.

**File States**:
```
Untracked → Staged → Committed
    ↑          ↓
    └─ Modified (unstaged)
```

**Safe Untracking**:
- `git rm --cached file` - remove from staging, keep local copy
- Update `.gitignore` to prevent re-tracking
- Commit the changes

**Large File Migration** (LFS pattern):
- Enable tracking for file types
- Migrate existing files
- Export to convert pointers back to files

---

### 4. Programmatic Index Manipulation
Reading and writing the git index for programmatic file operations.

**Index Entry Structure**:
- Creation/modification times
- File mode (100644, 100755, 120000)
- Content hash (SHA1)
- File path
- Merge stage (for conflict resolution)

**Reading Index**:
- `git ls-files --cached` - list tracked files
- `git ls-files --stage` - show mode, hash, stage, path
- `git ls-files --modified` - show unstaged changes
- `git ls-files --format` - custom formatting

**Updating Index**:
- `git update-index --add --cacheinfo MODE HASH PATH` - add with hash
- `git update-index --remove FILE` - remove file
- `git hash-object -w FILE` - hash and store content

---

### 5. Import/Export Mechanisms
Complete workflows for moving files between git and jin tracking.

**Import (Git → Jin)**:
1. Validate source is tracked in git
2. Read content and metadata
3. Hash content
4. Store in jin objects (atomic)
5. Update layer index (atomic)
6. Remove from git tracking
7. Add to .gitignore
8. Commit changes

**Export (Jin → Git)**:
1. Verify hash exists in jin
2. Retrieve content and metadata
3. Write to filesystem (atomic)
4. Restore file metadata
5. Add to git tracking
6. Remove from .gitignore
7. Commit changes

**Batch Operations**: Support for multiple files with per-operation error handling

---

### 6. Atomic Operations and Error Recovery
Ensuring data integrity and safe recovery from failures.

**Write-Ahead Logging (WAL)**:
- Record operations before executing
- Enable recovery of interrupted operations
- Track operation status (Started, Committed, RolledBack)

**Atomic File Operations**:
- Write to temporary file in same filesystem
- Atomic rename when complete (POSIX) or safe replace (Windows)
- Prevents corruption from crashes

**Transaction Pattern**:
```rust
let txn = Transaction::begin()?;  // Log operation start
txn.add_change(change);            // Queue changes
txn.commit()?;                    // Atomically apply and log commit
// On error: auto-rollback via Drop impl
```

**Error Propagation**:
- Use `?` operator for clean error handling
- Custom error types with context
- Graceful recovery with logging

---

### 7. Metadata Preservation
Maintaining file attributes through import/export.

**File Metadata to Preserve**:
- Unix permissions (0o755, 0o644)
- Executable flag
- Symbolic link targets
- Timestamps (created, modified, accessed)
- File size
- Git file mode (100644, 100755, 120000)
- Ownership (optional, with nix crate)

**Extraction**:
- Read from filesystem metadata
- Query git for file mode
- Extract extended attributes

**Restoration**:
- Set permissions with `fs::set_permissions`
- Restore timestamps with `filetime` crate
- Recreate symlinks

**PREMIS Metadata**:
- Digital object ID and type
- Provenance (original location, import method)
- Authenticity (checksums before/after)
- Transformation events

---

### 8. Rust Patterns for File System Operations
Safe and idiomatic Rust patterns for file operations.

**Safe File API**:
- `ensure_dir()` - create directory if not exists
- `write_file()` - atomic write with temp file
- `read_file()` - read with error context
- `walk_directory()` - recursive traversal with callbacks

**Error Handling**:
- Result<T, E> type for recoverable errors
- Custom error enum with From implementations
- Context extension trait for better error messages
- Recovery patterns with unwrap_or_else

**Error Propagation**:
- `?` operator converts errors up the stack
- `From` trait for type conversion
- `Result` and `Option` both support `?`

**Result Extensions**:
- `.context(msg)` - add error context
- `.recover_with(fn)` - provide fallback value
- `.map_err()` - transform error type

---

## Key Takeaways

### Context Display
- Use symbolic refs to track active mode
- Implement config hierarchy for flexibility
- Show scope level and configuration sources
- Display layer composition as stack

### Import Workflow
- 8-step process with atomic transactions
- Write-ahead logging for recovery
- Safe removal from git tracking
- Metadata preservation throughout

### Export Workflow
- 7-step reverse process
- Atomic file writes to working directory
- Metadata restoration
- Re-integration with git

### Safety Patterns
- Atomic file operations (write temp, rename)
- Transaction logging for recovery
- Error propagation with `?` operator
- Automatic rollback on error (Drop impl)

### Metadata
- Track permissions, times, mode bits
- Preserve git file mode (100644/100755/120000)
- Optional: extended attributes, ownership
- PREMIS metadata for provenance

---

## Implementation Checklist

### Context Display Command
- [ ] Read symbolic refs for active mode/scope
- [ ] Query configuration hierarchy
- [ ] Format context display (text/json/short)
- [ ] Show error for detached HEAD-like state

### Import Command
- [ ] Validate source in git
- [ ] Read content and metadata
- [ ] Hash with blob format: "blob SIZE\0CONTENT"
- [ ] Store in `.jin/objects/XX/XXXXXXX...`
- [ ] Update layer index atomically
- [ ] Execute git rm --cached
- [ ] Add to .gitignore
- [ ] Commit changes
- [ ] Support batch operations

### Export Command
- [ ] Verify hash in jin objects
- [ ] Retrieve content and metadata
- [ ] Write atomically to workspace
- [ ] Restore file permissions/timestamps
- [ ] Execute git add
- [ ] Remove from .gitignore
- [ ] Commit changes

### Error Recovery
- [ ] Write-ahead logging for all mutations
- [ ] Transaction log with operation tracking
- [ ] Automatic rollback on panic/error
- [ ] Recovery procedure for interrupted operations
- [ ] Cleanup of temporary files

---

## Related Resources

**In Research Directory**:
- `diff_research.md` - Diff algorithm patterns
- `log_research.md` - Log command patterns
- `repair_layers_list_research.md` - Layer management
- `RESEARCH_INDEX.md` - Master research index

**External References**: 20 authoritative sources linked in main document

**Git Documentation**:
- Pro Git book chapters on internals
- Official git-plumbing documentation
- Git configuration system
- Fast import/export format

**Rust Resources**:
- atomicwrites crate for atomic operations
- atomic-file crate for ACID compliance
- Standard library fs module
- Error handling patterns in Rust book

---

## Next Steps

1. **Reference Implementation**: Use research patterns to implement jin commands
2. **Test Cases**: Create tests for atomic operations and error recovery
3. **Performance**: Benchmark fast-import format vs other serialization
4. **Documentation**: User guide for import/export workflows
5. **Integration**: Connect with existing jin mode/scope/layer system

---

**Document Date**: 2025-12-27  
**Research Scope**: Complete  
**Sources Reviewed**: 20+  
**Code Examples**: 50+  
**Implementation Patterns**: 100+
