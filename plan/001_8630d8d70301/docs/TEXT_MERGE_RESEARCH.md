# Text Merge Research - P2.M4

## Rust Libraries Comparison

### Recommended: `diffy` (v0.4)

**Repository**: https://github.com/bmwill/diffy
**License**: MIT OR Apache-2.0
**Crates.io**: https://crates.io/crates/diffy

**Why chosen**:
- Simple API with `merge(original, a, b)` function
- Returns `Ok(String)` for clean merge, `Err(String)` with conflict markers for conflicts
- Uses Myers' diff algorithm (same as Git)
- Supports both UTF-8 text and binary data
- Actively maintained

**API**:
```rust
use diffy::merge;

// Clean merge case
let result = merge(original, a, b).unwrap();

// Conflict case - Err contains content with markers
match merge(original, a, b) {
    Ok(merged) => println!("Merged: {}", merged),
    Err(conflict) => println!("Has conflicts:\n{}", conflict),
}
```

### Alternatives Considered

#### `threeway_merge`
- Uses Git's xdiff algorithms (libgit2)
- More configuration options (algorithm choice, merge styles)
- LGPL-2.1+ license (may require consideration)
- Returns struct with conflict count

#### `similar`
- High-performance diff library
- No built-in 3-way merge (just diffing)
- Would need custom merge logic on top

#### `imara-diff`
- Ultra-high performance (10-100% faster than similar)
- No built-in merge, just optimized diffing
- Best for very large files

## 3-Way Merge Algorithm

### Concept

Three inputs:
1. **Base** (common ancestor)
2. **Ours** (version A's changes from base)
3. **Theirs** (version B's changes from base)

### Rules

```
if (base == ours && base == theirs): output = base          # No changes
if (base != ours && base == theirs): output = ours          # Only A changed
if (base == ours && base != theirs): output = theirs        # Only B changed
if (base != ours && base != theirs && ours == theirs): output = ours  # Same change
if (base != ours && base != theirs && ours != theirs): CONFLICT  # Different changes
```

### Conflict Marker Format (Git Standard)

```
<<<<<<< ours
our changes here
=======
their changes here
>>>>>>> theirs
```

### Diff3 Format (Extended)

```
<<<<<<< ours
our version
||||||| base
original version
=======
their version
>>>>>>> theirs
```

## Documentation URLs

### Primary Documentation
- diffy crate: https://docs.rs/diffy/latest/diffy/
- diffy merge(): https://docs.rs/diffy/latest/diffy/fn.merge.html
- diffy create_patch(): https://docs.rs/diffy/latest/diffy/fn.create_patch.html

### Algorithm Background
- Merging with diff3: https://blog.jcoglan.com/2017/05/08/merging-with-diff3/
- Magic of 3-Way Merge: https://blog.git-init.com/the-magic-of-3-way-merge/
- Myers Algorithm Paper: https://edit-distance.github.io/myers-1986/

### Git References
- Git merge: https://git-scm.com/docs/git-merge
- Merge strategies: https://git-scm.com/docs/merge-strategies
- Conflict presentation: https://git-scm.com/docs/git-merge#_how_conflicts_are_presented

## Implementation Notes

### Handling diffy's Return Type

```rust
// CRITICAL: diffy::merge returns Result<String, String>
// Ok(String) = clean merge
// Err(String) = content WITH conflict markers (not an error!)

match diffy::merge(base, ours, theirs) {
    Ok(clean) => TextMergeResult::Clean(clean),
    Err(with_markers) => {
        let count = with_markers.matches("<<<<<<<").count();
        TextMergeResult::Conflict {
            content: with_markers,
            conflict_count: count,
        }
    }
}
```

### Counting Conflicts

Simple approach - count opening markers:
```rust
fn count_conflicts(content: &str) -> usize {
    content.matches("<<<<<<<").count()
}
```

### Parsing Conflict Regions

Line-by-line parsing:
1. Find `<<<<<<<` line (start)
2. Find `=======` line (separator)
3. Find `>>>>>>>` line (end)
4. Extract ours (between start and sep) and theirs (between sep and end)
5. Track line numbers for ConflictRegion

### Edge Cases to Handle

1. **Empty files**: All combinations of empty base/ours/theirs
2. **Trailing newlines**: `"text\n"` vs `"text"` are different
3. **Large files**: Should handle 100KB+ efficiently
4. **Binary content**: Detect and reject (or pass through)
5. **Malformed markers**: Partial/incomplete conflict markers
6. **Multiple conflicts**: 2+ conflict regions in same file

## Performance Considerations

- diffy uses Myers algorithm: O((M+N)D) where D is edit distance
- For similar files (low D): very fast
- For completely different files: can be O(MN) worst case
- 100KB files should complete in milliseconds

## Test Scenarios

1. Identical ours/theirs (clean - both made same change)
2. Only ours changed (clean - take ours)
3. Only theirs changed (clean - take theirs)
4. Non-overlapping changes (clean - merge both)
5. Overlapping changes (conflict)
6. Multiple conflict regions
7. Empty base with changes
8. All empty
9. Large file performance
10. Special characters and Unicode
