//! UI translations: the Fluent files in `i18n/<lang>/frename.ftl`, embedded in the binary, and
//! the language they are shown in.
//!
//! One global loader, switched only from `update` ([`apply`]) and read by the views through
//! [`fl!`](crate::fl): the same pattern as the settings core keeps globally. English is the
//! source language and the fallback for any message a translation lacks.
//!
//! `LANGUAGES` is deliberately short today (English and Russian): each further language is its
//! own small PR that adds one `.ftl` file and one entry here, reviewed on its own. CJK languages
//! wait for a bundled font that covers them (issue #57).

use std::sync::{Arc, Mutex, OnceLock, RwLock};

use i18n_embed::fluent::{fluent_language_loader, FluentLanguageLoader};
use i18n_embed::unic_langid::LanguageIdentifier;
use i18n_embed::{DesktopLanguageRequester, LanguageLoader};
use rust_embed::RustEmbed;

/// The embedded `i18n/` folder.
#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

/// Language codes the UI is translated into, in the order Settings lists them.
pub const LANGUAGES: [&str; 2] = ["en", "ru"];

/// The source language, used when nothing else matches.
const FALLBACK: &str = "en";

/// Every language, loaded once with isolation off, and the one being shown.
///
/// Switching never builds bundles: it only selects among the loaded ones into a new small
/// loader and swaps that in. A freshly built bundle isolates arguments (Fluent's default) until
/// `set_use_isolating(false)` reaches it, so building bundles while other threads read text
/// (as tests do, and the UI does after a language switch) would show isolation marks.
struct Languages {
    all: FluentLanguageLoader,
    current: RwLock<Arc<FluentLanguageLoader>>,
}

impl Languages {
    fn new() -> Self {
        let all: FluentLanguageLoader = fluent_language_loader!();
        let ids: Vec<LanguageIdentifier> =
            LANGUAGES.iter().filter_map(|l| l.parse().ok()).collect();
        if let Err(e) = all.load_languages(&Localizations, &ids) {
            log::error!("cannot load the UI text: {e}");
        }
        // Before anything else can see the bundles: they are not published anywhere yet.
        all.set_use_isolating(false);
        let current = Arc::new(Self::select(&all, FALLBACK));
        Self {
            all,
            current: RwLock::new(current),
        }
    }

    fn select(all: &FluentLanguageLoader, language: &str) -> FluentLanguageLoader {
        let id: LanguageIdentifier = language
            .parse()
            .unwrap_or_else(|_| FALLBACK.parse().expect("the fallback is a valid code"));
        all.select_languages(&[id])
    }

    fn show(&self, language: &str) {
        let next = Arc::new(Self::select(&self.all, language));
        match self.current.write() {
            Ok(mut current) => *current = next,
            Err(poisoned) => *poisoned.into_inner() = next,
        }
    }

    fn shown(&self) -> Arc<FluentLanguageLoader> {
        match self.current.read() {
            Ok(current) => current.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }
}

fn languages() -> &'static Languages {
    static LANGUAGES_LOADED: OnceLock<Languages> = OnceLock::new();
    LANGUAGES_LOADED.get_or_init(Languages::new)
}

/// The loader every [`fl!`](crate::fl) reads: the language being shown.
pub fn loader() -> Arc<FluentLanguageLoader> {
    languages().shown()
}

/// The language `System` stands for: the OS language as read at start-up or when `System` was
/// last picked.
static SYSTEM_LANGUAGE: Mutex<&str> = Mutex::new(FALLBACK);

/// A message in the current UI language, checked against `i18n/en/frename.ftl` at compile time:
/// `fl!("batch-run-rename", count = 5)`.
#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        {
            let loader = $crate::i18n::loader();
            i18n_embed_fl::fl!(loader, $message_id)
        }
    }};
    ($message_id:literal, $($args:expr),* $(,)?) => {{
        {
            let loader = $crate::i18n::loader();
            i18n_embed_fl::fl!(loader, $message_id, $($args),*)
        }
    }};
}

