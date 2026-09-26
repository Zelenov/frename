# Localization of the UI

Design for issue #18: decide whether and how to localize the UI, then do it.
Code references are to `main` at `8a0bddc` (demo mode from #14 included; line numbers in other
files may be off by a few lines).

## Problem

Every word in the UI is an English literal inside a `view` function. The owner's language is
Russian, and a user who does not read English cannot find out what the Settings options or the
batch actions do. There is no place to put a translation, no way to pick a language, and a few
sentences are built by gluing English words (`files()` at `src/features/batch/view.rs:201`
returns `"1 file"` / `"5 files"`), which cannot be translated into a language with three plural
forms.

## Usage evidence

frename has no telemetry. What GitHub shows (collected 2026-09-26):

| Signal | Value |
|---|---|
| Release downloads, all versions, all assets | 3 (all on the v0.60 Windows zip; v0.67 Windows and Linux: 0) |
| Stars / forks / watchers besides the owner | 1 / 0 / 0 |
| Issue authors | the owner only (`Zelenov`) |
| Repository created | 2026-01-23 |

The one star says nothing about a language, and this design does not look further into who
gave it.

**The evidence is too thin to say anything about users' languages.** Three downloads cannot be
told apart from the owner's own. The one signal that is solid: the owner uses the app and their
language is Russian. The code agrees: `crates/frename-core/src/transliteration.rs:1-6` exists
because the owner types tag searches with the Russian keyboard layout while the tags are Latin.
So: English stays the source, Russian is the only target now, and no third language is supported
by the research (open question 5).

## What the code has today

About **75 translatable strings** (a hand count of `text(…)`, labels, tooltips, placeholders and
`format!` sentences; icons, key names and format names excluded):

| Where | Strings | Notes |
|---|---|---|
| `src/features/settings/view.rs` | ~19 | section titles, check boxes, radios, move offers, hints |
| `src/features/batch/actions/*.rs` | ~18 | 5 action labels (`LABEL` consts, e.g. `move_comments.rs:12`), hints, radios |
| `src/features/batch/view.rs` | ~12 | heading, `Run on {files}`, counts line, `Stopped after …`, `Finished …` |
| `src/features/folder_controls/view.rs` | ~9 | filter names and `Filter ({n})` (`:31-37`, `:90-94`), tooltips |
| `src/features/folder/view.rs` | ~6 | `All`, `Invert`, `{n} checked` (`:229`), outcome tooltips (`:255-257`) |
| `src/features/folder_workspace/state.rs:1760-1774` | 4 | inline rename errors, stored as `&'static str` in `InlineRename::error` (`src/features/folder/mod.rs:27`) |
| video controls, subtitle toggle, comment box, window title | ~7 | `video_controls/view.rs:82,98`, `media_viewer/video/view.rs:199-201`, `file_workspace/view.rs:77`, `app/state.rs:479` |

Most of the main window is icons (`📂`, `⚙`, `☑`, `⏪`, `🔊`) whose tooltips are key names
(`Page Up`, `Space`, `F12`, `Esc`, `Enter`, `Delete`). The empty start screen is an icon only
(`src/features/folder_workspace/view.rs:28-30`). The text lives in Settings and batch mode.

**Core returns no user-facing text.** Batch results are the enum `MoveOutcome`, mapped to
`ItemStatus` in the UI (`src/features/batch/actions/mod.rs:161-176`); failures say "see the log".
Core errors (`UndoError`, `XmpWriteError`, tag creation) go to the log only
(`folder_workspace/state.rs:1084`, `:1401`). The one core string the user sees is
`DEFAULT_COMMENTED_TAG = "Commented"` (`crates/frename-core/src/metadata/mod.rs:116`), which is
data (see "What is not translated").

