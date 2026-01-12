# PRP: Verify Deep Merge Handles Layer Precedence Correctly

## Goal

**Feature Goal**: Verify that the deep merge implementation correctly implements layer precedence (higher layers override lower layers) without requiring code changes.

**Deliverable**: Verification confirmation via code comment and/or test that deep_merge() correctly implements RFC 7396 semantics with proper layer precedence.

**Success Definition**:
- Confirm that `merge_file_across_layers()` passes layers in correct order (highest precedence last) to `deep_merge()`
- Verify that the `(_, overlay) => Ok(overlay)` match arm in `deep_merge()` correctly implements layer precedence
- Document the verification finding with a comment or test

## User Persona

**Target User**: Development team and PRD compliance verification

**Use Case**: Verify correctness of layer precedence in deep merge as part of bug fix P1.M1.T1

**User Journey**:
1. Review the architecture analysis confirming deep_merge() is already correct
2. Trace through the code to verify layer ordering from merge_file_across_layers() to deep_merge()
3. Add verification comment or test to document the finding

**Pain Points Addressed**: Ensures the bug fix for structured file auto-merge is built on correct layer precedence semantics

## Why

- **PRD Compliance**: Bug fix P1.M1.T1 (Remove pre-merge conflict check for structured files) depends on correct layer precedence
- **Risk Mitigation**: Verifying the merge engine correctness before making changes prevents introducing new bugs
- **Documentation**: Adding explicit verification makes layer precedence behavior clear to future developers
- **Architecture Confidence**: The merge_engine_analysis.md states deep_merge() is correct - this task confirms it

## What

Verify the layer precedence implementation without code changes. The verification should confirm:

1. **Layer Ordering**: Layers are passed to `deep_merge()` in correct order (lowest precedence first, highest last)
2. **Match Arm Correctness**: The `(_, overlay) => Ok(overlay)` match arm correctly implements "last one wins"
3. **RFC 7396 Semantics**: Overlay replacing base follows JSON Merge Patch standard

### Success Criteria

- [ ] Trace layer ordering from `get_applicable_layers()` through `merge_file_across_layers()` to `deep_merge()`
- [ ] Verify the accumulative merge pattern uses correct base/overlay ordering
- [ ] Add verification comment or test documenting layer precedence correctness
- [ ] Confirm no code changes are needed to deep_merge() implementation

## All Needed Context

### Context Completeness Check

**Validation**: "If someone knew nothing about this codebase, would they have everything needed to implement this successfully?"

Yes - this PRP provides:
- Exact file paths and line numbers for key functions
- Layer enum definition and precedence values
- Architecture analysis confirming correctness
- Existing test patterns to follow
- Expected behavior examples

### Documentation & References

```yaml
# MUST READ - Architecture documentation confirming deep_merge correctness
- url: plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/architecture/merge_engine_analysis.md
  why: Confirms deep_merge() implementation is already correct and follows RFC 7396
  critical: "The deep merge implementation is **correct** and follows RFC 7396 semantics" - no changes needed

# MUST READ - Deep merge implementation
- file: src/merge/deep.rs
  why: Contains the `(_, overlay) => Ok(overlay)` match arm that implements layer precedence
  pattern: Line 115: `(_, overlay) => Ok(overlay)` - the foundation of layer precedence
  gotcha: This is a catch-all match arm, so it must come AFTER all specific match arms (lines 80-112)

# MUST READ - Layer merge orchestration
- file: src/merge/layer.rs
  why: Contains `merge_file_across_layers()` which calls deep_merge() with accumulative pattern
  pattern: Lines 369-376: Accumulative merge loop
  gotcha: Layers are iterated in order provided (lowest to highest precedence)

# MUST READ - Layer enum definition
- file: src/core/layer.rs
  why: Defines the 9-layer hierarchy with precedence values
  pattern: Lines 34-46: `precedence()` method returns 1-9
  gotcha: WorkspaceActive has highest precedence (9), GlobalBase has lowest (1)

# MUST READ - Existing layer precedence test
- file: tests/mode_scope_workflow.rs
  why: Contains `test_layer_precedence_higher_wins()` and `test_mode_scope_deep_merge()`
  pattern: Lines 267-420: Tests that higher layers override lower layers
  section: Test functions for layer precedence behavior

# EXTERNAL REFERENCE - RFC 7396
- url: https://datatracker.ietf.org/doc/html/rfc7396
  why: JSON Merge Patch standard that deep_merge() implements
  critical: Specifies that null values delete keys, overlay values override base values
```

### Current Codebase Tree

```bash
src/
├── merge/
│   ├── deep.rs           # deep_merge() function with layer precedence match arm
│   ├── layer.rs          # merge_file_across_layers() with accumulative merge
│   └── mod.rs            # Merge module exports
├── core/
│   └── layer.rs          # Layer enum with 9 variants and precedence() method
└── git/
    └── repo.rs           # JinRepo for layer ref resolution

tests/
├── mode_scope_workflow.rs  # Existing layer precedence tests
└── common/
    └── assertions.rs       # Test assertion helpers

plan/001_8630d8d70301/bugfix/001_d2716c9eb3cf/
└── architecture/
    └── merge_engine_analysis.md  # Architecture analysis confirming correctness
```

