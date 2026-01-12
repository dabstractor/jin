# Rust Release Automation: Real Workflow Examples

## Complete Examples from Production CLIs

### Example 1: Release-plz + cargo-dist (Recommended Stack)

#### Repository Setup
```
project/
├── Cargo.toml
├── Cargo.lock
├── CHANGELOG.md
├── release-plz.toml
├── cliff.toml
├── Dist.toml
└── .github/workflows/
    ├── release.yml          (release-plz + cargo-dist)
    ├── ci.yml               (tests, lint, audit)
    └── (cargo-dist generates additional workflows)
```

#### `.github/workflows/release.yml` (Minimal - 60 lines)
```yaml
name: Release

on:
  push:
    branches:
      - main
  workflow_dispatch:

permissions:
  contents: write
  pull-requests: write

jobs:
  release-plz:
    name: Release-plz
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-dist
        run: cargo install cargo-dist --locked

      - name: Run release-plz
        uses: release-plz/action@v0.5
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

      # Optional: Build and distribute binaries
      - name: Build with cargo-dist
        run: |
          cargo dist build --output-format json > dist-manifest.json
          if [ -f dist-manifest.json ]; then
            # Artifacts available in target/dist/
            ls -la target/dist/
          fi
```

#### Configuration: `release-plz.toml`
```toml
[workspace]
changelog_file = "CHANGELOG.md"
git_release_enable = true
git_release_type = "github"

[workspace.changelog]
trim = true
template = """
## [{{ version }}] - {{ timestamp | date(format="%Y-%m-%d") }}

{% for group, commits in commits | group_by(attribute="group") %}
### {{ group }}
{% for commit in commits %}
- {{ commit.message }}\
{% endfor %}
{% endfor %}
"""

[package]
semver_check = true
changelog = true
```

#### Configuration: `cliff.toml`
```toml
[changelog]
# changelog header
header = "# Changelog\n\nAll notable changes to this project will be documented in this file.\n"
# changelog footer
footer = ""
# placeholder for version
version_placeholder = "<!-- next-header -->"
# remove the leading and trailing whitespace from the template
trim = true
# postprocessors
postprocessors = []

[git]
# parse the commits based on https://www.conventionalcommits.org
conventional_commits = true
# filter out the commits that are not conventional
filter_unconventional = true
# regex for parsing the commit messages
commit_parsers = [
    { message = "^feat", group = "Features" },
    { message = "^fix", group = "Bug Fixes" },
    { message = "^doc", group = "Documentation" },
    { message = "^perf", group = "Performance" },
    { message = "^refactor", group = "Refactoring" },
    { message = "^test", group = "Testing" },
    { message = "^chore|^ci", group = "Miscellaneous Tasks" },
    { message = "^revert", group = "Revert" },
]
# protect breaking changes from being skipped
protect_breaking_commits = false
# filter commits by the provided regex
filter_commits = false
# sort the tags in topological order
topo_order = false
# sort the commits inside each group by the provided regex. 0 = oldest, -1 = latest
sort_commits = "oldest"
limit_commits = 42
```

#### Configuration: `Dist.toml` (cargo-dist)
```toml
# The preferred way to define a dist workspace!
[workspace]
members = [".", "other-crate"]
# Rust version to use (for documentation)
rust-version = "1.70"

[dist]
# The archive format to use for windows builds (defaults .zip)
windows-archive = "zip"
# The archive format to use for non-windows builds (defaults .tar.xz)
unix-archive = "tar.xz"
# CI backend to use
ci = "github"
# The installers to generate for each app
installers = ["shell", "powershell"]
# A GitHub repo to push Homebrew formulas to
tap = "my-org/homebrew-tap"
# Target platforms
targets = [
    "x86_64-unknown-linux-gnu",
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-gnu",
    "x86_64-apple-darwin",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
]
# Whether to generate an installer
generate-installer = true
# Whether to publish to crates.io
publish-jobs = ["publish-crates-io"]
# Publish plan (branches to publish from)
publish-branch = "main"
```

---

### Example 2: Ripgrep's Approach (Manual + Automated Hybrid)

**Reference:** https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/release.yml

