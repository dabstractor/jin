# CLI Integration Testing and Git Workflow Testing Research Index

## Overview

This directory contains comprehensive research on testing patterns for CLI tools that interact with Git repositories. The research covers patterns from real-world tools like Cargo, Git itself, and Kubernetes infrastructure, providing evidence-based recommendations for reliable, fast, and maintainable tests.

## Document Structure

### 1. [CLI Multi-Command Workflows](./01-cli-multi-command-workflows.md)
Testing complex CLI workflows with multiple sequential commands.

**Key Topics**:
- Layered setup approach (expensive setup once, reuse across tests)
- Async process control for long-running commands
- Directory and file management patterns
- Multi-channel verification (exit codes, stdout, stderr, file changes, HTTP)
- Environment testing matrix (OS, version, bundler combinations)
- Port management for parallel tests
- Real-world examples from Cargo, Pika Web Framework
- Testing frameworks: Judo, Aruba, Bats, Expect

**Best for**: Designing test suites for tools with multi-step workflows

### 2. [Git Layer Testing](./02-git-layer-testing.md)
Testing Git operations and validating repository state.

**Key Topics**:
- Local repository testing (preferred over mocking)
- Repository state validation techniques
- Repository fixture patterns (pre-built vs. custom)
- Git operations testing pattern
- Avoiding common mocking pitfalls
- Error handling and recovery in Git workflows
- Real examples from Git's own test suite (C, TAP framework)
- Bats testing framework for Git operations

**Best for**: Ensuring actual Git behavior is tested, not mocked behavior

### 3. [Git Fixtures and Setup/Teardown](./03-git-fixtures-setup-teardown.md)
Patterns for initializing and cleaning up test repositories.

**Key Topics**:
- Fixture lifecycle (setup → test → teardown)
- Temporary directory utilities (language-specific)
- Manual cleanup patterns with guarantees
- Repository initialization patterns
- Creating test fixtures with specific state
- Remote repository setup
- Pytest fixtures for Git testing
- Bats test framework patterns
- Common mistakes and solutions

**Best for**: Reliable test isolation and reproducible test environments

### 4. [Filesystem Isolation](./04-filesystem-isolation.md)
Testing filesystem operations in isolation with proper mocking.

