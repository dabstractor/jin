# `similar` Crate Usage Guide

## Overview

The `similar` crate (v2.6) is a Rust library for diffing sequences of data using various algorithms. It is already included in Jin's dependencies.

## Key Types

### `TextDiff`

The main entry point for text diffing operations.

```rust
use similar::{Algorithm, TextDiff};

// Create a diff from two strings
let diff = TextDiff::from_lines(old, new);
```

### `Change`

Represents a single change operation in the diff.

```rust
pub enum Change<T> {
    Insert(T),
    Delete(T),
    Equal(T),
}
```

## API Examples

### Basic Line Diff

```rust
use similar::TextDiff;

let old = "line 1\nline 2\nline 3";
let new = "line 1\nline 2 modified\nline 3";

let diff = TextDiff::from_lines(old, new);

// Iterate over changes
for change in diff.iter_changes(None) {
    match change.tag() {
        similar::ChangeTag::Insert => print!("+"),
        similar::ChangeTag::Delete => print!("-"),
        similar::ChangeTag::Equal => print!(" "),
    }
    println!("{}", change.value());
}
```

### Getting Unified Diff Format

```rust
use similar::{Algorithm, TextDiff};

let diff = TextDiff::configure()
    .algorithm(Algorithm::Myers)
    .from_lines(old, new);

// Generate unified diff
let unified = diff.unified_diff();
```

### Unified Diff with Headers

```rust
use similar::TextDiff;

let diff = TextDiff::from_lines(old, new);

let unified = diff.unified_diff()
    .header(&["--- old.txt", "+++ new.txt"])
    .to_string();
```

## Algorithms

### Myers (Default)

The classic O(ND) diff algorithm. Good for most cases.

```rust
use similar::{Algorithm, TextDiff};

let diff = TextDiff::configure()
    .algorithm(Algorithm::Myers)
    .from_lines(old, new);
```

### Patience

Uses longest common subsequence for more readable diffs on reordered content.

```rust
use similar::{Algorithm, TextDiff};

let diff = TextDiff::configure()
    .algorithm(Algorithm::Patience)
    .from_lines(old, new);
```

## Inline Diff

For character-level or word-level diffs:

```rust
use similar::{ChangeTag, TextDiff};

let old = "hello world";
let new = "hello rust";

let diff = TextDiff::from_chars(old, new);

for change in diff.iter_changes(None) {
    match change.tag() {
        ChangeTag::Insert => print!("+"),
        ChangeTag::Delete => print!("-"),
        ChangeTag::Equal => print!(" "),
    }
    print!("{}", change.value());
}
```

## Capture Edits

To get the actual edit operations:

```rust
use similar::{TextDiff, ChangeTag};

let diff = TextDiff::from_lines(old, new);

// Get edit script
let edits: Vec<_> = diff
    .iter_changes(None)
    .collect();

// Process edits
for edit in &edits {
    // ... process each change
}
```

## Performance Considerations

1. **Large files**: For very large files, consider processing in chunks
2. **Algorithm choice**: Myers is O(ND) - fastest for similar files
3. **Memory**: The entire diff is computed upfront, then iterated

## Resources

- crates.io: https://crates.io/crates/similar
- docs.rs: https://docs.rs/similar/
- GitHub: https://github.com/mcarton/rust-similar
