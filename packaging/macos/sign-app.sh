#!/usr/bin/env bash
# Sign frename.app from the inside out: every bundled library and plugin, then the app.
#
#   packaging/macos/sign-app.sh <frename.app> [identity] [entitlements.plist]
#
# The identity defaults to `-`, an ad-hoc signature: no Apple account, and enough for Apple
# silicon, which runs only signed code. Gatekeeper still blocks an ad-hoc app downloaded from the
# web until its quarantine flag is removed (README). The Mac App Store build passes its
# "Apple Distribution" identity and its sandbox entitlements (packaging/macos-store/).
set -euo pipefail

app="$1"
identity="${2:--}"
entitlements="${3:-}"

timestamp=()
[ "$identity" = "-" ] && timestamp=(--timestamp=none)

# install_name_tool left the libraries' signatures invalid: replace them all.
while IFS= read -r -d '' file; do
  codesign --force --sign "$identity" ${timestamp[@]+"${timestamp[@]}"} "$file"
done < <(find "$app/Contents/Frameworks" "$app/Contents/PlugIns" -type f -name '*.dylib' -print0)

main=(--force --sign "$identity" ${timestamp[@]+"${timestamp[@]}"})
[ -n "$entitlements" ] && main+=(--entitlements "$entitlements")
codesign "${main[@]}" "$app"

codesign --verify --strict --deep --verbose=2 "$app"
codesign --display --verbose=2 "$app" 2>&1 | grep -E '^(Identifier|Format|Signature|Authority|TeamIdentifier)'
