# Commit Pipeline Patterns Research

## External Commit Pipeline Patterns from Similar Projects

### 1. Git2-rs Transaction Patterns

The git2-rs library provides the foundation for atomic multi-ref operations in Rust Git tooling:

#### Basic Transaction Implementation
```rust
// Pattern: Begin transaction with staging reference
let mut tx = repo.transaction()?;
let staging_ref = format!("refs/jin/staging/{}", transaction_id);

// Pattern: Lock multiple references atomically
for ref_name in &layer_refs {
    tx.lock_ref(ref_name)?;
}

// Pattern: Set targets with signature
tx.set_target("refs/jin/layers/global/base", new_oid, &signature, "Jin commit")?;

// Pattern: Atomic commit or rollback
tx.commit()?; // or let it auto-rollback on Drop
```

#### Reference from git2-rs Documentation
- **Transaction API**: https://docs.rs/git2/0.19.0/git2/struct.Transaction.html
- **Locking Strategy**: https://github.com/rust-lang/git2-rs/blob/main/TRANSACTION.md
- **Multi-ref Updates**: git2-rs supports atomic updates across multiple references through its transaction system

### 2. Jujutsu (JJ) Commit Pipeline

Jujutsu implements sophisticated commit orchestration with atomic operations:

#### Key Patterns
- **Automatic Transaction Recovery**: Detects and recovers from interrupted transactions
- **Operation Batching**: Groups related operations into atomic units
- **Conflict Resolution**: Built-in conflict detection during pipeline execution
- **Rollback Mechanisms**: Multiple rollback strategies based on operation type

#### Relevant Code Patterns
```rust
// Pattern: Transaction state machine
enum TransactionState {
    Started,
    Prepared,
    Committed,
    RolledBack,
}

// Pattern: Pipeline with validation stages
struct CommitPipeline {
    validator: Box<dyn ValidationStage>,
    tree_builder: Box<dyn TreeBuilder>,
    committer: Box<dyn Committer>,
    ref_updater: Box<dyn RefUpdater>,
}
```

### 3. DVC (Data Version Control) Pipeline

DVC implements atomic operations for machine learning workflows:

#### Atomic Multi-File Operations
- **Content Staging**: Stages files across multiple directories
- **Dependency Tracking**: Builds dependency graphs for atomic updates
- **Consistency Validation**: Ensures all related files are updated together

#### Pattern: Staged Commit Flow
```python
# Pattern: Validate all staged files
for file in staged_files:
    validate_file_integrity(file)

# Pattern: Build trees in isolation
tree_id = build_dependency_tree(staged_files)

# Pattern: Atomic reference update
with transaction():
    update_refs(tree_id, message)
```

### 4. Monorepo Tools (Nx, Rush)

Large-scale monorepo tools implement sophisticated commit pipelines:

#### Pattern: Multi-Project Atomic Commits
- **Change Graphs**: Build dependency graphs between projects
- **Batch Operations**: Group related project changes
- **Consistency Checks**: Verify all changes are compatible
- **Rollback Sequences**: Ordered rollback for complex operations

## Best Practices for Staging Orchestration

### 1. Layer-Aware Staging Patterns

#### Route-Aware Staging
```rust
// Pattern: Route-based staging determination
fn route_to_layer(file_path: &Path, context: &ProjectContext) -> Layer {
    match (context.mode(), context.scope()) {
        (Some(mode), Some(scope)) => Layer::ModeScopeProject {
            mode: mode.clone(),
            scope: scope.clone(),
            project: context.project().clone(),
        },
        (Some(mode), None) => Layer::ModeBase { mode: mode.clone() },
        (None, Some(scope)) => Layer::ScopeBase { scope: scope.clone() },
        _ => Layer::ProjectBase { project: context.project().clone() },
    }
}
```

#### Staging Index Patterns
```rust
// Pattern: Staged entry with metadata
struct StagedEntry {
    layer: Layer,
    path: PathBuf,
    hash: Sha256,
    size: u64,
    metadata: FileMetadata,
    staged_at: Instant,
}

// Pattern: Efficient index implementation
pub struct StagingIndex {
    entries: HashMap<PathBuf, StagedEntry>,
    layer_index: HashMap<Layer, Vec<PathBuf>>,
    hash_index: HashMap<Sha256, PathBuf>,
}
```

