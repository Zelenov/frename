//! Clip markers in XMP, as Premiere Pro reads and writes them: markers on the `xmpDM:Tracks`
//! whose `trackType` is `Comment`, with times counted in the track's `frameRate`, a GUID, and the
//! color in a `keywordExtDVAv1_<uuid>` cue point parameter. See
//! `docs/research/premiere-xmp-markers.md`.
//!
//! Writing edits the markers in place and never rebuilds a track: fields frename does not know
//! (`type`, `speaker`, other cue point parameters, …) and markers it did not read are kept.
//! Markers are matched by GUID against what is in the file at the time of the write.

use std::collections::HashSet;

use xmp_toolkit::{XmpError, XmpMeta, XmpValue};

use crate::markers::{Marker, MarkerColor};

use super::xmp::{track_path, TRACKS, XMP_DM};

const COMMENT_TRACK_TYPE: &str = "Comment";
/// Rate of a track frename creates: milliseconds, like the in/out marker.
const NEW_TRACK_RATE: &str = "f1000";
const MARKER_GUID_KEY: &str = "marker_guid";
const COLOR_KEY_PREFIX: &str = "keywordExtDVAv1_";

/// A track's `frameRate`: `f<num>` or `f<num>s<den>` ticks per second.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rate {
    num: u128,
    den: u128,
}

impl Rate {
    fn parse(text: &str) -> Option<Self> {
        let rest = text.trim().strip_prefix('f')?;
        let (num, den) = rest.split_once('s').unwrap_or((rest, "1"));
        let rate = Self {
            num: num.parse().ok()?,
            den: den.parse().ok()?,
        };
        (rate.num > 0 && rate.den > 0).then_some(rate)
    }

    /// Ticks to milliseconds, rounded; saturates instead of overflowing.
    fn to_ms(self, ticks: u64) -> u64 {
        let ms = (u128::from(ticks) * 1000 * self.den + self.num / 2) / self.num;
        u64::try_from(ms).unwrap_or(u64::MAX)
    }

    /// Milliseconds to ticks, rounded: `round(ms × rate / 1000)`.
    fn to_ticks(self, ms: u64) -> u64 {
        let divisor = 1000 * self.den;
        let ticks = (u128::from(ms) * self.num + divisor / 2) / divisor;
        u64::try_from(ticks).unwrap_or(u64::MAX)
    }
}

/// A GUID as compared: without the `xmp:id:` prefix some writers add, in lower case.
fn normalize_guid(guid: &str) -> String {
    let guid = guid.trim();
    guid.strip_prefix("xmp:id:")
        .unwrap_or(guid)
        .to_ascii_lowercase()
}

fn field(meta: &XmpMeta, path: &str, name: &str) -> Option<String> {
    meta.struct_field(XMP_DM, path, XMP_DM, name)
        .map(|value| value.value)
}

fn set_field(meta: &mut XmpMeta, path: &str, name: &str, value: &str) -> Result<(), XmpError> {
    meta.set_struct_field(
        XMP_DM,
        path,
        XMP_DM,
        name,
        &XmpValue::new(value.to_string()),
    )
}

/// Set a text field, or remove it when `value` is empty.
fn set_or_remove(meta: &mut XmpMeta, path: &str, name: &str, value: &str) -> Result<(), XmpError> {
    if value.is_empty() {
        meta.delete_struct_field(XMP_DM, path, XMP_DM, name)
    } else {
        set_field(meta, path, name, value)
    }
}

/// Indexes (1-based) of the Comment tracks with a rate frename understands.
fn comment_tracks(meta: &XmpMeta) -> Vec<(usize, Rate)> {
    (1..=meta.array_len(XMP_DM, TRACKS))
        .filter(|&i| {
            field(meta, &track_path(i), "trackType").is_some_and(|t| t == COMMENT_TRACK_TYPE)
        })
        .filter_map(|i| {
            let rate = field(meta, &track_path(i), "frameRate").and_then(|r| Rate::parse(&r))?;
            Some((i, rate))
        })
        .collect()
}

