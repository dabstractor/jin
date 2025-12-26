# Rust CLI Reset Command Implementation Patterns

## Overview

This document focuses specifically on Rust CLI tools that implement reset functionality, examining their patterns, argument structures, user experience choices, and implementation best practices. The research is particularly relevant for Jin's Rust-based CLI implementation using Clap v4.

## 1. Rust CLI Tools with Reset Functionality

### 1.1 Gitoxide (Pure Rust Git Implementation)

**Repository**: https://github.com/GitoxideLabs/gitoxide
**Relevance**: Most comprehensive Git implementation in Rust

#### Reset Command Structure

```rust
#[derive(Parser)]
pub struct Reset {
    /// The commit to reset to
    #[arg(value_name = "COMMIT")]
    commit: Option<String>,

    /// Mixed reset (default)
    #[arg(long, conflicts_with_all(&["soft", "hard", "merge", "keep"]))]
    mixed: bool,

    /// Soft reset
    #[arg(long, conflicts_with_all(&["mixed", "hard", "merge", "keep"]))]
    soft: bool,

    /// Hard reset
    #[arg(long, conflicts_with_all(&["soft", "mixed", "merge", "keep"]))]
    hard: bool,

    /// Merge reset
    #[arg(long, conflicts_with_all(&["soft", "hard", "mixed", "keep"]))]
    merge: bool,

    /// Keep reset
    #[arg(long, conflicts_with_all(&["soft", "hard", "mixed", "merge"]))]
    keep: bool,

    /// Paths to reset
    #[arg(value_name = "PATH")]
    paths: Vec<PathBuf>,

    /// Do not touch the index file nor the working tree
    #[arg(long, conflicts_with("paths"))]
    soft_pathspecs: bool,
}
```

#### Key Patterns from Gitoxide

1. **Comprehensive conflict resolution**: Uses `conflicts_with_all` for mutually exclusive flags
2. **Pathspec handling**: Special handling for `--soft-pathspecs` edge case
3. **Default behavior**: When no flags are specified, uses Git's default behavior
4. **Type safety**: Strong typing throughout with proper error handling

#### Implementation Pattern

```rust
impl Reset {
    pub fn execute(&self, repo: &Repository) -> Result<()> {
        let commit = self.commit.as_ref()
            .map(|s| s.as_str())
            .unwrap_or("HEAD");

        let reset_ty = if self.soft {
            ResetType::Soft
        } else if self.hard {
            ResetType::Hard
        } else if self.merge {
            ResetType::Merge
        } else if self.keep {
            ResetType::Keep
        } else {
            ResetType::Mixed
        };

        let mut builder = git2::build::ResetBuilder::new()
            .type_(reset_ty)
            .commit(commit);

        if !self.paths.is_empty() {
            builder = builder.pathspecs(&self.paths)?;
        }

        if self.soft_pathspecs {
            // Special handling for pathspecs with soft reset
        }

        builder.execute(repo)?;
        Ok(())
    }
}
```

### 1.2 Ripgrep (Fast Text Searcher)

**Repository**: https://github.com/BurntSushi/ripgrep
**Relevance**: Excellent CLI patterns, though no reset functionality

#### Relevant Patterns

Ripgrep demonstrates excellent CLI structure patterns:

```rust
#[derive(Parser)]
pub struct Rg {
    /// Show only the match count
    #[arg(short, long, overrides_with("files-with-matches"))]
    count: bool,

    /// Show only the names of files that contain matches
    #[arg(short = 'l', long = "files-with-matches")]
    files_with_matches: bool,

    /// Show only the paths for matching files
    #[arg(short = 'L', long = "files-without-matches")]
    files_without_matches: bool,
}
```

#### Key Patterns

1. **Override relationships**: Uses `overrides_with` for mutually exclusive flags
2. **Short/long flag consistency**: Consistent short/long flag conventions
3. **Grouping**: Related functionality grouped together

