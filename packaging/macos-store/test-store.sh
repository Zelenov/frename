#!/usr/bin/env bash
# Check the Mac App Store variant on a clean Mac (a fresh macos-14 runner: no GStreamer): it
# installs from its .pkg, runs sandboxed, and still works in its sandbox.
#
#   packaging/macos-store/test-store.sh <frename.pkg> [screenshot.png]
#
# 1. refuses to run where a GStreamer is installed;
# 2. installs the .pkg (as the App Store does, into /Applications), checks the app links only
#    macOS and itself and is signed with the sandbox entitlement;
# 3. proves the sandbox is on: the first start creates the app's container, and a clip outside
#    the container cannot be read;
# 4. runs the self-test on the test clips copied into the container: the bundled GStreamer
#    decodes them in the sandbox, and the Keychain works (the Store build's self-test checks it);
# 5. starts the app the way Finder does, checks it runs after 20 s and uses its own GStreamer,
#    and takes a screenshot.
# Needs an ad-hoc-signed pkg (CI's unsigned build): a Store-signed app runs only once the
# App Store (or TestFlight) installed it. Run from the repository root.
set -euo pipefail

pkg="$1"
screenshot="${2:-}"
repo="$(cd "$(dirname "$0")/../.." && pwd)"
bundle_id="io.github.zelenov.frename"
container="$HOME/Library/Containers/$bundle_id/Data"
log="$container/Library/Application Support/frename/frename_debug.log"
app=/Applications/frename.app
exe="$app/Contents/MacOS/frename"

echo "== 1. No GStreamer on this machine"
if [ -e /Library/Frameworks/GStreamer.framework ] || command -v gst-launch-1.0 > /dev/null \
  || { command -v brew > /dev/null && brew list --versions gstreamer > /dev/null 2>&1; }; then
  echo "This runner has a GStreamer; the test needs a machine without one." >&2
  exit 1
fi

echo "== 2. Install the .pkg"
pkgutil --payload-files "$pkg" | head -5
sudo installer -pkg "$pkg" -target /
[ -x "$exe" ] || { echo "The .pkg did not install $app" >&2; exit 1; }
"$repo/packaging/macos/check-bundle.sh" "$app"
sandbox="$(codesign --display --entitlements - --xml "$app" 2> /dev/null \
  | plutil -extract com.apple.security.app-sandbox raw -o - - || true)"
[ "$sandbox" = "true" ] || { echo "$app is not signed with the sandbox entitlement" >&2; exit 1; }

echo "== 3. The sandbox is on"
# No videos given: exits 1 at once, after the system has made the container.
if "$exe" --self-test; then
  echo "A self-test without videos must fail" >&2
  exit 1
fi
[ -d "$container" ] || { echo "No container at $container: not sandboxed" >&2; exit 1; }
outside="$repo/tests/folder/file_example_MP4_480_1_5MG.mp4"
if "$exe" --self-test "$outside"; then
  echo "The sandboxed app read $outside, outside its container" >&2
  exit 1
fi
grep "self-test: FAILED" "$log"

echo "== 4. Self-test inside the sandbox"
mkdir -p "$container/clips"
clips=()
while IFS= read -r clip; do
  cp "$repo/$clip" "$container/clips/"
  clips+=("$container/clips/$(basename "$clip")")
done < <(sed 's/#.*//; s/^[[:space:]]*//; s/[[:space:]]*$//' "$repo/tests/self-test-clips.txt" | grep .)
"$exe" --self-test "${clips[@]}"
grep -q "ok     credential store" "$log" || { echo "The Keychain check did not run" >&2; exit 1; }

echo "== 5. Start the app"
open -n "$app"
sleep 20
if ! pgrep -f "$exe" > /dev/null; then
  echo "frename is not running 20 s after start. Its log:" >&2
  cat "$log" >&2 || true
  exit 1
fi
[ -n "$screenshot" ] && { screencapture -x "$screenshot" || echo "(no screenshot: $?)"; }
pkill -f "$exe" || true
cat "$log"
grep -q "(App Store)" "$log" || { echo "Not the App Store build" >&2; exit 1; }
grep -q "using the GStreamer bundled with frename" "$log" \
  || { echo "The app did not use its bundled GStreamer" >&2; exit 1; }

echo "All Store app tests passed."
