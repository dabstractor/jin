# Test Structure Analysis

## Complete List of Test Files

**Integration Tests (in `/tests/` directory):**
- `/home/dustin/projects/jin/tests/atomic_operations.rs` - Tests atomic operations and error recovery
- `/home/dustin/projects/jin/tests/cli_basic.rs` - Basic CLI command tests
- `/home/dustin/projects/jin/tests/cli_diff.rs` - CLI diff command tests
- `/home/dustin/projects/jin/tests/cli_import.rs` - CLI import command tests
- `/home/dustin/projects/jin/tests/cli_list.rs` - CLI list command tests
- `/home/dustin/projects/jin/tests/cli_mv.rs` - CLI move command tests
- `/home/dustin/projects/jin/tests/cli_reset.rs` - CLI reset command tests
- `/home/dustin/projects/jin/tests/core_workflow.rs` - Core workflow integration tests
- `/home/dustin/projects/jin/tests/error_scenarios.rs` - Error handling and recovery tests
- `/home/dustin/projects/jin/tests/mode_scope_workflow.rs` - **Mode and scope specific tests**
- `/home/dustin/projects/jin/tests/sync_workflow.rs` - Remote synchronization tests

**Shared Test Modules:**
- `/home/dustin/projects/jin/tests/common/mod.rs` - Common test utilities module
- `/home/dustin/projects/jin/tests/common/assertions.rs` - Custom assertions for Jin state verification
- `/home/dustin/projects/jin/tests/common/fixtures.rs` - Test fixtures and setup helpers

## Test Directory Structure

```
/home/dustin/projects/jin/tests/
├── atomic_operations.rs      # Atomic operation tests
├── cli_basic.rs            # Basic CLI tests
├── cli_diff.rs             # Diff command tests
├── cli_import.rs           # Import command tests
├── cli_list.rs             # List command tests
├── cli_mv.rs               # Move command tests
├── cli_reset.rs            # Reset command tests
├── core_workflow.rs        # Core workflow tests
├── error_scenarios.rs      # Error handling tests
├── mode_scope_workflow.rs  # Mode and scope tests
├── sync_workflow.rs        # Sync workflow tests
└── common/                 # Shared test utilities
    ├── mod.rs             # Module declaration
    ├── assertions.rs      # Custom assertions
    └── fixtures.rs        # Test fixtures and helpers
```

## Mode and Scope Specific Test File

**Primary File:** `/home/dustin/projects/jin/tests/mode_scope_workflow.rs`

**What it tests:**
- Layer routing for all 9 layers in the hierarchy
- Layer precedence (higher layers win)
- Deep merge of JSON files across layers
- Mode creation, activation, and isolation
- Scope creation and usage
- Mode-scoped scopes requiring mode activation
- Multiple mode isolation

## Key Test Fixtures

**Located in `/home/dustin/projects/jin/tests/common/fixtures.rs`:**

1. **TestFixture** - Maintains isolated test directories
   - `new()` - Creates temporary directory
   - `path()` - Returns path to test directory

2. **RemoteFixture** - For testing with local/remote repositories
   - `new()` - Creates local and remote bare repositories

3. **Setup helpers:**
   - `jin_init(path)` - Initializes Jin in a directory
   - `setup_test_repo()` - Creates test repo with Jin initialized
   - `setup_jin_with_remote()` - Creates repo with local bare remote
   - `create_mode(name)` - Creates a mode in global Jin repo
   - `create_scope(name)` - Creates a scope in global Jin repo

**Key Issues:**
- `create_mode()` and `create_scope()` modify the global `~/.jin` repository without isolation
