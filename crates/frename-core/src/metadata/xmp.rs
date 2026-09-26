//! Comments and in/out points stored inside the media file as XMP.
//!
//! The comment goes into `dc:description`: Premiere Pro shows it in its Description column
//! and searches it. The other candidates lose to native QuickTime fields: Premiere fills
//! Log Note from `ilst ©cmt` and Title from `ilst ©nam` even when `xmpDM:logComment` and
//! `dc:title` are set, so a comment written there can be hidden by whatever the camera or
//! another app left in the file.
//!
//! The in/out segment goes next to it as a ranged clip marker on an `InOut` track in
//! `xmpDM:Tracks`. Premiere Pro keeps real In/Out points in the project only, but on import
//! it turns such a marker into a subclip next to the clip.

use std::collections::HashSet;
use std::path::Path;
use std::time::SystemTime;

use xmp_toolkit::{xmp_ns, OpenFileOptions, XmpError, XmpFile, XmpMeta, XmpValue};

use super::markers_xmp::{apply_markers, markers_of};
use crate::markers::Marker;

const DESCRIPTION: &str = "description";
const DEFAULT_LANGUAGE: &str = "x-default";

/// Why XMP could not be written into the file.
#[derive(Debug)]
pub(super) enum XmpWriteError {
    /// The format has no smart XMP handler (e.g. `.txt`, `.zip`): XMP cannot go in this file.
    Unsupported,
    Toolkit(XmpError),
    Io(std::io::Error),
}

impl std::fmt::Display for XmpWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported => write!(f, "the file format cannot hold XMP"),
            Self::Toolkit(e) => write!(f, "XMP toolkit error: {e}"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
        }
    }
}

impl From<XmpError> for XmpWriteError {
    fn from(e: XmpError) -> Self {
        Self::Toolkit(e)
    }
}

impl From<std::io::Error> for XmpWriteError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// What frename keeps in a file's XMP.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct XmpFields {
    /// Empty when there is no comment.
    pub comment: String,
    /// Both ends `None` when there is no in/out marker.
    pub segment: Segment,
}

/// Read the comment and in/out segment from the file's XMP. Empty when the file has none,
/// cannot hold XMP, or is a cloud placeholder whose content is not on disk.
pub(super) fn read(path: &Path) -> XmpFields {
    probe(path).unwrap_or_default()
}

/// Like [`read`], but `None` when the file cannot hold XMP or is a cloud placeholder,
/// so a caller can tell "no XMP here" from "XMP cannot go here".
pub(super) fn probe(path: &Path) -> Option<XmpFields> {
    if is_cloud_placeholder(path) {
        return None;
    }
    // MOV/MP4: read the packet straight from its box, which a folder scan needs to be fast.
    if let Some(packet) = super::bmff::find_xmp_packet(path) {
        let meta = packet.and_then(|packet| packet.parse::<XmpMeta>().ok());
        return Some(meta.map(|meta| fields_of(&meta)).unwrap_or_default());
    }
    probe_with_toolkit(path)
}

/// [`probe`] through the XMP Toolkit's file handlers, for every format they know.
fn probe_with_toolkit(path: &Path) -> Option<XmpFields> {
    let mut file = open(path, OpenFileOptions::default().for_read()).ok()?;
    let fields = file.xmp().map(|meta| fields_of(&meta)).unwrap_or_default();
    file.close();
    Some(fields)
}

fn fields_of(meta: &XmpMeta) -> XmpFields {
    XmpFields {
        comment: description(meta),
        segment: in_out_range(meta)
            .map(|range| range.to_segment(clip_duration_ms(meta)))
            .unwrap_or_default(),
    }
}

