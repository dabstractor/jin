# Testing File System Operations in Isolation

## Overview
Filesystem integration testing requires careful isolation to achieve fast, deterministic, and safe tests. This document covers patterns for isolating filesystem operations, mocking strategies, and real-world approaches.

## Core Isolation Principles

### 1. Dependency Injection Pattern

**Principle**: Never directly call filesystem functions; inject filesystem interface as dependency.

**Anti-pattern (tightly coupled)**:
```go
func ProcessFile(filename string) error {
    data, err := os.ReadFile(filename)  // Direct call - can't mock
    if err != nil {
        return err
    }
    return process(data)
}
```

**Good pattern (injectable)**:
```go
func ProcessFile(fs FileSystem, filename string) error {
    data, err := fs.ReadFile(filename)  // Injected - can mock
    if err != nil {
        return err
    }
    return process(data)
}
```

**Interface definition**:
```go
type FileSystem interface {
    Open(name string) (File, error)
    ReadFile(name string) ([]byte, error)
    WriteFile(name string, data []byte, perm FileMode) error
    MkdirAll(path string, perm FileMode) error
    Remove(name string) error
    // ... other operations
}
```

**Test implementation**:
```go
type MockFileSystem struct {
    files map[string][]byte
    // ... track errors, permissions, etc.
}

func (m *MockFileSystem) ReadFile(name string) ([]byte, error) {
    if data, ok := m.files[name]; ok {
        return data, nil
    }
    return nil, fs.ErrNotExist
}
```

### 2. In-Memory Filesystem Mock

**Pattern**: Fast, controllable test environments without disk I/O

**Go Example with fstest.MapFS**:
```go
func TestProcessing(t *testing.T) {
    // Define filesystem state in memory
    fsys := fstest.MapFS{
        "file.txt":      {Data: []byte("content")},
        "dir/nested.txt": {Data: []byte("nested content")},
    }

    // Test code uses this in-memory filesystem
    result, err := ProcessFile(fsys, "file.txt")
    if err != nil {
        t.Fatal(err)
    }
    // Assertions on result
}
```

**Advantages**:
- No disk I/O overhead
- Deterministic (no timing issues)
- Easily inject error conditions
- Fast test execution
- Parallel-safe (no file conflicts)

**Validation**:
```go
// Verify mock implementation is correct
err := fstest.TestFS(fsys)
if err != nil {
    t.Fatal("Mock filesystem doesn't conform to spec:", err)
}
```

### 3. Temporary Directory Pattern for Integration Tests

**Pattern**: Real filesystem testing with automatic cleanup

**Go Pattern**:
```go
func TestFileProcessingIntegration(t *testing.T) {
    // Create auto-cleanup temp directory
    tmpDir := t.TempDir()

    // Create test files
    testFile := filepath.Join(tmpDir, "test.txt")
    if err := os.WriteFile(testFile, []byte("content"), 0644); err != nil {
        t.Fatal(err)
    }

    // Test using real filesystem
    result, err := ProcessFile(os.DirFS(tmpDir), "test.txt")
    if err != nil {
        t.Fatal(err)
    }

    // Verify file operations occurred
    if _, err := os.Stat(filepath.Join(tmpDir, "output.txt")); err != nil {
        t.Fatal("Expected output file not created")
    }
}
```

**Features**:
- Real filesystem operations
- Automatic cleanup (even on panic)
- Cross-platform path handling
- Catches platform-specific issues (CRLF, permissions)

### 4. RAM-Based Filesystem (tmpfs)

**Pattern**: Speed of in-memory filesystem with real filesystem semantics

**Linux tmpfs characteristics**:
- Resides entirely in RAM
- All operations occur in memory (blazing fast)
- Behaves like real filesystem
- Survives across processes in same test
- Automatically cleaned up by OS

**Usage**:
```bash
# Create test using tmpfs
test_dir="/mnt/ramdisk/test_$$"
mkdir -p "$test_dir"

# Run tests
your_test_suite "$test_dir"

# Cleanup (automatic on reboot, or manual)
rm -rf "$test_dir"
```

**Benefits**:
- Real filesystem semantics
- 100-1000x faster than disk
- Prevents disk I/O contention
- No data corruption risk

## Isolation Strategy Matrix

```
                    Unit Tests          Integration Tests
┌──────────────────┼────────────────────┼──────────────────┐
│ File Operations  │ Mock (MapFS)       │ TempDir (Real)   │
│ Performance      │ Microseconds       │ Milliseconds     │
│ Coverage         │ Logic paths        │ Real behavior    │
│ Parallelization  │ Perfect            │ Good             │
│ Platform issues  │ None caught        │ Caught early     │
└──────────────────┴────────────────────┴──────────────────┘
```

## Advanced Testing Patterns

### 1. Error Injection with Mock Filesystem

**Pattern**: Simulate filesystem errors without real failures

```go
type ErrorInjectingFS struct {
    underlying fs.FS
    errOn      map[string]error
}

func (e *ErrorInjectingFS) ReadFile(name string) ([]byte, error) {
    if err, ok := e.errOn[name]; ok {
        return nil, err
    }
    return fs.ReadFile(e.underlying, name)
}

// Usage in test
func TestDiskFullError(t *testing.T) {
    mockFS := &ErrorInjectingFS{
        underlying: os.DirFS("."),
        errOn: map[string]error{
            "output.txt": syscall.ENOSPC, // Disk full
        },
    }

    err := ProcessFile(mockFS, "input.txt")
    if !errors.Is(err, syscall.ENOSPC) {
        t.Fatal("Expected ENOSPC error")
    }
}
```

### 2. State Tracking Mock Filesystem

