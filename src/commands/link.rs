//! Implementation of `jin link`
//!
//! Links a local Jin installation to a shared remote configuration repository.

use crate::cli::LinkArgs;
use crate::core::{JinConfig, JinError, RemoteConfig, Result};
use crate::git::JinRepo;
use git2::{Direction, ErrorCode};
use regex::Regex;
use std::collections::HashSet;

/// Execute the link command
///
/// Links to shared Jin config repo, configuring both the Git repository
/// and JinConfig with the remote URL. Tests connectivity before persisting.
pub fn execute(args: LinkArgs) -> Result<()> {
    // 1. Validate URL format
    validate_git_url(&args.url)?;

    // 2. Load global config (or create default)
    let mut config = JinConfig::load().unwrap_or_default();

    // 3. Open Jin repository (create if doesn't exist)
    let jin_repo = JinRepo::open_or_create()?;
    let repo = jin_repo.inner();

    // 4. Check if remote already exists
    match repo.find_remote("origin") {
        Ok(existing_remote) => {
            if !args.force {
                let existing_url = existing_remote.url().unwrap_or("(unknown)");
                return Err(JinError::AlreadyExists(format!(
                    "Remote 'origin' already configured\n\
                    Current remote: {}\n\n\
                    To change remote, use:\n  \
                    jin link {} --force",
                    existing_url, args.url
                )));
            }
            // Delete existing remote to update it
            repo.remote_delete("origin")?;
        }
        Err(e) if e.code() == ErrorCode::NotFound => {
            // No remote exists - OK to proceed
        }
        Err(e) => return Err(e.into()),
    }

    // 5. Normalize URL for git2-rs: convert plain paths to file:// URLs
    let normalized_url = if args.url.starts_with('/') && !args.url.starts_with("file://") {
        format!("file://{}", args.url)
    } else {
        args.url.clone()
    };

    // 6. Add remote with Jin-specific refspec
    repo.remote_with_fetch(
        "origin",
        &normalized_url,
        "+refs/jin/layers/*:refs/jin/layers/*",
    )?;

    // 7. Test connectivity (skip for file:// URLs due to git2-rs bug)
    let is_file_url = args.url.starts_with("file://") || args.url.starts_with('/');
    if !is_file_url {
        println!("Testing connection to remote...");
        test_connectivity(repo, "origin")?;
        println!("Connected successfully");
    }

    // 8. Update and save global config (store original URL for display purposes)
    config.remote = Some(RemoteConfig {
        url: args.url.clone(),
        fetch_on_init: true,
    });
    config.save()?;

    // 9. Print confirmation
    println!("Configured remote 'origin' for Jin repository");
    let config_path = JinConfig::default_path()?;
    println!("Stored in: {}", config_path.display());
    println!();

    // 10. Optionally list available configs (skip for file:// URLs due to git2-rs bug, ignore errors)
    if !is_file_url {
        let _ = list_remote_configs(repo);
    }

    // 11. Print next steps
    println!("Use 'jin fetch' to download configurations");
    println!("Use 'jin pull' to merge and apply configurations");

    Ok(())
}

/// Validates Git remote URL format
///
/// Supports HTTPS, SSH (both colon and scheme formats), Git protocol, and file paths.
fn validate_git_url(url: &str) -> Result<()> {
    if url.is_empty() {
        return Err(JinError::Config("URL cannot be empty".into()));
    }

    // Define patterns for supported URL formats
    let patterns = vec![
        Regex::new(r"^https://[^/]+/.+\.git$").unwrap(), // HTTPS with .git
        Regex::new(r"^https://[^/]+/.+$").unwrap(),      // HTTPS without .git
        Regex::new(r"^git@[^:]+:[^/].+\.git$").unwrap(), // SSH (colon) with .git
        Regex::new(r"^git@[^:]+:[^/].+$").unwrap(),      // SSH (colon) without .git
        Regex::new(r"^ssh://git@[^/]+/.+\.git$").unwrap(), // SSH (scheme) with .git
        Regex::new(r"^ssh://git@[^/]+/.+$").unwrap(),    // SSH (scheme) without .git
        Regex::new(r"^git://[^/]+/.+\.git$").unwrap(),   // Git protocol with .git
        Regex::new(r"^git://[^/]+/.+$").unwrap(),        // Git protocol without .git
        Regex::new(r"^(file://)?/.+$").unwrap(),         // File path (absolute)
    ];

    if !patterns.iter().any(|p| p.is_match(url)) {
        return Err(JinError::Config(format!(
            "Invalid remote URL format: {}\n\
            \n\
            Supported formats:\n  \
            HTTPS: https://github.com/org/repo.git\n  \
            SSH:   git@github.com:org/repo.git\n  \
            Git:   git://server.local/repo.git\n  \
            File:  /absolute/path or file:///absolute/path",
            url
        )));
    }
    Ok(())
}

