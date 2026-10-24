//! Release notes: the blocks of `version.md` (one per version, newest first), parsed for the
//! Updates page. The running build carries its own copy, so the notes of the installed version
//! and older ones need no network; the versions between it and a newer one come from the
//! `version.md` of that release (`source::fetch_notes`).

use semver::Version;

/// `version.md` as it was when this build was made: the installed version and everything before.
pub const BUNDLED: &str = include_str!("../../../version.md");

/// One version's block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionNotes {
    pub version: Version,
    pub groups: Vec<Group>,
}

/// One `## Added` / `## Changed` / `## Fixed` list; the title is empty for bullets that come
/// before any `##` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub title: String,
    pub items: Vec<Item>,
}

/// A bullet; `nested` for one indented under another.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub nested: bool,
    pub text: String,
}

/// How a run of text is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Plain,
    Bold,
    Code,
}

impl VersionNotes {
    /// The version as the UI shows it, `0.175`.
    pub fn label(&self) -> String {
        crate::package::short_version(&self.version.to_string())
    }
}

/// `0.175` or `0.175.2` as a version; `None` for anything else (`NEXT`).
pub fn parse_version(text: &str) -> Option<Version> {
    let text = text.trim();
    let dots = text.matches('.').count();
    let full = match dots {
        1 => format!("{text}.0"),
        2 => text.to_string(),
        _ => return None,
    };
    Version::parse(&full).ok()
}

/// The blocks of a `version.md`, newest first. A heading that is not a version number, and
/// what is under it, is left out.
pub fn parse(markdown: &str) -> Vec<VersionNotes> {
    let mut versions: Vec<VersionNotes> = Vec::new();
    // Whether the lines now being read belong to a version block.
    let mut inside = false;
    for line in markdown.lines() {
        if let Some(heading) = line.strip_prefix("# ") {
            match parse_version(heading) {
                Some(version) => {
                    versions.push(VersionNotes {
                        version,
                        groups: Vec::new(),
                    });
                    inside = true;
                }
                None => inside = false,
            }
            continue;
        }
        if !inside {
            continue;
        }
        let Some(current) = versions.last_mut() else {
            continue;
        };
        if let Some(title) = line.strip_prefix("## ") {
            current.groups.push(Group {
                title: title.trim().to_string(),
                items: Vec::new(),
            });
            continue;
        }
        let trimmed = line.trim_start();
        let bullet = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "));
        if let Some(text) = bullet {
            if current.groups.is_empty() {
                current.groups.push(Group {
                    title: String::new(),
                    items: Vec::new(),
                });
            }
            if let Some(group) = current.groups.last_mut() {
                group.items.push(Item {
                    nested: line.starts_with(' ') || line.starts_with('\t'),
                    text: text.trim().to_string(),
                });
            }
        } else if !trimmed.is_empty() {
            // The next line of a bullet that wraps.
            if let Some(item) = current.groups.last_mut().and_then(|g| g.items.last_mut()) {
                item.text.push(' ');
                item.text.push_str(trimmed);
            }
        }
    }
    versions.sort_by(|a, b| b.version.cmp(&a.version));
    versions
}

/// `base` with the blocks of `newer` added; a version in both takes the one in `newer`. Newest
/// first.
pub fn merge(base: Vec<VersionNotes>, newer: Vec<VersionNotes>) -> Vec<VersionNotes> {
    let replaced: Vec<Version> = newer.iter().map(|n| n.version.clone()).collect();
    let mut all: Vec<VersionNotes> = base
        .into_iter()
        .filter(|b| !replaced.contains(&b.version))
        .chain(newer)
        .collect();
    all.sort_by(|a, b| b.version.cmp(&a.version));
    all
}

/// The blocks newer than `after` and not newer than `up_to`, in the order of `all`.
pub fn between<'a>(
    all: &'a [VersionNotes],
    after: &Version,
    up_to: &Version,
) -> Vec<&'a VersionNotes> {
    all.iter()
        .filter(|n| n.version > *after && n.version <= *up_to)
        .collect()
}

