# Mocking and Testing Remote Git Operations

## Overview
Testing Git remote operations requires careful handling of network interactions, authentication scenarios, and concurrent access. This document covers strategies from real Git testing infrastructure and popular tools.

## Core Philosophy

**Key principle**: Mock the network, not Git itself.

```
Don't mock:
├─ git clone behavior
├─ git push mechanics
├─ git pull operations
└─ Repository state handling

Do mock:
├─ Network failures
├─ Authentication failures
├─ Server errors
├─ Rate limiting
└─ Latency/timeouts
```

## Real Git Server Approaches

### 1. Local File System Remotes (Preferred)

**Pattern**: Use local filesystem repositories as remotes instead of network servers.

**Setup**:
```bash
# Create bare repository (the "remote")
mkdir /tmp/test_remote.git
cd /tmp/test_remote.git
git init --bare

# Create local repository
mkdir /tmp/test_local
cd /tmp/test_local
git init
git config user.email "test@example.com"
git config user.name "Test User"

# Add remote pointing to filesystem path
git remote add origin /tmp/test_remote.git

# Create and push
echo "content" > file.txt
git add file.txt
git commit -m "Initial"
git push -u origin main
```

**Advantages**:
- Tests actual Git behavior
- No network involved
- Super fast execution
- Can run in parallel
- Tests survive Git implementation changes

**Verification**:
```bash
# Test can verify actual remote state
cd /tmp/test_remote.git
git log --oneline              # See commits
git show HEAD                  # See content
```

### 2. git-http-mock-server

**Purpose**: Real Git server for testing HTTP Git operations

**Key feature**: Copy-on-write isolation enables parallel test execution.

```
How it works:
1. Takes directory of bare git repositories
2. Serves them via HTTP using git-http-backend
3. Copy-on-write prevents push from modifying original
4. Multiple tests can push simultaneously
```

**Example usage**:
```bash
# Directory structure:
# test_repos/
# ├── repo1.git/
# ├── repo2.git/
# └── repo3.git/

# Start server
git-http-mock-server ./test_repos

# Tests can clone
git clone http://localhost:8174/repo1.git

# Push doesn't modify original
git push origin main
# Original /repo1.git is unchanged
```

**Features**:
- Serves real Git repositories over HTTP
- HTTP Basic Auth support for testing authentication
- CORS headers for browser-based access
- Supports `receive.denyNonFastforwards` configuration
- Perfect for HTTP Git protocol testing

