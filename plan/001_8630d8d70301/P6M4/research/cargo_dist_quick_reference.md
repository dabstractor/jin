# cargo-dist Quick Reference & Cheat Sheet

## Quick Start (5 minutes)

```bash
# 1. Install cargo-dist
cargo install cargo-dist --locked

# 2. Initialize
cargo dist init
# Answer: GitHub CI, shell installer, current platform targets

# 3. Test locally
cargo dist plan

# 4. Create release
git add .
git commit -m "chore: setup cargo-dist"
git tag v0.1.0
git push --tags
# CI automatically handles the rest!
```

---

## Essential Configuration

### Minimal Cargo.toml Setup

```toml
[package]
name = "my-tool"
version = "0.1.0"
repository = "https://github.com/user/my-tool"
license = "MIT"  # or "MIT OR Apache-2.0"
readme = "README.md"

[workspace.metadata.dist]
cargo-dist-version = "0.30.2"
ci = ["github"]
installers = ["shell", "powershell"]
targets = [
    "x86_64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
```

### Advanced dist-workspace.toml

```toml
[workspace]
members = ["cargo:my-package"]

[dist]
cargo-dist-version = "0.30.2"
ci = ["github"]
installers = ["shell", "powershell", "msi", "homebrew"]
targets = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc",
]

# Testing on PRs
pr-run-mode = "plan"  # or "upload"

# Allow generation in CI workflow
allow-dirty = ["ci"]

# Security & Supply Chain
github-attestations-phase = "announce"
github-attestations-filters = ["*.tar.gz", "*.zip", "*.msi", "*.sh", "*.ps1"]

# Minimum glibc for Linux compatibility
minimum-glibc-version = "2.17"

# Custom GitHub runners
[dist.github-custom-runners]
ubuntu-latest = "ubuntu-22.04"
macos-latest = "macos-14"

# Multiple binaries
[[dist.bin]]
name = "main-binary"
path = "src/main.rs"

[[dist.bin]]
name = "helper-tool"
path = "src/bin/helper.rs"

# System packages needed during build
[dist.global-build-config]
system-packages = ["libssl-dev", "pkg-config"]
```

---

## Supported Platforms by Installer Type

| Platform | shell | powershell | msi | homebrew |
|----------|-------|------------|----|----------|
| **macOS (Intel)** | ✓ | ✗ | ✗ | ✓ |
| **macOS (ARM)** | ✓ | ✗ | ✗ | ✓ |
| **Linux (GNU)** | ✓ | ✗ | ✗ | ✗ |
| **Linux (musl)** | ✓ | ✗ | ✗ | ✗ |
| **Windows** | ✗ | ✓ | ✓ | ✗ |

---

## Target Triples Quick Reference

### Tier 1 Supported (Fully Tested)

```
aarch64-apple-darwin       # macOS ARM64
aarch64-pc-windows-msvc    # Windows ARM64
i686-pc-windows-msvc       # Windows x86
x86_64-apple-darwin        # macOS x86-64
x86_64-pc-windows-msvc     # Windows x86-64
x86_64-unknown-linux-gnu   # Linux x86-64 (glibc)
```

### Tier 2+ Supported (Well Supported)

```
aarch64-unknown-linux-gnu          # Linux ARM64 (glibc)
armv7-unknown-linux-gnueabihf      # Linux ARMv7 (soft float)
x86_64-unknown-linux-musl          # Linux x86-64 (musl)
aarch64-unknown-linux-musl         # Linux ARM64 (musl)
i686-unknown-linux-gnu             # Linux x86 (glibc)
powerpc64le-unknown-linux-gnu      # Linux PPC64LE
riscv64gc-unknown-linux-gnu        # Linux RISC-V
s390x-unknown-linux-gnu            # Linux s390x
```

---

## Common Commands

```bash
# Setup & Configuration
cargo dist init              # Interactive setup
cargo dist generate          # Generate workflows
cargo dist config           # Show current config

# Testing & Planning
cargo dist build            # Build for current platform
cargo dist plan             # Show what CI will do
cargo dist plan --output-format=json  # JSON output

# Release Workflow
git tag v1.0.0
git push --tags             # Triggers CI automatically

# Updates
cargo dist init             # Safe to rerun—preserves settings
cargo install cargo-dist --locked  # Update to latest
```

---

## Environment Variables

### Installation Customization

For a tool named `my-cli`:

```bash
MY_CLI_INSTALL_DIR=/opt/bin ./install.sh
```

Find the env var name:
```bash
# In dist-manifest.json generated with each release
jq '.install_dir_env_var' dist-manifest.json
```

### Build Configuration

```bash
# Force specific features during release
RUSTFLAGS="--cfg tokio_unstable" cargo dist build

# Custom linker
RUSTFLAGS="-Clink-arg=-fuse-ld=mold" cargo dist build

# Parallel compilation
CARGO_INCREMENTAL=1 cargo dist build
```

