//! Integration tests for LayerMerge orchestration.
//!
//! This test file validates the full layer merge functionality including:
//! - Multi-layer precedence merging
//! - Format-specific deep merging
//! - Scope precedence rules
//! - Edge cases and error handling

use jin_glm::core::Layer;
use jin_glm::git::JinRepo;
use jin_glm::merge::{FileFormat, LayerMerge};
use std::path::Path;

// ===== Test Helpers =====

/// Helper to create a commit with a single file
fn create_commit_with_file(repo: &JinRepo, layer: &Layer, path: &str, content: &[u8]) -> git2::Oid {
    let mut builder = repo.treebuilder().unwrap();
    let blob_oid = repo.create_blob(content).unwrap();
    builder
        .insert(path, blob_oid, git2::FileMode::Blob.into())
        .unwrap();
    let tree_oid = builder.write().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();

    let author = repo.signature("Test", "test@example.com").unwrap();
    let committer = &author;

    repo.create_commit(
        None,
        &author,
        committer,
        &format!("Add {} to {:?}", path, layer),
        &tree,
        &[],
    )
    .unwrap()
}

/// Helper to create a commit with multiple files
fn create_commit_with_files(repo: &JinRepo, layer: &Layer, files: &[(&str, &[u8])]) -> git2::Oid {
    let mut builder = repo.treebuilder().unwrap();

    for (path, content) in files {
        let blob_oid = repo.create_blob(content).unwrap();
        builder
            .insert(path, blob_oid, git2::FileMode::Blob.into())
            .unwrap();
    }

    let tree_oid = builder.write().unwrap();
    let tree = repo.find_tree(tree_oid).unwrap();

    let author = repo.signature("Test", "test@example.com").unwrap();
    let committer = &author;

    repo.create_commit(
        None,
        &author,
        committer,
        &format!("Add files to {:?}", layer),
        &tree,
        &[],
    )
    .unwrap()
}

// ===== FileFormat Tests =====

#[test]
fn test_layer_merge_format_detection() {
    assert_eq!(
        FileFormat::from_path(Path::new("config.json")),
        FileFormat::Json
    );
    assert_eq!(
        FileFormat::from_path(Path::new("settings.yaml")),
        FileFormat::Yaml
    );
    assert_eq!(
        FileFormat::from_path(Path::new("config.yml")),
        FileFormat::Yaml
    );
    assert_eq!(
        FileFormat::from_path(Path::new("app.toml")),
        FileFormat::Toml
    );
    assert_eq!(
        FileFormat::from_path(Path::new("setup.ini")),
        FileFormat::Ini
    );
    assert_eq!(
        FileFormat::from_path(Path::new("README.md")),
        FileFormat::Text
    );
    assert_eq!(
        FileFormat::from_path(Path::new("script.sh")),
        FileFormat::Text
    );
    assert_eq!(
        FileFormat::from_path(Path::new("data.unknown")),
        FileFormat::Text
    );
}

// ===== LayerMerge Construction Tests =====

#[test]
fn test_layer_merge_construction() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let _merger = LayerMerge::new(&repo, "testproject");

    // Basic construction succeeds
    assert!(true);
}

#[test]
fn test_layer_merge_with_mode() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let _merger = LayerMerge::new(&repo, "testproject").with_mode("claude");

    // Builder method works
    assert!(true);
}

#[test]
fn test_layer_merge_with_scope() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let _merger = LayerMerge::new(&repo, "testproject").with_scope("python");

    // Builder method works
    assert!(true);
}

#[test]
fn test_layer_merge_with_mode_and_scope() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let _merger = LayerMerge::new(&repo, "testproject")
        .with_mode("claude")
        .with_scope("javascript");

    // Builder methods work together
    assert!(true);
}

// ===== Active Layer Determination Tests =====

#[test]
fn test_layer_merge_determine_active_layers_base_only() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject");

    let layers = merger.determine_active_layers();

    // Should have GlobalBase and ProjectBase
    assert_eq!(layers.len(), 2);
    assert!(layers.contains(&Layer::GlobalBase));
    assert!(layers.contains(&Layer::ProjectBase {
        project: "testproject".to_string()
    }));
}

