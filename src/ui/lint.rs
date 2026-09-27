//! Keeps views on the design system (`docs/design/design-system.md` §15.2): outside `src/ui/`, no
//! color literal and no number given as a size, padding, spacing or radius. `0` alone means
//! "none" and is allowed. Test code is not checked.

use std::path::{Path, PathBuf};

/// Files not on the design system yet, each with the issue that moves it. The list only shrinks:
/// a file here with nothing left to allow fails the test too.
const NOT_YET: [(&str, &str); 38] = [
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
    ("features/file_name_panel/mod.rs", "#59"),
    ("features/file_name_panel/trash_zone.rs", "#59"),
    ("features/file_name_panel/view.rs", "#59"),
    ("features/file_workspace/state.rs", "#59"),
    ("features/file_workspace/view.rs", "#59"),
    ("features/folder/mod.rs", "#59"),
    ("features/folder/view.rs", "#59"),
    ("features/folder_controls/view.rs", "#59"),
    ("features/folder_workspace/state.rs", "#59"),
    ("features/folder_workspace/view.rs", "#59"),
    ("features/markers/view.rs", "#59"),
    ("features/media_viewer/video/view.rs", "#59"),
    ("features/media_viewer/view.rs", "#59"),
    ("features/sync_panel/view.rs", "#59"),
    ("features/tag_grid/view.rs", "#59"),
    ("features/tag_panel/state.rs", "#59"),
    ("features/tag_panel/view.rs", "#59"),
    ("features/video_controls/progress_bar.rs", "#59"),
    ("features/video_controls/view.rs", "#59"),
    ("main.rs", "#59"),
    ("tag_colors.rs", "#53, #59"),
    ("widgets/file_name_display.rs", "#59"),
    ("widgets/height_handle.rs", "#59"),
    ("widgets/search_bar.rs", "#59"),
    ("widgets/splitter.rs", "#59"),
    ("widgets/starred_tags_panel.rs", "#59"),
    ("widgets/tag_chip.rs", "#59"),
    ("widgets/timecode_badge.rs", "#59"),
];

/// A method whose name ends in one of these takes a size, a padding or a spacing.
const SIZE_METHOD_ENDINGS: [&str; 7] = [
    "size",
    "spacing",
    "padding",
    "width",
    "height",
    "gap",
    "line_height",
];

/// Methods that take a size under another name.
const SIZE_METHODS: [&str; 2] = ["center_x", "center_y"];

/// Functions whose arguments are sizes, offsets or radii.
const SIZE_FUNCTIONS: [&str; 6] = [
    "rounded(",
    "radius(",
    "Length::Fixed(",
    "Padding::new(",
    "Vector::new(",
    "Size::new(",
];

/// Style structs whose fields are sizes.
const SIZE_STRUCTS: [&str; 3] = ["Padding {", "Border {", "Shadow {"];

const COLOR_LITERALS: [&str; 8] = [
    "Color::from_rgb",
    "Color::parse(",
    "Color::from_linear",
    "Color::new(",
    "Color {",
    "Color::WHITE",
    "Color::BLACK",
    "color!(",
];

/// The types a size constant has.
const SIZE_TYPES: [&str; 3] = ["f32", "u16", "u32"];

/// Words in a constant's name that make it a size (a ratio, an alpha or a count is not one).
const SIZE_NAMES: [&str; 11] = [
    "WIDTH", "HEIGHT", "SIZE", "SPACING", "PADDING", "GAP", "RADIUS", "MARGIN", "INDENT", "SIDE",
    "PX",
];

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

/// Whether `text` holds a number other than a bare zero. `FillPortion(n)` is a ratio, not a
/// size.
fn has_number(text: &str) -> bool {
    let mut text = text.to_string();
    while let Some(at) = text.find("FillPortion(") {
        let end = text[at..]
            .find(')')
            .map_or(text.len(), |close| at + close + 1);
        text.replace_range(at..end, "");
    }
    text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '.'))
        .filter(|token| token.starts_with(|c: char| c.is_ascii_digit()))
        .any(|token| !matches!(token.trim_end_matches(".into"), "0" | "0.0"))
}

