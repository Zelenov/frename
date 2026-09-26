//! UI translations: the Fluent files in `i18n/<lang>/frename.ftl`, embedded in the binary, and
//! the language they are shown in.
//!
//! One global loader, switched only from `update` ([`apply`]) and read by the views through
//! [`fl!`](crate::fl): the same pattern as the settings core keeps globally. English is the
//! source language and the fallback for any message a translation lacks.

use std::sync::{Mutex, OnceLock};

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

/// The loader every [`fl!`](crate::fl) reads.
pub fn loader() -> &'static FluentLanguageLoader {
    static LOADER: OnceLock<FluentLanguageLoader> = OnceLock::new();
    LOADER.get_or_init(|| {
        let loader: FluentLanguageLoader = fluent_language_loader!();
        if let Err(e) = loader.load_fallback_language(&Localizations) {
            log::error!("cannot load the English UI text: {e}");
        }
        loader.set_use_isolating(false);
        loader
    })
}

/// The language `System` stands for: the OS language as read at start-up or when `System` was
/// last picked.
static SYSTEM_LANGUAGE: Mutex<&str> = Mutex::new(FALLBACK);

/// A message in the current UI language, checked against `i18n/en/frename.ftl` at compile time:
/// `fl!("batch-run", count = 5)`.
#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::i18n::loader(), $message_id)
    }};
    ($message_id:literal, $($args:expr),* $(,)?) => {{
        i18n_embed_fl::fl!($crate::i18n::loader(), $message_id, $($args),*)
    }};
}

/// Show the UI in the language the `stored` setting asks for (empty: the OS language). Reads
/// the OS languages, so call it at start-up and when the setting changes, not per frame.
pub fn apply(stored: &str) {
    let os: Vec<String> = DesktopLanguageRequester::requested_languages()
        .iter()
        .map(|id| id.to_string())
        .collect();
    let system = resolve("", &os);
    if let Ok(mut current) = SYSTEM_LANGUAGE.lock() {
        *current = system;
    }
    let language = resolve(stored, &os);
    match language.parse::<LanguageIdentifier>() {
        Ok(id) => {
            if let Err(e) = loader().load_languages(&Localizations, &[id]) {
                log::error!("cannot load the UI language {language}: {e}");
            }
        }
        Err(e) => log::error!("bad UI language code {language}: {e}"),
    }
    // A load builds new bundles, which isolate arguments again.
    loader().set_use_isolating(false);
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

    #[test]
    fn russian_plurals_have_one_few_and_many() {
        let cldr = ["zero", "one", "two", "few", "many", "other"];
        let english = selects(&parse(&i18n_dir().join("en/frename.ftl")));
        let russian = selects(&parse(&i18n_dir().join("ru/frename.ftl")));
        for (id, english_selects) in &english {
            let plural = english_selects
                .iter()
                .any(|keys| keys.iter().any(|k| cldr.contains(&k.as_str())));
            if !plural {
                continue;
            }
            let russian_selects = &russian[id];
            assert!(!russian_selects.is_empty(), "ru: {id} lost its plural");
            for keys in russian_selects {
                for category in ["one", "few", "many"] {
                    assert!(keys.contains(category), "ru: {id} has no [{category}]");
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
    const NOT_UI_TEXT: [(&str, &str); 13] = [
        ("Page Up", "key name, as printed on the key"),
        ("Page Down", "key name, as printed on the key"),
        ("Space", "key name, as printed on the key"),
        ("Enter", "key name, as printed on the key"),
        ("Esc", "key name, as printed on the key"),
        ("Delete", "key name, as printed on the key"),
        ("IN", "badge that mirrors in_ in file names"),
        ("OUT", "badge that mirrors out_ in file names"),
        ("CC", "subtitle button, a symbol"),
        ("SRT", "file format name"),
        ("subtitle_cue_list", "widget id"),
        ("search-bar-input", "widget id"),
        ("file-search-bar-input", "widget id"),
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

    #[test]
    fn arguments_are_not_wrapped_in_isolation_marks() {
        let mut args = fluent_bundle::FluentArgs::new();
        args.set("count", 5);
        let text = loader("ru").get_args_fluent("batch-run", Some(&args));
        assert!(!text.contains(['\u{2068}', '\u{2069}']), "{text:?}");
        assert!(text.contains('5'), "{text:?}");
    }

    /// Every Russian message rendered with sample arguments, written to
    /// `target/ru-messages.txt` for the reviewer and the owner to read real sentences.
    #[test]
    fn the_russian_review_sheet_is_written() {
        let english = messages(&parse(&i18n_dir().join("en/frename.ftl")));
        let russian = loader("ru");
        let mut sheet = String::new();
        for (id, (_, arguments)) in &english {
            if arguments.is_empty() {
                sheet.push_str(&format!("{id}\n    {}\n", russian.get(id)));
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
                lines.push(russian.get_args_fluent(id, Some(&args)));
            }
            // Messages without a number read the same for every sample.
            lines.dedup();
            for line in lines {
                sheet.push_str(&format!("    {line}\n"));
            }
        }
        let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("ru-messages.txt"), sheet).unwrap();
    }
}
