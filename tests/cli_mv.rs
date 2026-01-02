//! Integration tests for `jin mv` command
//!
//! Tests file move/rename functionality in staging and workspace.

mod common;
use common::assertions::*;
use common::fixtures::*;

/// Test basic file rename in staging
#[test]
fn test_mv_single_file() {
    let fixture = setup_test_repo().unwrap();

    // Create and stage a file
    std::fs::write(fixture.path.join("config.json"), r#"{"key": "value"}"#).unwrap();
    jin()
        .args(["add", "config.json"])
        .current_dir(fixture.path())
        .assert()
        .success();

    // Delete workspace file to avoid confirmation prompt (mv without --force should only update staging)
    std::fs::remove_file(fixture.path.join("config.json")).unwrap();

    // Rename the file
    jin()
        .args(["mv", "config.json", "settings.json"])
        .current_dir(fixture.path())
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Moved 1 file(s) in project-base layer",
        ));

    // Verify staging was updated
    assert_staging_not_contains(fixture.path(), "config.json");
    assert_staging_contains(fixture.path(), "settings.json");

    // Verify workspace files (file was deleted after staging to avoid prompt)
    assert_workspace_file_not_exists(fixture.path(), "config.json");
    assert_workspace_file_not_exists(fixture.path(), "settings.json");
}

/// Test file rename with --force flag (workspace move)
#[test]
fn test_mv_force() {
    let fixture = setup_test_repo().unwrap();

    // Create and stage a file
    std::fs::write(fixture.path.join("data.txt"), "original content").unwrap();
    jin()
        .args(["add", "data.txt"])
        .current_dir(fixture.path())
        .assert()
        .success();

    // Rename with force
    jin()
        .args(["mv", "--force", "data.txt", "renamed.txt"])
        .current_dir(fixture.path())
        .assert()
        .success();

    // Verify staging was updated
    assert_staging_not_contains(fixture.path(), "data.txt");
    assert_staging_contains(fixture.path(), "renamed.txt");

    // Verify workspace file was moved
    assert_workspace_file_not_exists(fixture.path(), "data.txt");
    assert_workspace_file(fixture.path(), "renamed.txt", "original content");
}

