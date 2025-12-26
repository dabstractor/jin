//! Jin repository wrapper around `git2::Repository`.
//!
//! This module provides `JinRepo`, a wrapper around `git2::Repository` that
//! encapsulates Jin-specific Git operations including:
//! - Bare repository initialization and opening
//! - Layer reference management using `Layer.git_ref()`
//! - Object creation helpers (blob, tree, commit)
//! - Delegated common operations with `JinError` conversion
//!
//! # Jin-Specific Semantics
//!
//! - **Bare Repository**: Jin repos have no working directory
//! - **Logical Refs**: Layer refs under `refs/jin/layers/...` are not branches
//! - **Layer Integration**: Ref names come from `Layer.git_ref()`, not hardcoded
//!
//! # Examples
//!
//! ```ignore
//! use jin_glm::git::JinRepo;
//! use jin_glm::core::Layer;
//! use std::path::Path;
//!
//! // Open or create the Jin repository
//! let repo = JinRepo::open_or_create(Path::new("~/.jin/repo"))?;
//!
//! // Get or create a layer reference
//! let layer = Layer::GlobalBase;
//! match repo.get_layer_ref(&layer)? {
//!     Some(reference) => println!("Layer exists: {:?}", reference.target()),
//!     None => println!("Layer does not exist yet"),
//! }
//! ```

use crate::core::error::{JinError, Result};
use crate::core::Layer;
use git2::Repository;
use std::path::Path;

/// Jin repository wrapper around `git2::Repository`.
///
/// The wrapper owns the Repository (not borrowed) and provides:
/// - Jin-aware constructors (bare repos only)
/// - Layer reference management using `Layer.git_ref()`
/// - Object creation helpers
/// - Delegated common operations with `JinError` conversion
///
/// # Jin-Specific Semantics
///
/// - **Bare Repository**: Jin repos have no working directory
/// - **Logical Refs**: Layer refs under `refs/jin/layers/...` are not branches
/// - **Layer Integration**: Ref names come from `Layer.git_ref()`, not hardcoded
///
/// # Examples
///
/// ```ignore
/// use jin_glm::git::JinRepo;
/// use jin_glm::core::Layer;
/// use std::path::Path;
///
/// // Open or create the Jin repository
/// let repo = JinRepo::open_or_create(Path::new("~/.jin/repo"))?;
///
/// // Get or create a layer reference
/// let layer = Layer::GlobalBase;
/// match repo.get_layer_ref(&layer)? {
///     Some(reference) => println!("Layer exists: {:?}", reference.target()),
///     None => println!("Layer does not exist yet"),
/// }
/// ```
pub struct JinRepo {
    /// The underlying git2 repository (owned, not borrowed)
    pub(crate) inner: Repository,
}

impl std::fmt::Debug for JinRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JinRepo")
            .field("path", &self.inner.path())
            .field("is_bare", &self.inner.is_bare())
            .field("is_empty", &self.inner.is_empty().unwrap_or(false))
            .finish()
    }
}

// ===== Constructors =====

impl JinRepo {
    /// Opens an existing Jin repository at the specified path.
    ///
    /// The repository must already exist. Use `open_or_create()` to create
    /// a new repository if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the bare repository directory
    ///
    /// # Errors
    ///
    /// Returns `JinError::RepoNotFound` if the repository doesn't exist
    /// at the specified path.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let repo = JinRepo::open(Path::new("~/.jin/repo"))?;
    /// ```
    pub fn open(path: &Path) -> Result<Self> {
        let inner = Repository::open(path).map_err(|_e| JinError::RepoNotFound {
            path: path.display().to_string(),
        })?;
        Ok(Self { inner })
    }

    /// Initializes a new bare Jin repository at the specified path.
    ///
    /// **CRITICAL**: Uses `Repository::init_bare()` because Jin repos are
    /// always bare (no working directory).
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the bare repository should be created
    ///
    /// # Errors
    ///
    /// Returns `JinError::Message` if initialization fails.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let repo = JinRepo::init(Path::new("~/.jin/repo"))?;
    /// assert!(repo.is_bare());
    /// ```
    pub fn init(path: &Path) -> Result<Self> {
        let inner = Repository::init_bare(path)
            .map_err(|e| JinError::Message(format!("Failed to init bare repo: {}", e)))?;
        Ok(Self { inner })
    }

    /// Opens an existing repository or creates a new one if it doesn't exist.
    ///
    /// This is the recommended constructor for most use cases, as it handles
    /// both initial setup and subsequent access.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the repository directory
    ///
    /// # Errors
    ///
    /// Returns `JinError::RepoNotFound` if opening fails for reasons other
    /// than the repository not existing.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // First call creates the repo
    /// let repo = JinRepo::open_or_create(Path::new("~/.jin/repo"))?;
    /// // Subsequent calls open the existing repo
    /// let repo2 = JinRepo::open_or_create(Path::new("~/.jin/repo"))?;
    /// ```
    pub fn open_or_create(path: &Path) -> Result<Self> {
        match Repository::open(path) {
            Ok(repo) => Ok(Self { inner: repo }),
            Err(e) if e.code() == git2::ErrorCode::NotFound => Self::init(path),
            Err(e) => Err(JinError::from(e)),
        }
    }
}

// ===== Layer Reference Operations =====

impl JinRepo {
    /// Gets a layer reference, returning `None` if it doesn't exist.
    ///
    /// Uses `Layer.git_ref()` to get the reference name. Returns `None` if
    /// the reference doesn't exist (not an error).
    ///
    /// # Arguments
    ///
    /// * `layer` - The layer to look up
    ///
    /// # Returns
    ///
    /// * `Some(reference)` - The layer reference exists
    /// * `None` - The layer reference doesn't exist
    ///
    /// # Errors
    ///
    /// Returns `JinError::InvalidLayer` if the layer is `UserLocal` or
    /// `WorkspaceActive` (these don't have git refs).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let layer = Layer::GlobalBase;
    /// match repo.get_layer_ref(&layer)? {
    ///     Some(reference) => println!("Layer exists: {:?}", reference.target()),
    ///     None => println!("Layer does not exist yet"),
    /// }
    /// ```
    pub fn get_layer_ref(&self, layer: &Layer) -> Result<Option<git2::Reference>> {
        let ref_name = layer.git_ref().ok_or_else(|| JinError::InvalidLayer {
            name: format!("{:?}", layer),
        })?;

        match self.inner.find_reference(&ref_name) {
            Ok(reference) => Ok(Some(reference)),
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(e) => Err(JinError::from(e)),
        }
    }

    /// Sets or updates a layer reference to point to the specified object.
    ///
    /// Uses `force=true` to allow updating existing references.
    ///
    /// # Arguments
    ///
    /// * `layer` - The layer to update
    /// * `oid` - The object ID the reference should point to
    ///
    /// # Returns
    ///
    /// The created or updated reference.
    ///
    /// # Errors
    ///
    /// Returns `JinError::InvalidLayer` if the layer is `UserLocal` or
    /// `WorkspaceActive`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let layer = Layer::GlobalBase;
    /// let commit_oid = repo.create_commit(...)?;
    /// let reference = repo.set_layer_ref(&layer, commit_oid)?;
    /// ```
    pub fn set_layer_ref(&self, layer: &Layer, oid: git2::Oid) -> Result<git2::Reference> {
        let ref_name = layer.git_ref().ok_or_else(|| JinError::InvalidLayer {
            name: format!("{:?}", layer),
        })?;

        self.inner
            .reference(
                &ref_name,
                oid,
                true, // force=true to allow updates
                &format!("Update layer: {:?}", layer),
            )
            .map_err(JinError::from)
    }