/// Tests connectivity to the remote repository
///
/// Attempts to connect in Fetch direction (read-only) and list remote refs
/// to verify the repository is accessible.
fn test_connectivity(repo: &git2::Repository, remote_name: &str) -> Result<()> {
    let mut remote = repo.find_remote(remote_name)?;

    // Try to connect in Fetch direction (read-only)
    match remote.connect(Direction::Fetch) {
        Ok(_) => {
            // Connection successful, disconnect to cleanup
            remote.disconnect()?;
            Ok(())
        }
        Err(e) => {
            // Map error codes to user-friendly messages
            let msg = match e.code() {
                ErrorCode::Auth => {
                    "Cannot access remote repository\n\
                    Possible causes:\n  \
                    - Missing SSH key or credentials\n  \
                    - Repository requires authentication\n\n\
                    Check your SSH keys: ssh -T git@github.com"
                }
                ErrorCode::NotFound => {
                    "Repository not found or not accessible\n\
                    Possible causes:\n  \
                    - Repository does not exist\n  \
                    - Incorrect URL\n  \
                    - Missing access permissions"
                }
                _ => {
                    // Generic error for network issues, certificate problems, etc.
                    "Cannot access remote repository\n\
                    Possible causes:\n  \
                    - Network connectivity issues\n  \
                    - Firewall blocking access\n  \
                    - Certificate validation failure"
                }
            };
            Err(JinError::Config(msg.into()))
        }
    }
}

/// Lists available modes and scopes from the remote repository
///
/// Connects to the remote, lists all refs, and parses Jin layer refs
/// to show available configurations.
fn list_remote_configs(repo: &git2::Repository) -> Result<()> {
    let mut remote = repo.find_remote("origin")?;
    remote.connect(Direction::Fetch)?;

    let refs = remote.list()?;
    let mut modes = HashSet::new();
    let mut scopes = HashSet::new();
    let mut projects = HashSet::new();

    // Parse refs to extract layer names
    let mode_pattern = Regex::new(r"refs/jin/layers/mode/([^/]+)").unwrap();
    let scope_pattern = Regex::new(r"refs/jin/layers/scope/([^/]+)").unwrap();
    let project_pattern = Regex::new(r"refs/jin/layers/project/([^/]+)").unwrap();

    for head in refs {
        let name = head.name();

        if let Some(captures) = mode_pattern.captures(name) {
            modes.insert(captures[1].to_string());
        }
        if let Some(captures) = scope_pattern.captures(name) {
            scopes.insert(captures[1].to_string());
        }
        if let Some(captures) = project_pattern.captures(name) {
            projects.insert(captures[1].to_string());
        }
    }

    remote.disconnect()?;

    // Print available configurations if any found
    if !modes.is_empty() || !scopes.is_empty() || !projects.is_empty() {
        println!("Available configurations:");

        if !modes.is_empty() {
            let mut mode_list: Vec<_> = modes.into_iter().collect();
            mode_list.sort();
            println!("  Modes: {}", mode_list.join(", "));
        }

        if !scopes.is_empty() {
            let mut scope_list: Vec<_> = scopes.into_iter().collect();
            scope_list.sort();
            println!("  Scopes: {}", scope_list.join(", "));
        }

        if !projects.is_empty() {
            let mut project_list: Vec<_> = projects.into_iter().collect();
            project_list.sort();
            println!("  Projects: {}", project_list.join(", "));
        }

        println!();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_https_url() {
        assert!(validate_git_url("https://github.com/org/repo.git").is_ok());
        assert!(validate_git_url("https://github.com/org/repo").is_ok());
    }

    #[test]
    fn test_validate_ssh_colon_url() {
        assert!(validate_git_url("git@github.com:org/repo.git").is_ok());
        assert!(validate_git_url("git@github.com:org/repo").is_ok());
    }

    #[test]
    fn test_validate_ssh_scheme_url() {
        assert!(validate_git_url("ssh://git@github.com/org/repo.git").is_ok());
        assert!(validate_git_url("ssh://git@github.com/org/repo").is_ok());
    }

    #[test]
    fn test_validate_git_protocol_url() {
        assert!(validate_git_url("git://server.local/repo.git").is_ok());
        assert!(validate_git_url("git://server.local/repo").is_ok());
    }

    #[test]
    fn test_validate_file_url() {
        assert!(validate_git_url("file:///absolute/path/to/repo").is_ok());
        assert!(validate_git_url("/absolute/path/to/repo").is_ok());
    }

    #[test]
    fn test_validate_invalid_url() {
        assert!(validate_git_url("").is_err());
        assert!(validate_git_url("invalid-url").is_err());
        assert!(validate_git_url("ftp://unsupported.com/repo").is_err());
        assert!(validate_git_url("relative/path").is_err());
    }

    #[test]
    fn test_validate_url_error_message() {
        let result = validate_git_url("invalid");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid remote URL format"));
        assert!(err_msg.contains("Supported formats"));
    }
}
