//! Integration tests for `jin apply` conflict pause workflow
//!
//! These tests verify that when merge conflicts are detected:
//! - `.jinmerge` files are created
//! - `.jin/.paused_apply.yaml` is written
//! - Non-conflicting files are still applied
//! - User is instructed to run `jin resolve`

mod common;
use common::fixtures::{create_commit_in_repo, setup_test_repo, unique_test_id};

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

/// Get a Command for the jin binary
fn jin_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

#[test]
fn test_apply_with_conflicts_creates_jinmerge_files() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create a mode for testing
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Activate the mode in the project
    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add the same file to global layer with different content
    let config_path = fixture.path().join("config.json");
    fs::write(
        &config_path,
        r#"{"port": 8080, "debug": true, "version": "1.0"}"#,
    )
    .unwrap();

    jin_cmd()
        .args(["add", "config.json", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Commit to global layer
    create_commit_in_repo(
        fixture.path(),
        "config.json",
        r#"{"port": 8080, "debug": true, "version": "1.0"}"#,
        "Add config to global",
    )
    .unwrap();

    // Now modify and add to mode layer (this will cause conflict)
    fs::write(
        &config_path,
        r#"{"port": 9090, "debug": false, "production": true}"#,
    )
    .unwrap();

    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Commit to mode layer
    create_commit_in_repo(
        fixture.path(),
        "config.json",
        r#"{"port": 9090, "debug": false, "production": true}"#,
        "Add config to mode",
    )
    .unwrap();

    // Remove the file from workspace to test apply
    fs::remove_file(&config_path).unwrap();

    // Run apply - should create .jinmerge file
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Operation paused"))
        .stdout(predicate::str::contains("jin resolve"));

    // Verify .jinmerge file was created
    let jinmerge_path = fixture.path().join("config.json.jinmerge");
    assert!(jinmerge_path.exists(), ".jinmerge file should be created");

    // Verify .jinmerge file has correct format
    let jinmerge_content = fs::read_to_string(&jinmerge_path).unwrap();
    assert!(jinmerge_content.contains("# Jin merge conflict"));
    assert!(jinmerge_content.contains("<<<<<<<"));
    assert!(jinmerge_content.contains("======="));
    assert!(jinmerge_content.contains(">>>>>>>"));
}

#[test]
fn test_apply_with_conflicts_creates_paused_state() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create a mode for testing
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Activate the mode in the project
    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add file to global layer
    let config_path = fixture.path().join("settings.yaml");
    fs::write(&config_path, "key: value1\n").unwrap();

    jin_cmd()
        .args(["add", "settings.yaml", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    create_commit_in_repo(
        fixture.path(),
        "settings.yaml",
        "key: value1\n",
        "Add settings to global",
    )
    .unwrap();

    // Modify and add to mode layer
    fs::write(&config_path, "key: value2\n").unwrap();

    jin_cmd()
        .args(["add", "settings.yaml", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    create_commit_in_repo(
        fixture.path(),
        "settings.yaml",
        "key: value2\n",
        "Add settings to mode",
    )
    .unwrap();

    // Run apply - should create paused state
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify .jin/.paused_apply.yaml was created
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    assert!(
        paused_state_path.exists(),
        "Paused state file should be created"
    );

    // Verify paused state contains valid YAML
    let paused_content = fs::read_to_string(&paused_state_path).unwrap();
    assert!(paused_content.contains("timestamp"));
    assert!(paused_content.contains("conflict_files"));
    assert!(paused_content.contains("applied_files"));
}

#[test]
fn test_apply_with_conflicts_applies_non_conflicting_files() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create a mode for testing
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Activate the mode in the project
    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add a non-conflicting file to global layer
    let safe_path = fixture.path().join("safe.json");
    fs::write(&safe_path, r#"{"safe": true}"#).unwrap();

    jin_cmd()
        .args(["add", "safe.json", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    create_commit_in_repo(
        fixture.path(),
        "safe.json",
        r#"{"safe": true}"#,
        "Add safe file",
    )
    .unwrap();

    // Add conflicting file to global
    let conflict_path = fixture.path().join("conflict.json");
    fs::write(&conflict_path, r#"{"value": 1}"#).unwrap();

    jin_cmd()
        .args(["add", "conflict.json", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    create_commit_in_repo(
        fixture.path(),
        "conflict.json",
        r#"{"value": 1}"#,
        "Add conflict to global",
    )
    .unwrap();

    // Add conflicting file to mode (different value)
    fs::write(&conflict_path, r#"{"value": 2}"#).unwrap();

    jin_cmd()
        .args(["add", "conflict.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    create_commit_in_repo(
        fixture.path(),
        "conflict.json",
        r#"{"value": 2}"#,
        "Add conflict to mode",
    )
    .unwrap();

    // Delete both from workspace
    fs::remove_file(&safe_path).unwrap();
    fs::remove_file(&conflict_path).unwrap();

    // Run apply
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify non-conflicting file was applied
    assert!(safe_path.exists(), "Non-conflicting file should be applied");
    let safe_content = fs::read_to_string(&safe_path).unwrap();
    assert_eq!(safe_content, r#"{"safe": true}"#);

    // Verify conflicting file was NOT applied (only .jinmerge exists)
    assert!(
        !conflict_path.exists(),
        "Conflicting file should NOT be applied"
    );
    let jinmerge_path = fixture.path().join("conflict.json.jinmerge");
    assert!(
        jinmerge_path.exists(),
        ".jinmerge file should exist for conflict"
    );
}

#[test]
fn test_apply_dry_run_with_conflicts_shows_preview() {
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

    // Add conflicting file to global
    let config_path = fixture.path().join("config.toml");
    fs::write(&config_path, "port = 8080\n").unwrap();

    jin_cmd()
        .args(["add", "config.toml", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    create_commit_in_repo(
        fixture.path(),
        "config.toml",
        "port = 8080\n",
        "Add config to global",
    )
    .unwrap();

    // Add to mode (different value)
    fs::write(&config_path, "port = 9090\n").unwrap();

    jin_cmd()
        .args(["add", "config.toml", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    create_commit_in_repo(
        fixture.path(),
        "config.toml",
        "port = 9090\n",
        "Add config to mode",
    )
    .unwrap();

    // Run dry-run apply
    jin_cmd()
        .args(["apply", "--dry-run"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Merge conflicts detected"))
        .stdout(predicate::str::contains("--force"));

    // Verify no files were written in dry-run mode
    let jinmerge_path = fixture.path().join("config.toml.jinmerge");
    assert!(
        !jinmerge_path.exists(),
        ".jinmerge should NOT be created in dry-run mode"
    );

    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    assert!(
        !paused_state_path.exists(),
        "Paused state should NOT be created in dry-run mode"
    );
}

#[test]
fn test_apply_with_multiple_conflicts() {
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

    // Create multiple conflicting files
    for (name, global_val, mode_val) in [
        ("a.json", r#"{"v":1}"#, r#"{"v":2}"#),
        ("b.json", r#"{"v":1}"#, r#"{"v":2}"#),
        ("c.json", r#"{"v":1}"#, r#"{"v":2}"#),
    ] {
        let path = fixture.path().join(name);

        // Add to global
        fs::write(&path, global_val).unwrap();
        jin_cmd()
            .args(["add", name, "--global"])
            .current_dir(fixture.path())
            .env("JIN_DIR", &jin_dir)
            .assert()
            .success();
        create_commit_in_repo(fixture.path(), name, global_val, "Add to global").unwrap();

        // Add to mode
        fs::write(&path, mode_val).unwrap();
        jin_cmd()
            .args(["add", name, "--mode"])
            .current_dir(fixture.path())
            .env("JIN_DIR", &jin_dir)
            .assert()
            .success();
        create_commit_in_repo(fixture.path(), name, mode_val, "Add to mode").unwrap();
    }

    // Run apply
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("3 files"));

    // Verify all 3 .jinmerge files were created
    for name in ["a.json", "b.json", "c.json"] {
        let jinmerge_path = fixture.path().join(format!("{}.jinmerge", name));
        assert!(jinmerge_path.exists(), "{} .jinmerge should exist", name);
    }

    // Verify paused state contains all 3 conflicts
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    let paused_content = fs::read_to_string(&paused_state_path).unwrap();
    assert!(paused_content.contains("a.json"));
    assert!(paused_content.contains("b.json"));
    assert!(paused_content.contains("c.json"));
}

#[test]
fn test_apply_no_conflicts_works_normally() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Add a file to global layer
    let config_path = fixture.path().join("config.json");
    fs::write(&config_path, r#"{"port": 8080}"#).unwrap();

    jin_cmd()
        .args(["add", "config.json", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    create_commit_in_repo(
        fixture.path(),
        "config.json",
        r#"{"port": 8080}"#,
        "Add config",
    )
    .unwrap();

    // Run apply (no conflicts expected)
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Applied 1 files"));

    // Verify file was applied
    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).unwrap();
    assert_eq!(content, r#"{"port": 8080}"#);

    // Verify no paused state was created
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    assert!(
        !paused_state_path.exists(),
        "Paused state should NOT be created when there are no conflicts"
    );
}
