#!/usr/bin/env bash
# Install the official GStreamer macOS framework (runtime and development files) at a pinned
# version, checked against pinned SHA-256 sums. Downloads are kept in <download dir> (CI caches it).
#
#   packaging/macos/install-gstreamer.sh <version> <runtime sha256> <devel sha256> <download dir>
#
# Installs to /Library/Frameworks/GStreamer.framework (needs sudo). To build against it, put
# its bin/ first on PATH (its own pkg-config) and its lib/pkgconfig on PKG_CONFIG_PATH.
set -euo pipefail

version="$1"
runtime_sha256="$2"
devel_sha256="$3"
download="$4"
base="https://gstreamer.freedesktop.org/data/pkg/osx/$version"
mkdir -p "$download"

install() {
  local name="$1" expected="$2" actual
  if [ ! -f "$download/$name" ]; then
    curl -fsSL --retry 3 -o "$download/$name.part" "$base/$name"
    mv "$download/$name.part" "$download/$name"
  fi
  actual="$(shasum -a 256 "$download/$name" | cut -d' ' -f1)"
  if [ "$actual" != "$expected" ]; then
    echo "::error::$name has SHA-256 $actual, expected $expected" >&2
    rm -f "$download/$name"
    exit 1
  fi
  sudo installer -pkg "$download/$name" -target /
}

install "gstreamer-1.0-$version-universal.pkg" "$runtime_sha256"
install "gstreamer-1.0-devel-$version-universal.pkg" "$devel_sha256"
/Library/Frameworks/GStreamer.framework/Versions/1.0/bin/gst-inspect-1.0 --version
