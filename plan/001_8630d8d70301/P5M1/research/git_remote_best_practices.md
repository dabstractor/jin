# Git Remote Operations Best Practices and Patterns

## Table of Contents

1. [Fetch vs Pull](#fetch-vs-pull)
2. [Remote Authentication](#remote-authentication)
3. [Fast-Forward and Non-Fast-Forward Operations](#fast-forward-operations)
4. [Conflict Resolution Strategies](#conflict-resolution)
5. [Progress Reporting Patterns](#progress-reporting)
6. [Network Error Handling](#error-handling)
7. [Bare Repository Operations](#bare-repositories)
8. [Distributed Workflow Patterns](#workflows)
9. [Common Error Scenarios](#error-scenarios)

---

## 1. Fetch vs Pull {#fetch-vs-pull}

### Core Differences

**Git Fetch:**
- Downloads commits, files, and refs from remote repository into your local repo
- Isolates fetched content from existing local content
- Has no effect on your local development work or working directory
- Creates/updates remote-tracking branches (e.g., `origin/main`)
- Safe operation - never causes merge conflicts directly

**Git Pull:**
- Combines `git fetch` + `git merge` (or rebase, depending on configuration)
- Automatically merges remote changes into your current branch
- Updates your working directory to reflect merged content
- Simpler but less safe - can cause unexpected conflicts

### When to Use Each

#### Use `git fetch` when:
- Working in a shared codebase and want to avoid unexpected conflicts
- Expecting conflicts and want to resolve them manually at leisure
- Want to review remote changes before integrating them
- Need to work offline after fetching - no network required for merge
- Managing multiple remote branches and want fine-grained control

**Example workflow:**
```bash
# Review changes before merging
git fetch origin
git log origin/main..main    # See what's ahead locally
git log main..origin/main    # See what's new remotely
git diff main origin/main    # Inspect the differences
git merge origin/main        # Merge when ready
```

#### Use `git pull` when:
- Working solo or in a fast-paced project requiring immediate updates
- Current branch is configured to track a remote branch
- Trust the automatic integration strategy (merge or rebase)
- Simplicity is more important than fine-grained control

**Configuration for safe pull:**
```bash
# Choose merge strategy (creates merge commits)
git config --global pull.rebase "false"

# Choose rebase strategy (linear history)
git config --global pull.rebase "true"

# Require explicit strategy declaration
git config --global pull.ff "only"  # Only allow fast-forward pulls
```

### Safety Hierarchy
1. **Safest:** `git fetch` + manual inspection + `git merge`
2. **Safe:** `git fetch` + `git merge` (explicit)
3. **Moderate:** `git pull` with configured strategy
4. **Least safe:** `git pull` without configuration (behavior varies)

---

## 2. Remote Authentication {#remote-authentication}

### SSH Keys Best Practices

#### Key Generation and Algorithms

**Supported Key Types (Modern):**
- **ED25519** (Recommended): Most secure, smallest key size
  ```bash
  ssh-keygen -t ed25519 -C "your_email@example.com"
  ```

- **RSA** (4096-bit or larger): Widely compatible but larger
  ```bash
  ssh-keygen -t rsa -b 4096 -C "your_email@example.com"
  ```

**Deprecated/Unsupported:**
- DSA (ssh-dss) - GitHub removed support March 15, 2022
- RSA keys generated after November 2, 2021 must use SHA-2 signature algorithm

#### Passphrase Protection

**Best Practice: Always use a passphrase**
```bash
# When creating key, enter a strong passphrase
ssh-keygen -t ed25519 -C "your_email@example.com"
# When prompted: Enter passphrase []:  [enter passphrase]
```

Benefits:
- Extra security layer if computer is compromised
- Prevents attackers from using stolen keys
- Required for each SSH operation (cached by ssh-agent)

#### SSH Agent Configuration

Store passphrase in ssh-agent to avoid typing it repeatedly:

```bash
# Start ssh-agent in background
eval "$(ssh-agent -s)"

# Add key with passphrase
ssh-add ~/.ssh/id_ed25519

# Verify key is loaded
ssh-add -l
```

Persist across sessions (macOS):
```bash
# Add to ~/.ssh/config
Host github.com
  AddKeysToAgent yes
  UseKeychain yes
  IdentityFile ~/.ssh/id_ed25519
```

#### Key Management Best Practices

1. **Use separate keys for different purposes**
   - Authentication key: For git operations
   - Signing key: For commit signing (separate for integrity)
   - Deployment key: For CI/CD (restricted to specific repos)

2. **Set expiration dates**
   ```bash
   # When adding SSH key to GitHub/GitLab, set expiration
   # Limits damage if key is compromised
   # Recommended: 1 year for personal keys, 90 days for CI/CD
   ```

3. **Protect private keys**
   ```bash
   # Permissions should be restrictive
   chmod 700 ~/.ssh
   chmod 600 ~/.ssh/id_ed25519
   chmod 644 ~/.ssh/id_ed25519.pub
   ```

4. **Regularly audit credentials**
   - Review SSH keys on hosting platforms monthly
   - Delete unused or suspicious keys immediately
   - Check deploy keys for unused integrations
   - Monitor SSH key usage logs

5. **Never commit private keys**
   ```bash
   # Add to .gitignore
   ~/.ssh/id_*
   !~/.ssh/id_*.pub

   # Configure global exclusions
   git config --global core.excludesFile ~/.gitignore_global
   ```

### Alternative Authentication Methods

#### HTTPS with Personal Access Tokens

```bash
# Generate token on GitHub/GitLab (repo + read:user scopes minimum)
git clone https://github.com/user/repo.git
# When prompted for password, use personal access token

# Store credentials (macOS)
git config credential.helper osxkeychain

# Store credentials (Linux)
git config credential.helper store
```

#### SSH Key Signing Certificates

For enterprise environments using SSH certificate authority:
```bash
# CA can issue short-lived certificates
# More secure than long-lived keys
# Requires enterprise infrastructure
```

### Server-Side Authentication Patterns

1. **Public key authentication** (SSH): Most common, secure
2. **OAuth 2.0**: For web-based platforms
3. **2FA with backup codes**: Additional security layer
4. **Hardware security keys**: Highest security (FIDO2/U2F)

---

## 3. Fast-Forward and Non-Fast-Forward Operations {#fast-forward-operations}

### Understanding Fast-Forward

**Fast-Forward Merge:**
- Occurs when current branch has no divergent commits from target branch
- Simply moves the branch pointer forward to the target
- Linear history preserved
- No merge commit created

```
Before merge (linear path):
A - B - C (main)
         └─ D - E (feature)

After fast-forward merge:
A - B - C - D - E (main, merged feature)
```

**Non-Fast-Forward (3-Way) Merge:**
- Occurs when branches have diverged (both have new commits)
- Git must combine histories using a merge algorithm
- Creates a merge commit
- Preserves complete history of both branches

```
Before merge (divergent):
        C - D (main)
       /
A - B
       \
        E - F (feature)

After non-fast-forward merge:
        C - D
       /     \
A - B         G (merge commit)
       \     /
        E - F
```

### Configuration Best Practices

#### Default Merge Behavior

```bash
# Allow both fast-forward and non-fast-forward
git config --global merge.ff "true"  # Default behavior

# Only allow fast-forward (fails if non-fast-forward needed)
git config --global merge.ff "only"  # Strict mode

# Never fast-forward (always create merge commit)
git config --global merge.ff "false" # Traditional behavior
```

#### Per-Repository Configuration

```bash
# Enforce fast-forward only on main branch
git config branch.main.mergeoptions "--ff-only"

# Require merge commit on develop
git config branch.develop.mergeoptions "--no-ff"
```

### Push Strategies

#### Safe Force Push with Lease

```bash
# AVOID: Dangerous, can overwrite others' work
git push --force

# SAFE: Only force if remote hasn't changed since last fetch
git push --force-with-lease origin feature

# SAFER: Specify exact expected state
git push --force-with-lease=origin/feature:expected_hash origin feature

# SAFEST: Verify integration before forcing
git push --force-with-lease --force-if-includes origin feature
```

**When to use `--force-with-lease`:**
- Squashing commits before submission
- Reordering commits in PR review
- Fixing commit messages
- Only on feature branches, never on shared/main branches

#### Atomic Transactions

```bash
# All refs update or none (prevents partial failures)
git push --atomic origin main feature develop

# Useful for:
# - Multi-ref operations
# - CI/CD deployments
# - Maintaining consistency across branches
```

### Best Practices for Different Scenarios

| Scenario | Recommendation | Rationale |
|----------|----------------|-----------|
| Small features on main | Fast-forward merge | Clean linear history |
| Long-running features | Non-fast-forward merge | Preserve feature history |
| Quick bug fixes | Fast-forward merge | Less merge clutter |
| Release branches | Non-fast-forward (--no-ff) | Mark integration points |
| Git rebase before PR | Fast-forward merge | Linear history without merge commit |

---

## 4. Conflict Resolution Strategies {#conflict-resolution}

### Merge Conflicts

#### Prevention Strategies

```bash
# Keep branches short-lived (< 1 week)
# Frequent synchronization with main
git fetch origin
git rebase origin/main  # Update feature branch

# Small, focused commits reduce conflicts
git add --patch  # Add changes selectively

# Communicate with team about overlapping work
```

#### Resolution Workflow

```bash
# Start merge
git merge origin/feature

# If conflicts occur, Git shows status
git status

# Resolve conflicts in files
# Edit files to remove conflict markers:
# <<<<<<< HEAD
# Current branch code
# =======
# Incoming branch code
# >>>>>>> origin/feature

# Mark as resolved
git add <resolved-files>

# Complete merge
git commit -m "Merge branch 'feature' into main"
```

#### Merge Conflict Options

```bash
# Continue with manual resolution
git add <resolved-files>
git commit

# Abort the merge entirely
git merge --abort

# Accept their changes (use in rare cases)
git checkout --theirs <file>
git add <file>

# Keep our changes
git checkout --ours <file>
git add <file>
```

### Rebase Conflicts

**Golden Rule:** Never rebase on public/shared branches

#### Rebase Conflict Handling

```bash
# Start rebase
git rebase origin/main

# If conflicts occur during rebase
git status  # Shows conflicted files

# Resolve conflicts in editor
# Then mark resolved
git add <resolved-files>

# Continue rebase
git rebase --continue

# Or skip this commit entirely
git rebase --skip

# Or abort rebase
git rebase --abort
```

#### Rebase Options for Conflict Resolution

```bash
# Interactive rebase with conflict resolution
git rebase -i origin/main

# Rebase with automatic merge resolution
git rebase -X ours origin/main        # Prefer current branch
git rebase -X theirs origin/main      # Prefer incoming branch
git rebase -X recursive origin/main   # Default (3-way merge)

# Abort and retry with different strategy
git rebase --abort
git rebase -X theirs origin/main
```

### Advanced: Git Rerere (Reuse Recorded Resolution)

Enable Git to remember and auto-apply previous conflict resolutions:

```bash
# Enable rerere globally
git config --global rerere.enabled true

# Rerere automatically:
# 1. Records your conflict resolutions
# 2. Automatically applies same resolution next time
# 3. Requires your confirmation

# Useful for:
# - Rebasing topic branches repeatedly
# - Long-running features with frequent merges
# - Maintaining branches with stable conflicts

# View recorded resolutions
ls -la .git/rr-cache/

# Manually apply recorded resolutions
git rerere resolve
```

### Pull with Rebase Conflicts

```bash
# Pull using rebase instead of merge
git pull --rebase origin main

# If conflicts occur, resolve them
# Then continue rebase
git rebase --continue

# Configure pull to always rebase
git config --global pull.rebase "true"
```

### Distributed Collaboration Conflict Strategy

| Situation | Strategy | Reason |
|-----------|----------|--------|
| Feature merging to main | Merge (no --ff) | Preserve feature history |
| Updating feature branch | Rebase on main | Clean linear feature history |
| Public branch updates | Merge only | Never rebase public branches |
| Multiple conflicts | Use Rerere | Automate repetitive resolutions |
| Uncertain resolution | Consult team | Prevent logic errors |

---

## 5. Progress Reporting Patterns {#progress-reporting}

### Git Fetch Progress Reporting

#### Command Options

```bash
# Force progress output (even without terminal)
git fetch --progress origin

# Suppress progress (quiet mode)
git fetch --quiet origin
git fetch -q origin

# Verbose output with all refs
git fetch --verbose origin
git fetch -v origin

# Default behavior: Progress shown if stderr is a terminal
git fetch origin
```

#### Progress Information Includes

- Number of objects being transferred
- Compression percentage
- Network throughput (KB/s)
- Overall progress percentage
- Estimated time remaining

#### Example Progress Output

```
remote: Counting objects:  30%
remote: Compressing objects:  50%
Receiving objects:  45% (12345/27890)
Unpacking objects:  60%
Resolving deltas:  75%
```

### Git Push Progress Reporting

#### Command Options

```bash
# Force progress output
git push --progress origin main

# Suppress all output
git push --quiet origin main
git push -q origin main

# Verbose mode with detailed information
git push --verbose origin main
git push -v origin main

# Test without actually pushing
git push --dry-run --progress origin main
```

#### Parsing Progress for Automation

```bash
# Machine-readable format (JSON-like for newer Git)
GIT_TRACE=1 git push origin main 2>&1 | grep "Total\|Packfile"

# Extract transfer statistics
git push --progress origin main 2>&1 | grep -E "^Pushing|objects|bytes"

# Monitor with custom script
git fetch --progress origin 2>&1 | \
  while IFS= read -r line; do
    echo "$(date): $line"
  done | tee fetch.log
```

### Network Sideband Communication

Git uses sideband multiplexing for progress during large transfers:

```
Sideband 1: Packfile data
Sideband 2: Progress information
Sideband 3: Error messages
```

#### Server-Side Options

```bash
# Pass options to remote git command
git fetch --server-option=verbose origin

# Multiple options supported
git fetch \
  --server-option=verbose \
  --server-option=progress \
  origin

# Configuration-based approach
git config remote.origin.serverOption verbose
git config --add remote.origin.serverOption progress
```

### Long Operation Progress Monitoring

For scripts dealing with large repositories:

```bash
#!/bin/bash

# Monitor fetch progress with timeout
timeout 300 git fetch --progress origin 2>&1 | while read line; do
  timestamp=$(date '+%Y-%m-%d %H:%M:%S')
  echo "[$timestamp] $line"

  # Parse for specific milestones
  if [[ $line =~ Unpacking ]]; then
    notify_user "Unpacking objects..."
  fi
  if [[ $line =~ Resolving ]]; then
    notify_user "Resolving deltas..."
  fi
done

# Capture final status
if [ $? -eq 0 ]; then
  echo "Fetch completed successfully"
else
  echo "Fetch failed or timed out"
  exit 1
fi
```

---

## 6. Network Error Handling {#error-handling}

### Common Network Errors

#### Connection Timeout

```
fatal: unable to access 'https://github.com/...': Operation timed out
```

**Causes:**
- Network latency or poor connectivity
- Firewall blocking SSH/HTTPS ports
- Server overload
- DNS resolution issues

**Solutions:**

```bash
# Increase timeout for HTTPS
git config --global http.timeout 300  # 5 minutes

# SSH connection timeout
git config --global core.sshCommand "ssh -o ConnectTimeout=30"

# Persistent connection settings
git config --global ssh.variant ssh  # Use OpenSSH

# Add to ~/.ssh/config for persistent settings
Host github.com
  HostName github.com
  User git
  IdentityFile ~/.ssh/id_ed25519
  ConnectTimeout 30
  StrictHostKeyChecking accept-new
```

#### Host Key Verification Failed

```
fatal: Could not read from remote repository. Please make sure you have
the correct access rights and the repository exists.
```

**Solutions:**

```bash
# Add host to known_hosts
ssh-keyscan github.com >> ~/.ssh/known_hosts 2>/dev/null

# For automation (CI/CD), disable strict checking
git config --global ssh.strictHostKeyChecking no

# Better: Accept new keys only once
git config --global ssh.strictHostKeyChecking accept-new
```

#### Authentication Failed

```
fatal: Authentication failed for 'https://github.com/...'
```

**Solutions:**

```bash
# Update credentials (HTTPS)
git config --global credential.helper "store --file ~/.git-credentials"
git credential approve  # Then enter new password

# Reset stored credentials
git credential reject
# Re-enter at next git operation

# For SSH, verify key is loaded
ssh-add -l
ssh-add ~/.ssh/id_ed25519
```

### Retry Strategies

#### Don't Retry Automatically

**Why retrying long operations is problematic:**
- Server might be overloaded (retries worsen situation)
- Slow connections won't improve by restarting
- Wastes bandwidth on partial transfers
- Can cause rate limiting

**Better approach:** Fix the underlying issue

#### Proper Retry Logic

```bash
#!/bin/bash

# Exponential backoff retry strategy
retry_fetch() {
  local max_attempts=3
  local timeout=30
  local attempt=1

  while [ $attempt -le $max_attempts ]; do
    echo "Fetch attempt $attempt of $max_attempts..."

    if timeout $timeout git fetch --progress origin 2>&1; then
      return 0  # Success
    fi

    if [ $attempt -lt $max_attempts ]; then
      # Exponential backoff: 5s, 10s, 20s
      sleep_time=$((5 * (2 ** (attempt - 1))))
      echo "Fetch failed, retrying in ${sleep_time}s..."
      sleep $sleep_time
    fi

    attempt=$((attempt + 1))
  done

  return 1  # Failed after all retries
}

retry_fetch || exit 1
```

#### Handling Specific Errors

```bash
#!/bin/bash

perform_git_operation() {
  local operation=$1
  local remote=${2:-origin}
  local branch=${3:-main}

  case $operation in
    fetch)
      git fetch --progress "$remote" || handle_fetch_error
      ;;
    push)
      git push "$remote" "$branch" || handle_push_error
      ;;
    pull)
      git pull --progress "$remote" "$branch" || handle_pull_error
      ;;
  esac
}

handle_fetch_error() {
  local exit_code=$?

  case $exit_code in
    124)  # timeout
      echo "Error: Fetch timed out"
      return 1
      ;;
    128)  # authentication
      echo "Error: Authentication failed"
      ssh-add ~/.ssh/id_ed25519
      return 1
      ;;
    *)
      echo "Error: Fetch failed with code $exit_code"
      return 1
      ;;
  esac
}

handle_push_error() {
  local exit_code=$?

  case $exit_code in
    1)  # Non-fast-forward
      echo "Non-fast-forward push. Fetching and rebasing..."
      git fetch origin
      git rebase origin/main
      return 1  # Caller should retry
      ;;
    *)
      echo "Error: Push failed"
      return 1
      ;;
  esac
}
```

### Network Configuration Best Practices

```bash
# Optimize for slow networks
git config --global http.postBuffer 524288000      # 500MB buffer
git config --global core.compression 9             # Maximum compression
git config --global http.lowSpeedLimit 1000        # 1KB/s minimum
git config --global http.lowSpeedTime 60           # Timeout after 60s at slow speed

# IPv4/IPv6 preferences
git -c "url.https://github.com/.insteadOf=git://github.com/" clone ...

# Proxy configuration (if behind corporate firewall)
git config --global http.proxy http://[user[:passwd]@]proxyhost[:port]
git config --global https.proxy https://[user[:passwd]@]proxyhost[:port]

# Disable proxy for specific hosts
git config --global http.sslverify false  # NOT RECOMMENDED - security risk
```

### Monitoring Network Health

```bash
#!/bin/bash

# Monitor git operations with logging
log_file="/var/log/git-operations.log"

git_fetch_monitored() {
  local start_time=$(date +%s)
  echo "[$(date)] Starting fetch..." >> "$log_file"

  if timeout 300 git fetch --progress origin >> "$log_file" 2>&1; then
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    echo "[$(date)] Fetch completed in ${duration}s" >> "$log_file"
    return 0
  else
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    echo "[$(date)] Fetch failed after ${duration}s" >> "$log_file"
    return 1
  fi
}
```

---

## 7. Bare Repository Operations {#bare-repositories}

### What is a Bare Repository?

A bare repository is a Git repository without a working directory. It contains only the contents of the `.git` directory.

```
Regular Repository Structure:
myproject/
├── .git/           (repository metadata)
├── src/
├── README.md
└── ...

Bare Repository Structure:
myproject.git/      (only repository metadata, no working files)
├── HEAD
├── config
├── objects/
├── refs/
└── ...
```

### Creating Bare Repositories

```bash
# Clone with --bare flag
git clone --bare https://github.com/user/repo.git repo.git

# Convert existing repository
cd myrepo.git
git config --bool core.bare true
# Move all .git contents up one level

# Initialize as bare from scratch
git init --bare myrepo.git
```

### Best Practices for Bare Repositories

#### Naming Convention

```bash
# Always use .git suffix for bare repositories
myproject.git
myproject-backup.git

# Prevents confusion with regular repositories
myproject    # Working repository (with .git/ subdirectory)
myproject.git # Bare repository
```

#### Permissions and Ownership

```bash
# Create with restrictive permissions
umask 0027
git init --bare repo.git

# Ownership configuration
chown -R git:git repo.git
chmod -R go-w repo.git

# Shared repository (team access)
chmod -R g+w repo.git
git config -f repo.git/config core.sharedRepository true
```

### Bare Repository Workflow

#### Setting Up Central Repository

```bash
# On server
mkdir -p /var/git/projects
cd /var/git/projects
git init --bare myproject.git

# Configure for team access
cd myproject.git
git config core.sharedRepository true
chown -R :developers .
chmod -R g+w .
```

#### Working with Central Bare Repository

```bash
# Developer 1: Clone from bare repository
git clone git@server:/var/git/projects/myproject.git
cd myproject
# ... make changes ...
git push origin main

# Developer 2: Clone from same bare repository
git clone git@server:/var/git/projects/myproject.git
git pull origin main
# ... make changes ...
git push origin main
```

### Critical Caution: Never Push to Non-Bare Repositories

```bash
# DANGEROUS: Pushing to non-bare repository
git clone https://github.com/user/repo.git
cd repo
# ... make changes ...
git push origin main  # RISK: Can overwrite working directory changes

# SAFE: Use bare repositories as central hub
git init --bare repo.git  # Safe destination
git push repo.git main     # Safe operation
```

**Why this matters:**
- Non-bare repos have a working directory
- Pushing updates the `.git` directory but not working files
- Creates inconsistency between repo state and working files
- Can lose uncommitted work silently

### Bare Repository in CI/CD

```bash
# Post-receive hook for automatic deployment
cat > repo.git/hooks/post-receive << 'EOF'
#!/bin/bash

while read oldrev newrev refname; do
  if [ "$refname" = "refs/heads/main" ]; then
    # Deploy main branch
    GIT_WORK_TREE=/var/www/myapp git checkout -f main
    cd /var/www/myapp
    ./deploy.sh
  fi
done
EOF

chmod +x repo.git/hooks/post-receive
```

---

## 8. Distributed Workflow Patterns {#workflows}

### Centralized Workflow

**Structure:** Single central repository with multiple developers

```
      Central Repository (Bare)
             origin
               |
        _______|_______
       |               |
   Developer 1     Developer 2
```

**Process:**
1. Clone from central repository
2. Make changes and commit locally
3. Fetch remote changes
4. Merge remote branch into local
5. Push to central repository

**Commands:**
```bash
# Initial setup
git clone git@server:/var/git/myproject.git
cd myproject

# Daily workflow
git fetch origin
git merge origin/main
# ... make changes ...
git commit -m "Feature: ..."
git push origin main
```

**Best for:**
- Small teams
- Simple projects
- Teams familiar with centralized VCS
- High level of synchronization needed

**Challenges:**
- All work on main branch (conflicts common)
- No isolation of features
- Requires careful coordination

### Integration Manager Workflow

**Structure:** Distributed with central canonical repository

```
Developer's Fork 1
       |
   (Pull Request)
       |
Central Repository (Canonical)
       |
   Integration Manager
```

**Process:**
1. Developer forks canonical repository
2. Makes changes in own fork (different remote URL)
3. Pushes to personal fork
4. Creates pull request to canonical repository
5. Integration manager reviews and merges

**Commands:**
```bash
# Developer setup
git clone git@github.com:mydeveloper/project.git
cd project
git remote add upstream git@github.com:canonical/project.git

# Feature workflow
git checkout -b feature-A
# ... work ...
git push origin feature-A

# Create pull request via web interface
# After review, integration manager:
git fetch upstream
git merge upstream/main
git push upstream main
```

**Best for:**
- Open source projects
- Hub-based platforms (GitHub, GitLab, Gitea)
- Asynchronous collaboration
- Code review emphasis

**Advantages:**
- Decentralized contributions
- No direct access to main repo needed
- Clear review process
- Easy to reject or request changes

### Dictator and Lieutenants Workflow

**Structure:** Hierarchical with multiple integration levels

```
Regular Developers
       |
   Lieutenants (Maintainers)
       |
  Benevolent Dictator
       |
Reference Repository
```

**Process:**
1. Developers push to lieutenant repositories
2. Lieutenants integrate into their branches
3. Dictator pulls from all lieutenant repos
4. Dictator pushes integrated result to reference repo
5. Developers pull from reference repo

**Best for:**
- Large projects (100+ contributors)
- Hierarchical organizations
- Linux kernel style projects
- Multiple subsystems

**Example:**
```bash
# Developer -> Lieutenant -> Dictator flow
# Developer
git push lieutenant feature-A

# Lieutenant
git pull developer-repo feature-A
git merge feature-A  # Into lieutenant/main
git push reference-repo  # Or to dictator

# Dictator
git pull lieutenant1
git pull lieutenant2
git merge ...
git push public-repo  # Reference repository
```

### Feature Branch Workflow

**Suitable for all team sizes**

**Process:**
1. Create feature branch from main
2. Work on feature in isolation
3. Push feature branch to remote
4. Create merge request/pull request
5. Review and merge to main
6. Delete feature branch

**Commands:**
```bash
# Create and work on feature
git checkout -b feature/user-auth
# ... implement feature ...
git commit -m "feat: Add user authentication"
git push -u origin feature/user-auth

# Create PR/MR via web interface

# After approval, merge to main
git checkout main
git pull origin main
git merge --no-ff feature/user-auth
git push origin main

# Cleanup
git branch -d feature/user-auth
git push origin --delete feature/user-auth
```

### Git Flow Workflow

**Structured with multiple permanent branches**

```
main (release branch)
  ^
  |
release/
  ^
  |
develop (development branch)
  ^
  |
feature/*, bugfix/*
```

**Branches:**
- `main`: Production releases (tagged)
- `develop`: Integration branch for features
- `feature/*`: Individual features
- `release/*`: Release preparation
- `hotfix/*`: Emergency fixes to production

**Commands:**
```bash
# Start feature
git flow feature start user-profile

# Finish feature (merges to develop)
git flow feature finish user-profile

# Release
git flow release start 1.0.0
# ... bump versions ...
git flow release finish 1.0.0

# Hotfix
git flow hotfix start 1.0.1
# ... fix bug ...
git flow hotfix finish 1.0.1
```

---

## 9. Common Error Scenarios and Solutions {#error-scenarios}

### Non-Fast-Forward Push Error

**Error:**
```
error: failed to push some refs to 'git@github.com:user/repo.git'
hint: Updates were rejected because the tip of your current branch is behind
```

**Cause:** Remote branch has commits not in your local branch

**Solution:**

```bash
# Safe approach: fetch and merge
git fetch origin
git merge origin/main
git push origin main

# Alternative: Rebase (use with caution on shared branches)
git fetch origin
git rebase origin/main
git push origin main
```

**Prevention:**
```bash
# Always fetch before working
git fetch origin
git status
```

### Merge Conflict on Pull

**Error:**
```
CONFLICT (content): Merge conflict in file.txt
Automatic merge failed; fix conflicts and then commit the result.
```

**Solution:**

```bash
# 1. See conflicted files
git status

# 2. Edit files to resolve conflicts
# 3. Mark as resolved
git add file.txt

# 4. Complete merge
git commit -m "Merge origin/main: resolve conflicts"

# If you want to abort
git merge --abort
```

### Detached HEAD State

**Cause:** Checked out a commit instead of a branch

**Solution:**

```bash
# Go back to main branch
git checkout main

# Or create new branch from current commit
git checkout -b my-new-branch

# View current state
git status  # Shows "detached HEAD" message
```

### Push Rejected: Repository Locked

**Error:**
```
error: unable to create temporary file: Permission denied
```

**Cause:** Repository permissions issue or another process holding lock

**Solution:**

```bash
# Check and remove lock files (usually not needed)
rm -f .git/index.lock

# Verify permissions
ls -la .git/
chmod 755 .git

# Retry push
git push origin main
```

### SSH Key Not Found

**Error:**
```
git@github.com: Permission denied (publickey).
fatal: Could not read from remote repository.
```

**Solution:**

```bash
# Check if key is in ssh-agent
ssh-add -l

# Add key if missing
ssh-add ~/.ssh/id_ed25519

# Or configure permanent key location
git config --global core.sshCommand "ssh -i ~/.ssh/id_ed25519"

# Test SSH connection
ssh -T git@github.com
```

### Large File Push Timeout

**Error:**
```
fatal: the remote end hung up unexpectedly
```

**Cause:** Large repository or slow network

**Solution:**

```bash
# Increase timeout
git config --global http.postBuffer 524288000

# Use compression
git config --global core.compression 9

# Try SSH instead of HTTPS
git remote set-url origin git@github.com:user/repo.git

# For truly large files, use Git LFS
git lfs install
git lfs track "*.bin"
git add .gitattributes file.bin
git commit -m "Add large binary file"
git push origin main
```

### Authentication Expired

**Error:**
```
fatal: Authentication failed for 'https://github.com/...'
```

**Solution:**

```bash
# For HTTPS token
git credential reject
git credential approve  # Re-enter credentials at next push

# For SSH, verify key
ssh -T git@github.com

# SSH passphrase prompt
ssh-add ~/.ssh/id_ed25519
```

### Wrong Remote URL

**Error:**
```
fatal: unable to access 'https://github.com/...'
```

**Solution:**

```bash
# Check current URL
git remote -v

# Update to correct URL
git remote set-url origin git@github.com:correct-user/repo.git

# Or change protocol
git remote set-url origin https://github.com/user/repo.git
```

### History Rewrite Rejected

**Error:**
```
error: failed to push some refs to 'origin'
hint: Updates were rejected because the tip of your current branch is behind
hint: its remote-tracking branch.
```

**Cause:** Local branch has been reset/rebased differently from remote

**Solution:**

```bash
# Force with lease (safer than --force)
git push --force-with-lease origin feature

# If you're certain about the change
git push --force origin feature  # Use with caution!

# Better: Verify what's happening
git log origin/feature..HEAD  # See what you're pushing
git log HEAD..origin/feature  # See what you're losing
```

---

## References and Sources

### Official Documentation

- [Git - git-fetch Documentation](https://git-scm.com/docs/git-fetch)
- [Git - git-pull Documentation](https://git-scm.com/docs/git-pull)
- [Git - git-push Documentation](https://git-scm.com/docs/git-push)
- [Git - git-merge Documentation](https://git-scm.com/docs/git-merge)
- [Git - git-rebase Documentation](https://git-scm.com/docs/git-rebase)

### Pro Git Book Chapters

- [Git Basics - Working with Remotes](https://git-scm.com/book/en/v2/Git-Basics-Working-with-Remotes)
- [Git Branching - Remote Branches](https://git-scm.com/book/en/v2/Git-Branching-Remote-Branches)
- [Distributed Git - Distributed Workflows](https://git-scm.com/book/en/v2/Distributed-Git-Distributed-Workflows)
- [Distributed Git - Contributing to a Project](https://git-scm.com/book/en/v2/Distributed-Git-Contributing-to-a-Project)
- [Distributed Git - Maintaining a Project](https://git-scm.com/book/en/v2/Distributed-Git-Maintaining-a-Project)
- [Git Tools - Rerere](https://git-scm.com/book/en/v2/Git-Tools-Rerere)

### Platform-Specific Documentation

**GitHub:**
- [Generating a new SSH key and adding it to the ssh-agent](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/generating-a-new-ssh-key-and-adding-it-to-the-ssh-agent)
- [Best practices for securing accounts](https://docs.github.com/en/code-security/supply-chain-security/end-to-end-supply-chain/securing-accounts)
- [Dealing with non-fast-forward errors](https://docs.github.com/en/get-started/using-git/dealing-with-non-fast-forward-errors)
- [Resolving merge conflicts after a Git rebase](https://docs.github.com/en/get-started/using-git/resolving-merge-conflicts-after-a-git-rebase)

**GitLab:**
- [Use SSH keys to communicate with GitLab](https://docs.gitlab.com/user/ssh/)

**Atlassian/Bitbucket:**
- [What Does Git Fetch Do?](https://www.atlassian.com/git/tutorials/syncing/git-fetch)
- [How to Pull a Git Repository?](https://www.atlassian.com/git/tutorials/syncing/git-pull)
- [Git SSH Keys: A Complete Tutorial](https://www.atlassian.com/git/tutorials/git-ssh)
- [How to Resolve Merge Conflicts in Git?](https://www.atlassian.com/git/tutorials/using-branches/merge-conflicts)
- [Git fast forwards and branch management](https://support.atlassian.com/bitbucket-cloud/docs/git-fast-forwards-and-branch-management/)

---

## Implementation Notes for Jin

When implementing remote operations in Jin, consider:

1. **Fetch vs Pull Configuration**
   - Expose `pull.rebase` configuration option
   - Provide safe defaults (rebase or merge)
   - Document implications of each choice

2. **Progress Reporting**
   - Implement `--progress` flag for long operations
   - Consider machine-readable output format
   - Log to files for automation/debugging

3. **Error Handling**
   - Catch non-fast-forward errors specifically
   - Provide clear error messages with solutions
   - Offer automatic retry options with exponential backoff

4. **Conflict Resolution**
   - Support both merge and rebase workflows
   - Implement conflict marker parsing
   - Consider integration with rerere

5. **Authentication**
   - Support SSH key configuration
   - Handle credential storage securely
   - Provide SSH agent integration

6. **Bare Repository Operations**
   - Ensure operations work correctly with bare repos
   - Validate repository format before operations
   - Provide clear error messages for incompatible operations

---

**Document Version:** 1.0
**Last Updated:** 2025-12-27
**Scope:** Git remote operations best practices and patterns for Jin implementation
