use super::*;

/// A file as a run of frename leaves it: one AI part, one AI moment, one moment of the editor's.
const FILE: &str = r#"{
  "version": 1,
  "duration_s": 60.0,
  "frame": {"width": 1920, "height": 1080, "fps": 25.0},
  "lang": "en",
  "summary": "A walk to the fortress.",
  "parts": [
    {"id": "k3m9xa", "by": "ai", "start_s": 0.0, "end_s": 60.0, "title": "Fortress walk",
     "description": "Someone walks up to the gate."}
  ],
  "moments": [
    {"id": "p2r6kc", "by": "ai", "start_s": 14.0, "end_s": 16.5, "title": "Gate",
     "description": "The gate comes into view."},
    {"id": "1qc62f", "by": "user", "start_s": 30.1, "title": "Lion", "color": "red"}
  ]
}"#;

fn file() -> DescribeFile {
    DescribeFile::parse(FILE).expect("a valid file")
}

/// The markers frename has for [`FILE`]: the AI's as [`ai_markers`] makes them, and the editor's
/// red point.
fn markers(file: &DescribeFile) -> Vec<Marker> {
    let mut markers = ai_markers(file, Path::new("clip.mov"));
    let mut lion = Marker::new(30_100);
    lion.id = Some("1qc62f".to_string());
    lion.name = "Lion".to_string();
    lion.color = MarkerColor::Red;
    markers.push(lion);
    markers
}

fn ids(markers: &[Marker]) -> HashSet<String> {
    markers.iter().filter_map(|m| m.id.clone()).collect()
}

fn find<'a>(file: &'a DescribeFile, id: &str) -> &'a Item {
    items(file).find(|item| item.id == id).expect("the item")
}

fn no_names() -> HashMap<String, String> {
    HashMap::new()
}

#[test]
fn the_ai_items_become_ai_markers_with_their_ids_titles_and_texts() {
    let file = file();
    let markers = ai_markers(&file, Path::new("clip.mov"));
    let got: Vec<_> = markers
        .iter()
        .map(|m| {
            (
                m.id.as_deref().unwrap_or_default(),
                m.start_ms,
                m.duration_ms,
                m.name.as_str(),
                m.comment.as_str(),
                m.ai,
                m.color,
            )
        })
        .collect();
    assert_eq!(
        got,
        [
            (
                "k3m9xa",
                0,
                60_000,
                "Fortress walk",
                "Someone walks up to the gate.",
                true,
                AI_MARKER_COLOR
            ),
            (
                "p2r6kc",
                14_000,
                2_500,
                "Gate",
                "The gate comes into view.",
                true,
                AI_MARKER_COLOR
            ),
        ]
    );
    // The GUID Premiere sees is made from the short id, never the other way round.
    assert_eq!(markers[1].guid, Some(guid_for_id("p2r6kc")));
}

#[test]
fn mentions_are_shown_as_the_speakers_name_or_the_persons_label() {
    let mut file = file();
    file.people = Some(vec![
        Person {
            id: "p1".to_string(),
            by: By::Ai,
            label: Some("a woman in a red jacket".to_string()),
            speaker: Some("1".to_string()),
            faces: Vec::new(),
            extra: Extra::new(),
        },
        Person {
            id: "p2".to_string(),
            by: By::Ai,
            label: Some("a vendor".to_string()),
            speaker: None,
            faces: Vec::new(),
            extra: Extra::new(),
        },
    ]);
    let moments = file.moments.as_mut().expect("moments");
    moments[0].description = Some("@p1 buys corn from @p2.".to_string());
    let names = HashMap::from([("1".to_string(), "Ann".to_string())]);
    let people = people(&file).to_vec();
    assert_eq!(
        shown(
            find(&file, "p2r6kc").description.as_deref(),
            &people,
            &names
        ),
        "Ann buys corn from a vendor."
    );
    // As it was written: following an unchanged marker does not turn the mention into a name.
    let mut markers = markers(&file);
    markers[1].comment = "Ann buys corn from a vendor.".to_string();
    let before = ids(&markers);
    assert!(!apply_markers(&mut file, &markers, &before, &names));
    assert_eq!(
        find(&file, "p2r6kc").description.as_deref(),
        Some("@p1 buys corn from @p2.")
    );
}

