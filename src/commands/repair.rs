//! Implementation of `jin repair`
//!
//! Verifies and repairs Jin repository integrity by checking:
//! 1. Repository structure (~/.jin/ is valid bare repo)
//! 2. Layer refs (refs/jin/layers/* point to valid commits)
//! 3. Staging index (.jin/staging/index.json is parseable)
//! 4. .jinmap (.jin/.jinmap exists and is valid)
//! 5. Workspace metadata (.jin/workspace/ tracking files)

use crate::cli::RepairArgs;
use crate::core::{JinConfig, JinError, ProjectContext, Result};
use crate::git::{JinRepo, RefOps};
use crate::staging::{validate_workspace_attached, StagingIndex, WorkspaceMetadata};
use std::path::PathBuf;

/// Execute the repair command
///
/// Checks Jin repository integrity and repairs issues if not in dry-run mode.
///
/// # Arguments
///
/// * `args` - Command line arguments including dry_run flag
///
/// # Errors
///
/// Returns an error if:
/// - Fatal corruption is detected (manual recovery required)
/// - Repair operations fail
pub fn execute(args: RepairArgs) -> Result<()> {
    println!("Checking Jin repository integrity...");
    println!();

    let mut issues_found = Vec::new();
    let mut issues_fixed = Vec::new();

    // Check workspace attachment if --check flag is set
    if args.check {
        check_workspace_attachment(&args, &mut issues_found);

        // Display summary and return early
        println!();
        if issues_found.is_empty() {
            println!("Workspace is properly attached.");
        } else {
            println!("Workspace state:");
            for issue in &issues_found {
                println!("  - {}", issue);
            }
        }
        return Ok(());
    }

    // Check 1: Repository structure
    let repo_result = check_repository_structure(&args, &mut issues_found, &mut issues_fixed);

    // If repository is fatally corrupted, we can't continue (except in dry-run)
    if let Err(e) = &repo_result {
        if matches!(e, JinError::Other(_))
            && issues_found.len() > issues_fixed.len()
            && !args.dry_run
        {
            println!();
            println!("FATAL: Repository is severely corrupted.");
            println!();
            println!("Manual recovery steps:");
            println!("  1. Backup current repository: cp -r ~/.jin ~/.jin.backup");
            println!("  2. Reinitialize Jin: rm -rf ~/.jin && jin init");
            println!("  3. Restore configurations from backup if possible");
            return Err(JinError::Other(
                "Repository corruption requires manual recovery".to_string(),
            ));
        }
    }

    // Continue with other checks if we have a repository
    let repo = repo_result.ok();

    // Check 2: Layer refs (only if repository is valid)
    if let Some(ref repo) = repo {
        check_layer_refs(&args, repo, &mut issues_found, &mut issues_fixed);
    }

    // Check 3: Staging index
    check_staging_index(&args, &mut issues_found, &mut issues_fixed);

    // Check 4: .jinmap
    check_jinmap(&args, &mut issues_found, &mut issues_fixed);

    // Check 5: Workspace metadata
    check_workspace_metadata(&args, &mut issues_found, &mut issues_fixed);

    // Check 6: Global configuration
    check_global_config(&args, &mut issues_found, &mut issues_fixed);

    // Check 7: Project context
    check_project_context(&args, &mut issues_found, &mut issues_fixed);

    // Display summary
    println!();
    if args.dry_run {
        if issues_found.is_empty() {
            println!("No issues found.");
        } else {
            println!(
                "{} issue{} found (dry run - no changes made)",
                issues_found.len(),
                if issues_found.len() == 1 { "" } else { "s" }
            );
            println!();
            println!("Issues detected:");
            for issue in &issues_found {
                println!("  - {}", issue);
            }
        }
    } else if issues_found.is_empty() {
        println!("No issues found.");
    } else {
        println!(
            "Repair complete. {} issue{} fixed.",
            issues_fixed.len(),
            if issues_fixed.len() == 1 { "" } else { "s" }
        );

        if issues_fixed.len() < issues_found.len() {
            println!();
            println!(
                "Warning: {} issue{} could not be automatically fixed:",
                issues_found.len() - issues_fixed.len(),
                if issues_found.len() - issues_fixed.len() == 1 {
                    ""
                } else {
                    "s"
                }
            );
            for (i, issue) in issues_found.iter().enumerate() {
                if i >= issues_fixed.len() {
                    println!("  - {}", issue);
                }
            }
        }
    }

    Ok(())
}

