# Rust CI/CD Best Practices Research - Index

## Quick Navigation

### Main Document
📄 **[rust_cicd_best_practices.md](rust_cicd_best_practices.md)** (32 KB, 1,151 lines)
Comprehensive guide covering all aspects of Rust CI/CD with GitHub Actions

### Summary Document
📋 **[RESEARCH_SUMMARY.md](RESEARCH_SUMMARY.md)** (8.3 KB)
Executive summary of findings and key recommendations

---

## Document Structure Quick Links

### Section 1: Modern Workflows
- Multi-platform builds (Linux, macOS, Windows)
- Matrix builds with native platforms
- Cross-compilation with `cross`
- Swatinem/rust-cache setup
- Testing against multiple Rust versions

### Section 2: Testing & Quality
- cargo test execution
- cargo-nextest for parallel testing (3x faster)
- cargo clippy linting
- cargo fmt formatting
- Code coverage with cargo-tarpaulin
- Security audits with cargo-audit

### Section 3: Release Automation
- release-plz overview and benefits
- Semantic versioning based on commits
- GitHub release creation
- Binary distribution strategies
- Debian package generation
- Changelog automation

### Section 4: Deployment
- Publishing to crates.io
- Platform-specific binaries
- Docker/OCI container builds
- Multi-architecture containers

### Section 5: Performance
- Caching strategies and best practices
- Incremental compilation settings
- Parallel job execution
- Cost optimization checklist
- GitHub Actions-specific optimizations

### Section 6: Real Examples
- **ripgrep** (BurntSushi/ripgrep)
  - Multi-arch CI: https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/ci.yml
  - Release workflow: https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/release.yml
- **tokio** (tokio-rs/tokio)
- **clap** (clap-rs/clap)
- **serde** (serde-rs/serde)

### Section 7: Ready-to-Use Workflows
1. **Complete CI Workflow** - All quality gates
2. **Release Workflow** - Binary distribution
3. **Automated Release (release-plz)** - Full automation
4. **Multi-Arch Container Build** - Docker workflows

### Section 8: Tools Reference
Complete tables of:
- Essential tools with installation methods
- GitHub Actions with marketplace links
- Recommended workflow structures

---

## Key Findings Summary

### Essential Tools
| Tool | Purpose | Status |
|------|---------|--------|
| dtolnay/rust-toolchain | Install Rust | Essential |
| Swatinem/rust-cache | Cache dependencies | Essential |
| cargo-nextest | Fast parallel testing | Recommended |
| release-plz | Automated releases | Recommended |
| taiki-e actions | Binary uploads | Recommended |
| cargo-tarpaulin | Coverage reports | Recommended |
| rustsec/audit-check | Security audits | Recommended |

### Top 3 Performance Wins
1. **cargo-nextest** - 3x faster test execution
2. **Swatinem/rust-cache** - Significant compilation speedup
3. **CARGO_INCREMENTAL=0** - Faster CI builds (disable for CI only)

### Release Automation: release-plz
- Automates semantic versioning
- Detects API breaking changes
- Generates changelogs
- Publishes to crates.io
- Works with GitHub, Gitea, GitLab

---

## By Use Case

### I'm Building a New Rust Project
1. Start with "Complete CI Workflow" (Section 7)
2. Add release-plz for automation (Section 3)
3. Customize matrix for your needs (Section 1)

### I'm Optimizing Existing CI
1. Add Swatinem/rust-cache (Section 1)
2. Switch to cargo-nextest (Section 2)
3. Review performance checklist (Section 5)

### I Need to Release Binaries
1. Review binary distribution (Section 3)
2. Use "Release Workflow" example (Section 7)
3. Reference ripgrep's approach (Section 6)

### I Want to Use Containers
1. Review container builds (Section 4)
2. Use "Multi-Arch Container" example (Section 7)
3. Check Docker documentation link

---

## Research Statistics

- **Total Sources:** 40+
- **Repositories Analyzed:** 4 (ripgrep, tokio, clap, serde)
- **Search Queries:** 10
- **Web Fetches:** 3 (deep dives)
- **Code Examples:** 15+
- **Reference Tables:** 5
- **Complete Workflows:** 4
- **Document Pages:** ~35 (when printed)

