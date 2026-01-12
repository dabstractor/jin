# Best Practices for Diff Commands in CLI Tools

## Research Summary

This document outlines comprehensive best practices for implementing diff commands in CLI tools, based on research of git diff patterns, diff output formats, color standards, and modern UX practices.

## Key Documentation Sources

### 1. Git Diff Implementation
- **[Git - git-diff Documentation](https://git-scm.com/docs/git-diff)** - Official documentation covering all diff use cases and options
- **[Git - diff-config Documentation](https://git-scm.com/docs/diff-config)** - Configuration options for diff behavior
- **[Git Diff | Atlassian Tutorial](https://www.atlassian.com/git/tutorials/saving-changes/git-diff)** - Practical workflow patterns

### 2. Diff Output Formats
- **[Comparing and Merging Files - diff Output Formats](https://www.math.utah.edu/docs/info/diff_3.html)** - Comprehensive guide to unified, context, and normal formats
- **[diff(1) - Linux man page](https://man7.org/linux/man-pages/man1/diff.1.html)** - Technical reference for diff command
- **[Unified Diff Format Wikipedia](https://en.wikipedia.org/wiki/Diff)** - Historical context and format specifications

### 3. Colored Diff Output Standards
- **[How to Use diff --color: 7 Practical Examples](https://medium.com/@redswitches/how-to-use-diff-color-7-practical-examples-46de448e46a6)** - Comprehensive color customization guide
- **[Color output in console](https://wiki.archlinux.org/title/Color_output_in_console)** - Standard practices for terminal colors
- **[Colors and formatting in the output](https://bettercli.org/design/using-colors-in-cli/)** - Best practices for CLI color usage

### 4. CLI Diff UX Patterns
- **[Command Line Interface Guidelines](https://clig.dev/)** - Open-source guide for CLI design
- **[CLI UX Best Practices](https://evilmartians.com/chronicles/cli-ux-best-practices-3-patterns-for-improving-progress-displays)** - Progress display patterns
- **[UX Patterns for CLI Tools](https://lucasfcosta.com/blog/ux-patterns-cli-tools)** - Specifically for CLI applications
- **[Improved Diff UX in LazyGit](https://github.com/jesseduffield/lazygit/issues/2659)** - Real-world diff UX improvements

### 5. Multi-Version Comparison
- **[Git Range-Diff Documentation](https://blog.gitbutler.com/interdiff-review-with-git-range-diff)** - Comparing different versions
- **[Comparing Commits - Git Book](https://shafiul.github.io/gitbook/3_comparing_commits_-_git_diff.html)** - Git-specific multi-version patterns
- **[Academic Research on Diff Classification](https://lup.lub.lu.se/student-papers/record/9150755/file/9150756.pdf)** - Modern diff representation research

## Git Diff Implementation and Patterns

### Core Implementation Patterns

1. **Incremental Updates**
   - Compare working tree ↔ index
   - Compare index ↔ HEAD
   - Compare HEAD ↔ specific commit
   - Compare two arbitrary revisions

2. **Diff Object Model**
   - File-level vs. line-level granularity
   - Content, mode, and metadata changes
   - Binary vs. text file differentiation
   - Unicode normalization considerations

3. **Performance Optimizations**
   - Git-specific delta compression
   - Hash-based content caching
   - Lazy loading of large files
   - Parallel processing for multiple files

### Key Git Diff Options
```bash
# Basic comparison
git diff [options] [commit] [path...]

# Format options
git --diff-color-words=color
git --color-moved[=detect- copies|dimmed-plain]
git --color-words[=<regex>]

# Context control
git -U<num>         # Set unified diff context
git --no-color      # Disable colors
git --stat         # Show diffstat instead
git --shortstat     # Show abbreviated diffstat
```

## Common Diff Output Formats

### 1. Unified Diff Format (`-u` or `--unified`)
The most widely used format in modern development:

```
--- a/file.txt	2024-01-01 00:00:00.000000000 +0000
+++ b/file.txt	2024-01-01 00:01:00.000000000 +0000
@@ -1,5 +1,5 @@
 Line 1
-Line 2
+Modified line 2
 Line 3
 Line 4
-Removed line 5
+Added new line
```

**Characteristics:**
- Shows context lines (default 3, configurable)
- Prefixed with `@ @@` markers
- Lines start with `+` (added), `-` (removed), ` ` (unchanged)
- Most readable format for human review

### 2. Context Diff Format (`-c` or `-C`)
More verbose than unified format:

```
*** a/file.txt	2024-01-01 00:00:00.000000000 +0000
--- b/file.txt	2024-01-01 00:01:00.000000000 +0000
***************
*** 1,5 ****
  Line 1
- Line 2
  Line 3
  Line 4
- Removed line 5
--- 1,5 ----
  Line 1
+ Modified line 2
  Line 3
  Line 4
+ Added new line
```

### 3. Side-by-Side Format (`-y` or `--side-by-side`)
Best for manual comparison:

```
                                |  Line 1
     Line 1                       |
     Line 2                        Modified line 2
                                |  Line 3
     Line 3                       |
     Line 4                       |  Line 4
     Line 5                        Removed line 5
     Added new line                |
```

**Characteristics:**
- Two columns for easy visual comparison
- `|` separator
- Can specify column width with `--width`
- Ideal for reviewing changes side-by-side

### 4. Brief Format (`--brief` or `-q`)
Minimal output:

```
Files a/file.txt and b/file.txt differ
```

## Colored Diff Output Standards

### Standard Color Convention

| Line Type | Color | ANSI Code | Purpose |
|-----------|-------|----------|---------|
| Added | Green | `\e[32m` | Positive changes |
| Removed | Red | `\e[31m` | Deleted content |
| Context | Default | | Unchanged lines |
| Header | Cyan/Blue | `\e[36m` | File headers |
| Hunk Header | Yellow | `\e[33m` | Chunk markers |
| Word Diff | Multiple | | Character-level changes |

### Best Practices for Color

1. **Auto-detection**
   ```bash
   --color=auto  # Detect terminal capabilities
   --color=never # Never colorize
   --color=always # Always colorize
   ```

2. **Palette Customization**
   ```bash
   diff --color --palette=':ad=[ANSI code]:de=[ANSI code]'
   ```

3. **Accessibility Considerations**
   - Provide colorless alternatives (like `-u`)
   - Ensure contrast ratios meet WCAG standards
   - Use bold/highlight instead of color when possible

4. **Pipe Awareness**
   - Detect when output is piped and disable colors
   - Use `--color=auto` which handles this automatically

## Best Practices for Multi-Layer/Version Diffing

### 1. Three-Way Merging Pattern
```
ancestor
├── version A
└── version B
```

**Implementation:**
```bash
# Compare two divergent versions
git diff <ancestor> <version-a> <version-b>

# Range diff (for multiple commits)
git range-diff <old-base>..<new-base> <old-branch>..<new-branch>
```

### 2. Layered Diffing Strategies

**Filesystem Layer:**
- Directory structure changes
- File additions/removals
- Permission changes

**Content Layer:**
- Line-by-line differences
- Binary file changes
- Metadata modifications

**Semantic Layer:**
- Function-level changes
- Import/dependency changes
- Performance impact assessment

### 3. Performance Optimization for Multi-Version Diffs

- **Lazy Loading**: Only load changed files
- **Delta Compression**: Send only differences
- **Parallel Processing**: Process multiple files concurrently
- **Caching**: Cache recent diff results

## Diff Command UX Best Practices

### 1. Command Design Principles

#### Command Structure
```bash
# Good: git diff [options] [<commit>] [--] [<path>...]
# Bad: git show-differences -files=file1,file2 -view=sidebyside

diff [--options] file1 file2
git diff [<commit>] [<path>...]
```

#### Option Design
- **Long Options**: Prefer `--color` over `-c` for user-friendly options
- **Short Options**: Keep them consistent with established tools
- **Mutually Exclusive**: Prevent conflicting options
- **Defaults**: Sensible defaults that match common use cases

### 2. Progressive Disclosure Pattern

**Gradual Complexity:**
```bash
# Basic view
git diff

# With stats
git diff --stat

# With context
git diff --unified=5

# With color
git diff --color
```

### 3. Context Management

- **Configurable Context**: Allow users to control context lines
- **Auto-scaling**: Adjust context based on terminal size
- **Context Navigation**: Jump between hunks with `n`/`N`

### 4. Interactive Diff Features

- **Hunk Navigation**: Jump to next/previous hunk
- **Selection**: Select specific hunks for applying
- **Pagination**: Handle large diffs gracefully with `less` integration
- **Search**: Search within diff output

### 5. Error Handling and Feedback

- **Clear Error Messages**: "No files to compare" vs "file not found"
- **Progress Indicators**: For multi-file diff operations
- **Validation**: Validate input before processing
- **Recovery**: Graceful handling of corrupt files

## Common Pitfalls to Avoid

### 1. Performance Issues

**Problem:** Slow diffs on large repositories
```bash
# Bad: Process entire repository
git diff HEAD~100..HEAD

# Better: Process specific files
git diff HEAD~100..HEAD -- path/to/file
```

### 2. Unicode/Encoding Problems

**Problem:** Incorrect handling of different encodings
```bash
# Good: Normalize input
diff --ignore-matching-lines='\r' file1 file2

# Better: Specify encoding
diff -b --ignore-space-change file1 file2
```

### 3. Output Consistency

**Problem:** Inconsistent output between runs
```bash
# Ensure deterministic output
git diff --no-index --unified file1 file2

# Sort file lists consistently
ls -1 | sort | while read file; do diff ...; done
```

### 4. Memory Usage

**Problem:** Loading entire files into memory
```bash
# Stream-based processing for large files
while IFS= read -r line; do
  process_line "$line"
done < <(cat largefile.txt)
```

### 5. Pipe Handling

**Problem:** Colors break when piping to other commands
```bash
# Good: Use color auto-detection
git diff --color=auto | less -R

# Better: Provide colorless option
git diff --color=never | grep "pattern"
```

## Advanced Patterns

### 1. Delta Encoding
```bash
git diff --binary  # For binary files
git diff --patch   # Generate patches for redistribution
```

### 2. Statistical Diffing
```bash
git diff --numstat  # Show added/removed lines per file
git diff --shortstat  # Abbreviated diffstat
```

### 3. Content-Aware Diffing
```bash
git diff --word-diff=color  # Word-level changes
git diff --word-diff-regex=.  # Custom word boundaries
```

### 4. Git-Specific Patterns
```bash
git diff --cached  # Show staged changes
git diff --name-only  # Only show changed filenames
git diff --ignore-space-change  # Ignore whitespace changes
```

## Conclusion

Implementing a great diff command requires balancing functionality, performance, and usability. Key takeaways:

1. **Follow established conventions** (unified format, color schemes)
2. **Provide flexible output options** (context control, format selection)
3. **Implement smart defaults** (auto-color, sensible context)
4. **Consider accessibility** (colorless alternatives, keyboard navigation)
5. **Optimize for performance** (stream processing, lazy loading)
6. **Maintain consistency** (predictable output, stable API)

The best diff tools make complex comparisons intuitive while providing power users with the precision they need.