fn markers_path(track: usize) -> String {
    format!("{}/xmpDM:markers", track_path(track))
}

fn marker_path(track: usize, index: usize) -> String {
    format!("{}[{index}]", markers_path(track))
}

fn params_path(marker: &str) -> String {
    format!("{marker}/xmpDM:cuePointParams")
}

/// `(index, key, value)` of each cue point parameter of the marker.
fn cue_params(meta: &XmpMeta, marker: &str) -> Vec<(usize, String, String)> {
    let params = params_path(marker);
    (1..=meta.array_len(XMP_DM, &params))
        .filter_map(|i| {
            let item = format!("{params}[{i}]");
            let key = field(meta, &item, "key")?;
            Some((i, key, field(meta, &item, "value").unwrap_or_default()))
        })
        .collect()
}

fn color_param(meta: &XmpMeta, marker: &str) -> Option<(usize, String)> {
    cue_params(meta, marker)
        .into_iter()
        .find(|(_, key, _)| key.starts_with(COLOR_KEY_PREFIX))
        .map(|(i, _, value)| (i, value))
}

fn color_of(meta: &XmpMeta, marker: &str) -> MarkerColor {
    let value = color_param(meta, marker).and_then(|(_, json)| {
        let json: serde_json::Value = serde_json::from_str(&json).ok()?;
        u32::try_from(json.get("color")?.as_u64()?).ok()
    });
    MarkerColor::from_value(value)
}

/// The marker's GUID: `xmpDM:guid`, else its `marker_guid` cue point parameter.
fn guid_of(meta: &XmpMeta, marker: &str) -> Option<String> {
    field(meta, marker, "guid")
        .filter(|g| !g.trim().is_empty())
        .or_else(|| {
            cue_params(meta, marker)
                .into_iter()
                .find(|(_, key, _)| key == MARKER_GUID_KEY)
                .map(|(_, _, value)| value)
                .filter(|g| !g.trim().is_empty())
        })
}

/// Whether frename shows the marker: plain comment markers only, not chapters, web links or
/// cue points, which stay in the file untouched.
fn is_comment_marker(meta: &XmpMeta, marker: &str) -> bool {
    field(meta, marker, "type").is_none_or(|t| t.is_empty() || t == COMMENT_TRACK_TYPE)
}

fn read_marker(meta: &XmpMeta, marker: &str, rate: Rate) -> Marker {
    let ticks = |name: &str| {
        field(meta, marker, name)
            .and_then(|v| v.trim().parse::<i64>().ok())
            .map_or(0, |v| u64::try_from(v).unwrap_or(0))
    };
    Marker {
        guid: guid_of(meta, marker).map(|g| g.trim().to_string()),
        start_ms: rate.to_ms(ticks("startTime")),
        duration_ms: rate.to_ms(ticks("duration")),
        name: field(meta, marker, "name").unwrap_or_default(),
        comment: field(meta, marker, "comment")
            .unwrap_or_default()
            .replace("\r\n", "\n")
            .replace('\r', "\n"),
        color: color_of(meta, marker),
    }
}

/// Every comment marker of every Comment track, in file order.
pub(super) fn markers_of(meta: &XmpMeta) -> Vec<Marker> {
    let mut markers = Vec::new();
    for (track, rate) in comment_tracks(meta) {
        for index in 1..=meta.array_len(XMP_DM, &markers_path(track)) {
            let path = marker_path(track, index);
            if is_comment_marker(meta, &path) {
                markers.push(read_marker(meta, &path, rate));
            }
        }
    }
    markers
}

