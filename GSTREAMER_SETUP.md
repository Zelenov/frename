# GStreamer Setup Instructions for Windows

## Quick Setup

1. **Download GStreamer installers** from: https://gstreamer.freedesktop.org/download/

   Download BOTH files for MSVC x86_64:
   - Runtime installer: `gstreamer-1.0-msvc-x86_64-{VERSION}.msi`
   - Development installer: `gstreamer-1.0-devel-msvc-x86_64-{VERSION}.msi`

2. **Install Runtime**: Run the runtime MSI, use default location (usually `C:\gstreamer\1.0\msvc_x86_64\`)

3. **Install Development**: Run the development MSI to the SAME location

4. **Add to PATH**:
   - Search Windows for "environment variables"
   - Click "Edit the system environment variables"
   - Click "Environment Variables" button
   - Under "System variables", find "Path", click "Edit"
   - Click "New" and add: `C:\gstreamer\1.0\msvc_x86_64\bin`
   - Click OK on all dialogs

5. **Restart your terminal/IDE** - Environment variables won't be picked up until you restart

6. **Test installation**: 
   ```powershell
   gst-inspect-1.0 --version
   ```

   Should show GStreamer version info.

7. **Try building again**:
   ```powershell
   cargo clean
   cargo build
   ```

## If You Still Get Errors

If pkg-config is still missing, you may need to install it:
- Download from: https://sourceforge.net/projects/pkgconfiglite/
- Extract `pkg-config.exe` to a folder in your PATH (or to `C:\gstreamer\1.0\msvc_x86_64\bin\`)

## Alternative: Use Windows Video Player API Instead

If GStreamer is too problematic, I can implement a simpler solution using Windows Media Foundation or a different approach.
