# Test Patterns for route_to_layer() Function

## Overview

This document captures the exact test patterns used for testing `route_to_layer()` in `src/staging/router.rs`.

## Test Structure Components

### 1. Helper Functions for Context

```rust
// Helper for context with mode only
fn context_with_mode() -> ProjectContext {
    ProjectContext {
        mode: Some("claude".to_string()),
        ..Default::default()
    }
}

// Helper for context with both mode and scope
fn context_with_mode_and_scope() -> ProjectContext {
    ProjectContext {
        mode: Some("claude".to_string()),
        scope: Some("language:javascript".to_string()),
        ..Default::default()
    }
}
```

### 2. Successful Route Tests Pattern

**Structure**: Options → Context → Call → Assert Success

```rust
#[test]
fn test_route_default() {
    let options = RoutingOptions::default();  // All flags false
    let context = ProjectContext::default();
    let layer = route_to_layer(&options, &context).unwrap();
    assert_eq!(layer, Layer::ProjectBase);  // Exact layer match
}

#[test]
fn test_route_global() {
    let options = RoutingOptions {
        global: true,
        ..Default::default()
    };
    let context = ProjectContext::default();
    let layer = route_to_layer(&options, &context).unwrap();
    assert_eq!(layer, Layer::GlobalBase);
}

#[test]
fn test_route_mode() {
    let options = RoutingOptions {
        mode: true,
        ..Default::default()
    };
    let context = context_with_mode();
    let layer = route_to_layer(&options, &context).unwrap();
    assert_eq!(layer, Layer::ModeBase);
}
```

### 3. Error Handling Tests Pattern

**Structure**: Invalid Options → Context → Call → Assert Error

```rust
#[test]
fn test_route_mode_without_active_mode_fails() {
    let options = RoutingOptions {
        mode: true,
        ..Default::default()
    };
    let context = ProjectContext::default();  // No mode set
    let result = route_to_layer(&options, &context);
    assert!(result.is_err());  // Expect failure
}
```

### 4. Validation Tests Pattern

```rust
#[test]
fn test_validate_global_conflict() {
    let options = RoutingOptions {
        global: true,
        mode: true,
        ..Default::default()
    };
    let result = validate_routing_options(&options);
    assert!(result.is_err());
}
```

## Key Pattern Elements

### RoutingOptions Construction
- Use `..Default::default()` to set unspecified fields
- Boolean flags set explicitly (mode: true)
- Scope is `Option<String>`: use `Some("value".to_string())`

### ProjectContext Setup
- Use helper functions for common scenarios
- `context_with_mode()` for mode-based tests
- `context_with_mode_and_scope()` for combined tests
- `ProjectContext::default()` for tests without mode/scope

### Assertions
- Success: `assert_eq!(layer, Layer::{Variant})`
- Error: `assert!(result.is_err())`
- Use `.unwrap()` for expected success cases

## Test Coverage for --local Flag

Required tests for P1.M1.T2.S3:

```rust
// Test 1: Local flag routes to UserLocal
#[test]
fn test_route_local() {
    let options = RoutingOptions {
        local: true,
        ..Default::default()
    };
    let context = ProjectContext::default();
    let layer = route_to_layer(&options, &context).unwrap();
    assert_eq!(layer, Layer::UserLocal);
}

// Test 2: Local flag takes precedence over other flags (validation prevents this, but good to document)
// Note: This should never happen due to validation in validate_routing_options()
```

## Layer Types Tested

Current coverage:
- `Layer::ProjectBase` (default)
- `Layer::GlobalBase` (global flag)
- `Layer::ModeBase` (mode flag)
- `Layer::ModeProject` (mode + project)
- `Layer::ModeScope` (mode + scope)
- `Layer::ModeScopeProject` (mode + scope + project)
- `Layer::ScopeBase` (scope only)

Missing (to be added in P1.M1.T2.S3):
- `Layer::UserLocal` (local flag)