**Documentation**: [GitHub - isomorphic-git/git-http-mock-server](https://github.com/isomorphic-git/git-http-mock-server)

### 3. Fake Git Server (FGS)

**Purpose**: Production-grade Git HTTP server for testing

**Characteristics**:
- Real HTTP Git protocol implementation
- Wraps vanilla `git http-backend`
- Used in Kubernetes testing infrastructure
- Supports both read and write operations

**Example from Prow (Kubernetes testing)**:
```yaml
# Kubernetes test setup
apiVersion: v1
kind: Pod
metadata:
  name: git-server
spec:
  containers:
  - name: git-server
    image: fgs:latest
    ports:
    - containerPort: 8080
```

**Documentation**: [Prow: Fake Git Server](https://docs.prow.k8s.io/docs/test/integration/fakegitserver/)

## Selective Mocking Strategies

### 1. Mock Only Network Layer

**Pattern**: Intercept network calls but use real Git logic

**Go Example with httptest**:
```go
func TestGitCloneWithAuth(t *testing.T) {
    // Create mock HTTP server
    server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        // Check authentication
        auth := r.Header.Get("Authorization")
        if auth != "Basic dGVzdDp0ZXN0" {
            w.WriteHeader(http.StatusUnauthorized)
            return
        }
        // Return real git-http-backend response
        handleGitHTTPBackend(w, r)
    }))
    defer server.Close()

    // Test with authentication requirement
    cmd := exec.Command("git", "clone", server.URL+"/repo.git")
    cmd.Env = append(os.Environ(), "GIT_ASKPASS=echo", "GIT_ASKPASS_PROMPT=none")
    err := cmd.Run()
    // Verify authentication was required
}
```

### 2. Network Failure Simulation

**Pattern**: Inject network errors without modifying test code

**Using network simulation tools**:
```bash
# Simulate 500ms latency
tc qdisc add dev eth0 root netem delay 500ms

# Simulate 10% packet loss
tc qdisc add dev eth0 root netem loss 10%

# Simulate timeout (connection drop)
iptables -A OUTPUT -p tcp --dport 443 -j DROP

# Run tests with impaired network
pytest test_git_operations.py

# Cleanup
tc qdisc del dev eth0 root
iptables -F
```

### 3. Mock Git Commands Minimally

**Pattern**: If mocking Git, only mock integration points

**Node.js Example with mock-git**:
```javascript
const mockGit = require('mock-git');

// Only mock specific scenarios
mockGit.mock('clone', (url, dir) => {
    if (url.includes('error')) {
        throw new Error('Clone failed');
    }
    return realGitClone(url, dir);
});

// Run tests
test('handles clone errors', async () => {
    try {
        await git.clone('https://github.com/error/repo.git', tmpdir);
        fail('Should have thrown');
    } catch (e) {
        expect(e.message).toContain('Clone failed');
    }
});
```

**Downsides**:
- Mocks become stale as Git evolves
- Test doesn't verify real behavior
- Implementation details leak into tests

**Use only for**: Testing your error handling code, not testing git itself

## Authentication Testing Scenarios

### 1. HTTP Basic Auth

**Pattern**: Mock server checking credentials

```go
func TestPushWithAuth(t *testing.T) {
    // Mock server validates credentials
    server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        user, pass, ok := r.BasicAuth()
        if !ok || user != "testuser" || pass != "testpass" {
            w.Header().Set("WWW-Authenticate", `Basic realm="Git"`)
            w.WriteHeader(http.StatusUnauthorized)
            return
        }
        // Handle authenticated request
        handleGitRequest(w, r)
    }))

    // Test verifies git handles auth correctly
    cmd := exec.Command("git", "push", server.URL+"/repo.git")
    // Git will prompt for credentials (or fail if not provided)
}
```

### 2. SSH Key Authentication

**Pattern**: Create test SSH keys and configure git

```bash
# Create test keypair (no passphrase)
ssh-keygen -t ed25519 -f test_key -N ""

# Configure SSH for test environment
export GIT_SSH_COMMAND="ssh -i test_key -o StrictHostKeyChecking=no"

# Run git command
git clone git@localhost:test/repo.git

# Cleanup
rm test_key test_key.pub
```

### 3. Token-Based Auth

**Pattern**: Use mock server to validate tokens

```go
func TestCloneWithToken(t *testing.T) {
    const validToken = "ghp_test1234567890"

    server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        // Check Authorization header
        auth := r.Header.Get("Authorization")
        if !strings.HasPrefix(auth, "token ") {
            w.WriteHeader(http.StatusUnauthorized)
            return
        }
        token := strings.TrimPrefix(auth, "token ")
        if token != validToken {
            w.WriteHeader(http.StatusForbidden)
            return
        }
        // Valid token - handle request
        handleGitRequest(w, r)
    }))

    // Test code
    cmd := exec.Command("git", "clone",
        fmt.Sprintf("https://token:%s@%s/repo.git", validToken, server.Host))
}
```

## Concurrent Access Testing

### 1. Multiple Pushes Simultaneously

**Pattern**: Verify copy-on-write isolation

```python
import subprocess
import threading
import tempfile
from pathlib import Path

def test_concurrent_pushes():
    # Create shared remote
    remote = Path(tempfile.mkdtemp())
    remote_repo = remote / "repo.git"
    subprocess.run(["git", "init", "--bare", str(remote_repo)], check=True)

    def push_from_client(client_num):
        client_repo = Path(tempfile.mkdtemp()) / "repo"
        client_repo.mkdir(parents=True)

        subprocess.run(["git", "init"], cwd=client_repo, check=True)
        subprocess.run(["git", "remote", "add", "origin", str(remote_repo)],
                      cwd=client_repo, check=True)

        # Create unique content
        (client_repo / f"file_{client_num}.txt").write_text(f"Client {client_num}")
        subprocess.run(["git", "add", "."], cwd=client_repo, check=True)
        subprocess.run(["git", "commit", "-m", f"Client {client_num}"],
                      cwd=client_repo, check=True)
        subprocess.run(["git", "push", "origin", "main"],
                      cwd=client_repo, check=True)

    # Push from multiple clients simultaneously
    threads = []
    for i in range(5):
        t = threading.Thread(target=push_from_client, args=(i,))
        t.start()
        threads.append(t)

    for t in threads:
        t.join()

    # Verify all commits are in remote
    result = subprocess.run(["git", "log", "--oneline"],
                           cwd=remote_repo, capture_output=True, text=True, check=True)
    assert result.stdout.count("\n") >= 5, "Not all commits merged"
```

### 2. Parallel Clone Stress Test

```bash
#!/usr/bin/env bash

# Start git-http-mock-server with test repos
git-http-mock-server ./test_repos &
SERVER_PID=$!

# Parallel clones
for i in {1..100}; do
    (
        tmpdir=$(mktemp -d)
        cd "$tmpdir"
        git clone http://localhost:8174/repo.git repo_$i
    ) &
done

wait

kill $SERVER_PID
```

## Testing Error Conditions

### 1. Server Errors

**Pattern**: Mock server returning error codes

```go
func TestHandles500Error(t *testing.T) {
    server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        w.WriteHeader(http.StatusInternalServerError)
        w.Write([]byte("Internal Server Error"))
    }))

    // Git should fail gracefully
    cmd := exec.Command("git", "clone", server.URL+"/repo.git")
    err := cmd.Run()
    if err == nil {
        t.Fatal("Expected clone to fail with server error")
    }
}
```

### 2. Connection Timeouts

**Pattern**: Server that doesn't respond

```go
func TestHandlesTimeout(t *testing.T) {
    listener, _ := net.Listen("tcp", "localhost:0")
    // Don't actually serve - just accept connection and hang

    addr := listener.Addr().String()

    // Git should timeout (use short timeout for test)
    cmd := exec.Command("git", "clone", fmt.Sprintf("http://%s/repo.git", addr))
    cmd.Env = append(os.Environ(), "GIT_CURL_VERBOSE=1")

    // Will timeout after a few seconds
    err := cmd.Run()
    if err == nil {
        t.Fatal("Expected timeout")
    }
}
```

### 3. Rejection Scenarios

**Pattern**: Simulate server rejecting specific operations

```go
func TestHandleForcePushRejection(t *testing.T) {
    server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
        if strings.Contains(r.URL.Path, "receive-pack") && r.Method == "POST" {
            // Simulate server refusing non-fast-forward push
            w.WriteHeader(http.StatusOK)
            w.Write([]byte("0000\nERR non-fast-forward\n"))
            return
        }
        // Normal git operations
        handleGitRequest(w, r)
    }))

    // Git push --force should fail
    cmd := exec.Command("git", "push", "--force", server.URL+"/repo.git")
    err := cmd.Run()
    if err == nil {
        t.Fatal("Expected push to be rejected")
    }
}
```

## Decision Tree: What to Mock?

```
Testing git remote operations
│
├─ Testing Git BEHAVIOR (clone, push, pull)
│  └─→ Don't mock Git - use real Git with local filesystem remote
│
├─ Testing HTTP protocol (authentication, certificates)
│  └─→ Mock HTTP server, use git-http-mock-server or httptest
│
├─ Testing YOUR error handling
│  └─→ Mock network errors, server errors
│      (not git errors)
│
├─ Testing performance with latency
│  └─→ Use network simulation tools (tc, iptables)
│
└─ Testing custom git wrapper
   └─→ Test your code, not git itself
```

## Best Practices

1. **Prefer local filesystem remotes**: Fast, realistic, no network
2. **Test real Git behavior**: Catch issues early
3. **Mock network, not Git**: Only intercept HTTP/SSH
4. **Use copy-on-write for parallel tests**: Prevents interference
5. **Verify remote state**: Check what's actually on "server"
6. **Test authentication properly**: Don't skip security
7. **Clean up test servers**: Prevent port conflicts
8. **Handle timeouts**: Network operations should timeout
9. **Test concurrent operations**: Parallel pushes/pulls
10. **Document mock server behavior**: So others understand

## References

- [git-http-mock-server: GitHub](https://github.com/isomorphic-git/git-http-mock-server)
- [Prow: Fake Git Server](https://docs.prow.k8s.io/docs/test/integration/fakegitserver/)
- [Stefan Zweifel: Testing git-auto-commit](https://stefanzweifel.dev/posts/2020/12/22/writing-integration-tests-for-git-auto-commit/)
- [Ryan Djurovich: Testing systems needing git clone](https://ryan0x44.medium.com/how-to-test-a-system-in-isolation-which-needs-to-git-clone-eec3449e6f7c)
- [mock-git: NPM Package](https://www.npmjs.com/package/mock-git)
- [mock-github: Local GitHub environment](https://github.com/kiegroup/mock-github)
