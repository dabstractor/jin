# Comprehensive cargo-dist Guide: Rust Binary Distribution in 2025

**Last Updated:** December 2025

A deep-dive research guide on cargo-dist, the modern release and distribution tool for Rust binaries. This guide covers setup, configuration, real-world examples, and best practices.

---

## Table of Contents

1. [Overview & Why cargo-dist](#overview--why-cargo-dist)
2. [Architecture & How It Works](#architecture--how-it-works)
3. [Setup & Configuration](#setup--configuration)
4. [Features & Capabilities](#features--capabilities)
5. [Real-World Project Examples](#real-world-project-examples)
6. [Best Practices & Optimization](#best-practices--optimization)
7. [Common Issues & Solutions](#common-issues--solutions)
8. [Security Considerations](#security-considerations)
9. [Performance Tips](#performance-tips)

---

## Overview & Why cargo-dist

### What is cargo-dist?

cargo-dist (also called "dist") is an automated release and distribution tool for Rust applications developed by [axodotdev](https://axo.dev). It streamlines the entire release pipeline—from building binaries across multiple platforms to generating installers, publishing packages, and announcing releases.

**Official Documentation:** https://axodotdev.github.io/cargo-dist/book/

### Why It's Recommended

1. **Automation First:** Automatically generates CI/CD scripts (GitHub Actions) with a single command
2. **Cross-Platform:** Builds and tests binaries for Linux, macOS, Windows simultaneously
3. **Reproducibility:** The same command works locally and in CI—no surprises
4. **Developer Experience:** Minimal configuration needed; `cargo dist init` handles most setup
5. **Multi-Format Installers:** Generates shell installers, PowerShell installers, MSI packages, and Homebrew formulas
6. **Self-Hosting:** cargo-dist uses itself for releases, proving production-readiness
7. **Language Agnostic:** While born for Rust, it now supports C, JavaScript, and other languages

### Project Statistics

- **Repository:** https://github.com/axodotdev/cargo-dist
- **License:** MIT OR Apache-2.0
- **Current Version:** 0.30.2+ (as of late 2025)
- **Community:** 1.9k+ GitHub stars, 76+ contributors
- **Language:** 89.8% Rust

---

## Architecture & How It Works

### Two-Phase Workflow

cargo-dist operates in two complementary phases:

#### Phase 1: Building
- Plans the release (detects version from git tags)
- Compiles binaries for all configured platforms
- Generates tarballs and installers
- Creates machine-readable manifests

#### Phase 2: Distribution
- Auto-generates GitHub Actions workflows
- Publishes artifacts to GitHub Releases
- Publishes to package managers (Homebrew, etc.)
- Announces releases with changelog integration

### Workflow Trigger

When you push a git tag:

```
git tag v0.3.9
git push --tags
```

cargo-dist automatically:
1. Detects the version tag format
2. Identifies affected packages
3. Builds binaries for all platforms
4. Creates installers and archives
5. Publishes to GitHub Releases
6. Updates package managers

### Key Superpower

**Automatic CI Script Generation:** Running `cargo dist init` generates a complete `.github/workflows/release.yml` file that implements the full pipeline without manual intervention.

---

## Setup & Configuration

### Prerequisites

1. A Rust project with `Cargo.toml`
2. Git repository with remote configured
3. GitHub account with repository write access
4. Proper `Cargo.toml` metadata:

```toml
[package]
name = "my-awesome-tool"
version = "0.1.0"
description = "A great CLI tool"
repository = "https://github.com/username/repo"
license = "MIT"  # or "MIT OR Apache-2.0"
readme = "README.md"
```

### Installation

Install cargo-dist as a Cargo plugin:

```bash
cargo install cargo-dist --locked
```

Or using the installer script:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/axodotdev/cargo-dist/releases/download/v0.30.2/cargo-dist-installer.sh | sh
```

### Initial Setup: `cargo dist init`

Run the interactive setup:

```bash
cargo dist init
```

This command will:
1. Ask which CI system you use (e.g., GitHub Actions)
2. Ask which installers you want (shell, PowerShell, MSI, Homebrew)
3. Ask which platforms to target
4. Generate `dist-workspace.toml` or add `[workspace.metadata.dist]` to `Cargo.toml`
5. Create `.github/workflows/release.yml`

**Key Feature:** `cargo dist init` is safe to rerun—it preserves your settings while applying updates and migrations.

### Configuration: Cargo.toml vs. dist-workspace.toml

#### Option 1: Cargo.toml (Simple projects)

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

#### Option 2: dist-workspace.toml (Complex projects)

For workspaces with multiple packages or non-Rust projects, use a dedicated `dist-workspace.toml`:

```toml
[workspace]
members = ["cargo:my-package"]

[dist]
cargo-dist-version = "0.30.2"
ci = ["github"]
installers = ["shell", "powershell"]
targets = [
    "x86_64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc",
]
allow-dirty = ["ci"]
```

### Essential Configuration Fields

| Field | Purpose | Example |
|-------|---------|---------|
| `cargo-dist-version` | Pin exact dist version for reproducibility | `"0.30.2"` |
| `ci` | CI system to generate workflows for | `["github"]` |
| `installers` | Installer types to generate | `["shell", "powershell", "msi", "homebrew"]` |
| `targets` | Platform target triples to build | See [Supported Platforms](#supported-platforms) |
| `pr-run-mode` | What to do on PR (plan/upload) | `"plan"` or `"upload"` |
| `allow-dirty` | Allow dirty files during CI | `["ci"]` |
| `github-attestations-phase` | When to generate GitHub attestations | `"host"` |
| `minimum-glibc-version` | Minimum glibc for Linux targets | `"2.17"` |

### Supported Platforms

cargo-dist supports these target triples:

**macOS:**
- `aarch64-apple-darwin` (Apple Silicon)
- `x86_64-apple-darwin` (Intel)

**Linux (GNU):**
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `i686-unknown-linux-gnu`
- `armv7-unknown-linux-gnueabihf`

**Linux (musl):**
- `x86_64-unknown-linux-musl`
- `aarch64-unknown-linux-musl`

**Linux (Other):**
- `powerpc64le-unknown-linux-gnu`
- `riscv64gc-unknown-linux-gnu`
- `s390x-unknown-linux-gnu`

**Windows:**
- `x86_64-pc-windows-msvc`
- `i686-pc-windows-msvc`
- `aarch64-pc-windows-msvc`

### Customizing the Release Workflow

#### Custom Build Steps

Add pre-build setup (e.g., installing system dependencies):

```toml
[dist.github-custom-runners]
ubuntu-latest = "custom-runner"

[dist.global-build-config]
extra-build-steps = [
    "apt-get update && apt-get install -y libssl-dev",
]
```

#### Custom GitHub Runners

```toml
[dist.github-custom-runners]
ubuntu-22.04 = "depot.dev/depot.sh"
macos-15-intel = "macos-15-large"
```

#### PR Build Mode

Set `pr-run-mode` to test releases on pull requests:

```toml
[dist]
pr-run-mode = "plan"  # Just run plan step
# or
pr-run-mode = "upload"  # Build and upload as artifacts
```

---

## Features & Capabilities

### 1. Automatic Binary Builds

Builds optimized, production-ready binaries for all platforms using:
- Release profile with optimizations
- Thin LTO (Link-Time Optimization) enabled by default
- Cross-compilation support

### 2. Installer Generation

#### Shell Installer

Generated for Unix-like systems (Linux, macOS):
- Compatible with POSIX shells
- Respects `$HOME` and XDG directories
- Custom environment variables: `APP_NAME_INSTALL_DIR`
- Updater support (optional)

Example environment variable for installation:
```bash
AXOLOTLSAY_INSTALL_DIR=/opt/bin ./install.sh
```

#### PowerShell Installer

Generated for Windows:
- Respects `HTTPS_PROXY` and `ANY_PROXY` environment variables
- Proper execution policy handling
- Extended Unix permissions in ZIP archives

#### MSI Installer

For Windows installations:
- Requires WiX Toolset v3 (WiX v4 not yet supported)
- Modern UI and installer features
- Automatic PATH registration

Example configuration:

```toml
[dist]
installers = ["msi"]
```

#### Homebrew Formula

Auto-generates Homebrew formula for macOS:

```toml
[dist]
installers = ["homebrew"]
```

### 3. GitHub Release Integration

Automatically:
- Creates GitHub releases from git tags
- Uploads all artifacts (binaries, installers)
- Generates checksums (SHA256)
- Creates release notes from CHANGELOG.md
- Supports GitHub Attestations for artifact verification

### 4. Checksum & Artifact Verification

**Automatic Checksum Generation:**
cargo-dist generates SHA256 checksums for all artifacts in `*.sha256` format.

**GitHub Artifact Attestations:**
Uses GitHub's native attestation feature for supply chain security:

```bash
# Verify an artifact
gh attestation verify <artifact-path> --repo username/repo

# Or with downloaded attestation
gh attestation verify <artifact-path> --bundle <attestation-bundle>
```

### 5. Multiple Binary Support

Single release can include multiple binaries:

```toml
[[dist.bin]]
name = "uv"
path = "src/uv/main.rs"

[[dist.bin]]
name = "uvx"
path = "src/uvx/main.rs"

[[dist.bin]]
name = "uvw"
path = "src/uvw/main.rs"
only-platforms = ["x86_64-pc-windows-msvc"]  # Windows only
```

---

## Real-World Project Examples

### Example 1: UV (astral-sh)

**Project:** Python package installer and resolver

**Repository:** https://github.com/astral-sh/uv

**Configuration File:** https://github.com/astral-sh/uv/blob/main/dist-workspace.toml

#### Key Features

- **Version:** cargo-dist 0.30.2
- **18 Target Platforms** including:
  - Apple Silicon and Intel (macOS)
  - GNU and musl Linux variants
  - ARM, PowerPC, RISC-V, s390x
  - Windows (both x86_64 and aarch64)
- **Multiple Binaries:** `uv`, `uvx`, and `uvw` (Windows-only)
- **Installers:** Shell, PowerShell, and Homebrew
- **GitHub Attestations:** Attests JSON, shell scripts, PowerShell scripts, and archives
- **Custom Runners:** Uses Depot's Ubuntu container for optimization
- **Minimum glibc:** 2.17-2.31 per target

#### Configuration Highlights

```toml
[dist]
cargo-dist-version = "0.30.2"
ci = ["github"]
github-custom-runners = { ubuntu-latest = "depot.dev/..." }
allow-dirty = ["ci"]
installers = ["shell", "powershell"]
dispatch-releases = true  # Use GitHub dispatch instead of tag push
```

**Why This Works:** Large project managing multiple platform binaries with strict version requirements.

---

### Example 2: Ruff (astral-sh)

**Project:** Python linter and formatter written in Rust

**Repository:** https://github.com/astral-sh/ruff

**Configuration:** https://github.com/astral-sh/ruff/blob/main/.github/workflows/build-binaries.yml

#### Key Features

- Early adopter of cargo-dist (since Feb 2023)
- Generates distributable binaries and installers
- Publishes to PyPI and Homebrew simultaneously
- Full integration with Python ecosystem

---

### Example 3: Pixi (prefix-dev)

**Project:** Package management tool (Python ecosystem)

**Repository:** https://github.com/prefix-dev/pixi

**Implementation PR:** https://github.com/prefix-dev/pixi/pull/2566

#### Refactoring Highlights

The team moved from matrix-based CI to platform-specific job definitions and delegated all release logic to cargo-dist:

**Before:**
- CI matrix handling all platforms
- Release logic embedded in main pipeline
- Complex, tightly-coupled configuration

**After:**
- Standard CI independent from releases
- `cargo-dist` manages complete release pipeline
- Clean separation of concerns

#### Benefits Realized

- Faster iteration on CI changes
- Easier maintenance of platform-specific logic
- Dedicated release workflow isolated from testing

---

### Example 4: cargo-dist Itself

**Project:** The distribution tool itself (self-hosting)

**Repository:** https://github.com/axodotdev/cargo-dist

**Release Workflow:** https://github.com/axodotdev/cargo-dist/blob/main/.github/workflows/release.yml

#### Why It's Important

cargo-dist uses itself to distribute itself—this is the strongest proof of production-readiness. The tool's own releases are a living example of best practices.

**Key Process:**
1. Push git tag with version format `v0.30.2`
2. `cargo dist` generates releases with:
   - Multiple platform binaries
   - Installer scripts (shell, PowerShell)
   - Installation instructions
   - GitHub release artifacts
3. No manual release steps needed

---

## Best Practices & Optimization

### Pre-Release Checklist

1. **Version Management**
   ```bash
   # Bump version in Cargo.toml
   vim Cargo.toml  # Update version field

   # Optionally add release notes
   vim CHANGELOG.md  # Add entry under version heading

   # Commit changes
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: bump version to v0.2.0"
   ```

2. **Local Testing**
   ```bash
   # Test building for current platform
   cargo dist build

   # Test release planning without building
   cargo dist plan

   # This shows exactly what CI will do
   ```

3. **Create Release Tag**
   ```bash
   git tag v0.2.0
   git push --tags
   ```

### Configuration Best Practices

#### 1. Always Pin cargo-dist Version

```toml
[workspace.metadata.dist]
cargo-dist-version = "0.30.2"  # Always use pinned version
```

**Why:** Ensures reproducible releases. Without pinning, CI might use different versions across runs.

#### 2. Rerun `cargo dist init` After Updates

```bash
# When upgrading cargo-dist
cargo dist init
# This applies migrations and updates workflows
```

#### 3. Use `pr-run-mode = "plan"`

```toml
[dist]
pr-run-mode = "plan"  # Test release process on PRs
```

**Why:** Catches release pipeline breakage before actual release.

#### 4. Enable GitHub Attestations

```toml
[dist]
github-attestations-phase = "announce"
github-attestations-filters = ["*.json", "*.sh", "*.ps1", "*.zip", "*.tar.gz"]
```

**Why:** Provides cryptographic proof of artifact authenticity.

#### 5. Set Minimum glibc Version

```toml
[dist]
minimum-glibc-version = "2.17"  # For broad Linux compatibility
```

**Why:** Ensures binaries run on older Linux distributions.

### Workflow Optimization

#### Multi-Platform Builds Efficiently

```bash
# These run in parallel in CI
cargo dist build --target x86_64-unknown-linux-gnu
cargo dist build --target aarch64-apple-darwin
cargo dist build --target x86_64-pc-windows-msvc
```

#### Optimize Build Times

1. **Use Cached Docker/Container Images**
   ```toml
   [dist.github-custom-runners]
   ubuntu-latest = "ubuntu-22.04"
   ```

2. **Enable ccache for faster recompilation**
   ```toml
   [dist.global-build-config]
   system-packages = ["ccache"]
   ```

3. **Use alternative linkers for faster linking**
   ```toml
   [dist.global-build-config]
   extra-build-steps = ["apt-get install -y mold"]
   ```

#### Profile Settings for Distribution

```toml
# In Cargo.toml
[profile.dist]
inherits = "release"
lto = "thin"        # Balance between size and build time
codegen-units = 16  # Parallel compilation
```

For maximum optimization (slower builds):

```toml
[profile.dist-optimized]
inherits = "release"
lto = "fat"
codegen-units = 1
strip = true
```

---

## Common Issues & Solutions

### Issue 1: Ubuntu 20.04 Runner Deprecation (2025)

**Problem:** Ubuntu 20.04 GitHub Actions runner is deprecated and will be fully unsupported by April 1, 2025.

**Symptoms:**
```
Build fails with ubuntu-20.04 runner
Temporary failures during brownout periods
```

**Solution:**

Update your configuration:

```toml
[dist.github-custom-runners]
ubuntu-latest = "ubuntu-22.04"  # or ubuntu-24.04
```

Or rerun `cargo dist init` which auto-updates.

### Issue 2: OpenSSL Build Failures

**Problem:** Projects with OpenSSL dependencies fail to build in CI.

**Symptoms:**
```
error: failed to run custom build command for openssl-sys
linking with `cc` failed
```

**Solution:**

Add system package installation:

```toml
[dist.global-build-config]
system-packages = ["libssl-dev", "pkg-config"]
```

Or add to custom build steps:

```bash
apt-get update && apt-get install -y libssl-dev pkg-config
```

### Issue 3: Environment Variables Ignored

**Problem:** Settings in `.cargo/config.toml` are ignored by cargo-dist.

**Symptoms:**
```
RUSTFLAGS not applied
Feature flags not working
```

**Solution:**

Pass environment variables explicitly in CI workflow or use custom build steps:

```bash
RUSTFLAGS="--cfg tokio_unstable" cargo dist build
```

### Issue 4: Homebrew Formula Style Issues

**Problem:** Generated Homebrew formulas fail homebrew's style checks.

**Solution:**

Recent versions (0.22.0+) now run `brew style --fix` automatically. Update to latest cargo-dist:

```bash
cargo install cargo-dist --locked
cargo dist init
```

### Issue 5: ZIP Archive Subdirectory Problems

**Problem:** The `include` directive may have issues archiving subdirectories.

**Solution:**

Use the latest cargo-dist (0.30.0+) which improved ZIP handling:

```bash
cargo install cargo-dist@0.30.2 --locked
```

### Issue 6: Cross-Compilation Failures

**Problem:** Building for non-native targets fails.

**Solution:**

1. Install cross-compilation tools:
   ```bash
   cargo install cross
   ```

2. Ensure target is installed:
   ```bash
   rustup target add aarch64-apple-darwin
   rustup target add x86_64-unknown-linux-musl
   ```

3. Or let cargo-dist's GitHub Actions handle it (usually works automatically)

---

## Security Considerations

### 1. GitHub Artifact Attestations

cargo-dist automatically generates cryptographic attestations for releases:

```bash
# Users can verify artifacts
gh attestation verify ./my-tool-v1.0.0.tar.gz --repo username/repo
```

**What it verifies:**
- Artifact was built by your GitHub Actions workflow
- Built from specific commit and branch
- Built at specific time
- Signed with GitHub's keys

**Enable attestations:**

```toml
[dist]
github-attestations-phase = "announce"
github-attestations-filters = ["*.zip", "*.tar.gz", "*.msi"]
```

### 2. Checksum Verification

All artifacts come with SHA256 checksums:

```bash
# Users can verify authenticity
sha256sum -c artifact.sha256
```

Publish checksums prominently in release notes.

### 3. Signing Support (Windows)

cargo-dist supports Windows code signing via SignPath (free for some open-source projects):

**Status:** In development (Issue #1693)

```toml
[dist]
windows-signing = { provider = "signpath", project = "..." }
```

### 4. Supply Chain Security

**Recommendations:**

1. **Pin cargo-dist version** (not just major.minor)
   ```toml
   cargo-dist-version = "0.30.2"  # Exact version
   ```

2. **Review generated workflows** before merging
   ```bash
   # Review auto-generated workflow
   cat .github/workflows/release.yml
   ```

3. **Use GitHub branch protection**
   - Require review before releasing
   - Require status checks to pass

4. **Sign git tags** (optional but recommended)
   ```bash
   git tag -s v1.0.0 -m "Release 1.0.0"
   ```

5. **Use GitHub OIDC for package publishing**
   - Don't store credentials in secrets
   - Use temporary tokens from OIDC provider

### 5. Artifact Provenance

cargo-dist provides full provenance:

1. **Build environment:** Ubuntu, macOS, or Windows runner
2. **Build time:** Exact timestamp
3. **Source:** Git commit and branch
4. **Tools:** Rust version, cargo-dist version
5. **Attestation:** Cryptographically signed

---

## Performance Tips

### Build Time Optimization

#### 1. Parallel Compilation

cargo-dist builds all targets in parallel in GitHub Actions—no extra configuration needed.

**Local testing:**
```bash
# Uses all available cores automatically
cargo dist build
```

#### 2. Incremental Compilation

If building locally frequently, enable incremental compilation:

```bash
export CARGO_INCREMENTAL=1
cargo dist build
```

#### 3. Alternative Linkers

Use faster linkers in your GitHub Actions:

```toml
[dist.global-build-config]
system-packages = ["mold"]  # Faster than default ld
```

Or:

```bash
export RUSTFLAGS="-Clink-arg=-fuse-ld=mold"
```

#### 4. Split Compilation

For very large projects, consider splitting into separate crates:

```toml
[workspace]
members = ["cli", "core", "utils"]
```

### Binary Size Optimization

#### Enable Strip

```toml
[profile.dist]
strip = true  # Remove debugging symbols
```

#### Enable LTO

```toml
[profile.dist]
lto = "thin"   # Good balance
# or
lto = "fat"    # Maximum optimization (slower builds)
```

#### Optimize for Size

```toml
[profile.dist]
inherits = "release"
opt-level = "z"     # Optimize for size
lto = "fat"
codegen-units = 1
strip = true
```

### Caching Strategy

cargo-dist automatically caches:
- Compiled dependencies
- Cargo registry
- Build artifacts

No configuration needed, but ensure GitHub Actions runners have disk space.

### Pre-Built Artifacts

For frequently-changed projects, consider pre-building Docker images:

```bash
# Build once
cargo dist build --target x86_64-unknown-linux-gnu

# Distribute binary without recompiling
```

---

## Advanced Features

### 1. Dispatch-Based Releases

Instead of tag-based releases, use GitHub Actions dispatch:

```toml
[dist]
dispatch-releases = true
```

**Benefit:** Release without creating git tags.

### 2. Recursive Tarballs with Submodules

Include submodule contents in release tarballs:

```toml
[dist]
recursive-tarballs = true
```

### 3. Custom Build Targets

Build for custom, non-standard targets:

```toml
[[dist.targets]]
triple = "custom-target"
```

### 4. Non-Rust Projects

cargo-dist now supports other languages:

```toml
[workspace]
members = ["npm:my-js-project", "node:another-project"]
```

### 5. Updater Integration

Automatic updates for installed binaries:

```toml
[dist]
install-updater = true
install-path = "CARGO_HOME"  # or XDG_BIN_HOME
```

---

## Useful Commands Reference

### Planning & Testing

```bash
# Initialize cargo-dist for a project
cargo dist init

# Generate the release.yml workflow
cargo dist generate

# Test release without building
cargo dist plan

# Build for current platform
cargo dist build

# Check configuration
cargo dist config
```

### Local Release Simulation

```bash
# Plan step (what CI will do)
cargo dist plan

# Build all platforms
cargo dist build

# See what artifacts will be created
cargo dist plan --output-format=json | jq
```

### Upgrading

```bash
# Install latest version
cargo install cargo-dist --locked

# Update configuration and workflows
cargo dist init

# Generate new workflows
cargo dist generate
```

---

## Resources & Documentation

### Official Documentation

- **Main Book:** https://axodotdev.github.io/cargo-dist/book/
- **Alternative Site:** https://opensource.axo.dev/cargo-dist/book/
- **GitHub Repository:** https://github.com/axodotdev/cargo-dist
- **Crates.io:** https://crates.io/crates/cargo-dist

### Configuration Reference

- **Config Reference:** https://opensource.axo.dev/cargo-dist/book/reference/config.html
- **Installers Guide:** https://opensource.axo.dev/cargo-dist/book/installers/
- **CI Customization:** https://opensource.axo.dev/cargo-dist/book/ci/customizing.html

### Installer Documentation

- **Shell Installer:** https://opensource.axo.dev/cargo-dist/book/installers/shell.html
- **PowerShell Installer:** https://opensource.axo.dev/cargo-dist/book/installers/powershell.html
- **MSI Installer:** https://opensource.axo.dev/cargo-dist/book/installers/msi.html
- **Homebrew Formula:** https://opensource.axo.dev/cargo-dist/book/installers/homebrew.html

### Related Tools

- **cargo-release:** https://github.com/crate-ci/cargo-release (version management)
- **release-plz:** https://github.com/MarcoIeni/release-plz (automated releases)
- **cargo-wix:** https://github.com/volks73/cargo-wix (WiX MSI generation)
- **cross:** https://github.com/cross-rs/cross (cross-compilation)

### Blog Posts & Articles

- **"Release Engineering Is Exhausting So Here's cargo-dist"** - https://blog.axo.dev/2023/02/cargo-dist
- **"cargo-dist for any language"** - https://blog.axo.dev/2023/12/generic-builds
- **"Fully Automated Releases for Rust Projects"** - https://blog.orhun.dev/automated-rust-releases/
- **"A Tale of Cargo Dist"** - https://blog.cryingpotato.com/blog/a-tale-of-cargo-dist/
- **"Cargo-dist tips"** - https://sts10.github.io/docs/cargo-dist-tips.html

### Community Examples

- **UV (astral-sh):** https://github.com/astral-sh/uv/blob/main/dist-workspace.toml
- **Ruff (astral-sh):** https://github.com/astral-sh/ruff
- **Pixi (prefix-dev):** https://github.com/prefix-dev/pixi/pull/2566
- **OpenTelemetry Config:** https://github.com/open-telemetry/opentelemetry-configuration/blob/main/dist.toml
- **qlty:** https://github.com/qltysh/qlty/blob/main/Cargo.toml

---

## Conclusion

cargo-dist represents a significant advancement in Rust release engineering. By automating the entire distribution pipeline, it enables developers to focus on code rather than release mechanics.

**Key Takeaways:**

1. **Start Simple:** Use `cargo dist init` and accept defaults initially
2. **Test Before Release:** Always run `cargo dist plan` on pull requests
3. **Iterate:** Rerun `cargo dist init` when updating versions or changing configuration
4. **Monitor:** Watch the auto-generated `.github/workflows/release.yml` for issues
5. **Verify:** Use GitHub Attestations to verify artifact authenticity
6. **Document:** Keep your CHANGELOG.md up-to-date for release notes

The tool is production-ready, widely adopted, and actively maintained. Starting a new Rust project? cargo-dist should be part of your initial setup.

---

**Document Generated:** December 2025
**Research Sources:** Official cargo-dist documentation, GitHub repositories, community implementations, and release notes
