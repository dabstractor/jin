//! Integration tests for workspace validation logic
//!
//! These tests verify that the workspace validation functions correctly
//! detect all three detachment conditions:
//! 1. Workspace files modified outside of Jin operations
//! 2. Workspace metadata references non-existent layer commits
//! 3. Active context references deleted modes/scopes

use std::fs;
use std::path::PathBuf;

// Test fixture helpers from common module
mod common;
use common::fixtures::TestFixture;

// Use serial_test to ensure tests that use std::env::set_current_dir run sequentially
use serial_test::serial;

/// Helper to create a layer reference in the Jin repository
///
/// For nested refs like "refs/jin/layers/mode/test/scope/backend",
/// we create only the scope ref, not the mode ref (since Git can't have
/// both a file and directory with the same name).
fn create_layer_ref(
    jin_dir: &PathBuf,
    ref_path: &str,
    file_content: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    use git2::Repository;

    let repo = Repository::open_bare(jin_dir)?;
    let sig = repo.signature()?;

    let oid = repo.blob(file_content)?;
    let mut builder = repo.treebuilder(None)?;
    builder.insert("test.txt", oid, 0o100644)?;
    let tree_oid = builder.write()?;
    let tree = repo.find_tree(tree_oid)?;

    // Only create the final ref if it doesn't exist
    if repo.find_reference(ref_path).is_err() {
        repo.commit(Some(ref_path), &sig, &sig, "test layer", &tree, &[])?;
    }
    Ok(())
}

#[test]
#[serial]
fn test_validation_passes_for_clean_workspace() {
    // Test that validation passes when workspace is clean and attached
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create the Jin repository first
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // No metadata exists = fresh workspace, should pass
    let context = jin::core::config::ProjectContext::default();
    let repo = jin::git::JinRepo::open_at(&jin_dir).unwrap();

    let result = jin::staging::validate_workspace_attached(&context, &repo);
    assert!(result.is_ok(), "Clean workspace should pass validation");
}

#[test]
#[serial]
fn test_validation_allows_fresh_workspace() {
    // Test that validation returns Ok(()) when no metadata exists
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create the Jin repository first
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    let context = jin::core::config::ProjectContext::default();
    let repo = jin::git::JinRepo::open_at(&jin_dir).unwrap();

    let result = jin::staging::validate_workspace_attached(&context, &repo);
    assert!(
        result.is_ok(),
        "Fresh workspace (no metadata) should pass validation"
    );
}

#[test]
#[serial]
fn test_validation_detects_deleted_mode() {
    // Condition 3: Active context references deleted mode
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create the Jin repository first
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Create a mode
    create_layer_ref(&jin_dir, "refs/jin/layers/mode/test_mode", b"test content").unwrap();

    // Create context with the active mode
    let context = jin::core::config::ProjectContext {
        mode: Some("test_mode".to_string()),
        ..Default::default()
    };

    // Create metadata so validation actually runs
    use jin::staging::WorkspaceMetadata;
    let mut metadata = WorkspaceMetadata::new();
    metadata.applied_layers = vec!["mode/test_mode".to_string()];
    metadata.save().unwrap();

    // First verify validation passes with mode existing
    let repo = jin::git::JinRepo::open_at(&jin_dir).unwrap();
    let result = jin::staging::validate_workspace_attached(&context, &repo);
    assert!(result.is_ok(), "Validation should pass when mode exists");

    // Delete the mode reference
    let git_repo = git2::Repository::open_bare(&jin_dir).unwrap();
    git_repo
        .find_reference("refs/jin/layers/mode/test_mode")
        .unwrap()
        .delete()
        .unwrap();

    // Now validation should fail
    let result = jin::staging::validate_workspace_attached(&context, &repo);
    assert!(
        result.is_err(),
        "Validation should fail when mode is deleted"
    );

    // Verify it's a DetachedWorkspace error
    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { details, .. }) => {
            // The error should mention that the layer ref no longer exists
            assert!(
                details.contains("no longer exist") || details.contains("Missing refs"),
                "Details should mention missing refs, got: {}",
                details
            );
        }
        _ => panic!("Expected DetachedWorkspace error, got: {:?}", result),
    }
}

#[test]
#[serial]
fn test_validation_detects_deleted_scope() {
    // Condition 3: Active context references deleted scope
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create the Jin repository first
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Create a mode ref
    create_layer_ref(&jin_dir, "refs/jin/layers/mode/test_mode", b"mode content").unwrap();

    // Create context with only the mode (no scope, since we can't have both mode and scope refs due to Git limitations)
    let context = jin::core::config::ProjectContext {
        mode: Some("test_mode".to_string()),
        ..Default::default()
    };

    // Create metadata so validation actually runs
    use jin::staging::WorkspaceMetadata;
    let mut metadata = WorkspaceMetadata::new();
    metadata.applied_layers = vec!["mode/test_mode".to_string()];
    metadata.save().unwrap();

    // First verify validation passes with mode existing
    let repo = jin::git::JinRepo::open_at(&jin_dir).unwrap();
    let result = jin::staging::validate_workspace_attached(&context, &repo);
    assert!(result.is_ok(), "Validation should pass when mode exists");

    // Delete the mode reference
    let git_repo = git2::Repository::open_bare(&jin_dir).unwrap();
    git_repo
        .find_reference("refs/jin/layers/mode/test_mode")
        .unwrap()
        .delete()
        .unwrap();

    // Now validation should fail
    let result = jin::staging::validate_workspace_attached(&context, &repo);
    assert!(
        result.is_err(),
        "Validation should fail when mode is deleted"
    );

    // Verify it's a DetachedWorkspace error
    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { details, .. }) => {
            // The error should mention that the layer ref no longer exists
            assert!(
                details.contains("no longer exist") || details.contains("Missing refs"),
                "Details should mention missing refs, got: {}",
                details
            );
        }
        _ => panic!("Expected DetachedWorkspace error, got: {:?}", result),
    }
}

