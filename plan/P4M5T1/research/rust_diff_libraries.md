# Rust Diff Libraries Research

## Recommended Crate Stack

### 1. similar (Already in Dependencies)
- **URL**: https://docs.rs/similar
- **License**: MIT/Apache-2.0 dual
- **Version**: 2.6+ (already used in text.rs)
- **Features**: Myers diff algorithm, unified diff, various output formats
- **Why**: Already integrated with Jin's text merge module

### 2. console (Recommended Addition)
- **URL**: https://docs.rs/console
- **License**: MIT/Apache-2.0 dual
- **Features**: Cross-platform terminal colors and styles
- **Why**: Better color control than colored crate

### 3. serde_json (Already Available)
- **URL**: https://docs.rs/serde_json
- **Features**: JSON comparison, structured diff
- **Why**: Can compute semantic diffs for JSON configs

## Implementation Examples

### Text Diff with similar
```rust
use similar::{Algorithm, TextDiff, ChangeTag};

let diff = TextDiff::configure()
    .algorithm(Algorithm::Myers)
    .from_lines(old_content, new_content);

for change in diff.iter_changes(None) {
    match change.tag() {
        ChangeTag::Delete => print!("{}-{}", RED, change.value()),
        ChangeTag::Insert => print!("{}+{}", GREEN, change.value()),
        ChangeTag::Equal => print!(" {}", change.value()),
    }
}
```

### Colored Output with console
```rust
use console::Style;

let red = Style::new().red();
let green = Style::new().green();

println!("{}", red.apply_to("- removed line"));
println!("{}", green.apply_to("+ added line"));
```

## License Summary

| Crate | License | Commercial Use |
|-------|---------|----------------|
| similar | MIT/Apache-2.0 | ✅ Yes |
| console | MIT/Apache-2.0 | ✅ Yes |
| serde_json | MIT/Apache-2.0 | ✅ Yes |
| colored | MIT | ✅ Yes |

## Feature Considerations

### Text Diffs
- Word-level diffing (optional enhancement)
- Character-level diffing (for small changes)
- Context lines (like `git diff -U3`)
- Ignore whitespace option
- Diff statistics (insertions, deletions)

### Structured Data
- Deep object comparison for JSON/YAML/TOML
- Array diff handling
- Schema-aware diffing
- Key-level change tracking
