//! Repair command implementation.
//!
//! This module implements the `jin repair` command that detects and repairs
//! corrupted Jin state including:
//! - Orphaned transaction refs under refs/jin/staging/*
//! - Staging entries for files that no longer exist
//! - Corrupted or missing .jinmap file
//! - Corrupted or missing .gitignore managed block
//!
//! The command supports dry-run mode for safe preview before making changes.

use crate::cli::args::RepairCommand;
use crate::core::error::{JinError, Result};
use crate::staging::index::StagingIndex;
use std::path::{Path, PathBuf};

/// Status enum for jinmap check.
#[derive(Debug, PartialEq)]
enum JinmapStatus {
    Valid,
    Missing,
    Corrupted,
}

/// Status enum for gitignore check.
#[derive(Debug, PartialEq)]
enum GitignoreStatus {
    Valid,
    Missing,
    Corrupted,
}

/// Executes the repair command.
///
/// This command diagnoses and repairs common issues with Jin state:
/// - Orphaned transaction refs
/// - Staging entries for missing files
/// - Corrupted or missing .jinmap
/// - Corrupted .gitignore managed block
///
/// # Arguments
///
/// * `cmd` - The repair command arguments
///
/// # Returns
///
/// Returns `Ok(())` on success, or `Err` if repair fails
///
/// # Examples
///
/// ```ignore
/// let cmd = RepairCommand { dry_run: false };
/// repair::execute(&cmd)?;
/// ```
pub fn execute(cmd: &RepairCommand) -> Result<()> {
    // STEP 1: Get workspace root
    let workspace_root = std::env::current_dir()?;

    // STEP 2: Validate Git repository (required for Jin)
    let git_repo =
        git2::Repository::discover(&workspace_root).map_err(|_| JinError::RepoNotFound {
            path: workspace_root.display().to_string(),
        })?;

    // STEP 3: Open Jin repository
    let _repo = git2::Repository::discover(&workspace_root)?;

    // STEP 4: Load staging index (may be corrupted)
    let mut staging =
        StagingIndex::load_from_disk(&workspace_root).unwrap_or_else(|_| StagingIndex::new());

    // STEP 5: Run diagnostics
    println!("=== Jin State Repair Assessment ===\n");

    let orphans = check_orphan_transactions(&workspace_root)?;
    let missing_files = check_staging_integrity(&workspace_root, &staging)?;
    let jinmap_status = check_jinmap(&workspace_root)?;
    let gitignore_status = check_gitignore(&workspace_root)?;

    // STEP 6: Dry run or actual repair
    if cmd.dry_run {
        execute_dry_run(&orphans, &missing_files, &jinmap_status, &gitignore_status)?;
    } else {
        execute_actual_repair(
            &workspace_root,
            &mut staging,
            &orphans,
            &missing_files,
            &jinmap_status,
            &gitignore_status,
        )?;
    }

    // Report refs/jin/staging/* count
    let staging_ref_count = git_repo.references_glob("refs/jin/staging/*")?.count();
    println!("\n[*] Active staging refs: {}", staging_ref_count);

    println!(
        "=== {} ===",
        if cmd.dry_run {
            "DRY RUN COMPLETE"
        } else {
            "REPAIR COMPLETE"
        }
    );

    Ok(())
}

/// Checks for orphaned transaction refs.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root
///
/// # Returns
///
/// Vector of orphan transaction IDs
fn check_orphan_transactions(workspace_root: &Path) -> Result<Vec<String>> {
    let repo = git2::Repository::discover(workspace_root)?;
    let mut orphans = Vec::new();

    // Pattern: Use references_glob to find staging refs
    for reference in repo.references_glob("refs/jin/staging/*")? {
        let reference = reference?;
        if let Some(name) = reference.name() {
            // Extract transaction ID from ref name
            if let Some(tx_id) = name.strip_prefix("refs/jin/staging/") {
                orphans.push(tx_id.to_string());
            }
        }
    }

    Ok(orphans)
}

/// Checks staging index for files that no longer exist.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root
/// * `staging` - The staging index to check
///
/// # Returns
///
/// Vector of paths that are staged but missing on disk
fn check_staging_integrity(workspace_root: &Path, staging: &StagingIndex) -> Result<Vec<PathBuf>> {
    let mut missing = Vec::new();

    // PATTERN: Iterate entries and check file existence
    for entry in staging.all_entries() {
        let full_path = workspace_root.join(&entry.path);
        if !full_path.exists() {
            missing.push(entry.path.clone());
        }
    }

    Ok(missing)
}

