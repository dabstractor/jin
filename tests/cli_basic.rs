//! Basic CLI integration tests for Jin

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;

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
fn test_add_help() {
    jin()
        .args(["add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("LAYER ROUTING:"))
        .stdout(predicate::str::contains("Storage"))
        .stdout(predicate::str::contains("jin/project/<project>/"))
        .stdout(predicate::str::contains("jin/global/"))
        .stdout(predicate::str::contains("~/.jin/local/"));
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
#[serial]
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
#[serial]
fn test_mode_use_subcommand() {
    // Use a unique mode name that won't exist
    let unique_mode = format!("nonexistent_mode_{}", std::process::id());
    jin()
        .args(["mode", "use", &unique_mode])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
#[serial]
fn test_mode_list_subcommand() {
    // Mode list works without project init - uses global Jin repo
    jin().args(["mode", "list"]).assert().success().stdout(
        predicate::str::contains("Available modes").or(predicate::str::contains("No modes found")),
    );
}

#[test]
#[serial]
fn test_mode_show_subcommand() {
    // Mode show works without project init - shows active mode or "No active mode"
    jin()
        .args(["mode", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("mode").or(predicate::str::contains("No active mode")));
}

#[test]
#[serial]
fn test_mode_unset_subcommand() {
    // Mode unset works without project init - shows message if no mode is set
    jin().args(["mode", "unset"]).assert().success().stdout(
        predicate::str::contains("Deactivated").or(predicate::str::contains("No active mode")),
    );
}

#[test]
#[serial]
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
#[serial]
fn test_scope_create_with_mode() {
    jin()
        .args(["scope", "create", "language:javascript", "--mode=claude"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
#[serial]
fn test_scope_use_subcommand() {
    // Scope use works without Jin initialization - activates the scope in global Jin repo
    // Use a unique scope name to avoid conflicts with other tests
    let unique_scope = format!("test_{}", std::process::id());
    jin()
        .args(["scope", "use", &unique_scope])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn test_scope_list_subcommand() {
    // Scope list requires project initialization (context)
    // Use isolated temp directory to avoid existing .jin context
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    jin()
        .args(["scope", "list"])
        .current_dir(temp.path())
        .env("JIN_DIR", temp.path().join(".jin"))
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
#[serial]
fn test_commit_subcommand() {
    // Commit works without Jin initialization but fails with no staged files
    jin()
        .args(["commit", "-m", "Test commit"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No staged files to commit"));
}

#[test]
fn test_apply_subcommand() {
    // Apply requires Jin initialization
    let temp = tempfile::tempdir().unwrap();
    jin()
        .arg("apply")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_apply_dry_run() {
    // Apply --dry-run also requires Jin initialization
    let temp = tempfile::tempdir().unwrap();
    jin()
        .args(["apply", "--dry-run"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
#[serial]
fn test_reset_subcommand() {
    // Reset succeeds even without Jin initialization - shows "Nothing to reset"
    jin()
        .arg("reset")
        .assert()
        .success()
        .stdout(predicate::str::contains("Nothing to reset"));
}

#[test]
#[serial]
fn test_diff_subcommand() {
    // Diff succeeds even without Jin initialization - shows "No workspace metadata found"
    jin()
        .arg("diff")
        .assert()
        .success()
        .stdout(predicate::str::contains("No workspace metadata found"));
}

#[test]
fn test_log_subcommand() {
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    // Run in isolated environment
    jin()
        .arg("log")
        .current_dir(temp.path())
        .env("JIN_DIR", temp.path().join(".jin_global"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
#[serial]
fn test_context_subcommand() {
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    // Run in isolated environment - no Jin initialization
    jin()
        .arg("context")
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_context_subcommand_success() {
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

    // Test context shows default values
    jin()
        .arg("context")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Current Jin context"))
        .stdout(predicate::str::contains("(none)"));

    // Create and use a mode
    let mode_name = format!("test_mode_{}", std::process::id());
    jin()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and use a scope
    let scope_name = format!("test_scope_{}", std::process::id());
    jin()
        .args(["scope", "create", &scope_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["scope", "use", &scope_name])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Test context shows active mode and scope
    jin()
        .arg("context")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Current Jin context"))
        .stdout(predicate::str::contains(&mode_name))
        .stdout(predicate::str::contains(&scope_name));
}

#[test]
fn test_layers_subcommand() {
    // Layers requires Jin initialization
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    // Run in isolated environment
    jin()
        .arg("layers")
        .current_dir(temp.path())
        .env("JIN_DIR", temp.path().join(".jin_global"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
fn test_list_subcommand() {
    // List requires Jin initialization - use isolated JIN_DIR
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    jin()
        .arg("list")
        .env("JIN_DIR", temp.path().join(".jin"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
#[serial]
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
#[serial]
fn test_fetch_subcommand() {
    // Fetch fails without remote configured - use isolated JIN_DIR
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    jin()
        .arg("fetch")
        .env("JIN_DIR", temp.path().join(".jin"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("No remote configured"));
}

#[test]
#[serial]
fn test_pull_subcommand() {
    // Pull fails without remote configured - use isolated JIN_DIR
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    jin()
        .arg("pull")
        .env("JIN_DIR", temp.path().join(".jin"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("No remote configured"));
}

#[test]
#[serial]
fn test_push_subcommand() {
    // Push fails without remote configured
    jin()
        .arg("push")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No remote configured"));
}

#[test]
#[serial]
fn test_sync_subcommand() {
    // Sync fails without remote configured - use isolated JIN_DIR
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    jin()
        .arg("sync")
        .env("JIN_DIR", temp.path().join(".jin"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("No remote configured"));
}

#[test]
#[serial]
fn test_import_subcommand() {
    // Import requires the file to exist - use isolated JIN_DIR
    use tempfile::TempDir;
    let temp = TempDir::new().unwrap();

    jin()
        .args(["import", ".vscode/settings.json"])
        .current_dir(temp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Jin not initialized"));
}

#[test]
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
fn test_link_invalid_url_empty() {
    jin()
        .args(["link", ""])
        .assert()
        .failure()
        .stderr(predicate::str::contains("URL cannot be empty"));
}

#[test]
#[serial]
fn test_link_invalid_url_format() {
    jin()
        .args(["link", "invalid-url"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid remote URL format"))
        .stderr(predicate::str::contains("Supported formats"));
}

#[test]
#[serial]
fn test_link_invalid_url_relative_path() {
    jin()
        .args(["link", "relative/path"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid remote URL format"));
}

#[test]
#[serial]
fn test_link_invalid_url_unsupported_protocol() {
    jin()
        .args(["link", "ftp://example.com/repo.git"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid remote URL format"));
}

#[test]
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
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
#[serial]
fn test_completion_bash() {
    jin()
        .args(["completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_jin")) // Bash functions start with _
        .stdout(predicate::str::contains("complete")); // Bash uses 'complete' builtin
}

#[test]
#[serial]
fn test_completion_zsh() {
    jin()
        .args(["completion", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef")); // Zsh completions start with #compdef
}

#[test]
#[serial]
fn test_completion_fish() {
    jin()
        .args(["completion", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete")) // Fish uses 'complete' command
        .stdout(predicate::str::contains("-c jin")); // Fish completion for command 'jin'
}

#[test]
#[serial]
fn test_completion_powershell() {
    jin()
        .args(["completion", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Register-ArgumentCompleter"));
}

#[test]
#[serial]
fn test_completion_invalid_shell() {
    jin()
        .args(["completion", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"))
        .stderr(predicate::str::contains("possible values"));
}

#[test]
#[serial]
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
#[serial]
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

// ============================================================
// Status Command - Conflict State Integration Tests
// ============================================================

#[test]
fn test_status_no_conflicts_normal_display() {
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

    // Create and activate a mode
    let mode_name = format!("test_mode_no_conflict_{}", std::process::id());

    jin()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Check status does NOT show conflict section
    jin()
        .arg("status")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Jin status:"))
        .stdout(predicate::str::contains("Mode:"))
        .stdout(predicate::str::contains(&mode_name))
        .stdout(predicate::str::contains("Workspace state: Clean"))
        .stdout(predicate::str::contains("Merge conflicts").not());
}

#[test]
fn test_status_shows_conflict_state() {
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

    // Create and activate a mode
    let mode_name = format!("test_mode_conflict_{}", std::process::id());

    jin()
        .args(["mode", "create", &mode_name])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["mode", "use", &mode_name])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create and stage a file
    let test_file = temp.path().join("config.json");
    fs::write(&test_file, r#"{"test": "value1"}"#).unwrap();

    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "First commit"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Create a second mode with conflicting content
    let mode_name2 = format!("test_mode_conflict2_{}", std::process::id());

    jin()
        .args(["mode", "create", &mode_name2])
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Deactivate first mode
    jin()
        .args(["mode", "unset"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Activate second mode
    jin()
        .args(["mode", "use", &mode_name2])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Modify the file with conflicting content
    fs::write(&test_file, r#"{"test": "value2"}"#).unwrap();

    jin()
        .args(["add", "config.json", "--mode"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Second commit"])
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success();

    // Manually create a paused state to simulate conflict scenario
    // (In a real scenario, jin apply would create this)
    let jin_dir_local = temp.path().join(".jin");
    fs::create_dir_all(&jin_dir_local).unwrap();

    let paused_state_content = r#"
timestamp: "2024-01-03T14:30:00Z"
layer_config:
  layers:
    - "mode-base"
  mode: null
  scope: null
  project: null
conflict_files:
  - "config.json"
applied_files: []
conflict_count: 1
"#;

    fs::write(
        jin_dir_local.join(".paused_apply.yaml"),
        paused_state_content,
    )
    .unwrap();

    // Check status shows conflicts
    jin()
        .arg("status")
        .current_dir(temp.path())
        .env("JIN_DIR", &jin_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("Merge conflicts"))
        .stdout(predicate::str::contains("1 file"))
        .stdout(predicate::str::contains("config.json.jinmerge"))
        .stdout(predicate::str::contains("Resolve with: jin resolve"))
        .stdout(predicate::str::contains("Detected:"));
}
