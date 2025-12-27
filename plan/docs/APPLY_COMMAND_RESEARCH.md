# Apply Command Research

## Research Sources

This document synthesizes research from:
- Git's `git apply` command patterns
- Terraform's `terraform apply` workspace application patterns
- Kubectl's `kubectl apply` declarative configuration patterns
- Best practices for applying staged changes to workspace

## Key Patterns for Layered Configuration Systems

### 1. Preview Before Apply (Fail-Fast Pattern)

All successful systems implement preview capabilities:

**Git:**
- `--check` - Validate patch applicability without applying
- `--stat` - Display diffstat summary
- `--numstat` - Machine-friendly numeric stats

**Terraform:**
- `terraform plan -out=tfplan` - Generate and save plan
- `terraform show tfplan` - Review plan before applying

**Kubectl:**
- `kubectl diff -f config.yaml` - Show differences before applying
- `--dry-run=server` - Server-side validation
- `--dry-run=client` - Client-side validation (fast)

### 2. Atomic vs. Partial Application

| System | Default Behavior | Override Option |
|--------|------------------|-----------------|
| Git | Atomic (all-or-nothing) | `--reject` for partial |
| Terraform | Per-resource atomic | N/A |
| Kubectl | Per-object atomic | N/A |

**Key insight:** Git is the only system with truly atomic multi-change application by default.

### 3. Three-Way Merge Strategy (Kubectl Pattern)

Kubectl's apply uses three-way merge comparing:
1. Configuration file (desired state)
2. Live configuration (current state)
3. `last-applied-configuration` annotation (previous applied state)

**Merge calculation:**
```
Fields to DELETE:
  - In last-applied-configuration
  - Missing from configuration file

Fields to ADD/SET:
  - In configuration file
  - Different from live configuration
```

### 4. Recommended Apply Workflow

Based on all three systems:

```bash
# 1. VALIDATE - Check syntax and schema
validate_config_files()

# 2. MERGE - Combine layers into final configuration
merge_layers()

# 3. PREVIEW - Show what will change
preview_changes()

# 4. CHECK - Validate applicability
check_can_apply()

# 5. LOCK - Prevent concurrent modifications
acquire_lock()

# 6. BACKUP - Save current state
backup_current_state()

# 7. APPLY - Execute changes
apply_changes()

# 8. VERIFY - Confirm success
verify_application()

# 9. UNLOCK - Release lock
release_lock()

# 10. UPDATE METADATA - Track application
update_last_applied_metadata()
```

### 5. Error Handling Strategies

**Rollback approaches:**

**Git Apply:**
- Atomic failure - nothing applied, nothing to rollback
- With `--reject` - manual resolution required via `*.rej` files
- Reverse: `git apply --reverse patch.diff`

**Terraform Apply:**
- No automatic rollback - state may be partially modified
- Manual rollback via state restoration from `terraform.tfstate.backup`
- Re-apply old configuration to revert infrastructure

**Kubectl Apply:**
- No rollback - resources may be partially modified
- Recovery: re-apply previous configuration
- For Deployments: `kubectl rollout undo deployment/myapp`

### 6. Force and Dry-Run Flags

**Standard patterns:**

**Dry-run:**
- `--dry-run` (Git, Kubectl)
- `plan` command (Terraform)
- Show what would happen without executing

**Force:**
- `--force` - Override safety checks
- `--auto-approve` (Terraform) - Skip interactive confirmation
- Should require explicit user intent

### 7. Metadata Tracking (Inspired by Kubernetes)

Store metadata about each apply operation:
```yaml
metadata:
  lastAppliedConfiguration: "<full-config-hash>"
  lastAppliedTime: "2025-12-27T10:30:00Z"
  appliedBy: "user@example.com"
  appliedLayers:
    - layer: "base"
      source: "/config/base.yaml"
      hash: "abc123"
  fieldManagers:
    spec.replicas: "prod-override"
    spec.image: "base"
```

This enables:
- Three-way merges in future applies
- Field ownership tracking
- Audit trail
- Rollback to previous configurations

## Key Takeaways for Jin Apply

1. **Implement dry-run mode** - Preview changes before applying
2. **Show diff** - Display what files will change
3. **Atomic application** - All files succeed or none apply (with force override)
4. **Metadata tracking** - Store last-applied configuration for future merges
5. **Backup before apply** - Enable rollback on failure
6. **Locking** - Prevent concurrent workspace modifications
7. **Idempotency** - Applying same config multiple times is safe
8. **Validation gates** - Syntax, schema, and applicability checks
9. **Error reporting** - Clear messages about what failed and why
10. **Workspace dirty detection** - Warn if uncommitted changes exist

## Sources

- Git Apply Documentation: https://git-scm.com/docs/git-apply
- Terraform Apply Reference: https://developer.hashicorp.com/terraform/cli/commands/apply
- Kubectl Apply (Declarative Management): https://kubernetes.io/docs/tasks/manage-kubernetes-objects/declarative-config/
- Strategic Merge Patch: https://github.com/kubernetes/community/blob/master/contributors/devel/sig-api-machinery/strategic-merge-patch.md
