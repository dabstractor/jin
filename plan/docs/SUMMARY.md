# CLI Design Research - Quick Reference Summary

## Research Completed: December 27, 2025

### Files Generated
- **cli_design_patterns.md** (922 lines, 25KB) - Comprehensive research with detailed examples

---

## Key Findings by Topic

### 1. COMMAND NAMING CONVENTIONS

| Aspect | Recommendation | Examples |
|--------|---|---|
| **Letters** | Lowercase only with hyphens | `git`, `cargo`, `docker`, NOT `GitTool` |
| **Style** | Kebab-case for multi-word | `--example-flag`, NOT `--example_flag` |
| **Length** | Short but descriptive | `add`, `commit`, `build`, NOT `ProcessAndAdd` |
| **Conflicts** | Avoid generic names | `curl`, `grep`, `tar` are good |

#### Naming Patterns
- **Verb-Noun (Traditional Unix):** `git add`, `cargo build`, `kubectl apply`
- **Noun-Verb (Docker Modern):** `docker container run`, `docker image build`
- **Key Rule:** Pick ONE pattern and be consistent across all commands

---

### 2. SUBCOMMAND ORGANIZATION

#### Pattern A: Category-Based (Verb-Noun)
**Best for:** Sequential action-oriented tools
```
Build Commands: cargo build, cargo run, cargo test
Manifest Commands: cargo add, cargo update, cargo tree
Package Commands: cargo new, cargo install
Publishing Commands: cargo publish, cargo login
```

#### Pattern B: Resource-Based (Noun-Verb)
**Best for:** Resource management tools
```
docker container (run, ls, exec, stop)
docker image (build, push, pull, ls)
docker network (create, connect, inspect)
```

#### Pattern C: Verb-Based (Action-First)
**Best for:** Multi-resource tools
```
kubectl get (pods, nodes, services)
kubectl apply (pod, svc, deployment)
kubectl delete (pod, svc, deployment)
```

**Organization Tips:**
- Group related commands together logically
- Use consistent naming across groups
- Prefix related flags for discoverability

---

### 3. FLAG NAMING & CONSISTENCY

#### Standard Reserved Flags
```
-h, --help              Show help
-v, --verbose           Verbose output (stackable: -vv)
-q, --quiet             Suppress output
-f, --force             Force operation
-a, --all               All items
-n, --name              Specify name
-m, --message           Specify message
-d, --detach            Run in background
-i, --interactive       Interactive mode
-t, --tty               Allocate TTY
--json                  JSON output
--color [auto|always|never]  Color control
--dry-run               Preview without executing
```

#### Best Practices
1. **Always provide both short + long forms**
   ```
   -h / --help
   -v / --verbose
   -f / --force
   ```

2. **Use kebab-case ONLY**
   ```
   ✅ --example-flag
   ❌ --example_flag
   ❌ --exampleFlag
   ```

3. **Flags over arguments** (clearer, better autocomplete)
   ```
   ❌ heroku fork destapp -a sourceapp
   ✅ heroku fork --from sourceapp --to destapp
   ```

4. **Group related flags with prefixes**
   ```
   docker:
     --cap-add, --cap-drop
     --health-cmd, --health-interval, --health-timeout
     --cpu-shares, --cpus, --memory
   ```

5. **Common flag ordering in help**
   - Display flags first (--json, --color, --verbose)
   - Execution flags next (--dry-run, --force)
   - Resource flags last (--name, --label, --config)

---

### 4. HELP TEXT STRUCTURE

#### Three-Level Help System

**Level 1: Brief (Default No-Args)**
- Description: 1-2 sentences
- Usage: Show command syntax
- Examples: 1-2 practical examples
- Most common flags: 3-5 only
- Exit text: "See 'help' for more information"
- **Goal:** Understand what it does in 3-5 seconds

**Level 2: Full (-h / --help)**
- NAME: One-line description
- USAGE: Complete syntax with all options
- DESCRIPTION: Detailed behavior explanation
- OPTIONS/FLAGS: All flags with descriptions
- EXAMPLES: Multiple realistic examples
- SEE ALSO: Related commands
- EXIT STATUS: Exit codes explained
- **Goal:** Complete reference documentation

**Level 3: Deep Dive**
- `jin help <command>` — Opens man page
- Detailed walkthroughs
- Advanced usage patterns
- **Goal:** Learn all advanced features

#### Help Text Best Practices
| Practice | Do | Don't |
|----------|----|----|
| **Lead with examples** | Put examples early | Bury them in description |
| **Be concise** | 1 sentence per flag | Paragraph per flag |
| **Action-oriented** | "Force operation to..." | "The force flag causes..." |
| **Include defaults** | "Default: 30 seconds" | Omit default values |
| **Show constraints** | "1-3600 range" | Leave bounds undefined |
| **Skim-friendly** | Short paragraphs, bold terms | Dense text blocks |
| **Suggest next steps** | "See 'jin status' next" | Leave user hanging |

---

### 5. ERROR MESSAGE PATTERNS

#### Error Message Structure
```
<error-type>: <what-went-wrong>

<suggestion-for-fixing-it>

Run '<command> --help' for more information
```

#### Five Error Categories

**1. Validation Errors**
```
error: no changes added to commit

Hint: Did you forget to use 'git add'?
Hint: See 'git commit --help'
```

