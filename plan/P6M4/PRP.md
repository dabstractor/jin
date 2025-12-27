# PRP: P6.M4 - Release Preparation (CI/CD & Build Configuration)

---

## Goal

**Feature Goal**: Establish production-ready CI/CD infrastructure for Jin that automates testing, quality checks, and multi-platform binary releases with zero manual intervention after tagging a version.

**Deliverable**: Complete GitHub Actions CI/CD pipeline consisting of:
1. **CI Workflow** - Automated testing, linting, formatting, and security checks on every PR/push
2. **Release Workflow** - Automated multi-platform binary builds, GitHub releases, and changelog generation on version tags
3. **Configuration Files** - cargo-dist setup, release-plz config, and release profile optimization

**Success Definition**:
- CI workflow runs on every PR and blocks merging if tests/checks fail
- Release workflow triggers on `v*` tags and produces multi-platform binaries (Linux, macOS, Windows)
- GitHub releases auto-generated with changelogs from conventional commits
- Binary distribution includes installers (shell script, PowerShell, Homebrew formula)
- Security attestations enabled via GitHub Artifact Attestations
- All 7+ integration test files pass in CI (core_workflow, mode_scope_workflow, sync_workflow, atomic_operations, error_scenarios, cli_basic)
- Release process takes <5 minutes of human time (tag → push → automated release)

## User Persona

**Target User**: Maintainer of Jin who needs to release new versions to users

**Use Case**: Publishing a new Jin version with bug fixes or features to users across multiple platforms

**User Journey**:
1. Developer completes feature work and commits following conventional commits
2. Developer runs `git tag v0.2.0` to tag new version
3. Developer runs `git push --tags` to trigger release
4. GitHub Actions automatically: builds binaries for 6+ platforms, generates changelog, creates GitHub release, uploads artifacts
5. Users can install via `cargo install jin`, Homebrew, or download platform binaries
6. Developer verifies release on GitHub releases page (~10 minutes after push)

**Pain Points Addressed**:
- **Manual release toil**: No more manual binary compilation for multiple platforms
- **Changelog tedium**: Auto-generated from conventional commits via git-cliff
- **Version management**: release-plz handles semantic versioning automatically
- **Quality gates**: Can't merge broken code - CI enforces test passing, linting, formatting
- **Distribution complexity**: cargo-dist handles installers, checksums, and GitHub releases
- **Security concerns**: GitHub Attestations provide supply chain verification

## Why

**Business Value**:
- **Faster releases**: Automation reduces release time from hours to minutes
- **Higher quality**: Automated testing catches bugs before they reach users
- **Multi-platform support**: Jin runs on Linux, macOS, Windows without manual builds
- **User trust**: Security attestations and checksums prove artifact integrity
- **Maintainer efficiency**: Focus on features, not release mechanics

**Integration with Existing Features**:
- Runs comprehensive integration test suite from P6.M2 (25+ test cases)
- Validates shell completions from P6.M1 work correctly
- Ensures documentation from P6.M3 stays synchronized with code
- Tests all 33 Jin commands across core, mode, scope, sync workflows

**Problems This Solves**:
- Prevents regressions by running all tests on every change
- Catches formatting/linting issues early in PR review
- Eliminates "works on my machine" with consistent CI environment
- Provides trusted binaries for users who can't/won't build from source
- Enables rapid iteration with confidence via automated quality gates

## What

**Automated CI/CD Pipeline with 2 GitHub Actions Workflows:**

### 1. CI Workflow (`.github/workflows/ci.yml`)
Runs on: Every push to main, every PR
- **Multi-platform testing**: Linux (Ubuntu latest), macOS (latest), Windows (latest)
- **Rust version matrix**: Stable, 1.70.0 (MSRV)
- **Quality gates**:
  - `cargo test --all-features` - Run all unit and integration tests
  - `cargo clippy -- -D warnings` - Lint code, fail on warnings
  - `cargo fmt --check` - Enforce formatting
  - `cargo audit` - Security vulnerability scanning
- **Performance optimization**:
  - Swatinem/rust-cache for dependency caching (~80% speedup)
  - Parallel job execution across platforms
  - cargo-nextest for 3x faster test execution

### 2. Release Workflow (`.github/workflows/release.yml`)
Runs on: Tags matching `v*` pattern (e.g., `v0.2.0`)
- **Automated versioning**: release-plz handles version bumping from conventional commits
- **Multi-platform binary builds** via cargo-dist:
  - Linux: x86_64-unknown-linux-gnu, x86_64-unknown-linux-musl
  - macOS: x86_64-apple-darwin, aarch64-apple-darwin (Intel + Apple Silicon)
  - Windows: x86_64-pc-windows-msvc
- **Installer generation**:
  - Shell installer script (Linux/macOS)
  - PowerShell installer script (Windows)
  - Homebrew formula template
- **GitHub Release creation**:
  - Auto-generated changelog via git-cliff from conventional commits
  - Binary uploads with SHA256 checksums
  - GitHub Artifact Attestations for supply chain security
- **Cargo publishing**: Automated `cargo publish` to crates.io

### 3. Configuration Files

**Cargo.toml** (already has release profile, verify optimization):
```toml
[profile.release]
lto = true           # Link-time optimization
opt-level = 3        # Maximum optimization
strip = true         # Strip debug symbols for smaller binaries
```

