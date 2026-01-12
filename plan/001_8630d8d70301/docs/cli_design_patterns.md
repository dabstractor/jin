# CLI Design Best Practices Research

## Overview

This document synthesizes best practices for CLI design from leading tools: Git, Docker, Cargo, and Kubectl. Research based on authoritative sources and design guidelines from industry-leading projects.

**Research Date:** December 27, 2025

---

## 1. Command Naming Conventions

### Principle: Clarity Over Brevity

Choose simple, memorable names using **only lowercase letters and hyphens**. Keep them short for frequent typing—avoid generic terms that conflict with existing commands. Examples:
- ✅ Good: `curl`, `grep`, `tar`
- ❌ Bad: `DownloadURL`, `FetchData`, `ProcessFile`

**Sources:**
- [Command Line Interface Guidelines - clig.dev](https://clig.dev/)
- [CLI Design Best Practices - Cody A. Ray](https://codyaray.com/2020/07/cli-design-best-practices)

### Verb-Noun vs Noun-Verb Organization

**Two Primary Approaches:**

#### 1. Verb-Noun (Commands First)
Traditional Unix style where verbs come first:
- `git add <file>`
- `git commit -m "message"`
- `cargo build`
- `cargo test`

**Advantages:**
- Intuitive for users familiar with Unix tools
- Natural language flow
- Easy to discover actions: `git <TAB>`

**Tools using this:** Git, Cargo, Kubectl

#### 2. Noun-Verb (Resources First)
Docker's newer management command structure grouping by resource type:
- `docker container run`
- `docker container ls`
- `docker image build`
- `docker image push`
- `docker network create`

**Advantages:**
- Better organization as feature set grows
- Logical grouping of related commands
- Easier to maintain consistency across resource types

**Tools using this:** Docker (modern management commands)

**Important Note:** Consistency matters more than choice. Once you pick a pattern, stick with it across all commands.

**Sources:**
- [Heroku CLI Style Guide](https://devcenter.heroku.com/articles/cli-style-guide)
- Docker CLI Cheat Sheet
- [10 Design Principles for Delightful CLIs - Atlassian](https://www.atlassian.com/blog/it-teams/10-design-principles-for-delightful-clis)

### Real-World Examples

#### Git Command Structure (Verb-Noun)
```bash
git add <file>              # action + target
git commit -m "msg"         # action + message
git push origin main        # action + destination + branch
git log --oneline          # action + flag
```

#### Cargo Command Structure (Verb-Noun)
```bash
cargo new <project>         # action + target
cargo build --release       # action + flag
cargo test --verbose        # action + flag
cargo run -- <args>         # action + delimiter + args
```

#### Kubectl Command Structure (Verb-Noun)
```bash
kubectl get pods            # action + resource type
kubectl apply -f file.yaml  # action + flag + file
kubectl delete pod my-pod   # action + resource type + name
kubectl describe node node1 # action + resource type + name
```

#### Docker Command Structure (Noun-Verb)
```bash
docker container run        # resource + action
docker container ls         # resource + action
docker image build .        # resource + action + target
docker network create mynet # resource + action + name
```

---

## 2. Subcommand Organization Patterns

### Pattern 1: Category-Based Organization (Verb-Noun)

Organize subcommands by functional category while maintaining verb-first ordering:

**Cargo Example - Four Main Categories:**

1. **Build Commands** (compilation and execution)
   - `cargo build` — Compile a package
   - `cargo check` — Check for errors without building
   - `cargo run` — Compile and run
   - `cargo test` — Execute tests
   - `cargo bench` — Execute benchmarks
   - `cargo clean` — Remove artifacts

2. **Manifest Commands** (dependency management)
   - `cargo add` — Add dependencies
   - `cargo remove` — Remove dependencies
   - `cargo update` — Update dependencies
   - `cargo tree` — Show dependency tree
   - `cargo metadata` — Show project metadata

3. **Package Commands** (distribution)
   - `cargo new` — Create new project
   - `cargo init` — Initialize in existing directory
   - `cargo install` — Install binary
   - `cargo search` — Search crates.io

4. **Publishing Commands** (registry operations)
   - `cargo publish` — Publish to crates.io
   - `cargo login` — Authenticate with registry
   - `cargo owner` — Manage crate ownership
   - `cargo yank` — Prevent version from being used

**Benefits:**
- Commands are naturally discoverable by category
- Easy to remember where related functionality lives
- Scales well as command count grows

### Pattern 2: Resource-Based Organization (Noun-Verb)

Group commands by the resource type they operate on:

**Docker Example:**
```bash
docker container ls          # List containers
docker container run         # Run a container
docker container exec        # Execute in container
docker container stop        # Stop a container

docker image ls              # List images
docker image build           # Build an image
docker image push            # Push an image
docker image pull            # Pull an image

docker network create        # Create network
docker network connect       # Connect container to network
docker network inspect       # Inspect network
```

**Benefits:**
- Intuitive for resource-centric operations
- Related operations are grouped together
- New resource types integrate cleanly

### Pattern 3: Verb-Based Organization (kubectl-like)

Commands grouped by verb with multiple resource types:

**Kubectl Example:**
```bash
kubectl get pods             # Get pods
kubectl get nodes            # Get nodes
kubectl get services         # Get services

kubectl apply -f pod.yaml    # Apply pod definition
kubectl apply -f svc.yaml    # Apply service definition

kubectl delete pod my-pod    # Delete pod
kubectl delete service my-svc # Delete service

kubectl describe pod my-pod  # Describe pod
kubectl describe node node1  # Describe node
```

**Benefits:**
- Very discoverable: `kubectl <verb> <TAB>`
- Minimal command duplication
- Works well with many resource types

**Sources:**
- [Cargo Book - Commands](https://doc.rust-lang.org/cargo/commands/cargo.html)
- [Kubectl Cheat Sheets](https://spacelift.io/blog/kubernetes-cheat-sheet)
- [Docker Container Reference](https://docs.docker.com/reference/cli/docker/container/run/)

---

## 3. Flag Naming and Consistency

### Core Principles

1. **Provide Both Short and Long Forms**
   - Short: Single letter for frequently used flags (`-h`, `-v`, `-f`, `-q`)
   - Long: Full descriptive kebab-case (`--help`, `--verbose`, `--force`, `--quiet`)

2. **Reserve Single-Letter Flags Carefully**
   - Prevent namespace pollution by restricting to commonly expected abbreviations
   - Standard conventions to follow:
     - `-h` / `--help` — Show help
     - `-v` / `--verbose` — Verbose output
     - `-q` / `--quiet` — Suppress output
     - `-f` / `--force` — Force operation
     - `-d` / `--detach` — Run in background
     - `-i` / `--interactive` — Interactive mode
     - `-t` / `--tty` — Allocate TTY
     - `-a` / `--all` — Operate on all items
     - `-n` / `--name` — Specify name
     - `-m` / `--message` — Specify message

3. **Use Kebab-Case (Not Snake_Case or camelCase)**
   ```
   ✅ --example-flag
   ✅ --help-text
   ❌ --example_flag
   ❌ --exampleFlag
   ```

4. **Flags Over Arguments**
   - Flags are clearer than positional arguments
   - Users don't need to memorize argument order
   - Better for autocomplete and discovery

**Example: Heroku Pattern**

Before (confusing):
```bash
heroku fork destapp -a sourceapp
```

After (clear):
```bash
heroku fork --from sourceapp --to destapp
```

### Standard Flag Patterns

#### Display/Output Flags
```bash
--help, -h              # Show help information
--version, -V           # Show version
--verbose, -v           # Verbose output (stackable: -vv for more)
--quiet, -q             # Suppress non-essential output
--color [auto|always|never]  # Control color output
--json                  # Output as JSON
--terse                 # Condensed output format
```

#### Execution Control Flags
```bash
--dry-run               # Preview changes without executing
--force, -f             # Force operation
--interactive, -i       # Interactive mode
--confirm               # Require confirmation
--skip                  # Skip specific operations
```

#### Input/Output Flags
```bash
-f, --file <path>       # Input/output file path
--stdin                 # Read from standard input
--output, -o <path>     # Output file path
```

#### Resource/Environment Flags
```bash
--all, -a               # Operate on all items
-n, --name              # Specify resource name
--label, -l             # Add labels/tags
--config                # Configuration file
--env, -e               # Environment variable
```

#### Docker-Specific Pattern
Related flags share prefixes for discoverability:
```bash
--cap-add <capability>      # Add capability
--cap-drop <capability>     # Drop capability
--health-cmd <cmd>          # Health check command
--health-interval <duration> # Health check interval
--health-timeout <duration>  # Health check timeout
--cpu-shares <shares>       # CPU share weight
--cpus <number>             # CPU limit
--memory, -m <bytes>        # Memory limit
--memory-swap <bytes>       # Memory+swap limit
```

### Flag Organization Strategy

1. **Group related flags** using prefixes for discoverability
2. **Most common flags first** in help text
3. **Provide sensible defaults** to minimize required flags
4. **Support environment variables** as alternative to flags
5. **Document flag precedence** (CLI > env var > config file)

**Sources:**
- [Command Line Interface Guidelines - clig.dev](https://clig.dev/)
- [Heroku CLI Style Guide](https://devcenter.heroku.com/articles/cli-style-guide)
- [Rust Clap Documentation](https://docs.rs/clap/latest/clap/)
- [Docker Container Run Reference](https://docs.docker.com/reference/cli/docker/container/run/)

---

## 4. Help Text Structure and Clarity

### The Help Command Lifecycle

Users approach help at different times with different needs:

**New Users:**
- Want to understand what the tool does
- Need examples to get started
- Should get guidance in 3-5 seconds

**Experienced Users:**
- Want to remember flag details
- Need to discover advanced options
- Want command reference format

### Help Structure Hierarchy

#### Level 1: Brief Help (Default When No Arguments)
Show only essential information:
```
DESCRIPTION (1-2 sentences)
USAGE (command syntax)
EXAMPLES (1-2 practical examples)
FLAGS (most common 3-5 flags only)
See 'help' for more information
```

**Example:**
```
Usage: git push [OPTIONS] [REPOSITORY] [REFSPEC...]

Push commits to a remote repository

EXAMPLES:
  git push                    Push to default remote
  git push origin main        Push main branch to origin

FLAGS:
  -u, --set-upstream          Set upstream branch
  -f, --force                 Force push
  --all                       Push all branches
  -h, --help                  Show full help

Run 'git help push' for more information
```

#### Level 2: Full Help (-h / --help)
Comprehensive reference with all details:

**Standard Sections:**
```
NAME
  Brief one-line description

USAGE
  Usage pattern with all flags

DESCRIPTION
  Detailed explanation
  Multiple paragraphs allowed
  Explain behavior and edge cases

OPTIONS/FLAGS
  --flag-name, -f
    Description of what this flag does.
    Include default value if applicable.
    Include examples if behavior is non-obvious.

EXAMPLES
  $ command example 1
    Explanation of what this does

  $ command example 2 --flag
    Explanation of what this does

SEE ALSO
  Related commands or documentation

EXIT STATUS
  0   Success
  1   General error
  2   Misuse of command syntax
```

#### Level 3: Deep Dive
Separate documentation pages:
```bash
$ git help push              # Opens man page
$ cargo help build           # Shows detailed build docs
$ kubectl explain pod        # Explains resource type
$ docker help container run  # Full reference documentation
```

### Help Text Best Practices

#### 1. Lead with Examples
Users typically reference examples over descriptive text. Put them early.

**Bad:**
```
--verbose flag increases output verbosity by suppressing filtering
operations that would normally reduce output. When used with the
--quiet flag, --verbose takes precedence.
```

**Good:**
```
--verbose, -v
  Increase output verbosity. Show all operations.
  Examples:
    -v       Standard verbose output
    -vv      Extra verbose (debug info)
    -vvv     Extremely verbose (trace all calls)
```

#### 2. Keep Descriptions Concise
- One sentence per flag in brief help
- 2-3 sentences maximum in full help
- Save paragraphs for DESCRIPTION section

#### 3. Support Skim Reading
- Use visual hierarchy
- Bold key terms
- Break long descriptions into bullet points
- Keep paragraphs to 50-75 characters

#### 4. Be Action-Oriented
Start descriptions with what the flag does:

**Bad:**
```
The force flag causes the operation to proceed even in the face of errors
that would normally prevent it.
```

**Good:**
```
Force operation to proceed despite errors that would normally prevent it.
```

#### 5. Include Defaults and Constraints
```bash
--timeout <seconds>
  Maximum time to wait for operation. Default: 30 seconds.
  Must be between 1 and 3600.

--output, -o <format>
  Output format. Options: json, table, yaml. Default: table.

--workers <count>
  Number of parallel workers. Default: Number of CPU cores.
```

#### 6. Provide Next Steps
Point users toward related commands or next logical action:

```
FLAGS:
  ... flags ...

See 'git help workflows' for information on common workflows
Run 'git status' to see current state
```

### Example: Well-Structured Help

**Brief (default output):**
```
$ cargo build
error: the following required arguments were not provided:
  <WORKSPACE>

USAGE:
    cargo build [OPTIONS] [WORKSPACE]

For more information try --help
```

**Full (-h / --help):**
```
Build a project's main crate

USAGE:
    cargo build [OPTIONS] [PACKAGE_NAME]

OPTIONS:
  --release               Optimize for release
  --example <NAME>        Build example instead of library
  -j, --jobs <N>          Number of parallel jobs
  -v, --verbose           Verbose output
  --color <WHEN>          When to use colored output [default: auto]
  -h, --help              Print help information

EXAMPLES:
  cargo build             Build project in dev mode
  cargo build --release   Build optimized release binary

See 'cargo help build' for more information
```

**Sources:**
- [Command Line Interface Guidelines - clig.dev](https://clig.dev/)
- [10 Design Principles for Delightful CLIs - Atlassian](https://www.atlassian.com/blog/it-teams/10-design-principles-for-delightful-clis)
- [Better CLI - Help Pages](https://bettercli.org/)

---

## 5. Error Message Patterns

### Core Principle: Actionable Feedback

Error messages serve two purposes:
1. Tell users what went wrong
2. Guide them toward fixing it

The best errors catch expected errors early and rewrite them for human comprehension.

### Error Message Structure

#### Basic Format
```
<error-type>: <what-went-wrong>

<suggestion-for-fixing-it>

Run '<command> --help' for more information
```

**Example:**
```
error: could not open file 'config.yaml'

The configuration file 'config.yaml' does not exist. Make sure it's in the
current directory or specify its path with --config <path>

Run 'myapp --help' for more information
```

### Multi-Level Error Messaging

#### Level 1: Quick Error (Single Line)
For automation/scripting contexts where verbosity isn't helpful:

```bash
$ cargo build --invalid-flag
error: unexpected argument '--invalid-flag' found

tip: to pass '--invalid-flag' and value to your program, separate with '--'
```

#### Level 2: Context Error (3-5 Lines)
Most common scenario - provide problem + one suggestion:

```bash
$ docker run --memory 1g invalid-image
Error: image 'invalid-image' not found

Did you mean: 'nginx' or 'python:3.11'?
Use 'docker search <term>' to find available images
```

#### Level 3: Detailed Error (with examples)
Complex scenarios with multiple potential solutions:

```bash
$ kubectl apply -f deployment.yaml
error: resource validation failed

The Deployment 'myapp' has an invalid selector:
  matchLabels.app: "webserver"

The Pod template references different labels:
  app: "web-server"

Labels must match exactly. Choose one:
  1. Change matchLabels to: app: "web-server"
  2. Change Pod template label to: app: "webserver"

Example:
  apiVersion: apps/v1
  kind: Deployment
  metadata:
    name: myapp
  spec:
    selector:
      matchLabels:
        app: "webserver"      # Must match Pod template
    template:
      metadata:
        labels:
          app: "webserver"    # Must match selector

See 'kubectl explain deployment' for more
```

### Error Types and Patterns

#### 1. Validation Errors
**Pattern:** Missing required argument, invalid argument, constraint violation

```bash
# Missing required argument
$ git commit
error: no changes added to commit

Hint: Did you forget to use 'git add'?
Hint: See 'git commit --help'

# Invalid value
$ cargo build --jobs 0
error: invalid value '0' for '--jobs': must be at least 1

# Type mismatch
$ docker ps --filter status=wrong
Error: no such filter: 'status=wrong'

Available filters:
  ancestor=<image>
  id=<container_id>
  name=<pattern>
  status=running|paused|exited|created

See 'docker ps --help' for details
```

#### 2. Not Found Errors
**Pattern:** File, command, or resource doesn't exist

```bash
$ git show nonexistent-file
error: path 'nonexistent-file' does not exist in 'HEAD'

Did you mean one of these?
  existing-file
  other-file

See 'git show --help' for available options
```

**With suggestions:**
```bash
$ kubectl get pods my-app
Error: pod 'my-app' not found in namespace 'default'

Available pods:
  my-app-abc123 (Running)
  my-app-def456 (Pending)
  other-pod-xyz (Running)

Namespace 'default' was used. Specify a different namespace with:
  kubectl get pods -n <namespace>
```

#### 3. Permission/Authentication Errors
**Pattern:** Access denied, insufficient privileges

```bash
$ docker ps
permission denied while trying to connect to the Docker daemon

You need to add your user to the 'docker' group:
  sudo usermod -aG docker $USER
  newgrp docker

Then try again:
  docker ps
```

**With context:**
```bash
$ kubectl apply -f deployment.yaml
Error: operation create on deployments.apps is forbidden:
User 'user@example.com' cannot create deployments in namespace 'prod'

Current permissions in 'prod':
  list pods
  get pods
  view logs

Contact your administrator to request 'create' permission on deployments
```

#### 4. Configuration/State Errors
**Pattern:** Invalid state, conflicts, misconfigurations

```bash
$ git merge conflicting-branch
CONFLICT (content): Merge conflict in README.md
CONFLICT (add/add): Merge conflict in src/main.rs

Fix conflicts and complete the merge:
  1. Edit conflicting files
  2. Run: git add .
  3. Run: git commit

Or abort the merge:
  git merge --abort

See 'git merge --help' for more help
```

**With helpful context:**
```bash
$ cargo build
error: failed to resolve 'serde' dependency

The crate 'serde' is not found. Do one of:

1. Add it to Cargo.toml:
   cargo add serde

2. Check spelling (did you mean 'serde_json'?)

3. Ensure you're in a Rust project with Cargo.toml

See 'cargo add --help' for options
```

#### 5. Network/External Service Errors
**Pattern:** Temporary failures with retry guidance

```bash
$ cargo publish
error: failed to upload to crates.io

Connection timeout (30s). This may be temporary.

Retry the operation:
  cargo publish

Or check status:
  https://status.crates.io/

For more help:
  cargo publish --help
```

### Error Message Best Practices

#### 1. Prioritize Actionability
**Bad:** "Invalid argument"
**Good:** "Number of workers must be between 1 and 64, got 128"

#### 2. Avoid Jargon
**Bad:** "NullPointerException in sync context"
**Good:** "File not found. Make sure the file exists and the path is correct"

#### 3. Use Color Sparingly
- Red for errors/critical
- Yellow for warnings
- Green for success
- White/default for info

Reserve color for highlighting, not for all output.

#### 4. Signal-to-Noise Ratio
Group similar errors rather than outputting hundreds of lines:

**Bad:**
```
error: file not found 'config.yaml'
error: file not found 'secrets.json'
error: file not found 'data.csv'
error: file not found 'credentials.txt'
... (100 more lines)
```

**Good:**
```
error: 4 configuration files not found:
  - config.yaml
  - secrets.json
  - data.csv
  - credentials.txt

Create these files or specify alternate paths with --config flags
```

#### 5. Suggest Next Steps
Always provide at least one clear action:

```bash
error: port 8080 is already in use

Free the port:
  lsof -i :8080
  kill -9 <PID>

Or use a different port:
  myapp --port 8081
```

#### 6. Exit Codes Matter
```bash
0   - Success
1   - General error
2   - Misuse of shell command (argument errors)
126 - Permission denied
127 - Command not found
128 - Invalid argument to exit
```

**Example Implementation:**
```bash
$ myapp invalid-command
error: unknown command 'invalid-command'

Available commands:
  start, stop, restart, status

See 'myapp --help' for more information
$ echo $?
2  # Standard "command syntax misuse" code
```

### Error Message Examples by Tool

#### Git
- Clear about what went wrong
- Suggests next action ("Did you mean...")
- Shows hints for fixes
- Provides relevant help command

#### Docker
- Groups related errors
- Suggests similar images/resources
- Shows available options
- Links to documentation

#### Kubectl
- Explains constraint violations
- Shows current state
- Suggests fixes with examples
- Indicates permissions required

#### Cargo
- Suggests dependency additions/fixes
- Explains compilation errors
- Shows available options
- Provides documentation links

**Sources:**
- [Command Line Interface Guidelines - clig.dev](https://clig.dev/)
- [10 Design Principles for Delightful CLIs - Atlassian](https://www.atlassian.com/blog/it-teams/10-design-principles-for-delightful-clis)
- [Better CLI - Messaging Patterns](https://bettercli.org/)
- Git, Docker, Cargo, Kubectl documentation

---

## Key Takeaways for Jin CLI Design

Based on this research, here are the critical principles for the Jin debugger CLI:

### 1. Naming
- **Choose verb-noun ordering** (e.g., `jin run`, `jin debug`, `jin attach`)
- **Use lowercase with hyphens** for multi-word commands/flags
- **Be consistent** across all commands

### 2. Organization
- **Group by functional category** (e.g., build commands, debugging commands, inspection commands)
- **Use prefixes for related flags** (e.g., `--breakpoint-*`, `--watch-*`)
- **Keep command count manageable** - fewer, more powerful commands beat many specialized ones

### 3. Flags
- **Short forms for common flags:** `-h`, `-v`, `-f`, `-q`
- **Full kebab-case forms:** `--help`, `--verbose`, `--force`, `--quiet`
- **Group related flags** using common prefixes
- **Flags over arguments** for clarity

### 4. Help
- **Concise default help** (3-5 seconds to understand)
- **Full `--help` reference** with all details
- **Lead with examples** users can copy and adapt
- **Show next steps** to guide users forward

### 5. Errors
- **Catch expected errors early**
- **Provide actionable fixes**, not just "error occurred"
- **Suggest next actions** or related commands
- **Use proper exit codes** for automation

---

## Research Sources

### Official Guides
- [Command Line Interface Guidelines - clig.dev](https://clig.dev/)
- [Heroku CLI Style Guide](https://devcenter.heroku.com/articles/cli-style-guide)
- [Better CLI - Design Guidelines](https://bettercli.org/)

### Tool Documentation
- [Git Architecture & Design](https://aosabook.org/en/v2/git.html)
- [Cargo Book - Commands](https://doc.rust-lang.org/cargo/commands/cargo.html)
- [Docker Container Reference](https://docs.docker.com/reference/cli/docker/container/run/)
- [Kubernetes Kubectl Reference](https://kubernetes.io/docs/reference/kubectl/)

### Specialized Resources
- [10 Design Principles for Delightful CLIs - Atlassian](https://www.atlassian.com/blog/it-teams/10-design-principles-for-delightful-clis)
- [CLI Design Best Practices - Cody A. Ray](https://codyaray.com/2020/07/cli-design-best-practices)
- [Rust Clap Framework Documentation](https://docs.rs/clap/latest/clap/)
- [CLI UX Best Practices - Evil Martians](https://evilmartians.com/chronicles/cli-ux-best-practices-3-patterns-for-improving-progress-displays)
- [Elevate Developer Experiences with CLI Design - Thoughtworks](https://www.thoughtworks.com/en-us/insights/blog/engineering-effectiveness/elevate-developer-experiences-with-cli-design-guidelines)

---

## Document Information

**Created:** December 27, 2025
**Location:** `/home/dustin/projects/jin/plan/P4M1T1/research/cli_design_patterns.md`
**Status:** Comprehensive research complete with examples from Git, Docker, Cargo, Kubectl
