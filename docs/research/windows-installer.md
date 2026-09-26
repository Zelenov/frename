# Installer, auto-update and bundled GStreamer

Research notes for issue #10 (and #11/#15). Not yet implemented or verified in CI.

## Problems found in the current setup

- The vendored `gstreamer-minimal-msvc-x86_64.zip` (GStreamer 1.26.10) has only 24 plugins: no
  `matroska`, `avi`, `libav`, `dav1d`, `vorbis`, `debugutilsbad`. It cannot play MKV/WebM/AVI, and
  the VFR retry pipeline's `capssetter` (from `debugutilsbad`) is missing. Its `bin/` carries
  ~124 MB of DLLs the app does not need.
- `frename.db` (`crates/frename-core/src/db/app_database.rs`) and the log (`src/main.rs`) are
  written next to the exe via `current_exe()`. Velopack replaces the whole `current\` folder on each
  update, so both must move to `%LocalAppData%\frename\` first.

## Pipeline in use

`src/features/media_viewer/video/state.rs`: `playbin uri=… text-sink="appsink"
video-sink="videoscale ! videoconvert ! [capssetter] ! appsink caps=NV12"`, audio via playbin's
default `autoaudiosink`. `gst::init` in `src/main.rs`.

## Recommended architecture

- **Velopack** (`velopack` crate 1.2.158, `vpk` 1.2.158). `Setup.exe` = one-click per-user install to
  `%LocalAppData%\frename\` (launcher stub, `Update.exe`, app in `current\`); `--silent` for CI.
  Deltas are built when the previous release is in `-o`.
- **GStreamer inside `current\`**: all GStreamer/GLib/FFmpeg DLLs and `gst-plugin-scanner.exe` next to
  `frename.exe`, plugins in `lib\gstreamer-1.0\`, plus `msvcp140`/`vcruntime140(_1)` (or
  `vpk --framework vcredist143-x64`). The exe's own folder wins the DLL search over PATH, and
  GStreamer finds plugins relative to `gstreamer-1.0-0.dll`.
- Still override env before `gst::init`, so a system GStreamer never interferes:

```rust
fn main() {
    velopack::VelopackApp::build().run(); // must be first
    configure_bundled_gstreamer();        // before any thread / gst::init
}

