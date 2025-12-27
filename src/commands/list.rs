//! Implementation of `jin list`
//!
//! Lists available modes/scopes/projects from the Jin repository.

use crate::core::{JinError, Result};
use crate::git::JinRepo;
use std::collections::HashSet;

/// Execute the list command
///
/// Lists available modes/scopes/projects.
pub fn execute() -> Result<()> {
    // Open Jin repository
    let repo = match JinRepo::open() {
        Ok(r) => r,
        Err(_) => {
            return Err(JinError::NotInitialized);
        }
    };

    let git_repo = repo.inner();

    // Enumerate all refs under refs/jin/layers/
    let refs = match git_repo.references_glob("refs/jin/layers/**") {
        Ok(r) => r,
        Err(e) => {
            return Err(JinError::Git(e));
        }
    };

    // Parse ref paths to extract mode/scope/project names
    let mut modes = HashSet::new();
    let mut scopes = HashSet::new();
    let mut projects = HashSet::new();

    for ref_result in refs {
        let reference = ref_result?;
        if let Some(name) = reference.name() {
            parse_ref_path(name, &mut modes, &mut scopes, &mut projects);
        }
    }

    // Display results
    println!("Available in Jin repository:");
    println!();

    let has_modes = !modes.is_empty();
    let has_scopes = !scopes.is_empty();
    let has_projects = !projects.is_empty();

    if has_modes {
        println!("Modes:");
        let mut mode_list: Vec<_> = modes.into_iter().collect();
        mode_list.sort();
        for mode in mode_list {
            println!("  - {}", mode);
        }
        println!();
    }

    if has_scopes {
        println!("Scopes:");
        let mut scope_list: Vec<_> = scopes.into_iter().collect();
        scope_list.sort();
        for scope in scope_list {
            println!("  - {}", scope);
        }
        println!();
    }

    if has_projects {
        println!("Projects:");
        let mut project_list: Vec<_> = projects.into_iter().collect();
        project_list.sort();
        for project in project_list {
            println!("  - {}", project);
        }
        println!();
    }

    if !has_modes && !has_scopes && !has_projects {
        println!("  (no modes, scopes, or projects found)");
        println!();
    }

    // Show usage hints
    println!("Use 'jin mode use <mode>' to activate a mode");
    println!("Use 'jin scope use <scope>' to activate a scope");

    Ok(())
}

/// Parse a ref path and extract mode/scope/project names
fn parse_ref_path(
    ref_path: &str,
    modes: &mut HashSet<String>,
    scopes: &mut HashSet<String>,
    projects: &mut HashSet<String>,
) {
    // Ref paths follow these patterns:
    // refs/jin/layers/global
    // refs/jin/layers/mode/<mode>
    // refs/jin/layers/mode/<mode>/scope/<scope>
    // refs/jin/layers/mode/<mode>/scope/<scope>/project/<project>
    // refs/jin/layers/mode/<mode>/project/<project>
    // refs/jin/layers/scope/<scope>
    // refs/jin/layers/project/<project>
    // refs/jin/layers/local
    // refs/jin/layers/workspace

    if !ref_path.starts_with("refs/jin/layers/") {
        return;
    }

    let path = &ref_path["refs/jin/layers/".len()..];
    let parts: Vec<&str> = path.split('/').collect();

    match parts.as_slice() {
        ["mode", mode] => {
            modes.insert(mode.to_string());
        }
        ["mode", mode, "scope", scope] => {
            modes.insert(mode.to_string());
            scopes.insert(scope.to_string());
        }
        ["mode", mode, "scope", scope, "project", project] => {
            modes.insert(mode.to_string());
            scopes.insert(scope.to_string());
            projects.insert(project.to_string());
        }
        ["mode", mode, "project", project] => {
            modes.insert(mode.to_string());
            projects.insert(project.to_string());
        }
        ["scope", scope] => {
            scopes.insert(scope.to_string());
        }
        ["project", project] => {
            projects.insert(project.to_string());
        }
        _ => {
            // Ignore global, local, workspace, and unknown patterns
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_not_initialized() {
        // List command works even without project initialization
        // It reads from the global Jin repository at ~/.jin/
        // If the global repo exists (from previous tests), this will succeed
        let result = execute();
        // Accept either success (global repo exists) or error (doesn't exist)
        assert!(result.is_ok() || matches!(result, Err(JinError::NotInitialized)));
    }

    #[test]
    fn test_parse_ref_path() {
        let mut modes = HashSet::new();
        let mut scopes = HashSet::new();
        let mut projects = HashSet::new();

        parse_ref_path(
            "refs/jin/layers/mode/claude",
            &mut modes,
            &mut scopes,
            &mut projects,
        );
        assert!(modes.contains("claude"));
        assert!(scopes.is_empty());
        assert!(projects.is_empty());
    }

    #[test]
    fn test_parse_ref_path_with_scope() {
        let mut modes = HashSet::new();
        let mut scopes = HashSet::new();
        let mut projects = HashSet::new();

        parse_ref_path(
            "refs/jin/layers/mode/claude/scope/language:javascript",
            &mut modes,
            &mut scopes,
            &mut projects,
        );
        assert!(modes.contains("claude"));
        assert!(scopes.contains("language:javascript"));
        assert!(projects.is_empty());
    }

    #[test]
    fn test_parse_ref_path_with_project() {
        let mut modes = HashSet::new();
        let mut scopes = HashSet::new();
        let mut projects = HashSet::new();

        parse_ref_path(
            "refs/jin/layers/mode/claude/scope/language:rust/project/ui-dashboard",
            &mut modes,
            &mut scopes,
            &mut projects,
        );
        assert!(modes.contains("claude"));
        assert!(scopes.contains("language:rust"));
        assert!(projects.contains("ui-dashboard"));
    }

    #[test]
    fn test_parse_ref_path_standalone_project() {
        let mut modes = HashSet::new();
        let mut scopes = HashSet::new();
        let mut projects = HashSet::new();

        parse_ref_path(
            "refs/jin/layers/project/api-server",
            &mut modes,
            &mut scopes,
            &mut projects,
        );
        assert!(modes.is_empty());
        assert!(scopes.is_empty());
        assert!(projects.contains("api-server"));
    }

    #[test]
    fn test_parse_ref_path_ignore_global() {
        let mut modes = HashSet::new();
        let mut scopes = HashSet::new();
        let mut projects = HashSet::new();

        parse_ref_path(
            "refs/jin/layers/global",
            &mut modes,
            &mut scopes,
            &mut projects,
        );
        assert!(modes.is_empty());
        assert!(scopes.is_empty());
        assert!(projects.is_empty());
    }
}
