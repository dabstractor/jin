# git2-rs Error Handling Patterns Research

## Overview

This document researches best practices for handling and wrapping git2-rs errors in the Jin project. It covers git2 error types, wrapping strategies, exit code patterns, and recommended approaches for Jin-specific Git operations.

## 1. git2::Error Types

### git2::Error Structure

The `git2::Error` struct provides rich information about Git operations:

```rust
pub struct Error {
    code: ErrorCode,
    class: ErrorClass,
    message: String,
    path: Option<String>,
}
```

### ErrorCode Enum Variants

Based on git2-rs documentation and common usage:

- **NotFound** - Requested object could not be found (repository, ref, object)
- **Exists** - Object exists when it shouldn't (e.g., creating existing ref)
- **Ambiguous** - More than one object matches the request
- **BufSize** - Output buffer too small
- **User** - User-generated error (often from callbacks)
- **BareRepo** - Operation not allowed on bare repository
- **UnmergedEntries** - Unmerged entries in index
- **Uncommitted** - Uncommitted changes present
- **Directory** - Path is not a directory
- **EmptyIndex** - Index is empty
- **Auth** - Authentication failed
- **Certificate** - Invalid certificate
- **Applied** - Patch already applied
- **Peel** - Cannot peel object to requested type
- **EOF** - Unexpected end of file
- **Invalid** - Invalid input
- **Unfinished** - Operation not finished
- **Conflict** - Merge conflict detected
- **Locked** - File is locked
- **Modified** - File was modified
- **Patch** - Patch failed to apply
- **API** - API misuse detected

### ErrorClass Categories

ErrorClass groups related errors:

- **NotFound** - Repository or reference not found
- **Exists** - Resource already exists
- **Callback** - Error from user callbacks
- **OS** - Operating system error
- **Invalid** - Invalid input or state
- **Conflict** - Merge/conflict related
- **Request** - Invalid request
- **Repository** - Repository specific errors

### Inspecting git2::Error

```rust
match err.code() {
    git2::ErrorCode::NotFound => {
        if err.class() == git2::ErrorClass::NotFound {
            // Handle repository not found
        }
    }
    git2::ErrorCode::Conflict => {
        // Handle merge conflicts
    }
    _ => {
        // Handle other errors
    }
}

// Get the error message
println!("Error: {}", err.message());

// Get associated path if available
if let Some(path) = err.path() {
    println!("Failed on path: {}", path);
}
```

## 2. Wrapping git2 Errors with thiserror

### Basic Error Wrapping

```rust
use thiserror::Error;
use git2::Error as GitError;

#[derive(Error, Debug)]
pub enum JinError {
    #[error("Git operation failed: {0}")]
    GitError(#[from] git2::Error),

    #[error("Repository not found at path: {path}")]
    RepoNotFound { path: String },

    #[error("Ref not found: {ref_name}")]
    RefNotFound { ref_name: String },

    #[error("Invalid Git state: {message}")]
    InvalidGitState { message: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("Transaction conflict detected")]
    TransactionConflict,

    #[error("Merge conflict in file: {file_path}")]
    MergeConflict { file_path: String },
}

// From implementations for automatic conversion
impl From<git2::Error> for JinError {
    fn from(err: git2::Error) -> Self {
        match err.code() {
            git2::ErrorCode::NotFound => {
                match err.class() {
                    git2::ErrorClass::NotFound => {
                        if let Some(path) = err.path() {
                            JinError::RepoNotFound { path: path.clone() }
                        } else {
                            JinError::RefNotFound { ref_name: "unknown".into() }
                        }
                    }
                    _ => JinError::GitError(err),
                }
            }
            git2::ErrorCode::Conflict => {
                if let Some(path) = err.path() {
                    JinError::MergeConflict { file_path: path.clone() }
                } else {
                    JinError::TransactionConflict
                }
            }
            git2::ErrorCode::BareRepo => {
                JinError::InvalidGitState {
                    message: "Operation not allowed on bare repository".into()
                }
            }
            _ => JinError::GitError(err),
        }
    }
}
```

### Context-Preserving Wrapping

