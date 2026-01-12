# Rust Release Automation: Quick Setup Guide (15 Minutes)

## Step 1: Install Tools (5 minutes)

```bash
# Install release-plz (Rust tool - can be installed locally)
cargo install release-plz --locked

# Install git-cliff (optional - built into release-plz)
cargo install git-cliff --locked

# Verify cargo-dist initialization in your Cargo.toml
cargo install cargo-dist --locked
cargo dist init
```

## Step 2: Create Configuration Files (5 minutes)

### Create `release-plz.toml` in project root
```toml
[workspace]
changelog_file = "CHANGELOG.md"

[workspace.changelog]
trim = true

[package]
semver_check = true
```

### Create `cliff.toml` in project root
```toml
[changelog]
template = """
## [{{ version }}] - {{ timestamp | date(format="%Y-%m-%d") }}

{% for group, commits in commits | group_by(attribute="group") %}
### {{ group }}
{% for commit in commits %}
- {{ commit.message }}\
{% endfor %}
{% endfor %}
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
[[git.commit_parsers]]
message = "^perf"
group = "Performance"
[[git.commit_parsers]]
message = "^refactor"
group = "Refactoring"
```

## Step 3: GitHub Actions Setup (5 minutes)

### Create `.github/workflows/release.yml`

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

      - name: Run release-plz
        uses: release-plz/action@v0.5
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
```

### For Binary Distribution: Generate cargo-dist CI

```bash
cargo dist generate-ci
```

This creates `.github/workflows/release.yml` with multi-platform builds.

## Step 4: Use Conventional Commits

Your PR titles/commit messages must follow this pattern:

```
feat: Add new feature
fix: Fix a bug
docs: Update documentation
perf: Performance improvement
refactor: Code refactoring
test: Add tests
chore: Maintenance

BREAKING CHANGE: Describe the breaking change
```

## Step 5: Test Locally (Optional)

```bash
# Simulate a release
release-plz update --dry-run

# Review what would be changed
git diff
```

## Step 6: Enable in GitHub

1. Go to repo Settings → Actions → Workflow permissions
2. Select "Read and write permissions"
3. Push changes to main branch
4. release-plz creates a Release PR automatically
5. Review and merge the PR
6. Done! Release is automatic

## Verification Checklist

- [ ] `release-plz.toml` exists in project root
- [ ] `cliff.toml` exists in project root (or remove if using defaults)
- [ ] `.github/workflows/release.yml` exists
- [ ] Default branch protection requires PR reviews
- [ ] GitHub Actions has read+write permissions
- [ ] Commits follow Conventional Commits format
- [ ] Latest changes pushed to main branch

## First Release Flow (What Happens)

1. You push commits using Conventional Commits
2. Release-plz creates a "Release PR" with:
   - Updated `Cargo.toml` versions
   - Generated `CHANGELOG.md`
   - Updated `Cargo.lock`
3. You review and merge the Release PR
4. Automatically (in seconds):
   - Git tag created (e.g., `v1.0.0`)
   - GitHub Release created with changelog
   - Binary published to crates.io
   - Binaries built (if cargo-dist enabled)
   - GitHub Release updated with binary artifacts

## Subsequent Releases

1. Keep using Conventional Commits in PRs
2. Release-plz automatically creates Release PR weekly
3. Merge Release PR → Automatic release
4. Never manually edit versions or CHANGELOG again

## Troubleshooting

### Release PR not created
- Check GitHub Actions logs: Settings → Actions → workflow runs
- Verify branch protection allows workflow
- Check `GITHUB_TOKEN` has correct scopes

### Wrong version bumped
- Review your commit messages - must follow Conventional Commits
- `fix:` = PATCH, `feat:` = MINOR, `BREAKING CHANGE:` = MAJOR

### Attestations not working
- Enable in GitHub Actions workflow permissions
- cargo-dist v0.21.0+ required

### cargo-dist not building
- Run `cargo dist init` to update config
- Check `.github/workflows/release.yml` generated correctly
- Verify Rust targets installed: `rustup target add <target>`

## Key Repos to Reference

- **ripgrep:** https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/release.yml
- **starship:** https://github.com/starship/starship/blob/master/.github/workflows/release.yml
- **bat:** https://github.com/sharkdp/bat/blob/master/.github/workflows/CICD.yml
- **fd:** https://github.com/sharkdp/fd/blob/master/.github/workflows/CICD.yml

## Next: Read Full Research

See `release_automation.md` for comprehensive documentation on:
- Tool comparison matrices
- Supply chain security
- Package manager integration
- Real-world workflow examples
- Advanced configurations

---

**Time to Production:** ~15 minutes setup + first test merge
**Maintenance Burden:** ~5 minutes per release (review + merge)
**Zero Manual Steps:** After initial merge