### 2. Validation Patterns

#### Pre-Commit Validation
- **File Format Detection**: Auto-detect JSON, YAML, TOML, INI, text
- **Schema Validation**: Validate against known schemas for each format
- **Conflict Detection**: Check for conflicts with existing layer content
- **Dependency Resolution**: Verify file dependencies within layers

#### Validation Pipeline Pattern
```rust
struct ValidationPipeline {
    stages: Vec<Box<dyn ValidationStage>>,
}

impl ValidationPipeline {
    fn validate(&self, staged: &StagingIndex) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        for stage in &self.stages {
            stage.validate(staged, &mut report)?;

            if report.has_fatal_errors() {
                break; // Fail fast on fatal errors
            }
        }

        report
    }
}
```

## Error Handling Patterns for Commit Failures

### 1. Transaction Error Handling

#### Pattern: Comprehensive Error Types
```rust
#[derive(Debug, thiserror::Error)]
pub enum CommitError {
    #[error("Transaction preparation failed: {source}")]
    PreparationFailed {
        source: Box<dyn std::error::Error>,
        files: Vec<String>,
    },
    #[error("Validation failed: {validation_errors}")]
    ValidationFailed {
        validation_errors: Vec<ValidationError>,
    },
    #[error("Tree building failed for file: {file}")]
    TreeBuildFailed {
        file: PathBuf,
        source: Box<dyn std::error::Error>,
    },
    #[error("Git commit failed: {source}")]
    CommitFailed {
        source: Box<dyn std::error::Error>,
        files: Vec<String>,
    },
    #[error("Reference update conflict on: {ref_name}")]
    RefConflict {
        ref_name: String,
        current_oid: Oid,
        expected_oid: Oid,
    },
}
```

#### Pattern: Automatic Rollback
```rust
impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        // Pattern: Auto-rollback on drop
        if self.state != TransactionState::Committed
            && self.state != TransactionState::RolledBack {

            // Cleanup staging reference
            let _ = self.repo.delete_staging_ref(&self.id);

            // Release git2 transaction (auto-rollback)
            let _ = self.tx.take();
        }
    }
}
```

### 2. Recovery Patterns

#### Pattern: Orphan Transaction Detection
```rust
impl TransactionManager<'_> {
    pub fn detect_orphaned(&self) -> Result<Vec<String>> {
        let mut orphaned = Vec::new();

        // Pattern: Scan for staging references
        for reference in self.repo.inner.references_glob("refs/jin/staging/*")? {
            if let Ok(reference) = reference {
                if let Some(name) = reference.name() {
                    if name.starts_with("refs/jin/staging/") {
                        let tx_id = name.strip_prefix("refs/jin/staging/")
                                       .unwrap()
                                       .to_string();
                        orphaned.push(tx_id);
                    }
                }
            }
        }

        Ok(orphaned)
    }
}
```

### 3. Conflict Resolution Patterns

#### Pattern: Three-Way Merge for Text
```rust
fn resolve_text_conflict(
    base: &str,
    ours: &str,
    theirs: &str,
) -> Result<String, MergeError> {
    // Pattern: Use similar crate for 3-way merge
    let diff = similar::TextDiff::from_lines(base, ours);
    let merged = diff.diff_lines(theirs.as_bytes())
                     .unified()
                     .to_string();

    if merged.contains("<<<<<<<") {
        Err(MergeError::ConflictDetected)
    } else {
        Ok(merged)
    }
}
```

## Atomic Commit Strategies

### 1. Git Transaction API Pattern

