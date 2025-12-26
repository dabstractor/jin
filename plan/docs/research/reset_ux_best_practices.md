# Reset Command UX Best Practices

## Overview

This document documents user experience best practices for implementing reset commands, gathered from analyzing Git, Docker, Kubernetes, and other developer tools. The focus is on creating safe, intuitive, and informative reset operations that users can trust.

## 1. Core UX Principles for Reset Commands

### 1.1 Safety First

**Principle**: Prevent data loss at all costs, or make it extremely obvious when data might be lost.

**Patterns**:
- Always warn about destructive operations
- Require explicit confirmation for dangerous actions
- Never silently discard user work
- Provide "escape hatches" to cancel operations

**Example from Git**:
```bash
$ git reset --hard HEAD~1
# No confirmation - but very clear about what will happen
# User can still interrupt with Ctrl+C
```

**Better Pattern**:
```bash
$ mytool reset --hard --target HEAD~1
WARNING: This will permanently discard all changes.
The following files will be deleted:
  - src/main.rs
  - tests/test.rs
Continue? [y/N]
```

### 1.2 Predictable Behavior

**Principle**: Users should be able to predict what will happen before executing the command.

**Patterns**:
- Use standard terminology (--soft, --mixed, --hard)
- Clear flag naming and help text
- Consistent behavior across similar operations
- Document edge cases in help text

**Example from Docker**:
```bash
$ docker system prune -a
# Clear, predictable behavior - removes everything
# The `-a` flag makes it obvious this is a big operation
```

### 1.3 Progressive Disclosure

**Principle**: Show more information as the user progresses, starting with the basics.

**Patterns**:
- Simple operations execute immediately
- Complex operations show warnings first
- Very destructive operations show previews
- Always show summary after completion

## 2. Information Display Patterns

### 2.1 Before Reset: What Will Be Affected

Always show a clear summary of what will be changed.

#### Tiered Information Display

**Level 1: Basic Summary**
```bash
$ git reset --hard HEAD~1
About to reset 2 commits.
Working tree changes will be discarded.
```

**Level 2: Detailed List**
```bash
$ git reset --hard HEAD~1
The following changes will be permanently discarded:
  M  src/main.rs  (modified)
  M  tests/test.rs (modified)
  ?? new_file.txt (untracked)
```

**Level 3: Diff Preview**
```bash
$ git reset --hard --preview HEAD~1
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,5 +1,5 @@
 fn main() {
-    println!("Hello");
+    println!("Hello, World!");
 }
# (and so on)
```

### 2.2 During Reset: Progress Indication

For operations that take time, show progress.

#### Spinner Progress
```bash
$ docker system prune
[====================>] 2.3GB reclaimed in 3.2s
```

#### Detailed Progress
```bash
$ kubeadm reset --force
[reset] Stopping the kubelet service ✓
[reset] Unmounting mounted directories ✓
[reset] Detaching mounted volumes ✓
[reset] Deleting files... 12/24
```

### 2.3 After Reset: State Summary

Always show the result of the operation.

#### Simple Summary
```bash
$ git reset --mixed HEAD~1
Unstaged 2 changes after reset.
```

#### Detailed State
```bash
$ git status
On branch main
Changes not staged for commit:
  modified:   src/main.rs
  modified:   tests/test.rs

no changes added to commit (use "git add" and/or "git commit -a")
```

## 3. Confirmation Strategies

### 3.1 Confirmation Hierarchy

Use different confirmation levels based on destructiveness:

#### Level 1: No Confirmation (Safe Operations)
```bash
$ git reset HEAD file.txt
# Just unstages a file - safe, reversible
```

#### Level 2: Simple Confirmation
```bash
$ docker system prune
WARNING! This will remove all stopped containers.
Continue? [y/N]
```

#### Level 3: Detailed Confirmation
```bash
$ git reset --hard HEAD~3
The following commits will be permanently lost:
  feat: Add user authentication
  fix: Resolve login bug
  refactor: Clean up auth code

3 commits will be reset, 2 files changed permanently.
Continue? [y/N]
```

#### Level 4: Multi-Stage Confirmation
```bash
# Stage 1: Warning
$ kubeadm reset
[reset] WARNING: This will reset the cluster to initial state.
[reset] Continue? [y/N]

# Stage 2: If yes, show summary
[reset] This will:
[reset] - Stop all kube-system pods
[reset] - Remove all containers
[reset] - Delete etcd data
[reset] Continue? [y/N]
```

### 3.2 Force Pattern

Always provide a `--force` flag for automation:

