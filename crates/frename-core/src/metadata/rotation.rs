//! Rotation of MP4/MOV videos: the display matrix in the track header (`moov/trak/tkhd`) of each
//! video track, which Premiere Pro, GStreamer, VLC and Windows read to show the picture turned.
//!
//! Turning a video rewrites only those 36 bytes per video track, in place: the picture is not
//! re-encoded, the file keeps its size, and everything else in it (XMP included) stays byte for
//! byte. See `docs/research/video-rotation.md`.
//!
//! The matrix is `a b u / c d v / x y w`; a point `(p, q)` of the stored picture is shown at
//! `(a·p + c·q + x, b·p + d·q + y)` with `y` pointing down. `a b c d x y` are 16.16 fixed point.

use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::Path;

use super::bmff::{self, BoxHeader};
use super::xmp::FileTimes;

/// 1.0 in 16.16 fixed point.
const ONE: i32 = 0x0001_0000;
/// Length of the matrix in bytes: nine 32-bit values.
const MATRIX_LEN: usize = 36;

/// How a video is shown: turned clockwise by a number of quarter turns, and possibly mirrored
/// (left to right, before the turn).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rotation {
    quarter_turns: u8,
    mirrored: bool,
}

impl Rotation {
    /// Shown as stored.
    pub const UPRIGHT: Rotation = Rotation {
        quarter_turns: 0,
        mirrored: false,
    };

    /// Turned `degrees` clockwise (a multiple of 90; anything else is rounded down to one).
    pub fn clockwise(degrees: u16) -> Self {
        Self {
            quarter_turns: ((degrees / 90) % 4) as u8,
            mirrored: false,
        }
    }

    /// The clockwise turn in degrees: 0, 90, 180 or 270.
    pub fn degrees(self) -> u16 {
        u16::from(self.quarter_turns) * 90
    }

    /// Whether the picture is also mirrored.
    pub fn mirrored(self) -> bool {
        self.mirrored
    }

    /// This rotation turned further by `quarter_turns` (negative: counter-clockwise).
    pub fn turned(self, quarter_turns: i32) -> Self {
        Self {
            quarter_turns: (i32::from(self.quarter_turns) + quarter_turns).rem_euclid(4) as u8,
            mirrored: self.mirrored,
        }
    }

    /// The quarter turns that bring this rotation back to 0° (the mirror stays).
    pub fn turns_to_upright(self) -> i32 {
        (4 - i32::from(self.quarter_turns)) % 4
    }

    /// The 2×2 part of the matrix, entries -1, 0 or 1: a mirror first, then the turn.
    fn matrix(self) -> Mat2 {
        let mut m = if self.mirrored {
            Mat2 {
                a: -1,
                b: 0,
                c: 0,
                d: 1,
            }
        } else {
            Mat2::IDENTITY
        };
        for _ in 0..self.quarter_turns {
            m = m.times(Mat2::QUARTER_TURN);
        }
        m
    }

    fn of(m: Mat2) -> Option<Self> {
        (0..4u8)
            .flat_map(|turns| {
                [false, true].map(|mirrored| Rotation {
                    quarter_turns: turns,
                    mirrored,
                })
            })
            .find(|r| r.matrix() == m)
    }
}

/// Why a video's rotation cannot be read or changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RotationError {
    /// The format has no rotation flag readers follow (MKV, AVI, MTS, …).
    CannotRotate,
    /// Named MOV/MP4, but the content is not a movie (a download or copy that did not finish).
    Damaged,
    /// The movie has no video track.
    NoVideoTrack,
    /// The track's matrix scales, skews or turns by an odd angle: frename leaves it alone.
    UnusualMatrix,
    /// Reading or writing the file failed: read-only, open in another app (Premiere), gone.
    /// The text says what the system reported.
    Io(String),
}

impl std::fmt::Display for RotationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CannotRotate => write!(f, "this format has no rotation flag"),
            Self::Damaged => write!(
                f,
                "the file is damaged: its content is not a video (a download or copy that did \
                 not finish?)"
            ),
            Self::NoVideoTrack => write!(f, "the file has no video track"),
            Self::UnusualMatrix => write!(f, "the video has an unusual display matrix"),
            Self::Io(reason) => write!(f, "{reason}"),
        }
    }
}

