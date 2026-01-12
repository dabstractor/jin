# Rust CI/CD Best Practices for GitHub Actions in 2025

## Executive Summary

This document provides a comprehensive research synthesis of modern Rust CI/CD best practices for GitHub Actions in 2025, including multi-platform builds, testing strategies, release automation, continuous deployment, and performance optimization.

---

## Table of Contents

1. [Modern GitHub Actions Workflows for Rust Projects](#modern-github-actions-workflows)
2. [Testing and Quality Gates](#testing-and-quality-gates)
3. [Release Automation](#release-automation)
4. [Continuous Deployment](#continuous-deployment)
5. [Performance and Cost Optimization](#performance-and-cost-optimization)
6. [Exemplary Repository Workflows](#exemplary-repository-workflows)
7. [Complete Workflow Examples](#complete-workflow-examples)
8. [Tools and Actions Reference](#tools-and-actions-reference)

---

## Modern GitHub Actions Workflows for Rust Projects

### Multi-Platform Builds (Linux, macOS, Windows)

#### Strategy 1: Matrix Builds with Native Platforms

The recommended approach uses GitHub Actions matrix strategy to run builds on different operating systems. Each job runs on its respective native platform for optimal compatibility and performance.

**Key Platforms:**
- **Linux**: `ubuntu-latest` with targets like `x86_64-unknown-linux-gnu` and `x86_64-unknown-linux-musl`
- **macOS**: `macos-latest` for both Intel (`x86_64-apple-darwin`) and ARM64 (`aarch64-apple-darwin`)
- **Windows**: `windows-latest` with MSVC (`x86_64-pc-windows-msvc`) and optional GNU variants

**Matrix Configuration Example:**
```yaml
jobs:
  test:
    strategy:
      matrix:
        include:
          - { os: ubuntu-latest, target: x86_64-unknown-linux-gnu }
          - { os: macos-latest, target: x86_64-apple-darwin }
          - { os: windows-latest, target: x86_64-pc-windows-msvc }
    runs-on: ${{ matrix.os }}
```

#### Strategy 2: Cross-Compilation with `cross`

For building binaries for architectures not native to GitHub's hosted runners (ARM, PowerPC, s390x, RISC-V), use the `cross` tool which employs Docker containers for cross-platform builds.

**Benefits:**
- Compile for exotic architectures from Ubuntu runners
- No special configuration required beyond specifying the target
- Supports: aarch64, armv7, powerpc64, s390x, riscv64gc, wasm32, and more

**Implementation:**
```yaml
- uses: taiki-e/setup-cross-toolchain-action@v1
  with:
    target: aarch64-unknown-linux-gnu
- run: cross build --release --target aarch64-unknown-linux-gnu
```

### Caching Strategies for Cargo Dependencies

#### Recommended: Swatinem/rust-cache

**Tool:** [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache) - GitHub Marketplace Action

**What Gets Cached:**
- `~/.cargo` - Registry index, git dependencies, and cached builds
- `./target` - Compiled artifacts and build outputs (but NOT `target/doc`)

**Cache Key Determinants:**
- Hashes of all `Cargo.lock` and `Cargo.toml` files
- Rust toolchain version in use
- `.cargo/config.toml` modifications

**Key Features:**
- Automatically cleans up old binaries and artifacts older than 1 week
- Removes incremental compilation artifacts before saving
- Supports `prefix-key` for manual cache invalidation
- Enables `shared-key` for stable keys across different jobs

**Basic Configuration:**
```yaml
- uses: Swatinem/rust-cache@v2
  with:
    # Optional: specify a prefix for cache key
    prefix-key: ${{ matrix.os }}
    # Optional: enable shared key across jobs
    shared-key: build
```

#### Alternative: sccache for Compiler Caching

**Tool:** [sccache](https://github.com/mozilla/sccache)

**Benefits:**
- Compiler-level caching (faster than artifact caching)
- Supports GitHub Actions cache backend
- Enables incremental compilation in CI

**Configuration:**
```yaml
env:
  RUSTC_WRAPPER: sccache
  SCCACHE_GHA_ENABLED: true
  ACTIONS_CACHE_URL: ${{ github.server_url }}/repos/${{ github.repository }}/actions/cache
  ACTIONS_RUNTIME_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Matrix Builds for Different Rust Versions

**Recommended Versions to Test:**
- **Stable**: Latest stable release (default)
- **MSRV** (Minimum Supported Rust Version): Often pinned (e.g., 1.70.0)
- **Beta**: Pre-release testing
- **Nightly**: For experimental features or miri

**Multi-Version Matrix:**
```yaml
strategy:
  matrix:
    rust: [1.70.0, stable, beta, nightly]  # Example versions
    exclude:
      # Skip certain combinations to reduce CI time
      - rust: beta
        os: windows-latest
runs-on: ubuntu-latest

steps:
  - uses: dtolnay/rust-toolchain@master
    with:
      toolchain: ${{ matrix.rust }}
```

**Real-World Example:** Ripgrep tests against pinned 1.85.0, stable, beta, and nightly across multiple Linux architectures, macOS, and Windows variants.

---

## Testing and Quality Gates

### cargo test with Coverage

#### Running Tests

**Basic Test Execution:**
```yaml
- name: Run tests
  run: cargo test --verbose --workspace --all-features
```

**Parallel Testing with cargo-nextest:**

[cargo-nextest](https://nexte.st/) provides 3x faster test execution through parallel process isolation.

```yaml
- name: Install cargo-nextest
  uses: taiki-e/install-action@nextest

- name: Run tests with nextest
  run: cargo nextest run --all-features
```

**Benefits of nextest:**
- Each test runs in isolated process (true parallelism)
- One test failure doesn't abort entire suite
- JUnit XML export support
- Test retries on failure
- Archive and partition across workers

#### Code Coverage

**Tool:** cargo-tarpaulin

```yaml
- name: Generate coverage
  uses: taiki-e/install-action@cargo-tarpaulin

- run: cargo tarpaulin --workspace --out Xml --exclude-files tests/*

- name: Upload coverage
  uses: codecov/codecov-action@v3
  with:
    files: ./cobertura.xml
```

### cargo clippy for Linting

**Static Linting with Clippy:**

```yaml
- name: Check with clippy
  run: cargo clippy --all-targets --all-features -- -D warnings
```

**Key Flags:**
- `--all-targets` - Check lib, bin, tests, examples
- `--all-features` - Enable all feature combinations
- `-D warnings` - Deny clippy warnings (fail on any)

### cargo fmt for Formatting Checks

**Format Verification:**

```yaml
- name: Check formatting
  run: cargo fmt --all -- --check
```

**Key Flags:**
- `--all` - Check all source files
- `--check` - Verify without modifying (fail if changes needed)

### Security Audits with cargo-audit

#### Official RustSec GitHub Actions

**Primary Option:** [rustsec/audit-check](https://github.com/rustsec/audit-check)

```yaml
- name: Run security audit
  uses: rustsec/audit-check-action@v1
  with:
    token: ${{ secrets.GITHUB_TOKEN }}
```

**Secondary Option:** [actions-rust-lang/audit](https://github.com/actions-rust-lang/audit)

```yaml
- name: Audit Rust dependencies
  uses: actions-rust-lang/audit@v1
```

**Configuration for Dependency Changes Only:**

```yaml
- name: Run cargo audit
  run: cargo audit
  # Only run if Cargo.lock changed
  if: |
    contains(github.event.head_commit.modified, 'Cargo.lock') ||
    contains(github.event.pull_request.changed_files, 'Cargo.lock')
```

**Scheduled Audits:**

```yaml
on:
  schedule:
    # Daily audit at midnight
    - cron: '0 0 * * *'
```

---

## Release Automation

### Semantic Versioning with release-plz

#### Overview

[release-plz](https://release-plz.dev/) automates the entire release workflow by analyzing Conventional Commits and detecting API breaking changes.

**Workflow:**
1. `release-plz release-pr` - Creates PR with version bumps and changelog
2. Manual merge of PR triggers automatic release
3. `release-plz release` - Tags, creates GitHub release, publishes to crates.io

#### Installation & Configuration

```yaml
# In Cargo.toml [package]
publish = true
description = "..."
license = "MIT OR Apache-2.0"
```

#### GitHub Action Setup

```yaml
name: Release

on:
  push:
    branches: [main]

jobs:
  release-plz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: dtolnay/rust-toolchain@stable

      - name: Install release-plz
        run: cargo install --locked release-plz

      - name: Create release PR
        run: release-plz release-pr --backend github --token ${{ secrets.GITHUB_TOKEN }}

      - name: Publish release
        run: release-plz release --backend github --token ${{ secrets.GITHUB_TOKEN }}
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_TOKEN }}
```

#### Key Features

- **Semantic Versioning**: Automatic version bumping based on commit messages
- **API Breaking Detection**: Uses cargo-semver-checks to detect breaking changes
- **Conventional Commits**: Follows standard commit message format
- **Changelog Generation**: Uses git-cliff with Keep a Changelog format
- **Workspace Support**: No configuration needed for cargo workspaces
- **Multiple Backends**: GitHub, Gitea, and GitLab support

### Creating GitHub Releases with Artifacts

#### Using taiki-e/upload-rust-binary-action

**Combined Release Workflow:**

```yaml
name: Release Binaries

on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+'

jobs:
  create-release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: taiki-e/create-gh-release-action@v1
        with:
          # Extract version from tag
          changelog: CHANGELOG.md
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  upload-assets:
    strategy:
      matrix:
        include:
          - { os: ubuntu-latest, target: x86_64-unknown-linux-gnu }
          - { os: macos-latest, target: x86_64-apple-darwin }
          - { os: macos-latest, target: aarch64-apple-darwin }
          - { os: windows-latest, target: x86_64-pc-windows-msvc }

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4

      - uses: taiki-e/upload-rust-binary-action@v1
        with:
          bin: my-app
          target: ${{ matrix.target }}
          archive: $bin-$tag-$target
          checksum: sha256
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Binary Distribution Strategies

#### Multi-Platform Archive Strategy

**Output Structure:**
```
my-app-v1.0.0-x86_64-unknown-linux-gnu.tar.gz
├── my-app (binary)
├── README.md
├── CHANGELOG.md
└── man/my-app.1

my-app-v1.0.0-x86_64-unknown-linux-gnu.tar.gz.sha256
```

**Implementation Example (from ripgrep):**

```yaml
- name: Build release artifacts
  run: |
    cargo build --release --target ${{ matrix.target }}
    # Generate shell completions
    cargo run --release -- --generate-shell-completion bash > completions/my-app.bash
    cargo run --release -- --generate-shell-completion zsh > completions/_my-app
    cargo run --release -- --generate-shell-completion fish > completions/my-app.fish

- name: Create release archive
  run: |
    mkdir -p release
    cp target/${{ matrix.target }}/release/my-app release/
    cp CHANGELOG.md README.md completions/* release/
    tar czf my-app-v${{ env.VERSION }}-${{ matrix.target }}.tar.gz release/
    sha256sum my-app-v${{ env.VERSION }}-${{ matrix.target }}.tar.gz > checksums.txt
```

#### Debian Package Generation

**Tool:** cargo-deb

```yaml
- name: Install cargo-deb
  run: cargo install cargo-deb

- name: Create .deb package
  run: cargo deb --target x86_64-unknown-linux-gnu
```

**Cargo.toml Configuration:**
```toml
[package.metadata.deb]
maintainer = "Your Name <you@example.com>"
copyright = "2025, Your Name <you@example.com>"
license-file = ["LICENSE", "4"]
extended-description = "Long description here"
assets = [
    ["target/x86_64-unknown-linux-gnu/release/my-app", "usr/bin/", "755"],
]
```

### Changelog Generation

**Automatic with release-plz:**
- Uses [git-cliff](https://github.com/orhun/git-cliff)
- Reads Conventional Commits
- Generates CHANGELOG.md in Keep a Changelog format
- Automatically included in release PRs

**Manual Alternative (git-cliff):**

```yaml
- name: Generate changelog
  uses: orhun/git-cliff-action@v2
  with:
    config: cliff.toml
    args: --verbose
  env:
    OUTPUT: CHANGELOG.md
```

---

## Continuous Deployment

### Publishing to crates.io

#### With release-plz (Recommended)

```yaml
- name: Publish to crates.io
  run: cargo publish --token ${{ secrets.CARGO_TOKEN }}
```

**Requirements:**
- `CARGO_TOKEN` secret configured in repository
- Package published to crates.io before
- Version incremented in Cargo.toml

#### Manual Publish Workflow

```yaml
name: Publish Crate

on:
  release:
    types: [published]

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Publish to crates.io
        run: cargo publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_TOKEN }}
```

### Creating Platform-Specific Binaries

**Matrix Strategy for Binaries:**

```yaml
strategy:
  matrix:
    include:
      # Linux (musl for static linking)
      - os: ubuntu-latest
        target: x86_64-unknown-linux-musl
        binary_suffix: ""

      # macOS Intel
      - os: macos-latest
        target: x86_64-apple-darwin
        binary_suffix: ""

      # macOS ARM64
      - os: macos-latest
        target: aarch64-apple-darwin
        binary_suffix: ""

      # Windows
      - os: windows-latest
        target: x86_64-pc-windows-msvc
        binary_suffix: .exe

steps:
  - uses: dtolnay/rust-toolchain@stable
    with:
      targets: ${{ matrix.target }}

  - name: Build binary
    run: cargo build --release --target ${{ matrix.target }}
```

### Container Image Builds (OCI/Docker)

#### Multi-Stage Docker Build

**Dockerfile:**
```dockerfile
# Build stage
FROM rust:latest as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/my-app /usr/local/bin/

ENTRYPOINT ["my-app"]
```

#### GitHub Actions Container Build

```yaml
name: Build and Push Container

on:
  push:
    branches: [main]
    tags: ['v*']

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: docker/setup-buildx-action@v2

      - uses: docker/login-action@v2
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - uses: docker/build-push-action@v4
        with:
          context: .
          push: true
          tags: ghcr.io/${{ github.repository }}:latest
          platforms: linux/amd64,linux/arm64
          cache-from: type=registry,ref=ghcr.io/${{ github.repository }}:buildcache
          cache-to: type=registry,ref=ghcr.io/${{ github.repository }}:buildcache,mode=max
```

---

## Performance and Cost Optimization

### Workflow Caching Best Practices

#### 1. Cache Invalidation Strategy

**Automatic via Swatinem/rust-cache:**
- Based on Cargo.lock hash
- Detects toolchain changes
- Handles .cargo/config.toml modifications

**Manual Invalidation:**
```yaml
- uses: Swatinem/rust-cache@v2
  with:
    prefix-key: "v1"  # Increment to invalidate all caches
```

#### 2. Cache Size Management

**Recommendations:**
- Cache size limit: ~5-10 GB per repository
- GitHub default: 5 GB, upgradeable to 10 GB
- Weekly cleanup removes artifacts >1 week old automatically
- Remove unnecessary files with cargo-sweep:

```yaml
- name: Clean cache
  run: cargo sweep -r -i 7d
```

#### 3. Shared Cache Keys

**Use for stable builds across multiple jobs:**
```yaml
- uses: Swatinem/rust-cache@v2
  with:
    shared-key: "shared-cache"
    prefix-key: ${{ runner.os }}
```

### Incremental Compilation

#### For CI: DISABLE Incremental Compilation

**Reason:** CI builds are typically from-scratch, and incremental compilation adds overhead.

```yaml
env:
  CARGO_INCREMENTAL: 0
```

#### For Local Development: ENABLE Incremental

Developers should keep `CARGO_INCREMENTAL=1` (default) for faster iterative builds.

### Parallel Job Execution

#### Matrix Strategies

**Maximize parallelism with matrices:**
```yaml
strategy:
  matrix:
    rust: [1.70.0, stable, nightly]
    os: [ubuntu-latest, macos-latest, windows-latest]
    exclude:
      # Reduce combinations to manage costs
      - rust: nightly
        os: windows-latest
```

#### Job Dependencies

**Reduce unnecessary runs:**
```yaml
jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - run: cargo clippy -- -D warnings

  test:
    needs: lint  # Only run if lint passes
    runs-on: ubuntu-latest
    steps:
      - run: cargo test

  release:
    needs: [lint, test]  # Both must pass
    runs-on: ubuntu-latest
    steps:
      - run: cargo publish
```

#### Parallel Test Execution

**Use cargo-nextest:**
```yaml
- name: Install nextest
  uses: taiki-e/install-action@nextest

- run: cargo nextest run --all-features --test-threads=num_cpus
```

**Cost Savings:**
- 3x faster test execution = lower CI minutes used
- Example: 3-minute test suite becomes 1 minute

### Optimization Checklist

- [ ] **Caching:** Using Swatinem/rust-cache with appropriate keys
- [ ] **Incremental:** `CARGO_INCREMENTAL=0` in CI environment
- [ ] **Parallel Tests:** Using cargo-nextest for test parallelism
- [ ] **Matrix Reduction:** Excluding unnecessary OS/Rust version combinations
- [ ] **Job Dependencies:** Fail fast with `needs:` relationships
- [ ] **Workspace Caching:** Shared cache keys for workspace members
- [ ] **Linker:** Consider faster linker (lld) for reduced build time
- [ ] **Feature Matrix:** Only test necessary feature combinations

---

## Exemplary Repository Workflows

### 1. ripgrep - BurntSushi/ripgrep

**Repository:** [github.com/BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep)

**Workflow Files:**
- Main CI: [.github/workflows/ci.yml](https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/ci.yml)
- Release: [.github/workflows/release.yml](https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/release.yml)

**Key Characteristics:**

**CI Workflow (ci.yml):**
- Tests on **pinned (1.85.0), stable, beta, nightly** Rust versions
- Multiple Linux targets: x86_64 (musl), i686, aarch64, armv7 (gnueabihf/musleabihf), powerpc64, s390x, riscv64gc
- macOS with nightly
- Windows with MSVC and GNU toolchains
- WASM compilation testing (wasm32-wasip1)
- Rustfmt verification
- Documentation checks with `-D warnings`
- Fuzz target compilation
- Uses `dtolnay/rust-toolchain@master` for installation
- Cross compilation via `cross` v0.2.5

**Release Workflow (release.yml):**
- Triggers on version tags matching `[0-9]+.[0-9]+.[0-9]+`
- Verifies tag matches Cargo.toml version
- Creates draft GitHub release
- Builds binaries across 10+ platform combinations
- Generates shell completions (bash, fish, zsh, PowerShell)
- Creates man pages
- Produces `.deb` packages for Linux
- Generates SHA256 checksums
- Supports PCRE2 optional feature

**Lessons:**
- Comprehensive platform coverage while managing CI time
- Separate CI and release workflows
- Feature flag testing (PCRE2)
- Shell completion generation in release

### 2. Tokio - tokio-rs/tokio

**Repository:** [github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio)

**Key Characteristics:**
- Complex async runtime requiring thorough testing
- Multi-workspace project
- Tests on stable, beta, nightly channels
- Feature matrix testing (parking_lot, unstable features)
- Uses `miri` for undefined behavior detection
- Cross compilation tests
- Doctests for unstable APIs

### 3. Clap - clap-rs/clap

**Repository:** [github.com/clap-rs/clap](https://github.com/clap-rs/clap)

**Key Characteristics:**
- CLI argument parser with broad feature support
- Release workflow uses taiki-e actions
- Multi-platform binary distribution
- Comprehensive feature matrix

**Release Strategy:**
- Uses `taiki-e/create-gh-release-action@v1` for release creation
- Uses `taiki-e/upload-rust-binary-action@v1` for binary upload
- Supports checksum generation
- Cross-compilation with `taiki-e/setup-cross-toolchain-action@v1`

### 4. Serde - serde-rs/serde

**Repository:** [github.com/serde-rs/serde](https://github.com/serde-rs/serde)

**Key Characteristics:**
- Complex serialization library
- Unified stable/beta CI workflow using matrix
- Uses `dtolnay/rust-toolchain@master` with matrix-specified versions
- Feature-gated compilation tests

---

## Complete Workflow Examples

### Example 1: Complete CI Workflow

**File:** `.github/workflows/ci.yml`

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 0 * * 0'  # Weekly

env:
  RUST_BACKTRACE: 1
  CARGO_INCREMENTAL: 0

jobs:
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --all-targets --all-features -- -D warnings

  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - { os: ubuntu-latest, target: x86_64-unknown-linux-gnu }
          - { os: macos-latest, target: x86_64-apple-darwin }
          - { os: windows-latest, target: x86_64-pc-windows-msvc }
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: Swatinem/rust-cache@v2

      - name: Run tests
        run: cargo test --target ${{ matrix.target }}

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --workspace --out Xml --exclude-files tests/*

      - name: Upload to codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml

  audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check-action@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

### Example 2: Release Workflow

**File:** `.github/workflows/release.yml`

```yaml
name: Release

on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+'

jobs:
  create-release:
    name: Create Release
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: taiki-e/create-gh-release-action@v1
        with:
          changelog: CHANGELOG.md
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  upload-assets:
    name: Upload Assets
    needs: create-release
    strategy:
      matrix:
        include:
          - { os: ubuntu-latest, target: x86_64-unknown-linux-gnu }
          - { os: ubuntu-latest, target: x86_64-unknown-linux-musl }
          - { os: ubuntu-latest, target: aarch64-unknown-linux-gnu }
          - { os: macos-latest, target: x86_64-apple-darwin }
          - { os: macos-latest, target: aarch64-apple-darwin }
          - { os: windows-latest, target: x86_64-pc-windows-msvc }
    runs-on: ${{ matrix.os }}
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - uses: Swatinem/rust-cache@v2

      - uses: taiki-e/upload-rust-binary-action@v1
        with:
          bin: my-app
          target: ${{ matrix.target }}
          archive: $bin-$tag-$target
          checksum: sha256
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

  publish-crate:
    name: Publish to Crates.io
    runs-on: ubuntu-latest
    needs: upload-assets
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo publish
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_TOKEN }}
```

### Example 3: Automated Release with release-plz

**File:** `.github/workflows/release-plz.yml`

```yaml
name: Release Plz

on:
  push:
    branches: [main]

jobs:
  release-plz:
    name: Release Plz
    runs-on: ubuntu-latest
    permissions:
      pull-requests: write
      contents: write
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: dtolnay/rust-toolchain@stable

      - name: Install release-plz
        run: cargo install --locked release-plz

      - name: Create release PR
        run: release-plz release-pr
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Publish release
        run: release-plz release
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_TOKEN }}
```

### Example 4: Multi-Architecture Container Build

**File:** `.github/workflows/build-container.yml`

```yaml
name: Build Container

on:
  push:
    branches: [main]
    tags: ['v*']

jobs:
  build:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write

    steps:
      - uses: actions/checkout@v4

      - uses: docker/setup-buildx-action@v2

      - uses: docker/login-action@v2
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - uses: docker/metadata-action@v4
        id: meta
        with:
          images: ghcr.io/${{ github.repository }}
          tags: |
            type=ref,event=branch
            type=semver,pattern={{version}}
            type=sha

      - uses: docker/build-push-action@v4
        with:
          context: .
          platforms: linux/amd64,linux/arm64
          push: ${{ github.event_name != 'pull_request' }}
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=registry,ref=ghcr.io/${{ github.repository }}:buildcache
          cache-to: type=registry,ref=ghcr.io/${{ github.repository }}:buildcache,mode=max
```

---

## Tools and Actions Reference

### Essential Tools

| Tool | Purpose | Install | Reference |
|------|---------|---------|-----------|
| **dtolnay/rust-toolchain** | Rust installation | GitHub Action | [Marketplace](https://github.com/marketplace/actions/install-rust) |
| **Swatinem/rust-cache** | Dependency caching | GitHub Action | [GitHub](https://github.com/Swatinem/rust-cache) |
| **cargo-nextest** | Parallel testing | `taiki-e/install-action@nextest` | [Website](https://nexte.st/) |
| **cargo-tarpaulin** | Code coverage | `taiki-e/install-action@cargo-tarpaulin` | [GitHub](https://github.com/xd009642/tarpaulin) |
| **cargo-clippy** | Linting | Built-in (install with rustfmt) | [Docs](https://doc.rust-lang.org/clippy/) |
| **cargo-fmt** | Formatting | Built-in (install with rustfmt) | [Docs](https://github.com/rust-lang/rustfmt) |
| **cargo-audit** | Security audit | via rustsec/audit-check | [GitHub](https://github.com/rustsec/rustsec) |
| **cross** | Cross-compilation | `taiki-e/setup-cross-toolchain-action` | [GitHub](https://github.com/cross-rs/cross) |
| **release-plz** | Release automation | `cargo install --locked release-plz` | [Website](https://release-plz.dev/) |

### GitHub Actions

| Action | Purpose | URL |
|--------|---------|-----|
| **taiki-e/create-gh-release-action** | Create GitHub release | [Marketplace](https://github.com/marketplace/actions/create-gh-release) |
| **taiki-e/upload-rust-binary-action** | Upload binaries to release | [Marketplace](https://github.com/marketplace/actions/upload-rust-binary-to-github-releases) |
| **taiki-e/install-action** | Install dev tools quickly | [Marketplace](https://github.com/marketplace/actions/install-action) |
| **taiki-e/setup-cross-toolchain-action** | Setup cross-compilation | [Marketplace](https://github.com/marketplace/actions/setup-cross-toolchain) |
| **rustsec/audit-check-action** | Security audit | [Marketplace](https://github.com/marketplace/actions/rust-audit-check) |
| **actions-rust-lang/audit** | Dependency audit | [Marketplace](https://github.com/marketplace/actions/audit-rust-dependencies) |
| **docker/setup-buildx-action** | Docker BuildKit | [Marketplace](https://github.com/marketplace/actions/docker-setup-buildx) |
| **docker/build-push-action** | Build and push Docker | [Marketplace](https://github.com/marketplace/actions/build-and-push-docker-images) |
| **codecov/codecov-action** | Coverage reporting | [Marketplace](https://github.com/marketplace/actions/codecov) |

---

## Recommended Workflow Structure

### For Library Crates

**Three-workflow approach:**

1. **ci.yml** - Lint, format, test on multiple versions
2. **release-plz.yml** - Automated semantic versioning and changelog
3. **publish.yml** - Triggered on release creation

### For Binary Crates

**Four-workflow approach:**

1. **ci.yml** - Lint, format, test on multiple platforms
2. **release-plz.yml** - Automated version management
3. **release.yml** - Build and upload binaries to GitHub releases
4. **docker.yml** - Build and push container images (if applicable)

### Cost Optimization Tips

1. **Use matrix efficiently** - Skip expensive combinations (e.g., don't test all Rust versions on all OSes)
2. **Enable caching** - Always use Swatinem/rust-cache
3. **Fail fast** - Use job dependencies to skip unnecessary work
4. **Test selectively** - Only run full test suite on main branch, faster subset on PRs
5. **Leverage nightly** - Use nightly only for feature/miri jobs, not full matrix
6. **Scheduled audits** - Run expensive security checks on schedule, not every push

---

## References and Resources

### Official Documentation

- [The Cargo Book - Continuous Integration](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
- [GitHub Actions - Building and testing Rust](https://docs.github.com/en/actions/tutorials/build-and-test-code/rust)
- [Docker and Rust Best Practices](https://docs.docker.com/guides/rust/configure-ci-cd/)

### Comprehensive Guides

- [Shuttle.dev - Setup Rust CI/CD in 2025](https://www.shuttle.dev/blog/2025/01/23/setup-rust-ci-cd)
- [LogRocket - Optimizing CI/CD Pipelines in Rust](https://blog.logrocket.com/optimizing-ci-cd-pipelines-rust-projects/)
- [Markaicode - Rust CI/CD Pipeline Setup Comparison 2025](https://markaicode.com/rust-cicd-pipeline-setup-comparison-2025/)
- [Corrode - Tips for Faster Rust CI Builds](https://corrode.dev/blog/tips-for-faster-ci-builds/)

### Tool Documentation

- [cargo-nextest](https://nexte.st/)
- [release-plz](https://release-plz.dev/)
- [RustSec Advisory Database](https://rustsec.org/)
- [cross-rs](https://github.com/cross-rs/cross)
- [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache)

### Related Articles

- [DEV Community - Rust CI with GitHub Actions](https://dev.to/bampeers/rust-ci-with-github-actions-1ne9)
- [How to Deploy Rust Binaries with GitHub Actions](https://dzfrias.dev/blog/deploy-rust-cross-platform-github-actions/)
- [Multi-platform Rust Building](https://jondot.medium.com/building-rust-on-multiple-platforms-using-github-6f3e6f8b8458)
- [Uffizzi - Optimizing Rust Builds for GitHub Actions](https://www.uffizzi.com/blog/optimizing-rust-builds-for-faster-github-actions-pipelines)

---

## Version Notes

- **Document Date:** December 2025
- **Research Focus:** GitHub Actions workflows and best practices for Rust projects
- **Key Tools Covered:** cargo-nextest, release-plz, Swatinem/rust-cache, taiki-e actions
- **Exemplary Projects:** ripgrep, tokio, clap, serde

---

## Document Metadata

- **Research Completed:** December 27, 2025
- **Sources Reviewed:** 40+ articles, official documentation, real-world repositories
- **Key Contributors:** GitHub Actions community, Rust core team documentation
- **Repository Examples:** BurntSushi/ripgrep, tokio-rs/tokio, clap-rs/clap, serde-rs/serde
