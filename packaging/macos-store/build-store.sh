#!/usr/bin/env bash
# Build the Mac App Store variant of frename.app and its .pkg for upload.
#
#   packaging/macos-store/build-store.sh <version> <build number> <frename binary> <output dir>
#
# The binary must be built with `--features store` (sandboxed behaviour, no self-update).
# Makes <output dir>/frename.app and <output dir>/frename.pkg: the app from
# packaging/macos/build-app.sh plus the privacy manifest and export-compliance key, signed with
# the sandbox entitlements.
#
# Signed for the Store when all of these are set (docs/mac-app-store-setup.md):
#   MAS_APP_IDENTITY        the "Apple Distribution: …" identity in the keychain
#   MAS_INSTALLER_IDENTITY  the "Mac Installer Distribution: …" (or "3rd Party Mac Developer
#                           Installer: …") identity
#   MAS_TEAM_ID             the Apple Developer team ID
#   MAS_PROFILE             path to the "Mac App Store Connect" provisioning profile
# Otherwise the app is signed ad hoc, with the same sandbox (so CI can test it sandboxed), and
# the .pkg is unsigned: fine for testing, refused by App Store Connect. It says so.
set -euo pipefail

version="$1"
build_number="$2"
binary="$3"
out="$4"
here="$(cd "$(dirname "$0")" && pwd)"
bundle_id="io.github.zelenov.frename"
app="$out/frename.app"
pkg="$out/frename.pkg"

signed=false
if [ -n "${MAS_APP_IDENTITY:-}" ] && [ -n "${MAS_INSTALLER_IDENTITY:-}" ] \
  && [ -n "${MAS_TEAM_ID:-}" ] && [ -n "${MAS_PROFILE:-}" ]; then
  signed=true
fi

mkdir -p "$out"
BUNDLE_ID="$bundle_id" BUILD_VERSION="$build_number" \
  "$here/../macos/build-app.sh" "$version" "$binary" "$app"

cp "$here/PrivacyInfo.xcprivacy" "$app/Contents/Resources/"
# HTTPS only (Anthropic, Soniox): exempt from export documentation; the owner confirms this in
# App Store Connect once (docs/mac-app-store-setup.md).
/usr/libexec/PlistBuddy -c "Add :ITSAppUsesNonExemptEncryption bool false" "$app/Contents/Info.plist"

entitlements="$(mktemp -d)/entitlements.plist"
cp "$here/entitlements.plist" "$entitlements"
if $signed; then
  cp "$MAS_PROFILE" "$app/Contents/embedded.provisionprofile"
  /usr/libexec/PlistBuddy \
    -c "Add :com.apple.application-identifier string $MAS_TEAM_ID.$bundle_id" \
    -c "Add :com.apple.developer.team-identifier string $MAS_TEAM_ID" \
    "$entitlements"
  "$here/../macos/sign-app.sh" "$app" "$MAS_APP_IDENTITY" "$entitlements"
  productbuild --component "$app" /Applications --sign "$MAS_INSTALLER_IDENTITY" "$pkg"
  pkgutil --check-signature "$pkg"
  echo "Built $pkg, signed for the Mac App Store"
else
  "$here/../macos/sign-app.sh" "$app" - "$entitlements"
  productbuild --component "$app" /Applications "$pkg"
  echo "::warning::Mac App Store signing secrets are not set: $app is signed ad hoc (sandboxed," \
    "for testing) and $pkg is unsigned, which App Store Connect refuses." \
    "See docs/mac-app-store-setup.md."
fi
codesign --display --entitlements - "$app"
