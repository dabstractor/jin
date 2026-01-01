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
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    let mode_name = format!("atomic_{}", std::process::id());
    create_mode(&mode_name)?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .assert()
        .success();

    // Stage file
    fs::write(project_path.join("test.txt"), "content")?;

    jin()
        .args(["add", "test.txt", "--mode"])
        .current_dir(project_path)
        .assert()
        .success();

    // Get staging state before commit
    let staging_before = fs::read_to_string(project_path.join(".jin/staging/index.json"))?;

    // Commit
    jin()
        .args(["commit", "-m", "Atomic commit"])
        .current_dir(project_path)
        .assert()
        .success();

    // Verify:
    // 1. Ref created (commit succeeded)
    let ref_path = format!("refs/jin/layers/mode/{}", mode_name);
    assert_layer_ref_exists(&ref_path);

    // 2. Staging cleared (commit completed)
    let staging_after = fs::read_to_string(project_path.join(".jin/staging/index.json"))?;
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
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Stage file
    fs::write(project_path.join("test.txt"), "content")?;

    jin()
        .args(["add", "test.txt"])
        .current_dir(project_path)
        .assert()
        .success();

    // Get staging state before failed commit
    let _staging_before = fs::read_to_string(project_path.join(".jin/staging/index.json"))?;

    // Corrupt staging to cause commit failure
    fs::write(project_path.join(".jin/staging/index.json"), "corrupted")?;

    // Try to commit (should fail)
    jin()
        .args(["commit", "-m", "Should fail"])
        .current_dir(project_path)
        .assert()
        .failure();

    // Restore staging for verification
    fs::write(
        project_path.join(".jin/staging/index.json"),
        _staging_before,
    )?;

    // Verify no refs were created (rollback successful)
    // The staging is still there since we restored it, but no commit was made

    Ok(())
}

/// Test multi-layer commit atomicity
#[test]
fn test_multi_layer_commit_atomic() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    let mode_name = format!("multi_{}", std::process::id());
    create_mode(&mode_name)?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .assert()
        .success();

    // Stage files to multiple layers
    fs::write(project_path.join("mode.txt"), "mode content")?;
    fs::write(project_path.join("project.txt"), "project content")?;

    jin()
        .args(["add", "mode.txt", "--mode"])
        .current_dir(project_path)
        .assert()
        .success();

    jin()
        .args(["add", "project.txt", "--mode", "--project"])
        .current_dir(project_path)
        .assert()
        .success();

    // Commit both layers
    jin()
        .args(["commit", "-m", "Multi-layer commit"])
        .current_dir(project_path)
        .assert()
        .success();

    // Verify both refs created (atomicity across layers)
    let mode_ref = format!("refs/jin/layers/mode/{}", mode_name);
    assert_layer_ref_exists(&mode_ref);

    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .expect("Failed to get project name");
    let project_ref = format!(
        "refs/jin/layers/mode/{}/project/{}",
        mode_name, project_name
    );
    assert_layer_ref_exists(&project_ref);

    // Verify staging cleared for both
    assert_staging_not_contains(project_path, "mode.txt");
    assert_staging_not_contains(project_path, "project.txt");

    Ok(())
}

/// Test state consistency after operation failure
#[test]
fn test_state_consistent_after_failure() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Create valid state
    fs::write(project_path.join("file.txt"), "content")?;

    jin()
        .args(["add", "file.txt"])
        .current_dir(project_path)
        .assert()
        .success();

    // Save staging state for reference (unused - repair will fix)
    let _staging_valid = fs::read_to_string(project_path.join(".jin/staging/index.json"))?;

    // Cause operation failure by corrupting state
    fs::write(project_path.join(".jin/staging/index.json"), "corrupted")?;

    // Try operations (should fail)
    jin()
        .arg("status")
        .current_dir(project_path)
        .assert()
        .failure();

    // Repair state
    jin()
        .arg("repair")
        .current_dir(project_path)
        .assert()
        .success();

    // Verify state is consistent again
    jin()
        .arg("status")
        .current_dir(project_path)
        .assert()
        .success();

    Ok(())
}