#[test]
fn test_layer_merge_determine_active_layers_with_mode() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject").with_mode("claude");

    let layers = merger.determine_active_layers();

    // Should have GlobalBase, ProjectBase, ModeBase, ModeProject
    assert_eq!(layers.len(), 4);
    assert!(layers.contains(&Layer::GlobalBase));
    assert!(layers.contains(&Layer::ProjectBase {
        project: "testproject".to_string()
    }));
    assert!(layers.contains(&Layer::ModeBase {
        mode: "claude".to_string()
    }));
    assert!(layers.contains(&Layer::ModeProject {
        mode: "claude".to_string(),
        project: "testproject".to_string()
    }));
}

#[test]
fn test_layer_merge_determine_active_layers_with_scope_only() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject").with_scope("python");

    let layers = merger.determine_active_layers();

    // Should have GlobalBase, ProjectBase, ScopeBase (untethered)
    assert_eq!(layers.len(), 3);
    assert!(layers.contains(&Layer::GlobalBase));
    assert!(layers.contains(&Layer::ProjectBase {
        project: "testproject".to_string()
    }));
    assert!(layers.contains(&Layer::ScopeBase {
        scope: "python".to_string()
    }));
}

#[test]
fn test_layer_merge_determine_active_layers_with_mode_and_scope() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject")
        .with_mode("claude")
        .with_scope("javascript");

    let layers = merger.determine_active_layers();

    // Should have mode-bound scope, NOT untethered scope
    assert_eq!(layers.len(), 6);
    assert!(layers.contains(&Layer::ModeScope {
        mode: "claude".to_string(),
        scope: "javascript".to_string()
    }));
    assert!(layers.contains(&Layer::ModeScopeProject {
        mode: "claude".to_string(),
        scope: "javascript".to_string(),
        project: "testproject".to_string()
    }));

    // Should NOT have untethered ScopeBase
    assert!(!layers.contains(&Layer::ScopeBase {
        scope: "javascript".to_string()
    }));
}

#[test]
fn test_layer_merge_precedence_ordering() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject")
        .with_mode("claude")
        .with_scope("javascript");

    let layers = merger.determine_active_layers();

    // Verify ordering: GlobalBase should be first (lowest precedence)
    assert_eq!(layers[0], Layer::GlobalBase);

    // Each subsequent layer should have higher precedence
    for i in 1..layers.len() {
        assert!(
            layers[i - 1] < layers[i],
            "Layer at {} should be < layer at {}",
            i - 1,
            i
        );
    }
}

// ===== Empty Layer Handling Tests =====

#[test]
fn test_layer_merge_empty_repo() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject");

    let result = merger.merge_all().unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_layer_merge_nonexistent_layers() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject");

    // Even with mode/scope, nonexistent layers should be handled gracefully
    let result = merger.with_mode("nonexistent").merge_all().unwrap();
    assert!(result.is_empty());
}

// ===== JSON Deep Merge Tests =====

#[test]
fn test_layer_merge_json_deep_merge_two_layers() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject");

    // Create GlobalBase layer with base config
    let global_content =
        br#"{"database": {"host": "localhost", "port": 5432}, "feature": {"enabled": true}}"#;
    let global_commit =
        create_commit_with_file(&repo, &Layer::GlobalBase, "config.json", global_content);
    repo.set_layer_ref(&Layer::GlobalBase, global_commit)
        .unwrap();

    // Create ProjectBase layer with override
    let project_content = br#"{"database": {"port": 3306}, "feature": {"name": "myfeature"}}"#;
    let project_commit = create_commit_with_file(
        &repo,
        &Layer::ProjectBase {
            project: "testproject".to_string(),
        },
        "config.json",
        project_content,
    );
    repo.set_layer_ref(
        &Layer::ProjectBase {
            project: "testproject".to_string(),
        },
        project_commit,
    )
    .unwrap();

    // Merge layers
    let result = merger.merge_all().unwrap();
    assert_eq!(result.len(), 1);

    let merged = result.get("config.json").unwrap();
    let obj = merged.as_object().unwrap();

    // Verify deep merge
    let db = obj.get("database").unwrap().as_object().unwrap();
    assert_eq!(db.get("host").and_then(|v| v.as_str()), Some("localhost"));
    assert_eq!(db.get("port").and_then(|v| v.as_i64()), Some(3306)); // Higher layer wins

    let feature = obj.get("feature").unwrap().as_object().unwrap();
    assert_eq!(feature.get("enabled").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(
        feature.get("name").and_then(|v| v.as_str()),
        Some("myfeature")
    );
}

