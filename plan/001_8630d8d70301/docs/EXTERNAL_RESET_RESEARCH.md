# External Reset/Undo Command Research

## Overview
This document researches external implementations of reset/undo commands across version control systems and configuration management tools, focusing on actionable implementation patterns for safe destructive operations.

## 1. Git's Reset Command Implementation

### Three Modes and Semantics

#### `--soft` Mode
- **Behavior**: Only moves HEAD pointer; preserves staging area and working directory
- **Use Case**: Rewriting commit messages, combining commits
- **Safety**: High - no data loss, all changes remain staged
- **Implementation**: Updates `HEAD` and reflog, leaves index and worktree unchanged

#### `--mixed` Mode (Default)
- **Behavior**: Resets HEAD and updates staging area; preserves working directory
- **Use Case**: Unstaging changes, reorganizing commits
- **Safety**: Medium - unstaged changes remain in working directory
- **Implementation**: Updates `HEAD`, resets index to match HEAD, leaves worktree unchanged

#### `--hard` Mode
- **Behavior**: Resets HEAD, staging area, AND working directory
- **Use Case**: Completely discarding uncommitted changes
- **Safety**: Low - **Destructive** - all uncommitted changes are lost
- **Implementation**: Updates `HEAD`, resets index AND worktree to match HEAD

### Key Implementation Patterns

```bash
# Soft reset - preserves staged changes
git reset --soft HEAD~1

# Mixed reset - unstages everything (default)
git reset HEAD~1

# Hard reset - destroys all uncommitted changes
git reset --hard HEAD~1
```

### Safety Mechanisms
- **Reflog**: All operations are logged for potential recovery
- **Pre-commit hooks**: Can warn about destructive operations
- **Branch protection**: Can block force pushes and history rewriting
- **Dry-run mode**: Preview changes before execution