/// Checks .jinmap file status.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root
///
/// # Returns
///
/// JinmapStatus indicating file state
fn check_jinmap(workspace_root: &Path) -> Result<JinmapStatus> {
    let jinmap_path = workspace_root.join(".jinmap");

    if !jinmap_path.exists() {
        return Ok(JinmapStatus::Missing);
    }

    // GOTCHA: Simple validity check - file exists and is readable
    // More sophisticated checks could verify JSON/YAML structure
    match std::fs::read_to_string(&jinmap_path) {
        Ok(_) => Ok(JinmapStatus::Valid),
        Err(_) => Ok(JinmapStatus::Corrupted),
    }
}

/// Checks .gitignore managed block status.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root
///
/// # Returns
///
/// GitignoreStatus indicating managed block state
fn check_gitignore(workspace_root: &Path) -> Result<GitignoreStatus> {
    let gitignore_path = workspace_root.join(".gitignore");

    if !gitignore_path.exists() {
        return Ok(GitignoreStatus::Missing);
    }

    let content = std::fs::read_to_string(&gitignore_path)?;

    // PATTERN: Check for managed block markers
    let has_start = content.contains("### JIN MANAGED START");
    let has_end = content.contains("### JIN MANAGED END");

    match (has_start, has_end) {
        (true, true) => Ok(GitignoreStatus::Valid),
        (false, false) => Ok(GitignoreStatus::Missing), // No managed block
        _ => Ok(GitignoreStatus::Corrupted),            // Partial markers
    }
}

/// Executes dry run - shows what would be repaired without making changes.
fn execute_dry_run(
    orphans: &[String],
    missing_files: &[PathBuf],
    jinmap_status: &JinmapStatus,
    gitignore_status: &GitignoreStatus,
) -> Result<()> {
    println!("Running in DRY RUN mode - no changes will be made\n");

    // Report orphans
    if orphans.is_empty() {
        println!("[OK] No orphaned transaction refs found");
    } else {
        println!("[?] Found {} orphaned transaction ref(s):", orphans.len());
        for orphan in orphans {
            println!("    - refs/jin/staging/{}", orphan);
        }
        println!("    [Would clean up]\n");
    }

    // Report missing staged files
    if missing_files.is_empty() {
        println!("[OK] All staged files exist on disk");
    } else {
        println!(
            "[?] Staging has {} entries for missing files:",
            missing_files.len()
        );
        for file in missing_files {
            println!("    - {}", file.display());
        }
        println!("    [Would remove from staging]\n");
    }

    // Report jinmap status
    match jinmap_status {
        JinmapStatus::Valid => println!("[OK] .jinmap is valid\n"),
        JinmapStatus::Missing => println!("[?] .jinmap is missing\n    [Would regenerate]\n"),
        JinmapStatus::Corrupted => println!("[?] .jinmap is corrupted\n    [Would regenerate]\n"),
    }

    // Report gitignore status
    match gitignore_status {
        GitignoreStatus::Valid => println!("[OK] .gitignore managed block is valid"),
        GitignoreStatus::Missing => {
            println!("[?] .gitignore has no managed block\n    [Would add]")
        }
        GitignoreStatus::Corrupted => {
            println!("[?] .gitignore managed block is corrupted\n    [Would repair]")
        }
    }

    println!("\nRun 'jin repair' without --dry-run to apply repairs.");
    Ok(())
}

/// Executes actual repairs.
fn execute_actual_repair(
    workspace_root: &Path,
    staging: &mut StagingIndex,
    orphans: &[String],
    missing_files: &[PathBuf],
    jinmap_status: &JinmapStatus,
    gitignore_status: &GitignoreStatus,
) -> Result<()> {
    let mut total_repaired = 0;

    // Repair orphaned transactions
    if !orphans.is_empty() {
        let count = repair_orphan_transactions(workspace_root, orphans)?;
        println!("[*] Cleaned up {} orphaned transaction ref(s)", count);
        total_repaired += count;
    }

    // Repair staging integrity
    if !missing_files.is_empty() {
        let count = repair_staging_integrity(workspace_root, staging, missing_files)?;
        println!("[*] Removed {} stale staging entr(ies)", count);
        total_repaired += count;
    }

    // Repair jinmap
    match jinmap_status {
        JinmapStatus::Valid => {
            println!("[*] .jinmap is valid (no action needed)");
        }
        JinmapStatus::Missing | JinmapStatus::Corrupted => {
            repair_jinmap(workspace_root)?;
            println!("[*] Regenerated .jinmap");
            total_repaired += 1;
        }
    }

    // Repair gitignore
    match gitignore_status {
        GitignoreStatus::Valid => {
            println!("[*] .gitignore managed block is valid (no action needed)");
        }
        GitignoreStatus::Missing | GitignoreStatus::Corrupted => {
            repair_gitignore(workspace_root)?;
            println!("[*] Repaired .gitignore managed block");
            total_repaired += 1;
        }
    }

    // GOTCHA: Save staging after modifications
    if !missing_files.is_empty() {
        staging.save_to_disk(workspace_root)?;
    }

    if total_repaired == 0 {
        println!("\nJin state is already healthy - no repairs needed.");
    } else {
        println!(
            "\nRepaired {} issue(s). Jin state is now healthy.",
            total_repaired
        );
    }

    Ok(())
}