**Key Topics**:
- Dependency injection pattern (injectable filesystem interfaces)
- In-memory filesystem mocks (Go's fstest.MapFS)
- Temporary directory pattern for integration tests
- RAM-based filesystems (tmpfs) for speed
- Error injection with mock filesystems
- State tracking in mocks
- Process isolation patterns (separate working directory per test)
- File assertion patterns
- Real-world examples from Rust and Ruby

**Best for**: Fast, deterministic filesystem testing with proper isolation

### 5. [Git Remote Mocking](./05-git-remote-mocking.md)
Mocking and testing Git remote operations without network dependencies.

**Key Topics**:
- Local filesystem remotes (preferred approach)
- git-http-mock-server (copy-on-write for parallel tests)
- Fake Git Server (FGS) for HTTP Git protocol
- Selective mocking strategies (mock network, not Git)
- Authentication testing scenarios (Basic Auth, SSH, tokens)
- Concurrent access testing (parallel pushes)
- Testing error conditions (server errors, timeouts, rejections)
- Decision tree: what to mock vs. what to test

**Best for**: Testing remote operations without real network calls

### 6. [Error Condition and Recovery Testing](./06-error-recovery-testing.md)
Testing error paths and recovery mechanisms thoroughly.

**Key Topics**:
- Error categories (system, Git-specific, network)
- Recovery scenario testing
- Error injection techniques (mocks, conditionals, environment)
- Specific test cases (conflicts, corrupted repos, permissions)
- Exit code conventions
- Error message quality
- Rollback mechanisms
- Incremental recovery and checkpoints
- Testing cleanup on failure

**Best for**: Ensuring robust error handling and recovery paths

### 7. [Atomic Operations Testing](./07-atomic-operations-testing.md)
Testing transactional operations and atomicity guarantees.

**Key Topics**:
- Atomicity guarantees (all-or-nothing)
- Transaction boundaries
- Git atomicity patterns (refs, commits, pushes)
- Rollback testing patterns
- Savepoint-based recovery
- Nested transaction testing
- Crash safety verification
- Consistency verification before/after transactions
- Durability verification
- Locking and mutual exclusion
- Deadlock detection

**Best for**: Ensuring complex multi-step operations are reliable

## Key Patterns Summary

### Setup and Isolation

**Pattern: Layered Setup**
```
Expensive setup (once per suite)
  ↓
Copy artifacts to isolated temp dir (per test)
  ↓
Run test in isolated environment
  ↓
Auto cleanup
```

**Benefits**: Performance, isolation, parallelism

### Testing Strategy

**Pattern: Real Git + Local Filesystem Remotes**
```
- Unit tests: Mock filesystem interfaces
- Integration tests: Real Git with local bare repos
- Network tests: Mock HTTP layer only
- Remote tests: git-http-mock-server with copy-on-write
```

**Benefits**: Tests actual behavior, survives Git updates, fast

### Error Testing

**Pattern: Errors are First-Class**
```
Test mix should be approximately:
- 10% happy path
- 70% error conditions
- 15% recovery scenarios
- 5% edge cases
```

**Benefits**: Catches issues early, robust error handling

## Real-World Implementation Examples

### Cargo (Rust Package Manager)
- Integration tests in `/tests` directory
- Uses `assert_cmd` for CLI testing
- Uses `assert_fs` for filesystem assertions
- Tests both success and failure paths
- Covers multiple platforms and configurations

### Git (C Implementation)
- Custom TAP (Test Anything Protocol) framework
- Pure C unit tests alongside shell tests
- Covers edge cases and error conditions
- ~10,000 tests
- Fast execution via unit tests rather than integration tests

### Pika Web Framework
- Layered setup with shared build system
- Tests against multiple bundlers (webpack, esbuild, vite)
- CI matrix for Windows, Linux, macOS
- Free port detection for parallel servers
- File verification and HTTP assertions

### Kubernetes Infrastructure (Prow)
- Fake Git Server for HTTP Git protocol testing
- Real Git repositories served over HTTP
- Supports concurrent test execution
- Automatic cleanup

## Tool Recommendations by Use Case

### For Bash/Shell CLIs
- **Bats**: TAP-compliant testing framework
- **Expect**: Process control with interaction
- **tempfile**: Portable temp directories

### For Rust/Cargo
- **assert_cmd**: Command execution and assertions
- **assert_fs**: Filesystem fixtures and assertions
- **predicates**: Pattern matching for outputs
- **tempfile**: Temp directory management

### For Python
- **pytest**: Fixtures with setup/teardown
- **pytest-subprocess**: Process mocking
- **tmp_path**: Automatic temp directory cleanup
- **subprocess**: Real process execution

### For Node.js/JavaScript
- **Judo**: YAML-driven CLI testing
- **mock-git**: Git command mocking
- **git-http-mock-server**: HTTP Git protocol mocking
- **node-temp**: Temp directory management

## Quick Decision Trees

### How to Test Git Operations?

```
Are you testing Git's behavior itself?
├─ YES → Use local filesystem repositories as remotes
└─ NO → Use real Git with local repos
        (Don't mock git commands)

Are you testing authentication?
├─ YES → Mock HTTP layer with credentials validation
└─ NO → Use local repos or git-http-mock-server

Are you testing network errors?
├─ YES → Mock HTTP server or inject network failures
└─ NO → Avoid network entirely
```

### How to Test Filesystem Operations?

```
Is this a unit test?
├─ YES → Use mock filesystem (MapFS, in-memory)
└─ NO (integration test) → Use temp directory

Do you need to test error conditions?
├─ YES → Use error-injecting mock filesystem
└─ NO → Use real filesystem with temp dir

Do you need to test permissions?
├─ YES → Use real filesystem with proper permissions
└─ NO → Mock is fine

Do you need speed?
├─ YES → Use tmpfs (RAM-based) for integration tests
└─ NO → Regular temp directory is fine
```

### How to Test Error Scenarios?

```
Are you testing YOUR error handling code?
├─ YES → Inject specific errors, test response
└─ NO → Don't test

Can you simulate the error condition realistically?
├─ YES → Do it (real permissions, disk full, etc.)
└─ NO → Mock the error

Is the error condition rare or dangerous?
├─ YES → Use safe injection (mock, env vars)
└─ NO → Can cause real condition in test
```

## Testing Anti-Patterns to Avoid

1. **Mocking Git commands** - Tests become brittle, don't verify real behavior
2. **Shared test state** - Tests interfere with each other unpredictably
3. **Hardcoded paths** - Tests don't work in CI/different environments
4. **No cleanup** - Test files accumulate and cause interference
5. **Only happy path testing** - 70% of bugs are in error handling
6. **Ignoring platform differences** - CRLF, permissions, path separators
7. **Slow tests with network** - Makes test suite agonizingly slow
8. **No timeout on long operations** - Tests hang indefinitely
9. **Relying on timing** - Tests fail unpredictably
10. **Over-abstraction in test code** - Tests become harder to understand

## Measuring Test Quality

### Metrics to Track

1. **Coverage**: % of code paths executed by tests
2. **Flakiness**: % of tests that fail inconsistently
3. **Speed**: Total test suite runtime
4. **Isolation**: % of tests that pass when run individually
5. **Maintainability**: Time to update tests vs. implementation

### Target Goals

- **Coverage**: >80% for CLI tools (>70% acceptable)
- **Flakiness**: <1% (detect and fix immediately)
- **Speed**: Full suite <5 minutes
- **Isolation**: 100% (all tests pass in any order)
- **Maintainability**: Tests shouldn't require more upkeep than code

## Getting Started

### For a New CLI Project

1. Start with `/tests` directory for integration tests
2. Separate business logic into lib.rs/lib.py for unit testing
3. Use dependency injection for filesystem/Git operations
4. Write both happy path and error tests
5. Use temporary directories for isolation
6. Test on multiple platforms early

### For an Existing Project

1. Add error path testing (often missing)
2. Fix flaky tests (usually timing/environment issues)
3. Improve test isolation (likely root cause of flakiness)
4. Add filesystem/Git operation tests
5. Establish CI matrix for multiple platforms
6. Document test patterns for team

## References and Further Reading

### Official Documentation
- [Git Testing Documentation](https://git-scm.com/docs/unit-tests)
- [Cargo Testing Guide](https://doc.rust-lang.org/cargo/guide/tests.html)
- [Rust CLI Book: Testing](https://rust-cli.github.io/book/tutorial/testing.html)
- [GitLab Testing Best Practices](https://docs.gitlab.com/ee/development/testing_guide/best_practices.html)

### Tools and Frameworks
- [assert_cmd - Rust CLI testing](https://docs.rs/assert_cmd/)
- [Bats - Bash testing](https://github.com/bats-core/bats-core)
- [Judo - YAML-driven CLI testing](https://github.com/intuit/judo)
- [git-http-mock-server](https://github.com/isomorphic-git/git-http-mock-server)

### Articles and Guides
- [DEV Community: CLI Integration Tests](https://dev.to/florianrappl/how-we-wrote-our-cli-integration-tests-53i3)
- [Stefan Zweifel: Testing git-auto-commit](https://stefanzweifel.dev/posts/2020/12/22/writing-integration-tests-for-git-auto-commit/)
- [Testing Filesystem Code](https://dev.to/rezmoss/testing-file-system-code-mocking-stubbing-and-test-patterns-99-1fkh)
- [Alex W-L Chan: Testing Rust CLI Apps](https://alexwlchan.net/2025/testing-rust-cli-apps-with-assert-cmd/)

## Document Maintenance

**Last updated**: 2025-12-27
**Coverage scope**: CLI integration testing, Git workflows, filesystem operations
**Sources**: 25+ authoritative sources including official documentation, real-world tool implementations, and best practices guides

---

## Quick Links

- [Setup/Teardown Patterns](./03-git-fixtures-setup-teardown.md) - How to initialize tests
- [Error Testing](./06-error-recovery-testing.md) - Testing error paths
- [Git Testing](./02-git-layer-testing.md) - How to test Git operations
- [Filesystem Testing](./04-filesystem-isolation.md) - Isolated filesystem tests
- [Atomic Operations](./07-atomic-operations-testing.md) - Multi-step operations
- [Remote Operations](./05-git-remote-mocking.md) - Testing without network
- [Multi-Command Workflows](./01-cli-multi-command-workflows.md) - Complex workflows