#### Release Checklist (`RELEASE-CHECKLIST.md`)
```markdown
# Ripgrep Release Checklist

## Pre-release
- [ ] Run `cargo update`
- [ ] Review `Cargo.lock` changes for semver-incompatible updates
- [ ] Run `cargo outdated`
- [ ] Update `CHANGELOG.md` manually (ripgrep uses manual updates)
- [ ] Bump version in `Cargo.toml`
- [ ] Run `cargo test --all` locally
- [ ] Run `cargo doc --no-deps --all-features`

## Create Release
- [ ] Create annotated tag: `git tag -a vX.Y.Z -m "Release X.Y.Z"`
- [ ] Push tag: `git push origin tag vX.Y.Z`

## Post-release (Automated by GitHub Actions)
- GitHub Actions builds binaries for 13 platforms
- Artifacts uploaded to GitHub Release
- Version appears on crates.io

## Verification
- [ ] Check all platform binaries built
- [ ] SHA256 checksums present
- [ ] Man pages generated
- [ ] Shell completions present
```

#### `.github/workflows/release.yml` (Excerpt)
```yaml
name: release
on:
  push:
    tags:
      - '[0-9]+.[0-9]+.[0-9]+'

jobs:
  create-release:
    name: create-release
    runs-on: ubuntu-latest
    outputs:
      rg_version: ${{ env.RG_VERSION }}
    steps:
      - name: Get the release version from the tag
        shell: bash
        if: env.VERSION == ''
        run: echo "VERSION=${GITHUB_REF#refs/tags/}" >> $GITHUB_ENV

      - name: Validate version matches Cargo.toml
        shell: bash
        run: |
          export VERSION="${{ env.VERSION }}"
          grep "^version = \"$VERSION\"" Cargo.toml

      - name: Create GitHub release
        uses: softprops/action-gh-release@v1
        with:
          draft: true

  build-release:
    name: build-release
    needs: ['create-release']
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        build: [linux-gnu, linux-musl, macos-x64, macos-aarch64, win-msvc, win-gnu]
        include:
          - build: linux-gnu
            os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - build: linux-musl
            os: ubuntu-latest
            target: x86_64-unknown-linux-musl
          - build: macos-x64
            os: macos-latest
            target: x86_64-apple-darwin
          - build: macos-aarch64
            os: macos-latest
            target: aarch64-apple-darwin
          - build: win-msvc
            os: windows-latest
            target: x86_64-pc-windows-msvc
          - build: win-gnu
            os: ubuntu-latest
            target: x86_64-pc-windows-gnu

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - uses: taiki-e/install-action@cross

      - name: Build release binary
        shell: bash
        run: |
          cargo build --release --verbose --target ${{ matrix.target }}

      - name: Strip binary (non-Windows)
        if: matrix.os != 'windows-latest'
        shell: bash
        run: |
          strip "target/${{ matrix.target }}/release/rg"

      - name: Generate completions
        shell: bash
        run: |
          ./target/${{ matrix.target }}/release/rg --generate {bash,fish,zsh,powershell}

      - name: Create archive
        shell: bash
        run: |
          mkdir -p "rg-${{ env.VERSION }}-${{ matrix.target }}"
          cp \
            Cargo.lock \
            LICENSE-MIT \
            LICENSE-UNLICENSE \
            README.md \
            "rg-${{ env.VERSION }}-${{ matrix.target }}/"
          cp target/${{ matrix.target }}/release/rg \
            "rg-${{ env.VERSION }}-${{ matrix.target }}/rg"
          cp completions/* \
            "rg-${{ env.VERSION }}-${{ matrix.target }}/" || true

          tar czf "rg-${{ env.VERSION }}-${{ matrix.target }}.tar.gz" \
            "rg-${{ env.VERSION }}-${{ matrix.target }}"

      - name: Upload release archive
        uses: softprops/action-gh-release@v1
        with:
          files: |
            rg-${{ env.VERSION }}-${{ matrix.target }}.tar.gz
```

---

### Example 3: Starship's Full-Stack Automation

**Reference:** https://github.com/starship/starship/blob/master/.github/workflows/release.yml