/// Write the given fields into the file's XMP; `None` leaves a field as it is. An empty
/// comment removes the description; a segment with no in and no out removes the in/out
/// marker. Does not touch the file when it already holds these values, and keeps the file's
/// modified and created times when it does write.
///
/// Returns whether the file now holds the segment. It does not when a marker cannot express
/// it: an in point at or past the clip end with no out, or one whose clip length is unknown.
pub(super) fn write(
    path: &Path,
    comment: Option<&str>,
    segment: Option<Segment>,
) -> Result<bool, XmpWriteError> {
    let mut file = open(path, OpenFileOptions::default().for_update())?;
    let mut meta = match file.xmp() {
        Some(meta) => meta,
        None => XmpMeta::new()?,
    };
    // Outer `None`: leave the marker alone; inner `None`: remove it.
    let target_range = segment
        .map(|s| (s, s.range_ms(clip_duration_ms(&meta))))
        .filter(|(s, range)| range.is_some() || s.is_empty())
        .map(|(_, range)| range);
    let segment_stored = segment.is_some() && target_range.is_some();
    let comment = comment.filter(|c| description(&meta) != *c);
    let range = target_range.filter(|r| in_out_range(&meta) != *r);
    if comment.is_none() && range.is_none() {
        file.close();
        return Ok(segment_stored);
    }
    if let Some(comment) = comment {
        // Replace the whole alt-text array so no stale per-language item survives next to x-default.
        meta.delete_property(xmp_ns::DC, DESCRIPTION)?;
        if !comment.is_empty() {
            meta.set_localized_text(xmp_ns::DC, DESCRIPTION, None, DEFAULT_LANGUAGE, comment)?;
        }
    }
    if let Some(range) = range {
        set_in_out_range(&mut meta, range)?;
    }
    if !file.can_put_xmp(&meta) {
        file.close();
        return Err(XmpWriteError::Unsupported);
    }
    let times = FileTimes::read(path)?;
    file.put_xmp(&meta)?;
    file.try_close()?;
    times.restore(path)?;
    Ok(segment_stored)
}

/// The clip markers in the file's XMP (see [`super::markers_xmp`]). `None` when the file cannot
/// hold XMP or is a cloud placeholder; empty when it has no markers.
pub(super) fn read_markers(path: &Path) -> Option<Vec<Marker>> {
    if is_cloud_placeholder(path) {
        return None;
    }
    if let Some(packet) = super::bmff::find_xmp_packet(path) {
        let meta = packet.and_then(|packet| packet.parse::<XmpMeta>().ok());
        return Some(meta.map(|meta| markers_of(&meta)).unwrap_or_default());
    }
    let mut file = open(path, OpenFileOptions::default().for_read()).ok()?;
    let markers = file.xmp().map(|meta| markers_of(&meta)).unwrap_or_default();
    file.close();
    Some(markers)
}

/// Length of the clip in milliseconds, as the toolkit reads it from the container's header;
/// `None` when it cannot tell.
pub(super) fn clip_length_ms(path: &Path) -> Option<u64> {
    if is_cloud_placeholder(path) {
        return None;
    }
    let mut file = open(path, OpenFileOptions::default().for_read()).ok()?;
    let length = file.xmp().and_then(|meta| clip_duration_ms(&meta));
    file.close();
    length
}

/// Bring the file's clip markers in line with `markers`; see [`super::markers_xmp::apply_markers`]
/// for what `known` means. Does not touch the file when nothing differs, and keeps its times.
pub(super) fn write_markers(
    path: &Path,
    markers: &[Marker],
    known: &HashSet<String>,
) -> Result<(), XmpWriteError> {
    let mut file = open(path, OpenFileOptions::default().for_update())?;
    let mut meta = match file.xmp() {
        Some(meta) => meta,
        None => XmpMeta::new()?,
    };
    if !apply_markers(&mut meta, markers, known)? {
        file.close();
        return Ok(());
    }
    if !file.can_put_xmp(&meta) {
        file.close();
        return Err(XmpWriteError::Unsupported);
    }
    let times = FileTimes::read(path)?;
    file.put_xmp(&meta)?;
    file.try_close()?;
    times.restore(path)?;
    Ok(())
}

/// Open with the format's smart handler only. Packet scanning would read a whole unknown
/// file looking for XMP, and could not safely write into it anyway.
fn open(path: &Path, options: OpenFileOptions) -> Result<XmpFile, XmpWriteError> {
    let mut file = XmpFile::new()?;
    file.open_file(path, options.use_smart_handler())
        .map_err(|_| XmpWriteError::Unsupported)?;
    Ok(file)
}

fn description(meta: &XmpMeta) -> String {
    meta.localized_text(xmp_ns::DC, DESCRIPTION, None, DEFAULT_LANGUAGE)
        .map(|(value, _)| value.value.trim().to_string())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// In/out range as an XMP clip marker
// ---------------------------------------------------------------------------

/// XMP Dynamic Media namespace (`xmpDM`), home of clip markers and duration.
pub(super) const XMP_DM: &str = "http://ns.adobe.com/xmp/1.0/DynamicMedia/";
pub(super) const TRACKS: &str = "Tracks";
/// Track type Premiere Pro turns into a subclip named `{file}.{marker name}` on import.
const IN_OUT_TRACK_TYPE: &str = "InOut";
const IN_OUT_MARKER_NAME: &str = "in-out";
/// Marker times are counted in milliseconds, independent of the clip's frame rate.
const MILLISECOND_RATE: &str = "f1000";

/// The segment's in and out points in seconds, as the file snapshot carries them.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Segment {
    pub start: Option<f32>,
    pub end: Option<f32>,
}

