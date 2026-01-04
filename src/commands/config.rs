//! Implementation of `jin config` subcommands

use crate::cli::ConfigAction;
use crate::core::config::{JinConfig, RemoteConfig, UserConfig};
use crate::core::{JinError, Result};

/// Execute a config subcommand
pub fn execute(action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::List => list(),
        ConfigAction::Get { key } => get(&key),
        ConfigAction::Set { key, value } => set(&key, &value),
    }
}

/// List all configuration values
fn list() -> Result<()> {
    let config = JinConfig::load()?;

    println!("Jin Configuration:");

    // JIN_DIR special handling
    let jin_dir_display = get_jin_dir_display()?;
    println!("  jin-dir: {}", jin_dir_display);

    // Remote configuration
    if let Some(ref remote) = config.remote {
        println!("  remote.url: {}", remote.url);
        println!("  remote.fetch-on-init: {}", remote.fetch_on_init);
    } else {
        println!("  remote.url: (not set)");
        println!("  remote.fetch-on-init: (not set)");
    }

    // User configuration
    if let Some(ref user) = config.user {
        println!(
            "  user.name: {}",
            user.name.as_deref().unwrap_or("(not set)")
        );
        println!(
            "  user.email: {}",
            user.email.as_deref().unwrap_or("(not set)")
        );
    } else {
        println!("  user.name: (not set)");
        println!("  user.email: (not set)");
    }

    Ok(())
}

/// Get a specific configuration value
fn get(key: &str) -> Result<()> {
    match key {
        "jin-dir" => {
            let display = get_jin_dir_display()?;
            println!("{}", display);
        }
        _ => {
            let config = JinConfig::load()?;
            let value = get_config_value(&config, key)?;
            println!("{}", value);
        }
    }
    Ok(())
}

/// Set a configuration value
fn set(key: &str, value: &str) -> Result<()> {
    let mut config = JinConfig::load()?;

    match key {
        "remote.url" => {
            config
                .remote
                .get_or_insert_with(|| RemoteConfig {
                    url: String::new(),
                    fetch_on_init: false,
                })
                .url = value.to_string();
        }
        "remote.fetch-on-init" => {
            let bool_val = value.parse::<bool>().map_err(|_| {
                JinError::Config(format!(
                    "Invalid boolean value: {}. Use 'true' or 'false'",
                    value
                ))
            })?;
            config
                .remote
                .get_or_insert_with(|| RemoteConfig {
                    url: String::new(),
                    fetch_on_init: false,
                })
                .fetch_on_init = bool_val;
        }
        "user.name" => {
            config
                .user
                .get_or_insert(UserConfig {
                    name: None,
                    email: None,
                })
                .name = Some(value.to_string());
        }
        "user.email" => {
            config
                .user
                .get_or_insert(UserConfig {
                    name: None,
                    email: None,
                })
                .email = Some(value.to_string());
        }
        _ => {
            return Err(JinError::NotFound(format!(
                "Unknown config key: '{}'. Valid keys are: jin-dir, remote.url, remote.fetch-on-init, user.name, user.email",
                key
            )));
        }
    }

    config.save()?;
    println!("Set {} = {}", key, value);
    Ok(())
}

/// Helper: Get config value by key
fn get_config_value(config: &JinConfig, key: &str) -> Result<String> {
    match key {
        "remote.url" => Ok(config
            .remote
            .as_ref()
            .map(|r| r.url.clone())
            .unwrap_or_else(|| "(not set)".to_string())),
        "remote.fetch-on-init" => Ok(config
            .remote
            .as_ref()
            .map(|r| r.fetch_on_init.to_string())
            .unwrap_or_else(|| "(not set)".to_string())),
        "user.name" => Ok(config
            .user
            .as_ref()
            .and_then(|u| u.name.as_ref())
            .cloned()
            .unwrap_or_else(|| "(not set)".to_string())),
        "user.email" => Ok(config
            .user
            .as_ref()
            .and_then(|u| u.email.as_ref())
            .cloned()
            .unwrap_or_else(|| "(not set)".to_string())),
        _ => Err(JinError::NotFound(format!(
            "Unknown config key: '{}'. Valid keys are: jin-dir, remote.url, remote.fetch-on-init, user.name, user.email",
            key
        ))),
    }
}