#### Workflow Stages
```yaml
name: Release

on:
  push:
    branches: [main]  # Previously master

permissions:
  contents: write
  pull-requests: write

jobs:
  # Stage 1: Generate Release PR
  release-please:
    runs-on: ubuntu-latest
    outputs:
      release_created: ${{ steps.release.outputs.release_created }}
    steps:
      - uses: googleapis/release-please-action@v4
        id: release
        with:
          release-type: rust
          target-branch: main

  # Stage 2: Build Binaries
  build:
    name: Build Release Binaries
    runs-on: ${{ matrix.os }}
    needs: release-please
    if: ${{ needs.release-please.outputs.release_created == 'true' }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            use_cross: false
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            use_cross: true
          - os: ubuntu-latest
            target: aarch64-unknown-linux-gnu
            use_cross: true
          - os: macos-latest
            target: x86_64-apple-darwin
            use_cross: false
          - os: macos-latest
            target: aarch64-apple-darwin
            use_cross: false
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            use_cross: false
          - os: windows-latest
            target: aarch64-pc-windows-msvc
            use_cross: false
    steps:
      - uses: actions/checkout@v4

      - name: Install toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cross
        if: matrix.use_cross
        uses: taiki-e/install-action@cross

      - name: Build
        run: |
          if [ "${{ matrix.use_cross }}" == "true" ]; then
            cross build --release --target ${{ matrix.target }}
          else
            cargo build --release --target ${{ matrix.target }}
          fi

      - name: Create archives
        id: artifacts
        run: |
          # Create platform-specific archives
          # ...

      - name: Upload release artifacts
        uses: softprops/action-gh-release@v1
        with:
          files: |
            ${{ steps.artifacts.outputs.artifact }}

  # Stage 3: Sign Windows Binaries
  sign-windows:
    name: Sign Windows Binaries
    runs-on: windows-latest
    needs: build
    if: ${{ needs.release-please.outputs.release_created == 'true' }}
    steps:
      - uses: actions/checkout@v4
      # SignPath integration for code signing
      # ...

  # Stage 4: Notarize macOS
  notarize-macos:
    name: Notarize macOS Release
    runs-on: macos-latest
    needs: build
    if: ${{ needs.release-please.outputs.release_created == 'true' }}
    steps:
      - uses: actions/checkout@v4
      # Apple notarization workflow
      # ...

  # Stage 5: Publish to Package Managers
  publish-crate:
    name: Publish to crates.io
    runs-on: ubuntu-latest
    needs: build
    if: ${{ needs.release-please.outputs.release_created == 'true' }}
    steps:
      - uses: actions/checkout@v4
      - run: cargo publish --token ${{ secrets.CARGO_REGISTRY_TOKEN }}

  publish-homebrew:
    name: Update Homebrew formula
    runs-on: ubuntu-latest
    needs: build
    if: ${{ needs.release-please.outputs.release_created == 'true' }}
    steps:
      - uses: actions/checkout@v4
      # Homebrew formula update
      # ...

  publish-winget:
    name: Update Winget manifest
    runs-on: ubuntu-latest
    needs: build
    if: ${{ needs.release-please.outputs.release_created == 'true' }}
    steps:
      - uses: actions/checkout@v4
      # Winget manifest update
      # ...
```

---

### Example 4: Sharkdp's CICD Pattern (bat/fd)

**Reference:**
- https://github.com/sharkdp/bat/blob/master/.github/workflows/CICD.yml
- https://github.com/sharkdp/fd/blob/master/.github/workflows/CICD.yml

