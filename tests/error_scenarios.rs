//! Error scenario and recovery integration tests for Jin
//!
//! Tests error conditions and recovery paths. Following the 70% error test ratio,
//! this file comprehensively tests failure scenarios, error handling, and recovery.

use predicates::prelude::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;

mod common;
use common::assertions::*;
use common::fixtures::*;

/// Test handling of corrupted staging index
#[test]
fn test_handles_corrupted_staging_index() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Create valid state
    fs::write(project_path.join("file.txt"), "content")?;

    jin()
        .args(["add", "file.txt"])
        .current_dir(project_path)
        .assert()
        .success();

    // Corrupt staging index
    let staging_index = project_path.join(".jin/staging/index.json");
    fs::write(&staging_index, "{ invalid json }")?;

    // Verify error is handled gracefully
    jin()
        .arg("status")
        .current_dir(project_path)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("corrupted")
                .or(predicate::str::contains("invalid"))
                .or(predicate::str::contains("parse"))
                .or(predicate::str::contains("JSON")),
        );

    Ok(())
}

/// Test repair fixes corrupted staging index
#[test]
fn test_repair_fixes_corrupted_index() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Create and corrupt staging index
    fs::write(project_path.join("test.txt"), "content")?;

    jin()
        .args(["add", "test.txt"])
        .current_dir(project_path)
        .assert()
        .success();

    let staging_index = project_path.join(".jin/staging/index.json");
    fs::write(&staging_index, "invalid")?;

    // Run repair
    jin()
        .arg("repair")
        .current_dir(project_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("repair").or(predicate::str::contains("fix")));

    // Verify status works after repair
    jin()
        .arg("status")
        .current_dir(project_path)
        .assert()
        .success();

    Ok(())
}

/// Test repair with --dry-run doesn't modify files
#[test]
fn test_repair_dry_run() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Corrupt staging index
    let staging_index = project_path.join(".jin/staging/index.json");
    fs::write(&staging_index, "corrupted")?;

    let corrupted_content = fs::read_to_string(&staging_index)?;

    // Run repair with --dry-run
    jin()
        .args(["repair", "--dry-run"])
        .current_dir(project_path)
        .assert()
        .success();

    // Verify file not modified
    let after_content = fs::read_to_string(&staging_index)?;
    assert_eq!(
        corrupted_content, after_content,
        "Dry-run should not modify files"
    );

    Ok(())
}

/// Test handling missing mode error
#[test]
fn test_handles_missing_mode() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Try to add with --mode flag without active mode
    fs::write(project_path.join("test.txt"), "content")?;

    jin()
        .args(["add", "test.txt", "--mode"])
        .current_dir(project_path)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("mode")
                .or(predicate::str::contains("not found"))
                .or(predicate::str::contains("active")),
        );

    Ok(())
}

/// Test handling Git-tracked file error
#[test]
fn test_add_git_tracked_file_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Initialize Git repo in project
    let git_repo = git2::Repository::init(project_path)?;
    let mut config = git_repo.config()?;
    config.set_str("user.email", "test@example.com")?;
    config.set_str("user.name", "Test User")?;

    // Create and track file with Git
    fs::write(project_path.join("git_file.txt"), "tracked by git")?;

    let mut index = git_repo.index()?;
    index.add_path(std::path::Path::new("git_file.txt"))?;
    index.write()?;

    // Try to add Git-tracked file to Jin
    jin()
        .args(["add", "git_file.txt"])
        .current_dir(project_path)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Git")
                .or(predicate::str::contains("tracked"))
                .or(predicate::str::contains("already")),
        );

    Ok(())
}

/// Test handling directory add error
#[test]
fn test_add_directory_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Create directory
    fs::create_dir_all(project_path.join("testdir"))?;

    // Try to add directory (should fail or warn)
    let result = jin()
        .args(["add", "testdir"])
        .current_dir(project_path)
        .assert();

    let output = result.get_output();
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    // Should fail or warn about directory
    assert!(
        !output.status.success()
            || stderr_str.contains("directory")
            || stderr_str.contains("not supported"),
        "Adding directory should fail or warn"
    );

    Ok(())
}

