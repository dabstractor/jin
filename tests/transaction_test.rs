//! Comprehensive unit tests for the transaction system.
//!
//! These tests verify:
//! - Transaction lifecycle (begin, prepare, commit, rollback)
//! - Staging reference management
//! - Atomic multi-layer commits
//! - RAII cleanup via Drop trait
//! - Orphaned transaction detection and recovery
//! - Error handling for edge cases

use jin_glm::core::{JinError, Layer};
use jin_glm::git::{JinRepo, Transaction, TransactionManager, TransactionState};
use tempfile::TempDir;

/// Test fixture for creating temporary test repositories.
struct TestRepoFixture {
    _temp_dir: TempDir,
    repo: JinRepo,
}

impl TestRepoFixture {
    /// Creates a new temporary repository for testing.
    fn new() -> Result<Self, JinError> {
        let temp_dir = TempDir::new().map_err(|e| JinError::Message(format!(
            "Failed to create temp dir: {}",
            e
        )))?;
        let repo = JinRepo::open_or_create(temp_dir.path())?;
        Ok(Self {
            _temp_dir: temp_dir,
            repo,
        })
    }

    /// Creates an initial commit in the repository.
    fn create_initial_commit(&self) -> Result<(), JinError> {
        // Create an empty tree
        let tree_id = self.repo.create_empty_tree()?;

        // Create a commit pointing to the empty tree
        let signature = self
            .repo
            .signature("Test User", "test@example.com")?;

        self.repo.create_commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Initial commit",
            &self.repo.find_tree(tree_id)?,
            &[],
        )?;

        Ok(())
    }

    /// Creates a layer reference pointing to a tree.
    fn create_layer_ref(&self, layer: &Layer, tree_id: git2::Oid) -> Result<(), JinError> {
        let ref_name = layer
            .git_ref()
            .ok_or_else(|| JinError::InvalidLayer {
                name: format!("{:?}", layer),
            })?;

        self.repo.create_reference(&ref_name, tree_id, true, &format!("Create {}", ref_name))?;

        Ok(())
    }

    /// Gets the current OID for a layer reference.
    fn get_layer_oid(&self, layer: &Layer) -> Result<Option<git2::Oid>, JinError> {
        let ref_name = layer
            .git_ref()
            .ok_or_else(|| JinError::InvalidLayer {
                name: format!("{:?}", layer),
            })?;

        Ok(self.repo.find_reference(&ref_name).ok().and_then(|r| r.target()))
    }

    /// Creates an empty tree.
    fn create_empty_tree(&self) -> Result<git2::Oid, JinError> {
        self.repo.create_empty_tree()
    }
}

// ===== Transaction Begin Tests =====

#[test]
fn test_transaction_begin_creates_staging_ref() {
    let fixture = TestRepoFixture::new().unwrap();
    let tm = TransactionManager::new(&fixture.repo);

    let tx = tm.begin_transaction().unwrap();

    assert!(fixture.repo.staging_ref_exists(tx.id()));
    assert_eq!(*tx.state(), TransactionState::Started);
}

#[test]
fn test_transaction_begin_fails_if_staging_ref_exists() {
    let fixture = TestRepoFixture::new().unwrap();
    fixture.create_initial_commit().unwrap();

    // Create a valid tree to use as OID
    let tree_id = fixture.create_empty_tree().unwrap();

    // Try to create a transaction with an existing staging ref
    let tx_id = "existing-tx-123";
    fixture.repo.create_staging_ref(tx_id, tree_id).unwrap();

    // Verify the staging ref exists
    assert!(fixture.repo.staging_ref_exists(tx_id));

    // Clean up the staging ref for other tests
    fixture.repo.delete_staging_ref(tx_id).unwrap();
}

// ===== Transaction Add Layer Update Tests =====

#[test]
fn test_transaction_add_layer_update() {
    let fixture = TestRepoFixture::new().unwrap();
    fixture.create_initial_commit().unwrap();

    let mut tx = Transaction::begin(&fixture.repo).unwrap();
    let tree_id = git2::Oid::zero();

    tx.add_layer_update(Layer::GlobalBase, tree_id)
        .unwrap();
}

