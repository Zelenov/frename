//! Where playback stopped in each clip (#161): a clip opened again continues there.
//!
//! The position is personal view state written every few seconds, so it lives in the app
//! database ([`crate::AppDatabase`]), keyed by the clip's folder and file name, never in the
//! folder's `.frename` file or the video. Renames done in frename carry it along
//! ([`crate::Directory::rename_file`]); a rename done outside frename that only changed the tags
//! is followed the next time the folder is opened ([`tidy`]), the same way the folder's last
//! viewed file is found again.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use crate::FileSnapshot;

/// How far before the remembered moment a clip opens, so the editor sees what led up to it.
pub const RESUME_LEAD: Duration = Duration::from_secs(2);

/// A clip left earlier than this has nothing worth continuing: it opens at the start, and the
/// moment is not kept.
pub const WORTH_RESUMING: Duration = Duration::from_secs(5);

/// A clip left within this of its end was watched to the end: it opens at the start again.
pub const NEAR_THE_END: Duration = Duration::from_secs(5);

/// A clip left past this share of its length was watched to the end, too.
const WATCHED_SHARE: f64 = 0.95;

/// The most positions kept in all; the oldest go first.
pub const MOST_KEPT: usize = 5000;

/// Where a clip opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenAt {
    /// At the start, as a clip never seen.
    Start,
    /// At its in point: it was watched to the end before.
    InPoint(Duration),
    /// At `at`, a little before `stopped`, where playback stopped last time.
    Resume { at: Duration, stopped: Duration },
}

/// Where a clip of `duration` opens, given where playback `stopped` in it last time and its in
/// point. A duration of zero is unknown: then only the start of the clip is judged.
pub fn open_at(
    stopped: Option<Duration>,
    duration: Duration,
    in_point: Option<Duration>,
) -> OpenAt {
    let Some(stopped) = stopped.filter(|stopped| worth_remembering(*stopped)) else {
        return OpenAt::Start;
    };
    let watched = !duration.is_zero()
        && (stopped + NEAR_THE_END >= duration
            || stopped.as_secs_f64() >= duration.as_secs_f64() * WATCHED_SHARE);
    if watched {
        return in_point.map_or(OpenAt::Start, OpenAt::InPoint);
    }
    OpenAt::Resume {
        at: stopped.saturating_sub(RESUME_LEAD),
        stopped,
    }
}

/// Whether playback stopped at `position` is worth keeping (see [`WORTH_RESUMING`]).
pub fn worth_remembering(position: Duration) -> bool {
    position >= WORTH_RESUMING
}

/// What becomes of a position remembered for a file name once its folder is listed again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tidy {
    /// The file was renamed outside frename (its tags changed): the position follows it to this
    /// name.
    Follow(String),
    /// The file is gone: the position is dropped.
    Forget,
}

