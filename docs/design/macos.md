# macOS build (issue #15)

frename for Macs with Apple silicon: an ad-hoc-signed `frename.app` with GStreamer inside, in every
release as `frename-macos-arm64-v<version>.zip`. No paid Apple signing (owner's decision in #15);
the Mac App Store is a separate issue (#85).

The only Mac that checks it is CI's `macos-14` runner (Apple silicon). So CI proves what it can:
the app is built the way a release builds it, then on a **fresh runner with no GStreamer** it
must link only macOS and itself, decode a frame of every test clip, and start and stay up.

## Shape: the same as Windows and Linux

| | Windows (#10) | Linux (#11, #25) | macOS (this) |
|---|---|---|---|
| GStreamer source | official MSVC package, pinned + SHA-256 | Ubuntu packages | official macOS framework, pinned + SHA-256 |
| Bundler | `packaging/windows/bundle.ps1` | linuxdeploy + GStreamer plugin | `packaging/macos/build-app.sh` |
| Plugin list | `gstreamer-plugins.txt` | linuxdeploy's default set | `gstreamer-plugins.txt` (same set, macOS audio/hardware decoders) |
| Library discovery | walk `dumpbin /imports` | linuxdeploy | walk `otool -L` |
| Bundle check | `check-bundle.ps1` | clean `ubuntu:24.04` container | `check-bundle.sh` + fresh runner |
| Clean-machine test | `test-package.ps1` on fresh runner | `test-appimage.sh` in container | `test-app.sh` on fresh runner |
| App finds its GStreamer | `bundled_gstreamer.rs` (`WINDOWS`) | AppImage hooks | `bundled_gstreamer.rs` (`MACOS`) |

## The official framework

- Files: `gstreamer-1.0-<v>-universal.pkg` and `gstreamer-1.0-devel-<v>-universal.pkg` from
  `gstreamer.freedesktop.org/data/pkg/osx/<v>/`, "macOS Universal (X86_64 & ARM64)", oldest
  macOS 10.13 ([download page source](https://github.com/GStreamer/www/blob/main/src/htdocs/download/download.md)).
  Version 1.28.7, the same as Windows (`GST_VERSION` in the workflows).
- Installs to `/Library/Frameworks/GStreamer.framework/Versions/1.0/`; `.pc` files in
  `lib/pkgconfig`, and its own `pkg-config` in `bin/`, which gstreamer-rs recommends putting
  first on `PATH` ([on-mac-osx.md](https://github.com/GStreamer/gstreamer/blob/main/subprojects/gst-docs/markdown/installing/on-mac-osx.md),
  [gstreamer-rs README](https://github.com/sdroege/gstreamer-rs/blob/main/README.md)).
- SHA-256 pins: the runtime's value is the one three independent projects pin
  ([1](https://github.com/philberthfz/homebrew-wine/blob/main/Casks/gstreamer-runtime.rb),
  [2](https://github.com/lieranderl/moviestracker-app/blob/main/macos/install-gstreamer.sh),
  [3](https://github.com/tx3stn/atolla/blob/main/.scripts/install-gstreamer.sh)); the devel
  value comes from one ([cask](https://github.com/philberthfz/homebrew-wine/blob/main/Casks/gstreamer-development.rb)).
  The agent's sandbox cannot reach freedesktop.org; CI downloads the files and fails on any
  mismatch, printing the real value.
- Relocatable since 1.22: the libraries "use LC_RPATH entries"
  ([1.22 NEWS](https://github.com/GStreamer/gstreamer/blob/1.22.0/subprojects/gstreamer/NEWS)).
  cerbero's `osxrelocator` replaces the install prefix with `@rpath`, so a library is named
  `@rpath/lib/libgstreamer-1.0.0.dylib` ([osxrelocator.py](https://github.com/GStreamer/cerbero/blob/main/cerbero/tools/osxrelocator.py)).
- Plugins are `lib/gstreamer-1.0/libgst<name>.so` (cerbero's Darwin module extension is `.so`,
  [filesprovider.py](https://github.com/GStreamer/cerbero/blob/main/cerbero/build/filesprovider.py)).
- GStreamer's own "Deploying on macOS" page predates relocation (copy the framework, run
  `osxrelocator`, set `GST_PLUGIN_SYSTEM_PATH` and `GST_PLUGIN_SCANNER`)
  ([mac-osx.md](https://github.com/GStreamer/gstreamer/blob/main/subprojects/gst-docs/markdown/deploying/mac-osx.md)).
  With `@rpath` names only rpaths need changing, not every library.

## Bundle layout

```
frename.app/Contents/
  Info.plist                       packaging/macos/Info.plist, version filled in
  MacOS/frename                    rpath @executable_path/../Frameworks
  Frameworks/lib/*.dylib           only the libraries something links, keeping @rpath/lib/<name>
  PlugIns/gstreamer/libgst*.so     rpath @loader_path/../../Frameworks
  Resources/frename.icns           from packaging/macos/frename-512.png (sips + iconutil)
  Resources/licenses/              the framework's licenses + README with source links
```

- Icon: `packaging/macos/frename-512.png` is the 512 px image of `frename-icon.ico` (macOS has no
  ImageMagick to read the `.ico`); `sips` + `iconutil` make every `.icns` size from it.
- Libraries go to `Contents/Frameworks` and plug-ins to `Contents/PlugIns`, the places Apple's
  code signing expects nested code. The plugin folder has no dot in its name (`gstreamer`, not
  `gstreamer-1.0`): `codesign` takes a folder with an extension for a bundle.
- `build-app.sh` walks `otool -L` from the binary and every plugin, copying each `@rpath/…`
  library from the framework; anything that is neither in macOS (`/usr/lib`, `/System`) nor in the
  framework fails the build. Absolute rpaths (the binary gets the framework's at link time) are
  deleted, so a Mac with the framework installed cannot hide a missing library.
  `check-bundle.sh` re-checks the finished app and `codesign --verify --strict --deep`.
- **No `gst-plugin-scanner`.** GStreamer normally scans plugins in a child process; with
  `GST_REGISTRY_FORK=no` it scans in-process ([running.md](https://github.com/GStreamer/gstreamer/blob/main/subprojects/gstreamer/docs/gst/running.md),
  `gstregistry.c`). One executable fewer to sign, ship and keep in the bundle layout.
  The cost, a crashing plugin taking the app down during the first scan, applies to a fixed,
  tested plugin set only.
- Environment set before `gst::init` (`src/bundled_gstreamer.rs`, as on Windows): the plugin
  variables a Homebrew or framework GStreamer may have set are removed;
  `GST_PLUGIN_SYSTEM_PATH_1_0=Contents/PlugIns/gstreamer`, `GST_REGISTRY_FORK=no`,
  `GST_REGISTRY_1_0=<data folder>/gst-registry-arm64.bin`. Without our variable GStreamer would
  look in `<dir of libgstreamer>/gstreamer-1.0` ([gstregistry.c](https://github.com/GStreamer/gstreamer/blob/main/subprojects/gstreamer/gst/gstregistry.c)).
- Plugins: the Windows set with `osxaudio` (sound out) and `applemedia` (VideoToolbox hardware
  decoding, `vtdec`) instead of `wasapi*`/`directsound`/`d3d11`. `libav` (FFmpeg), `dav1d`
  (gst-plugins-rs), `opus`, `vorbis`, `mpg123`, `jpeg` as on Windows
  ([cerbero recipes](https://github.com/GStreamer/cerbero/tree/main/recipes)). The self-test
  checks the same required elements as on Windows.

## Data folder

A signed bundle must not change after signing, and the app may sit in a folder the user cannot
write, so a `frename.app` keeps its database, log and GStreamer registry in
`~/Library/Application Support/frename` (`frename_core::app_dir`, detected from the executable
being `<name>.app/Contents/MacOS/<exe>`).

## Signing and Gatekeeper

- Apple silicon runs only signed code; "a simple ad-hoc signature is sufficient", and after
  `install_name_tool` "you might need to manually call codesign"
  ([Big Sur 11.0.1 release notes](https://developer.apple.com/documentation/macos-release-notes/macos-big-sur-11_0_1-universal-apps-release-notes)).
  `sign-app.sh` signs every library and plugin, then the app, with `-` (ad hoc).
- Ad-hoc signed apps "cannot pass through Gatekeeper" (same source). A browser download carries the
  quarantine flag; since macOS 15 Control-click → Open no longer overrides Gatekeeper
  ([Apple](https://developer.apple.com/news/?id=saqachfa)); the way left is System Settings →
  Privacy & Security → Open Anyway, or removing the flag:
  `xattr -dr com.apple.quarantine /Applications/frename.app`
  ([hacks.guide](https://wiki.hacks.guide/wiki/Open_unsigned_applications_on_macOS_Sequoia_and_newer)).
  The README gives the command (as sonisub's does), with `-r` because Archive Utility flags every
  file inside.
- App Translocation runs a quarantined app from a random read-only path; removing the flag or
  moving the app with Finder ends it ([Eclectic Light](https://eclecticlight.co/2023/05/09/what-causes-app-translocation/)).
  With the data folder outside the bundle nothing breaks even then.

## CI

- `ci.yml` → `ci-macos` (`macos-14`): install the framework, clippy, the full test suite (the
  decoding tests run on macOS too), release build, `build-app.sh`, `sign-app.sh`,
  `check-bundle.sh`, zip with `ditto` (an artifact would lose the executable bits and signature).
- `ci-macos-app-test` on a fresh `macos-14` (`test-app.sh`): refuses to run if any GStreamer is
  present; unzips; `check-bundle.sh`; `--self-test` on `tests/self-test-clips.txt`; again with
  `GST_PLUGIN_PATH`/`GST_PLUGIN_SYSTEM_PATH`/`GST_PLUGIN_SCANNER`/`GST_REGISTRY` pointing
  elsewhere; then `open`s the app on `tests/folder`, checks it still runs after 20 s and that its
  log says it uses the bundled GStreamer, and takes a `screencapture` screenshot (artifact).
- `release.yml` → `build-macos` + `test-macos` (same scripts); the release waits for them and
  publishes `frename-macos-arm64-v<version>.zip`.
- The ruleset's required checks stay `ci-linux` and `ci-windows`; the owner may add `ci-macos`
  and `ci-macos-app-test`.

## What CI cannot prove

- Sound: runners have no audio device; the self-test sends sound nowhere, as on Windows/Linux.
- The first-open experience of a downloaded app (quarantine, Gatekeeper dialogs): artifacts
  downloaded by CI are not quarantined. The owner checks it once (PR checklist).
- Intel Macs: not built (the issue says Apple silicon).

## Decisions made without the owner

- **Official framework, not Homebrew.** Pinned and hashed like the Windows package, relocatable,
  LGPL-only plugin choice; Homebrew bottles are built for the runner's macOS and pull in far more.
- **Zip, not DMG.** `ditto` keeps signatures and permissions; one download, drag to Applications.
- **arm64 only, macOS 12 minimum** (`LSMinimumSystemVersion`). Below what the parts need
  (the GStreamer framework: macOS 10.13, [download page source](https://github.com/GStreamer/www/blob/main/src/htdocs/download/download.md);
  Rust's `aarch64-apple-darwin`: macOS 11, [platform support](https://doc.rust-lang.org/rustc/platform-support/apple-darwin.html));
  12 is the oldest arm64-only minimum the App Store accepts
  ([Apple forums](https://developer.apple.com/forums/thread/810409)), which keeps one binary for
  both. CI runs macOS 14 only.
- **Bundle id `io.github.zelenov.frename`**.
- **No self-update on macOS.** Velopack is not used for the Mac app (as for the AppImage);
  Settings → Updates says updates work in the installed version. A new release is a new zip.
- **Workflow steps**: build, sign, check and zip are four script calls in each workflow rather
  than one wrapper, because the Store build (#85) signs the same app differently.
- **Shortcuts:** frename checks `Modifiers::command()`, which iced maps to `⌘` on macOS, so the
  README's `Ctrl` shortcuts are `⌘` on a Mac; the README says so.
- **Opening from Finder** ("Open With", dropping on the Dock icon) needs Apple Events handling that
  iced does not expose; not in this issue. Drag onto the window and 📂 work.
- `ffmpeg` for subtitles of mkv/m2ts/avi: an app started from Finder has a minimal `PATH`
  (`/usr/bin:/bin:/usr/sbin:/sbin`), so a Homebrew ffmpeg is not found; the built-in decoder
  covers mp4/mov. Left as is.