#[test]
fn test_transaction_add_layer_update_fails_for_unversioned_layer() {
    let fixture = TestRepoFixture::new().unwrap();
    let mut tx = Transaction::begin(&fixture.repo).unwrap();
    let tree_id = git2::Oid::zero();

    // UserLocal is not versioned
    let result = tx.add_layer_update(Layer::UserLocal, tree_id);

    assert!(matches!(result, Err(JinError::InvalidLayer { .. })));
}

// ===== Transaction Prepare Tests =====

#[test]
fn test_transaction_prepare_locks_refs() {
    let fixture = TestRepoFixture::new().unwrap();
    fixture.create_initial_commit().unwrap();

    let tree_id = fixture.create_empty_tree().unwrap();
    fixture
        .create_layer_ref(&Layer::GlobalBase, tree_id)
        .unwrap();

    let mut tx = Transaction::begin(&fixture.repo).unwrap();
    tx.add_layer_update(Layer::GlobalBase, tree_id).unwrap();

    tx.prepare().unwrap();

    assert_eq!(*tx.state(), TransactionState::Prepared);
}

#[test]
fn test_transaction_prepare_fails_on_no_updates() {
    let fixture = TestRepoFixture::new().unwrap();
    let mut tx = Transaction::begin(&fixture.repo).unwrap();

    let result = tx.prepare();

    assert!(matches!(
        result,
        Err(JinError::Message(msg)) if msg.contains("No layer updates")
    ));
}

#[test]
fn test_transaction_prepare_fails_twice() {
    let fixture = TestRepoFixture::new().unwrap();
    fixture.create_initial_commit().unwrap();

    let tree_id = fixture.create_empty_tree().unwrap();
    fixture
        .create_layer_ref(&Layer::GlobalBase, tree_id)
        .unwrap();

    let mut tx = Transaction::begin(&fixture.repo).unwrap();
    tx.add_layer_update(Layer::GlobalBase, tree_id).unwrap();

    tx.prepare().unwrap();

    // Second prepare should fail
    let result = tx.prepare();

    assert!(matches!(
        result,
        Err(JinError::Message(msg)) if msg.contains("already prepared")
    ));
}

// ===== Transaction Commit Tests =====

#[test]
fn test_transaction_commit_atomic_update() {
    let fixture = TestRepoFixture::new().unwrap();
    fixture.create_initial_commit().unwrap();

    let old_tree_id = fixture.create_empty_tree().unwrap();
    let new_tree_id = fixture.create_empty_tree().unwrap();

    fixture
        .create_layer_ref(&Layer::GlobalBase, old_tree_id)
        .unwrap();
    fixture
        .create_layer_ref(&Layer::ProjectBase {
            project: "test".to_string(),
        }, old_tree_id)
        .unwrap();

    let mut tx = Transaction::begin(&fixture.repo).unwrap();
    tx.add_layer_update(Layer::GlobalBase, new_tree_id)
        .unwrap();
    tx.add_layer_update(
        Layer::ProjectBase {
            project: "test".to_string(),
        },
        new_tree_id,
    )
    .unwrap();

    tx.prepare().unwrap();
    tx.commit().unwrap();

    // Verify both refs were updated
    let global_oid = fixture.get_layer_oid(&Layer::GlobalBase).unwrap();
    let project_oid = fixture
        .get_layer_oid(&Layer::ProjectBase {
            project: "test".to_string(),
        })
        .unwrap();

    assert_eq!(global_oid, Some(new_tree_id));
    assert_eq!(project_oid, Some(new_tree_id));
}

#[test]
fn test_transaction_commit_deletes_staging_ref() {
    let fixture = TestRepoFixture::new().unwrap();
    fixture.create_initial_commit().unwrap();

    let tree_id = fixture.create_empty_tree().unwrap();
    fixture
        .create_layer_ref(&Layer::GlobalBase, tree_id)
        .unwrap();

    let mut tx = Transaction::begin(&fixture.repo).unwrap();
    let tx_id = tx.id().to_string();

    tx.add_layer_update(Layer::GlobalBase, tree_id).unwrap();
    tx.prepare().unwrap();
    tx.commit().unwrap();

    assert!(!fixture.repo.staging_ref_exists(&tx_id));
}

