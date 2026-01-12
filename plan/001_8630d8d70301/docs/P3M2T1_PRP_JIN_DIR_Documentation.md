# PRP: P3.M2.T1 - Document JIN_DIR in README

---

## Goal

**Feature Goal**: Add comprehensive documentation to README.md explaining the JIN_DIR environment variable, its default location, how to customize it, and common use cases.

**Deliverable**: A new "Configuration Directory" section in README.md that documents:
- What JIN_DIR is and what it controls
- Default location across platforms
- How to customize with environment variable
- Use cases for custom locations
- Relationship to other configuration

**Success Definition**:
- README.md contains clear JIN_DIR documentation after the "How Jin Works" section
- Documentation follows existing README style and formatting
- Section includes code examples for setting JIN_DIR
- Documentation covers default location, customization, and use cases
- README renders correctly with proper markdown formatting
- No syntax errors in markdown

---

## User Persona

**Target User**: Developers using Jin who want to understand or customize where Jin stores its configuration and data.

**Use Case**: A developer wants to:
- Understand where Jin stores its internal Git repository
- Move Jin data to a different location (e.g., different drive, network storage)
- Set up multiple isolated Jin environments for testing
- Configure Jin in a CI/CD environment with non-standard paths

**User Journey**:
1. User reads README.md to understand Jin
2. User sees reference to `~/.jin/` directory
3. User looks for documentation on how to customize this location
4. User finds "Configuration Directory" section with clear explanation
5. User learns about JIN_DIR environment variable
6. User sets `export JIN_DIR=/custom/path` and Jin uses the new location

**Pain Points Addressed**:
- No explicit documentation of JIN_DIR in user-facing docs
- Users don't know they can customize Jin's storage location
- Unclear what's stored in `~/.jin/` directory
- No examples of customization scenarios

---

## Why

- **PRD Requirement**: PRD Section 19.1 states "Jin uses its own internal Git repo at `$JIN_DIR` (default: `~/.jin/`)" - this needs user-facing documentation
- **Configuration Transparency**: Users should understand where Jin stores data and how to control it
- **Flexibility**: Advanced users need ability to customize Jin directory location
- **Testing & CI/CD**: Tests and CI environments often require custom Jin locations
- **Completeness**: Milestone 3.2 focuses on documentation completeness - JIN_DIR is a core configuration element

---

## What

### User-Visible Behavior

After this change, the README.md will include a new "Configuration Directory" section that explains:

```markdown
## Configuration Directory

Jin stores its internal Git repository and configuration data in a directory controlled by the `JIN_DIR` environment variable.

### Default Location

By default, Jin uses:
- **Linux/macOS**: `~/.jin/`
- **Windows**: `%USERPROFILE%\.jin\`

This directory contains:
- `config.toml` - Global Jin configuration
- Internal Git repository with layers, refs, and objects
- Cached data and metadata

### Customizing JIN_DIR

You can override the default location by setting the `JIN_DIR` environment variable:

```bash
# Set custom Jin directory
export JIN_DIR="$HOME/.local/share/jin"

# Jin will now use this location for all operations
jin init
```

### Use Cases

**Different Drive/Partition**:
Store Jin data on a different drive to save space on your system drive:

```bash
# On Linux/macOS
export JIN_DIR="/mnt/storage/jin"

# On Windows
set JIN_DIR=D:\jin
```

**Network Storage**:
Share Jin configuration across multiple machines:

```bash
export JIN_DIR="/mnt/network-shared/jin"
```

**Isolated Testing**:
Create separate Jin environments for testing:

```bash
# Test environment
export JIN_DIR="/tmp/jin-test"
jin init

# Production uses default ~/.jin/
unset JIN_DIR
jin init
```

**CI/CD Environments**:
Configure Jin for CI pipelines with non-standard home directories:

```bash
export JIN_DIR="$CI_PROJECT_DIR/.jin"
jin sync
```

### Persistence

To make `JIN_DIR` persistent across sessions, add it to your shell configuration:

**Bash** (`~/.bashrc` or `~/.bash_profile`):
```bash
export JIN_DIR="$HOME/.local/share/jin"
```

**Zsh** (`~/.zshrc`):
```bash
export JIN_DIR="$HOME/.local/share/jin"
```

**Fish** (`~/.config/fish/config.fish`):
```fish
set -x JIN_DIR "$HOME/.local/share/jin"
```

**PowerShell** (`$PROFILE`):
```powershell
$env:JIN_DIR = "$env:USERPROFILE\.local\share\jin"
```
```

