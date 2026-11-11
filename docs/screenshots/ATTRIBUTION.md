# Footage of the README images

The clips in `footage/` are short cuts of two films by the Blender Foundation, used under the
[Creative Commons Attribution 3.0](https://creativecommons.org/licenses/by/3.0/) license. They
were cut, cropped to 16:9 in the middle, scaled to 640x360 and re-encoded without sound for
frename's README images (`docs/images/`, made by `docs/screenshots/render-readme.sh`).

| Clips | Film | Credit |
|---|---|---|
| `tears-of-steel-*.mp4` | Tears of Steel (2012) | (CC) Blender Foundation, [mango.blender.org](https://mango.blender.org) |
| `sintel-*.mp4` | Sintel (2010) | (CC) Blender Foundation, [durian.blender.org](https://durian.blender.org) |

Source files: `videos/tears-of-steel-battle-clip-medium.mp4` and `videos/sintel-short.mp4` of
[mdn/shared-assets](https://github.com/mdn/shared-assets) at commit `f160bcf`. Each clip was made with

```sh
ffmpeg -ss <start> -i <source> -t <length> -an -vf "crop=ih*16/9:ih,scale=640:360" -c:v libx264 -preset veryslow \
  -crf 24 -pix_fmt yuv420p -g 50 -movflags +faststart -map_metadata -1 -map_chapters -1 -fflags +bitexact <clip>
```

| Clip | Source | Start (s) | Length (s) |
|---|---|---|---|
| `tears-of-steel-thom.mp4` | Tears of Steel | 14 | 12 |
| `sintel-dragon.mp4` | Sintel | 103 | 8 |
| `sintel-city.mp4` | Sintel | 62 | 8 |

The clip names in the README images (`MVI_0201.MP4` and so on) are made up for the demo.
