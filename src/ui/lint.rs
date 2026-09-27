//! Keeps views on the design system (`docs/design/design-system.md` §15.2): outside `src/ui/`, no
//! color literal and no number given as a size, padding, spacing or radius. `0` alone means
//! "none" and is allowed. Test code is not checked.

use std::path::{Path, PathBuf};

/// Files not on the design system yet, each with the issue that moves it. The list only shrinks:
/// a file here with nothing left to allow fails the test too.
const NOT_YET: [(&str, &str); 30] = [
    ("features/batch/actions/describe_ai/mod.rs", "#58"),
    ("features/batch/actions/generate_subtitles.rs", "#58"),
    ("features/batch/actions/markers_comment.rs", "#58"),
    ("features/batch/actions/mod.rs", "#58"),
    ("features/batch/actions/move_comments.rs", "#58"),
    ("features/batch/actions/move_in_out.rs", "#58"),
    ("features/batch/actions/tag_commented.rs", "#58"),
    ("features/batch/actions/tag_spacing.rs", "#58"),
    ("features/batch/view.rs", "#58"),
    ("features/file_name_panel/file_name_line.rs", "#59"),
    ("features/file_name_panel/trash_zone.rs", "#59"),
    ("features/file_name_panel/view.rs", "#59"),
    ("features/file_workspace/view.rs", "#59"),
    ("features/folder/view.rs", "#59"),
    ("features/folder_controls/view.rs", "#59"),
    ("features/folder_workspace/view.rs", "#59"),
    ("features/markers/view.rs", "#59"),
    ("features/media_viewer/video/view.rs", "#59"),
    ("features/media_viewer/view.rs", "#59"),
    ("features/sync_panel/view.rs", "#59"),
    ("features/tag_grid/view.rs", "#59"),
    ("features/tag_panel/view.rs", "#59"),
    ("features/video_controls/progress_bar.rs", "#59"),
    ("features/video_controls/view.rs", "#59"),
    ("tag_colors.rs", "#53, #59"),
    ("widgets/file_name_display.rs", "#59"),
    ("widgets/search_bar.rs", "#59"),
    ("widgets/starred_tags_panel.rs", "#59"),
    ("widgets/tag_chip.rs", "#59"),
    ("widgets/timecode_badge.rs", "#59"),
];

/// Calls whose argument is a size, a padding, a spacing or a radius.
const SIZE_CALLS: [&str; 13] = [
    ".padding(",
    ".spacing(",
    ".size(",
    ".text_size(",
    ".gap(",
    ".width(",
    ".height(",
    ".max_width(",
    ".line_height(",
    "rounded(",
    "Length::Fixed(",
    "Padding::new(",
    "padding::",
];

const COLOR_LITERALS: [&str; 4] = ["Color::from_rgb", "Color::WHITE", "Color::BLACK", "color!("];

/// The code of a file without comments and without its `#[cfg(test)]` items.
fn code(text: &str) -> String {
    let text: String = text
        .lines()
        .map(|line| match line.find("//") {
            Some(at) if !line[..at].contains('"') => &line[..at],
            _ => line,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut kept = String::new();
    let mut rest = text.as_str();
    while let Some(at) = rest.find("#[cfg(test)]") {
        kept.push_str(&rest[..at]);
        let item = &rest[at..];
        // The item ends at its `;`, or at the brace that closes its body.
        rest = match (item.find(';'), item.find('{')) {
            (Some(semicolon), Some(open)) if semicolon < open => &item[semicolon + 1..],
            (_, Some(open)) => {
                let body = enclosed(item, open + 1, '{', '}');
                &item[(open + 1 + body.len() + 1).min(item.len())..]
            }
            (Some(semicolon), None) => &item[semicolon + 1..],
            (None, None) => "",
        };
    }
    kept.push_str(rest);
    kept
}

/// The text from `start` to the bracket that closes the one just before it.
fn enclosed(code: &str, start: usize, open: char, close: char) -> &str {
    let mut depth = 1;
    for (i, c) in code[start..].char_indices() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return &code[start..start + i];
            }
        }
    }
    &code[start..]
}

