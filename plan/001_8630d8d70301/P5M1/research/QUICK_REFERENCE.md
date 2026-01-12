# Rust Git Sync Patterns - Quick Reference

## Essential Code Snippets

### Basic Fetch with Progress

```rust
use git2::{Repository, RemoteCallbacks, FetchOptions};
use std::io::{self, Write};

fn fetch_from_remote(repo: &Repository, remote_name: &str) -> Result<(), git2::Error> {
    let mut cb = RemoteCallbacks::new();

    // Track download progress
    cb.transfer_progress(|stats| {
        print!("Received {}/{} objects\r",
            stats.received_objects(),
            stats.total_objects());
        io::stdout().flush().unwrap();
        true
    });

    // Track server messages
    cb.sideband_progress(|data| {
        print!("remote: {}", String::from_utf8_lossy(data));
        io::stdout().flush().unwrap();
        true
    });

    let mut remote = repo.find_remote(remote_name)?;
    let mut opts = FetchOptions::new();
    opts.remote_callbacks(cb);

    remote.download(&[] as &[&str], Some(&mut opts))?;
    remote.update_tips(None, Default::default(), Default::default(), None)?;

    Ok(())
}
```

### Authentication with SSH Agent

```rust
use git2::{RemoteCallbacks, Cred};

let mut callbacks = RemoteCallbacks::new();
callbacks.credentials(|_url, username_from_url, _allowed_types| {
    Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
});
```

### Push with Refspec

```rust
use git2::{Repository, RemoteCallbacks, PushOptions};

let mut push = repo.find_remote("origin")?;
let mut callbacks = RemoteCallbacks::new();

callbacks.push_update_reference(|refname, status| {
    match status {
        Some(s) => eprintln!("Failed to push {}: {}", refname, s),
        None => println!("Pushed {}", refname),
    }
    Ok(())
});

let mut opts = PushOptions::new();
opts.remote_callbacks(callbacks);

push.push(&["+refs/heads/main:refs/heads/main"], Some(&mut opts))?;
```

### Authentication with Fallback (Using auth-git2)

```rust
use auth_git2::GitAuthenticator;
use git2::RemoteCallbacks;

let auth = GitAuthenticator::default();
let git_config = git2::Config::open_default()?;

let mut callbacks = RemoteCallbacks::new();
callbacks.credentials(auth.credentials(&git_config));
```

### Error Handling with Retry

```rust
use git2::{Repository, ErrorClass};

fn fetch_with_retry(repo: &Repository, remote: &str) -> Result<(), Box<dyn std::error::Error>> {
    for attempt in 0..3 {
        match fetch_from_remote(repo, remote) {
            Ok(()) => return Ok(()),
            Err(e) => {
                match e.class() {
                    ErrorClass::Net | ErrorClass::Os => {
                        eprintln!("Attempt {} failed: {}. Retrying...", attempt + 1, e.message());
                        std::thread::sleep(std::time::Duration::from_secs(2_u64.pow(attempt as u32)));
                    }
                    _ => return Err(e.into()),
                }
            }
        }
    }
    Err("Max retries exceeded".into())
}
```

### Clone with Progress

```rust
use git2::build::RepoBuilder;
use git2::{FetchOptions, RemoteCallbacks};

let mut cb = RemoteCallbacks::new();
cb.transfer_progress(|stats| {
    println!("Received {}/{} objects",
        stats.received_objects(),
        stats.total_objects());
    true
});

let mut fo = FetchOptions::new();
fo.remote_callbacks(cb);

let mut builder = RepoBuilder::new();
builder.fetch_options(fo);

let repo = builder.clone(url, path)?;
```

### Testing with Local Bare Repository

```rust
#[cfg(test)]
mod tests {
    use git2::Repository;
    use tempfile::TempDir;

    #[test]
    fn test_fetch_from_local() {
        // Create bare repo to simulate remote
        let bare_dir = TempDir::new().unwrap();
        let _bare = Repository::init_bare(bare_dir.path()).unwrap();

        // Create working repo
        let work_dir = TempDir::new().unwrap();
        let repo = Repository::init(work_dir.path()).unwrap();

        // Add bare as remote and test
        let mut remote = repo.remote(
            "origin",
            bare_dir.path().to_str().unwrap()
        ).unwrap();

        // Test fetch/push operations
    }
}
```

## Error Codes Cheat Sheet

| Code | Meaning | Recovery |
|------|---------|----------|
| `EAUTH (-16)` | Authentication failed | Check credentials, try different auth method |
| `ECERTIFICATE (-17)` | SSL certificate invalid | Check certificate chain, update CA bundle |
| `EAGAIN (-12)` | Transient error | Retry operation |
| `EUNBORNBRANCH (-9)` | No commits on branch | Create initial commit first |
| `ENONFASTFORWARD (-11)` | Forced push needed | Use `+` prefix in refspec |

## Key RemoteCallbacks Methods