impl From<io::Error> for RotationError {
    fn from(e: io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

/// The rotation of the video at `path`: its first video track's.
pub(crate) fn read(path: &Path) -> Result<Rotation, RotationError> {
    check_format(path)?;
    let mut file = File::open(path)?;
    let tracks = video_tracks(&mut file, path)?;
    Ok(tracks[0].rotation)
}

/// Turn the video at `path` by `quarter_turns` clockwise (negative: counter-clockwise), in place,
/// keeping the file's modified and created times. Returns the new rotation of its first video
/// track. Nothing is written when any video track's matrix is one frename leaves alone, or when
/// the turn is a whole number of full turns.
pub(crate) fn rotate(path: &Path, quarter_turns: i32) -> Result<Rotation, RotationError> {
    check_format(path)?;
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let tracks = video_tracks(&mut file, path)?;
    let first = tracks[0].rotation.turned(quarter_turns);
    if quarter_turns.rem_euclid(4) == 0 {
        return Ok(first);
    }
    let times = FileTimes::read(path)?;
    for track in &tracks {
        let turned = track.rotation.turned(quarter_turns);
        let bytes = track.matrix_bytes(turned);
        file.seek(SeekFrom::Start(track.matrix_at))?;
        file.write_all(&bytes)?;
    }
    file.sync_all()?;
    drop(file);
    times.restore(path)?;
    Ok(first)
}

fn check_format(path: &Path) -> Result<(), RotationError> {
    let is_movie = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "mov" | "mp4" | "m4v"));
    if !is_movie {
        return Err(RotationError::CannotRotate);
    }
    if bmff::is_damaged(path) {
        return Err(RotationError::Damaged);
    }
    Ok(())
}

/// A 2×2 matrix with entries -1, 0 or 1 (in units of 1.0).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Mat2 {
    a: i32,
    b: i32,
    c: i32,
    d: i32,
}

impl Mat2 {
    const IDENTITY: Mat2 = Mat2 {
        a: 1,
        b: 0,
        c: 0,
        d: 1,
    };
    /// 90° clockwise: `(p, q)` → `(-q, p)`.
    const QUARTER_TURN: Mat2 = Mat2 {
        a: 0,
        b: 1,
        c: -1,
        d: 0,
    };

    /// `self` applied first, then `then` (row vectors: `p · self · then`).
    fn times(self, then: Mat2) -> Mat2 {
        Mat2 {
            a: self.a * then.a + self.b * then.c,
            b: self.a * then.b + self.b * then.d,
            c: self.c * then.a + self.d * then.c,
            d: self.c * then.b + self.d * then.d,
        }
    }

    /// The translation that puts a `width`×`height` picture shown through this matrix back at
    /// the origin (16.16 fixed point in and out).
    fn origin_translation(self, width: i64, height: i64) -> (i64, i64) {
        let corners = [(0, 0), (width, 0), (0, height), (width, height)];
        let min_x = corners
            .iter()
            .map(|&(p, q)| i64::from(self.a) * p + i64::from(self.c) * q)
            .min()
            .unwrap_or(0);
        let min_y = corners
            .iter()
            .map(|&(p, q)| i64::from(self.b) * p + i64::from(self.d) * q)
            .min()
            .unwrap_or(0);
        (-min_x, -min_y)
    }
}

/// A video track's header, as far as rotation goes.
#[derive(Debug)]
struct VideoTrack {
    /// Absolute offset of the matrix in the file.
    matrix_at: u64,
    /// The matrix as stored.
    matrix: [u8; MATRIX_LEN],
    rotation: Rotation,
    /// Track width and height, 16.16 fixed point.
    width: i64,
    height: i64,
}

impl VideoTrack {
    /// The matrix for `rotation`, with this track's `u v w`. The translation moves the turned
    /// picture back to the origin, as phones and exiftool write it; ffmpeg writes none, which
    /// the readers that matter ignore, so a file in that style gets the phones' one.
    fn matrix_bytes(&self, rotation: Rotation) -> [u8; MATRIX_LEN] {
        let m = rotation.matrix();
        let (x, y) = m.origin_translation(self.width, self.height);
        let mut bytes = self.matrix;
        let mut put = |index: usize, value: i32| {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_be_bytes());
        };
        put(0, m.a * ONE);
        put(1, m.b * ONE);
        put(3, m.c * ONE);
        put(4, m.d * ONE);
        // A track size that does not fit the matrix's 32 bits has no translation to write.
        put(6, i32::try_from(x).unwrap_or(0));
        put(7, i32::try_from(y).unwrap_or(0));
        bytes
    }
}

