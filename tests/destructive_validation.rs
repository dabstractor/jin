//! Integration tests for workspace validation in destructive operations
//!
//! These tests verify that `validate_workspace_attached()` is correctly called
//! before destructive operations (reset --hard, apply --force) to prevent
//! data loss and provide early detection of detached workspace states.

use std::fs;
use std::path::PathBuf;

// Test fixture helpers from common module
mod common;
use common::fixtures::TestFixture;

// Use serial_test to ensure tests that use std::env::set_current_dir run sequentially
use serial_test::serial;

/// Helper to create a layer reference in the Jin repository
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

/// Helper to initialize a Jin project with repository and context
fn init_jin_project(jin_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    jin::git::JinRepo::create_at(jin_dir)?;
    jin::core::config::ProjectContext::default().save()?;
    Ok(())
}

/// Helper to stage a file and commit it to create a proper workspace state
fn setup_tracked_file(
    fixture: &TestFixture,
    file_path: &str,
    content: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    use jin::git::JinRepo;
    use jin::staging::WorkspaceMetadata;

    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    // Write the file
    fs::write(fixture.path().join(file_path), content)?;

    // Create blob and get hash
    let repo = JinRepo::open_at(&jin_dir)?;
    let oid = repo.inner().blob(content)?;
    let hash = oid.to_string();

    // Create metadata with the file
    let mut metadata = WorkspaceMetadata::new();
    metadata.add_file(PathBuf::from(file_path), hash.clone());
    metadata.save()?;

    Ok(())
}

// ============================================================================
// reset --hard tests
// ============================================================================

#[test]
#[serial]
fn test_reset_hard_rejected_when_files_modified() {
    // Test that reset --hard is rejected when workspace files are modified externally
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize the Jin project (repository and context)
    init_jin_project(&jin_dir).unwrap();

    // Setup a tracked file
    let file_path = "config.txt";
    let original_content = b"original content";
    setup_tracked_file(&fixture, file_path, original_content).unwrap();

    // Modify file externally to create detached state
    fs::write(fixture.path().join(file_path), b"modified content").unwrap();

    // Attempt reset --hard, should be rejected
    let result = jin::commands::reset::execute(jin::cli::ResetArgs {
        soft: false,
        mixed: false,
        hard: true,
        mode: false,
        scope: None,
        project: false,
        global: false,
        force: true, // Skip confirmation for test
    });

    assert!(
        result.is_err(),
        "reset --hard should fail when files are modified"
    );

    // Verify it's a DetachedWorkspace error
    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { details, .. }) => {
            assert!(
                details.contains("modified") || details.contains("Workspace files"),
                "Error should mention file modification, got: {}",
                details
            );
        }
        _ => panic!("Expected DetachedWorkspace error, got: {:?}", result),
    }
}

#[test]
#[serial]
fn test_reset_hard_rejected_when_layer_refs_missing() {
    // Test that reset --hard is rejected when layer refs are missing
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize the Jin project (repository and context)
    init_jin_project(&jin_dir).unwrap();

    // Create a mode ref
    create_layer_ref(&jin_dir, "refs/jin/layers/mode/dev", b"dev content").unwrap();

    // Create metadata with the mode ref
    use jin::staging::WorkspaceMetadata;
    let mut metadata = WorkspaceMetadata::new();
    metadata.applied_layers = vec!["mode/dev".to_string()];
    metadata.save().unwrap();

    // Delete the mode reference to create detached state
    let git_repo = git2::Repository::open_bare(&jin_dir).unwrap();
    git_repo
        .find_reference("refs/jin/layers/mode/dev")
        .unwrap()
        .delete()
        .unwrap();

    // Attempt reset --hard, should be rejected
    let result = jin::commands::reset::execute(jin::cli::ResetArgs {
        soft: false,
        mixed: false,
        hard: true,
        mode: false,
        scope: None,
        project: false,
        global: false,
        force: true,
    });

    assert!(
        result.is_err(),
        "reset --hard should fail when layer refs are missing"
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
        _ => panic!("Expected DetachedWorkspace error, got: {:?}", result),
    }
}

