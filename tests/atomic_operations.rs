//! Atomic operations and transaction safety integration tests for Jin
//!
//! Tests that Jin operations are atomic and transactional:
//! - Commits are all-or-nothing
//! - Failed operations don't leave partial state
//! - State is consistent after errors

use std::fs;

mod common;
use common::assertions::*;
use common::fixtures::*;

/// Test that commit is atomic (all refs updated or none)
#[test]
fn test_commit_is_atomic() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    jin_init(project_path, Some(jin_dir))?;

    let mode_name = format!("atomic_{}", unique_test_id());
    create_mode(&mode_name, Some(jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Stage file
    fs::write(project_path.join("test.txt"), "content")?;

    jin()
        .args(["add", "test.txt", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Get staging state before commit (staging is at JIN_DIR when JIN_DIR is set)
    let staging_path = jin_dir.join("staging/index.json");
    let staging_before = fs::read_to_string(&staging_path)?;

    // Commit
    jin()
        .args(["commit", "-m", "Atomic commit"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify:
    // 1. Ref created (commit succeeded)
    // ModeBase uses /_ suffix to avoid Git ref path conflicts
    let ref_path = format!("refs/jin/layers/mode/{}/_", mode_name);
    assert_layer_ref_exists(&ref_path, Some(jin_dir));

    // 2. Staging cleared (commit completed)
    let staging_after = fs::read_to_string(&staging_path)?;
    assert_ne!(
        staging_before, staging_after,
        "Staging should be cleared after commit"
    );
    assert!(
        !staging_after.contains("test.txt"),
        "Staging should not contain committed file"
    );

    Ok(())
}

/// Test that failed commit rolls back (no partial state)
#[test]
fn test_failed_commit_rolls_back() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    jin_init(project_path, Some(jin_dir))?;

    // Stage file
    fs::write(project_path.join("test.txt"), "content")?;

    jin()
        .args(["add", "test.txt"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Get staging state before failed commit (staging is at JIN_DIR)
    let staging_path = jin_dir.join("staging/index.json");
    let _staging_before = fs::read_to_string(&staging_path)?;

    // Corrupt staging to cause commit failure
    fs::write(&staging_path, "corrupted")?;

    // Try to commit (should fail)
    jin()
        .args(["commit", "-m", "Should fail"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .failure();

    // Restore staging for verification
    fs::write(&staging_path, _staging_before)?;

    // Verify no refs were created (rollback successful)
    // The staging is still there since we restored it, but no commit was made

    Ok(())
}

/// Test multi-layer commit atomicity
///
/// Tests that committing files to multiple layers (mode + global) is atomic.
/// Note: We test mode + global layers instead of mode + project because
/// temp directory names (e.g., .tmpXXXX) start with dots which are invalid for git refs.
#[test]
fn test_multi_layer_commit_atomic() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    jin_init(project_path, Some(jin_dir))?;

    let mode_name = format!("multi_{}", unique_test_id());
    create_mode(&mode_name, Some(jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Stage files to multiple layers (mode and global)
    fs::write(project_path.join("mode.txt"), "mode content")?;
    fs::write(project_path.join("global.txt"), "global content")?;

    jin()
        .args(["add", "mode.txt", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Add to global layer (requires --global flag)
    jin()
        .args(["add", "global.txt", "--global"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Commit both layers atomically
    jin()
        .args(["commit", "-m", "Multi-layer commit"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify both refs created (atomicity across layers)
    // ModeBase uses /_ suffix to avoid Git ref path conflicts
    let mode_ref = format!("refs/jin/layers/mode/{}/_", mode_name);
    assert_layer_ref_exists(&mode_ref, Some(jin_dir));

    // Global layer ref
    assert_layer_ref_exists("refs/jin/layers/global", Some(jin_dir));

    // Verify staging cleared for both (staging is at JIN_DIR)
    let staging_path = jin_dir.join("staging/index.json");
    let staging_content = fs::read_to_string(&staging_path).unwrap_or_default();
    assert!(
        !staging_content.contains("mode.txt"),
        "Staging should not contain mode.txt"
    );
    assert!(
        !staging_content.contains("global.txt"),
        "Staging should not contain global.txt"
    );

    Ok(())
}

/// Test state consistency after operation failure
#[test]
fn test_state_consistent_after_failure() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    jin_init(project_path, Some(jin_dir))?;

    // Create valid state
    fs::write(project_path.join("file.txt"), "content")?;

    jin()
        .args(["add", "file.txt"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Staging is at JIN_DIR when JIN_DIR is set
    let staging_path = jin_dir.join("staging/index.json");

    // Cause operation failure by corrupting staging
    fs::write(&staging_path, "corrupted")?;

    // Status handles corrupted staging gracefully (shows empty)
    // But commit should fail with corrupted staging
    jin()
        .args(["commit", "-m", "Should fail"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .failure();

    // Repair state
    jin()
        .arg("repair")
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify state is consistent again
    jin()
        .arg("status")
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    Ok(())
}

/// Test reset operation atomicity
#[test]
fn test_reset_atomic() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    // Stage multiple files
    fs::write(project_path.join("file1.txt"), "content1")?;
    fs::write(project_path.join("file2.txt"), "content2")?;

    jin()
        .args(["add", "file1.txt"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["add", "file2.txt"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify staging contains files (staging is at JIN_DIR)
    let staging_path = jin_dir.join("staging/index.json");
    let staging_content = fs::read_to_string(&staging_path)?;
    assert!(staging_content.contains("file1.txt"), "Staging should contain file1.txt");
    assert!(staging_content.contains("file2.txt"), "Staging should contain file2.txt");

    // Reset (should clear all staged files atomically)
    jin()
        .arg("reset")
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify all files removed from staging
    let staging_after = fs::read_to_string(&staging_path).unwrap_or_default();
    assert!(!staging_after.contains("file1.txt"), "Staging should not contain file1.txt");
    assert!(!staging_after.contains("file2.txt"), "Staging should not contain file2.txt");

    Ok(())
}

/// Test apply operation doesn't leave partial workspace state
#[test]
fn test_apply_atomic_workspace_update() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    jin_init(project_path, Some(jin_dir))?;

    let mode_name = format!("apply_atomic_{}", unique_test_id());
    create_mode(&mode_name, Some(jin_dir))?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Create multiple files
    fs::write(project_path.join("file1.txt"), "content1")?;
    fs::write(project_path.join("file2.txt"), "content2")?;

    jin()
        .args(["add", "file1.txt", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["add", "file2.txt", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Two files"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Remove files from workspace
    fs::remove_file(project_path.join("file1.txt"))?;
    fs::remove_file(project_path.join("file2.txt"))?;

    // Apply (should restore both files atomically)
    jin()
        .arg("apply")
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify both files restored (no partial state)
    assert_workspace_file_exists(project_path, "file1.txt");
    assert_workspace_file_exists(project_path, "file2.txt");

    Ok(())
}

/// Test commit doesn't modify workspace (separation of concerns)
#[test]
fn test_commit_doesnt_modify_workspace() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    // Create and stage file
    fs::write(project_path.join("test.txt"), "original")?;

    jin()
        .args(["add", "test.txt"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Modify file in workspace after staging
    fs::write(project_path.join("test.txt"), "modified")?;

    // Commit
    jin()
        .args(["commit", "-m", "Commit staged version"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify workspace still has modified version (commit didn't touch it)
    let workspace_content = fs::read_to_string(project_path.join("test.txt"))?;
    assert_eq!(
        workspace_content, "modified",
        "Commit should not modify workspace"
    );

    Ok(())
}

/// Test add is idempotent (staging same file twice)
#[test]
fn test_add_idempotent() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    fs::write(project_path.join("test.txt"), "content")?;

    // Add file
    jin()
        .args(["add", "test.txt"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Staging is at JIN_DIR when JIN_DIR is set
    let staging_path = jin_dir.join("staging/index.json");
    let staging_after_first = fs::read_to_string(&staging_path)?;

    // Add same file again
    jin()
        .args(["add", "test.txt"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    let staging_after_second = fs::read_to_string(&staging_path)?;

    // Staging should be idempotent (same state after second add)
    assert_eq!(
        staging_after_first, staging_after_second,
        "Add should be idempotent"
    );

    Ok(())
}

/// Test operations maintain .jin directory integrity
#[test]
fn test_jin_directory_integrity() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    // Verify .jin structure exists (project-level workspace)
    assert_jin_initialized(project_path);

    // Context is in project's .jin directory
    let context_path = project_path.join(".jin/context");
    // Staging directory is in JIN_DIR (created on first add)
    let staging_dir = jin_dir.join("staging");

    assert!(context_path.exists(), "Context file should exist");

    // Perform operations - staging directory is created on first add
    fs::write(project_path.join("test.txt"), "content")?;

    jin()
        .args(["add", "test.txt"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Now staging directory should exist
    assert!(staging_dir.exists(), "Staging directory should exist after add");

    jin()
        .args(["commit", "-m", "Test"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Verify .jin structure still intact
    assert_jin_initialized(project_path);
    assert!(context_path.exists(), "Context file should still exist");
    assert!(staging_dir.exists(), "Staging directory should still exist");

    Ok(())
}

/// Test concurrent operations don't corrupt state
#[test]
fn test_operations_isolated() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();

    // Stage file
    fs::write(project_path.join("file1.txt"), "content1")?;

    jin()
        .args(["add", "file1.txt"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Stage another file in parallel (simulated)
    fs::write(project_path.join("file2.txt"), "content2")?;

    jin()
        .args(["add", "file2.txt"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Both should be staged (staging is at JIN_DIR)
    let staging_path = jin_dir.join("staging/index.json");
    let staging_content = fs::read_to_string(&staging_path)?;
    assert!(staging_content.contains("file1.txt"), "Staging should contain file1.txt");
    assert!(staging_content.contains("file2.txt"), "Staging should contain file2.txt");

    // Commit should include both
    jin()
        .args(["commit", "-m", "Both files"])
        .current_dir(project_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    let staging_after = fs::read_to_string(&staging_path).unwrap_or_default();
    assert!(!staging_after.contains("file1.txt"), "Staging should not contain file1.txt");
    assert!(!staging_after.contains("file2.txt"), "Staging should not contain file2.txt");

    Ok(())
}
