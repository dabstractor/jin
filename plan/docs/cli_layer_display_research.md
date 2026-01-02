# CLI Layer Display Research: Best Practices and Patterns

This research document covers best practices and patterns for displaying hierarchical layer information in CLI tools, with a focus on applications for a `jin layers` command.

## Table of Contents
1. [Git Visualization Patterns](#git-visualization-patterns)
2. [Rust CLI Table Formatting Crates](#rust-cli-table-formatting-crates)
3. [Docker/Kubernetes Resource Listing Patterns](#dockerkubernetes-resource-listing-patterns)
4. [Terminal Color and Unicode Usage](#terminal-color-and-unicode-usage)
5. [File Count and Storage Path Display Patterns](#file-count-and-storage-path-display-patterns)
6. [Summary Statistics Output Patterns](#summary-statistics-output-patterns)
7. [Recommended Implementation for `jin layers`](#recommended-implementation-for-jin-layers)

---

## 1. Git Visualization Patterns

### Basic Git Log Graph Commands
```bash
# Standard branch visualization
git log --all --decorate --oneline --graph
git log --graph --oneline --decorate --all
```

### ASCII/Unicode Tree Visualization
Git uses Unicode characters to create tree-like visualizations:
- `*` for commits
- `|` for vertical lines
- `/` for forward branches
- `\` for backward branches
- `─` for horizontal lines
- `├` for junction points
- `└` for end points

### Color Schemes in Git
- Green for current branch
- Yellow for local branches
- Red for remote branches
- Cyan for tags
- No color by default (respectful of NO_COLOR)

### Resources
- [StackOverflow: Pretty Git branch graphs](https://stackoverflow.com/questions/1057564/pretty-git-branch-graphs)
- [How to Show Git Branch Graph in Terminal](https://cuda-chen.github.io/git/2021/08/14/git-pretty-branch-graph.html)
- [Display a graph of all Git commits](https://jeffkreeftmeijer.com/git-graph/)

---

## 2. Rust CLI Table Formatting Crates

### Popular Libraries Comparison

| Crate | Features | Compile Time | Best For |
|-------|----------|--------------|----------|
| **comfy-table** | UTF-8, dynamic content wrapping, round corners | Moderate | Rich table styling |
| **cli-table** | Low compile time, small binary size, CSV support | Fast | Minimal binary size |
| **tabled** | Rust structs pretty print, active development | Fast | Struct-based tables |
| **term-table** | Column width control via tuples | Fast | Precise width management |
| **cli-table-derive** | Derive macro for quick table creation | Fast | Rapid development |

### Example Usage Patterns

#### comfy-table
```rust
use comfy_table::{Table, Cell};

let mut table = Table::new();
table.set_header(vec!["Name", "Status", "Size"]);
table.add_row(vec![
    Cell::new("layer1").set_alignment(CellAlignment::Left),
    Cell::new("active").fg(Color::Green),
    Cell::new("1.2MB").fg(Color::Blue)
]);
```

#### tabled
```rust
use tabled::{Table, Tabled};

#[derive(Tabled)]
struct Layer {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Size")]
    size: String,
}

let table = Table::new(vec![Layer { name: "layer1".into(), status: "active".into(), size: "1.2MB".into() }]);
```

### Resources
- [comfy-table GitHub](https://github.com/Nukesor/comfy-table)
- [tabled GitHub](https://github.com/zhiburt/tabled)
- [cli-table on crates.io](https://crates.io/crates/cli-table)

---

## 3. Docker/Kubernetes Resource Listing Patterns

### Custom Output Patterns

#### Docker Custom Format
```bash
docker ps --format "table {{.ID}}\t{{.Names}}\t{{.Status}}\t{{.Ports}}"
```

#### Kubectl Custom Columns
```bash
kubectl get pods -o custom-columns=NAME:.metadata.name,STATUS:.status.phase,NODE:.spec.nodeName
```

### Table Output Patterns
- **Fixed-width columns** for consistent alignment
- **Truncation** with ellipsis (...) for long values
- **Right-alignment** for numeric data
- **Left-alignment** for text data
- **Conditional columns** (show only when data exists)

### Color Schemes
- **Green**: Running/Ready/Healthy
- **Yellow/Orange**: Pending/Warning
- **Red**: Failed/Error/Terminated
- **Gray**: Unknown/Not Ready

### Status Indicators
- Simple icons: ✓, ✗, ⚠
- Colored dots: 🔴, 🟡, 🟢
- Text labels: active, inactive, pending

### Resources
- [Kubectl Output Formatting Guide](https://www.baeldung.com/ops/kubectl-output-format)
- [Docker Format Options](https://blog.csdn.net/m0_67394002/article/details/123731574)
- [Custom Column Mastery](https://pratikpanda.hashnode.dev/unlocking-kubernetes-efficiency-mastering-custom-column-formatting-with-kubectl)

---

## 4. Terminal Color and Unicode Usage

### Color Best Practices
1. **Never use color as the only indicator** - ensure accessibility for color-blind users
2. **Respect NO_COLOR environment variable** - check for `NO_COLOR=1`
3. **Auto-detect terminal capabilities** - check if stdout is a TTY
4. **Provide monochrome fallbacks** - when colors aren't available

### Rust Color Crates
| Crate | Features | Best For |
|-------|----------|----------|
| **colored** | Simple API, easy to use | Quick color addition |
| **termcolor** | Cross-platform, rich features | Complex terminal apps |
| **ansi_term** | ANSI control, styles | Fine-grained control |
| **anstream** | Modern, simplified | New projects |

### Unicode Tree Characters
```
├── Branch with child
├── Another branch
└── Final branch
```

### Color Scheme Examples
```rust
use colored::*;

// Status colors
let active = "active".green();
let warning = "warning".yellow();
let error = "error".red();
let info = "info".blue();
```

### Resources
- [BetterCLI: Using Colors](https://bettercli.org/design/using-colors-in-cli/)
- [anstream Blog](https://epage.github.io/blog/2023/03/anstream-simplifying-terminal-styling/)
- [Terminal Colors Guide](https://marvinh.dev/blog/terminal-colors/)

---

## 5. File Count and Storage Path Display Patterns

### Human-Readable Size Formatting
```bash
# Standard Unix tools
du -h /path          # Human-readable sizes
du -sh /path         # Summary with human-readable
ls -lh              # List with human-readable sizes
```

### Size Formatting Functions
- Bytes: `1,234 B`
- KB: `1.2 KB`
- MB: `1.5 MB`
- GB: `2.1 GB`
- TB: `1.8 TB`

### Path Display Patterns
- **Truncate with ellipsis**: `/very/long/path/to/.../file`
- **Relative paths**: `./layers/active`
- **Breadcrumbs**: `layers/active/config`
- **Icons**: 📁 for directories, 📄 for files

### File Count Patterns
- Total: `123 files`
- By type: `45 images, 78 documents`
- Tree view: `├── 3 files`

### Resources
- [Linux File Size Commands](https://bitlaunch.io/blog/linux-file-size-bytes/)
- [Directory Size Linux](https://gcore.com/learning/how-to-get-directory-size-linux)
- [List Files with Paths](https://stackoverflow.com/questions/43904684/list-files-with-path-and-file-size-only-in-command-line)

---

## 6. Summary Statistics Output Patterns

### Common Statistics to Display
- Total count: `123 layers`
- Active count: `45 active`
- Total size: `456 MB`
- Average size: `3.7 MB`
- Size distribution: `12 tiny, 67 small, 44 medium`

### Table Format Examples
```
┌─────────────┬───────┬─────────┐
│ Property    │ Count │ Size   │
├─────────────┼───────┼─────────┤
│ Total       │ 123   │ 456 MB │
│ Active      │ 45    │ 234 MB │
│ Inactive    │ 78    │ 222 MB │
│ Average     │ -     │ 3.7 MB │
└─────────────┴───────┴─────────┘
```

### Progress Indicators
- Simple: `[████████████] 100%`
- Detailed: `[██████░░░░] 60% (12/20 layers)`
- With ETA: `[██████░░░░] 60% (12/20) - ETA: 2m`

### Resources
- [CLI UX Best Practices](https://evilmartians.com/chronicles/cli-ux-best-practices-3-patterns-for-improving-progress-displays)
- [Command Line Interface Guidelines](https://clig.dev/)
- [Generate Command Line Reports](https://www.intel.com/content/www/us/en/docs/vtune-profiler/user-guide/2023-1/generating-command-line-reports.html)

---

## 7. Recommended Implementation for `jin layers`

### Suggested Architecture

```rust
// Main table structure using comfy-table
use comfy_table::{Table, Cell, Color, CellAlignment};

pub fn display_layers(layers: Vec<Layer>, options: DisplayOptions) -> Table {
    let mut table = Table::new();

    // Configure table based on options
    if options.compact {
        table.load_preset(comfy_table::presets::NOTHING);
    } else {
        table.load_preset(comfy_table::presets::UTF8_FULL);
    }

    // Set headers
    table.set_header(get_headers(options));

    // Add rows
    for layer in layers {
        table.add_row(format_layer_row(layer, options));
    }

    // Add summary row if requested
    if options.show_summary {
        table.add_separator();
        table.add_row(create_summary_row(&layers));
    }

    table
}

fn format_layer_row(layer: Layer, options: DisplayOptions) -> Vec<Cell> {
    vec![
        Cell::new(&layer.name)
            .fg(get_status_color(&layer.status))
            .set_alignment(CellAlignment::Left),
        Cell::new(&layer.status)
            .fg(get_status_color(&layer.status))
            .set_alignment(CellAlignment::Center),
        Cell::new(&format_size(layer.size))
            .set_alignment(CellAlignment::Right),
        // Additional columns based on options
    ]
}
```

### Color Scheme for Layers
- **Active**: Green (`#00FF00`)
- **Inactive**: Gray (`#808080`)
- **Pending**: Yellow (`#FFFF00`)
- **Error**: Red (`#FF0000`)
- **Merging**: Blue (`#0000FF`)

### Unicode Indicators
- Active: `●` (filled circle)
- Inactive: `○` (empty circle)
- Pending: `◐` (half-filled)
- Error: `✗`
- Success: `✓`

### Example Output
```
┌─────────────────┬────────┬─────────┬─────────────┐
│ Name            │ Status │ Size    │ Files      │
├─────────────────┼────────┼─────────┼─────────────┤
│ production-v1   │ ● active│ 45.2 MB │ 1,234      │
│ staging         │ ○ inactive│ 12.1 MB │ 567        │
│ feature-auth    │ ◐ pending│ 3.4 MB  │ 89         │
│ bugfix-security │ ✗ error │ 0 B     │ 0          │
├─────────────────┼────────┼─────────┼─────────────┤
│ Total           │        │ 60.7 MB │ 1,890      │
└─────────────────┴────────┴─────────┴─────────────┘
```

### Recommended Dependencies
1. **comfy-table** - For table formatting
2. **colored** - For simple color support
3. **humanize** - For size formatting (or custom implementation)
4. **clap** - For CLI argument parsing (already in use)

### Additional Features to Consider
- **Tree view** for nested layer structures
- **Graph view** showing layer dependencies
- **JSON output** option for programmatic use
- **Filtering** by status, size, or name
- **Sorting** by any column
- **Pagination** for large numbers of layers

---

## Sources and Further Reading

### Git Visualization
1. [StackOverflow: Pretty Git branch graphs](https://stackoverflow.com/questions/1057564/pretty-git-branch-graphs)
2. [How to Show Git Branch Graph in Terminal](https://cuda-chen.github.io/git/2021/08/14/git-pretty-branch-graph.html)
3. [Display a graph of all Git commits](https://jeffkreeftmeijer.com/git-graph/)

### Rust Table Libraries
1. [comfy-table GitHub](https://github.com/Nukesor/comfy-table)
2. [tabled GitHub](https://github.com/zhiburt/tabled)
3. [cli-table on crates.io](https://crates.io/crates/cli-table)

### Docker/Kubernetes Patterns
1. [Kubectl Output Formatting Guide](https://www.baeldung.com/ops/kubectl-output-format)
2. [Docker Format Options](https://blog.csdn.net/m0_67394002/article/details/123731574)
3. [Custom Column Mastery](https://pratikpanda.hashnode.dev/unlocking-kubernetes-efficiency-mastering-custom-column-formatting-with-kubectl)

### Terminal Colors
1. [BetterCLI: Using Colors](https://bettercli.org/design/using-colors-in-cli/)
2. [anstream Blog](https://epage.github.io/blog/2023/03/anstream-simplifying-terminal-styling/)
3. [Terminal Colors Guide](https://marvinh.dev/blog/terminal-colors/)

### CLI Best Practices
1. [CLI UX Best Practices](https://evilmartians.com/chronicles/cli-ux-best-practices-3-patterns-for-improving-progress-displays)
2. [Command Line Interface Guidelines](https://clig.dev/)
3. [NO_COLOR Discussion](https://forums.freebsd.org/threads/no_color-output-with-ansi-colour-by-default-good-or-bad.82236/)