/// Bring the file's markers in line with `markers`, editing in place:
/// - a marker whose GUID is in the file gets the fields that differ rewritten;
/// - one whose GUID is not is appended to the first Comment track (made when there is none);
/// - a file marker whose GUID is in `known` but not in `markers` was deleted and is removed,
///   along with a track, and the track list, left empty by that;
/// - every other file marker (added by another app meanwhile, or without a GUID) is kept.
///
/// Markers without a GUID in `markers` are read-only and ignored. Returns whether anything
/// changed.
pub(super) fn apply_markers(
    meta: &mut XmpMeta,
    markers: &[Marker],
    known: &HashSet<String>,
) -> Result<bool, XmpError> {
    let wanted: Vec<(String, &Marker)> = markers
        .iter()
        .filter_map(|m| Some((normalize_guid(m.guid.as_deref()?), m)))
        .collect();
    let known: HashSet<String> = known.iter().map(|g| normalize_guid(g)).collect();
    let mut changed = false;
    let mut in_file: HashSet<String> = HashSet::new();

    // Update and delete, walking backwards so deletions keep the earlier indexes valid.
    for (track, rate) in comment_tracks(meta).into_iter().rev() {
        let mut deleted_here = false;
        for index in (1..=meta.array_len(XMP_DM, &markers_path(track))).rev() {
            let path = marker_path(track, index);
            let Some(guid) = guid_of(meta, &path).map(|g| normalize_guid(&g)) else {
                continue;
            };
            in_file.insert(guid.clone());
            match wanted.iter().find(|(g, _)| *g == guid) {
                Some((_, marker)) => {
                    if is_comment_marker(meta, &path) {
                        changed |= update_marker(meta, &path, rate, marker)?;
                    }
                }
                None if known.contains(&guid) => {
                    meta.delete_array_item(XMP_DM, &markers_path(track), index as i32)?;
                    deleted_here = true;
                }
                None => {}
            }
        }
        if deleted_here && meta.array_len(XMP_DM, &markers_path(track)) == 0 {
            meta.delete_array_item(XMP_DM, TRACKS, track as i32)?;
            if meta.array_len(XMP_DM, TRACKS) == 0 {
                meta.delete_property(XMP_DM, TRACKS)?;
            }
        }
        changed |= deleted_here;
    }

    // Append the new ones.
    for (guid, marker) in &wanted {
        if in_file.contains(guid) {
            continue;
        }
        let (track, rate) = match comment_tracks(meta).first() {
            Some(&found) => found,
            None => (append_comment_track(meta)?, Rate { num: 1000, den: 1 }),
        };
        append_marker(meta, track, rate, marker)?;
        in_file.insert(guid.clone());
        changed = true;
    }
    Ok(changed)
}

/// Rewrite the fields of the marker at `path` that differ from `marker`. Returns whether any did.
fn update_marker(
    meta: &mut XmpMeta,
    path: &str,
    rate: Rate,
    marker: &Marker,
) -> Result<bool, XmpError> {
    let current = read_marker(meta, path, rate);
    let mut changed = false;
    if current.start_ms != marker.start_ms {
        set_field(
            meta,
            path,
            "startTime",
            &rate.to_ticks(marker.start_ms).to_string(),
        )?;
        changed = true;
    }
    if current.duration_ms != marker.duration_ms {
        if marker.duration_ms == 0 {
            meta.delete_struct_field(XMP_DM, path, XMP_DM, "duration")?;
        } else {
            set_field(
                meta,
                path,
                "duration",
                &rate.to_ticks(marker.duration_ms).to_string(),
            )?;
        }
        changed = true;
    }
    if current.name != marker.name {
        set_or_remove(meta, path, "name", &marker.name)?;
        changed = true;
    }
    if current.comment != marker.comment {
        set_or_remove(meta, path, "comment", &marker.comment.replace('\n', "\r"))?;
        changed = true;
    }
    if current.color != marker.color {
        set_color(meta, path, marker.color)?;
        changed = true;
    }
    Ok(changed)
}

fn new_struct() -> XmpValue<String> {
    XmpValue::new(String::new()).set_is_struct(true)
}