**Pattern**: Verify all filesystem operations, not just final state

```go
type TrackedFS struct {
    operations []Operation
    files      map[string][]byte
}

type Operation struct {
    Type      string // "read", "write", "delete"
    Path      string
    Data      []byte
    Timestamp time.Time
}

func (t *TrackedFS) WriteFile(name string, data []byte, perm fs.FileMode) error {
    t.operations = append(t.operations, Operation{
        Type:      "write",
        Path:      name,
        Data:      data,
        Timestamp: time.Now(),
    })
    t.files[name] = data
    return nil
}

// Test assertions on operations
func TestOperationOrder(t *testing.T) {
    mockFS := &TrackedFS{}
    ProcessFile(mockFS, "input.txt")

    // Verify operations in expected order
    if len(mockFS.operations) < 2 {
        t.Fatal("Expected multiple operations")
    }
    if mockFS.operations[0].Type != "read" {
        t.Fatal("First operation should be read")
    }
}
```

### 3. Process Isolation with Separate Working Directory

**Pattern**: Run each test in separate process with isolated working directory

**Cargo/Rust pattern**:
```rust
// Using cargo-nextest for process isolation
// Each test runs in separate process
// Automatic working directory isolation
```

**Benefits**:
- Complete isolation (no global state)
- Can't affect other tests even with permissions
- True parallel execution
- Can use relative paths safely

## File Assertion Patterns

### 1. File Content Verification

```go
func assertFileContent(t *testing.T, path, expected string) {
    data, err := os.ReadFile(path)
    if err != nil {
        t.Fatalf("Failed to read %s: %v", path, err)
    }
    if string(data) != expected {
        t.Fatalf("File content mismatch.\nExpected:\n%s\nGot:\n%s",
            expected, string(data))
    }
}
```

### 2. Directory Structure Verification

```go
func assertDirStructure(t *testing.T, dir string, expected map[string]string) {
    entries, err := os.ReadDir(dir)
    if err != nil {
        t.Fatal(err)
    }

    for _, entry := range entries {
        path := filepath.Join(dir, entry.Name())
        if expectedContent, ok := expected[entry.Name()]; ok {
            assertFileContent(t, path, expectedContent)
        }
    }
}
```

### 3. Using assert_fs in Rust

```rust
#[test]
fn test_file_creation() {
    let temp = TempDir::new().unwrap();

    // Test code creates files
    create_config(temp.path()).unwrap();

    // Assert files exist with correct content
    assert_fs::assert_eq_dir! {
        temp,
        "config.toml" => contains("[app]"),
        "data/" => dir::contains([
            "default.json",
        ])
    }
}
```

## Real-World Isolation Examples

### Rust CLI with assert_fs

```rust
use assert_cmd::Command;
use assert_fs::prelude::*;

#[test]
fn test_generates_output_file() {
    // Isolated temp directory
    let temp = assert_fs::TempDir::new().unwrap();
    let output_file = temp.child("output.txt");

    // Run CLI
    let mut cmd = Command::cargo_bin("my_cli").unwrap();
    cmd.current_dir(temp.path())
        .arg("--output")
        .arg("output.txt")
        .assert()
        .success();

    // Assert file was created with correct content
    output_file.assert(predicate::str::contains("expected content"));

    // Cleanup automatic when temp is dropped
}
```

### Ruby RSpec with tmpfs

```ruby
describe MyFilesystemTask do
  around do |example|
    # Create tmpfs mount for this test
    temp_dir = "/mnt/tmpfs/test_#{Process.pid}"
    system("mkdir -p #{temp_dir}")

    example.run

    # Cleanup
    system("rm -rf #{temp_dir}")
  end

  it "processes large files efficiently" do
    # Tests here use tmpfs, making them very fast
  end
end
```

## Best Practices

1. **Inject all filesystem operations**: Never call os.Open() directly
2. **Unit tests use mocks**: Fast, focused testing
3. **Integration tests use temp dirs**: Real behavior validation
4. **Error inject selectively**: Test error paths without real failures
5. **Track operations**: Verify not just final state but how you got there
6. **Assert both positive and negative**: File exists AND doesn't have bad content
7. **Use platform-appropriate paths**: filepath.Join(), not string concatenation
8. **Clean up explicitly**: Even if test framework helps, be explicit
9. **Avoid hardcoded paths**: Use temp dirs, never /tmp or C:\ directly
10. **Parallel-safe defaults**: Design for parallel execution from start

## Anti-Patterns

1. **Direct filesystem calls**: Can't mock, can't test errors
2. **Shared test files**: Tests interfere with each other
3. **Assuming specific OS paths**: Fails on different platforms
4. **No cleanup**: Files accumulate and interfere
5. **Over-mocking**: Testing mock, not real behavior
6. **Ignoring permissions**: Windows/Linux differences
7. **File locking issues**: No cross-platform file deletion

## References

- [DEV Community: Testing File System Code](https://dev.to/rezmoss/testing-file-system-code-mocking-stubbing-and-test-patterns-99-1fkh)
- [Isolated Integration Tests with TestContainers](https://timdeschryver.dev/blog/writing-isolated-integrationtests-with-testcontainers)
- [Advanced Rust Testing: Filesystem Isolation](https://rust-exercises.com/advanced-testing/05_filesystem_isolation/04_outro.html)
- [Semaphore: Getting Integration Testing Right](https://semaphore.io/blog/integration-tests)
- [NVISIA: Isolated Integration Tests](https://www.nvisia.com/insights/isolated-integration-tests-oxymoron-or-best-practice)
- [Go: io/fs Testing Package](https://golang.org/pkg/io/fs/)
- [Rust: tempfile crate](https://docs.rs/tempfile/)
