# Conflict Marker Conventions

## Standard Git Conflict Markers

Git uses standardized conflict markers that are recognized by most merge tools and editors.

## Format

```
<<<<<<< marker_name
Content from version A
=======
Content from version B
>>>>>>> marker_name
```

### Marker Components

1. **`<<<<<<<`**: Start of conflict marker (7 `<` symbols)
2. **Marker Name**: Identifier for the source branch/version
3. **`=======`**: Separator (7 `=` symbols)
4. **Conflict content**: The conflicting versions
5. **`>>>>>>>`**: End of conflict marker (7 `>` symbols)
6. **Marker Name**: Identifier for the target branch/version

## Git's Default Marker Names

| Context | Left Marker | Right Marker |
|---------|-------------|--------------|
| `git merge` | `HEAD` | branch name being merged |
| `git rebase` | commit hash of original change | `HEAD` |
| `git cherry-pick` | commit hash | `HEAD` |
| `git stash apply` | stash reference | `HEAD` |

## Example: Standard Merge Conflict

```
<<<<<<< HEAD
def calculate_total(items):
    return sum(item.price for item in items)
=======
def calculate_total(items):
    total = 0
    for item in items:
        total += item.price * item.quantity
    return total
>>>>>>> feature/new-calculation
```

## Jin-Specific Conflict Markers (PRD §11.3)

According to the PRD, Jin should use layer information in conflict markers:

```
Conflict in file: .claude/config.json
Layer 1: mode/claude/scope/language:javascript/
Layer 2: mode/claude/project/ui-dashboard/

<<<<<<< mode/claude/scope/language:javascript/
{ "mcpServers": ["server-a"] }
=======
{ "mcpServers": ["server-b"] }
>>>>>>> mode/claude/project/ui-dashboard/
```

## Implementation Guidelines

### Marker Name Format

For Jin, use the layer path as the marker name:

```rust
// Examples of valid marker names
"mode/claude"
"scope/language:javascript"
"mode/claude/scope/language:javascript/project/ui-dashboard"
"project/ui-dashboard"
```

### Constants Definition

```rust
/// Start marker for conflicts (7 `<` symbols)
pub const CONFLICT_START: &str = "<<<<<<<";

/// Separator marker for conflicts (7 `=` symbols)
pub const CONFLICT_SEPARATOR: &str = "=======";

/// End marker for conflicts (7 `>` symbols)
pub const CONFLICT_END: &str = ">>>>>>>";
```

### Generating Markers

```rust
fn format_conflict_start(marker_name: &str) -> String {
    format!("{} {}", CONFLICT_START, marker_name)
}

fn format_conflict_end(marker_name: &str) -> String {
    format!("{} {}", CONFLICT_END, marker_name)
}

fn format_conflict_block(
    left_marker: &str,
    left_content: &str,
    right_marker: &str,
    right_content: &str,
) -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n",
        format_conflict_start(left_marker),
        left_content,
        CONFLICT_SEPARATOR,
        right_content,
        format_conflict_end(right_marker),
    )
}
```

## Detection

To detect if content contains conflict markers:

```rust
fn has_conflict_markers(content: &str) -> bool {
    content.lines().any(|line| {
        line.starts_with(CONFLICT_START) ||
        line.starts_with(CONFLICT_SEPARATOR) ||
        line.starts_with(CONFLICT_END)
    })
}
```

## Edge Cases

### Nested Markers

If conflict markers appear in the source content (e.g., code documentation about conflicts), escape or detect them:

```rust
fn contains_literal_markers(content: &str) -> bool {
    // Check for actual Git-style conflict markers
    content.contains("<<<<<<<") && content.contains("=======")
}
```

### Binary Content

Conflict markers should never be added to binary content. Always validate text content before attempting merge.

## Related PRD Sections

- **PRD §11.1**: Text files use 3-way diff
- **PRD §11.3**: Conflict resolution with `.jinmerge` files and layer information

## References

- Git conflict marker specification: https://git-scm.com/docs/merge-strategies#_conflict_markers
- Git merge-file documentation: https://git-scm.com/docs/git-merge-file