#[test]
#[serial]
fn test_reset_hard_rejected_when_context_invalid() {
    // Test that reset --hard is rejected when active context references deleted mode/scope
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize the Jin project (repository and context)
    init_jin_project(&jin_dir).unwrap();

    // Create a mode
    create_layer_ref(&jin_dir, "refs/jin/layers/mode/production", b"prod content").unwrap();

    // Create context with the active mode
    let context = jin::core::config::ProjectContext {
        mode: Some("production".to_string()),
        ..Default::default()
    };
    context.save().unwrap();

    // Create metadata so validation actually runs
    use jin::staging::WorkspaceMetadata;
    let mut metadata = WorkspaceMetadata::new();
    metadata.applied_layers = vec!["mode/production".to_string()];
    metadata.save().unwrap();

    // Delete the mode reference
    let git_repo = git2::Repository::open_bare(&jin_dir).unwrap();
    git_repo
        .find_reference("refs/jin/layers/mode/production")
        .unwrap()
        .delete()
        .unwrap();

    // Attempt reset --hard, should be rejected
    let result = jin::commands::reset::execute(jin::cli::ResetArgs {
        soft: false,
        mixed: false,
        hard: true,
        mode: false,
        scope: None,
        project: false,
        global: false,
        force: true,
    });

    assert!(
        result.is_err(),
        "reset --hard should fail when context is invalid"
    );

    // Verify it's a DetachedWorkspace error
    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { .. }) => {
            // Expected DetachedWorkspace error
        }
        _ => panic!("Expected DetachedWorkspace error, got: {:?}", result),
    }
}

#[test]
#[serial]
fn test_reset_soft_skips_validation() {
    // Test that reset --soft does NOT validate (not destructive)
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize the Jin project (repository and context)
    init_jin_project(&jin_dir).unwrap();

    // Setup a tracked file
    let file_path = "config.txt";
    let original_content = b"original content";
    setup_tracked_file(&fixture, file_path, original_content).unwrap();

    // Modify file externally (would normally cause detached state)
    fs::write(fixture.path().join(file_path), b"modified content").unwrap();

    // reset --soft should succeed (no validation for non-destructive operations)
    let result = jin::commands::reset::execute(jin::cli::ResetArgs {
        soft: true,
        mixed: false,
        hard: false,
        mode: false,
        scope: None,
        project: false,
        global: false,
        force: false,
    });

    assert!(
        result.is_ok(),
        "reset --soft should skip validation and succeed"
    );
}

#[test]
#[serial]
fn test_reset_mixed_skips_validation() {
    // Test that reset --mixed does NOT validate (not destructive)
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize the Jin project (repository and context)
    init_jin_project(&jin_dir).unwrap();

    // Setup a tracked file
    let file_path = "config.txt";
    let original_content = b"original content";
    setup_tracked_file(&fixture, file_path, original_content).unwrap();

    // Modify file externally (would normally cause detached state)
    fs::write(fixture.path().join(file_path), b"modified content").unwrap();

    // reset --mixed should succeed (no validation for non-destructive operations)
    let result = jin::commands::reset::execute(jin::cli::ResetArgs {
        soft: false,
        mixed: true,
        hard: false,
        mode: false,
        scope: None,
        project: false,
        global: false,
        force: false,
    });

    assert!(
        result.is_ok(),
        "reset --mixed should skip validation and succeed"
    );
}

// ============================================================================
// apply --force tests
// ============================================================================

#[test]
#[serial]
fn test_apply_force_rejected_when_files_modified() {
    // Test that apply --force is rejected when workspace files are modified externally
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize the Jin project (repository and context)
    init_jin_project(&jin_dir).unwrap();

    // Setup a tracked file
    let file_path = "config.txt";
    let original_content = b"original content";
    setup_tracked_file(&fixture, file_path, original_content).unwrap();

    // Modify file externally to create detached state
    fs::write(fixture.path().join(file_path), b"modified content").unwrap();

    // Attempt apply --force, should be rejected
    let result = jin::commands::apply::execute(jin::cli::ApplyArgs {
        force: true,
        dry_run: false,
    });

    assert!(
        result.is_err(),
        "apply --force should fail when files are modified"
    );

    // Verify it's a DetachedWorkspace error
    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { details, .. }) => {
            assert!(
                details.contains("modified") || details.contains("Workspace files"),
                "Error should mention file modification, got: {}",
                details
            );
        }
        _ => panic!("Expected DetachedWorkspace error, got: {:?}", result),
    }
}

