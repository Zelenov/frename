//! `frename --self-test <video or folder>...`: checks that this build's GStreamer can play video,
//! without opening a window. Every video given, and every video in each folder given, must decode
//! one frame through the same pipeline the player builds, with the audio sent nowhere (CI runners
//! have no audio device).

use std::path::{Path, PathBuf};
use std::time::Duration;

use frename_core::FileKind;
use gst::prelude::*;
use gstreamer as gst;

use crate::features::media_viewer::video::{pipeline_description, pipeline_sinks};

/// How long one clip may take to produce its first frame.
const FRAME_TIMEOUT: Duration = Duration::from_secs(20);

/// Run the self-test on `paths` (videos, or folders whose videos are all tested) and return the
/// process exit code: 0 when every video decoded a frame, 1 otherwise, including when there are
/// no videos at all.
pub fn run(paths: &[PathBuf]) -> i32 {
    let mut videos = Vec::new();
    for path in paths {
        if path.is_dir() {
            match videos_in(path) {
                Ok(found) => videos.extend(found),
                Err(e) => {
                    report(&format!("self-test: cannot read {}: {e}", path.display()));
                    return 1;
                }
            }
        } else {
            videos.push(path.clone());
        }
    }
    if videos.is_empty() {
        report("self-test: no videos to test");
        return 1;
    }
    let mut failed = 0;
    for video in &videos {
        match decode_first_frame(video) {
            Ok(()) => report(&format!("self-test: ok     {}", video.display())),
            Err(e) => {
                failed += 1;
                report(&format!("self-test: FAILED {}: {e}", video.display()));
            }
        }
    }
    report(&format!(
        "self-test: {} of {} videos decoded",
        videos.len() - failed,
        videos.len()
    ));
    i32::from(failed > 0)
}

/// The log goes to the terminal (CI reads it there) and to the log file (a release build on
/// Windows has no console).
fn report(line: &str) {
    log::info!("{line}");
}

/// The videos in `folder`, sorted so the report reads the same on every run.
fn videos_in(folder: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut videos: Vec<PathBuf> = std::fs::read_dir(folder)?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| FileKind::from_extension(ext) == FileKind::Video)
        })
        .collect();
    videos.sort();
    Ok(videos)
}

/// Build the player's pipeline for `path` and wait for the first decoded frame.
fn decode_first_frame(path: &Path) -> Result<(), String> {
    let absolute = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
    let uri = url::Url::from_file_path(&absolute)
        .map_err(|()| format!("not a file path: {}", absolute.display()))?;
    let description = format!("{} audio-sink=fakesink", pipeline_description(&uri, false));
    let pipeline = gst::parse::launch(&description)
        .map_err(|e| e.to_string())?
        .downcast::<gst::Pipeline>()
        .map_err(|_| "not a pipeline".to_string())?;
    let result = pull_frame(&pipeline);
    let _ = pipeline.set_state(gst::State::Null);
    result
}

fn pull_frame(pipeline: &gst::Pipeline) -> Result<(), String> {
    let (sink, _) = pipeline_sinks(pipeline).map_err(|e| e.to_string())?;
    pipeline
        .set_state(gst::State::Playing)
        .map_err(|e| format!("cannot start: {e}{}", bus_error(pipeline)))?;
    let timeout = gst::ClockTime::from_nseconds(FRAME_TIMEOUT.as_nanos() as u64);
    match sink.try_pull_preroll(timeout) {
        Some(_) => Ok(()),
        None => Err(format!(
            "no frame within {FRAME_TIMEOUT:?}{}",
            bus_error(pipeline)
        )),
    }
}

/// The first error on the pipeline's bus, as `": <message>"`, or nothing.
fn bus_error(pipeline: &gst::Pipeline) -> String {
    let Some(bus) = pipeline.bus() else {
        return String::new();
    };
    while let Some(message) = bus.pop() {
        if let gst::MessageView::Error(err) = message.view() {
            return format!(": {}", err.error());
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_folder() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/folder")
    }

    #[test]
    fn only_videos_are_tested() {
        let videos = videos_in(&test_folder()).expect("tests/folder");
        let names: Vec<String> = videos
            .iter()
            .filter_map(|p| p.extension()?.to_str().map(str::to_lowercase))
            .collect();
        assert_eq!(names, ["avi", "webm", "mov", "mp4"]);
    }

    #[test]
    fn a_folder_without_videos_fails() {
        let empty = std::env::temp_dir().join(format!("frename-self-test-{}", std::process::id()));
        std::fs::create_dir_all(&empty).expect("temp dir");
        assert_eq!(run(&[empty]), 1);
    }

    #[test]
    fn a_missing_file_fails() {
        assert_eq!(run(&[test_folder().join("no such clip.mp4")]), 1);
    }

    /// The clips CI's self-test plays (see `.github/workflows/ci.yml`). Linux only: the Windows CI
    /// job builds against the vendored build-only GStreamer, which has no H.264 or Matroska
    /// decoders (`docs/research/windows-installer.md`); Windows decoding is tested once the app
    /// bundles its own GStreamer (#10).
    #[cfg(target_os = "linux")]
    #[test]
    fn the_ci_clips_decode_a_frame() {
        gst::init().expect("GStreamer");
        let clips = [
            "file_example_MP4_480_1_5MG.mp4",
            "file_example_MOV_480_700kB.mov",
            "Short.Travel.Health.Views.file_example_WEBM_480_900KB.webm",
        ]
        .map(|name| test_folder().join(name));
        assert_eq!(run(&clips), 0);
    }
}
