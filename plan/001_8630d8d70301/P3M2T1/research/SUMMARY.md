# Research Summary: P3.M2.T1 - Document JIN_DIR in README

## Overview

This document summarizes the research conducted for creating a PRP to document the `JIN_DIR` environment variable in the Jin README.

## JIN_DIR Implementation

### Definition
`JIN_DIR` is an environment variable that specifies the root directory where Jin stores its internal Git repository and configuration data.

### Default Location
- **Linux/macOS**: `~/.jin/`
- **Windows**: `%USERPROFILE%\.jin\`

### How It Works

The implementation checks for `JIN_DIR` in two key locations:

**1. Configuration Loading** (`src/core/config.rs:75-85`):
```rust
pub fn default_path() -> Result<PathBuf> {
    // Check JIN_DIR environment variable first for test isolation
    if let Ok(jin_dir) = std::env::var("JIN_DIR") {
        return Ok(PathBuf::from(jin_dir).join("config.toml"));
    }

    // Fall back to default ~/.jin location
    dirs::home_dir()
        .map(|h| h.join(".jin").join("config.toml"))
        .ok_or_else(|| JinError::Config("Cannot determine home directory".into()))
}
```

**2. Git Repository Path** (`src/git/repo.rs:152-161`):
```rust
pub fn default_path() -> Result<PathBuf> {
    // Check for JIN_DIR environment variable first (for testing)
    if let Ok(jin_dir) = std::env::var("JIN_DIR") {
        return Ok(PathBuf::from(jin_dir));
    }

    dirs::home_dir()
        .map(|h| h.join(".jin"))
        .ok_or_else(|| JinError::Config("Cannot determine home directory".into()))
}
```

### Priority Order
1. `JIN_DIR` environment variable (if set)
2. Default `~/.jin/` location

### Directory Contents
```
$JIN_DIR/
├── config.toml           # Global configuration
├── refs/                 # Git references
│   └── jin/             # Jin-specific refs (layers, modes, scopes)
├── objects/             # Git objects (blobs, trees, commits)
└── jin/                 # Jin metadata (if present)
```

## README Structure Analysis

### Current README Sections
1. Title & Tagline
2. What is Jin?
3. Quick Start
4. Installation
5. Command Overview
6. Documentation
7. Why Jin?
8. How Jin Works
9. **[INSERT CONFIGURATION DIRECTORY HERE]**
10. Example Use Cases
11. Features
12. Contributing & License
13. Support

### Style Guidelines
- **Headings**: H2 `##` for main sections, H3 `###` for subsections
- **Code blocks**: Fenced with `\`\`\`bash`
- **Bold text**: `**text**` for emphasis
- **Lists**: Dash `-` for bullet points
- **Tone**: Professional yet approachable, concise
- **Code examples**: Copy-pasteable, show expected output

### Placement Recommendation
The JIN_DIR documentation should be placed after the "How Jin Works" section (around line 174), as this is where the README currently mentions the `~/.jin/` directory.

## Best Practices from CLI Tools

### Git (GIT_DIR)
- Shows environment variable syntax
- Explains what it controls
- Documents default location
- Provides override examples

### Cargo (CARGO_HOME)
- Platform-specific default locations
- Clear override mechanism
- Use cases for customization
- Shell persistence examples

### rustup (RUSTUP_HOME)
- Purpose statement
- Default locations per platform
- Setup instructions
- Migration examples

### Docker (DOCKER_CONFIG)
- Explains what directory contains
- Default location
- Environment variable override
- Command-line alternative

## Common Patterns Identified

1. **Clear Purpose Statement**: Explain what the variable controls in 1-2 sentences
2. **Platform-Specific Defaults**: Show default for Linux/macOS/Windows
3. **Override Syntax**: Show exactly how to set the variable
4. **Use Cases**: Provide 3-5 practical scenarios
5. **Persistence**: Document shell configuration methods
6. **Code Examples**: Copy-pasteable examples with comments

## Use Cases to Document

Based on research and common CLI tool patterns:

1. **Different Drive/Partition**: Store Jin data on a different drive
2. **Network Storage**: Share Jin config across multiple machines
3. **Isolated Testing**: Separate Jin environments for testing
4. **CI/CD Environments**: Configure for CI pipelines

## Shell Persistence

Users need to know how to make JIN_DIR persistent:

- **Bash**: Add to `~/.bashrc` or `~/.bash_profile`
- **Zsh**: Add to `~/.zshrc`
- **Fish**: Add to `~/.config/fish/config.fish`
- **PowerShell**: Add to `$PROFILE`

## Key Gotchas

1. **Timing**: JIN_DIR must be set BEFORE running Jin commands
2. **Process-Global**: Once set, it affects the entire process
3. **Test Isolation**: Tests extensively use JIN_DIR for isolation
4. **Bare Repository**: JIN_DIR points to a bare Git repo (no working directory)

## Sources

### Codebase Analysis
- `src/core/config.rs` - Configuration loading with JIN_DIR
- `src/git/repo.rs` - Git repository path resolution
- `src/test_utils.rs` - Test isolation pattern
- `README.md` - Current documentation structure

### External Documentation
- [Git Configuration Documentation](https://git-scm.com/docs/git-config)
- [Cargo Environment Variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
- [rustup Environment Variables](https://rust-lang.github.io/rustup/environment-variables.html)
- [Docker CLI Reference](https://docs.docker.com/reference/cli/docker/)

## PRP Quality Assessment

The resulting PRP meets all quality gates:

- [x] Passes "No Prior Knowledge" test
- [x] All references are specific and accessible
- [x] Implementation tasks include exact guidance
- [x] Validation commands are specific to this project

**Confidence Score**: 10/10 for one-pass implementation success

**Rationale**:
- Pure documentation task with no code changes
- Clear placement location in README.md
- Comprehensive research and examples
- Reversible and easy to iterate