### Desired Codebase Tree (No changes expected)

```bash
# No file additions expected - this is a verification task
# Possible addition of verification comment or test
```

### Known Gotchas of Jin Codebase & Library Quirks

```rust
// CRITICAL: The (_, overlay) => Ok(overlay) match arm MUST be last
// It's a catch-all pattern that will match anything not caught by earlier arms
// Position: src/merge/deep.rs:115

// CRITICAL: Layer precedence is determined by ORDER in the vector, NOT by precedence() value
// Layers are collected in lowest-to-highest order, then passed in that order
// Function: get_applicable_layers() in src/merge/layer.rs:477-504

// CRITICAL: Accumulative merge pattern means overlay becomes base for next iteration
// First layer: base = layer[0]
// Second layer: base = deep_merge(layer[0], layer[1])
// Third layer: base = deep_merge(result, layer[2])
// Function: merge_file_across_layers() in src/merge/layer.rs:369-376

// GOTCHA: Git ref paths use /_ suffix for parent refs to avoid conflicts
// Example: ModeBase uses "refs/jin/layers/mode/claude/_" not "refs/jin/layers/mode/claude"
// Reason: Git refs are files, can't have both file and directory at same path
```

## Implementation Blueprint

### Data Models and Structures

No new models needed - verification only task.

```rust
// Existing Layer enum with precedence values (src/core/layer.rs)
pub enum Layer {
    GlobalBase,        // precedence: 1 (lowest)
    ModeBase,          // precedence: 2
    ModeScope,         // precedence: 3
    ModeScopeProject,  // precedence: 4
    ModeProject,       // precedence: 5
    ScopeBase,         // precedence: 6
    ProjectBase,       // precedence: 7
    UserLocal,         // precedence: 8
    WorkspaceActive,   // precedence: 9 (highest)
}
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: VERIFY layer ordering in get_applicable_layers()
  - CONFIRM: Function returns layers in precedence order (lowest first)
  - READ: src/merge/layer.rs:477-504
  - TRACE: Layer collection from GlobalBase through WorkspaceActive
  - VERIFY: Each layer is added in increasing precedence order
  - OUTPUT: Confirmation that layers are ordered correctly

Task 2: VERIFY merge_file_across_layers() passes layers correctly to deep_merge()
  - CONFIRM: Accumulative merge pattern uses correct base/overlay ordering
  - READ: src/merge/layer.rs:369-376 (structured file deep merge loop)
  - TRACE: First layer becomes initial base, subsequent layers become overlay
  - VERIFY: Each call to deep_merge() uses (accumulated, current_layer) ordering
  - OUTPUT: Confirmation that overlay (higher precedence) wins

Task 3: VERIFY deep_merge() match arm implements layer precedence
  - CONFIRM: The (_, overlay) => Ok(overlay) match arm correctly implements precedence
  - READ: src/merge/deep.rs:114-115
  - VERIFY: This catch-all arm is LAST in match expression (after specific arms)
  - VERIFY: Returns overlay value, implementing "higher layer wins"
  - OUTPUT: Confirmation that match arm is correct

Task 4: ADD verification comment or test
  - CREATE: Comment in src/merge/deep.rs or test in tests/mode_scope_workflow.rs
  - FOLLOW pattern: Existing test patterns from mode_scope_workflow.rs
  - CONTENT: Document that layer precedence is correctly implemented
  - EXAMPLE: "// VERIFIED: Layer precedence implemented correctly - overlay wins"
  - PLACEMENT: Near deep_merge() function or in test suite
```

### Implementation Patterns & Key Details

```rust
// Pattern: Layer collection in precedence order
// Location: src/merge/layer.rs:477-504
pub fn get_applicable_layers(...) -> Vec<Layer> {
    let mut layers = vec![Layer::GlobalBase];  // Layer 1 (lowest)
    if let Some(_mode) = mode {
        layers.push(Layer::ModeBase);          // Layer 2
        // ... more layers in increasing order
    }
    layers.push(Layer::UserLocal);    // Layer 8
    layers.push(Layer::WorkspaceActive);  // Layer 9 (highest)
    layers
}

// Pattern: Accumulative merge with correct precedence
// Location: src/merge/layer.rs:369-376
let mut accumulated: Option<MergeValue> = None;
for (_layer, content_str) in text_contents {
    let layer_value = parse_content(&content_str, format)?;
    accumulated = Some(match accumulated {
        Some(base) => deep_merge(base, layer_value)?,  // base=accumulated, layer_value=current
        None => layer_value,
    });
}

// Pattern: Layer precedence match arm
// Location: src/merge/deep.rs:114-115
// CRITICAL: This is the foundation of layer precedence
// Different types or scalars: overlay wins
(_, overlay) => Ok(overlay),

// Example trace for scenario: Layer 2 {"a": 1} + Layer 7 {"a": 2, "b": 2}
// Step 1: accumulated = Some({"a": 1})  // Layer 2 content
// Step 2: deep_merge({"a": 1}, {"a": 2, "b": 2})
//         -> In merge: base["a"] = 1, overlay["a"] = 2
//         -> Recursively merge keys: deep_merge(1, 2) = Ok(2) via (_, overlay)
//         -> Add new keys: {"b": 2}
//         -> Result: {"a": 2, "b": 2}
// VERIFIED: Layer 7 (higher precedence) wins for key "a", keeps key "b"
```