**Sources**:
- [Git - git-reset Documentation](https://git-scm.com/docs/git-reset)
- [What's the difference between git reset --mixed, --soft, and](https://stackoverflow.com/questions/3528245/whats-the-difference-between-git-reset-mixed-soft-and-hard)

## 2. Mercurial (Hg) Implementation

### Core Commands

#### `hg rollback`
- **Behavior**: Undoes the last commit/transaction
- **Limitations**: Only one level of rollback, cannot undo a rollback
- **Safety**: Very low - permanent removal of commit history
- **Use Case**: Immediate undo of last commit

#### `hg backout`
- **Behavior**: Creates a new changeset that reverses changes from another changeset
- **Safety**: High - preserves history, creates new reverse changeset
- **Use Case**: Safe reversal of committed changes

#### `hg revert`
- **Behavior**: Undoes local changes in working directory
- **Safety**: Medium - only affects working directory, not history
- **Use Case**: Discarding local modifications

### Implementation Patterns

```bash
# Rollback last commit (use immediately after mistake)
hg rollback

# Safe reversal via new changeset
hg backout -r <revision>

# Revert local changes
hg revert <file>
```

### Safety Patterns
- **Immutable history**: True deletion isn't possible, only reversal
- **Immediate use**: Rollback should be used immediately after mistake
- **Backup strategy**: Use version control for rollback capability

**Sources**:
- [Finding and fixing mistakes](https://book.mercurial-scm.org/read/undo.html)
- [Help: rollback](https://www.mercurial-scm.org/repo/hg/help/rollback)

## 3. Subversion (SVN) Implementation

### Core Commands

#### `svn revert`
- **Behavior**: Reverts local changes to files/directories, resolves conflicts
- **Safety**: Medium - only affects working copy, not repository
- **Implementation**: Restores pristine copy from repository
- **Use Case**: Undoing local modifications

#### `svn merge -c -N`
- **Behavior**: Reverse merge to undo committed revisions
- **Safety**: Medium - creates new revision with reverse changes
- **Implementation**: Merges changes in reverse direction
- **Use Case**: Reverting committed changes safely

### Implementation Patterns

```bash
# Revert local changes
svn revert <path>

# Reverse commit revision
svn merge -c -<revision> .

# GUI approach using TortoiseSVN
# - Right-click → Show log → Select revision → Right-click → Merge...
```

### Safety Patterns
- **Working copy isolation**: Local reverts don't affect repository
- **Merge-based reversal**: Commits are reversed via new merge revisions
- **Plan-first approach**: Always preview merge results before applying

**Sources**:
- [svn revert](https://svnbook.red-bean.com/en/1.8/svn.ref.svn.c.revert.html)
- [Roll back (Undo) revisions in the repository](https://tortoisesvn.net/docs/release/TortoiseSVN_en/tsvn-howto-rollback.html)

## 4. Terraform Implementation Patterns

### Destructive Operations

#### `terraform destroy`
- **Behavior**: Removes all managed resources
- **Safety**: Very low - **Permanent destruction** of infrastructure
- **Critical Warning**: Not idempotent, purely destructive
- **Implementation**: Reads state file and deletes all tracked resources

#### `terraform apply`
- **Behavior**: Applies changes to infrastructure
- **Safety**: Medium when used with proper planning
- **Implementation**: Compares desired state with actual state, creates execution plan

### Safety Implementation Patterns

```bash
# Always plan first
terraform plan

# Targeted destroy (safer than full destroy)
terraform destroy -target=aws_instance.web

# State file backup before destructive operations
cp terraform.tfstate terraform.tfstate.backup

# Use version control for rollback
git commit -m "Before: terraform apply"
terraform apply
git commit -m "After: terraform apply"
```

### Safety Best Practices
1. **Never use `terraform destroy` in production** without strict controls
2. **Always use version control** for rollback capability
3. **Review all plans** carefully before execution
4. **Use targeted operations** (`-target` flag) when possible
5. **Implement state file backups** before destructive operations
6. **Explicitly declare dependencies** to prevent unintended destruction

### Recovery Limitations
- **Provisioner actions**: Cannot be automatically undone
- **External dependencies**: Terraform cannot know what external systems depend on resources
- **State corruption**: May prevent proper rollback

**Sources**:
- [How to Rollback to Previous State in terraform](https://stackoverflow.com/questions/57821319/how-to-rollback-to-previous-state-in-terraform)
- [Terraform Destroy Should Be Forbidden in All Environments](https://aws.plainenglish.io/terraform-destroy-should-be-forbidden-in-all-environments-98342fc776df)

## 5. Ansible Implementation Patterns

### Deployment and Rollback

#### Rolling Upgrade Pattern
- **Behavior**: Gradually replaces old instances with new ones
- **Safety**: High - maintains availability throughout deployment
- **Implementation**: Uses rolling upgrade strategy with health checks
- **Use Case**: Zero-downtime deployments

#### Backup and Restore Pattern
- **Behavior**: Creates backups before changes, restores on failure
- **Safety**: High - rollback via restore from backup
- **Implementation**: Tar backups, file copies, database dumps
- **Use Case**: Application rollback scenarios

### Implementation Patterns

```yaml
# Rolling upgrade playbook example
- name: Rolling upgrade
  hosts: webservers
  serial: 1  # One host at a time
  tasks:
    - name: Stop service
      systemd:
        name: myapp
        state: stopped

    - name: Backup application
      archive:
        path: /opt/myapp
        dest: /backups/myapp-{{ ansible_date_time.iso8601 }}.tar.gz

    - name: Deploy new version
      copy:
        src: ./new-version/
        dest: /opt/myapp/

    - name: Start service
      systemd:
        name: myapp
        state: started
```

### Safety Patterns
1. **Backup before deployment**: Create recoverable backups
2. **Rolling updates**: Deploy gradually with health checks
3. **Atomic deployments**: Use blue-green deployments
4. **Health checks**: Verify deployment success before continuing
5. **Audit trails**: Log all operations for forensics

**Sources**:
- [Continuous Delivery and Rolling Upgrades](https://docs.ansible.com/projects/ansible/latest/playbook_guide/guide_rolling_upgrade.html)
- [Rollback of Applications Using Ansible or Puppet](https://stackoverflow.com/questions/30886771/rollback-of-applications-using-ansible-or-puppet)

## 6. Common Safety Patterns Across Systems

### Prevention Mechanisms
1. **Branch Protection Rules**
   - Block direct commits to main
   - Require pull requests
   - Prevent force pushes
   - Admin restrictions on protected branches

2. **Approval Gates**
   - Require explicit approval for destructive operations
   - Multi-factor authentication for critical operations
   - Escalation procedures for high-risk operations

3. **Auto-Detection Systems**
   - Identify dangerous keywords (DROP, rm -rf, etc.)
   - Pattern matching for destructive commands
   - Risk level assessment with escalation triggers

### Detection and Monitoring
1. **Command Logging**
   - Log all destructive operations
   - Audit trails for compliance
   - Real-time monitoring of high-risk operations

2. **Pre-flight Checks**
   - Validate targets before execution
   - Check for dependent resources
   - Verify environment safety

### Recovery Mechanisms
1. **Reflog/History Utilization**
   - Git's reflog for recovery
   - Version control backups
   - Snapshots and checkpoints

2. **State Backups**
   - Regular state file backups
   - Offsite backup storage
   - Automated backup verification

### Error Handling Patterns
```python
# Example error handling pattern
def safe_reset_operation(operation, target, dry_run=True):
    try:
        # Pre-flight validation
        validate_targets(target)
        check_dependencies(target)

        # Dry run first
        if dry_run:
            preview_changes(operation, target)
            get_user_confirmation()

        # Execute with safeguards
        execute_with_safeguards(operation, target)

        # Verify success
        verify_operation_result()

    except DestructiveOperationError as e:
        # Handle destructive operation errors
        initiate_rollback()
        alert_administrators()
        log_incident(e)

    except ValidationError as e:
        # Handle validation errors
        raise OperationCancelled(f"Validation failed: {e}")
```

**Sources**:
- [Claude Code Safety Net](https://github.com/kenryu42/claude-code-safety-net)
- [Best Practices for Branch Protection](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/managing-a-branch-protection-rule)

## 7. Best Practices for Safe Destructive Operations

### 1. Always Use Dry-Run Mode
- Preview changes before execution
- Validate operation targets
- Get explicit user confirmation

### 2. Implement Multi-Step Verification
- Separate planning from execution
- Require approval for high-risk operations
- Use timeouts for cancellation opportunities

### 3. Maintain Comprehensive Backups
- Regular automated backups
- Offsite backup storage
- Backup verification procedures

### 4. Use Targeted Operations
- Avoid blanket destructive commands
- Specify exact targets
- Use include/exclude patterns

### 5. Implement Proper Logging
- Log all operations with timestamps
- Capture user intent and context
- Preserve audit trails for compliance

### 6. Provide Clear Recovery Paths
- Document rollback procedures
- Train teams on recovery operations
- Test recovery procedures regularly

### 7. Environment Isolation
- Use staging environments for testing
- Implement environment checks
- Prevent cross-environment operations

### 8. Rate Limiting and Throttling
- Limit destructive operations per time period
- Implement cooldown periods
- Require re-authentication for critical operations

## 8. Common Gotchas and Pitfalls

### 1. Git Reset --hard
- **Gotcha**: Irreversible loss of uncommitted changes
- **Mitigation**: Use `git stash` before hard reset
- **Recovery**: Check reflog with `git reflog`

### 2. Terraform Destroy
- **Gotcha**: Cannot rollback infrastructure once destroyed
- **Mitigation**: Use version control and state backups
- **Recovery**: From last known good state

### 3. SVN Merges
- **Gotcha**: Complex merge conflicts during reversal
- **Mitigation**: Test merges in staging first
- **Recovery**: Manual conflict resolution

### 4. Ansible Rolling Updates
- **Gotcha**: Partial deployment failures
- **Mitigation**: Use serial deployments and health checks
- **Recovery**: Manual intervention required

## 9. Implementation Recommendations

### For Version Control Systems
1. Implement multi-mode reset commands (soft, mixed, hard)
2. Provide clear safety warnings for destructive operations
3. Maintain comprehensive reflog/history
4. Implement branch protection rules

### For Configuration Management
1. Always use dry-run and preview modes
2. Implement backup and restore patterns
3. Use rolling updates for zero-downtime deployments
4. Maintain detailed audit logs

### For Safety Systems
1. Auto-detect destructive operations
2. Implement approval workflows
3. Provide clear error messages and recovery options
4. Regular testing of recovery procedures

## 10. Conclusion

The research shows that safe destructive operations require:
- **Clear understanding** of operation semantics
- **Multiple safety layers** (prevention, detection, recovery)
- **Comprehensive logging** for audit trails
- **User education** on safe usage patterns
- **Regular testing** of recovery procedures

The most successful implementations combine technical safeguards with proper processes and user training to minimize the risk of destructive operations while maintaining system recoverability.