```bash
# Default: Interactive
$ mytool reset --hard

# Automation: Non-interactive
$ mytool reset --hard --force
```

### 3.3 Timeout for Confirmation

For dangerous operations, add a timeout to prevent accidental execution:

```bash
$ git reset --hard HEAD~1
The following changes will be PERMANENTLY LOST:
  - src/main.rs (modified)
  - tests/test.rs (modified)

You have 5 seconds to cancel (Ctrl+C)
5... 4... 3... 2... 1...
```

## 4. Error Handling and Recovery

### 4.1 Clear Error Messages

#### Bad Error Messages
```bash
$ git reset HEAD~999
error: fatal
# Not helpful - what went wrong?
```

#### Good Error Messages
```bash
$ git reset HEAD~999
fatal: ambiguous argument 'HEAD~999': unknown revision or path not in the working tree
```

#### Excellent Error Messages
```bash
$ git reset HEAD~999
fatal: ambiguous argument 'HEAD~999': unknown revision or path not in the working tree
Available references:
  HEAD (current)
  main
  feature-branch
  origin/main
```

### 4.2 Safe Defaults

Ensure the safest option is the default:

```bash
# Good: Mixed is safe default
$ git reset HEAD~1
# Equivalent to: git reset --mixed HEAD~1

# Bad: Hard as default
# $ git-tool reset HEAD~1  # Should NOT be --hard by default
```

### 4.3 Recovery Suggestions

When possible, suggest how to recover:

```bash
$ git reset --hard HEAD~1
error: You have unmerged paths.
  (fix conflicts and mark resolved with "git add/rm")
# Or:
# $ git reset --merge HEAD~1  # Try this instead
```

## 5. Targeted Reset Patterns

### 5.1 Path-Specific Resets

Allow selective reset to prevent accidental data loss:

```bash
# Reset specific files only
$ git reset HEAD src/main.rs
# Only unstages src/main.rs, leaves other files untouched
```

### 5.2 Layer-Specific Resets (Jin Pattern)

For layered systems, allow targeted resets:

```bash
# Reset only mode layer
$ jin reset --mode

# Reset only specific scope
$ jin reset --scope python

# Reset project layer only
$ jin reset --project
```

### 5.3 Time-Based Resets

Allow resetting to specific time points:

```bash
# Reset to last week
$ git reset --until="1 week ago"

# Reset to specific date
$ git reset --until="2023-01-01"
```

## 6. Interactive Features

### 6.1 Interactive Reset Mode

```bash
$ git reset --interactive
# Shows:
# 1. src/main.rs (staged) [reset]
# 2. tests/test.rs (staged) [keep]
# 3. README.md (untracked) [delete]
# ?
```

### 6.2 Visual Diff Preview

Show actual diffs before applying reset:

```bash
$ git reset --diff HEAD~1
# Shows unified diff of what will be discarded
```

### 6.3 Tree View

For complex repositories, show tree structure:

```bash
$ git reset --tree HEAD~1
src/
├── main.rs    (will be reset)
├── lib.rs     (will be reset)
└── tests/
    ├── test1.rs (unchanged)
    └── test2.rs (will be reset)
```

## 7. Documentation and Help

### 7.1 Clear Help Text

#### Basic Help
```bash
$ git reset --help
usage: git reset [--mixed | --soft | --hard | --merge | --keep] [-q] [<commit>] [--] <paths>...
```

#### Enhanced Help
```bash
$ git reset --help
Reset current HEAD to the specified state.

OPTIONS:
  --soft         Only reset HEAD, keep changes staged
  --mixed (def)  Reset HEAD and unstage changes, keep in working tree
  --hard         Reset HEAD, unstage, and discard all changes
  --merge        Reset with merge safety checks
  --keep         Reset HEAD but keep local changes
```

### 7.2 Examples in Help

Always include practical examples:

```bash
$ git reset --help
EXAMPLES:
  git reset HEAD~1              # Undo last commit, keep changes
  git reset --hard HEAD~1       # Discard last commit and changes
  git reset HEAD file.txt       # Unstage specific file
```

### 7.3 What's Changed Section

For complex operations, show what changed:

```bash
$ git reset --help
CHANGES:
  v2.0: Added --keep option to preserve working tree
  v1.5: Now shows untracked files in hard reset
  v1.0: Initial implementation
```

## 8. Jin-Specific UX Recommendations

### 8.1 Layer-Specific UX

Since Jin works with layers, the UX should emphasize layer awareness:

