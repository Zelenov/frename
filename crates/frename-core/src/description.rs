//! The clip's description file, `clip.describe.json` next to the video (#388): what an AI run
//! found and what the editor set, in clipscribe's format (`docs/describe-format.md` of
//! clipscribe, read and written by `clipscribe::describe_file`).
//!
//! frename maps it onto what it already has:
//! - a part or a moment is a clip marker in the video: its `id` is the marker's short id
//!   ([`Marker::id`]), `by: ai` is [`Marker::ai`], `title` the marker's name, `description` its
//!   comment and `color` its colour (`green` for Premiere's default);
//! - `in_out` is the clip's In and Out points;
//! - `summary` and `usable` are the `AI:` and `Suggested In/Out:` lines of the comment, which
//!   the AI run writes there as before.
//!
//! An AI run writes the file (the app does, through `clipscribe::describe_file::save_run`) and
//! then its markers from the file's AI items ([`ai_markers`]). Every later write of the clip's
//! markers or In/Out brings an existing file in line ([`markers_saved`], [`in_out_saved`]): an
//! item the editor changed becomes `by: user`, one they added is added, one they deleted goes.
//! A clip without the file gets none from an edit: the file needs what only a run measures
//! (the picture's size and frame rate). A file this version cannot read is never written.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use clipscribe::describe_file::{
    describe_file_path, render_mentions, By, DescribeFile, Extra, InOut, Marker as Item, Person,
};

use crate::markers::{guid_for_id, Marker, MarkerColor, AI_MARKER_COLOR};
use crate::metadata::Segment;

/// Where the description file of `video` is: `clip.mp4` → `clip.describe.json`.
pub fn path(video: &Path) -> PathBuf {
    describe_file_path(video)
}

/// The description file of `video`. `None` when there is none, or when it cannot be read (a
/// newer version, not this format, a read error), which is logged.
pub fn read(video: &Path) -> Option<DescribeFile> {
    let path = path(video);
    match DescribeFile::read(&path) {
        Ok(Some(Ok(file))) => Some(file),
        Ok(None) => None,
        Ok(Some(Err(e))) => {
            log::warn!("description: {} is left as it is: {e}", path.display());
            None
        }
        Err(e) => {
            log::warn!("description: {} not read: {e}", path.display());
            None
        }
    }
}

/// The AI items of `file` (its parts and moments with `by: ai`) as the markers an AI run writes
/// into the video: the item's id and the GUID made from it, the title as the name and the
/// description as the comment, `@pN` shown as the person's name (the speaker's in the CAPT file
/// next to `video`, else the person's label), in [`AI_MARKER_COLOR`].
pub fn ai_markers(file: &DescribeFile, video: &Path) -> Vec<Marker> {
    let names = speaker_names(file, video);
    let people = people(file);
    items(file)
        .filter(|item| item.by == By::Ai)
        .map(|item| {
            let start_ms = ms(item.start_s);
            Marker {
                guid: Some(guid_for_id(&item.id)),
                id: Some(item.id.clone()),
                ai: true,
                start_ms,
                duration_ms: item.end_s.map_or(0, |end| ms(end).saturating_sub(start_ms)),
                name: shown(item.title.as_deref(), people, &names),
                comment: shown(item.description.as_deref(), people, &names),
                color: AI_MARKER_COLOR,
            }
        })
        .collect()
}

/// The clip's markers were just written into `video`: bring its description file, if it has
/// one, in line with `after`, the markers it holds now. `before` holds the ids its markers had
/// before the write: an item whose id was among them and is gone now was deleted by the editor.
/// Items whose markers were never in the video (made elsewhere) stay.
pub(crate) fn markers_saved(video: &Path, before: &HashSet<String>, after: &[Marker]) {
    update(video, |file, names| {
        apply_markers(file, after, before, names)
    });
}

/// The clip's In and Out points were just saved: bring its description file, if it has one,
/// in line with them.
pub(crate) fn in_out_saved(video: &Path, segment: Segment) {
    update(video, |file, _| apply_in_out(file, segment));
}

