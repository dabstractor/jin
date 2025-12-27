//! Commit pipeline implementation

use crate::core::{JinError, Layer, ProjectContext, Result};
use crate::git::{JinRepo, LayerTransaction, ObjectOps, RefOps};
use crate::staging::{StagedEntry, StagingIndex};
use git2::Oid;

/// Configuration for a commit operation
#[derive(Debug)]
pub struct CommitConfig {
    /// Commit message
    pub message: String,
    /// Author name (optional, uses Git config if not specified)
    pub author_name: Option<String>,
    /// Author email (optional, uses Git config if not specified)
    pub author_email: Option<String>,
    /// Dry run - don't actually commit
    pub dry_run: bool,
}

impl CommitConfig {
    /// Create a new commit configuration
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            author_name: None,
            author_email: None,
            dry_run: false,
        }
    }

    /// Set dry run mode
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }
}

/// Result of a commit operation
#[derive(Debug)]
pub struct CommitResult {
    /// Layers that were committed to
    pub committed_layers: Vec<Layer>,
    /// Number of files committed
    pub file_count: usize,
    /// Commit hashes for each layer
    pub commit_hashes: Vec<(Layer, String)>,
}

/// Pipeline for executing atomic commits across layers
#[derive(Debug)]
pub struct CommitPipeline {
    /// The staging index to commit from
    staging: StagingIndex,
}

impl CommitPipeline {
    /// Create a new commit pipeline
    pub fn new(staging: StagingIndex) -> Self {
        Self { staging }
    }

    /// Execute the commit
    ///
    /// This will:
    /// 1. Validate staging is not empty
    /// 2. Group staged entries by target layer
    /// 3. For each layer, build a tree and create a commit
    /// 4. Execute all ref updates atomically via LayerTransaction
    /// 5. Clear staging on success
    pub fn execute(&mut self, config: &CommitConfig) -> Result<CommitResult> {
        // Validate staging not empty
        if self.staging.is_empty() {
            return Err(JinError::Other("Nothing to commit".to_string()));
        }

        let affected_layers = self.staging.affected_layers();
        let file_count = self.staging.len();

        // Handle dry-run mode
        if config.dry_run {
            return self.execute_dry_run(&affected_layers, file_count);
        }

        // Load context for ref path generation (use default if not initialized)
        let context = ProjectContext::load().unwrap_or_default();

        // Open Jin repository
        let repo = JinRepo::open_or_create()?;

        // Create commits for each layer
        let mut layer_commits: Vec<(Layer, Oid)> = Vec::new();

        for layer in &affected_layers {
            let entries = self.staging.entries_for_layer(*layer);
            let commit_oid =
                self.create_layer_commit(&repo, *layer, &entries, &context, &config.message)?;
            layer_commits.push((*layer, commit_oid));
        }

        // Apply all updates atomically via transaction
        let mut tx = LayerTransaction::begin(&repo, &config.message)?;
        for (layer, commit_oid) in &layer_commits {
            tx.add_layer_update(
                *layer,
                context.mode.as_deref(),
                context.scope.as_deref(),
                context.project.as_deref(),
                *commit_oid,
            )?;
        }
        tx.commit()?;

        // Clear staging on success
        self.staging.clear();
        self.staging.save()?;

        // Build result
        let commit_hashes: Vec<(Layer, String)> = layer_commits
            .iter()
            .map(|(l, oid)| (*l, oid.to_string()))
            .collect();

        Ok(CommitResult {
            committed_layers: affected_layers,
            file_count,
            commit_hashes,
        })
    }

    /// Create a commit for a single layer
    fn create_layer_commit(
        &self,
        repo: &JinRepo,
        layer: Layer,
        entries: &[&StagedEntry],
        context: &ProjectContext,
        message: &str,
    ) -> Result<Oid> {
        // Build tree from entries
        let tree_oid = self.build_layer_tree(repo, entries)?;

        // Get parent commit if layer ref exists
        let parent_oids = self.get_parent_commits(repo, layer, context)?;

        // Create commit (don't update ref directly - transaction handles that)
        repo.create_commit(None, message, tree_oid, &parent_oids)
    }