### Technical Requirements

1. **Placement**: Add section after "How Jin Works" (around line 174), before "Example Use Cases"
2. **Style**: Match existing README formatting (H2 for main heading, H3 for subsections)
3. **Code Examples**: Use fenced code blocks with bash language specifier
4. **Tone**: Professional, concise, benefit-oriented
5. **Cross-Reference**: Link to related documentation if applicable

### Success Criteria

- [ ] README.md contains "Configuration Directory" section
- [ ] Section explains what JIN_DIR controls
- [ ] Default location documented for Linux/macOS/Windows
- [ ] Environment variable usage shown with code examples
- [ ] At least 3 use cases documented (different drive, network storage, testing)
- [ ] Shell persistence examples provided (Bash, Zsh, Fish, PowerShell)
- [ ] Markdown renders correctly with proper formatting
- [ ] Section placement follows logical flow (after "How Jin Works")
- [ ] No markdown syntax errors

---

## All Needed Context

### Context Completeness Check

**Question**: If someone knew nothing about this codebase, would they have everything needed to implement this successfully?

**Answer**: Yes - this is a pure documentation task. The PRP provides:
- Exact section placement in README.md
- Content structure and examples
- Style guidelines matching existing README
- All technical details about JIN_DIR
- Markdown formatting requirements

### Documentation & References

```yaml
# MUST READ - Files to reference

- file: /home/dustin/projects/jin/README.md
  why: "Target file for documentation - understand structure and style"
  lines: 1-243
  pattern: |
    - H2 ## headings for main sections
    - H3 ### headings for subsections
    - Fenced code blocks with ```bash
    - Bold text for emphasis with **text**
    - Bullet points with -
    - Professional, concise tone
  placement: "Add new section after line 174 (after 'How Jin Works')"

- file: /home/dustin/projects/jin/src/core/config.rs
  why: "JIN_DIR implementation reference - confirms default and behavior"
  lines: 75-85
  pattern: |
    if let Ok(jin_dir) = std::env::var("JIN_DIR") {
        return Ok(PathBuf::from(jin_dir).join("config.toml"));
    }
    dirs::home_dir().map(|h| h.join(".jin").join("config.toml"))
  critical: "JIN_DIR takes precedence over default ~/.jin location"

- file: /home/dustin/projects/jin/src/git/repo.rs
  why: "JinRepo default_path implementation - confirms directory structure"
  lines: 152-161
  pattern: |
    if let Ok(jin_dir) = std::env::var("JIN_DIR") {
        return Ok(PathBuf::from(jin_dir));
    }
    dirs::home_dir().map(|h| h.join(".jin"))
  critical: "JIN_DIR points to the bare Git repository root"

- file: /home/dustin/projects/jin/src/test_utils.rs
  why: "Test isolation pattern using JIN_DIR - demonstrates usage"
  lines: 140
  pattern: std::env::set_var("JIN_DIR", &jin_dir);
  gotcha: "JIN_DIR is process-global - must be set before Jin operations"

# EXTERNAL REFERENCES - Best Practices

- url: https://git-scm.com/docs/git-config
  why: "Git's approach to documenting directory configuration (GIT_DIR)"
  section: "Configuration File Location"
  pattern: "Shows environment variable, default locations, override pattern"

- url: https://doc.rust-lang.org/cargo/reference/environment-variables.html
  why: "Cargo's CARGO_HOME documentation - industry standard pattern"
  section: "CARGO_HOME"
  pattern: "Default locations per platform, environment variable override, use cases"

- url: https://rust-lang.github.io/rustup/environment-variables.html
  why: "rustup's RUSTUP_HOME documentation - similar to JIN_DIR"
  section: "RUSTUP_HOME"
  pattern: "Clear explanation of purpose + customization examples"

- url: https://docs.docker.com/reference/cli/docker/
  why: "Docker's DOCKER_CONFIG documentation - configuration directory pattern"
  section: "DOCKER_CONFIG"
  pattern: "Environment variable with platform-specific defaults"

