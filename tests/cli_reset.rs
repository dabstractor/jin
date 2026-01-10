//! Integration tests for `jin reset` command

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Get a Command for the jin binary
fn jin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

#[test]
fn test_reset_requires_initialization() {
    let temp = TempDir::new().unwrap();
    jin()
        .arg("reset")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_reset_mixed_mode() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize
    jin()
        .arg("init")
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and stage a file
    fs::write(project_path.join("config.json"), r#"{"test": true}"#).unwrap();
    jin()
        .args(["add", "config.json"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Reset (default mixed mode)
    jin()
        .arg("reset")
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Unstaged"));

    // Verify file still exists in workspace
    assert!(project_path.join("config.json").exists());
}

#[test]
fn test_reset_soft_mode() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize
    jin()
        .arg("init")
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and stage a file
    fs::write(project_path.join("config.json"), r#"{"test": true}"#).unwrap();
    jin()
        .args(["add", "config.json"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Reset soft mode
    jin()
        .args(["reset", "--soft"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("kept in staging"));
}

#[test]
fn test_reset_hard_mode_with_confirmation() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize
    jin()
        .arg("init")
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and stage a file
    fs::write(project_path.join("config.json"), r#"{"test": true}"#).unwrap();
    jin()
        .args(["add", "config.json"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Reset hard mode (should prompt - will cancel due to no input)
    jin()
        .args(["reset", "--hard"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success(); // Will cancel due to no input, but exit success
}

#[test]
fn test_reset_hard_mode_with_force() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize
    jin()
        .arg("init")
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and stage a file
    fs::write(project_path.join("config.json"), r#"{"test": true}"#).unwrap();
    jin()
        .args(["add", "config.json"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Reset hard mode with force (should skip confirmation)
    jin()
        .args(["reset", "--hard", "--force"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Discarded"));

    // Verify file removed from workspace
    assert!(!project_path.join("config.json").exists());
}

#[test]
fn test_reset_layer_targeting_mode() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize
    jin()
        .arg("init")
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create mode
    let mode_name = format!("test_mode_{}", std::process::id());
    jin()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and stage a file to mode layer
    fs::write(project_path.join("config.json"), r#"{"test": true}"#).unwrap();
    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Reset mode layer
    jin()
        .args(["reset", "--mode"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Unstaged"));
}

#[test]
fn test_reset_global_layer() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize
    jin()
        .arg("init")
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and stage a file to global layer
    fs::write(project_path.join("global.json"), r#"{"global": true}"#).unwrap();
    jin()
        .args(["add", "global.json", "--global"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Reset global layer
    jin()
        .args(["reset", "--global"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Unstaged"));

    // Verify file still exists
    assert!(project_path.join("global.json").exists());
}

#[test]
fn test_reset_empty_staging() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize
    jin()
        .arg("init")
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Reset with empty staging
    jin()
        .arg("reset")
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Nothing to reset"));
}

#[test]
fn test_reset_invalid_layer_combination() {
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();
    let jin_dir = temp.path().join(".jin_global");

    // Initialize
    jin()
        .arg("init")
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Try to reset --project without --mode
    jin()
        .args(["reset", "--project"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("--project requires --mode"));
}

#[test]
fn test_reset_help() {
    jin()
        .args(["reset", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Reset staged or committed changes",
        ))
        .stdout(predicate::str::contains("--soft"))
        .stdout(predicate::str::contains("--mixed"))
        .stdout(predicate::str::contains("--hard"))
        .stdout(predicate::str::contains("--global"))
        .stdout(predicate::str::contains("--force"));
}

#[test]
fn test_reset_hard_force_in_detached_state() {
    // ================== SETUP ==================
    let temp = TempDir::new().unwrap();
    let project_path = temp.path();
    let jin_dir = temp.path().join(".jin_global");

    // ================== STEP 1: INITIALIZE ==================
    jin()
        .arg("init")
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // ================== STEP 2: CREATE AND STAGE FILE ==================
    fs::write(project_path.join("config.json"), r#"{"original": true}"#).unwrap();

    jin()
        .args(["add", "config.json"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // ================== STEP 3: COMMIT AND APPLY TO CREATE WORKSPACE METADATA ==================
    // First, create a complete workflow to establish WorkspaceMetadata
    // This is needed for validate_workspace_attached to have something to check
    jin()
        .args(["commit", "-m", "Initial commit"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .arg("apply")
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // ================== STEP 4: ADD ANOTHER FILE TO STAGING ==================
    // Now add a different file to staging so reset has something to operate on
    fs::write(project_path.join("settings.json"), r#"{"setting": "value"}"#).unwrap();

    jin()
        .args(["add", "settings.json"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // ================== STEP 5: CREATE DETACHED STATE ==================
    // Modify config.json (which is in WorkspaceMetadata) externally.
    // This creates a detached state because WorkspaceMetadata has the original hash.
    fs::write(project_path.join("config.json"), r#"{"modified": true}"#).unwrap();

    // ================== STEP 6: VERIFY reset --hard FAILS ==================
    // Without --force, should fail with detached state error
    jin()
        .args(["reset", "--hard"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("detached")
                .or(predicate::str::contains("modified"))
                .or(predicate::str::contains("Workspace files")),
        );

    // ================== STEP 7: VERIFY reset --hard --force SUCCEEDS ==================
    // With --force, should skip validation and succeed
    jin()
        .args(["reset", "--hard", "--force"])
        .current_dir(project_path)
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Discarded"));

    // ================== STEP 8: VERIFY FILES STATUS ==================
    // After hard reset, settings.json should be removed from staging and workspace
    assert!(!project_path.join("settings.json").exists());
    // config.json should still exist (it was modified, not reset)
    assert!(project_path.join("config.json").exists());
}