/// Every video track of the movie, in file order. At least one, or an error.
fn video_tracks(file: &mut File, path: &Path) -> Result<Vec<VideoTrack>, RotationError> {
    let len = file.metadata()?.len();
    let mut tracks = Vec::new();
    let mut pos = 0;
    while let Some(b) = bmff::read_header(file, pos, len)? {
        if &b.kind == b"moov" {
            let mut child_pos = b.payload;
            while let Some(child) = bmff::read_header(file, child_pos, b.end)? {
                if &child.kind == b"trak" {
                    if let Some(track) = video_track(file, &child)? {
                        tracks.push(track);
                    }
                }
                child_pos = child.end;
            }
        }
        pos = b.end;
    }
    if tracks.is_empty() {
        log::info!("rotation: no video track in {path:?}");
        return Err(RotationError::NoVideoTrack);
    }
    Ok(tracks)
}

/// The track in `trak`, when it is a video track (`mdia/hdlr` says `vide`).
fn video_track(file: &mut File, trak: &BoxHeader) -> Result<Option<VideoTrack>, RotationError> {
    let mut tkhd = None;
    let mut is_video = false;
    let mut pos = trak.payload;
    while let Some(b) = bmff::read_header(file, pos, trak.end)? {
        match &b.kind {
            b"tkhd" => tkhd = Some((b.payload, b.end)),
            b"mdia" => is_video = media_is_video(file, &b)?,
            _ => {}
        }
        pos = b.end;
    }
    let (Some((payload, end)), true) = (tkhd, is_video) else {
        return Ok(None);
    };
    let version = bmff::read_bytes(file, payload, 1)?[0];
    // Version, flags, times, track id, reserved, duration, then 8 reserved bytes, layer,
    // alternate group, volume and 2 reserved bytes before the matrix.
    let matrix_offset = match version {
        0 => 40,
        1 => 52,
        _ => return Err(RotationError::UnusualMatrix),
    };
    let matrix_at = payload + matrix_offset;
    // Matrix, then width and height.
    if matrix_at + MATRIX_LEN as u64 + 8 > end {
        return Err(bmff::invalid().into());
    }
    let bytes = bmff::read_bytes(file, matrix_at, MATRIX_LEN + 8)?;
    let value = |index: usize| {
        i32::from_be_bytes([
            bytes[index * 4],
            bytes[index * 4 + 1],
            bytes[index * 4 + 2],
            bytes[index * 4 + 3],
        ])
    };
    let unit = |raw: i32| match raw {
        0 => Some(0),
        ONE => Some(1),
        r if r == -ONE => Some(-1),
        _ => None,
    };
    let (Some(a), Some(b), Some(c), Some(d)) = (
        unit(value(0)),
        unit(value(1)),
        unit(value(3)),
        unit(value(4)),
    ) else {
        return Err(RotationError::UnusualMatrix);
    };
    let m = Mat2 { a, b, c, d };
    let rotation = Rotation::of(m).ok_or(RotationError::UnusualMatrix)?;
    let width = i64::from(u32::from_be_bytes([
        bytes[36], bytes[37], bytes[38], bytes[39],
    ]));
    let height = i64::from(u32::from_be_bytes([
        bytes[40], bytes[41], bytes[42], bytes[43],
    ]));
    let stored = (i64::from(value(6)), i64::from(value(7)));
    // Placed at the origin (phones, exiftool) or not moved at all (ffmpeg); a picture placed
    // somewhere of its own could not be brought back there by turns.
    if stored != m.origin_translation(width, height) && stored != (0, 0) {
        return Err(RotationError::UnusualMatrix);
    }
    let mut matrix = [0; MATRIX_LEN];
    matrix.copy_from_slice(&bytes[..MATRIX_LEN]);
    Ok(Some(VideoTrack {
        matrix_at,
        matrix,
        rotation,
        width,
        height,
    }))
}