    /// Creates a new layer reference, failing if it already exists.
    ///
    /// Uses `force=false` for safety. Use `set_layer_ref()` to update
    /// existing references.
    ///
    /// # Arguments
    ///
    /// * `layer` - The layer to create
    /// * `oid` - The object ID the reference should point to
    ///
    /// # Returns
    ///
    /// The newly created reference.
    ///
    /// # Errors
    ///
    /// - `JinError::InvalidLayer` if the layer is `UserLocal` or `WorkspaceActive`
    /// - `JinError::RefExists` if the reference already exists
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let layer = Layer::GlobalBase;
    /// let commit_oid = repo.create_commit(...)?;
    /// // First call succeeds
    /// let reference = repo.create_layer_ref(&layer, commit_oid)?;
    /// // Second call fails with RefExists
    /// let result = repo.create_layer_ref(&layer, commit_oid);
    /// assert!(matches!(result, Err(JinError::RefExists { .. })));
    /// ```
    pub fn create_layer_ref(&self, layer: &Layer, oid: git2::Oid) -> Result<git2::Reference> {
        let ref_name = layer.git_ref().ok_or_else(|| JinError::InvalidLayer {
            name: format!("{:?}", layer),
        })?;

        self.inner
            .reference(
                &ref_name,
                oid,
                false, // force=false - fail if exists
                &format!("Create layer: {:?}", layer),
            )
            .map_err(|e| match e.code() {
                git2::ErrorCode::Exists => JinError::RefExists {
                    name: ref_name.clone(),
                    layer: format!("{:?}", layer),
                },
                _ => JinError::from(e),
            })
    }

    /// Deletes a layer reference.
    ///
    /// Uses `Layer.git_ref()` to get the reference name.
    ///
    /// # Arguments
    ///
    /// * `layer` - The layer whose reference should be deleted
    ///
    /// # Errors
    ///
    /// - `JinError::InvalidLayer` if the layer is `UserLocal` or `WorkspaceActive`
    /// - `JinError::RefNotFound` if the reference doesn't exist
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let layer = Layer::GlobalBase;
    /// repo.delete_layer_ref(&layer)?;
    /// ```
    pub fn delete_layer_ref(&self, layer: &Layer) -> Result<()> {
        let ref_name = layer.git_ref().ok_or_else(|| JinError::InvalidLayer {
            name: format!("{:?}", layer),
        })?;

        // CRITICAL: Must be mut for delete()
        let mut reference = self
            .inner
            .find_reference(&ref_name)
            .map_err(|e| match e.code() {
                git2::ErrorCode::NotFound => JinError::RefNotFound {
                    name: ref_name.clone(),
                    layer: format!("{:?}", layer),
                },
                _ => JinError::from(e),
            })?;

        reference.delete()?;
        Ok(())
    }

    /// Checks if a layer reference exists.
    ///
    /// Uses `Layer.git_ref()` to get the reference name.
    /// Returns `false` for `UserLocal` and `WorkspaceActive` layers.
    ///
    /// # Arguments
    ///
    /// * `layer` - The layer to check
    ///
    /// # Returns
    ///
    /// * `true` - The layer reference exists
    /// * `false` - The layer reference doesn't exist or is unversioned
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let layer = Layer::GlobalBase;
    /// if repo.layer_ref_exists(&layer) {
    ///     println!("Layer exists");
    /// }
    /// ```
    pub fn layer_ref_exists(&self, layer: &Layer) -> bool {
        let ref_name = match layer.git_ref() {
            Some(name) => name,
            None => return false, // UserLocal/WorkspaceActive never exist as refs
        };

        self.inner.find_reference(&ref_name).is_ok()
    }

    /// Lists all layer references.
    ///
    /// Returns a vector of `(Layer, Oid)` tuples for all layer refs that exist.
    ///
    /// # Returns
    ///
    /// A vector of tuples containing the layer and its target object ID.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let refs = repo.list_layer_refs()?;
    /// for (layer, oid) in refs {
    ///     println!("{:?} -> {}", layer, oid);
    /// }
    /// ```
    pub fn list_layer_refs(&self) -> Result<Vec<(Layer, git2::Oid)>> {
        let mut refs = Vec::new();

        for reference in self.inner.references_glob("refs/jin/layers/*")? {
            let reference = reference?;
            if let (Some(name), Some(oid)) = (reference.name(), reference.target()) {
                // Parse ref name back to Layer
                if let Some(layer) = Self::ref_name_to_layer(name) {
                    refs.push((layer, oid));
                }
            }
        }

        Ok(refs)
    }

    /// Lists layer references matching a glob pattern.
    ///
    /// Use this for pattern-based queries like "refs/jin/layers/mode/*".
    ///
    /// # Arguments
    ///
    /// * `pattern` - Git glob pattern for matching refs (e.g., "refs/jin/layers/mode/*")
    ///
    /// # Returns
    ///
    /// A vector of reference names matching the pattern.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // List all mode refs
    /// let mode_refs = repo.list_layer_refs_by_pattern("refs/jin/layers/mode/*")?;
    /// ```
    pub fn list_layer_refs_by_pattern(&self, pattern: &str) -> Result<Vec<String>> {
        let mut refs = Vec::new();

        for reference in self.inner.references_glob(pattern)? {
            let reference = reference?;
            if let Some(name) = reference.name() {
                refs.push(name.to_string());
            }
        }

        Ok(refs)
    }

    /// Creates a staging reference for a transaction.
    ///
    /// Staging refs follow the pattern `refs/jin/staging/<transaction-id>`.
    /// Uses `force=false` to detect conflicting transactions.
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - Unique identifier for the transaction
    /// * `oid` - The object ID the staging ref should point to
    ///
    /// # Returns
    ///
    /// The newly created staging reference.
    ///
    /// # Errors
    ///
    /// - `JinError::RefExists` if a staging ref with this ID already exists
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let transaction_id = "txn-abc123";
    /// let staging_ref = repo.create_staging_ref(transaction_id, commit_oid)?;
    /// ```
    pub fn create_staging_ref(
        &self,
        transaction_id: &str,
        oid: git2::Oid,
    ) -> Result<git2::Reference> {
        let ref_name = format!("refs/jin/staging/{}", transaction_id);

        self.inner
            .reference(
                &ref_name,
                oid,
                false, // force=false - fail if exists (detects conflicts)
                &format!("Staging ref for transaction: {}", transaction_id),
            )
            .map_err(|e| match e.code() {
                git2::ErrorCode::Exists => JinError::RefExists {
                    name: ref_name.clone(),
                    layer: "staging".to_string(),
                },
                _ => JinError::from(e),
            })
    }

    /// Deletes a staging reference.
    ///
    /// Used for cleanup after a transaction completes.
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - The transaction ID whose staging ref should be deleted
    ///
    /// # Errors
    ///
    /// - `JinError::RefNotFound` if the staging ref doesn't exist
    ///
    /// # Examples
    ///
    /// ```ignore
    /// repo.delete_staging_ref("txn-abc123")?;
    /// ```
    pub fn delete_staging_ref(&self, transaction_id: &str) -> Result<()> {
        let ref_name = format!("refs/jin/staging/{}", transaction_id);

        let mut reference = self
            .inner
            .find_reference(&ref_name)
            .map_err(|e| match e.code() {
                git2::ErrorCode::NotFound => JinError::RefNotFound {
                    name: ref_name.clone(),
                    layer: "staging".to_string(),
                },
                _ => JinError::from(e),
            })?;

        reference.delete()?;
        Ok(())
    }

