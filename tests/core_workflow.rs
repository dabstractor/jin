//! Core workflow integration tests for Jin
//!
//! Tests the complete core workflow: init → mode → add → commit → apply
//! Validates that the basic Jin operations work end-to-end.

use predicates::prelude::*;
use std::fs;

mod common;
use common::assertions::*;
use common::fixtures::*;

/// Test that `jin init` creates context and .jin directory
#[test]
fn test_init_creates_context_and_repo() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();

    // Run jin init
    jin()
        .arg("init")
        .current_dir(project_path)
        .assert()
        .success();

    // Verify .jin directory created
    assert_jin_initialized(project_path);

    // Verify context.json created
    let context_path = project_path.join(".jin/context.json");
    assert!(
        context_path.exists(),
        "Context file should exist at {:?}",
        context_path
    );

    Ok(())
}

/// Test mode create and use commands
#[test]
fn test_mode_create_and_use() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Create unique mode name to avoid conflicts
    let mode_name = format!("test_mode_{}", std::process::id());

    // Create mode
    jin()
        .args(["mode", "create", &mode_name])
        .assert()
        .success();

    // Use mode in project
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .assert()
        .success();

    // Verify mode in context
    assert_context_mode(project_path, &mode_name);

    Ok(())
}

/// Test adding files to mode layer
#[test]
fn test_add_files_to_mode_layer() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Create and use mode
    let mode_name = format!("test_mode_{}", std::process::id());
    create_mode(&mode_name)?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .assert()
        .success();

    // Create test file
    let test_file = project_path.join("config.json");
    fs::write(&test_file, r#"{"test": true}"#)?;

    // Add file to mode layer
    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(project_path)
        .assert()
        .success();

    // Verify file in staging
    assert_staging_contains(project_path, "config.json");

    Ok(())
}

/// Test that commit creates layer commit
#[test]
fn test_commit_creates_layer_commit() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Create and use mode
    let mode_name = format!("test_mode_{}", std::process::id());
    create_mode(&mode_name)?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .assert()
        .success();

    // Create and add test file
    fs::write(project_path.join("test.txt"), "content")?;

    jin()
        .args(["add", "test.txt", "--mode"])
        .current_dir(project_path)
        .assert()
        .success();

    // Commit
    jin()
        .args(["commit", "-m", "Test commit"])
        .current_dir(project_path)
        .assert()
        .success();

    // Verify layer ref exists
    let ref_path = format!("refs/jin/layers/mode/{}", mode_name);
    assert_layer_ref_exists(&ref_path);

    // Verify staging cleared after commit
    assert_staging_not_contains(project_path, "test.txt");

    Ok(())
}

/// Test that apply merges to workspace
#[test]
fn test_apply_merges_to_workspace() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Create and use mode
    let mode_name = format!("test_mode_{}", std::process::id());
    create_mode(&mode_name)?;

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .assert()
        .success();

    // Create mode-specific directory and file
    let mode_dir = project_path.join(format!(".{}", mode_name));
    fs::create_dir_all(&mode_dir)?;

    let config_file = mode_dir.join("config.json");
    let test_content = r#"{"mode": "test", "enabled": true}"#;
    fs::write(&config_file, test_content)?;

    // Add and commit
    jin()
        .args(["add", &format!(".{}/config.json", mode_name), "--mode"])
        .current_dir(project_path)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Add mode config"])
        .current_dir(project_path)
        .assert()
        .success();

    // Remove file from workspace to test apply
    fs::remove_file(&config_file)?;
    assert!(!config_file.exists(), "Config file should be removed");

    // Apply
    jin()
        .arg("apply")
        .current_dir(project_path)
        .assert()
        .success();

    // Verify file restored in workspace
    assert_workspace_file_exists(project_path, &format!(".{}/config.json", mode_name));

    let restored_content = fs::read_to_string(&config_file)?;
    assert_eq!(
        restored_content, test_content,
        "Applied file should have original content"
    );

    Ok(())
}

