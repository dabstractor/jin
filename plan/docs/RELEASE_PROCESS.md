# Jin Release Process

This document describes the automated release process for Jin using cargo-dist and conventional commits.

## Overview

Jin uses a fully automated CI/CD pipeline that:
- Runs tests, linting, and security checks on every PR
- Builds multi-platform binaries (Linux, macOS, Windows)
- Generates installers and checksums
- Creates GitHub releases automatically on version tags
- Provides supply chain security via GitHub Attestations

## Prerequisites for Maintainers

1. **Commit Message Format**: All commits must follow [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat: add new feature` → Minor version bump (0.1.0 → 0.2.0)
   - `fix: resolve bug` → Patch version bump (0.1.0 → 0.1.1)
   - `feat!: breaking change` or `BREAKING CHANGE:` footer → Major version bump (0.1.0 → 1.0.0)
   - `docs:`, `refactor:`, `perf:`, `test:`, `chore:` → No version bump, included in changelog

2. **GitHub Repository Settings**:
   - Actions enabled with "Read and write permissions"
   - Optional: `CARGO_REGISTRY_TOKEN` secret for publishing to crates.io

## Release Workflow

### Step 1: Prepare the Release

1. Ensure all changes are merged to `main` branch
2. Verify CI is passing on the latest main commit
3. Review recent commits since last release:
   ```bash
   git log <last-tag>..HEAD --oneline
   ```

### Step 2: Create and Push Version Tag

1. Determine the next version based on conventional commits:
   - Count `feat:` commits → minor bump
   - Count `fix:` commits → patch bump
   - Any `BREAKING CHANGE:` → major bump

2. Create annotated tag:
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   ```

3. Push the tag to GitHub:
   ```bash
   git push origin v0.2.0
   ```

### Step 3: Automated Build Process

Once the tag is pushed, GitHub Actions automatically:

1. **Plan Stage** (~1 minute)
   - Validates tag format
   - Generates build manifest

2. **Build Stage** (~15-30 minutes)
   - Builds binaries for 6 platforms:
     - `x86_64-unknown-linux-gnu` (Linux glibc)
     - `x86_64-unknown-linux-musl` (Linux musl)
     - `aarch64-unknown-linux-gnu` (Linux ARM64)
     - `x86_64-apple-darwin` (macOS Intel)
     - `aarch64-apple-darwin` (macOS Apple Silicon)
     - `x86_64-pc-windows-msvc` (Windows)
   - Creates archives (.tar.xz for Unix, .zip for Windows)
   - Generates SHA256 checksums
   - Creates installers (shell script, PowerShell script, Homebrew formula)

3. **Release Stage** (~2 minutes)
   - Creates GitHub Release
   - Uploads all artifacts
   - Generates changelog from conventional commits
   - Creates GitHub Attestations for supply chain security

4. **Total Time**: ~20-35 minutes for complete release

### Step 4: Verify the Release

1. Navigate to GitHub Releases: `https://github.com/<owner>/jin/releases`
2. Verify the release was created with tag `v0.2.0`
3. Check that all artifacts are present:
   - [ ] 6 platform-specific binary archives
   - [ ] 6 SHA256 checksum files
   - [ ] Source tarball (source.tar.gz)
   - [ ] Shell installer (jin-installer.sh)
   - [ ] PowerShell installer (jin-installer.ps1)
   - [ ] Homebrew formula (jin.rb)
   - [ ] Combined checksums file (sha256.sum)

4. Verify changelog is auto-generated and grouped by commit type

5. Test binary download and installation:
   ```bash
   # Download for your platform
   curl -L https://github.com/<owner>/jin/releases/download/v0.2.0/jin-x86_64-unknown-linux-gnu.tar.xz -o jin.tar.xz

   # Extract
   tar -xf jin.tar.xz

   # Verify version
   ./jin --version
   # Expected output: jin 0.2.0
   ```

6. Verify checksum:
   ```bash
   sha256sum -c jin-x86_64-unknown-linux-gnu.tar.xz.sha256
   # Expected: jin-x86_64-unknown-linux-gnu.tar.xz: OK
   ```

7. Test installer script:
   ```bash
   curl -LsSf https://github.com/<owner>/jin/releases/download/v0.2.0/jin-installer.sh | sh
   jin --version
   ```

## Installation Methods for Users

### Via Cargo
```bash
cargo install jin
```

### Via Shell Installer (Linux/macOS)
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/<owner>/jin/releases/latest/download/jin-installer.sh | sh
```

### Via PowerShell Installer (Windows)
```powershell
irm https://github.com/<owner>/jin/releases/latest/download/jin-installer.ps1 | iex
```

### Via Homebrew (macOS)
```bash
# After Homebrew formula is published to a tap
brew install <org>/tap/jin
```

### Direct Binary Download
Download from GitHub Releases page and extract to a directory in your PATH.

## Troubleshooting

### Release Workflow Failed

1. Check GitHub Actions logs: `https://github.com/<owner>/jin/actions`
2. Common issues:
   - **Permission denied**: Verify GitHub Actions has "Read and write permissions"
   - **cargo-dist version mismatch**: Run `cargo dist init` to update workflows
   - **Build failure**: Check specific platform build logs for compilation errors
   - **Missing secrets**: Verify `CARGO_REGISTRY_TOKEN` if publishing to crates.io

### Tests Failing in CI

1. Tests require Git configuration (automatically set in CI)
2. Integration tests create isolated temporary Git repositories
3. Run locally with:
   ```bash
   cargo test --all-features
   ```

### cargo-dist Configuration Issues

1. Validate configuration:
   ```bash
   cargo dist plan
   ```

2. Regenerate workflows after config changes:
   ```bash
   cargo dist generate
   ```

3. Update cargo-dist version:
   ```bash
   cargo install cargo-dist --locked
   cargo dist init  # Applies migrations
   ```

### Homebrew Formula Not Working

1. Ensure `homepage` field is set in Cargo.toml
2. Homebrew formula generated in release but requires manual tap setup
3. See cargo-dist documentation for Homebrew tap configuration

## Security Features

### GitHub Artifact Attestations

All release artifacts are cryptographically signed using GitHub's Artifact Attestations:

```bash
# Verify an artifact (requires GitHub CLI)
gh attestation verify jin-x86_64-unknown-linux-gnu.tar.xz --repo <owner>/jin
```

### Supply Chain Security

- All builds run in GitHub-hosted runners (trusted environment)
- Build provenance included via Sigstore
- SHA256 checksums for all artifacts
- Artifacts attested to specific Git commit and workflow

## CI/CD Pipeline Architecture

### CI Workflow (`.github/workflows/ci.yml`)
**Triggers**: Every push to main, every PR
**Jobs**:
- **Test**: Runs on Linux, macOS, Windows with Rust stable and MSRV (1.70.0)
- **Lint**: cargo clippy with `-D warnings`
- **Format**: cargo fmt --check
- **Audit**: Security vulnerability scanning

**Runtime**: ~15-20 minutes (with caching)

### Release Workflow (`.github/workflows/release.yml`)
**Triggers**: Tags matching `v*` pattern
**Jobs**:
- **Plan**: Determine what to build
- **Build-local-artifacts**: Build platform-specific binaries
- **Build-global-artifacts**: Create installers and checksums
- **Host**: Upload to GitHub Releases
- **Announce**: Finalize release

**Runtime**: ~20-35 minutes

## Configuration Files

- **Cargo.toml**: Project metadata, dependencies, release profile
- **dist-workspace.toml**: cargo-dist configuration (targets, installers, attestations)
- **release-plz.toml**: Semantic versioning rules (optional, for future automation)
- **cliff.toml**: Changelog generation config (optional, for customization)
- **.github/workflows/ci.yml**: CI pipeline
- **.github/workflows/release.yml**: Release pipeline (auto-generated by cargo-dist)

## Future Enhancements

- [ ] Publish to crates.io automatically (requires `CARGO_REGISTRY_TOKEN`)
- [ ] Set up Homebrew tap for easier installation
- [ ] Add release-plz automation for version bumping
- [ ] Windows code signing (requires SignPath account)
- [ ] macOS notarization (requires Apple Developer account)

## References

- [Conventional Commits](https://www.conventionalcommits.org/)
- [cargo-dist Documentation](https://axodotdev.github.io/cargo-dist/book/)
- [Semantic Versioning](https://semver.org/)
- [GitHub Artifact Attestations](https://github.blog/news-insights/product-news/introducing-artifact-attestations-now-in-public-beta/)