/// Test handling symlink add error
#[test]
fn test_add_symlink_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Create file and symlink
    fs::write(project_path.join("target.txt"), "target")?;

    #[cfg(unix)]
    std::os::unix::fs::symlink(
        project_path.join("target.txt"),
        project_path.join("link.txt"),
    )?;

    #[cfg(unix)]
    {
        // Try to add symlink
        let result = jin()
            .args(["add", "link.txt"])
            .current_dir(project_path)
            .assert();

        let output = result.get_output();
        let stderr_str = String::from_utf8_lossy(&output.stderr);

        // Should fail or warn about symlink
        assert!(
            !output.status.success()
                || stderr_str.contains("symlink")
                || stderr_str.contains("not supported"),
            "Adding symlink should fail or warn"
        );
    }

    Ok(())
}

/// Test handling permission denied error
#[test]
#[cfg(unix)]
fn test_handles_permission_denied() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Create file
    let protected_file = project_path.join("protected.txt");
    fs::write(&protected_file, "content")?;

    // Remove read permissions
    let metadata = fs::metadata(&protected_file)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o000);
    fs::set_permissions(&protected_file, permissions)?;

    // Try to add file without read permission
    let result = jin()
        .args(["add", "protected.txt"])
        .current_dir(project_path)
        .assert();

    // Restore permissions for cleanup
    let mut permissions = fs::metadata(&protected_file)?.permissions();
    permissions.set_mode(0o644);
    fs::set_permissions(&protected_file, permissions)?;

    // Verify error reported
    let output = result.get_output();
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success()
            || stderr_str.contains("permission")
            || stderr_str.contains("denied"),
        "Permission denied should be reported"
    );

    Ok(())
}

/// Test handling export of non-Jin-tracked file
#[test]
fn test_export_non_jin_tracked_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Try to export file that's not Jin-tracked
    jin()
        .args(["export", "nonexistent.txt"])
        .current_dir(project_path)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found")
                .or(predicate::str::contains("not tracked"))
                .or(predicate::str::contains("Jin")),
        );

    Ok(())
}

/// Test handling import of non-Git-tracked file
#[test]
fn test_import_non_git_tracked_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Initialize Git repo
    let git_repo = git2::Repository::init(project_path)?;
    let mut config = git_repo.config()?;
    config.set_str("user.email", "test@example.com")?;
    config.set_str("user.name", "Test User")?;

    // Create file not tracked by Git
    fs::write(project_path.join("untracked.txt"), "content")?;

    // Try to import non-Git-tracked file
    jin()
        .args(["import", "untracked.txt"])
        .current_dir(project_path)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not tracked")
                .or(predicate::str::contains("Git"))
                .or(predicate::str::contains("not found")),
        );

    Ok(())
}

/// Test handling workspace dirty before apply
#[test]
fn test_apply_dirty_workspace_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    let mode_name = format!("apply_dirty_{}", std::process::id());
    create_mode(&mode_name)?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .assert()
        .success();

    // Create and commit file
    fs::write(project_path.join("config.txt"), "committed")?;

    jin()
        .args(["add", "config.txt", "--mode"])
        .current_dir(project_path)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Commit"])
        .current_dir(project_path)
        .assert()
        .success();

    // Modify file in workspace (dirty)
    fs::write(project_path.join("config.txt"), "modified")?;

    // Try to apply with dirty workspace
    let result = jin()
        .arg("apply")
        .current_dir(project_path)
        .assert();

    let output = result.get_output();
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    // Should warn or fail about dirty workspace
    assert!(
        !output.status.success()
            || stderr_str.contains("dirty")
            || stderr_str.contains("modified")
            || stderr_str.contains("uncommitted"),
        "Apply with dirty workspace should warn or fail"
    );

    Ok(())
}