    /// Checks if a staging reference exists.
    ///
    /// Used for transaction recovery detection.
    ///
    /// # Arguments
    ///
    /// * `transaction_id` - The transaction ID to check
    ///
    /// # Returns
    ///
    /// * `true` - The staging ref exists
    /// * `false` - The staging ref doesn't exist
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if repo.staging_ref_exists("txn-abc123") {
    ///     println!("Transaction is in progress");
    /// }
    /// ```
    pub fn staging_ref_exists(&self, transaction_id: &str) -> bool {
        let ref_name = format!("refs/jin/staging/{}", transaction_id);
        self.inner.find_reference(&ref_name).is_ok()
    }
}

// ===== Private Helpers =====

impl JinRepo {
    /// Parses a reference name back to a Layer variant.
    ///
    /// This is the inverse of `Layer.git_ref()`. Returns `None` if the
    /// reference name doesn't match the expected pattern or is invalid.
    ///
    /// # Arguments
    ///
    /// * `ref_name` - The full reference name (e.g., "refs/jin/layers/global")
    ///
    /// # Returns
    ///
    /// * `Some(Layer)` - The parsed layer variant
    /// * `None` - The reference name is not a valid layer ref
    fn ref_name_to_layer(ref_name: &str) -> Option<Layer> {
        if !ref_name.starts_with("refs/jin/layers/") {
            return None;
        }

        let path = ref_name.strip_prefix("refs/jin/layers/")?;

        // Parse based on path structure
        let parts: Vec<&str> = path.split('/').collect();
        match parts.as_slice() {
            ["global"] => Some(Layer::GlobalBase),
            ["mode", mode] => Some(Layer::ModeBase {
                mode: mode.to_string(),
            }),
            ["mode", mode, "scope", scope] => Some(Layer::ModeScope {
                mode: mode.to_string(),
                scope: scope.to_string(),
            }),
            ["mode", mode, "scope", scope, "project", project] => Some(Layer::ModeScopeProject {
                mode: mode.to_string(),
                scope: scope.to_string(),
                project: project.to_string(),
            }),
            ["mode", mode, "project", project] => Some(Layer::ModeProject {
                mode: mode.to_string(),
                project: project.to_string(),
            }),
            ["scope", scope] => Some(Layer::ScopeBase {
                scope: scope.to_string(),
            }),
            ["project", project] => Some(Layer::ProjectBase {
                project: project.to_string(),
            }),
            _ => None,
        }
    }
}

// ===== Object Creation Helpers =====

impl JinRepo {
    /// Creates a blob from the provided data.
    ///
    /// # Arguments
    ///
    /// * `data` - The blob content as bytes
    ///
    /// # Returns
    ///
    /// The object ID of the created blob.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let content = b"Hello, World!";
    /// let blob_oid = repo.create_blob(content)?;
    /// ```
    pub fn create_blob(&self, data: &[u8]) -> Result<git2::Oid> {
        Ok(self.inner.blob(data)?)
    }

    /// Writes a tree builder to create a tree object.
    ///
    /// # Arguments
    ///
    /// * `builder` - The tree builder to write
    ///
    /// # Returns
    ///
    /// The object ID of the created tree.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut builder = repo.treebuilder()?;
    /// let blob_oid = repo.create_blob(b"content")?;
    /// builder.insert("file.txt", blob_oid, git2::FileMode::Blob)?;
    /// let tree_oid = repo.create_tree(&mut builder)?;
    /// ```
    pub fn create_tree(&self, builder: &mut git2::TreeBuilder) -> Result<git2::Oid> {
        Ok(builder.write()?)
    }

    /// Creates a new commit.
    ///
    /// # Arguments
    ///
    /// * `update_ref` - Optional reference to update (e.g., `Some("HEAD")`)
    /// * `author` - The author signature
    /// * `committer` - The committer signature
    /// * `message` - The commit message
    /// * `tree` - The tree object for this commit
    /// * `parents` - Slice of parent commits
    ///
    /// # Returns
    ///
    /// The object ID of the created commit.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let tree = repo.find_tree(tree_oid)?;
    /// let author = repo.signature("User", "user@example.com")?;
    /// let committer = &author;
    /// let commit_oid = repo.create_commit(
    ///     Some("HEAD"),
    ///     &author,
    ///     committer,
    ///     "Initial commit",
    ///     &tree,
    ///     &[],
    /// )?;
    /// ```
    pub fn create_commit(
        &self,
        update_ref: Option<&str>,
        author: &git2::Signature,
        committer: &git2::Signature,
        message: &str,
        tree: &git2::Tree,
        parents: &[&git2::Commit],
    ) -> Result<git2::Oid> {
        Ok(self
            .inner
            .commit(update_ref, author, committer, message, tree, parents)?)
    }
}

// ===== Delegation Methods =====

impl JinRepo {
    /// Gets the HEAD reference.
    ///
    /// # Errors
    ///
    /// Returns an error if the repository is empty (no commits).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// match repo.head() {
    ///     Ok(reference) => println!("HEAD: {:?}", reference.target()),
    ///     Err(_) => println!("Repository is empty"),
    /// }
    /// ```
    pub fn head(&self) -> Result<git2::Reference> {
        Ok(self.inner.head()?)
    }

    /// Finds a commit by its object ID.
    ///
    /// # Arguments
    ///
    /// * `oid` - The commit object ID
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let commit = repo.find_commit(commit_oid)?;
    /// println!("Commit message: {}", commit.message().unwrap_or("(none)"));
    /// ```
    pub fn find_commit(&self, oid: git2::Oid) -> Result<git2::Commit> {
        Ok(self.inner.find_commit(oid)?)
    }

    /// Finds a tree by its object ID.
    ///
    /// # Arguments
    ///
    /// * `oid` - The tree object ID
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let tree = repo.find_tree(tree_oid)?;
    /// println!("Tree has {} entries", tree.len());
    /// ```
    pub fn find_tree(&self, oid: git2::Oid) -> Result<git2::Tree> {
        Ok(self.inner.find_tree(oid)?)
    }

    /// Finds a blob by its object ID.
    ///
    /// # Arguments
    ///
    /// * `oid` - The blob object ID
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let blob = repo.find_blob(blob_oid)?;
    /// let content = blob.content();
    /// ```
    pub fn find_blob(&self, oid: git2::Oid) -> Result<git2::Blob> {
        Ok(self.inner.find_blob(oid)?)
    }

    /// Creates a new tree builder.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut builder = repo.treebuilder()?;
    /// let blob_oid = repo.create_blob(b"content")?;
    /// builder.insert("file.txt", blob_oid, git2::FileMode::Blob)?;
    /// ```
    pub fn treebuilder(&self) -> Result<git2::TreeBuilder> {
        Ok(self.inner.treebuilder(None)?)
    }

    /// Creates an empty tree.
    ///
    /// # Returns
    ///
    /// The object ID of the newly created empty tree.
    pub fn create_empty_tree(&self) -> Result<git2::Oid> {
        Ok(self.treebuilder()?.write()?)
    }

    /// Creates a new reference.
    ///
    /// # Arguments
    ///
    /// * `name` - The reference name (e.g., "refs/heads/main")
    /// * `id` - The object ID the reference should point to
    /// * `force` - Whether to overwrite an existing reference
    /// * `log_message` - Message to write to the reflog
    pub fn create_reference(
        &self,
        name: &str,
        id: git2::Oid,
        force: bool,
        log_message: &str,
    ) -> Result<git2::Reference> {
        Ok(self.inner.reference(name, id, force, log_message)?)
    }

