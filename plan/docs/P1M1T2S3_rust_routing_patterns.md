# Rust If-Else Routing Chain Patterns

## Overview

This document captures Rust best practices for if-else routing chains based on the codebase analysis.

## Current Implementation Analysis

### route_to_layer() Function Pattern

The existing `route_to_layer()` function in `src/staging/router.rs` demonstrates excellent routing patterns:

```rust
pub fn route_to_layer(options: &RoutingOptions, context: &ProjectContext) -> Result<Layer> {
    // Global flag takes precedence
    if options.global {
        return Ok(Layer::GlobalBase);
    }

    // Check mode flag
    if options.mode {
        // Require active mode
        context.require_mode()?;

        if let Some(ref _scope) = options.scope {
            // Mode + Scope
            if options.project {
                // Mode + Scope + Project
                Ok(Layer::ModeScopeProject)
            } else {
                // Mode + Scope
                Ok(Layer::ModeScope)
            }
        } else if options.project {
            // Mode + Project
            Ok(Layer::ModeProject)
        } else {
            // Mode only
            Ok(Layer::ModeBase)
        }
    } else if let Some(ref _scope) = options.scope {
        // Scope without mode (untethered scope)
        Ok(Layer::ScopeBase)
    } else {
        // Default: Project Base
        Ok(Layer::ProjectBase)
    }
}
```

## Best Practices Identified

### 1. Early Returns for Precedence

```rust
// High-precedence cases return immediately
if options.global {
    return Ok(Layer::GlobalBase);
}
```

**Benefits**:
- Reduces nesting
- Clear precedence order
- Easier to read and maintain

### 2. Clear Comment Groups

```rust
// Global flag takes precedence
if options.global { ... }

// Check mode flag
if options.mode { ... }

// Scope without mode (untethered scope)
} else if let Some(ref _scope) = options.scope { ... }

// Default: Project Base
} else { ... }
```

**Benefits**:
- Each routing path is explained
- Makes complex logic easier to understand
- Helps with maintenance

### 3. Proper Error Handling

```rust
context.require_mode()?;  // Propagate errors with ? operator
```

**Benefits**:
- Clean error propagation
- No unwrap() panic risks
- Idiomatic Rust error handling

### 4. if let for Option Handling

```rust
if let Some(ref _scope) = options.scope {
    // Use scope
} else {
    // No scope
}
```

**Benefits**:
- Safe Option unwrapping
- No panic risks
- Clear intent

### 5. Separation of Concerns

**Validation** is separate from **Routing**:
- `validate_routing_options()` - Validates flag combinations
- `route_to_layer()` - Determines target layer

**Benefits**:
- Each function has single responsibility
- Easier to test
- Easier to maintain

## Pattern for Adding --local Routing

Based on the existing patterns, the --local routing should:

### 1. Use Early Return (like --global)

```rust
// Global flag takes precedence
if options.global {
    return Ok(Layer::GlobalBase);
}

// Local flag takes precedence (after global)
if options.local {
    return Ok(Layer::UserLocal);
}
```

**Placement**: After --global check, before mode checks
**Reason**: Both --global and --local are "standalone" flags that target independent layers

### 2. No Context Required

```rust
// Local routing doesn't need mode/scope/project context
if options.local {
    return Ok(Layer::UserLocal);
}
```

**Benefits**:
- Layer 8 (UserLocal) is independent of mode/scope/project
- No validation needed (unlike mode which requires active mode)
- Simple and straightforward

### 3. Same Pattern as --global

Both flags:
- Target independent layers
- Don't require context
- Return early with Ok(Layer::{Variant})
- Are mutually exclusive with other flags (enforced by validation)

## Anti-Patterns to Avoid

### 1. Deep Nesting Without Early Returns

```rust
// BAD - Avoid this
if options.local {
    if some_condition {
        if another_condition {
            Ok(Layer::UserLocal)
        }
    }
}

// GOOD - Use early return
if options.local {
    return Ok(Layer::UserLocal);
}
```

### 2. Complex Boolean Expressions Without Comments

```rust
// BAD - What does this mean?
if options.local && options.mode || options.project { ... }

// GOOD - Clear intent with comments
// Can't combine --local with other flags (validation prevents this)
if options.local {
    return Ok(Layer::UserLocal);
}
```

### 3. Mixed Responsibilities

```rust
// BAD - Validation mixed with routing
if options.local && options.mode {
    return Err(JinError::Config("...".to_string()));
}

// GOOD - Validation is separate (in validate_routing_options)
// Routing logic assumes valid input
if options.local {
    return Ok(Layer::UserLocal);
}
```

## External Rust Best Practices (2025)

Based on current Rust community practices:

1. **Match expressions** for complex routing (when applicable)
2. **Let-else patterns** for early returns with pattern matching
3. **The `?` operator** for error propagation
4. **If-let chains** for combining multiple let bindings (RFC 2497)

For this implementation, **early returns** are the most appropriate pattern because:
- Simple boolean checks
- Clear precedence order
- No need for pattern matching
- Consistent with existing codebase style

## References

- [RFC 2497 - if-let-chains](https://rust-lang.github.io/rfcs/2497-if-let-chains.html)
- [Stop Writing Ugly If-Else Chains](https://medium.com/@bhesaniyavatsal/stop-writing-ugly-if-else-chains-this-one-rust-feature-will-change-how-you-code-forever-10e9f93e41c4)
- [Using match Ergonomically](https://dev.to/sgchris/using-match-ergonomics-avoid-the-if-else-chains-19dm)