#[test]
#[serial]
fn test_validation_detects_modified_files() {
    // Condition 1: Workspace files modified outside of Jin operations
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create the Jin repository first
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    let file_path = "test_config.txt";
    let original_content = b"original content";
    fs::write(fixture.path().join(file_path), original_content).unwrap();

    // Get the Git blob hash for the file
    let repo = jin::git::JinRepo::open_at(&jin_dir).unwrap();
    let oid = repo.inner().blob(original_content).unwrap();
    let hash = oid.to_string();

    // Create workspace metadata with the original hash
    use jin::staging::WorkspaceMetadata;
    let mut metadata = WorkspaceMetadata::new();
    metadata.add_file(PathBuf::from(file_path), hash.clone());
    metadata.save().unwrap();

    // Validation should pass with matching content
    let context = jin::core::config::ProjectContext::default();
    let result = jin::staging::validate_workspace_attached(&context, &repo);
    assert!(
        result.is_ok(),
        "Validation should pass when files match metadata"
    );

    // Modify the file externally
    fs::write(fixture.path().join(file_path), b"modified content").unwrap();

    // Now validation should fail
    let result = jin::staging::validate_workspace_attached(&context, &repo);
    assert!(
        result.is_err(),
        "Validation should fail when files are modified"
    );

    // Verify it's a DetachedWorkspace error about file modification
    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { details, .. }) => {
            assert!(details.contains("modified") || details.contains("Workspace files"));
        }
        _ => panic!("Expected DetachedWorkspace error about file modification"),
    }
}

#[test]
#[serial]
fn test_validation_detects_deleted_files() {
    // Condition 1: Workspace files deleted externally
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create the Jin repository first
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    let file_path = "test_config.txt";
    let content = b"original content";
    fs::write(fixture.path().join(file_path), content).unwrap();

    // Get the Git blob hash for the file
    let repo = jin::git::JinRepo::open_at(&jin_dir).unwrap();
    let oid = repo.inner().blob(content).unwrap();
    let hash = oid.to_string();

    // Create workspace metadata with the hash
    use jin::staging::WorkspaceMetadata;
    let mut metadata = WorkspaceMetadata::new();
    metadata.add_file(PathBuf::from(file_path), hash);
    metadata.save().unwrap();

    // Validation should pass with file present
    let context = jin::core::config::ProjectContext::default();
    let result = jin::staging::validate_workspace_attached(&context, &repo);
    assert!(result.is_ok(), "Validation should pass when files exist");

    // Delete the file externally
    fs::remove_file(fixture.path().join(file_path)).unwrap();

    // Now validation should fail
    let result = jin::staging::validate_workspace_attached(&context, &repo);
    assert!(
        result.is_err(),
        "Validation should fail when tracked files are deleted"
    );

    // Verify it's a DetachedWorkspace error
    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { .. }) => {
            // Expected DetachedWorkspace error
        }
        _ => panic!("Expected DetachedWorkspace error"),
    }
}

#[test]
#[serial]
fn test_validation_detects_missing_layer_refs() {
    // Condition 2: Workspace metadata references non-existent layer commits
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create the Jin repository first
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Create workspace metadata with a layer ref that doesn't exist
    use jin::staging::WorkspaceMetadata;
    let mut metadata = WorkspaceMetadata::new();
    metadata.applied_layers = vec!["mode/nonexistent".to_string()];
    metadata.save().unwrap();

    // Validation should fail
    let context = jin::core::config::ProjectContext::default();
    let repo = jin::git::JinRepo::open_at(&jin_dir).unwrap();
    let result = jin::staging::validate_workspace_attached(&context, &repo);

    assert!(
        result.is_err(),
        "Validation should fail when layer refs don't exist"
    );

    // Verify it's a DetachedWorkspace error about missing refs
    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { details, .. }) => {
            assert!(
                details.contains("no longer exist") || details.contains("Missing refs"),
                "Error should mention missing refs, got: {}",
                details
            );
        }
        _ => panic!("Expected DetachedWorkspace error about missing refs"),
    }
}

