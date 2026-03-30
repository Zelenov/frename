# GStreamer Setup

frename uses GStreamer for video playback. Install it once — the app will work automatically.

**Download: https://gstreamer.freedesktop.org/download/**

---

## Windows

frename runtime requires a normal system GStreamer installation and `PATH` entry.
The vendored `vendor\gstreamer\minimal_msvc_x86_64` bundle is for CI/build only.

1. Open **https://gstreamer.freedesktop.org/download/**
2. Under **MSVC**, download the **Runtime** installer for your architecture:
   - 64-bit: `gstreamer-1.0-msvc-x86_64-VERSION.msi`
   - 32-bit: `gstreamer-1.0-msvc-x86-VERSION.msi`
3. Run the installer. Default location: `C:\gstreamer\1.0\msvc_x86_64\`
4. Add GStreamer to PATH:
   - Open **Start → Edit the system environment variables → Environment Variables**
   - Under System variables, edit **Path**, add: `C:\gstreamer\1.0\msvc_x86_64\bin`
5. **Restart** the app or terminal.

---

## Linux (Ubuntu / Debian)

```bash
sudo apt-get install \
  gstreamer1.0-plugins-base \
  gstreamer1.0-plugins-good \
  gstreamer1.0-plugins-bad \
  gstreamer1.0-plugins-ugly \
  gstreamer1.0-libav
```

Other distributions: use `dnf`, `pacman`, or `zypper` with equivalent package names.

---

## macOS

1. Open **https://gstreamer.freedesktop.org/download/**
2. Under **macOS**, download:
   - `gstreamer-1.0-VERSION-universal.pkg`
3. Run the installer.
4. Add to `~/.zshrc` or `~/.bash_profile`:
   ```bash
   export PATH="/Library/Frameworks/GStreamer.framework/Versions/1.0/bin:$PATH"
   ```
5. Reload: `source ~/.zshrc`

---

## Verify

```bash
gst-inspect-1.0 --version
```

---

## Building from source

In addition to the runtime, you need the **development package**:

- **Windows:** download and install `gstreamer-1.0-devel-msvc-x86_64-VERSION.msi` from the same page
- **Linux:** `sudo apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev`
- **macOS:** download `gstreamer-1.0-devel-VERSION-universal.pkg` from the same page

Then: `cargo build`