/// Append an empty Comment track in milliseconds; returns its index.
fn append_comment_track(meta: &mut XmpMeta) -> Result<usize, XmpError> {
    meta.append_array_item(
        XMP_DM,
        &XmpValue::new(TRACKS.to_string()).set_is_array(true),
        &new_struct(),
    )?;
    let track = meta.array_len(XMP_DM, TRACKS);
    let path = track_path(track);
    set_field(meta, &path, "trackName", COMMENT_TRACK_TYPE)?;
    set_field(meta, &path, "trackType", COMMENT_TRACK_TYPE)?;
    set_field(meta, &path, "frameRate", NEW_TRACK_RATE)?;
    Ok(track)
}

fn append_marker(
    meta: &mut XmpMeta,
    track: usize,
    rate: Rate,
    marker: &Marker,
) -> Result<(), XmpError> {
    let markers = XmpValue::new(markers_path(track))
        .set_is_array(true)
        .set_is_ordered(true);
    meta.append_array_item(XMP_DM, &markers, &new_struct())?;
    let path = marker_path(track, meta.array_len(XMP_DM, &markers_path(track)));
    set_field(
        meta,
        &path,
        "startTime",
        &rate.to_ticks(marker.start_ms).to_string(),
    )?;
    if marker.duration_ms > 0 {
        set_field(
            meta,
            &path,
            "duration",
            &rate.to_ticks(marker.duration_ms).to_string(),
        )?;
    }
    set_or_remove(meta, &path, "name", &marker.name)?;
    set_or_remove(meta, &path, "comment", &marker.comment.replace('\n', "\r"))?;
    let guid = marker.guid.clone().unwrap_or_default();
    set_field(meta, &path, "guid", &guid)?;
    append_param(meta, &path, MARKER_GUID_KEY, &guid)?;
    set_color(meta, &path, marker.color)
}

fn append_param(meta: &mut XmpMeta, marker: &str, key: &str, value: &str) -> Result<(), XmpError> {
    let params = params_path(marker);
    let array = XmpValue::new(params.clone())
        .set_is_array(true)
        .set_is_ordered(true);
    meta.append_array_item(XMP_DM, &array, &new_struct())?;
    let item = format!("{params}[{}]", meta.array_len(XMP_DM, &params));
    set_field(meta, &item, "key", key)?;
    set_field(meta, &item, "value", value)
}

