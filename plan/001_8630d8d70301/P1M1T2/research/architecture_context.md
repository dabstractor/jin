# Architecture Context for P1M1T2

## Key Architectural Decisions

1. **Git-Style Conflict Resolution**: The system follows Git's familiar conflict resolution pattern with `.jinmerge` files containing layer-aware conflict markers.

2. **State Tracking Architecture**:
   - Paused operations tracked in `.jin/.paused_state` (YAML format)
   - Records operation type, affected files, conflict count, and timestamp
   - Integrates with existing transaction system for persistence

3. **Layer-Aware Conflict Markers**: Instead of "ours/theirs", markers use full layer ref paths (e.g., `refs/jin/layers/mode/claude/scope/language:javascript`) for clear context.

## Integration Context

This task fits into Phase 1 (Critical PRD Compliance) and Milestone 1.1 (.jinmerge Conflict Resolution). The apply command currently aborts on conflicts at `src/commands/apply.rs:63-76` and needs to be modified to:
- Write `.jinmerge` files instead of aborting
- Track paused operation state
- Allow resume via `jin resolve` command
- Update status command to show pending conflicts

## Design Patterns

1. **Command Integration Pattern**: Apply command returns `PausedOperation` error instead of failing, with state persisted to Git
2. **Transaction Consistency**: Paused operations treated as incomplete transactions with full recovery support
3. **Error Recovery**: Robust handling of lost/corrupted state with repair commands

## Dependencies and Constraints

- **Requires**: P1M1T1 (.jinmerge file format module) - COMPLETE
- **Compatible**: With existing merge engine, transaction system, and error handling patterns
- **Constrained**: Must maintain atomicity, not break existing workflows, and support all file types

The architecture leverages Jin's existing completion status, using established patterns while adding the missing conflict resolution capability.