**2. Not Found Errors**
```
Error: pod 'my-app' not found in namespace 'default'

Did you mean:
  my-app-abc123 (Running)
  my-app-def456 (Pending)
```

**3. Permission Errors**
```
permission denied while trying to connect to Docker daemon

You need to add your user to 'docker' group:
  sudo usermod -aG docker $USER
```

**4. Configuration/State Errors**
```
CONFLICT (content): Merge conflict in README.md

Fix conflicts and complete:
  1. Edit conflicting files
  2. Run: git add .
  3. Run: git commit
```

**5. Network/External Errors**
```
error: failed to upload to crates.io

Connection timeout (30s). This may be temporary.

Retry: cargo publish
Check status: https://status.crates.io/
```

#### Error Message Best Practices
- **Prioritize actionability:** What can user do to fix it?
- **Avoid jargon:** Use plain language
- **Use color sparingly:** Red for errors, green for success
- **Signal-to-noise ratio:** Group similar errors, not dozens of lines
- **Suggest next steps:** Always provide one clear action
- **Exit codes matter:** Use standard codes for automation
  - 0 = Success
  - 1 = General error
  - 2 = Command syntax misuse

---

## Real-World Tool Patterns

### Git (Verb-Noun, Unix-style)
- **Pattern:** Verb-noun with extensive subcommands
- **Strengths:** Intuitive for Unix users, highly discoverable
- **Flags:** Consistent -short/--long naming
- **Errors:** Helpful hints, "Did you mean?" suggestions

### Docker (Noun-Verb, Resource-based)
- **Pattern:** Evolution from flat commands to grouped resource types
- **Strengths:** Scales well as features grow, logical organization
- **Flags:** Related flags grouped with prefixes (--cap-*, --health-*, etc.)
- **Errors:** Suggests similar images/resources, lists options

### Cargo (Verb-Noun, Rust-focused)
- **Pattern:** Organized into 4 functional categories
- **Strengths:** Clear grouping: Build, Manifest, Package, Publishing
- **Flags:** Global flags vs command-specific flags clearly separated
- **Errors:** Suggests fixes, explains error codes

### Kubectl (Verb-Noun, Multi-resource)
- **Pattern:** Actions first, applied to multiple resource types
- **Strengths:** Minimal command duplication, very discoverable
- **Flags:** Consistent across resource types
- **Errors:** Explains constraint violations, shows current state

---

## Quick Checklist for Jin CLI Design

- [ ] Choose verb-noun or noun-verb pattern (then stick with it)
- [ ] Organize commands into 4-6 logical categories
- [ ] Reserve standard single-letter flags (-h, -v, -q, -f, -a)
- [ ] Use kebab-case ONLY for all multi-word elements
- [ ] Implement three-level help system
- [ ] Lead help text with examples
- [ ] Provide actionable error messages with suggestions
- [ ] Group related flags with common prefixes
- [ ] Document all flags with defaults and constraints
- [ ] Use proper exit codes (0, 1, 2, 126, 127, 128)
- [ ] Suggest next steps in error messages and help
- [ ] Test help text with new users (3-5 second rule)

---

## Recommended Tools for Implementation

### Rust CLI Framework: Clap
- **Status:** Most popular Rust CLI parser
- **Features:**
  - Derive API (simpler) and Builder API (flexible)
  - Automatic help generation
  - Subcommand support
  - Shell completion
  - Argument validation
- **Kebab-case enforcement:** Built-in
- **Documentation:** [docs.rs/clap](https://docs.rs/clap/latest/clap/)

---

## Sources Used

### Authoritative Guides
1. [Command Line Interface Guidelines - clig.dev](https://clig.dev/)
2. [Heroku CLI Style Guide](https://devcenter.heroku.com/articles/cli-style-guide)
3. [Better CLI - Design Guidelines](https://bettercli.org/)

### Tool Documentation
4. [10 Design Principles for Delightful CLIs - Atlassian](https://www.atlassian.com/blog/it-teams/10-design-principles-for-delightful-clis)
5. [Git Architecture & Design](https://aosabook.org/en/v2/git.html)
6. [Cargo Book - Commands](https://doc.rust-lang.org/cargo/commands/cargo.html)
7. [Docker Container Reference](https://docs.docker.com/reference/cli/docker/container/run/)
8. [Kubernetes Kubectl Reference](https://kubernetes.io/docs/reference/kubectl/)

### Specialized Resources
9. [CLI Design Best Practices - Cody A. Ray](https://codyaray.com/2020/07/cli-design-best-practices)
10. [Rust Clap Framework Documentation](https://docs.rs/clap/latest/clap/)
11. [CLI UX Best Practices - Evil Martians](https://evilmartians.com/chronicles/cli-ux-best-practices-3-patterns-for-improving-progress-displays)
12. [Elevate Developer Experiences with CLI Design - Thoughtworks](https://www.thoughtworks.com/en-us/insights/blog/engineering-effectiveness/elevate-developer-experiences-with-cli-design-guidelines)

---

## Document Information

**Research Completed:** December 27, 2025
**Location:** `/home/dustin/projects/jin/plan/P4M1T1/research/`

**Files:**
- `cli_design_patterns.md` - Comprehensive guide (922 lines, 25KB)
- `SUMMARY.md` - This quick reference (this file)