/// A bullet's text cut into plain, `**bold**` and `` `code` `` runs. An unclosed marker stays as
/// the text it is.
pub fn inline(text: &str) -> Vec<(Style, String)> {
    let mut runs: Vec<(Style, String)> = Vec::new();
    let mut plain = String::new();
    let mut rest = text;
    let flush = |plain: &mut String, runs: &mut Vec<(Style, String)>| {
        if !plain.is_empty() {
            runs.push((Style::Plain, std::mem::take(plain)));
        }
    };
    while !rest.is_empty() {
        let marked = [("**", Style::Bold), ("`", Style::Code)]
            .into_iter()
            .find_map(|(marker, style)| {
                let inner = rest.strip_prefix(marker)?;
                let end = inner.find(marker)?;
                (end > 0).then_some((marker.len(), end, style))
            });
        match marked {
            Some((marker, end, style)) => {
                flush(&mut plain, &mut runs);
                runs.push((style, rest[marker..marker + end].to_string()));
                rest = &rest[marker + end + marker..];
            }
            None => {
                let mut chars = rest.chars();
                if let Some(c) = chars.next() {
                    plain.push(c);
                }
                rest = chars.as_str();
            }
        }
    }
    flush(&mut plain, &mut runs);
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "# 0.12\n## Added\n- First **thing** with `code`.\n  - Nested one.\n## Fixed\n- A bullet that\n  wraps.\n\n# 0.11.1\n- No group.\n\n# NEXT\n## Added\n- Dropped.\n\n# 0.9\n## Changed\n- Old.\n";

    fn v(text: &str) -> Version {
        parse_version(text).expect("version")
    }

    #[test]
    fn versions_with_two_or_three_parts_parse_and_others_do_not() {
        assert_eq!(v("0.175"), Version::new(0, 175, 0));
        assert_eq!(v("0.68.1"), Version::new(0, 68, 1));
        assert_eq!(parse_version("NEXT"), None);
        assert_eq!(parse_version("1"), None);
    }

    #[test]
    fn a_version_md_is_cut_into_versions_groups_and_bullets() {
        let notes = parse(SAMPLE);
        let labels: Vec<String> = notes.iter().map(VersionNotes::label).collect();
        assert_eq!(labels, ["0.12", "0.11.1", "0.9"], "NEXT is left out");

        let first = &notes[0];
        assert_eq!(
            first
                .groups
                .iter()
                .map(|g| g.title.as_str())
                .collect::<Vec<_>>(),
            ["Added", "Fixed"]
        );
        assert_eq!(first.groups[0].items.len(), 2);
        assert!(!first.groups[0].items[0].nested);
        assert!(first.groups[0].items[1].nested);
        assert_eq!(
            first.groups[1].items[0].text, "A bullet that wraps.",
            "a wrapped line joins its bullet"
        );
        assert_eq!(
            notes[1].groups[0].title, "",
            "bullets before any ## heading"
        );
    }

    #[test]
    fn the_bundled_version_md_parses_with_the_running_version_first() {
        let notes = parse(BUNDLED);
        assert!(!notes.is_empty());
        assert!(
            notes
                .windows(2)
                .all(|pair| pair[0].version > pair[1].version),
            "newest first, no version twice"
        );
        assert!(notes
            .iter()
            .all(|n| n.groups.iter().all(|g| !g.items.is_empty())));
    }

    #[test]
    fn the_versions_between_the_installed_and_the_target_are_picked() {
        let notes = parse(SAMPLE);
        let picked: Vec<String> = between(&notes, &v("0.9"), &v("0.12"))
            .iter()
            .map(|n| n.label())
            .collect();
        assert_eq!(
            picked,
            ["0.12", "0.11.1"],
            "not the installed one, newest first"
        );
        assert!(between(&notes, &v("0.12"), &v("0.12")).is_empty());
        let skipped = between(&notes, &v("0.9"), &v("0.11.1"));
        assert_eq!(skipped.len(), 1, "not past the target");
    }

    #[test]
    fn fetched_notes_replace_the_bundled_ones_of_the_same_version() {
        let bundled = parse("# 0.2\n- old text\n\n# 0.1\n- one\n");
        let fetched = parse("# 0.3\n- three\n\n# 0.2\n- new text\n");
        let all = merge(bundled, fetched);
        let labels: Vec<String> = all.iter().map(VersionNotes::label).collect();
        assert_eq!(labels, ["0.3", "0.2", "0.1"]);
        assert_eq!(all[1].groups[0].items[0].text, "new text");
    }

    #[test]
    fn bold_and_code_are_cut_out_and_a_lone_marker_stays_text() {
        assert_eq!(
            inline("Press **F5** or `Esc` now"),
            [
                (Style::Plain, "Press ".to_string()),
                (Style::Bold, "F5".to_string()),
                (Style::Plain, " or ".to_string()),
                (Style::Code, "Esc".to_string()),
                (Style::Plain, " now".to_string()),
            ]
        );
        assert_eq!(
            inline("a ** b and ` c"),
            [(Style::Plain, "a ** b and ` c".to_string())]
        );
        assert_eq!(
            inline("ünï **ö**"),
            [
                (Style::Plain, "ünï ".to_string()),
                (Style::Bold, "ö".to_string()),
            ]
        );
    }
}
