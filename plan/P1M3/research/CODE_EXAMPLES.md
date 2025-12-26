# Code Examples: Atomic Transaction Patterns in Rust

## Example 1: Basic Atomic File Write (using atomicwrites)

```rust
use atomicwrites::{AtomicFile, DisallowOverwrite};
use std::io::Write;

fn atomic_write_config(path: &str, contents: &str) -> Result<()> {
    let af = AtomicFile::new(path, DisallowOverwrite);
    af.write(|f| {
        f.write_all(contents.as_bytes())?;
        f.flush()
    })?;
    println!("Config atomically written");
    Ok(())
}

#[test]
fn test_atomic_write() {
    atomic_write_config("/tmp/config.json", r#"{"key": "value"}"#).unwrap();

    // If process crashes between write and rename, one of:
    // - New contents written successfully
    // - Old contents unchanged (no partial state)
}
```

## Example 2: Atomic File Write with Durability (using atomic-write-file)

```rust
use atomic_write_file::AtomicWriteFile;
use std::io::Write;

fn atomic_write_with_fsync(path: &str, contents: &[u8]) -> Result<()> {
    let mut file = AtomicWriteFile::open(path)?;

    // Write all data
    file.write_all(contents)?;

    // Ensure durability before committing
    file.sync_all()?;

    // Atomic rename (safe even if crash here)
    file.commit()?;

    Ok(())
}
```

## Example 3: Atomic Git Reference Updates (git update-ref --atomic)

```rust
use std::process::{Command, Stdio};
use std::io::Write;

struct GitTransaction {
    repo_path: String,
}

impl GitTransaction {
    fn new(repo_path: &str) -> Self {
        GitTransaction {
            repo_path: repo_path.to_string(),
        }
    }

    /// Atomically update multiple git references
    /// Either ALL succeed or NONE are modified
    fn atomic_update(&self, updates: &[(&str, &str, &str)]) -> Result<()> {
        // updates: (ref_name, old_oid, new_oid)

        let mut child = Command::new("git")
            .arg("-C").arg(&self.repo_path)
            .arg("update-ref")
            .arg("--atomic")
            .arg("--stdin")
            .stdin(Stdio::piped())
            .spawn()?;

        {
            let stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;

            for (ref_name, old_oid, new_oid) in updates {
                writeln!(
                    stdin,
                    "update {} {} {}",
                    ref_name, old_oid, new_oid
                )?;
            }
        }

        let status = child.wait()?;

        if status.success() {
            Ok(())
        } else {
            Err("Atomic update failed - NO changes applied".into())
        }
    }
}

#[test]
fn test_atomic_git_refs() {
    let tx = GitTransaction::new("/tmp/test.git");

    // Either ALL succeed or ALL fail - no partial state
    let result = tx.atomic_update(&[
        ("refs/heads/main", "abc123", "def456"),
        ("refs/heads/feature", "123abc", "456def"),
        ("refs/tags/v1.0.0", "000000", "789abc"),
    ]);

    assert!(result.is_ok());
}
```

## Example 4: Write-Ahead Log Pattern (Basic)

```rust
use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Serialize, Deserialize, Debug)]
enum LogEntry {
    TransactionBegin { id: String, timestamp: u64 },
    RefUpdate { ref_name: String, new_oid: String },
    RefUpdateComplete { ref_name: String },
    TransactionCommit { id: String },
    TransactionCleanup { id: String },
}

struct SimpleWAL {
    log_file: BufWriter<File>,
}

impl SimpleWAL {
    fn new(path: &str) -> Result<Self> {
        let file = File::create(path)?;
        Ok(SimpleWAL {
            log_file: BufWriter::new(file),
        })
    }

    /// Log entry and ensure durability
    fn write_entry(&mut self, entry: &LogEntry) -> Result<()> {
        // Serialize to JSON
        let json = serde_json::to_string(entry)?;

        // Write to buffer
        writeln!(&mut self.log_file, "{}", json)?;

        // CRITICAL: Flush to OS and fsync to disk
        self.log_file.flush()?;

        // Get underlying file and fsync
        // (In real implementation, use: file.sync_all()?)

        Ok(())
    }
}

fn wal_transaction_example() -> Result<()> {
    let mut wal = SimpleWAL::new(".jin/transaction.log")?;
    let tx_id = "tx-001";

    // Step 1: Log transaction start
    wal.write_entry(&LogEntry::TransactionBegin {
        id: tx_id.to_string(),
        timestamp: current_timestamp(),
    })?;

    // Step 2: Log ref update (before executing)
    wal.write_entry(&LogEntry::RefUpdate {
        ref_name: "refs/heads/main".to_string(),
        new_oid: "abc123def456".to_string(),
    })?;

    // Step 3: Now safe to execute (WAL is durable)
    // If crash here, recovery knows to redo the commit
    execute_git_update("refs/heads/main", "abc123def456")?;

    // Step 4: Log completion
    wal.write_entry(&LogEntry::RefUpdateComplete {
        ref_name: "refs/heads/main".to_string(),
    })?;

    // Step 5: Log commit
    wal.write_entry(&LogEntry::TransactionCommit {
        id: tx_id.to_string(),
    })?;

    // Step 6: Cleanup
    wal.write_entry(&LogEntry::TransactionCleanup {
        id: tx_id.to_string(),
    })?;

    Ok(())
}
```

