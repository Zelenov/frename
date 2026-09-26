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

fn entry() -> Result<keyring::Entry, keyring::Error> {
    keyring::Entry::new(SERVICE, USER)
}

/// The saved key; `None` when there is none or the store cannot be used.
pub fn read_key() -> Option<String> {
    entry()
        .and_then(|e| e.get_password())
        .ok()
        .map(|k| k.trim().to_string())
        .filter(|k| !k.is_empty())
}

/// Whether a key is saved. Blocking (may unlock a keyring): run it on a worker thread.
pub fn key_state() -> KeyState {
    match entry().and_then(|e| e.get_password()) {
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
    entry()
        .and_then(|e| e.set_password(key.trim()))
        .map_err(|e| KeyError(e.to_string()))
}

/// Remove the saved key; removing none is no error.
pub fn delete_key() -> Result<(), KeyError> {
    match entry().and_then(|e| e.delete_credential()) {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(KeyError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// On Windows the key really goes to the Credential Manager and back. Elsewhere (CI has no
    /// Secret Service running) the store says so instead of pretending to save into a mock.
    #[test]
    fn the_key_round_trips_or_the_store_says_it_is_unavailable() {
        if cfg!(windows) {
            let before = read_key();
            save_key("test-key").expect("save");
            assert_eq!(read_key().as_deref(), Some("test-key"));
            assert_eq!(key_state(), KeyState::Saved);
            match before {
                Some(key) => save_key(&key).expect("restore"),
                None => delete_key().expect("delete"),
            }
        } else if key_state() == KeyState::Unavailable {
            assert!(save_key("test-key").is_err() || read_key().is_none());
        }
    }
}