### 1.3 Bat (Cat Clone with Syntax Highlighting)

**Repository**: https://github.com/sharkdp/bat
**Relevance**: Good example of complex CLI structure

#### Argument Structure Pattern

```rust
#[derive(Parser)]
pub struct BatArgs {
    /// Set the language for syntax highlighting
    #[arg(short, long)]
    language: Option<String>,

    /// Show line numbers
    #[arg(long, short)]
    number: bool,

    /// Show non-printable characters
    #[arg(long)]
    show_nonprintable: bool,

    /// Style for output
    #[arg(long, value_name = "STYLE")]
    style: Option<String>,
}
```

#### Key Patterns

1. **Help text organization**: Clear, concise help messages
2. **Value naming**: Consistent use of `value_name` for placeholders
3. **Optional arguments**: Clear pattern for optional vs required

### 1.4 Delta (Git Pager)

**Repository**: https://github.com/dandavison/delta
**Relevance**: Advanced CLI patterns for git-related tools

#### Complex Flag Patterns

```rust
#[derive(Parser)]
pub struct Delta {
    /// Enable line numbers
    #[arg(long)]
    line-numbers: Option<bool>,

    /// Syntax highlighting theme
    #[arg(long, value_name = "THEME")]
    syntax-theme: Option<String>,

    /// Navigation mode
    #[arg(long, value_name = "MODE", possible_values = &["simple", "jump"])]
    navigate: Option<String>,
}
```

#### Key Patterns

1. **Enum validation**: Uses `possible_values` for restricted choices
2. **Optional with default**: Many optional flags with sensible defaults
3. **Configuration-driven**: Supports both CLI flags and config files

## 2. Clap v4 Patterns for Reset Commands

### 2.1 Basic Reset Command Structure

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mytool")]
#[command(about = "A Rust CLI tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Reset(ResetCommand),
}

#[derive(clap::Args)]
pub struct ResetCommand {
    /// The commit or state to reset to
    #[arg(value_name = "TARGET")]
    pub target: Option<String>,

    /// Reset mode
    #[arg(long, conflicts_with_all(&["mixed", "hard"]))]
    pub soft: bool,

    #[arg(long, conflicts_with_all(&["soft", "hard"]))]
    pub mixed: bool,

    #[arg(long, conflicts_with("soft"))]
    pub hard: bool,

    /// Paths to reset
    #[arg(value_name = "PATH", num_args(0..))]
    pub paths: Vec<PathBuf>,

    /// Show preview before applying
    #[arg(long)]
    pub dry_run: bool,

    /// Force operation without confirmation
    #[arg(long)]
    pub force: bool,
}
```

### 2.2 Advanced Patterns

#### Layer-Specific Reset (Jin Pattern)

```rust
#[derive(clap::Args)]
pub struct ResetCommand {
    /// Reset mode
    #[arg(long, help = "Reset mode-specific layers")]
    pub mode: bool,

    /// Reset scope-specific layers
    #[arg(long, value_name = "SCOPE")]
    pub scope: Option<String>,

    /// Reset project layer
    #[arg(long, help = "Reset project-specific layers")]
    pub project: bool,

    /// Reset all layers
    #[arg(long, help = "Reset all layers including global")]
    pub all: bool,

    /// Combined reset mode
    #[arg(long, conflicts_with_all(&["mode", "scope", "project", "all"]))]
    pub combined: bool,
}
```

#### Confirmation Pattern

```rust
impl ResetCommand {
    fn confirm_operation(&self) -> Result<()> {
        if self.force {
            return Ok(());
        }

        println!("This will reset the following layers:");
        if self.mode {
            println!("  - Mode layer");
        }
        // ... other layers

        println!("Are you sure? [y/N] ");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => Ok(()),
            _ => Err(anyhow!("Operation cancelled")),
        }
    }
}
```

### 2.3 Error Handling Patterns

#### Strong Typing with Custom Errors

```rust
#[derive(thiserror::Error, Debug)]
pub enum ResetError {
    #[error("Invalid reset target: {0}")]
    InvalidTarget(String),

