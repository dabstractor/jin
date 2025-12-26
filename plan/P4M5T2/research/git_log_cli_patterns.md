# Git Log Command Patterns and Best Practices for CLI Tools

## 1. Git Log's Most Useful Display Formats and Options

### Basic Format Options
```bash
# Basic usage
git log

# One-line format (most useful for quick overviews)
git log --oneline

# Custom pretty formats
git log --pretty=format:"%h - %an, %ar : %s"
git log --pretty=format:"%H %an %ae %ad %s" --date=short
```

### Display Patterns and Customization

**Pretty Format Specifiers:**
- `%H` - Commit hash (full)
- `%h` - Commit hash (abbreviated)
- `%an` - Author name
- `%ae` - Author email
- `%ad` - Author date
- `%ar` - Author date, relative
- `%s` - Subject (commit message)
- `%b` - Body
- `%d` - Ref names (tags, branches)
- `%D` - Ref names without decoration

**Common Display Options:**
```bash
# Show graph with branches
git log --graph --oneline --all

# Colorized output
git log --color

# Patch (diff) for each commit
git log -p

# Statistics
git log --stat

# Limit number of commits
git log -n 5

# Date range
git log --since="2024-01-01" --until="2024-12-31"

# Follow file history
git log --follow filename

# Show parent/child relationships
git log --parents

# Show changes
git log --name-only
```

## 2. Popular Rust CLI Tools and Their Commit History Display Patterns

### ripgrep (ripgrep)
- **Repository**: [BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep)
- **Approach**: Fast recursive search tool, not primarily for git history
- **Display patterns**: Focus on search results with line numbers, context

