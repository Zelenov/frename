# CAPT as the subtitle source (issue #353)

[CAPT](https://github.com/Zelenov/sonisub/blob/main/docs/capt.md) is sonisub's text file for a whole
transcript: every piece of a word with its time, cut points, sentences, speakers, languages, sounds,
voices, places, italic. sonisub 0.9 makes SRT, WebVTT, ASS, Premiere, SBV, LRC, a plain transcript,
TTML, a Fairlight ADR cue list and YTT from it, without Soniox. frename pins sonisub 0.9.1.

The owner wants CAPT to be frename's main subtitle format. This is written in steps; this PR is
step 1. Every question of the issue is decided here by the agent (nothing waits for the owner).

## Decisions made without the owner

1. **CAPT is the source of truth, next to the clip** (`clip.capt`, like `clip.soniox.json`). Not in a
   library folder: the editor already moves clips between folders and drives, and every other
   sidecar travels with the clip. It moves with the clip on rename, and counts as "name taken".
2. **CAPT is written with every output**, SRT included. `.soniox.json` stays for now (it is the
   input of today's free rebuild); CAPT makes it redundant later (step 3).
3. **A clip whose outputs all exist is never sent to Soniox only to get a CAPT.** The job adds the
   CAPT to what it writes only when it writes something anyway (a transcription, or a free rebuild
   from the saved transcript). Folders of old `.srt` files stay "already have subtitles", free.
4. **A hand-edited `.srt` is never overwritten** (unchanged rule: only missing outputs are written;
   Replace means transcribe again). A hand-edited `.capt` is overwritten only by Replace, as the
   word says. A `.capt` with neither `.srt` nor `.soniox.json` still means a new transcription for the
   `.srt` (the edited `.capt` is kept); step 2 closes this.
5. **The result list does not name the CAPT file** (it goes with every run, not news).
6. **Older files stay readable**: nothing is read back from an `.srt` into CAPT (no word timing, so
   a coarse CAPT would look like a real one). The way from an old `.srt` to a CAPT is to
   transcribe again, with the cost shown first, as the plan already does for Replace.

## Steps

1. *(this PR)* `clip.capt` written next to new subtitles; sidecar handling in core (rename, name
   taken, undo); tests.
2. A saved `.capt` is a transcript source: "rebuilt free" also when only `clip.capt` is there (no
   `.soniox.json`), so a hand-edited CAPT (cut points, speaker names) is what other formats are
   made from. Needs a library call that lays out a `Doc` (`job::write_outputs`).
3. More formats in the Generate subtitles panel: WebVTT, ASS, TTML, SBV, LRC, transcript, YTT as
   checkboxes behind "More formats"; ADR with frame rate and timeline start. The choice is per
   batch, remembered as the default. The result list names what was written.
4. The folder as a whole: per clip, which of `.capt`, `.srt`, … exist and which are older than the
   CAPT (modification time), shown in the clip's subtitle tab; a "make again from CAPT" action.
5. Stop keeping `.soniox.json` for new clips once step 2 is out for a release.