**dist.toml** (cargo-dist configuration):
```toml
[workspace.metadata.dist]
dist-version = "0.30.2"
ci = ["github"]
installers = ["shell", "powershell", "homebrew"]
targets = [
  "x86_64-unknown-linux-gnu",
  "x86_64-unknown-linux-musl",
  "x86_64-apple-darwin",
  "aarch64-apple-darwin",
  "x86_64-pc-windows-msvc"
]
install-path = "~/.cargo/bin"
```

**release-plz.toml** (semantic versioning):
```toml
[workspace]
changelog_update = true
git_tag_enable = true

[[package]]
name = "jin"
changelog_include = ["feat", "fix", "perf", "refactor", "docs"]
```

### Success Criteria

- [ ] CI workflow passes on all 3 platforms (Linux, macOS, Windows)
- [ ] CI workflow blocks PR merging if any check fails
- [ ] Release workflow builds binaries for 5+ target platforms
- [ ] GitHub releases include auto-generated changelog with conventional commit sections
- [ ] Binary downloads work on fresh systems (test installation flow)
- [ ] Security attestations visible on GitHub releases
- [ ] cargo publish succeeds and Jin installable via `cargo install jin`
- [ ] CI caching reduces build time by 60%+ vs. uncached
- [ ] All existing tests pass in CI (25+ integration test cases)
- [ ] Release process completes in <15 minutes from tag push to published release

## All Needed Context

### Context Completeness Check

_"If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"_

**Yes** - This PRP provides:
- Specific GitHub Actions workflow configurations
- Exact tool versions and configurations (cargo-dist 0.30.2, release-plz, git-cliff)
- Platform targets and installer types
- Integration with existing test infrastructure (assert_cmd, predicates, tempfile)
- Step-by-step implementation tasks with dependencies
- Validation commands to verify each component

### Documentation & References

```yaml
# MUST READ - Essential CI/CD Research

- file: plan/P6M4/research/rust_cicd_best_practices.md
  why: Comprehensive GitHub Actions patterns for Rust - multi-platform builds, caching strategies, testing
  critical: Swatinem/rust-cache is the #1 performance optimization (saves 5-10min per build)
  gotcha: Use CARGO_TERM_COLOR=always for readable CI logs

- file: plan/P6M4/research/release_automation.md
  why: release-plz vs cargo-release comparison, conventional commits → SemVer mapping
  critical: release-plz is recommended for Rust-native semantic versioning (better than semantic-release)
  gotcha: Requires conventional commits format (feat:, fix:, BREAKING CHANGE:)

- file: plan/P6M4/research/cargo_dist_guide.md
  why: cargo-dist setup for multi-platform binary distribution, installer generation
  critical: cargo-dist handles GitHub releases, installers, checksums automatically
  gotcha: Must run `cargo dist init` to generate initial config and workflow

- file: plan/P6M4/research/workflow_examples.md
  why: Production-ready workflow examples from ripgrep, starship, bat, fd
  pattern: Complete GitHub Actions YAML to reference for structure
  critical: ripgrep pattern (line 45) shows multi-arch release automation

- url: https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/release.yml
  why: Real-world release workflow from popular Rust CLI tool
  critical: Shows cross-compilation matrix setup, artifact upload patterns

- url: https://github.com/starship/starship/blob/master/.github/workflows/release.yml
  why: Full automation example with 5-stage pipeline
  critical: Demonstrates cargo-dist integration with release-plz

- url: https://axodotdev.github.io/cargo-dist/book/
  why: Official cargo-dist documentation - setup, configuration, best practices
  critical: "Quick Start" section (init → plan → build workflow)

- url: https://release-plz.dev/docs/usage/config
  why: release-plz configuration reference - changelog, version bumping
  critical: Conventional commits format mapping to semantic versions

- url: https://git-cliff.org/docs/
  why: Changelog generation from Git commits
  critical: Template customization for grouping commits by type (feat, fix, etc.)

# MUST REVIEW - Existing Test Infrastructure

- file: tests/core_workflow.rs
  why: Integration test patterns using assert_cmd and predicates
  pattern: TestFixture pattern for isolated Git repositories in tests
  critical: Tests use `Command::cargo_bin("jin")` for CLI testing

- file: tests/common/fixtures.rs
  why: Test fixture utilities - setup helpers for isolated test environments
  pattern: TempDir usage, jin_init(), setup_test_repo() patterns
  critical: TempDir MUST be kept in scope or directory deleted prematurely

- file: tests/common/assertions.rs
  why: Custom assertion helpers for Jin-specific validations
  pattern: assert_jin_initialized(), assert_context_mode(), assert_staging_contains()

- file: Cargo.toml
  why: Existing dependencies and release profile
  pattern: dev-dependencies section shows test crates (assert_cmd 2.0, predicates 3.0, tempfile 3.0)
  critical: Release profile already optimized (lto=true, opt-level=3)

# MUST UNDERSTAND - Project Context

- file: PRD.md
  why: Project requirements and scope boundaries
  critical: "CI/CD integration (Jin is for local development workspace only)" means CI/CD is for building Jin itself, NOT for Jin to provide CI/CD features to users
  gotcha: Jin tracks ignored files, runs alongside Git - ensure CI doesn't conflict with this design
```

### Current Codebase Tree

