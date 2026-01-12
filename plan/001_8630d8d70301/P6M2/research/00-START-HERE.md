# CLI Integration Testing and Git Workflow Testing Research

Welcome! This directory contains comprehensive research on testing patterns for CLI tools that interact with Git repositories.

## Quick Start (5 minutes)

### 1. Read This First
Start with **[QUICK_REFERENCE.md](./QUICK_REFERENCE.md)** for:
- Golden rule (test with real Git using local remotes)
- Checklists for your tests
- Decision trees for specific scenarios
- One-liners and templates

### 2. Then Read by Need
- **Testing multi-command workflows?** → [01-cli-multi-command-workflows.md](./01-cli-multi-command-workflows.md)
- **Testing Git operations?** → [02-git-layer-testing.md](./02-git-layer-testing.md)
- **Setting up test repos?** → [03-git-fixtures-setup-teardown.md](./03-git-fixtures-setup-teardown.md)
- **Testing filesystem ops?** → [04-filesystem-isolation.md](./04-filesystem-isolation.md)
- **Testing remote Git?** → [05-git-remote-mocking.md](./05-git-remote-mocking.md)
- **Testing error paths?** → [06-error-recovery-testing.md](./06-error-recovery-testing.md)
- **Testing transactions?** → [07-atomic-operations-testing.md](./07-atomic-operations-testing.md)

### 3. Full Overview
Read **[INDEX.md](./INDEX.md)** for:
- Complete topic coverage
- Real-world examples (Cargo, Git, Kubernetes)
- Tool recommendations by language
- Decision trees and anti-patterns

## The Golden Pattern

```
Test Implementation:
├─ Unit Tests
│  └─ Mock filesystem (fast, focused)
│
├─ Integration Tests
│  └─ Real Git with local filesystem remotes (actual behavior)
│  └─ Real filesystem with temp directories (cross-platform)
│
├─ Error Scenario Tests
│  └─ Safe error injection (don't corrupt data)
│
└─ Error Mix
   └─ 70% of tests should test error paths
```

## Key Findings

1. **Don't mock Git** - Use local filesystem repositories as remotes
2. **Use temp directories** - Automatic cleanup, isolation, parallel-safe
3. **Test errors thoroughly** - 70% of bugs are in error handling
4. **Use injectable dependencies** - Makes filesystem/Git operations testable
5. **Layer expensive setups** - Run once per suite, copy to isolated test dirs

## Document Guide

| Document | Purpose | Length | Time |
|----------|---------|--------|------|
| [QUICK_REFERENCE.md](./QUICK_REFERENCE.md) | Fast lookup, templates, checklists | 5 pages | 5 min |
| [INDEX.md](./INDEX.md) | Complete overview, all topics | 15 pages | 20 min |
| [01-07 Research Docs](./01-cli-multi-command-workflows.md) | Deep dives on specific topics | 60+ pages | 2+ hours |
| [SOURCES.md](./SOURCES.md) | All reference materials | 5 pages | 5 min |
| [RESEARCH_SUMMARY.md](./RESEARCH_SUMMARY.md) | Summary and statistics | 10 pages | 15 min |

## Real-World Examples

This research is validated by real production tools:

- **Cargo (Rust)** - Uses assert_cmd, assert_fs, tempfile
- **Git (C)** - Custom TAP framework, ~10,000 tests
- **Kubernetes/Prow** - Fake Git Server for HTTP testing
- **Pika Web Framework** - Layered setup with bundler matrix

## By Language

### Rust
See: [01](./01-cli-multi-command-workflows.md), [QUICK_REFERENCE](./QUICK_REFERENCE.md)
Tools: `assert_cmd`, `assert_fs`, `predicates`, `tempfile`

### Python
See: [INDEX.md](./INDEX.md) tool section, [QUICK_REFERENCE](./QUICK_REFERENCE.md)
Tools: `pytest`, `pytest-subprocess`, `tempfile`

### Bash
See: [02-git-layer-testing.md](./02-git-layer-testing.md), [QUICK_REFERENCE](./QUICK_REFERENCE.md)
Tools: `bats`, `expect`, native bash

### JavaScript/Node.js
See: [01-cli-multi-command-workflows.md](./01-cli-multi-command-workflows.md)
Tools: `judo`, `git-http-mock-server`, `mock-git`

### Go
See: [04-filesystem-isolation.md](./04-filesystem-isolation.md)
Tools: `testing`, `fstest.MapFS`, `tempfile`

## Common Scenarios

**I need to test...**

- CLI with multiple commands → [01-cli-multi-command-workflows.md](./01-cli-multi-command-workflows.md)
- Git clone/push/pull → [02-git-layer-testing.md](./02-git-layer-testing.md)
- Setting up test repos → [03-git-fixtures-setup-teardown.md](./03-git-fixtures-setup-teardown.md)
- File operations → [04-filesystem-isolation.md](./04-filesystem-isolation.md)
- Remote operations → [05-git-remote-mocking.md](./05-git-remote-mocking.md)
- Error handling → [06-error-recovery-testing.md](./06-error-recovery-testing.md)
- Multi-step operations → [07-atomic-operations-testing.md](./07-atomic-operations-testing.md)

## Statistics

- **Total Documentation**: 8,000+ lines
- **Code Examples**: 100+
- **Sources**: 44 authoritative sources
- **Languages Covered**: Rust, Python, Go, Bash, JavaScript
- **Real-World Tools**: Cargo, Git, Kubernetes, GitLab, more

## TL;DR

1. Use `t.TempDir()` or equivalent for isolated tests
2. Test with real Git, use local filesystem remotes
3. Make filesystem operations injectable
4. Write 70% error path tests, 10% happy path
5. Run tests in parallel to verify isolation
6. Don't mock Git commands - mock only network layer
7. Test on multiple platforms (use CI matrix)
8. See [QUICK_REFERENCE.md](./QUICK_REFERENCE.md) for templates

## Need Help?

- **Quick answer**: [QUICK_REFERENCE.md](./QUICK_REFERENCE.md)
- **Decision tree**: See decision trees in [QUICK_REFERENCE.md](./QUICK_REFERENCE.md)
- **Full explanation**: [INDEX.md](./INDEX.md)
- **Deep dive**: Individual research documents (01-07)
- **Sources**: [SOURCES.md](./SOURCES.md)

## Next Steps

1. ✓ Read [QUICK_REFERENCE.md](./QUICK_REFERENCE.md) (5 min)
2. ✓ Pick your scenario from [INDEX.md](./INDEX.md) (5 min)
3. ✓ Read relevant research document (30 min)
4. ✓ Look at code templates in [QUICK_REFERENCE.md](./QUICK_REFERENCE.md)
5. ✓ Implement with confidence!

---

**Last Updated**: December 27, 2025
**Total Time Investment**: 2-3 hours to understand all topics, 30 minutes to apply to your project
**Expected Result**: Reliable, fast, maintainable integration tests