fn configure_bundled_gstreamer() {
    let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())) else { return };
    let plugins = dir.join("lib").join("gstreamer-1.0");
    if !plugins.join("gstcoreelements.dll").exists() { return; } // dev build -> system gst
    for k in ["GST_PLUGIN_PATH", "GST_PLUGIN_PATH_1_0", "GST_PLUGIN_SYSTEM_PATH", "GST_PLUGIN_SCANNER", "GST_REGISTRY"] {
        std::env::remove_var(k);
    }
    std::env::set_var("GST_PLUGIN_SYSTEM_PATH_1_0", &plugins);
    std::env::set_var("GST_PLUGIN_SCANNER_1_0", dir.join("gst-plugin-scanner.exe"));
    std::env::set_var("GST_REGISTRY_1_0", data_dir().join("gst-registry-x86_64.bin"));
}
```

- **Updates** (blocking; run in `spawn_blocking`):

```rust
let src = velopack::sources::GithubSource::new("https://github.com/Zelenov/frename", None, false);
let um = velopack::UpdateManager::new(src, None, None)?; // Err(NotInstalled) under cargo run
if let velopack::UpdateCheck::UpdateAvailable(u) = um.check_for_updates()? {
    um.download_updates(&u, None)?;
    um.apply_updates_and_restart(&*u)?;
}
```

  Draft releases are invisible to the updater: branch builds need `--channel` or prereleases.
  Versions must be 3-part semver (drop the `.0` the workflow appends today → use `0.67.0`).

## Plugin set (sizes from GStreamer 1.28.7 MSVC x64)

| Group | Plugins |
|---|---|
| Core/base | coreelements, playback, typefindfunctions, app, videoconvertscale, audioconvert, audioresample, volume, pbtypes |
| Audio out | autodetect, wasapi2, wasapi, directsound |
| Demux/parse | isomp4, matroska, avi, audioparsers, videoparsersbad |
| capssetter | debugutilsbad (needs gstnet, gio) |
| Decoders | libav (+ avcodec-61, avformat-61, avfilter-10, avutil-59), d3d11 (HW H.264/HEVC/VP9/AV1), dav1d, opus |
| Optional | vorbis, mpg123, jpeg, mediafoundation; skip d3d12 |

Libraries: glib-2.0, gobject, gmodule, gio, intl-8, ffi-7, pcre2-8, z-1, bz2, orc-0.4, gstreamer-1.0,
gstbase, gstapp, gstvideo, gstaudio, gstpbutils, gsttag, gstriff, gstrtp, gstnet, gstcodecparsers,
gstcodecs, gstd3d11-1.0, gstd3dshader, gstdxva. Verify with `dumpbin /dependents` and a CI check
that every plugin loads.

Estimated size: ~65–75 MB installed, ~28–35 MB Setup.exe, a few MB per delta update.

## CI outline

- Get GStreamer: pinned 1.28.x MSVC runtime MSI, `msiexec /a gst.msi /qn TARGETDIR=C:\gstx`
  (extract without installing), copy the allowlist. Replaces the vendored zip.
- `vpk pack -u frename -v X.Y.Z -p dist\frename -e frename.exe -o Releases --packTitle frename
  --icon frename-icon.ico --framework vcredist143-x64`; `vpk upload github … --publish`.
- Smoke test on a fresh `windows-2022`: fail if a system GStreamer exists; `Setup.exe --silent`;
  run `%LOCALAPPDATA%\frename\current\frename.exe --self-test <fixtures>` and check the exit code.
  `--self-test`: check the registry has playbin, qtdemux, matroskademux, avidemux, capssetter,
  avdec_h264/h265/aac, dav1ddec, opusdec; play tiny fixtures (H.264/AAC mp4, HEVC mov, VP9/Opus webm,
  AV1 mkv, MPEG-4/MP3 avi) with `audio-sink=fakesink` until a sample or EOS. Runners have no GPU or
  audio device: this covers the software decode path.
- Linux: AppDir via `linuxdeploy` + `linuxdeploy-plugin-gstreamer`, `vpk pack` → AppImage; smoke
  test in a clean `ubuntu:24.04` container. Build on ubuntu-22.04 for the glibc floor.
- macOS (macos-14): trimmed `GStreamer.framework` in `Contents/Frameworks`,
  `install_name_tool -add_rpath @executable_path/../Frameworks`, ad-hoc `codesign --force --deep -s -`,
  `vpk pack` with `.icns`. Whether unsigned `vpk pack` works on macOS needs testing.

## Licensing and risks

- GStreamer core/base/good/bad LGPL-2.1+, official FFmpeg build LGPL: fine with dynamic linking;
  ship license texts and a source link. No x264/x265/ugly/GPL. dav1d, opus BSD.
- Unsigned Windows builds show SmartScreen "unknown publisher"; unsigned macOS needs "Open Anyway"
  or `xattr -dr com.apple.quarantine`.
- Set GStreamer env vars before any thread starts.

## Sources

- https://github.com/velopack/velopack.docs
- https://docs.velopack.io/packaging/overview
- https://crates.io/crates/velopack
- https://www.nuget.org/packages/vpk
- https://github.com/GStreamer/gstreamer/blob/main/subprojects/gstreamer/gst/gstregistry.c
- https://github.com/GStreamer/gstreamer/blob/main/subprojects/gstreamer/gst/gstpluginloader-win32.c
- https://pypi.org/project/gstreamer-libs/
- https://gstreamer.freedesktop.org/download/
- https://github.com/linuxdeploy/linuxdeploy-plugin-gstreamer