#[test]
#[serial]
fn test_apply_force_rejected_when_layer_refs_missing() {
    // Test that apply --force is rejected when layer refs are missing
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize the Jin project (repository and context)
    init_jin_project(&jin_dir).unwrap();

    // Create a mode ref
    create_layer_ref(&jin_dir, "refs/jin/layers/mode/dev", b"dev content").unwrap();

    // Create metadata with the mode ref
    use jin::staging::WorkspaceMetadata;
    let mut metadata = WorkspaceMetadata::new();
    metadata.applied_layers = vec!["mode/dev".to_string()];
    metadata.save().unwrap();

    // Delete the mode reference to create detached state
    let git_repo = git2::Repository::open_bare(&jin_dir).unwrap();
    git_repo
        .find_reference("refs/jin/layers/mode/dev")
        .unwrap()
        .delete()
        .unwrap();

    // Attempt apply --force, should be rejected
    let result = jin::commands::apply::execute(jin::cli::ApplyArgs {
        force: true,
        dry_run: false,
    });

    assert!(
        result.is_err(),
        "apply --force should fail when layer refs are missing"
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
        _ => panic!("Expected DetachedWorkspace error, got: {:?}", result),
    }
}

#[test]
#[serial]
fn test_apply_force_rejected_when_context_invalid() {
    // Test that apply --force is rejected when active context references deleted mode/scope
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize the Jin project (repository and context)
    init_jin_project(&jin_dir).unwrap();

    // Create a mode
    create_layer_ref(&jin_dir, "refs/jin/layers/mode/production", b"prod content").unwrap();

    // Create context with the active mode
    let context = jin::core::config::ProjectContext {
        mode: Some("production".to_string()),
        ..Default::default()
    };
    context.save().unwrap();

    // Create metadata so validation actually runs
    use jin::staging::WorkspaceMetadata;
    let mut metadata = WorkspaceMetadata::new();
    metadata.applied_layers = vec!["mode/production".to_string()];
    metadata.save().unwrap();

    // Delete the mode reference
    let git_repo = git2::Repository::open_bare(&jin_dir).unwrap();
    git_repo
        .find_reference("refs/jin/layers/mode/production")
        .unwrap()
        .delete()
        .unwrap();

    // Attempt apply --force, should be rejected
    let result = jin::commands::apply::execute(jin::cli::ApplyArgs {
        force: true,
        dry_run: false,
    });

    assert!(
        result.is_err(),
        "apply --force should fail when context is invalid"
    );

    // Verify it's a DetachedWorkspace error
    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { .. }) => {
            // Expected DetachedWorkspace error
        }
        _ => panic!("Expected DetachedWorkspace error, got: {:?}", result),
    }
}

#[test]
#[serial]
fn test_apply_without_force_skips_validation() {
    // Test that apply without --force does NOT validate (restores workspace, not destructive)
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize the Jin project (repository and context)
    init_jin_project(&jin_dir).unwrap();

    // Setup a tracked file
    let file_path = "config.txt";
    let original_content = b"original content";
    setup_tracked_file(&fixture, file_path, original_content).unwrap();

    // Modify file externally (would normally cause detached state)
    fs::write(fixture.path().join(file_path), b"modified content").unwrap();

    // apply without --force should fail with dirty check, NOT DetachedWorkspace error
    let result = jin::commands::apply::execute(jin::cli::ApplyArgs {
        force: false,
        dry_run: false,
    });

    // Should fail with "Workspace has uncommitted changes" error, not DetachedWorkspace
    match result {
        Err(jin::core::error::JinError::Other(msg)) => {
            assert!(
                msg.contains("uncommitted changes"),
                "Should get 'uncommitted changes' error, got: {}",
                msg
            );
        }
        Err(jin::core::error::JinError::DetachedWorkspace { .. }) => {
            panic!(
                "apply without --force should NOT validate workspace, got DetachedWorkspace error"
            );
        }
        _ => panic!("Expected 'uncommitted changes' error, got: {:?}", result),
    }
}