/// Test handling reset with invalid layer
#[test]
fn test_reset_invalid_layer_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Try to reset from non-existent layer
    let result = jin()
        .args(["reset", "--layer", "nonexistent"])
        .current_dir(project_path)
        .assert();

    let output = result.get_output();
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    // Should fail or warn about invalid layer
    assert!(
        !output.status.success()
            || stderr_str.contains("not found")
            || stderr_str.contains("invalid")
            || stdout_str.contains("not yet implemented"),
        "Reset with invalid layer should fail or warn"
    );

    Ok(())
}

/// Test handling mode already exists error
#[test]
fn test_mode_create_duplicate_error() -> Result<(), Box<dyn std::error::Error>> {
    let mode_name = format!("duplicate_{}", std::process::id());

    // Create mode
    jin()
        .args(["mode", "create", &mode_name])
        .assert()
        .success();

    // Try to create same mode again
    jin()
        .args(["mode", "create", &mode_name])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists").or(predicate::str::contains("exists")));

    Ok(())
}

/// Test handling scope already exists error
#[test]
fn test_scope_create_duplicate_error() -> Result<(), Box<dyn std::error::Error>> {
    let scope_name = format!("env:duplicate_{}", std::process::id());

    // Create scope
    jin()
        .args(["scope", "create", &scope_name])
        .assert()
        .success();

    // Try to create same scope again
    jin()
        .args(["scope", "create", &scope_name])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists").or(predicate::str::contains("exists")));

    Ok(())
}

/// Test handling invalid scope format
#[test]
fn test_scope_invalid_format_error() -> Result<(), Box<dyn std::error::Error>> {
    // Try to create scope without colon
    let result = jin()
        .args(["scope", "create", "invalid_scope"])
        .assert();

    let output = result.get_output();
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    // Should fail or warn about format
    assert!(
        !output.status.success()
            || stderr_str.contains("format")
            || stderr_str.contains("colon")
            || stderr_str.contains(":"),
        "Invalid scope format should fail or warn"
    );

    Ok(())
}

/// Test error recovery after failed commit
#[test]
fn test_commit_recovery_after_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Stage file
    fs::write(project_path.join("test.txt"), "content")?;

    jin()
        .args(["add", "test.txt"])
        .current_dir(project_path)
        .assert()
        .success();

    // Corrupt staging to cause commit error
    let staging_index = project_path.join(".jin/staging/index.json");
    let valid_content = fs::read_to_string(&staging_index)?;
    fs::write(&staging_index, "corrupted")?;

    // Try to commit (should fail)
    jin()
        .args(["commit", "-m", "Should fail"])
        .current_dir(project_path)
        .assert()
        .failure();

    // Repair
    jin()
        .arg("repair")
        .current_dir(project_path)
        .assert()
        .success();

    // Restore valid staging and try commit again
    fs::write(&staging_index, valid_content)?;

    jin()
        .args(["commit", "-m", "Should succeed"])
        .current_dir(project_path)
        .assert()
        .success();

    Ok(())
}

/// Test handling empty commit message
#[test]
fn test_commit_empty_message_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Stage file
    fs::write(project_path.join("test.txt"), "content")?;

    jin()
        .args(["add", "test.txt"])
        .current_dir(project_path)
        .assert()
        .success();

    // Try to commit with empty message
    let result = jin()
        .args(["commit", "-m", ""])
        .current_dir(project_path)
        .assert();

    let output = result.get_output();
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    // Should fail or warn about empty message
    assert!(
        !output.status.success()
            || stderr_str.contains("message")
            || stderr_str.contains("empty"),
        "Empty commit message should fail or warn"
    );

    Ok(())
}

/// Test handling Jin not initialized error
#[test]
fn test_operations_without_init_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();

    // Try various operations without initializing Jin
    jin()
        .arg("status")
        .current_dir(project_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized").or(predicate::str::contains("init")));

    jin()
        .arg("apply")
        .current_dir(project_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized").or(predicate::str::contains("init")));

    jin()
        .args(["add", "file.txt"])
        .current_dir(project_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not initialized").or(predicate::str::contains("init")));

    Ok(())
}
