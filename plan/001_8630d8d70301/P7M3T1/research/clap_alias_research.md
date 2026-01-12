# CLAP Alias Best Research - For `jin modes` Command

## Research Overview

This document researches CLAP alias best practices for the specific use case of making `jin modes` behave like `jin mode list`. The research covers different approaches, patterns, limitations, and testing considerations.

## 1. Findings: Rust CLIs Using Command Aliases

### Examples from Real-world Rust CLIs

#### a. Fast Node Manager (fnm)
- **GitHub Repository**: [fnm](https://github.com/Schniz/fnm)
- **Pattern**: Uses visible aliases for subcommands
- **Code Example**:
  ```rust
  #[clap(name = "list-remote", visible_aliases = &["ls-remote"])]
  LsRemote
  ```
- **Usage**: Both `fnm list-remote` and `fnm ls-remote` work identically

#### b. Cargo (clap-rs itself)
- **GitHub Repository**: [clap-rs](https://github.com/clap-rs/clap)
- **Pattern**: Uses aliases for common typos and abbreviations
- **Code Example**:
  ```rust
  #[command(alias = "clippy")]
  Lint {
      #[arg(short, long, alias = "allow")]
      allow_warnings: bool,
  }
  ```

#### c. Ripgrep (rg)
- **GitHub Repository**: [ripgrep](https://github.com/BurntSushi/ripgrep)
- **Pattern**: Uses multiple aliases for search commands
- **Code Example**:
  ```rust
  Command::new("search")
      .visible_aliases(&["grep", "find"])
      .arg(...)
  ```

### Key Patterns Observed

1. **Hidden Aliases**: Most CLIs use hidden aliases (.alias()) for internal routing
2. **Visible Aliases**: User-facing aliases use .visible_alias() or .visible_aliases()
3. **Short Form Aliases**: Common to provide short single-letter aliases
4. **Plural/Singular**: Many CLIs handle both singular and plural forms (mode/modes)

## 2. Patterns for `jin modes` behaving like `jin mode list`

### Option 1: Use Subcommand Aliases (Recommended)

This is the cleanest approach using clap's built-in alias support.

```rust
// In src/cli/mod.rs

#[derive(Subcommand, Debug)]
pub enum ModeAction {
    /// Create a new mode
    Create { name: String },

    /// Activate a mode
    Use { name: String },

    /// List available modes
    #[command(visible_alias = "modes")]
    List,

    /// Delete a mode
    Delete { name: String },

    /// Show current mode
    Show,

    /// Deactivate current mode
    Unset,
}
```

**Pros**:
- Clean, clap-native solution
- Built-in help text shows aliases
- No additional routing logic needed
- Follows clap best practices

**Cons**:
- Aliases appear in help text (may clutter if many aliases)
- Limited to subcommands only

### Option 2: Custom Command Routing

Create a separate `Modes` command that routes to `Mode::List`.

```rust
// In src/cli/mod.rs

#[derive(Subcommand, Debug)]
pub enum ModeAction {
    // ... existing commands ...
    List,
    Show,
    Unset,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    // ... existing commands ...

    /// Mode lifecycle management
    #[command(subcommand)]
    Mode(ModeAction),

    /// List available modes (alias for `jin mode list`)
    #[command(visible_alias = "modes")]
    Modes(ListModeArgs),
}

#[derive(Args, Debug)]
pub struct ListModeArgs {
    // No additional args needed for now
}
```

**Pros**:
- Complete separation of concerns
- Can have different help text
- More flexible for future expansion

**Cons**:
- Duplicates code/arguments
- More complex implementation
- Requires custom routing logic

### Option 3: Use Command Grouping

Create a mode group with multiple entry points.

```rust
// In src/cli/mod.rs

#[derive(Subcommand, Debug)]
pub enum Commands {
    // ... existing commands ...

    /// Mode lifecycle management
    #[command(subcommand)]
    Mode(ModeAction),

    /// Mode management (alias group)
    #[command(flatten)]
    ModeGroup(ModeGroup),
}

#[derive(Subcommand, Debug)]
pub enum ModeGroup {
    /// List available modes
    #[command(name = "mode")]
    ListMode {
        #[command(subcommand)]
        action: ModeListAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum ModeListAction {
    List,
}
```

**Pros**:
- Keeps related commands together
- Clean structure

**Cons**:
- Complex nesting
- May confuse users
- Not commonly used pattern

## 3. Better Patterns Than Creating Separate Commands

### Recommended Approach: Use Subcommand Aliases

For the specific use case of `jin modes` behaving like `jin mode list`, the best pattern is:

```rust
#[derive(Subcommand, Debug)]
pub enum ModeAction {
    /// Create a new mode
    Create { name: String },

    /// Activate a mode
    Use { name: String },

    /// List available modes
    #[command(visible_alias = "modes")]
    List,

    /// Delete a mode
    Delete { name: String },

    /// Show current mode
    Show,

    /// Deactivate current mode
    Unset,
}
```

### Alternative: Multiple Visible Aliases

If you want to support multiple aliases:

```rust
/// List available modes
#[command(visible_aliases = &["modes", "ls"])]
List,
```

### Why This is Better:

1. **Single Source of Truth**: Only one implementation needed
2. **Automatic Help**: clap automatically shows aliases in help
3. **No Duplication**: No need to maintain separate command handlers
4. **Clap Native**: Uses built-in clap functionality
5. **Future-Proof**: Easy to add more aliases later

## 4. Limitations and Gotchas with CLAP Aliases

### a. Short Option Conflicts
- **Issue**: Short options must be unique across all arguments
- **Example**: Can't have two arguments using `-h`
- **Solution**: Use explicit short flags with `#[arg(short = 'x')]`

### b. Argument Aliases Not Supported
- **Issue**: clap does NOT support aliases for regular arguments
- **Only Works**: For subcommands, not for individual arguments
- **Workaround**: Use subcommands with hidden arguments if needed

### c. Hidden vs Visible Aliases
```rust
// Hidden alias (functional but not in help)
.alias("modes")

// Visible alias (shown in help)
.visible_alias("modes")

// Multiple visible aliases
.visible_aliases(&["modes", "ls"])
```

### d. Help Text Formatting
- Aliases appear in brackets after command name
- Format: `command [aliases: modes, ls]`
- Some users find this formatting awkward

### e. Conflict Detection
- Conflict validation happens at runtime, not compile time
- Can lead to surprises if aliases conflict unexpectedly

### f. POSIX Mode Compatibility
- clap treats conflicts as errors by default
- Can configure with `.overrides_with()` for POSIX behavior

### g. Testing Considerations
- Must test both original and alias forms
- Aliases don't appear in type system
- Need to test help output includes aliases

## 5. Testing Considerations

### a. Unit Testing
Test that both forms work identically:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_mode_list_alias() {
        let cli = Cli::parse_from(&["jin", "mode", "list"]);
        assert!(matches!(cli.command, Commands::Mode(ModeAction::List)));

        let cli2 = Cli::parse_from(&["jin", "mode", "modes"]);
        assert!(matches!(cli2.command, Commands::Mode(ModeAction::List)));
    }

    #[test]
    fn test_help_includes_alias() {
        let app = Cli::command();
        let help = app.render_help();
        assert!(help.to_string().contains("modes"));
    }
}
```

### b. Integration Testing
Use `assert_cmd` for end-to-end testing:

```rust
use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn test_modes_alias() {
    Command::cargo_bin("jin")
        .args(&["mode", "modes"])
        .assert()
        .success()
        .stdout(contains("Available modes:"));
}
```

### c. Coverage Testing
- Ensure both paths trigger same code
- Test help output includes aliases
- Test error handling works for both forms

### Testing Tools:
- **assert_cmd**: For integration testing
- **predicates**: For output assertions
- **cargo-llvm-cov**: For coverage measurement
- **clap::Parser::parse_from()**: For unit testing argument parsing

## 6. Recommended Implementation for Jin

Based on this research, here's the recommended implementation:

```rust
// In src/cli/mod.rs

#[derive(Subcommand, Debug)]
pub enum ModeAction {
    /// Create a new mode
    Create { name: String },

    /// Activate a mode
    Use { name: String },

    /// List available modes
    #[command(visible_alias = "modes")]
    List,

    /// Delete a mode
    Delete { name: String },

    /// Show current mode
    Show,

    /// Deactivate current mode
    Unset,
}
```

### Testing Implementation:

```rust
// In tests/cli_mode.rs

#[test]
fn test_mode_list_and_modes_equivalent() {
    // Test that both commands parse to the same variant
    let cli1 = Cli::parse_from(&["jin", "mode", "list"]);
    let cli2 = Cli::parse_from(&["jin", "mode", "modes"]);

    match (cli1.command, cli2.command) {
        (Commands::Mode(ModeAction::List), Commands::Mode(ModeAction::List)) => (),
        _ => panic!("Both commands should parse to ModeAction::List"),
    }
}

#[test]
fn test_modes_alias_in_help() {
    let app = Cli::command();
    let help = app.render_help();
    let help_text = help.to_string();

    assert!(help_text.contains("modes"));
    assert!(help_text.contains("[aliases: modes]"));
}
```

## 7. Conclusion

For making `jin modes` behave like `jin mode list`, the best approach is to use clap's built-in subcommand aliases with `.visible_alias("modes")`. This provides:

1. Clean implementation without code duplication
2. Built-in help text support
3. Native clap functionality
4. Easy to test
5. Future-extensible

The key limitation is that aliases only work for subcommands, not for individual arguments, but this is not a limitation for our use case.

## References

- [CLAP Command Documentation](https://docs.rs/clap/latest/clap/struct.Command.html)
- [StackOverflow: Multiple subcommands same functionality](https://stackoverflow.com/questions/73789062/how-can-i-make-multiple-subcommands-that-do-the-same-thing)
- [GitHub Issue: Better looking visible aliases](https://github.com/clap-rs/clap/issues/1398)
- [fnm CLI Example](https://github.com/Schniz/fnm)
- [Testing with assert_cmd](https://docs.rs/assert_cmd/latest/assert_cmd/)