//! "Rotate videos": turns each checked MP4/MOV by changing its rotation flag (see
//! [`FileTagger::rotate_video`]); the picture is not re-encoded. Not undoable, like every batch
//! action: running the opposite turn brings the files back.

use std::path::Path;

use frename_core::FileTagger;
use iced::widget::{column, radio};
use iced::Element;

use super::super::{ItemResult, ItemStatus};

/// The log is always English, unlike the UI text `label()` returns.
const LOG_LABEL: &str = "Rotate videos";

pub fn label() -> String {
    fl!("batch-action-rotate")
}

/// What to do with each file's rotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Turn {
    Right,
    Left,
    Half,
    /// Back to 0° (a mirror, if any, stays).
    Upright,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetTurn(Turn),
}

#[derive(Debug, Clone)]
pub struct Options {
    turn: Turn,
}

impl Default for Options {
    fn default() -> Self {
        Self { turn: Turn::Right }
    }
}

impl Options {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SetTurn(turn) => self.turn = turn,
        }
    }

    pub fn operation(&self) -> super::Operation {
        super::Operation::Rotate(self.turn)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let choice = |label: String, turn: Turn| {
            radio(label, turn, Some(self.turn), Message::SetTurn).text_size(13)
        };
        let choices = column![
            choice(fl!("batch-action-rotate-right"), Turn::Right),
            choice(fl!("batch-action-rotate-left"), Turn::Left),
            choice(fl!("batch-action-rotate-half"), Turn::Half),
            choice(fl!("batch-action-rotate-reset"), Turn::Upright),
        ]
        .spacing(8);
        super::panel(label(), fl!("batch-action-rotate-hint"), choices.into())
    }
}

/// Turn the file at `path` as `turn` says. A file already upright is skipped by `Upright`.
pub fn run(turn: Turn, path: &Path) -> ItemResult {
    let quarter_turns = match turn {
        Turn::Right => Ok(1),
        Turn::Left => Ok(-1),
        Turn::Half => Ok(2),
        Turn::Upright => FileTagger::video_rotation(path).map(|r| r.turns_to_upright()),
    };
    match quarter_turns.and_then(|turns| {
        if turns == 0 {
            return Ok(false);
        }
        FileTagger::rotate_video(path, turns).map(|_| true)
    }) {
        Ok(true) => ItemResult::new(ItemStatus::Done, None),
        Ok(false) => ItemResult::new(ItemStatus::Skipped, None),
        // No rotation flag in this format, damaged, or the write failed (read-only or open in
        // Premiere): the file is as it was.
        Err(error) => {
            log::warn!("{LOG_LABEL}: {path:?} not turned: {error}");
            ItemResult::failed(error.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn copy_of_clip(test: &str) -> std::path::PathBuf {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("crates/frename-core/tests/fixtures/wide.mp4");
        let dir = std::env::temp_dir().join(format!(
            "frename-batch-rotate-{test}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let file = dir.join("clip.mp4");
        std::fs::copy(fixture, &file).expect("copy");
        file
    }

    fn degrees(path: &Path) -> u16 {
        FileTagger::video_rotation(path)
            .expect("rotation")
            .degrees()
    }

    #[test]
    fn each_turn_and_back_upright() {
        let file = copy_of_clip("turns");
        assert_eq!(run(Turn::Upright, &file).status, ItemStatus::Skipped);
        assert_eq!(run(Turn::Right, &file).status, ItemStatus::Done);
        assert_eq!(degrees(&file), 90);
        assert_eq!(run(Turn::Half, &file).status, ItemStatus::Done);
        assert_eq!(degrees(&file), 270);
        assert_eq!(run(Turn::Left, &file).status, ItemStatus::Done);
        assert_eq!(degrees(&file), 180);
        assert_eq!(run(Turn::Upright, &file).status, ItemStatus::Done);
        assert_eq!(degrees(&file), 0);
    }

    #[test]
    fn a_format_without_a_flag_fails_with_the_reason() {
        let file = copy_of_clip("format").with_file_name("clip.mkv");
        std::fs::write(&file, b"not a movie").expect("write");
        let result = run(Turn::Right, &file);
        assert_eq!(result.status, ItemStatus::Failed);
        assert_eq!(
            result.reason.as_deref(),
            Some("this format has no rotation flag")
        );
    }
}