/// Show the UI in the language the `stored` setting asks for (empty: the OS language). Reads
/// the OS languages, so call it at start-up and when the setting changes, not per frame.
pub fn apply(stored: &str) {
    let os = os_languages();
    let system = resolve("", &os);
    if let Ok(mut current) = SYSTEM_LANGUAGE.lock() {
        *current = system;
    }
    languages().show(resolve(stored, &os));
}

/// The OS's preferred languages. Tests see none: they read English, and a machine whose
/// language is Russian must not switch the global loader under their assertions.
fn os_languages() -> Vec<String> {
    if cfg!(test) {
        return Vec::new();
    }
    DesktopLanguageRequester::requested_languages()
        .iter()
        .map(|id| id.to_string())
        .collect()
}

/// The language `System` currently stands for.
pub fn system_language() -> &'static str {
    SYSTEM_LANGUAGE.lock().map(|l| *l).unwrap_or(FALLBACK)
}

/// The UI language for the `stored` setting and the OS's preferred languages (BCP 47, most
/// preferred first): a stored language this build knows wins; otherwise the first OS language
/// whose language subtag is translated; otherwise English.
pub fn resolve(stored: &str, os_languages: &[String]) -> &'static str {
    if let Some(language) = supported(stored) {
        return language;
    }
    os_languages
        .iter()
        .find_map(|tag| supported(tag.split(['-', '_']).next().unwrap_or_default()))
        .unwrap_or(FALLBACK)
}

/// `code` as one of [`LANGUAGES`], ignoring case.
fn supported(code: &str) -> Option<&'static str> {
    LANGUAGES
        .iter()
        .find(|l| l.eq_ignore_ascii_case(code))
        .copied()
}

