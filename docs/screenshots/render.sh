#!/usr/bin/env bash
# Renders the README screenshots: runs the app in demo mode for each scenario here, then puts the
# screenshot into its annotated template.
#
#   docs/screenshots/render.sh <frename binary>
#
# Needs a display that fits a 1920x1009 window at scale 1 (in CI: Xvfb :99 -screen 0 1920x1080x24),
# rsvg-convert (librsvg2-bin), ImageMagick and the Open Sans font. Writes
# docs/frename-screenshot.jpg and docs/frename-screenshot-batch.jpg.
set -euo pipefail

binary=$(realpath "$1")
here=$(cd "$(dirname "$0")" && pwd)
docs=$(dirname "$here")

# render <template> <output> [demo flags]: both templates show the one folder of main.toml.
render() {
  local template=$1 output=$2
  shift 2
  "$binary" --demo "$here/main.toml" --out "$here/$template.png" "$@"
  # librsvg only loads images next to or below the template, so the PNG stays in this folder.
  rsvg-convert "$here/$template.svg" -o "$here/$template-annotated.png"
  convert "$here/$template-annotated.png" -quality 88 "$docs/$output"
  rm "$here/$template-annotated.png"
}

render main frename-screenshot.jpg
render batch frename-screenshot-batch.jpg --batch