/// A marker range in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RangeMs {
    start: u64,
    duration: u64,
}

impl Segment {
    /// Whether neither in nor out is set.
    pub fn is_empty(self) -> bool {
        self.start.is_none() && self.end.is_none()
    }

    /// The marker range: a missing in starts at the beginning of the clip, a missing out ends
    /// at its end. `None` when neither is set, or when the range is empty or its end unknown.
    fn range_ms(self, clip_duration_ms: Option<u64>) -> Option<RangeMs> {
        if self.is_empty() {
            return None;
        }
        let to_ms = |seconds: f32| (f64::from(seconds.max(0.0)) * 1000.0).round() as u64;
        let start = self.start.map(to_ms).unwrap_or(0);
        let end = self.end.map(to_ms).or(clip_duration_ms)?;
        let duration = end.checked_sub(start).filter(|&d| d > 0)?;
        Some(RangeMs { start, duration })
    }
}

impl RangeMs {
    /// Inverse of [`Segment::range_ms`]: a range from the clip start has no in, a range to the
    /// clip end has no out. A range covering the whole clip keeps its in, so it still reads
    /// as a segment.
    fn to_segment(self, clip_duration_ms: Option<u64>) -> Segment {
        let end = self.start + self.duration;
        let end = (Some(end) != clip_duration_ms).then_some(end);
        let start = (self.start != 0 || end.is_none()).then_some(self.start);
        let seconds = |ms: u64| ms as f32 / 1000.0;
        Segment {
            start: start.map(seconds),
            end: end.map(seconds),
        }
    }
}

/// Clip length from `xmpDM:duration`, which the toolkit fills from the container's own header.
fn clip_duration_ms(meta: &XmpMeta) -> Option<u64> {
    let value: f64 = meta
        .struct_field(XMP_DM, "duration", XMP_DM, "value")?
        .value
        .parse()
        .ok()?;
    let scale = meta
        .struct_field(XMP_DM, "duration", XMP_DM, "scale")?
        .value;
    let scale = match scale.split_once('/') {
        Some((numerator, denominator)) => {
            numerator.trim().parse::<f64>().ok()? / denominator.trim().parse::<f64>().ok()?
        }
        None => scale.trim().parse::<f64>().ok()?,
    };
    let ms = value * scale * 1000.0;
    (ms.is_finite() && ms > 0.0).then(|| ms.round() as u64)
}

/// Path of the `index`-th (1-based) item of `xmpDM:Tracks`.
pub(super) fn track_path(index: usize) -> String {
    format!("{TRACKS}[{index}]")
}

/// Indexes of the tracks frename owns, i.e. every InOut track, in ascending order.
fn in_out_tracks(meta: &XmpMeta) -> Vec<usize> {
    (1..=meta.array_len(XMP_DM, TRACKS))
        .filter(|&i| {
            meta.struct_field(XMP_DM, &track_path(i), XMP_DM, "trackType")
                .is_some_and(|t| t.value == IN_OUT_TRACK_TYPE)
        })
        .collect()
}

/// The range of the first marker on the first InOut track, when it is one frename wrote.
fn in_out_range(meta: &XmpMeta) -> Option<RangeMs> {
    let track = track_path(*in_out_tracks(meta).first()?);
    let rate = meta.struct_field(XMP_DM, &track, XMP_DM, "frameRate")?;
    if rate.value != MILLISECOND_RATE {
        return None;
    }
    let marker = format!("{track}/xmpDM:markers[1]");
    let field = |name: &str| -> Option<u64> {
        meta.struct_field(XMP_DM, &marker, XMP_DM, name)?
            .value
            .parse()
            .ok()
    };
    Some(RangeMs {
        start: field("startTime")?,
        duration: field("duration")?,
    })
}

