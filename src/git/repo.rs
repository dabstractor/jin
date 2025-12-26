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
}