## Example 5: Marker Files for Transaction State Detection

```rust
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug)]
struct TransactionMarker {
    id: String,
    timestamp: u64,
    operation: String,
    pid: u32,
    last_op_index: usize,
}

const MARKER_DIR: &str = ".jin";

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Start a transaction by creating marker file
fn start_transaction(id: &str, operation: &str) -> Result<()> {
    fs::create_dir_all(MARKER_DIR)?;

    let marker = TransactionMarker {
        id: id.to_string(),
        timestamp: current_timestamp(),
        operation: operation.to_string(),
        pid: std::process::id(),
        last_op_index: 0,
    };

    // Write to temp file
    let temp_path = format!("{}/.transaction.tmp", MARKER_DIR);
    let content = serde_json::to_string(&marker)?;
    fs::write(&temp_path, content)?;

    // Atomic rename to in-progress
    let in_progress_path = format!("{}/.transaction_in_progress", MARKER_DIR);
    fs::rename(&temp_path, &in_progress_path)?;

    println!("Transaction {} started", id);
    Ok(())
}

/// Update progress during transaction
fn update_transaction_progress(id: &str, op_index: usize) -> Result<()> {
    let marker_path = format!("{}/.transaction_in_progress", MARKER_DIR);

    let mut marker: TransactionMarker = serde_json::from_str(
        &fs::read_to_string(&marker_path)?
    )?;

    marker.last_op_index = op_index;
    marker.timestamp = current_timestamp();

    fs::write(&marker_path, serde_json::to_string(&marker)?)?;
    Ok(())
}

/// Detect interrupted transactions on startup
fn detect_interrupted_transactions() -> Result<Vec<TransactionMarker>> {
    let marker_path = format!("{}/.transaction_in_progress", MARKER_DIR);
    let mut interrupted = Vec::new();

    if Path::new(&marker_path).exists() {
        if let Ok(content) = fs::read_to_string(&marker_path) {
            if let Ok(marker) = serde_json::from_str::<TransactionMarker>(&content) {
                let now = current_timestamp();

                // Check if marker is stale (older than 5 minutes)
                if now - marker.timestamp > 300 {
                    // Check if process is still running
                    if !is_process_running(marker.pid) {
                        interrupted.push(marker);
                    }
                }
            }
        }
    }

    Ok(interrupted)
}

fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        unsafe {
            libc::kill(pid as i32, 0) == 0
        }
    }
    #[cfg(not(unix))]
    {
        false
    }
}

/// Complete transaction by moving marker
fn complete_transaction(id: &str, success: bool) -> Result<()> {
    let in_progress = format!("{}/.transaction_in_progress", MARKER_DIR);
    let completed = if success {
        format!("{}/.transaction_committed", MARKER_DIR)
    } else {
        format!("{}/.transaction_failed", MARKER_DIR)
    };

    if Path::new(&in_progress).exists() {
        fs::rename(&in_progress, &completed)?;
    }

    println!("Transaction {} {}", id, if success { "committed" } else { "failed" });
    Ok(())
}

#[test]
fn test_transaction_markers() {
    let _ = fs::remove_dir_all(MARKER_DIR);

    // Start transaction
    start_transaction("tx-001", "update-refs").unwrap();

    // Update progress
    update_transaction_progress("tx-001", 1).unwrap();
    update_transaction_progress("tx-001", 2).unwrap();

    // Complete
    complete_transaction("tx-001", true).unwrap();

    // Verify
    let marker_path = format!("{}/.transaction_committed", MARKER_DIR);
    assert!(Path::new(&marker_path).exists());
}
```

## Example 6: Recovery on Startup