```bash
jin/
├── .github/              # DOES NOT EXIST YET - to be created
│   └── workflows/
│       ├── ci.yml        # CI workflow (test, lint, format, audit)
│       └── release.yml   # Release workflow (build, publish)
├── Cargo.toml            # HAS release profile, NEEDS cargo-dist metadata
├── Cargo.lock
├── src/
│   ├── main.rs           # CLI entry point
│   ├── lib.rs
│   ├── cli/              # Clap CLI definitions
│   ├── commands/         # Command implementations (33 commands)
│   ├── core/             # Core types (Config, Context, Layer, etc.)
│   ├── git/              # Git operations (JinRepo wrapper)
│   ├── merge/            # Merge engine (deep merge, text merge)
│   ├── commit/           # Commit pipeline
│   └── staging/          # Staging system
├── tests/
│   ├── core_workflow.rs          # 10+ tests
│   ├── mode_scope_workflow.rs    # 8+ tests
│   ├── sync_workflow.rs          # 6+ tests
│   ├── atomic_operations.rs      # 7+ tests
│   ├── error_scenarios.rs        # 8+ tests
│   ├── cli_basic.rs              # Basic CLI tests
│   └── common/
│       ├── mod.rs
│       ├── fixtures.rs           # Test setup utilities
│       └── assertions.rs         # Custom assertions
├── README.md             # User documentation (P6.M3 complete)
├── docs/                 # Detailed documentation
└── plan/
    └── P6M4/
        └── research/     # Research on CI/CD best practices
```

### Desired Codebase Tree with Files to be Added

```bash
jin/
├── .github/              # NEW - GitHub Actions configuration
│   └── workflows/
│       ├── ci.yml        # NEW - CI workflow (15-20 min runtime)
│       │                 # Responsibility: Run tests, linting, formatting, security checks
│       │                 # Triggers: push to main, pull_request
│       │                 # Platforms: Linux, macOS, Windows
│       │                 # Jobs: test (matrix), lint, format, audit
│       │
│       └── release.yml   # NEW - Release workflow (30-45 min runtime)
│                         # Responsibility: Build multi-platform binaries, create GitHub release
│                         # Triggers: tags matching v*
│                         # Uses: cargo-dist for builds, release-plz for versioning
│                         # Outputs: GitHub release with binaries, changelog, checksums
│
├── Cargo.toml            # MODIFIED - Add cargo-dist metadata section
│                         # Responsibility: Configure cargo-dist targets and installers
│
├── release-plz.toml      # NEW - release-plz configuration
│                         # Responsibility: Define semantic versioning rules, changelog generation
│
└── cliff.toml            # NEW (optional) - git-cliff changelog config
                          # Responsibility: Customize changelog format and commit grouping
```

### Known Gotchas of Our Codebase & Library Quirks

```bash
# CRITICAL: Jin uses Git repository isolation for tests
# Tests create isolated TempDir environments with full Git repos
# CI must have Git configured (user.name, user.email) for test commits
# Solution: Add git config setup in CI workflow before running tests

# CRITICAL: cargo-dist requires clean Git working tree
# cargo dist init modifies Cargo.toml and creates workflows
# Must commit changes before testing release workflow
# Solution: Commit cargo-dist config changes before tagging

# CRITICAL: GitHub Actions permissions for releases
# cargo-dist release workflow needs write permissions to create releases
# Default GITHUB_TOKEN may have read-only permissions
# Solution: Add permissions: contents: write in release workflow

# CRITICAL: Rust cache key sensitivity
# Swatinem/rust-cache uses Cargo.lock hash for cache key
# Changing dependencies invalidates cache
# Solution: Use cache restore-keys for fallback cache hits

# CRITICAL: Integration tests use ~/.jin/ global repository
# Tests may interfere with each other if run in parallel
# Solution: Tests already use unique mode/scope names (e.g., test_mode_{PID})
# Verify tests pass with --test-threads=1 if parallel issues occur

# LIBRARY: assert_cmd requires cargo bin to exist
# Command::cargo_bin("jin") fails if binary not built
# CI must build before testing
# Solution: cargo build --release before running tests (or tests build automatically)

# LIBRARY: git2-rs requires libgit2 system library
# Cargo.toml uses features = ["vendored-libgit2"] to bundle
# This works on all platforms but increases build time by ~30 seconds
# Solution: Already configured correctly, no action needed

# LIBRARY: clap completion generation
# Jin generates shell completions via clap_complete
# Should verify completions work on multiple shells in CI
# Solution: Add smoke test for completion generation in CI

# PERFORMANCE: cargo test is slow (~2-3 minutes)
# 25+ integration tests each create Git repos
# cargo-nextest can parallelize better (3x faster)
# Solution: Use cargo-nextest in CI for faster feedback
```

## Implementation Blueprint

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: INSTALL cargo-dist and initialize configuration
  - INSTALL: cargo install cargo-dist --locked
  - RUN: cargo dist init (interactive setup)
  - SELECT: "GitHub CI" when prompted for CI provider
  - SELECT: Installers - shell, powershell, homebrew
  - SELECT: Targets - linux-gnu, linux-musl, darwin-x86, darwin-arm, windows-msvc
  - COMMIT: Changes to Cargo.toml and generated .github/workflows/release.yml
  - PLACEMENT: Root directory (modifies Cargo.toml, creates .github/workflows/release.yml)
  - NAMING: Use default cargo-dist naming (.github/workflows/release.yml)
  - DEPENDENCIES: None (first task)
  - VALIDATION: cargo dist plan (should show build plan without errors)

