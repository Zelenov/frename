//! The Soniox API key, kept in the operating system's credential store (Windows Credential
//! Manager, macOS Keychain, the Secret Service on Linux); never in `frename.db`, logs or the
//! repository. Where there is no store, the `SONIOX_API_KEY` environment variable is used.
//!
//! Every function here blocks (the store may be a D-Bus round trip): call them from a
//! blocking thread.

use std::sync::Arc;

const SERVICE: &str = "frename";
const USER: &str = "soniox-api-key";
/// Read when no key is stored: agent sessions, CI, a system without a credential store.
const ENV_VAR: &str = "SONIOX_API_KEY";

/// A Soniox API key. Printed as `***`: operations and messages carrying it are logged.
#[derive(Clone, PartialEq, Eq)]
pub struct SonioxKey(Arc<str>);

impl SonioxKey {
    /// `None` for an empty or blank key.
    pub fn new(key: &str) -> Option<Self> {
        let key = key.trim();
        (!key.is_empty()).then(|| Self(Arc::from(key)))
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SonioxKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SonioxKey(***)")
    }
}

/// Where the key in use comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    Stored,
    Environment,
}

/// What was found when the key was read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyInfo {
    pub key: Option<(SonioxKey, KeySource)>,
    /// Whether this system has a credential store to save a key in.
    pub store_available: bool,
}

impl KeyInfo {
    pub fn key(&self) -> Option<&SonioxKey> {
        self.key.as_ref().map(|(key, _)| key)
    }
}

/// The name of this system's credential store, for the settings window.
pub fn store_name() -> &'static str {
    if cfg!(windows) {
        "Windows Credential Manager"
    } else if cfg!(target_os = "macos") {
        "the macOS Keychain"
    } else {
        "the system keyring (Secret Service)"
    }
}

/// Read the stored key, falling back to the environment variable.
pub fn load() -> KeyInfo {
    let env = std::env::var(ENV_VAR).ok().and_then(|k| SonioxKey::new(&k));
    load_from(USER, env)
}

/// Save `key` in the credential store.
pub fn save(key: &SonioxKey) -> Result<(), String> {
    save_as(USER, key)
}

/// Delete the stored key. Nothing stored is not an error.
pub fn remove() -> Result<(), String> {
    remove_as(USER)
}

fn entry(user: &str) -> Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new(SERVICE, user)
}

fn load_from(user: &str, env: Option<SonioxKey>) -> KeyInfo {
    let stored = entry(user).and_then(|e| e.get_password());
    let (stored, store_available) = match stored {
        Ok(key) => (SonioxKey::new(&key), true),
        Err(keyring::Error::NoEntry) => (None, true),
        Err(e) => {
            log::warn!("soniox key: credential store not available: {e}");
            (None, false)
        }
    };
    let key = stored
        .map(|k| (k, KeySource::Stored))
        .or_else(|| env.map(|k| (k, KeySource::Environment)));
    KeyInfo {
        key,
        store_available,
    }
}

fn save_as(user: &str, key: &SonioxKey) -> Result<(), String> {
    entry(user)
        .and_then(|e| e.set_password(key.expose()))
        .map_err(|e| {
            log::warn!("soniox key: cannot save: {e}");
            store_error(&e)
        })
}

fn remove_as(user: &str) -> Result<(), String> {
    match entry(user).and_then(|e| e.delete_credential()) {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => {
            log::warn!("soniox key: cannot remove: {e}");
            Err(store_error(&e))
        }
    }
}

/// A short text for the settings window; the details are in the log.
fn store_error(e: &keyring::Error) -> String {
    match e {
        keyring::Error::NoStorageAccess(_) | keyring::Error::PlatformFailure(_) => {
            "Cannot store the key on this system.".to_string()
        }
        _ => format!("The key could not be saved: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_key_is_never_printed() {
        let key = SonioxKey::new(" secret-key ").expect("a key");
        assert_eq!(key.expose(), "secret-key");
        assert_eq!(format!("{key:?}"), "SonioxKey(***)");
        assert!(SonioxKey::new("  ").is_none());
    }

    /// A round trip where the system has a credential store (Windows CI); where it has none
    /// (headless Linux CI) the store must say so rather than keep the key in memory.
    #[test]
    fn a_saved_key_is_read_back_and_removed_or_the_store_is_reported_missing() {
        let user = format!("soniox-api-key-test-{}", std::process::id());
        let key = SonioxKey::new("test-key").expect("a key");
        match save_as(&user, &key) {
            Ok(()) => {
                let info = load_from(&user, None);
                assert!(info.store_available);
                assert_eq!(info.key, Some((key, KeySource::Stored)));
                remove_as(&user).expect("remove");
                assert_eq!(load_from(&user, None).key, None);
            }
            Err(_) => {
                let info = load_from(&user, None);
                assert!(!info.store_available, "saving failed but the store works");
                assert_eq!(info.key, None);
            }
        }
    }

    #[test]
    fn the_environment_variable_is_used_when_nothing_is_stored() {
        let user = format!("soniox-api-key-none-{}", std::process::id());
        let env = SonioxKey::new("from-env");
        let info = load_from(&user, env.clone());
        assert_eq!(
            info.key,
            env.map(|k| (k, KeySource::Environment)),
            "nothing stored under {user}"
        );
    }
}