/// Whether the media in `mdia` is video: its handler (`mdia/hdlr`, not the data handler in
/// `minf` that MOV files also have) is `vide`.
fn media_is_video(file: &mut File, mdia: &BoxHeader) -> io::Result<bool> {
    let mut pos = mdia.payload;
    while let Some(b) = bmff::read_header(file, pos, mdia.end)? {
        if &b.kind == b"hdlr" && b.end >= b.payload + 12 {
            // Version and flags, pre-defined (the component type in QuickTime), handler type.
            return Ok(&bmff::read_bytes(file, b.payload + 8, 4)?[..] == b"vide");
        }
        pos = b.end;
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const FIXTURES: [&str; 2] = ["wide.mp4", "wide.mov"];

    /// A fresh copy of a 32×16 H.264 + AAC fixture (0.2 s), in its own temp folder.
    fn copy_of(fixture: &str, test: &str) -> PathBuf {
        let source = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(fixture);
        let dir = std::env::temp_dir().join(format!(
            "frename-rotation-{test}-{}-{}",
            fixture.replace('.', "-"),
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let file = dir.join(fixture);
        std::fs::copy(source, &file).expect("copy fixture");
        file
    }

    /// The matrices of every track (audio included), as `[a, b, u, c, d, v, x, y, w]`.
    fn all_matrices(path: &Path) -> Vec<[i32; 9]> {
        let mut file = File::open(path).expect("open");
        let len = file.metadata().expect("meta").len();
        let mut found = Vec::new();
        let mut pos = 0;
        while let Some(b) = bmff::read_header(&mut file, pos, len).expect("box") {
            if &b.kind == b"moov" {
                let mut child_pos = b.payload;
                while let Some(trak) = bmff::read_header(&mut file, child_pos, b.end).expect("box")
                {
                    if &trak.kind == b"trak" {
                        let mut inner = trak.payload;
                        while let Some(t) =
                            bmff::read_header(&mut file, inner, trak.end).expect("box")
                        {
                            if &t.kind == b"tkhd" {
                                let bytes =
                                    bmff::read_bytes(&mut file, t.payload + 40, 36).expect("read");
                                let mut m = [0; 9];
                                for (i, v) in m.iter_mut().enumerate() {
                                    *v = i32::from_be_bytes(
                                        bytes[i * 4..i * 4 + 4].try_into().expect("4 bytes"),
                                    );
                                }
                                found.push(m);
                            }
                            inner = t.end;
                        }
                    }
                    child_pos = trak.end;
                }
            }
            pos = b.end;
        }
        found
    }

    const W: i32 = 32 * ONE;
    const H: i32 = 16 * ONE;
    const IDENTITY: [i32; 9] = [ONE, 0, 0, 0, ONE, 0, 0, 0, 0x4000_0000];

    #[test]
    fn the_fixtures_start_upright_with_a_video_and_an_audio_track() {
        for fixture in FIXTURES {
            let file = copy_of(fixture, "upright");
            assert_eq!(read(&file), Ok(Rotation::UPRIGHT), "{fixture}");
            assert_eq!(all_matrices(&file), vec![IDENTITY, IDENTITY], "{fixture}");
        }
    }

    #[test]
    fn each_turn_writes_the_matrix_phones_write() {
        let expected = [
            (1, 90, [0, ONE, 0, -ONE, 0, 0, H, 0, 0x4000_0000]),
            (2, 180, [-ONE, 0, 0, 0, -ONE, 0, W, H, 0x4000_0000]),
            (3, 270, [0, -ONE, 0, ONE, 0, 0, 0, W, 0x4000_0000]),
            (-1, 270, [0, -ONE, 0, ONE, 0, 0, 0, W, 0x4000_0000]),
        ];
        for fixture in FIXTURES {
            for (turns, degrees, matrix) in expected {
                let file = copy_of(fixture, &format!("turn{turns}"));
                let rotation = rotate(&file, turns).expect("rotate");
                assert_eq!(rotation.degrees(), degrees, "{fixture} {turns}");
                assert_eq!(read(&file).map(Rotation::degrees), Ok(degrees));
                // The audio track keeps its identity matrix.
                assert_eq!(
                    all_matrices(&file),
                    vec![matrix, IDENTITY],
                    "{fixture} {turns}"
                );
            }
        }
    }

    #[test]
    fn four_turns_either_way_give_back_the_same_bytes() {
        for fixture in FIXTURES {
            let file = copy_of(fixture, "four");
            let original = std::fs::read(&file).expect("read");
            for step in 1..=4 {
                rotate(&file, 1).expect("rotate");
                assert_eq!(read(&file).map(Rotation::degrees), Ok((step % 4) * 90));
            }
            assert_eq!(std::fs::read(&file).expect("read"), original, "{fixture}");
            for _ in 0..4 {
                rotate(&file, -1).expect("rotate");
            }
            assert_eq!(std::fs::read(&file).expect("read"), original, "{fixture}");
            rotate(&file, 1).expect("rotate");
            rotate(&file, -1).expect("rotate");
            assert_eq!(std::fs::read(&file).expect("read"), original, "{fixture}");
        }
    }

    #[test]
    fn only_the_matrix_bytes_change_and_the_times_are_kept() {
        for fixture in FIXTURES {
            let file = copy_of(fixture, "only-matrix");
            let before = std::fs::read(&file).expect("read");
            let modified = std::fs::metadata(&file).and_then(|m| m.modified());
            rotate(&file, 1).expect("rotate");
            let after = std::fs::read(&file).expect("read");
            assert_eq!(after.len(), before.len());
            let changed: Vec<usize> = (0..before.len())
                .filter(|&i| before[i] != after[i])
                .collect();
            let first = *changed.first().expect("something changed");
            assert!(
                changed.iter().all(|&i| i < first + MATRIX_LEN),
                "{fixture}: bytes outside one matrix changed: {changed:?}"
            );
            assert_eq!(
                std::fs::metadata(&file).and_then(|m| m.modified()).ok(),
                modified.ok()
            );
        }
    }

    #[test]
    fn a_turned_matrix_without_translation_is_read_and_turned() {
        // ffmpeg's `-display_rotation` style: turned, no translation.
        for fixture in FIXTURES {
            let file = copy_of(fixture, "zero-translation");
            let at = {
                let mut f = File::open(&file).expect("open");
                video_tracks(&mut f, &file).expect("tracks")[0].matrix_at
            };
            let mut bytes = std::fs::read(&file).expect("read");
            let ffmpeg: [i32; 9] = [0, -ONE, 0, ONE, 0, 0, 0, 0, 0x4000_0000];
            for (i, v) in ffmpeg.iter().enumerate() {
                let o = at as usize + i * 4;
                bytes[o..o + 4].copy_from_slice(&v.to_be_bytes());
            }
            std::fs::write(&file, &bytes).expect("write");
            assert_eq!(read(&file).map(Rotation::degrees), Ok(270));

            rotate(&file, 1).expect("rotate");
            assert_eq!(all_matrices(&file)[0], IDENTITY);
            rotate(&file, 3).expect("rotate");
            assert_eq!(
                all_matrices(&file)[0],
                [0, -ONE, 0, ONE, 0, 0, 0, W, 0x4000_0000],
                "{fixture}: back at 270° with the phones' translation"
            );
        }
    }

    #[test]
    fn a_mirror_is_kept_through_turns() {
        let file = copy_of("wide.mp4", "mirror");
        let at = {
            let mut f = File::open(&file).expect("open");
            video_tracks(&mut f, &file).expect("tracks")[0].matrix_at as usize
        };
        let mut bytes = std::fs::read(&file).expect("read");
        bytes[at..at + 4].copy_from_slice(&(-ONE).to_be_bytes());
        bytes[at + 24..at + 28].copy_from_slice(&W.to_be_bytes());
        std::fs::write(&file, &bytes).expect("write");
        let mirrored = read(&file).expect("read");
        assert!(mirrored.mirrored());
        assert_eq!(mirrored.degrees(), 0);

        let turned = rotate(&file, 1).expect("rotate");
        assert!(turned.mirrored());
        assert_eq!(turned.degrees(), 90);
        rotate(&file, 3).expect("rotate");
        assert_eq!(std::fs::read(&file).expect("read"), bytes);
    }

    #[test]
    fn a_scaled_or_displaced_matrix_is_left_alone() {
        let file = copy_of("wide.mp4", "scaled");
        let at = {
            let mut f = File::open(&file).expect("open");
            video_tracks(&mut f, &file).expect("tracks")[0].matrix_at as usize
        };
        let original = std::fs::read(&file).expect("read");

        let mut scaled = original.clone();
        scaled[at..at + 4].copy_from_slice(&(2 * ONE).to_be_bytes());
        std::fs::write(&file, &scaled).expect("write");
        assert_eq!(rotate(&file, 1), Err(RotationError::UnusualMatrix));
        assert_eq!(std::fs::read(&file).expect("read"), scaled);

        let mut displaced = original;
        displaced[at + 24..at + 28].copy_from_slice(&(5 * ONE).to_be_bytes());
        std::fs::write(&file, &displaced).expect("write");
        assert_eq!(read(&file), Err(RotationError::UnusualMatrix));
    }

    #[test]
    fn formats_without_a_flag_and_damaged_files_cannot_turn() {
        let dir = copy_of("wide.mp4", "formats");
        let mkv = dir.with_file_name("clip.mkv");
        std::fs::write(&mkv, b"\x1aE\xdf\xa3 not really matroska").expect("write");
        assert_eq!(read(&mkv), Err(RotationError::CannotRotate));
        assert_eq!(rotate(&mkv, 1), Err(RotationError::CannotRotate));

        let zeros = dir.with_file_name("zeros.mov");
        std::fs::write(&zeros, [0u8; 512]).expect("write");
        assert_eq!(rotate(&zeros, 1), Err(RotationError::Damaged));

        // A movie with no video: only `ftyp` and an empty `moov`.
        let empty = dir.with_file_name("empty.mp4");
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&16u32.to_be_bytes());
        bytes.extend_from_slice(b"ftypisom\0\0\0\0");
        bytes.extend_from_slice(&8u32.to_be_bytes());
        bytes.extend_from_slice(b"moov");
        std::fs::write(&empty, bytes).expect("write");
        assert_eq!(read(&empty), Err(RotationError::NoVideoTrack));
    }

    #[test]
    fn a_read_only_file_fails_and_stays_as_it_was() {
        let file = copy_of("wide.mov", "read-only");
        let before = std::fs::read(&file).expect("read");
        let mut permissions = std::fs::metadata(&file).expect("meta").permissions();
        permissions.set_readonly(true);
        std::fs::set_permissions(&file, permissions.clone()).expect("read-only");
        let result = rotate(&file, 1);
        #[allow(clippy::permissions_set_readonly_false)]
        permissions.set_readonly(false);
        std::fs::set_permissions(&file, permissions).expect("writable again");
        assert!(matches!(result, Err(RotationError::Io(_))), "{result:?}");
        assert_eq!(std::fs::read(&file).expect("read"), before);
    }

    /// A version-1 track header (64-bit times) puts the matrix 12 bytes later.
    #[test]
    fn a_version_1_track_header_is_found() {
        fn boxed(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
            let mut b = ((payload.len() + 8) as u32).to_be_bytes().to_vec();
            b.extend_from_slice(kind);
            b.extend_from_slice(payload);
            b
        }
        let mut tkhd = vec![1, 0, 0, 7]; // version 1, flags
        tkhd.extend_from_slice(&[0; 8 + 8 + 4 + 4 + 8]); // times, id, reserved, duration
        tkhd.extend_from_slice(&[0; 8 + 2 + 2 + 2 + 2]); // reserved, layer, group, volume
        for v in IDENTITY {
            tkhd.extend_from_slice(&v.to_be_bytes());
        }
        tkhd.extend_from_slice(&W.to_be_bytes());
        tkhd.extend_from_slice(&H.to_be_bytes());
        let mut hdlr = vec![0; 8];
        hdlr.extend_from_slice(b"vide");
        hdlr.extend_from_slice(&[0; 13]);
        let mdia = boxed(b"mdia", &boxed(b"hdlr", &hdlr));
        let trak = boxed(b"trak", &[boxed(b"tkhd", &tkhd), mdia].concat());
        let movie = [boxed(b"ftyp", b"isom\0\0\0\0"), boxed(b"moov", &trak)].concat();

        let file = copy_of("wide.mp4", "v1").with_file_name("v1.mp4");
        std::fs::write(&file, &movie).expect("write");
        assert_eq!(rotate(&file, 1).map(Rotation::degrees), Ok(90));
        let bytes = std::fs::read(&file).expect("read");
        // ftyp, then the moov, trak and tkhd headers, then 52 bytes of the version-1 header.
        let at = 16 + 8 + 8 + 8 + 52;
        assert_eq!(&bytes[at + 4..at + 8], &ONE.to_be_bytes());
        assert_eq!(&bytes[at + 12..at + 16], &(-ONE).to_be_bytes());
        assert_eq!(&bytes[at + 24..at + 28], &H.to_be_bytes());
        rotate(&file, -1).expect("rotate back");
        assert_eq!(std::fs::read(&file).expect("read"), movie);
    }

    #[test]
    fn rotations_turn_and_come_back() {
        assert_eq!(Rotation::UPRIGHT.turned(-1).degrees(), 270);
        assert_eq!(Rotation::clockwise(90).turned(3), Rotation::UPRIGHT);
        assert_eq!(Rotation::clockwise(270).turns_to_upright(), 1);
        assert_eq!(Rotation::UPRIGHT.turns_to_upright(), 0);
        for r in [0, 90, 180, 270] {
            let rotation = Rotation::clockwise(r);
            assert_eq!(Rotation::of(rotation.matrix()), Some(rotation));
        }
    }
}
