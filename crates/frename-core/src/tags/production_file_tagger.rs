//! Production FileTagger: renames files on disk.

use std::path::{Path, PathBuf};

use super::file_snapshot::FileSnapshot;
use super::screenshot::Screenshot;
use super::file_tagger_backend::FileTaggerBackend;

pub struct ProductionFileTagger;

// ---------------------------------------------------------------------------
// Sidecar path helpers (private)
// ---------------------------------------------------------------------------

fn screenshot_path(file_path: &Path, position_ms: u64) -> PathBuf {
    let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let time_str = Screenshot::new(position_ms).format_time();
    let sidecar_name = format!("{}.snap.{}.jpg", file_name, time_str);
    file_path
        .parent()
        .unwrap_or(Path::new("."))
        .join(sidecar_name)
}

fn is_screenshot_sidecar(name: &str) -> bool {
    if let Some(idx) = name.find(".snap.") {
        let rest = &name[idx + 6..];
        if let Some(time_str) = rest.strip_suffix(".jpg") {
            return Screenshot::parse_time(time_str).is_some();
        }
    }
    false
}

fn load_screenshot_positions(file_path: &Path) -> Vec<Screenshot> {
    let Some(file_name) = file_path.file_name().and_then(|n| n.to_str()) else {
        return Vec::new();
    };
    let prefix = format!("{}.snap.", file_name);
    let parent = file_path.parent().unwrap_or(Path::new("."));
    let mut screenshots: Vec<Screenshot> = std::fs::read_dir(parent)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let rest = name.strip_prefix(&*prefix)?;
            let time_str = rest.strip_suffix(".jpg")?;
            Screenshot::parse_time(time_str).map(Screenshot::new)
        })
        .collect();
    screenshots.sort();
    screenshots
}

// ---------------------------------------------------------------------------
// FileTaggerBackend impl
// ---------------------------------------------------------------------------

impl FileTaggerBackend for ProductionFileTagger {
    fn parse(&self, path: &Path) -> FileSnapshot {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let mut snapshot = FileSnapshot::parse(name);
        snapshot.set_comment(crate::comment::load_comment(path));
        snapshot.set_screenshots(load_screenshot_positions(path));
        snapshot
    }

    fn save(&self, snapshot: &FileSnapshot, path: &Path) -> PathBuf {
        let new_file_name = snapshot.file_name();
        let new_path = path
            .parent()
            .map(|p| p.join(&new_file_name))
            .unwrap_or_else(|| PathBuf::from(&new_file_name));

        if new_path != path {
            crate::comment::rename_comment_file(path, &new_path);
            for s in snapshot.screenshots() {
                let old_shot = screenshot_path(path, s.position_ms);
                let new_shot = screenshot_path(&new_path, s.position_ms);
                if old_shot.exists() {
                    let _ = std::fs::rename(&old_shot, &new_shot);
                }
            }
            if let Err(e) = std::fs::rename(path, &new_path) {
                log::error!("ProductionFileTagger: rename {:?} → {:?} failed: {}", path, new_path, e);
                return path.to_path_buf();
            }
            log::info!("Renamed on disk: {:?} → {:?}", path, new_path);
        }
        crate::comment::save_comment(&new_path, snapshot.comment());
        new_path
    }

    fn is_sidecar_file(&self, path: &Path) -> bool {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        crate::comment::is_comment_file(path) || is_screenshot_sidecar(name)
    }

    fn save_screenshot(&self, file_path: &Path, position_ms: u64, image_data: &[u8]) {
        let path = screenshot_path(file_path, position_ms);
        log::info!("save_screenshot: writing {} bytes to {:?}", image_data.len(), path);
        match std::fs::write(&path, image_data) {
            Ok(()) => log::info!("save_screenshot: ok"),
            Err(e) => log::error!("save_screenshot: failed {:?}: {}", path, e),
        }
    }

    #[allow(dead_code)]
    fn load_screenshot_image(&self, file_path: &Path, position_ms: u64) -> Option<Vec<u8>> {
        std::fs::read(screenshot_path(file_path, position_ms)).ok()
    }
}