```rust
#[derive(Error, Debug)]
pub enum JinError {
    #[error("Failed to open repository at '{path}': {source}")]
    RepoOpenFailed {
        path: String,
        #[source] source: git2::Error,
    },

    #[error("Failed to create ref '{ref_name}' in layer '{layer}': {source}")]
    RefCreateFailed {
        ref_name: String,
        layer: String,
        #[source] source: git2::Error,
    },

    #[error("Transaction failed while updating layers: {source}")]
    TransactionFailed {
        #[source] source: git2::Error,
        attempted_layers: Vec<String>,
    },
}

impl From<(git2::Error, String)> for JinError {
    fn from((err, path): (git2::Error, String)) -> Self {
        JinError::RepoOpenFailed { path, source: err }
    }
}
```

## 3. Best Practices from Projects Using git2-rs

### Gitoxide Error Handling

The gitoxide project ([github.com/GitoxideLabs/gitoxide](https://github.com/GitoxideLabs/gitoxide)) demonstrates excellent error handling:

```rust
#[derive(Error, Debug)]
pub enum GitoxideError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Invalid object id: {0}")]
    InvalidObjectId(String),
}

// Detailed error context
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Repository not found at: {path}")]
    NotFound { path: String },

    #[error("Repository already exists at: {path}")]
    AlreadyExists { path: String },

    #[error("Invalid repository state: {message}")]
    InvalidState { message: String },

    #[error("Git error: {source}")]
    Git {
        #[source]
        source: git2::Error,
        context: String,
    },
}
```

### Rust-Git Examples

The rust-git project shows how to handle specific Git scenarios:

```rust
fn open_repository(path: &Path) -> Result<git2::Repository, GitError> {
    match git2::Repository::open(path) {
        Ok(repo) => Ok(repo),
        Err(e) => match e.code() {
            git2::ErrorCode::NotFound => Err(GitError::RepositoryNotFound {
                path: path.display().to_string(),
            }),
            git2::ErrorCode::Invalid => Err(GitError::InvalidRepository {
                path: path.display().to_string(),
            }),
            _ => Err(GitError::GitError(e)),
        },
    }
}
```

## 4. Jin-Specific Git Error Scenarios

### Repository Operations

```rust
// Repository not found (P4.M2.T1.S1)
fn find_git_repo() -> Result<PathBuf, JinError> {
    let current_dir = std::env::current_dir()?;
    git2::Repository::discover(&current_dir)
        .map_err(|e| {
            if e.code() == git2::ErrorCode::NotFound {
                JinError::RepoNotFound {
                    path: current_dir.display().to_string()
                }
            } else {
                e.into()
            }
        })
}

// Invalid repository state
fn validate_repo_state(repo: &git2::Repository) -> Result<(), JinError> {
    if repo.is_bare() {
        return Err(JinError::InvalidGitState {
            message: "Cannot initialize Jin in a bare repository".into(),
        });
    }

    // Check for uncommitted changes in project Git
    let status = repo.statuses(None)?;
    if !status.is_empty() {
        return Err(JinError::UncommittedChanges {
            files: status.iter()
                .filter(|s| s.status() != git2::Status::IGNORED)
                .map(|s| s.path().unwrap_or("").to_string())
                .collect(),
        });
    }

    Ok(())
}
```

### Reference Management (P1.M2.T2.S1)

```rust
#[derive(Error, Debug)]
pub enum RefError {
    #[error("Failed to create ref '{name}' in layer '{layer}': {source}")]
    RefCreateFailed {
        name: String,
        layer: String,
        #[source] source: git2::Error,
    },

    #[error("Ref '{name}' not found in layer '{layer}'")]
    RefNotFound { name: String, layer: String },

    #[error("Ref '{name}' already exists in layer '{layer}'")]
    RefExists { name: String, layer: String },
}

impl Refs {
    pub fn create_layer_ref(
        &self,
        layer: &Layer,
        name: &str,
        target: &git2::Commit,
    ) -> Result<(), RefError> {
        let ref_name = format!("refs/jin/layers/{}/{}", layer.prefix(), name);

        match self.repo.reference(&ref_name, target.id(), false, "Jin layer ref") {
            Ok(_) => Ok(()),
            Err(e) => match e.code() {
                git2::ErrorCode::NotFound => {
                    if e.class() == git2::ErrorClass::NotFound {
                        Err(RefError::RefNotFound {
                            name: name.into(),
                            layer: layer.name().into(),
                        })
                    } else {
                        Err(RefError::RefCreateFailed {
                            name: name.into(),
                            layer: layer.name().into(),
                            source: e,
                        })
                    }
                }
                git2::ErrorCode::Exists => {
                    Err(RefError::RefExists {
                        name: name.into(),
                        layer: layer.name().into(),
                    })
                }
                _ => Err(RefError::RefCreateFailed {
                    name: name.into(),
                    layer: layer.name().into(),
                    source: e,
                }),
            },
        }
    }
}
```

### Transaction Conflicts (P1.M3.T1.S3)

```rust
#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Transaction conflict detected: {conflict}")]
    Conflict { conflict: String },

    #[error("Transaction failed to prepare: {source}")]
    PrepareFailed {
        #[source] source: git2::Error,
        staged_files: Vec<String>,
    },

    #[error("Transaction failed to commit: {source}")]
    CommitFailed {
        #[source] source: git2::Error,
        staged_files: Vec<String>,
    },

    #[error("Transaction cleanup failed: {source}")]
    CleanupFailed {
        #[source] source: git2::Error,
        orphan_refs: Vec<String>,
    },
}

impl Transaction {
    pub fn prepare(&mut self) -> Result<(), TransactionError> {
        // Check for conflicts with existing refs
        for (layer, updates) in &self.updates {
            for (name, _) in updates {
                let ref_name = format!("refs/jin/layers/{}/{}", layer.prefix(), name);
                if self.repo.find_reference(&ref_name).is_ok() {
                    return Err(TransactionError::Conflict {
                        conflict: format!(
                            "Ref {} already exists in layer {}",
                            name, layer.name()
                        ),
                    });
                }
            }
        }

        // Create staging refs
        for (layer, updates) in &self.updates {
            for (name, commit) in updates {
                let ref_name = format!("refs/jin/staging/{}/{}", layer.prefix(), name);
                if let Err(e) = self.repo.reference(&ref_name, commit.id(), true, "Jin staging") {
                    return Err(TransactionError::PrepareFailed {
                        source: e,
                        staged_files: self.get_staged_files(),
                    });
                }
            }
        }

        Ok(())
    }
}
```

### Merge Conflicts (P2.M4.T1.S1)

```rust
#[derive(Error, Debug)]
pub enum MergeError {
    #[error("3-way merge failed for file: {file_path}")]
    MergeFailed { file_path: String },

    #[error("Merge conflict in {file_path} at {conflict_path}")]
    Conflict {
        file_path: String,
        conflict_path: String,
    },

    #[error("Failed to perform text merge: {source}")]
    TextMergeFailed {
        #[source] source: git2::Error,
        file_path: String,
    },
}

impl TextMerger {
    pub fn merge_files(
        &self,
        base: &str,
        ours: &str,
        theirs: &str,
        file_path: &str,
    ) -> Result<String, MergeError> {
        // Create blobs for each version
        let base_blob = self.repo.blob(base.as_bytes())
            .map_err(|e| MergeError::TextMergeFailed {
                source: e,
                file_path: file_path.into(),
            })?;

        let ours_blob = self.repo.blob(ours.as_bytes())
            .map_err(|e| MergeError::TextMergeFailed {
                source: e,
                file_path: file_path.into(),
            })?;

        let theirs_blob = self.repo.blob(theirs.as_bytes())
            .map_err(|e| MergeError::TextMergeFailed {
                source: e,
                file_path: file_path.into(),
            })?;

        // Perform 3-way merge
        let merge_result = self.repo.merge_commits(
            &ours_blob.into(),
            &theirs_blob.into(),
            Some(base_blob.into()),
        );

        match merge_result {
            Ok(merge_index) => {
                // Check for conflicts
                let mut conflicts = Vec::new();
                for entry in merge_index.iter() {
                    let entry = entry?;
                    if entry.stage() != 0 {
                        conflicts.push(entry.path().unwrap_or("unknown").to_string());
                    }
                }

                if !conflicts.is_empty() {
                    return Err(MergeError::Conflict {
                        file_path: file_path.into(),
                        conflict_path: conflicts.first().unwrap().clone(),
                    });
                }

                // Get merged content
                let merged = merge_index.read_to_buf(None)?;
                Ok(merged.as_str()?.into())
            }
            Err(e) => Err(MergeError::TextMergeFailed {
                source: e,
                file_path: file_path.into(),
            }),
        }
    }
}
```

## 5. Exit Code Patterns for CLI

### Standard Exit Codes

```rust
use std::process;

pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    InvalidArgs = 2,
    NoRepository = 3,
    RefNotFound = 4,
    MergeConflict = 5,
    PermissionDenied = 126,
    CommandNotFound = 127,
    Timeout = 124,
}

impl From<JinError> for ExitCode {
    fn from(err: JinError) -> Self {
        match err {
            JinError::GitError(err) => match err.code() {
                git2::ErrorCode::NotFound => ExitCode::RefNotFound,
                git2::ErrorCode::Conflict => ExitCode::MergeConflict,
                git2::ErrorCode::Auth => ExitCode::PermissionDenied,
                _ => ExitCode::GeneralError,
            },
            JinError::RepoNotFound { .. } => ExitCode::NoRepository,
            JinError::RefNotFound { .. } => ExitCode::RefNotFound,
            JinError::PermissionDenied { .. } => ExitCode::PermissionDenied,
            JinError::MergeConflict { .. } => ExitCode::MergeConflict,
            _ => ExitCode::GeneralError,
        }
    }
}

// CLI command handler
fn execute_command() -> Result<(), JinError> {
    // ... command logic

    Ok(())
}

fn main() {
    match execute_command() {
        Ok(_) => process::exit(ExitCode::Success as i32),
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(ExitCode::from(err) as i32);
        }
    }
}
```

### Enhanced Exit Code Mapping

```rust
impl ExitCode {
    pub fn from_error(error: &JinError) -> i32 {
        match error {
            // Success codes
            JinError::NoError => Self::Success as i32,

            // Git-specific errors (10-19)
            JinError::GitError(err) => match err.code() {
                git2::ErrorCode::NotFound => 11,
                git2::ErrorCode::Exists => 12,
                git2::ErrorCode::Conflict => 13,
                git2::ErrorCode::Auth => 14,
                git2::ErrorCode::Certificate => 15,
                _ => 10,
            },

            // Repository errors (20-29)
            JinError::RepoNotFound { .. } => 21,
            JinError::InvalidGitState { .. } => 22,
            JinError::BareRepo { .. } => 23,

            // Reference errors (30-39)
            JinError::RefNotFound { .. } => 31,
            JinError::RefExists { .. } => 32,
            JinError::RefCreateFailed { .. } => 33,

            // Transaction errors (40-49)
            JinError::TransactionConflict => 41,
            JinError::TransactionFailed { .. } => 42,
            JinError::PrepareFailed { .. } => 43,

            // Merge errors (50-59)
            JinError::MergeConflict { .. } => 51,
            JinError::MergeFailed { .. } => 52,

            // File/IO errors (60-69)
            JinError::PermissionDenied { .. } => 61,
            JinError::FileNotFound { .. } => 62,
            JinError::IoError { .. } => 63,

            // Configuration errors (70-79)
            JinError::ConfigError { .. } => 71,
            JinError::InvalidConfig { .. } => 72,

            // General errors (1-9)
            _ => 1,
        }
    }
}
```

## 6. Recommended Error Variant Structure for Jin

Based on the research, here's the recommended error hierarchy:

```rust
// src/core/error.rs

use thiserror::Error;
use git2::Error as GitError;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum JinError {
    // ===== Core Git Operations =====
    #[error("Git operation failed: {0}")]
    Git(#[from] GitError),

    #[error("Repository not found: {path}")]
    RepoNotFound { path: String },

    #[error("Repository error: {message}")]
    RepoError { message: String },

    // ===== Reference Management =====
    #[error("Ref not found: '{name}' in layer '{layer}'")]
    RefNotFound { name: String, layer: String },

    #[error("Ref already exists: '{name}' in layer '{layer}'")]
    RefExists { name: String, layer: String },

    #[error("Ref operation failed: {operation} on '{name}' in layer '{layer}': {source}")]
    RefOpFailed {
        operation: String,
        name: String,
        layer: String,
        #[source] source: GitError,
    },

    // ===== Transactions =====
    #[error("Transaction conflict: {conflict}")]
    TransactionConflict { conflict: String },

    #[error("Transaction prepare failed: {source}")]
    TransactionPrepareFailed {
        #[source] source: GitError,
        files: Vec<String>,
    },

    #[error("Transaction commit failed: {source}")]
    TransactionCommitFailed {
        #[source] source: GitError,
        files: Vec<String>,
    },

    // ===== Merges =====
    #[error("Merge conflict in file: {file_path}")]
    MergeConflict { file_path: String },

    #[error("Merge failed for file: {file_path}")]
    MergeFailed { file_path: String },

    // ===== File Operations =====
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: String },

    #[error("IO error: {source}")]
    Io {
        #[source]
        source: std::io::Error,
        path: Option<String>,
    },

    // ===== Configuration =====
    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },

    // ===== Layer Management =====
    #[error("Invalid layer: {name}")]
    InvalidLayer { name: String },

    #[error("Layer operation failed: {operation} on '{name}': {source}")]
    LayerOpFailed {
        operation: String,
        name: String,
        #[source] source: GitError,
    },

    // ===== Workspace =====
    #[error("Workspace dirty with uncommitted changes")]
    WorkspaceDirty { files: Vec<String> },

    #[error("Workspace apply failed: {reason}")]
    WorkspaceApplyFailed { reason: String },
}

// Helper implementations
impl JinError {
    pub fn is_transient(&self) -> bool {
        matches!(self,
            JinError::Git(err) if matches!(
                err.code(),
                git2::ErrorCode::Lock |
                git2::ErrorCode::Modified
            )
        )
    }

    pub fn is_retryable(&self) -> bool {
        self.is_transient() || matches!(
            self,
            JinError::TransactionConflict { .. }
        )
    }
}

// From implementations
impl From<std::io::Error> for JinError {
    fn from(err: std::io::Error) -> Self {
        JinError::Io {
            source: err,
            path: None,
        }
    }
}

impl From<(std::io::Error, String)> for JinError {
    fn from((err, path): (std::io::Error, String)) -> Self {
        JinError::Io {
            source: err,
            path: Some(path),
        }
    }
}

// Exit code mapping
impl std::convert::From<JinError> for i32 {
    fn from(err: JinError) -> Self {
        match err {
            // Success
            _ if matches!(err, JinError::RepoError { .. } if err.message().contains("success")) => 0,

            // Not found (3)
            JinError::RepoNotFound { .. } | JinError::RefNotFound { .. } | JinError::FileNotFound { .. } => 3,

            // Conflicts (4)
            JinError::TransactionConflict { .. } | JinError::MergeConflict { .. } => 4,

            // Permission issues (5)
            JinError::PermissionDenied { .. } => 5,

            // Invalid arguments (2)
            JinError::InvalidConfig { .. } | JinError::InvalidLayer { .. } => 2,

            // General error (1)
            _ => 1,
        }
    }
}
```

## 7. Additional Resources

### Documentation

- [git2-rs Documentation](https://docs.rs/git2/) - Official crate documentation
- [libgit2 Error Codes](https://libgit2.org/libgit2/#HEAD/group/error) - Underlying libgit2 error codes
- [thiserror Documentation](https://docs.rs/thiserror/latest/thiserror/) - Error derivation macro

### Project Examples

- [Gitoxide](https://github.com/GitoxideLabs/gitoxide) - Advanced Git tooling with excellent error handling
- [GitKraken](https://github.com/gitkraken/gitkraken-lib) - Git client implementation
- [Rugged](https://github.com/libgit2/rugged) - Ruby bindings with similar error patterns

### Recommended Reading

- [Rust Error Handling Best Practices](https://github.com/rust-lang/rfcs/blob/master/text/0239-error-patterns.md)
- [The Rust Error Handling Cookbook](https://github.com/rust-lang-nursery/error-handling)
- [Writing Exceptional Code in Rust](https://github.com/dtolnay/anyhow) - Though not directly related, patterns apply