/// Read the file of `video`, `change` it, and write it back when that changed anything.
fn update(video: &Path, change: impl FnOnce(&mut DescribeFile, &HashMap<String, String>) -> bool) {
    let path = path(video);
    if !path.is_file() {
        return;
    }
    let Some(mut file) = read(video) else {
        return;
    };
    let names = speaker_names(&file, video);
    if !change(&mut file, &names) {
        return;
    }
    match file.write(&path) {
        Ok(()) => log::info!("description: {} brought in line", path.display()),
        Err(e) => log::warn!("description: {} not written: {e}", path.display()),
    }
}

/// The names of the CAPT file's speakers, read only when the file has people to name.
fn speaker_names(file: &DescribeFile, video: &Path) -> HashMap<String, String> {
    if people(file).is_empty() {
        return HashMap::new();
    }
    crate::subtitles::capt_speaker_names_by_id(video)
}

fn people(file: &DescribeFile) -> &[Person] {
    file.people.as_deref().unwrap_or_default()
}

/// Every part and moment of `file`.
fn items(file: &DescribeFile) -> impl Iterator<Item = &Item> {
    file.parts.iter().chain(&file.moments).flatten()
}

/// `text` as the editor reads it: `@pN` as a name (see [`render_mentions`]).
fn shown(text: Option<&str>, people: &[Person], names: &HashMap<String, String>) -> String {
    text.map(|t| render_mentions(t, people, |speaker| names.get(speaker).cloned()))
        .unwrap_or_default()
}

/// Bring the parts and moments of `file` in line with the clip's markers `markers`:
/// - an item whose marker differs from it (time, name, comment, colour) takes the marker's;
///   it is `by: ai` while the marker is the AI's ([`Marker::ai`]), else `by: user`;
/// - a part that became a point is a moment;
/// - a marker the file has no item for is added as a moment;
/// - an item whose id is in `before` but has no marker now goes.
///
/// Only markers frename can write (with a GUID and an id) count. Returns whether `file` changed.
fn apply_markers(
    file: &mut DescribeFile,
    markers: &[Marker],
    before: &HashSet<String>,
    names: &HashMap<String, String>,
) -> bool {
    let original = file.clone();
    let by_id: HashMap<&str, &Marker> = markers
        .iter()
        .filter(|m| m.is_editable())
        .filter_map(|m| Some((m.id.as_deref()?, m)))
        .collect();
    let people = people(file).to_vec();
    let duration_s = file.duration_s;
    let mut in_file: HashSet<String> = HashSet::new();
    for list in [&mut file.parts, &mut file.moments].into_iter().flatten() {
        list.retain(|item| by_id.contains_key(item.id.as_str()) || !before.contains(&item.id));
        for item in list.iter_mut() {
            in_file.insert(item.id.clone());
            if let Some(marker) = by_id.get(item.id.as_str()) {
                follow(item, marker, &people, names, duration_s);
            }
        }
    }
    // A part is a stretch: one the editor made a point of is a moment now.
    if let Some(parts) = &mut file.parts {
        let (points, stretches): (Vec<Item>, Vec<Item>) =
            parts.drain(..).partition(|p| p.end_s.is_none());
        *parts = stretches;
        if !points.is_empty() {
            file.moments.get_or_insert_with(Vec::new).extend(points);
        }
    }
    let mut added: Vec<Item> = markers
        .iter()
        .filter(|m| m.is_editable())
        .filter(|m| m.id.as_deref().is_some_and(|id| !in_file.contains(id)))
        .filter_map(|m| new_item(m, duration_s))
        .collect();
    if !added.is_empty() {
        file.moments.get_or_insert_with(Vec::new).append(&mut added);
    }
    for list in [&mut file.parts, &mut file.moments].into_iter().flatten() {
        sort_items(list);
    }
    *file != original
}

/// Set the In and Out of `file` to `segment` (none when neither is set). Returns whether it
/// changed.
fn apply_in_out(file: &mut DescribeFile, segment: Segment) -> bool {
    let d = file.duration_s;
    let at = |t: Option<f32>| t.map(|t| round_ms(f64::from(t)).clamp(0.0, d));
    let (start_s, end_s) = (at(segment.start), at(segment.end));
    let unchanged = match &file.in_out {
        Some(io) => same_time(io.start_s, start_s) && same_time(io.end_s, end_s),
        None => start_s.is_none() && end_s.is_none(),
    };
    if unchanged {
        return false;
    }
    file.in_out = (start_s.is_some() || end_s.is_some()).then(|| InOut {
        start_s,
        end_s: end_s.filter(|end| start_s.is_none_or(|start| *end > start)),
        extra: file
            .in_out
            .as_ref()
            .map(|io| io.extra.clone())
            .unwrap_or_default(),
    });
    true
}

