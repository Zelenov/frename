#!/usr/bin/env bash
# Check that frename.app needs nothing outside itself and macOS: every library a Mach-O file in
# the bundle links is part of macOS or inside Contents/Frameworks, no rpath points outside the
# bundle, and the signature is valid.
#
#   packaging/macos/check-bundle.sh <frename.app>
set -euo pipefail

app="$(cd "$1" && pwd)"
frameworks="$app/Contents/Frameworks"
problems=0

while IFS= read -r -d '' file; do
  file "$file" | grep -q 'Mach-O' || continue
  own="$(otool -D "$file" | sed -n 2p)"
  while read -r dep; do
    case "$dep" in
      "" | "$own" | /usr/lib/* | /System/*) ;;
      @rpath/*)
        if [ ! -f "$frameworks/${dep#@rpath/}" ]; then
          echo "MISSING $dep (linked by ${file#"$app"/})"
          problems=$((problems + 1))
        fi
        ;;
      *)
        echo "OUTSIDE $dep (linked by ${file#"$app"/})"
        problems=$((problems + 1))
        ;;
    esac
  done < <(otool -L "$file" | tail -n +2 | awk '{print $1}')
  while read -r rpath; do
    case "$rpath" in
      @executable_path | @executable_path/* | @loader_path | @loader_path/*) ;;
      *)
        echo "RPATH $rpath (in ${file#"$app"/})"
        problems=$((problems + 1))
        ;;
    esac
  done < <(otool -l "$file" | awk '/cmd LC_RPATH/ { getline; getline; print $2 }')
done < <(find "$app/Contents" -type f -print0)

codesign --verify --strict --deep "$app" || problems=$((problems + 1))

if [ "$problems" -gt 0 ]; then
  echo "$problems problems in $app" >&2
  exit 1
fi
echo "$app links only macOS and its own libraries; signature valid"
