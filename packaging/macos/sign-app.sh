#!/usr/bin/env bash
# Sign frename.app ad hoc, from the inside out: every bundled library and plugin, then the app.
#
#   packaging/macos/sign-app.sh <frename.app>
#
# An ad-hoc signature needs no Apple account and is enough for Apple silicon, which runs only
# signed code. Gatekeeper still blocks an ad-hoc app downloaded from the web until its
# quarantine flag is removed (README).
set -euo pipefail

app="$1"

# install_name_tool left the libraries' signatures invalid: replace them all.
while IFS= read -r -d '' file; do
  codesign --force --sign - --timestamp=none "$file"
done < <(find "$app/Contents/Frameworks" "$app/Contents/PlugIns" -type f \( -name '*.dylib' -o -name '*.so' \) -print0)

codesign --force --sign - --timestamp=none "$app"

codesign --verify --strict --deep --verbose=2 "$app"
codesign --display --verbose=2 "$app" 2>&1 | grep -E '^(Identifier|Format|Signature|Authority|TeamIdentifier)'