/// Test complete workflow from init to apply
#[test]
fn test_complete_workflow_init_to_apply() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    let project_path = fixture.path();

    // Step 1: Initialize
    jin()
        .arg("init")
        .current_dir(project_path)
        .assert()
        .success();

    assert_jin_initialized(project_path);

    // Step 2: Create and use mode
    let mode_name = format!("workflow_test_{}", std::process::id());

    jin()
        .args(["mode", "create", &mode_name])
        .assert()
        .success();

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .assert()
        .success();

    assert_context_mode(project_path, &mode_name);

    // Step 3: Create test file
    let test_dir = project_path.join(format!(".{}", mode_name));
    fs::create_dir_all(&test_dir)?;

    let config_path = test_dir.join("settings.yaml");
    let config_content = "debug: true\nverbose: false\n";
    fs::write(&config_path, config_content)?;

    // Step 4: Add file to mode
    jin()
        .args(["add", &format!(".{}/settings.yaml", mode_name), "--mode"])
        .current_dir(project_path)
        .assert()
        .success();

    assert_staging_contains(project_path, "settings.yaml");

    // Step 5: Commit
    jin()
        .args(["commit", "-m", "Add workflow settings"])
        .current_dir(project_path)
        .assert()
        .success();

    assert_staging_not_contains(project_path, "settings.yaml");

    // Step 6: Remove file and apply to restore it
    fs::remove_file(&config_path)?;
    assert!(!config_path.exists());

    jin()
        .arg("apply")
        .current_dir(project_path)
        .assert()
        .success();

    // Step 7: Verify file restored with correct content
    assert_workspace_file(
        project_path,
        &format!(".{}/settings.yaml", mode_name),
        config_content,
    );

    // Step 8: Verify status shows clean state
    jin()
        .arg("status")
        .current_dir(project_path)
        .assert()
        .success();

    Ok(())
}

/// Test add without mode flag (project base layer)
#[test]
fn test_add_to_project_base_layer() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Create test file
    fs::write(project_path.join("readme.md"), "# Project\n")?;

    // Add to project base (no flags)
    jin()
        .args(["add", "readme.md"])
        .current_dir(project_path)
        .assert()
        .success();

    assert_staging_contains(project_path, "readme.md");

    // Commit
    jin()
        .args(["commit", "-m", "Add readme"])
        .current_dir(project_path)
        .assert()
        .success();

    Ok(())
}

/// Test error: add non-existent file
#[test]
fn test_add_nonexistent_file_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Try to add non-existent file
    jin()
        .args(["add", "nonexistent.txt"])
        .current_dir(project_path)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found").or(predicate::str::contains("does not exist")),
        );

    Ok(())
}

/// Test error: commit with no staged changes
#[test]
fn test_commit_no_staged_changes_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Try to commit without staging anything
    let result = jin()
        .args(["commit", "-m", "Empty commit"])
        .current_dir(project_path)
        .assert();

    let output = result.get_output();
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    // Should either fail or warn about no changes
    assert!(
        !output.status.success()
            || stderr_str.contains("no changes")
            || stderr_str.contains("nothing"),
        "Expected error or warning about no staged changes"
    );

    Ok(())
}

/// Test error: use mode that doesn't exist
#[test]
fn test_mode_use_nonexistent_error() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Try to use non-existent mode
    jin()
        .args(["mode", "use", "nonexistent_mode_12345"])
        .current_dir(project_path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));

    Ok(())
}

/// Test init in already initialized directory
#[test]
fn test_init_already_initialized() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo()?;
    let project_path = fixture.path();

    // Try to init again
    let result = jin().arg("init").current_dir(project_path).assert();

    // Should either succeed (idempotent) or warn about existing init
    let output = result.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    // Accept success or warning
    assert!(
        output.status.success() || stderr_str.contains("already") || stdout_str.contains("already"),
        "Init should handle already initialized directory gracefully"
    );

    Ok(())
}
