# Phantom Git Layer Patterns - Research Documentation

## Overview

This research directory contains comprehensive documentation on implementing "phantom" or overlay Git systems that operate alongside the user's primary repository without interference. The research covers how sophisticated tools like git-worktree, git-annex, vcsh, and git-stash implement these patterns.

## Documents

### 1. phantom_git_patterns.md (32 KB)

**Comprehensive technical research covering:**

- **Overlay/Shadow Patterns**: How git-worktree manages multiple working directories, git-annex's key-value store for large files, git-stash's ref management, and vcsh's multi-repo coexistence
- **Custom Ref Namespaces**: Safe namespace usage, garbage collection considerations, conflict avoidance, and security limitations
- **Internal Git Storage**: Storing multiple logical branches without user-visible branches, commit orphaning prevention, and ref transactions
- **Best Practices for Parallel Git**: Managing separate databases, GIT_DIR redirection, and workflow isolation
- **Implementation Recommendations**: Specific guidance for Jin's architecture

**Key Sections:**
- 7 major pattern categories with detailed explanations
- 51 source citations to official documentation and research
- Architecture comparison table
- Appendices with quick-reference commands and decision trees

### 2. implementation_patterns.md (19 KB)

**Copy-paste-ready code patterns for implementation:**

- **Pattern 1**: Ref namespace-based layer storage
- **Pattern 2**: Separate database for complete isolation
- **Pattern 3**: Per-worktree phantom state
- **Pattern 4**: Transaction-based atomic operations
- **Pattern 5**: Reference-transaction hook validation
- **Pattern 6**: GC protection for custom refs
- **Pattern 7**: Layer snapshot and history
- **Pattern 8**: Danger-free experimental merges
- **Pattern 9**: Debugging and inspection tools
- **Pattern 10**: User workflow integration

**Each pattern includes:**
- Complete shell script implementations
- Configuration examples
- Usage examples
- Integration notes

## Key Findings

### Recommended Architecture for Jin

Based on comprehensive research across multiple tools:

1. **Primary: Ref Namespace Storage**
   - Use `refs/phantom/layers/*/refs/heads/*` for layer tracking
   - Completely invisible to `git branch`, `git tag` commands
   - Automatic garbage collection protection
   - Atomic updates via `git update-ref --stdin --atomic`

2. **Secondary: Optional Separate Database**
   - `.git/phantom.git/` for independent object storage
   - Better isolation if needed for multi-user scenarios
   - Independent garbage collection

3. **Atomic Operations**
   - All multi-ref updates use transaction control
   - Reference-transaction hook for validation
   - Rollback on any failure (all-or-nothing semantics)

4. **Worktree Support**
   - Per-worktree phantom state in `.git/worktrees/<id>/phantom/`
   - Shared layer definitions in main `.git/refs/phantom/`
   - Compatible with git 2.5.0+

### Safety Guarantees

- **Invisible to User**: Phantom refs don't appear in normal Git commands
- **Atomic**: Multiple-ref updates are all-or-nothing
- **GC-Safe**: Custom refs never garbage collected
- **Recallable**: All state preserved in reflogs for 30+ days
- **Per-Worktree**: Isolated state if using git-worktree
- **Transactional**: Explicit rollback on failure via hooks

## Research Sources

This research synthesizes information from:

- **Git Official Documentation** (git-scm.com)
- **Academic/Research Papers** (LWN.net, Hacker News)
- **Tool Repositories** (git, git-annex, vcsh, libgit2)
- **Case Studies** (DVC experiments, ShadowGit, git-shadow)
- **Community Best Practices** (Medium, Dev.to, Atlassian)

Over **20+ authoritative sources** were consulted for each major pattern.

## Quick Start for Implementation

1. **Start with Pattern 1** (Ref Namespaces) for basic layer tracking
2. **Add Pattern 4** (Atomic Operations) for safe multi-ref updates
3. **Implement Pattern 6** (GC Protection) for production safety
4. **Add Pattern 9** (Debugging Tools) for troubleshooting
5. **Integrate Pattern 10** (User Safety) for workflow isolation

Each pattern is self-contained and can be implemented independently or combined.

## Relevant for Jin's Architecture

The research directly informs:

- **Layer Storage Mechanism**: Using `refs/phantom/layers/` namespace
- **Atomic Merges**: Transaction-based layer synchronization
- **Ref Management**: Safe custom ref handling in shared repository
- **User Protection**: Ensuring overlay doesn't interfere with normal workflow
- **Garbage Collection**: Protecting layer commits from auto-cleanup
- **Debugging**: Tools for inspecting and recovering phantom state

## Technical Highlights

### Atomic Multi-Ref Updates

```bash
# Safe way to update multiple layers atomically
git update-ref --stdin --atomic << 'EOF'
start
update refs/phantom/layers/base/refs/heads/main abc123 def456
update refs/phantom/layers/changes/refs/heads/main def456 ghi789
prepare
commit
EOF
```

### Per-Worktree Safety

```bash
# Each worktree has isolated phantom state
.git/worktrees/test-next/phantom/
├── state                    # Per-worktree state
├── active_layers            # Which layers are active
└── metadata/                # Worktree-specific metadata
```

### User Workflow Protection

```bash
# User commands never see phantom refs
git branch -a                    # Only user branches
git log --all --graph           # Only user-accessible refs
git rev-parse refs/phantom/*    # Explicit only
```

## Implementation Checklist

- [ ] Read `phantom_git_patterns.md` section 1-3 for theory
- [ ] Review implementation patterns 1-5 for architecture
- [ ] Implement GC safety (Pattern 6)
- [ ] Add debugging tools (Pattern 9)
- [ ] Test user workflow isolation (Pattern 10)
- [ ] Document Jin's specific namespace conventions
- [ ] Add reference-transaction hook for validation
- [ ] Set up automatic snapshot mechanism
- [ ] Create recovery/restore procedures
- [ ] Test with multi-worktree scenarios

## File Guide

```
/home/dustin/projects/jin/plan/P1M2/research/
├── README.md                        # This file
├── phantom_git_patterns.md          # Full technical research (32 KB)
└── implementation_patterns.md       # Code patterns (19 KB)
```

## Next Steps

1. Use these documents as reference for Jin's phantom layer design
2. Adapt implementation patterns to Jin's specific use case
3. Create tests verifying user workflow isolation
4. Document Jin-specific ref namespace conventions
5. Implement incremental layers with atomic safeguards

---

Generated: December 26, 2025
Based on research from 20+ authoritative Git sources
