#!/usr/bin/env bash
# Build frename.app from a release binary, with GStreamer from the official macOS framework inside.
#
#   packaging/macos/build-app.sh <version> <path to frename binary> <output .app path>
#
#   frename.app/Contents/Info.plist
#   frename.app/Contents/MacOS/frename
#   frename.app/Contents/Frameworks/lib/*.dylib      GStreamer, GLib, FFmpeg, ... that are used
#   frename.app/Contents/PlugIns/gstreamer/*.so     the plugins in gstreamer-plugins.txt
#   frename.app/Contents/Resources/frename.icns
#   frename.app/Contents/Resources/licenses/        licenses of the bundled parts, with sources
#
# Libraries are not hand-listed: every library the binary or a plugin links, directly or not, is
# copied from the framework. The framework's libraries name each other `@rpath/lib/<name>`, so
# they keep that layout under Contents/Frameworks and the binary and the plugins get an rpath to
# it. There is no gst-plugin-scanner: the app scans its plugins in-process (GST_REGISTRY_FORK=no,
# src/bundled_gstreamer.rs). The result is not signed: packaging/macos/sign-app.sh does that.
#
# Needs: the framework (packaging/macos/install-gstreamer.sh), Xcode's command line tools (otool,
# install_name_tool), sips and iconutil.
set -euo pipefail

version="$1"
binary="$2"
app="$3"
bundle_id="io.github.zelenov.frename"
framework="${GST_FRAMEWORK:-/Library/Frameworks/GStreamer.framework/Versions/1.0}"
here="$(cd "$(dirname "$0")" && pwd)"

[ -x "$binary" ] || { echo "No binary at $binary" >&2; exit 1; }
[ -d "$framework/lib/gstreamer-1.0" ] || { echo "No GStreamer framework at $framework" >&2; exit 1; }

rm -rf "$app"
contents="$app/Contents"
mkdir -p "$contents/MacOS" "$contents/Frameworks" "$contents/PlugIns/gstreamer" \
  "$contents/Resources"
cp "$binary" "$contents/MacOS/frename"
sed -e "s/@VERSION@/$version/g" -e "s/@BUNDLE_ID@/$bundle_id/g" "$here/Info.plist" \
  > "$contents/Info.plist"
plutil -lint "$contents/Info.plist"

# The icon: every size of an .icns from frename-512.png, the 512 px image of frename-icon.ico
# (`convert 'frename-icon.ico[5]' PNG32:frename-512.png`; macOS has no ImageMagick). 1024 px for
# 512@2x is scaled up.
iconset="$(mktemp -d)/frename.iconset"
mkdir -p "$iconset"
for size in 16 32 128 256 512; do
  sips -z "$size" "$size" "$here/frename-512.png" --out "$iconset/icon_${size}x${size}.png" > /dev/null
  double=$((size * 2))
  sips -z "$double" "$double" "$here/frename-512.png" --out "$iconset/icon_${size}x${size}@2x.png" > /dev/null
done
iconutil -c icns "$iconset" -o "$contents/Resources/frename.icns"

plugins=0
while read -r name; do
  plugin="$framework/lib/gstreamer-1.0/libgst$name.so"
  if [ ! -f "$plugin" ]; then
    echo "Plugin not in this GStreamer: $plugin. It has:" >&2
    ls "$framework/lib/gstreamer-1.0" >&2
    exit 1
  fi
  cp "$plugin" "$contents/PlugIns/gstreamer/"
  plugins=$((plugins + 1))
done < <(sed 's/#.*//; s/[[:space:]]//g' "$here/gstreamer-plugins.txt" | grep .)

# The libraries a Mach-O file links (not its own name, which `otool -L` lists first for a dylib).
linked() {
  local own
  own="$(otool -D "$1" | sed -n 2p)"
  otool -L "$1" | tail -n +2 | awk '{print $1}' | grep -vxF -- "${own:-/nonexistent}" || true
}

# Walk the links from the binary and the plugins, copying each library the framework provides.
# (macOS's own bash is 3.2: no associative arrays, so the copied ones are the files themselves.)
queue=("$contents/MacOS/frename" "$contents"/PlugIns/gstreamer/*.so)
while [ "${#queue[@]}" -gt 0 ]; do
  file="${queue[0]}"
  queue=("${queue[@]:1}")
  while read -r dep; do
    [ -n "$dep" ] || continue
    case "$dep" in
      /usr/lib/* | /System/*) continue ;;  # part of macOS
      @rpath/*)
        relative="${dep#@rpath/}"
        [ -e "$contents/Frameworks/$relative" ] && continue
        source="$framework/$relative"
        [ -f "$source" ] || { echo "$file links $dep, which is not in $framework" >&2; exit 1; }
        mkdir -p "$(dirname "$contents/Frameworks/$relative")"
        cp "$source" "$contents/Frameworks/$relative"
        chmod u+w "$contents/Frameworks/$relative"
        queue+=("$contents/Frameworks/$relative")
        ;;
      *)
        echo "$file links $dep, which is neither in macOS nor in the framework" >&2
        exit 1
        ;;
    esac
  done < <(linked "$file")
done

# rpaths: drop those pointing outside the bundle (the framework's absolute path, which the
# binary gets at link time), then point the binary and the plugins at Contents/Frameworks.
rpaths() {
  otool -l "$1" | awk '/cmd LC_RPATH/ { getline; getline; print $2 }'
}
while IFS= read -r -d '' file; do
  while read -r rpath; do
    case "$rpath" in
      /*) install_name_tool -delete_rpath "$rpath" "$file" 2> /dev/null ;;
    esac
  done < <(rpaths "$file")
done < <(find "$contents/MacOS" "$contents/Frameworks" "$contents/PlugIns" -type f -print0)
install_name_tool -add_rpath "@executable_path/../Frameworks" "$contents/MacOS/frename" 2> /dev/null
for plugin in "$contents"/PlugIns/gstreamer/*.so; do
  install_name_tool -add_rpath "@loader_path/../../Frameworks" "$plugin" 2> /dev/null
done

# Licenses: the framework keeps one folder per component, as the Windows package does.
licenses="$contents/Resources/licenses"
if [ -d "$framework/share/licenses" ]; then
  cp -R "$framework/share/licenses" "$licenses"
else
  mkdir -p "$licenses"
fi
gst_version="$(sed -n 's/^Version: //p' "$framework/lib/pkgconfig/gstreamer-1.0.pc")"
cat > "$licenses/README.txt" << TEXT
frename bundles GStreamer $gst_version (the official macOS framework) with GLib and FFmpeg,
linked dynamically. GStreamer and FFmpeg are LGPL. Their sources:
  https://gstreamer.freedesktop.org/src/
  https://gitlab.freedesktop.org/gstreamer/cerbero (how the official build is made)
  https://ffmpeg.org/download.html
Each folder here holds the licenses of one component of that build.
TEXT

libraries="$(find "$contents/Frameworks" -type f | wc -l | tr -d ' ')"
size="$(du -sm "$app" | cut -f1)"
echo "Built $app: $plugins plugins, $libraries libraries, $size MB"