### Integration Points

```yaml
NO_INTEGRATION_CHANGES:
  - This is a verification task only
  - No code changes needed to merge engine
  - No new dependencies
  - No configuration changes

VERIFICATION_OUTPUT:
  - Comment or test confirming layer precedence
  - Documentation of verification finding
```

## Validation Loop

### Level 1: Syntax & Style (Immediate Feedback)

```bash
# No code changes expected - skip validation
# If adding comment/test:
cargo fmt --check
cargo clippy -- -D warnings
```

### Level 2: Unit Tests (Component Validation)

```bash
# Run existing layer precedence tests
cargo test test_layer_precedence_higher_wins
cargo test test_mode_scope_deep_merge

# Run all merge tests
cargo test --test mode_scope_workflow

# Expected: All tests pass (deep_merge is already correct)
```

### Level 3: Integration Testing (System Validation)

```bash
# Run full merge test suite
cargo test --package jin --lib merge

# Run integration tests
cargo test --test '*'

# Expected: All tests pass
```

### Level 4: Verification Output Validation

```bash
# Verify the verification finding is documented
grep -r "layer precedence" src/merge/deep.rs tests/

# OR verify new test exists
cargo test verify_layer_precedence

# Expected: Comment or test exists documenting layer precedence
```

## Final Validation Checklist

### Technical Validation

- [ ] All 4 verification tasks completed successfully
- [ ] Confirmed layers are ordered lowest-to-highest in get_applicable_layers()
- [ ] Confirmed merge_file_across_layers() uses correct base/overlay ordering
- [ ] Confirmed deep_merge() (_, overlay) => Ok(overlay) implements precedence
- [ ] Verification comment or test added

### Feature Validation

- [ ] Layer 2 + Layer 7 scenario verified: {"a": 1} + {"a": 2, "b": 2} = {"a": 2, "b": 2}
- [ ] Architecture analysis finding confirmed: deep_merge() is correct
- [ ] RFC 7396 semantics verified
- [ ] No code changes needed to deep_merge()

### Code Quality Validation

- [ ] Verification finding documented clearly
- [ ] Existing test patterns followed
- [ ] No breaking changes introduced

### Documentation & Deployment

- [ ] Verification comment includes rationale
- [ ] Test (if added) follows naming conventions
- [ ] PRP completion confirmed

---

## Anti-Patterns to Avoid

- **Don't modify deep_merge()**: Implementation is already correct per architecture analysis
- **Don't skip verification**: Even though implementation is correct, verify the claim
- **Don't add unnecessary code**: This is verification-only, minimal changes
- **Don't confuse precedence values with order**: Precedence is 1-9, but order matters in iteration
- **Don't miss the accumulative pattern**: The key is that overlay becomes base for next merge

## Verification Example

The following scenario demonstrates layer precedence:

```rust
// Scenario: Layer 2 (ModeBase) + Layer 7 (ProjectBase)
// Layer 2 content: {"a": 1}
// Layer 7 content: {"a": 2, "b": 2}

// Expected result: {"a": 2, "b": 2} (Layer 7 wins for key "a")

// Verification trace:
// 1. get_applicable_layers() returns [GlobalBase, ModeBase, ..., ProjectBase, ...]
// 2. merge_file_across_layers() processes layers in order
// 3. accumulated = deep_merge({"a": 1}, {"a": 2, "b": 2})
// 4. In deep_merge:
//    - Both are objects, so merge recursively
//    - For key "a": deep_merge(1, 2) -> (_, overlay) => Ok(2)
//    - For key "b": new key from overlay -> {"b": 2}
//    - Result: {"a": 2, "b": 2}
// 5. VERIFIED: Layer 7 (higher precedence) wins
```

## Success Metrics

**Confidence Score**: 10/10 for one-pass verification success

**Rationale**:
- Architecture analysis explicitly states deep_merge() is correct
- Code trace confirms proper layer ordering
- Match arm implementation is straightforward and correct
- No code changes needed, only verification/documentation

**Expected Output**: Verification comment or test confirming layer precedence is correctly implemented per RFC 7396 semantics.