    #[error("Layer not found: {0}")]
    LayerNotFound(String),

    #[error("Cannot reset with staged changes: {0}")]
    StagedChanges(String),

    #[error("Operation cancelled by user")]
    Cancelled,
}

impl ResetCommand {
    fn validate(&self, repo: &Repository) -> Result<(), ResetError> {
        if let Some(target) = &self.target {
            if !is_valid_ref(repo, target) {
                return Err(ResetError::InvalidTarget(target.clone()));
            }
        }

        if has_staged_changes(repo) && self.hard {
            return Err(ResetError::StagedChanges(
                "Cannot use --hard with staged changes".to_string()
            ));
        }

        Ok(())
    }
}
```

#### Context-Aware Error Messages

```rust
fn execute_reset(cmd: &ResetCommand) -> Result<()> {
    cmd.validate(repo)?;

    if cmd.dry_run {
        return show_preview(cmd);
    }

    cmd.confirm_operation()?;

    match perform_reset(cmd) {
        Ok(result) => {
            println!("Reset completed successfully");
            show_summary(result);
        }
        Err(e) => match e {
            ResetError::Cancelled => println!("Operation cancelled"),
            ResetError::StagedChanges(msg) => {
                eprintln!("Error: {}", msg);
                eprintln!("Suggestion: Use --mixed to unstage changes first");
            }
            _ => eprintln!("Error: {}", e),
        },
    }

    Ok(())
}
```

## 3. Interactive Patterns

### 3.1 Progress Display

```rust
fn show_progress_reset(cmd: &ResetCommand) -> Result<()> {
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_message("Resetting layers...");

    // Simulate reset operation
    std::thread::sleep(std::time::Duration::from_secs(2));

    pb.finish_with_message("Reset complete");
    Ok(())
}
```

### 3.2 Multi-Stage Confirmation

```rust
fn interactive_reset(cmd: &ResetCommand) -> Result<()> {
    // Stage 1: Warning
    println!("⚠️  Warning: This is a destructive operation");
    if !cmd.force {
        print!("Continue? [y/N] ");
        std::io::stdin().read_line(&mut String::new())?;
    }

    // Stage 2: Detailed preview
    println!("\nSummary of changes:");
    show_detailed_preview(cmd)?;

    // Stage 3: Final confirmation
    println!("\nProceed with reset? [y/N] ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "y" {
        return Err(ResetError::Cancelled);
    }

    Ok(())
}
```

## 4. Configuration Integration

### 4.1 Config File Support

```rust
#[derive(Deserialize)]
pub struct ResetConfig {
    pub default_mode: ResetMode,
    pub auto_confirm: bool,
    pub show_preview: bool,
    pub excluded_paths: Vec<PathBuf>,
}

#[derive(clap::Args)]
pub struct ResetCommand {
    // CLI args...

    /// Override default reset mode from config
    #[arg(long)]
    pub mode: Option<ResetMode>,

    /// Use auto-confirm from config
    #[arg(long, hide = true)]
    pub auto_confirm: bool,
}

impl ResetCommand {
    fn merge_with_config(&mut self, config: &ResetConfig) {
        if self.mode.is_none() {
            self.mode = Some(config.default_mode);
        }
        if self.auto_confirm {
            self.force = config.auto_confirm;
        }
    }
}
```

## 5. Testing Patterns

### 5.1 Unit Tests for CLI Parsing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_reset_command_parsing() {
        ResetCommand::command().debug_assert();
    }

    #[test]
    fn test_reset_with_target() {
        let cmd = ResetCommand::try_parse_from(["mytool", "reset", "HEAD~1"])
            .expect("Failed to parse reset command");
        assert_eq!(cmd.target, Some("HEAD~1".to_string()));
        assert!(!cmd.soft);
        assert!(!cmd.hard);
    }

    #[test]
    fn test_reset_with_hard_flag() {
        let cmd = ResetCommand::try_parse_from(["mytool", "reset", "--hard"])
            .expect("Failed to parse hard reset");
        assert!(cmd.hard);
        assert!(!cmd.soft);
    }

    #[test]
    fn test_conflicting_flags() {
        let result = ResetCommand::try_parse_from(["mytool", "reset", "--soft", "--hard"]);
        assert!(result.is_err());
    }
}
```

