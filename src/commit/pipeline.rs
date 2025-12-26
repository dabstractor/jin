//! Commit pipeline orchestrating atomic commits of staged files to Jin layers.
//!
//! This module provides the `CommitPipeline` that orchestrates the entire
//! commit process including validation, tree building, commit creation,
//! transaction management, jinmap updates, and audit logging.
//!
//! # Pipeline Flow
//!
//! 1. **Validation**: Validate all staged entries
//! 2. **Build Trees**: Create Git trees for each affected layer
//! 3. **Create Commits**: Create commits for each layer tree
//! 4. **Transaction**: Update all layer refs atomically via Transaction
//! 5. **Update Jinmap**: Update .jinmap file with new mappings
//! 6. **Audit Log**: Log audit entry for the commit
//!
//! # Examples
//!
//! ```ignore
//! use jin_glm::commit::pipeline::CommitPipeline;
//!
//! let pipeline = CommitPipeline::new(&repo, &workspace_root, "myapp".to_string())?;
//! let result = pipeline.execute(&mut staging)?;
//! println!("Committed {} files to {} layers",
//!     result.files.len(),
//!     result.commits.len());
//! ```

use crate::commit::audit::AuditEntry;
use crate::commit::jinmap::Jinmap;
use crate::commit::validate::validate_staging_index;
use crate::core::{
    error::{JinError, Result},
    Layer,
};
use crate::git::{JinRepo, TransactionManager};
use crate::staging::StagingIndex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

// ===== COMMIT RESULT =====

/// Commit result containing all commit information.
///
/// Returned by `CommitPipeline::execute()` containing all the details
/// of a successful commit operation.
///
/// # Fields
///
/// - `transaction_id`: Unique ID for this transaction
/// - `commits`: Layer -> commit OID mapping
/// - `files`: Files that were committed
/// - `jinmap`: Updated jinmap after commit
/// - `audit_entry`: Audit entry logged for this commit
#[derive(Debug, Clone)]
pub struct CommitResult {
    /// Transaction ID for this commit
    pub transaction_id: String,
    /// Layer -> commit OID mapping
    pub commits: HashMap<Layer, git2::Oid>,
    /// Files that were committed
    pub files: Vec<PathBuf>,
    /// Updated jinmap
    pub jinmap: Jinmap,
    /// Audit entry
    pub audit_entry: AuditEntry,
}

// ===== COMMIT PIPELINE =====

/// Commit pipeline orchestrating the entire commit flow.
///
/// The pipeline processes staged entries through validation,
/// tree building, commit creation, and jinmap updates.
///
/// # Lifecycle
///
/// 1. Create pipeline with `CommitPipeline::new()`
/// 2. Execute with `pipeline.execute(&mut staging)`
/// 3. Examine result for commit details
///
/// # Examples
///
/// ```ignore
/// let pipeline = CommitPipeline::new(&repo, &workspace_root, "myapp".to_string())?;
/// let result = pipeline.execute(&mut staging)?;
///
/// for (layer, oid) in &result.commits {
///     println!("Layer {:?} -> {}", layer, oid);
/// }
/// ```
pub struct CommitPipeline<'a> {
    /// The Jin repository for layer operations
    repo: &'a JinRepo,
    /// Project workspace root
    workspace_root: &'a Path,
    /// Project name
    project: String,
}

impl<'a> CommitPipeline<'a> {
    /// Creates a new commit pipeline.
    ///
    /// # Arguments
    ///
    /// * `repo` - The Jin repository for layer operations
    /// * `workspace_root` - Path to the project workspace root
    /// * `project` - Project name
    ///
    /// # Returns
    ///
    /// A new `CommitPipeline` ready to execute.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let pipeline = CommitPipeline::new(&repo, &workspace_root, "myapp".to_string());
    /// ```
    pub fn new(repo: &'a JinRepo, workspace_root: &'a Path, project: String) -> Self {
        Self {
            repo,
            workspace_root,
            project,
        }
    }