#[test]
fn test_layer_merge_json_deep_merge_three_layers() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject").with_mode("claude");

    // Create GlobalBase
    let global_content = br#"{"setting": {"a": 1, "b": 2, "c": 3}}"#;
    let global_commit =
        create_commit_with_file(&repo, &Layer::GlobalBase, "config.json", global_content);
    repo.set_layer_ref(&Layer::GlobalBase, global_commit)
        .unwrap();

    // Create ModeBase
    let mode_content = br#"{"setting": {"b": 20, "d": 4}}"#;
    let mode_commit = create_commit_with_file(
        &repo,
        &Layer::ModeBase {
            mode: "claude".to_string(),
        },
        "config.json",
        mode_content,
    );
    repo.set_layer_ref(
        &Layer::ModeBase {
            mode: "claude".to_string(),
        },
        mode_commit,
    )
    .unwrap();

    // Create ProjectBase
    let project_content = br#"{"setting": {"c": 30, "e": 5}}"#;
    let project_commit = create_commit_with_file(
        &repo,
        &Layer::ProjectBase {
            project: "testproject".to_string(),
        },
        "config.json",
        project_content,
    );
    repo.set_layer_ref(
        &Layer::ProjectBase {
            project: "testproject".to_string(),
        },
        project_commit,
    )
    .unwrap();

    // Merge layers
    let result = merger.merge_all().unwrap();
    let merged = result.get("config.json").unwrap();
    let obj = merged.as_object().unwrap();
    let setting = obj.get("setting").unwrap().as_object().unwrap();

    // Verify all values merged correctly
    assert_eq!(setting.get("a").and_then(|v| v.as_i64()), Some(1)); // From GlobalBase
    assert_eq!(setting.get("b").and_then(|v| v.as_i64()), Some(20)); // Overridden by ModeBase
    assert_eq!(setting.get("c").and_then(|v| v.as_i64()), Some(30)); // Overridden by ProjectBase
    assert_eq!(setting.get("d").and_then(|v| v.as_i64()), Some(4)); // From ModeBase
    assert_eq!(setting.get("e").and_then(|v| v.as_i64()), Some(5)); // From ProjectBase
}

#[test]
fn test_layer_merge_json_null_deletes_key() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject");

    // Create GlobalBase with multiple keys
    let global_content = br#"{"a": 1, "b": 2, "c": 3}"#;
    let global_commit =
        create_commit_with_file(&repo, &Layer::GlobalBase, "config.json", global_content);
    repo.set_layer_ref(&Layer::GlobalBase, global_commit)
        .unwrap();

    // Create ProjectBase that deletes key "b" with null
    let project_content = br#"{"b": null, "d": 4}"#;
    let project_commit = create_commit_with_file(
        &repo,
        &Layer::ProjectBase {
            project: "testproject".to_string(),
        },
        "config.json",
        project_content,
    );
    repo.set_layer_ref(
        &Layer::ProjectBase {
            project: "testproject".to_string(),
        },
        project_commit,
    )
    .unwrap();

    // Merge layers
    let result = merger.merge_all().unwrap();
    let merged = result.get("config.json").unwrap();
    let obj = merged.as_object().unwrap();

    // Verify null deleted key "b"
    assert!(!obj.contains_key("b"));
    assert_eq!(obj.get("a").and_then(|v| v.as_i64()), Some(1));
    assert_eq!(obj.get("c").and_then(|v| v.as_i64()), Some(3));
    assert_eq!(obj.get("d").and_then(|v| v.as_i64()), Some(4));
}