| Method | Purpose | Return |
|--------|---------|--------|
| `credentials()` | Fetch auth credentials | `Result<Cred, Error>` |
| `transfer_progress()` | Track download progress | `bool` (true=continue) |
| `sideband_progress()` | Server messages | `bool` (true=continue) |
| `update_tips()` | Reference updates | `bool` (true=continue) |
| `push_update_reference()` | Per-ref push status | `Result<(), Error>` |
| `certificate_check()` | SSL verification | `Result<CertificateCheckStatus, Error>` |

## Testing Pattern Selection

| Scenario | Pattern | Pros | Cons |
|----------|---------|------|------|
| Basic ops | Local bare repo | Realistic, no network | Slow, heavy setup |
| Auth testing | Mock credentials | Fast, isolated | Incomplete coverage |
| Integration | Trait mocking | Fast, testable | Needs wrapper trait |
| HTTPS testing | httpmock server | Realistic HTTP | Complex setup |

## Common Issues and Solutions

### "authentication required but no callback set"
```rust
// WRONG: Missing callback
let mut remote = repo.find_remote("origin")?;
remote.fetch(&[], None)?;  // Will fail

// RIGHT: Set credentials callback
let mut cb = RemoteCallbacks::new();
cb.credentials(|_url, user, _| {
    Cred::ssh_key_from_agent(user.unwrap_or("git"))
});
let mut opts = FetchOptions::new();
opts.remote_callbacks(cb);
remote.fetch(&[], Some(&mut opts))?;
```

### "infinite credential retry loop"
```rust
// Use auth-git2 or implement your own tracking:
let mut auth_attempts = 0;
callbacks.credentials(|_url, user, _| {
    auth_attempts += 1;
    if auth_attempts > 3 {
        return Err(git2::Error::from_str("Auth failed"));
    }
    Cred::ssh_key_from_agent(user.unwrap_or("git"))
});
```

### "push failed: non-fast-forward"
```rust
// Use + prefix to force push (if allowed)
push.push(&["+refs/heads/main:refs/heads/main"], Some(&mut opts))?;
```

## Crate Selection

| Need | Crate | Why |
|------|-------|-----|
| Git operations | `git2` | Mature, widely used, C-based |
| Auth simplification | `auth-git2` | Handles multiple methods, fallbacks |
| Credentials | `git2_credentials` | Priority-based, built-in helpers |
| Testing | `tempfile` | Temporary files/dirs, auto-cleanup |
| Test fixtures | `rstest` | Parameter injection, fixture reuse |
| Pure Rust future | `gitoxide` | No C deps, growing alternative |

## Refspec Syntax

| Pattern | Meaning |
|---------|---------|
| `refs/heads/main:refs/remotes/origin/main` | Fetch specific branch |
| `+refs/heads/*:refs/remotes/origin/*` | Wildcard (note: limitations exist) |
| `+refs/heads/main:refs/heads/main` | Push with force prefix |
| Empty/None | Use configured refspecs |

## Progress Tracking Pattern

```rust
struct ProgressState {
    total_objects: usize,
    received_objects: usize,
    total_deltas: usize,
    indexed_deltas: usize,
}

impl ProgressState {
    fn fetch_progress(&mut self, stats: &git2::Progress) -> bool {
        self.total_objects = stats.total_objects();
        self.received_objects = stats.received_objects();

        let percent = if self.total_objects > 0 {
            (self.received_objects * 100) / self.total_objects
        } else {
            0
        };

        print!("Fetch: {}%\r", percent);
        io::stdout().flush().unwrap();
        true // Continue
    }

    fn delta_progress(&mut self, stats: &git2::Progress) -> bool {
        self.total_deltas = stats.total_deltas();
        self.indexed_deltas = stats.indexed_deltas();

        let percent = if self.total_deltas > 0 {
            (self.indexed_deltas * 100) / self.total_deltas
        } else {
            0
        };

        print!("Deltas: {}%\r", percent);
        io::stdout().flush().unwrap();
        true // Continue
    }
}
```

## Key Design Principles for Jin

1. **Abstraction First**: Use traits to wrap git2 for testability
2. **Authentication Priority**: SSH agent > SSH files > credential helper
3. **Error Context**: Wrap errors with operation context
4. **Progress Normalization**: Track all phases (fetch, checkout) as single 0-100% range
5. **Local Testing**: Prefer local bare repos over mocks for integration tests
6. **Retry Strategy**: Exponential backoff with max attempts, respect timeout
7. **Network Awareness**: Distinguish network errors from authentication/git errors
8. **Future-Proofing**: Plan gitoxide migration path

## Document References

Full documentation available in: `/home/dustin/projects/jin/plan/P5M1/research/rust_git_sync_examples.md`

- Section 1: Complete working examples
- Section 2: Cargo and gitoxide patterns
- Section 3: Authentication detailed patterns
- Section 4: Progress callback details
- Section 5: Error handling deep dive
- Section 6: Testing strategies
- Section 7: Mock/stub patterns
- Section 8: Crate comparisons
- Section 10: Implementation recommendations

GitHub References:
- [git2-rs](https://github.com/rust-lang/git2-rs) - Official bindings
- [gitoxide](https://github.com/GitoxideLabs/gitoxide) - Pure Rust alternative
- [Cargo git utils](https://github.com/rust-lang/cargo/blob/master/src/cargo/sources/git/utils.rs) - Production patterns
