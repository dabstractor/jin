# 3-Way Merge Algorithm Research

## Overview

3-way merge is a version control algorithm that merges two modified versions of a file by using their common ancestor as a reference point. This is the standard merge approach used by Git and other version control systems.

## Algorithm Steps

### Inputs
- **Base (O)**: The common ancestor version
- **Left (A)**: One modified version (e.g., "ours" or "current")
- **Right (B)**: Another modified version (e.g., "theirs" or "incoming")

### Process

1. **Diff Analysis**
   - Compute diff between Base and Left: `diff(O, A)`
   - Compute diff between Base and Right: `diff(O, B)`

2. **Change Classification**
   For each line/region:
   - Changed only in Left → accept Left's change
   - Changed only in Right → accept Right's change
   - Changed identically in both → accept the change
   - Changed differently in both → **CONFLICT**

3. **Merging**
   - Apply non-conflicting changes automatically
   - Generate conflict markers for overlapping changes

### Conflict Detection

Conflicts occur when:
- Both Left and Right modify the same region differently
- One side modifies a region that the other side deletes
- Both sides make incompatible changes to the same area

## Pseudocode

```
function three_way_merge(base, left, right):
    left_changes = diff(base, left)
    right_changes = diff(base, right)

    merged = copy(base)
    conflicts = []

    for each line in base:
        left_line = corresponding_line(left, base_line_index)
        right_line = corresponding_line(right, base_line_index)

        if left_line == base_line and right_line == base_line:
            # No changes - keep base line
            continue
        else if left_line == right_line:
            # Both changed identically - accept
            merged[line_index] = left_line
        else if left_line != base_line and right_line == base_line:
            # Only left changed - accept left
            merged[line_index] = left_line
        else if left_line == base_line and right_line != base_line:
            # Only right changed - accept right
            merged[line_index] = right_line
        else:
            # Both changed differently - CONFLICT
            add_conflict(conflicts, line_index, left_line, right_line)

    return merged, conflicts
```

## Example

### Input
```
Base (O):
  line 1
  line 2
  line 3

Left (A):
  line 1 modified
  line 2
  line 3

Right (B):
  line 1
  line 2 modified
  line 3
```

### Output (No Conflict)
```
line 1 modified    # from Left
line 2 modified    # from Right
line 3             # from Base
```

### Example with Conflict

```
Base (O):
  line 1
  line 2

Left (A):
  line 1 changed by A
  line 2

Right (B):
  line 1 changed by B
  line 2
```

### Conflict Output
```
<<<<<<< left
line 1 changed by A
=======
line 1 changed by B
>>>>>>> right
line 2
```

## Time Complexity

- **Myers Diff Algorithm**: O(ND) where:
  - N = length of the sequences
  - D = edit distance (number of differences)

- **Space Complexity**: O(D) for storing edit script

## Key Considerations

1. **Granularity**: Merge can be line-based or character-based
2. **Whitespace**: Handling of trailing whitespace and line endings
3. **Encoding**: All inputs must be valid UTF-8
4. **Binary Detection**: Binary files should be rejected, not merged

## References

- Myers, E. W. (1986). "An O(ND) Difference Algorithm and Its Variations"
- Git merge-file documentation: https://git-scm.com/docs/git-merge-file
- Git merge strategies: https://git-scm.com/docs/merge-strategies