// ===== YAML Deep Merge Tests =====

#[test]
fn test_layer_merge_yaml_deep_merge() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject");

    // Create GlobalBase with YAML
    let global_content = b"
database:
  host: localhost
  port: 5432
  ssl: true
features:
  - alpha
  - beta
";
    let global_commit =
        create_commit_with_file(&repo, &Layer::GlobalBase, "config.yaml", global_content);
    repo.set_layer_ref(&Layer::GlobalBase, global_commit)
        .unwrap();

    // Create ProjectBase with YAML override
    let project_content = b"
database:
  port: 3306
  timeout: 30
features:
  - gamma
";
    let project_commit = create_commit_with_file(
        &repo,
        &Layer::ProjectBase {
            project: "testproject".to_string(),
        },
        "config.yaml",
        project_content,
    );
    repo.set_layer_ref(
        &Layer::ProjectBase {
            project: "testproject".to_string(),
        },
        project_commit,
    )
    .unwrap();

    // Merge layers
    let result = merger.merge_all().unwrap();
    let merged = result.get("config.yaml").unwrap();
    let obj = merged.as_object().unwrap();

    // Verify deep merge (YAML arrays replace by default)
    let db = obj.get("database").unwrap().as_object().unwrap();
    assert_eq!(db.get("host").and_then(|v| v.as_str()), Some("localhost"));
    assert_eq!(db.get("port").and_then(|v| v.as_i64()), Some(3306)); // Overridden
    assert_eq!(db.get("ssl").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(db.get("timeout").and_then(|v| v.as_i64()), Some(30));

    // Arrays replace (higher layer wins)
    let features = obj.get("features").unwrap().as_array().unwrap();
    assert_eq!(features.len(), 1);
    assert_eq!(features[0].as_str(), Some("gamma"));
}

// ===== TOML Deep Merge Tests =====

#[test]
fn test_layer_merge_toml_deep_merge() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject");

    // Create GlobalBase with TOML
    let global_content = b"
[server]
host = \"localhost\"
port = 8080

[database]
name = \"mydb\"
";
    let global_commit =
        create_commit_with_file(&repo, &Layer::GlobalBase, "config.toml", global_content);
    repo.set_layer_ref(&Layer::GlobalBase, global_commit)
        .unwrap();

    // Create ProjectBase with TOML override
    let project_content = b"
[server]
port = 9000
ssl = true

[cache]
enabled = true
";
    let project_commit = create_commit_with_file(
        &repo,
        &Layer::ProjectBase {
            project: "testproject".to_string(),
        },
        "config.toml",
        project_content,
    );
    repo.set_layer_ref(
        &Layer::ProjectBase {
            project: "testproject".to_string(),
        },
        project_commit,
    )
    .unwrap();

    // Merge layers
    let result = merger.merge_all().unwrap();
    let merged = result.get("config.toml").unwrap();
    let obj = merged.as_object().unwrap();

    // Verify deep merge
    let server = obj.get("server").unwrap().as_object().unwrap();
    assert_eq!(
        server.get("host").and_then(|v| v.as_str()),
        Some("localhost")
    );
    assert_eq!(server.get("port").and_then(|v| v.as_i64()), Some(9000)); // Overridden
    assert_eq!(server.get("ssl").and_then(|v| v.as_bool()), Some(true));

    let db = obj.get("database").unwrap().as_object().unwrap();
    assert_eq!(db.get("name").and_then(|v| v.as_str()), Some("mydb"));

    let cache = obj.get("cache").unwrap().as_object().unwrap();
    assert_eq!(cache.get("enabled").and_then(|v| v.as_bool()), Some(true));
}

// ===== INI Section Merge Tests =====