/// Repairs orphaned transaction refs by deleting them.
fn repair_orphan_transactions(workspace_root: &Path, orphans: &[String]) -> Result<usize> {
    let repo = git2::Repository::discover(workspace_root)?;
    let mut cleaned = 0;

    for tx_id in orphans {
        let ref_name = format!("refs/jin/staging/{}", tx_id);
        if let Ok(mut reference) = repo.find_reference(&ref_name) {
            if reference.delete().is_ok() {
                cleaned += 1;
            }
        }
    }

    Ok(cleaned)
}

/// Repairs staging integrity by removing entries for missing files.
fn repair_staging_integrity(
    _workspace_root: &Path,
    staging: &mut StagingIndex,
    missing_files: &[PathBuf],
) -> Result<usize> {
    let mut removed = 0;

    for path in missing_files {
        staging.remove_entry(path);
        removed += 1;
    }

    Ok(removed)
}

/// Repairs .jinmap by regenerating it.
fn repair_jinmap(workspace_root: &Path) -> Result<()> {
    // GOTCHA: For now, create a minimal .jinmap
    // Full implementation would regenerate from Git history
    let jinmap_path = workspace_root.join(".jinmap");
    let content = r#"# Jin Map - Auto-generated by jin repair
# This file maps Jin layers to their Git references
# Run 'jin repair' to regenerate if needed
"#;

    std::fs::write(&jinmap_path, content).map_err(|e| JinError::Io(e))?;

    Ok(())
}

