//! Integration tests for `jin resolve` command
//!
//! These tests verify that:
//! - `jin resolve <file>` validates .jinmerge files have been manually resolved
//! - Resolved content is written to the workspace
//! - .jinmerge files are cleaned up
//! - Paused state is updated or deleted
//! - Apply operation completes automatically when all conflicts resolved

mod common;
use common::fixtures::{setup_test_repo, unique_test_id};

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

/// Get a Command for the jin binary
fn jin_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

#[test]
fn test_resolve_simple_conflict() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create a mode for testing
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Activate the mode
    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add file to global layer
    let config_path = fixture.path().join("config.json");
    fs::write(&config_path, r#"{"port": 8080}"#).unwrap();

    jin_cmd()
        .args(["add", "config.json", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add config to global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Remove and add to mode with different value (creates conflict scenario)
    fs::write(&config_path, r#"{"port": 9090}"#).unwrap();

    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add config to mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Remove file from workspace to ensure apply creates .jinmerge
    fs::remove_file(&config_path).ok();

    // Run apply - should create .jinmerge file
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Manually create .jinmerge file to simulate conflict (since actual merge conflict
    // detection requires git-backed layers which is complex to set up in tests)
    let jinmerge_path = fixture.path().join("config.json.jinmerge");
    fs::write(
        &jinmerge_path,
        "# Jin merge conflict. Resolve and run 'jin resolve <file>'\n{\"port\": 9090}",
    )
    .unwrap();

    // Manually create paused state
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    fs::create_dir_all(fixture.path().join(".jin")).ok();
    fs::write(
        &paused_state_path,
        r#"timestamp: "2099-01-01T00:00:00Z"
layer_config:
  layers: ["global"]
  mode: Some("test_mode")
  scope: None
  project: None
conflict_files:
  - config.json
applied_files: []
conflict_count: 1
"#,
    )
    .unwrap();

    // Run resolve
    jin_cmd()
        .args(["resolve", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved 1 file"))
        .stdout(predicate::str::contains("Apply operation completed"));

    // Verify resolved file was written to workspace
    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).unwrap();
    assert_eq!(
        content,
        "# Jin merge conflict. Resolve and run 'jin resolve <file>'\n{\"port\": 9090}"
    );

    // Verify .jinmerge file was deleted
    assert!(!jinmerge_path.exists());

    // Verify paused state was deleted
    assert!(!paused_state_path.exists());
}

#[test]
fn test_resolve_all_conflicts() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create a mode for testing
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Activate the mode
    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Manually set up paused state with multiple conflicts
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    fs::create_dir_all(fixture.path().join(".jin")).ok();
    fs::write(
        &paused_state_path,
        r#"timestamp: "2099-01-01T00:00:00Z"
layer_config:
  layers: ["global"]
  mode: Some("test_mode")
  scope: None
  project: None
conflict_files:
  - a.json
  - b.json
  - c.json
applied_files: []
conflict_count: 3
"#,
    )
    .unwrap();

    // Create .jinmerge files
    for name in ["a.json", "b.json", "c.json"] {
        let jinmerge_path = fixture.path().join(format!("{}.jinmerge", name));
        fs::write(&jinmerge_path, r#"{"v":2}"#).unwrap();
    }

    // Run resolve --all
    jin_cmd()
        .args(["resolve", "--all"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved 3 file"))
        .stdout(predicate::str::contains("Apply operation completed"));

    // Verify all files were resolved
    for name in ["a.json", "b.json", "c.json"] {
        let path = fixture.path().join(name);
        assert!(path.exists());

        // Verify .jinmerge files were deleted
        let jinmerge_path = fixture.path().join(format!("{}.jinmerge", name));
        assert!(!jinmerge_path.exists());
    }
}

#[test]
fn test_resolve_invalid_markers() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Manually set up paused state
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    fs::create_dir_all(fixture.path().join(".jin")).ok();
    fs::write(
        &paused_state_path,
        r#"timestamp: "2099-01-01T00:00:00Z"
layer_config:
  layers: ["global"]
  mode: None
  scope: None
  project: None
conflict_files:
  - config.json
applied_files: []
conflict_count: 1
"#,
    )
    .unwrap();

    // Create .jinmerge file with conflict markers
    let jinmerge_path = fixture.path().join("config.json.jinmerge");
    fs::write(
        &jinmerge_path,
        "# Jin merge conflict. Resolve and run 'jin resolve <file>'\n<<<<<<< global/\n{\"port\": 8080}\n=======\n{\"port\": 9090}\n>>>>>>> mode/\n",
    )
    .unwrap();

    // Try to resolve without removing conflict markers - should fail
    jin_cmd()
        .args(["resolve", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Conflict markers still present"));
}

#[test]
fn test_resolve_no_paused_state() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Try to resolve without paused apply operation
    jin_cmd()
        .args(["resolve", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("No paused apply operation found"));
}

#[test]
fn test_resolve_dry_run() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Manually set up paused state
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    fs::create_dir_all(fixture.path().join(".jin")).ok();
    fs::write(
        &paused_state_path,
        r#"timestamp: "2099-01-01T00:00:00Z"
layer_config:
  layers: ["global"]
  mode: None
  scope: None
  project: None
conflict_files:
  - config.json
applied_files: []
conflict_count: 1
"#,
    )
    .unwrap();

    // Create .jinmerge file
    let jinmerge_path = fixture.path().join("config.json.jinmerge");
    fs::write(&jinmerge_path, r#"{"port": 9090}"#).unwrap();

    // Run dry-run resolve
    jin_cmd()
        .args(["resolve", "--dry-run"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Would resolve 1 files"));

    // Verify nothing changed
    assert!(jinmerge_path.exists());
}

#[test]
fn test_resolve_file_not_in_conflict() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Manually set up paused state with one conflict
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    fs::create_dir_all(fixture.path().join(".jin")).ok();
    fs::write(
        &paused_state_path,
        r#"timestamp: "2099-01-01T00:00:00Z"
layer_config:
  layers: ["global"]
  mode: None
  scope: None
  project: None
conflict_files:
  - config.json
applied_files: []
conflict_count: 1
"#,
    )
    .unwrap();

    // Try to resolve a file that's not in conflict
    jin_cmd()
        .args(["resolve", "other.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("not in conflict state"));
}

#[test]
fn test_resolve_partial_conflicts() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Manually set up paused state with two conflicts
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    fs::create_dir_all(fixture.path().join(".jin")).ok();
    fs::write(
        &paused_state_path,
        r#"timestamp: "2099-01-01T00:00:00Z"
layer_config:
  layers: ["global"]
  mode: None
  scope: None
  project: None
conflict_files:
  - a.json
  - b.json
applied_files: []
conflict_count: 2
"#,
    )
    .unwrap();

    // Create both .jinmerge files
    let jinmerge_path_a = fixture.path().join("a.json.jinmerge");
    fs::write(&jinmerge_path_a, r#"{"v":2}"#).unwrap();

    let jinmerge_path_b = fixture.path().join("b.json.jinmerge");
    fs::write(&jinmerge_path_b, r#"{"v":2}"#).unwrap();

    // Resolve only one file
    jin_cmd()
        .args(["resolve", "a.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved 1 file"))
        .stdout(predicate::str::contains("Remaining conflicts: 1"));

    // Verify a.json was resolved
    let a_path = fixture.path().join("a.json");
    assert!(a_path.exists());
    assert!(!jinmerge_path_a.exists());

    // Verify b.json is still in conflict state
    assert!(jinmerge_path_b.exists());

    // Verify paused state still exists
    assert!(paused_state_path.exists());
}
