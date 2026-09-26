//! The Anthropic API key, kept in the operating system's credential store (Windows Credential
//! Manager, macOS Keychain, Secret Service on Linux). Never in the database, logs or files.

const SERVICE: &str = "frename";
const USER: &str = "anthropic-api-key";

/// Whether a key is saved, or that the store cannot be used on this system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Saved,
    Missing,
    /// No credential store works here (e.g. a Linux desktop without Secret Service).
    Unavailable,
}

/// Why the key could not be read or written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyError(pub String);

impl std::fmt::Display for KeyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The store's entry for the key. Tests use an entry of their own, so a developer's real key
/// is never touched.
fn entry(user: &str) -> Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new(SERVICE, user)
}

/// The saved key; `None` when there is none or the store cannot be used.
pub fn read_key() -> Option<String> {
    read_key_of(USER)
}

fn read_key_of(user: &str) -> Option<String> {
    entry(user)
        .and_then(|e| e.get_password())
        .ok()
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty())
}

/// Whether a key is saved. Blocking (may unlock a keyring): run it on a worker thread.
pub fn key_state() -> KeyState {
    key_state_of(USER)
}

fn key_state_of(user: &str) -> KeyState {
    match entry(user).and_then(|e| e.get_password()) {
        Ok(key) if !key.trim().is_empty() => KeyState::Saved,
        Ok(_) | Err(keyring::Error::NoEntry) => KeyState::Missing,
        Err(e) => {
            log::warn!("ai: credential store unavailable: {e}");
            KeyState::Unavailable
        }
    }
}

/// Save `key`, replacing any saved one.
pub fn save_key(key: &str) -> Result<(), KeyError> {
    save_key_of(USER, key)
}

fn save_key_of(user: &str, key: &str) -> Result<(), KeyError> {
    entry(user)
        .and_then(|e| e.set_password(key.trim()))
        .map_err(|e| KeyError(e.to_string()))
}

/// Remove the saved key; removing none is no error.
pub fn delete_key() -> Result<(), KeyError> {
    delete_key_of(USER)
}

fn delete_key_of(user: &str) -> Result<(), KeyError> {
    match entry(user).and_then(|e| e.delete_credential()) {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(KeyError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The key goes to the store and back under a test entry of its own; where no store works
    /// (CI's Linux runner has no Secret Service), it says so instead of pretending to save
    /// into a mock.
    #[test]
    fn the_key_round_trips_or_the_store_says_it_is_unavailable() {
        let user = format!("{USER}-test-{}", std::process::id());
        match key_state_of(&user) {
            KeyState::Unavailable => {
                #[cfg(windows)]
                panic!("Windows always has its Credential Manager");
                assert!(save_key_of(&user, "test-key").is_err() || read_key_of(&user).is_none());
            }
            state => {
                assert_eq!(state, KeyState::Missing);
                save_key_of(&user, " test-key ").expect("save");
                assert_eq!(read_key_of(&user).as_deref(), Some("test-key"));
                assert_eq!(key_state_of(&user), KeyState::Saved);
                delete_key_of(&user).expect("delete");
                assert_eq!(key_state_of(&user), KeyState::Missing);
                delete_key_of(&user).expect("deleting none is fine");
            }
        }
    }
}