/// Helper: Get JIN_DIR display with guidance
fn get_jin_dir_display() -> Result<String> {
    if let Ok(jin_dir) = std::env::var("JIN_DIR") {
        Ok(format!(
            "{} (set via JIN_DIR environment variable)",
            jin_dir
        ))
    } else {
        Ok("~/.jin (default)".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_list_empty_config() {
        let _ctx = crate::test_utils::setup_unit_test();
        let result = list();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_list_with_values() {
        let _ctx = crate::test_utils::setup_unit_test();

        // Set some config values
        let mut config = JinConfig::load().unwrap();
        config.remote = Some(RemoteConfig {
            url: "https://github.com/test/jin-config".to_string(),
            fetch_on_init: true,
        });
        config.user = Some(UserConfig {
            name: Some("Test User".to_string()),
            email: Some("test@example.com".to_string()),
        });
        config.save().unwrap();

        let result = list();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_get_jin_dir() {
        let _ctx = crate::test_utils::setup_unit_test();
        let result = get("jin-dir");
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_get_remote_url() {
        let _ctx = crate::test_utils::setup_unit_test();

        // Set remote.url
        let mut config = JinConfig::load().unwrap();
        config.remote = Some(RemoteConfig {
            url: "https://github.com/test/jin-config".to_string(),
            fetch_on_init: false,
        });
        config.save().unwrap();

        let result = get("remote.url");
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_get_remote_url_not_set() {
        let _ctx = crate::test_utils::setup_unit_test();
        let result = get("remote.url");
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_get_unknown_key() {
        let _ctx = crate::test_utils::setup_unit_test();
        let result = get("unknown.key");
        assert!(matches!(result, Err(JinError::NotFound(_))));
    }

    #[test]
    #[serial]
    fn test_set_remote_url() {
        let _ctx = crate::test_utils::setup_unit_test();

        let result = set("remote.url", "https://github.com/test/jin-config");
        assert!(result.is_ok());

        // Verify the value was set
        let config = JinConfig::load().unwrap();
        assert!(config.remote.is_some());
        assert_eq!(
            config.remote.unwrap().url,
            "https://github.com/test/jin-config"
        );
    }

    #[test]
    #[serial]
    fn test_set_remote_fetch_on_init_true() {
        let _ctx = crate::test_utils::setup_unit_test();

        let result = set("remote.fetch-on-init", "true");
        assert!(result.is_ok());

        // Verify the value was set
        let config = JinConfig::load().unwrap();
        assert!(config.remote.is_some());
        assert_eq!(config.remote.unwrap().fetch_on_init, true);
    }

    #[test]
    #[serial]
    fn test_set_remote_fetch_on_init_false() {
        let _ctx = crate::test_utils::setup_unit_test();

        let result = set("remote.fetch-on-init", "false");
        assert!(result.is_ok());

        // Verify the value was set
        let config = JinConfig::load().unwrap();
        assert!(config.remote.is_some());
        assert_eq!(config.remote.unwrap().fetch_on_init, false);
    }

    #[test]
    #[serial]
    fn test_set_remote_fetch_on_init_invalid() {
        let _ctx = crate::test_utils::setup_unit_test();

        let result = set("remote.fetch-on-init", "not-a-boolean");
        assert!(matches!(result, Err(JinError::Config(_))));
    }

    #[test]
    #[serial]
    fn test_set_user_name() {
        let _ctx = crate::test_utils::setup_unit_test();

        let result = set("user.name", "Test User");
        assert!(result.is_ok());

        // Verify the value was set
        let config = JinConfig::load().unwrap();
        assert!(config.user.is_some());
        assert_eq!(config.user.unwrap().name, Some("Test User".to_string()));
    }

    #[test]
    #[serial]
    fn test_set_user_email() {
        let _ctx = crate::test_utils::setup_unit_test();

        let result = set("user.email", "test@example.com");
        assert!(result.is_ok());

        // Verify the value was set
        let config = JinConfig::load().unwrap();
        assert!(config.user.is_some());
        assert_eq!(
            config.user.unwrap().email,
            Some("test@example.com".to_string())
        );
    }

    #[test]
    #[serial]
    fn test_set_unknown_key() {
        let _ctx = crate::test_utils::setup_unit_test();

        let result = set("unknown.key", "value");
        assert!(matches!(result, Err(JinError::NotFound(_))));
    }

    #[test]
    #[serial]
    fn test_set_jin_dir_fails() {
        let _ctx = crate::test_utils::setup_unit_test();

        // jin-dir cannot be set via config command
        let result = set("jin-dir", "/custom/path");
        assert!(matches!(result, Err(JinError::NotFound(_))));
    }

    #[test]
    #[serial]
    fn test_set_overwrites_existing_value() {
        let _ctx = crate::test_utils::setup_unit_test();

        // Set initial value
        set("remote.url", "https://github.com/original/config").unwrap();

        // Overwrite with new value
        set("remote.url", "https://github.com/new/config").unwrap();

        // Verify the new value
        let config = JinConfig::load().unwrap();
        assert_eq!(config.remote.unwrap().url, "https://github.com/new/config");
    }

    #[test]
    #[serial]
    fn test_get_config_value_helper() {
        let _ctx = crate::test_utils::setup_unit_test();

        let mut config = JinConfig::load().unwrap();
        config.remote = Some(RemoteConfig {
            url: "https://example.com".to_string(),
            fetch_on_init: true,
        });
        config.user = Some(UserConfig {
            name: Some("Test".to_string()),
            email: Some("test@example.com".to_string()),
        });

        // Test all valid keys
        assert!(get_config_value(&config, "remote.url").is_ok());
        assert!(get_config_value(&config, "remote.fetch-on-init").is_ok());
        assert!(get_config_value(&config, "user.name").is_ok());
        assert!(get_config_value(&config, "user.email").is_ok());

        // Test unknown key
        assert!(get_config_value(&config, "unknown.key").is_err());
    }

    #[test]
    #[serial]
    fn test_get_jin_dir_display_with_env_var() {
        let _ctx = crate::test_utils::setup_unit_test();

        // JIN_DIR is set by setup_unit_test()
        let display = get_jin_dir_display().unwrap();
        assert!(display.contains("JIN_DIR environment variable"));
    }

    #[test]
    #[serial]
    fn test_execute_list() {
        let _ctx = crate::test_utils::setup_unit_test();

        let action = ConfigAction::List;
        let result = execute(action);
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_execute_get() {
        let _ctx = crate::test_utils::setup_unit_test();

        let action = ConfigAction::Get {
            key: "jin-dir".to_string(),
        };
        let result = execute(action);
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_execute_set() {
        let _ctx = crate::test_utils::setup_unit_test();

        let action = ConfigAction::Set {
            key: "user.name".to_string(),
            value: "Test User".to_string(),
        };
        let result = execute(action);
        assert!(result.is_ok());

        // Verify value was set
        let config = JinConfig::load().unwrap();
        assert_eq!(config.user.unwrap().name, Some("Test User".to_string()));
    }
}