Task 2: CREATE .github/workflows/ci.yml
  - IMPLEMENT: Multi-platform CI workflow (Linux, macOS, Windows)
  - JOBS:
    - test: Matrix build across 3 OSes, run cargo test with all features
    - lint: cargo clippy with -D warnings
    - format: cargo fmt --check
    - audit: cargo audit for security vulnerabilities
  - CACHING: Use Swatinem/rust-cache@v2 for ~80% speedup
  - FOLLOW pattern: plan/P6M4/research/workflow_examples.md (ripgrep CI pattern)
  - NAMING: .github/workflows/ci.yml (standard convention)
  - PLACEMENT: .github/workflows/ directory
  - DEPENDENCIES: Task 1 (creates .github/workflows/ directory)
  - CRITICAL: Add git config for user.name/email before tests (required for commits in tests)
  - VALIDATION: Push to branch and verify CI runs, all checks pass

Task 3: CONFIGURE release-plz for semantic versioning
  - CREATE: release-plz.toml in root directory
  - IMPLEMENT: Workspace configuration with changelog settings
  - ENABLE: changelog_update, git_tag_enable
  - CONFIGURE: Conventional commit types to include (feat, fix, perf, refactor, docs)
  - FOLLOW pattern: plan/P6M4/research/release_automation.md (release-plz config section)
  - NAMING: release-plz.toml (standard name)
  - PLACEMENT: Root directory
  - DEPENDENCIES: None (can run in parallel with Task 2)
  - VALIDATION: release-plz update --dry-run (shows version bump and changelog)

Task 4: CONFIGURE git-cliff for changelog generation
  - CREATE: cliff.toml in root directory (optional but recommended)
  - IMPLEMENT: Changelog template with conventional commit grouping
  - SECTIONS: Features, Bug Fixes, Performance, Refactoring, Documentation, Tests
  - FOLLOW pattern: plan/P6M4/research/release_automation.md (git-cliff section)
  - NAMING: cliff.toml (standard name)
  - PLACEMENT: Root directory
  - DEPENDENCIES: None (can run in parallel with Tasks 2, 3)
  - VALIDATION: git cliff --unreleased (shows changelog for uncommitted changes)

Task 5: VERIFY Cargo.toml release profile optimization
  - READ: Cargo.toml [profile.release] section
  - VERIFY: lto = true (link-time optimization enabled)
  - VERIFY: opt-level = 3 (maximum optimization enabled)
  - ADD: strip = true (strip debug symbols for smaller binaries)
  - CRITICAL: Ensure vendored-libgit2 feature enabled (already configured)
  - DEPENDENCIES: None (read-only verification)
  - VALIDATION: cargo build --release (verify optimized build succeeds)

Task 6: MODIFY release workflow to integrate release-plz
  - MODIFY: .github/workflows/release.yml (generated by cargo-dist)
  - ADD: release-plz step before cargo-dist build
  - INTEGRATE: Changelog generation in release body
  - ENABLE: GitHub Artifact Attestations (id-token: write permission)
  - FOLLOW pattern: plan/P6M4/research/workflow_examples.md (starship full automation)
  - DEPENDENCIES: Task 1 (modifies generated release.yml), Task 3 (uses release-plz config)
  - CRITICAL: Add permissions block for contents:write and id-token:write
  - VALIDATION: Dry-run release workflow with act (if available) or test tag

Task 7: ADD cargo-nextest for faster CI test execution
  - MODIFY: .github/workflows/ci.yml test job
  - INSTALL: cargo-nextest via taiki-e/install-action
  - REPLACE: cargo test with cargo nextest run
  - BENEFIT: 3x faster test execution vs cargo test
  - FOLLOW pattern: plan/P6M4/research/rust_cicd_best_practices.md (cargo-nextest section)
  - DEPENDENCIES: Task 2 (modifies ci.yml)
  - VALIDATION: Run locally with cargo nextest run (verify all tests pass)

Task 8: CONFIGURE crates.io publishing in release workflow
  - MODIFY: .github/workflows/release.yml
  - ADD: cargo publish step with CARGO_REGISTRY_TOKEN secret
  - PLACEMENT: After successful build, before GitHub release
  - CRITICAL: Requires CARGO_REGISTRY_TOKEN secret configured in GitHub repo settings
  - GOTCHA: cargo publish is irreversible - test with --dry-run first
  - DEPENDENCIES: Task 6 (modifies release.yml)
  - VALIDATION: cargo publish --dry-run (verify package metadata)

Task 9: TEST complete release workflow end-to-end
  - CREATE: Pre-release tag (e.g., v0.1.0-rc.1)
  - PUSH: Tag to GitHub to trigger release workflow
  - VERIFY: Release workflow completes successfully
  - VERIFY: GitHub release created with:
    - Multi-platform binaries (5+ platforms)
    - SHA256 checksums for all artifacts
    - Auto-generated changelog from commits
    - Installers (shell, PowerShell scripts)
  - VERIFY: Binaries are executable and show correct version
  - DEPENDENCIES: All previous tasks (end-to-end validation)
  - VALIDATION: Download and run binary: ./jin-x86_64-unknown-linux-gnu --version

Task 10: DOCUMENT release process for maintainers
  - CREATE: RELEASE.md or add section to CONTRIBUTING.md
  - DOCUMENT: Release process (conventional commits, tagging, versioning)
  - DOCUMENT: How to trigger release (git tag vX.Y.Z && git push --tags)
  - DOCUMENT: How to verify release (check GitHub releases, test binaries)
  - DOCUMENT: Troubleshooting common issues (failed builds, missing secrets)
  - PLACEMENT: Root directory (RELEASE.md) or docs/ directory
  - DEPENDENCIES: Task 9 (validates process before documenting)
  - VALIDATION: Follow documented process to create test release
