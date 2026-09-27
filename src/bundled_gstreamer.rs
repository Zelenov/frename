//! The Windows packages and the macOS app carry their own GStreamer. On Windows it sits next to
//! `frename.exe`: its DLLs, `gst-plugin-scanner.exe`, and the plugins in `lib\gstreamer-1.0\`;
//! the exe's folder comes first in the DLL search order, so a GStreamer installed on the system
//! is not loaded. In `frename.app` the libraries are in `Contents/Frameworks/`, found through the
//! executable's rpath, and the plugins in `Contents/PlugIns/gstreamer/`; there is no scanner
//! process (a sandboxed app could not start one), GStreamer scans the plugins in-process.
//! A system GStreamer's plugin environment variables would still be read, so they are replaced
//! before `gst::init`. A developer build has no bundled plugins and uses the system GStreamer.

// Everything but the Windows and macOS caller is also built elsewhere, so its tests run on every
// OS.
#![cfg_attr(not(any(windows, target_os = "macos")), allow(dead_code))]

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Where a bundle keeps GStreamer, relative to the folder of the executable.
#[derive(Debug, Clone, Copy)]
pub struct BundleLayout {
    /// The plugin folder.
    plugins: &'static str,
    /// A plugin every bundle has; its presence marks the folder as a bundle.
    marker_plugin: &'static str,
    /// The plugin scanner; `None` scans in-process.
    scanner: Option<&'static str>,
    /// The registry file's name, in the data folder.
    registry: &'static str,
}

/// The Windows packages (`packaging/windows/bundle.ps1`).
pub const WINDOWS: BundleLayout = BundleLayout {
    plugins: "lib/gstreamer-1.0",
    marker_plugin: "gstcoreelements.dll",
    scanner: Some("gst-plugin-scanner.exe"),
    registry: "gst-registry-x86_64.bin",
};

/// `frename.app` (`packaging/macos/build-app.sh`); the executable is in `Contents/MacOS/`.
pub const MACOS: BundleLayout = BundleLayout {
    plugins: "../PlugIns/gstreamer",
    marker_plugin: "libgstcoreelements.so",
    scanner: None,
    registry: "gst-registry-arm64.bin",
};

/// Variables that point GStreamer at another installation's plugins or registry.
const REMOVED: [&str; 5] = [
    "GST_PLUGIN_PATH",
    "GST_PLUGIN_PATH_1_0",
    "GST_PLUGIN_SYSTEM_PATH",
    "GST_PLUGIN_SCANNER",
    "GST_REGISTRY",
];

/// The environment changes that make GStreamer use only the bundle.
#[derive(Debug, PartialEq, Eq)]
pub struct GstEnvironment {
    pub remove: Vec<&'static str>,
    pub set: Vec<(&'static str, OsString)>,
}

/// The changes for an executable in `exe_dir` laid out as `layout`, with the plugin registry kept
/// in `data_dir` (the executable's folder may be replaced by an update, or be read-only); `None`
/// when `exe_dir` has no bundled GStreamer.
pub fn bundled_gstreamer_environment(
    layout: BundleLayout,
    exe_dir: &Path,
    data_dir: &Path,
) -> Option<GstEnvironment> {
    let plugins = normalize(&exe_dir.join(layout.plugins));
    if !plugins.join(layout.marker_plugin).is_file() {
        return None;
    }
    let mut set = vec![("GST_PLUGIN_SYSTEM_PATH_1_0", plugins.into_os_string())];
    match layout.scanner {
        Some(scanner) => set.push((
            "GST_PLUGIN_SCANNER_1_0",
            exe_dir.join(scanner).into_os_string(),
        )),
        None => set.push(("GST_REGISTRY_FORK", OsString::from("no"))),
    }
    set.push((
        "GST_REGISTRY_1_0",
        data_dir.join(layout.registry).into_os_string(),
    ));
    Some(GstEnvironment {
        remove: REMOVED.to_vec(),
        set,
    })
}

/// `path` without `..` steps (lexically: the bundle has no symlinks to follow), so the logged
/// plugin folder reads as a real place.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other),
        }
    }
    out
}