// ============================================================================
// Recovery hint tests
// ============================================================================

#[test]
#[serial]
fn test_reset_hard_error_includes_recovery_hint() {
    // Test that reset --hard errors include actionable recovery hints
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize the Jin project (repository and context)
    init_jin_project(&jin_dir).unwrap();

    // Setup a tracked file
    let file_path = "config.txt";
    let original_content = b"original content";
    setup_tracked_file(&fixture, file_path, original_content).unwrap();

    // Modify file externally to create detached state
    fs::write(fixture.path().join(file_path), b"modified content").unwrap();

    // Attempt reset --hard
    let result = jin::commands::reset::execute(jin::cli::ResetArgs {
        soft: false,
        mixed: false,
        hard: true,
        mode: false,
        scope: None,
        project: false,
        global: false,
        force: true,
    });

    // Check error includes recovery hint
    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { recovery_hint, .. }) => {
            assert!(
                !recovery_hint.is_empty(),
                "Recovery hint should not be empty"
            );
            assert!(
                recovery_hint.contains("apply") || recovery_hint.contains("activate"),
                "Recovery hint should suggest an action, got: {}",
                recovery_hint
            );
        }
        _ => panic!("Expected DetachedWorkspace error with recovery hint"),
    }
}

#[test]
#[serial]
fn test_apply_force_error_includes_recovery_hint() {
    // Test that apply --force errors include actionable recovery hints
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize the Jin project (repository and context)
    init_jin_project(&jin_dir).unwrap();

    // Setup a tracked file
    let file_path = "config.txt";
    let original_content = b"original content";
    setup_tracked_file(&fixture, file_path, original_content).unwrap();

    // Modify file externally to create detached state
    fs::write(fixture.path().join(file_path), b"modified content").unwrap();

    // Attempt apply --force
    let result = jin::commands::apply::execute(jin::cli::ApplyArgs {
        force: true,
        dry_run: false,
    });

    // Check error includes recovery hint
    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { recovery_hint, .. }) => {
            assert!(
                !recovery_hint.is_empty(),
                "Recovery hint should not be empty"
            );
            assert!(
                recovery_hint.contains("apply") || recovery_hint.contains("activate"),
                "Recovery hint should suggest an action, got: {}",
                recovery_hint
            );
        }
        _ => panic!("Expected DetachedWorkspace error with recovery hint"),
    }
}

// ============================================================================
// Fresh workspace tests
// ============================================================================

#[test]
#[serial]
fn test_reset_hard_allows_fresh_workspace() {
    // Test that reset --hard passes validation on fresh workspace (no metadata)
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize the Jin project (repository and context)
    init_jin_project(&jin_dir).unwrap();

    // No metadata = fresh workspace, should pass validation
    // (The command will fail with "Nothing to reset", but that's expected behavior,
    // not a DetachedWorkspace error)
    let result = jin::commands::reset::execute(jin::cli::ResetArgs {
        soft: false,
        mixed: false,
        hard: true,
        mode: false,
        scope: None,
        project: false,
        global: false,
        force: true,
    });

    // Should not be a DetachedWorkspace error
    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { .. }) => {
            panic!("Fresh workspace should pass validation, got DetachedWorkspace error");
        }
        _ => {
            // Any other result is acceptable (likely "Nothing to reset")
        }
    }
}

#[test]
#[serial]
fn test_apply_force_allows_fresh_workspace() {
    // Test that apply --force passes validation on fresh workspace (no metadata)
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap().clone();

    fixture.set_jin_dir();
    std::env::set_current_dir(fixture.path()).unwrap();

    // Initialize the Jin project (repository and context)
    init_jin_project(&jin_dir).unwrap();

    // No metadata = fresh workspace, should pass validation
    let result = jin::commands::apply::execute(jin::cli::ApplyArgs {
        force: true,
        dry_run: false,
    });

    // Should not be a DetachedWorkspace error
    match result {
        Err(jin::core::error::JinError::DetachedWorkspace { .. }) => {
            panic!("Fresh workspace should pass validation, got DetachedWorkspace error");
        }
        _ => {
            // Any other result is acceptable
        }
    }
}