/// Check 1: Repository structure
fn check_repository_structure(
    args: &RepairArgs,
    issues_found: &mut Vec<String>,
    issues_fixed: &mut Vec<String>,
) -> Result<JinRepo> {
    print!("Checking repository structure... ");

    match JinRepo::open() {
        Ok(repo) => {
            // Verify it's actually a bare repository
            if !repo.inner().is_bare() {
                println!("✗");
                let issue = "Repository exists but is not bare".to_string();
                issues_found.push(issue.clone());

                if !args.dry_run {
                    // This is a fatal error - can't automatically fix
                    eprintln!("Error: Repository at ~/.jin exists but is not a bare repository");
                    eprintln!("Manual intervention required.");
                } else {
                    println!("  Issue: {}", issue);
                }

                return Err(JinError::Other("Repository is not bare".to_string()));
            }

            println!("✓");
            Ok(repo)
        }
        Err(_) => {
            println!("✗");
            let issue = "Repository not found or corrupted".to_string();
            issues_found.push(issue.clone());

            if !args.dry_run {
                // Try to recreate repository
                match JinRepo::create() {
                    Ok(repo) => {
                        let fix = "Repository recreated".to_string();
                        issues_fixed.push(fix.clone());
                        println!("  Fixed: {}", fix);
                        Ok(repo)
                    }
                    Err(e) => {
                        eprintln!("  Failed to recreate repository: {}", e);
                        Err(e)
                    }
                }
            } else {
                println!("  Issue: {}", issue);
                println!("  Would recreate repository");
                Err(JinError::Other("Repository missing".to_string()))
            }
        }
    }
}

/// Check 2: Layer refs
fn check_layer_refs(
    args: &RepairArgs,
    repo: &JinRepo,
    issues_found: &mut Vec<String>,
    issues_fixed: &mut Vec<String>,
) {
    print!("Checking layer references... ");

    // Get all layer refs
    let refs_result = repo.list_refs("refs/jin/layers/*");

    match refs_result {
        Ok(refs) => {
            let mut invalid_refs = Vec::new();

            // Check each ref points to a valid commit
            for ref_name in &refs {
                if let Ok(oid) = repo.resolve_ref(ref_name) {
                    // Check if OID points to a valid commit
                    if let Ok(obj) = repo.inner().find_object(oid, None) {
                        if obj.kind() != Some(git2::ObjectType::Commit) {
                            invalid_refs.push((ref_name.clone(), "not a commit".to_string()));
                        }
                    } else {
                        invalid_refs.push((ref_name.clone(), "object not found".to_string()));
                    }
                } else {
                    invalid_refs.push((ref_name.clone(), "cannot resolve".to_string()));
                }
            }

            if invalid_refs.is_empty() {
                println!("✓");
            } else {
                println!("✗");

                for (ref_name, reason) in &invalid_refs {
                    let issue = format!("Invalid ref {}: {}", ref_name, reason);
                    issues_found.push(issue.clone());

                    if !args.dry_run {
                        // Try to recover from reflog
                        match recover_ref_from_reflog(repo, ref_name) {
                            Ok(true) => {
                                let fix = format!("Recovered {} from reflog", ref_name);
                                issues_fixed.push(fix.clone());
                                println!("  Fixed: {}", fix);
                            }
                            Ok(false) => {
                                // No valid reflog entry, delete the ref
                                if let Ok(()) = repo.delete_ref(ref_name) {
                                    let fix = format!("Deleted invalid ref {}", ref_name);
                                    issues_fixed.push(fix.clone());
                                    println!("  Fixed: {}", fix);
                                } else {
                                    println!("  Failed to delete ref {}", ref_name);
                                }
                            }
                            Err(e) => {
                                println!("  Failed to recover {}: {}", ref_name, e);
                            }
                        }
                    } else {
                        println!("  Issue: {}", issue);
                        println!("    Would attempt recovery from reflog");
                    }
                }
            }
        }
        Err(e) => {
            println!("✗");
            let issue = format!("Cannot list layer refs: {}", e);
            issues_found.push(issue.clone());
            println!("  Issue: {}", issue);
        }
    }
}