    /// Build a tree from staged entries
    fn build_layer_tree(&self, repo: &JinRepo, entries: &[&StagedEntry]) -> Result<Oid> {
        // Convert entries to (path, oid) tuples, filtering out deletions
        let files: Vec<(String, Oid)> = entries
            .iter()
            .filter(|e| !e.is_delete())
            .map(|e| {
                let oid = Oid::from_str(&e.content_hash).map_err(|err| {
                    JinError::Transaction(format!(
                        "Invalid content hash for {}: {}",
                        e.path.display(),
                        err
                    ))
                })?;
                Ok((e.path.display().to_string(), oid))
            })
            .collect::<Result<Vec<_>>>()?;

        // Handle empty tree (all deletions)
        if files.is_empty() {
            return repo.create_tree(&[]);
        }

        repo.create_tree_from_paths(&files)
    }

    /// Get parent commit OIDs for a layer
    fn get_parent_commits(
        &self,
        repo: &JinRepo,
        layer: Layer,
        context: &ProjectContext,
    ) -> Result<Vec<Oid>> {
        let ref_path = layer.ref_path(
            context.mode.as_deref(),
            context.scope.as_deref(),
            context.project.as_deref(),
        );

        // CRITICAL: Check ref_exists() before resolve_ref() to avoid panic
        if repo.ref_exists(&ref_path) {
            let parent_oid = repo.resolve_ref(&ref_path)?;
            Ok(vec![parent_oid])
        } else {
            // No parent - this is the initial commit for this layer
            Ok(vec![])
        }
    }

    /// Execute dry-run mode
    fn execute_dry_run(
        &self,
        affected_layers: &[Layer],
        file_count: usize,
    ) -> Result<CommitResult> {
        println!(
            "Would commit {} files to {} layers:",
            file_count,
            affected_layers.len()
        );
        for layer in affected_layers {
            let layer_entries = self.staging.entries_for_layer(*layer);
            println!(
                "  {} ({}): {} files",
                layer,
                layer.precedence(),
                layer_entries.len()
            );
            for entry in layer_entries {
                println!("    {}", entry.path.display());
            }
        }

        Ok(CommitResult {
            committed_layers: affected_layers.to_vec(),
            file_count,
            commit_hashes: Vec::new(),
        })
    }