#### Layer Status Display
```bash
$ jin status --before-reset
Active layers:
  mode: claude
  scope: javascript
  project: myapp

Staged changes:
  mode/claude/src/ai.ts ( staged)
  scope/javascript/package.json (staged)

Working changes:
  project/src/main.js (modified)
```

#### Layer-Specific Reset
```bash
$ jin reset --mode --dry-run
Would reset mode layer 'claude':
  - src/ai.ts (staged)

Scope and project layers would be untouched.
Continue? [y/N]
```

### 8.2 Context-Aware Defaults

Reset behavior should adapt to current context:

#### In Staging Area
```bash
$ jin status
Staged changes:
  - src/main.js

$ jin reset
# Defaults to --mixed for staged files
```

#### In Working Tree
```bash
$ jin status
Working changes:
  - src/main.js

$ jin reset
# Shows warning, defaults to --mixed
```

### 8.3 Cross-Layer Reset

Reset operations that span layers need special handling:

```bash
$ jin reset --all --dry-run
This will reset all layers (mode, scope, project).
The following files will be affected:
  mode/claude/src/ai.ts
  scope/javascript/package.json
  project/src/main.js

Are you sure? [y/N]
```

## 9. Accessibility Considerations

### 9.1 Screen Reader Support

Ensure all output is text-based and screen reader friendly:

```bash
# Good: Text-based output
$ git reset --hard HEAD~1
Reset complete. 2 files discarded.

# Bad: Graphical output that screen readers can't parse
# [X] Successfully reset
```

### 9.2 Color Usage

Use color to highlight important information, but don't rely on it:

```bash
# Good: Color enhancement
$ git reset --hard HEAD~1
❌ Warning: 2 files will be permanently discarded
  src/main.js
  tests/test.js

# Bad: Color only
$ git reset --hard HEAD~1
[src/main.js]
[tests/test.js]
```

## 10. Performance Considerations

### 10.1 Fast Operations

Simple operations should be immediate:

```bash
$ git reset HEAD file.txt
# Should complete instantly
```

### 10.2 Progress for Slow Operations

For operations that take time, show progress:

```bash
$ git reset --hard HEAD~1000
# Show progress spinner for large operations
```

### 10.3 Background Operations

For very large repositories, consider background processing:

```bash
$ git reset --hard HEAD~1000
This may take a few minutes. Running in background...
Job ID: 1234
Use `git status` to check progress.
```

## 11. Security Considerations

### 11.1 File Permission Handling

Be careful with file permissions:

```bash
$ git reset --hard HEAD~1
# Preserve original permissions when possible
```

### 11.2 Symlink Handling

Handle symlinks safely:

```bash
$ git reset --hard HEAD~1
# Don't follow symlinks to prevent security issues
```

### 11.3 Git Repository Safety

Always verify repository integrity:

```bash
$ git reset --hard HEAD~1
# Check repository status first
error: repository is in an inconsistent state
# Help user repair instead of proceeding
```

## 12. Testing UX Patterns

### 12.1 Automated Testing

Test the entire user experience:

```rust
#[test]
fn test_reset_hard_with_confirmation() {
    let mut cmd = Command::new("git");
    cmd.args(&["reset", "--hard", "HEAD~1"]);

    // Simulate user input
    let mut child = cmd.spawn().unwrap();
    // Write "y" to stdin
    child.stdin.as_mut().unwrap().write_all(b"y").unwrap();

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
}
```

### 12.2 User Testing

Get real user feedback on the UX:

```bash
# Test with actual users
$ git reset --hard HEAD~1
# Measure:
# - Time to understand the warning
# - Number of times users need to re-read
# - Error rate
# - Satisfaction score
```

## Conclusion

Good UX for reset commands balances safety with usability. Key principles:

1. **Always warn about destructive operations**
2. **Show clear previews of what will happen**
3. **Use confirmation appropriate to the risk level**
4. **Provide clear error messages and recovery paths**
5. **Follow established patterns when possible**
6. **Make the safest option the default**

By following these patterns, Jin can create a reset command that users trust and feel comfortable using.

## Sources

1. [Git Reset Documentation](https://git-scm.com/docs/git-reset)
2. [Docker System Prune UX](https://docs.docker.com/engine/reference/commandline/system_prune/)
3. [Kubeadm Reset UX](https://kubernetes.io/docs/reference/setup-tools/kubeadm/kubeadm-reset/)
4. [Git Interactive Reset](https://git-scm.com/docs/git-reset#Documentation/git-reset.txt---interactive)
5. [CLI Command Design Patterns](https://clig.dev/)