**Fonts.** frename sets no font; iced 0.14 is built without its `fira-sans` feature
(`Cargo.toml` enables only `tokio`, `image`), so text uses system fonts through cosmic-text 0.15.
Its sans-serif family is "Open Sans" (`cosmic-text-0.15.0/src/font/system.rs:159`), with "Segoe UI"
as the Windows fallback (`src/font/fallback/windows.rs:33`); both cover Cyrillic, and Cyrillic file
names and tags already render in the file list today. The screenshot script already requires
Open Sans on Linux (`docs/screenshots/render.sh:7`).

**Settings** are one SQLite row, `app_settings`, one column per field, added by migrations
(`crates/frename-core/src/db/schema.rs:59-92`), read into `AppSettings`
(`crates/frename-core/src/db/traits.rs:41-68`). The settings window is fixed at 560×560 and not
resizable (`src/app/state.rs:197`, `:414`), and its body is a plain column with no
scrollable (`src/features/settings/view.rs:109`).

## Options

Versions are current on crates.io on 2026-09-26. "New crates" counts crates of
`cargo tree -e normal` that frename's `Cargo.lock` does not already have. Size is the growth of
a minimal release binary that formats one Russian plural message, over an empty `main`
(436 KB); frename's release binary is ~42 MB.

| | rust-i18n 4.2.2 | fluent-bundle 0.16 + sys-locale | i18n-embed 0.16 + i18n-embed-fl 0.10.1 | hand-rolled enum/match |
|---|---|---|---|---|
| Resource files | YAML/TOML/JSON, compiled in | `.ftl`, loaded by our code | `.ftl`, embedded with rust-embed | Rust source |
| Typo in a key | compiles; shows the key at run time | compiles; our lookup returns `None` | **compile error** (`fl!` checks the English file) | compile error |
| Wrong argument name | compiles | compiles | **compile error** | compile error |
| Key missing in `ru` | falls back to `en`; test needed | test needed | falls back to `en`; test needed | compile error if every language is a `match` |
| Russian plurals (one/few/many) | not in the README: hand-written | CLDR rules built in (`intl_pluralrules`) | same, via fluent | hand-written |
| Switch at run time | `set_locale` (global) | swap our bundle | `load_languages` (global, `ArcSwap`) | swap a global enum |
| New crates / size | 25 / +78 KB | 8 / +393 KB | 33 / +494 KB | 0 / 0 |
| Translator edits | YAML | `.ftl` | `.ftl` | Rust |

