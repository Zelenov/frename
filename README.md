# frename

frename is a free app that gets a folder of raw video ready for the edit. You watch each clip,
tag it and mark the moments you want, and AI can describe it for you. Premiere Pro shows all of
it when you import the clips.

[Download for Windows](https://github.com/Zelenov/frename/releases/latest/download/frename-win-Setup.exe)
or get the [Linux AppImage and the Windows zip](https://github.com/Zelenov/frename/releases/latest).
A Mac version is not available yet. The app is in English, Russian, German, French, Spanish,
Italian, Brazilian Portuguese, Ukrainian, Serbian, Japanese, Korean and Chinese.

![The frename window with a tagged folder of clips, colored markers and an in and out range on the timeline, and the start of an AI description in the comment box](docs/images/main.jpg)

## Describe each clip with AI

frename can write a short description of each clip and list what happens at which second, even
in a clip with no speech. The description goes into the clip's comment, where you can search for
it later. Premiere Pro shows it in the Description column. AI can also suggest tags, and in and
out points for the part you want to keep.

![A clip of a small dragon next to the comment box, which holds the AI's summary of the clip and what happens at each second](docs/images/describe.jpg)

You use your own key for Claude, ChatGPT or Gemini, and Settings has a button that opens the
page where you get one. With Claude Haiku, 1,000 clips of one minute each cost about $10.

## Mark key moments and the part to keep

Press `F2` to put a marker at a moment you want to find again, and give it a name and a color.
Press `[` and `]` to set the in and out points, so only the good part of the clip goes into
the edit. Premiere Pro shows the markers on the clip, and it uses the in and out points as a
subclip.

![The marker list over the video, and the colored markers and the in and out range on the timeline](docs/images/markers.jpg)

## Tag clips so the footage has a structure

Click the tags that fit a clip, or pick them with the keyboard. frename adds them to the file
name when you move to the next clip, so `MVI_0207.mp4` becomes
`pick.medium.people.action.MVI_0207.mp4`. Your file manager, Premiere Pro and your backup tools
all see the tags and can sort by them.

![The same folder before and after tagging, with the camera's file names on the left and the tagged names on the right](docs/images/tags.jpg)

## Work on a whole folder at once

Batch mode runs one action on every clip you check, e.g. subtitles from the speech or AI
descriptions. Prepare clips does both for a whole folder in one run, and you can set the most
it may spend. A long action runs in the background, and you can keep working while it runs.

![Batch mode with every clip of the folder checked and Describe with AI ready to describe 11 videos for about 7 cents](docs/images/batch.jpg)

## What you get in Premiere Pro

When you import the clips into Premiere Pro, you get the following:

- the tags in each file name, so you can find and sort the clips in the Project panel;
- the markers on each clip, with their names, colors and lengths;
- the in and out points, which Premiere Pro uses as a subclip;
- the comment in the Description column, where search finds it.

If you made subtitles, you can import them into Premiere Pro as captions or as a transcript. On
Windows you can drag clips from frename straight into Premiere Pro.

## Your files and your keys

- frename keeps everything in the clips and next to them, so your work goes with the footage
  when you copy the folder.
- frename sends your footage nowhere unless you ask. For an AI description, the frames, the
  subtitles and your notes for the AI go to the AI service you chose. For subtitles, the sound
  goes to Soniox. Both use your own key.
- Your keys stay in the password store of your computer.
- On Windows, frename checks once a day for a new version, and you can turn that off in Settings.
- frename is open source under the MIT license.

## Undo and recovery

`Ctrl+Z` undoes almost everything. frename also keeps a copy of your unsaved work on the open
clip, so you don't lose it if frename crashes or the power goes out.

## More

- [The guide](docs/guide.md) explains every feature, the keyboard shortcuts, the install and
  where frename keeps your data.
- [Building frename from source](docs/development.md) is for developers.
- [What changed in each version](version.md).
- The images show footage from Sintel and Tears of Steel, (CC) Blender Foundation, under the
  [CC BY 3.0](docs/screenshots/ATTRIBUTION.md) license.
