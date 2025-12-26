# Error Handling Patterns in Popular Rust CLI Tools

## Introduction

This research document analyzes error handling patterns in popular Rust CLI tools similar to Jin (Git-based tools and CLI utilities). The analysis focuses on how these tools structure their error enums, wrap library errors, provide context in error messages, and handle exit codes.

## Analyzed Tools

### 1. ripgrep (https://github.com/BurntSushi/ripgrep)

**Key Findings:**
- Uses `anyhow::Result<ExitCode>` for main function return type
- Implements custom error handling with broken pipe detection
- Graceful exit on broken pipe errors (exit code 0)
- Structured error types with context

**Source Files:**
- `/tmp/ripgrep/ripgrep/crates/regex/src/error.rs`
- `/tmp/ripgrep/ripgrep/crates/core/main.rs`

**Error Pattern Analysis:**
```rust
// Main function uses anyhow::Result<ExitCode>
fn main() -> ExitCode {
    match run(flags::parse()) {
        Ok(code) => code,
        Err(err) => {
            // Special handling for broken pipe errors
            for cause in err.chain() {
                if let Some(ioerr) = cause.downcast_ref::<std::io::Error>() {
                    if ioerr.kind() == std::io::ErrorKind::BrokenPipe {
                        return ExitCode::from(0);
                    }
                }
            }
            eprintln_locked!("{:#}", err);
            ExitCode::from(2)
        }
    }
}
```

**Key Patterns:**
1. **Graceful broken pipe handling**: Exits with code 0 on broken pipe
2. **Context-rich error messages**: Uses `err.chain()` to provide full context
3. **Structured exit codes**: Different codes for different error conditions

### 2. bat (https://github.com/sharkdp/bat)

**Key Findings:**
- Uses comprehensive error enum with `thiserror` crate
- Implements `From` traits for all common error types
- Custom error handler for special cases (broken pipe, YAML errors)
- Public `Result<T>` type alias

**Source Files:**
- `/tmp/bat/src/error.rs`

**Error Pattern Analysis:**
```rust
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    Fmt(#[from] ::std::fmt::Error),
    #[error(transparent)]
    SyntectError(#[from] ::syntect::Error),
    #[error(transparent)]
    SyntectLoadingError(#[from] ::syntect::LoadingError),
    #[error(transparent)]
    ParseIntError(#[from] ::std::num::ParseIntError),
    #[error(transparent)]
    GlobParsingError(#[from] ::globset::Error),
    #[error(transparent)]
    SerdeYamlError(#[from] ::serde_yaml::Error),
    #[error("unable to detect syntax for {0}")]
    UndetectedSyntax(String),
    #[error("unknown syntax: '{0}'")]
    UnknownSyntax(String),
    #[error("Unknown style '{0}'")]
    UnknownStyle(String),
    #[error("Use of bat as a pager is disallowed in order to avoid infinite recursion problems")]
    InvalidPagerValueBat,
    #[error("{0}")]
    Msg(String),
    // ... more variants with feature gates
}

pub type Result<T> = std::result::Result<T, Error>;
```

**Key Patterns:**
1. **Comprehensive error enum**: Covers all possible error cases
2. **Transparent error forwarding**: Uses `#[error(transparent)]` and `#[from]` for library errors
3. **Custom error variants**: Domain-specific errors with descriptive messages
4. **Global error handler**: `default_error_handler` for special cases
5. **Type alias**: Convenient `Result<T>` type alias

### 3. gitoxide (https://github.com/Byron/gitoxide)

**Key Findings:**
- Uses both `anyhow` and `thiserror` strategically
- Modular error handling with separate error modules per crate
- Rich error context with custom error variants
- Progress-aware error handling

**Source Files:**
- `/tmp/gitoxide/gix-blame/src/error.rs`
- `/tmp/gitoxide/gitoxide-core/Cargo.toml`
- `/tmp/gitoxide/src/porcelain/main.rs`
- `/tmp/gitoxide/src/shared.rs`

**Error Pattern Analysis:**
```rust
// Example from gix-blame error.rs
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error("No commit was given")]
    EmptyTraversal,
    #[error(transparent)]
    BlobDiffSetResource(#[from] gix_diff::blob::platform::set_resource::Error),
    #[error(transparent)]
    BlobDiffPrepare(#[from] gix_diff::blob::platform::prepare_diff::Error),
    #[error("The file to blame at '{file_path}' wasn't found in the first commit at {commit_id}")]
    FileMissing {
        file_path: BString,
        commit_id: gix_hash::ObjectId,
    },
    #[error("Couldn't find commit or tree in the object database")]
    FindObject(#[from] gix_object::find::Error),
    // ... more variants
}
```

**Key Patterns:**
1. **Modular error definitions**: Separate error modules per feature/crate
2. **Structured error data**: Rich error types with multiple fields
3. **Dual error strategy**: `thiserror` for library errors, `anyhow` for application errors
4. **Progress integration**: Error handling works with progress reporting
5. **Interrupt handling**: Proper signal handling for graceful cancellation