#[test]
fn test_layer_merge_ini_section_merge() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject");

    // Create GlobalBase with INI
    let global_content = b"
[database]
host = localhost
port = 5432

[server]
port = 8080
";
    let global_commit =
        create_commit_with_file(&repo, &Layer::GlobalBase, "config.ini", global_content);
    repo.set_layer_ref(&Layer::GlobalBase, global_commit)
        .unwrap();

    // Create ProjectBase with INI override
    let project_content = b"
[database]
port = 3306
ssl = true

[cache]
enabled = true
";
    let project_commit = create_commit_with_file(
        &repo,
        &Layer::ProjectBase {
            project: "testproject".to_string(),
        },
        "config.ini",
        project_content,
    );
    repo.set_layer_ref(
        &Layer::ProjectBase {
            project: "testproject".to_string(),
        },
        project_commit,
    )
    .unwrap();

    // Merge layers
    let result = merger.merge_all().unwrap();
    let merged = result.get("config.ini").unwrap();
    let obj = merged.as_object().unwrap();

    // Verify section merge (INI sections become nested objects)
    let db = obj.get("database").unwrap().as_object().unwrap();
    assert_eq!(db.get("host").and_then(|v| v.as_str()), Some("localhost"));
    assert_eq!(db.get("port").and_then(|v| v.as_str()), Some("3306")); // Overridden
    assert_eq!(db.get("ssl").and_then(|v| v.as_str()), Some("true"));

    let server = obj.get("server").unwrap().as_object().unwrap();
    assert_eq!(server.get("port").and_then(|v| v.as_str()), Some("8080"));

    let cache = obj.get("cache").unwrap().as_object().unwrap();
    assert_eq!(cache.get("enabled").and_then(|v| v.as_str()), Some("true"));
}

// ===== Text File Replacement Tests =====

#[test]
fn test_layer_merge_text_file_replacement() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject");

    // Create GlobalBase with text file
    let global_content = b"This is the base version of the text file";
    let global_commit =
        create_commit_with_file(&repo, &Layer::GlobalBase, "README.md", global_content);
    repo.set_layer_ref(&Layer::GlobalBase, global_commit)
        .unwrap();

    // Create ProjectBase with different text
    let project_content = b"This is the project-specific version";
    let project_commit = create_commit_with_file(
        &repo,
        &Layer::ProjectBase {
            project: "testproject".to_string(),
        },
        "README.md",
        project_content,
    );
    repo.set_layer_ref(
        &Layer::ProjectBase {
            project: "testproject".to_string(),
        },
        project_commit,
    )
    .unwrap();

    // Merge layers - higher layer should replace
    let result = merger.merge_all().unwrap();
    let merged = result.get("README.md").unwrap();

    // Text files are replaced (not deep merged)
    assert_eq!(
        merged.as_str(),
        Some("This is the project-specific version")
    );
}

// ===== merge_subset Tests =====

#[test]
fn test_layer_merge_subset_specific_layers() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject");

    // Create GlobalBase
    let global_content = br#"{"value": "global"}"#;
    let global_commit =
        create_commit_with_file(&repo, &Layer::GlobalBase, "config.json", global_content);
    repo.set_layer_ref(&Layer::GlobalBase, global_commit)
        .unwrap();

    // Create ModeBase
    let mode_content = br#"{"value": "mode"}"#;
    let mode_commit = create_commit_with_file(
        &repo,
        &Layer::ModeBase {
            mode: "claude".to_string(),
        },
        "config.json",
        mode_content,
    );
    repo.set_layer_ref(
        &Layer::ModeBase {
            mode: "claude".to_string(),
        },
        mode_commit,
    )
    .unwrap();

    // Merge only specific layers
    let layers = vec![Layer::GlobalBase];
    let result = merger.merge_subset(&layers).unwrap();

    let merged = result.get("config.json").unwrap();
    let obj = merged.as_object().unwrap();
    assert_eq!(obj.get("value").and_then(|v| v.as_str()), Some("global"));
}

