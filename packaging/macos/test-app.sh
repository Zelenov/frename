#!/usr/bin/env bash
# Check frename.app on a clean Mac (a fresh macos-14 runner: no GStreamer, never had one).
#
#   packaging/macos/test-app.sh <frename-macos-arm64-*.zip> [screenshot.png]
#
# 1. refuses to run where a GStreamer is installed, so a pass means "works on a clean Mac";
# 2. unzips the app as a user's Finder would (ditto), checks that it links only macOS and itself
#    and that its signature is valid;
# 3. runs its self-test on the clips in tests/self-test-clips.txt, first as it is, then with
#    GStreamer variables pointing elsewhere, as a Homebrew GStreamer's would: they must be ignored;
# 4. starts the app the way Finder does (`open`) on tests/folder, checks it is still running after
#    20 s and the log says it uses its own GStreamer, and takes a screenshot of the screen.
# Run from the repository root. Exits non-zero on the first failure.
set -euo pipefail

zip="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
screenshot="${2:-}"
repo="$(cd "$(dirname "$0")/../.." && pwd)"
data="$HOME/Library/Application Support/frename"

echo "== 1. No GStreamer on this machine"
if [ -e /Library/Frameworks/GStreamer.framework ] || command -v gst-launch-1.0 > /dev/null \
  || { command -v brew > /dev/null && brew list --versions gstreamer > /dev/null 2>&1; }; then
  echo "This runner has a GStreamer; the test needs a machine without one." >&2
  exit 1
fi

echo "== 2. Unzip and check the bundle"
work="$(mktemp -d)"
ditto -x -k "$zip" "$work"
app="$work/frename.app"
[ -x "$app/Contents/MacOS/frename" ] || { echo "No frename.app in $zip" >&2; exit 1; }
"$repo/packaging/macos/check-bundle.sh" "$app"
du -sh "$app"

clips=()
while IFS= read -r clip; do
  clips+=("$repo/$clip")
done < <(sed 's/#.*//; s/^[[:space:]]*//; s/[[:space:]]*$//' "$repo/tests/self-test-clips.txt" | grep .)

echo "== 3. Self-test"
"$app/Contents/MacOS/frename" --self-test "${clips[@]}"
echo "== 3b. Self-test with a foreign GStreamer's variables set"
GST_PLUGIN_PATH=/opt/homebrew/lib/gstreamer-1.0 GST_PLUGIN_SYSTEM_PATH=/nonexistent \
  GST_PLUGIN_SCANNER=/nonexistent/gst-plugin-scanner GST_REGISTRY=/nonexistent/registry.bin \
  "$app/Contents/MacOS/frename" --self-test "${clips[@]}"

echo "== 4. Start the app"
open -n "$app" --args "$repo/tests/folder"
sleep 20
if ! pgrep -f "$app/Contents/MacOS/frename" > /dev/null; then
  echo "frename is not running 20 s after start. Its log:" >&2
  cat "$data/frename_debug.log" >&2 || true
  exit 1
fi
[ -n "$screenshot" ] && { screencapture -x "$screenshot" || echo "(no screenshot: $?)"; }
pkill -f "$app/Contents/MacOS/frename" || true
cat "$data/frename_debug.log"
grep -q "using the GStreamer bundled with frename" "$data/frename_debug.log" \
  || { echo "The app did not use its bundled GStreamer" >&2; exit 1; }

echo "All app tests passed."
