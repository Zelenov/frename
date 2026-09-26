#!/usr/bin/env bash
# Check a frename AppImage on a clean desktop: run inside a bare ubuntu container as root.
#
#   packaging/linux/test-appimage.sh <path to AppImage>
#
# Installs only the libraries every Linux desktop has and linuxdeploy therefore leaves out of
# AppImages (X11, fonts, ALSA, GL, ...), makes sure no GStreamer is on the system, then runs
# the AppImage's --self-test on the clips in tests/self-test-clips.txt: they must play with the
# GStreamer inside the AppImage alone.
set -euo pipefail

appimage="$(readlink -f "$1")"
repo="$(cd "$(dirname "$0")/../.." && pwd)"

apt-get update -qq
DEBIAN_FRONTEND=noninteractive apt-get install -y -qq --no-install-recommends \
  libx11-6 libxcb1 libx11-xcb1 libxcb-dri3-0 libfontconfig1 libfreetype6 libharfbuzz0b \
  libfribidi0 libdrm2 libgbm1 libwayland-client0 libasound2t64 libgl1 libegl1 libxkbcommon0 \
  > /dev/null

# Not `grep -q`: it stops reading early, dpkg dies of SIGPIPE, and pipefail turns a match into a miss.
if dpkg -l | grep -i gstreamer > /dev/null; then
  echo "A system GStreamer is installed; this test needs a machine without one." >&2
  exit 1
fi

mapfile -t clips < <(sed 's/^[[:space:]]*//' "$repo/tests/self-test-clips.txt" | grep -v '^#' | grep .)
cd "$repo"
chmod +x "$appimage"
# No FUSE in containers: the runtime extracts itself instead.
APPIMAGE_EXTRACT_AND_RUN=1 "$appimage" --self-test "${clips[@]}"
