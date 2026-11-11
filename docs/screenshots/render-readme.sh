#!/usr/bin/env bash
# Makes the README images (docs/images/*.jpg) from readme-main.toml and readme-before.toml here:
# runs the app in demo mode, then crops and labels the shots with ImageMagick. (render.sh here
# makes the annotated window of the guide.)
#
#   docs/screenshots/render-readme.sh [frename binary]     (default: target/debug/frename)
#
# Runs on Linux and in Git Bash on Windows. Needs ImageMagick (`magick`, or `convert` on Linux)
# and a display that fits a 1600x860 window with the pointer outside it, or a tooltip shows: on
# Linux `Xvfb :99 -screen 0 3400x1800x24`, whose pointer starts in the middle of the screen.
# CI runs it in ci-linux and uploads the result. Look at every image before committing it
# (.claude/skills/readme/SKILL.md).
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
root=$(dirname "$(dirname "$here")")
images=$root/docs/images
binary=${1:-$root/target/debug/frename}
[ -x "$binary" ] || [ -x "$binary.exe" ] || { echo "no binary at $binary: run cargo build" >&2; exit 1; }
binary=$(cd "$(dirname "$binary")" && pwd)/$(basename "$binary")

# `convert` in Git Bash may be Windows' own convert.exe, which is not ImageMagick.
if command -v magick >/dev/null; then
  im=(magick)
elif convert -version 2>/dev/null | grep -q ImageMagick; then
  im=(convert)
else
  echo "needs ImageMagick (magick, or convert on Linux)" >&2
  exit 1
fi
# The labels' font when it is installed; ImageMagick's own default otherwise.
font=()
if "${im[@]}" -list font 2>/dev/null | grep -q 'Font: DejaVu-Sans$'; then font=(-font DejaVu-Sans); fi

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# The footage folder as the app reads it (a Windows path in Git Bash).
footage=$here/footage
if command -v cygpath >/dev/null; then footage=$(cygpath -m "$footage"); fi

# variant <name> <sed script>: readme-main.toml changed by the sed script, in the work folder. The
# script adds keys before the first [[files]], or replaces a line.
variant() {
  sed -e "s|^source = .*|source = \"$footage\"|" -e "$2" "$here/readme-main.toml" > "$work/$1.toml"
}
variant markers '0,/^\[\[files\]\]/s//marker_list = true\n\n[[files]]/'
variant describe 's/^open = .*/open = "pick.close.wildlife.MVI_0205.mp4"/; s/^seek = .*/seek = 6.5/; 0,/^\[\[files\]\]/s//comment_expanded = true\n\n[[files]]/'

# shot <scenario> <png> [demo flags]: the main window of one scenario, 1600x860.
shot() {
  local scenario=$1 out=$2
  shift 2
  (cd "$here" && "$binary" --demo "$scenario" --out "$work/$out" "$@")
}

# jpg <png> <jpg>: the published form, sized for GitHub (at most 1600 pixels wide).
jpg() {
  "${im[@]}" "$1" -resize '1600x>' -strip -quality 88 "$images/$2"
  echo "wrote docs/images/$2"
}

mkdir -p "$images"
shot readme-main.toml main.png
shot readme-main.toml batch.png --batch --ai
shot readme-before.toml before.png
shot "$work/markers.toml" markers.png
shot "$work/describe.toml" describe.png

# The crop boxes below follow the window layout of the scenarios (window 1600x860, video panel
# 820, file list 380, with the dividers between them): WxH+X+Y in window pixels. The file list
# leaves out the bar with Single file and Batch above it, and the empty rows and buttons below
# the twelve clips. A layout change can leave them wrong while the script still succeeds: look
# at the images.
VIDEO_PANEL=826x860+0+0
FILE_LIST=392x660+826+36
RIGHT_COLUMN=382x860+1218+0

jpg "$work/main.png" main.jpg
jpg "$work/batch.png" batch.jpg
"${im[@]}" "$work/markers.png" -crop "$VIDEO_PANEL" +repage "$work/markers-crop.png"
jpg "$work/markers-crop.png" markers.jpg

# Describe: the video next to the right column with the expanded comment box.
"${im[@]}" "$work/describe.png" \( -clone 0 -crop "$VIDEO_PANEL" +repage \) \
  \( -clone 0 -crop "$RIGHT_COLUMN" +repage \) -delete 0 +append "$work/describe-crop.png"
jpg "$work/describe-crop.png" describe.jpg

# Tags: the file list before and after tagging, side by side, each with a label over it.
label() {
  "${im[@]}" "$1" -crop "$FILE_LIST" +repage -background '#282923' -fill '#E3E3DC' \
    "${font[@]}" -pointsize 28 -gravity center label:"$2" +swap -append \
    -bordercolor '#282923' -border 20 "$3"
}
label "$work/before.png" "Before" "$work/before-list.png"
label "$work/main.png" "After" "$work/after-list.png"
"${im[@]}" "$work/before-list.png" "$work/after-list.png" +append "$work/tags.png"
jpg "$work/tags.png" tags.jpg
