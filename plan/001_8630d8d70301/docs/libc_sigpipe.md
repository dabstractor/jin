# libc Crate SIGPIPE Signal Handling Research

## Overview

This document provides comprehensive research on SIGPIPE signal handling in Rust using the `libc` crate, including official documentation, usage patterns, platform considerations, and best practices.

---

## 1. Official libc Crate Documentation URLs

### Primary Documentation
- **libc crate main page**: https://docs.rs/libc
- **libc crate (latest version 0.2.170)**: https://docs.rs/libc/latest/libc/index.html

### Signal-Specific Documentation
- **`signal()` function**: https://docs.rs/libc/latest/libc/fn.signal.html
- **`SIGPIPE` constant**: https://docs.rs/libc/latest/libc/constant.SIGPIPE.html
- **`SIG_DFL` constant**: https://docs.rs/libc/latest/libc/constant.SIG_DFL.html
- **`SIG_IGN` constant**: https://docs.rs/libc/latest/libc/constant.SIG_IGN.html

### Alternative Documentation Views
- **ESP-RS libc documentation**: https://docs.esp-rs.org/esp-idf-hal/libc/fn.signal.html
- **SIG_DFL (Diesel docs mirror)**: https://docs.diesel.rs/main/libc/constant.SIG_DFL.html

### Repository
- **libc GitHub repository**: https://github.com/rust-lang/libc

---

## 2. Function Signatures and Constants

### `libc::signal()` Function

**Full signature:**
```rust
pub unsafe extern "C" fn signal(
    signum: c_int,
    handler: sighandler_t,
) -> sighandler_t
```

**Parameters:**
- `signum: c_int` - The signal number (e.g., `libc::SIGPIPE`)
- `handler: sighandler_t` - Signal handler function or special constant

**Return value:**
- Returns `sighandler_t` - Previous signal handler, or `SIG_ERR` on error

### Signal Constants

```rust
// Signal number for broken pipe
pub const SIGPIPE: c_int = 13;

// Default signal handling
pub const SIG_DFL: sighandler_t = _; // 0usize

// Ignore signal
pub const SIG_IGN: sighandler_t = _; // 1usize
```

### Type Definitions

```rust
// Signal handler type
pub type sighandler_t = usize;
```

---

## 3. How to Use libc::signal() with SIGPIPE and SIG_DFL

### Basic Usage Pattern

The most common pattern for resetting SIGPIPE to default behavior:

```rust
#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    };
}

fn main() {
    reset_sigpipe();
    // Rest of your program
}
```

### Complete Example with Error Handling

```rust
#[cfg(unix)]
use std::io::{self, Write};

#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

fn main() {
    #[cfg(unix)]
    reset_sigpipe();

    // Your program logic here
    // When writing to a closed pipe, the program will now
    // exit with SIGPIPE (signal 13) instead of panicking
}
```

### Check for Errors

```rust
#[cfg(unix)]
fn reset_sigpipe() -> Result<(), String> {
    unsafe {
        let result = libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        if result == libc::SIG_ERR {
            return Err("Failed to reset SIGPIPE handler".to_string());
        }
    }
    Ok(())
}
```

---

## 4. Proper Usage of Unsafe Blocks with libc Calls

### Why Unsafe is Required

All `libc` FFI bindings are `unsafe` because:
1. They can cause undefined behavior if used incorrectly
2. The compiler cannot verify signal handler validity
3. Signal handling has global state effects

### Best Practices for Unsafe Blocks

#### 1. Keep Unsafe Blocks Minimal

```rust
// Good: Minimal unsafe scope
#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

// Avoid: Large unsafe blocks without clear reason
#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        // ... lots of other unrelated code ...
    }
}
```

#### 2. Document Safety Invariants

```rust
/// Resets SIGPIPE to default behavior.
///
/// # Safety
///
/// This function calls `libc::signal()` which is inherently unsafe as
/// it modifies global signal handler state. The call is safe because:
/// - SIGPIPE is a valid signal number on all Unix systems
/// - SIG_DFL is a valid handler constant
/// - The function has no other side effects that depend on signal state
#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}
```

#### 3. Handle Return Values

