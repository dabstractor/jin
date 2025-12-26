# Rust CLI Best Practices Research

## Key Documentation Links

### Clap & CLI Structure
- [Clap Official Documentation](https://docs.rs/clap/latest/clap/)
- [Clap Derive Tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html)
- [Writing a CLI Tool in Rust with Clap - Shuttle](https://www.shuttle.dev/blog/2023/12/08/clap-rust)
- [Implementing Subcommands with Clap](https://www.rustadventure.dev/building-a-digital-garden-cli/clap-v4/implementing-subcommands-with-clap)
- [Handling Arguments and Subcommands](https://rust-cli-recommendations.sunshowers.io/handling-arguments.html)

### Project Structure & Modules
- [The Rust Programming Language - Packages, Crates, Modules](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html)
- [Cargo Project Layout](https://doc.rust-lang.org/cargo/guide/project-layout.html)
- [How to Structure a Rust Project Idiomatically - DEV Community](https://dev.to/sgchris/how-to-structure-a-rust-project-idiomatically-500k)

### Error Handling
- [Rust Error Handling Guide 2025](https://markaicode.com/rust-error-handling-2025-guide/)
- [Error Type Design Patterns](https://nrc.github.io/error-design/error-type-design.html)
- [Rust By Example - Define Custom Errors](https://doc.rust-lang.org/rust-by-example/error/multiple_error_types/define_error_type.html)
- [Effective Error Handling in Rust CLI Apps](https://technorely.com/insights/effective-error-handling-in-rust-cli-apps-best-practices-examples-and-advanced-techniques)

### Configuration Management
- [Config Crate Documentation](https://docs.rs/config/latest/config/)
- [Cargo Configuration Reference](https://doc.rust-lang.org/cargo/reference/config.html)

### Testing Patterns
- [Testing CLI Applications - Rust CLI Book](https://rust-cli.github.io/book/tutorial/testing.html)
- [How to Test Rust CLI Applications - Neil Henning](https://www.neilhenning.dev/posts/rust-lit/)
- [Approaches for E2E Testing - Sling Academy](https://www.slingacademy.com/article/approaches-for-end-to-end-testing-in-rust-cli-applications/)

### Cargo.toml Best Practices
- [Specifying Dependencies - The Cargo Book](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
- [Features System - The Cargo Book](https://doc.rust-lang.org/cargo/reference/features.html)
- [The Manifest Format](https://doc.rust-lang.org/cargo/reference/manifest.html)

## Key Patterns

### Recommended Subcommand Architecture

```rust
// RECOMMENDED - Struct with Command enum (allows future global options)
#[derive(Debug, Parser)]
#[clap(name = "my-app", version)]
pub struct App {
    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Read { /* args */ },
    Write(WriteArgs),
    Delete(DeleteArgs),
}
```

### Error Handling Pattern

```rust
// Library code: Use thiserror
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Failed to read file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
```

### CLI Testing Pattern

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_output() {
    Command::cargo_bin("my-cli")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}
```

## Recommended Dependencies

```toml
[dependencies]
clap = { version = "4.5", features = ["derive", "cargo"] }
thiserror = "2.0"
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

[dev-dependencies]
assert_cmd = "2.0"
assert_fs = "1.0"
predicates = "3.0"
tempfile = "3.0"
```