# PROJECT DOCUMENTATION

- docfile: /home/dustin/projects/jin/PRD.md
  why: "PRD Section 19.1 defines JIN_DIR behavior"
  section: "Section 19.1: Jin Repository Location"
  critical: "Confirms JIN_DIR as the canonical environment variable name"

- docfile: /home/dustin/projects/jin/plan/P3M2/PRP.md
  why: "Parent milestone PRP - provides context for JIN_DIR documentation goal"
  section: "Milestone 3.2: JIN_DIR Documentation"
```

### Current README Structure

```markdown
# Jin - Phantom Git Layer System

## What is Jin?
## Quick Start
## Installation
## Command Overview
## Documentation
## Why Jin?
## How Jin Works
  [INSERT CONFIGURATION DIRECTORY SECTION HERE - after line 174]
## Example Use Cases
## Features
## Contributing
## License
## Support
```

### README Style Guide

Based on existing README.md content analysis:

| Element | Pattern | Example |
|---------|---------|---------|
| **Headings** | H2 `##` for main sections, H3 `###` for subsections | `## How Jin Works` |
| **Code blocks** | Fenced with language specifier | `\`\`\`bash` |
| **Bold text** | `**text**` for emphasis | `**Non-disruptive**` |
| **Lists** | Dash `-` for bullet points | `- **Feature 1**` |
| **Tone** | Professional yet approachable, concise | "Think of it as..." |
| **Code examples** | Copy-pasteable, show expected output | `$ jin init` |
| **Cross-references** | Link to detailed docs | `[Layer System](docs/LAYER_SYSTEM.md)` |

---

## Implementation Blueprint

### Documentation Structure

The new "Configuration Directory" section will be added to README.md:

```markdown
## Configuration Directory

Jin stores its internal Git repository and configuration data in a directory controlled by the `JIN_DIR` environment variable.

### Default Location

By default, Jin uses:
- **Linux/macOS**: `~/.jin/`
- **Windows**: `%USERPROFILE%\.jin\`

This directory contains:
- `config.toml` - Global Jin configuration (remote URL, user settings)
- Internal Git repository - Stores layers, refs, and objects
- Cached data and metadata

### Customizing JIN_DIR

You can override the default location by setting the `JIN_DIR` environment variable before running Jin commands:

```bash
# Set custom Jin directory
export JIN_DIR="/custom/path/to/jin"

# All subsequent Jin commands use this location
jin init
jin mode create dev
```

**Important**: `JIN_DIR` must be set **before** running any Jin commands, as it is read at process startup.

### Use Cases

**Different Drive or Partition**:
Store Jin data on a different drive to save space on your system drive:

```bash
# Linux/macOS
export JIN_DIR="/mnt/storage/jin"

# Windows (PowerShell)
$env:JIN_DIR = "D:\jin"
```

**Network Storage**:
Share Jin configuration across multiple machines via network mount:

```bash
export JIN_DIR="/mnt/network-shared/jin"
```

**Isolated Testing**:
Create separate Jin environments for testing without affecting your main setup:

```bash
# Create test environment
export JIN_DIR="/tmp/jin-test"
git init test-project && cd test-project
jin init
# ... run tests ...

# Return to production (uses default ~/.jin/)
unset JIN_DIR
```

**CI/CD Environments**:
Configure Jin for CI pipelines with non-standard home directories:

```bash
# GitLab CI, GitHub Actions, etc.
export JIN_DIR="$CI_PROJECT_DIR/.jin"
jin sync
```

### Making JIN_DIR Persistent

To make `JIN_DIR` persist across shell sessions, add it to your shell configuration:

**Bash** (`~/.bashrc` or `~/.bash_profile`):
```bash
export JIN_DIR="$HOME/.local/share/jin"
```

**Zsh** (`~/.zshrc`):
```bash
export JIN_DIR="$HOME/.local/share/jin"
```

**Fish** (`~/.config/fish/config.fish`):
```fish
set -x JIN_DIR "$HOME/.local/share/jin"
```

**PowerShell** (`$PROFILE`):
```powershell
$env:JIN_DIR = "$env:USERPROFILE\.local\share\jin"
```

After adding to your shell config, reload your shell or run `source ~/.bashrc` (or equivalent).

### Directory Structure

Inside `JIN_DIR`, you'll find:

```
$JIN_DIR/
├── config.toml           # Global configuration (remote URL, user info)
├── refs/                 # Git references
│   └── jin/             # Jin-specific refs (layers, modes, scopes)
├── objects/             # Git objects (blobs, trees, commits)
└── jin/                 # Jin metadata (if present)
```

This is a standard **bare Git repository** structure, which you can inspect with Git commands:

```bash
# List all Jin layers
git --git-dir=$JIN_DIR show-ref | grep refs/jin/layers

