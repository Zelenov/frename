# Self-test clips for the Windows bundle

One tiny clip per container and codec the bundled GStreamer carries plugins for
(`packaging/windows/gstreamer-plugins.txt`). The installer test in CI runs
`frename.exe --self-test tests/media` on the installed and the portable package; every clip must
decode a frame. A new format the bundle should play gets a clip here.

Made with FFmpeg 7 (a gyan.dev full build) from a test pattern and a tone:

```sh
SRC="-f lavfi -i testsrc2=size=320x240:rate=15:duration=1 -f lavfi -i sine=frequency=440:duration=1"
ffmpeg $SRC -c:v libx264 -pix_fmt yuv420p -c:a aac -b:a 32k h264-aac.mp4
ffmpeg $SRC -c:v libx265 -pix_fmt yuv420p -tag:v hvc1 -c:a aac -b:a 32k hevc-aac.mov
ffmpeg $SRC -c:v libvpx-vp9 -b:v 50k -c:a libopus -b:a 16k vp9-opus.webm
ffmpeg $SRC -c:v libaom-av1 -cpu-used 8 -b:v 50k -c:a libopus -b:a 16k av1-opus.mkv
ffmpeg $SRC -c:v mpeg4 -q:v 10 -c:a libmp3lame -b:a 32k mpeg4-mp3.avi
# Variable frame rate (frames 10-20 dropped), for the capssetter retry in the player
ffmpeg -f lavfi -i "testsrc2=size=320x240:rate=30:duration=2,select='not(between(n\,10\,20))'" \
  -f lavfi -i sine=frequency=440:duration=2 -fps_mode vfr \
  -c:v libx264 -pix_fmt yuv420p -c:a aac -b:a 32k h264-vfr.mp4
```
