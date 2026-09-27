fn main() {
    #[cfg(windows)]
    windows_resources();
    macos_gstreamer_rpath();
}

/// The official GStreamer macOS framework names its libraries `@rpath/<name>.dylib` and adds no
/// rpath to what links them: without one, `cargo run` and `cargo test` cannot load GStreamer.
/// Point the rpath at the framework's `lib`; `packaging/macos/build-app.sh` replaces it with the
/// app's own `Contents/Frameworks`. Nothing to do for a Homebrew GStreamer (absolute names).
fn macos_gstreamer_rpath() {
    println!("cargo:rerun-if-env-changed=GST_FRAMEWORK");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("macos") {
        return;
    }
    let framework = std::env::var("GST_FRAMEWORK")
        .unwrap_or_else(|_| "/Library/Frameworks/GStreamer.framework/Versions/1.0".to_string());
    let lib = std::path::Path::new(&framework).join("lib");
    if lib.is_dir() {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib.display());
    }
}

/// Embeds the icon and version into the Windows executable. `winres` is a Windows-only
/// build dependency, so this only compiles when building on Windows.
#[cfg(windows)]
fn windows_resources() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("frename-icon.ico");

    if let Ok(app_version) = std::env::var("APP_VERSION") {
        res.set("FileVersion", &app_version);
        res.set("ProductVersion", &app_version);
    }

    res.compile().unwrap();
}