/// Whether `text` holds a number other than a bare zero.
fn has_number(text: &str) -> bool {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
        .filter(|token| token.starts_with(|c: char| c.is_ascii_digit()))
        .any(|token| !matches!(token.trim_end_matches(".into"), "0" | "0.0"))
}

/// What in `code` breaks the rules.
fn violations(code: &str) -> Vec<String> {
    let mut found = Vec::new();
    for literal in COLOR_LITERALS {
        if code.contains(literal) {
            found.push(literal.to_string());
        }
    }
    for call in SIZE_CALLS {
        for (at, _) in code.match_indices(call) {
            let after = at + call.len();
            // `padding::all(…)`: the argument starts after the function's name.
            let start = if call == "padding::" {
                match code[after..].find('(') {
                    Some(open) => after + open + 1,
                    None => continue,
                }
            } else {
                after
            };
            let argument = enclosed(code, start, '(', ')');
            if has_number(argument) {
                found.push(format!("{call}{argument})"));
            }
        }
    }
    for (at, _) in code.match_indices("Padding {") {
        let fields = enclosed(code, at + "Padding {".len(), '{', '}');
        if has_number(fields) {
            found.push(format!("Padding {{{fields}}}"));
        }
    }
    for (at, _) in code.match_indices("radius:") {
        let value = code[at + "radius:".len()..]
            .split([',', '}', '\n'])
            .next()
            .unwrap_or_default();
        if has_number(value) {
            found.push(format!("radius:{value}"));
        }
    }
    found
}

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if !path.ends_with("ui") {
                sources(&path, out);
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Every source file outside `src/ui/` with what breaks the rules in it, by path relative to
/// `src/` with `/` separators.
fn scan() -> Vec<(String, Vec<String>)> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&src, &mut files);
    assert!(files.len() > 50, "the scan found the sources");
    files
        .into_iter()
        .map(|path| {
            let relative = path
                .strip_prefix(&src)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let text = std::fs::read_to_string(&path).unwrap();
            (relative, violations(&code(&text)))
        })
        .collect()
}

#[test]
fn views_take_colors_and_sizes_from_the_design_system() {
    let mut problems = Vec::new();
    for (file, found) in scan() {
        let allowed = NOT_YET.iter().any(|(f, _)| *f == file);
        match (allowed, found.is_empty()) {
            (false, false) => problems.push(format!(
                "{file}: use ui::tokens instead of {}",
                found.join(", ")
            )),
            (true, true) => problems.push(format!(
                "{file} is on the design system now: remove it from NOT_YET"
            )),
            _ => {}
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn every_file_not_yet_on_the_system_exists() {
    let files: Vec<String> = scan().into_iter().map(|(file, _)| file).collect();
    for (file, _) in NOT_YET {
        assert!(files.iter().any(|f| f == file), "{file} is gone");
    }
}

#[test]
fn the_scan_finds_literals_and_lets_tokens_and_zero_through() {
    let code = code(
        "fn v() { text(\"a\").size(13); row![].spacing(SPACE_S).padding(0);\n\
         container(x).padding([4, 8]).width(Length::Fixed(260.0));\n\
         let p = Padding { top: 0.0, left: 26.0, ..Padding::ZERO };\n\
         let b = Border { radius: 3.0.into(), ..b }; let c = Color::from_rgb(1.0, 0.8, 0.0);\n\
         let q = padding::all(SPACE_M); // .size(99) in a comment\n}\n\
         #[cfg(test)]\nmod tests { fn t() { let _ = x.size(16); } }",
    );
    let found = violations(&code);
    assert_eq!(found.len(), 7, "{found:#?}");
    assert!(found.iter().any(|f| f.starts_with("Color::from_rgb")));
    assert!(found.iter().any(|f| f == ".size(13)"));
    assert!(found.iter().any(|f| f == ".padding([4, 8])"));
    assert!(found.iter().any(|f| f == "Length::Fixed(260.0)"));
    assert!(found.iter().any(|f| f == ".width(Length::Fixed(260.0))"));
    assert!(found.iter().any(|f| f.starts_with("Padding {")));
    assert!(found.iter().any(|f| f.starts_with("radius:")));
}
