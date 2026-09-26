//! `frename --self-test <video or folder>...`: checks that this build's GStreamer can play video,
//! without opening a window. Every video given, and every video in each folder given, must be
//! opened the way the player opens it and deliver one decoded frame, with the sound sent nowhere
//! (CI runners have no audio device).

use std::path::{Path, PathBuf};
use std::time::Duration;

use frename_core::FileKind;

use crate::features::media_viewer::video::check_decodes;

/// How long an opened clip may take to deliver its first frame.
const FRAME_TIMEOUT: Duration = Duration::from_secs(20);

/// Run the self-test on `paths` (videos, or folders whose videos are all tested) and return the
/// process exit code: 0 when every video decoded a frame, 1 otherwise, including when there are
/// no videos at all. The report goes to the log, which prints on the terminal (CI reads it there)
/// and to the log file (a release build on Windows has no console).
pub fn run(paths: &[PathBuf]) -> i32 {
    let mut videos = Vec::new();
    for path in paths {
        if path.is_dir() {
            match videos_in(path) {
                Ok(found) => videos.extend(found),
                Err(e) => {
                    log::error!("self-test: cannot read {}: {e}", path.display());
                    return 1;
                }
            }
        } else {
            videos.push(path.clone());
        }
    }
    if videos.is_empty() {
        log::error!("self-test: no videos to test");
        return 1;
    }
    let mut failed = 0;
    for video in &videos {
        match decode_first_frame(video) {
            Ok(()) => log::info!("self-test: ok     {}", video.display()),
            Err(e) => {
                failed += 1;
                log::error!("self-test: FAILED {}: {e}", video.display());
            }
        }
    }
    log::info!(
        "self-test: {} of {} videos decoded",
        videos.len() - failed,
        videos.len()
    );
    i32::from(failed > 0)
}

/// The videos in `folder`, sorted so the report reads the same on every run.
fn videos_in(folder: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut videos: Vec<PathBuf> = std::fs::read_dir(folder)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_file() && is_video(path))
        .collect();
    videos.sort();
    Ok(videos)
}

fn is_video(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| FileKind::from_extension(ext) == FileKind::Video)
}

fn decode_first_frame(path: &Path) -> Result<(), String> {
    let absolute = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
    let uri = url::Url::from_file_path(&absolute)
        .map_err(|()| format!("not a file path: {}", absolute.display()))?;
    check_decodes(&uri, FRAME_TIMEOUT)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
    }

    /// The clips listed in `tests/self-test-clips.txt`, which CI's self-test plays too.
    fn ci_clips() -> Vec<PathBuf> {
        std::fs::read_to_string(repo().join("tests/self-test-clips.txt"))
            .expect("tests/self-test-clips.txt")
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| repo().join(line))
            .collect()
    }

    /// A fresh, empty folder of its own in the temp dir.
    fn temp_folder(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("frename-self-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn a_folder_yields_only_its_videos() {
        let videos = videos_in(&repo().join("tests/folder")).expect("tests/folder");
        assert!(videos.iter().all(|v| is_video(v)));
        for clip in ci_clips() {
            assert!(videos.contains(&clip), "{} not found", clip.display());
        }
    }

    #[test]
    fn a_folder_without_videos_fails() {
        let empty = temp_folder("empty");
        assert_eq!(run(std::slice::from_ref(&empty)), 1);
        let _ = std::fs::remove_dir_all(&empty);
    }

    #[test]
    fn a_missing_file_fails() {
        assert_eq!(run(&[repo().join("tests/folder/no such clip.mp4")]), 1);
    }

    /// Linux only, like the decoding tests below: the Windows CI job builds against the vendored
    /// build-only GStreamer, which has no H.264 or Matroska decoders
    /// (`docs/research/windows-installer.md`); Windows decoding is tested once the app bundles
    /// its own GStreamer (#10).
    #[cfg(target_os = "linux")]
    #[test]
    fn the_ci_clips_decode_a_frame() {
        let clips = ci_clips();
        assert!(!clips.is_empty());
        assert_eq!(run(&clips), 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_folder_of_good_clips_passes() {
        let folder = temp_folder("folder");
        let clip = &ci_clips()[0];
        std::fs::copy(clip, folder.join(clip.file_name().expect("name"))).expect("copy");
        assert_eq!(run(std::slice::from_ref(&folder)), 0);
        let _ = std::fs::remove_dir_all(&folder);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn a_file_that_is_not_a_video_fails() {
        let folder = temp_folder("garbage");
        let fake = folder.join("fake.mp4");
        std::fs::write(&fake, b"definitely not a movie").expect("write");
        assert_eq!(run(&[fake]), 1);
        let _ = std::fs::remove_dir_all(&folder);
    }
}
