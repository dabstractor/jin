//! 3-way merge integration tests for jin pull
//!
//! Tests the 3-way merge implementation in the pull command for divergent
//! layer histories. Validates clean merges, conflict handling, and .jinmerge
//! file creation.

use std::fs;

mod common;
use common::fixtures::*;

/// Test that fast-forward merge still works (regression test)
///
/// Ensures that the 3-way merge implementation doesn't break existing
/// fast-forward behavior when local is behind remote.
#[test]
fn test_pull_fast_forward_still_works() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Setup: Create initial commit in "remote" via temp workspace
    // CRITICAL: Use the SAME JIN_DIR for both workspaces to share layer refs
    let temp_workspace = TestFixture::new()?;
    jin_init(temp_workspace.path(), Some(jin_dir))?;

    // Create initial file and commit
    fs::write(temp_workspace.path().join("config.txt"), "version=1")?;

    jin()
        .args(["add", "config.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Initial commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Link to remote and push
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("Successfully pushed"));

    // Setup main local repo - link and fetch
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Pull should fast-forward
    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("✓ global: Updated (fast-forward)"));

    Ok(())
}

/// Test clean 3-way merge with non-overlapping changes
///
/// Creates divergent commits where changes are in different parts of files,
/// which should merge cleanly without conflicts.
#[test]
fn test_pull_divergent_clean_merge() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Step 1: Create base commit in remote via temp workspace
    // CRITICAL: Use the SAME JIN_DIR for both workspaces to share layer refs
    let temp_workspace = TestFixture::new()?;
    jin_init(temp_workspace.path(), Some(jin_dir))?;

    fs::write(
        temp_workspace.path().join("config.txt"),
        "setting1=value1\nsetting2=value2",
    )?;

    jin()
        .args(["add", "config.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Base commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Link and push base commit
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("Successfully pushed"));

    // Setup local repo with base
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Step 2: Make local change (add setting3 at end)
    fs::write(
        remote_fixture.local_path.join("config.txt"),
        "setting1=value1\nsetting2=value2\nsetting3=local",
    )?;

    jin()
        .args(["add", "config.txt", "--global"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Local change"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Step 3: Make remote change (modify setting1)
    fs::write(
        temp_workspace.path().join("config.txt"),
        "setting1=remote\nsetting2=value2",
    )?;

    jin()
        .args(["add", "config.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Remote change"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("Successfully pushed"));

    // Step 4: Pull with divergent history - should 3-way merge cleanly
    jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("✓ global: Merged (3-way)"));

    // Apply to workspace and verify merged content
    jin()
        .arg("apply")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    let merged_content = fs::read_to_string(remote_fixture.local_path.join("config.txt"))?;
    assert!(merged_content.contains("setting1=remote"));
    assert!(merged_content.contains("setting3=local"));

    Ok(())
}

/// Test 3-way merge with conflicts creates .jinmerge files
///
/// Creates divergent commits with overlapping changes that cause conflicts.
/// Verifies that .jinmerge files are created following the Phase 1 workflow.
#[test]
fn test_pull_divergent_with_conflicts() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Step 1: Create base commit in remote via temp workspace
    let temp_workspace = TestFixture::new()?;
    let jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(jin_dir))?;

    fs::write(temp_workspace.path().join("config.txt"), "version=1")?;

    jin()
        .args(["add", "config.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Base commit"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Link and push base commit
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("Successfully pushed"));

    // Setup local repo with base
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Step 2: Make conflicting local change
    fs::write(remote_fixture.local_path.join("config.txt"), "version=2")?;

    jin()
        .args(["add", "config.txt", "--global"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Local conflicting change"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Step 3: Make conflicting remote change
    fs::write(temp_workspace.path().join("config.txt"), "version=3")?;

    jin()
        .args(["add", "config.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Remote conflicting change"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("Successfully pushed"));

    // Step 4: Pull with divergent history - should create .jinmerge file
    jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("! global: Merged with 1 conflicts"))
        .stdout(
            predicates::str::contains("config.txt has conflicts (.jinmerge created)"),
        );

    // Verify .jinmerge file was created
    let jinmerge_path = remote_fixture.local_path.join("config.txt.jinmerge");
    assert!(
        jinmerge_path.exists(),
        ".jinmerge file should be created for conflicts"
    );

    // Verify .jinmerge file has correct format
    let jinmerge_content = fs::read_to_string(&jinmerge_path)?;
    assert!(jinmerge_content.contains("# Jin merge conflict"));
    assert!(jinmerge_content.contains("<<<<<<<"));
    assert!(jinmerge_content.contains("======="));
    assert!(jinmerge_content.contains(">>>>>>>"));

    // Verify both versions are in the conflict
    assert!(jinmerge_content.contains("version=2")); // Local
    assert!(jinmerge_content.contains("version=3")); // Remote

    Ok(())
}

/// Test clean 3-way merge with files added in both branches
///
/// When files are added in both branches but have different names,
/// the merge should combine both sets of files.
#[test]
fn test_pull_divergent_clean_merge_different_files() -> Result<(), Box<dyn std::error::Error>> {
    let remote_fixture = setup_jin_with_remote()?;
    let jin_dir = remote_fixture.jin_dir.as_ref().unwrap();

    // Step 1: Create base commit (with initial file)
    let temp_workspace = TestFixture::new()?;
    let jin_dir = temp_workspace.jin_dir.as_ref().unwrap();
    jin_init(temp_workspace.path(), Some(jin_dir))?;

    // Create initial file
    fs::write(temp_workspace.path().join("base.txt"), "base content")?;

    jin()
        .args(["add", "base.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Base commit with initial file"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Link and push
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("Successfully pushed"));

    // Setup local repo
    jin()
        .args([
            "link",
            remote_fixture.remote_path.to_str().unwrap(),
            "--force",
        ])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Step 2: Add file in local
    fs::write(remote_fixture.local_path.join("local.txt"), "local content")?;

    jin()
        .args(["add", "local.txt", "--global"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Add local file"])
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    // Step 3: Add different file in remote
    fs::write(temp_workspace.path().join("remote.txt"), "remote content")?;

    jin()
        .args(["add", "remote.txt", "--global"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .args(["commit", "-m", "Add remote file"])
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("push")
        .current_dir(temp_workspace.path())
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("Successfully pushed"));

    // Step 4: Pull should merge both files
    jin()
        .arg("fetch")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    jin()
        .arg("pull")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success()
        .stdout(predicates::str::contains("✓ global: Merged (3-way)"));

    // Apply and verify both files exist
    jin()
        .arg("apply")
        .current_dir(&remote_fixture.local_path)
        .env("JIN_DIR", jin_dir)
        .assert()
        .success();

    assert!(remote_fixture.local_path.join("local.txt").exists());
    assert!(remote_fixture.local_path.join("remote.txt").exists());
    assert!(remote_fixture.local_path.join("base.txt").exists());

    let local_content = fs::read_to_string(remote_fixture.local_path.join("local.txt"))?;
    assert_eq!(local_content, "local content");

    let remote_content = fs::read_to_string(remote_fixture.local_path.join("remote.txt"))?;
    assert_eq!(remote_content, "remote content");

    Ok(())
}