    /// Finds a reference by name.
    ///
    /// # Arguments
    ///
    /// * `name` - The reference name (e.g., "refs/heads/main")
    pub fn find_reference(&self, name: &str) -> Result<git2::Reference> {
        Ok(self.inner.find_reference(name)?)
    }
}

// ===== Tree Walking Methods =====

impl JinRepo {
    /// Walks a tree recursively, calling a callback for each file entry.
    ///
    /// Uses an iterative stack-based approach to avoid recursion depth issues
    /// with deeply nested directory structures.
    ///
    /// # Arguments
    ///
    /// * `tree_id` - The object ID of the tree to walk
    /// * `callback` - Function called for each blob entry with (full_path, entry)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - All entries processed successfully
    /// * `Err(JinError)` - Walking failed or callback returned an error
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // List all file paths in a tree
    /// let mut paths = Vec::new();
    /// repo.walk_tree(tree_id, |path, entry| {
    ///     paths.push(path.to_string());
    ///     Ok(())
    /// })?;
    /// ```
    pub fn walk_tree<F>(&self, tree_id: git2::Oid, mut callback: F) -> Result<()>
    where
        F: FnMut(&str, &git2::TreeEntry) -> Result<()>,
    {
        // Use iterative stack to avoid recursion depth issues
        let mut stack = vec![(tree_id, String::new())];

        while let Some((current_id, base_path)) = stack.pop() {
            let tree = self.find_tree(current_id)?;

            // Walk in reverse order to maintain natural processing order
            // (because stack is LIFO, reversing ensures correct order)
            for entry in tree.iter().rev() {
                let name = entry.name().unwrap_or("<unnamed>");
                let full_path = if base_path.is_empty() {
                    name.to_string()
                } else {
                    format!("{}/{}", base_path, name)
                };

                match entry.kind() {
                    Some(git2::ObjectType::Blob) => {
                        // File entry - invoke callback
                        callback(&full_path, &entry)?;
                    }
                    Some(git2::ObjectType::Tree) => {
                        // Directory entry - push to stack for further traversal
                        stack.push((entry.id(), full_path));
                    }
                    _ => {
                        // Ignore other object types (links, commits, etc.)
                    }
                }
            }
        }

        Ok(())
    }

    /// Collects all files in a tree as a map of path -> content.
    ///
    /// Uses `walk_tree()` internally and reads blob content for each file entry.
    ///
    /// # Arguments
    ///
    /// * `tree_id` - The object ID of the tree
    ///
    /// # Returns
    ///
    /// A HashMap mapping file paths to their content as bytes.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let files = repo.list_tree_files(tree_id)?;
    /// for (path, content) in files {
    ///     println!("{}: {} bytes", path, content.len());
    /// }
    /// ```
    pub fn list_tree_files(
        &self,
        tree_id: git2::Oid,
    ) -> Result<std::collections::HashMap<String, Vec<u8>>> {
        let mut files = std::collections::HashMap::new();

        self.walk_tree(tree_id, |path, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob) {
                let blob = self.find_blob(entry.id())?;
                files.insert(path.to_string(), blob.content().to_vec());
            }
            Ok(())
        })?;

        Ok(files)
    }

    /// Finds a specific file path in a tree.
    ///
    /// Searches recursively through the tree for the given path.
    /// Returns the blob object ID if found, None if not found.
    ///
    /// # Arguments
    ///
    /// * `tree_id` - The object ID of the tree to search
    /// * `path` - The file path to find (e.g., "subdir/file.txt")
    ///
    /// # Returns
    ///
    /// * `Ok(Some(oid))` - File found, returns blob object ID
    /// * `Ok(None)` - File not found
    /// * `Err(JinError)` - Tree walking failed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(blob_oid) = repo.find_in_tree(tree_id, "config.json")? {
    ///     let blob = repo.find_blob(blob_oid)?;
    ///     let content = blob.content();
    /// }
    /// ```
    pub fn find_in_tree(&self, tree_id: git2::Oid, target_path: &str) -> Result<Option<git2::Oid>> {
        let result = std::cell::RefCell::new(None);

        self.walk_tree(tree_id, |path, entry| {
            if path == target_path && entry.kind() == Some(git2::ObjectType::Blob) {
                *result.borrow_mut() = Some(entry.id());
            }
            Ok(())
        })?;

        Ok(result.into_inner())
    }
}

// ===== Helper Methods =====

impl JinRepo {
    /// Returns `true` if this is a bare repository.
    ///
    /// Jin repositories are always bare (no working directory).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// assert!(repo.is_bare());
    /// ```
    pub fn is_bare(&self) -> bool {
        self.inner.is_bare()
    }

    /// Returns the path to the repository `.git` directory.
    ///
    /// For bare repositories, this is the repository path itself.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// println!("Repository path: {}", repo.path().display());
    /// ```
    pub fn path(&self) -> &Path {
        self.inner.path()
    }

    /// Creates a new signature with the current time.
    ///
    /// # Arguments
    ///
    /// * `name` - The name (e.g., "User Name")
    /// * `email` - The email address (e.g., "user@example.com")
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let author = repo.signature("User", "user@example.com")?;
    /// ```
    pub fn signature(&self, name: &str, email: &str) -> Result<git2::Signature> {
        Ok(git2::Signature::now(name, email)?)
    }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ===== Test Fixture =====

    struct TestFixture {
        _temp_dir: TempDir,
        repo: JinRepo,
    }

    impl TestFixture {
        fn new() -> Self {
            let temp_dir = TempDir::new().unwrap();
            let repo = JinRepo::init(temp_dir.path()).unwrap();
            Self {
                _temp_dir: temp_dir,
                repo,
            }
        }

        fn create_initial_commit(&self) -> git2::Oid {
            let tree_builder = self.repo.treebuilder().unwrap();
            let tree_oid = tree_builder.write().unwrap();
            let tree = self.repo.find_tree(tree_oid).unwrap();

            let author = self
                .repo
                .signature("Test Author", "test@example.com")
                .unwrap();
            let committer = &author;

            self.repo
                .create_commit(
                    Some("HEAD"),
                    &author,
                    committer,
                    "Initial commit",
                    &tree,
                    &[],
                )
                .unwrap()
        }
    }

    // ===== Constructor Tests =====

    #[test]
    fn test_jinrepo_init_creates_bare_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();

