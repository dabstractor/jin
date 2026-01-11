//! Shared argument types for CLI commands

use clap::Args;

/// Arguments for the `add` command
#[derive(Args, Debug)]
#[command(after_help = r#"LAYER ROUTING:
  Flags                  Target Layer           Storage
  ─────────────────────────────────────────────────────────────────────────────
  (no flags)             → Layer 7 (ProjectBase)     jin/project/<project>/
  --mode                 → Layer 2 (ModeBase)        jin/mode/<mode>/
  --mode --project       → Layer 5 (ModeProject)     jin/mode/<mode>/project/<project>/
  --scope=<X>            → Layer 6 (ScopeBase)       jin/scope/<scope>/
  --mode --scope=<X>     → Layer 3 (ModeScope)       jin/mode/<mode>/scope/<scope>/
  --mode --scope=<X> --project
                         → Layer 4 (ModeScopeProject) jin/mode/<mode>/scope/<scope>/project/<project>/
  --global               → Layer 1 (GlobalBase)      jin/global/
  --local                → Layer 8 (UserLocal)       ~/.jin/local/
"#)]
pub struct AddArgs {
    /// Files to stage
    pub files: Vec<String>,

    /// Target mode layer
    #[arg(long)]
    pub mode: bool,

    /// Target scope layer
    #[arg(long)]
    pub scope: Option<String>,

    /// Add to mode-project layer (Layer 5, requires --mode). For project-base layer (Layer 7), use: jin add <file> without flags
    #[arg(long)]
    pub project: bool,

    /// Target global layer
    #[arg(long)]
    pub global: bool,

    /// Target user-local layer (Layer 8, machine-specific)
    #[arg(long)]
    pub local: bool,
}

/// Arguments for the `commit` command
#[derive(Args, Debug)]
pub struct CommitArgs {
    /// Commit message
    #[arg(short, long)]
    pub message: String,

    /// Dry run - show what would be committed
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for the `apply` command
#[derive(Args, Debug)]
#[command(after_help = r#"CONFLICT RESOLUTION:
  When merge conflicts are detected:
  • Operation pauses and creates .jinmerge files
  • Non-conflicting files are still applied
  • Resolve conflicts manually in .jinmerge files
  • Validate resolution: jin resolve <file>
  • Check status: jin status

  .jinmerge files contain Git-style conflict markers:
    <<<<<<< layer1/ref/path/
    content from layer 1
    =======
    content from layer 2
    >>>>>>> layer2/ref/path/

  Remove conflict markers and keep desired content,
  then run 'jin resolve' to apply the resolution.
"#)]
pub struct ApplyArgs {
    /// Force apply even if workspace is dirty
    #[arg(long)]
    pub force: bool,

    /// Show what would be applied
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for the `reset` command
#[derive(Args, Debug)]
pub struct ResetArgs {
    /// Keep changes in staging
    #[arg(long)]
    pub soft: bool,

    /// Unstage but keep in workspace (default)
    #[arg(long)]
    pub mixed: bool,

    /// Discard all changes
    #[arg(long)]
    pub hard: bool,

    /// Reset mode layer
    #[arg(long)]
    pub mode: bool,

    /// Reset scope layer
    #[arg(long)]
    pub scope: Option<String>,

    /// Reset project layer
    #[arg(long)]
    pub project: bool,

    /// Reset global layer
    #[arg(long)]
    pub global: bool,

    /// Skip confirmation prompt and bypass detached state validation (use for recovery)
    #[arg(long, short = 'f')]
    pub force: bool,
}

/// Arguments for the `rm` command
#[derive(Args, Debug)]
pub struct RmArgs {
    /// Files to remove
    pub files: Vec<String>,

    /// Target mode layer
    #[arg(long)]
    pub mode: bool,

    /// Target scope layer
    #[arg(long)]
    pub scope: Option<String>,

    /// Target project layer
    #[arg(long)]
    pub project: bool,

    /// Target global layer
    #[arg(long)]
    pub global: bool,

    /// Target user-local layer (Layer 8, machine-specific)
    #[arg(long)]
    pub local: bool,

    /// Skip confirmation prompt for workspace deletion
    #[arg(long, short = 'f')]
    pub force: bool,

    /// Show what would be removed without doing it
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for the `mv` command
#[derive(Args, Debug)]
pub struct MvArgs {
    /// Source and destination file pairs (src1, dst1, src2, dst2, ...)
    pub files: Vec<String>,

