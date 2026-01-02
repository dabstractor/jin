# External References for Test Isolation Research

## Documentation URLs

### Rust Testing
- **[tempfile crate API](https://docs.rs/tempfile/latest/tempfile/)** - TempDir, NamedTempFile, automatic cleanup
- **[Rust Book - Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html#the-tests-directory)** - Integration test architecture
- **[Rust Book - Writing Tests](https://doc.rust-lang.org/book/ch11-01-writing-tests.html)** - Test fundamentals

### git2 Crate
- **[git2::Repository Documentation](https://docs.rs/git2/latest/git2/struct.Repository.html)** - Repository operations
- **[git2 on crates.io](https://crates.io/crates/git2)** - Crate information
- **[git2-rs GitHub](https://github.com/rust-lang/git2-rs)** - Source code
- **[Issue #194 - Send/Sync](https://github.com/rust-lang/git2-rs/issues/194)** - Thread safety implications

### Testing Frameworks
- **[rstest GitHub](https://github.com/la10736/rstest)** - Fixture-based testing
- **[git2-testing on crates.io](https://crates.io/crates/git2-testing)** - Git testing utilities

## Blog Posts and Articles

### Test Isolation
- **[Testing in Rust: Temporary Files](http://www.andrewra.dev/2019/03/01/testing-in-rust-temporary-files/)** - Fixture patterns
- **[Fun Fixture Tests in Rust](https://dzfrias.dev/blog/fun-fixtures/)** - Advanced fixture patterns
- **[Test setup and teardown in Rust](https://medium.com/@ericdreichert/test-setup-and-teardown-in-rust-without-a-framework-ba32d97aa5ab)** - Panic-resistant patterns

### Git Operations
- **[Git index.lock file handling](https://stackoverflow.com/questions/9282632/git-index-lock-file-exists-when-i-try-to-commit-but-i-cannot-delete-the-file)** - StackOverflow discussion

## Community Resources

### Rust Forums
- **[Best practices for managing test data](https://users.rust-lang.org/t/best-practices-for-managing-test-data/18979)** - Community discussion
- **[Unique ID per test](https://users.rust-lang.org/t/unique-id-per-test/33882)** - Test identifier strategies
- **[How to use git2::Remote::push correctly](https://users.rust-lang.org/t/how-to-use-git2-push-correctly/97202)** - Real-world git2 usage

## Code Examples

### Test Fixtures
- **[Cargo's git utilities](https://doc.rust-lang.org/nightly/nightly-rustc/src/cargo/sources/git/utils.rs.html)** - Reference implementation
- **[Awesome Rust Testing](https://github.com/hoodie/awesome-rust-testing)** - Testing tool collection
