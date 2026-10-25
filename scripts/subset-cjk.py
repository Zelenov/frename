#!/usr/bin/env python3
"""Rebuilds assets/fonts/NotoSansCJK-<JP|KR|SC|TC>-{Regular,Bold}.otf: Noto Sans CJK cut down to the
glyphs the interface uses.

    sudo apt-get install fonts-noto-cjk python3-fonttools
    scripts/subset-cjk.py ja            # one language; no argument: every language that has files

The glyphs are ASCII, Latin-1 and Latin Extended-A, common punctuation, and every character of
`i18n/<lang>/frename.ftl` (which also holds the language names of the other languages). Run it
again after an edit to that file adds a character; the test `cjk_fonts_cover_their_texts` in
src/ui/tokens/typography.rs fails until you do. Source: the Debian package fonts-noto-cjk (Noto
Sans CJK, SIL OFL 1.1).
"""
import os
import sys

from fontTools import subset
from fontTools.ttLib import TTCollection

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SOURCE = "/usr/share/fonts/opentype/noto/NotoSansCJK-{weight}.ttc"
# language code -> (face index in the collection, file name suffix)
FACES = {"ja": (0, "JP"), "ko": (1, "KR"), "zh-Hans": (2, "SC"), "zh-Hant": (3, "TC")}
EXTRA = "“”‘’…–—·•→←↑↓⇄✨★☆✓×°′″「」『』、。，．：；！？（）【】〜～・"


def glyphs(language):
    with open(os.path.join(ROOT, "i18n", language, "frename.ftl"), encoding="utf8") as f:
        text = set(f.read())
    text |= {chr(c) for c in range(0x20, 0x7F)} | {chr(c) for c in range(0xA0, 0x180)} | set(EXTRA)
    return "".join(sorted(text))


def build(language):
    index, suffix = FACES[language]
    for weight in ("Regular", "Bold"):
        font = TTCollection(SOURCE.format(weight=weight)).fonts[index]
        options = subset.Options()
        options.layout_features = ["kern", "locl", "vert", "palt", "halt"]
        options.name_IDs = [1, 2, 3, 4, 6]
        options.notdef_outline = True
        options.hinting = False
        options.desubroutinize = True
        subsetter = subset.Subsetter(options)
        subsetter.populate(text=glyphs(language))
        subsetter.subset(font)
        font.flavor = None
        out = os.path.join(ROOT, "assets", "fonts", f"NotoSansCJK-{suffix}-{weight}.otf")
        font.save(out)
        print(out, os.path.getsize(out) // 1024, "KB")


if __name__ == "__main__":
    wanted = sys.argv[1:] or [
        l for l in FACES if os.path.exists(os.path.join(ROOT, "i18n", l, "frename.ftl"))
    ]
    for language in wanted:
        build(language)