```

### Implementation Patterns & Key Details

```yaml
# CI Workflow Pattern (comprehensive)

name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    name: Test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, 1.70.0]  # MSRV

    steps:
      - uses: actions/checkout@v4

      # CRITICAL: Git config required for test commits
      - name: Configure Git
        run: |
          git config --global user.name "CI Bot"
          git config --global user.email "ci@example.com"

      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}

      # PERFORMANCE: Swatinem/rust-cache is essential (80% speedup)
      - uses: Swatinem/rust-cache@v2

      # PERFORMANCE: Use cargo-nextest for 3x faster tests
      - uses: taiki-e/install-action@nextest

      - name: Build
        run: cargo build --all-features --verbose

      - name: Run tests
        run: cargo nextest run --all-features

  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2

      - name: Clippy
        run: cargo clippy --all-features -- -D warnings

  format:
    name: Format
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt

      - name: Check formatting
        run: cargo fmt --all -- --check

  audit:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v2
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

# PATTERN: Release workflow structure (cargo-dist generated, then enhanced)

name: Release

permissions:
  contents: write      # CRITICAL: Required for creating releases
  id-token: write      # CRITICAL: Required for GitHub Attestations

on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+*'  # Matches v1.2.3, v1.2.3-rc.1, etc.

# INTEGRATION: cargo-dist will generate most of this workflow
# CUSTOMIZATION: Add release-plz step before cargo-dist build
# PATTERN: See plan/P6M4/research/workflow_examples.md for complete example

# GOTCHA: cargo-dist init will scaffold this - customize after generation
```

```toml
# Cargo.toml additions (cargo-dist metadata)

[workspace.metadata.dist]
# CRITICAL: Pin to specific version for reproducible builds
dist-version = "0.30.2"

# CI provider
ci = ["github"]

# Installers to generate
installers = ["shell", "powershell", "homebrew"]

# Target platforms (5 recommended for broad compatibility)
targets = [
  "x86_64-unknown-linux-gnu",      # Linux (glibc) - most common
  "x86_64-unknown-linux-musl",     # Linux (musl) - static binaries
  "x86_64-apple-darwin",           # macOS Intel
  "aarch64-apple-darwin",          # macOS Apple Silicon (M1/M2/M3)
  "x86_64-pc-windows-msvc",        # Windows
]

# Install path for installers
install-path = "~/.cargo/bin"

# PERFORMANCE: Use cargo-dist's archive format (smaller, faster)
archive-format = "tar.gz"

# SECURITY: Enable GitHub Artifact Attestations (2025 best practice)
github-attestations = true

# GOTCHA: cargo dist init will add this section - review targets and installers
```

```toml
# release-plz.toml (semantic versioning configuration)

[workspace]
# Update CHANGELOG.md automatically
changelog_update = true

# Create git tags for releases
git_tag_enable = true

# PATTERN: Conventional commit mapping
# feat: minor version bump (0.1.0 -> 0.2.0)
# fix: patch version bump (0.1.0 -> 0.1.1)
# BREAKING CHANGE: major version bump (0.1.0 -> 1.0.0)

[[package]]
name = "jin"

# Include these commit types in changelog
changelog_include = ["feat", "fix", "perf", "refactor", "docs"]

# GOTCHA: Requires conventional commit format
# Good: "feat: add remote sync command"
# Bad: "added remote sync" (won't trigger version bump)
```

```toml
# cliff.toml (optional - enhanced changelog formatting)

[changelog]
header = """
# Changelog

All notable changes to Jin will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

"""

# PATTERN: Group commits by type with emoji and description
body = """
{% for group, commits in commits | group_by(attribute="group") %}
### {{ group | upper_first }}
{% for commit in commits %}
  - {{ commit.message | upper_first }}{% if commit.breaking %} [**BREAKING**]{% endif %}
{% endfor %}
{% endfor %}
"""

[git]
# Parse conventional commits
conventional_commits = true
filter_unconventional = false

# Commit grouping
commit_parsers = [
  { message = "^feat", group = "Features" },
  { message = "^fix", group = "Bug Fixes" },
  { message = "^perf", group = "Performance" },
  { message = "^refactor", group = "Refactoring" },
  { message = "^docs", group = "Documentation" },
  { message = "^test", group = "Tests" },
  { message = "^chore", group = "Chore" },
]

# GOTCHA: git-cliff is optional - release-plz can generate changelogs too
# Use cliff.toml if you want more control over changelog formatting
```

### Integration Points

```yaml
GITHUB_REPOSITORY_SETTINGS:
  - action: "Add CARGO_REGISTRY_TOKEN secret"
    value: "Token from https://crates.io/me/tokens for cargo publish"
    location: "Settings → Secrets and variables → Actions → New repository secret"
    required: true

  - action: "Enable GitHub Actions"
    location: "Settings → Actions → General"
    permission: "Allow all actions and reusable workflows"
    required: true

  - action: "Configure workflow permissions"
    location: "Settings → Actions → General → Workflow permissions"
    permission: "Read and write permissions" (for release workflow to create releases)
    required: true

