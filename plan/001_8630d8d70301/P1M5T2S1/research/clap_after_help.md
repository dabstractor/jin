# Clap Crate's `after_help` Attribute Research

## Overview
This document provides a comprehensive overview of using the `after_help` attribute with Clap's derive API in Rust. The `after_help` attribute allows you to add custom text at the end of the help display when users run `--help` or `-h`.

---

## 1. Official Documentation

### Primary Sources

1. **Clap Builder API Documentation**
   - URL: [docs.rs/clap/latest/clap/builder/struct.Command.html](https://docs.rs/clap/latest/clap/builder/struct.Command.html)
   - Description: Shows the `after_help` method definition
   - Quote: "Longer explanation to appear after the options when displaying the help information from --help or -h"

2. **Clap Derive Documentation**
   - URL: [docs.rs/clap/latest/clap/_derive/index.html](https://docs.rs/clap/latest/clap/_derive/index.html)
   - Description: Explains how derive macros work with raw attributes
   - Quote: "This allows users to access the raw behavior of an attribute via `<attr>(<value>)` syntax"

3. **Clap Derive Tutorial**
   - URL: [docs.rs/clap/latest/clap/_derive/_tutorial/index.html](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html)
   - Description: Provides examples of derive macro usage

### GitHub Discussions and Issues

1. **"Undocumented" derive attributes #4090**
   - URL: [github.com/clap-rs/clap/discussions/4090](https://github.com/clap-rs/clap/discussions/4090)
   - Confirms that any builder method can be used as a derive attribute

2. **Advice on help menu that shows same possible enum values**
   - URL: [github.com/clap-rs/clap/discussions/5203](https://github.com/clap-rs/clap/discussions/5203)
   - Quote: "The derive reference clarifies that any builder method may be used as a derive attribute. So you can do `#[command(after_help = "")]`"

3. **Provide examples in help #3725**
   - URL: [github.com/clap-rs/clap/discussions/3725](https://github.com/clap-rs/clap/discussions/3725)
   - Discusses adding examples to help text

4. **The header for my examples are not colored like the others #3108**
   - URL: [github.com/clap-rs/clap/issues/3108](https://github.com/clap-rs/clap/issues/3108)
   - Mentions that `after_help` could take a section name

---

## 2. Code Examples

### Basic Usage

```rust
#[derive(Parser)]
#[command(after_help = "Additional information about this command")]
pub struct Cli {
    #[clap(short, long, help = "Input file")]
    input: String,

    #[clap(short, long, help = "Output file")]
    output: String,
}
```

### Real-World Examples from Projects

#### Example 1: cargo-insta project
```rust
#[derive(Debug, clap::Parser)]
#[command(after_help = "For the online documentation of ...")]
pub struct Cli {
    // ... fields
}
```

#### Example 2: czkawka project
```rust
#[derive(Debug, clap::Parser)]
#[command(after_help =
    "EXAMPLE:\n czkawka dup -d /home\nEXAMPLE:\n czkawka empty-folders -d /home"
)]
pub struct ScanCommand {
    #[clap(long, help = "Directory to scan")]
    pub scan_duration: u32,
}
```

### Complex Example with Examples Section

```rust
#[derive(Parser)]
#[command(
    name = "myapp",
    about = "A sample CLI application",
    after_help = r#"EXAMPLES:
    myapp init         # Initialize a new project
    myapp build -o out  # Build to output directory

    ENVIRONMENT:
    DEBUG=1            # Enable debug output
    CONFIG_PATH       # Custom config file path"#
)]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize a new project")]
    Init,

    #[command(about = "Build the project")]
    Build {
        #[clap(short, long, help = "Output directory")]
        output: Option<String>,
    },
}
```

---

## 3. Multi-line String Formatting in Rust Attributes

### Raw String Syntax

Rust provides raw string literals using the `r#` syntax, which is perfect for `after_help` attributes:

#### Basic Raw Strings
```rust
// Single line raw string
#[command(after_help = r"Basic raw string")]

// Multi-line raw string
#[command(after_help = r#"Line one
Line two
Line three"#)]
```

### Advanced Raw String Features

#### Escaping Quotes
```rust
#[command(after_help = r#"String with "quotes" works fine"#)]
```

#### Including Hash Symbols
Use matching `#` signs to include `#` in your string:
```rust
#[command(after_help = r###"String with # symbols and even # signs"###)]
```

#### String Interpolation (at compile time)
```rust
const HELP_TEXT: &str = "EXAMPLE:\n myapp run --port 8080";
#[command(after_help = HELP_TEXT)]
```

### Best Practices for Multi-line Strings

1. **Use `r#"..."#` for most cases** - This preserves formatting and requires no escaping
2. **Maintain consistent indentation** - The string content will appear exactly as written
3. **Use proper line breaks** - Clap respects `\n` characters in help text
4. **Consider readability** - Break long help text into logical sections

---

## 4. Table Formatting in after_help

### Manual Table Formatting

Since Clap doesn't provide built-in table formatting for `after_help`, you need to format tables manually:

#### Example Table Format
```rust
#[command(after_help = r#"EXAMPLES:
    NAME     ACTION  RESULT
    app run  start   Starts the server
    app run  stop    Stops the server

    OPTIONS:
    -p, --port  PORT   Specify port number (default: 8080)
    -v, --verbose      Enable verbose output"#
)]
```

### Color Enhancement

Use ANSI color codes to improve readability:

```rust
use color_print::cstr;

const AFTER_HELP: &'static str = cstr!(
    r#"<bold><underline>Examples</underline></bold>:
  <green>$</green> myapp run --port 3000

<bold>Environment</bold>:
  <blue>DEBUG</blue>=1    Enable debug mode
  <blue>CONFIG</blue>     Custom config path"#
);
```

### Best Practices for Table Formatting

1. **Use consistent spacing** between columns
2. **Align text properly** for readability
3. **Include headers** for tabular data
4. **Use separators** (like `---`) between sections
5. **Limit width** to avoid line wrapping issues
6. **Use bold/underline** for headers when colors are supported

---

## 5. Gotchas and Limitations

### Known Issues

1. **`\n` Character Handling**
   - Issue: [github.com/clap-rs/clap/issues/617](https://github.com/clap-rs/clap/issues/617)
   - Clap treats `\n` as byte counts rather than line breaks in some versions
   - Solution: Use multi-line raw strings instead

2. **No Built-in Table Support**
   - Clap doesn't provide automatic table formatting
   - You must manually format table-like content
   - Consider using the `clap-help` crate for advanced formatting

3. **Color Support Limitations**
   - Not all terminals support colors
   - Always provide readable fallback formatting
   - Test with `CLAP_COLORS=0` to see uncolored output

### Performance Considerations

- Large help texts are parsed each time help is displayed
- Consider using constants for complex help text
- Avoid expensive calculations in help strings

---

## 6. Alternative Approaches

### Using clap-help Crate

For advanced formatting, consider the `clap-help` crate:

```toml
[dependencies]
clap-help = "0.x"
```

### Custom Help Renderer

Create a custom help renderer for complex layouts:

```rust
fn create_command() -> Command {
    Command::new("myapp")
        .after_help("Long help text...")
        .help_template(get_custom_help_template())
}
```

### Using Templates

Clap supports custom help templates:

```rust
#[command(
    help_template = "{bin} {version}\n\n{about-with-newline}\n{usage-heading}\n    {usage}\n\n{all-args}\n{after-help}"
)]
pub struct Cli;
```

---

## 7. Summary

The `after_help` attribute in Clap's derive API is a powerful way to add custom help text at the end of help output. Key points:

1. **Use `#[command(after_help = "...")]`** syntax in derive macros
2. **Prefer raw strings** (`r#"..."#`) for multi-line help text
3. **Format tables manually** with proper spacing and alignment
4. **Consider colors** for enhanced readability (with fallbacks)
5. **Be aware of limitations** regarding line breaks and table formatting

The attribute is fully supported and documented, making it a reliable choice for adding examples, additional context, or troubleshooting information to your CLI applications.