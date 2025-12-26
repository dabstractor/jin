# thiserror Best Practices and Patterns for Error Hierarchies

## Overview

This document researches thiserror crate best practices and patterns for defining error hierarchies in Rust, with a focus on thiserror 2.0 features and patterns applicable to the Jin project.

## 1. thiserror Crate Fundamentals

### 1.1 Basic thiserror Usage

thiserror is a derive macro for creating error types with minimal boilerplate:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("something went wrong: {msg}")]
    Something { msg: String },

    #[error("invalid input: {0}")]
    InvalidInput(String),
}
```

### 1.2 Derive Macro Attributes

The `Error` derive macro supports several attributes:

| Attribute | Purpose | Usage |
|-----------|---------|-------|
| `#[error("...")]` | Display message | On enum or struct |
| `#[from]` | Auto From impl | On variant field (single field only) |
| `#[source]` | Explicit source field | On variant field |
| `#[error(transparent)]` | Forward source's Display | On variant with single field |

### 1.3 thiserror 2.0 Specific Features

Version 2.0 (specified in Jin's Cargo.toml) includes:
- Improved error chain formatting
- Better integration with `std::error::Error`
- Enhanced support for generic error types
- Stable attribute syntax

## 2. Error Hierarchy Patterns

### 2.1 Flat Enum (Recommended for Jin)

A single comprehensive enum covering all error cases:

```rust
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum JinError {
    // Library errors
    #[error(transparent)]
    Git(#[from] git2::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    // Domain-specific errors
    #[error("Repository not found at: {path}")]
    RepoNotFound { path: String },

    #[error("Merge conflict in file: {file_path}")]
    MergeConflict { file_path: String },
}
```

**Advantages**:
- Single type to handle
- Easy to add new variants
- Clear error categorization
- No nesting confusion

### 2.2 Structured Error Variants

Use struct variants when you need multiple pieces of context:

```rust
#[error("Ref operation failed: {operation} on '{name}' in layer '{layer}'")]
RefOpFailed {
    operation: String,
    name: String,
    layer: String,
    #[source]
    source: git2::Error,
}
```

**Best practices**:
- Use descriptive field names
- Include `#[source]` for underlying errors
- Format all relevant context in error message

### 2.3 When to Use Nested Error Types

Generally avoid nested error enums. However, separate error modules make sense for:

```rust
// In src/git/error.rs - git-specific errors
#[derive(Error, Debug)]
pub enum GitError {
    #[error("Repository not found: {0}")]
    RepoNotFound(String),
    // ...
}

// In src/core/error.rs - main error type
#[derive(Error, Debug)]
pub enum JinError {
    #[error("Git error: {0}")]
    Git(#[from] git::error::GitError),
    // ...
}
```

**For Jin**: Use a single flat enum in `src/core/error.rs` - simpler and sufficient.

## 3. Error Context Propagation

### 3.1 Using #[from] for Automatic Conversion

```rust
#[derive(Error, Debug)]
pub enum JinError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Automatic conversion works
fn open_repo() -> Result<Repository, JinError> {
    let repo = git2::Repository::open(".")?;  // git2::Error -> JinError
    Ok(repo)
}
```

**Rules**:
- Use on single-field variants only
- Field type must implement `Error`
- Automatically generates `From<SourceError> for JinError`

### 3.2 Using #[source] for Manual Context

When you need custom From implementations or additional fields:

```rust
#[derive(Error, Debug)]
pub enum JinError {
    #[error("Failed to open repository at '{path}': {source}")]
    RepoOpenFailed {
        path: String,
        #[source]
        source: git2::Error,
    },
}

// Manual implementation
impl From<(git2::Error, String)> for JinError {
    fn from((err, path): (git2::Error, String)) -> Self {
        JinError::RepoOpenFailed { path, source: err }
    }
}
```

### 3.3 Using #[error(transparent)]

Forward to the source error's Display implementation:

```rust
#[derive(Error, Debug)]
pub enum JinError {
    #[error(transparent)]
    Git(#[from] git2::Error),
}
```

This is equivalent to `#[error("{0}")]` but more explicit about forwarding.

## 4. Error Display Patterns

### 4.1 Field Formatting

```rust
#[error("File '{path}' not found in layer '{layer}'")]
FileNotFound { path: String, layer: String }

// Display: "File '.claude/config.json' not found in layer 'project/ui'"
```

### 4.2 Unnamed Field Formatting

```rust
#[error("Parse error: {0}")]
ParseError(String)

// Or use Debug for complex types
#[error("Invalid config: {0:?}")]
InvalidConfig(Config),
```

### 4.3 Conditional Display

```rust
#[derive(Error, Debug)]
pub enum JinError {
    #[error("Transaction failed{failed_layers}")]
    TransactionFailed {
        failed_layers: String,
    },
}

// In From impl
impl TransactionError {
    fn display_layers(&self) -> String {
        if self.layers.is_empty() {
            String::new()
        } else {
            format!(": {}", self.layers.join(", "))
        }
    }
}
```

## 5. Best Practices from the Rust Ecosystem

### 5.1 Common Patterns

From successful Rust projects:

1. **Use #[non_exhaustive]** on public error enums
2. **Group related errors** with comments
3. **Provide Result type alias** for convenience
4. **Implement helper methods** for common queries
5. **Document error conditions** with doc comments

### 5.2 Error Variant Naming

Use descriptive names that indicate the problem:

| Good | Bad |
|------|------|
| `RepoNotFound` | `RepoError` |
| `MergeConflict` | `MergeError` |
| `PermissionDenied` | `FsError` |

### 5.3 Message Formatting

Good error messages:
- Say what went wrong
- Include relevant context
- Suggest fixes when possible

```rust
// Good
#[error("Repository not found at '{path}'. Run 'jin init' to initialize.")]
RepoNotFound { path: String }

// Less helpful
#[error("Repository error")]
RepoError
```

## 6. Jin-Specific Error Categories

Based on PRD analysis, Jin needs error variants for:

### 6.1 Git Operations

```rust
#[error("Git operation failed: {0}")]
Git(#[from] git2::Error),

#[error("Repository not found: {path}")]
RepoNotFound { path: String },

#[error("Ref not found: '{name}' in layer '{layer}'")]
RefNotFound { name: String, layer: String },
```

### 6.2 Transaction System

```rust
#[error("Transaction conflict: {conflict}")]
TransactionConflict { conflict: String },

#[error("Transaction prepare failed: {source}")]
PrepareFailed {
    #[source]
    source: Box<JinError>,
    files: Vec<String>,
},
```

### 6.3 Merge Operations

```rust
#[error("Merge conflict in file: {file_path}")]
MergeConflict { file_path: String },

#[error("Merge failed: {file_path} - {reason}")]
MergeFailed { file_path: String, reason: String },

#[error("Unsupported file format: {format}")]
UnsupportedFormat { format: String },
```

### 6.4 File I/O

```rust
#[error("Symlinks not supported: {path}")]
SymlinkNotSupported { path: String },

#[error("Binary files not supported: {path}")]
BinaryFileNotSupported { path: String },

#[error("Submodules not tracked by Jin: {path}")]
SubmoduleNotSupported { path: String },
```

### 6.5 Configuration

```rust
#[error("Configuration error: {message}")]
ConfigError { message: String },

#[error("Invalid configuration: {message}")]
InvalidConfig { message: String },

#[error("Validation failed: {message}")]
ValidationError { message: String },
```

### 6.6 Serialization

```rust
#[error("JSON parse error: {0}")]
JsonParse(#[from] serde_json::Error),

#[error("YAML parse error: {0}")]
YamlParse(#[from] serde_yaml_ng::Error),

#[error("TOML parse error: {0}")]
TomlParse(#[from] toml::Error),

#[error("INI parse error: {0}")]
IniParse(#[from] configparser::Error),
```

## 7. Exit Code Mapping

### 7.1 Standard Exit Codes

```rust
impl From<JinError> for i32 {
    fn from(err: JinError) -> Self {
        match &err {
            JinError::RepoNotFound { .. } => 3,
            JinError::MergeConflict { .. } => 4,
            JinError::PermissionDenied { .. } => 5,
            _ => 1,
        }
    }
}
```

### 7.2 Helper Method Pattern

```rust
impl JinError {
    pub fn exit_code(&self) -> i32 {
        (*self).into()
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self,
            JinError::Git(err) if matches!(
                err.code(),
                git2::ErrorCode::Locked | git2::ErrorCode::Modified
            )
        )
    }
}
```

## 8. Type Alias Pattern

Always provide a Result type alias:

```rust
pub type Result<T> = std::result::Result<T, JinError>;
```

This allows:
```rust
fn do_something() -> Result<()> {
    // Instead of std::result::Result<(), JinError>
}
```

## 9. Testing Error Display

### 9.1 Unit Test Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = JinError::RepoNotFound {
            path: "/tmp/test".into()
        };
        assert_eq!(err.to_string(), "Repository not found: /tmp/test");
    }

    #[test]
    fn test_exit_codes() {
        assert_eq!(JinError::RepoNotFound { path: "".into() }.exit_code(), 3);
    }
}
```

### 9.2 Property-Based Testing

Consider using proptest for error property testing:
- All errors produce valid display strings
- Exit codes are in valid range
- Error chaining preserves information

## 10. Documentation Best Practices

### 10.1 Module Documentation

```rust
//! Core error types for Jin.
//!
//! This module defines the [`JinError`] enum, which represents all possible
//! errors that can occur in Jin operations.
//!
//! # Example
//!
//! ```
//! use jin_glm::core::error::{JinError, Result};
//!
//! fn open_repo() -> Result<Repository> {
//!     // ...
//!     # Ok(unimplemented!())
//! }
//! ```
```

### 10.2 Variant Documentation

```rust
/// Error occurred when opening a Git repository.
///
/// This typically means the directory is not a Git repository or
/// does not exist.
#[error("Repository not found: {path}")]
RepoNotFound { path: String },
```

## 11. Common Gotchas

### 11.1 Cannot Use Both #[from] and #[source]

```rust
// ❌ WRONG
#[error("Error: {source}")]
Error {
    #[from]  // Conflict!
    #[source]
    source: git2::Error
}

// ✅ CORRECT - use #[from] alone
#[error(transparent)]
Git(#[from] git2::Error)
```

### 11.2 Must Implement Display for Custom Formatting

```rust
// If you need custom formatting beyond what #[error(...)] provides:
impl std::fmt::Display for JinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JinError::Custom { details } => write!(f, "Custom: {}", details),
            _ => write!(f, "{}", self),  // Use derived default
        }
    }
}
```

### 11.3 Generic Error Types

thiserror works with generics:

```rust
#[derive(Error, Debug)]
pub enum JinError<T = String> {
    #[error("Custom error: {0}")]
    Custom(T),
}
```

## 12. Additional Resources

- [thiserror Documentation](https://docs.rs/thiserror/2.0/thiserror/)
- [Rust Error Handling Guide](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
- [The Rust Error Handling Book](https://rust-cli.github.io/book/tutorial/errors.html)
- [thiserror GitHub Repository](https://github.com/dtolnay/thiserror)