/// Attempt to recover a ref from reflog
fn recover_ref_from_reflog(repo: &JinRepo, ref_name: &str) -> Result<bool> {
    // Try to read reflog
    let reflog = repo.inner().reflog(ref_name)?;

    // Find the most recent valid commit
    for i in 0..reflog.len() {
        if let Some(entry) = reflog.get(i) {
            let oid = entry.id_new();

            // Check if this OID points to a valid commit
            if let Ok(obj) = repo.inner().find_object(oid, None) {
                if obj.kind() == Some(git2::ObjectType::Commit) {
                    // Found a valid commit, restore the ref
                    let message = format!("Recovered from reflog entry {}", i);
                    repo.set_ref(ref_name, oid, &message)?;
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

/// Check 3: Staging index
fn check_staging_index(
    args: &RepairArgs,
    issues_found: &mut Vec<String>,
    issues_fixed: &mut Vec<String>,
) {
    print!("Checking staging index... ");

    let index_path = StagingIndex::default_path();

    if !index_path.exists() {
        // Missing index is not an issue - just means nothing is staged
        println!("✓ (not present)");
        return;
    }

    match StagingIndex::load() {
        Ok(_) => {
            println!("✓");
        }
        Err(_) => {
            println!("✗");
            let issue = "Staging index corrupted".to_string();
            issues_found.push(issue.clone());

            if !args.dry_run {
                // Rebuild index - we lose staging data but it's better than corruption
                match rebuild_staging_index(&index_path) {
                    Ok(()) => {
                        let fix = "Staging index rebuilt (staged changes lost)".to_string();
                        issues_fixed.push(fix.clone());
                        println!("  Fixed: {}", fix);
                    }
                    Err(e) => {
                        println!("  Failed to rebuild index: {}", e);
                    }
                }
            } else {
                println!("  Issue: {}", issue);
                println!("    Would rebuild index (staged changes would be lost)");
            }
        }
    }
}

/// Rebuild a corrupted staging index
fn rebuild_staging_index(index_path: &PathBuf) -> Result<()> {
    // Create a new empty index
    let index = StagingIndex::new();

    // Backup corrupted index
    let backup_path = index_path.with_extension("json.corrupted");
    if index_path.exists() {
        std::fs::rename(index_path, backup_path)?;
    }

    // Save new index
    index.save()?;

    Ok(())
}

/// Check 4: .jinmap
fn check_jinmap(args: &RepairArgs, issues_found: &mut Vec<String>, issues_fixed: &mut Vec<String>) {
    print!("Checking .jinmap... ");

    let jinmap_path = crate::core::JinMap::default_path();

    if !jinmap_path.exists() {
        // .jinmap is optional, but if .jin/ exists it should have one
        let jin_dir = jinmap_path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from(".jin"));
        if jin_dir.exists() && jin_dir.is_dir() {
            println!("✗");
            let issue = ".jinmap missing".to_string();
            issues_found.push(issue.clone());

            if !args.dry_run {
                // Create default .jinmap
                match create_default_jinmap(&jinmap_path) {
                    Ok(()) => {
                        let fix = ".jinmap created".to_string();
                        issues_fixed.push(fix.clone());
                        println!("  Fixed: {}", fix);
                    }
                    Err(e) => {
                        println!("  Failed to create .jinmap: {}", e);
                    }
                }
            } else {
                println!("  Issue: {}", issue);
                println!("    Would create default .jinmap");
            }
        } else {
            println!("✓ (not initialized)");
        }
        return;
    }

    // Verify it's valid YAML
    match std::fs::read_to_string(&jinmap_path) {
        Ok(content) => match serde_yaml::from_str::<serde_yaml::Value>(&content) {
            Ok(_) => {
                println!("✓");
            }
            Err(_) => {
                println!("✗");
                let issue = ".jinmap is not valid YAML".to_string();
                issues_found.push(issue.clone());

                if !args.dry_run {
                    // Backup and recreate
                    match repair_jinmap(&jinmap_path) {
                        Ok(()) => {
                            let fix =
                                ".jinmap repaired (backed up to .jinmap.corrupted)".to_string();
                            issues_fixed.push(fix.clone());
                            println!("  Fixed: {}", fix);
                        }
                        Err(e) => {
                            println!("  Failed to repair .jinmap: {}", e);
                        }
                    }
                } else {
                    println!("  Issue: {}", issue);
                    println!("    Would backup and recreate .jinmap");
                }
            }
        },
        Err(_) => {
            println!("✗");
            let issue = ".jinmap is not readable".to_string();
            issues_found.push(issue.clone());
            println!("  Issue: {}", issue);
        }
    }
}

/// Create a default .jinmap file
fn create_default_jinmap(path: &PathBuf) -> Result<()> {
    let default_content = r#"# Jin layer map
# This file tracks which files belong to which layers
# Format: path: layer_name
"#;

    std::fs::write(path, default_content)?;
    Ok(())
}

/// Repair corrupted .jinmap
fn repair_jinmap(path: &PathBuf) -> Result<()> {
    // Backup corrupted file
    let backup_path = path.with_extension("jinmap.corrupted");
    std::fs::rename(path, backup_path)?;

    // Create new default
    create_default_jinmap(path)?;

    Ok(())
}

/// Check 5: Workspace metadata
fn check_workspace_metadata(
    args: &RepairArgs,
    issues_found: &mut Vec<String>,
    issues_fixed: &mut Vec<String>,
) {
    print!("Checking workspace metadata... ");

    let metadata_path = WorkspaceMetadata::default_path();

    if !metadata_path.exists() {
        // Missing metadata is not an issue - just means no apply has been done
        println!("✓ (not present)");
        return;
    }

    match WorkspaceMetadata::load() {
        Ok(_) => {
            println!("✓");
        }
        Err(_) => {
            println!("✗");
            let issue = "Workspace metadata corrupted".to_string();
            issues_found.push(issue.clone());

            if !args.dry_run {
                // Rebuild metadata
                match rebuild_workspace_metadata(&metadata_path) {
                    Ok(()) => {
                        let fix = "Workspace metadata rebuilt".to_string();
                        issues_fixed.push(fix.clone());
                        println!("  Fixed: {}", fix);
                    }
                    Err(e) => {
                        println!("  Failed to rebuild metadata: {}", e);
                    }
                }
            } else {
                println!("  Issue: {}", issue);
                println!("    Would rebuild metadata");
            }
        }
    }
}

/// Rebuild corrupted workspace metadata
fn rebuild_workspace_metadata(path: &PathBuf) -> Result<()> {
    // Backup corrupted metadata
    let backup_path = path.with_extension("json.corrupted");
    if path.exists() {
        std::fs::rename(path, backup_path)?;
    }

    // Create new empty metadata
    let metadata = WorkspaceMetadata::new();
    metadata.save()?;

    Ok(())
}

/// Check 6: Global configuration
fn check_global_config(
    args: &RepairArgs,
    issues_found: &mut Vec<String>,
    issues_fixed: &mut Vec<String>,
) {
    print!("Checking global configuration... ");

    match JinConfig::load() {
        Ok(_) => {
            println!("✓");
        }
        Err(JinError::Config(_)) => {
            // Config exists but is invalid
            println!("✗");
            let issue = "Global config is invalid".to_string();
            issues_found.push(issue.clone());

            if !args.dry_run {
                match repair_global_config() {
                    Ok(()) => {
                        let fix = "Global config repaired (backed up to config.toml.corrupted)"
                            .to_string();
                        issues_fixed.push(fix.clone());
                        println!("  Fixed: {}", fix);
                    }
                    Err(e) => {
                        println!("  Failed to repair config: {}", e);
                    }
                }
            } else {
                println!("  Issue: {}", issue);
                println!("    Would backup and recreate config");
            }
        }
        Err(_) => {
            // Config doesn't exist - this is fine, use defaults
            println!("✓ (using defaults)");
        }
    }
}

/// Repair global configuration
fn repair_global_config() -> Result<()> {
    let config_path = JinConfig::default_path()?;

    // Backup corrupted config
    if config_path.exists() {
        let backup_path = config_path.with_extension("toml.corrupted");
        std::fs::rename(&config_path, backup_path)?;
    }

    // Create new default config
    let config = JinConfig::default();
    config.save()?;

    Ok(())
}

/// Check 7: Project context
fn check_project_context(
    args: &RepairArgs,
    issues_found: &mut Vec<String>,
    issues_fixed: &mut Vec<String>,
) {
    print!("Checking project context... ");

    let context_path = ProjectContext::default_path();

    // Only check if .jin directory exists
    if !context_path.parent().map(|p| p.exists()).unwrap_or(false) {
        println!("✓ (not initialized)");
        return;
    }

    if !context_path.exists() {
        println!("✗");
        let issue = "Project context missing".to_string();
        issues_found.push(issue.clone());

        if !args.dry_run {
            match create_default_context() {
                Ok(()) => {
                    let fix = "Project context created".to_string();
                    issues_fixed.push(fix.clone());
                    println!("  Fixed: {}", fix);
                }
                Err(e) => {
                    println!("  Failed to create context: {}", e);
                }
            }
        } else {
            println!("  Issue: {}", issue);
            println!("    Would create default context");
        }
        return;
    }

    match ProjectContext::load() {
        Ok(_) => {
            println!("✓");
        }
        Err(_) => {
            println!("✗");
            let issue = "Project context corrupted".to_string();
            issues_found.push(issue.clone());

            if !args.dry_run {
                match repair_project_context() {
                    Ok(()) => {
                        let fix =
                            "Project context repaired (backed up to context.corrupted)".to_string();
                        issues_fixed.push(fix.clone());
                        println!("  Fixed: {}", fix);
                    }
                    Err(e) => {
                        println!("  Failed to repair context: {}", e);
                    }
                }
            } else {
                println!("  Issue: {}", issue);
                println!("    Would backup and recreate context");
            }
        }
    }
}

/// Create default project context
fn create_default_context() -> Result<()> {
    let context = ProjectContext::default();
    context.save()?;
    Ok(())
}

/// Repair project context
fn repair_project_context() -> Result<()> {
    let context_path = ProjectContext::default_path();

    // Backup corrupted context
    if context_path.exists() {
        let backup_path = context_path.with_extension("corrupted");
        std::fs::rename(&context_path, backup_path)?;
    }

    // Create new default context
    create_default_context()?;

    Ok(())
}

/// Check workspace attachment state
///
/// Validates that the workspace is properly attached to the active context.
/// This check is used by the --check flag to diagnose detached workspace issues.
fn check_workspace_attachment(args: &RepairArgs, issues_found: &mut Vec<String>) {
    print!("Checking workspace attachment... ");

    // Load project context
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            println!("✓ (not initialized)");
            return;
        }
        Err(_) => {
            println!("✓ (no context)");
            return;
        }
    };

    // Open Jin repository
    let repo = match JinRepo::open() {
        Ok(r) => r,
        Err(_) => {
            println!("✓ (no repository)");
            return;
        }
    };

    // Validate workspace
    match validate_workspace_attached(&context, &repo) {
        Ok(()) => {
            println!("✓");
        }
        Err(JinError::DetachedWorkspace {
            details,
            recovery_hint,
            ..
        }) => {
            println!("✗");
            let issue = format!(
                "Workspace is detached. {}. Recovery: {}",
                details, recovery_hint
            );
            issues_found.push(issue.clone());

            if !args.dry_run {
                println!("  Issue: {}", issue);
            }
        }
        Err(e) => {
            println!("✗");
            let issue = format!("Workspace check failed: {}", e);
            issues_found.push(issue.clone());

            if !args.dry_run {
                println!("  Issue: {}", issue);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    /// Scope guard that restores the original directory when dropped
    struct DirGuard {
        original_dir: PathBuf,
        _temp: TempDir, // Keep temp alive
    }

    impl DirGuard {
        fn new(temp: TempDir) -> Self {
            let original_dir = std::env::current_dir().unwrap();
            std::env::set_current_dir(temp.path()).unwrap();
            Self {
                original_dir,
                _temp: temp,
            }
        }
    }

    impl Drop for DirGuard {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.original_dir).ok();
        }
    }

    fn setup_isolated_test() -> TempDir {
        let temp = TempDir::new().unwrap();
        let jin_dir = temp.path().join(".jin_global");
        // Create parent directory for JIN_DIR to ensure JinRepo::create() works
        std::fs::create_dir_all(&jin_dir).unwrap();
        std::env::set_var("JIN_DIR", &jin_dir);
        // Do NOT change current directory - tests should use temp.path() explicitly
        temp
    }

    #[test]
    #[serial]
    fn test_execute_dry_run() {
        let _guard = DirGuard::new(setup_isolated_test());

        let args = RepairArgs {
            dry_run: true,
            check: false,
        };
        let result = execute(args);
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_execute_no_issues() {
        let _guard = DirGuard::new(setup_isolated_test());

        let args = RepairArgs {
            dry_run: false,
            check: false,
        };
        let result = execute(args);
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_check_staging_index_missing() {
        let _guard = DirGuard::new(setup_isolated_test());

        let args = RepairArgs {
            dry_run: true,
            check: false,
        };
        let mut issues_found = Vec::new();
        let mut issues_fixed = Vec::new();

        check_staging_index(&args, &mut issues_found, &mut issues_fixed);

        // Missing index is fine
        assert_eq!(issues_found.len(), 0);
    }

    #[test]
    #[serial]
    fn test_check_staging_index_corrupted() {
        let temp = TempDir::new().unwrap();

        // Create corrupted staging index in temp directory
        let index_path = temp.path().join(".jin/staging/index.json");
        std::fs::create_dir_all(index_path.parent().unwrap()).unwrap();
        std::fs::write(&index_path, "invalid json").unwrap();

        // Use DirGuard to change to temp directory and auto-restore
        let _guard = DirGuard::new(temp);

        let args = RepairArgs {
            dry_run: true,
            check: false,
        };
        let mut issues_found = Vec::new();
        let mut issues_fixed = Vec::new();

        check_staging_index(&args, &mut issues_found, &mut issues_fixed);

        assert_eq!(issues_found.len(), 1);
        assert!(issues_found[0].contains("corrupted"));
    }

    #[test]
    fn test_create_default_jinmap() {
        let temp = TempDir::new().unwrap();
        let jinmap_path = temp.path().join(".jinmap");

        let result = create_default_jinmap(&jinmap_path);
        assert!(result.is_ok());
        assert!(jinmap_path.exists());

        let content = std::fs::read_to_string(&jinmap_path).unwrap();
        assert!(content.contains("Jin layer map"));
    }

    #[test]
    #[serial]
    fn test_rebuild_staging_index() {
        let temp = TempDir::new().unwrap();

        // Create corrupted index in temp directory
        let index_path = temp.path().join(".jin/staging/index.json");
        std::fs::create_dir_all(index_path.parent().unwrap()).unwrap();
        std::fs::write(&index_path, "invalid json").unwrap();

        // Use DirGuard to change to temp directory and auto-restore
        let _guard = DirGuard::new(temp);

        let result = rebuild_staging_index(&index_path);
        assert!(result.is_ok());

        // Verify new index is valid
        let loaded = StagingIndex::load();
        assert!(loaded.is_ok());

        // Verify backup was created
        let backup_path = index_path.with_extension("json.corrupted");
        assert!(backup_path.exists());
    }

    #[test]
    #[serial]
    fn test_check_jinmap_valid_yaml() {
        let temp = TempDir::new().unwrap();

        // Create .jin directory and valid .jinmap in temp directory
        let jin_dir = temp.path().join(".jin");
        std::fs::create_dir_all(&jin_dir).unwrap();
        // Use a valid JinMap YAML structure - proper format with mappings as empty map
        std::fs::write(
            jin_dir.join(".jinmap"),
            "---\nversion: 1\nmappings: {}\nmeta:\n  generated-by: jin\n",
        )
        .unwrap();

        // Use DirGuard to change to temp directory and auto-restore
        let _guard = DirGuard::new(temp);

        let args = RepairArgs {
            dry_run: true,
            check: false,
        };
        let mut issues_found = Vec::new();
        let mut issues_fixed = Vec::new();

        check_jinmap(&args, &mut issues_found, &mut issues_fixed);

        assert_eq!(issues_found.len(), 0);
    }

    #[test]
    #[serial]
    fn test_check_jinmap_invalid_yaml() {
        let temp = TempDir::new().unwrap();

        // Create .jin directory and invalid .jinmap in temp directory
        let jin_dir = temp.path().join(".jin");
        std::fs::create_dir_all(&jin_dir).unwrap();
        // Use content that YAML will actually reject - unclosed quote
        std::fs::write(jin_dir.join(".jinmap"), "key: \"unclosed").unwrap();

        // Use DirGuard to change to temp directory and auto-restore
        let _guard = DirGuard::new(temp);

        let args = RepairArgs {
            dry_run: true,
            check: false,
        };
        let mut issues_found = Vec::new();
        let mut issues_fixed = Vec::new();

        check_jinmap(&args, &mut issues_found, &mut issues_fixed);

        assert_eq!(issues_found.len(), 1);
        assert!(issues_found[0].contains("not valid YAML"));
    }

    #[test]
    #[serial]
    fn test_check_workspace_metadata_missing() {
        let _guard = DirGuard::new(TempDir::new().unwrap());

        let args = RepairArgs {
            dry_run: true,
            check: false,
        };
        let mut issues_found = Vec::new();
        let mut issues_fixed = Vec::new();

        check_workspace_metadata(&args, &mut issues_found, &mut issues_fixed);

        // Missing metadata is fine
        assert_eq!(issues_found.len(), 0);
    }

    #[test]
    #[serial]
    fn test_rebuild_workspace_metadata() {
        let temp = setup_isolated_test();

        // Create corrupted metadata
        let metadata_path = temp.path().join(WorkspaceMetadata::default_path());
        std::fs::create_dir_all(metadata_path.parent().unwrap()).unwrap();
        std::fs::write(&metadata_path, "invalid json").unwrap();

        // Use DirGuard to change to temp directory and auto-restore
        let _guard = DirGuard::new(temp);

        let result = rebuild_workspace_metadata(&metadata_path);
        assert!(result.is_ok());

        // Verify new metadata is valid
        let loaded = WorkspaceMetadata::load();
        assert!(loaded.is_ok());

        // Verify backup was created (with correct path)
        let parent = metadata_path.parent().unwrap();
        let backup_path = parent.join("last_applied.json.corrupted");
        assert!(
            backup_path.exists(),
            "Backup file should exist at {}",
            backup_path.display()
        );
    }

    #[test]
    #[serial]
    fn test_create_default_context() {
        let temp = setup_isolated_test();

        // Create .jin directory first - this is required for context to be saved
        let jin_dir = temp.path().join(".jin");
        std::fs::create_dir_all(&jin_dir).unwrap();

        // Use DirGuard to change to temp directory and auto-restore
        let _guard = DirGuard::new(temp);

        let result = create_default_context();
        assert!(result.is_ok(), "Failed to create context: {:?}", result);

        let context_path = ProjectContext::default_path();
        assert!(
            context_path.exists(),
            "Context file was not created at {}",
            context_path.display()
        );
    }
}
