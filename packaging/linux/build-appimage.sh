#!/usr/bin/env bash
# Build a frename AppImage from a release binary, with the system's GStreamer plugins inside.
#
#   packaging/linux/build-appimage.sh <version> <path to frename binary> <output folder>
#
# Needs: the GStreamer plugin packages the app should carry (as in .github/workflows), patchelf,
# file, imagemagick. linuxdeploy and its GStreamer plugin are downloaded at pinned versions and
# checked against their SHA-256.
set -euo pipefail

version="$1"
binary="$(readlink -f "$2")"
out="$(readlink -f "$3")"

here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

linuxdeploy_url="https://github.com/linuxdeploy/linuxdeploy/releases/download/1-alpha-20251107-1/linuxdeploy-x86_64.AppImage"
linuxdeploy_sha256="c20cd71e3a4e3b80c3483cef793cda3f4e990aca14014d23c544ca3ce1270b4d"
plugin_url="https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gstreamer/2a2e67491c32995a3f279ad0ecbe77abd512b42a/linuxdeploy-plugin-gstreamer.sh"
plugin_sha256="c107b49d84edbffc6ab226ed1007e0626a4f7aa2c3a36b7782bef62351d49e94"

fetch() {
  curl -fsSL -o "$2" "$1"
  echo "$3  $2" | sha256sum -c -
  chmod +x "$2"
}

cd "$work"
fetch "$linuxdeploy_url" linuxdeploy-x86_64.AppImage "$linuxdeploy_sha256"
# linuxdeploy finds plugins next to itself by their file name.
fetch "$plugin_url" linuxdeploy-plugin-gstreamer.sh "$plugin_sha256"

convert "$repo/frename-icon.ico[0]" -resize 256x256 frename.png

# No FUSE on CI runners and in containers: run the tools extracted.
export APPIMAGE_EXTRACT_AND_RUN=1
export LINUXDEPLOY_OUTPUT_VERSION="$version"
./linuxdeploy-x86_64.AppImage \
  --appdir AppDir \
  --executable "$binary" \
  --desktop-file "$here/frename.desktop" \
  --icon-file frename.png \
  --plugin gstreamer \
  --output appimage

mkdir -p "$out"
mv "frename-$version-x86_64.AppImage" "$out/"
echo "Built $out/frename-$version-x86_64.AppImage"