# View Jin configuration
cat $JIN_DIR/config.toml
```
```

### Implementation Tasks (ordered by dependencies)

```yaml
Task 1: READ and understand current README structure
  - READ: /home/dustin/projects/jin/README.md lines 1-243
  - IDENTIFY: Section "How Jin Works" ends around line 174
  - CONFIRM: "Example Use Cases" section starts around line 177
  - NOTE: Existing style patterns (H2, H3, code blocks, bold text)
  - DELIVERABLE: Understanding of where to insert new section

Task 2: CREATE new "Configuration Directory" section content
  - CREATE: Markdown content following template structure above
  - INCLUDE: H2 heading "## Configuration Directory"
  - INCLUDE: H3 subsections (Default Location, Customizing, Use Cases, Persistence)
  - INCLUDE: Code examples with ```bash fenced blocks
  - INCLUDE: Platform-specific examples (Linux/macOS/Windows)
  - FOLLOW: Existing README style and tone
  - DELIVERABLE: Complete markdown content ready to insert

Task 3: INSERT new section into README.md
  - FIND: Line 174 (end of "How Jin Works" section)
  - INSERT: New "Configuration Directory" section
  - PRESERVE: All existing content
  - MAINTAIN: Single blank line between sections
  - ENSURE: Proper markdown formatting
  - DELIVERABLE: Updated README.md with new section

Task 4: VERIFY markdown renders correctly
  - CHECK: No markdown syntax errors
  - VERIFY: All code blocks properly closed
  - VERIFY: All links work (if any added)
  - VERIFY: Proper heading hierarchy (H2 → H3)
  - TEST: View rendered markdown in GitHub/GitLab preview
  - DELIVERABLE: Confirmation of correct rendering

Task 5: CREATE summary documentation
  - CREATE: /home/dustin/projects/jin/plan/P3M2T1/research/SUMMARY.md
  - DOCUMENT: Research findings on JIN_DIR
  - DOCUMENT: Best practices from similar tools
  - DOCUMENT: Rationale for content decisions
  - DELIVERABLE: Research summary for future reference
```

### Implementation Patterns & Key Details

```markdown
# ============================================================================
# PATTERN: README Section Structure
# ============================================================================

## Configuration Directory

[Purpose paragraph - 2-3 sentences explaining what JIN_DIR is and controls]

