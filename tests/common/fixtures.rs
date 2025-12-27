//! Test fixture utilities for Jin integration tests
//!
//! Provides setup helpers for creating isolated test environments with
//! Git repositories, Jin initialization, and local filesystem remotes.

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Test fixture that maintains isolated directory
///
/// CRITICAL: TempDir must be stored to prevent premature cleanup.
/// When TempDir is dropped, the directory is deleted immediately.
pub struct TestFixture {
    /// The temporary directory (must be kept in scope)
    _tempdir: TempDir,
    /// Path to the test directory
    pub path: PathBuf,
}

impl TestFixture {
    /// Create a new isolated test directory
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let path = tempdir.path().to_path_buf();
        Ok(TestFixture {
            _tempdir: tempdir,
            path,
        })
    }

    /// Get the path to the test directory
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// Test fixture with local and remote repositories
pub struct RemoteFixture {
    /// The temporary directory (must be kept in scope)
    pub _tempdir: TempDir,
    /// Path to the local repository
    pub local_path: PathBuf,
    /// Path to the remote bare repository
    pub remote_path: PathBuf,
}

impl RemoteFixture {
    /// Create a new fixture with local and remote repositories
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let tempdir = TempDir::new()?;
        let local_path = tempdir.path().join("local");
        let remote_path = tempdir.path().join("remote");

        fs::create_dir(&local_path)?;
        fs::create_dir(&remote_path)?;

        Ok(RemoteFixture {
            _tempdir: tempdir,
            local_path,
            remote_path,
        })
    }
}

/// Initialize Jin in the specified directory
///
/// Runs `jin init` and verifies success.
pub fn jin_init(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    Command::cargo_bin("jin")?
        .arg("init")
        .current_dir(path)
        .assert()
        .success();
    Ok(())
}

/// Create a test repository with Jin initialized
///
/// Returns a TestFixture with Jin already initialized.
pub fn setup_test_repo() -> Result<TestFixture, Box<dyn std::error::Error>> {
    let fixture = TestFixture::new()?;
    jin_init(fixture.path())?;
    Ok(fixture)
}

/// Create a test repository with local bare remote configured
///
/// Returns a RemoteFixture with:
/// - Local repository with Jin initialized
/// - Bare remote repository
/// - Remote NOT yet linked (caller should use `jin link`)
pub fn setup_jin_with_remote() -> Result<RemoteFixture, Box<dyn std::error::Error>> {
    let fixture = RemoteFixture::new()?;

    // Initialize Jin in local directory
    jin_init(&fixture.local_path)?;

    // Initialize bare Git repository as remote
    git2::Repository::init_bare(&fixture.remote_path)?;

    Ok(fixture)
}

/// Create a commit in a Git repository
///
/// Creates a file with specified content and commits it.
/// Configures Git user if not already configured.
///
/// # Arguments
/// * `repo_path` - Path to the Git repository
/// * `file` - Relative path to the file to create
/// * `content` - Content to write to the file
/// * `msg` - Commit message
pub fn create_commit_in_repo(
    repo_path: &Path,
    file: &str,
    content: &str,
    msg: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Open the repository
    let repo = git2::Repository::open(repo_path)?;

    // Configure Git user if not already set (required for commits)
    let config = repo.config()?;
    if config.get_string("user.email").is_err() {
        let mut config = repo.config()?;
        config.set_str("user.email", "test@example.com")?;
        config.set_str("user.name", "Test User")?;
    }

    // Write file
    let file_path = repo_path.join(file);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&file_path, content)?;

    // Stage file
    let mut index = repo.index()?;
    index.add_path(Path::new(file))?;
    index.write()?;

    // Create commit
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let signature = repo.signature()?;

    // Get parent commit if exists
    let parent_commit = match repo.head() {
        Ok(head) => {
            let oid = head.target().ok_or("HEAD has no target")?;
            Some(repo.find_commit(oid)?)
        }
        Err(_) => None,
    };

    // Create the commit
    if let Some(parent) = parent_commit {
        repo.commit(Some("HEAD"), &signature, &signature, msg, &tree, &[&parent])?;
    } else {
        repo.commit(Some("HEAD"), &signature, &signature, msg, &tree, &[])?;
    }

    Ok(())
}

/// Get the jin binary command for testing
pub fn jin() -> Command {
    Command::cargo_bin("jin").expect("Failed to find jin binary")
}

/// Create a mode in the global Jin repository
///
/// This is a helper for tests that need modes to exist.
pub fn create_mode(mode_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let result = jin().args(["mode", "create", mode_name]).assert();

    // Accept either success (new mode) or error (already exists)
    let output = result.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    if !stdout_str.contains(mode_name) && !stderr_str.contains("already exists") {
        return Err(format!(
            "Failed to create mode {}: stdout={}, stderr={}",
            mode_name, stdout_str, stderr_str
        )
        .into());
    }

    Ok(())
}

/// Create a scope in the global Jin repository
///
/// This is a helper for tests that need scopes to exist.
pub fn create_scope(scope_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let result = jin().args(["scope", "create", scope_name]).assert();

    // Accept either success (new scope) or error (already exists)
    let output = result.get_output();
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    if !stdout_str.contains(scope_name) && !stderr_str.contains("already exists") {
        return Err(format!(
            "Failed to create scope {}: stdout={}, stderr={}",
            scope_name, stdout_str, stderr_str
        )
        .into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_creates_directory() {
        let fixture = TestFixture::new().unwrap();
        assert!(fixture.path().exists());
        assert!(fixture.path().is_dir());
    }

    #[test]
    fn test_remote_fixture_creates_directories() {
        let fixture = RemoteFixture::new().unwrap();
        assert!(fixture.local_path.exists());
        assert!(fixture.remote_path.exists());
    }
}
