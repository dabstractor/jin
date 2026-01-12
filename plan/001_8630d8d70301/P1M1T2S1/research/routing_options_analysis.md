# RoutingOptions Struct Analysis

## Overview

The `RoutingOptions` struct in `src/staging/router.rs` is the core data structure for layer routing in the Jin CLI. It defines which layer a file should be routed to based on CLI flags.

## Current Structure (Before P1.M1.T2.S1)

```rust
/// Options for routing a file to a layer
#[derive(Debug, Default)]
pub struct RoutingOptions {
    /// Target mode layer
    pub mode: bool,
    /// Target scope
    pub scope: Option<String>,
    /// Target project layer
    pub project: bool,
    /// Target global layer
    pub global: bool,
}
```

## Field Semantics

| Field  | Type               | Meaning                                  | Requires                  |
|--------|--------------------|------------------------------------------|---------------------------|
| mode   | bool               | Route to a mode layer (2, 3, 4, 5)      | Active mode in context    |
| scope  | Option\<String\>   | Target scope identifier                  | -                         |
| project| bool               | Route to project layer (4, 5, 7)        | mode = true (for 4, 5)    |
| global | bool               | Route to global layer (1)               | - (takes precedence)      |

## Consumers of RoutingOptions

1. **src/commands/add.rs** (lines 50-55)
   - Builds RoutingOptions from AddArgs
   - Calls `validate_routing_options()` then `route_to_layer()`

2. **src/commands/mv.rs** (lines 54-59)
   - Builds RoutingOptions from MvArgs
   - Same pattern as add.rs

3. **src/commands/rm.rs** (lines 49-54)
   - Builds RoutingOptions from RmArgs
   - Same pattern as add.rs

4. **src/commands/import_cmd.rs**
   - Builds RoutingOptions from ImportArgs
   - Same pattern as add.rs

## Derive Macros

```rust
#[derive(Debug, Default)]
```

- **Debug**: Enables `{:?}` formatting for debugging
- **Default**: Provides default values (all bools = false, scope = None)

## Pattern for Adding New Field

Following existing patterns (mirroring AddArgs from P1.M1.T1.S1):

```rust
/// Options for routing a file to a layer
#[derive(Debug, Default)]
pub struct RoutingOptions {
    /// Target mode layer
    pub mode: bool,
    /// Target scope
    pub scope: Option<String>,
    /// Target project layer
    pub project: bool,
    /// Target global layer
    pub global: bool,
    /// Target user-local layer (Layer 8)
    pub local: bool,
}
```

## Key Insights

1. **Field Order**: New `local` field should go after `global` for consistency with AddArgs
2. **No clap attributes**: RoutingOptions is NOT a clap struct - it's a plain data struct
3. **Public fields**: All fields are `pub` for direct struct initialization
4. **Default trait**: bool fields default to `false`, Option\<String\> defaults to `None`
5. **No validation in struct**: Validation happens in `validate_routing_options()` function
