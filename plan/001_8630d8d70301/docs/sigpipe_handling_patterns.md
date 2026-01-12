# SIGPIPE Handling Patterns in Rust CLI Applications

## Table of Contents
1. [Why SIGPIPE Causes Panics in Rust](#why-sigpipe-causes-panics-in-rust)
2. [Common Solutions Used by Popular Rust CLI Tools](#common-solutions-used-by-popular-rust-cli-tools)
3. [Best Practices for Signal Handling in Rust CLI Applications](#best-practices-for-signal-handling-in-rust-cli-applications)
4. [Testing Approaches for SIGPIPE Handling](#testing-approaches-for-sigpipe-handling)
5. [Crates Providing Safer Alternatives to Raw libc](#crates-providing-safer-alternatives-to-raw-libc)

---

## Why SIGPIPE Causes Panics in Rust

### Root Cause

Rust's standard library **ignores SIGPIPE by default** for binary targets before `fn main()` is invoked. This behavior was introduced to prevent listening servers from exiting unexpectedly when a client disconnects. However, this creates unexpected behavior for CLI applications:

1. **Traditional Unix behavior**: When a process writes to a closed pipe (e.g., when piping to `head` which terminates early), the OS sends a SIGPIPE signal, which by default terminates the process silently.

2. **Rust's behavior**: Rust sets the SIGPIPE handler to `SIG_IGN` before main() runs. This means:
   - Writing to a closed pipe doesn't trigger a signal
   - The write operation fails with an `ErrorKind::BrokenPipe` error
   - If using `println!` or `print!` macros (which panic on write errors), the program panics with an error message instead of exiting silently

### Key Sources

- **[Should Rust still ignore SIGPIPE by default? · Issue #62569](https://github.com/rust-lang/rust/issues/62569)** - Official discussion about whether this default behavior is appropriate
- **[Tracking Issue for unix_sigpipe #97889](https://github.com/rust-lang/rust/issues/97889)** - Tracks improvements to SIGPIPE handling
- **[unix_sigpipe - The Rust Unstable Book](https://dev-doc.rust-lang.org/beta/unstable-book/language-features/unix-sigpipe.html)** - Documents the unstable feature for SIGPIPE handling
- **[on-broken-pipe compiler flag](https://doc.rust-lang.org/beta/unstable-book/compiler-flags/on-broken-pipe.html)** - Compiler flag documentation for controlling SIGPIPE behavior

### Example of the Problem

```rust
// This program panics when piped to head
fn main() {
    for i in 0..1000 {
        println!("Line {}", i);  // Panics after head closes
    }
}
```

When run with `cargo run | head -n 5`, this produces:
```
Line 0
Line 1
Line 2
Line 3
Line 4
thread 'main' panicked at 'failed printing to stdout: Broken pipe (os error 32)', ...
```

### Historical Context

- Rust has set SIGPIPE to SIG_IGN since 2014 (source: `rust/library/std/src/rt.rs`)
- This was originally designed to prevent servers from crashing when clients disconnect
- The behavior differs from traditional Unix tools written in C, which get killed by SIGPIPE

---

## Common Solutions Used by Popular Rust CLI Tools

### Solution 1: Explicit Error Handling with `writeln!`

The most common pattern is to replace `println!` with `writeln!` and handle errors explicitly:

```rust
use std::io::{Write, ErrorKind};

fn main() {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    for i in 0..1000 {
        if let Err(e) = writeln!(handle, "Line {}", i) {
            if e.kind() == ErrorKind::BrokenPipe {
                // Exit silently on broken pipe (like traditional Unix tools)
                std::process::exit(0);
            } else {
                // Propagate other errors
                panic!("Failed to write to stdout: {}", e);
            }
        }
    }
}
```

**Simplified version:**
```rust
use std::io::Write;

fn main() {
    for i in 0..1000 {
        // Ignore errors entirely (quick and dirty)
        let _ = writeln!(std::io::stdout(), "Line {}", i);
    }
}
```

**Using `.ok()`:**
```rust
fn main() {
    for i in 0..1000 {
        writeln!(std::io::stdout(), "Line {}", i).ok();
    }
}
```

### Solution 2: Using the `sigpipe` Crate

The [`sigpipe`](https://crates.io/crates/sigpipe) crate restores traditional Unix SIGPIPE behavior:

```rust
// Cargo.toml
// [dependencies]
// sigpipe = "2.0"

fn main() {
    // Restore default SIGPIPE handling
    sigpipe::reset().expect("reset sigpipe");

    for i in 0..1000 {
        println!("Line {}", i);  // Will exit silently on broken pipe
    }
}
```

**GitHub Repository:** [kurtbuilds/sigpipe](https://github.com/kurtbuilds/sigpipe)

### Solution 3: Using the `signal-hook` Crate

For more comprehensive signal handling:

```rust
// Cargo.toml
// [dependencies]
// signal-hook = "0.3"

use signal_hook::consts::signal::SIGPIPE;
use signal_hook::iterator::Signals;

fn main() {
    // Register SIGPIPE handler
    let mut signals = Signals::new([SIGPIPE])
        .expect("Failed to register signal handler");

    // Handle signals in a background thread
    std::thread::spawn(move || {
        for sig in signals.forever() {
            if sig == SIGPIPE {
                // Exit gracefully on SIGPIPE
                std::process::exit(0);
            }
        }
    });

    for i in 0..1000 {
        println!("Line {}", i);
    }
}
```

### Solution 4: Helper Function Pattern

Create a reusable helper function:

```rust
use std::io::{Write, ErrorKind};

fn print_quietly(args: std::fmt::Arguments<'_>) {
    if let Err(e) = writeln!(std::io::stdout(), "{}", args) {
        if e.kind() != ErrorKind::BrokenPipe {
            // Only warn if it's NOT a broken pipe
            eprintln!("Warning: Failed to write to stdout: {}", e);
        }
    }
}

fn main() {
    for i in 0..1000 {
        print_quietly(format_args!("Line {}", i));
    }
}
```

### Real-World Examples

**ripgrep (BurntSushi/ripgrep):**
- **[Issue #200: ripgrep doesn't stop when its pipe is closed](https://github.com/BurntSushi/ripgrep/issues/200)** - Discussion about handling broken pipes
- **[Issue #2939: SIGPIPE can race with the exit code](https://github.com/BurntSushi/ripgrep/issues/2939)** - SIGPIPE handling in ripgrep 13.0.0
- Ripgrep implements proper broken pipe detection to stop searching when stdout fails

**Discussion from BurntSushi:**
- **[writeln!(io::stdout()) vs println - BurntSushi/advent-of-code#17](https://github.com/BurntSushi/advent-of-code/issues/17)** - Discussion about using `writeln!` instead of `println!` for better error control

---

## Best Practices for Signal Handling in Rust CLI Applications

### 1. Choose the Right Approach for Your Use Case

**For CLI Tools (most common):**
- Use explicit error handling with `writeln!`
- Exit silently on `ErrorKind::BrokenPipe`
- This matches traditional Unix tool behavior

**For Servers:**
- Keep Rust's default SIGPIPE ignoring behavior
- Handle broken pipe errors explicitly in your connection logic
- This prevents the entire server from crashing when a client disconnects

**For Libraries:**
- Don't modify signal handlers in library code
- Let the application decide on signal handling strategy
- Return errors that the application can handle appropriately

### 2. Always Handle Write Errors

**Avoid:**
```rust
// Bad: Will panic on broken pipe
println!("Output");
```

**Prefer:**
```rust
// Good: Handles errors gracefully
use std::io::Write;

fn print_output(output: &str) -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();
    writeln!(handle, "{}", output)
}

fn main() {
    if let Err(e) = print_output("Output") {
        if e.kind() != std::io::ErrorKind::BrokenPipe {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
```

### 3. Use Buffered Writing for Performance

```rust
use std::io::{BufWriter, Write};

fn main() {
    let stdout = std::io::stdout();
    let handle = BufWriter::new(stdout.lock());

    // Use a scope to ensure handle is flushed before exit
    let result = {
        let mut handle = handle;
        for i in 0..1000 {
            writeln!(handle, "Line {}", i)?;
        }
        Ok::<_, std::io::Error>(())
    };

    // Handle errors after the scope
    if let Err(e) = result {
        if e.kind() != std::io::ErrorKind::BrokenPipe {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
```

### 4. Document Your Signal Handling Strategy

```rust
/// This CLI tool follows traditional Unix conventions for SIGPIPE:
/// - When stdout is closed (e.g., when piped to `head`), the tool exits silently
/// - Other write errors are reported to stderr
/// - Exit code 0 is used for broken pipe scenarios
```

### 5. Consider Using Unstable Features (When Appropriate)

The `unix_sigpipe` feature (unstable) provides more control:

```rust
// Requires Rust nightly and -Z unix_sigpipe flag
// See: https://dev-doc.rust-lang.org/beta/unstable-book/language-features/unix-sigpipe.html
```

### 6. Be Careful with Child Processes

Rust's signal handling affects child processes:

- **[Issue #62569 Discussion](https://github.com/rust-lang/rust/issues/62569)** - Notes that Rust resets signal masks in child processes
- Consider using `std::process::Command` with proper signal handling
- Be aware that child processes inherit the parent's signal disposition

---

## Testing Approaches for SIGPIPE Handling

### 1. Manual Testing with Pipes

**Basic test:**
```bash
# Test that your tool exits silently when piped to head
cargo run | head -n 5

# Should see only 5 lines, no error messages
```

**Test with various commands:**
```bash
# Test with head
cargo run | head -n 1

# Test with grep -q (exits after first match)
cargo run | grep -q "pattern"

# Test with tail (closes early)
cargo run | tail -n 5
```

### 2. Automated Testing with Child Processes

```rust
#[cfg(test)]
mod tests {
    use std::process::{Command, Stdio};
    use std::io::Write;

    #[test]
    fn test_sigpipe_handling() {
        // Spawn your process
        let mut child = Command::new("cargo")
            .args(&["run", "--", "arg1"])
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn child");

        // Close stdin immediately (simulating head closing early)
        if let Some(mut stdout) = child.stdout.take() {
            drop(stdout);
        }

        // Wait for the process to exit
        let status = child.wait().expect("Failed to wait for child");

        // Should exit successfully (not panic)
        assert!(status.success(), "Process should exit cleanly on broken pipe");
    }
}
```

### 3. Testing with Actual Pipes

```rust
#[cfg(test)]
mod tests {
    use std::process::{Command, Stdio, Pipe};

    #[test]
    fn test_piped_to_head() {
        // Create a pipe
        let mut child = Command::new("head")
            .arg("-n")
            .arg("5")
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to spawn head");

        // Write to head's stdin
        if let Some(mut stdin) = child.stdin.as_mut() {
            for i in 0..1000 {
                writeln!(stdin, "Line {}", i).ok();
            }
        }

        // Wait for head to exit
        let status = child.wait().expect("Failed to wait");

        // Head should exit successfully
        assert!(status.success());
    }
}
```

### 4. Integration Tests

**Create `tests/sigpipe.rs`:**
```rust
use std::process::Command;

#[test]
fn test_no_panic_on_broken_pipe() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("cargo run --quiet 2>&1 | head -n 1")
        .output()
        .expect("Failed to execute command");

    // Should not contain panic messages
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("panicked"), "Should not panic on broken pipe");

    // Should exit successfully
    assert!(output.status.success());
}
```

### 5. Property-Based Testing

```rust
#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn test_various_pipe_scenarios() {
        let scenarios = vec![
            "head -n 1",
            "head -n 10",
            "head -n 100",
            "tail -n 5",
            "grep -q test",
        ];

        for scenario in scenarios {
            let output = Command::new("sh")
                .arg("-c")
                .arg(&format!("echo test | {}", scenario))
                .output()
                .expect("Failed to execute command");

            assert!(output.status.success(),
                    "Failed for scenario: {}", scenario);
        }
    }
}
```

### 6. Testing Exit Codes

```rust
#[test]
fn test_exit_code_on_broken_pipe() {
    let output = Command::new("sh")
        .arg("-c")
        .arg("cargo run --quiet 2>&1 | head -n 1; echo $?")
        .output()
        .expect("Failed to execute command");

    // Exit code should be 0 (success) for broken pipe
    assert!(output.status.success());
}
```

---

## Crates Providing Safer Alternatives to Raw libc

### 1. `sigpipe` Crate

**Source:** [kurtbuilds/sigpipe](https://github.com/kurtbuilds/sigpipe)
**Crates.io:** https://crates.io/crates/sigpipe

**Purpose:** Fixes the issue where the default Rust runtime panics when `println!` and similar macros write to a closed pipe.

**Usage:**
```rust
fn main() {
    // Restore default SIGPIPE handling
    sigpipe::reset().expect("reset sigpipe");

    // Now println! will exit silently on broken pipe
    println!("Hello, world!");
}
```

**Pros:**
- Single function call
- Minimal dependencies
- Well-maintained
- Specifically designed for this problem

**Cons:**
- Only handles SIGPIPE
- Not a general signal handling solution

### 2. `signal-hook` Crate

**Source:** https://github.com/vorner/signal-hook
**Crates.io:** https://crates.io/crates/signal-hook
**Documentation:** https://docs.rs/signal-hook

**Purpose:** Library for safe and correct Unix signal handling in Rust.

**Usage:**
```rust
use signal_hook::consts::signal::SIGPIPE;
use signal_hook::iterator::Signals;

fn main() {
    let mut signals = Signals::new([SIGPIPE])
        .expect("Failed to register signal handler");

    std::thread::spawn(move || {
        for sig in signals.forever() {
            match sig {
                SIGPIPE => {
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    });

    // Your application code here
}
```

**Pros:**
- Comprehensive signal handling
- Type-safe API
- Well-documented
- Active maintenance
- Supports multiple signal handling strategies

**Cons:**
- More complex than needed for just SIGPIPE
- Requires managing a background thread

**Documentation:**
- [SIGPIPE constant documentation](https://docs.rs/signal-hook/latest/signal_hook/consts/signal/constant.SIGPIPE.html)
- [Crate documentation](https://docs.rs/signal-hook)

### 3. `nix` Crate

**Crates.io:** https://crates.io/crates/nix
**Documentation:** https://docs.rs/nix

**Purpose:** Rust-friendly bindings to Unix/Linux system calls and signal handling.

**Usage:**
```rust
use nix::sys::signal::{self, SigHandler, Signal};

fn main() {
    // Set SIGPIPE to default behavior
    unsafe {
        signal::signal(Signal::SIGPIPE, SigHandler::SigDfl)
            .expect("Failed to set signal handler");
    }

    // Your application code here
}
```

**Pros:**
- Comprehensive Unix API coverage
- Type-safe
- Well-maintained
- Provides low-level control

**Cons:**
- Requires `unsafe` blocks
- More complex than needed for simple cases
- Larger dependency

### 4. `sigpipe-default` Crate

**Crates.io:** https://crates.io/crates/sigpipe-default

**Purpose:** Makes the Rust standard library keep SIGPIPE as SIG_DFL (default behavior).

**Usage:**
```rust
fn main() {
    sigpipe_default::reset();
    // Your application code here
}
```

**Pros:**
- Simple API
- Addresses the root cause

**Cons:**
- Less popular than `sigpipe`
- Minimal documentation

### 5. `signal-hook-registry` Crate

**Crates.io:** https://crates.io/crates/signal-hook-registry

**Purpose:** Backend crate for signal-hook, provides the registry for signal handlers.

**Usage:** Typically used indirectly through `signal-hook`.

### Comparison Table

| Crate | Complexity | Safety | Scope | Maintenance |
|-------|-----------|--------|-------|-------------|
| `sigpipe` | Low | High | SIGPIPE only | Active |
| `signal-hook` | Medium | High | All signals | Active |
| `nix` | High | High (with unsafe) | All Unix APIs | Active |
| `sigpipe-default` | Low | High | SIGPIPE only | Less active |

### Recommendation

**For CLI applications:**
- Use the `sigpipe` crate for simplicity
- OR use explicit error handling with `writeln!` (no dependencies)

**For comprehensive signal handling:**
- Use the `signal-hook` crate
- Provides a safe, idiomatic Rust API

**For low-level control:**
- Use the `nix` crate
- Requires `unsafe` but provides complete control

### Additional Resources

**Learning Resources:**
- **[Handling POSIX Signals in Rust - Medium](https://sigridjin.medium.com/handling-posix-signals-in-rust-fac42c33e5b6)** - Explains SIGPIPE and signal handling
- **[Guide to Signal Handling in Rust - LogRocket](https://blog.logrocket.com/guide-signal-handling-rust/)** - Comprehensive guide with `nix` examples
- **[Rust CLI Book - Signal Handling](https://rust-cli.github.io/book/in-depth/signals.html)** - Official CLI documentation
- **[Handling Unix Kill Signals in Rust - dev.to](https://dev.to/talzvon/handling-unix-kill-signals-in-rust-55g6)** - Tutorial covering signal-hook

**Community Discussions:**
- **[Something In The Crate signal_hook Is Puzzling Me](https://users.rust-lang.org/t/something-in-the-crate-signal-hook-is-puzzling-me/107343)** - Forum discussion with examples
- **[Rust Users Forum: Broken pipe when attempt to write to stdout](https://users.rust-lang.org/t/broken-pipe-when-attempt-to-write-to-stdout/111186)** - Community discussion
- **[Defusing println! when stdout is closed](https://internals.rust-lang.org/t/defusing-println-when-stdout-is-closed/20325)** - Rust internals discussion

---

## Summary and Key Takeaways

### Why SIGPIPE Causes Issues

1. Rust ignores SIGPIPE by default (sets to SIG_IGN before main())
2. Writing to a closed pipe returns `ErrorKind::BrokenPipe` instead of terminating
3. `println!` panics on write errors, showing error messages instead of exiting silently
4. This differs from traditional Unix tools that exit silently on broken pipes

### Common Solutions

1. **Explicit error handling** - Use `writeln!` and check for `ErrorKind::BrokenPipe`
2. **`sigpipe` crate** - Restore traditional Unix behavior with one function call
3. **`signal-hook` crate** - Comprehensive signal handling solution
4. **Helper functions** - Create reusable functions for safe printing

### Best Practices

1. Always handle write errors in CLI applications
2. Exit silently (code 0) on broken pipe errors
3. Use `writeln!` instead of `println!` for better error control
4. Consider using buffered writing for performance
5. Document your signal handling strategy

### Testing

1. Test manually with pipes: `cargo run | head -n 5`
2. Use integration tests with child processes
3. Test various pipe scenarios (head, tail, grep -q)
4. Verify exit codes and lack of panic messages

### Crates

1. **`sigpipe`** - Simple, focused solution for SIGPIPE
2. **`signal-hook`** - Comprehensive, safe signal handling
3. **`nix`** - Low-level Unix bindings (requires unsafe)
4. **`sigpipe-default`** - Alternative to `sigpipe`

---

## Additional References

### Rust Language Issues

- **[Issue #46016: Spurious "broken pipe" error messages](https://github.com/rust-lang/rust/issues/46016)** - Early discussion about stdout panicking in pipe scenarios
- **[Issue #35108: Stdout does panic in pipe](https://github.com/rust-lang/rust/issues/35108)** - Earlier discussion about the issue
- **[Issue #98700: rustc -C help panics when piped to closed stdout](https://github.com/rust-lang/rust/issues/98700)** - Example of the issue affecting rustc itself
- **[RFC #1368: Signal handling](https://github.com/rust-lang/rfcs/issues/1368)** - RFC on signal handling

### StackOverflow Discussions

- **[How to prevent SIGPIPEs (or handle them properly)](https://stackoverflow.com/questions/108183/how-to-prevent-sigpipes-or-handle-them-properly)** - General SIGPIPE handling
- **[Simple word count rust program panicks when piped to head](https://stackoverflow.com/questions/65755853/simple-word-count-rust-program-outputs-valid-stdout-but-panicks-when-piped-to-he)** - Specific example
- **[Rust how to ignore print to stdout error](https://stackoverflow.com/questions/77799521/rust-how-to-ignore-print-to-stdout-error)** - Error handling discussion
- **[How to fix "Broken Pipe" error when trying to write a named pipe](https://stackoverflow.com/questions/70483641/how-to-fix-broken-pipe-error-when-trying-to-write-a-named-pipe)** - Named pipes
- **[Program received signal SIGPIPE, Broken pipe](https://stackoverflow.com/questions/18935446/program-received-signal-sigpipe-broken-pipe)** - C discussion but relevant

### Unix/Linux Resources

- **[Under what conditions exactly does SIGPIPE happen?](https://unix.stackexchange.com/questions/433345/under-what-conditions-exactly-does-sigpipe-happen)** - Unix & Linux explanation
- **[Effective handling of SIGPIPE](http://www.pixelbeat.org/programming/sigpipe_handling.html)** - General guide
- **[Preventing SIGPIPE - linux](https://stackoverflow.com/questions/11302873/preventing-sigpipe)** - Linux-specific discussion

### Community Posts

- **[Hacker News: Fixing Ctrl+C in Rust terminal apps](https://news.ycombinator.com/item?id=44728796)** - Discusses SIGPIPE behavior
- **[Reddit: Helps understanding a post on performances/syscalls](https://www.reddit.com/r/eksakythelps_understanding_a_post_on_performancessyscalls/)** - Confirms Rust blocks SIGPIPE
- **[Reddit: Detect closed/invalid/… stdout](https://www.reddit.com/r/rust/comments/akavh1/detect_closedinvalid_stdout/)** - Detection strategies

### Other

- **[SIGPIPE signal handling - StackOverflow](https://stackoverflow.com/questions/23860847/sigpipe-signal-handling)** - General signal handling
- **[bash - How can I fix a Broken Pipe error](https://superuser.com/questions/554855/how-can-i-fix-a-broken-pipe-error)** - Shell perspective
- **[Lobsters: Rewritten in Rust: Modern Alternatives of Command-Line](https://lobste.rs/s/2mxwdm/rewritten_rust_modern_alternatives)** - Discussion of Rust CLI tools
- **[Reddit: I LOVE Rust's exception handling](https://www.reddit.com/r/rust/comments/13dcxy3/i_love_rusts_exception_handling/)** - Error handling discussion

---

*Last updated: January 10, 2026*
*Research conducted for: P1M2T1S1 - SIGPIPE Handling Patterns*
