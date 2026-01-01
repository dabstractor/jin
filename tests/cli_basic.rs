//! Basic CLI integration tests for Jin

use assert_cmd::Command;
use predicates::prelude::*;

/// Get a Command for the jin binary
fn jin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_jin"))
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
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    // Set JIN_DIR to isolated directory
    let jin_dir = temp.path().join(".jin_global");
    std::env::set_var("JIN_DIR", &jin_dir);

    jin()
        .arg("init")
        .current_dir(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized Jin"));
}

#[test]
fn test_status_subcommand() {
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    // Run in isolated environment
    jin()
        .arg("status")
        .current_dir(temp.path())
        .env("JIN_DIR", temp.path().join(".jin_global"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_status_with_active_mode() {
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    // Use temp path for unique JIN_DIR
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create a unique mode name
    let mode_name = format!("test_mode_{}", std::process::id());

    // Create mode
    jin()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Use mode
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Check status shows active mode
    jin()
        .arg("status")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Mode:"))
        .stdout(predicate::str::contains(&mode_name))
        .stdout(predicate::str::contains("(active)"))
        .stdout(predicate::str::contains("Workspace state: Clean"));
}

#[test]
fn test_status_dirty_workspace() {
    use std::fs;
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    // Use temp path for unique JIN_DIR
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create a unique mode name
    let mode_name = format!("test_mode_{}", std::process::id());

    // Create mode
    jin()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Use mode
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and add a file
    let test_file = temp.path().join("config.json");
    fs::write(&test_file, r#"{"test": true}"#).unwrap();

    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Commit the file
    jin()
        .args(["commit", "-m", "Add config"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Apply to create workspace metadata
    jin()
        .arg("apply")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Modify the file to make workspace dirty
    fs::write(&test_file, r#"{"test": false}"#).unwrap();

    // Check status shows dirty workspace
    jin()
        .arg("status")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Workspace state: Dirty"))
        .stdout(predicate::str::contains("config.json (modified)"))
        .stdout(predicate::str::contains("jin diff"));
}

#[test]
fn test_status_with_staged_files() {
    use std::fs;
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    // Use temp path for unique JIN_DIR
    let jin_dir = temp.path().join(".jin_global");

    // Initialize Jin
    jin()
        .arg("init")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create a unique mode name
    let mode_name = format!("test_mode_{}", std::process::id());

    // Create mode
    jin()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Use mode
    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and add a file
    let test_file = temp.path().join("settings.yaml");
    fs::write(&test_file, "key: value").unwrap();

    jin()
        .args(["add", "settings.yaml", "--mode"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Check status shows staged changes
    jin()
        .arg("status")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Staged changes"))
        .stdout(predicate::str::contains("settings.yaml"))
        .stdout(predicate::str::contains("mode-base"))
        .stdout(predicate::str::contains("jin commit"));
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
    let result = jin()
        .args(["scope", "create", "language:javascript"])
        .assert();
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
        .stderr(
            predicate::str::contains("Jin not initialized")
                .or(predicate::str::contains("not found")),
        );
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
    // First create a test file, then try to add without initializing jin
    let temp = tempfile::tempdir().unwrap();
    let test_file = temp.path().join("config.json");
    std::fs::write(&test_file, "{}").unwrap();

    jin()
        .current_dir(temp.path())
        .args(["add", "config.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_add_with_mode_flag() {
    // Add with --mode flag requires initialization first (before checking for active mode)
    let temp = tempfile::tempdir().unwrap();
    let test_file = temp.path().join("config.json");
    std::fs::write(&test_file, "{}").unwrap();

    jin()
        .current_dir(temp.path())
        .args(["add", "config.json", "--mode"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_add_with_scope_flag() {
    // Add with --scope flag works without initialization (uses default context)
    let temp = tempfile::tempdir().unwrap();
    let test_file = temp.path().join("config.json");
    std::fs::write(&test_file, "{}").unwrap();

    jin()
        .current_dir(temp.path())
        .args(["add", "config.json", "--scope=language:javascript"])
        .assert()
        .failure();
    // Fails because file not found relative to where jin is run
}

#[test]
fn test_commit_subcommand() {
    // Commit is a stub - will be implemented later
    jin()
        .args(["commit", "-m", "Test commit"])
        .assert()
        .success()
        .stdout(predicate::str::contains("not yet implemented"));
}

#[test]
fn test_apply_subcommand() {
    // Apply requires Jin initialization
    jin()
        .arg("apply")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_apply_dry_run() {
    // Apply --dry-run also requires Jin initialization
    jin()
        .args(["apply", "--dry-run"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_reset_subcommand() {
    // Reset requires Jin initialization
    jin()
        .arg("reset")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_diff_subcommand() {
    // Diff requires Jin initialization
    jin()
        .arg("diff")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_log_subcommand() {
    // Log requires Jin initialization
    jin()
        .arg("log")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
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
    // Layers requires Jin initialization
    jin()
        .arg("layers")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_list_subcommand() {
    // List requires Jin initialization
    jin()
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_link_subcommand() {
    // Link command may fail for different reasons:
    // - Remote already exists (if run after other tests)
    // - Cannot access repository (network/auth issues)
    let result = jin()
        .args(["link", "git@github.com:org/config.git"])
        .assert()
        .failure();

    let output = result.get_output();
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    // Accept either "already exists" or "cannot access" errors
    assert!(
        stderr_str.contains("Already exists") || stderr_str.contains("Cannot access remote"),
        "Expected 'Already exists' or 'Cannot access' error, got: {}",
        stderr_str
    );
}

#[test]
fn test_fetch_subcommand() {
    // Fetch fails without remote configured
    jin()
        .arg("fetch")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No remote configured"));
}

#[test]
fn test_pull_subcommand() {
    // Pull fails without remote configured
    jin()
        .arg("pull")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Remote")));
}

#[test]
fn test_push_subcommand() {
    // Push fails without remote configured
    jin()
        .arg("push")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No remote configured"));
}

#[test]
fn test_sync_subcommand() {
    // Sync fails without remote configured
    jin()
        .arg("sync")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Remote")));
}

#[test]
fn test_import_subcommand() {
    // Import requires Jin initialization
    jin()
        .args(["import", ".vscode/settings.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_export_subcommand() {
    // Export requires Jin initialization (or at least checks staging)
    jin()
        .args(["export", ".claude/config.json"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("not found").or(predicate::str::contains("not Jin-tracked")),
        );
}

#[test]
fn test_repair_subcommand() {
    // Repair can run even without initialization
    jin()
        .arg("repair")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Checking Jin repository integrity",
        ));
}

#[test]
fn test_invalid_subcommand() {
    jin()
        .arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

// ============================================================
// Link Command Integration Tests
// ============================================================

#[test]
fn test_link_invalid_url_empty() {
    jin()
        .args(["link", ""])
        .assert()
        .failure()
        .stderr(predicate::str::contains("URL cannot be empty"));
}

#[test]
fn test_link_invalid_url_format() {
    jin()
        .args(["link", "invalid-url"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid remote URL format"))
        .stderr(predicate::str::contains("Supported formats"));
}

#[test]
fn test_link_invalid_url_relative_path() {
    jin()
        .args(["link", "relative/path"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid remote URL format"));
}

#[test]
fn test_link_invalid_url_unsupported_protocol() {
    jin()
        .args(["link", "ftp://example.com/repo.git"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid remote URL format"));
}

#[test]
fn test_link_valid_https_url() {
    // Valid HTTPS URL should pass validation but may fail on connectivity
    let result = jin()
        .args(["link", "https://github.com/nonexistent/repo.git"])
        .assert();

    let output = result.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    // Should either succeed, fail on connectivity, or fail if remote already exists
    assert!(
        stdout_str.contains("Testing connection")
            || stderr_str.contains("Cannot access remote repository")
            || stderr_str.contains("Repository not found")
            || stderr_str.contains("Already exists"),
        "Expected connectivity test, error, or already exists: stdout={}, stderr={}",
        stdout_str,
        stderr_str
    );
}

#[test]
fn test_link_valid_ssh_url() {
    // Valid SSH URL should pass validation but may fail on connectivity
    let result = jin()
        .args(["link", "git@github.com:nonexistent/repo.git"])
        .assert();

    let output = result.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    // Should either succeed, fail on connectivity, or fail if remote already exists
    assert!(
        stdout_str.contains("Testing connection")
            || stderr_str.contains("Cannot access remote repository")
            || stderr_str.contains("Repository not found")
            || stderr_str.contains("Already exists"),
        "Expected connectivity test, error, or already exists: stdout={}, stderr={}",
        stdout_str,
        stderr_str
    );
}

#[test]
fn test_link_force_flag() {
    // Test that --force flag is recognized (actual functionality requires setup)
    let result = jin()
        .args(["link", "https://github.com/example/repo.git", "--force"])
        .assert();

    let output = result.get_output();
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    // Should not complain about unknown flag
    assert!(
        !stderr_str.contains("unexpected argument '--force'"),
        "Force flag should be recognized"
    );
}

#[test]
fn test_link_help() {
    jin()
        .args(["link", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Remote repository URL"))
        .stdout(predicate::str::contains("--force"));
}

// ============================================================
// Completion Command Integration Tests
// ============================================================

#[test]
fn test_completion_bash() {
    jin()
        .args(["completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_jin")) // Bash functions start with _
        .stdout(predicate::str::contains("complete")); // Bash uses 'complete' builtin
}

#[test]
fn test_completion_zsh() {
    jin()
        .args(["completion", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef")); // Zsh completions start with #compdef
}

#[test]
fn test_completion_fish() {
    jin()
        .args(["completion", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete")) // Fish uses 'complete' command
        .stdout(predicate::str::contains("-c jin")); // Fish completion for command 'jin'
}

#[test]
fn test_completion_powershell() {
    jin()
        .args(["completion", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Register-ArgumentCompleter"));
}

#[test]
fn test_completion_invalid_shell() {
    jin()
        .args(["completion", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"))
        .stderr(predicate::str::contains("possible values"));
}

#[test]
fn test_completion_no_shell() {
    jin()
        .args(["completion"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "required arguments were not provided",
        ))
        .stderr(predicate::str::contains("<SHELL>"));
}

#[test]
fn test_completion_help() {
    jin()
        .args(["completion", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Generate shell completion scripts",
        ))
        .stdout(predicate::str::contains("bash"))
        .stdout(predicate::str::contains("zsh"))
        .stdout(predicate::str::contains("fish"))
        .stdout(predicate::str::contains("powershell"));
}
