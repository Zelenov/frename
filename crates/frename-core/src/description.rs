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
//! Until clipscribe has an API for an editor's edits (#666), the few format rules an edit needs
//! (sorting, parts covering the clip) are kept here.
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
/// one, in line with `after`, the markers it holds now. `before` is the markers it held before
/// the write: an item whose marker was among them and is gone now was deleted by the editor,
/// and a part whose marker spans other times now was moved by them. Items whose markers were
/// never in the video (made elsewhere) stay.
pub(crate) fn markers_saved(video: &Path, before: &[Marker], after: &[Marker]) {
    let before = spans(before);
    update(video, |file, names| {
        apply_markers(file, after, &before, names)
    });
}

/// Each editable marker's `(start_ms, duration_ms)` by its id. Of markers that share an id (a
/// copy made in Premiere), the one [`crate::markers::give_ids`] lets keep it counts: the first by
/// time. The copy's times are not a move of the original (#668).
fn spans(markers: &[Marker]) -> HashMap<String, (u64, u64)> {
    let mut sorted: Vec<&Marker> = markers.iter().filter(|m| m.is_editable()).collect();
    sorted.sort_by_key(|m| (m.start_ms, m.duration_ms));
    let mut spans = HashMap::new();
    for m in sorted {
        if let Some(id) = &m.id {
            spans
                .entry(id.clone())
                .or_insert((m.start_ms, m.duration_ms));
        }
    }
    spans
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
    // A length no video has (a hand-edited file): nothing to place times in.
    if !(file.duration_s.is_finite() && file.duration_s > 0.0) {
        log::warn!(
            "description: {} is left as it is: its duration_s is {}",
            path.display(),
            file.duration_s
        );
        return;
    }
    let names = speaker_names(&file, video);
    if !change(&mut file, &names) {
        return;
    }
    // As clipscribe's own save does: a rule the file breaks is in the log.
    for problem in file.problems() {
        log::warn!("description: {}: {problem}", path.display());
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
/// - an item whose marker differs from it (name, comment, colour, and for a moment its time)
///   takes the marker's; it is `by: ai` while the marker is the AI's ([`Marker::ai`]), else
///   `by: user`;
/// - a part the editor moved, resized or made a point of leaves the parts and is a moment of
///   theirs, and a part they deleted goes; either way its stretch goes to the AI part next to
///   it, so the parts still cover the clip (the format's rule);
/// - a marker the file has no item for is added as a moment;
/// - a moment whose id is in `before` but has no marker now goes.
///
/// `before` is the span each marker had in the video before this write: a part was moved when
/// its marker's span changed, not when it differs from the file (a part may hold a stretch its
/// marker does not show, and a track's frame rate rounds times).
///
/// Only markers frename can write (with a GUID and an id) count. Returns whether `file` changed.
fn apply_markers(
    file: &mut DescribeFile,
    markers: &[Marker],
    before: &HashMap<String, (u64, u64)>,
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
    let deleted = |id: &str| !by_id.contains_key(id) && before.contains_key(id);
    let moved_by_editor = |m: &Marker| {
        let id = m.id.as_deref().unwrap_or_default();
        !m.ai
            && before
                .get(id)
                .is_some_and(|span| *span != (m.start_ms, m.duration_ms))
    };
    let mut in_file: HashSet<String> = HashSet::new();
    let mut left_parts: Vec<Item> = Vec::new();
    if let Some(parts) = file.parts.take() {
        let mut kept = Vec::new();
        for mut part in parts {
            if deleted(&part.id) {
                continue;
            }
            in_file.insert(part.id.clone());
            let Some(marker) = by_id.get(part.id.as_str()) else {
                kept.push(part);
                continue;
            };
            if moved_by_editor(marker) {
                follow(&mut part, marker, &people, names, duration_s);
                left_parts.push(part);
            } else {
                // The AI's part keeps the time the file gives it: it may have taken over the
                // stretch of a part the editor moved, which its marker does not show.
                let (start_s, end_s) = (part.start_s, part.end_s);
                follow(&mut part, marker, &people, names, duration_s);
                (part.start_s, part.end_s) = (start_s, end_s);
                kept.push(part);
            }
        }
        sort_items(&mut kept);
        cover(&mut kept, duration_s);
        file.parts = Some(kept);
    }
    if let Some(moments) = &mut file.moments {
        moments.retain(|item| !deleted(&item.id));
        for item in moments.iter_mut() {
            in_file.insert(item.id.clone());
            if let Some(marker) = by_id.get(item.id.as_str()) {
                follow(item, marker, &people, names, duration_s);
            }
        }
    }
    let added: Vec<Item> = markers
        .iter()
        .filter(|m| m.is_editable())
        .filter(|m| m.id.as_deref().is_some_and(|id| !in_file.contains(id)))
        .filter_map(|m| new_item(m, duration_s))
        .collect();
    if !added.is_empty() || !left_parts.is_empty() {
        let moments = file.moments.get_or_insert_with(Vec::new);
        moments.extend(left_parts);
        moments.extend(added);
        sort_items(moments);
    }
    *file != original
}

/// Give every stretch of 0..`duration_s` that `parts` (sorted) leave uncovered to the AI part
/// that ends where it starts, else to the AI part that starts where it ends. A part of the
/// editor's is never moved: a gap between two of theirs stays (and is logged).
fn cover(parts: &mut [Item], duration_s: f64) {
    if parts.is_empty() {
        return;
    }
    let mut gaps = Vec::new();
    let mut at = 0.0;
    for (n, part) in parts.iter().enumerate() {
        if ms(part.start_s) > ms(at) {
            gaps.push((at, part.start_s, n.checked_sub(1), Some(n)));
        }
        at = f64::max(at, part.end_or_start());
    }
    if ms(at) < ms(duration_s) {
        gaps.push((at, duration_s, Some(parts.len() - 1), None));
    }
    for (start, end, before, after) in gaps {
        let ai = |n: &Option<usize>| n.filter(|n| parts[*n].by == By::Ai);
        if let Some(n) = ai(&before) {
            parts[n].end_s = Some(end);
        } else if let Some(n) = ai(&after) {
            parts[n].start_s = start;
        } else {
            log::warn!("description: no AI part to cover {start}–{end} s");
        }
    }
}

/// Whether `marker` puts `item` somewhere else: a start or an end more than a millisecond off
/// (times pass through the video's own frame rate, which may round them by one).
fn moved(item: &Item, marker: &Marker, duration_s: f64) -> bool {
    let (_, end_s) = times(marker, duration_s);
    ms(item.start_s).abs_diff(marker.start_ms) > 1
        || match (item.end_s, end_s) {
            (Some(a), Some(b)) => ms(a).abs_diff(ms(b)) > 1,
            (None, None) => false,
            _ => true,
        }
}

/// Set the In and Out of `file` to `segment` (none when neither is set). Returns whether it
/// changed.
fn apply_in_out(file: &mut DescribeFile, segment: Segment) -> bool {
    let d = file.duration_s;
    let at = |t: Option<f32>| t.map(|t| seconds(ms(f64::from(t))).clamp(0.0, d));
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
    if moved(item, marker, duration_s) {
        (item.start_s, item.end_s) = times(marker, duration_s);
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

/// Sorted by start, then end, as the format keeps them. (clipscribe's own sorting is not public
/// yet: #666.)
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

#[cfg(test)]
mod tests;
