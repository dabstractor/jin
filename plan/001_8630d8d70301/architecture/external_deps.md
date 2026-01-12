# Jin External Dependencies Analysis

**Date:** 2026-01-03
**Scope:** Analysis of external crates and their capabilities for implementing missing features

---

## Executive Summary

The Jin project uses a solid foundation of Rust crates with git2 for Git operations and diffy for text merging. However, implementing the PRD-mandated features requires custom implementations for:
1. Layer-aware conflict markers (diffy doesn't support custom labels)
2. Detached workspace state detection (custom logic required)
3. Fetch-before-push validation (git2 provides all needed capabilities)

---

## 1. Current Dependencies Analysis

### 1.1 Git Operations: `git2` (v0.19)

**Current Usage:**
- Repository management in `src/git/repo.rs`
- Ref operations in `src/git/transaction.rs`
- Remote operations in `src/commands/{fetch,pull,push}.rs`

**Capabilities:**
✅ Remote fetch with reference discovery
✅ Push with force flag support
✅ Reference comparison (local vs remote)
✅ Authentication handling
✅ Merge analysis (fast-forward vs divergent)
✅ Commit ancestry checking

**Required for Gap #2 (Fetch-before-push):**
```rust
use git2::{Repository, BranchType, Error};

// Check if local is behind remote
fn is_behind_remote(repo: &Repository, local_ref: &str, remote_ref: &str) -> Result<bool, Error> {
    let local_obj = repo.revparse_single(local_ref)?;
    let remote_obj = repo.revparse_single(remote_ref)?;

    // git2 provides graph operations
    Ok(repo.graph_descendant_of(remote_obj.id(), local_obj.id())?)
}
```

**Assessment:** git2 has **full capabilities** for fetch-before-push enforcement. No additional dependencies needed.

---

### 1.2 Text Merging: `diffy` (v0.4)

**Current Usage:**
- Conflict detection in `src/merge/text.rs`
- Generic conflict markers with `ours`/`theirs` labels

**Capabilities from crate documentation:**
```rust
// diffy only supports standard Git markers:
pub fn apply_merge<T: AsRef<str>>(
    original: T,
    left: T,
    right: T
) -> Result<Merge, MergeError>

// Markers are always:
// <<<<<<< ours
// -------
// >>>>>>> theirs
```

**Limitations for Gap #1 (.jinmerge):**
❌ **Cannot support custom layer path labels**
❌ Markers are hardcoded to `ours`/`theirs`
❌ No API for custom marker labels
❌ No multi-way conflict support (only 2-way)

**Required Implementation:**
```rust
// Custom implementation needed in src/merge/jinmerge.rs:
pub struct JinMergeMarker {
    pub layer1_ref: String,  // e.g., "mode/claude/scope/language:javascript/"
    pub layer2_ref: String,  // e.g., "mode/claude/project/ui-dashboard/"
    pub content_conflict: String,
}

impl JinMergeMarker {
    pub fn to_git_format(&self) -> String {
        format!(
            "<<<<<<< {}\n{}\n=======\n{}\n>>>>>> {}\n",
            self.layer1_ref,
            self.layer1_content,
            self.layer2_content,
            self.layer2_ref
        )
    }
}
```

**Assessment:** diffy is **insufficient** for .jinmerge format. Custom implementation required.

---

### 1.3 Serialization Crates

**JSON:** `serde_json` (v1.0)
- Used for: Staging index, JinMap, audit logs
- Capabilities: Full serde support, parsing, serialization
- Assessment: ✅ Fully capable

**YAML:** `serde_yaml` (v0.9)
- Used for: JinMap storage, configuration
- Capabilities: Full serde support
- Assessment: ✅ Fully capable

**TOML:** `toml` (v0.8)
- Used for: Configuration files, layer metadata
- Capabilities: Full serde support
- Assessment: ✅ Fully capable

**INI:** `rust-ini` (v0.21)
- Used for: INI file merging
- Capabilities: Parser and serialization
- Assessment: ✅ Fully capable

---

### 1.4 Error Handling: `thiserror` (v2.0) + `anyhow` (v1.0)

**Current Usage:**
- JinError enum in `src/core/error.rs`
- Error propagation throughout codebase

**Capabilities:**
✅ Custom error enums with context
✅ Error downcasting
✅ Stack trace preservation

**Required for Gap #3 (Detached Workspace):**
```rust
// Add to JinError enum in src/core/error.rs:
#[error("workspace is in a detached state")]
DetachedWorkspace {
    workspace_commit: Option<String>,
    expected_layer_ref: String,
    recovery_hint: String,
}
```

**Assessment:** thiserror fully supports custom error types. No changes needed.

---

### 1.5 Data Structures: `indexmap` (v2.0)

**Current Usage:**
- Ordered HashMap for layer contents
- Merge operations requiring key ordering

**Capabilities:**
✅ Insertion-ordered HashMap
✅ Serde integration
✅ All standard HashMap operations

**Assessment:** ✅ Fully capable for all needs

---

### 1.6 CLI: `clap` (v4.5)

**Current Usage:**
- All CLI command definitions in `src/cli/mod.rs`
- Argument parsing and validation

**Capabilities:**
✅ Derive macros for commands
✅ Subcommand support
✅ Argument validation
✅ Auto-generated help text

**Required for new commands:**
```rust
// Add to src/cli/mod.rs for Gap #1 (resolve command):
#[derive(Parser, Debug)]
pub struct ResolveArgs {
    /// Paths to resolved conflict files
    pub paths: Vec<PathBuf>,

    /// Mark all conflicts as resolved
    #[arg(short, long)]
    pub all: bool,
}
```

**Assessment:** ✅ clap supports all required command structures

---

## 2. Alternative Crates Considered

### 2.1 Advanced Text Merging Crates

**`threeway_merge`** (Not currently used)
- Purpose: Git-compatible 3-way merge
- Capabilities: Multiple merge algorithms (mine, theirs, union)
- Status: Mature, maintained
- **Recommendation:** Consider for Gap #4 (3-way merge in pull)

```rust
use threeway_merge::{Merge, MergeStrategy};

// For Gap #4 implementation:
fn merge_layers_three_way(
    base: &str,
    layer1: &str,
    layer2: &str
) -> Result<String, MergeError> {
    Merge::new(base, layer1, layer2)
        .with_strategy(MergeStrategy::Git)
        .merge()
}
```

**`mergiraf`** (Not currently used)
- Purpose: Syntax-aware merging for code
- Capabilities: AST-based merging, better conflict resolution
- Status: Research-level, less stable
- **Recommendation:** Consider for future enhancement

**Decision:** Use existing `src/merge/text.rs` for Gap #4. Consider `threeway_merge` if current implementation insufficient.

---

### 2.2 Git Operations Alternatives

**`gitoxide`** (Not currently used)
- Purpose: Pure Rust Git implementation
- Capabilities: Faster than git2, more idiomatic Rust
- Status: Maturing, not yet feature-complete
- **Recommendation:** Keep git2 for now, consider migration in future

**Decision:** **Stay with git2** - mature, fully-featured, well-documented.

---

## 3. Research Questions Answered

### Q1: Can diffy support custom conflict markers?

**Answer:** **No.**

The `diffy` crate hardcodes conflict markers to `ours`/`theirs`. There is no API for custom labels.

**Evidence from diffy source:**
```rust
// diffy/src/merge.rs (simplified)
pub enum Merge<'input> {
    Conflict {
        left: &'input str,
        right: &'input str,
        // Labels are hardcoded
    }
}

impl<'input> Display for Merge<'input> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Conflict { left, right } => {
                write!(f, "<<<<<<< ours\n{}\n=======\n{}\n>>>>>>>\n", left, right)
            }
        }
    }
}
```

**Required Solution:** Custom implementation in `src/merge/jinmerge.rs` for layer-aware markers.

---

### Q2: What's the standard approach for detached workspace detection?

**Answer:** **Validate workspace state before Git operations.**

Best practices from version control systems:

1. **Check HEAD reference type:**
   ```rust
   // In Git, detached HEAD is when HEAD points to commit SHA, not branch
   fn is_detached(repo: &Repository) -> bool {
       repo.head().ok()
           .and_then(|head| head.name())
           .map(|name| !name.starts_with("refs/heads/"))
           .unwrap_or(true)
   }
   ```

2. **Validate workspace contents match expected state:**
   ```rust
   // For Jin's workspace:
   fn validate_workspace_attached(workspace: &Workspace, expected_layer: &LayerRef) -> Result<()> {
       let workspace_commit = workspace.current_commit()?;
       let layer_commit = expected_layer.peel_to_commit()?;

       if workspace_commit != layer_commit {
           return Err(JinError::DetachedWorkspace { ... });
       }

       Ok(())
   }
   ```

3. **Provide recovery options:**
   - `jin reset --hard <layer-ref>` - Reattach workspace
   - `jin repair --check` - Detect and report issues
   - Error messages with recovery hints

**Assessment:** Custom implementation required. No existing crate provides this specific functionality.

---

### Q3: How do other tools handle 3-way merge for non-fast-forward updates?

**Answer:** **Use Git's merge algorithms or custom text merging.**

**Approach 1: Git-native merge (git2)**
```rust
use git2::{MergeOptions, AnnotatedCommit};

// For Gap #4 implementation in pull command:
fn merge_layer_update(
    repo: &Repository,
    local_ref: &str,
    remote_ref: &str
) -> Result<()> {
    let local_commit = repo.find_reference(local_ref)?
        .peel_to_commit()?;
    let remote_commit = repo.find_reference(remote_ref)?
        .peel_to_commit()?;

    // Find merge base
    let merge_base = repo.merge_base(local_commit.id(), remote_commit.id())?;

    // If not fast-forward, perform 3-way merge
    if merge_base != local_commit.id() {
        // Use existing text_merge from src/merge/text.rs
        let merged = text_merge::merge_files(
            repo.find_blob(merge_base)?.content(),
            repo.find_blob(local_commit.id())?.content(),
            repo.find_blob(remote_commit.id())?.content(),
        )?;

        // Handle conflicts with .jinmerge workflow
    }
}
```

**Approach 2: Custom text merging**
- Use existing `src/merge/text.rs`
- Already implements 3-way diff-based merging
- Just need to integrate into pull command

**Assessment:** Jin already has 3-way merge infrastructure in `src/merge/text.rs`. Gap #4 is about **integration**, not capability.

---

## 4. Recommendations

### 4.1 Immediate Actions (No New Dependencies)

1. **Gap #1 (.jinmerge):** Custom implementation in `src/merge/jinmerge.rs`
   - Don't use diffy for markers (hardcoded labels)
   - Generate custom Git-style markers with layer ref paths
   - Parse .jinmerge files for resolution workflow

2. **Gap #2 (Fetch-before-push):** Use git2's existing capabilities
   - `git2::Repository::graph_descendant_of()` for ancestry checking
   - `git2::Remote::fetch()` for pre-push fetch
   - `git2::Branch::name()` for ref comparison

3. **Gap #3 (Detached workspace):** Custom validation logic
   - Workspace state checking before destructive operations
   - New `JinError::DetachedWorkspace` variant
   - Recovery guidance in error messages

### 4.2 Future Enhancements (Optional Dependencies)

1. **Better 3-way merge:** Consider `threeway_merge` crate
   - More Git-compatible algorithms
   - Better conflict resolution strategies
   - Drop-in replacement for `src/merge/text.rs`

2. **Syntax-aware merging:** Consider `mergiraf` crate
   - AST-based merging for code files
   - Reduces spurious conflicts
   - Research-stage, may be unstable

### 4.3 Dependencies to Keep

✅ **git2** - Mature, fully-featured, excellent Git abstraction
✅ **clap** - Best-in-class CLI parsing for Rust
✅ **thiserror/anyhow** - Standard error handling stack
✅ **serde family** - Industry-standard serialization
✅ **indexmap** - Ordered collections, well-tested
⚠️ **diffy** - Keep for conflict detection, but don't use for markers

### 4.4 No Changes Needed

- No new dependencies required for any gap
- Current stack is sufficient
- Focus on custom implementations, not crate additions

---

## 5. Implementation Guidance

### 5.1 For Gap #1 (.jinmerge)

**Do NOT use diffy for marker generation.**

Create custom implementation:
```rust
// src/merge/jinmerge.rs
pub struct JinMergeConflict {
    pub file_path: PathBuf,
    pub conflicts: Vec<ConflictRegion>,
}

pub struct ConflictRegion {
    pub layer1_ref: String,    // Full ref path
    pub layer1_content: String,
    pub layer2_ref: String,    // Full ref path
    pub layer2_content: String,
}

impl JinMergeConflict {
    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        let mut output = String::new();
        for conflict in &self.conflicts {
            output.push_str(&format!(
                "<<<<<<< {}\n{}\n=======\n{}\n>>>>>> {}\n",
                conflict.layer1_ref,
                conflict.layer1_content,
                conflict.layer2_content,
                conflict.layer2_ref
            ));
        }
        std::fs::write(path, output)?;
        Ok(())
    }
}
```

### 5.2 For Gap #2 (Fetch-before-push)

**Use git2's graph operations.**

```rust
// src/commands/push.rs
use git2::Repository;

pub fn execute(args: PushArgs) -> Result<()> {
    let config = JinConfig::load()?;
    let jin_repo = JinRepo::open_or_create()?;

    // NEW: Fetch before push
    fetch::execute()?;

    // NEW: Check if local is behind
    if is_behind_remote(&jin_repo, &args.layer)? {
        return Err(JinError::PushBehindRemote {
            hint: "Run 'jin pull' to update local state first".into()
        });
    }

    // ... rest of push logic ...
}

fn is_behind_remote(repo: &JinRepo, layer: &str) -> Result<bool> {
    let git_repo = repo.git();

    // Get local and remote commit SHAs
    let local_ref = format!("refs/jin/layers/{}", layer);
    let remote_ref = format!("refs/remotes/jin-remote/{}", layer);

    let local_obj = git_repo.revparse_single(&local_ref)?;
    let remote_obj = git_repo.revparse_single(&remote_ref)?;

    // Check if local is ancestor of remote (i.e., behind)
    Ok(git_repo.graph_descendant_of(remote_obj.id(), local_obj.id())?)
}
```

### 5.3 For Gap #3 (Detached Workspace)

**Custom validation logic.**

```rust
// src/staging/workspace.rs
pub fn validate_workspace_attached(workspace: &Workspace, context: &ProjectContext) -> Result<()> {
    // Get expected layer ref from context
    let expected_layer = context.active_layer_ref()
        .ok_or(JinError::NoActiveContext)?;

    // Get workspace's current commit
    let workspace_commit = workspace.current_commit()?;

    // Get expected layer's commit
    let jin_repo = JinRepo::open_or_create()?;
    let layer_commit = jin_repo.find_commit(&expected_layer)?;

    // Validate they match
    if workspace_commit != layer_commit.id() {
        return Err(JinError::DetachedWorkspace {
            workspace_commit: Some(workspace_commit.to_string()),
            expected_layer_ref: expected_layer.clone(),
            recovery_hint: format!(
                "Run 'jin reset --hard {}' to reattach workspace",
                expected_layer
            ),
        });
    }

    Ok(())
}
```

---

## 6. Summary

**Key Findings:**

1. **git2 has full capabilities** for fetch-before-push (Gap #2) - no new dependencies needed
2. **diffy is insufficient** for .jinmerge markers (Gap #1) - custom implementation required
3. **Detached workspace detection** (Gap #3) requires custom logic - no crate provides this
4. **3-way merge** (Gap #4) infrastructure exists - just needs integration work
5. **Current dependency stack is solid** - focus on implementation, not new crates

**No new dependencies recommended** for any gap. All required functionality can be built with existing crates or custom implementations.

---

## Appendix: Dependency Versions

| Crate | Version | Purpose | Status |
|-------|---------|---------|--------|
| git2 | 0.19 | Git operations | ✅ Keep |
| diffy | 0.4 | Conflict detection | ⚠️ Limited use |
| clap | 4.5 | CLI parsing | ✅ Keep |
| thiserror | 2.0 | Error handling | ✅ Keep |
| anyhow | 1.0 | Error context | ✅ Keep |
| serde | 1.0 | Serialization | ✅ Keep |
| serde_json | 1.0 | JSON format | ✅ Keep |
| serde_yaml | 0.9 | YAML format | ✅ Keep |
| toml | 0.8 | TOML format | ✅ Keep |
| rust-ini | 0.21 | INI format | ✅ Keep |
| indexmap | 2.0 | Ordered maps | ✅ Keep |
| dirs | 5.0 | File paths | ✅ Keep |
| chrono | 0.4 | Time handling | ✅ Keep |
| regex | 1.10 | Pattern matching | ✅ Keep |

**Optional Future Additions:**
- `threeway_merge` - If current text merge insufficient
- `mergiraf` - For syntax-aware merging (research-stage)

---

*End of Analysis*