/// Replace every InOut track with one holding `range`, or with none when `range` is `None`.
/// Other tracks, such as Premiere's own comment markers, are left alone.
fn set_in_out_range(meta: &mut XmpMeta, range: Option<RangeMs>) -> Result<(), XmpError> {
    for index in in_out_tracks(meta).into_iter().rev() {
        meta.delete_array_item(XMP_DM, TRACKS, index as i32)?;
    }
    let Some(range) = range else {
        if meta.array_len(XMP_DM, TRACKS) == 0 {
            meta.delete_property(XMP_DM, TRACKS)?;
        }
        return Ok(());
    };
    let new_struct = || XmpValue::new(String::new()).set_is_struct(true);
    let text = |value: &str| XmpValue::new(value.to_string());

    meta.append_array_item(
        XMP_DM,
        &XmpValue::new(TRACKS.to_string()).set_is_array(true),
        &new_struct(),
    )?;
    let track = track_path(meta.array_len(XMP_DM, TRACKS));
    meta.set_struct_field(
        XMP_DM,
        &track,
        XMP_DM,
        "trackName",
        &text(IN_OUT_TRACK_TYPE),
    )?;
    meta.set_struct_field(
        XMP_DM,
        &track,
        XMP_DM,
        "trackType",
        &text(IN_OUT_TRACK_TYPE),
    )?;
    meta.set_struct_field(XMP_DM, &track, XMP_DM, "frameRate", &text(MILLISECOND_RATE))?;

    let markers = XmpValue::new(format!("{track}/xmpDM:markers"))
        .set_is_array(true)
        .set_is_ordered(true);
    meta.append_array_item(XMP_DM, &markers, &new_struct())?;
    let marker = format!("{track}/xmpDM:markers[1]");
    meta.set_struct_field(XMP_DM, &marker, XMP_DM, "name", &text(IN_OUT_MARKER_NAME))?;
    meta.set_struct_field(
        XMP_DM,
        &marker,
        XMP_DM,
        "startTime",
        &text(&range.start.to_string()),
    )?;
    meta.set_struct_field(
        XMP_DM,
        &marker,
        XMP_DM,
        "duration",
        &text(&range.duration.to_string()),
    )?;
    Ok(())
}

/// Whether the file is a cloud-sync placeholder (e.g. Dropbox or OneDrive "online only").
/// Opening one downloads the whole file, which a folder scan must not do for every video.
#[cfg(windows)]
fn is_cloud_placeholder(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_OFFLINE: u32 = 0x0000_1000;
    const FILE_ATTRIBUTE_RECALL_ON_OPEN: u32 = 0x0004_0000;
    const FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS: u32 = 0x0040_0000;
    const PLACEHOLDER: u32 = FILE_ATTRIBUTE_OFFLINE
        | FILE_ATTRIBUTE_RECALL_ON_OPEN
        | FILE_ATTRIBUTE_RECALL_ON_DATA_ACCESS;
    std::fs::metadata(path).is_ok_and(|m| m.file_attributes() & PLACEHOLDER != 0)
}

#[cfg(not(windows))]
fn is_cloud_placeholder(_path: &Path) -> bool {
    false
}

/// A file's modified and created times, saved before a metadata write and put back after it,
/// so the clip keeps its place in date-sorted lists and backups don't see it as new.
struct FileTimes {
    modified: SystemTime,
    #[cfg_attr(not(windows), allow(dead_code))]
    created: Option<SystemTime>,
}