    /// Target mode layer
    #[arg(long)]
    pub mode: bool,

    /// Target scope layer
    #[arg(long)]
    pub scope: Option<String>,

    /// Target project layer
    #[arg(long)]
    pub project: bool,

    /// Target global layer
    #[arg(long)]
    pub global: bool,

    /// Target user-local layer (Layer 8, machine-specific)
    #[arg(long)]
    pub local: bool,

    /// Skip confirmation prompt for workspace moves
    #[arg(long, short = 'f')]
    pub force: bool,

    /// Show what would be moved without doing it
    #[arg(long)]
    pub dry_run: bool,
}

/// Arguments for the `diff` command
#[derive(Args, Debug)]
pub struct DiffArgs {
    /// First layer to compare
    pub layer1: Option<String>,

    /// Second layer to compare
    pub layer2: Option<String>,

    /// Show staged changes
    #[arg(long)]
    pub staged: bool,
}

/// Arguments for the `log` command
#[derive(Args, Debug)]
pub struct LogArgs {
    /// Layer to show history for
    #[arg(long)]
    pub layer: Option<String>,

    /// Number of entries to show
    #[arg(long, default_value = "10")]
    pub count: usize,
}

/// Arguments for the `import` command
#[derive(Args, Debug)]
#[command(after_help = r#"LAYER ROUTING:
  Flags                  Target Layer           Storage
  ─────────────────────────────────────────────────────────────────────────────
  (no flags)             → Layer 7 (ProjectBase)     jin/project/<project>/
  --mode                 → Layer 2 (ModeBase)        jin/mode/<mode>/
  --mode --project       → Layer 5 (ModeProject)     jin/mode/<mode>/project/<project>/
  --scope=<X>            → Layer 6 (ScopeBase)       jin/scope/<scope>/
  --mode --scope=<X>     → Layer 3 (ModeScope)       jin/mode/<mode>/scope/<scope>/
  --mode --scope=<X> --project
                         → Layer 4 (ModeScopeProject) jin/mode/<mode>/scope/<scope>/project/<project>/
  --global               → Layer 1 (GlobalBase)      jin/global/
  --local                → Layer 8 (UserLocal)       ~/.jin/local/
"#)]
pub struct ImportArgs {
    /// Files to import from Git
    pub files: Vec<String>,

    /// Force import even if files are modified
    #[arg(long)]
    pub force: bool,

    /// Target mode layer
    #[arg(long)]
    pub mode: bool,

    /// Target scope layer
    #[arg(long)]
    pub scope: Option<String>,

    /// Add to mode-project layer (Layer 5, requires --mode). For project-base layer (Layer 7), use: jin import <file> without flags
    #[arg(long)]
    pub project: bool,

    /// Target global layer
    #[arg(long)]
    pub global: bool,

    /// Target user-local layer (Layer 8, machine-specific)
    #[arg(long)]
    pub local: bool,
}

/// Arguments for the `export` command
#[derive(Args, Debug)]
pub struct ExportArgs {
    /// Files to export back to Git
    pub files: Vec<String>,
}

/// Arguments for the `repair` command
#[derive(Args, Debug)]
pub struct RepairArgs {
    /// Show what would be repaired
    #[arg(long)]
    pub dry_run: bool,

    /// Check workspace state without making repairs
    #[arg(long)]
    pub check: bool,
}

/// Arguments for the `link` command
#[derive(Args, Debug)]
pub struct LinkArgs {
    /// Remote repository URL
    pub url: String,

    /// Force update existing remote
    #[arg(long)]
    pub force: bool,
}

/// Arguments for the `push` command
#[derive(Args, Debug)]
#[command(after_help = r#"PUSH SAFETY:
  • Fetches automatically before pushing
  • Requires clean merge state
  • Rejects push if local is behind remote
  • Use --force to bypass (caution: may overwrite remote changes)
"#)]
pub struct PushArgs {
    /// Force push (overwrite remote)
    #[arg(long)]
    pub force: bool,
}

/// Arguments for the `resolve` command
#[derive(Args, Debug)]
pub struct ResolveArgs {
    /// File(s) to resolve (optional, resolves all if not specified)
    pub files: Vec<String>,

    /// Resolve all remaining conflicts
    #[arg(long, short = 'a')]
    pub all: bool,

    /// Skip confirmation prompts
    #[arg(long, short = 'f')]
    pub force: bool,

    /// Show what would be resolved without doing it
    #[arg(long)]
    pub dry_run: bool,
}
