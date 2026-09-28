#!/usr/bin/env bash
# Renders the README screenshots by hand, when a change makes them wrong: runs the app in demo mode
# for each template here, then puts the window into its annotated template.
#
#   docs/screenshots/render.sh [frename binary]     (default: target/debug/frename)
#
# Runs on Windows (Git Bash) and on Linux. Needs ImageMagick (`magick`, or `convert` with
# rsvg-convert on Linux) and a display that fits a 1440x720 window. Writes
# docs/frename-screenshot*.jpg; look at every one before committing.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
docs=$(dirname "$here")
root=$(dirname "$docs")
binary=${1:-$root/target/debug/frename}
[ -x "$binary" ] || [ -x "$binary.exe" ] || { echo "no binary at $binary: run cargo build" >&2; exit 1; }
binary=$(cd "$(dirname "$binary")" && pwd)/$(basename "$binary")

# SVG to PNG: librsvg in both cases (ImageMagick for Windows has it built in).
if command -v rsvg-convert >/dev/null; then
  svg_to_png() { rsvg-convert "$1" -o "$2"; }
  to_jpg() { convert "$1" -quality 88 "$2"; }
elif command -v magick >/dev/null; then
  svg_to_png() { magick "$1" "$2"; }
  to_jpg() { magick "$1" -quality 88 "$2"; }
else
  echo "needs rsvg-convert + ImageMagick, or ImageMagick 7 (magick)" >&2
  exit 1
fi

# render <template> <scenario> <output> [demo flags]: main and batch show main.toml's folder;
# mono has its own scenario (mono.toml) so it can add a tag that is not in the list.
render() {
  local template=$1 scenario=$2 output=$3
  shift 3
  (cd "$here" && "$binary" --demo "$scenario" --out "$template.png" "$@")
  # librsvg only loads images next to or below the template, so the PNGs stay in this folder.
  svg_to_png "$here/$template.svg" "$here/$template-annotated.png"
  to_jpg "$here/$template-annotated.png" "$docs/$output"
  rm "$here/$template-annotated.png"
  echo "wrote docs/$output"
}

render main main.toml frename-screenshot.jpg
render batch main.toml frename-screenshot-batch.jpg --batch
render mono mono.toml frename-screenshot-mono.jpg --mono