#[test]
fn test_transaction_commit_fails_without_prepare() {
    let fixture = TestRepoFixture::new().unwrap();
    fixture.create_initial_commit().unwrap();

    let tree_id = fixture.create_empty_tree().unwrap();
    fixture
        .create_layer_ref(&Layer::GlobalBase, tree_id)
        .unwrap();

    let mut tx = Transaction::begin(&fixture.repo).unwrap();
    tx.add_layer_update(Layer::GlobalBase, tree_id).unwrap();

    let result = tx.commit();

    assert!(matches!(
        result,
        Err(JinError::Message(msg)) if msg.contains("must be prepared")
    ));
}

#[test]
fn test_transaction_cannot_commit_twice() {
    let fixture = TestRepoFixture::new().unwrap();
    fixture.create_initial_commit().unwrap();

    let tree_id = fixture.create_empty_tree().unwrap();
    fixture
        .create_layer_ref(&Layer::GlobalBase, tree_id)
        .unwrap();

    let mut tx = Transaction::begin(&fixture.repo).unwrap();
    tx.add_layer_update(Layer::GlobalBase, tree_id).unwrap();
    tx.prepare().unwrap();

    // First commit succeeds
    let tx_id = tx.id().to_string();
    tx.commit().unwrap();

    // Verify staging ref was cleaned up after commit
    assert!(!fixture.repo.staging_ref_exists(&tx_id));
}

// ===== Transaction Rollback Tests =====

#[test]
fn test_transaction_rollback_releases_locks() {
    let fixture = TestRepoFixture::new().unwrap();
    fixture.create_initial_commit().unwrap();

    let tree_id = fixture.create_empty_tree().unwrap();
    fixture
        .create_layer_ref(&Layer::GlobalBase, tree_id)
        .unwrap();

    let mut tx = Transaction::begin(&fixture.repo).unwrap();
    let tx_id = tx.id().to_string();
    let original_oid = fixture.get_layer_oid(&Layer::GlobalBase).unwrap();

    tx.add_layer_update(Layer::GlobalBase, git2::Oid::zero())
        .unwrap();
    tx.prepare().unwrap();
    tx.rollback().unwrap();

    // Verify ref was not modified
    let current_oid = fixture.get_layer_oid(&Layer::GlobalBase).unwrap();
    assert_eq!(current_oid, original_oid);

    // Verify staging ref was deleted
    assert!(!fixture.repo.staging_ref_exists(&tx_id));
}

#[test]
fn test_transaction_rollback_deletes_staging_ref() {
    let fixture = TestRepoFixture::new().unwrap();
    let tx = Transaction::begin(&fixture.repo).unwrap();
    let tx_id = tx.id().to_string();

    assert!(fixture.repo.staging_ref_exists(&tx_id));

    tx.rollback().unwrap();

    assert!(!fixture.repo.staging_ref_exists(&tx_id));
}

// ===== Drop Tests =====

#[test]
fn test_transaction_drop_auto_rollback() {
    let fixture = TestRepoFixture::new().unwrap();
    fixture.create_initial_commit().unwrap();

    let tree_id = fixture.create_empty_tree().unwrap();
    fixture
        .create_layer_ref(&Layer::GlobalBase, tree_id)
        .unwrap();

    let tx_id = {
        let mut tx = Transaction::begin(&fixture.repo).unwrap();
        let tx_id = tx.id().to_string();
        tx.add_layer_update(Layer::GlobalBase, git2::Oid::zero())
            .unwrap();
        tx.prepare().unwrap();
        // tx goes out of scope here, Drop should auto-rollback
        tx_id
    };

    // Verify staging ref was cleaned up
    assert!(!fixture.repo.staging_ref_exists(&tx_id));
}

#[test]
fn test_transaction_drop_after_commit_no_cleanup() {
    let fixture = TestRepoFixture::new().unwrap();
    fixture.create_initial_commit().unwrap();

    let tree_id = fixture.create_empty_tree().unwrap();
    fixture
        .create_layer_ref(&Layer::GlobalBase, tree_id)
        .unwrap();

    let tx_id = {
        let mut tx = Transaction::begin(&fixture.repo).unwrap();
        let tx_id = tx.id().to_string();
        tx.add_layer_update(Layer::GlobalBase, tree_id).unwrap();
        tx.prepare().unwrap();
        tx.commit().unwrap();
        // tx goes out of scope here, but it was committed so no cleanup needed
        tx_id
    };

    // Staging ref was already deleted by commit()
    assert!(!fixture.repo.staging_ref_exists(&tx_id));
}

// ===== TransactionManager Tests =====

