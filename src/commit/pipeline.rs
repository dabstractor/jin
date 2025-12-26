//! Commit pipeline implementation

use crate::core::{JinError, Layer, Result};
use crate::staging::StagingIndex;

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
    /// TODO: Implement proper commit execution in later milestone
    ///
    /// This should:
    /// 1. Group staged entries by target layer
    /// 2. For each layer, create blobs and trees
    /// 3. Create a commit for each layer
    /// 4. Execute all updates atomically via transaction
    /// 5. Clear staging on success
    pub fn execute(&mut self, config: &CommitConfig) -> Result<CommitResult> {
        if self.staging.is_empty() {
            return Err(JinError::Other("Nothing to commit".to_string()));
        }

        let affected_layers = self.staging.affected_layers();
        let file_count = self.staging.len();

        if config.dry_run {
            // Dry run - just report what would happen
            println!(
                "Would commit {} files to {} layers:",
                file_count,
                affected_layers.len()
            );
            for layer in &affected_layers {
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

            return Ok(CommitResult {
                committed_layers: affected_layers,
                file_count,
                commit_hashes: Vec::new(),
            });
        }

        // TODO: Implement actual commit logic
        // 1. Open Jin repository
        // 2. For each affected layer:
        //    a. Get or create parent commit
        //    b. Build tree with staged entries
        //    c. Create commit object
        //    d. Add ref update to transaction
        // 3. Execute transaction atomically
        // 4. Clear staging

        Ok(CommitResult {
            committed_layers: affected_layers,
            file_count,
            commit_hashes: Vec::new(),
        })
    }

    /// Abort the commit and roll back any changes
    pub fn abort(&mut self) -> Result<()> {
        // TODO: Implement rollback
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::staging::StagedEntry;
    use std::path::PathBuf;

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
    }

    #[test]
    fn test_commit_pipeline_with_entries() {
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
    }
}
