//! Custom assertions for Jin-specific state verification
//!
//! Provides assertion helpers to verify Jin repository state, layer commits,
//! staging index contents, and workspace files.

use std::fs;
use std::path::Path;

/// Assert that a workspace file exists and matches expected content
///
/// # Arguments
/// * `project_path` - Path to the project workspace
/// * `file` - Relative path to the file
/// * `expected_content` - Expected file content
///
/// # Panics
/// Panics if the file doesn't exist or content doesn't match
pub fn assert_workspace_file(project_path: &Path, file: &str, expected_content: &str) {
    let file_path = project_path.join(file);
    assert!(
        file_path.exists(),
        "Workspace file {} should exist at {:?}",
        file,
        file_path
    );

    let actual_content = fs::read_to_string(&file_path)
        .unwrap_or_else(|e| panic!("Failed to read file {:?}: {}", file_path, e));

    assert_eq!(
        actual_content, expected_content,
        "Workspace file {} content mismatch.\nExpected: {}\nActual: {}",
        file, expected_content, actual_content
    );
}

/// Assert that a workspace file exists (without checking content)
///
/// # Arguments
/// * `project_path` - Path to the project workspace
/// * `file` - Relative path to the file
///
/// # Panics
/// Panics if the file doesn't exist
pub fn assert_workspace_file_exists(project_path: &Path, file: &str) {
    let file_path = project_path.join(file);
    assert!(
        file_path.exists(),
        "Workspace file {} should exist at {:?}",
        file,
        file_path
    );
}

/// Assert that a workspace file does not exist
///
/// # Arguments
/// * `project_path` - Path to the project workspace
/// * `file` - Relative path to the file
///
/// # Panics
/// Panics if the file exists
pub fn assert_workspace_file_not_exists(project_path: &Path, file: &str) {
    let file_path = project_path.join(file);
    assert!(
        !file_path.exists(),
        "Workspace file {} should not exist at {:?}",
        file,
        file_path
    );
}

/// Assert that staging index contains a file
///
/// # Arguments
/// * `project_path` - Path to the project (with .jin directory)
/// * `file` - Relative path to the file that should be staged
///
/// # Panics
/// Panics if staging index doesn't exist or doesn't contain the file
pub fn assert_staging_contains(project_path: &Path, file: &str) {
    let staging_index_path = project_path.join(".jin/staging/index.json");
    assert!(
        staging_index_path.exists(),
        "Staging index should exist at {:?}",
        staging_index_path
    );

    let staging_content = fs::read_to_string(&staging_index_path)
        .unwrap_or_else(|e| panic!("Failed to read staging index: {}", e));

    assert!(
        staging_content.contains(file),
        "Staging index should contain file '{}'. Staging content:\n{}",
        file,
        staging_content
    );
}

/// Assert that staging index does not contain a file
///
/// # Arguments
/// * `project_path` - Path to the project (with .jin directory)
/// * `file` - Relative path to the file that should not be staged
///
/// # Panics
/// Panics if staging index contains the file
pub fn assert_staging_not_contains(project_path: &Path, file: &str) {
    let staging_index_path = project_path.join(".jin/staging/index.json");

    if !staging_index_path.exists() {
        // If staging index doesn't exist, the file is definitely not staged
        return;
    }

    let staging_content = fs::read_to_string(&staging_index_path)
        .unwrap_or_else(|e| panic!("Failed to read staging index: {}", e));

    assert!(
        !staging_content.contains(file),
        "Staging index should not contain file '{}'. Staging content:\n{}",
        file,
        staging_content
    );
}

