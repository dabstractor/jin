//! Basic CLI integration tests for Jin

use assert_cmd::Command;
use predicates::prelude::*;

/// Get a Command for the jin binary
fn jin() -> Command {
    Command::cargo_bin("jin").unwrap()
}

#[test]
fn test_help() {
    jin()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Phantom Git layer system"));
}

#[test]
fn test_version() {
    jin()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("jin"));
}

#[test]
fn test_init_subcommand() {
    jin()
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_status_subcommand() {
    jin()
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_mode_create_subcommand() {
    // Mode create doesn't require Jin init, it creates the mode in global Jin repo
    // May fail if mode already exists from previous test run
    let result = jin().args(["mode", "create", "testmode"]).assert();
    // Accept either success (new mode) or error (already exists)
    let output = result.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout_str.contains("testmode") || stderr_str.contains("already exists"),
        "Expected mode creation or already exists error"
    );
}

#[test]
fn test_mode_use_subcommand() {
    jin()
        .args(["mode", "use", "claude"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_mode_list_subcommand() {
    jin()
        .args(["mode", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_mode_show_subcommand() {
    jin()
        .args(["mode", "show"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_mode_unset_subcommand() {
    jin()
        .args(["mode", "unset"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_scope_create_subcommand() {
    // Scope create doesn't require Jin init, it creates the scope in global Jin repo
    // May fail if scope already exists from previous test run
    let result = jin().args(["scope", "create", "language:javascript"]).assert();
    // Accept either success (new scope) or error (already exists)
    let output = result.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout_str.contains("language:javascript") || stderr_str.contains("already exists"),
        "Expected scope creation or already exists error"
    );
}

#[test]
fn test_scope_create_with_mode() {
    jin()
        .args(["scope", "create", "language:javascript", "--mode=claude"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_scope_use_subcommand() {
    // Scope use requires Jin to be initialized to load context
    jin()
        .args(["scope", "use", "language:javascript"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized").or(predicate::str::contains("not found")));
}

#[test]
fn test_scope_list_subcommand() {
    jin()
        .args(["scope", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_add_subcommand() {
    // Add is implemented and checks for initialization
    jin()
        .args(["add", ".claude/config.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_add_with_mode_flag() {
    // Add is implemented and checks for initialization
    jin()
        .args(["add", ".claude/config.json", "--mode"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_add_with_scope_flag() {
    // Add is implemented and checks for initialization
    jin()
        .args(["add", ".claude/config.json", "--scope=language:javascript"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_commit_subcommand() {
    jin()
        .args(["commit", "-m", "Test commit"])
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_apply_subcommand() {
    jin()
        .arg("apply")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_apply_dry_run() {
    jin()
        .args(["apply", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_reset_subcommand() {
    jin()
        .arg("reset")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_diff_subcommand() {
    jin()
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_log_subcommand() {
    jin()
        .arg("log")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_context_subcommand() {
    jin()
        .arg("context")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_layers_subcommand() {
    jin()
        .arg("layers")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_list_subcommand() {
    jin()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_link_subcommand() {
    jin()
        .args(["link", "git@github.com:org/config"])
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_fetch_subcommand() {
    jin()
        .arg("fetch")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_pull_subcommand() {
    jin()
        .arg("pull")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_push_subcommand() {
    jin()
        .arg("push")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_sync_subcommand() {
    jin()
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_import_subcommand() {
    jin()
        .args(["import", ".vscode/settings.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_export_subcommand() {
    jin()
        .args(["export", ".claude/config.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_repair_subcommand() {
    jin()
        .arg("repair")
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_invalid_subcommand() {
    jin()
        .arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}