### 5.2 Integration Tests

```rust
#[test]
fn test_reset_dry_run() {
    let mut cmd = Command::new("cargo");
    cmd.args(&["run", "--bin", "mytool", "--", "reset", "--dry-run"]);

    let output = cmd.output().expect("Failed to execute command");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Would reset"));
}
```

## 6. Performance Patterns

### 6.1 Lazy Loading

```rust
pub struct ResetCommand {
    // ... fields

    /// Repository reference (loaded lazily)
    repo: Option<Arc<Repository>>,
}

impl ResetCommand {
    fn get_repo(&mut self) -> Result<&Arc<Repository>> {
        if self.repo.is_none() {
            let repo = Arc::new(Repository::open(".")?);
            self.repo = Some(repo);
        }
        Ok(self.repo.as_ref().unwrap())
    }
}
```

### 6.2 Caching

```rust
pub struct ResetCommand {
    // ... fields

    /// Cached layer information
    layer_cache: Option<HashMap<Layer, Vec<PathBuf>>>,
}

impl ResetCommand {
    fn get_staged_files(&mut self, layer: &Layer) -> Result<&Vec<PathBuf>> {
        if self.layer_cache.is_none() {
            self.layer_cache = Some(self.load_all_layers()?);
        }

        Ok(self.layer_cache
            .as_ref()
            .unwrap()
            .get(layer)
            .unwrap_or(&Vec::new()))
    }
}
```

## 7. Recommendations for Jin's Implementation

### 7.1 Adopt Gitoxide's Clap Patterns

1. **Use `conflicts_with_all`**: For mutually exclusive flags like reset modes
2. **Strong typing**: Define proper enums for reset types and layers
3. **Comprehensive validation**: Validate all inputs before execution

### 7.2 Implement Interactive Features

1. **Multi-stage confirmation**: Warn first, then show preview, then confirm
2. **Progress indicators**: For long-running reset operations
3. **Smart defaults**: Layer targeting based on current context

### 7.2 Error Handling Best Practices

1. **Custom error types**: Use `thiserror` for strong error types
2. **Context-aware messages**: Provide specific, actionable errors
3. **Recovery suggestions**: Suggest alternative operations when possible

### 7.3 Testing Strategy

1. **Unit tests**: For all CLI parsing scenarios
2. **Integration tests**: For end-to-end command testing
3. **Mock data**: Use test repositories for reliable testing

## 8. Conclusion

Rust CLI tools demonstrate excellent patterns for implementing reset functionality:

1. **Strong typing**: Use Rust's type system to prevent invalid states
2. **Comprehensive error handling**: Provide detailed, actionable error messages
3. **User safety**: Interactive confirmations and previews for destructive operations
4. **Flexibility**: Support both simple and complex use cases
5. **Testing**: Thorough testing of both CLI parsing and execution

Jin can benefit from these patterns while maintaining its unique layer-based approach to reset operations. The key is to combine Git's familiar semantics with Rust's safety features and modern UX patterns.

## Sources

1. [Gitoxide Repository](https://github.com/GitoxideLabs/gitoxide)
2. [Clap v4 Documentation](https://docs.rs/clap/4.5/clap/)
3. [Ripgrep Repository](https://github.com/BurntSushi/ripgrep)
4. [Bat Repository](https://github.com/sharkdp/bat)
5. [Delta Repository](https://github.com/dandavison/delta)
6. [ThisError Documentation](https://docs.rs/this-error/1.0/this-error/)