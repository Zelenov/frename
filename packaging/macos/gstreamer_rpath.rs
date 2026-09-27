// Included by the build scripts of frename and frename-core: frename-core's test binary links
// GStreamer too (the workspace builds clipscribe with its `frames` feature for both).

/// The official GStreamer macOS framework names its libraries `@rpath/<name>.dylib` and adds no
/// rpath to what links them: without one, `cargo run` and `cargo test` cannot load GStreamer.
/// Point the rpath at the framework's `lib`; `packaging/macos/build-app.sh` replaces it with the
/// app's own `Contents/Frameworks`. Nothing to do for a Homebrew GStreamer (absolute names).
/// No `rerun-if` line: that would stop cargo re-running the build script on every change, which
/// the Windows version resource (`APP_VERSION`) relies on.
fn macos_gstreamer_rpath() {
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