```rust
#[cfg(unix)]
fn reset_sigpipe() -> io::Result<()> {
    unsafe {
        let result = libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        if result == libc::SIG_ERR {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}
```

#### 4. Platform-Specific Compilation

```rust
// Only compile on Unix-like systems
#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

// Provide a no-op implementation for other platforms
#[cfg(not(unix))]
fn reset_sigpipe() {
    // SIGPIPE doesn't exist on non-Unix platforms
}
```

---

## 5. Platform-Specific Considerations

### Unix vs Windows

#### Unix/Linux/macOS/BSD
- **SIGPIPE exists**: Yes
- **libc::signal() available**: Yes
- **Default behavior**: Process terminates when writing to broken pipe
- **Rust's default**: Ignores SIGPIPE (sets to SIG_IGN before main)

**Example:**
```rust
#[cfg(unix)]
mod unix_sigpipe {
    pub fn reset() {
        unsafe {
            libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        }
    }
}
```

#### Windows
- **SIGPIPE exists**: No (Windows uses different mechanisms)
- **libc::signal() available**: Limited support (see [Issue #1600](https://github.com/rust-lang/libc/issues/1600))
- **Alternative**: Use `SetErrorMode` or structured exception handling
- **SIG_DFL/SIG_IGN**: May not be defined on Windows targets

**Platform-specific handling:**
```rust
#[cfg(unix)]
mod platform {
    pub use libc::{SIGPIPE, SIG_DFL, signal};
}

#[cfg(windows)]
mod platform {
    // Windows doesn't have SIGPIPE in the traditional sense
    // Provide stub implementations or use Windows-specific APIs
}
```

### Platform Target Triples

According to [libc Issue #1600](https://github.com/rust-lang/libc/issues/1600), `SIG_DFL` and `SIG_IGN` were missing on Windows targets:
- `*-pc-windows-*` triples
- May not have complete signal constant support

### Cross-Platform Pattern

```rust
#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

#[cfg(not(unix))]
fn reset_sigpipe() {
    // No-op on non-Unix platforms
    // Windows handles broken pipes differently via error codes
}
```

---

## 6. Common Patterns for SIGPIPE Reset in Rust CLI Applications

### Pattern 1: Simple Function Call at Start

```rust
fn main() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    // Rest of application
}
```

### Pattern 2: Dedicated Module

```rust
mod sigpipe {
    #[cfg(unix)]
    pub fn reset() {
        unsafe {
            libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        }
    }

    #[cfg(not(unix))]
    pub fn reset() {
        // No-op on non-Unix platforms
    }
}

fn main() {
    sigpipe::reset();
    // Application logic
}
```

### Pattern 3: Using Third-Party Crates

#### sigpipe crate (by kurtbuilds)

**Repository**: https://github.com/kurtbuilds/sigpipe
**crates.io**: https://crates.io/crates/sigpipe

**Usage:**
```rust
// Cargo.toml
[dependencies]
sigpipe = "0.1"

// main.rs
fn main() {
    sigpipe::reset();
    // Rest of your program
}
```

**Implementation:**
```rust
#[cfg(unix)]
pub fn reset() {
    unsafe {
        ::libc::signal(::libc::SIGPIPE, ::libc::SIG_DFL);
    }
}
```

### Pattern 4: Once Initialization (for libraries)

```rust
use std::sync::Once;

static INIT: Once = Once::new();

fn ensure_sigpipe_reset() {
    INIT.call_once(|| {
        #[cfg(unix)]
        unsafe {
            libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        }
    });
}
```

### Pattern 5: With Error Handling

```rust
#[cfg(unix)]
use std::io;

#[cfg(unix)]
fn reset_sigpipe() -> io::Result<()> {
    unsafe {
        let result = libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        if result == libc::SIG_ERR {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

fn main() {
    if let Err(e) = reset_sigpipe() {
        eprintln!("Warning: Could not reset SIGPIPE: {}", e);
    }

    // Continue with application
}
```

---

## 7. Best Practices and Gotchas

### Best Practices

#### 1. Call Early in main()

Signal handlers should be set up before any other code runs that might write to stdout/stderr.

```rust
fn main() {
    // First thing: reset SIGPIPE
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    // Then initialize logging
    env_logger::init();

    // Then rest of application
}
```

#### 2. Use Platform-Specific Compilation

Always use `#[cfg(unix)]` to avoid compilation failures on Windows.

```rust
#[cfg(unix)]
fn setup_signal_handlers() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}
```

#### 3. Document the Behavior

```rust
/// Reset SIGPIPE handler to default behavior.
///
/// By default, Rust programs ignore SIGPIPE, which can cause unexpected
/// panics when writing to closed pipes. This function restores the Unix
/// default behavior of terminating the process with a SIGPIPE signal.
///
/// # Example
///
/// ```rust
/// # #[cfg(unix)]
/// # fn main() {
/// reset_sigpipe();
/// println!("Hello, world!");
/// # }
/// # #[cfg(not(unix))]
/// # fn main() {}
/// ```
#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}
```

#### 4. Consider Your Use Case

**For CLI applications**: Usually want SIG_DFL (default terminate behavior)

**For network servers**: Usually want SIG_IGN (ignore SIGPIPE, handle errors manually)

```rust
enum SignalBehavior {
    Default,   // Let process die on broken pipe (good for CLI tools)
    Ignore,    // Ignore signal, handle errors manually (good for servers)
}

#[cfg(unix)]
fn set_sigpipe_behavior(behavior: SignalBehavior) {
    use libc::{SIGPIPE, SIG_DFL, SIG_IGN};

    unsafe {
        let handler = match behavior {
            SignalBehavior::Default => SIG_DFL,
            SignalBehavior::Ignore => SIG_IGN,
        };
        libc::signal(SIGPIPE, handler);
    }
}
```

### Gotchas and Common Pitfalls

#### 1. Rust Ignores SIGPIPE by Default

**Problem**: The Rust runtime sets SIGPIPE to SIG_IGN before `main()` is called.

**Evidence**: From [rust-lang/rust#62569](https://github.com/rust-lang/rust/issues/62569):
> "Back in 2014, the Rust startup code (which runs before main) started ignoring SIGPIPE by default: #13158"

**Solution**: Explicitly reset to SIG_DFL if you want Unix-standard behavior.

#### 2. println! Panics on Broken Pipe with SIG_IGN

**Problem**: When SIGPIPE is ignored, writing to a closed pipe returns an error, which `println!` converts to a panic.

**Example**:
```bash
$ cargo run | head -n1
thread 'main' panicked at 'failed printing to stdout: Broken pipe (os error 32)', library/std/src/io/stdio.rs:1019:9
```

**Solution**: Reset SIGPIPE to SIG_DFL, or handle BrokenPipe errors explicitly.

```rust
use std::io::Write;

fn safe_println() -> std::io::Result<()> {
    let mut stdout = std::io::stdout();
    writeln!(stdout, "Hello")?;
    Ok(())
}
```

#### 3. Signal Handlers are Process-Wide

**Problem**: Signal handlers affect the entire process, not just your code.

**Gotcha**: If you're writing a library, changing signal handlers affects the host application.

**Solution**: Document clearly, or avoid changing signal handlers in libraries.

#### 4. Signal Dispositions are Inherited Across fork/exec

**Problem**: If your Rust code spawns child processes, they inherit the SIGPIPE disposition.

**From [rust-lang/rust#62569](https://github.com/rust-lang/rust/issues/62569)**:
> "Changing any signal disposition by default in the startup code is inherently broken, because dispositions are inherited across fork/exec."

**Solution**: Be aware of how signal state affects child processes.

#### 5. exit(141) is Not the Same as SIGPIPE

**Problem**: Some programs manually exit with code 141 (128 + 13) to simulate SIGPIPE, but this is not equivalent.

**Evidence**: From [rust-lang/rust#62569 comment](https://github.com/rust-lang/rust/issues/62569#issuecomment-974317506):
> "You can distinguish the two cases from the output of waitpid(2)... exit(141) terminates with a code and no signal, whereas being killed terminates with a signal and no code."

**Solution**: Use proper signal handling rather than fake exit codes.

#### 6. Signal Safety is Limited

**Problem**: You can only safely call async-signal-safe functions from signal handlers.

**Gotcha**: `libc::signal()` itself is not guaranteed to be safe in all contexts.

**Best Practice**: Set up signal handlers early, before any other threads or complex initialization.

#### 7. Thread Safety

**Problem**: Signal handlers are process-wide, but signal masks are thread-local on some systems.

**Gotcha**: The interaction between signals and threads can be complex.

**Best Practice**: Set signal handlers early in `main()`, before spawning threads.

---

## 8. Alternative Approaches

### Option 1: unix_sigpipe Attribute (Nightly Only)

**Tracking Issue**: [rust-lang/rust#97889](https://github.com/rust-lang/rust/issues/97889)

**Usage**:
```rust
#![feature(unix_sigpipe)]
#[unix_sigpipe = "sig_dfl"]
fn main() {
    // SIGPIPE is set to default behavior automatically
}
```

**Status**: Unstable feature, available in nightly Rust

### Option 2: Handle BrokenPipe Errors Explicitly

Instead of resetting SIGPIPE, handle errors manually:

```rust
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for i in 0..10000 {
        writeln!(handle, "Line {}", i)?;
    }

    Ok(())
}
```

### Option 3: Use signal-hook Crate

**Repository**: https://github.com/vorner/signal-hook

```rust
// Cargo.toml
[dependencies]
signal-hook = "0.3"

use signal_hook::consts::SIGPIPE;
use signal_hook::flag;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    let sigpipe_received = Arc::new(AtomicBool::new(false));
    flag::register(SIGPIPE, sigpipe_received.clone()).unwrap();

    // Check flag periodically
}
```

---

## 9. Real-World Examples

### Example 1: ripgrep

ripgrep uses the libc pattern (mentioned in StackOverflow answer by BurntSushi).

### Example 2: CLI Tool with Proper Error Handling

```rust
use std::io::{self, Write};
use std::process;

#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

fn run() -> io::Result<()> {
    for i in 0..10000 {
        println!("Line {}", i);
    }
    Ok(())
}

fn main() {
    #[cfg(unix)]
    reset_sigpipe();

    if let Err(e) = run() {
        if e.kind() == io::ErrorKind::BrokenPipe {
            // Exit gracefully with code that indicates SIGPIPE
            #[cfg(unix)]
            process::exit(128 + libc::SIGPIPE as i32);
            #[cfg(not(unix))]
            process::exit(0);
        } else {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}
```

---

## 10. Resources and References

### Official Documentation
- [libc crate docs.rs](https://docs.rs/libc)
- [libc GitHub repository](https://github.com/rust-lang/libc)

### Rust Issues and Discussions
- [Should Rust still ignore SIGPIPE by default? #62569](https://github.com/rust-lang/rust/issues/62569)
- [Tracking Issue for unix_sigpipe #97889](https://github.com/rust-lang/rust/issues/97889)
- [SIG_DFL and SIG_IGN on windows #1600](https://github.com/rust-lang/libc/issues/1600)

### Community Resources
- [Rust CLI Book - Signal handling](https://rust-cli.github.io/book/in-depth/signals.html)
- [Handling POSIX Signals in Rust](https://sigridjin.medium.com/handling-posix-signals-in-rust-fac42c33e5b6)
- [sigpipe crate](https://github.com/kurtbuilds/sigpipe)

### Background on SIGPIPE
- [Why does SIGPIPE exist?](https://stackoverflow.com/questions/8369506/why-does-sigpipe-exist)
- [Proper handling of SIGINT/SIGQUIT](http://www.cons.org/cracauer/sigint.html)

---

## Summary

The libc crate provides direct FFI bindings to C library functions for signal handling. For SIGPIPE handling:

1. **Use `libc::signal(SIGPIPE, SIG_DFL)`** to restore default Unix behavior
2. **Wrap in `unsafe` block** - required for all libc FFI calls
3. **Use `#[cfg(unix)]`** for platform-specific compilation
4. **Call early in `main()`** before any I/O operations
5. **Consider your use case**: CLI tools typically want SIG_DFL, servers typically want SIG_IGN
6. **Be aware**: Rust ignores SIGPIPE by default, which can cause unexpected panics

The simplest working pattern:

```rust
#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

fn main() {
    #[cfg(unix)]
    reset_sigpipe();

    // Your application code here
}
```