/// For positions remembered under `remembered` file names of a folder that now `listed` these
/// files: the ones that must follow a rename or be dropped. A name still listed (exactly, case
/// included) keeps its position. A gone name follows a rename done outside frename that only
/// changed the tags ([`FileSnapshot::untagged_key`]; with `ignore_case`, as on Windows, also one
/// that changed only letter case), but only when that is certain: exactly one listed file is
/// that clip, it has no position of its own, and no other gone name points at it. Otherwise the
/// position is dropped: a rename keeps the file's time, so a file that sat beside the gone one
/// all along cannot be told from a renamed one.
pub fn tidy(remembered: &[String], listed: &[String], ignore_case: bool) -> Vec<(String, Tidy)> {
    let listed_names: HashSet<&str> = listed.iter().map(String::as_str).collect();
    let has_position: HashSet<&str> = remembered
        .iter()
        .map(String::as_str)
        .filter(|name| listed_names.contains(name))
        .collect();
    // Each listed file parsed once: the clips by their key without tags.
    let mut clips: HashMap<(String, String), Vec<&String>> = HashMap::new();
    for name in listed {
        clips
            .entry(FileSnapshot::parse(name).untagged_key(ignore_case))
            .or_default()
            .push(name);
    }
    let gone: Vec<(&String, Option<&String>)> = remembered
        .iter()
        .filter(|name| !listed_names.contains(name.as_str()))
        .map(|name| {
            let key = FileSnapshot::parse(name).untagged_key(ignore_case);
            let renamed = match clips.get(&key).map(Vec::as_slice) {
                Some([only]) if !has_position.contains(only.as_str()) => Some(*only),
                _ => None,
            };
            (name, renamed)
        })
        .collect();
    let mut claims: HashMap<&String, usize> = HashMap::new();
    for renamed in gone.iter().filter_map(|(_, renamed)| *renamed) {
        *claims.entry(renamed).or_default() += 1;
    }
    gone.iter()
        .map(|(name, renamed)| {
            let change = match renamed {
                Some(renamed) if claims.get(renamed) == Some(&1) => {
                    Tidy::Follow((*renamed).clone())
                }
                _ => Tidy::Forget,
            };
            ((*name).clone(), change)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secs(secs: u64) -> Duration {
        Duration::from_secs(secs)
    }

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|name| name.to_string()).collect()
    }

    /// [`tidy`] where letter case matters.
    fn tidy_exact(remembered: &[String], listed: &[String]) -> Vec<(String, Tidy)> {
        tidy(remembered, listed, false)
    }

    #[test]
    fn a_rename_that_changed_only_letter_case_is_followed_where_case_does_not_matter() {
        let remembered = names(&["pick.MVI_0410.mp4"]);
        let listed = names(&["pick.mvi_0410.MP4"]);
        assert_eq!(
            tidy(&remembered, &listed, true),
            vec![(
                "pick.MVI_0410.mp4".to_string(),
                Tidy::Follow("pick.mvi_0410.MP4".to_string())
            )]
        );
        assert_eq!(
            tidy(&remembered, &listed, false),
            vec![("pick.MVI_0410.mp4".to_string(), Tidy::Forget)]
        );
    }

    #[test]
    fn a_clip_opens_a_little_before_where_it_stopped() {
        assert_eq!(
            open_at(Some(secs(40)), secs(120), None),
            OpenAt::Resume {
                at: secs(38),
                stopped: secs(40)
            }
        );
    }

    #[test]
    fn a_clip_never_left_or_left_near_its_start_opens_at_the_start() {
        assert_eq!(open_at(None, secs(120), None), OpenAt::Start);
        assert_eq!(open_at(Some(secs(4)), secs(120), None), OpenAt::Start);
        assert_eq!(
            open_at(Some(secs(5)), secs(120), None),
            OpenAt::Resume {
                at: secs(3),
                stopped: secs(5)
            }
        );
    }

    #[test]
    fn a_clip_watched_to_the_end_opens_at_the_start_or_its_in_point() {
        // Within the last seconds.
        assert_eq!(open_at(Some(secs(116)), secs(120), None), OpenAt::Start);
        // Past 95 % of a long clip, more than a few seconds before its end.
        assert_eq!(open_at(Some(secs(1000)), secs(1050), None), OpenAt::Start);
        assert_eq!(
            open_at(Some(secs(990)), secs(1050), None),
            OpenAt::Resume {
                at: secs(988),
                stopped: secs(990)
            }
        );
        // At the very end, where a clip played to its end stops.
        assert_eq!(
            open_at(Some(secs(120)), secs(120), Some(secs(12))),
            OpenAt::InPoint(secs(12))
        );
    }

    #[test]
    fn a_clip_of_unknown_length_continues_where_it_stopped() {
        assert_eq!(
            open_at(Some(secs(40)), Duration::ZERO, None),
            OpenAt::Resume {
                at: secs(38),
                stopped: secs(40)
            }
        );
    }

    #[test]
    fn a_position_near_the_start_is_not_worth_keeping() {
        assert!(!worth_remembering(Duration::ZERO));
        assert!(!worth_remembering(Duration::from_millis(4_999)));
        assert!(worth_remembering(secs(5)));
    }

    #[test]
    fn listed_names_keep_their_positions() {
        assert_eq!(
            tidy_exact(
                &names(&["a.mp4", "b.mp4"]),
                &names(&["a.mp4", "b.mp4", "c.mp4"])
            ),
            vec![]
        );
    }

    #[test]
    fn a_gone_file_is_forgotten() {
        assert_eq!(
            tidy_exact(&names(&["gone.mp4"]), &names(&["a.mp4"])),
            vec![("gone.mp4".to_string(), Tidy::Forget)]
        );
    }

    #[test]
    fn a_file_whose_tags_changed_outside_frename_takes_its_position_along() {
        assert_eq!(
            tidy_exact(
                &names(&["pick.MVI_0410.mp4"]),
                &names(&["skip.night.MVI_0410.MP4"])
            ),
            vec![(
                "pick.MVI_0410.mp4".to_string(),
                Tidy::Follow("skip.night.MVI_0410.MP4".to_string())
            )]
        );
    }

    #[test]
    fn a_rename_is_not_followed_across_extensions_or_onto_a_file_with_its_own_position() {
        assert_eq!(
            tidy_exact(&names(&["pick.clip.mkv"]), &names(&["review.clip.mp4"])),
            vec![("pick.clip.mkv".to_string(), Tidy::Forget)]
        );
        assert_eq!(
            tidy_exact(
                &names(&["pick.clip.mp4", "review.clip.mp4"]),
                &names(&["review.clip.mp4"])
            ),
            vec![("pick.clip.mp4".to_string(), Tidy::Forget)]
        );
        // Two gone names for one listed file: which one it was is not known.
        assert_eq!(
            tidy_exact(
                &names(&["a.clip.mp4", "b.clip.mp4"]),
                &names(&["c.clip.mp4"])
            ),
            vec![
                ("a.clip.mp4".to_string(), Tidy::Forget),
                ("b.clip.mp4".to_string(), Tidy::Forget)
            ]
        );
        // Two listed files that could be it: neither.
        assert_eq!(
            tidy_exact(
                &names(&["a.clip.mp4"]),
                &names(&["b.clip.mp4", "c.clip.MP4"])
            ),
            vec![("a.clip.mp4".to_string(), Tidy::Forget)]
        );
    }
}