    /// Abort the commit and roll back any changes
    pub fn abort(&mut self) -> Result<()> {
        // If there's an incomplete transaction, RecoveryManager handles it
        // This method exists for explicit abort during pipeline execution
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::objects::TreeEntry;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Creates an isolated test environment with Jin repo and staging directory
    fn create_test_setup() -> (TempDir, JinRepo, PathBuf) {
        let temp = TempDir::new().unwrap();
        let base_path = temp.path().to_path_buf();

        // Create .jin directory for staging and context
        std::fs::create_dir_all(base_path.join(".jin").join("staging")).unwrap();

        // Create Jin bare repository
        let repo_path = base_path.join(".jin-repo");
        let repo = JinRepo::create_at(&repo_path).unwrap();

        (temp, repo, base_path)
    }

    #[test]
    fn test_commit_config_new() {
        let config = CommitConfig::new("Test commit");
        assert_eq!(config.message, "Test commit");
        assert!(!config.dry_run);
    }

    #[test]
    fn test_commit_config_dry_run() {
        let config = CommitConfig::new("Test").dry_run(true);
        assert!(config.dry_run);
    }

    #[test]
    fn test_commit_pipeline_empty() {
        let staging = StagingIndex::new();
        let mut pipeline = CommitPipeline::new(staging);
        let config = CommitConfig::new("Empty commit");

        let result = pipeline.execute(&config);
        assert!(result.is_err());
        if let Err(JinError::Other(msg)) = result {
            assert_eq!(msg, "Nothing to commit");
        }
    }

    #[test]
    fn test_commit_pipeline_with_entries_dry_run() {
        let mut staging = StagingIndex::new();
        staging.add(StagedEntry::new(
            PathBuf::from("test.json"),
            Layer::ProjectBase,
            "hash123".to_string(),
        ));

        let mut pipeline = CommitPipeline::new(staging);
        let config = CommitConfig::new("Test commit").dry_run(true);

        let result = pipeline.execute(&config).unwrap();
        assert_eq!(result.file_count, 1);
        assert!(result.committed_layers.contains(&Layer::ProjectBase));
        // Dry run should not produce commit hashes
        assert!(result.commit_hashes.is_empty());
    }

    #[test]
    fn test_build_layer_tree_single_file() {
        let (_temp, repo, _base_path) = create_test_setup();

        // Create a blob first
        let blob_oid = repo.create_blob(b"test content").unwrap();

        let staging = StagingIndex::new();
        let entry = StagedEntry::new(
            PathBuf::from("config.json"),
            Layer::ProjectBase,
            blob_oid.to_string(),
        );

        let pipeline = CommitPipeline::new(staging);
        let entries = vec![&entry];

        let tree_oid = pipeline.build_layer_tree(&repo, &entries).unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();

        assert_eq!(tree.len(), 1);
        assert!(tree.get_name("config.json").is_some());
    }

    #[test]
    fn test_build_layer_tree_multiple_files() {
        let (_temp, repo, _base_path) = create_test_setup();

        let blob1 = repo.create_blob(b"content 1").unwrap();
        let blob2 = repo.create_blob(b"content 2").unwrap();

        let staging = StagingIndex::new();
        let entry1 = StagedEntry::new(
            PathBuf::from("file1.json"),
            Layer::GlobalBase,
            blob1.to_string(),
        );
        let entry2 = StagedEntry::new(
            PathBuf::from("file2.json"),
            Layer::GlobalBase,
            blob2.to_string(),
        );

        let pipeline = CommitPipeline::new(staging);
        let entries = vec![&entry1, &entry2];

        let tree_oid = pipeline.build_layer_tree(&repo, &entries).unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();

        assert_eq!(tree.len(), 2);
        assert!(tree.get_name("file1.json").is_some());
        assert!(tree.get_name("file2.json").is_some());
    }

    #[test]
    fn test_build_layer_tree_nested_paths() {
        let (_temp, repo, _base_path) = create_test_setup();

        let blob_oid = repo.create_blob(b"nested content").unwrap();

        let staging = StagingIndex::new();
        let entry = StagedEntry::new(
            PathBuf::from(".claude/config/settings.json"),
            Layer::ModeBase,
            blob_oid.to_string(),
        );

        let pipeline = CommitPipeline::new(staging);
        let entries = vec![&entry];

        let tree_oid = pipeline.build_layer_tree(&repo, &entries).unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();

        // Should have .claude directory at root
        assert!(tree.get_name(".claude").is_some());
        let claude_entry = tree.get_name(".claude").unwrap();
        assert_eq!(claude_entry.kind(), Some(git2::ObjectType::Tree));
    }

    #[test]
    fn test_build_layer_tree_with_deletions() {
        let (_temp, repo, _base_path) = create_test_setup();

        let blob_oid = repo.create_blob(b"keep this").unwrap();

        let staging = StagingIndex::new();
        let keep_entry = StagedEntry::new(
            PathBuf::from("keep.json"),
            Layer::ProjectBase,
            blob_oid.to_string(),
        );
        let delete_entry = StagedEntry::delete(PathBuf::from("delete.json"), Layer::ProjectBase);

        let pipeline = CommitPipeline::new(staging);
        let entries = vec![&keep_entry, &delete_entry];

        let tree_oid = pipeline.build_layer_tree(&repo, &entries).unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();

        // Only keep.json should be in tree
        assert_eq!(tree.len(), 1);
        assert!(tree.get_name("keep.json").is_some());
        assert!(tree.get_name("delete.json").is_none());
    }

    #[test]
    fn test_build_layer_tree_all_deletions() {
        let (_temp, repo, _base_path) = create_test_setup();

        let staging = StagingIndex::new();
        let delete_entry1 = StagedEntry::delete(PathBuf::from("file1.json"), Layer::ProjectBase);
        let delete_entry2 = StagedEntry::delete(PathBuf::from("file2.json"), Layer::ProjectBase);

        let pipeline = CommitPipeline::new(staging);
        let entries = vec![&delete_entry1, &delete_entry2];

        let tree_oid = pipeline.build_layer_tree(&repo, &entries).unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();

        // Empty tree when all entries are deletions
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn test_get_parent_commits_no_ref() {
        let (_temp, repo, _base_path) = create_test_setup();

        let staging = StagingIndex::new();
        let context = ProjectContext::default();
        let pipeline = CommitPipeline::new(staging);

        let parents = pipeline
            .get_parent_commits(&repo, Layer::GlobalBase, &context)
            .unwrap();

        assert!(parents.is_empty());
    }

    #[test]
    fn test_get_parent_commits_with_existing_ref() {
        let (_temp, repo, _base_path) = create_test_setup();

        // Create an initial commit
        let blob_oid = repo.create_blob(b"initial content").unwrap();
        let tree_oid = repo
            .create_tree(&[TreeEntry::blob("file.txt", blob_oid)])
            .unwrap();
        let initial_commit = repo
            .create_commit(
                Some("refs/jin/layers/global"),
                "Initial commit",
                tree_oid,
                &[],
            )
            .unwrap();

        let staging = StagingIndex::new();
        let context = ProjectContext::default();
        let pipeline = CommitPipeline::new(staging);

        let parents = pipeline
            .get_parent_commits(&repo, Layer::GlobalBase, &context)
            .unwrap();

        assert_eq!(parents.len(), 1);
        assert_eq!(parents[0], initial_commit);
    }

    #[test]
    fn test_create_layer_commit_initial() {
        let (_temp, repo, _base_path) = create_test_setup();

        let blob_oid = repo.create_blob(b"commit content").unwrap();

        let staging = StagingIndex::new();
        let entry = StagedEntry::new(
            PathBuf::from("config.json"),
            Layer::GlobalBase,
            blob_oid.to_string(),
        );
        let context = ProjectContext::default();

        let pipeline = CommitPipeline::new(staging);
        let entries = vec![&entry];

        let commit_oid = pipeline
            .create_layer_commit(&repo, Layer::GlobalBase, &entries, &context, "Test commit")
            .unwrap();

        // Verify commit was created
        let commit = repo.find_commit(commit_oid).unwrap();
        assert_eq!(commit.message().unwrap(), "Test commit");
        assert_eq!(commit.parent_count(), 0); // Initial commit, no parents
    }

    #[test]
    fn test_create_layer_commit_with_parent() {
        let (_temp, repo, _base_path) = create_test_setup();

        // Create initial commit
        let blob1 = repo.create_blob(b"initial").unwrap();
        let tree1 = repo
            .create_tree(&[TreeEntry::blob("file.txt", blob1)])
            .unwrap();
        let _initial = repo
            .create_commit(Some("refs/jin/layers/global"), "Initial", tree1, &[])
            .unwrap();

        // Now create a new commit that should have parent
        let blob2 = repo.create_blob(b"updated").unwrap();
        let staging = StagingIndex::new();
        let entry = StagedEntry::new(
            PathBuf::from("file.txt"),
            Layer::GlobalBase,
            blob2.to_string(),
        );
        let context = ProjectContext::default();

        let pipeline = CommitPipeline::new(staging);
        let entries = vec![&entry];

        let commit_oid = pipeline
            .create_layer_commit(
                &repo,
                Layer::GlobalBase,
                &entries,
                &context,
                "Update commit",
            )
            .unwrap();

        // Verify commit has parent
        let commit = repo.find_commit(commit_oid).unwrap();
        assert_eq!(commit.message().unwrap(), "Update commit");
        assert_eq!(commit.parent_count(), 1); // Should have one parent
    }

    #[test]
    fn test_dry_run_mode() {
        let mut staging = StagingIndex::new();
        staging.add(StagedEntry::new(
            PathBuf::from("file1.json"),
            Layer::GlobalBase,
            "hash1".to_string(),
        ));
        staging.add(StagedEntry::new(
            PathBuf::from("file2.json"),
            Layer::ModeBase,
            "hash2".to_string(),
        ));

        let mut pipeline = CommitPipeline::new(staging);
        let config = CommitConfig::new("Dry run test").dry_run(true);

        let result = pipeline.execute(&config).unwrap();

        assert_eq!(result.file_count, 2);
        assert_eq!(result.committed_layers.len(), 2);
        assert!(result.commit_hashes.is_empty()); // No actual commits in dry run
    }

    #[test]
    fn test_abort() {
        let staging = StagingIndex::new();
        let mut pipeline = CommitPipeline::new(staging);

        // Abort should succeed without error
        let result = pipeline.abort();
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_content_hash() {
        let (_temp, repo, _base_path) = create_test_setup();

        let staging = StagingIndex::new();
        let entry = StagedEntry::new(
            PathBuf::from("file.json"),
            Layer::GlobalBase,
            "invalid_hash".to_string(), // Not a valid Git OID
        );

        let pipeline = CommitPipeline::new(staging);
        let entries = vec![&entry];

        let result = pipeline.build_layer_tree(&repo, &entries);
        assert!(result.is_err());
        if let Err(JinError::Transaction(msg)) = result {
            assert!(msg.contains("Invalid content hash"));
        }
    }
}