### bat (cat clone with syntax highlighting)
- **Repository**: [sharkdp/bat](https://github.com/sharkdp/bat)
- **Approach**: Enhanced cat with syntax highlighting and pagination
- **Display patterns**: Colorized output, line numbers, git integration

### gitui (Terminal UI for Git)
- **Repository**: [extrawurst/gitui](https://github.com/extrawurst/gitui)
- **Approach**: Terminal-based UI for git operations
- **Display patterns**:
  - TUI-based commit history visualization
  - Interactive branch selection
  - Rich text rendering with colors
  - Keyboard shortcuts for navigation

### tui-rs (Terminal UI framework)
- **Repository**: [fdehau/tui-rs](https://github.com/fdehau/tui-rs)
- **Approach**: Rust library for building rich terminal user interfaces
- **Display patterns**: Widget-based rendering, event handling

### popular-git (CLI tool for git)
- **Repository**: [muchdogeship/popular-git](https://github.com/muchdogeship/popular-git)
- **Approach**: Community-driven git cli with modern UX
- **Display patterns**: Focused on fast, visual commit browsing

## 3. git2-rs Documentation for RevWalk and Commit Iteration

### git2-rs Overview
- **Documentation**: [git2-rs on docs.rs](https://docs.rs/git2/)
- **GitHub**: [rust-lang/git2-rs](https://github.com/rust-lang/git2-rs)
- **libgit2 docs**: [libgit2.org/docs](https://libgit2.org/docs/)

### RevWalk API Usage

```rust
use git2::Repository;

fn main() -> Result<(), git2::Error> {
    let repo = Repository::open("/path/to/repo")?;

    // Create a RevWalk for iterating commits
    let mut revwalk = repo.revwalk()?;

    // Push a reference to start from (e.g., HEAD)
    revwalk.push_head()?;

    // Set sorting order
    revwalk.sort(git2::Sort::TOPOLOGICAL | git2::Sort::REVERSE)?;

    // Iterate over commit IDs
    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        println!("{}: {}",
            commit.id(),
            commit.message().unwrap_or("")
        );

        // Access commit details
        println!("Author: {}", commit.author().name().unwrap_or(""));
        println!("Date: {}", commit.time());
        println!("Message: {}", commit.message().unwrap_or(""));
    }

    Ok(())
}
```

### Key RevWalk Methods
- **`revwalk()`** - Creates a new RevWalk instance from a repository
- **`push_head()`** - Starts iteration from HEAD
- **`push(ref)`** - Add a reference to start from
- **`hide(ref)`** - Exclude a reference from iteration
- **`sort(...)`** - Set sorting order (topological, time, reverse)
- **`simplify_first_parent()`** - Simplify to first parent only (like --first-parent)
- **`show()`** - Include the specified commit in output
- **`hide()`** - Exclude the specified commit from output

## 4. Best Practices for Commit History Display

### One-Line vs Detailed Display

**One-Line Format (`git log --oneline`)**
- **Best for**: Quick overviews, understanding commit flow, identifying commit ranges
- **Benefits**: Compact, scannable output, less cognitive load
- **Use cases**: Daily workflow, status checks, understanding repository structure

**Detailed Format (default or `--pretty=fuller`)**
- **Best for**: Code reviews, understanding context, auditing, bug investigation
- **Benefits**: Complete commit message, full author/committer info, file changes
- **Use cases**: Deep analysis, compliance, understanding specific changes

### Graph Views (`git log --graph`)
- **Best for**: Visualizing branch structure, understanding merge history
- **Benefits**: Clear visual representation of repository evolution
- **Use cases**: Complex workflows, branch management, understanding divergences

### Recommended Command Aliases
```bash
# Go-to command for overview
git config --global alias.lg "log --color --graph --oneline --all --decorate"

# Detailed log with colors
git config --global alias.hist "log --pretty=format:'%h %ad | %s%d [%an]' --graph --date=short --color"

# Simple oneline with date
git config --global alias.ld "log --date=short --pretty=format:'%h %ad %s'"
```

### Performance Considerations
- Use `--oneline` for large repositories when quick browsing
- Limit output with `-n` for performance
- Combine with `--no-merges` if not needed
- Use `--abbrev-commit` to reduce output size

## 5. ANSI Color Coding for Commit Logs

### Git's Color Placeholders
```bash
# Basic colors
git log --pretty=format:'%Cred%h%Creset - %Cgreen%an%Creset - %s'

# Color options
%Cred     - Red
%Cgreen   - Green
%Cblue    - Blue
%Cyellow  - Yellow
%Cmagenta - Magenta
%Ccyan    - Cyan
%Cwhite   - White
%Creset   - Reset to default
%C(...)    - Custom color specification

# Background colors
%C(red)    - Red foreground (can also set background with red background)
```

### Advanced Color Formatting
```bash
# Colorful log format
git log --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset' --abbrev-commit

# Branch/tag coloring
git log --pretty=format:'%C(yellow)%h%Creset %Cblue%ad %Cgreen%an%Creset %Cred%s' --date=short

# Full colorized output
git log --color --graph --pretty=format:'%Cred%h%Creset -%C(yellow)%d%Creset %s %Cgreen(%cr) %C(bold blue)<%an>%Creset' --abbrev-commit --no-merges
```

### ANSI Color Codes Reference
- `\033[31m` - Red
- `\033[32m` - Green
- `\033[33m` - Yellow
- `\033[34m` - Blue
- `\033[35m` - Magenta
- `\033[36m` - Cyan
- `\033[0m` - Reset
- `\033[1m` - Bold
- `\033[4m` - Underline

### Color Configuration
```bash
# Enable colors by default
git config --global color.ui true

# Specific color settings
git config --global color.commit red
git config --global color.branch blue
git config --global color.diff meta yellow
```

## Additional Resources

### Official Git Documentation
- [Git Log Documentation](https://git-scm.com/docs/git-log)
- [Git Pretty Formats](https://git-scm.com/docs/git-log#_pretty_formats)
- [Git Configuration](https://git-scm.com/docs/git-config)

### Rust Git Libraries
- [git2-rs Documentation](https://docs.rs/git2/)
- [git2-rs GitHub](https://github.com/rust-lang/git2-rs)
- [libgit2 Documentation](https://libgit2.org/)

### Example Implementations
- [gitui - Terminal-based Git client](https://github.com/extrawurst/gitui)
- [popular-git - Modern git CLI](https://github.com/muchdogeship/popular-git)
- [lazygit - Simple terminal UI for git commands](https://github.com/jesseduffield/lazygit)