/// Assert that a layer ref exists in the Jin repository
///
/// # Arguments
/// * `ref_path` - Git ref path (e.g., "refs/jin/layers/mode/dev")
/// * `jin_repo_path` - Optional path to Jin repository (None uses ~/.jin)
///
/// # Panics
/// Panics if the ref doesn't exist in the specified Jin repository
///
/// # Gotchas
/// - When jin_repo_path is None, falls back to JIN_DIR env var or ~/.jin
/// - For test isolation, always pass Some(jin_dir) with test-specific path
pub fn assert_layer_ref_exists(ref_path: &str, jin_repo_path: Option<&std::path::Path>) {
    let repo_path = match jin_repo_path {
        Some(path) => path.to_path_buf(),
        None => {
            // Fallback to environment variable or home directory
            if let Ok(jin_dir) = std::env::var("JIN_DIR") {
                std::path::PathBuf::from(jin_dir)
            } else {
                dirs::home_dir().expect("Failed to get home directory").join(".jin")
            }
        }
    };

    assert!(
        repo_path.exists(),
        "Jin repository should exist at {:?}",
        repo_path
    );

    let repo = git2::Repository::open(&repo_path).unwrap_or_else(|e| {
        panic!(
            "Failed to open Jin repository at {:?}: {}",
            repo_path, e
        )
    });

    match repo.find_reference(ref_path) {
        Ok(_) => {} // Success
        Err(e) => panic!(
            "Layer ref '{}' should exist in Jin repository at {:?}: {}",
            ref_path, repo_path, e
        ),
    };
}

/// Assert that the Jin context file contains expected mode
///
/// # Arguments
/// * `project_path` - Path to the project (with .jin directory)
/// * `expected_mode` - Expected active mode name
///
/// # Panics
/// Panics if context doesn't exist or doesn't contain the mode
pub fn assert_context_mode(project_path: &Path, expected_mode: &str) {
    let context_path = project_path.join(".jin/context");
    assert!(
        context_path.exists(),
        "Context file should exist at {:?}",
        context_path
    );

    let context_content = fs::read_to_string(&context_path)
        .unwrap_or_else(|e| panic!("Failed to read context file: {}", e));

    // Context is saved as YAML (mode: value) not JSON ("mode": "value")
    assert!(
        context_content.contains(&format!("mode: {}", expected_mode))
            || context_content.contains(&format!("\"mode\": \"{}\"", expected_mode))
            || context_content.contains(&format!("\"mode\":\"{}\"", expected_mode)),
        "Context should contain mode '{}'. Context content:\n{}",
        expected_mode,
        context_content
    );
}

/// Assert that the Jin context file contains expected scope
///
/// # Arguments
/// * `project_path` - Path to the project (with .jin directory)
/// * `expected_scope` - Expected active scope name
///
/// # Panics
/// Panics if context doesn't exist or doesn't contain the scope
pub fn assert_context_scope(project_path: &Path, expected_scope: &str) {
    let context_path = project_path.join(".jin/context");
    assert!(
        context_path.exists(),
        "Context file should exist at {:?}",
        context_path
    );

    let context_content = fs::read_to_string(&context_path)
        .unwrap_or_else(|e| panic!("Failed to read context file: {}", e));

    assert!(
        context_content.contains(&format!("\"scope\": \"{}\"", expected_scope))
            || context_content.contains(&format!("\"scope\":\"{}\"", expected_scope)),
        "Context should contain scope '{}'. Context content:\n{}",
        expected_scope,
        context_content
    );
}

/// Assert that .jin directory exists
///
/// # Arguments
/// * `project_path` - Path to the project
///
/// # Panics
/// Panics if .jin directory doesn't exist
pub fn assert_jin_initialized(project_path: &Path) {
    let jin_dir = project_path.join(".jin");
    assert!(
        jin_dir.exists() && jin_dir.is_dir(),
        "Jin should be initialized (.jin directory should exist) at {:?}",
        project_path
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_assert_workspace_file_exists() {
        let temp = TempDir::new().unwrap();
        let test_file = temp.path().join("test.txt");
        fs::write(&test_file, "content").unwrap();

        assert_workspace_file_exists(temp.path(), "test.txt");
    }

    #[test]
    #[should_panic(expected = "should exist")]
    fn test_assert_workspace_file_exists_fails() {
        let temp = TempDir::new().unwrap();
        assert_workspace_file_exists(temp.path(), "nonexistent.txt");
    }
}