CARGO_CONFIGURATION:
  - modify: "Cargo.toml"
    section: "[profile.release]"
    add: "strip = true  # Reduce binary size by stripping debug symbols"

  - modify: "Cargo.toml"
    section: "[workspace.metadata.dist]"
    pattern: "Add cargo-dist configuration (generated by cargo dist init)"

VERSION_CONTROL:
  - commit_convention: "Conventional Commits 1.0.0"
    format: "<type>(<scope>): <description>"
    examples:
      - "feat: add remote sync command"
      - "fix: resolve merge conflict in JSON files"
      - "perf: optimize tree walking algorithm"
      - "docs: update README with sync workflow"
      - "chore: update dependencies"
    breaking: "Add 'BREAKING CHANGE:' footer for major version bumps"

  - tagging_strategy: "Semantic Versioning (SemVer)"
    format: "vMAJOR.MINOR.PATCH"
    examples:
      - "v0.1.0" (initial release)
      - "v0.2.0" (new feature)
      - "v0.2.1" (bug fix)
      - "v1.0.0" (first stable release or breaking change)
```

## Validation Loop

### Level 1: Local Development Validation (Immediate Feedback)

```bash
# CRITICAL: Run these locally before pushing to catch issues early

# Format check (fix automatically)
cargo fmt --all

# Lint check (fix common issues)
cargo clippy --all-features --fix --allow-dirty

# Lint verification (must pass for CI)
cargo clippy --all-features -- -D warnings
# Expected: Zero warnings. If warnings exist, fix before committing.

# Build verification
cargo build --release --all-features
# Expected: Successful build with optimizations. Check binary size is reasonable (<10MB).

# Test verification
cargo nextest run --all-features
# OR (if nextest not installed):
cargo test --all-features
# Expected: All tests pass. If failures, debug and fix before pushing.

# Security audit
cargo audit
# Expected: No vulnerabilities. If found, update dependencies or assess risk.

# cargo-dist configuration check
cargo dist plan
# Expected: Shows build plan for all target platforms without errors.
# Verify: 5+ targets listed (linux-gnu, linux-musl, darwin-x86, darwin-arm, windows-msvc)

# cargo-dist build test (local multi-platform build simulation)
cargo dist build --artifacts=local
# Expected: Builds for current platform successfully
# NOTE: Full cross-platform build only works in CI with proper toolchains

# Release-plz dry run (version bump preview)
release-plz update --dry-run
# Expected: Shows what version changes would be made based on commits
# Verify: Version bumps make sense for commit types (feat=minor, fix=patch)

# Changelog preview (if using git-cliff)
git cliff --unreleased
# Expected: Shows formatted changelog for commits since last tag
# Verify: Commits grouped correctly by type (Features, Bug Fixes, etc.)
```

### Level 2: CI Workflow Validation (Push to Branch)

```bash
# Push to feature branch to trigger CI workflow
git checkout -b test-ci-workflow
git push origin test-ci-workflow

# Monitor CI run in GitHub Actions tab
# URL: https://github.com/<owner>/jin/actions

# Verify all jobs pass:
# [ ] test (ubuntu-latest, stable) - passes
# [ ] test (ubuntu-latest, 1.70.0) - passes (MSRV check)
# [ ] test (macos-latest, stable) - passes
# [ ] test (windows-latest, stable) - passes
# [ ] lint - passes (clippy with -D warnings)
# [ ] format - passes (rustfmt check)
# [ ] audit - passes (no security vulnerabilities)

# Expected total runtime: 15-20 minutes (with caching)
# Expected on cache miss: 25-35 minutes

# If CI fails:
# 1. Read failure logs in GitHub Actions
# 2. Reproduce locally: cargo test --all-features (or specific test)
# 3. Fix issue and push again
# 4. CI runs automatically on new push

# Caching verification:
# - First run: ~25-35 minutes (no cache)
# - Subsequent runs: ~15-20 minutes (80% cache hit rate)
# - Check: "Swatinem/rust-cache" step should show "Cache hit: true"
```

### Level 3: Release Workflow Validation (Test Release)

```bash
# CRITICAL: Test release workflow before production release

# Step 1: Ensure main branch is clean and CI passes
git checkout main
git pull origin main
# Verify: CI workflow shows green checkmark on latest main commit

# Step 2: Create pre-release tag (won't publish to crates.io)
git tag v0.1.0-rc.1
git push origin v0.1.0-rc.1

# Step 3: Monitor release workflow in GitHub Actions
# URL: https://github.com/<owner>/jin/actions
# Expected runtime: 30-45 minutes (cross-platform builds)

# Verify release workflow jobs:
# [ ] plan - cargo-dist plans release
# [ ] build-local-artifacts - builds binaries for all targets
# [ ] build-global-artifacts - creates installers and checksums
# [ ] host - generates installer scripts
# [ ] publish-release - creates GitHub release
# [ ] publish-homebrew-formula - publishes Homebrew formula (if configured)

# Step 4: Verify GitHub release created
# URL: https://github.com/<owner>/jin/releases/tag/v0.1.0-rc.1

# Checklist:
# [ ] Release title matches tag: "v0.1.0-rc.1"
# [ ] Changelog auto-generated with commit groups
# [ ] Binary artifacts present (5+ files):
#     - jin-x86_64-unknown-linux-gnu.tar.gz
#     - jin-x86_64-unknown-linux-musl.tar.gz
#     - jin-x86_64-apple-darwin.tar.gz
#     - jin-aarch64-apple-darwin.tar.gz
#     - jin-x86_64-pc-windows-msvc.zip
# [ ] Checksum file present: jin-v0.1.0-rc.1-SHA256SUMS.txt
# [ ] Installer scripts present:
#     - install.sh (shell installer)
#     - install.ps1 (PowerShell installer)
# [ ] Attestations visible (if enabled)