        assert!(repo.is_bare());
        assert!(repo.path().starts_with(temp_dir.path()));
    }

    #[test]
    fn test_jinrepo_open_existing_repo() {
        let temp_dir = TempDir::new().unwrap();
        let _repo = JinRepo::init(temp_dir.path()).unwrap();

        // Opening existing repo should succeed
        let repo2 = JinRepo::open(temp_dir.path()).unwrap();
        assert!(repo2.is_bare());
    }

    #[test]
    fn test_jinrepo_open_nonexistent_errors() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent");

        let result = JinRepo::open(&nonexistent);
        assert!(matches!(result, Err(JinError::RepoNotFound { .. })));
    }

    #[test]
    fn test_jinrepo_open_or_create_new() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("new_repo");

        // First call should create
        let repo = JinRepo::open_or_create(&path).unwrap();
        assert!(repo.is_bare());
    }

    #[test]
    fn test_jinrepo_open_or_create_existing() {
        let temp_dir = TempDir::new().unwrap();
        let _repo = JinRepo::init(temp_dir.path()).unwrap();

        // Second call should open existing
        let repo2 = JinRepo::open_or_create(temp_dir.path()).unwrap();
        assert!(repo2.is_bare());
    }

    // ===== Layer Reference Tests =====

    #[test]
    fn test_jinrepo_get_layer_ref_not_found() {
        let fixture = TestFixture::new();

        // Non-existent layer should return None
        let result = fixture.repo.get_layer_ref(&Layer::GlobalBase).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_jinrepo_set_layer_ref() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        // Set layer ref
        let reference = fixture
            .repo
            .set_layer_ref(&Layer::GlobalBase, commit_oid)
            .unwrap();

        assert_eq!(reference.target(), Some(commit_oid));
    }

    #[test]
    fn test_jinrepo_set_layer_ref_updates() {
        let fixture = TestFixture::new();
        let commit1 = fixture.create_initial_commit();

        // Create a second commit with the first as parent
        let tree_builder = fixture.repo.treebuilder().unwrap();
        let tree_oid = tree_builder.write().unwrap();
        let tree = fixture.repo.find_tree(tree_oid).unwrap();
        let parent_commit = fixture.repo.find_commit(commit1).unwrap();

        let author = fixture
            .repo
            .signature("Test Author", "test@example.com")
            .unwrap();
        let committer = &author;

        let commit2 = fixture
            .repo
            .create_commit(
                Some("HEAD"),
                &author,
                committer,
                "Second commit",
                &tree,
                &[&parent_commit],
            )
            .unwrap();

        // Set layer ref
        fixture
            .repo
            .set_layer_ref(&Layer::GlobalBase, commit1)
            .unwrap();

        // Update to different commit
        let reference = fixture
            .repo
            .set_layer_ref(&Layer::GlobalBase, commit2)
            .unwrap();

        assert_eq!(reference.target(), Some(commit2));
    }

    #[test]
    fn test_jinrepo_create_layer_ref() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        // Create layer ref
        let reference = fixture
            .repo
            .create_layer_ref(&Layer::GlobalBase, commit_oid)
            .unwrap();

        assert_eq!(reference.target(), Some(commit_oid));
    }

    #[test]
    fn test_jinrepo_create_layer_ref_fails_if_exists() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        // First call should succeed
        fixture
            .repo
            .create_layer_ref(&Layer::GlobalBase, commit_oid)
            .unwrap();

        // Second call should fail
        let result = fixture
            .repo
            .create_layer_ref(&Layer::GlobalBase, commit_oid);
        assert!(matches!(result, Err(JinError::RefExists { .. })));
    }

    #[test]
    fn test_jinrepo_unversioned_layers_error() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        // UserLocal should error
        let result = fixture.repo.set_layer_ref(&Layer::UserLocal, commit_oid);
        assert!(matches!(result, Err(JinError::InvalidLayer { .. })));

        // WorkspaceActive should error
        let result = fixture
            .repo
            .set_layer_ref(&Layer::WorkspaceActive, commit_oid);
        assert!(matches!(result, Err(JinError::InvalidLayer { .. })));
    }

    // ===== Object Creation Tests =====

    #[test]
    fn test_jinrepo_create_blob() {
        let fixture = TestFixture::new();
        let data = b"Hello, World!";

        let blob_oid = fixture.repo.create_blob(data).unwrap();
        let blob = fixture.repo.find_blob(blob_oid).unwrap();

        assert_eq!(blob.content(), data);
    }

    #[test]
    fn test_jinrepo_create_tree() {
        let fixture = TestFixture::new();
        let blob_oid = fixture.repo.create_blob(b"content").unwrap();

        let mut builder = fixture.repo.treebuilder().unwrap();
        builder
            .insert("file.txt", blob_oid, git2::FileMode::Blob.into())
            .unwrap();

        let tree_oid = fixture.repo.create_tree(&mut builder).unwrap();
        let tree = fixture.repo.find_tree(tree_oid).unwrap();

        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn test_jinrepo_create_commit() {
        let fixture = TestFixture::new();
        let tree_builder = fixture.repo.treebuilder().unwrap();
        let tree_oid = tree_builder.write().unwrap();
        let tree = fixture.repo.find_tree(tree_oid).unwrap();

        let author = fixture
            .repo
            .signature("Test Author", "test@example.com")
            .unwrap();
        let committer = &author;

        let commit_oid = fixture
            .repo
            .create_commit(Some("HEAD"), &author, committer, "Test commit", &tree, &[])
            .unwrap();

        let commit = fixture.repo.find_commit(commit_oid).unwrap();
        assert_eq!(commit.message().unwrap(), "Test commit");
    }

    // ===== Delegation Method Tests =====

    #[test]
    fn test_jinrepo_head() {
        let fixture = TestFixture::new();
        let _commit_oid = fixture.create_initial_commit();

        let head = fixture.repo.head().unwrap();
        // HEAD can be retrieved after creating a commit
        // In bare repos, HEAD may be a direct reference or symbolic
        // The important thing is we can access it and it points to a commit
        assert!(head.target().is_some());
    }

    #[test]
    fn test_jinrepo_find_commit() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        let commit = fixture.repo.find_commit(commit_oid).unwrap();
        assert_eq!(commit.id(), commit_oid);
    }

    #[test]
    fn test_jinrepo_find_tree() {
        let fixture = TestFixture::new();
        let tree_builder = fixture.repo.treebuilder().unwrap();
        let tree_oid = tree_builder.write().unwrap();

        let tree = fixture.repo.find_tree(tree_oid).unwrap();
        assert_eq!(tree.id(), tree_oid);
    }

    #[test]
    fn test_jinrepo_find_blob() {
        let fixture = TestFixture::new();
        let blob_oid = fixture.repo.create_blob(b"test").unwrap();

        let blob = fixture.repo.find_blob(blob_oid).unwrap();
        assert_eq!(blob.id(), blob_oid);
    }

    #[test]
    fn test_jinrepo_treebuilder() {
        let fixture = TestFixture::new();
        let _builder = fixture.repo.treebuilder().unwrap();

        // Tree builder was created successfully
        assert!(true);
    }

    // ===== Helper Method Tests =====

    #[test]
    fn test_jinrepo_is_bare() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();

        assert!(repo.is_bare());
    }

    #[test]
    fn test_jinrepo_path() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JinRepo::init(temp_dir.path()).unwrap();

        assert!(repo.path().starts_with(temp_dir.path()));
    }

    #[test]
    fn test_jinrepo_signature() {
        let fixture = TestFixture::new();
        let sig = fixture
            .repo
            .signature("Test User", "test@example.com")
            .unwrap();

        assert_eq!(sig.name().unwrap(), "Test User");
        assert_eq!(sig.email().unwrap(), "test@example.com");
    }

    // ===== Layer Integration Tests =====

    #[test]
    fn test_jinrepo_layer_git_ref_integration() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        // Test GlobalBase ref format
        let reference = fixture
            .repo
            .set_layer_ref(&Layer::GlobalBase, commit_oid)
            .unwrap();
        assert_eq!(reference.name().unwrap(), "refs/jin/layers/global");

        // Test ModeBase ref format
        let reference = fixture
            .repo
            .set_layer_ref(
                &Layer::ModeBase {
                    mode: "claude".to_string(),
                },
                commit_oid,
            )
            .unwrap();
        assert_eq!(reference.name().unwrap(), "refs/jin/layers/mode/claude");

        // Test ScopeBase ref format
        let reference = fixture
            .repo
            .set_layer_ref(
                &Layer::ScopeBase {
                    scope: "python".to_string(),
                },
                commit_oid,
            )
            .unwrap();
        assert_eq!(reference.name().unwrap(), "refs/jin/layers/scope/python");

        // Test ProjectBase ref format
        let reference = fixture
            .repo
            .set_layer_ref(
                &Layer::ProjectBase {
                    project: "myapp".to_string(),
                },
                commit_oid,
            )
            .unwrap();
        assert_eq!(reference.name().unwrap(), "refs/jin/layers/project/myapp");
    }

    // ===== Error Conversion Tests =====

    #[test]
    fn test_jinrepo_error_conversion() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("nonexistent");

        // Test RepoNotFound
        let result = JinRepo::open(&nonexistent);
        assert!(matches!(result, Err(JinError::RepoNotFound { .. })));

        // Test InvalidLayer for UserLocal
        let repo = JinRepo::init(temp_dir.path()).unwrap();
        let commit_oid = repo.create_blob(b"test").unwrap();
        let result = repo.set_layer_ref(&Layer::UserLocal, commit_oid);
        assert!(matches!(result, Err(JinError::InvalidLayer { .. })));

        // Test RefExists
        let _ = repo
            .create_layer_ref(&Layer::GlobalBase, commit_oid)
            .unwrap();
        let result = repo.create_layer_ref(&Layer::GlobalBase, commit_oid);
        assert!(matches!(result, Err(JinError::RefExists { .. })));
    }

    // ===== Delete Layer Ref Tests =====

    #[test]
    fn test_jinrepo_delete_layer_ref() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        // Create a layer ref first
        fixture
            .repo
            .create_layer_ref(&Layer::GlobalBase, commit_oid)
            .unwrap();

        // Verify it exists
        assert!(fixture.repo.layer_ref_exists(&Layer::GlobalBase));

        // Delete it
        fixture.repo.delete_layer_ref(&Layer::GlobalBase).unwrap();

        // Verify it's gone
        assert!(!fixture.repo.layer_ref_exists(&Layer::GlobalBase));
    }

    #[test]
    fn test_jinrepo_delete_layer_ref_not_found_errors() {
        let fixture = TestFixture::new();

        // Deleting non-existent ref should error
        let result = fixture.repo.delete_layer_ref(&Layer::GlobalBase);
        assert!(matches!(result, Err(JinError::RefNotFound { .. })));
    }

    #[test]
    fn test_jinrepo_delete_unversioned_layer_errors() {
        let fixture = TestFixture::new();

        // UserLocal should error
        let result = fixture.repo.delete_layer_ref(&Layer::UserLocal);
        assert!(matches!(result, Err(JinError::InvalidLayer { .. })));

        // WorkspaceActive should error
        let result = fixture.repo.delete_layer_ref(&Layer::WorkspaceActive);
        assert!(matches!(result, Err(JinError::InvalidLayer { .. })));
    }

    // ===== Layer Ref Exists Tests =====

    #[test]
    fn test_jinrepo_layer_ref_exists_true() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        // Create a layer ref
        fixture
            .repo
            .set_layer_ref(&Layer::GlobalBase, commit_oid)
            .unwrap();

        // Should exist now
        assert!(fixture.repo.layer_ref_exists(&Layer::GlobalBase));
    }

    #[test]
    fn test_jinrepo_layer_ref_exists_false() {
        let fixture = TestFixture::new();

        // Non-existent layer should not exist
        assert!(!fixture.repo.layer_ref_exists(&Layer::GlobalBase));
    }

    #[test]
    fn test_jinrepo_layer_ref_exists_unversioned_layers() {
        let fixture = TestFixture::new();

        // UserLocal should return false (never exists as ref)
        assert!(!fixture.repo.layer_ref_exists(&Layer::UserLocal));

        // WorkspaceActive should return false (never exists as ref)
        assert!(!fixture.repo.layer_ref_exists(&Layer::WorkspaceActive));
    }

    // ===== List Layer Refs Tests =====

    #[test]
    fn test_jinrepo_list_layer_refs_empty() {
        let fixture = TestFixture::new();

        // Empty repo should have no layer refs
        let refs = fixture.repo.list_layer_refs().unwrap();
        assert!(refs.is_empty());
    }

    #[test]
    fn test_jinrepo_list_layer_refs_multiple() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        // Create multiple layer refs
        fixture
            .repo
            .set_layer_ref(&Layer::GlobalBase, commit_oid)
            .unwrap();
        fixture
            .repo
            .set_layer_ref(
                &Layer::ModeBase {
                    mode: "claude".to_string(),
                },
                commit_oid,
            )
            .unwrap();
        fixture
            .repo
            .set_layer_ref(
                &Layer::ScopeBase {
                    scope: "python".to_string(),
                },
                commit_oid,
            )
            .unwrap();
        fixture
            .repo
            .set_layer_ref(
                &Layer::ProjectBase {
                    project: "myapp".to_string(),
                },
                commit_oid,
            )
            .unwrap();

        // List all refs
        let refs = fixture.repo.list_layer_refs().unwrap();
        assert_eq!(refs.len(), 4);

        // Verify all layers are present
        let layer_names: Vec<_> = refs.iter().map(|(l, _)| format!("{:?}", l)).collect();
        assert!(layer_names.contains(&"GlobalBase".to_string()));
        assert!(layer_names.contains(&"ModeBase { mode: \"claude\" }".to_string()));
        assert!(layer_names.contains(&"ScopeBase { scope: \"python\" }".to_string()));
        assert!(layer_names.contains(&"ProjectBase { project: \"myapp\" }".to_string()));
    }

    #[test]
    fn test_jinrepo_list_layer_refs_by_pattern() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        // Create refs for different layers
        fixture
            .repo
            .set_layer_ref(&Layer::GlobalBase, commit_oid)
            .unwrap();
        fixture
            .repo
            .set_layer_ref(
                &Layer::ModeBase {
                    mode: "claude".to_string(),
                },
                commit_oid,
            )
            .unwrap();
        fixture
            .repo
            .set_layer_ref(
                &Layer::ModeBase {
                    mode: "cursor".to_string(),
                },
                commit_oid,
            )
            .unwrap();
        fixture
            .repo
            .set_layer_ref(
                &Layer::ScopeBase {
                    scope: "python".to_string(),
                },
                commit_oid,
            )
            .unwrap();

        // List all mode refs
        let mode_refs = fixture
            .repo
            .list_layer_refs_by_pattern("refs/jin/layers/mode/*")
            .unwrap();
        assert_eq!(mode_refs.len(), 2);
        assert!(mode_refs.contains(&"refs/jin/layers/mode/claude".to_string()));
        assert!(mode_refs.contains(&"refs/jin/layers/mode/cursor".to_string()));

        // List all scope refs
        let scope_refs = fixture
            .repo
            .list_layer_refs_by_pattern("refs/jin/layers/scope/*")
            .unwrap();
        assert_eq!(scope_refs.len(), 1);
        assert!(scope_refs.contains(&"refs/jin/layers/scope/python".to_string()));
    }

    // ===== ref_name_to_layer Tests =====

    #[test]
    fn test_jinrepo_ref_name_to_layer_parsing() {
        // Test all layer type parsing
        assert_eq!(
            JinRepo::ref_name_to_layer("refs/jin/layers/global"),
            Some(Layer::GlobalBase)
        );

        assert_eq!(
            JinRepo::ref_name_to_layer("refs/jin/layers/mode/claude"),
            Some(Layer::ModeBase {
                mode: "claude".to_string()
            })
        );

        assert_eq!(
            JinRepo::ref_name_to_layer("refs/jin/layers/mode/claude/scope/python"),
            Some(Layer::ModeScope {
                mode: "claude".to_string(),
                scope: "python".to_string()
            })
        );

        assert_eq!(
            JinRepo::ref_name_to_layer("refs/jin/layers/mode/claude/scope/python/project/myapp"),
            Some(Layer::ModeScopeProject {
                mode: "claude".to_string(),
                scope: "python".to_string(),
                project: "myapp".to_string()
            })
        );

        assert_eq!(
            JinRepo::ref_name_to_layer("refs/jin/layers/mode/claude/project/myapp"),
            Some(Layer::ModeProject {
                mode: "claude".to_string(),
                project: "myapp".to_string()
            })
        );

        assert_eq!(
            JinRepo::ref_name_to_layer("refs/jin/layers/scope/python"),
            Some(Layer::ScopeBase {
                scope: "python".to_string()
            })
        );

        assert_eq!(
            JinRepo::ref_name_to_layer("refs/jin/layers/project/myapp"),
            Some(Layer::ProjectBase {
                project: "myapp".to_string()
            })
        );

        // Invalid refs should return None
        assert_eq!(JinRepo::ref_name_to_layer("refs/heads/main"), None);
        assert_eq!(JinRepo::ref_name_to_layer("refs/jin/layers/invalid"), None);
        assert_eq!(JinRepo::ref_name_to_layer("refs/jin/staging/abc123"), None);
    }

    // ===== Staging Ref Tests =====

    #[test]
    fn test_jinrepo_create_staging_ref() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        // Create a staging ref
        let staging_ref = fixture
            .repo
            .create_staging_ref("txn-abc123", commit_oid)
            .unwrap();

        assert_eq!(staging_ref.name().unwrap(), "refs/jin/staging/txn-abc123");
        assert_eq!(staging_ref.target(), Some(commit_oid));
    }

    #[test]
    fn test_jinrepo_create_staging_ref_fails_if_exists() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        // First call should succeed
        fixture
            .repo
            .create_staging_ref("txn-abc123", commit_oid)
            .unwrap();

        // Second call should fail
        let result = fixture.repo.create_staging_ref("txn-abc123", commit_oid);
        assert!(matches!(result, Err(JinError::RefExists { .. })));
    }

    #[test]
    fn test_jinrepo_delete_staging_ref() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        // Create a staging ref first
        fixture
            .repo
            .create_staging_ref("txn-abc123", commit_oid)
            .unwrap();

        // Verify it exists
        assert!(fixture.repo.staging_ref_exists("txn-abc123"));

        // Delete it
        fixture.repo.delete_staging_ref("txn-abc123").unwrap();

        // Verify it's gone
        assert!(!fixture.repo.staging_ref_exists("txn-abc123"));
    }

    #[test]
    fn test_jinrepo_delete_staging_ref_not_found_errors() {
        let fixture = TestFixture::new();

        // Deleting non-existent staging ref should error
        let result = fixture.repo.delete_staging_ref("txn-nonexistent");
        assert!(matches!(result, Err(JinError::RefNotFound { .. })));
    }

    #[test]
    fn test_jinrepo_staging_ref_exists_true() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        // Create a staging ref
        fixture
            .repo
            .create_staging_ref("txn-abc123", commit_oid)
            .unwrap();

        // Should exist
        assert!(fixture.repo.staging_ref_exists("txn-abc123"));
    }

    #[test]
    fn test_jinrepo_staging_ref_exists_false() {
        let fixture = TestFixture::new();

        // Non-existent staging ref should not exist
        assert!(!fixture.repo.staging_ref_exists("txn-abc123"));
    }

    // ===== Integration Tests =====

    #[test]
    fn test_jinrepo_layer_ref_crud_cycle() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        // Create
        fixture
            .repo
            .create_layer_ref(&Layer::GlobalBase, commit_oid)
            .unwrap();

        // Read
        assert!(fixture.repo.layer_ref_exists(&Layer::GlobalBase));
        let found = fixture.repo.get_layer_ref(&Layer::GlobalBase).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().target(), Some(commit_oid));

        // List
        let refs = fixture.repo.list_layer_refs().unwrap();
        assert_eq!(refs.len(), 1);

        // Delete
        fixture.repo.delete_layer_ref(&Layer::GlobalBase).unwrap();

        // Verify gone
        assert!(!fixture.repo.layer_ref_exists(&Layer::GlobalBase));
        let refs = fixture.repo.list_layer_refs().unwrap();
        assert!(refs.is_empty());
    }

    #[test]
    fn test_jinrepo_staging_ref_crud_cycle() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        // Create
        fixture
            .repo
            .create_staging_ref("txn-test", commit_oid)
            .unwrap();

        // Exists
        assert!(fixture.repo.staging_ref_exists("txn-test"));

        // Delete
        fixture.repo.delete_staging_ref("txn-test").unwrap();

        // Verify gone
        assert!(!fixture.repo.staging_ref_exists("txn-test"));
    }

    #[test]
    fn test_jinrepo_staging_and_layer_refs_isolated() {
        let fixture = TestFixture::new();
        let commit_oid = fixture.create_initial_commit();

        // Create both layer and staging refs
        fixture
            .repo
            .create_layer_ref(&Layer::GlobalBase, commit_oid)
            .unwrap();
        fixture
            .repo
            .create_staging_ref("txn-abc123", commit_oid)
            .unwrap();

        // Both should exist independently
        assert!(fixture.repo.layer_ref_exists(&Layer::GlobalBase));
        assert!(fixture.repo.staging_ref_exists("txn-abc123"));

        // List layer refs should not include staging refs
        let refs = fixture.repo.list_layer_refs().unwrap();
        assert_eq!(refs.len(), 1);

        // Delete staging ref should not affect layer ref
        fixture.repo.delete_staging_ref("txn-abc123").unwrap();
        assert!(fixture.repo.layer_ref_exists(&Layer::GlobalBase));
    }

    // ===== Tree Walking Tests =====

    #[test]
    fn test_jinrepo_walk_empty_tree() {
        let fixture = TestFixture::new();
        let tree_oid = fixture.repo.treebuilder().unwrap().write().unwrap();

        // Walk empty tree should complete without error
        let mut count = 0;
        fixture
            .repo
            .walk_tree(tree_oid, |_path, _entry| {
                count += 1;
                Ok(())
            })
            .unwrap();

        assert_eq!(count, 0);
    }

    #[test]
    fn test_jinrepo_walk_single_file_tree() {
        let fixture = TestFixture::new();
        let mut builder = fixture.repo.treebuilder().unwrap();
        let blob_oid = fixture.repo.create_blob(b"content").unwrap();
        builder
            .insert("file.txt", blob_oid, git2::FileMode::Blob.into())
            .unwrap();
        let tree_oid = builder.write().unwrap();

        let mut entries = Vec::new();
        fixture
            .repo
            .walk_tree(tree_oid, |path, entry| {
                entries.push((path.to_string(), entry.id()));
                Ok(())
            })
            .unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "file.txt");
        assert_eq!(entries[0].1, blob_oid);
    }

    #[test]
    fn test_jinrepo_walk_nested_tree() {
        let fixture = TestFixture::new();

        // Create nested tree structure
        let mut subdir_builder = fixture.repo.treebuilder().unwrap();
        let sub_blob = fixture.repo.create_blob(b"sub content").unwrap();
        subdir_builder
            .insert("sub.txt", sub_blob, git2::FileMode::Blob.into())
            .unwrap();
        let subdir_oid = subdir_builder.write().unwrap();

        let mut root_builder = fixture.repo.treebuilder().unwrap();
        let root_blob = fixture.repo.create_blob(b"root content").unwrap();
        root_builder
            .insert("root.txt", root_blob, git2::FileMode::Blob.into())
            .unwrap();
        root_builder
            .insert("subdir", subdir_oid, git2::FileMode::Tree.into())
            .unwrap();
        let tree_oid = root_builder.write().unwrap();

        let mut paths = Vec::new();
        fixture
            .repo
            .walk_tree(tree_oid, |path, _entry| {
                paths.push(path.to_string());
                Ok(())
            })
            .unwrap();

        paths.sort();
        assert_eq!(paths, &["root.txt", "subdir/sub.txt"]);
    }

    #[test]
    fn test_jinrepo_list_tree_files() {
        let fixture = TestFixture::new();
        let mut builder = fixture.repo.treebuilder().unwrap();
        let blob1 = fixture.repo.create_blob(b"content 1").unwrap();
        let blob2 = fixture.repo.create_blob(b"content 2").unwrap();
        builder
            .insert("a.txt", blob1, git2::FileMode::Blob.into())
            .unwrap();
        builder
            .insert("b.txt", blob2, git2::FileMode::Blob.into())
            .unwrap();
        let tree_oid = builder.write().unwrap();

        let files = fixture.repo.list_tree_files(tree_oid).unwrap();

        assert_eq!(files.len(), 2);
        assert_eq!(files.get("a.txt"), Some(&b"content 1".to_vec()));
        assert_eq!(files.get("b.txt"), Some(&b"content 2".to_vec()));
    }

    #[test]
    fn test_jinrepo_list_nested_tree_files() {
        let fixture = TestFixture::new();

        // Create nested tree structure
        let mut subdir_builder = fixture.repo.treebuilder().unwrap();
        let sub_blob = fixture.repo.create_blob(b"sub file content").unwrap();
        subdir_builder
            .insert("sub.txt", sub_blob, git2::FileMode::Blob.into())
            .unwrap();
        let subdir_oid = subdir_builder.write().unwrap();

        let mut root_builder = fixture.repo.treebuilder().unwrap();
        let root_blob = fixture.repo.create_blob(b"root file content").unwrap();
        root_builder
            .insert("root.txt", root_blob, git2::FileMode::Blob.into())
            .unwrap();
        root_builder
            .insert("subdir", subdir_oid, git2::FileMode::Tree.into())
            .unwrap();
        let tree_oid = root_builder.write().unwrap();

        let files = fixture.repo.list_tree_files(tree_oid).unwrap();

        assert_eq!(files.len(), 2);
        assert_eq!(files.get("root.txt"), Some(&b"root file content".to_vec()));
        assert_eq!(
            files.get("subdir/sub.txt"),
            Some(&b"sub file content".to_vec())
        );
    }

    #[test]
    fn test_jinrepo_find_in_tree() {
        let fixture = TestFixture::new();

        // Create nested tree
        let mut subdir_builder = fixture.repo.treebuilder().unwrap();
        let sub_blob = fixture.repo.create_blob(b"sub content").unwrap();
        subdir_builder
            .insert("sub.txt", sub_blob, git2::FileMode::Blob.into())
            .unwrap();
        let subdir_oid = subdir_builder.write().unwrap();

        let mut root_builder = fixture.repo.treebuilder().unwrap();
        let root_blob = fixture.repo.create_blob(b"root content").unwrap();
        root_builder
            .insert("root.txt", root_blob, git2::FileMode::Blob.into())
            .unwrap();
        root_builder
            .insert("subdir", subdir_oid, git2::FileMode::Tree.into())
            .unwrap();
        let tree_oid = root_builder.write().unwrap();

        // Find root file
        let found = fixture.repo.find_in_tree(tree_oid, "root.txt").unwrap();
        assert_eq!(found, Some(root_blob));

        // Find nested file
        let found = fixture
            .repo
            .find_in_tree(tree_oid, "subdir/sub.txt")
            .unwrap();
        assert_eq!(found, Some(sub_blob));

        // Not found
        let found = fixture.repo.find_in_tree(tree_oid, "missing.txt").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_jinrepo_walk_tree_deeply_nested() {
        let fixture = TestFixture::new();

        // Create deeply nested structure: a/b/c/d/file.txt
        let mut d_builder = fixture.repo.treebuilder().unwrap();
        let file_blob = fixture.repo.create_blob(b"deep file").unwrap();
        d_builder
            .insert("file.txt", file_blob, git2::FileMode::Blob.into())
            .unwrap();
        let d_oid = d_builder.write().unwrap();

        let mut c_builder = fixture.repo.treebuilder().unwrap();
        c_builder
            .insert("d", d_oid, git2::FileMode::Tree.into())
            .unwrap();
        let c_oid = c_builder.write().unwrap();

        let mut b_builder = fixture.repo.treebuilder().unwrap();
        b_builder
            .insert("c", c_oid, git2::FileMode::Tree.into())
            .unwrap();
        let b_oid = b_builder.write().unwrap();

        let mut a_builder = fixture.repo.treebuilder().unwrap();
        a_builder
            .insert("b", b_oid, git2::FileMode::Tree.into())
            .unwrap();
        let a_oid = a_builder.write().unwrap();

        // Walk the deeply nested tree
        let mut paths = Vec::new();
        fixture
            .repo
            .walk_tree(a_oid, |path, _entry| {
                paths.push(path.to_string());
                Ok(())
            })
            .unwrap();

        assert_eq!(paths, &["b/c/d/file.txt"]);
    }

    #[test]
    fn test_jinrepo_walk_tree_multiple_files_same_level() {
        let fixture = TestFixture::new();

        // Create tree with multiple files at same level
        let mut builder = fixture.repo.treebuilder().unwrap();
        let blob1 = fixture.repo.create_blob(b"content 1").unwrap();
        let blob2 = fixture.repo.create_blob(b"content 2").unwrap();
        let blob3 = fixture.repo.create_blob(b"content 3").unwrap();
        builder
            .insert("a.txt", blob1, git2::FileMode::Blob.into())
            .unwrap();
        builder
            .insert("b.txt", blob2, git2::FileMode::Blob.into())
            .unwrap();
        builder
            .insert("c.txt", blob3, git2::FileMode::Blob.into())
            .unwrap();
        let tree_oid = builder.write().unwrap();

        let mut paths = Vec::new();
        fixture
            .repo
            .walk_tree(tree_oid, |path, _entry| {
                paths.push(path.to_string());
                Ok(())
            })
            .unwrap();

        paths.sort();
        assert_eq!(paths, &["a.txt", "b.txt", "c.txt"]);
    }

    #[test]
    fn test_jinrepo_walk_tree_empty_subdirectory() {
        let fixture = TestFixture::new();

        // Create tree with an empty subdirectory
        let empty_builder = fixture.repo.treebuilder().unwrap();
        let empty_oid = empty_builder.write().unwrap();

        let mut root_builder = fixture.repo.treebuilder().unwrap();
        let root_blob = fixture.repo.create_blob(b"root content").unwrap();
        root_builder
            .insert("root.txt", root_blob, git2::FileMode::Blob.into())
            .unwrap();
        root_builder
            .insert("empty", empty_oid, git2::FileMode::Tree.into())
            .unwrap();
        let tree_oid = root_builder.write().unwrap();

        // Walk should only find root.txt, empty dir has no files
        let mut paths = Vec::new();
        fixture
            .repo
            .walk_tree(tree_oid, |path, _entry| {
                paths.push(path.to_string());
                Ok(())
            })
            .unwrap();

        assert_eq!(paths, &["root.txt"]);
    }

    #[test]
    fn test_jinrepo_find_in_tree_empty_tree() {
        let fixture = TestFixture::new();
        let tree_oid = fixture.repo.treebuilder().unwrap().write().unwrap();

        // Finding in empty tree should return None
        let found = fixture.repo.find_in_tree(tree_oid, "anything").unwrap();
        assert!(found.is_none());
    }
}