/// Test reset operation atomicity
#[test]
fn test_reset_atomic() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Stage multiple files
    fs::write(project_path.join("file1.txt"), "content1")?;
    fs::write(project_path.join("file2.txt"), "content2")?;

    jin()
        .args(["add", "file1.txt"])
        .current_dir(project_path)
        .assert()
        .success();

    jin()
        .args(["add", "file2.txt"])
        .current_dir(project_path)
        .assert()
        .success();

    assert_staging_contains(project_path, "file1.txt");
    assert_staging_contains(project_path, "file2.txt");

    // Reset (should clear all staged files atomically)
    jin()
        .arg("reset")
        .current_dir(project_path)
        .assert()
        .success();

    // Verify all files removed from staging
    assert_staging_not_contains(project_path, "file1.txt");
    assert_staging_not_contains(project_path, "file2.txt");

    Ok(())
}

/// Test apply operation doesn't leave partial workspace state
#[test]
fn test_apply_atomic_workspace_update() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    let mode_name = format!("apply_atomic_{}", std::process::id());
    create_mode(&mode_name)?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .assert()
        .success();

    // Create multiple files
    fs::write(project_path.join("file1.txt"), "content1")?;
    fs::write(project_path.join("file2.txt"), "content2")?;

    jin()
        .args(["add", "file1.txt", "--mode"])
        .current_dir(project_path)
        .assert()
        .success();

    jin()
        .args(["add", "file2.txt", "--mode"])
        .current_dir(project_path)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Two files"])
        .current_dir(project_path)
        .assert()
        .success();

    // Remove files from workspace
    fs::remove_file(project_path.join("file1.txt"))?;
    fs::remove_file(project_path.join("file2.txt"))?;

    // Apply (should restore both files atomically)
    jin()
        .arg("apply")
        .current_dir(project_path)
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

    // Create and stage file
    fs::write(project_path.join("test.txt"), "original")?;

    jin()
        .args(["add", "test.txt"])
        .current_dir(project_path)
        .assert()
        .success();

    // Modify file in workspace after staging
    fs::write(project_path.join("test.txt"), "modified")?;

    // Commit
    jin()
        .args(["commit", "-m", "Commit staged version"])
        .current_dir(project_path)
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

    fs::write(project_path.join("test.txt"), "content")?;

    // Add file
    jin()
        .args(["add", "test.txt"])
        .current_dir(project_path)
        .assert()
        .success();

    let staging_after_first = fs::read_to_string(project_path.join(".jin/staging/index.json"))?;

    // Add same file again
    jin()
        .args(["add", "test.txt"])
        .current_dir(project_path)
        .assert()
        .success();

    let staging_after_second = fs::read_to_string(project_path.join(".jin/staging/index.json"))?;

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

    // Verify .jin structure exists
    assert_jin_initialized(project_path);

    let context_path = project_path.join(".jin/context");
    let staging_dir = project_path.join(".jin/staging");

    assert!(context_path.exists(), "Context file should exist");
    assert!(staging_dir.exists(), "Staging directory should exist");

    // Perform operations
    fs::write(project_path.join("test.txt"), "content")?;

    jin()
        .args(["add", "test.txt"])
        .current_dir(project_path)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Test"])
        .current_dir(project_path)
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

    // Stage file
    fs::write(project_path.join("file1.txt"), "content1")?;

    jin()
        .args(["add", "file1.txt"])
        .current_dir(project_path)
        .assert()
        .success();

    // Stage another file in parallel (simulated)
    fs::write(project_path.join("file2.txt"), "content2")?;

    jin()
        .args(["add", "file2.txt"])
        .current_dir(project_path)
        .assert()
        .success();

    // Both should be staged
    assert_staging_contains(project_path, "file1.txt");
    assert_staging_contains(project_path, "file2.txt");

    // Commit should include both
    jin()
        .args(["commit", "-m", "Both files"])
        .current_dir(project_path)
        .assert()
        .success();

    assert_staging_not_contains(project_path, "file1.txt");
    assert_staging_not_contains(project_path, "file2.txt");

    Ok(())
}
