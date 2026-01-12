# Git Remote Management - Quick Reference for `jin link`

## API Quick Lookup

### Add/Update Remote
```rust
// Add with default refspec (branches)
repo.remote("name", "https://...")?;

// Add with custom refspec (FOR JIN)
repo.remote_with_fetch("jin-remote", "https://...", "+refs/jin/layers/*:refs/jin/layers/*")?;

// Replace existing
repo.remote_delete("jin-remote")?;
repo.remote("jin-remote", "https://...")?;

// Change URL only
repo.remote_set_url("jin-remote", "new-url")?;
```

### Find/List Remote
```rust
// Get existing remote
let remote = repo.find_remote("jin-remote")?;

// List all remote names
for name in repo.remotes()? {
    println!("{}", name);
}

// Check existence
let exists = repo.find_remote("jin-remote").is_ok();
```

### Validate Connection
```rust
let mut remote = repo.remote_anonymous(url)?;
match remote.connect(git2::Direction::Fetch) {
    Ok(_) => println!("✓ Connected"),
    Err(e) => {
        eprintln!("✗ Cannot reach: {}", e);
        // Error codes: Net, NotFound, Auth
    }
}
remote.disconnect()?;
```

### Get Remote Info
```rust
let remote = repo.find_remote("jin-remote")?;
println!("Name: {}", remote.name().unwrap_or("(anonymous)"));
println!("URL: {}", remote.url().unwrap_or("(invalid UTF-8)"));
```

---

## URL Validation

### Valid Formats
```
✅ https://github.com/org/repo.git
✅ https://github.com/org/repo
✅ git@github.com:org/repo.git
✅ ssh://git@github.com/org/repo.git
✅ git://server.local/repo.git
✅ file:///absolute/path/to/repo
✅ /absolute/path/to/repo
```

### Invalid Formats
```
❌ http://[no-host]
❌ random-string
❌ ftp://unsupported.protocol
❌ (empty string)
```

### Validation Function
```rust
fn is_valid_git_url(url: &str) -> bool {
    if url.is_empty() { return false; }

    // Check protocol
    let valid_start = url.starts_with("https://") ||
                      url.starts_with("http://") ||
                      url.starts_with("git@") ||
                      url.starts_with("ssh://") ||
                      url.starts_with("git://") ||
                      url.starts_with("file://") ||
                      url.starts_with("/");

    valid_start && url.len() > 3  // At least 'minimal' URL
}
```

---

## Error Handling

### Error Code Mapping
```rust
match error.code() {
    git2::ErrorCode::Exists => {
        // Remote already exists
        // Solution: Use --force to replace
    }
    git2::ErrorCode::NotFound => {
        // Remote doesn't exist (when looking it up)
        // OR repository not found (when connecting)
        // Solution: Check repository exists and you can access it
    }
    git2::ErrorCode::Auth => {
        // Authentication failed
        // Solution: Check SSH keys or credentials
    }
    git2::ErrorCode::Net => {
        // Network error
        // Solution: Check connectivity, firewall, DNS
    }
    _ => {
        // Other errors
    }
}
```

### Error Messages for Users

| Error | Message | Recovery |
|-------|---------|----------|
| AlreadyExists | "Remote 'jin-remote' already exists. Use --force to replace." | Add `--force` flag |
| InvalidUrl | "Invalid URL format. Supported: https://, ssh, git, file://" | Use valid URL format |
| NotFound | "Repository not found or not accessible. Check URL and permissions." | Verify URL is correct |
| Auth | "Authentication failed. Check SSH keys or credentials." | Set up SSH keys or HTTPS token |
| Net | "Cannot reach remote. Check network connectivity." | Check internet connection |

---

## Configuration Storage

### In Git Config File (`.jin/config`)
```ini
[remote "jin-remote"]
    url = https://github.com/org/config.git
    fetch = +refs/jin/layers/*:refs/jin/layers/*
```

### In JinConfig (`~/.jin/config.toml`)
```toml
[remote]
url = "https://github.com/org/config.git"
fetch_on_init = true
```

### Verification
```rust
// Verify in git config
let remote = repo.find_remote("jin-remote")?;
assert_eq!(remote.url(), Some("https://..."));

// Verify in JinConfig
let config = JinConfig::load()?;
assert_eq!(config.remote.as_ref().map(|r| &r.url), Some(&url));
```

