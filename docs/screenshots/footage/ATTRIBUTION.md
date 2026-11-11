# Footage of the README images

The clips in this folder are short cuts of two films by the Blender Foundation, used under the
[Creative Commons Attribution 3.0](https://creativecommons.org/licenses/by/3.0/) license. They
were cut, scaled to 640 pixels wide and re-encoded without sound for frename's README images
(`docs/images/`, made by `docs/images/render.sh`).

| Clips | Film | Credit |
|---|---|---|
| `tears-of-steel-*.mp4` | Tears of Steel (2012) | (CC) Blender Foundation, [mango.blender.org](https://mango.blender.org) |
| `sintel-*.mp4` | Sintel (2010) | (CC) Blender Foundation, [durian.blender.org](https://durian.blender.org) |

Source files: `videos/tears-of-steel-battle-clip-medium.mp4` and `videos/sintel-short.mp4` of
[mdn/shared-assets](https://github.com/mdn/shared-assets) at commit `f160bcf`. Each clip was made with

```sh
ffmpeg -ss <start> -i <source> -t <length> -an -vf scale=640:-2 -c:v libx264 -preset veryslow \
  -crf 31 -pix_fmt yuv420p -g 50 -movflags +faststart -map_metadata -1 -fflags +bitexact <clip>
```

| Clip | Source | Start (s) | Length (s) |
|---|---|---|---|
| `tears-of-steel-tower.mp4` | Tears of Steel | 3 | 5 |
| `tears-of-steel-thom.mp4` | Tears of Steel | 14 | 12 |
| `tears-of-steel-battle.mp4` | Tears of Steel | 37.5 | 7 |
| `tears-of-steel-rope.mp4` | Tears of Steel | 49.5 | 6 |
| `tears-of-steel-organ.mp4` | Tears of Steel | 60 | 6 |
| `sintel-mountains.mp4` | Sintel | 0 | 8 |
| `sintel-city.mp4` | Sintel | 62 | 8 |
| `sintel-hut.mp4` | Sintel | 33 | 8 |
| `sintel-rooftop.mp4` | Sintel | 94 | 7 |
| `sintel-dragon.mp4` | Sintel | 103 | 8 |

The clip names in the README images (`MVI_0201.MP4` and so on) are made up for the demo.
