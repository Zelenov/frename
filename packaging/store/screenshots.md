# Store screenshots

Upload in this order in Partner Center → Store listings → English (United States) →
Screenshots → Desktop. The Store takes PNG, 1366×768 or larger, up to 10 per listing.
The files here are 1920×1080 captures of the app's demo mode, from the scenarios
`docs/screenshots/store-*.toml`.

| File | Caption (up to 200 characters) |
|---|---|
| `screenshots/1-main-window.png` | Watch each clip, tag what you see, and move on: the clip you leave is renamed with its tags. |
| `screenshots/2-markers-and-subtitles.png` | Name and color markers on the timeline; Premiere Pro shows them on the clip. Subtitles sit in a list beside the picture. |
| `screenshots/3-ai-in-out.png` | Describe with AI writes a summary and suggests an In and Out; one click on the AI pill sets them. |
| `screenshots/4-batch-actions.png` | Batch actions over a whole folder, for example moving comments between the videos and text files. |

When the app's look changes, render them again (Linux, Xvfb 1920×1080 as in `CLAUDE.md` "Looking at the UI";
the scenarios keep the screen's centre, where Xvfb starts the pointer, over the video, so no
tooltip is open) and look at every image before committing:

```sh
cd docs/screenshots
frename --demo store-main.toml --out ../../packaging/store/screenshots/1-main-window.png
frename --demo store-markers.toml --out ../../packaging/store/screenshots/2-markers-and-subtitles.png
frename --demo store-ai.toml --out ../../packaging/store/screenshots/3-ai-in-out.png
frename --demo store-batch.toml --batch --out ../../packaging/store/screenshots/4-batch-actions.png
```

Optional later: a screenshot of the Settings window, and a 1:1 300×300 app logo
(`packaging/windows/msix/Assets` holds only the package logos).
