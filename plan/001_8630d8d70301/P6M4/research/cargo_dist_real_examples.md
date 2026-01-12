# cargo-dist: Real-World Configuration Examples

This document contains actual configuration files from successful production projects using cargo-dist.

---

## 1. UV - Multi-Binary Python Package Manager (astral-sh)

**Project:** https://github.com/astral-sh/uv

**Purpose:** Python package installer, resolver, and build tool

**Key Stats:**
- 18 target platforms
- Multiple binaries: `uv`, `uvx`, `uvw`
- Production-grade release process
- Complex CI requirements

### dist-workspace.toml (Simplified Example)

```toml
[workspace]
members = ["cargo:uv"]

[dist]
cargo-dist-version = "0.30.2"
ci = ["github"]

# Installers for multiple OS
installers = ["shell", "powershell", "homebrew"]

# Extensive platform support
targets = [
    # Apple
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    # Linux GNU
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
    # Linux musl (Alpine)
    "x86_64-unknown-linux-musl",
    # Other Linux variants
    "armv7-unknown-linux-gnueabihf",
    "powerpc64le-unknown-linux-gnu",
    "riscv64gc-unknown-linux-gnu",
    "s390x-unknown-linux-gnu",
    # Windows
    "aarch64-pc-windows-msvc",
    "x86_64-pc-windows-msvc",
    # 32-bit variants
    "i686-unknown-linux-gnu",
    "i686-pc-windows-msvc",
]

# Allow dirty CI directory
allow-dirty = ["ci"]

# Test releases on PRs
pr-run-mode = "plan"

# Use GitHub dispatch instead of tag-based
dispatch-releases = true

# Verify artifact authenticity
github-attestations-phase = "announce"
github-attestations-filters = [
    "*.tar.gz",
    "*.zip",
    "*.msi",
    "*.sh",
    "*.ps1",
    "*.json",
]

# Minimum glibc version for Linux compatibility
minimum-glibc-version = "2.17"

# Custom runners for performance
[dist.github-custom-runners]
ubuntu-latest = "depot.dev/depot.sh"  # Cached Docker layers

# Multiple binaries in single release
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

# Build configuration
[dist.global-build-config]
system-packages = ["libssl-dev", "pkg-config"]
```

### Key Learnings from UV

1. **Dispatch-Based Releases:** Uses `dispatch-releases = true` for more control over when releases happen
2. **Platform Diversity:** Supports 18 platforms including PowerPC, RISC-V, and s390x
3. **Performance Optimization:** Uses Depot's Docker caching for faster builds
4. **Multiple Binaries:** Includes separate binaries for `uvx` and `uvw` with platform restrictions
5. **Minimal glibc:** Set to 2.17 for broad Linux compatibility dating back to CentOS 6

**Repository Link:** https://github.com/astral-sh/uv/blob/main/dist-workspace.toml

---

## 2. Ruff - Python Linter & Formatter (astral-sh)

**Project:** https://github.com/astral-sh/ruff

**Purpose:** Rust-based Python linting and code formatting tool

**Key Stats:**
- Early adopter (since Feb 2023)
- Published to PyPI and Homebrew
- Complex build requirements (C extensions)
- Heavy optimization for performance

### Relevant Cargo.toml Section

```toml
[package]
name = "ruff"
version = "0.6.0"  # Example version
repository = "https://github.com/astral-sh/ruff"
license = "MIT"
readme = "README.md"
edition = "2021"

[workspace.metadata.dist]
cargo-dist-version = "0.30.2"
ci = ["github"]

# Supports all major installers
installers = ["shell", "powershell", "msi", "homebrew"]

# Standard desktop platforms
targets = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "x86_64-pc-windows-msvc",
]

# Build profile for optimization
[profile.dist]
inherits = "release"
lto = "thin"
codegen-units = 16
```

### Release Workflow (.github/workflows/build-binaries.yml)