#### Unified CICD Workflow (Single File for All Operations)
```yaml
name: CICD

on:
  push:
    branches:
      - master
    tags:
      - "v*"
  pull_request:
  workflow_dispatch:

permissions:
  contents: write
  attestations: write

jobs:
  crate_metadata:
    name: Extract crate metadata
    runs-on: ubuntu-latest
    outputs:
      name: ${{ steps.metadata.outputs.name }}
      version: ${{ steps.metadata.outputs.version }}
      maintainer: ${{ steps.metadata.outputs.maintainer }}
      homepage: ${{ steps.metadata.outputs.homepage }}
      msrv: ${{ steps.metadata.outputs.msrv }}
    steps:
      - uses: actions/checkout@v4
      - name: Extract metadata
        id: metadata
        run: |
          cargo metadata --format-version 1 --manifest-path Cargo.toml | \
          jq -r '.packages[0] |
            @json "name=\(.name) version=\(.version) maintainer=\(.authors[0]) homepage=\(.homepage // .repository)"' | \
          xargs -I {} bash -c 'echo "{}"' >> "$GITHUB_OUTPUT"

  ensure_cargo_fmt:
    name: Ensure 'cargo fmt' has been run
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  lint_check:
    name: Lint Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - run: cargo clippy --all-targets --all-features -- -D warnings

  min_version:
    name: Check MSRV
    runs-on: ubuntu-latest
    needs: crate_metadata
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ needs.crate_metadata.outputs.msrv }}
      - run: cargo test --all-features

  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      - run: cargo test --all --all-features

  build:
    name: Build artifacts
    runs-on: ${{ matrix.os }}
    needs: crate_metadata
    strategy:
      matrix:
        build:
          - linux-gnu
          - linux-musl
          - macos-x64
          - macos-aarch64
          - win-msvc
        include:
          - build: linux-gnu
            os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - build: linux-musl
            os: ubuntu-latest
            target: x86_64-unknown-linux-musl
          - build: macos-x64
            os: macos-latest
            target: x86_64-apple-darwin
          - build: macos-aarch64
            os: macos-latest
            target: aarch64-apple-darwin
          - build: win-msvc
            os: windows-latest
            target: x86_64-pc-windows-msvc
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: taiki-e/install-action@cross

      - name: Build
        run: |
          cargo build --release --target ${{ matrix.target }}

      - name: Create archive (Unix)
        if: runner.os != 'Windows'
        run: |
          NAME="${{ needs.crate_metadata.outputs.name }}"
          VERSION="${{ needs.crate_metadata.outputs.version }}"
          TARGET="${{ matrix.target }}"
          mkdir -p "$NAME-$VERSION-$TARGET"
          cp target/$TARGET/release/$NAME "$NAME-$VERSION-$TARGET/"
          tar czf "$NAME-$VERSION-$TARGET.tar.gz" "$NAME-$VERSION-$TARGET"
          shasum -a 256 "$NAME-$VERSION-$TARGET.tar.gz" > "$NAME-$VERSION-$TARGET.tar.gz.sha256"

      - name: Create archive (Windows)
        if: runner.os == 'Windows'
        run: |
          $NAME = "${{ needs.crate_metadata.outputs.name }}"
          $VERSION = "${{ needs.crate_metadata.outputs.version }}"
          $TARGET = "${{ matrix.target }}"
          mkdir "$NAME-$VERSION-$TARGET"
          cp "target/$TARGET/release/$NAME.exe" "$NAME-$VERSION-$TARGET/"
          7z a "$NAME-$VERSION-$TARGET.zip" "$NAME-$VERSION-$TARGET"
          (Get-FileHash "$NAME-$VERSION-$TARGET.zip" -Algorithm SHA256).Hash | Out-File -FilePath "$NAME-$VERSION-$TARGET.zip.sha256"

      - name: Create attestation
        uses: actions/attest-build-provenance@v1
        if: startsWith(github.ref, 'refs/tags/')
        with:
          subject-path: target/${{ matrix.target }}/release/${{ needs.crate_metadata.outputs.name }}

      - name: Upload release artifacts
        uses: softprops/action-gh-release@v1
        if: startsWith(github.ref, 'refs/tags/')
        with:
          files: |
            ${{ needs.crate_metadata.outputs.name }}-${{ needs.crate_metadata.outputs.version }}-*

  winget:
    name: Publish to Winget
    runs-on: ubuntu-latest
    needs: [build, crate_metadata]
    if: startsWith(github.ref, 'refs/tags/')
    steps:
      - uses: actions/checkout@v4
      - name: Publish to Winget
        run: |
          # Winget publication logic
```

---

## Minimal Reference Template

### For New Projects (Copy-Paste Ready)

#### `release-plz.toml`
```toml
[workspace]
changelog_file = "CHANGELOG.md"

[package]
semver_check = true
```

#### `cliff.toml`
```toml
[changelog]
template = """
## [{{ version }}] - {{ timestamp | date(format="%Y-%m-%d") }}
{% for group, commits in commits | group_by(attribute="group") %}
### {{ group }}
{% for commit in commits %}- {{ commit.message }}\
{% endfor %}{% endfor %}
"""
[git]
conventional_commits = true
filter_unconventional = true
[git.commit_parsers]
[[git.commit_parsers]]
message = "^feat"
group = "Features"
[[git.commit_parsers]]
message = "^fix"
group = "Bug Fixes"
[[git.commit_parsers]]
message = "^doc"
group = "Documentation"
```

#### `.github/workflows/release.yml`
```yaml
name: Release
on:
  push:
    branches: [main]
  workflow_dispatch:
permissions:
  contents: write
  pull-requests: write
jobs:
  release-plz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: dtolnay/rust-toolchain@stable
      - uses: release-plz/action@v0.5
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

---

## Key Takeaways

1. **ripgrep:** Manual version bumps + automated builds
2. **starship:** Full automation with release-please (Google approach)
3. **bat/fd:** Unified CICD workflow + metadata extraction
4. **Recommended (2025):** release-plz + cargo-dist combo

**Time to Setup:** 15-30 minutes
**Maintenance:** ~5 minutes per release (review + merge)