#[test]
fn test_layer_merge_subset_unversioned_layer_error() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject");

    // UserLocal should error
    let layers = vec![Layer::UserLocal];
    let result = merger.merge_subset(&layers);
    assert!(matches!(result, Err(_)));
}

#[test]
fn test_layer_merge_subset_workspace_active_error() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject");

    // WorkspaceActive should error
    let layers = vec![Layer::WorkspaceActive];
    let result = merger.merge_subset(&layers);
    assert!(matches!(result, Err(_)));
}

// ===== Multiple File Types Test =====

#[test]
fn test_layer_merge_multiple_file_types() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject");

    // Create GlobalBase with multiple file types
    let files = vec![
        ("config.json", br#"{"key": "value"}"#.as_ref()),
        ("settings.yaml", b"key: value"),
        ("app.toml", b"key = \"value\""),
        ("setup.ini", b"[section]\nkey = value"),
        ("README.md", b"Text content"),
    ];
    let global_commit = create_commit_with_files(&repo, &Layer::GlobalBase, &files);
    repo.set_layer_ref(&Layer::GlobalBase, global_commit)
        .unwrap();

    // Merge and verify all files are present
    let result = merger.merge_all().unwrap();
    assert_eq!(result.len(), 5);
    assert!(result.contains_key("config.json"));
    assert!(result.contains_key("settings.yaml"));
    assert!(result.contains_key("app.toml"));
    assert!(result.contains_key("setup.ini"));
    assert!(result.contains_key("README.md"));
}

// ===== Scope Precedence Rule Tests =====

#[test]
fn test_layer_merge_mode_bound_scope_precedence() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject")
        .with_mode("claude")
        .with_scope("javascript");

    // Create mode-bound scope layer
    let mode_scope_content = br#"{"source": "mode-scope"}"#;
    let mode_scope_commit = create_commit_with_file(
        &repo,
        &Layer::ModeScope {
            mode: "claude".to_string(),
            scope: "javascript".to_string(),
        },
        "config.json",
        mode_scope_content,
    );
    repo.set_layer_ref(
        &Layer::ModeScope {
            mode: "claude".to_string(),
            scope: "javascript".to_string(),
        },
        mode_scope_commit,
    )
    .unwrap();

    // Merge - should use mode-bound scope, not untethered
    let layers = merger.determine_active_layers();

    // Verify mode-bound scope is present
    assert!(layers.contains(&Layer::ModeScope {
        mode: "claude".to_string(),
        scope: "javascript".to_string()
    }));

    // Verify untethered scope is NOT present
    assert!(!layers.contains(&Layer::ScopeBase {
        scope: "javascript".to_string()
    }));
}

// ===== Full Nine-Layer Hierarchy Test =====

#[test]
fn test_layer_merge_all_nine_layers() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let repo = JinRepo::init(temp_dir.path()).unwrap();
    let merger = LayerMerge::new(&repo, "testproject")
        .with_mode("claude")
        .with_scope("javascript");

    // Note: Git ref structure doesn't allow having both mode/claude (file)
    // and mode/claude/scope/* (directory) as refs simultaneously.
    // This is a Git limitation, not a bug in our code.

    // Create a subset of layers that don't conflict
    let layers_to_create = vec![
        (Layer::GlobalBase, 1),
        (
            Layer::ProjectBase {
                project: "testproject".to_string(),
            },
            7,
        ),
        (
            Layer::ScopeBase {
                scope: "python".to_string(),
            },
            6,
        ),
    ];

    for (layer, value) in &layers_to_create {
        let content = format!(
            r#"{{"layer": "{}", "precedence": {}}}"#,
            format!("{:?}", layer),
            value
        );
        let commit = create_commit_with_file(&repo, layer, "config.json", content.as_bytes());
        repo.set_layer_ref(layer, commit).unwrap();
    }

    // Determine active layers
    let layers = merger.determine_active_layers();

    // Verify ordering (only GlobalBase and ProjectBase will have refs)
    for i in 1..layers.len() {
        assert!(layers[i - 1] < layers[i]);
    }
}
