//! Integration tests for jin add --local command
//!
//! Tests the --local flag functionality for adding files to Layer 8 (UserLocal).

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

mod common;
use common::assertions::*;
use common::fixtures::*;

/// Get a Command for the jin binary
fn jin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

// ================== TEST 1: ROUTING TO USERLOCAL ==================

#[test]
fn test_add_local_routes_to_layer_8() -> Result<(), Box<dyn std::error::Error>> {
    // SETUP: Create isolated test environment with Jin initialized
    let fixture = setup_test_repo()?;
    fixture.set_jin_dir();

    // Create test file
    let test_file = fixture.path().join(".config/settings.json");
    fs::create_dir_all(test_file.parent().unwrap())?;
    fs::write(&test_file, r#"{"theme": "dark"}"#)?;

    // ACT: Add file with --local flag
    jin()
        .args(["add", ".config/settings.json", "--local"])
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Staged"))
        .stdout(predicate::str::contains("user-local"));

    // ASSERT: File is staged in Layer 8 (UserLocal)
    // Check the staging index at the correct location (JIN_DIR/staging/index.json)
    let staging_index = fixture.jin_dir.as_ref().unwrap().join("staging/index.json");
    assert!(
        staging_index.exists(),
        "Staging index should exist at {:?}",
        staging_index
    );
    let staging_content = fs::read_to_string(&staging_index)?;
    assert!(
        staging_content.contains(".config/settings.json"),
        "Staging index should contain .config/settings.json. Content:\n{}",
        staging_content
    );
    assert!(
        staging_content.contains("UserLocal") || staging_content.contains("user_local"),
        "Staging index should contain UserLocal layer reference. Content:\n{}",
        staging_content
    );

    Ok(())
}

// ================== TEST 2: REJECTS --MODE FLAG ==================

#[test]
fn test_add_local_rejects_mode_flag() -> Result<(), Box<dyn std::error::Error>> {
    // SETUP: Create isolated test environment with Jin initialized
    let fixture = setup_test_repo()?;
    fixture.set_jin_dir();

    // Create test file
    let test_file = fixture.path().join("config.json");
    fs::write(&test_file, "{}")?;

    // ACT: Try to add with both --local and --mode flags
    jin()
        .args(["add", "config.json", "--local", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Cannot combine --local with other layer flags",
        ));

    Ok(())
}

// ================== TEST 3: REJECTS --GLOBAL FLAG ==================

#[test]
fn test_add_local_rejects_global_flag() -> Result<(), Box<dyn std::error::Error>> {
    // SETUP: Create isolated test environment with Jin initialized
    let fixture = setup_test_repo()?;
    fixture.set_jin_dir();

    // Create test file
    let test_file = fixture.path().join("config.json");
    fs::write(&test_file, "{}")?;

    // ACT: Try to add with both --local and --global flags
    jin()
        .args(["add", "config.json", "--local", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "Cannot combine --local with other layer flags",
        ));

    Ok(())
}

// ================== TEST 4: COMPLETE WORKFLOW ==================

#[test]
fn test_add_local_commit_apply_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // SETUP: Create isolated test environment with Jin initialized
    let fixture = setup_test_repo()?;
    fixture.set_jin_dir();

    // Create test file
    let test_file = fixture.path().join(".local/config.toml");
    fs::create_dir_all(test_file.parent().unwrap())?;
    fs::write(
        &test_file,
        r#"[settings]
theme = "dark"
editor = "vim"
"#,
    )?;

    // STEP 1: Add file with --local flag
    jin()
        .args(["add", ".local/config.toml", "--local"])
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Staged"))
        .stdout(predicate::str::contains("user-local"));

    // ASSERT: File is staged
    let staging_index = fixture.jin_dir.as_ref().unwrap().join("staging/index.json");
    assert!(
        staging_index.exists(),
        "Staging index should exist at {:?}",
        staging_index
    );
    let staging_content = fs::read_to_string(&staging_index)?;
    assert!(
        staging_content.contains(".local/config.toml"),
        "Staging index should contain .local/config.toml. Content:\n{}",
        staging_content
    );

    // STEP 2: Commit the staged file
    jin()
        .args(["commit", "-m", "Add local config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", fixture.jin_dir.as_ref().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Committed"));

    // ASSERT: File is no longer staged after commit
    let staging_content_after = fs::read_to_string(&staging_index)?;
    assert!(
        !staging_content_after.contains(".local/config.toml"),
        "Staging index should not contain .local/config.toml after commit. Content:\n{}",
        staging_content_after
    );

    // STEP 3: Verify file was committed to UserLocal layer
    // Check that the commit was created in Jin repository by verifying
    // the staging index now has no entries
    assert!(
        staging_content_after.contains("\"entries\": {}"),
        "Staging index should have empty entries after commit. Content:\n{}",
        staging_content_after
    );

    Ok(())
}