/// Point GStreamer at the bundle around the running executable, if there is one. Call before any
/// thread starts (changing the environment is not thread-safe) and before `gst::init`.
/// Returns whether a bundle was found.
#[cfg(any(windows, target_os = "macos"))]
pub fn configure_bundled_gstreamer(data_dir: &Path) -> bool {
    #[cfg(windows)]
    let layout = WINDOWS;
    #[cfg(target_os = "macos")]
    let layout = MACOS;
    let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    else {
        return false;
    };
    let Some(environment) = bundled_gstreamer_environment(layout, &exe_dir, data_dir) else {
        return false;
    };
    for name in environment.remove {
        std::env::remove_var(name);
    }
    for (name, value) in environment.set {
        std::env::set_var(name, value);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_folder(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("frename-bundled-gst-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[test]
    fn a_windows_bundle_gets_its_own_plugins_scanner_and_a_registry_in_the_data_folder() {
        let exe_dir = temp_folder("bundle");
        let plugins = exe_dir.join("lib/gstreamer-1.0");
        std::fs::create_dir_all(&plugins).expect("plugins");
        std::fs::write(plugins.join("gstcoreelements.dll"), b"MZ").expect("plugin");
        let data_dir = PathBuf::from("data");

        let environment =
            bundled_gstreamer_environment(WINDOWS, &exe_dir, &data_dir).expect("a bundle");

        assert_eq!(environment.remove, REMOVED.to_vec());
        assert_eq!(
            environment.set,
            vec![
                ("GST_PLUGIN_SYSTEM_PATH_1_0", normalize(&plugins).into()),
                (
                    "GST_PLUGIN_SCANNER_1_0",
                    exe_dir.join("gst-plugin-scanner.exe").into()
                ),
                (
                    "GST_REGISTRY_1_0",
                    data_dir.join("gst-registry-x86_64.bin").into()
                ),
            ]
        );
        let _ = std::fs::remove_dir_all(&exe_dir);
    }

    #[test]
    fn a_macos_app_gets_its_plugins_from_plugins_and_scans_them_in_process() {
        let app = temp_folder("app").join("frename.app/Contents");
        let exe_dir = app.join("MacOS");
        let plugins = app.join("PlugIns/gstreamer");
        std::fs::create_dir_all(&exe_dir).expect("MacOS");
        std::fs::create_dir_all(&plugins).expect("plugins");
        std::fs::write(plugins.join("libgstcoreelements.so"), b"\xcf\xfa").expect("plugin");
        let data_dir = PathBuf::from("data");

        let environment =
            bundled_gstreamer_environment(MACOS, &exe_dir, &data_dir).expect("a bundle");

        assert_eq!(environment.remove, REMOVED.to_vec());
        assert_eq!(
            environment.set,
            vec![
                ("GST_PLUGIN_SYSTEM_PATH_1_0", normalize(&plugins).into()),
                ("GST_REGISTRY_FORK", "no".into()),
                (
                    "GST_REGISTRY_1_0",
                    data_dir.join("gst-registry-arm64.bin").into()
                ),
            ]
        );
        let _ = std::fs::remove_dir_all(app.parent().and_then(Path::parent).expect("temp"));
    }

    #[test]
    fn without_bundled_plugins_nothing_changes() {
        let exe_dir = temp_folder("no-bundle");
        for layout in [WINDOWS, MACOS] {
            assert_eq!(
                bundled_gstreamer_environment(layout, &exe_dir, Path::new("data")),
                None
            );
        }
        let _ = std::fs::remove_dir_all(&exe_dir);
    }

    #[test]
    fn parent_steps_are_resolved() {
        assert_eq!(
            normalize(Path::new(
                "/a/frename.app/Contents/MacOS/../PlugIns/gstreamer"
            )),
            PathBuf::from("/a/frename.app/Contents/PlugIns/gstreamer")
        );
    }
}