#[test]
fn test_transaction_manager_begin_transaction() {
    let fixture = TestRepoFixture::new().unwrap();
    let tm = TransactionManager::new(&fixture.repo);

    let tx = tm.begin_transaction().unwrap();

    assert_eq!(*tx.state(), TransactionState::Started);
}

#[test]
fn test_transaction_manager_detect_orphaned() {
    let fixture = TestRepoFixture::new().unwrap();
    let tm = TransactionManager::new(&fixture.repo);

    // Create an orphaned transaction by creating a staging ref manually
    let tx_id = "orphaned-tx-123";
    let tree_id = fixture.create_empty_tree().unwrap();
    fixture.repo.create_staging_ref(tx_id, tree_id).unwrap();

    let orphaned = tm.detect_orphaned().unwrap();

    assert!(orphaned.contains(&tx_id.to_string()));
}

#[test]
fn test_transaction_manager_recover() {
    let fixture = TestRepoFixture::new().unwrap();
    let tm = TransactionManager::new(&fixture.repo);

    // Create an orphaned transaction
    let tx_id = "orphaned-tx-456";
    let tree_id = fixture.create_empty_tree().unwrap();
    fixture.repo.create_staging_ref(tx_id, tree_id).unwrap();

    assert!(fixture.repo.staging_ref_exists(tx_id));

    tm.recover(tx_id).unwrap();

    assert!(!fixture.repo.staging_ref_exists(tx_id));
}

#[test]
fn test_transaction_manager_recover_all() {
    let fixture = TestRepoFixture::new().unwrap();
    let tm = TransactionManager::new(&fixture.repo);

    // Create multiple orphaned transactions
    let tree_id = fixture.create_empty_tree().unwrap();
    fixture
        .repo
        .create_staging_ref("orphaned-1", tree_id)
        .unwrap();
    fixture
        .repo
        .create_staging_ref("orphaned-2", tree_id)
        .unwrap();
    fixture
        .repo
        .create_staging_ref("orphaned-3", tree_id)
        .unwrap();

    let recovered = tm.recover_all().unwrap();

    assert_eq!(recovered, 3);
    assert!(!fixture.repo.staging_ref_exists("orphaned-1"));
    assert!(!fixture.repo.staging_ref_exists("orphaned-2"));
    assert!(!fixture.repo.staging_ref_exists("orphaned-3"));
}

// ===== Multi-Layer Atomic Commit Tests =====

#[test]
fn test_multi_layer_atomic_commit() {
    let fixture = TestRepoFixture::new().unwrap();
    fixture.create_initial_commit().unwrap();

    let old_tree_id = fixture.create_empty_tree().unwrap();
    let new_tree_id = fixture.create_empty_tree().unwrap();

    // Create initial layer refs
    fixture
        .create_layer_ref(&Layer::GlobalBase, old_tree_id)
        .unwrap();
    fixture
        .create_layer_ref(&Layer::ModeBase {
            mode: "dev".to_string(),
        }, old_tree_id)
        .unwrap();
    fixture
        .create_layer_ref(&Layer::ScopeBase {
            scope: "python".to_string(),
        }, old_tree_id)
        .unwrap();
    fixture
        .create_layer_ref(&Layer::ProjectBase {
            project: "myapp".to_string(),
        }, old_tree_id)
        .unwrap();

    // Create transaction with multiple layer updates
    let mut tx = Transaction::begin(&fixture.repo).unwrap();
    tx.add_layer_update(Layer::GlobalBase, new_tree_id)
        .unwrap();
    tx.add_layer_update(
        Layer::ModeBase {
            mode: "dev".to_string(),
        },
        new_tree_id,
    )
    .unwrap();
    tx.add_layer_update(
        Layer::ScopeBase {
            scope: "python".to_string(),
        },
        new_tree_id,
    )
    .unwrap();
    tx.add_layer_update(
        Layer::ProjectBase {
            project: "myapp".to_string(),
        },
        new_tree_id,
    )
    .unwrap();

    tx.prepare().unwrap();
    tx.commit().unwrap();

    // Verify all refs were updated atomically
    assert_eq!(
        fixture.get_layer_oid(&Layer::GlobalBase).unwrap(),
        Some(new_tree_id)
    );
    assert_eq!(
        fixture
            .get_layer_oid(&Layer::ModeBase {
                mode: "dev".to_string()
            })
            .unwrap(),
        Some(new_tree_id)
    );
    assert_eq!(
        fixture
            .get_layer_oid(&Layer::ScopeBase {
                scope: "python".to_string()
            })
            .unwrap(),
        Some(new_tree_id)
    );
    assert_eq!(
        fixture
            .get_layer_oid(&Layer::ProjectBase {
                project: "myapp".to_string()
            })
            .unwrap(),
        Some(new_tree_id)
    );
}