/// Store `color` on the marker: green removes the color parameter, any other color sets the
/// `color` of an existing one (keeping its other JSON fields) or adds one.
fn set_color(meta: &mut XmpMeta, marker: &str, color: MarkerColor) -> Result<(), XmpError> {
    let existing = color_param(meta, marker);
    let Some(value) = color.value() else {
        if let Some((index, _)) = existing {
            let params = params_path(marker);
            meta.delete_array_item(XMP_DM, &params, index as i32)?;
            if meta.array_len(XMP_DM, &params) == 0 {
                meta.delete_struct_field(XMP_DM, marker, XMP_DM, "cuePointParams")?;
            }
        }
        return Ok(());
    };
    match existing {
        Some((index, json)) => {
            let mut object = serde_json::from_str::<serde_json::Value>(&json)
                .ok()
                .filter(serde_json::Value::is_object)
                .unwrap_or_else(|| serde_json::json!({}));
            object["color"] = serde_json::json!(value);
            let item = format!("{}[{index}]", params_path(marker));
            set_field(meta, &item, "value", &object.to_string())
        }
        None => {
            let key = format!("{COLOR_KEY_PREFIX}{}", uuid::Uuid::new_v4());
            let json = serde_json::json!({"color": value, "index": 0, "name": "", "payload": ""});
            append_param(meta, marker, &key, &json.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    /// The research sample: what Premiere writes, `parseType="Resource"` form.
    const PREMIERE: &str = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about="" xmlns:xmpDM="http://ns.adobe.com/xmp/1.0/DynamicMedia/">
   <xmpDM:Tracks><rdf:Bag>
    <rdf:li rdf:parseType="Resource">
     <xmpDM:trackName>Markers</xmpDM:trackName>
     <xmpDM:trackType>Comment</xmpDM:trackType>
     <xmpDM:frameRate>f254016000000</xmpDM:frameRate>
     <xmpDM:markers><rdf:Seq>
      <rdf:li rdf:parseType="Resource">
       <xmpDM:startTime>21210336000000</xmpDM:startTime>
       <xmpDM:duration>508032000000</xmpDM:duration>
       <xmpDM:name>Take 3</xmpDM:name>
       <xmpDM:comment>line one&#xD;line two</xmpDM:comment>
       <xmpDM:type>Comment</xmpDM:type>
       <xmpDM:guid>c19c0922-775d-465c-917b-6a7c3a5c0902</xmpDM:guid>
       <xmpDM:cuePointParams><rdf:Seq>
        <rdf:li rdf:parseType="Resource"><xmpDM:key>marker_guid</xmpDM:key>
          <xmpDM:value>c19c0922-775d-465c-917b-6a7c3a5c0902</xmpDM:value></rdf:li>
        <rdf:li rdf:parseType="Resource"><xmpDM:key>keywordExtDVAv1_e8dd1f3f-12f9-41f0-8672-e3296ce19cec</xmpDM:key>
          <xmpDM:value>{"color":4281740498,"index":0,"name":"","payload":""}</xmpDM:value></rdf:li>
        <rdf:li rdf:parseType="Resource"><xmpDM:key>vendor_thing</xmpDM:key>
          <xmpDM:value>keep me</xmpDM:value></rdf:li>
       </rdf:Seq></xmpDM:cuePointParams>
      </rdf:li>
      <rdf:li rdf:parseType="Resource">
       <xmpDM:startTime>254016000000</xmpDM:startTime>
       <xmpDM:name>Second</xmpDM:name>
       <xmpDM:speaker>Anna</xmpDM:speaker>
       <xmpDM:guid>xmp:id:AAAAAAAA-775d-465c-917b-6a7c3a5c0902</xmpDM:guid>
      </rdf:li>
      <rdf:li rdf:parseType="Resource">
       <xmpDM:startTime>0</xmpDM:startTime>
       <xmpDM:name>Chapter one</xmpDM:name>
       <xmpDM:type>Chapter</xmpDM:type>
       <xmpDM:guid>dddddddd-0000-0000-0000-000000000000</xmpDM:guid>
      </rdf:li>
     </rdf:Seq></xmpDM:markers>
    </rdf:li>
    <rdf:li rdf:parseType="Resource">
     <xmpDM:trackName>InOut</xmpDM:trackName>
     <xmpDM:trackType>InOut</xmpDM:trackType>
     <xmpDM:frameRate>f1000</xmpDM:frameRate>
     <xmpDM:markers><rdf:Seq><rdf:li><rdf:Description xmpDM:startTime="50" xmpDM:duration="100" xmpDM:name="in-out"/></rdf:li></rdf:Seq></xmpDM:markers>
    </rdf:li>
   </rdf:Bag></xmpDM:Tracks>
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;

    /// Attribute form, NTSC rate, no GUID on one marker.
    const ATTRIBUTES: &str = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about="" xmlns:xmpDM="http://ns.adobe.com/xmp/1.0/DynamicMedia/">
   <xmpDM:Tracks><rdf:Bag>
    <rdf:li><rdf:Description xmpDM:trackName="Comment" xmpDM:trackType="Comment" xmpDM:frameRate="f30000s1001">
     <xmpDM:markers><rdf:Seq>
      <rdf:li><rdf:Description xmpDM:startTime="300" xmpDM:name="Ten seconds" xmpDM:guid="11111111-0000-0000-0000-000000000000"/></rdf:li>
      <rdf:li><rdf:Description xmpDM:startTime="30" xmpDM:name="No guid"/></rdf:li>
     </rdf:Seq></xmpDM:markers>
    </rdf:Description></rdf:li>
   </rdf:Bag></xmpDM:Tracks>
  </rdf:Description>
 </rdf:RDF>
</x:xmpmeta>"#;

    fn meta(xml: &str) -> XmpMeta {
        XmpMeta::from_str(xml).expect("parse")
    }

    fn guids(markers: &[Marker]) -> HashSet<String> {
        markers.iter().filter_map(|m| m.guid.clone()).collect()
    }

    fn reparse(meta: &XmpMeta) -> XmpMeta {
        XmpMeta::from_str(&meta.to_string()).expect("reparse")
    }

    #[test]
    fn premiere_markers_are_read_with_every_field() {
        let markers = markers_of(&meta(PREMIERE));
        assert_eq!(markers.len(), 2, "the chapter marker is not shown");
        let first = &markers[0];
        assert_eq!(first.start_ms, 83_500);
        assert_eq!(first.duration_ms, 2_000);
        assert_eq!(first.name, "Take 3");
        assert_eq!(first.comment, "line one\nline two");
        assert_eq!(first.color, MarkerColor::Red);
        assert_eq!(
            first.guid.as_deref(),
            Some("c19c0922-775d-465c-917b-6a7c3a5c0902")
        );
        assert_eq!(markers[1].start_ms, 1_000);
        assert_eq!(markers[1].color, MarkerColor::Green);
    }

    #[test]
    fn attribute_form_and_fractional_rates_are_read() {
        let markers = markers_of(&meta(ATTRIBUTES));
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].start_ms, 10_010);
        assert_eq!(markers[0].name, "Ten seconds");
        assert_eq!(markers[1].guid, None);
        assert!(!markers[1].is_editable());
    }

    #[test]
    fn editing_one_marker_keeps_the_others_and_every_unknown_field() {
        let mut meta = meta(PREMIERE);
        let before = markers_of(&meta);
        let mut edited = before.clone();
        edited[0].name = "Take 4".to_string();
        edited[0].color = MarkerColor::Blue;
        assert!(apply_markers(&mut meta, &edited, &guids(&before)).expect("apply"));
        let meta = reparse(&meta);
        let after = markers_of(&meta);
        assert_eq!(after, edited);
        let first = marker_path(1, 1);
        assert_eq!(field(&meta, &first, "type").as_deref(), Some("Comment"));
        assert_eq!(
            field(&meta, &first, "startTime").as_deref(),
            Some("21210336000000"),
            "an unchanged time keeps its exact ticks"
        );
        let params = cue_params(&meta, &first);
        assert!(params
            .iter()
            .any(|(_, k, v)| k == "vendor_thing" && v == "keep me"));
        let (_, json) = color_param(&meta, &first).expect("color");
        let json: serde_json::Value = serde_json::from_str(&json).expect("json");
        assert_eq!(json["color"], MarkerColor::Blue.value().expect("blue"));
        assert_eq!(json["index"], 0, "other JSON fields survive");
        assert_eq!(
            field(&meta, &marker_path(1, 2), "speaker").as_deref(),
            Some("Anna")
        );
        assert_eq!(meta.array_len(XMP_DM, &markers_path(1)), 3, "chapter kept");
        assert_eq!(
            field(&meta, &track_path(2), "trackType").as_deref(),
            Some("InOut")
        );
    }

    #[test]
    fn nothing_changes_when_the_markers_are_the_same() {
        let mut meta = meta(PREMIERE);
        let markers = markers_of(&meta);
        assert!(!apply_markers(&mut meta, &markers, &guids(&markers)).expect("apply"));
    }

    #[test]
    fn a_new_marker_is_appended_once_in_the_track_rate() {
        let mut meta = meta(PREMIERE);
        let mut markers = markers_of(&meta);
        let mut new = Marker::new(36_000_000);
        new.name = "Ten hours".to_string();
        new.comment = "a\nb".to_string();
        new.duration_ms = 1_500;
        new.color = MarkerColor::Magenta;
        markers.push(new.clone());
        assert!(apply_markers(&mut meta, &markers, &HashSet::new()).expect("apply"));
        // Saving again (e.g. after undo and redo) cannot duplicate it.
        assert!(!apply_markers(&mut meta, &markers, &HashSet::new()).expect("apply"));
        let meta = reparse(&meta);
        let read = markers_of(&meta);
        assert_eq!(read.iter().filter(|m| m.guid == new.guid).count(), 1);
        assert_eq!(read.last(), Some(&new));
        let path = marker_path(1, 4);
        assert_eq!(
            field(&meta, &path, "startTime").as_deref(),
            Some("9144576000000000"),
            "10 h at f254016000000 does not overflow"
        );
        assert_eq!(field(&meta, &path, "comment").as_deref(), Some("a\rb"));
    }

    #[test]
    fn a_file_without_tracks_gets_a_millisecond_comment_track() {
        let mut meta = XmpMeta::new().expect("meta");
        let marker = Marker::new(1_234);
        assert!(
            apply_markers(&mut meta, std::slice::from_ref(&marker), &HashSet::new())
                .expect("apply")
        );
        let meta = reparse(&meta);
        assert_eq!(
            field(&meta, &track_path(1), "frameRate").as_deref(),
            Some("f1000")
        );
        assert_eq!(markers_of(&meta), vec![marker]);
    }

    #[test]
    fn only_known_markers_are_deleted_and_empty_tracks_go() {
        let mut meta = meta(PREMIERE);
        let markers = markers_of(&meta);
        // The user deleted the first marker; the second one frename never knew about.
        let known: HashSet<String> = markers[0].guid.iter().cloned().collect();
        assert!(apply_markers(&mut meta, &[], &known).expect("apply"));
        let left = markers_of(&reparse(&meta));
        assert_eq!(left.len(), 1);
        assert_eq!(left[0].name, "Second");

        let mut only_mine = XmpMeta::new().expect("meta");
        let mine = Marker::new(10);
        apply_markers(&mut only_mine, std::slice::from_ref(&mine), &HashSet::new()).expect("add");
        apply_markers(&mut only_mine, &[], &guids(&[mine])).expect("delete");
        assert!(!only_mine.contains_property(XMP_DM, TRACKS));
    }

    #[test]
    fn prefixed_guids_match_their_marker() {
        let mut meta = meta(PREMIERE);
        let mut markers = markers_of(&meta);
        markers[1].name = "Renamed".to_string();
        apply_markers(&mut meta, &markers, &guids(&markers)).expect("apply");
        let read = markers_of(&reparse(&meta));
        assert_eq!(read.len(), 2);
        assert_eq!(read[1].name, "Renamed");
    }

    #[test]
    fn markers_without_a_guid_are_never_touched() {
        let mut meta = meta(ATTRIBUTES);
        let mut markers = markers_of(&meta);
        markers[1].name = "Changed".to_string();
        markers.remove(0);
        apply_markers(&mut meta, &markers, &HashSet::new()).expect("apply");
        let read = markers_of(&reparse(&meta));
        assert_eq!(read.len(), 2);
        assert_eq!(read[1].name, "No guid");
    }

    #[test]
    fn green_removes_the_color_parameter() {
        let mut meta = meta(PREMIERE);
        let mut markers = markers_of(&meta);
        markers[0].color = MarkerColor::Green;
        apply_markers(&mut meta, &markers, &guids(&markers)).expect("apply");
        let meta = reparse(&meta);
        assert!(color_param(&meta, &marker_path(1, 1)).is_none());
        assert_eq!(markers_of(&meta)[0].color, MarkerColor::Green);
    }

    #[test]
    fn rates_convert_both_ways() {
        let ntsc = Rate::parse("f30000s1001").expect("rate");
        assert_eq!(ntsc.to_ms(30), 1_001);
        assert_eq!(ntsc.to_ticks(1_001), 30);
        let premiere = Rate::parse("f254016000000").expect("rate");
        assert_eq!(premiere.to_ticks(36_000_000), 9_144_576_000_000_000);
        assert_eq!(premiere.to_ms(9_144_576_000_000_000), 36_000_000);
        assert_eq!(Rate::parse("f0"), None);
        assert_eq!(Rate::parse("25"), None);
    }
}
