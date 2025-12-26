# Git Staging/Index Research

Git's staging area (index) is a binary file in `.git/index` containing cache entries for files to be committed.

## Index File Format

**Header** (12 bytes):
- Signature: "DIRC" (4 bytes)
- Version: 2 or 3 (4 bytes)
- Entry Count: number of entries (4 bytes)

**Cache Entry** (per file):
```
ctime_sec: u32      // Creation time
ctime_nsec: u32     // Creation time nanoseconds
mtime_sec: u32      // Modification time
mtime_nsec: u32     // Modification time nanoseconds
dev: u32           // Device ID
ino: u32           // Inode number
mode: u32          // File mode/permissions
uid: u32           // User ID
gid: u32           // Group ID
size: u32          // File size
oid: [u8; 20]      // SHA-1 hash pointing to blob
flags: u16         // Entry flags
path: [u8]         // Variable-length null-terminated path
```

**Footer**: 20-byte SHA-1 checksum of entire index

## Key Concepts for Jin Implementation

1. **Path-based indexing**: Entries sorted by path for efficient lookups
2. **Stat information**: Used to detect modifications without disk I/O
3. **Object IDs**: Point to blob objects in Git's object database
4. **Layer-aware entries**: Jin adds layer information to each entry

## References

- https://github.com/git/git/blob/master/Documentation/technical/index-format.txt
- https://docs.rs/git2/latest/git2/struct.Index.html