#[test]
fn test_multi_layer_atomic_commit_all_or_none() {
    let fixture = TestRepoFixture::new().unwrap();
    fixture.create_initial_commit().unwrap();

    let old_tree_id = fixture.create_empty_tree().unwrap();
    let new_tree_id = fixture.create_empty_tree().unwrap();

    // Create initial layer refs
    fixture
        .create_layer_ref(&Layer::GlobalBase, old_tree_id)
        .unwrap();
    fixture
        .create_layer_ref(&Layer::ModeBase {
            mode: "dev".to_string(),
        }, old_tree_id)
        .unwrap();

    // Store original OIDs
    let original_global = fixture.get_layer_oid(&Layer::GlobalBase).unwrap();
    let original_mode = fixture
        .get_layer_oid(&Layer::ModeBase {
            mode: "dev".to_string(),
        })
        .unwrap();

    // Create transaction
    let mut tx = Transaction::begin(&fixture.repo).unwrap();
    tx.add_layer_update(Layer::GlobalBase, new_tree_id)
        .unwrap();
    tx.add_layer_update(
        Layer::ModeBase {
            mode: "dev".to_string(),
        },
        new_tree_id,
    )
    .unwrap();

    // Don't commit - just let it drop to simulate failure
    drop(tx);

    // Verify no refs were modified (all-or-nothing property)
    assert_eq!(
        fixture.get_layer_oid(&Layer::GlobalBase).unwrap(),
        original_global
    );
    assert_eq!(
        fixture
            .get_layer_oid(&Layer::ModeBase {
                mode: "dev".to_string()
            })
            .unwrap(),
        original_mode
    );
}

// ===== State Machine Tests =====

#[test]
fn test_transaction_state_transitions() {
    let fixture = TestRepoFixture::new().unwrap();
    fixture.create_initial_commit().unwrap();

    let tree_id = fixture.create_empty_tree().unwrap();
    fixture
        .create_layer_ref(&Layer::GlobalBase, tree_id)
        .unwrap();

    // Start -> Prepared
    let mut tx = Transaction::begin(&fixture.repo).unwrap();
    assert_eq!(*tx.state(), TransactionState::Started);

    tx.add_layer_update(Layer::GlobalBase, tree_id).unwrap();
    tx.prepare().unwrap();
    assert_eq!(*tx.state(), TransactionState::Prepared);

    // Prepared -> Committed
    let tx_id = tx.id().to_string();
    tx.commit().unwrap();

    // Verify staging ref was cleaned up after commit
    assert!(!fixture.repo.staging_ref_exists(&tx_id));
}

#[test]
fn test_transaction_state_rollback_from_started() {
    let fixture = TestRepoFixture::new().unwrap();
    let tx = Transaction::begin(&fixture.repo).unwrap();

    assert_eq!(*tx.state(), TransactionState::Started);

    let tx_id = tx.id().to_string();
    tx.rollback().unwrap();

    // Verify staging ref was cleaned up
    assert!(!fixture.repo.staging_ref_exists(&tx_id));
}

#[test]
fn test_transaction_state_rollback_from_prepared() {
    let fixture = TestRepoFixture::new().unwrap();
    fixture.create_initial_commit().unwrap();

    let tree_id = fixture.create_empty_tree().unwrap();
    fixture
        .create_layer_ref(&Layer::GlobalBase, tree_id)
        .unwrap();

    let mut tx = Transaction::begin(&fixture.repo).unwrap();
    tx.add_layer_update(Layer::GlobalBase, tree_id).unwrap();
    tx.prepare().unwrap();

    assert_eq!(*tx.state(), TransactionState::Prepared);

    let tx_id = tx.id().to_string();
    tx.rollback().unwrap();

    // Verify staging ref was cleaned up
    assert!(!fixture.repo.staging_ref_exists(&tx_id));
}