```rust
use std::fs;
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum TransactionState {
    InProgress { marker: TransactionMarker },
    Committed { marker: TransactionMarker },
    Failed { marker: TransactionMarker },
}

fn recovery_on_startup() -> Result<()> {
    println!("=== Recovery on Startup ===");

    // Step 1: Detect interrupted transactions
    let interrupted = detect_interrupted_transactions()?;
    println!("Found {} interrupted transactions", interrupted.len());

    for marker in interrupted {
        println!("Recovering transaction: {}", marker.id);

        // Step 2: Read transaction log to determine state
        let wal_entries = read_transaction_log(&marker.id)?;

        // Step 3: Determine if refs were persisted
        let mut refs_persisted = Vec::new();
        for entry in &wal_entries {
            // Check if each ref actually exists on disk
            // ...
        }

        // Step 4: Decide action based on WAL state
        if has_fsync_complete_marker(&wal_entries) {
            println!("  -> All changes persisted, redoing commit");
            redo_commit(&marker.id, &refs_persisted)?;
        } else {
            println!("  -> Crash before durability, rolling back");
            rollback(&marker.id)?;
        }

        // Step 5: Mark recovery complete
        complete_recovery(&marker.id)?;
    }

    println!("=== Recovery Complete ===");
    Ok(())
}

fn has_fsync_complete_marker(entries: &[serde_json::Value]) -> bool {
    entries.iter().any(|e| {
        e.get("type").and_then(|v| v.as_str()) == Some("CommitFsyncComplete")
    })
}

fn redo_commit(tx_id: &str, refs: &[&str]) -> Result<()> {
    // Idempotent: check if already complete
    if is_commit_already_complete(tx_id)? {
        println!("  -> Commit already complete, skipping");
        return Ok(());
    }

    println!("  -> Executing commit");
    // Execute git update-ref for all persisted refs
    // ...

    // Mark as complete
    mark_commit_complete(tx_id)?;
    Ok(())
}

fn rollback(tx_id: &str) -> Result<()> {
    // Idempotent: check if already rolled back
    if is_rollback_complete(tx_id)? {
        println!("  -> Rollback already complete, skipping");
        return Ok(());
    }

    println!("  -> Executing rollback");
    // Undo any partial changes
    // ...

    // Mark as complete
    mark_rollback_complete(tx_id)?;
    Ok(())
}

fn complete_recovery(tx_id: &str) -> Result<()> {
    // Clean up marker files
    let marker_path = format!(".jin/.transaction_in_progress");
    let _ = fs::remove_file(marker_path);
    Ok(())
}

// Stub implementations for brevity
fn read_transaction_log(tx_id: &str) -> Result<Vec<serde_json::Value>> {
    Ok(vec![])
}

fn is_commit_already_complete(tx_id: &str) -> Result<bool> {
    Ok(false)
}

fn mark_commit_complete(tx_id: &str) -> Result<()> {
    Ok(())
}

fn is_rollback_complete(tx_id: &str) -> Result<bool> {
    Ok(false)
}

fn mark_rollback_complete(tx_id: &str) -> Result<()> {
    Ok(())
}
```

## Example 7: Integrated Transaction Manager

```rust
use std::fs;
use std::path::Path;

struct TransactionManager {
    repo_path: String,
    wal_dir: String,
}

struct Transaction {
    id: String,
    manager: std::rc::Rc<TransactionManager>,
    operations: Vec<Operation>,
    committed: bool,
}

#[derive(Clone)]
enum Operation {
    UpdateRef { name: String, new_oid: String },
    WriteFile { path: String, content: Vec<u8> },
}

impl TransactionManager {
    fn new(repo_path: &str, wal_dir: &str) -> Result<Self> {
        fs::create_dir_all(wal_dir)?;

        // Recover on startup
        Self::recover_incomplete_transactions(wal_dir)?;

        Ok(TransactionManager {
            repo_path: repo_path.to_string(),
            wal_dir: wal_dir.to_string(),
        })
    }

    fn begin_transaction(&self, id: &str) -> Result<Transaction> {
        // Create marker file
        start_transaction(id, "custom")?;

        Ok(Transaction {
            id: id.to_string(),
            manager: std::rc::Rc::new(TransactionManager {
                repo_path: self.repo_path.clone(),
                wal_dir: self.wal_dir.clone(),
            }),
            operations: Vec::new(),
            committed: false,
        })
    }

    fn recover_incomplete_transactions(wal_dir: &str) -> Result<()> {
        let interrupted = detect_interrupted_transactions()?;
        for marker in interrupted {
            println!("Recovering: {}", marker.id);
            recovery_on_startup()?;
        }
        Ok(())
    }
}

impl Transaction {
    fn add_operation(&mut self, op: Operation) -> Result<()> {
        // Log to WAL
        self.operations.push(op);
        Ok(())
    }

    fn commit(&mut self) -> Result<()> {
        // Log all operations
        // ...

        // Fsync
        // ...

        // Execute all operations atomically
        // ...

        // Mark as committed
        complete_transaction(&self.id, true)?;
        self.committed = true;

        Ok(())
    }
}

impl Drop for Transaction {
    fn drop(&mut self) {
        if !self.committed {
            // Rollback on drop if not explicitly committed
            let _ = complete_transaction(&self.id, false);
        }
    }
}

#[test]
fn test_integrated_transaction() {
    let manager = TransactionManager::new(".git", ".jin/wal").unwrap();

    let mut tx = manager.begin_transaction("tx-001").unwrap();

    tx.add_operation(Operation::UpdateRef {
        name: "refs/heads/main".to_string(),
        new_oid: "abc123".to_string(),
    }).unwrap();

    tx.add_operation(Operation::WriteFile {
        path: "config.json".to_string(),
        content: b"{}".to_vec(),
    }).unwrap();

    tx.commit().unwrap();
}
```

## Key Takeaways

1. **Always log before executing** - Write to WAL/marker file first
2. **Always fsync** - Data isn't durable without explicit fsync
3. **Use atomic operations** - Rename, git --atomic, compare-and-swap
4. **Make recovery idempotent** - Can be interrupted and resumed safely
5. **Test crash scenarios** - SIGKILL at each step in the transaction

## Running the Examples

```bash
# Add to Cargo.toml
[dependencies]
atomicwrites = "0.4"
atomic-write-file = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
git2 = "0.28"

# Run tests
cargo test --lib
```
