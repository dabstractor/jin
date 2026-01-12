# libc::signal() Function Research for SIGPIPE Handling in Rust

This document provides comprehensive research on the `libc::signal()` function for SIGPIPE handling in Rust.

## Table of Contents
1. [Function Signature](#function-signature)
2. [Return Value](#return-value)
3. [Safety Requirements](#safety-requirements)
4. [SIG_DFL Constant](#sig_dfl-constant)
5. [Platform-Specific Behavior](#platform-specific-behavior)
6. [Code Examples](#code-examples)
7. [References](#references)

---

## Function Signature

### C Function Signature
```c
typedef void (*sighandler_t)(int);
sighandler_t signal(int signum, sighandler_t handler);
```

### Rust/libc Signature
```rust
pub unsafe extern "C" fn signal(
    signum: c_int,
    handler: sighandler_t
) -> sighandler_t
```

### Parameters
- **`signum`**: The signal number (e.g., `libc::SIGPIPE` = 13 on most Unix systems)
- **`handler`**: A function pointer to the signal handler, which can be:
  - A custom handler function: `extern "C" fn(i32)`
  - `libc::SIG_DFL`: Default signal handling behavior (value: 0)
  - `libc::SIG_IGN`: Ignore the signal (value: 1)

### Type Definition
The `sighandler_t` type is a function pointer:
```c
typedef void (*sighandler_t)(int);
```

This is a GNU extension exposed when `_GNU_SOURCE` is defined. glibc also defines the BSD-derived `sig_t` when `_BSD_SOURCE` is defined (glibc 2.19 and earlier).

**Sources:**
- [signal(2) - Linux manual page](https://man7.org/linux/man-pages/man2/signal.2.html)
- [FreeBSD man pages](https://man.freebsd.org/cgi/man.cgi?query=signal&sektion=2&manpath=Red+Hat+8.0)
- [libc - Rust documentation](https://docs.rs/libc/latest/libc/fn.signal.html)

---

## Return Value

### What It Returns
The `libc::signal()` function returns the **previous signal handler** that was in place before the call.

### Return Type
```rust
sighandler_t  // Equivalent to: Option<unsafe extern "C" fn(i32)>
```

### Possible Return Values
1. **Previous handler function pointer**: If a custom handler was installed before
2. **`SIG_DFL`**: If the signal was using default handling
3. **`SIG_IGN`**: If the signal was being ignored
4. **`SIG_ERR`**: If an error occurred (typically cast as `(sighandler_t)-1`)

### Should We Check It?

**Yes, checking the return value is recommended** because:

1. **Error Detection**: `SIG_ERR` indicates the call failed (e.g., invalid signal number)
2. **State Restoration**: Important if you need to restore the previous handler later
3. **Debugging**: Helps verify that signal handling was set up correctly

### Example Return Value Check
```rust
use libc::{signal, SIGPIPE, SIG_DFL, SIG_ERR};

unsafe {
    let old_handler = signal(SIGPIPE, SIG_DFL);
    if old_handler == SIG_ERR {
        eprintln!("Failed to set SIGPIPE handler");
        std::process::exit(1);
    }
    // old_handler now contains the previous handler for restoration if needed
}
```

**Sources:**
- [signal(2) - Linux manual page](https://man7.org/linux/man-pages/man2/signal.2.html)
- [libc::signal function documentation](https://docs.rs/libc/latest/libc/fn.signal.html)

---

## Safety Requirements

### Why `libc::signal()` is Unsafe

The `libc::signal()` function is marked as `unsafe` in Rust for several critical reasons:

#### 1. **Asynchronous Execution**
Signal handlers can be called at any time, interrupting the program's execution:
- Can run between any two CPU instructions
- Can interrupt atomic operations
- May execute while holding locks or in the middle of unsafe operations

#### 2. **Restricted Operations in Signal Handlers**
Signal handlers have severe limitations:
- **Cannot allocate memory** (heap allocation is not async-signal-safe)
- **Cannot acquire mutexes** (can cause deadlocks)
- **Limited set of safe functions** (only async-signal-safe functions)
- **Cannot touch Rust's standard library** (most of it is not async-signal-safe)

#### 3. **Memory Safety Concerns**
- Signal handlers can corrupt program state if they modify shared data
- Rust's ownership system cannot guarantee safety across asynchronous interruptions
- Potential for race conditions and undefined behavior

#### 4. **Global State Modification**
- Signals are process-wide, affecting all threads
- Multiple threads setting handlers simultaneously can cause race conditions

### Safety Guarreements Required

When using `libc::signal()`, you must guarantee:

1. **The signal handler is async-signal-safe**
   ```rust
   // Correct: Minimal, async-signal-safe handler
   extern "C" fn handle_sigpipe(_signum: c_int) {
       // Only set a flag or call async-signal-safe functions
       // No allocations, no locks, no standard library
   }
   ```

2. **No data races with global state**
   - Any global state modified must use atomic operations
   - Consider using `std::sync::atomic` types

3. **Proper signal mask management**
   - Ensure appropriate signals are blocked during critical sections
   - Use `sigprocmask()` when needed

4. **Reentrancy considerations**
   - The handler must be reentrant-safe
   - Cannot rely on global state that may be inconsistent

### Rust's Approach to Signal Safety

Rust provides safer alternatives:

1. **`#[unix_sigpipe = "sig_dfl"]` attribute** (Rust 1.73+)
   ```rust
   #![unix_sigpipe = "sig_dfl"]  // Restore default SIGPIPE behavior
   ```

2. **Higher-level crates**:
   - `signal-hook`: Safe wrapper for signal handling
   - `nix`: Rust-friendly Unix signal API
   - `rustix`: Safer FFI for Unix systems

### FFI Safety Best Practices

From 2025 research on Rust FFI safety:

1. **Use safe wrapper crates** when available (e.g., `nix`, `rustix`)
2. **Minimize unsafe code** scope
3. **Document safety invariants** clearly
4. **Consider async-signal-safety** at all times
5. **Prefer `sigaction()` over `signal()`** for portable code

**Sources:**
- [Handling POSIX Signals in Rust](https://sigridjin.medium.com/handling-posix-signals-in-rust-fac42c33e5b6)
- [Unix Signal Handling Best Practices (Rust Forum)](https://users.rust-lang.org/t/what-are-the-best-practices-for-unix-signal-handling/114206)
- [SafeFFI Research Paper (2025)](https://arxiv.org/html/2510.20688v1)
- [Rust Unsafe Code Guidelines](https://github.com/rust-lang/unsafe-code-guidelines/issues/428)
- [How to use libc's signal function? (Rust Forum)](https://users.rust-lang.org/t/how-to-use-libcs-signal-function/3067)

---

## SIG_DFL Constant

### Definition

**SIG_DFL** (Signal Default) is a special constant used to set a signal's disposition to its default behavior.

### Value and Type
```c
#define SIG_DFL ((sighandler_t)0)  // Value: 0
```

### Behavior

When a signal is set to `SIG_DFL`:
- The system performs the **default action** for that signal
- Default actions vary by signal type:
  - **Terminate the process** (most common, e.g., SIGPIPE)
  - **Ignore the signal** (e.g., SIGCHLD)
  - **Stop the process** (e.g., SIGSTOP)
  - **Dump core** (e.g., SIGSEGV, SIGABRT)

### SIG_DFL vs SIG_IGN

| Aspect | SIG_DFL | SIG_IGN |
|--------|---------|---------|
| **Value** | 0 | 1 |
| **Purpose** | Default system behavior | Explicitly ignore the signal |
| **SIGPIPE behavior** | Terminates process immediately | Signal is discarded |
| **Use case** | Restore default handling | Prevent signal interruption |
| **Effect on writes** | Process dies on broken pipe | Write fails with EPIPE error |

### SIGPIPE with SIG_DFL

When SIGPIPE is set to `SIG_DFL` (the traditional Unix default):
```rust
use libc::{SIGPIPE, SIG_DFL, signal};

unsafe {
    signal(SIGPIPE, SIG_DFL);
}

// Result:
// - Writing to a broken pipe causes SIGPIPE to be sent
// - Default action: process terminates immediately
// - Exit status: 141 (128 + 13)
// - This mimics traditional Unix/C behavior
```

### SIGPIPE with SIG_IGN

When SIGPIPE is set to `SIG_IGN` (Rust's default):
```rust
use libc::{SIGPIPE, SIG_IGN, signal};

unsafe {
    signal(SIGPIPE, SIG_IGN);
}

// Result:
// - Writing to a broken pipe does NOT send SIGPIPE
// - Instead, write() fails with EPIPE error
// - Process continues and can handle the error gracefully
// - Better for error handling and cross-platform compatibility
```

### Which to Use?

**Use SIG_DFL when:**
- You want traditional Unix behavior
- You want the process to terminate on broken pipe
- Writing to other processes (e.g., pipes, sockets)
- Building Unix tools that should behave like C equivalents

**Use SIG_IGN when:**
- You want custom error handling
- Building cross-platform applications (Windows has no SIGPIPE)
- Need graceful shutdown on errors
- Rust's default behavior is preferred

**Sources:**
- [signal(2) - Linux manual page](https://man7.org/linux/man-pages/man2/signal.2.html)
- [Basic Signal Handling (GNU C Library)](https://www.gnu.org/s/libc/manual/html_node/Basic-Signal-Handling.html)
- [Should Rust still ignore SIGPIPE by default?](https://github.com/rust-lang/rust/issues/62569)
- [Tracking Issue for unix_sigpipe #97889](https://github.com/rust-lang/rust/issues/97889)

---

## Platform-Specific Behavior

### SIGPIPE Across Unix-like Systems

#### Linux
- **SIGPIPE value**: 13
- **Default behavior**: Terminate process
- **Socket option**: `MSG_NOSIGNAL` flag for `send()` calls
- **Rust default**: Ignores SIGPIPE
- **Portable approach**: Use `MSG_NOSIGNAL` or set `SIG_IGN`

```rust
// Linux-specific: MSG_NOSIGNAL flag
use libc::{send, MSG_NOSIGNAL};

let flags = MSG_NOSIGNAL;  // Prevent SIGPIPE for this send only
```

#### macOS / BSD (Darwin, FreeBSD, OpenBSD, NetBSD)
- **SIGPIPE value**: 13
- **Default behavior**: Terminate process
- **Socket option**: `SO_NOSIGPIPE` socket option (setsockopt)
- **Behavior**: Similar to Linux but with different socket API
- **Note**: macOS pipes are often implemented as Unix domain sockets

```rust
// macOS/BSD-specific: SO_NOSIGPIPE socket option
use libc::{setsockopt, SOL_SOCKET, SO_NOSIGPIPE};

let opt: c_int = 1;
setsockopt(sockfd, SOL_SOCKET, SO_NOSIGPIPE, &opt as *const _ as *const _, std::mem::size_of::<c_int>() as socklen_t);
```

#### Solaris / illumos
- **SIGPIPE value**: 13
- **Default behavior**: Terminate process
- **Notes**: Generally follows POSIX standard

#### Windows
- **SIGPIPE**: Does not exist
- **Alternative**: Broken pipe returns error (similar to `SIG_IGN` behavior)
- **Implication**: Cross-platform code must check write errors

```rust
// Cross-platform approach
match write_result {
    Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => {
        // Handle broken pipe
    }
    // ... other errors
}
```

### Platform Differences Summary

| Platform | SIGPIPE Value | Socket Option | Default | Rust Default |
|----------|---------------|---------------|---------|--------------|
| Linux | 13 | `MSG_NOSIGNAL` | Terminate | Ignore |
| macOS | 13 | `SO_NOSIGPIPE` | Terminate | Ignore |
| BSD | 13 | `SO_NOSIGPIPE` | Terminate | Ignore |
| Windows | N/A | N/A | N/A | N/A |

### Behavioral Differences

#### Signal Handler Reset Behavior
Different systems behave differently after a signal handler is invoked:

**BSD/macOS behavior**:
- Signal handlers are reset to `SIG_DFL` after invocation
- Can cause unexpected process termination

**System V/Linux behavior**:
- Signal handlers persist after invocation
- More predictable behavior

**Recommendation**: Use `sigaction()` instead of `signal()` for portable code, as it provides consistent behavior.

#### Signal Availability
Different platforms support different numbers of signals:
- **FreeBSD 8.0**: 32 signals
- **macOS 10.6.8**: 31 signals
- **Linux 3.2.0**: 31 signals
- **Solaris 10**: 40 signals

### Portability Considerations

1. **Always check return values**: Detect platform-specific failures
2. **Use `sigaction()` for new code**: More portable than `signal()`
3. **Consider using crates**: `nix`, `rustix`, or `signal-hook` handle platform differences
4. **Test on target platforms**: Behavior can vary significantly
5. **Use Rust's `#[unix_sigpipe]` attribute**: For compile-time control (Rust 1.73+)

**Sources:**
- [How to prevent SIGPIPEs (or handle them properly) - Stack Overflow](https://stackoverflow.com/questions/108183/how-to-prevent-sigpipes-or-handle-them-properly)
- [Why SIGPIPE behaves differently in different kernels - Unix StackExchange](https://unix.stackexchange.com/questions/638579/why-sigpipe-behaves-different-in-different-kernels)
- [SIGPIPE just prevented in OSX/BSD - libuv GitHub Issue](https://github.com/joyent/libuv/issues/1254)
- [Signals and the Deep Declarations Between macOS and Linux - Medium](https://medium.com/macos-is-not-linux-and-other-nix-reflections/signals-and-the-deep-declarations-between-macos-and-linux-37230c06d422)
- [Chapter 10. Signals - Shichao's Notes](https://notes.shichao.io/apue/ch10/)
- [FreeBSD Bug Report #224270](https://bugs.freebsd.org/224270)
- [IPC Buffer Sizes - netmeister.org](https://www.netmeister.org/blog/ipcbufs.html)

---

## Code Examples

### Example 1: Basic SIGPIPE Restoration with libc

```rust
use libc::{signal, SIGPIPE, SIG_DFL, SIG_ERR};
use std::process;

fn main() {
    // Restore default SIGPIPE behavior (traditional Unix behavior)
    unsafe {
        let old_handler = signal(SIGPIPE, SIG_DFL);
        if old_handler == SIG_ERR {
            eprintln!("Failed to set SIGPIPE handler");
            process::exit(1);
        }
        // Optionally restore old handler later:
        // signal(SIGPIPE, old_handler);
    }

    // Now writes to broken pipe will terminate with SIGPIPE
    println!("SIGPIPE set to default behavior");
}
```

### Example 2: Using Rust's unix_sigpipe Attribute (Rust 1.73+)

```rust
#![unix_sigpipe = "sig_dfl"]  // Set at crate level

fn main() {
    // Default SIGPIPE behavior is restored for the entire program
    // No need for unsafe libc calls
    println!("SIGPIPE will terminate on broken pipe");
}
```

### Example 3: Using the nix Crate (Recommended)

```rust
use nix::sys::signal::{self, SigHandler, Signal};
use std::process;

fn main() {
    // Safer, more idiomatic Rust approach
    match unsafe { signal::signal(Signal::SIGPIPE, SigHandler::SigDfl) } {
        Ok(SigHandler::SigDfl) => {
            println!("SIGPIPE was already set to default");
        }
        Ok(SigHandler::SigIgn) => {
            println!("SIGPIPE was being ignored, now set to default");
        }
        Ok(SigHandler::Handler(_)) => {
            println!("SIGPIPE had a custom handler, now set to default");
        }
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to set SIGPIPE handler: {}", e);
            process::exit(1);
        }
    }

    println!("SIGPIPE configured successfully");
}
```

### Example 4: Cross-Platform Error Handling Approach

```rust
use std::io::{self, Write};

fn write_data(data: &[u8]) -> io::Result<()> {
    match std::io::stdout().write_all(data) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
            // Handle broken pipe gracefully
            eprintln!("Broken pipe: downstream reader closed");
            std::process::exit(0);  // Exit gracefully like traditional Unix tools
        }
        Err(e) => Err(e),
    }
}

fn main() {
    // This works on all platforms:
    // - Unix with SIGPIPE: Broken pipe causes write error (SIGPIPE ignored)
    // - Windows: Broken pipe causes write error (no SIGPIPE)
    // - Unix with SIG_DFL: Process terminates before error handling

    if let Err(e) = write_data(b"Hello, world!\n") {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

### Example 5: Temporary SIG_DFL with Scope Guard

```rust
use libc::{signal, SIGPIPE, SIG_DFL, sighandler_t};

struct SigpipeGuard {
    old_handler: sighandler_t,
}

impl SigpipeGuard {
    fn new() -> Option<Self> {
        unsafe {
            let old_handler = signal(SIGPIPE, SIG_DFL);
            if old_handler == libc::SIG_ERR {
                None
            } else {
                Some(Self { old_handler })
            }
        }
    }
}

impl Drop for SigpipeGuard {
    fn drop(&mut self) {
        unsafe {
            signal(SIGPIPE, self.old_handler);
        }
    }
}

fn perform_operation_with_default_sigpipe() {
    let _guard = SigpipeGuard::new();
    // During this scope, SIGPIPE will terminate on broken pipe
    // When guard goes out of scope, old handler is restored
}
```

### Example 6: Ripgrep-style Approach (Check Errors, Ignore SIGPIPE)

```rust
use std::io::{self, Write};

// This is similar to how ripgrep handles SIGPIPE
// Rust ignores SIGPIPE by default, so we handle broken pipes manually

fn search_and_print() -> io::Result<()> {
    // Simulate streaming output
    for i in 0..1000 {
        writeln!(std::io::stdout(), "Result {}", i)?;

        // Flush to ensure immediate detection of broken pipe
        std::io::stdout().flush()?;

        // When piped to `head -n 10`, this will:
        // 1. Write succeeds for first 10 lines
        // 2. Write fails with BrokenPipe on 11th line
        // 3. We detect and exit gracefully (exit code 0)
        // 4. Mimics traditional Unix tool behavior
    }
    Ok(())
}

fn main() {
    match search_and_print() {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
            // Exit gracefully on broken pipe (like traditional Unix tools)
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
```

**Sources:**
- [ripgrep Issue #200: doesn't stop when pipe is closed](https://github.com/BurntSushi/ripgrep/issues/200)
- [ripgrep Issue #2939: SIGPIPE can race with exit code](https://github.com/BurntSushi/ripgrep/issues/2939)
- [How to use libc's signal function? (Rust Forum)](https://users.rust-lang.org/t/how-to-use-libcs-signal-function/3067)
- [libc crate documentation](https://docs.rs/libc)

---

## References

### Official Documentation
1. **[libc crate - docs.rs](https://docs.rs/libc/latest/libc/fn.signal.html)** - Official libc::signal() function documentation
2. **[signal(2) - Linux man page](https://man7.org/linux/man-pages/man2/signal.2.html)** - Complete Linux signal() documentation
3. **[Basic Signal Handling - GNU C Library](https://www.gnu.org/s/libc/manual/html_node/Basic-Signal-Handling.html)** - GNU libc signal handling guide

### Rust-Specific Resources
4. **[Handling POSIX Signals in Rust - Sigrid Jin](https://sigridjin.medium.com/handling-posix-signals-in-rust-fac42c33e5b6)** - Comprehensive Rust signal handling guide
5. **[Should Rust still ignore SIGPIPE by default? - Rust Issue #62569](https://github.com/rust-lang/rust/issues/62569)** - Discussion on Rust's SIGPIPE policy
6. **[Tracking Issue for unix_sigpipe #97889](https://github.com/rust-lang/rust/issues/97889)** - Rust 1.73+ unix_sigpipe attribute tracking
7. **[How to use libc's signal function? - Rust Forum](https://users.rust-lang.org/t/how-to-use-libcs-signal-function/3067)** - Community discussion on signal handler creation

### Safety and FFI Research
8. **[SafeFFI: Efficient Sanitization at the Boundary (2025)](https://arxiv.org/html/2510.20688v1)** - Academic paper on FFI safety
9. **[What Is Rust's Unsafe? (2019)](https://news.ycombinator.com/item?id=30979484)** - Discussion on unsafe code and signal handling
10. **[Unix Signal Handling Best Practices - Rust Forum](https://users.rust-lang.org/t/what-are-the-best-practices-for-unix-signal-handling/114206)** - Community best practices
11. **[Rust Unsafe Code Guidelines - IO Safety](https://github.com/rust-lang/unsafe-code-guidelines/issues/428)** - Official unsafe code guidelines

### Platform-Specific Documentation
12. **[How to prevent SIGPIPEs - Stack Overflow](https://stackoverflow.com/questions/108183/how-to-prevent-sigpipes-or-handle-them-properly)** - Portable SIGPIPE handling
13. **[SIGPIPE on macOS/BSD - libuv Issue #1254](https://github.com/joyent/libuv/issues/1254)** - macOS/BSD-specific considerations
14. **[Signals: macOS vs Linux - Medium](https://medium.com/macos-is-not-linux-and-other-nix-reflections/signals-and-the-deep-declarations-between-macos-and-linux-37230c06d422)** - Platform comparison

### Real-World Examples
15. **[ripgrep Issue #200 - SIGPIPE handling](https://github.com/BurntSushi/ripgrep/issues/200)** - How ripgrep handles broken pipes
16. **[sigpipe crate - lib.rs](https://lib.rs/crates/sigpipe)** - Dedicated SIGPIPE handling crate

### Alternative Crates
17. **[nix crate - signal module](https://docs.rs/nix/latest/nix/sys/signal/fn.signal.html)** - Safe Rust signal handling wrapper
18. **[rustix crate](https://docs.rs/rustix/latest/x86_64-unknown-illumos/rustix/process/struct.Signal.html)** - Safe Unix system call wrapper
19. **[signal-hook crate](https://docs.rs/signal_hook)** - Signal handling with channel-based approach

---

## Summary

### Key Takeaways

1. **Function Signature**: `libc::signal(c_int, sighandler_t) -> sighandler_t`
   - Takes signal number and handler function/SIG_DFL/SIG_IGN
   - Returns previous handler (check for SIG_ERR)

2. **Safety**: Signal handling is inherently unsafe due to:
   - Asynchronous execution
   - Restricted operations in handlers
   - Global state modification risks
   - Use safe wrappers (nix, rustix) when possible

3. **SIG_DFL vs SIG_IGN**:
   - SIG_DFL (value 0): Default behavior, terminates on SIGPIPE
   - SIG_IGN (value 1): Ignore signal, write fails with EPIPE
   - Choose based on desired behavior (Unix traditional vs Rust default)

4. **Platform Differences**:
   - Linux: Use MSG_NOSIGNAL flag
   - macOS/BSD: Use SO_NOSIGPIPE option
   - Windows: No SIGPIPE, handle errors manually
   - Prefer sigaction() over signal() for portability

5. **Recommended Approach**:
   - Use `#![unix_sigpipe = "sig_dfl"]` for Rust 1.73+
   - Or use `nix` crate for safer API
   - Or check write errors manually (ripgrep approach)
   - Always test on target platforms

---

**Document created**: 2026-01-10
**Last updated**: 2026-01-10
**Research scope**: libc::signal() for SIGPIPE handling in Rust