---

## Release Process (Step-by-Step)

### 1. Prepare

```bash
# Update version
vim Cargo.toml  # Bump [package].version

# Update changelog
vim CHANGELOG.md  # Add entry:
# ## 1.2.0 (2025-01-15)
# - Added feature X
# - Fixed bug Y
# - Improved performance

git add Cargo.toml CHANGELOG.md
git commit -m "chore: bump version to 1.2.0"
```

### 2. Test (Local)

```bash
cargo dist plan   # Should show what CI will do
cargo dist build  # Build for current platform
```

### 3. Release

```bash
git tag v1.2.0
git push --tags
# CI automatically:
# - Builds all platforms
# - Creates installers
# - Publishes to GitHub Releases
# - Updates Homebrew formula
# - Generates attestations
```

### 4. Verify

```bash
# Check GitHub Releases page
# - All binaries present
# - All installers present
# - Checksums available
# - Release notes correct

# Verify artifact authenticity
gh attestation verify ./my-tool-v1.2.0.tar.gz --repo user/repo
```

---

## Troubleshooting

### Build Fails on Linux

**Problem:** `error: failed to run custom build command`

**Solution:**
```toml
[dist.global-build-config]
system-packages = ["libssl-dev", "pkg-config"]
```

### CI Error: ubuntu-20.04 unsupported

**Problem:** Runner no longer exists in GitHub Actions

**Solution:**
```toml
[dist.github-custom-runners]
ubuntu-latest = "ubuntu-22.04"
```

### Release Variables Ignored

**Problem:** `.cargo/config.toml` settings not applied

**Solution:**
```bash
# Pass explicitly in build
RUSTFLAGS="--cfg feature" cargo dist build
```

### Slow Builds

**Solution 1:** Use mold linker
```toml
[dist.global-build-config]
system-packages = ["mold"]
```

**Solution 2:** Enable LTO
```toml
[profile.dist]
lto = "thin"
```

**Solution 3:** Limit to critical platforms
```toml
targets = [
    "x86_64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
```

---

## Security Checklist

- [ ] Pin `cargo-dist-version` to exact version
- [ ] Review generated `.github/workflows/release.yml`
- [ ] Enable `github-attestations-phase`
- [ ] Publish checksums in release notes
- [ ] Test release on pull request with `pr-run-mode = "plan"`
- [ ] Use branch protection for release tags
- [ ] Consider signing git tags: `git tag -s v1.0.0`
- [ ] Enable GitHub OIDC for package publishing
- [ ] Review `dist-manifest.json` in releases

---

## Performance Settings

### Fast Builds (for testing)

```toml
[profile.dist]
inherits = "release"
lto = false  # No LTO
codegen-units = 16  # Parallel
```

### Optimized Binaries (for release)

```toml
[profile.dist]
inherits = "release"
lto = "thin"      # Balanced
codegen-units = 16
strip = false  # Keep symbols for now
```

### Maximum Optimization (slowest builds)

```toml
[profile.dist]
inherits = "release"
lto = "fat"
codegen-units = 1
strip = true
opt-level = "z"
```

---

## Real-World Configuration Examples

### Small CLI Tool

```toml
[workspace.metadata.dist]
cargo-dist-version = "0.30.2"
ci = ["github"]
installers = ["shell", "powershell"]
targets = [
    "x86_64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
```

### Large Multi-Binary Project (UV, Ruff style)

```toml
[dist]
cargo-dist-version = "0.30.2"
ci = ["github"]
installers = ["shell", "powershell", "homebrew"]
targets = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "aarch64-pc-windows-msvc",
    "x86_64-pc-windows-msvc",
]
pr-run-mode = "plan"
allow-dirty = ["ci"]
github-attestations-phase = "announce"
minimum-glibc-version = "2.17"

[[dist.bin]]
name = "uv"
[[dist.bin]]
name = "uvx"
[[dist.bin]]
name = "uvw"
only-platforms = ["x86_64-pc-windows-msvc"]

[dist.github-custom-runners]
ubuntu-latest = "depot.dev/..."
```

### Python Project (Ruff style)

```toml
[dist]
cargo-dist-version = "0.30.2"
ci = ["github"]
installers = ["shell", "powershell"]
targets = [
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
]
# Don't publish to crates.io
publish = false
```

---

## External Resources

**Official:** https://axodotdev.github.io/cargo-dist/book/

**GitHub:** https://github.com/axodotdev/cargo-dist

**Crates.io:** https://crates.io/crates/cargo-dist

**Installer Docs:** https://opensource.axo.dev/cargo-dist/book/installers/

**Config Reference:** https://opensource.axo.dev/cargo-dist/book/reference/config.html

---

**Last Updated:** December 2025