### Default Location
[Explain platform-specific defaults with bullet points]
[Show what's stored in the directory]

### Customizing JIN_DIR
[Explain how to override with environment variable]
[Include code example]

### Use Cases
[3-4 practical use cases with code examples]
[Each use case has clear heading and explanation]

### Making JIN_DIR Persistent
[Shell-specific configuration examples]
[Bash, Zsh, Fish, PowerShell]

# ============================================================================
# PATTERN: Code Examples in README
# ============================================================================

```bash
# Single-line comment above code
export JIN_DIR="$HOME/.local/share/jin"

# Multi-line example with context
# First, set the variable
export JIN_DIR="/tmp/jin-test"

# Then run Jin commands
jin init
```

# ============================================================================
# PATTERN: Platform-Specific Documentation
# ============================================================================

**Linux/macOS**:
```bash
export JIN_DIR="$HOME/.local/share/jin"
```

**Windows**:
```powershell
$env:JIN_DIR = "$env:USERPROFILE\.local\share\jin"
```

# ============================================================================
# GOTCHA: JIN_DIR Timing
# ============================================================================

# CRITICAL: JIN_DIR is process-global - must be set BEFORE Jin operations
# WRONG: jin init && export JIN_DIR=/path  # Too late!
# CORRECT: export JIN_DIR=/path && jin init

# Document this in the README to prevent user confusion
```

### Integration Points

```yaml
README_MD:
  - modify: README.md
  - insert_after: line 174 (end of "How Jin Works" section)
  - insert_before: "## Example Use Cases" section
  - preserve: All existing content
  - maintain: Consistent formatting and style

FUTURE_WORK:
  - P3.M2.T2 will add `jin config` command
  - May want to cross-reference config command once implemented
  - Consider adding link to config command documentation: `See [Configuration](#configuration)`

CROSS_REFERENCES:
  - "How Jin Works" section (line 151-174) - mentions ~/.jin/ directory
  - "Quick Start" section - may want to reference JIN_DIR for advanced setup
```

---

## Validation Loop

### Level 1: Content Validation (Immediate Feedback)

```bash
# Verify README has no markdown syntax errors
# Use a markdown linter if available
command -v markdownlint && markdownlint README.md || echo "No markdownlint installed"

# Or use basic checks
grep -n "^## " README.md | head -20  # Show H2 headings
grep -n "^### " README.md | head -20  # Show H3 headings

# Expected: New "## Configuration Directory" heading appears
# Expected: Proper heading hierarchy (H2 → H3)
```

### Level 2: Link Validation

```bash
# Check for broken internal links
grep -o '\[.*\](.*)' README.md | while read link; do
    # Extract URL/path and validate
    echo "Checking: $link"
done

# Expected: No broken links
# Expected: Any new links are valid
```

### Level 3: Rendering Validation

```bash
# Test markdown rendering
command -v glow && glow README.md || echo "No glow renderer installed"

# Or open in browser if using GitHub/GitLab
echo "Open README.md in GitHub/GitLab preview to verify rendering"

# Expected:
# - Code blocks render correctly
# - Bold text displays as bold
# - Headings have proper hierarchy
# - No formatting artifacts
```

### Level 4: Content Quality Validation

```bash
# Manual review checklist
echo "Check the following:"
echo "✓ Section explains what JIN_DIR is"
echo "✓ Default location documented for all platforms"
echo "✓ Environment variable usage shown with examples"
echo "✓ At least 3 use cases documented"
echo "✓ Shell persistence examples provided"
echo "✓ Tone matches existing README content"
echo "✓ No markdown syntax errors"

# Read the new section
sed -n '/^## Configuration Directory/,/^## [^C]/p' README.md | head -100

# Expected: Clear, concise, actionable documentation
```

---

## Final Validation Checklist

### Technical Validation

- [ ] README.md contains "## Configuration Directory" heading
- [ ] Section placed after "How Jin Works" (line ~174)
- [ ] No markdown syntax errors (all code blocks closed, proper escaping)
- [ ] Heading hierarchy correct (H2 → H3)
- [ ] Code blocks use proper language specifier (```bash)
- [ ] No broken links (if any links added)

### Content Validation

- [ ] Section explains what JIN_DIR controls
- [ ] Default location documented for Linux/macOS/Windows
- [ ] Environment variable syntax shown: `export JIN_DIR="/path"`
- [ ] At least 3 use cases with code examples:
  - [ ] Different drive/partition
  - [ ] Network storage
  - [ ] Isolated testing
  - [ ] CI/CD environments
- [ ] Shell persistence examples for Bash, Zsh, Fish, PowerShell
- [ ] Warning about timing (must set before Jin operations)
- [ ] Directory structure explanation

### Style & Tone Validation

- [ ] Matches existing README tone (professional yet approachable)
- [ ] Uses consistent formatting (bold, code blocks, lists)
- [ ] Code examples are copy-pasteable
- [ ] Explanations are concise and clear
- [ ] No jargon without explanation

### User Experience Validation

- [ ] Section flows logically from previous content
- [ ] User can find section via table of contents (if applicable)
- [ ] Examples work when copied to terminal
- [ ] Cross-references to related sections (if applicable)
- [ ] Advanced users can skim, beginners can understand

---

## Anti-Patterns to Avoid

- **Don't** add JIN_DIR documentation to unrelated sections (keep it focused)
- **Don't** use overly technical jargon without explanation
- **Don't** forget to document the timing requirement (set JIN_DIR before Jin commands)
- **Don't** use inconsistent code block language specifiers
- **Don't** break existing README structure or formatting
- **Don't** add broken links or references
- **Don't** make the section too long (keep it concise and scannable)
- **Don't** forget platform-specific examples (Windows users exist!)
- **Don't** use complex examples - keep them simple and practical
- **Don't** assume knowledge - explain what's in the JIN_DIR directory

