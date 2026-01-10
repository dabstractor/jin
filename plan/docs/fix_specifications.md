# Fix Specifications

## Fix 1: Add `--local` Flag to `jin add`

### Overview
Enable access to Layer 8 (UserLocal) via `jin add --local` command.

### Files to Modify

1. **`src/cli/args.rs`** - Add flag to AddArgs
```rust
pub struct AddArgs {
    pub files: Vec<String>,
    pub mode: bool,
    pub scope: Option<String>,
    pub project: bool,
    pub global: bool,
    pub local: bool,  // ADD THIS
}
```

2. **`src/staging/router.rs`** - Update route_to_layer()
```rust
// Add validation: --local cannot combine with other flags
// Add routing: if options.local { return Layer::UserLocal }
```

3. **`src/commands/add.rs`** - Pass local flag through

### Validation Rules
- `--local` cannot be combined with `--mode`, `--scope`, `--project`, or `--global`
- `--local` does NOT require active mode or scope
- Files go to `~/.jin/local/` storage path

### Test Cases
1. `jin add .config --local` → targets Layer 8
2. `jin add .config --local --mode` → error: incompatible flags
3. `jin add .config --local --global` → error: incompatible flags
4. Verify file appears in `~/.jin/local/` after commit

---

## Fix 2: SIGPIPE Handling

### Overview
Prevent panic when `jin log` output is piped to commands like `head` that close stdin early.

### Files to Modify

1. **`src/main.rs`** - Add SIGPIPE handler at program start
```rust
// Option A: Reset SIGPIPE to default behavior
#[cfg(unix)]
fn reset_sigpipe() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

// Option B: Use nix crate
use nix::sys::signal::{self, Signal, SigHandler};
signal::signal(Signal::SIGPIPE, SigHandler::SigDfl).ok();
```

### Alternative Approach
Wrap all stdout writes in `src/commands/log.rs` with error handling:
```rust
use std::io::{self, Write};

fn write_output(s: &str) -> io::Result<()> {
    writeln!(io::stdout(), "{}", s)?;
    Ok(())
}

// Use write_output() instead of println!() and handle BrokenPipe gracefully
```

### Dependencies
- May need to add `libc` or `nix` crate to Cargo.toml
- Or use `signal-hook` crate for cleaner handling

### Test Cases
1. `jin log | head -1` → exits gracefully (no panic)
2. `jin log | cat` → full output shown
3. `jin log` (no pipe) → normal operation

---

## Fix 3: Mode Switching UX

### Overview
When switching modes with `jin mode use`, automatically handle workspace metadata to prevent detached state.

### Option A: Auto-Clear Metadata (Recommended)

**File**: `src/commands/mode.rs`

In `ModeAction::Use` handler:
```rust
// After activating mode, check if workspace metadata references different mode
let metadata = WorkspaceMetadata::load()?;
if metadata.mode != new_mode {
    // Clear last_applied.json to prevent detached state
    let metadata_path = jin_dir.join("workspace/last_applied.json");
    if metadata_path.exists() {
        std::fs::remove_file(&metadata_path)?;
    }
    println!("Cleared workspace metadata (mode changed). Run 'jin apply' to apply new mode.");
}
```

### Option B: New `jin switch` Command

**New File**: `src/commands/switch.rs`

```rust
// Combines: mode use + metadata clear + apply
pub fn execute(name: String, force: bool) -> Result<()> {
    // 1. Activate mode
    mode::activate(&name)?;

    // 2. Clear workspace metadata
    clear_workspace_metadata()?;

    // 3. Apply new mode
    apply::execute(ApplyArgs { force, dry_run: false })?;

    Ok(())
}
```

### Test Cases
1. Switch from mode A to mode B → no detached state error
2. After switch, `jin apply` works without `--force`
3. Modified workspace files trigger appropriate warning

---

## Fix 4: Reset in Detached State (Lower Priority)

### Overview
Allow `jin reset --hard --force` to bypass detached state validation.

### File: `src/commands/reset.rs`

```rust
// Current: Always validates workspace attached before hard reset
// Change: Skip validation if both --hard and --force are provided

if args.hard {
    if !args.force {
        validate_workspace_attached(&context, &repo)?;
        // Prompt for confirmation
    }
    // If --force, skip validation and proceed
}
```

### Test Cases
1. `jin reset --hard` in detached state → error with instructions
2. `jin reset --hard --force` in detached state → proceeds with reset
3. Files are removed from workspace and .gitignore updated