#[test]
fn markers_as_the_file_has_them_change_nothing() {
    let mut file = file();
    let markers = markers(&file);
    assert!(!apply_markers(
        &mut file,
        &markers,
        &ids(&markers),
        &no_names()
    ));
    assert_eq!(file, self::file());
    assert!(file.problems().is_empty(), "{:?}", file.problems());
}

#[test]
fn a_marker_the_editor_renamed_recolored_or_moved_is_theirs_in_the_file() {
    let mut file = file();
    let mut markers = markers(&file);
    let before = ids(&markers);
    // The gate moment: renamed and moved, as the editor's change clears `ai`.
    markers[1].name = "Main gate".to_string();
    markers[1].start_ms = 13_000;
    markers[1].ai = false;
    // The part: recolored.
    markers[0].color = MarkerColor::Blue;
    markers[0].ai = false;
    assert!(apply_markers(&mut file, &markers, &before, &no_names()));
    let gate = find(&file, "p2r6kc");
    assert_eq!(gate.by, By::User);
    assert_eq!(gate.title.as_deref(), Some("Main gate"));
    assert_eq!(
        (gate.start_s, gate.end_s),
        (13.0, Some(15.5)),
        "moved with its length"
    );
    assert_eq!(gate.color.as_deref(), Some("white"), "the colour it has");
    let part = find(&file, "k3m9xa");
    assert_eq!(part.by, By::User);
    assert_eq!(part.color.as_deref(), Some("blue"));
    assert!(file.problems().is_empty(), "{:?}", file.problems());
}

#[test]
fn an_undo_that_gives_the_ai_marker_back_makes_the_item_the_ais_again() {
    let original = file();
    let mut file = original.clone();
    let mut markers = markers(&file);
    let before = ids(&markers);
    let ai = markers[1].clone();
    markers[1].name = "Main gate".to_string();
    markers[1].ai = false;
    assert!(apply_markers(&mut file, &markers, &before, &no_names()));
    markers[1] = ai;
    assert!(apply_markers(&mut file, &markers, &before, &no_names()));
    assert_eq!(file, original);
}

#[test]
fn a_marker_the_editor_added_is_a_moment_of_theirs_and_one_they_deleted_goes() {
    let mut file = file();
    let mut markers = markers(&file);
    let before = ids(&markers);
    // The editor's lion point is deleted; a green range is added.
    markers.retain(|m| m.id.as_deref() != Some("1qc62f"));
    let mut added = Marker::new(40_000);
    added.duration_ms = 2_000;
    added.name = "Crowd".to_string();
    added.comment = "Line one\nline two".to_string();
    let added_id = added.id.clone().expect("an id");
    markers.push(added);
    assert!(apply_markers(&mut file, &markers, &before, &no_names()));
    assert!(items(&file).all(|item| item.id != "1qc62f"));
    let crowd = find(&file, &added_id);
    assert_eq!(crowd.by, By::User);
    assert_eq!((crowd.start_s, crowd.end_s), (40.0, Some(42.0)));
    assert_eq!(crowd.description.as_deref(), Some("Line one\nline two"));
    assert_eq!(crowd.color.as_deref(), Some("green"), "Premiere's default");
    assert!(file.problems().is_empty(), "{:?}", file.problems());
}

#[test]
fn an_item_whose_marker_was_never_in_the_video_stays() {
    let mut file = file();
    // Only the AI part is in the video, and was before: the moments came from elsewhere.
    let markers: Vec<Marker> = markers(&file).into_iter().take(1).collect();
    let before = ids(&markers);
    assert!(!apply_markers(&mut file, &markers, &before, &no_names()));
    assert_eq!(file, self::file());
}

#[test]
fn a_part_made_a_point_is_a_moment() {
    let mut file = file();
    let mut markers = markers(&file);
    let before = ids(&markers);
    markers[0].duration_ms = 0;
    markers[0].ai = false;
    assert!(apply_markers(&mut file, &markers, &before, &no_names()));
    assert_eq!(file.parts.as_deref().map(<[Item]>::len), Some(0));
    let point = find(&file, "k3m9xa");
    assert_eq!(
        (point.start_s, point.end_s, point.by),
        (0.0, None, By::User)
    );
    assert!(file.problems().is_empty(), "{:?}", file.problems());
}

