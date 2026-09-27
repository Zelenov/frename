# Store screenshots

Upload in this order in Partner Center → Store listings → English (United States) →
Screenshots → Desktop. The Store takes PNG, 1366×768 or larger, up to 10 per listing.
The files here are the README screenshots (`docs/frename-screenshot*.jpg`, 2020×1128) as PNG.

| File | Caption (up to 200 characters) |
|---|---|
| `screenshots/1-main-window.png` | Watch each clip, tag what you see, and move on: the clip you leave is renamed with its tags. |
| `screenshots/2-batch-actions.png` | Batch actions over the checked clips: move comments and in/out points, tag commented clips, reorder tags. |
| `screenshots/3-monochrome-tags.png` | Subtitles, in and out points and a comment per clip; tags in one neutral color if you prefer. |

When the README screenshots change (`docs/screenshots/render.sh`), refresh these with ImageMagick:

```sh
convert docs/frename-screenshot.jpg -strip PNG24:packaging/store/screenshots/1-main-window.png
convert docs/frename-screenshot-batch.jpg -strip PNG24:packaging/store/screenshots/2-batch-actions.png
convert docs/frename-screenshot-mono.jpg -strip PNG24:packaging/store/screenshots/3-monochrome-tags.png
```

Optional later: a fourth screenshot of the Settings window, and a 1:1 300×300 app logo
(`packaging/windows/msix/Assets` holds only the package logos).