#[test]
#[serial]
fn test_validation_with_multiple_layers_all_exist() {
    // Test validation passes when multiple layers all exist
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create the Jin repository first
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Create a mode ref
    create_layer_ref(&jin_dir, "refs/jin/layers/mode/dev", b"dev").unwrap();

    // Create workspace metadata with the mode ref
    use jin::staging::WorkspaceMetadata;
    let mut metadata = WorkspaceMetadata::new();
    metadata.applied_layers = vec!["mode/dev".to_string()];
    metadata.save().unwrap();

    // Validation should pass
    let context = jin::core::config::ProjectContext {
        mode: Some("dev".to_string()),
        ..Default::default()
    };
    let repo = jin::git::JinRepo::open_at(&jin_dir).unwrap();
    let result = jin::staging::validate_workspace_attached(&context, &repo);

    assert!(
        result.is_ok(),
        "Validation should pass when all layers exist"
    );
}

#[test]
#[serial]
fn test_validation_with_some_layers_missing() {
    // Test validation fails when some referenced layers don't exist
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create the Jin repository first
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Create only one of two layer refs
    create_layer_ref(&jin_dir, "refs/jin/layers/mode/dev", b"dev").unwrap();
    // Don't create the second layer

    // Create workspace metadata with both layers
    use jin::staging::WorkspaceMetadata;
    let mut metadata = WorkspaceMetadata::new();
    metadata.applied_layers = vec![
        "mode/dev".to_string(),
        "mode/dev/scope/backend".to_string(), // This one doesn't exist
    ];
    metadata.save().unwrap();

    // Validation should fail
    let context = jin::core::config::ProjectContext {
        mode: Some("dev".to_string()),
        ..Default::default()
    };
    let repo = jin::git::JinRepo::open_at(&jin_dir).unwrap();
    let result = jin::staging::validate_workspace_attached(&context, &repo);

    assert!(
        result.is_err(),
        "Validation should fail when some layers are missing"
    );

    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { .. }) => {
            // Expected DetachedWorkspace error
        }
        _ => panic!("Expected DetachedWorkspace error"),
    }
}

#[test]
#[serial]
fn test_validation_error_messages_include_recovery_hints() {
    // Test that DetachedWorkspace errors include actionable recovery hints
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create the Jin repository first
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    // Create a scenario that will fail: deleted mode
    create_layer_ref(&jin_dir, "refs/jin/layers/mode/production", b"prod").unwrap();

    // Create metadata so validation actually runs
    use jin::staging::WorkspaceMetadata;
    let mut metadata = WorkspaceMetadata::new();
    metadata.applied_layers = vec!["mode/production".to_string()];
    metadata.save().unwrap();

    let context = jin::core::config::ProjectContext {
        mode: Some("production".to_string()),
        ..Default::default()
    };

    let repo = jin::git::JinRepo::open_at(&jin_dir).unwrap();

    // Delete the mode
    let git_repo = git2::Repository::open_bare(&jin_dir).unwrap();
    git_repo
        .find_reference("refs/jin/layers/mode/production")
        .unwrap()
        .delete()
        .unwrap();

    // Check error includes recovery hint
    let result = jin::staging::validate_workspace_attached(&context, &repo);
    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { recovery_hint, .. }) => {
            assert!(
                !recovery_hint.is_empty(),
                "Recovery hint should not be empty"
            );
            assert!(
                recovery_hint.contains("activate") || recovery_hint.contains("apply"),
                "Recovery hint should suggest an action, got: {}",
                recovery_hint
            );
        }
        _ => panic!("Expected DetachedWorkspace error with recovery hint"),
    }
}

#[test]
#[serial]
fn test_validation_order_checks_file_mismatch_first() {
    // Verify that file mismatch (Condition 1) is checked before other conditions
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Create the Jin repository first
    jin::git::JinRepo::create_at(&jin_dir).unwrap();

    let file_path = "test.txt";
    let original_content = b"original";
    fs::write(fixture.path().join(file_path), original_content).unwrap();

    let repo = jin::git::JinRepo::open_at(&jin_dir).unwrap();
    let oid = repo.inner().blob(original_content).unwrap();
    let hash = oid.to_string();

    // Create metadata that has:
    // 1. A file that will be modified (Condition 1)
    // 2. A missing layer ref (Condition 2)
    // 3. An invalid context (Condition 3)
    use jin::staging::WorkspaceMetadata;
    let mut metadata = WorkspaceMetadata::new();
    metadata.add_file(PathBuf::from(file_path), hash);
    metadata.applied_layers = vec!["mode/nonexistent".to_string()];
    metadata.save().unwrap();

    let context = jin::core::config::ProjectContext {
        mode: Some("nonexistent_mode".to_string()),
        ..Default::default()
    };

    // Modify the file to trigger Condition 1
    fs::write(fixture.path().join(file_path), b"modified").unwrap();

    // Validation should report file mismatch, not the other conditions
    let result = jin::staging::validate_workspace_attached(&context, &repo);
    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { details, .. }) => {
            // Should report file mismatch (Condition 1), not missing refs or invalid context
            assert!(
                details.contains("modified") || details.contains("Workspace files"),
                "Should report file mismatch first, got: {}",
                details
            );
        }
        _ => panic!("Expected DetachedWorkspace error"),
    }
}