#### Pattern: Multi-Ref Atomic Update
```rust
pub fn commit_layers_atomically(
    repo: &JinRepo,
    updates: &[(Layer, Oid)],
    message: &str,
) -> Result<Oid> {
    // Pattern: Create git2 transaction
    let mut tx = repo.inner.transaction()?;

    // Pattern: Lock all references first
    for (layer, _) in updates {
        let ref_name = layer.git_ref()?;
        tx.lock_ref(&ref_name)?;
    }

    // Pattern: Update all references with same signature
    let signature = repo.inner.signature()?;
    for (layer, new_oid) in updates {
        let ref_name = layer.git_ref()?;
        tx.set_target(&ref_name, new_oid, &signature, message)?;
    }

    // Pattern: Atomic commit
    let commit_id = tx.commit()?;
    Ok(commit_id)
}
```

### 2. Staging Reference Pattern

#### Pattern: Two-Phase Commit with Staging
```rust
impl Transaction<'_> {
    pub fn commit(mut self) -> Result<()> {
        // Phase 1: Prepare all changes
        self.prepare()?;

        // Phase 2: Atomic commit via git2 transaction
        let mut git_tx = self.repo.inner.transaction()?;

        for (layer, new_oid, _) in &self.updates {
            let ref_name = layer.git_ref()?;
            git_tx.set_target(
                &ref_name,
                *new_oid,
                &self.signature,
                &format!("Jin transaction: {}", self.id),
            )?;
        }

        // Pattern: All-or-nothing update
        git_tx.commit()?;

        // Pattern: Cleanup staging reference on success
        self.repo.delete_staging_ref(&self.id)?;

        self.state = TransactionState::Committed;
        Ok(())
    }
}
```

### 3. Recovery and Cleanup Patterns

#### Pattern: Automatic Cleanup
```rust
impl TransactionManager<'_> {
    pub fn recover_all(&self) -> Result<usize> {
        let orphaned = self.detect_orphaned()?;
        let mut recovered = 0;

        for tx_id in orphaned {
            // Pattern: Safe cleanup - delete staging refs
            match self.repo.delete_staging_ref(&tx_id) {
                Ok(_) => recovered += 1,
                Err(e) => {
                    eprintln!("Failed to recover transaction {}: {}", tx_id, e);
                }
            }
        }

        Ok(recovered)
    }
}
```

## URLs to Relevant Documentation/Examples

### Git2-rs Documentation
- [git2-rs GitHub Repository](https://github.com/rust-lang/git2-rs)
- [git2-rs Documentation](https://docs.rs/git2/0.19.0/git2/)
- [Transaction API Reference](https://docs.rs/git2/0.19.0/git2/struct.Transaction.html)

### Transaction and Atomic Operations
- [Git Core Documentation - Atomic Operations](https://git-scm.com/docs/git-read-tree#_atomic_tree_merge_updates)
- [Jujutsu Transaction Patterns](https://github.com/martinvonz/jj/blob/main/docs/transactions.md)

### Staging Area Patterns
- [Git Index Documentation](https://git-scm.com/docs/git-index)
- [DVC Staging Patterns](https://dvc.org/doc/user-guide/project-structure/dvc-files)

### Error Handling Patterns
- [thiserror - Error Handling in Rust](https://github.com/dtolnay/thiserror)
- [anyhow - Error Context Propagation](https://github.com/dtolnay/anyhow)

### Version Control Research
- [Git Internals - Plumbing and Porcelain](https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain)
- [Monorepo Tool Patterns](https://nx.dev/latest/core-concepts/monorepo)
- [Advanced Git Workflows](https://www.atlassian.com/git/tutorials/comparing-workflows)

## Key Takeaways for Jin Implementation

1. **Leverage git2-rs Transactions**: Use git2-rs's built-in transaction support for atomic multi-ref operations
2. **Staging Reference Pattern**: Use unique staging refs for transaction tracking and recovery
3. **State Machine**: Implement clear transaction states with RAII cleanup
4. **Fail-Fast Validation**: Validate all staged files before attempting commit
5. **Automatic Recovery**: Detect and clean up orphaned transactions
6. **Comprehensive Error Handling**: Provide detailed error messages with context
7. **Layer-Aware Operations**: Design pipeline to understand layer precedence and routing

This research provides patterns and best practices that can be adapted for Jin's commit pipeline implementation, ensuring robust atomic operations and proper error handling.