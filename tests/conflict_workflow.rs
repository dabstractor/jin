//! End-to-end integration tests for conflict resolution workflow
//!
//! These tests verify the complete workflow from conflict detection
//! through resolution, including:
//! - .jinmerge file creation during apply conflicts
//! - Paused state persistence and recovery
//! - Resolve command validation and workflow
//! - Status command conflict state display
//! - Error scenarios and edge cases

mod common;
use common::fixtures::{setup_test_repo, unique_test_id};

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

/// Get a Command for the jin binary
fn jin_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
}

/// Helper to create a paused state for testing
fn create_paused_state(fixture: &common::fixtures::TestFixture, conflict_files: &[&str]) {
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    fs::create_dir_all(fixture.path().join(".jin")).ok();

    let conflict_list = conflict_files
        .iter()
        .map(|f| format!("  - {}", f))
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(
        &paused_state_path,
        format!(
            r#"timestamp: "2099-01-01T00:00:00Z"
layer_config:
  layers: ["global"]
  mode: Some("test_mode")
  scope: None
  project: None
conflict_files:
{}
applied_files: []
conflict_count: {}
"#,
            conflict_list,
            conflict_files.len()
        ),
    )
    .unwrap();
}

#[test]
fn test_full_workflow_conflict_to_resolution() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // 1. Create a mode for testing
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // 2. Activate the mode
    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // 3. Add file to global layer
    let config_path = fixture.path().join("config.json");
    fs::write(&config_path, r#"{"port": 8080}"#).unwrap();
    jin_cmd()
        .args(["add", "config.json", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add to global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // 4. Modify and add to mode layer (creates conflict)
    fs::write(&config_path, r#"{"port": 9090}"#).unwrap();
    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add to mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // 5. Remove from workspace
    fs::remove_file(&config_path).unwrap();

    // 6. Run apply - should create .jinmerge
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Operation paused"))
        .stdout(predicate::str::contains("jin resolve"));

    // 7. Verify .jinmerge file
    let jinmerge_path = fixture.path().join("config.json.jinmerge");
    assert!(jinmerge_path.exists());

    // 8. Verify paused state
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    assert!(paused_state_path.exists());

    // 9. Check status shows conflicts
    jin_cmd()
        .arg("status")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Merge conflicts"));

    // 10. Resolve the conflict by providing resolved content
    fs::write(
        &jinmerge_path,
        "# Jin merge conflict. Resolve and run 'jin resolve <file>'\n{\"port\": 9999}",
    )
    .unwrap();

    // 11. Run resolve
    jin_cmd()
        .args(["resolve", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved 1 file"))
        .stdout(predicate::str::contains("Apply operation completed"));

    // 12. Verify resolved file was written
    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("9999"));

    // 13. Verify .jinmerge file was deleted
    assert!(!jinmerge_path.exists());

    // 14. Verify paused state was deleted
    assert!(!paused_state_path.exists());

    // 15. Verify status shows no conflicts
    jin_cmd()
        .arg("status")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Clean").or(predicate::str::contains("No staged")));
}

#[test]
fn test_status_shows_conflicts_during_pause() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create a mode for testing
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create conflict scenario
    let config_path = fixture.path().join("config.json");
    fs::write(&config_path, r#"{"port": 8080}"#).unwrap();
    jin_cmd()
        .args(["add", "config.json", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add to global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    fs::write(&config_path, r#"{"port": 9090}"#).unwrap();
    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add to mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    fs::remove_file(&config_path).unwrap();

    // Run apply to create paused state
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Check status shows conflict state
    jin_cmd()
        .arg("status")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Merge conflicts"))
        .stdout(predicate::str::contains("1 file"))
        .stdout(predicate::str::contains("config.json.jinmerge"))
        .stdout(predicate::str::contains("Resolve with:"))
        .stdout(predicate::str::contains("Detected:"));
}

#[test]
fn test_resolve_validates_conflict_markers() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Manually set up paused state
    create_paused_state(&fixture, &["config.json"]);

    // Create .jinmerge file with conflict markers
    let jinmerge_path = fixture.path().join("config.json.jinmerge");
    fs::write(
        &jinmerge_path,
        "# Jin merge conflict. Resolve and run 'jin resolve <file>'\n<<<<<<< global/\n{\"port\": 8080}\n=======\n{\"port\": 9090}\n>>>>>>> mode/\n",
    ).unwrap();

    // Try to resolve without removing conflict markers - should fail
    jin_cmd()
        .args(["resolve", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Conflict markers still present"));

    // Now remove markers and resolve successfully
    fs::write(
        &jinmerge_path,
        "# Jin merge conflict. Resolve and run 'jin resolve <file>'\n{\"port\": 9999}",
    )
    .unwrap();

    jin_cmd()
        .args(["resolve", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved 1 file"));

    // Verify file was resolved
    let config_path = fixture.path().join("config.json");
    assert!(config_path.exists());
    assert!(!jinmerge_path.exists());
}

#[test]
fn test_apply_creates_multiple_jinmerge_files() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create a mode for testing
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create multiple conflicting files
    // First, add all files to global layer and commit together
    for (name, global_val, _mode_val) in [
        ("a.json", r#"{"v":1}"#, r#"{"v":2}"#),
        ("b.json", r#"{"v":1}"#, r#"{"v":2}"#),
        ("c.json", r#"{"v":1}"#, r#"{"v":2}"#),
    ] {
        let path = fixture.path().join(name);
        fs::write(&path, global_val).unwrap();
        jin_cmd()
            .args(["add", name, "--global"])
            .current_dir(fixture.path())
            .env("JIN_DIR", &jin_dir)
            .assert()
            .success();
    }
    jin_cmd()
        .args(["commit", "-m", "Add all to global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Then, add all files to mode layer with different values and commit together
    for (name, _global_val, mode_val) in [
        ("a.json", r#"{"v":1}"#, r#"{"v":2}"#),
        ("b.json", r#"{"v":1}"#, r#"{"v":2}"#),
        ("c.json", r#"{"v":1}"#, r#"{"v":2}"#),
    ] {
        let path = fixture.path().join(name);
        fs::write(&path, mode_val).unwrap();
        jin_cmd()
            .args(["add", name, "--mode"])
            .current_dir(fixture.path())
            .env("JIN_DIR", &jin_dir)
            .assert()
            .success();
    }
    jin_cmd()
        .args(["commit", "-m", "Add all to mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

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
        assert!(jinmerge_path.exists());
    }

    // Verify paused state contains all 3 conflicts
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    let paused_content = fs::read_to_string(&paused_state_path).unwrap();
    assert!(paused_content.contains("a.json"));
    assert!(paused_content.contains("b.json"));
    assert!(paused_content.contains("c.json"));
    assert!(paused_content.contains("conflict_count: 3"));
}

#[test]
fn test_resolve_partial_updates_state() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Manually set up paused state with two conflicts
    create_paused_state(&fixture, &["a.json", "b.json"]);

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
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    assert!(paused_state_path.exists());

    // Verify paused state was updated
    let paused_content = fs::read_to_string(&paused_state_path).unwrap();
    assert!(paused_content.contains("b.json"));
    assert!(!paused_content.contains("a.json"));
    assert!(paused_content.contains("conflict_count: 1"));
}

#[test]
fn test_resolve_all_completes_apply_operation() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create a mode for testing
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Manually set up paused state with multiple conflicts
    create_paused_state(&fixture, &["a.json", "b.json", "c.json"]);

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

    // Verify paused state was deleted
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    assert!(!paused_state_path.exists());
}

// ========== Error Scenario Tests ==========

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
fn test_resolve_file_not_in_conflict() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Manually set up paused state with one conflict
    create_paused_state(&fixture, &["config.json"]);

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
fn test_resolve_empty_jinmerge_fails() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Manually set up paused state
    create_paused_state(&fixture, &["config.json"]);

    // Create empty .jinmerge file
    let jinmerge_path = fixture.path().join("config.json.jinmerge");
    fs::write(
        &jinmerge_path,
        "# Jin merge conflict. Resolve and run 'jin resolve <file>'\n",
    )
    .unwrap();

    // Try to resolve - should fail
    jin_cmd()
        .args(["resolve", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Empty resolution"));
}

#[test]
fn test_resolve_missing_jinmerge_file() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Manually set up paused state
    create_paused_state(&fixture, &["config.json"]);

    // Don't create .jinmerge file - try to resolve without it
    jin_cmd()
        .args(["resolve", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("No .jinmerge file found"));
}

#[test]
fn test_apply_non_conflicting_files_still_applied() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create a mode for testing
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

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

    // Add conflicting file to global
    let conflict_path = fixture.path().join("conflict.json");
    fs::write(&conflict_path, r#"{"value": 1}"#).unwrap();

    jin_cmd()
        .args(["add", "conflict.json", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Commit both files to global layer together
    jin_cmd()
        .args(["commit", "-m", "Add files to global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add conflicting file to mode (different value)
    fs::write(&conflict_path, r#"{"value": 2}"#).unwrap();

    jin_cmd()
        .args(["add", "conflict.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add conflict to mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

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
    assert!(safe_path.exists());
    let safe_content = fs::read_to_string(&safe_path).unwrap();
    // JSON is pretty-printed by Jin, so check for the key-value pair
    assert!(safe_content.contains("\"safe\": true"));

    // Verify conflicting file was NOT applied (only .jinmerge exists)
    assert!(!conflict_path.exists());
    let jinmerge_path = fixture.path().join("conflict.json.jinmerge");
    assert!(jinmerge_path.exists());
}

// ========== Dry Run Tests ==========

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

    jin_cmd()
        .args(["commit", "-m", "Add config to global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add to mode (different value)
    fs::write(&config_path, "port = 9090\n").unwrap();

    jin_cmd()
        .args(["add", "config.toml", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add to mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

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
    assert!(!jinmerge_path.exists());

    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    assert!(!paused_state_path.exists());
}

#[test]
fn test_resolve_dry_run_shows_preview() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Manually set up paused state
    create_paused_state(&fixture, &["config.json"]);

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

    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    assert!(paused_state_path.exists());
}

// ========== Status Graceful Degradation ==========

#[test]
fn test_status_handles_corrupted_paused_state() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create corrupted paused state
    let paused_state_path = fixture.path().join(".jin/.paused_apply.yaml");
    fs::create_dir_all(fixture.path().join(".jin")).ok();
    fs::write(&paused_state_path, "invalid: yaml: content: [").unwrap();

    // Status should not crash
    jin_cmd()
        .arg("status")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // The corrupted state file should still exist (not auto-deleted)
    assert!(paused_state_path.exists());
}

// ========== Additional Edge Case Tests ==========

#[test]
fn test_resolve_with_multiple_conflict_markers_fails() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Manually set up paused state
    create_paused_state(&fixture, &["config.json"]);

    // Create .jinmerge file with multiple conflict marker sets
    let jinmerge_path = fixture.path().join("config.json.jinmerge");
    fs::write(
        &jinmerge_path,
        "# Jin merge conflict. Resolve and run 'jin resolve <file>'\n\
         <<<<<<< global/\n\
         {\"a\": 1}\n\
         =======\n\
         {\"a\": 2}\n\
         >>>>>>> mode/\n\
         <<<<<<< global/\n\
         {\"b\": 1}\n\
         =======\n\
         {\"b\": 2}\n\
         >>>>>>> mode/\n",
    )
    .unwrap();

    // Try to resolve - should fail due to conflict markers
    jin_cmd()
        .args(["resolve", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Conflict markers still present"));
}

#[test]
fn test_apply_force_with_conflicts_applies_non_conflicting() {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create a mode for testing
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add non-conflicting file
    let safe_path = fixture.path().join("safe.json");
    fs::write(&safe_path, r#"{"safe": true}"#).unwrap();
    jin_cmd()
        .args(["add", "safe.json", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add conflicting file
    let conflict_path = fixture.path().join("conflict.json");
    fs::write(&conflict_path, r#"{"v": 1}"#).unwrap();
    jin_cmd()
        .args(["add", "conflict.json", "--global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Commit both files to global layer together
    jin_cmd()
        .args(["commit", "-m", "Add files to global"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    fs::write(&conflict_path, r#"{"v": 2}"#).unwrap();
    jin_cmd()
        .args(["add", "conflict.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();
    jin_cmd()
        .args(["commit", "-m", "Add to mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Remove from workspace
    fs::remove_file(&safe_path).unwrap();
    fs::remove_file(&conflict_path).unwrap();

    // Run apply with --force
    jin_cmd()
        .args(["apply", "--force"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify non-conflicting file was applied
    assert!(safe_path.exists());

    // Verify conflict has .jinmerge file
    let jinmerge_path = fixture.path().join("conflict.json.jinmerge");
    assert!(jinmerge_path.exists());
}

// ========== Structured File Auto-Merge Tests ==========

/// Test that structured files (JSON) automatically deep merge across layers
/// without creating .jinmerge conflict files.
///
/// This test uses ModeBase (Layer 2) and ModeProject (Layer 5) layers to verify
/// that structured files deep merge using RFC 7396 semantics instead of creating
/// conflict files.
///
/// This is a regression test for the bug where structured files were incorrectly
/// creating .jinmerge files even when content could be deep merged. After the fix
/// in S1-S2, structured files should always merge using RFC 7396 semantics.
#[test]
fn test_structured_file_auto_merge() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create and activate a mode
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add config.json to ModeBase layer (Layer 2)
    let config_path = fixture.path().join("config.json");
    fs::write(&config_path, r#"{"common": {"a": 1}, "mode": true}"#)?;

    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add to mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Modify config.json and add to ModeProject layer (Layer 5)
    // Note: Use --mode --project to add to mode-project layer
    fs::write(
        &config_path,
        r#"{"common": {"a": 1, "b": 2}, "project": false}"#,
    )?;

    jin_cmd()
        .args(["add", "config.json", "--mode", "--project"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add to project"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Remove from workspace to test apply behavior
    fs::remove_file(&config_path)?;

    // Run apply - should NOT create .jinmerge (structured files auto-merge)
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify no .jinmerge file was created
    let jinmerge_path = fixture.path().join("config.json.jinmerge");
    assert!(
        !jinmerge_path.exists(),
        "No .jinmerge should exist for structured files that can deep merge. \
        This indicates structured files are incorrectly being treated as conflicts."
    );

    // Verify merged file exists
    assert!(config_path.exists(), "Merged config.json should exist");

    // Verify merged content contains both layers' keys with correct precedence
    let content = fs::read_to_string(&config_path)?;

    // Common key should exist
    assert!(
        content.contains(r#""common":"#),
        "Merged content should contain 'common' key. Content: {}",
        content
    );

    // common.a should be 1 (same value in both layers)
    assert!(
        content.contains(r#""a": 1"#),
        "Merged content should contain 'a: 1' from both layers. Content: {}",
        content
    );

    // common.b should be 2 (from ProjectBase only)
    assert!(
        content.contains(r#""b": 2"#),
        "Merged content should contain 'b: 2' from ProjectBase layer. Content: {}",
        content
    );

    // mode should be true (from ModeBase only)
    assert!(
        content.contains(r#""mode": true"#),
        "Merged content should contain 'mode: true' from ModeBase layer. Content: {}",
        content
    );

    // project should be false (from ProjectBase only)
    assert!(
        content.contains(r#""project": false"#),
        "Merged content should contain 'project: false' from ProjectBase layer. Content: {}",
        content
    );

    Ok(())
}

/// Test nested object deep merge across layers
///
/// Scenario:
/// - Layer 2 (ModeBase): {"config": {"database": {"host": "localhost", "port": 5432}}, "app": "base"}
/// - Layer 7 (ProjectBase): {"config": {"database": {"port": 5433, "ssl": true}}, "app": "override"}
///
/// Expected result after deep merge:
/// - config.database.host: "localhost" (preserved from ModeBase)
/// - config.database.port: 5433 (overridden by ProjectBase)
/// - config.database.ssl: true (added from ProjectBase)
/// - app: "override" (overridden by ProjectBase)
///
/// This test verifies that the deep merge engine correctly handles nested JSON
/// structures with 3-level nesting and follows RFC 7396 JSON Merge Patch semantics
/// with Jin's layer precedence (higher layer wins).
#[test]
fn test_nested_object_deep_merge() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create and activate a mode
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add config.json to ModeBase layer (Layer 2) with 3-level nested structure
    let config_path = fixture.path().join("config.json");
    fs::write(
        &config_path,
        r#"{"config": {"database": {"host": "localhost", "port": 5432}}, "app": "base"}"#,
    )?;

    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add base config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Overwrite config.json and add to ProjectBase layer (Layer 7)
    // Note: No flags needed for ProjectBase (it's the default layer)
    fs::write(
        &config_path,
        r#"{"config": {"database": {"port": 5433, "ssl": true}}, "app": "override"}"#,
    )?;

    jin_cmd()
        .args(["add", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add override config"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify no conflict file created (clean merge expected)
    let merge_conflict_path = config_path.with_extension(".jinmerge");
    assert!(
        !merge_conflict_path.exists(),
        "No conflict file should be created for mergeable nested objects"
    );

    // Apply merge to workspace
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify merged result matches expected output
    let merged_content = fs::read_to_string(&config_path)?;
    let merged: serde_json::Value = serde_json::from_str(&merged_content)?;

    // Verify nested merge behavior
    assert_eq!(
        merged["config"]["database"]["host"], "localhost",
        "host should be preserved from ModeBase"
    );
    assert_eq!(
        merged["config"]["database"]["port"], 5433,
        "port should be overridden by ProjectBase"
    );
    assert_eq!(
        merged["config"]["database"]["ssl"], true,
        "ssl should be added from ProjectBase"
    );
    assert_eq!(
        merged["app"], "override",
        "app should be overridden by ProjectBase"
    );

    Ok(())
}

/// Test array key-based merging across layers
///
/// Scenario:
/// - Layer 2 (ModeBase): [{"id": "1", "name": "task1", "status": "pending"}]
/// - Layer 7 (ProjectBase): [{"id": "1", "priority": "high"}, {"id": "2", "name": "task2"}]
///
/// Expected result after deep merge:
/// - Item id=1: Merged with all fields {"id": "1", "name": "task1", "status": "pending", "priority": "high"}
/// - Item id=2: Appended from overlay {"id": "2", "name": "task2"}
/// - Total items: 2 (no duplicates)
///
/// This test verifies that the deep merge engine correctly handles arrays
/// with key-based matching (id/name fields) following RFC 7396 semantics
/// with Jin's layer precedence.
///
/// Note: Key fields must have string values for key-based merging to work.
/// The default key fields ["id", "name"] are checked in priority order.
#[test]
fn test_array_key_based_merge() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = setup_test_repo().unwrap();
    let jin_dir = fixture.jin_dir.clone().unwrap();

    // Create and activate a mode
    let mode_name = format!("test_mode_{}", unique_test_id());
    jin_cmd()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["mode", "use", &mode_name])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Add config.json to ModeBase layer (Layer 2) with keyed array
    let config_path = fixture.path().join("config.json");
    fs::write(
        &config_path,
        r#"[{"id": "1", "name": "task1", "status": "pending"}]"#,
    )?;

    jin_cmd()
        .args(["add", "config.json", "--mode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add base array"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Overwrite config.json and add to ProjectBase layer (Layer 7)
    // Note: No flags needed for ProjectBase (it's the default layer)
    fs::write(
        &config_path,
        r#"[{"id": "1", "priority": "high"}, {"id": "2", "name": "task2"}]"#,
    )?;

    jin_cmd()
        .args(["add", "config.json"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin_cmd()
        .args(["commit", "-m", "Add overlay array"])
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify no conflict file created (clean merge expected)
    let merge_conflict_path = config_path.with_extension(".jinmerge");
    assert!(
        !merge_conflict_path.exists(),
        "No conflict file should be created for mergeable arrays"
    );

    // Apply merge to workspace
    jin_cmd()
        .arg("apply")
        .current_dir(fixture.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Verify merged result matches expected output
    let merged_content = fs::read_to_string(&config_path)?;
    let merged: serde_json::Value = serde_json::from_str(&merged_content)?;

    // CRITICAL ASSERTIONS for array merging

    // 1. Array length check (should be 2, not 3)
    let merged_array = merged
        .as_array()
        .expect("Merged value should be an array");
    assert_eq!(
        merged_array.len(),
        2,
        "Array should have 2 items after merge (id=1 merged, id=2 appended)"
    );

    // 2. Find item with id=1 and verify all fields present
    let item_1 = merged_array
        .iter()
        .find(|v| v.get("id").and_then(|id| id.as_str()) == Some("1"))
        .expect("Item with id=1 should exist");

    assert_eq!(
        item_1.get("name").and_then(|n| n.as_str()),
        Some("task1"),
        "name should be preserved from ModeBase (Layer 2)"
    );
    assert_eq!(
        item_1.get("status").and_then(|s| s.as_str()),
        Some("pending"),
        "status should be preserved from ModeBase (Layer 2)"
    );
    assert_eq!(
        item_1.get("priority").and_then(|p| p.as_str()),
        Some("high"),
        "priority should be added from ProjectBase (Layer 7)"
    );

    // 3. Find item with id=2 and verify it was appended
    let item_2 = merged_array
        .iter()
        .find(|v| v.get("id").and_then(|id| id.as_str()) == Some("2"))
        .expect("Item with id=2 should exist (appended from overlay)");

    assert_eq!(
        item_2.get("name").and_then(|n| n.as_str()),
        Some("task2"),
        "name should be present for id=2"
    );

    // 4. Verify order: id=1 before id=2 (base order preserved)
    let ids: Vec<_> = merged_array
        .iter()
        .filter_map(|v| v.get("id").and_then(|id| id.as_str()))
        .collect();
    assert_eq!(
        ids,
        vec!["1", "2"],
        "Order should be preserved: base items first, new items appended"
    );

    Ok(())
}