## Common Patterns Across Tools

### 1. Error Crate Usage
- **`thiserror`**: Standard for library-level errors with custom types
- **`anyhow`**: Standard for application-level error context
- **`miette`**: Used by some for rich diagnostics (not found in analyzed tools)
- **`color-eyre`**: Alternative to `anyhow` with better error formatting

### 2. Error Structure Patterns

#### Comprehensive Error Enums
```rust
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    ParseError(#[from] ParseErrorType),
    #[error("custom error with context: {0}")]
    CustomContext(String),
    #[error("structured error: {field}")]
    StructuredError { field: Type },
}
```

#### Type Aliases
```rust
pub type Result<T> = std::result::Result<T, Error>;
```

### 3. Exit Code Patterns

#### ripgrep Pattern
```rust
fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            // Handle broken pipe specially
            if has_broken_pipe(&err) {
                return ExitCode::from(0);
            }
            eprintln!("{}", err);
            ExitCode::from(2) // General error
        }
    }
}
```

#### Common Exit Codes
- `0`: Success
- `1`: General error/no matches found (ripgrep)
- `2`: Error conditions (bat, ripgrep)
- `130`: Interrupted (Ctrl+C)

### 4. Error Context Patterns

#### Chain-based Context
```rust
// ripgrep pattern
for cause in err.chain() {
    if let Some(ioerr) = cause.downcast_ref::<std::io::Error>() {
        // Handle specific error type
    }
}
```

#### Structured Context
```rust
// gitoxide pattern
#[error("The file to blame at '{file_path}' wasn't found in the first commit at {commit_id}")]
FileMissing {
    file_path: BString,
    commit_id: gix_hash::ObjectId,
}
```

## Recommended Structure for Jin

Based on the analysis, here's a recommended error handling structure for Jin:

### 1. Error Enum Structure
```rust
// src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    // IO errors
    #[error(transparent)]
    Io(#[from] std::io::Error),

    // Git-related errors
    #[error(transparent)]
    Git(#[from] git2::Error),

    // Clap parsing errors
    #[error(transparent)]
    Clap(#[from] clap::Error),

    // JSON/YAML parsing
    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    // Jin-specific errors
    #[error("no git repository found here or in any parent directory")]
    NotAGitRepository,

    #[error("configuration error: {0}")]
    Config(String),

    #[error("invalid command: {0}")]
    InvalidCommand(String),

    #[error("operation not supported: {0}")]
    NotSupported(String),

    #[error("timeout after {0} seconds")]
    Timeout(u64),

    #[error("{0}")]
    Message(String),
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Message(s.to_owned())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Message(s)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
```

### 2. Main Function Pattern
```rust
// src/main.rs
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            // Handle broken pipe gracefully
            if has_broken_pipe(&err) {
                return ExitCode::from(0);
            }

            // Handle specific error types
            handle_error(&err);

            // Default exit code
            ExitCode::from(1)
        }
    }
}

fn has_broken_pipe(err: &Error) -> bool {
    err.chain().any(|cause| {
        if let Some(ioerr) = cause.downcast_ref::<std::io::Error>() {
            ioerr.kind() == std::io::ErrorKind::BrokenPipe
        } else {
            false
        }
    })
}

fn handle_error(err: &Error) {
    match err {
        Error::NotAGitRepository => {
            eprintln!("Error: {}", err);
        }
        Error::Config(_) => {
            eprintln!("Configuration error: {}", err);
        }
        _ => {
            eprintln!("Error: {}", err);
        }
    }
}
```

### 3. Usage in Library Code
```rust
// src/lib.rs
pub fn do_something() -> Result<()> {
    // Use Jin's error type
    let repo = git_repository.open()?;

    // Or convert from other error types
    let config = load_config().map_err(Error::Config)?;

    Ok(())
}
```

## Key Lessons Learned

1. **Dual Error Strategy**: Use `thiserror` for library errors and `anyhow` for application-level code
2. **Comprehensive Error Coverage**: Enumerate all possible error cases explicitly
3. **Broken Pipe Handling**: Always handle broken pipe errors gracefully
4. **Rich Context**: Provide as much context as possible in error messages
5. **Structured Data**: Use structs for errors that need to carry multiple pieces of information
6. **Type Aliases**: Always provide convenient `Result<T>` type aliases
7. **Exit Code Consistency**: Use standard exit codes where possible
8. **Error Chaining**: Support error chaining for better debugging

## Additional Resources

- [Rust Error Handling Guide](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
- [thiserror Documentation](https://docs.rs/thiserror/latest/thiserror/)
- [anyhow Documentation](https://docs.rs/anyhow/latest/anyhow/)
- [Common Rust CLI Error Patterns](https://github.com/rust-cli/book)