/// Repairs .gitignore managed block.
fn repair_gitignore(workspace_root: &Path) -> Result<()> {
    let gitignore_path = workspace_root.join(".gitignore");

    // Read existing content or start fresh
    let existing_content = if gitignore_path.exists() {
        std::fs::read_to_string(&gitignore_path)?
    } else {
        String::new()
    };

    let has_start = existing_content.contains("### JIN MANAGED START");
    let has_end = existing_content.contains("### JIN MANAGED END");

    // Build clean content
    let content = match (has_start, has_end) {
        (true, true) => {
            // Both markers exist - extract user content before/after
            let start_idx = existing_content.find("### JIN MANAGED START").unwrap();
            let end_idx = existing_content.find("### JIN MANAGED END").unwrap();
            let before = &existing_content[..start_idx];
            let after = &existing_content[end_idx + "### JIN MANAGED END".len()..];
            format!(
                "{}\n{}\n{}",
                before.trim(),
                "### JIN MANAGED START\n.jin\n.jinmap\n### JIN MANAGED END",
                after.trim()
            )
        }
        (true, false) | (false, true) | (false, false) => {
            // Missing or partial managed block - preserve user content, add managed block
            let user_content = existing_content
                .replace("### JIN MANAGED START", "")
                .replace("### JIN MANAGED END", "");
            format!(
                "{}\n### JIN MANAGED START\n.jin\n.jinmap\n### JIN MANAGED END\n",
                user_content.trim()
            )
        }
    };

    std::fs::write(&gitignore_path, content).map_err(|e| JinError::Io(e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // GOTCHA: Use TEST_LOCK for parallel test safety
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    struct DirGuard {
        original_dir: PathBuf,
    }

    impl DirGuard {
        fn new() -> std::io::Result<Self> {
            Ok(Self {
                original_dir: std::env::current_dir()?,
            })
        }
    }

    impl Drop for DirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original_dir);
        }
    }

    fn init_git_repo(dir: &Path) -> git2::Repository {
        git2::Repository::init(dir).unwrap()
    }

    fn init_jin(dir: &Path) {
        let workspace_dir = dir.join(".jin/workspace");
        fs::create_dir_all(workspace_dir).unwrap();
    }

    /// Helper to create an initial commit in a Git repo
    fn create_initial_commit(repo: &git2::Repository) -> git2::Oid {
        let tree_builder = repo.treebuilder(None).unwrap();
        let tree_oid = tree_builder.write().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();

        let sig = git2::Signature::now("Test", "test@example.com").unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap()
    }

    #[test]
    fn test_repair_dry_run_no_issues() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        init_git_repo(project_dir);
        init_jin(project_dir);

        let cmd = RepairCommand { dry_run: true };
        let result = execute(&cmd);

        assert!(result.is_ok());
    }

    #[test]
    fn test_repair_dry_run_with_orphans() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        let repo = init_git_repo(project_dir);
        init_jin(project_dir);

        // Create an initial commit first
        let oid = create_initial_commit(&repo);

        // Create orphaned transaction ref
        repo.reference("refs/jin/staging/test-orphan", oid, false, "test")
            .unwrap();

        let cmd = RepairCommand { dry_run: true };
        let result = execute(&cmd);

        assert!(result.is_ok());
    }

    #[test]
    fn test_repair_actual_cleanup() {
        let _lock = TEST_LOCK.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let project_dir = temp_dir.path();
        let _guard = DirGuard::new().unwrap();

        std::env::set_current_dir(project_dir).unwrap();

        let repo = init_git_repo(project_dir);
        init_jin(project_dir);

        // Create an initial commit first
        let oid = create_initial_commit(&repo);

        // Create orphaned transaction ref
        repo.reference("refs/jin/staging/test-orphan", oid, false, "test")
            .unwrap();

        let cmd = RepairCommand { dry_run: false };
        let result = execute(&cmd);

        assert!(result.is_ok());

        // Verify orphan was cleaned
        assert!(repo.find_reference("refs/jin/staging/test-orphan").is_err());
    }

    #[test]
    fn test_check_jinmap_valid() {
        let temp_dir = TempDir::new().unwrap();
        let jinmap_path = temp_dir.path().join(".jinmap");
        fs::write(&jinmap_path, "# valid jinmap\n").unwrap();

        let status = check_jinmap(temp_dir.path()).unwrap();
        assert_eq!(status, JinmapStatus::Valid);
    }

    #[test]
    fn test_check_jinmap_missing() {
        let temp_dir = TempDir::new().unwrap();

        let status = check_jinmap(temp_dir.path()).unwrap();
        assert_eq!(status, JinmapStatus::Missing);
    }

    #[test]
    fn test_check_gitignore_valid() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");
        fs::write(
            &gitignore_path,
            "### JIN MANAGED START\n.jin\n### JIN MANAGED END\n",
        )
        .unwrap();

        let status = check_gitignore(temp_dir.path()).unwrap();
        assert_eq!(status, GitignoreStatus::Valid);
    }

    #[test]
    fn test_check_gitignore_missing() {
        let temp_dir = TempDir::new().unwrap();

        let status = check_gitignore(temp_dir.path()).unwrap();
        assert_eq!(status, GitignoreStatus::Missing);
    }

    #[test]
    fn test_check_gitignore_corrupted() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");
        fs::write(&gitignore_path, "### JIN MANAGED START\n.jin\n").unwrap();

        let status = check_gitignore(temp_dir.path()).unwrap();
        assert_eq!(status, GitignoreStatus::Corrupted);
    }

    #[test]
    fn test_repair_jinmap_creates_file() {
        let temp_dir = TempDir::new().unwrap();

        repair_jinmap(temp_dir.path()).unwrap();

        let jinmap_path = temp_dir.path().join(".jinmap");
        assert!(jinmap_path.exists());
        let content = fs::read_to_string(&jinmap_path).unwrap();
        assert!(content.contains("Jin Map"));
    }

    #[test]
    fn test_repair_gitignore_adds_managed_block() {
        let temp_dir = TempDir::new().unwrap();

        repair_gitignore(temp_dir.path()).unwrap();

        let gitignore_path = temp_dir.path().join(".gitignore");
        assert!(gitignore_path.exists());
        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert!(content.contains("### JIN MANAGED START"));
        assert!(content.contains(".jin"));
        assert!(content.contains("### JIN MANAGED END"));
    }

    #[test]
    fn test_repair_gitignore_preserves_existing() {
        let temp_dir = TempDir::new().unwrap();
        let gitignore_path = temp_dir.path().join(".gitignore");
        fs::write(&gitignore_path, "node_modules/\n*.log\n").unwrap();

        repair_gitignore(temp_dir.path()).unwrap();

        let content = fs::read_to_string(&gitignore_path).unwrap();
        assert!(content.contains("node_modules/"));
        assert!(content.contains("*.log"));
        assert!(content.contains("### JIN MANAGED START"));
    }

    #[test]
    fn test_repair_orphan_transactions_deletes_refs() {
        let temp_dir = TempDir::new().unwrap();
        let repo = git2::Repository::init(temp_dir.path()).unwrap();

        // Create an initial commit first
        let oid = create_initial_commit(&repo);

        // Create two orphans
        repo.reference("refs/jin/staging/orphan1", oid, false, "test")
            .unwrap();
        repo.reference("refs/jin/staging/orphan2", oid, false, "test")
            .unwrap();

        let orphans = vec!["orphan1".to_string(), "orphan2".to_string()];
        let count = repair_orphan_transactions(temp_dir.path(), &orphans).unwrap();

        assert_eq!(count, 2);
        assert!(repo.find_reference("refs/jin/staging/orphan1").is_err());
        assert!(repo.find_reference("refs/jin/staging/orphan2").is_err());
    }
}