Sources: rust-i18n README (https://github.com/longbridge/rust-i18n, v4.2.2: "`t!` macro",
"chain of fallback locales", `log-miss-tr` "log missing translations at the warning level", no
plural feature); fluent-bundle (https://docs.rs/fluent-bundle/0.16.0,
`FluentBundle::set_use_isolating`); Fluent selectors and plural categories
(https://projectfluent.org/fluent/guide/selectors.html); i18n-embed
(https://github.com/kellpossible/cargo-i18n/tree/master/i18n-embed, features `fluent-system`,
`desktop-requester` "makes use of the sys-locale crate"); i18n-embed-fl
(https://docs.rs/i18n-embed-fl/0.10.1: "compile time check for message id, and the `name`
argument … in the `fallback_language`'s fluent resource file"); CLDR Russian plural rules
(https://www.unicode.org/cldr/charts/latest/supplemental/language_plural_rules.html); sys-locale
0.3.2 (https://github.com/1Password/sys-locale). Checked by building the minimal examples: a
misspelled id and a misspelled argument fail the build with i18n-embed-fl; a key present only in
`ru` does not; `ru` formats 1/2/5/21 as `файл/файла/файлов/файл`.

Also relevant: sys-locale 0.3.2 is already in `Cargo.lock` (cosmic-text depends on it). On
Windows it reads `GetUserPreferredUILanguages` (the display language, not the regional format);
on Linux `LANGUAGE`, `LC_ALL`, `LC_MESSAGES`, `LANG` (`sys-locale-0.3.2/src/windows.rs:5`,
`src/unix.rs:30`). COSMIC, the largest iced-based desktop, localizes its apps with i18n-embed and
`fl!` (e.g. https://github.com/pop-os/cosmic-files, `Cargo.toml`, `src/localize.rs`).

## Recommendation

**i18n-embed with i18n-embed-fl (Fluent).** In an unattended pipeline the most likely mistake is
a key or argument typo in a view; `fl!` turns it into a build error before review. Fluent lets the
Russian file choose its own plural forms per sentence, which a helper like `files()` cannot: "Run on
14 files" needs the genitive (`для 1 файла`, `для 5 файлов`), "14 checked" another construction. The
remaining gap (a key missing from `ru`, a key no longer used) is closed by two tests. The cost,
33 small crates and ~0.5 MB in a 42 MB binary, is small next to iced and GStreamer.

rust-i18n is lighter but has no plurals and catches nothing at compile time. Bare fluent-bundle
saves 25 crates but loses the compile-time check, which is the main reason to prefer a library over
a hand-rolled table. A hand-rolled `match` is fully checked and dependency-free, but every
translation becomes Rust code with hand-written plural rules for each language.

## User flows

1. **First start on a Russian Windows.** No language is stored. At start-up (`FrenameApp::new`,
   next to the other settings applied there, `src/app/state.rs:229-231`) frename reads the OS
   display languages and takes the first one it supports: `ru-RU` → Russian. The whole UI, window
   titles included, is Russian. Nothing is written to the database.
2. **Switching in Settings.** Settings → Language → pick `English`. The choice is saved at once,
   like every setting, and both windows redraw in English on the next frame; no restart, the open
   file and video are untouched. Picking `System` goes back to following the OS.
3. **Fallback.** OS language unsupported (e.g. `de-DE`, `uk-UA`) or unreadable → English. A stored
   language this build does not know (after a downgrade) → treated as `System`.

## UI sketch

A new first section in the settings window:

```
┌ Settings ────────────────────────────────────────────┐
│ Language                                             │
│   [ System (English)          ▾ ]                    │
│                                                      │
│ Video                                                │
│   [x] Play videos automatically when opened          │
│ Tags                                                 │
│   ...                                                │
└──────────────────────────────────────────────────────┘

open list:   ✓ System (English)
               English
               Русский
```

- A `pick_list`, width fixed at 220 px. `System` is translated and shows the language it resolves
  to in brackets; language names are always written in their own language, so a user stuck in a
  language they cannot read still finds theirs.
- The section title is translated like the others; the list sits under it, as the other sections'
  controls do.

## Keyboard shortcuts

None new. Key names in tooltips and the README stay as printed on the keys (`Page Up`, `F12`).

## Data format

```
i18n.toml                  fallback_language = "en", [fluent] assets_dir = "i18n"
i18n/en/frename.ftl        source strings, the reference for keys and arguments
i18n/ru/frename.ftl        same keys, same order
src/i18n.rs                loader, fl! wrapper, language resolution
```

- **Keys:** `<feature>-<element>[-<detail>]`, kebab-case, the feature being the folder under
  `src/features/`: `settings-video-autoplay`, `batch-run`, `folder-rename-error-exists`,
  `folder-controls-filter-active`. Batch actions: `batch-action-move-comments`,
  `batch-action-move-comments-hint`. Every `fl!` call takes a string literal (needed for the
  compile-time check and the unused-key test).
- **Arguments** are named for what they hold (`$count`, `$tag`, `$done`), never positional.
- **Plurals** are selectors in the message, per language:

```ftl
# en
batch-run = Run on { $count ->
    [one] { $count } file
   *[other] { $count } files
}
# ru
batch-run = Запустить для { $count ->
    [one] { $count } файла
    [few] { $count } файлов
   *[many] { $count } файлов
}
```

- **Setting:** migration 8, `ALTER TABLE app_settings ADD COLUMN language TEXT NOT NULL DEFAULT ''`;
  `AppSettings::language: String`, `""` meaning System. Core stores the code and knows nothing
  about translation; `src/i18n.rs` owns the supported list (`en`, `ru`).
- **Runtime:** one global `FluentLanguageLoader` in `src/i18n.rs`, set only from `update`
  (`Message::Settings(SetLanguage)` in `src/app/state.rs:269`, the way `set_commented_tag` is) and
  read in `view`. This follows the existing global settings in core
  (`crates/frename-core/src/metadata/mod.rs:118`) and avoids threading a translator through every
  view. `set_use_isolating(false)` is applied after every load: the Unicode isolation marks Fluent
  puts around arguments are only needed for right-to-left text, and whether iced draws them as
  invisible is not verified.
- **State holds meaning, views hold words.** `InlineRename::error` becomes an enum
  (`RenameProblem::{Empty, BadCharacter, TrailingDotOrSpace, Exists}`), batch `LABEL` consts
  become `fn label() -> String`, and `move_offer`/`section` take `String` instead of
  `&'static str` (`src/features/settings/view.rs:156-168`), so a language switch redraws
  everything already on screen.

## What is translated and what is not

Translated: every label, tooltip that is a sentence, placeholder, hint, error and summary in
both windows, and the settings window title.

Not translated:
- **Tag names and file names**, including file-name tokens `in_HH_MM_SS` / `out_HH_MM_SS`,
  `.comment.txt`, `.snap.` — they are data on disk; the Settings radio text that shows the tokens
  keeps them verbatim inside the translated sentence.
- **The "Commented" default** (`DEFAULT_COMMENTED_TAG`). It is written into file names and matched
  when a comment is cleared; a Russian default would rename files differently depending on the UI
  language, and switching language would orphan the old tag. It stays `Commented`; users rename it
  in Settings (open question 2).
- **Key names** (`Space`, `Page Up`, `Esc`), badges `IN`/`OUT`, `CC`, `SRT`, `XMP`, product names.
- **Log messages**, command-line errors (`--demo`, `--self-test`), `GSTREAMER_SETUP.md` text.
- **README, version.md, docs, code, commits**: English only (`AGENTS.md`). The README gains one
  sentence: the UI follows the system language, English or Russian, changeable in Settings.

## Edge cases

- **Missing key in `ru`:** Fluent falls back to English for that message at run time; the key
  test fails in CI first.
- **Unsupported OS locale:** English. Only the language subtag is matched (`ru-RU`, `ru-KZ` → ru);
  other languages are never mapped to Russian (`uk`, `be` → English).
- **Long strings:** text in iced wraps by default, so a longer label grows a row rather than
  overflowing. Places with fixed widths to check: batch action list 190 px
  (`batch/view.rs:16`), commented-tag input row (`settings/view.rs:121-143`), the rename error next
  to the input in the file list (`folder/view.rs:343`), and the fixed 560 px settings height, which
  has no scrollable (open question 6).
- **Plurals:** the Russian file uses `one`/`few`/`many` with `many` as default; numbers are
  integers, so `other` never occurs. Sentences with counts use a colon form where it reads better
  (`Отмечено: 14`) to avoid agreement.
- **Strings from core:** none today. Rule going forward: core returns enums or data, the UI turns
  them into words; a core `Display` text is for logs.
- **A string built from pieces** (`FilterItem`'s `Display`, `folder_controls/view.rs:56-60`,
  the counts line `batch/view.rs:115-118`): one message with arguments per sentence, never
  concatenated translations. The existing hint at `settings/view.rs:130` contains a run of spaces
  from a broken line continuation; the English `.ftl` text has one space.

## Migration plan

Each PR goes through the full gate. Russian becomes selectable only in the last one, so no
release ships a half-translated UI.

1. **Infrastructure + Settings** (`Refs #18`): `i18n.toml`, `i18n/{en,ru}/frename.ftl`,
   `src/i18n.rs`, the `language` column and migration, OS language resolution (until PR 3
   Russian is neither taken from the OS nor listed in Settings; only a stored `ru`, which just
   the demo `--lang` writes, turns it on), Settings window strings through
   `fl!`, the two key tests, and demo flags `--lang <code>` and `--settings` (see Test plan).
   No `version.md` change.
2. **Batch and file list** (`Refs #18`): `batch/view.rs`, `batch/actions/*`, the batch header
   and outcome tooltips in `folder/view.rs`, `files()` removed. No `version.md` change.
3. **The rest and switch-on** (`Closes #18`): folder controls, rename errors, video controls,
   subtitle toggle, comment placeholder, window titles; `ru` added to the offered list; the
   Language row in Settings; README sentence and `version.md`.

## Test plan

Unit tests in the UI crate (fluent-syntax is already a transitive dependency; add it as a
dev-dependency to parse files):
- **Key parity:** every `.ftl` under `i18n/` parses without errors and has exactly the English
  message ids and attributes, and each message uses the same `$arguments` as in English. Fails
  naming the file and key.
- **No unused keys:** collect `fl!("…")` ids from `src/**/*.rs` with a regex; the set equals the
  English ids.
- **Russian plurals:** every Russian message that selects on a number has `one`, `few`, `many`.
- **Resolution:** `resolve(stored, os_languages)` — `""` + `["ru-RU"]` → ru; `""` + `["uk-UA",
  "ru-RU"]` → ru (second preference); `""` + `["de-DE"]` → en; `"xx"` stored → System; `"en"`
  stored overrides a Russian OS.
- **Formatting:** `batch-run` for 1, 2, 5, 11, 21 in `ru`; no U+2068/U+2069 in any output.
- **Settings migration:** a version-7 database gains `language = ''`; round-trip of `AppSettings`.

Layout check (PR 1 adds, every PR runs it under Xvfb and gives the screenshots to the product
reviewer): `frename --demo docs/screenshots/main.toml --out ru.png --lang ru`, the same with
`--batch`, and with `--settings` (a screenshot of the settings window instead of the main one;
demo mode today only captures the main window, `src/demo.rs`). `--lang` sets the seeded
`AppSettings::language`, so the Russian UI takes the same path as a user's choice. Look for
wrapped action labels, clipped buttons and settings content past 560 px. Running this in CI needs
a change to `.github/workflows/screenshots.yml`, a guarded file (open question 7).

By hand, by the owner: Russian Windows first start, switch in Settings, wording of the Russian text.

## Out of scope

- Languages other than English and Russian; right-to-left layout.
- Translating the README, release notes, log files or command-line messages.
- Localized number, date and timecode formats (timecodes stay `HH:MM:SS`).
- Loading translation files from disk at run time or community translation tooling.
- Changing the `Commented` tag of existing files.

## Open questions (with recommended answers)

1. **Library.** *Recommended:* i18n-embed + i18n-embed-fl, for the compile-time key check.
2. **Should the Commented default follow the UI language on a fresh install?** *Recommended:*
   no; it is written into file names, so it stays `Commented` everywhere.
3. **Translate key names in tooltips (`Space` → `Пробел`)?** *Recommended:* no; they match the
   keycaps and the README tables.
4. **Translate the `IN`/`OUT` badges?** *Recommended:* no; they mirror `in_`/`out_` in file names.
5. **A third language?** *Recommended:* none until a user asks in an issue; the evidence shows no
   users besides the owner.
6. **Settings overflow in Russian.** *Recommended:* wrap the settings body in a `scrollable` and
   keep 560×560, if the PR 1 screenshot shows it does not fit.
7. **Russian screenshots in CI.** *Recommended:* the owner allows one step in
   `screenshots.yml` that renders `--lang ru` as a build artifact (never published to the README);
   until then the agent runs it locally for each PR.
8. **Global loader vs. a translator passed to every view.** *Recommended:* global, set only in
   `update`, as core already does for the commented tag and storage settings.