/// Every call `.name(` in `code`: the method's name and where its argument starts.
fn method_calls(code: &str) -> impl Iterator<Item = (&str, usize)> {
    code.match_indices('.').filter_map(move |(at, _)| {
        let rest = &code[at + 1..];
        let name_end = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))?;
        (name_end > 0 && rest[name_end..].starts_with('('))
            .then(|| (&rest[..name_end], at + 1 + name_end + 1))
    })
}

/// What in `code` breaks the rules.
fn violations(code: &str) -> Vec<String> {
    let mut found = Vec::new();
    for literal in COLOR_LITERALS {
        if code.contains(literal) {
            found.push(literal.to_string());
        }
    }
    for (name, start) in method_calls(code) {
        let takes_size = SIZE_METHODS.contains(&name)
            || SIZE_METHOD_ENDINGS
                .iter()
                .any(|ending| name.ends_with(ending));
        let argument = enclosed(code, start, '(', ')');
        if takes_size && has_number(argument) {
            found.push(format!(".{name}({argument})"));
        }
    }
    for function in SIZE_FUNCTIONS {
        for (at, _) in code.match_indices(function) {
            let argument = enclosed(code, at + function.len(), '(', ')');
            if has_number(argument) {
                found.push(format!("{function}{argument})"));
            }
        }
    }
    // `padding::all(…)`, `border::width(…)`: the argument starts after the function's name.
    for module in ["padding::", "border::"] {
        for (at, _) in code.match_indices(module) {
            if let Some(open) = code[at..].find('(') {
                let argument = enclosed(code, at + open + 1, '(', ')');
                if has_number(argument) {
                    found.push(format!("{module}…({argument})"));
                }
            }
        }
    }
    for structure in SIZE_STRUCTS {
        for (at, _) in code.match_indices(structure) {
            let fields = enclosed(code, at + structure.len(), '{', '}');
            if has_number(fields) {
                found.push(format!("{structure}{fields}}}"));
            }
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
    // A size moved into a constant of the view is still a size of its own.
    for (at, _) in code.match_indices("const ") {
        let declaration = code[at..].split(';').next().unwrap_or_default();
        let Some((name_and_type, value)) = declaration.split_once('=') else {
            continue;
        };
        let size_type = SIZE_TYPES
            .iter()
            .any(|t| name_and_type.trim_end().ends_with(&format!(": {t}")));
        let name = name_and_type.split(':').next().unwrap_or_default();
        let size_name = SIZE_NAMES.iter().any(|word| name.contains(word));
        if size_type && size_name && has_number(value) {
            found.push(declaration.trim().to_string());
        }
    }
    found
}

fn sources(dir: &Path, skip: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path != skip {
                sources(&path, skip, out);
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
    sources(&src, &src.join("ui"), &mut files);
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
        "const GAP: f32 = 6.0; const NONE: f32 = 0.0; const ID: &str = \"x-1\";
         const IDLE_ALPHA: f32 = 0.6; const RETRIES: u32 = 3;
         fn v() { text(\"a\").size(13); row![].spacing(SPACE_S).padding(0).spacing(GAP);
         container(x).padding([4, 8]).width(Length::Fixed(260.0)).center_y(56);
         row![].vertical_spacing(6); let p = Padding { top: 0.0, left: 26.0, ..Padding::ZERO };
         let c = Color::from_rgb(1.0, 0.8, 0.0); let q = padding::all(SPACE_M);
         let r = border::rounded(RADIUS_S); // .size(99) in a comment
         row![].width(Length::FillPortion(3)); let s = iced::Size::new(800.0, 600.0);
}
         #[cfg(test)]
mod tests { fn t() { let _ = x.size(16); } }
         fn w() { let b = Border { radius: 3.0.into(), ..b }; }",
    );
    let mut found = violations(&code);
    found.sort();
    let expected = [
        ".center_y(56)",
        ".padding([4, 8])",
        ".size(13)",
        ".vertical_spacing(6)",
        ".width(Length::Fixed(260.0))",
        "Border { radius: 3.0.into(), ..b }",
        "Color::from_rgb",
        "Length::Fixed(260.0)",
        "Padding { top: 0.0, left: 26.0, ..Padding::ZERO }",
        "Size::new(800.0, 600.0)",
        "const GAP: f32 = 6.0",
        "radius: 3.0.into()",
    ];
    assert_eq!(found, expected);
}
