# Rust CI/CD Research - Complete URL Reference

## Quick Link Compilation

All URLs and resources referenced in the comprehensive research document are organized here for easy access.

---

## Official Documentation

### Rust & Cargo
- [The Cargo Book - Continuous Integration](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
  - Official guide for CI/CD with Cargo

- [GitHub Actions - Building and testing Rust](https://docs.github.com/en/actions/tutorials/build-and-test-code/rust)
  - Official GitHub Actions tutorial for Rust

- [Docker Docs - Rust Configuration](https://docs.docker.com/guides/rust/configure-ci-cd/)
  - Docker best practices for Rust

---

## Tool Documentation & Repositories

### Package Management & Releases
- [release-plz Official Website](https://release-plz.dev/)
  - Automated semantic versioning and releases
  - Includes comprehensive docs and examples

- [release-plz on crates.io](https://crates.io/crates/release-plz)
  - Installation source

- [release-plz GitHub Repository](https://github.com/release-plz/release-plz)
  - Source code and changelog

### Testing Tools
- [cargo-nextest Official Website](https://nexte.st/)
  - Next-generation test runner for Rust
  - Pre-built binaries available

- [cargo-nextest GitHub](https://github.com/nextest-rs/nextest)
  - Source repository

- [cargo-nextest on crates.io](https://crates.io/crates/cargo-nextest)
  - Package registry

### Coverage & Linting
- [cargo-tarpaulin GitHub](https://github.com/xd009642/tarpaulin)
  - Code coverage tool for Rust

### Security Auditing
- [RustSec Advisory Database](https://rustsec.org/)
  - Central repository for Rust security advisories

- [RustSec GitHub Organization](https://github.com/RustSec/rustsec)
  - Main source for security tools

- [cargo-audit README](https://github.com/RustSec/rustsec/blob/main/cargo-audit/README.md)
  - Official documentation

### Cross-Compilation
- [cross-rs Repository](https://github.com/cross-rs/cross)
  - Cross-platform compilation tool

### Caching
- [Swatinem/rust-cache GitHub](https://github.com/Swatinem/rust-cache)
  - Smart caching for Rust projects

- [Swatinem/rust-cache on Marketplace](https://github.com/marketplace/actions/rust-cache)
  - GitHub Actions Marketplace

---

## GitHub Actions

### Tool Installation & Setup
- [dtolnay/rust-toolchain@stable](https://github.com/marketplace/actions/install-rust)
  - Install Rust toolchain
  - Marketplace link: https://github.com/marketplace/actions/install-rust

- [taiki-e/install-action](https://github.com/marketplace/actions/install-action)
  - Fast installation of dev tools
  - GitHub: https://github.com/taiki-e/install-action

### Release & Binary Distribution
- [taiki-e/create-gh-release-action](https://github.com/marketplace/actions/create-gh-release)
  - Create GitHub releases
  - GitHub: https://github.com/taiki-e/create-gh-release-action

- [taiki-e/upload-rust-binary-action](https://github.com/marketplace/actions/upload-rust-binary-to-github-releases)
  - Build and upload binaries
  - GitHub: https://github.com/taiki-e/upload-rust-binary-action

### Cross-Compilation Setup
- [taiki-e/setup-cross-toolchain-action](https://github.com/marketplace/actions/setup-cross-toolchain)
  - Setup cross-compilation toolchain
  - GitHub: https://github.com/taiki-e/setup-cross-toolchain-action

### Security Auditing
- [rustsec/audit-check-action](https://github.com/marketplace/actions/rust-audit-check)
  - Official security audit action
  - GitHub: https://github.com/rustsec/audit-check

- [actions-rust-lang/audit](https://github.com/marketplace/actions/audit-rust-dependencies)
  - Alternative audit action
  - GitHub: https://github.com/actions-rust-lang/audit

### Container & Docker
- [docker/setup-buildx-action](https://github.com/marketplace/actions/docker-setup-buildx)
  - Setup Docker BuildKit

- [docker/build-push-action](https://github.com/marketplace/actions/build-and-push-docker-images)
  - Build and push Docker images

- [docker/login-action](https://github.com/marketplace/actions/docker-login)
  - Login to Docker registries

### Code Coverage
- [codecov/codecov-action](https://github.com/marketplace/actions/codecov)
  - Upload coverage reports

---

## Exemplary Repository Workflows

### ripgrep - Multi-Platform Builds & Releases
**Repository:** [github.com/BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep)

**Workflow Files:**
- **CI Workflow:** https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/ci.yml
  - Multi-arch testing (10+ architectures)
  - Tests: pinned 1.85.0, stable, beta, nightly
  - Platforms: Linux (musl, i686, aarch64, armv7, powerpc64, s390x, riscv64gc), macOS, Windows
  - WASM, rustfmt, docs, fuzzing

- **Release Workflow:** https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/release.yml
  - Binary distribution across platforms
  - Shell completions (bash, fish, zsh, PowerShell)
  - Man pages, Debian packages
  - SHA256 checksums

### tokio - Async Runtime Testing
**Repository:** [github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio)

**Actions:**
- **Main Actions:** https://github.com/tokio-rs/tokio/actions?query=workflow:CI+branch:master
- **Feature Tests:** Tests parking_lot, unstable features
- **Miri Tests:** Undefined behavior detection
- **Cross Tests:** Cross-platform validation

### clap - CLI Tool Release
**Repository:** [github.com/clap-rs/clap](https://github.com/clap-rs/clap)

**Resources:**
- **Repository:** https://github.com/clap-rs/clap
- **Workflows:** https://github.com/clap-rs/clap/actions
- **Uses:** taiki-e actions for releases

### serde - Serialization Library
**Repository:** [github.com/serde-rs/serde](https://github.com/serde-rs/serde)

**Resources:**
- **Repository:** https://github.com/serde-rs/serde
- **CI Pattern:** Matrix strategy for multiple Rust versions
- **Features:** Feature-gated compilation testing

---

## Educational Resources & Guides

### Comprehensive Articles
- [Shuttle.dev - Setup Rust CI/CD (2025)](https://www.shuttle.dev/blog/2025/01/23/setup-rust-ci-cd)
  - Current best practices for 2025

- [LogRocket - Optimizing CI/CD Pipelines in Rust](https://blog.logrocket.com/optimizing-ci-cd-pipelines-rust-projects/)
  - In-depth optimization strategies

- [Markaicode - Rust CI/CD Pipeline Setup Comparison (2025)](https://markaicode.com/rust-cicd-pipeline-setup-comparison-2025/)
  - Comparison of different approaches

- [Corrode - Tips for Faster Rust CI Builds](https://corrode.dev/blog/tips-for-faster-ci-builds/)
  - Build optimization techniques

- [Corrode - Tips for Faster Rust Compile Times](https://corrode.dev/blog/tips-for-faster-rust-compile-times/)
  - General compilation improvements

### Multi-Platform Building
- [Medium - Building Rust on Multiple Platforms](https://jondot.medium.com/building-rust-on-multiple-platforms-using-github-6f3e6f8b8458)
  - Cross-platform build strategies

- [dzfrias - Deploy Rust Binaries with GitHub Actions](https://dzfrias.dev/blog/deploy-rust-cross-platform-github-actions/)
  - Binary distribution guide

- [Cross-Compiling Rust on GitHub Actions](https://blog.timhutt.co.uk/cross-compiling-rust/)
  - Cross-compilation setup

- [Blog.urth.org - Cross Compiling with GitHub Actions](https://blog.urth.org/2023/03/05/cross-compiling-rust-projects-in-github-actions/)
  - Detailed cross-compilation guide

### Specific Techniques
- [Optimization - Rust CI Pipeline with GitHub Actions](https://jwsong.github.io/blog/ci-optimization/)
  - Caching strategies deep dive

- [Uffizzi - Optimizing Rust Builds for Faster Pipelines](https://www.uffizzi.com/blog/optimizing-rust-builds-for-faster-github-actions-pipelines)
  - Pipeline performance tuning

- [DEV Community - Rust CI with GitHub Actions](https://dev.to/bampeers/rust-ci-with-github-actions-1ne9)
  - Beginner-friendly guide

- [DEV Community - Getting Started with GitHub Actions for Rust](https://dev.to/rogertorres/getting-started-with-github-actions-for-rust-1o6g)
  - Quick start guide

### Advanced Topics
- [Fast Rust Builds](https://matklad.github.io/2021/09/04/fast-rust-builds.html)
  - Build performance fundamentals

- [RustSec Issue - Incremental Compilation Tracking](https://github.com/rust-lang/rust/issues/57968)
  - Incremental compilation discussion

- [Rust RFC - Incremental Compilation](https://rust-lang.github.io/rfcs/1298-incremental-compilation.html)
  - Official RFC documentation

- [depot.dev - sccache in GitHub Actions](https://depot.dev/blog/sccache-in-github-actions)
  - Compiler caching strategies

### Docker & Containers
- [Rust Binary and Docker Releases](https://codingpackets.com/blog/rust-binary-and-docker-releases-using-github-actions)
  - Docker and GitHub Actions integration

- [Medium - Containerise Rust Applications](https://medium.com/@emilia.jaser/containerise-rust-applications-on-ubuntu-alpine-with-github-actions-or-docker-builders-9378a02b98fd)
  - Container building techniques

- [GitHub Actions on ARM](https://cjwebb.com/rust-github-actions-on-arm/)
  - ARM-specific considerations

- [Multi-Arch Container Builds](https://github.com/f2calv/multi-arch-container-rust)
  - Example repository for multi-arch builds

### Helper Tools
- [Actions-RS Project](https://github.com/actions-rs/meta)
  - Community Rust GitHub Actions

- [Gist - GitHub Actions Rust Setup](https://gist.github.com/LukeMathWalker/5ae1107432ce283310c3e601fac915f3)
  - Quick reference setup

---

## Tools Reference Links

### Alternative/Supplementary Tools
- [cargo-deb](https://github.com/kornelski/cargo-deb)
  - Debian package generation

- [git-cliff](https://github.com/orhun/git-cliff)
  - Changelog generation

- [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks)
  - API breaking change detection

- [semantic-release-cargo](https://github.com/semantic-release-cargo/semantic-release-cargo)
  - Alternative semantic release integration

---

## By Category

### Essential (Must Have)
1. dtolnay/rust-toolchain
2. Swatinem/rust-cache
3. cargo test/clippy/fmt (built-in)

### Highly Recommended
1. cargo-nextest (for test speed)
2. release-plz (for releases)
3. rustsec/audit-check (for security)
4. taiki-e actions (for binaries)

### Nice to Have
1. cargo-tarpaulin (for coverage)
2. cargo-deb (for distributions)
3. git-cliff (for changelogs)
4. Docker actions (for containers)

---

## How to Use This Reference

### Finding Tool Documentation
- All tools are organized by category
- Each has links to GitHub, crates.io, or official docs
- Marketplace links provided for GitHub Actions

### Finding Examples
- Each exemplary repository is listed with workflow file URLs
- Direct links to live CI workflows on GitHub
- Can be viewed raw or cloned for reference

### Finding Learning Resources
- Articles organized by topic
- From beginner to advanced content
- Links to official documentation

---

## Quick Links by Need

**Need to install Rust?**
→ https://github.com/marketplace/actions/install-rust

**Need to cache dependencies?**
→ https://github.com/Swatinem/rust-cache

**Want faster tests?**
→ https://nexte.st/

**Want automated releases?**
→ https://release-plz.dev/

**Need security audits?**
→ https://github.com/rustsec/audit-check

**Need real examples?**
→ https://github.com/BurntSushi/ripgrep/blob/master/.github/workflows/

---

## Complete URL Count

- Official Documentation: 3
- Tools & Repositories: 15
- GitHub Actions: 15
- Exemplary Repositories: 4 (with 6 direct workflow links)
- Educational Resources: 20+
- Alternative Tools: 4
- **Total: 60+ URLs**

---

## Last Updated

December 27, 2025 - All links verified and current for 2025

---

## How to Update This Reference

When new tools, actions, or articles become available:
1. Verify the link and content
2. Add to appropriate section
3. Maintain alphabetical or logical order
4. Update the count at bottom
5. Update "Last Updated" date

---

## Notes

- All GitHub URLs use master/main branch (may vary by project)
- Marketplace links point to official actions
- Documentation links are official and current
- Educational resources focus on 2024-2025 content
- Some links may require GitHub authentication for private repositories