/// The name of `language` in that language, as Settings lists it.
pub fn language_name(language: &str) -> String {
    match language {
        "ru" => crate::fl!("language-name-ru"),
        _ => crate::fl!("language-name-en"),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};

    use fluent_syntax::ast;

    use super::*;

    fn os(tags: &[&str]) -> Vec<String> {
        tags.iter().map(|t| t.to_string()).collect()
    }

    #[test]
    fn the_language_follows_the_setting_then_the_os_then_english() {
        assert_eq!(resolve("", &os(&["ru-RU"])), "ru");
        assert_eq!(
            resolve("", &os(&["uk-UA", "ru-RU"])),
            "ru",
            "second preference"
        );
        assert_eq!(resolve("", &os(&["de-DE"])), "en");
        assert_eq!(
            resolve("", &os(&["uk-UA"])),
            "en",
            "never mapped to Russian"
        );
        assert_eq!(resolve("", &os(&[])), "en");
        assert_eq!(
            resolve("xx", &os(&["ru-RU"])),
            "ru",
            "unknown stored = System"
        );
        assert_eq!(resolve("en", &os(&["ru-RU"])), "en", "the setting wins");
        assert_eq!(resolve("ru", &os(&["en-US"])), "ru");
    }

    fn i18n_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("i18n")
    }

    fn parse(path: &Path) -> ast::Resource<String> {
        let text = std::fs::read_to_string(path).unwrap();
        fluent_syntax::parser::parse(text)
            .unwrap_or_else(|(_, errors)| panic!("{}: {errors:?}", path.display()))
    }

    /// Every message of a file: id -> (attribute names, `$arguments` used).
    fn messages(
        resource: &ast::Resource<String>,
    ) -> BTreeMap<String, (Vec<String>, BTreeSet<String>)> {
        let mut out = BTreeMap::new();
        for entry in &resource.body {
            if let ast::Entry::Message(message) = entry {
                let mut arguments = BTreeSet::new();
                if let Some(value) = &message.value {
                    pattern_arguments(value, &mut arguments);
                }
                let attributes = message
                    .attributes
                    .iter()
                    .map(|a| a.id.name.clone())
                    .collect();
                for attribute in &message.attributes {
                    pattern_arguments(&attribute.value, &mut arguments);
                }
                out.insert(message.id.name.clone(), (attributes, arguments));
            }
        }
        out
    }

    fn pattern_arguments(pattern: &ast::Pattern<String>, out: &mut BTreeSet<String>) {
        for element in &pattern.elements {
            if let ast::PatternElement::Placeable { expression } = element {
                expression_arguments(expression, out);
            }
        }
    }

    fn expression_arguments(expression: &ast::Expression<String>, out: &mut BTreeSet<String>) {
        match expression {
            ast::Expression::Select { selector, variants } => {
                inline_arguments(selector, out);
                for variant in variants {
                    pattern_arguments(&variant.value, out);
                }
            }
            ast::Expression::Inline(inline) => inline_arguments(inline, out),
        }
    }

    fn inline_arguments(inline: &ast::InlineExpression<String>, out: &mut BTreeSet<String>) {
        match inline {
            ast::InlineExpression::VariableReference { id } => {
                out.insert(id.name.clone());
            }
            ast::InlineExpression::Placeable { expression } => {
                expression_arguments(expression, out)
            }
            _ => {}
        }
    }

    /// The variant keys of each select expression in a message, in order.
    fn selects(resource: &ast::Resource<String>) -> BTreeMap<String, Vec<BTreeSet<String>>> {
        fn walk(pattern: &ast::Pattern<String>, out: &mut Vec<BTreeSet<String>>) {
            for element in &pattern.elements {
                if let ast::PatternElement::Placeable {
                    expression: ast::Expression::Select { variants, .. },
                } = element
                {
                    out.push(
                        variants
                            .iter()
                            .map(|v| match &v.key {
                                ast::VariantKey::Identifier { name } => name.clone(),
                                ast::VariantKey::NumberLiteral { value } => value.clone(),
                            })
                            .collect(),
                    );
                    for variant in variants {
                        walk(&variant.value, out);
                    }
                }
            }
        }
        let mut out = BTreeMap::new();
        for entry in &resource.body {
            if let ast::Entry::Message(message) = entry {
                let mut found = Vec::new();
                if let Some(value) = &message.value {
                    walk(value, &mut found);
                }
                out.insert(message.id.name.clone(), found);
            }
        }
        out
    }

    fn translations() -> Vec<(String, PathBuf)> {
        LANGUAGES
            .iter()
            .filter(|l| **l != FALLBACK)
            .map(|l| (l.to_string(), i18n_dir().join(l).join("frename.ftl")))
            .collect()
    }

    #[test]
    fn every_translation_has_exactly_the_english_messages_and_arguments() {
        let english = messages(&parse(&i18n_dir().join("en/frename.ftl")));
        for (language, path) in translations() {
            let translated = messages(&parse(&path));
            for (id, (attributes, arguments)) in &english {
                let Some((t_attributes, t_arguments)) = translated.get(id) else {
                    panic!("{language}: {id} is missing");
                };
                assert_eq!(attributes, t_attributes, "{language}: attributes of {id}");
                assert_eq!(arguments, t_arguments, "{language}: arguments of {id}");
            }
            for id in translated.keys() {
                assert!(
                    english.contains_key(id),
                    "{language}: {id} is not in English"
                );
            }
        }
        // Settings lists every language, and its name is the same in every file.
        for language in LANGUAGES {
            assert!(english.contains_key(&format!("language-name-{language}")));
        }
    }

    /// The plural categories each language's Fluent selects are expected to use (CLDR), with the
    /// last one the Fluent default (`*[..]`). Two-category languages (most of Western Europe)
    /// need no entry: they fall back to `["one", "other"]` below.
    fn plural_categories(language: &str) -> &'static [&'static str] {
        match language {
            "ru" => &["one", "few", "many"],
            _ => &["one", "other"],
        }
    }

    #[test]
    fn plurals_use_exactly_their_languages_cldr_categories() {
        let cldr = ["zero", "one", "two", "few", "many", "other"];
        let english = selects(&parse(&i18n_dir().join("en/frename.ftl")));
        for (language, path) in translations() {
            let categories = plural_categories(&language);
            let translated = selects(&parse(&path));
            for (id, english_selects) in &english {
                let plural = english_selects
                    .iter()
                    .any(|keys| keys.iter().any(|k| cldr.contains(&k.as_str())));
                if !plural {
                    continue;
                }
                let translated_selects = &translated[id];
                assert!(
                    !translated_selects.is_empty(),
                    "{language}: {id} lost its plural"
                );
                for keys in translated_selects {
                    for category in categories.iter() {
                        assert!(
                            keys.contains(*category),
                            "{language}: {id} has no [{category}]"
                        );
                    }
                    for key in keys {
                        assert!(
                            categories.contains(&key.as_str()),
                            "{language}: {id} has [{key}], not a {language} CLDR category"
                        );
                    }
                }
            }
        }
    }

    /// Every `fl!("…")` id in `src/`, found by scanning the text.
    fn used_ids() -> BTreeSet<String> {
        fn scan(dir: &Path, out: &mut BTreeSet<String>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    scan(&path, out);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    let text = std::fs::read_to_string(&path).unwrap();
                    // Comments (the macro's own example) are not uses.
                    let code: String = text
                        .lines()
                        .filter(|l| !l.trim_start().starts_with("//"))
                        .collect::<Vec<_>>()
                        .join("\n");
                    for (at, _) in code.match_indices("fl!(") {
                        let rest = code[at + 4..].trim_start();
                        if let Some(rest) = rest.strip_prefix('"') {
                            if let Some(end) = rest.find('"') {
                                out.insert(rest[..end].to_string());
                            }
                        }
                    }
                }
            }
        }
        let mut out = BTreeSet::new();
        scan(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut out);
        out
    }

    #[test]
    fn every_message_is_used() {
        let english: BTreeSet<String> = messages(&parse(&i18n_dir().join("en/frename.ftl")))
            .into_keys()
            .collect();
        let used = used_ids();
        let unused: Vec<_> = english.difference(&used).collect();
        assert!(unused.is_empty(), "unused messages: {unused:?}");
    }

    /// Switching the language while other threads read text must never show isolation marks.
    /// Uses its own `Languages`, so the global language other tests read stays English.
    #[test]
    fn switching_the_language_never_shows_isolation_marks() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let languages = Languages::new();
        let done = AtomicBool::new(false);
        std::thread::scope(|scope| {
            scope.spawn(|| {
                for i in 0..2000 {
                    languages.show(if i % 2 == 0 { "ru" } else { "en" });
                }
                done.store(true, Ordering::SeqCst);
            });
            while !done.load(Ordering::SeqCst) {
                let loader = languages.shown();
                let text = i18n_embed_fl::fl!(loader, "batch-run-rename", count = 5);
                assert!(!text.contains('\u{2068}'), "isolation marks in {text:?}");
            }
        });
    }

    #[test]
    fn showing_a_language_reads_its_text() {
        let languages = Languages::new();
        assert!(languages
            .shown()
            .get("batch-run-rename")
            .starts_with("Rename"));
        languages.show("ru");
        assert!(languages
            .shown()
            .get("batch-run-rename")
            .starts_with("Переименовать"));
        languages.show("en");
        assert!(languages
            .shown()
            .get("batch-run-rename")
            .starts_with("Rename"));
    }

    #[test]
    fn russian_plural_categories_follow_cldr() {
        use fluent_bundle::{FluentArgs, FluentBundle, FluentResource};
        let resource = FluentResource::try_new(
            "n = { $n ->\n    [one] one\n    [few] few\n   *[many] many\n}\n".to_string(),
        )
        .unwrap();
        let mut bundle = FluentBundle::new(vec!["ru".parse().unwrap()]);
        bundle.add_resource(resource).unwrap();
        let category = |n: i64| {
            let mut args = FluentArgs::new();
            args.set("n", n);
            let message = bundle.get_message("n").unwrap();
            let mut errors = vec![];
            bundle
                .format_pattern(message.value().unwrap(), Some(&args), &mut errors)
                .to_string()
        };
        for (n, expected) in [
            (1, "one"),
            (21, "one"),
            (2, "few"),
            (22, "few"),
            (5, "many"),
            (11, "many"),
            (25, "many"),
        ] {
            assert_eq!(category(n), expected, "{n}");
        }
    }

    /// String literals in UI code that may hold Latin words, each with why it is not UI text.
    const NOT_UI_TEXT: [(&str, &str); 75] = [
        ("comment-editor", "widget id"),
        (
            "Rotate videos",
            "English-only log id and log label, see Action::log_id",
        ),
        ("PgUp", "key name, as printed on the key"),
        ("PgDn", "key name, as printed on the key"),
        ("Space", "key name, as printed on the key"),
        ("Enter", "key name, as printed on the key"),
        ("Esc", "key name, as printed on the key"),
        ("Ctrl", "key name, as printed on the key"),
        ("Alt", "key name, as printed on the key"),
        ("Shift", "key name, as printed on the key"),
        ("Delete", "key name, as printed on the key"),
        ("IN", "badge that mirrors in_ in file names"),
        ("OUT", "badge that mirrors out_ in file names"),
        ("CC", "subtitle button, a symbol"),
        ("SRT", "file format name"),
        ("AI", "marker color badge, mirrors the AI marker color name"),
        ("subtitle_cue_list", "widget id"),
        ("search-bar-input", "widget id"),
        ("file-search-bar-input", "widget id"),
        ("marker_list", "widget id"),
        ("marker_name_input", "widget id"),
        ("comment-scrollable", "widget id"),
        ("folder-file-list", "widget id"),
        ("folder-rename-input", "widget id"),
        ("settings-content", "widget id"),
        (
            "Markers <-> comment",
            "English-only log label, see LOG_LABEL",
        ),
        ("SONIOX_API_KEY", "environment variable name"),
        (
            "frename-{}",
            "an internal reference sent to Soniox, never shown",
        ),
        ("ffmpeg.exe", "executable name looked up on PATH"),
        ("ffmpeg", "executable name looked up on PATH"),
        ("PATH", "environment variable name"),
        (
            "request to Soniox failed",
            "matched against the error chain's own (English) text, not shown",
        ),
        (
            "unsupported container",
            "matched against the error chain's own (English) text, not shown",
        ),
        (
            "unsupported audio codec",
            "matched against the error chain's own (English) text, not shown",
        ),
        (
            "ffmpeg is not on PATH",
            "matched against the error chain's own (English) text, not shown",
        ),
        (
            "unauthenticated",
            "Soniox API error_type value, matched against the API's own response",
        ),
        (
            "permission_denied",
            "Soniox API error_type value, matched against the API's own response",
        ),
        (
            "organization_balance_exhausted",
            "Soniox API error_type value, matched against the API's own response",
        ),
        (
            "budget_exhausted",
            "Soniox API error_type value, matched against the API's own response",
        ),
        ("Move comments", "English-only log id, see Action::log_id"),
        (
            "In/out points: comment <-> video",
            "English-only log id, see Action::log_id",
        ),
        (
            "Move in/out points out of file names",
            "English-only log id, see Action::log_id",
        ),
        (
            "Tag commented videos",
            "English-only log id, see Action::log_id",
        ),
        (
            "Fix tags by priority",
            "English-only log id, see Action::log_id",
        ),
        (
            "Apply tag spacing",
            "English-only log id, see Action::log_id",
        ),
        (
            "Reset cache and reload",
            "English-only log id, see Action::log_id",
        ),
        (
            "Describe with AI",
            "English-only log id, see Action::log_id",
        ),
        (
            "Generate subtitles",
            "English-only log id, see Action::log_id",
        ),
        (
            "spent {seconds:.0}s (${usd:.2})",
            "English-only log text, see SubtitleJob::log_report",
        ),
        (
            "{failed_deletes} upload(s) not deleted from Soniox",
            "English-only log text, see SubtitleJob::log_report",
        ),
        (
            "move_comments",
            "stable action id for the batch run remembered across restarts (#65), see Action::id",
        ),
        (
            "move_in_out",
            "stable action id for the batch run remembered across restarts (#65), see Action::id",
        ),
        (
            "in_out_from_names",
            "stable action id for the batch run remembered across restarts (#65), see Action::id",
        ),
        (
            "markers_comment",
            "stable action id for the batch run remembered across restarts (#65), see Action::id",
        ),
        (
            "rotate",
            "stable action id for the batch run remembered across restarts (#65), see Action::id",
        ),
        (
            "tag_commented",
            "stable action id for the batch run remembered across restarts (#65), see Action::id",
        ),
        (
            "fix_tags",
            "stable action id for the batch run remembered across restarts (#65), see Action::id",
        ),
        (
            "respace_tags",
            "stable action id for the batch run remembered across restarts (#65), see Action::id",
        ),
        (
            "reload_files",
            "stable action id for the batch run remembered across restarts (#65), see Action::id",
        ),
        (
            "describe_ai",
            "stable action id for the batch run remembered across restarts (#65), see Action::id",
        ),
        (
            "generate_subtitles",
            "stable action id for the batch run remembered across restarts (#65), see Action::id",
        ),
        (
            "comment_to_markers",
            "stable option value for the batch run remembered across restarts (#65), see Direction::as_str",
        ),
        (
            "markers_to_comment",
            "stable option value for the batch run remembered across restarts (#65), see Direction::as_str",
        ),
        (
            "right",
            "stable option value for the batch run remembered across restarts (#65), see Turn::as_str",
        ),
        (
            "left",
            "stable option value for the batch run remembered across restarts (#65), see Turn::as_str",
        ),
        (
            "half",
            "stable option value for the batch run remembered across restarts (#65), see Turn::as_str",
        ),
        (
            "reset",
            "stable option value for the batch run remembered across restarts (#65), see Turn::as_str",
        ),
        (
            "to",
            "stable option key for the batch run remembered across restarts (#65), see Actions::persist",
        ),
        (
            "direction",
            "stable option key for the batch run remembered across restarts (#65), see Actions::persist",
        ),
        (
            "turn",
            "stable option key for the batch run remembered across restarts (#65), see Actions::persist",
        ),
        (
            "redo",
            "stable option key for the batch run remembered across restarts (#65), see Actions::persist",
        ),
        (
            "replace",
            "stable option key for the batch run remembered across restarts (#65), see Actions::persist",
        ),
        (
            "srt",
            "stable option key for the batch run remembered across restarts (#126), see Actions::persist",
        ),
        (
            "premiere",
            "stable option key for the batch run remembered across restarts (#126), see Actions::persist",
        ),
        (
            "true",
            "stable option value (a bool) for the batch run remembered across restarts (#65), see Actions::persist",
        ),
    ];

    /// The string literals of a Rust file, skipping attributes (doc comments included),
    /// `#[cfg(test)]` items and the message id of `fl!(…)`. Comments are dropped by the lexer.
    fn literals(tokens: proc_macro2::TokenStream, out: &mut Vec<String>) {
        use proc_macro2::{Delimiter, TokenTree};
        let tokens: Vec<TokenTree> = tokens.into_iter().collect();
        let mut i = 0;
        while i < tokens.len() {
            match &tokens[i] {
                TokenTree::Punct(p) if p.as_char() == '#' => {
                    let mut at = i + 1;
                    if matches!(tokens.get(at), Some(TokenTree::Punct(p)) if p.as_char() == '!') {
                        at += 1;
                    }
                    if let Some(TokenTree::Group(attribute)) = tokens.get(at) {
                        let is_test =
                            attribute.stream().to_string().replace(' ', "") == "cfg(test)";
                        i = at + 1;
                        if is_test {
                            // Skip the item up to and including its body.
                            while i < tokens.len() {
                                let end = matches!(&tokens[i], TokenTree::Group(g) if g.delimiter() == Delimiter::Brace)
                                    || matches!(&tokens[i], TokenTree::Punct(p) if p.as_char() == ';');
                                i += 1;
                                if end {
                                    break;
                                }
                            }
                        }
                        continue;
                    }
                }
                TokenTree::Ident(name) if name == "fl" => {
                    if let (Some(TokenTree::Punct(bang)), Some(TokenTree::Group(arguments))) =
                        (tokens.get(i + 1), tokens.get(i + 2))
                    {
                        if bang.as_char() == '!' {
                            // The first literal is the message id.
                            let mut inner = Vec::new();
                            literals(arguments.stream(), &mut inner);
                            out.extend(inner.into_iter().skip(1));
                            i += 3;
                            continue;
                        }
                    }
                }
                // `cfg!(target_os = "macos")` and similar: a compile-time match, not UI text.
                TokenTree::Ident(name) if name == "cfg" => {
                    if let (Some(TokenTree::Punct(bang)), Some(TokenTree::Group(_))) =
                        (tokens.get(i + 1), tokens.get(i + 2))
                    {
                        if bang.as_char() == '!' {
                            i += 3;
                            continue;
                        }
                    }
                }
                // `log::warn!(...)` and its sibling levels: the log is always English.
                TokenTree::Ident(name) if name == "log" => {
                    let is_log_macro = matches!(tokens.get(i + 1), Some(TokenTree::Punct(p)) if p.as_char() == ':')
                        && matches!(tokens.get(i + 2), Some(TokenTree::Punct(p)) if p.as_char() == ':')
                        && matches!(tokens.get(i + 3), Some(TokenTree::Ident(level)) if ["warn", "info", "error", "debug", "trace"].contains(&level.to_string().as_str()))
                        && matches!(tokens.get(i + 4), Some(TokenTree::Punct(p)) if p.as_char() == '!')
                        && matches!(tokens.get(i + 5), Some(TokenTree::Group(_)));
                    if is_log_macro {
                        i += 6;
                        continue;
                    }
                }
                TokenTree::Group(group) => literals(group.stream(), out),
                TokenTree::Literal(literal) => {
                    let text = literal.to_string();
                    if let Some(inner) = text.strip_prefix('"').and_then(|t| t.strip_suffix('"')) {
                        out.push(inner.to_string());
                    } else if text.starts_with('r') && text.contains('"') {
                        out.push(
                            text.trim_start_matches(['r', '#'])
                                .trim_end_matches('#')
                                .trim_matches('"')
                                .to_string(),
                        );
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// Whether `literal` holds a word in Latin letters, `{…}` format placeholders aside.
    fn has_latin_word(literal: &str) -> bool {
        let mut outside = String::new();
        let mut depth = 0;
        for c in literal.chars() {
            match c {
                '{' => depth += 1,
                '}' => depth = (depth - 1).max(0),
                _ if depth == 0 => outside.push(c),
                _ => {}
            }
        }
        outside
            .split(|c: char| !c.is_ascii_alphabetic())
            .any(|word| word.len() >= 2)
    }

    fn ui_files(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                ui_files(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let text = path.to_string_lossy().replace('\\', "/");
                if text.contains("/src/widgets/")
                    || text.ends_with("/view.rs")
                    || text.contains("/batch/actions/")
                    || text.contains("/folder_controls/")
                {
                    out.push(path);
                }
            }
        }
    }

    #[test]
    fn ui_code_has_no_english_words_outside_the_translations() {
        let mut files = Vec::new();
        ui_files(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
            &mut files,
        );
        assert!(files.len() > 10, "the scan found the UI files");
        let mut found = Vec::new();
        for path in files {
            let text = std::fs::read_to_string(&path).unwrap();
            let tokens: proc_macro2::TokenStream = text
                .parse()
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            let mut all = Vec::new();
            literals(tokens, &mut all);
            for literal in all {
                if has_latin_word(&literal) && !NOT_UI_TEXT.iter().any(|(l, _)| *l == literal) {
                    found.push(format!("{}: {literal:?}", path.display()));
                }
            }
        }
        assert!(
            found.is_empty(),
            "UI text belongs in i18n/en/frename.ftl (or NOT_UI_TEXT, with a reason):\n{}",
            found.join("\n")
        );
    }

    #[test]
    fn the_scan_sees_multi_line_calls_and_skips_ids_and_tests() {
        let source = "fn v() { section(\n \"Video\", fl!(\"settings-video\", n = \"Tag\")); }\n\
                      /// Doc words\n#[cfg(test)]\nmod tests { fn t() { let _ = \"Words\"; } }";
        let mut all = Vec::new();
        literals(source.parse().unwrap(), &mut all);
        assert_eq!(all, ["Video", "Tag"]);
        assert!(has_latin_word("Video"));
        assert!(!has_latin_word("{} / {}"));
        assert!(!has_latin_word("✓ {count}"));
    }

    /// A loader of its own, so tests never switch the global one under each other.
    fn loader(language: &str) -> FluentLanguageLoader {
        let loader: FluentLanguageLoader = fluent_language_loader!();
        loader
            .load_languages(&Localizations, &[language.parse().unwrap()])
            .unwrap();
        loader.set_use_isolating(false);
        loader
    }

    /// Issue #274: these toasts were English literals and stayed English in the Russian UI.
    #[test]
    fn the_toasts_of_issue_274_are_translated_and_used() {
        let keys = [
            "folder-frame-saved",
            "folder-not-saved",
            "markers-deleted-notice",
            "markers-not-saved-notice",
            "markers-comment-not-saved-notice",
        ];
        let (en, ru) = (loader("en"), loader("ru"));
        let mut code = String::new();
        let mut files = Vec::new();
        collect_rs(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
            &mut files,
        );
        for file in files {
            code.push_str(&std::fs::read_to_string(file).unwrap());
        }
        for key in keys {
            assert_ne!(en.get(key), ru.get(key), "{key} is not translated");
            assert!(
                ru.get(key).chars().any(|c| ('а'..='я').contains(&c)),
                "{key}: {}",
                ru.get(key)
            );
            assert!(code.contains(&format!("fl!(\"{key}\")")), "{key} is unused");
        }
        assert_eq!(en.get("folder-frame-saved"), "Frame saved");
    }

    fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect_rs(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    #[test]
    fn arguments_are_not_wrapped_in_isolation_marks() {
        let mut args = fluent_bundle::FluentArgs::new();
        args.set("count", 5);
        let text = loader("ru").get_args_fluent("batch-run-rename", Some(&args));
        assert!(!text.contains(['\u{2068}', '\u{2069}']), "{text:?}");
        assert!(text.contains('5'), "{text:?}");
    }

    /// Every non-English message rendered with sample arguments, written to
    /// `target/<language>-messages.txt` for the reviewer and the owner to read real sentences.
    #[test]
    fn a_review_sheet_is_written_for_every_language() {
        let english = messages(&parse(&i18n_dir().join("en/frename.ftl")));
        let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
        std::fs::create_dir_all(&target).unwrap();
        for (language, _) in translations() {
            let bundle = loader(&language);
            let mut sheet = String::new();
            for (id, (_, arguments)) in &english {
                if arguments.is_empty() {
                    sheet.push_str(&format!("{id}\n    {}\n", bundle.get(id)));
                    continue;
                }
                sheet.push_str(&format!("{id}\n"));
                let mut lines: Vec<String> = Vec::new();
                for n in [1, 2, 5, 11, 21] {
                    let mut args = fluent_bundle::FluentArgs::new();
                    for argument in arguments {
                        match argument.as_str() {
                            "tag" => args.set(argument.clone(), "Commented"),
                            "language" => args.set(argument.clone(), "English"),
                            _ => args.set(argument.clone(), n),
                        }
                    }
                    lines.push(bundle.get_args_fluent(id, Some(&args)));
                }
                // Messages without a number read the same for every sample.
                lines.dedup();
                for line in lines {
                    sheet.push_str(&format!("    {line}\n"));
                }
            }
            std::fs::write(target.join(format!("{language}-messages.txt")), sheet).unwrap();
        }
    }
}