---

## GitHub Repository Workflow Files

### Direct Links to Real Workflows

**ripgrep - Multi-Architecture Testing:**
https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/ci.yml

**ripgrep - Release Distribution:**
https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/release.yml

**tokio - Async Runtime Testing:**
https://github.com/tokio-rs/tokio/actions?query=workflow:CI+branch:master

**clap - CLI Release Pipeline:**
https://github.com/clap-rs/clap/actions

---

## Key Environment Variables

```yaml
# Always use in CI
RUST_BACKTRACE: 1

# Use in CI, NOT in local development
CARGO_INCREMENTAL: 0

# Use with sccache (if enabled)
RUSTC_WRAPPER: sccache
SCCACHE_GHA_ENABLED: true
```

---

## Essential GitHub Actions

```yaml
# Setup Rust
- uses: dtolnay/rust-toolchain@stable

# Cache dependencies
- uses: Swatinem/rust-cache@v2

# Install dev tools
- uses: taiki-e/install-action@cargo-nextest

# Create release
- uses: taiki-e/create-gh-release-action@v1

# Upload binaries
- uses: taiki-e/upload-rust-binary-action@v1

# Security audit
- uses: rustsec/audit-check-action@v1

# Docker builds
- uses: docker/build-push-action@v4
```

---

## Common Optimization Patterns

### For Test Speed
```yaml
- uses: taiki-e/install-action@nextest
- run: cargo nextest run --all-features
```

### For Dependency Caching
```yaml
- uses: Swatinem/rust-cache@v2
  with:
    shared-key: build
```

### For Security
```yaml
- uses: rustsec/audit-check-action@v1
  with:
    token: ${{ secrets.GITHUB_TOKEN }}
```

### For Releases
```yaml
- uses: taiki-e/upload-rust-binary-action@v1
  with:
    bin: my-app
    target: ${{ matrix.target }}
    checksum: sha256
```

---

## Recommended Reading Order

1. **Start Here:** Read this INDEX (you are here)
2. **Overview:** Read RESEARCH_SUMMARY.md
3. **Dive Deep:** Read rust_cicd_best_practices.md Sections 1-5
4. **Examples:** Reference Section 6 (Real Repositories)
5. **Implement:** Copy Section 7 (Ready-to-Use Workflows)
6. **Lookup:** Use Section 8 (Tools Reference)

---

## Document Metadata

- **Research Date:** December 27, 2025
- **Focus:** GitHub Actions CI/CD for Rust 2025
- **Last Updated:** December 27, 2025
- **Files:** 3 markdown documents (44 KB total)
- **Coverage:** Multi-platform, testing, releases, deployment, optimization

---

## Questions? Start Here

**"I need a complete CI setup"**
→ Jump to Section 7, Example 1 (Complete CI Workflow)

**"How do I make tests faster?"**
→ Section 2 (cargo-nextest) + Section 5 (Optimization)

**"I want automated releases"**
→ Section 3 (release-plz) + Section 7, Example 3

**"What about container images?"**
→ Section 4 (Containers) + Section 7, Example 4

**"Which tools should I use?"**
→ Section 8 (Tools Reference) or RESEARCH_SUMMARY.md

**"Show me real examples"**
→ Section 6 (Exemplary Repositories) with GitHub links

---

## File Organization

```
plan/P6M4/research/
├── INDEX.md (this file)
├── RESEARCH_SUMMARY.md (executive summary)
└── rust_cicd_best_practices.md (main document - 1,151 lines)
```

---

## Key Takeaways

1. **Use dtolnay/rust-toolchain** for modern Rust setup
2. **Enable Swatinem/rust-cache** for dependency caching
3. **Consider cargo-nextest** for 3x faster tests
4. **Use release-plz** for automated semantic versioning
5. **Test on multiple platforms** using matrix strategy
6. **Disable CARGO_INCREMENTAL** in CI (but not local)
7. **Add security audits** with rustsec
8. **Optimize early** with proper caching strategies

---

For detailed information, see the main document:
📄 [rust_cicd_best_practices.md](rust_cicd_best_practices.md)