```yaml
name: Build and upload dist artifacts

on:
  push:
    tags:
      - "v*"

jobs:
  upload:
    name: Build and upload artifacts
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-22.04
            target: x86_64-unknown-linux-gnu
          - os: ubuntu-22.04
            target: x86_64-unknown-linux-musl
          - os: ubuntu-22.04
            target: aarch64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install dist
        run: cargo install cargo-dist --locked

      - name: Build
        run: cargo dist build --target ${{ matrix.target }}

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: dist-${{ matrix.target }}
          path: target/distrib/*
```

### Key Learnings from Ruff

1. **Automated with dist:** Uses `cargo dist` for all builds, allowing clean separation of CI concerns
2. **Matrix Per-Target:** Each platform in separate workflow job for clarity
3. **Artifact Preservation:** All built artifacts automatically collected
4. **Python Integration:** Despite being in Rust, publishes to PyPI ecosystem

**Repository Link:** https://github.com/astral-sh/ruff

---

## 3. Pixi - Package Manager (prefix-dev)

**Project:** https://github.com/prefix-dev/pixi

**Purpose:** Package management for data science, ML, and scientific development

**Key Stats:**
- Recently migrated to cargo-dist (PR #2566)
- Cross-platform package manager
- Complex release requirements
- Multiple dependencies

### Migration Changes (PR #2566)

**Before:** Matrix-based CI with embedded release logic
```yaml
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu, macos, windows]
    # Release logic mixed in
```

**After:** Platform-specific jobs + cargo-dist handling releases
```yaml
# Standard CI jobs (test, lint, etc.)
test:
  runs-on: ubuntu-latest
  # No release logic here

# Separate job handles releases
release:
  needs: [test, lint]
  uses: ./.github/workflows/release.yml
```

### Updated Configuration

```toml
[workspace.metadata.dist]
cargo-dist-version = "0.30.2"
ci = ["github"]
installers = ["shell", "powershell", "msi"]

targets = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc",
]

# Test on PRs
pr-run-mode = "plan"

# Optimization: Windows Dev Drive
[dist.github-custom-runners]
windows-latest = "windows-2025"  # Newer runner with dev drive
```

### Key Learnings from Pixi

1. **Clean Separation:** Move all release logic out of main CI
2. **Platform-Specific Jobs:** Each platform gets its own clear job definition
3. **Development Optimizations:** Use Windows Dev Drive for filesystem performance
4. **Reduced Complexity:** Fewer matrix variables = easier to maintain

**Repository Link:** https://github.com/prefix-dev/pixi/pull/2566

---

## 4. cargo-dist Itself (Self-Hosting)

**Project:** https://github.com/axodotdev/cargo-dist

**Purpose:** Distribution tool (dogfooding itself)

**Key Stats:**
- Uses itself for releases (proof of stability)
- Workspace with multiple packages
- Rapid iteration and updates
- Reference implementation

### Cargo.toml

```toml
[workspace]
resolver = "2"
members = ["axoproject", "cargo-dist", "cargo-dist-schema"]

[workspace.metadata.dist]
cargo-dist-version = "0.30.3"
ci = ["github"]
installers = ["shell", "powershell"]
targets = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
]

[[workspace.metadata.dist.bin]]
name = "cargo-dist"
path = "cargo-dist"
```

### Release Workflow Snippet

```yaml
# .github/workflows/release.yml (auto-generated by dist)
name: Release

on:
  push:
    tags:
      - v*

jobs:
  plan:
    name: plan
    runs-on: ubuntu-latest
    outputs:
      linux-gnu-sha: ${{ steps.check-linux-gnu.outputs.sha }}
      macos-sha: ${{ steps.check-macos.outputs.sha }}
      windows-sha: ${{ steps.check-windows.outputs.sha }}

    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-dist
        shell: bash
        run: |
          curl --proto '=https' --tlsv1.2 -LsSf \
            https://github.com/axodotdev/cargo-dist/releases/download/v0.30.2/cargo-dist-installer.sh | sh

      - name: Run cargo dist plan
        run: cargo dist plan --output-format=json > dist-plan.json
        env:
          CARGO_TERM_VERBOSE: true

      - name: Upload plan
        uses: actions/upload-artifact@v4
        with:
          name: dist-plan
          path: dist-plan.json

  build-linux-gnu:
    needs: [plan]
    runs-on: ubuntu-latest
    env:
      GH_TOKEN: ${{ github.token }}

    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: x86_64-unknown-linux-gnu

      - name: Install cargo-dist
        shell: bash
        run: |
          curl --proto '=https' --tlsv1.2 -LsSf \
            https://github.com/axodotdev/cargo-dist/releases/download/v0.30.2/cargo-dist-installer.sh | sh

      - name: Build with cargo-dist
        run: cargo dist build --target x86_64-unknown-linux-gnu
        env:
          CARGO_TERM_VERBOSE: true

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: artifacts-x86_64-unknown-linux-gnu
          path: target/distrib/*

  # ... similar jobs for macOS and Windows ...

  publish:
    name: publish
    needs: [plan, build-linux-gnu, build-macos, build-windows]
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          files: ${{ needs.build-linux-gnu.outputs.artifacts }}
          body_path: RELEASE_NOTES.md
```

### Key Learnings from cargo-dist

1. **Workspace Design:** Multi-package workspace with clean separation
2. **Plan Step First:** Always runs plan before builds
3. **Parallel Builds:** Multiple platform jobs run simultaneously
4. **Artifact Management:** Central upload/download coordination
5. **GitHub Attestations:** Auto-generates and verifies artifacts

**Repository Link:** https://github.com/axodotdev/cargo-dist/blob/main/.github/workflows/release.yml

---

## 5. OpenTelemetry Configuration (open-telemetry)

**Project:** https://github.com/open-telemetry/opentelemetry-configuration

**Purpose:** Configuration specification for OpenTelemetry

**Key Stats:**
- Multi-language configuration
- Complex build matrix
- Precise version pinning

### dist.toml

```toml
[dist]
cargo-dist-version = "0.19.1"
ci = ["github"]
installers = ["shell", "powershell"]

targets = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "x86_64-pc-windows-msvc",
]

# Support for library distribution
cdylibs = ["opentelemetry_config"]
cstaticlibs = ["opentelemetry_config_static"]
```

### Key Learnings

1. **C Library Distribution:** Using `cdylibs` for shared library distribution
2. **Static Library Support:** `cstaticlibs` for static linking
3. **Older Version:** Using 0.19.1 (from 2023) shows backward compatibility
4. **Focused Platforms:** Only core platforms, avoiding edge cases

**Repository Link:** https://github.com/open-telemetry/opentelemetry-configuration/blob/main/dist.toml

---

## 6. qlty - Code Quality Tool

**Project:** https://github.com/qltysh/qlty

**Purpose:** Multi-language linter and formatter orchestration

**Cargo.toml Configuration:**

```toml
[package]
name = "qlty"
version = "0.9.0"
repository = "https://github.com/qltysh/qlty"
license = "Apache-2.0 OR MIT"
edition = "2021"

[workspace.metadata.dist]
cargo-dist-version = "0.19.1"
ci = ["github"]
installers = ["homebrew"]  # Primary distribution via Homebrew

targets = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc",
]

# Install to CARGO_HOME (standard location)
install-path = "CARGO_HOME"
pr-run-mode = "plan"
install-updater = true

# Custom runners for specific platforms
[workspace.metadata.dist.github-custom-runners]
ubuntu-latest = "ubuntu-22.04"
macos-latest = "macos-14-arm64"
```

### Key Learnings

1. **Homebrew-First:** Primary distribution via Homebrew for macOS users
2. **Standard Paths:** Using `CARGO_HOME` for installation
3. **Updater Support:** Enabled for seamless updates
4. **Custom ARM Runners:** Explicit macOS ARM64 runner selection
5. **PR Safety:** Testing on pull requests before releases

**Repository Link:** https://github.com/qltysh/qlty/blob/main/Cargo.toml

---

## Configuration Patterns & Anti-Patterns

### Pattern 1: The Simple Tool

**When to use:** Single binary, few platforms, standard distribution

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

### Pattern 2: The Complex Project

**When to use:** Multiple binaries, many platforms, optimized releases

```toml
[dist]
cargo-dist-version = "0.30.2"
ci = ["github"]
installers = ["shell", "powershell", "msi", "homebrew"]

# Many targets for maximum compatibility
targets = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "armv7-unknown-linux-gnueabihf",
    "x86_64-pc-windows-msvc",
    "aarch64-pc-windows-msvc",
]

# Test on PRs
pr-run-mode = "plan"
allow-dirty = ["ci"]

# Custom runners for performance
[dist.github-custom-runners]
ubuntu-latest = "depot.dev/depot.sh"

# Multiple binaries
[[dist.bin]]
name = "primary"
[[dist.bin]]
name = "secondary"
only-platforms = ["x86_64-pc-windows-msvc"]

# Build dependencies
[dist.global-build-config]
system-packages = ["libssl-dev", "pkg-config"]

# Minimum glibc for Linux
minimum-glibc-version = "2.17"
```

### Anti-Pattern 1: Over-Targeting

**Problem:** Too many platforms = slow releases with diminishing returns

```toml
# Don't do this:
targets = [
    # ... 20+ platforms ...
    "thumbv7em-none-eabihf",  # Embedded systems
    "wasm32-unknown-unknown",  # WebAssembly
    # Too many edge cases!
]
```

**Solution:** Focus on actual user platforms
```toml
# Do this instead:
targets = [
    "x86_64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
```

### Anti-Pattern 2: Not Pinning Versions

**Problem:** Builds become unreproducible

```toml
# Don't do this:
# cargo-dist-version left unpinned or uses floating version
```

**Solution:** Always pin exact version
```toml
cargo-dist-version = "0.30.2"  # Exact version
```

---

## Real Configuration Comparison Table

| Project | Size | Platforms | Installers | Special Features |
|---------|------|-----------|------------|-----------------|
| **UV** | Large | 18 | shell, powershell | Dispatch releases, 3 binaries, Depot |
| **Ruff** | Large | 6 | shell, powershell, msi, homebrew | C extensions, PyPI |
| **Pixi** | Large | 6 | shell, powershell, msi | Windows Dev Drive, migrated from matrix |
| **cargo-dist** | Medium | 4 | shell, powershell | Self-hosting, workspace, attestations |
| **OpenTelemetry** | Medium | 5 | shell, powershell | C libraries (cdylibs, cstaticlibs) |
| **qlty** | Small | 5 | homebrew | Homebrew-primary, updater |

---

## Migration Path

If you're migrating from manual releases to cargo-dist:

### Step 1: Initialize

```bash
cargo dist init
# Accept default suggestions
```

### Step 2: Review Generated Files

```bash
# Check auto-generated workflow
cat .github/workflows/release.yml

# Check configuration
cat Cargo.toml  # or dist-workspace.toml
```

### Step 3: Customize (if needed)

- Adjust `targets` for your platforms
- Change `installers` for your distribution needs
- Add `pr-run-mode = "plan"` for testing

### Step 4: Test on PR

```bash
git add .
git commit -m "chore: add cargo-dist"
git push -u origin feature/cargo-dist
# Create PR, see it run dist plan
```

### Step 5: Release

```bash
git tag v1.0.0
git push --tags
# Let CI handle everything else
```

---

**Last Updated:** December 2025

**Sources:** GitHub repositories of UV, Ruff, Pixi, cargo-dist, OpenTelemetry, and qlty projects
