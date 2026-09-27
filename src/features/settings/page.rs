//! The pages of the settings window (`docs/design/design-system.md` §14.2).

use crate::ui::icons::Icon;

/// A category of the settings window, in the order of its navigation list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Interface,
    Saving,
    Ai,
    Subtitles,
    Updates,
}

impl Page {
    pub const ALL: [Page; 5] = [
        Page::Interface,
        Page::Saving,
        Page::Ai,
        Page::Subtitles,
        Page::Updates,
    ];

    /// The page Settings opens on: the one asked for, else Updates when an update is ready (the
    /// dot on the settings button leads there); `None` keeps the page shown last.
    pub fn to_open(requested: Option<Page>, update_ready: bool) -> Option<Page> {
        requested.or(update_ready.then_some(Page::Updates))
    }

    /// The page's name on the command line (`--settings <name>`).
    pub fn name(self) -> &'static str {
        match self {
            Page::Interface => "interface",
            Page::Saving => "saving",
            Page::Ai => "ai",
            Page::Subtitles => "subtitles",
            Page::Updates => "updates",
        }
    }

    /// The page called `name` on the command line.
    pub fn from_name(name: &str) -> Option<Page> {
        Page::ALL.into_iter().find(|page| page.name() == name)
    }

    /// The page `steps` away in the list, wrapping around at either end.
    pub fn step(self, steps: isize) -> Page {
        let count = Page::ALL.len() as isize;
        let at = Page::ALL.iter().position(|p| *p == self).unwrap_or(0) as isize;
        Page::ALL[(at + steps).rem_euclid(count) as usize]
    }

    pub fn icon(self) -> Icon {
        match self {
            Page::Interface => Icon::Languages,
            Page::Saving => Icon::Folder,
            Page::Ai => Icon::Sparkles,
            Page::Subtitles => Icon::Captions,
            Page::Updates => Icon::RefreshCw,
        }
    }

    pub fn label(self) -> String {
        match self {
            Page::Interface => fl!("settings-page-interface"),
            Page::Saving => fl!("settings-page-saving"),
            Page::Ai => fl!("settings-page-ai"),
            Page::Subtitles => fl!("settings-subtitles"),
            Page::Updates => fl!("settings-updates"),
        }
    }

    /// The page's heading: its name in the list, or a longer one where the list needs it short.
    pub fn heading(self) -> String {
        match self {
            Page::Ai => fl!("settings-ai"),
            page => page.label(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_is_found_by_its_name() {
        for page in Page::ALL {
            assert_eq!(Page::from_name(page.name()), Some(page));
        }
        assert_eq!(Page::from_name("general"), None);
    }

    #[test]
    fn settings_open_on_the_page_asked_for_then_on_a_ready_update() {
        assert_eq!(Page::to_open(Some(Page::Ai), true), Some(Page::Ai));
        assert_eq!(Page::to_open(None, true), Some(Page::Updates));
        assert_eq!(Page::to_open(None, false), None, "the page shown last");
    }

    #[test]
    fn stepping_wraps_around_the_list() {
        assert_eq!(Page::Interface.step(1), Page::Saving);
        assert_eq!(Page::Updates.step(1), Page::Interface);
        assert_eq!(Page::Interface.step(-1), Page::Updates);
        assert_eq!(Page::Ai.step(-1), Page::Saving);
    }
}