    /// Executes the commit pipeline for a staging index.
    ///
    /// This is the main orchestration method that:
    /// 1. Validates all staged entries
    /// 2. Builds Git trees for each affected layer
    /// 3. Creates commits for each layer
    /// 4. Updates all layer refs atomically via Transaction
    /// 5. Updates .jinmap file
    /// 6. Logs audit entry
    ///
    /// # Arguments
    ///
    /// * `staging` - Mutable reference to the staging index (will be cleared on success)
    ///
    /// # Returns
    ///
    /// - `Ok(CommitResult)` - Commit completed successfully
    /// - `Err(JinError)` - Commit failed at some stage
    ///
    /// # Errors
    ///
    /// - `JinError::ValidationFailed` - Validation errors in staged entries
    /// - `JinError::TransactionConflict` - Concurrent modification detected
    /// - `JinError::CommitFailed` - Transaction commit failed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// match pipeline.execute(&mut staging) {
    ///     Ok(result) => {
    ///         println!("Committed {} files", result.files.len());
    ///     }
    ///     Err(JinError::ValidationFailed { errors }) => {
    ///         for error in errors {
    ///             eprintln!("Validation error: {}", error.path.display());
    ///         }
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Commit failed: {}", e);
    ///     }
    /// }
    /// ```
    pub fn execute(&self, staging: &mut StagingIndex) -> Result<CommitResult> {
        // Step 1: Validate
        let validation = validate_staging_index(staging, self.workspace_root)?;
        if !validation.is_valid() {
            return Err(JinError::ValidationError {
                message: format!("Validation failed for {} file(s)", validation.errors.len()),
            });
        }

        // Collect files before clearing staging
        let files: Vec<PathBuf> = staging
            .all_entries()
            .iter()
            .map(|e| e.path.clone())
            .collect();

        // Step 2: Build trees for each layer
        let layer_updates = self.build_layer_trees(staging)?;

        // Step 3: Create commits for each layer
        let layer_commits = self.create_layer_commits(&layer_updates)?;

        // Step 4: Update refs atomically via Transaction
        let tm = TransactionManager::new(self.repo);
        let mut tx = tm.begin_transaction()?;

        for (layer, commit_oid) in &layer_commits {
            tx.add_layer_update(layer.clone(), *commit_oid)?;
        }

        tx.prepare()?;

        // Save transaction ID before commit (which consumes tx)
        let transaction_id = tx.id().to_string();
        tx.commit()?;

        // Step 5: Update .jinmap
        let mut jinmap = Jinmap::load_from_disk(self.workspace_root)?;
        for entry in staging.all_entries() {
            let ref_name = entry
                .layer
                .git_ref()
                .ok_or_else(|| JinError::InvalidLayer {
                    name: format!("{:?}", entry.layer),
                })?;
            let layer_path = ref_name.strip_prefix("refs/jin/layers/").unwrap();
            jinmap.add_mapping(layer_path, &entry.path.display().to_string());
        }
        jinmap.save_to_disk(self.workspace_root)?;

        // Step 6: Log audit
        let audit_entry = self.create_audit_entry(&layer_commits, &files)?;
        audit_entry.save(self.repo.path())?;

        // Clear staging after successful commit
        staging.clear();

        Ok(CommitResult {
            transaction_id,
            commits: layer_commits,
            files,
            jinmap,
            audit_entry,
        })
    }

    /// Builds Git trees for all affected layers.
    ///
    /// Groups entries by layer and creates a Git tree for each layer
    /// containing all files staged for that layer.
    ///
    /// # Arguments
    ///
    /// * `staging` - The staging index to build trees from
    ///
    /// # Returns
    ///
    /// - `Ok(HashMap<Layer, Oid>)` - Mapping of layer to tree OID
    /// - `Err(JinError)` - Failed to build trees
    fn build_layer_trees(&self, staging: &StagingIndex) -> Result<HashMap<Layer, git2::Oid>> {
        let mut layer_updates = HashMap::new();

        // Get unique layers from staging
        let mut layers = Vec::new();
        for entry in staging.all_entries() {
            if !layers.contains(&entry.layer) {
                layers.push(entry.layer.clone());
            }
        }

        // Build tree for each layer
        for layer in layers {
            let entries = staging.entries_by_layer(&layer);

            if entries.is_empty() {
                continue;
            }

            // Build tree from entries
            let mut builder = self.repo.treebuilder()?;

            for entry in entries {
                // Read file content
                let full_path = self.workspace_root.join(&entry.path);
                let content = std::fs::read(&full_path)?;

                // Create blob
                let blob_oid = self.repo.create_blob(&content)?;

                // Insert into tree (use forward slashes)
                let tree_path = entry
                    .path
                    .to_str()
                    .ok_or_else(|| JinError::Message("Invalid path encoding".to_string()))?
                    .replace("\\", "/");

                builder.insert(&tree_path, blob_oid, git2::FileMode::Blob.into())?;
            }

            let tree_oid = self.repo.create_tree(&mut builder)?;
            layer_updates.insert(layer, tree_oid);
        }

        Ok(layer_updates)
    }