---

## Confidence Score

**Score: 10/10**

**Rationale**:
- Pure documentation task - no code implementation risks
- Clear placement location in README.md
- Comprehensive research on JIN_DIR implementation
- Best practices researched from similar CLI tools
- Template and examples provided for all content
- No external dependencies or API changes
- Reversible change (easy to modify or remove)
- Existing PRP examples in codebase show successful pattern

**Risk Mitigation**:
- Markdown-only changes are inherently low-risk
- Can preview changes before committing
- Easy to iterate based on feedback
- No impact on codebase functionality

---

## Research Artifacts

Research has been completed covering:

| Topic | Key Insights |
|-------|--------------|
| **JIN_DIR Implementation** | Defined in config.rs and repo.rs, checked via `std::env::var("JIN_DIR")`, defaults to `~/.jin/` |
| **README Structure** | Clear sections with H2/H3 hierarchy, professional tone, code-first examples |
| **Best Practices** | Git, Cargo, rustup, Docker all document similar config variables with defaults + override patterns |
| **Use Cases** | Different drive, network storage, isolated testing, CI/CD are common scenarios |
| **Platform Support** | Must document Linux/macOS (bash/zsh/fish) and Windows (PowerShell) |

Key external references:
- Git Configuration: https://git-scm.com/docs/git-config
- Cargo CARGO_HOME: https://doc.rust-lang.org/cargo/reference/environment-variables.html
- rustup RUSTUP_HOME: https://rust-lang.github.io/rustup/environment-variables.html
- Docker DOCKER_CONFIG: https://docs.docker.com/reference/cli/docker/

---

## Appendix: Research Summary

### JIN_DIR Implementation Summary

**Definition**: `JIN_DIR` is an environment variable that specifies the root directory where Jin stores its internal Git repository and configuration data.

**Default Value**: `~/.jin/` (user's home directory)

**Override Method**: Set `JIN_DIR` environment variable before running Jin commands

**Implementation Files**:
- `src/core/config.rs:75-85` - Config file loading with JIN_DIR support
- `src/git/repo.rs:152-161` - JinRepo path resolution with JIN_DIR support
- `src/test_utils.rs:140` - Test isolation using JIN_DIR

**Directory Contents**:
- `config.toml` - Global configuration
- `refs/jin/` - Jin-specific Git references
- `objects/` - Git object storage
- Standard bare Git repository structure

### Documentation Best Practices Summary

Based on research of Git, Cargo, rustup, and Docker:

1. **Clear Purpose Statement**: Explain what the variable controls
2. **Platform-Specific Defaults**: Show default for each OS
3. **Override Pattern**: Show environment variable syntax
4. **Use Cases**: Provide 3-5 practical scenarios
5. **Persistence**: Document shell configuration methods
6. **Code Examples**: Copy-pasteable examples with comments

### Content Template

```markdown
## Configuration Directory

[2-3 sentence purpose statement]

### Default Location
[Platform-specific defaults]
[What's stored in the directory]

### Customizing JIN_DIR
[How to override]
[Code example]

### Use Cases
[3-4 practical scenarios]

### Making JIN_DIR Persistent
[Shell configuration examples]
```

---

## Appendix: Example Output

After implementation, the new README section will look like:

```markdown
## Configuration Directory

Jin stores its internal Git repository and configuration data in a directory controlled by the `JIN_DIR` environment variable.

### Default Location

By default, Jin uses:
- **Linux/macOS**: `~/.jin/`
- **Windows**: `%USERPROFILE%\.jin\`

This directory contains:
- `config.toml` - Global Jin configuration (remote URL, user settings)
- Internal Git repository - Stores layers, refs, and objects
- Cached data and metadata

### Customizing JIN_DIR

You can override the default location by setting the `JIN_DIR` environment variable before running Jin commands:

```bash
# Set custom Jin directory
export JIN_DIR="/custom/path/to/jin"

# All subsequent Jin commands use this location
jin init
```

**Important**: `JIN_DIR` must be set **before** running any Jin commands, as it is read at process startup.

[... additional use cases and persistence examples ...]
```

---

**End of PRP**