impl FileTimes {
    fn read(path: &Path) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(path)?;
        Ok(Self {
            modified: metadata.modified()?,
            created: metadata.created().ok(),
        })
    }

    fn restore(&self, path: &Path) -> std::io::Result<()> {
        let times = std::fs::FileTimes::new().set_modified(self.modified);
        #[cfg(windows)]
        let times = match self.created {
            Some(created) => std::os::windows::fs::FileTimesExt::set_created(times, created),
            None => times,
        };
        std::fs::OpenOptions::new()
            .write(true)
            .open(path)?
            .set_times(times)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A fresh copy of a tiny 0.2 s QuickTime clip with no XMP, in its own temp folder.
    fn copy_of_clip(name: &str) -> PathBuf {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny.mov");
        let dir = std::env::temp_dir().join(format!("frename-xmp-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let file = dir.join("clip.mov");
        std::fs::copy(fixture, &file).expect("copy fixture");
        file
    }

    fn read_meta(path: &Path) -> XmpMeta {
        let mut file = open(path, OpenFileOptions::default().for_read()).expect("open");
        let meta = file.xmp().expect("xmp");
        file.close();
        meta
    }

    fn segment(start: Option<f32>, end: Option<f32>) -> Segment {
        Segment { start, end }
    }

    #[test]
    fn in_and_out_become_one_ranged_marker_on_an_in_out_track() {
        let file = copy_of_clip("range");
        write(&file, None, Some(segment(Some(0.05), Some(0.15)))).expect("write");
        let meta = read_meta(&file);
        assert_eq!(in_out_tracks(&meta), vec![1]);
        assert_eq!(
            in_out_range(&meta),
            Some(RangeMs {
                start: 50,
                duration: 100
            })
        );
    }

    #[test]
    fn missing_out_runs_to_the_end_of_the_clip_and_missing_in_starts_at_zero() {
        let file = copy_of_clip("open-ended");
        write(&file, None, Some(segment(Some(0.05), None))).expect("write");
        assert_eq!(
            in_out_range(&read_meta(&file)),
            Some(RangeMs {
                start: 50,
                duration: 150
            })
        );

        write(&file, None, Some(segment(None, Some(0.1)))).expect("write");
        assert_eq!(
            in_out_range(&read_meta(&file)),
            Some(RangeMs {
                start: 0,
                duration: 100
            })
        );
    }

    #[test]
    fn rewriting_replaces_the_marker_and_clearing_removes_it() {
        let file = copy_of_clip("replace");
        write(&file, Some("note"), Some(segment(Some(0.05), Some(0.15)))).expect("write");
        write(&file, Some("note"), Some(segment(Some(0.1), Some(0.15)))).expect("write");
        let meta = read_meta(&file);
        assert_eq!(in_out_tracks(&meta).len(), 1);
        assert_eq!(
            in_out_range(&meta),
            Some(RangeMs {
                start: 100,
                duration: 50
            })
        );

        write(&file, Some("note"), Some(segment(None, None))).expect("write");
        let meta = read_meta(&file);
        assert!(in_out_tracks(&meta).is_empty());
        assert_eq!(description(&meta), "note");
    }

    #[test]
    fn other_marker_tracks_are_kept() {
        let file = copy_of_clip("other-tracks");
        let mut meta = XmpMeta::new().expect("meta");
        let tracks = XmpValue::new(TRACKS.to_string()).set_is_array(true);
        meta.append_array_item(
            XMP_DM,
            &tracks,
            &XmpValue::new(String::new()).set_is_struct(true),
        )
        .expect("append");
        meta.set_struct_field(
            XMP_DM,
            &track_path(1),
            XMP_DM,
            "trackType",
            &XmpValue::new("Comment".to_string()),
        )
        .expect("field");
        let mut xmp_file = open(&file, OpenFileOptions::default().for_update()).expect("open");
        xmp_file.put_xmp(&meta).expect("put");
        xmp_file.try_close().expect("close");

        write(&file, None, Some(segment(Some(0.05), Some(0.15)))).expect("write");
        write(&file, None, Some(segment(None, None))).expect("write");
        let meta = read_meta(&file);
        assert_eq!(meta.array_len(XMP_DM, TRACKS), 1);
        assert_eq!(
            meta.struct_field(XMP_DM, &track_path(1), XMP_DM, "trackType")
                .map(|v| v.value),
            Some("Comment".to_string())
        );
    }

    #[test]
    fn range_needs_a_positive_length() {
        assert_eq!(segment(Some(2.0), Some(1.0)).range_ms(None), None);
        assert_eq!(segment(Some(1.0), None).range_ms(None), None);
        assert_eq!(segment(None, None).range_ms(Some(5000)), None);
    }

    #[test]
    fn the_fast_reader_sees_what_the_toolkit_wrote() {
        let file = copy_of_clip("fast-read");
        assert!(
            super::super::bmff::find_xmp_packet(&file)
                .expect("a MOV")
                .is_none(),
            "no XMP yet"
        );

        write(
            &file,
            Some("Козёл 🐐"),
            Some(segment(Some(0.05), Some(0.15))),
        )
        .expect("write");
        let fast = probe(&file).expect("fields");
        assert_eq!(fast, probe_with_toolkit(&file).expect("fields"));
        assert_eq!(fast.comment, "Козёл 🐐");
        assert_eq!(fast.segment, segment(Some(0.05), Some(0.15)));
    }

    #[test]
    fn files_that_are_not_mov_or_mp4_are_left_to_the_toolkit() {
        let file = copy_of_clip("not-bmff").with_file_name("clip.mp4");
        std::fs::write(&file, b"definitely not a movie").expect("write");
        assert!(super::super::bmff::find_xmp_packet(&file).is_none());
    }

    /// Compares the fast reader with the toolkit on every video of a real folder.
    /// Run with `FRENAME_XMP_DIR=<folder> cargo test -p frename-core -- --ignored fast_reader`.
    #[test]
    #[ignore]
    fn fast_reader_matches_the_toolkit_on_a_real_folder() {
        let Ok(dir) = std::env::var("FRENAME_XMP_DIR") else {
            return;
        };
        let mut compared = 0;
        for entry in std::fs::read_dir(dir).expect("folder").flatten() {
            let path = entry.path();
            if super::super::bmff::find_xmp_packet(&path).is_none() {
                continue;
            }
            assert_eq!(
                probe(&path),
                probe_with_toolkit(&path),
                "{}",
                path.display()
            );
            compared += 1;
        }
        println!("compared {compared} files");
    }
}