#[test]
fn a_millisecond_of_rounding_by_the_video_is_not_a_move() {
    let mut file = file();
    let mut markers = markers(&file);
    let before = ids(&markers);
    markers[1].start_ms += 1;
    assert!(!apply_markers(&mut file, &markers, &before, &no_names()));
}

#[test]
fn an_ai_marker_from_before_titles_gets_a_short_title_and_its_text_as_the_description() {
    let mut file = file();
    let mut old = Marker::new(50_000);
    old.ai = true;
    old.color = AI_MARKER_COLOR;
    old.name = "A long sentence the AI wrote before it gave titles to moments.".to_string();
    let id = old.id.clone().expect("an id");
    let mut markers = markers(&file);
    markers.push(old.clone());
    let before = ids(&markers);
    assert!(apply_markers(&mut file, &markers, &before, &no_names()));
    let item = find(&file, &id);
    assert_eq!(item.by, By::Ai);
    assert_eq!(item.description.as_deref(), Some(old.name.as_str()));
    assert!(item.title.as_deref().is_some_and(|t| !t.ends_with('.')));
    assert!(file.problems().is_empty(), "{:?}", file.problems());
}

#[test]
fn in_and_out_follow_the_clip() {
    let mut file = file();
    assert!(apply_in_out(
        &mut file,
        Segment {
            start: Some(1.5),
            end: None
        }
    ));
    let io = file.in_out.clone().expect("in/out");
    assert_eq!((io.start_s, io.end_s), (Some(1.5), None));
    assert!(!apply_in_out(
        &mut file,
        Segment {
            start: Some(1.5),
            end: None
        }
    ));
    assert!(apply_in_out(
        &mut file,
        Segment {
            start: Some(3.2),
            end: Some(41.5)
        }
    ));
    let io = file.in_out.clone().expect("in/out");
    assert_eq!((io.start_s, io.end_s), (Some(3.2), Some(41.5)));
    assert!(file.problems().is_empty(), "{:?}", file.problems());
    assert!(apply_in_out(&mut file, Segment::default()));
    assert_eq!(file.in_out, None);
}

#[test]
fn colours_have_their_names_and_others_their_value() {
    assert_eq!(color_name(MarkerColor::Green), "green");
    assert_eq!(color_name(MarkerColor::Lavender), "lavender");
    // 0xAABBGGRR
    assert_eq!(color_name(MarkerColor::Other(0x8033_2211)), "#11223380");
    assert!(clipscribe::describe_file::is_color(&color_name(
        MarkerColor::Other(0xFF00_FF00)
    )));
}

/// A copy of the tiny fixture clip with [`FILE`] next to it.
fn clip_with_file(test: &str) -> PathBuf {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tiny.mov");
    let dir = crate::test_support::fresh_dir(&format!("description-{test}"));
    let clip = dir.join("clip.mov");
    std::fs::copy(fixture, &clip).expect("copy fixture");
    std::fs::write(path(&clip), FILE).expect("description file");
    clip
}

#[test]
fn a_marker_renamed_in_the_video_is_renamed_in_the_file_and_an_undo_brings_it_back() {
    let clip = clip_with_file("rename");
    let markers = markers(&file());
    crate::metadata::save_markers(&clip, &markers, &HashSet::new()).expect("saved");
    assert_eq!(read(&clip), Some(file()), "nothing to change");

    let mut renamed = markers.clone();
    renamed[1].name = "Main gate".to_string();
    renamed[1].ai = false;
    crate::metadata::save_markers(&clip, &renamed, &HashSet::new()).expect("renamed");
    let changed = read(&clip).expect("file");
    assert_eq!(find(&changed, "p2r6kc").title.as_deref(), Some("Main gate"));
    assert_eq!(find(&changed, "p2r6kc").by, By::User);

    crate::metadata::save_markers(&clip, &markers, &HashSet::new()).expect("undone");
    assert_eq!(read(&clip), Some(file()));
    let _ = std::fs::remove_dir_all(clip.parent().expect("dir"));
}