---

## Implementation Checklist

### Before Adding Remote
- [ ] Validate URL format with `is_valid_git_url()`
- [ ] Check if remote already exists (unless --force)
- [ ] Test connection with anonymous remote
- [ ] Get user confirmation for --force if replacing

### When Adding Remote
- [ ] Call `repo.remote_with_fetch()` with custom refspec
- [ ] Use refspec: `"+refs/jin/layers/*:refs/jin/layers/*"`
- [ ] Persist to JinConfig
- [ ] Verify both config and JinConfig saved

### After Adding Remote
- [ ] Print success message: "✓ Linked to remote: <url>"
- [ ] Show next steps: "Run 'jin fetch' to sync layers"
- [ ] If --fetch-on-init: Save that preference

---

## Common Scenarios

### Scenario 1: First Time Linking
```bash
$ jin link https://github.com/org/jin-config.git
✓ Linked to remote: https://github.com/org/jin-config.git
Run 'jin fetch' to sync your layer configuration
```

### Scenario 2: Update Existing Remote
```bash
$ jin link https://github.com/neworg/config.git --force
✓ Remote updated to: https://github.com/neworg/config.git
```

### Scenario 3: Invalid URL
```bash
$ jin link invalid-url
✗ Invalid URL format
Supported formats:
  - https://github.com/org/repo.git
  - git@github.com:org/repo.git
  - ssh://git@server/path/repo.git
  - file:///absolute/path
```

### Scenario 4: Unreachable Remote
```bash
$ jin link https://github.com/org/nonexistent.git
✗ Cannot reach remote
Check:
  - Repository exists and is public/accessible
  - URL is correct
  - Network connectivity is working
```

---

## Key Design Decisions for Jin

### Why Custom Refspec?
- **Default**: `+refs/heads/*:refs/remotes/jin-remote/*` - fetches all branches
- **Jin**: `+refs/jin/layers/*:refs/jin/layers/*` - fetches only layer refs
- **Benefit**: Minimal bandwidth, no branch clutter, clean namespace

### Why Test Before Persisting?
- Validate URL reachability before saving config
- Fail early with clear error message
- Prevent corrupt configuration state

### Why Store in Both Places?
- **git config**: Git tools can introspect Jin repo
- **JinConfig**: Jin can manage fetch_on_init preference
- **Not duplication**: complementary storage

---

## Testing Patterns

### Unit Test: URL Validation
```rust
#[test]
fn test_valid_urls() {
    assert!(is_valid_git_url("https://github.com/org/repo"));
    assert!(is_valid_git_url("git@github.com:org/repo.git"));
    assert!(is_valid_git_url("/absolute/path"));
}

#[test]
fn test_invalid_urls() {
    assert!(!is_valid_git_url(""));
    assert!(!is_valid_git_url("invalid"));
}
```

### Integration Test: Link Remote
```rust
#[test]
fn test_link_remote() {
    let repo = temp_jin_repo();
    link_remote(&repo, "jin-remote", "https://...").unwrap();

    // Verify in config
    let remote = repo.find_remote("jin-remote").unwrap();
    assert_eq!(remote.url(), Some("https://..."));
}
```

### Integration Test: Error Handling
```rust
#[test]
fn test_link_invalid_url() {
    let repo = temp_jin_repo();
    let result = link_remote(&repo, "jin-remote", "invalid");
    assert!(result.is_err());
}
```

---

## Documentation References

| Topic | URL |
|-------|-----|
| repo.remote() | https://docs.rs/git2/latest/git2/struct.Repository.html#method.remote |
| repo.remote_with_fetch() | https://docs.rs/git2/latest/git2/struct.Repository.html#method.remote_with_fetch |
| repo.find_remote() | https://docs.rs/git2/latest/git2/struct.Repository.html#method.find_remote |
| Remote::connect() | https://docs.rs/git2/latest/git2/struct.Remote.html#method.connect |
| Remote::url() | https://docs.rs/git2/latest/git2/struct.Remote.html#method.url |
| Git Protocols | https://git-scm.com/book/en/v2/Git-on-the-Server-The-Protocols |
| Git Refspecs | https://git-scm.com/book/en/v2/Git-Internals-The-Refspec |

