# Git Refspec Patterns and Custom Namespace Operations Research

## Executive Summary

This document provides comprehensive research on Git refspec syntax, custom namespace operations, and real-world patterns used by tools like Gerrit, GitHub, and GitLab. Refspecs are fundamental to understanding how Git fetch and push operations map references between local and remote repositories.

---

## 1. Git Refspec Fundamentals

### 1.1 What is a Refspec?

A **refspec** is a mapping that specifies how references (branches and tags) on a remote repository are tracked or pushed locally. It controls the source and destination of fetch and push operations.

**Source:** [Git - The Refspec (ProGit)](https://git-scm.com/book/en/v2/Git-Internals-The-Refspec)

### 1.2 Refspec Syntax

The basic format is:

```
[+]<src>:<dst>
```

**Components:**

- **`+`** (optional): Force update even if it's not a fast-forward. Allows non-ancestral updates and history rewrites.
- **`<src>`**: Pattern for references on the remote side (source of fetch, destination of push)
- **`:`**: Separator between source and destination
- **`<dst>`**: Where those references are tracked locally (for fetch) or pushed to (for push)

**Example:**
```
+refs/heads/master:refs/remotes/origin/master
```

### 1.3 Default Refspec

When you add a remote with `git remote add`, Git automatically creates a default fetch refspec:

```
[remote "origin"]
    url = https://github.com/user/repo.git
    fetch = +refs/heads/*:refs/remotes/origin/*
```

This means:
- Fetch all branches from `refs/heads/` on the remote
- Store them locally under `refs/remotes/origin/`
- The `+` allows non-fast-forward updates

---

## 2. Refspec Pattern Matching and Wildcards

### 2.1 Glob Pattern Syntax

Refspecs support simple glob patterns using `*` for pattern matching:

```
refs/heads/*:refs/remotes/origin/*
```

**Rules:**
- A pattern refspec must have **exactly one `*`** in both `<src>` and `<dst>`
- The `*` matches any ref matching the pattern
- The matched portion replaces the `*` in the destination

**Examples:**

| Refspec | Behavior |
|---------|----------|
| `refs/heads/*:refs/remotes/origin/*` | Fetch all branches |
| `refs/heads/master:refs/remotes/origin/master` | Fetch only master |
| `refs/tags/*:refs/tags/*` | Fetch all tags |
| `refs/heads/qa/*:refs/remotes/origin/qa/*` | Fetch all QA branches into qa namespace |
| `refs/heads/feature/*:refs/remotes/origin/feature/*` | Fetch feature branches into namespace |

### 2.2 Tag Shorthand

Git provides shorthand notation for tags:

```
tag <tagname>
# Equivalent to:
refs/tags/<tagname>:refs/tags/<tagname>
```

---

## 3. Custom Namespaces in Git

### 3.1 How Git Namespaces Work

Git supports dividing refs of a single repository into multiple namespaces, each with its own branches, tags, and HEAD. This allows multiple logical repositories to share the same object store.

**Source:** [Git - gitnamespaces Documentation](https://git-scm.com/docs/gitnamespaces)

### 3.2 Namespace Storage Structure

Namespaces are specified via the `GIT_NAMESPACE` environment variable or `--namespace` option:

```bash
GIT_NAMESPACE=<namespace> git upload-pack
GIT_NAMESPACE=<namespace> git receive-pack
```

**Storage:**
- Refs are stored under `refs/namespaces/<namespace>/`
- Example: `GIT_NAMESPACE=foo` stores refs under `refs/namespaces/foo/`

### 3.3 Hierarchical Namespaces

Namespaces with forward slashes create hierarchical structures:

```
GIT_NAMESPACE=foo/bar
# Stores refs under: refs/namespaces/foo/refs/namespaces/bar/
```

**Benefits:**
- Avoids directory/file conflicts
- Cloning with `GIT_NAMESPACE=foo/bar` produces same result as chained clones
- Maintains namespace hierarchy semantics

### 3.4 Custom Namespace Examples

#### Creating Custom Namespaces

```bash
# Expose repository as namespace 'foo'
GIT_NAMESPACE=foo git upload-pack /path/to/repo

# Clone from a namespace
git clone ext::'git --namespace=foo %s /tmp/prefixed.git'
```

#### Using Refspecs with Custom Namespaces

```bash
# Fetch all branches into custom namespace
git fetch origin refs/heads/*:refs/remotes/origin/custom/*

# Push to custom namespace on server
git push origin HEAD:refs/custom/mybranch

# Multiple namespace fetches
git fetch origin \
  refs/heads/master:refs/remotes/origin/master \
  refs/heads/qa/*:refs/remotes/origin/qa/*
```

### 3.5 Custom Namespace Security Considerations

**Important:** Namespaces are **not** effective for read access control.

From the official documentation:
> "Namespaces on a server are not effective for read access control; you should only grant read access to a namespace to clients that you would trust with read access to the entire repository."

The fetch/push protocols aren't designed to prevent data theft between repositories.

---

## 4. Gerrit's Custom Refs Namespace

### 4.1 The refs/for Namespace

Gerrit uses a custom `refs/for/[BRANCH_NAME]` namespace to distinguish code submissions for review from direct commits.

**Source:** [The refs/for namespace (Gerrit Documentation)](https://gerrit-review.googlesource.com/Documentation/concept-refs-for-namespace.html)

### 4.2 Push for Review Workflow

To submit code for review in Gerrit:

```bash
# Push to refs/for namespace for review
git push origin HEAD:refs/for/master

# Push with topic for grouping
git push origin HEAD:refs/for/master%topic=my-feature

# Push for specific branch
git push origin HEAD:refs/for/develop
```

### 4.3 Internal Reference Mapping

When you push to `refs/for/[BRANCH]`, Gerrit creates internal references under `refs/changes/`:

```
refs/changes/[CD]/[ABCD]/[EF]
```

**Components:**
- **[CD]**: Final two digits of change number
- **[ABCD]**: Complete change number
- **[EF]**: Patch set number

This mapping allows Gerrit to:
- Track change iterations
- Present unified interface to developers
- Manage code review metadata

### 4.4 Reserved Namespaces in Gerrit

Gerrit reserves these namespaces for its own use:
- `refs/for/*` - Code review submissions
- `refs/meta/*` - Gerrit metadata
- `refs/heads/*` - Regular branches
- `refs/tags/*` - Tags

When importing external repositories into Gerrit, conflicts with these namespaces will cause errors.

---

## 5. GitHub Pull Request Custom Refs

### 5.1 Pull Request Ref Structure

GitHub exposes two refs per pull request:

```
refs/pull/<PR_NUMBER>/head  # Points to the PR branch tip
refs/pull/<PR_NUMBER>/merge # Points to hypothetical merge commit
```

**Example:**
- PR #45 tip: `refs/pull/45/head`
- PR #45 merge: `refs/pull/45/merge`

**Source:** [Checkout github pull requests locally](https://gist.github.com/piscisaureus/3342247)

### 5.2 Fetching GitHub Pull Requests

#### One-Time Fetch

```bash
# Fetch a specific PR without configuration
git fetch origin pull/7324/head:pr-7324
```

#### Persistent Configuration

Add to `.git/config`:

```ini
[remote "origin"]
    url = git@github.com:username/repo.git
    fetch = +refs/heads/*:refs/remotes/origin/*
    fetch = +refs/pull/*/head:refs/remotes/origin/pr/*
```

After adding this refspec:

```bash
# Fetch all PRs
git fetch origin

# Checkout a PR
git checkout pr/999

# View PR metadata
git show-ref | grep pull
```

### 5.3 GitHub PR Refs Limitations

**Important limitations:**

- GitHub **does not allow pushing** to pull request refs
- PR refs are **read-only**
- Only repository maintainers can update PRs via the GitHub interface
- Contributors must push to their fork and open a new PR

---

## 6. Force Fetch Semantics (The + Prefix)

### 6.1 Fast-Forward vs. Non-Fast-Forward Updates

By default, Git only allows **fast-forward updates** (updates that only add new commits):

```
A -- B -- C  (old state)
          |
          D -- E  (new commits to fetch)
```

Non-fast-forward updates involve history rewriting:

```
A -- B -- C (old state)

A -- B -- X -- Y  (force-updated history)
```

### 6.2 The + Prefix

Adding `+` to a refspec allows non-fast-forward updates:

```
+refs/heads/*:refs/remotes/origin/*
```

This tells Git to:
- Update the reference even if it's not a fast-forward
- Allow history rewrites
- Overwrite local branch tips

### 6.3 Force Fetch Equivalent

The `--force-if-includes` and `--force-with-lease` options provide safer alternatives:

```bash
# Equivalent to + in refspec
git fetch origin --force

# Safer: only force if we have the expected state
git fetch origin --force-if-includes

# Safest: allow force only if remote matches last fetch
git fetch origin --force-with-lease
```

### 6.4 Constraints on Force Updates

From the official documentation:
> "No amount of forcing will make the `refs/heads/*` namespace accept a non-commit object."

This means forced updates must still point to commit objects, preventing corruption.

---

## 7. Negative Refspecs

### 7.1 Syntax and Purpose

Negative refspecs use the `^` prefix to exclude refs:

```
^refs/heads/dontwant
```

A ref matches if:
1. It matches at least one **positive** refspec
2. AND it does NOT match any **negative** refspec

### 7.2 Examples

#### Exclude a Single Branch

```bash
# Fetch all branches except 'dontwant'
git fetch origin refs/heads/*:refs/remotes/origin/* ^refs/heads/dontwant
```

#### Exclude Multiple Branches

```bash
git fetch origin \
  refs/heads/*:refs/remotes/origin/* \
  ^refs/heads/dontwant \
  ^refs/heads/deprecated
```

#### Exclude Pattern Matching

```bash
# Fetch all except branches starting with 'foo/'
git fetch origin \
  refs/heads/*:refs/remotes/origin/* \
  ^refs/heads/foo/*
```

### 7.3 Negative Refspec Limitations

From the documentation:
- Negative refspecs **can be patterns** themselves
- Negative refspecs **may only contain source** (no destination)
- Fully spelled out hex object names are **not supported** for negative refspecs

---

## 8. Fetching Only Specific Refs

### 8.1 Selective Fetch Strategies

#### Fetch Single Branch

```bash
# Configuration approach
[remote "origin"]
    fetch = +refs/heads/master:refs/remotes/origin/master

# Command-line approach
git fetch origin master:refs/remotes/origin/master
```

#### Fetch Multiple Specific Branches

```bash
# Using command-line with multiple refspecs
git fetch origin \
  master:refs/remotes/origin/mymaster \
  topic:refs/remotes/origin/topic
```

#### Fetch All Tags Only

```bash
# Using -t flag
git fetch origin -t

# Equivalent refspec
git fetch origin refs/tags/*:refs/tags/*
```

#### Fetch All Branches

```bash
# Standard approach
git fetch origin refs/heads/*:refs/remotes/origin/*

# Fetch with all tags
git fetch origin \
  refs/heads/*:refs/remotes/origin/* \
  refs/tags/*:refs/tags/*
```

### 8.2 Config-Based Selective Fetch

```ini
[remote "origin"]
    url = https://github.com/user/repo.git

    # Fetch master and qa branches
    fetch = +refs/heads/master:refs/remotes/origin/master
    fetch = +refs/heads/qa/*:refs/remotes/origin/qa/*

    # Also fetch pull requests
    fetch = +refs/pull/*/head:refs/remotes/origin/pr/*
```

### 8.3 Prefetch Namespace

Git 2.30+ introduced `--prefetch` which stores refs in `refs/prefetch/`:

```bash
git fetch --prefetch origin
# Refs stored in refs/prefetch/origin/* instead of refs/remotes/origin/*
```

---

## 9. Bidirectional Sync Patterns

### 9.1 Challenges of Bidirectional Mirroring

Source: [Bidirectional mirroring (GitLab Documentation)](https://docs.gitlab.com/user/project/repository/mirror/bidirectional/)

**Key challenges:**
1. **Race conditions**: Simultaneous commits to same branch cause conflicts
2. **History rewriting**: Rewriting mirrored commits causes failures
3. **No guarantees**: Either repository may fail to update without errors
4. **Performance impact**: Pre-receive hooks negatively affect push operations

### 9.2 Mirror Configuration

#### Simple Mirror (One-Way)

```bash
# Clone mirror
git clone --mirror <source-url> mirror.git

# Push to target
git push --mirror <target-url>
```

#### Using Mirror Refspecs

The `--mirror` option creates a bare repo mapping all refs:

```bash
git push --mirror \
  refs/heads/*:refs/heads/* \
  refs/tags/*:refs/tags/*
```

### 9.3 Selective Mirroring (Avoid Platform Refs)

**Important:** Hosting platforms store metadata in proprietary refs that cannot be pushed.

**GitHub Platform Refs:**
- `refs/pull/*` - Pull request metadata (read-only)

**Azure Repos Platform Refs:**
- `refs/pull/*` - Pull request metadata (read-only)

**Solution:** Mirror only standard refs:

```bash
git push target \
  refs/heads/*:refs/heads/* \
  refs/tags/*:refs/tags/* \
  refs/notes:refs/notes
```

### 9.4 Recommended Bidirectional Patterns

#### Protected Branch Mirroring

```bash
# Mirror only protected branches
git push github refs/heads/main:refs/heads/main
git push github refs/heads/develop:refs/heads/develop
git push github refs/tags/*:refs/tags/*
```

#### Push Event Webhook Sync

Instead of continuous mirroring:
1. Set up push event webhooks
2. Trigger sync on push events
3. Reduces race condition window
4. Better performance

#### Configuration Example

```ini
[remote "github"]
    url = https://github.com/user/repo.git
    fetch = +refs/heads/main:refs/heads/main
    fetch = +refs/heads/develop:refs/heads/develop
    push = refs/heads/main:refs/heads/main
    push = refs/heads/develop:refs/heads/develop

[remote "gitlab"]
    url = https://gitlab.com/user/repo.git
    fetch = +refs/heads/main:refs/heads/main
    push = refs/heads/main:refs/heads/main
```

### 9.5 Best Practices for Bidirectional Sync

1. **Mirror only protected branches** - reduces conflicts
2. **Use push event webhooks** - better than continuous polling
3. **Protect branches on both remotes** - prevents history rewrites
4. **Exclude platform-specific refs** - avoid read-only ref conflicts
5. **Monitor for conflicts** - set up alerts for mirror failures
6. **Use pre-receive hooks carefully** - consider performance impact

---

## 10. Real-World Examples from Tools

### 10.1 Gerrit Code Review

**Namespace:** `refs/for/*` and `refs/changes/*`

**Workflow:**
```bash
# Developer pushes for review
git push origin HEAD:refs/for/master

# Gerrit creates tracking refs internally
refs/changes/12/312/1
refs/changes/12/312/2  # Patch set 2
```

**Benefits:**
- Distinguishes review requests from commits
- Tracks multiple patch set iterations
- Integrates with code review UI

### 10.2 GitHub Pull Requests

**Namespace:** `refs/pull/*`

**Configuration:**
```ini
[remote "origin"]
    fetch = +refs/pull/*/head:refs/remotes/origin/pr/*
```

**Usage:**
```bash
git fetch origin
git checkout pr/999
```

**Benefits:**
- Access PRs without cloning forks
- Efficient namespace isolation
- Works with standard Git workflows

### 10.3 GitLab Merge Requests

**Namespace:** `refs/merge-requests/*` (when fetching from GitLab)

**Configuration:**
```ini
[remote "origin"]
    fetch = +refs/merge-requests/*/head:refs/remotes/origin/mr/*
```

### 10.4 DVC (Data Version Control)

**Custom Refs:** `refs/exps/*` for ML experiments

**Purpose:**
- Track ML experiments without polluting main refs
- Local-only references (not transferred on push/pull)
- Useful for experimentation workflows

**Benefits:**
- Keeps experiment refs separate
- No remote namespace pollution
- Perfect for local-only metadata

---

## 11. Common Pitfalls and Gotchas

### 11.1 Branch Name Mismatches

**Problem:** Error when default branch name doesn't match

```
error: src refspec main does not match any
```

**Cause:** Repository uses 'main' but refspec specifies 'master'

**Solution:** Verify branch exists

```bash
git branch -a
git branch -r  # Remote branches
```

### 11.2 Accidental Branch Deletion

**Problem:** Empty source deletes destination

```bash
git push origin :topic
# Deletes 'topic' branch on remote
```

**Safe Alternative (Git 1.7.0+):**
```bash
git push origin --delete topic
```

### 11.3 Pattern Matching Errors

**Problem:** Multiple `*` in refspec

```bash
# WRONG: Two wildcards
refs/heads/*/bugfix/*:refs/remotes/origin/*

# CORRECT: Single wildcard
refs/heads/*/bugfix:refs/remotes/origin/*/bugfix
```

### 11.4 Non-Fast-Forward Rejection

**Problem:** Fetch rejected for non-fast-forward update

```
! [rejected] update for ref 'refs/heads/feature'
```

**Cause:** Local ref is ahead of remote (history rewritten on remote)

**Solution:** Use `+` prefix or `--force`

```bash
# In config
fetch = +refs/heads/*:refs/remotes/origin/*

# Or command-line
git fetch --force
```

### 11.5 Confusing Colon Behavior

**Problem:** Missing colon has different meaning

```bash
git fetch origin master        # Fetch and merge to current branch
git fetch origin master:topic  # Fetch to topic branch only
```

### 11.6 Read-Only Namespace Failures

**Problem:** Cannot push to platform-specific refs

```bash
git push origin refs/pull/45/head:refs/pull/45/head
# ERROR: Cannot push to read-only ref on GitHub
```

**Solution:** Only fetch PR refs, don't push to them

### 11.7 Namespace Access Control Misconception

**Gotcha:** Namespaces don't provide read access control

```bash
GIT_NAMESPACE=secret git clone <url>
# User can still access other namespaces if they have repo access
```

---

## 12. Best Practices Summary

### 12.1 Configuration Best Practices

```ini
[remote "origin"]
    url = https://github.com/user/repo.git

    # Always use + for fetch to allow history rewrites
    fetch = +refs/heads/*:refs/remotes/origin/*

    # Include PRs if working with GitHub
    fetch = +refs/pull/*/head:refs/remotes/origin/pr/*

    # Use specific refspecs for push if needed
    push = refs/heads/master:refs/heads/main
```

### 12.2 Fetch Best Practices

1. **Verify branch names** before attempting fetch
2. **Use specific refspecs** for large repositories (avoid fetching all)
3. **Use `--force-with-lease`** instead of `--force` when possible
4. **Use negative refspecs** to exclude experimental branches
5. **Document custom refspecs** in team guidelines

### 12.3 Push Best Practices

1. **Avoid force push** to shared branches
2. **Use `--force-if-includes`** if force push is necessary
3. **Review refspec destination** carefully (easy to push to wrong branch)
4. **Prefer `--delete`** over empty source deletion
5. **Test with dry-run first** for complex refspecs

### 12.4 Namespace Best Practices

1. **Use hierarchical namespaces** for organization (e.g., `team/project`)
2. **Document namespace purposes** in your workflow documentation
3. **Use namespaces for separation** not access control
4. **Mirror only standard refs** (avoid platform-specific refs)
5. **Protect critical namespaces** at the repository level

### 12.5 Mirror/Sync Best Practices

1. **Mirror only protected branches** to reduce conflicts
2. **Use webhook triggers** instead of continuous polling
3. **Exclude platform-specific refs** (GitHub, GitLab, Azure)
4. **Monitor mirror health** with alerts
5. **Document mirror direction** and conflict resolution

---

## 13. Official Documentation References

### Core Git Documentation

- **ProGit Book - The Refspec:** [https://git-scm.com/book/en/v2/Git-Internals-The-Refspec](https://git-scm.com/book/en/v2/Git-Internals-The-Refspec)
- **Git Namespaces:** [https://git-scm.com/docs/gitnamespaces](https://git-scm.com/docs/gitnamespaces)
- **git-fetch Documentation:** [https://git-scm.com/docs/git-fetch](https://git-scm.com/docs/git-fetch)
- **git-push Documentation:** [https://git-scm.com/docs/git-push](https://git-scm.com/docs/git-push)
- **git-check-ref-format:** [https://git-scm.com/docs/git-check-ref-format](https://git-scm.com/docs/git-check-ref-format)

### Gerrit Documentation

- **Gerrit - The refs/for namespace:** [https://gerrit-review.googlesource.com/Documentation/concept-refs-for-namespace.html](https://gerrit-review.googlesource.com/Documentation/concept-refs-for-namespace.html)
- **Pushing a Commit (Gerrit):** [https://gerrit-review.googlesource.com/Documentation/user-upload.html](https://gerrit-review.googlesource.com/Documentation/user-upload.html)

### GitHub Documentation

- **GitHub - Using Git refs to checkout PRs:** [https://gist.github.com/piscisaureus/3342247](https://gist.github.com/piscisaureus/3342247)
- **ProGit - GitHub - Maintaining a Project:** [https://git-scm.com/book/en/v2/GitHub-Maintaining-a-Project](https://git-scm.com/book/en/v2/GitHub-Maintaining-a-Project)

### GitLab Documentation

- **GitLab - Bidirectional mirroring:** [https://docs.gitlab.com/user/project/repository/mirror/bidirectional/](https://docs.gitlab.com/user/project/repository/mirror/bidirectional/)
- **GitLab - Repository mirroring:** [https://docs.gitlab.com/user/project/repository/mirror/](https://docs.gitlab.com/user/project/repository/mirror/)

### Other References

- **Atlassian Git Tutorials - Refs:** [https://www.atlassian.com/git/tutorials/refs-and-the-reflog](https://www.atlassian.com/git/tutorials/refs-and-the-reflog)
- **Edward Thomson - Mirroring Git Repositories:** [https://www.edwardthomson.com/blog/mirroring_git_repositories](https://www.edwardthomson.com/blog/mirroring_git_repositories)
- **Git Cookbook - Refspecs:** [https://git.seveas.net/the-meaning-of-refs-and-refspecs.html](https://git.seveas.net/the-meaning-of-refs-and-refspecs.html)

---

## 14. Refspec Quick Reference

### Common Patterns

```bash
# Fetch all branches
+refs/heads/*:refs/remotes/origin/*

# Fetch only master
+refs/heads/master:refs/remotes/origin/master

# Fetch all tags
+refs/tags/*:refs/tags/*

# GitHub PR refs
+refs/pull/*/head:refs/remotes/origin/pr/*

# Gerrit review refs
+refs/for/*:refs/remotes/origin/changes/*

# Custom namespace fetch
+refs/heads/*:refs/remotes/origin/custom/*

# Exclude specific branch
refs/heads/*:refs/remotes/origin/* ^refs/heads/dontwant

# Force update
+refs/heads/*:refs/remotes/origin/*

# Delete remote branch
:refs/heads/topic

# Push to different name
refs/heads/local:refs/heads/remote-name
```

### Configuration Template

```ini
[remote "origin"]
    url = https://github.com/user/repo.git

    # Standard fetch
    fetch = +refs/heads/*:refs/remotes/origin/*

    # Optional: fetch PRs
    # fetch = +refs/pull/*/head:refs/remotes/origin/pr/*

    # Optional: custom namespace
    # fetch = +refs/heads/qa/*:refs/remotes/origin/qa/*

    # Optional: exclude experimental
    # fetch = ^refs/heads/experimental/*
```

---

## 15. Implementation Notes for Jin

### Potential Applications

1. **Custom Jin Namespace:** Could use `refs/jin/*` for internal metadata
2. **Mode/Scope Tracking:** Could store workspace state in custom refs
3. **History Preservation:** Could use custom refs for workspace history
4. **Synchronization:** Could implement bidirectional sync patterns

### Example Jin Use Cases

```bash
# Store current workspace state
git update-ref refs/jin/workspace/current <commit-hash>

# Track workspace history
git update-ref refs/jin/workspace/history/<timestamp> <commit-hash>

# Store applied layers
git update-ref refs/jin/layers/applied <commit-hash>

# Mirror with custom namespace
git fetch origin refs/heads/*:refs/jin/upstream/*
```

### Security Considerations for Jin

- Namespaces under `refs/jin/*` would not provide access control
- Would need repository-level permissions for protection
- Consider using refs/private/* for sensitive workspace data if supported

---

## Appendix: Git Internals

### How Refs Are Stored

Git stores refs as plain text files in `.git/refs/`:

```
.git/refs/
├── heads/
│   ├── master
│   └── develop
├── remotes/
│   └── origin/
│       ├── master
│       └── develop
├── tags/
│   ├── v1.0
│   └── v1.1
└── jin/          # Custom namespace
    ├── workspace
    └── metadata
```

Each file contains a single commit hash:
```
$ cat .git/refs/heads/master
a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6
```

### Ref Validation

Use `git check-ref-format` to validate ref names:

```bash
git check-ref-format refs/heads/my-branch  # Valid
git check-ref-format refs/heads/my branch  # Invalid (space)
```

Valid ref names:
- No spaces
- No control characters
- No `..` sequences
- No `@{` sequences (reflog syntax)
- No backslashes or leading dots

---

## Document Metadata

- **Created:** 2025-12-27
- **Research Scope:** Git refspec syntax, custom namespaces, real-world patterns
- **Sources:** Official Git documentation, ProGit book, Gerrit/GitHub/GitLab docs
- **Audience:** Development teams implementing custom Git workflows
- **Target Application:** Jin project custom namespace implementation