/// Whether two optional times are the same to the millisecond.
fn same_time(a: Option<f64>, b: Option<f64>) -> bool {
    a.map(ms) == b.map(ms)
}

/// Make `item` say what `marker` says, leaving what already matches as it is (an AI text with
/// `@pN` stays as written while the marker shows it unchanged).
fn follow(
    item: &mut Item,
    marker: &Marker,
    people: &[Person],
    names: &HashMap<String, String>,
    duration_s: f64,
) {
    let (start_s, end_s) = times(marker, duration_s);
    // Times pass through the video's own frame rate, which may round them by a millisecond.
    let moved = ms(item.start_s).abs_diff(marker.start_ms) > 1
        || match (item.end_s, end_s) {
            (Some(a), Some(b)) => ms(a).abs_diff(ms(b)) > 1,
            (None, None) => false,
            _ => true,
        };
    if moved {
        item.start_s = start_s;
        item.end_s = end_s;
    }
    if shown(item.title.as_deref(), people, names) != marker.name {
        item.title = text(&marker.name);
    }
    if shown(item.description.as_deref(), people, names) != marker.comment {
        item.description = text(&marker.comment);
    }
    // The AI never sets a colour; the editor's is kept by name.
    item.color = (!marker.ai).then(|| color_name(marker.color));
    item.by = if marker.ai { By::Ai } else { By::User };
}

/// The item for a marker the file does not have yet, as a moment. `None` when it starts past
/// the end of the video.
fn new_item(marker: &Marker, duration_s: f64) -> Option<Item> {
    let (start_s, end_s) = times(marker, duration_s);
    if seconds(marker.start_ms) > duration_s {
        return None;
    }
    // An AI item needs a short title: a long name (an AI marker from before titles) is its
    // description, under a title made from it.
    let (title, description) = if marker.ai {
        let title = clipscribe::marker_title(&marker.name, &marker.comment);
        let description = if marker.comment.trim().is_empty() && marker.name.trim() != title {
            text(&marker.name)
        } else {
            text(&marker.comment)
        };
        (text(&title), description)
    } else {
        (text(&marker.name), text(&marker.comment))
    };
    Some(Item {
        id: marker.id.clone()?,
        by: if marker.ai { By::Ai } else { By::User },
        start_s,
        end_s,
        title,
        description,
        color: (!marker.ai).then(|| color_name(marker.color)),
        extra: Extra::new(),
    })
}

/// A marker's start and end in seconds of the video, inside it; no end for a point.
fn times(marker: &Marker, duration_s: f64) -> (f64, Option<f64>) {
    let start_s = seconds(marker.start_ms).min(duration_s);
    let end_s = (marker.duration_ms > 0)
        .then(|| seconds(marker.end_ms()).min(duration_s))
        .filter(|end| *end > start_s);
    (start_s, end_s)
}

/// A text field: absent when empty.
fn text(value: &str) -> Option<String> {
    (!value.trim().is_empty()).then(|| value.to_string())
}

/// The format's name for a marker colour: one of Premiere's nine in lower case, or
/// `#rrggbbaa` for a value frename has no name for.
fn color_name(color: MarkerColor) -> String {
    match color {
        // Stored as 0xAABBGGRR.
        MarkerColor::Other(value) => {
            let [r, g, b, a] = value.to_le_bytes();
            format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
        }
        named => named.name().to_ascii_lowercase(),
    }
}

/// Sorted by start, then end, as the format keeps them.
fn sort_items(items: &mut [Item]) {
    items.sort_by(|a, b| {
        a.start_s
            .total_cmp(&b.start_s)
            .then(a.end_or_start().total_cmp(&b.end_or_start()))
    });
}

fn seconds(ms: u64) -> f64 {
    ms as f64 / 1000.0
}

fn ms(seconds: f64) -> u64 {
    (seconds.max(0.0) * 1000.0).round() as u64
}

fn round_ms(seconds: f64) -> f64 {
    (seconds * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests;
