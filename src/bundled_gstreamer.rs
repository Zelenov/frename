//! The Windows packages carry their own GStreamer next to `frename.exe`: its DLLs,
//! `gst-plugin-scanner.exe`, and the plugins in `lib\gstreamer-1.0\`. The exe's folder comes first
//! in the Windows DLL search order, so a GStreamer installed on the system is not loaded; its
//! plugin environment variables would still be read, so they are replaced before `gst::init`.
//! A developer build has no bundled plugins and uses the system GStreamer, as before.

use std::path::{Path, PathBuf};

/// A plugin every bundle has; its presence marks the exe's folder as a bundle.
const MARKER_PLUGIN: &str = "gstcoreelements.dll";

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
    pub set: Vec<(&'static str, PathBuf)>,
}

/// The changes for an exe in `exe_dir`, with the plugin registry kept in `data_dir` (the exe's
/// folder may be replaced by an update); `None` when `exe_dir` has no bundled GStreamer.
pub fn bundled_gstreamer_environment(exe_dir: &Path, data_dir: &Path) -> Option<GstEnvironment> {
    let plugins = exe_dir.join("lib").join("gstreamer-1.0");
    if !plugins.join(MARKER_PLUGIN).is_file() {
        return None;
    }
    Some(GstEnvironment {
        remove: REMOVED.to_vec(),
        set: vec![
            ("GST_PLUGIN_SYSTEM_PATH_1_0", plugins),
            (
                "GST_PLUGIN_SCANNER_1_0",
                exe_dir.join("gst-plugin-scanner.exe"),
            ),
            ("GST_REGISTRY_1_0", data_dir.join("gst-registry-x86_64.bin")),
        ],
    })
}

/// Point GStreamer at the bundle next to the running exe, if there is one. Call before any
/// thread starts (changing the environment is not thread-safe) and before `gst::init`.
/// Returns whether a bundle was found.
#[cfg(windows)]
pub fn configure_bundled_gstreamer(data_dir: &Path) -> bool {
    let Some(exe_dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    else {
        return false;
    };
    let Some(environment) = bundled_gstreamer_environment(&exe_dir, data_dir) else {
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
    fn a_bundle_gets_its_own_plugins_scanner_and_a_registry_in_the_data_folder() {
        let exe_dir = temp_folder("bundle");
        let plugins = exe_dir.join("lib").join("gstreamer-1.0");
        std::fs::create_dir_all(&plugins).expect("plugins");
        std::fs::write(plugins.join("gstcoreelements.dll"), b"MZ").expect("plugin");
        let data_dir = PathBuf::from("data");

        let environment = bundled_gstreamer_environment(&exe_dir, &data_dir).expect("a bundle");

        assert_eq!(environment.remove, REMOVED.to_vec());
        assert_eq!(
            environment.set,
            vec![
                ("GST_PLUGIN_SYSTEM_PATH_1_0", plugins),
                (
                    "GST_PLUGIN_SCANNER_1_0",
                    exe_dir.join("gst-plugin-scanner.exe")
                ),
                ("GST_REGISTRY_1_0", data_dir.join("gst-registry-x86_64.bin")),
            ]
        );
        let _ = std::fs::remove_dir_all(&exe_dir);
    }

    #[test]
    fn without_bundled_plugins_nothing_changes() {
        let exe_dir = temp_folder("no-bundle");
        assert_eq!(
            bundled_gstreamer_environment(&exe_dir, Path::new("data")),
            None
        );
        let _ = std::fs::remove_dir_all(&exe_dir);
    }
}