# Step 5: Download and test binary
wget https://github.com/<owner>/jin/releases/download/v0.1.0-rc.1/jin-x86_64-unknown-linux-gnu.tar.gz
tar -xzf jin-x86_64-unknown-linux-gnu.tar.gz
./jin --version
# Expected: "jin 0.1.0-rc.1" or similar

# Test basic functionality
./jin init
# Expected: Creates .jin directory and context

# Step 6: Test installer script
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/<owner>/jin/releases/download/v0.1.0-rc.1/install.sh | sh
jin --version
# Expected: jin installed to ~/.cargo/bin/jin and works

# Step 7: Verify checksums
sha256sum -c jin-v0.1.0-rc.1-SHA256SUMS.txt
# Expected: All checksums match

# If release fails:
# 1. Check GitHub Actions logs for specific error
# 2. Common issues:
#    - Missing CARGO_REGISTRY_TOKEN secret
#    - Insufficient permissions (check workflow permissions)
#    - cargo-dist config error (run cargo dist plan locally)
#    - Cross-compilation failure (check target compatibility)
# 3. Fix issue, delete tag, recreate and push

# Clean up test release (optional)
# Delete tag locally: git tag -d v0.1.0-rc.1
# Delete release on GitHub: Settings → Releases → Delete release
```

### Level 4: Production Release Validation (Final Checklist)

```bash
# CRITICAL: Final validation before production release

# Pre-release checklist:
# [ ] All CI checks passing on main branch
# [ ] Test release (v0.x.0-rc.1) validated successfully
# [ ] CHANGELOG.md updated (if not auto-generated)
# [ ] Version in Cargo.toml correct (if not using release-plz)
# [ ] Documentation up-to-date (README, docs/)
# [ ] CARGO_REGISTRY_TOKEN secret configured in GitHub
# [ ] Conventional commits followed for accurate version bumping

# Step 1: Verify conventional commits since last release
git log <last-tag>..HEAD --oneline | grep -E '^[a-f0-9]+ (feat|fix|perf|docs|refactor|test|chore)'
# Review: Ensure commits follow format "type: description"
# Verify: Version bump will be correct (feat=minor, fix=patch, BREAKING CHANGE=major)

# Step 2: Let release-plz handle version bumping (recommended)
# OR manually bump version in Cargo.toml if not using release-plz

# Step 3: Create production release tag
git tag v0.2.0  # Use semantic version based on changes
git push origin v0.2.0

# Step 4: Monitor release workflow (same as Level 3)
# Expected: Release workflow completes in 30-45 minutes

# Step 5: Verify crates.io publication
# URL: https://crates.io/crates/jin
# [ ] New version appears on crates.io within 5-10 minutes
# [ ] Documentation link works
# [ ] Install works: cargo install jin

# Step 6: Test installation methods
# cargo install
cargo install jin
jin --version
# Expected: Latest version installed

# Homebrew (if formula published)
# brew install jin
# jin --version

# Shell installer
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/<owner>/jin/releases/latest/download/install.sh | sh
jin --version

# PowerShell installer (Windows)
# irm https://github.com/<owner>/jin/releases/latest/download/install.ps1 | iex
# jin --version

# Step 7: Smoke test installed binary
jin init
jin mode create test
jin mode use test
jin status
# Expected: All commands work without errors

# Step 8: Verify GitHub Attestations (supply chain security)
# URL: https://github.com/<owner>/jin/attestations/
# [ ] Attestations generated for all artifacts
# [ ] Sigstore signatures present
# [ ] Provenance information visible

# Step 9: Announce release
# [ ] Update README badges if needed
# [ ] Post release notes (GitHub Discussions, Twitter, etc.)
# [ ] Update project website/documentation if exists

# Post-release monitoring:
# - Watch for installation issues (GitHub Issues)
# - Monitor download stats (GitHub Insights → Traffic → Popular content)
# - Check crates.io download stats
# - Verify security alerts (Dependabot, cargo-audit)
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 validation levels completed successfully
- [ ] CI workflow passes on all platforms (Linux, macOS, Windows)
- [ ] CI workflow blocks merging if checks fail
- [ ] Release workflow creates multi-platform binaries (5+ targets)
- [ ] GitHub releases include auto-generated changelogs
- [ ] Binary downloads work on fresh systems
- [ ] Security attestations enabled and visible
- [ ] cargo publish succeeds to crates.io
- [ ] Caching reduces CI time by 60%+ vs uncached
- [ ] All integration tests pass in CI (25+ test cases)

### Feature Validation

- [ ] CI runs on every PR and push to main
- [ ] Release triggered on `v*` tags automatically
- [ ] Binaries built for: Linux (gnu, musl), macOS (Intel, ARM), Windows
- [ ] Installers generated: shell script, PowerShell script, Homebrew formula
- [ ] Changelogs auto-generated from conventional commits
- [ ] Version bumping follows semantic versioning (feat=minor, fix=patch)
- [ ] GitHub releases created with all artifacts and checksums
- [ ] cargo-nextest used for 3x faster test execution
- [ ] Security audit runs on every CI check

### Code Quality Validation

