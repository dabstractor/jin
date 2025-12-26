# Git Diff UX Patterns Research

## Key UX Patterns to Follow

### 1. Unified Diff Format Display
- Use standard unified diff format with `@@ -line,count +line,count @@` hunk headers
- Show 3 lines of context by default (configurable with `-U<n>`)
- Color code additions (+green), deletions (-red), and context (neutral)
- Display file headers with `diff --git a/file b/file` format

### 2. Common Flag Patterns
- `git diff` - Show unstaged changes (working tree vs index)
- `git diff --staged` / `git diff --cached` - Show staged changes (index vs HEAD)
- `git diff HEAD` - Show all changes (working tree vs HEAD)
- `git diff branch1 branch2` - Compare branches or commits
- `git diff -- path/to/file` - Show specific file changes

### 3. Comparison Target Patterns
- **Working tree vs index**: `git diff` (unstaged)
- **Index vs HEAD**: `git diff --staged` (staged)
- **Working tree vs HEAD**: `git diff HEAD`
- **Commit vs commit**: `git diff commit1 commit2`
- **Branch vs branch**: `git diff main feature`

### 4. Output Formatting Conventions
- Display line numbers in hunk headers for quick navigation
- Show file modification modes (100644, 100755, etc.)
- Use compact headers for script-friendliness
- Provide summary statistics with `--stat` option
- Support both human-readable (`--color`) and machine-readable (`--numstat`) formats

### 5. Error Handling Patterns
- Return exit code 0 when no differences found (success)
- Return exit code 1 when differences found (for scripting)
- Handle binary files gracefully (skip or show with `--binary`)
- Show helpful error messages for invalid paths or non-existent references

## Specific URLs

1. **Git diff documentation**: https://git-scm.com/docs/git-diff
2. **Unified diff format specification**: https://git-scm.com/docs/diff-format
3. **CLI best practices**: https://clig.dev/ (covers general CLI diff patterns)
4. **Git diff output examples**: https://git-scm.com/book/en/v2/Git-Basics-Viewing-the-Commit-History#_viewing_the_difference

## Implementation Recommendations for Jin Diff

### 1. Command Structure
```bash
jin diff [layer1] [layer2]
jin diff --staged
jin diff --stat
```

### 2. Layer Comparison Patterns
- If no layers specified: Compare workspace to merged result
- `--staged`: Compare staged files to their layer's HEAD
- Support layer identifiers: `jin diff mode/claude project/myproject`
- Handle file paths: `jin diff -- path/to/file`

### 3. Output Format
- Follow unified diff format exactly
- Include layer identifiers in headers
- Show hunk headers with line number ranges
- Color code additions/deletions like git diff

### 4. Exit Codes
- Return 0 if no differences
- Return 1 if differences found (script-friendly)
- Return 2 for errors