#[test]
fn a_file_this_version_cannot_read_is_never_written() {
    let clip = clip_with_file("unreadable");
    let newer = r#"{"version": 2, "parts": "not like this"}"#;
    std::fs::write(path(&clip), newer).expect("newer file");
    let mut marker = Marker::new(1_000);
    marker.name = "Mine".to_string();
    crate::metadata::save_markers(&clip, &[marker], &HashSet::new()).expect("saved");
    crate::description::in_out_saved(
        &clip,
        Segment {
            start: Some(0.1),
            end: None,
        },
    );
    assert_eq!(std::fs::read_to_string(path(&clip)).expect("kept"), newer);
    let _ = std::fs::remove_dir_all(clip.parent().expect("dir"));
}

#[test]
fn a_clip_without_the_file_gets_none_from_an_edit() {
    let clip = clip_with_file("none");
    std::fs::remove_file(path(&clip)).expect("no file");
    crate::metadata::save_markers(&clip, &[Marker::new(10)], &HashSet::new()).expect("saved");
    in_out_saved(
        &clip,
        Segment {
            start: Some(0.1),
            end: None,
        },
    );
    assert!(!path(&clip).exists());
    let _ = std::fs::remove_dir_all(clip.parent().expect("dir"));
}

/// The owner's rule for ids: a marker made in Premiere (a GUID, no `frename_id`) gets a short
/// id the first time frename reads it, keeps it on every later read, and keeps its GUID.
#[test]
fn a_premiere_marker_gets_a_short_id_once_and_keeps_its_guid() {
    let clip = clip_with_file("premiere");
    let premiere_guid = "e8dd1f3f-12f9-41f0-8672-e3296ce19cec";
    let mut premiere = Marker::new(5_000);
    premiere.guid = Some(premiere_guid.to_string());
    premiere.id = None;
    premiere.color = MarkerColor::Green;
    crate::metadata::save_markers(&clip, &[premiere], &HashSet::new()).expect("saved");

    let first = crate::metadata::load_markers(&clip).expect("markers");
    let id = first[0].id.clone().expect("an id given at the first read");
    assert!(clipscribe::describe_file::is_short_id(&id), "{id}");
    assert_eq!(first[0].guid.as_deref(), Some(premiere_guid));
    assert!(!first[0].ai, "a green marker of Premiere is the editor's");
    for _ in 0..2 {
        let again = crate::metadata::load_markers(&clip).expect("markers");
        assert_eq!(again[0].id.as_deref(), Some(id.as_str()), "the same id");
        assert_eq!(
            again[0].guid.as_deref(),
            Some(premiere_guid),
            "the GUID stays"
        );
    }
    let _ = std::fs::remove_dir_all(clip.parent().expect("dir"));
}

/// Described before ids: an AI run wrote white markers without parameters. On a described clip
/// (its comment has an `AI:` block) they read as the AI's, and keep being the AI's once frename
/// has given them an id; on a clip never described, a white marker is the editor's.
#[test]
fn a_white_marker_from_before_ids_is_the_ais_only_on_a_described_clip() {
    let clip = clip_with_file("legacy-ai");
    let white = || {
        let mut legacy = Marker::new(1_000);
        legacy.guid = Some("c19c0922-775d-465c-917b-6a7c3a5c0902".to_string());
        legacy.id = None;
        legacy.color = AI_MARKER_COLOR;
        legacy
    };
    crate::metadata::save_markers(&clip, &[white()], &HashSet::new()).expect("saved");
    for _ in 0..2 {
        let read = crate::metadata::load_markers(&clip).expect("markers");
        assert!(
            !read[0].ai && read[0].id.is_some(),
            "the editor's: {read:?}"
        );
    }

    let described = clip.with_file_name("described.mov");
    std::fs::copy(&clip, &described).expect("copy");
    crate::comment::save_comment(&described, "Mine\n\nAI: A lion walks past.");
    let mut legacy = white();
    legacy.guid = Some("0b8e7d1c-1f4a-4b5e-9c3d-2a6f8e9b7c10".to_string());
    crate::metadata::save_markers(&described, &[legacy], &HashSet::new()).expect("saved");
    for _ in 0..2 {
        let read = crate::metadata::load_markers(&described).expect("markers");
        let mine = read.iter().find(|m| {
            m.start_ms == 1_000 && m.guid.as_deref() == Some("0b8e7d1c-1f4a-4b5e-9c3d-2a6f8e9b7c10")
        });
        assert!(
            mine.is_some_and(|m| m.ai && m.id.is_some()),
            "the AI's: {read:?}"
        );
    }
    let _ = std::fs::remove_dir_all(clip.parent().expect("dir"));
}