- [ ] cargo fmt --check passes (formatting enforced)
- [ ] cargo clippy passes with -D warnings (no linting warnings)
- [ ] cargo audit passes (no known vulnerabilities)
- [ ] All tests pass on MSRV (Rust 1.70.0)
- [ ] Release profile optimized (lto=true, opt-level=3, strip=true)
- [ ] Binary size reasonable (<10MB for release builds)

### Documentation & Deployment

- [ ] Release process documented (RELEASE.md or CONTRIBUTING.md)
- [ ] Conventional commit format documented for contributors
- [ ] CI/CD architecture documented (workflow purposes, triggers)
- [ ] Troubleshooting guide for common CI/CD issues
- [ ] CARGO_REGISTRY_TOKEN secret configured in repository settings
- [ ] GitHub Actions permissions set to "Read and write"

---

## Anti-Patterns to Avoid

### CI/CD Anti-Patterns
- ❌ Don't skip caching (Swatinem/rust-cache) - wastes 10+ minutes per build
- ❌ Don't use generic cargo test when cargo-nextest is 3x faster
- ❌ Don't hardcode versions - pin via Cargo.toml and tools
- ❌ Don't skip MSRV testing - users may have older Rust versions
- ❌ Don't ignore security audit failures - vulnerabilities must be addressed
- ❌ Don't manually manage versions - use release-plz or semantic-release
- ❌ Don't skip changelog generation - users need to know what changed

### Workflow Anti-Patterns
- ❌ Don't create workflows that require manual steps - defeats automation purpose
- ❌ Don't publish to crates.io before GitHub release succeeds
- ❌ Don't use --allow-dirty or --no-verify flags - defeats safety checks
- ❌ Don't skip testing release workflow before production release
- ❌ Don't ignore conventional commit format - breaks automatic versioning
- ❌ Don't create releases from non-main branches (except for hotfixes)

### Configuration Anti-Patterns
- ❌ Don't skip cargo-dist platforms - users need multi-platform support
- ❌ Don't disable GitHub Attestations - supply chain security matters
- ❌ Don't use default release profiles - optimize with lto and strip
- ❌ Don't forget Git config in CI - tests require user.name/email for commits
- ❌ Don't skip permissions configuration - workflows need write access

### Testing Anti-Patterns
- ❌ Don't skip integration tests in CI - they catch real-world issues
- ❌ Don't run tests without --all-features - feature combinations may break
- ❌ Don't ignore test failures - all tests must pass before merge
- ❌ Don't skip cross-platform testing - platform-specific bugs exist

---

## Confidence Score: 9/10

**Rationale for High Confidence:**

**Strengths (Why 9/10):**
1. **Comprehensive research completed** - 232 KB of research across 14 documents covering Rust CI/CD best practices, release automation, and binary distribution
2. **Real-world patterns validated** - Research includes proven workflows from ripgrep, starship, bat, fd (popular Rust CLI tools)
3. **Tool recommendations backed by research** - cargo-dist (self-hosted, 76+ contributors), release-plz (Rust-native semantic versioning), cargo-nextest (3x faster tests)
4. **Existing test infrastructure ready** - 25+ integration tests already passing, fixtures and assertions in place
5. **Clear implementation path** - 10 ordered tasks with explicit dependencies, validation gates, and gotchas documented
6. **Proven toolchain** - All recommended tools are production-proven (cargo-dist used by UV, Ruff, Pixi; release-plz actively maintained)
7. **Security built-in** - GitHub Attestations, cargo-audit, SHA256 checksums included
8. **Performance optimized** - Swatinem/rust-cache (80% speedup), cargo-nextest (3x faster), LTO, strip

**Minor Uncertainties (Why not 10/10):**
1. **First-time cargo-dist setup** - May encounter platform-specific cross-compilation issues not covered in research (e.g., musl target build failures)
2. **Release-plz integration** - Workflow integration with cargo-dist may require iteration to get right
3. **GitHub repo secrets** - CARGO_REGISTRY_TOKEN and permissions setup are manual steps outside code

**Mitigation for Uncertainties:**
- Task 9 (test release workflow end-to-end) will catch integration issues early with pre-release tag
- Research documents include troubleshooting sections for common cargo-dist and CI issues
- Validation loop has 4 levels to catch issues progressively (local → CI → test release → production)

**One-Pass Implementation Likelihood:** 85%
- With provided research, task ordering, and validation gates, implementation should succeed in one pass
- Test release (Task 9) may reveal minor integration tweaks needed before production release
- All foundational pieces are in place (tests, docs, project structure)

---

## Research Sources Referenced

This PRP was created using comprehensive research documented at:
- `plan/P6M4/research/rust_cicd_best_practices.md` (32 KB, GitHub Actions patterns)
- `plan/P6M4/research/release_automation.md` (24 KB, semantic versioning, release-plz)
- `plan/P6M4/research/cargo_dist_guide.md` (26 KB, multi-platform distribution)
- `plan/P6M4/research/workflow_examples.md` (19 KB, production workflow examples)
- `plan/P6M4/research/COMPLETE_PACKAGE_SUMMARY.md` (17 KB, research overview)

External references:
- https://github.com/BurntSushi/ripgrep (release workflow patterns)
- https://github.com/starship/starship (full automation example)
- https://axodotdev.github.io/cargo-dist/book/ (cargo-dist official docs)
- https://release-plz.dev/ (release-plz documentation)
- https://git-cliff.org/ (changelog generation)

All research sources are verified and current as of December 27, 2025.
