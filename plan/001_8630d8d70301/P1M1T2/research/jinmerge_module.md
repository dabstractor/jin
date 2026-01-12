# .jinmerge File Format Module Research Report

## 1. File Locations

### Primary Module File
- **Path**: `/home/dustin/projects/jin/src/merge/jinmerge.rs`
- **Status**: Complete and fully implemented
- **Size**: 910 lines (including comprehensive tests)

### Module Integration
- **Module Declaration**: `/home/dustin/projects/jin/src/merge/mod.rs` (line 24: `pub mod jinmerge;`)
- **Public Exports**: `/home/dustin/projects/jin/src/merge/mod.rs` (lines 44-45):
  ```rust
  pub use jinmerge::{JinMergeConflict, JinMergeRegion, JINMERGE_HEADER};
  ```

## 2. Key Data Structures

### JinMergeRegion Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JinMergeRegion {
    /// Full ref path for first layer (e.g., "mode/claude/scope:javascript/")
    pub layer1_ref: String,
    /// Content from first layer
    pub layer1_content: String,
    /// Full ref path for second layer (e.g., "mode/claude/project/ui-dashboard/")
    pub layer2_ref: String,
    /// Content from second layer
    pub layer2_content: String,
    /// Starting line number (1-indexed, for user display)
    pub start_line: usize,
    /// Ending line number (1-indexed, inclusive)
    pub end_line: usize,
}
```

### JinMergeConflict Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JinMergeConflict {
    /// Original file path (without .jinmerge extension)
    pub file_path: PathBuf,
    /// All conflict regions in the file
    pub conflicts: Vec<JinMergeRegion>,
}
```

### Constants
```rust
/// Header comment added to all .jinmerge files
pub const JINMERGE_HEADER: &str = "# Jin merge conflict. Resolve and run 'jin resolve <file>'";

/// Marker constants (Git-compatible - exactly 7 characters)
pub const MARKER_START: &str = "<<<<<<< ";
pub const MARKER_SEP: &str = "=======";
pub const MARKER_END: &str = ">>>>>>> ";
```

## 3. Function Signatures and Types

### Constructor Functions
```rust
/// Create from text merge result and layer ref paths
pub fn from_text_merge(
    file_path: PathBuf,
    layer1_ref: String,
    layer1_content: String,
    layer2_ref: String,
    layer2_content: String,
) -> Self
```

### File I/O Functions
```rust
/// Write to .jinmerge file with layer-aware markers
pub fn write_to_file(&self, merge_path: &Path) -> Result<()>

/// Parse existing .jinmerge file
pub fn parse_from_file(merge_path: &Path) -> Result<Self>
```

### Utility Functions
```rust
/// Count total conflict regions
pub fn conflict_count(&self) -> usize

/// Check if file appears to be a .jinmerge file
pub fn is_jinmerge_file(path: &Path) -> bool

/// Get the .jinmerge file path for an original file
pub fn merge_path_for_file(original: &Path) -> PathBuf
```

## 4. Integration Points

### Error Handling
- Uses `JinError` from `/home/dustin/projects/jin/src/core/error.rs`
- Format errors use `JinError::Parse { format: "jinmerge", message: String }`
- File I/O errors use `JinError::Io`

### Module Dependencies
```rust
use crate::core::{JinError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
```

### Export Path
The module is publicly accessible as:
```rust
use jin::merge::jinmerge::{JinMergeConflict, JinMergeRegion, JINMERGE_HEADER};
```

## 5. File Format Specification

### Structure
```
# Jin merge conflict. Resolve and run 'jin resolve <file>'
<<<<<<< mode/claude/scope:javascript/
{"target": "es6", "modules": true}
=======
{"target": "es2020", "modules": false, "strict": true}
>>>>>>> mode/claude/project/ui-dashboard/
```

### Key Features
1. **Header**: Always starts with `JINMERGE_HEADER` comment
2. **Markers**: Git-compatible exact 7-character markers
3. **Layer References**: Full ref paths as labels (not generic "ours/theirs")
4. **Content**: Preserves original content with newlines
5. **Multiple Conflicts**: Supports multiple conflict regions in one file

## 6. Test Patterns and Locations

### Test File Location
- **File**: `/home/dustin/projects/jin/src/merge/jinmerge.rs` (lines 415-909)
- **Structure**: `#[cfg(test)] mod tests` block

### Test Categories
- Constructor Tests
- Format Generation Tests
- File I/O Tests
- Parsing Tests
- Round-trip Tests
- Helper Method Tests
- Marker Format Tests
- Line Number Tests
- Empty Content Tests
- Integration Tests

### Test Patterns Used
1. **Testing File I/O**: Uses `TempDir` for temporary file creation
2. **Atomic Write Verification**: Checks temp file cleanup and final file existence
3. **Error Testing**: Validates error conditions with malformed files
4. **Round-trip Testing**: Ensures write-parse cycles maintain data integrity
5. **Edge Case Testing**: Tests empty content, missing markers, etc.

## 7. Key Implementation Details

### Atomic Write Pattern
```rust
let temp_path = merge_path.with_extension("jinmerge.tmp");
std::fs::write(&temp_path, content).map_err(JinError::Io)?;
std::fs::rename(&temp_path, merge_path).map_err(JinError::Io)?;
```

### Layer Reference Format
- Full ref paths from Jin's layer system
- Examples: `mode/claude/`, `mode/claude/scope:javascript/`, `global/`
- Extracted from layer refs during merge operations
