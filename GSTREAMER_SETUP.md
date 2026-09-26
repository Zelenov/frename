# GStreamer for building frename

frename plays video with GStreamer. **Users do not need this page:** the Windows installer, the
portable Windows zip and the Linux AppImage carry their own GStreamer. It is for building frename
from source.

---

## Windows

Install the official GStreamer package (runtime and development files) the way CI does:

```powershell
packaging/windows/install-gstreamer.ps1 -Version 1.28.7 `
    -Sha256 032fc6062b8539838fc8da22589cb9b24c5d820baa7f8cc160af9ea08395badf `
    -Dir C:\gstreamer
```

It installs in portable mode: no environment variables, no registry keys, so it does not disturb
another GStreamer on the machine. Then point the build at it, for example in PowerShell:

```powershell
$env:PKG_CONFIG = "C:\gstreamer\bin\pkg-config.exe"
$env:PKG_CONFIG_PATH = "C:\gstreamer\lib\pkgconfig"
$env:PATH = "C:\gstreamer\bin;$env:PATH"
cargo build
```

The version and checksum CI uses are in `.github/workflows/ci.yml` (`GST_VERSION`, `GST_SHA256`).
A GStreamer installed with the package's own installer works too, as long as it includes the
development files and the `libav` plugin (the tests play H.264 and HEVC clips).

To build the Windows packages as a release does (needs Visual Studio's C++ tools and the .NET SDK):

```powershell
cargo build --release
packaging/windows/bundle.ps1 -GstRoot C:\gstreamer -Exe target\release\frename.exe -Out dist\frename
dotnet tool install vpk --version 1.2.158 --tool-path C:\vpk
C:\vpk\vpk.exe pack -u frename -v 0.0.1 -p dist\frename -e frename.exe -o releases --packTitle frename --icon frename-icon.ico
```

`dist\frename\frename.exe --self-test tests\media` checks that the bundle plays every test clip.

---

## Linux (Ubuntu / Debian)

```bash
sudo apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
  libgstreamer-plugins-bad1.0-dev gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
  gstreamer1.0-libav
```

Other distributions: use `dnf`, `pacman`, or `zypper` with equivalent package names.

---

## macOS

Download the runtime and development packages (`gstreamer-1.0-VERSION-universal.pkg` and
`gstreamer-1.0-devel-VERSION-universal.pkg`) from https://gstreamer.freedesktop.org/download/ and
add `/Library/Frameworks/GStreamer.framework/Versions/1.0/bin` to `PATH`.
