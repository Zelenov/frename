# Localization (#18)

Working notes for the agent, not a gate: the owner's decision (issue #18) is to ship as many
languages as are ready, agent translations included, no owner check before release.

## Scope

- **Source language:** English, `i18n/en/frename.ftl`.
- **Owner's language list:** the languages Premiere Pro's own UI ships in (English, Russian,
  German, French, Spanish, Italian, Portuguese (Brazil), Japanese, Korean, Chinese Simplified,
  Chinese Traditional), plus Serbian (`sr-Cyrl` and `sr-Latn`) and Ukrainian if cheap.
- **This PR ships:** English (source) and Russian. The infrastructure and the full string
  extraction were the hard, correctness-critical part (every literal in `src/features/*/view.rs`,
  `src/widgets/*.rs`, `src/features/batch/actions/*.rs` and `src/features/folder_controls/*.rs`
  moved to `i18n/en/frename.ftl`), so they get the deepest review round; each further language is
  now just one `.ftl` file plus one entry in `crate::i18n::LANGUAGES`, and ships as its own small
  PR once translated, per "ship each language as soon as it is complete".
- **CJK deferred:** Japanese, Korean and both Chinese scripts wait for a bundled font that covers
  them (issue #57); the fixed-width UI font today falls back to boxes for CJK glyphs.

## Architecture

- **i18n-embed + Fluent** (`i18n-embed`, `i18n-embed-fl`, `rust-embed`, `fluent-syntax` /
  `fluent-bundle` in dev-dependencies): `i18n/<lang>/frename.ftl` files are embedded in the binary;
  `fl!("key-id", arg = value)` (in `src/i18n.rs`) checks the key and its Fluent arguments at
  compile time against the fallback (English) file.
- **One global loader** (`crate::i18n::loader()`), switched by `crate::i18n::apply(stored)` from
  `FrenameApp::new` and on `settings::Message::SetUiLanguage`. `resolve()` picks the stored
  setting, then the OS's preferred languages, then English.
- **Settings → Language**: a pick list, `System (<name>)` first, then every supported language in
  its own name. Saved as `AppSettings.ui_language` (empty = System), migration 14
  (`ui_language TEXT NOT NULL DEFAULT ''`).
- **Demo mode**: `--lang <code>` (default `en`, so README screenshots never follow the renderer's
  OS language).
- **State holds meaning, views hold words**: places that used to hand a `&'static str` around (a
  rename error, a batch action's label, a helper like `small_button`/`link_button`) now hand
  around either an enum (`folder::RenameProblem`) or an owned `String` built from `fl!`, so a
  language switch redraws everything without any state recomputation.

## Tests (`src/i18n.rs`)

- Key/attribute/argument parity between every language and English.
- No unused message (every key is read by some `fl!` call) and no message missing (every `fl!`
  call has a key).
- Every language's Fluent `select` variants match exactly its own CLDR plural categories (a small
  per-language table in `plural_categories()`; two-category languages need no entry — `["one",
  "other"]` is the default). Russian's actual category boundaries (1/21 → one, 2/22 → few, 5/11/25
  → many) are checked against `fluent_bundle` directly.
- A lexer-based scan (`proc_macro2`) of every `view.rs`, `src/widgets/*.rs`,
  `src/features/batch/actions/*.rs` and `src/features/folder_controls/*.rs` file fails the build
  on a Latin word left outside `fl!` and outside `NOT_UI_TEXT` (key names printed on a physical
  key, badges that mirror file-name tokens such as `IN`/`OUT`, widget ids, log messages, API
  protocol constants — each with its reason). It skips `fl!`'s own id argument, `cfg!(...)`,
  `log::{warn,info,error,debug,trace}!(...)` and `#[cfg(test)]` items.
- Isolation marks: `set_use_isolating(false)` (needed again after every `load_languages`, which
  builds fresh bundles) so `{ $count }` never comes back wrapped in U+2068/U+2069.
- A review sheet per language (`target/<language>-messages.txt`): every message rendered with
  sample numbers, for the reviewer and the owner to read real sentences.

## Glossary (Russian)

Kept at the top of `i18n/ru/frename.ftl`: тег (never «метка», that is Premiere's Label), точки
входа/выхода, subclip / «подклип», Description / «Описание», Все/Обратить, System → «Как в
системе», Cancel/Stopping… → Отмена/Остановка…, batch action → пакетное действие. Each further
language keeps its own glossary the same way, at the top of its own file.

## Decisions made without the owner

- **Naming**: the new AI/Subtitles features (added since the original English+Russian design)
  needed many new keys; named them `batch-ai-*` / `batch-subtitles-*` / `batch-action-describe-ai-*`
  / `batch-action-generate-subtitles-*`, matching the existing `batch-action-<action>-*` pattern.
- **`RenameProblem` enum**: `folder::check_new_file_name` used to return `Result<(), &'static str>`
  with the English text baked in; it now returns `Result<(), RenameProblem>` and the view converts
  it to text with `fl!`, so the four `folder-rename-error-*` keys (already in the reviewed design)
  needed no wording change.
- **Debug-only field names** (`SubtitleJob`'s hand-written `Debug` impl) use `stringify!(field)`
  instead of a string literal, so the literal scan does not have to except ordinary English words
  like "languages" or "force" project-wide.
- **Money/duration formatting**: the number itself stays `format!("{x:.2}")` (Fluent's `NUMBER()`
  builtin was not worth the complexity here); only the surrounding words ("min", "h", "about",
  "under") are `fl!` keys.
- **"AI" badge**: the marker color named "AI" (`src/features/markers/view.rs`) is a badge like
  `IN`/`OUT`, not translated (`NOT_UI_TEXT`); the Settings section titled "AI" is a real heading
  and is a translated key (`settings-ai`), even though both render the same English text today.

## Review notes not taken

(none yet — first review round is still ahead of this PR)