    /// Creates commits for each layer tree.
    ///
    /// For each layer tree, creates a Git commit with the appropriate
    /// parent (current layer ref) and commit message.
    ///
    /// # Arguments
    ///
    /// * `layer_updates` - Mapping of layer to tree OID
    ///
    /// # Returns
    ///
    /// - `Ok(HashMap<Layer, Oid>)` - Mapping of layer to commit OID
    /// - `Err(JinError)` - Failed to create commits
    fn create_layer_commits(
        &self,
        layer_updates: &HashMap<Layer, git2::Oid>,
    ) -> Result<HashMap<Layer, git2::Oid>> {
        let mut commits = HashMap::new();

        // Get signature for commits
        let signature = self.repo.signature("Jin", "jin@local")?;

        for (layer, tree_oid) in layer_updates {
            // Find the tree
            let tree = self.repo.find_tree(*tree_oid)?;

            // Get parent commit (if layer exists)
            let parent = self
                .repo
                .get_layer_ref(layer)?
                .and_then(|r| r.target())
                .and_then(|oid| self.repo.find_commit(oid).ok());

            // Prepare parents slice as &[&commit]
            let parent_refs: Vec<&git2::Commit> =
                parent.as_ref().map(|p| vec![p]).unwrap_or_default();

            // Create commit message
            let message = format!("Jin commit to layer: {}", layer);

            // Create commit
            let commit_oid = self.repo.create_commit(
                None, // Don't update HEAD
                &signature,
                &signature,
                &message,
                &tree,
                &parent_refs,
            )?;

            commits.insert(layer.clone(), commit_oid);
        }

        Ok(commits)
    }

    /// Creates an audit entry for this commit.
    ///
    /// Captures all commit metadata for audit logging.
    ///
    /// # Arguments
    ///
    /// * `layer_commits` - Mapping of layer to commit OID
    /// * `files` - Files that were committed
    ///
    /// # Returns
    ///
    /// - `Ok(AuditEntry)` - Audit entry ready to save
    /// - `Err(JinError)` - Failed to create audit entry
    fn create_audit_entry(
        &self,
        layer_commits: &HashMap<Layer, git2::Oid>,
        files: &[PathBuf],
    ) -> Result<AuditEntry> {
        // Get first layer and commit for audit entry
        let (layer, commit_oid) = layer_commits
            .iter()
            .next()
            .ok_or_else(|| JinError::Message("No commits created".to_string()))?;

        // Get parent commit
        let parent_oid = self
            .repo
            .get_layer_ref(layer)?
            .and_then(|r| r.target())
            .ok_or_else(|| JinError::Message("No parent commit".to_string()))?;

        // Collect files as strings
        let file_strings: Vec<String> = files
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        // Get user from git config
        let user = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(AuditEntry::new(
            user,
            self.project.clone(),
            layer.mode().map(|m| m.to_string()),
            layer.scope().map(|s| s.to_string()),
            layer.clone(),
            file_strings,
            parent_oid,
            *commit_oid,
            layer.mode().map(|m| m.to_string()),
            layer.scope().map(|s| s.to_string()),
        ))
    }
}

// ===== TESTS =====

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::TempDir;

    // ===== CommitPipeline::new Tests =====

    #[test]
    fn test_commit_pipeline_new() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let workspace = TempDir::new().unwrap();

        let pipeline = CommitPipeline::new(&repo, workspace.path(), "testproject".to_string());

        assert_eq!(pipeline.project, "testproject");
    }

    // ===== CommitResult Tests =====

    #[test]
    fn test_commit_result_fields() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();

        let result = CommitResult {
            transaction_id: "test-tx-id".to_string(),
            commits: HashMap::new(),
            files: vec![PathBuf::from("test.txt")],
            jinmap: Jinmap::new(),
            audit_entry: create_test_audit_entry(),
        };

        assert_eq!(result.transaction_id, "test-tx-id");
        assert_eq!(result.files.len(), 1);
        assert_eq!(result.files[0], PathBuf::from("test.txt"));
    }

    // ===== Helper Functions =====

    fn create_test_audit_entry() -> AuditEntry {
        AuditEntry {
            timestamp: Utc::now(),
            user: "test".to_string(),
            project: "testproject".to_string(),
            mode: None,
            scope: None,
            layer: 1,
            files: vec!["test.txt".to_string()],
            base_commit: "0000000000000000000000000000000000000000".to_string(),
            merge_commit: "abc123".to_string(),
            context: crate::commit::audit::AuditContext {
                active_mode: None,
                active_scope: None,
            },
        }
    }
}
