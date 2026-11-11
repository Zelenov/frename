# Building frename

frename is written in Rust. The interface uses [iced](https://iced.rs), and video plays through
[GStreamer](https://gstreamer.freedesktop.org). The AI descriptions come from
[clipscribe](https://github.com/Zelenov/clipscribe) and the subtitles from
[sonisub](https://github.com/Zelenov/sonisub), two small libraries made for frename.

## Build and run

You need Rust and the GStreamer development files. [GSTREAMER_SETUP.md](../GSTREAMER_SETUP.md)
explains how to install them on Windows and Linux.

```sh
cargo build
cargo run
```

Every change is tested on Windows and Linux in GitHub Actions, and every release is built and
published from there. `CLAUDE.md` lists the checks CI runs.

## README images

The images in `docs/images/` are screenshots of the app in demo mode, made by
`docs/images/render.sh` from the scenarios `docs/screenshots/readme-*.toml`. The footage in them
is in `docs/screenshots/footage/`, with its license in `ATTRIBUTION.md` there. CI renders them on
every pull request (job `readme-images`), so a change to the interface can be checked against
them, and the rendered set can be downloaded from the run as an artifact.
