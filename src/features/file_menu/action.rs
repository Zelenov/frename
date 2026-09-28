//! What the file menu can do, and the keys that do it without the menu.

use iced::keyboard::{self, key::Named};

/// An action on one file, from the file menu or its key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAction {
    /// Open the file's folder in Explorer (the file manager on Linux) with the file selected.
    ShowInFileManager,
    /// Put the file's full path on the clipboard.
    CopyPath,
    /// Put the file's name, with its extension, on the clipboard.
    CopyName,
}

impl FileAction {
    /// The menu's items, in order: the most used first.
    pub const ALL: [FileAction; 3] = [
        FileAction::ShowInFileManager,
        FileAction::CopyPath,
        FileAction::CopyName,
    ];

    /// The keys shown next to the item, as printed on the keyboard.
    pub fn keys(self) -> &'static str {
        match self {
            FileAction::ShowInFileManager => "F11",
            FileAction::CopyPath => "Shift+F11",
            FileAction::CopyName => "Ctrl+F11",
        }
    }

    /// The action a key press asks for, if any: `F11` shows the file, `Shift+F11` copies its
    /// path, `Ctrl+F11` its name. Other modifiers (Alt, both Shift and Ctrl) do nothing.
    pub fn from_key(key: &keyboard::Key, modifiers: keyboard::Modifiers) -> Option<FileAction> {
        if *key != keyboard::Key::Named(Named::F11) || modifiers.alt() || modifiers.logo() {
            return None;
        }
        match (modifiers.shift(), modifiers.command()) {
            (false, false) => Some(FileAction::ShowInFileManager),
            (true, false) => Some(FileAction::CopyPath),
            (false, true) => Some(FileAction::CopyName),
            (true, true) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyboard::Modifiers;

    fn f11(modifiers: Modifiers) -> Option<FileAction> {
        FileAction::from_key(&keyboard::Key::Named(Named::F11), modifiers)
    }

    #[test]
    fn f11_shows_shift_copies_the_path_ctrl_copies_the_name() {
        assert_eq!(f11(Modifiers::empty()), Some(FileAction::ShowInFileManager));
        assert_eq!(f11(Modifiers::SHIFT), Some(FileAction::CopyPath));
        assert_eq!(f11(Modifiers::COMMAND), Some(FileAction::CopyName));
    }

    #[test]
    fn other_combinations_and_keys_do_nothing() {
        assert_eq!(f11(Modifiers::SHIFT | Modifiers::COMMAND), None);
        assert_eq!(f11(Modifiers::ALT), None);
        assert_eq!(f11(Modifiers::SHIFT | Modifiers::ALT), None);
        let f12 = keyboard::Key::Named(Named::F12);
        assert_eq!(FileAction::from_key(&f12, Modifiers::empty()), None);
        let f1 = keyboard::Key::Named(Named::F1);
        assert_eq!(FileAction::from_key(&f1, Modifiers::SHIFT), None);
    }

    #[test]
    fn every_item_shows_the_key_that_does_it() {
        for action in FileAction::ALL {
            let (named, modifiers) = match action.keys() {
                "F11" => (Named::F11, Modifiers::empty()),
                "Shift+F11" => (Named::F11, Modifiers::SHIFT),
                "Ctrl+F11" => (Named::F11, Modifiers::COMMAND),
                other => panic!("unexpected keys {other}"),
            };
            let key = keyboard::Key::Named(named);
            assert_eq!(FileAction::from_key(&key, modifiers), Some(action));
        }
    }
}