/// Test dry-run mode
#[test]
fn test_mv_dry_run() {
    let fixture = setup_test_repo().unwrap();

    // Create and stage a file
    std::fs::write(fixture.path.join("test.json"), r#"{"test": true}"#).unwrap();
    jin()
        .args(["add", "test.json"])
        .current_dir(fixture.path())
        .assert()
        .success();

    // Dry-run move
    jin()
        .args(["mv", "--dry-run", "test.json", "renamed.json"])
        .current_dir(fixture.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Would move:"));

    // Verify staging was NOT changed
    assert_staging_contains(fixture.path(), "test.json");
    assert_staging_not_contains(fixture.path(), "renamed.json");
}

/// Test layer routing with --mode flag
#[test]
fn test_mv_with_layer_flags() {
    let fixture = TestFixture::new().unwrap();
    let jin_dir = fixture.jin_dir.as_ref().unwrap();
    fixture.set_jin_dir();
    jin_init(fixture.path()).unwrap();

    // Create a mode
    create_mode("testmode", Some(jin_dir)).unwrap();

    // Set active mode
    jin()
        .args(["mode", "use", "testmode"])
        .current_dir(fixture.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Create and stage a file to mode layer
    std::fs::write(
        fixture.path.join("mode-config.json"),
        r#"{"mode": "config"}"#,
    )
    .unwrap();
    jin()
        .args(["add", "--mode", "mode-config.json"])
        .current_dir(fixture.path())
        .assert()
        .success();

    // Delete workspace file to avoid confirmation prompt
    std::fs::remove_file(fixture.path.join("mode-config.json")).unwrap();

    // Move the file within mode layer
    jin()
        .args([
            "mv",
            "--mode",
            "mode-config.json",
            "mode-config-renamed.json",
        ])
        .current_dir(fixture.path())
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Moved 1 file(s) in mode-base layer",
        ));

    // Verify staging was updated
    assert_staging_not_contains(fixture.path(), "mode-config.json");
    assert_staging_contains(fixture.path(), "mode-config-renamed.json");
}

/// Test error: source file not in staging
#[test]
fn test_mv_source_not_staged() {
    let fixture = setup_test_repo().unwrap();

    // Create a file but don't stage it
    std::fs::write(fixture.path.join("unstaged.txt"), "content").unwrap();

    // Try to move unstaged file
    jin()
        .args(["mv", "unstaged.txt", "target.txt"])
        .current_dir(fixture.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("not in staging"));
}

/// Test error: destination already staged
#[test]
fn test_mv_destination_exists() {
    let fixture = setup_test_repo().unwrap();

    // Create and stage two files
    std::fs::write(fixture.path.join("source.txt"), "source").unwrap();
    std::fs::write(fixture.path.join("dest.txt"), "dest").unwrap();
    jin()
        .args(["add", "source.txt", "dest.txt"])
        .current_dir(fixture.path())
        .assert()
        .success();

    // Delete workspace files to avoid confirmation prompt
    std::fs::remove_file(fixture.path.join("source.txt")).unwrap();
    std::fs::remove_file(fixture.path.join("dest.txt")).unwrap();

    // Try to move source to dest (already staged)
    jin()
        .args(["mv", "source.txt", "dest.txt"])
        .current_dir(fixture.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("already in staging"));
}

/// Test error: odd number of arguments
#[test]
fn test_mv_odd_number_of_files() {
    let fixture = setup_test_repo().unwrap();

    // Try with odd number of args (not pairs)
    jin()
        .args(["mv", "file1.txt", "file2.txt", "file3.txt"])
        .current_dir(fixture.path())
        .assert()
        .failure();
}

/// Test batch move (multiple file pairs)
#[test]
fn test_mv_batch() {
    let fixture = setup_test_repo().unwrap();

    // Create and stage multiple files
    std::fs::write(fixture.path.join("file1.txt"), "content1").unwrap();
    std::fs::write(fixture.path.join("file2.txt"), "content2").unwrap();
    std::fs::write(fixture.path.join("file3.txt"), "content3").unwrap();
    std::fs::write(fixture.path.join("file4.txt"), "content4").unwrap();

    jin()
        .args(["add", "file1.txt", "file2.txt", "file3.txt", "file4.txt"])
        .current_dir(fixture.path())
        .assert()
        .success();

    // Delete workspace files to avoid confirmation prompt
    std::fs::remove_file(fixture.path.join("file1.txt")).unwrap();
    std::fs::remove_file(fixture.path.join("file2.txt")).unwrap();
    std::fs::remove_file(fixture.path.join("file3.txt")).unwrap();
    std::fs::remove_file(fixture.path.join("file4.txt")).unwrap();

    // Batch move all files
    jin()
        .args([
            "mv",
            "file1.txt",
            "renamed1.txt",
            "file2.txt",
            "renamed2.txt",
            "file3.txt",
            "renamed3.txt",
            "file4.txt",
            "renamed4.txt",
        ])
        .current_dir(fixture.path())
        .assert()
        .success()
        .stdout(predicates::str::contains("Moved 4 file(s)"));

    // Verify all files were moved in staging
    assert_staging_not_contains(fixture.path(), "file1.txt");
    assert_staging_not_contains(fixture.path(), "file2.txt");
    assert_staging_not_contains(fixture.path(), "file3.txt");
    assert_staging_not_contains(fixture.path(), "file4.txt");

    assert_staging_contains(fixture.path(), "renamed1.txt");
    assert_staging_contains(fixture.path(), "renamed2.txt");
    assert_staging_contains(fixture.path(), "renamed3.txt");
    assert_staging_contains(fixture.path(), "renamed4.txt");
}

/// Test move to subdirectory
#[test]
fn test_mv_to_subdirectory() {
    let fixture = setup_test_repo().unwrap();

    // Create and stage a file
    std::fs::write(fixture.path.join("config.json"), r#"{"key": "value"}"#).unwrap();
    jin()
        .args(["add", "config.json"])
        .current_dir(fixture.path())
        .assert()
        .success();

    // Move to subdirectory
    jin()
        .args(["mv", "--force", "config.json", "settings/config.json"])
        .current_dir(fixture.path())
        .assert()
        .success();

    // Verify staging was updated
    // Check for exact JSON path (with quotes to avoid substring matches)
    let staging_content =
        std::fs::read_to_string(fixture.path.join(".jin/staging/index.json")).unwrap();
    assert!(!staging_content.contains("\"config.json\""));
    assert!(staging_content.contains("\"settings/config.json\""));

    // Verify workspace file was moved to subdirectory
    assert_workspace_file_not_exists(fixture.path(), "config.json");
    assert_workspace_file(
        fixture.path(),
        "settings/config.json",
        r#"{"key": "value"}"#,
    );
}

/// Test error: --project without --mode
#[test]
fn test_mv_project_without_mode() {
    let fixture = setup_test_repo().unwrap();

    // Try to use --project without --mode
    jin()
        .args(["mv", "--project", "source.txt", "dest.txt"])
        .current_dir(fixture.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("--project requires --mode"));
}

/// Test error: --global with other flags
#[test]
fn test_mv_global_with_mode() {
    let fixture = setup_test_repo().unwrap();

    // Try to use --global with --mode
    jin()
        .args(["mv", "--global", "--mode", "source.txt", "dest.txt"])
        .current_dir(fixture.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("Cannot combine --global"));
}

/// Test .gitignore updates for move operation
#[test]
fn test_mv_gitignore_update() {
    let fixture = setup_test_repo().unwrap();

    // Create and stage a file
    std::fs::write(fixture.path.join("config.json"), r#"{"key": "value"}"#).unwrap();
    jin()
        .args(["add", "config.json"])
        .current_dir(fixture.path())
        .assert()
        .success();

    // Delete workspace file to avoid confirmation prompt
    std::fs::remove_file(fixture.path.join("config.json")).unwrap();

    // Verify .gitignore has the original path
    let gitignore = std::fs::read_to_string(fixture.path.join(".gitignore")).unwrap();
    assert!(gitignore.contains("config.json"));

    // Move the file
    jin()
        .args(["mv", "config.json", "settings.json"])
        .current_dir(fixture.path())
        .assert()
        .success();

    // Verify .gitignore was updated (old removed, new added)
    let gitignore_after = std::fs::read_to_string(fixture.path.join(".gitignore")).unwrap();
    assert!(!gitignore_after.contains("config.json"));
    assert!(gitignore_after.contains("settings.json"));
}

/// Test confirmation prompt for workspace moves
#[test]
fn test_mv_confirmation_prompt() {
    let fixture = setup_test_repo().unwrap();

    // Create and stage a file
    std::fs::write(fixture.path.join("test.txt"), "content").unwrap();
    jin()
        .args(["add", "test.txt"])
        .current_dir(fixture.path())
        .assert()
        .success();

    // Try to move with --force but without --force flag
    // Should prompt for confirmation
    jin()
        .args(["mv", "test.txt", "moved.txt"])
        .current_dir(fixture.path())
        .write_stdin("no")
        .assert()
        .success()
        .stdout(predicates::str::contains("Move cancelled"));

    // Verify nothing changed
    assert_staging_contains(fixture.path(), "test.txt");
    assert_staging_not_contains(fixture.path(), "moved.txt");
}

/// Test partial success: some files fail
#[test]
fn test_mv_partial_success() {
    let fixture = setup_test_repo().unwrap();

    // Stage one file, leave another unstaged
    std::fs::write(fixture.path.join("staged.txt"), "staged").unwrap();
    std::fs::write(fixture.path.join("unstaged.txt"), "unstaged").unwrap();

    jin()
        .args(["add", "staged.txt"])
        .current_dir(fixture.path())
        .assert()
        .success();

    // Delete workspace files to avoid confirmation prompt
    std::fs::remove_file(fixture.path.join("staged.txt")).unwrap();
    std::fs::remove_file(fixture.path.join("unstaged.txt")).unwrap();

    // Try to move both (one will fail)
    jin()
        .args([
            "mv",
            "staged.txt",
            "renamed-staged.txt",
            "unstaged.txt",
            "renamed-unstaged.txt",
        ])
        .current_dir(fixture.path())
        .assert()
        .success() // Partial success returns Ok
        .stderr(predicates::str::contains("Error:"));

    // Verify the staged file was moved (use exact JSON path matching)
    let staging_content =
        std::fs::read_to_string(fixture.path.join(".jin/staging/index.json")).unwrap();
    assert!(!staging_content.contains("\"staged.txt\""));
    assert!(staging_content.contains("\"renamed-staged.txt\""));

    // Verify the unstaged file was NOT moved
    assert!(!staging_content.contains("\"renamed-unstaged.txt\""));
}