/// The debug backend (`--debug`) never writes a file: reading markers that have no id gives
/// them one in memory only, the same on every read.
#[test]
fn the_in_memory_backend_gives_ids_without_writing_the_video() {
    use crate::FileTaggerBackend;
    let clip = clip_with_file("in-memory");
    let mut premiere = Marker::new(10);
    premiere.id = None;
    crate::metadata::save_markers(&clip, &[premiere], &HashSet::new()).expect("saved");
    let bytes = std::fs::read(&clip).expect("clip");
    let backend = crate::InMemoryFileTagger::default();
    let first = backend.load_markers(&clip).expect("markers");
    let again = backend.load_markers(&clip).expect("markers");
    assert!(first[0].id.is_some());
    assert_eq!(first[0].id, again[0].id, "the same id");
    assert_eq!(
        std::fs::read(&clip).expect("clip"),
        bytes,
        "the video is untouched"
    );
    let _ = std::fs::remove_dir_all(clip.parent().expect("dir"));
}

/// Two AI parts covering the clip, as a run with Moments = Full writes them.
fn two_parts() -> DescribeFile {
    let mut file = file();
    file.moments = None;
    let parts = file.parts.as_mut().expect("parts");
    parts[0].end_s = Some(30.0);
    let mut second = parts[0].clone();
    second.id = "b4n7qe".to_string();
    second.start_s = 30.0;
    second.end_s = Some(60.0);
    second.title = Some("Gate".to_string());
    parts.push(second);
    assert!(file.problems().is_empty(), "{:?}", file.problems());
    file
}

#[test]
fn a_part_the_editor_resizes_is_a_moment_and_the_parts_still_cover_the_clip() {
    let mut file = two_parts();
    let mut markers = ai_markers(&file, Path::new("clip.mov"));
    let before = ids(&markers);
    let ai = markers[0].clone();
    // The first part's end dragged from 0:30 to 0:20.
    markers[0].duration_ms = 20_000;
    markers[0].ai = false;
    assert!(apply_markers(&mut file, &markers, &before, &no_names()));
    assert!(file.problems().is_empty(), "{:?}", file.problems());
    let resized = find(&file, "k3m9xa");
    assert_eq!((resized.end_s, resized.by), (Some(20.0), By::User));
    assert!(file.moments.iter().flatten().any(|m| m.id == "k3m9xa"));
    assert_eq!(
        find(&file, "b4n7qe").start_s,
        0.0,
        "the other part covers the clip"
    );

    // Undone: the marker is the AI's again, as it was; the file stays valid.
    markers[0] = ai;
    apply_markers(&mut file, &markers, &before, &no_names());
    assert!(file.problems().is_empty(), "{:?}", file.problems());
    let back = find(&file, "k3m9xa");
    assert_eq!(
        (back.start_s, back.end_s, back.by),
        (0.0, Some(30.0), By::Ai)
    );
    // A second save with nothing new changes nothing.
    assert!(!apply_markers(&mut file, &markers, &before, &no_names()));
}

#[test]
fn a_part_the_editor_deletes_goes_and_the_parts_still_cover_the_clip() {
    let mut file = two_parts();
    let markers = ai_markers(&file, Path::new("clip.mov"));
    let before = ids(&markers);
    assert!(apply_markers(
        &mut file,
        &markers[1..],
        &before,
        &no_names()
    ));
    assert!(file.problems().is_empty(), "{:?}", file.problems());
    assert!(items(&file).all(|item| item.id != "k3m9xa"));
    let left = find(&file, "b4n7qe");
    assert_eq!((left.start_s, left.end_s), (0.0, Some(60.0)));

    // Undone: the marker comes back, as a moment; the file stays valid.
    assert!(apply_markers(&mut file, &markers, &before, &no_names()));
    assert!(file.problems().is_empty(), "{:?}", file.problems());
    assert!(items(&file).any(|item| item.id == "k3m9xa"));
}
