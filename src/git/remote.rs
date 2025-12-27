//! Remote operation utilities for Jin
//!
//! This module provides shared utilities for remote operations (fetch, pull, push)
//! including authentication callbacks, progress reporting, and option builders.

use crate::core::Result;
use git2::{Cred, FetchOptions, PushOptions, RemoteCallbacks};
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// Authentication attempt counter to prevent infinite loops
#[derive(Debug, Clone)]
pub struct AuthCounter {
    count: Arc<Mutex<u32>>,
}

impl Default for AuthCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthCounter {
    /// Create a new authentication counter
    pub fn new() -> Self {
        Self {
            count: Arc::new(Mutex::new(0)),
        }
    }

    /// Increment and check if max attempts exceeded
    pub fn increment_and_check(&self, max: u32) -> bool {
        let mut count = self.count.lock().unwrap();
        *count += 1;
        *count <= max
    }

    /// Get current count
    pub fn get(&self) -> u32 {
        *self.count.lock().unwrap()
    }
}

/// Setup authentication callbacks for remote operations
///
/// Tries multiple authentication methods in order:
/// 1. SSH key from SSH agent (most secure, no keys on disk)
/// 2. Default SSH keys (~/.ssh/id_rsa, ~/.ssh/id_ed25519)
/// 3. Fails after 3 attempts to prevent infinite loops
///
/// # Example
///
/// ```no_run
/// use jin::git::remote::setup_callbacks;
///
/// let mut callbacks = git2::RemoteCallbacks::new();
/// setup_callbacks(&mut callbacks);
/// ```
pub fn setup_callbacks(callbacks: &mut RemoteCallbacks) {
    let auth_counter = AuthCounter::new();

    callbacks.credentials(move |_url, username, _allowed| {
        if !auth_counter.increment_and_check(3) {
            return Err(git2::Error::from_str(
                "Authentication failed after 3 attempts",
            ));
        }

        let username = username.unwrap_or("git");

        // Try SSH agent first (most secure)
        match Cred::ssh_key_from_agent(username) {
            Ok(cred) => Ok(cred),
            Err(_) => {
                // SSH agent failed, try default keys
                // Try id_ed25519 first (modern), then id_rsa (legacy)
                if let Ok(home) = std::env::var("HOME") {
                    let ssh_dir = std::path::Path::new(&home).join(".ssh");

                    // Try ed25519 key
                    let ed25519_key = ssh_dir.join("id_ed25519");
                    if ed25519_key.exists() {
                        if let Ok(cred) = Cred::ssh_key(username, None, &ed25519_key, None) {
                            return Ok(cred);
                        }
                    }

                    // Try RSA key
                    let rsa_key = ssh_dir.join("id_rsa");
                    if rsa_key.exists() {
                        if let Ok(cred) = Cred::ssh_key(username, None, &rsa_key, None) {
                            return Ok(cred);
                        }
                    }
                }

                // All authentication methods failed
                Err(git2::Error::from_str(
                    "No valid SSH credentials found. Ensure SSH agent is running or SSH keys exist.",
                ))
            }
        }
    });
}

/// Setup transfer progress callback for fetch operations
///
/// Displays download progress in the format: "Received X/Y objects (Z%)"
/// with carriage return for line overwriting.
pub fn setup_transfer_progress(callbacks: &mut RemoteCallbacks) {
    callbacks.transfer_progress(|stats| {
        if stats.total_objects() > 0 {
            let percent = (stats.received_objects() * 100) / stats.total_objects();
            print!(
                "Received {}/{} objects ({}%)\r",
                stats.received_objects(),
                stats.total_objects(),
                percent
            );
            io::stdout().flush().unwrap();
        }
        true // Continue
    });
}

/// Setup sideband progress callback for remote messages
///
/// Displays messages from the remote server (e.g., "Compressing objects: 100%")
pub fn setup_sideband_progress(callbacks: &mut RemoteCallbacks) {
    callbacks.sideband_progress(|data| {
        print!("remote: {}", String::from_utf8_lossy(data));
        io::stdout().flush().unwrap();
        true
    });
}

/// Setup push update reference callback
///
/// Validates that each ref push was successful
pub fn setup_push_update_callback(callbacks: &mut RemoteCallbacks) {
    callbacks.push_update_reference(|refname, status| match status {
        Some(msg) => {
            eprintln!("Failed to push {}: {}", refname, msg);
            Err(git2::Error::from_str(msg))
        }
        None => {
            println!("  → {}", refname);
            Ok(())
        }
    });
}

/// Build FetchOptions with all standard callbacks
///
/// Configures authentication, transfer progress, and sideband progress.
///
/// # Example
///
/// ```no_run
/// use jin::git::remote::build_fetch_options;
///
/// let mut opts = build_fetch_options();
/// // Use with remote.fetch()
/// ```
pub fn build_fetch_options() -> Result<FetchOptions<'static>> {
    let mut callbacks = RemoteCallbacks::new();
    setup_callbacks(&mut callbacks);
    setup_transfer_progress(&mut callbacks);
    setup_sideband_progress(&mut callbacks);

    let mut opts = FetchOptions::new();
    opts.remote_callbacks(callbacks);

    Ok(opts)
}

/// Build PushOptions with all standard callbacks
///
/// Configures authentication and push validation.
///
/// # Example
///
/// ```no_run
/// use jin::git::remote::build_push_options;
///
/// let mut opts = build_push_options();
/// // Use with remote.push()
/// ```
pub fn build_push_options() -> Result<PushOptions<'static>> {
    let mut callbacks = RemoteCallbacks::new();
    setup_callbacks(&mut callbacks);
    setup_push_update_callback(&mut callbacks);

    let mut opts = PushOptions::new();
    opts.remote_callbacks(callbacks);

    Ok(opts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_counter() {
        let counter = AuthCounter::new();
        assert_eq!(counter.get(), 0);

        assert!(counter.increment_and_check(3));
        assert_eq!(counter.get(), 1);

        assert!(counter.increment_and_check(3));
        assert_eq!(counter.get(), 2);

        assert!(counter.increment_and_check(3));
        assert_eq!(counter.get(), 3);

        assert!(!counter.increment_and_check(3));
        assert_eq!(counter.get(), 4);
    }

    #[test]
    fn test_build_fetch_options() {
        let opts = build_fetch_options();
        assert!(opts.is_ok());
    }

    #[test]
    fn test_build_push_options() {
        let opts = build_push_options();
        assert!(opts.is_ok());
    }

    #[test]
    fn test_setup_callbacks() {
        let mut callbacks = RemoteCallbacks::new();
        setup_callbacks(&mut callbacks);
        // Just verify it doesn't panic - actual credential testing requires real SSH setup
    }
}
