# Research: Command Implementation Examples

## Example 1: Mode Command (Subcommand Pattern)

**File**: `/home/dustin/projects/jin/src/commands/mode.rs`

### CLI Definition
```rust
#[derive(Subcommand, Debug)]
pub enum ModeAction {
    Create { name: String },
    Use { name: String },
    List,
    Delete { name: String },
    Show,
    Unset,
}
```

### Execute Pattern
```rust
pub fn execute(action: ModeAction) -> Result<()> {
    match action {
        ModeAction::Create { name } => create(&name),
        ModeAction::Use { name } => use_mode(&name),
        ModeAction::List => list(),
        ModeAction::Delete { name } => delete(&name),
        ModeAction::Show => show(),
        ModeAction::Unset => unset(),
    }
}
```

### State Management Pattern
```rust
fn use_mode(name: &str) -> Result<()> {
    // Validate
    validate_mode_name(name)?;

    // Load context
    let mut context = ProjectContext::load()?;

    // Update state
    context.mode = Some(name.to_string());

    // Save
    context.save()?;

    println!("Active mode set to: {}", name);
    Ok(())
}
```

### Validation Pattern
```rust
fn validate_mode_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(JinError::Other("Mode name cannot be empty".to_string()));
    }
    // Additional validation...
    Ok(())
}
```

---

## Example 2: Context Command (Simple Display)

**File**: `/home/dustin/projects/jin/src/commands/context.rs`

### Execute Pattern
```rust
pub fn execute() -> Result<()> {
    let context = match ProjectContext::load() {
        Ok(ctx) => ctx,
        Err(JinError::NotInitialized) => {
            return Err(JinError::NotInitialized);
        }
        Err(_) => ProjectContext::default(),
    };

    println!("Current Jin context:");
    println!("  Active mode:   {}", context.mode.as_deref().unwrap_or("(none)"));
    println!("  Active scope:  {}", context.scope.as_deref().unwrap_or("(none)"));
    println!("  Active project: {}", context.project.as_deref().unwrap_or("(none)"));

    Ok(())
}
```

---

## Example 3: Reset Command (Arguments with Options)

**File**: `/home/dustin/projects/jin/src/commands/reset.rs`

### Args Definition
```rust
#[derive(Args, Debug)]
pub struct ResetArgs {
    #[arg(long)]
    pub soft: bool,
    #[arg(long)]
    pub mixed: bool,
    #[arg(long)]
    pub hard: bool,
    #[arg(long)]
    pub mode: bool,
    #[arg(long)]
    pub scope: Option<String>,
    #[arg(long)]
    pub project: bool,
    #[arg(long)]
    pub global: bool,
    #[arg(long, short = 'f')]
    pub force: bool,
}
```

### Execute Pattern
```rust
pub fn execute(args: ResetArgs) -> Result<()> {
    // Determine reset mode
    let mode = if args.soft {
        ResetMode::Soft
    } else if args.hard {
        ResetMode::Hard
    } else {
        ResetMode::Mixed // Default
    };

    // Load context
    let context = ProjectContext::load()?;

    // Determine target layer
    let target = determine_target_layer(&args, &context)?;

    // User confirmation for destructive operations
    if mode == ResetMode::Hard && !args.force {
        if !prompt_confirmation(&message)? {
            println!("Reset cancelled");
            return Ok(());
        }
    }

    // Execute reset...
    Ok(())
}
```

---

## Summary of Key Patterns

| Pattern | Description | Use For |
|---------|-------------|---------|
| `execute()` | Main entry point returning `Result<()>` | All commands |
| Context load/save | `ProjectContext::load()` / `save()` | Stateful commands |
| Input validation | Separate validation functions | User input |
| Error handling | `JinError::NotInitialized`, etc. | All commands |
| Graceful fallbacks | `match ProjectContext::load()` | Display commands |

## Sources
- `/home/dustin/projects/jin/src/commands/mode.rs`
- `/home/dustin/projects/jin/src/commands/context.rs`
- `/home/dustin/projects/jin/src/commands/reset.rs`
