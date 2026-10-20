//! Timecodes typed or pasted by the editor: every style frename shows or Premiere Pro and
//! DaVinci Resolve write, read as milliseconds from the start of the clip.

/// The playhead as `HH:MM:SS.mmm`: what Copy puts on the clipboard.
pub fn format(ms: u64) -> String {
    let total = ms / 1000;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        total / 3600,
        total / 60 % 60,
        total % 60,
        ms % 1000
    )
}

/// How times are written in the player, the marker and subtitle lists and the In / Out chips
/// (#309). The setting only changes what is shown (and what Copy puts on the clipboard): files
/// keep their own formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TimecodeStyle {
    /// `mm:ss`, and `00:10.250` when paused.
    #[default]
    Default,
    /// `HH:MM:SS:FF` the way Premiere Pro shows it; drop-frame is `HH;MM;SS;FF`.
    Premiere,
    /// `HH:MM:SS:FF` the way DaVinci Resolve shows it; drop-frame is `HH:MM:SS;FF`.
    Resolve,
}

impl TimecodeStyle {
    /// Every style, in the order the setting lists them.
    pub const ALL: [Self; 3] = [Self::Default, Self::Premiere, Self::Resolve];

    /// Stable name for persisting the setting.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Premiere => "premiere",
            Self::Resolve => "resolve",
        }
    }

    /// Parse a persisted name; unknown names fall back to the default.
    pub fn from_name(name: &str) -> Self {
        match name {
            "premiere" => Self::Premiere,
            "resolve" => Self::Resolve,
            _ => Self::Default,
        }
    }
}

/// A style with the frame rate of the clip it is shown for: everything a time needs to be
/// written. A clip without a steady frame rate (`fps` is `None`, zero or not a number) is
/// written in the default style whatever the setting says.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TimeFormat {
    style: TimecodeStyle,
    fps: Option<f64>,
}

impl TimeFormat {
    pub fn new(style: TimecodeStyle, fps: Option<f64>) -> Self {
        let fps = fps.filter(|fps| fps.is_finite() && *fps > 0.0);
        Self { style, fps }
    }

    /// The style in effect: the default one when the clip has no frame rate to count frames in.
    pub fn effective_style(self) -> TimecodeStyle {
        match self.fps {
            Some(_) => self.style,
            None => TimecodeStyle::Default,
        }
    }

    /// `ms` as a timecode with frames, or `None` in the default style.
    fn frame_timecode(self, ms: u64) -> Option<String> {
        let fps = self.fps?;
        match self.style {
            TimecodeStyle::Default => None,
            TimecodeStyle::Premiere => Some(frame_timecode(ms, fps, DropSeparator::Everywhere)),
            TimecodeStyle::Resolve => Some(frame_timecode(ms, fps, DropSeparator::BeforeFrames)),
        }
    }

    /// A moment of the clip: `HH:MM:SS:FF` in the frame styles, `fallback(ms)` in the default.
    pub fn format_with(self, ms: u64, fallback: impl FnOnce(u64) -> String) -> String {
        self.frame_timecode(ms).unwrap_or_else(|| fallback(ms))
    }

    /// What Copy puts on the clipboard and the time field starts with.
    pub fn copied(self, ms: u64) -> String {
        self.format_with(ms, format)
    }
}

/// Where a drop-frame timecode puts its `;`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DropSeparator {
    /// `00;01;00;02`: Premiere Pro.
    Everywhere,
    /// `00:01:00;02`: DaVinci Resolve.
    BeforeFrames,
}

/// Drop-frame numbers skipped each minute (but every tenth) at a frame rate: 2 at 29.97, 4 at
/// 59.94, none elsewhere (23.976 and the others count whole frames).
fn dropped_per_minute(fps: f64) -> u64 {
    match fps.round() as u64 {
        30 if (fps - 29.97).abs() < 0.01 => 2,
        60 if (fps - 59.94).abs() < 0.01 => 4,
        _ => 0,
    }
}

/// `ms` as `HH:MM:SS:FF` at `fps`. 29.97 and 59.94 fps are drop-frame, the way NTSC timelines
/// count; every other rate counts `fps.round()` frames a second.
fn frame_timecode(ms: u64, fps: f64, separator: DropSeparator) -> String {
    let nominal = fps.round().max(1.0) as u64;
    let drop = dropped_per_minute(fps);
    // The frame the moment falls in; the inverse of `frames` when parsing.
    let mut count = (ms as f64 * fps / 1000.0).round() as u64;
    if drop > 0 {
        let per_ten_minutes = nominal * 600 - drop * 9;
        let per_minute = nominal * 60 - drop;
        let tens = count / per_ten_minutes;
        let rest = count % per_ten_minutes;
        count += drop * 9 * tens;
        if rest >= drop {
            count += drop * ((rest - drop) / per_minute);
        }
    }
    let frames = count % nominal;
    let total = count / nominal;
    let (hours, minutes, seconds) = (total / 3600, total / 60 % 60, total % 60);
    let (between, before_frames) = match (drop > 0, separator) {
        (false, _) => (':', ':'),
        (true, DropSeparator::Everywhere) => (';', ';'),
        (true, DropSeparator::BeforeFrames) => (':', ';'),
    };
    format!("{hours:02}{between}{minutes:02}{between}{seconds:02}{before_frames}{frames:02}")
}

/// Read a timecode as milliseconds. `fps` is the clip's frame rate, for the frames field.
///
/// Accepted: plain seconds (`65.25`), `M:SS`, `H:MM:SS`, either with `.mmm`, and
/// `H:MM:SS:FF` / `H:MM:SS;FF` with frames. A `;` before the frames (or at 29.97 / 59.94 fps
/// a `;` anywhere) is drop-frame. `None` for anything else: a field out of range (minutes or
/// seconds from 60, frames from the frame rate), or a frame number drop-frame skips.
pub fn parse(text: &str, fps: f64) -> Option<u64> {
    let text = text.trim();
    if text.is_empty() || !fps.is_finite() || fps <= 0.0 {
        return None;
    }
    if !text.contains([':', ';']) {
        return seconds(text);
    }
    let drop = text.contains(';');
    let parts: Vec<&str> = text.split([':', ';']).collect();
    match parts.as_slice() {
        // `M:SS` takes any minutes; `H:MM:SS` has minutes below 60.
        [m, s] if !drop => clock(0, number(m)?, s),
        [h, m, s] if !drop => {
            let m = number(m)?;
            (m < 60).then(|| clock(number(h)?, m, s)).flatten()
        }
        [h, m, s, f] => frames(number(h)?, number(m)?, number(s)?, number(f)?, drop, fps),
        _ => None,
    }
}

/// A whole number of digits (no sign, no point).
fn number(text: &str) -> Option<u64> {
    (!text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()))
        .then(|| text.parse().ok())
        .flatten()
}

/// `65.25`, `.5`, `7`: seconds with an optional fraction.
fn seconds(text: &str) -> Option<u64> {
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    if whole.is_empty() && fraction.is_empty() {
        return None;
    }
    let whole = if whole.is_empty() { 0 } else { number(whole)? };
    whole.checked_mul(1000)?.checked_add(millis(fraction)?)
}

/// The digits after the point as milliseconds (`5` is 500, `2501` is 250).
fn millis(fraction: &str) -> Option<u64> {
    if fraction.is_empty() {
        return Some(0);
    }
    number(fraction)?;
    format!("{fraction:0<3}")[..3].parse().ok()
}

/// `h`, `m` and the `ss[.mmm]` field.
fn clock(hours: u64, minutes: u64, seconds_field: &str) -> Option<u64> {
    let (s, fraction) = seconds_field.split_once('.').unwrap_or((seconds_field, ""));
    let s = number(s)?;
    if s >= 60 {
        return None;
    }
    let total = ((hours * 60 + minutes) * 60 + s).checked_mul(1000)?;
    total.checked_add(millis(fraction)?)
}

/// `H:MM:SS:FF` (or `;`): frames at `fps`, drop-frame at 29.97 / 59.94 with `;`.
fn frames(h: u64, m: u64, s: u64, f: u64, drop: bool, fps: f64) -> Option<u64> {
    let nominal = fps.round() as u64;
    if m >= 60 || s >= 60 || f >= nominal {
        return None;
    }
    let drop_frames = match nominal {
        30 if (fps - 29.97).abs() < 0.01 => 2,
        60 if (fps - 59.94).abs() < 0.01 => 4,
        _ => 0,
    };
    let mut count = ((h * 60 + m) * 60 + s) * nominal + f;
    if drop && drop_frames > 0 {
        // Frames 0..drop of every minute not divisible by ten do not exist.
        if s == 0 && f < drop_frames && !m.is_multiple_of(10) {
            return None;
        }
        let minutes = h * 60 + m;
        count -= drop_frames * (minutes - minutes / 10);
    }
    Some((count as f64 * 1000.0 / fps).round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_style_reads_the_same_moment() {
        for text in ["00:01:05.250", "1:05.25", "65.25", "1:05.250"] {
            assert_eq!(parse(text, 25.0), Some(65_250), "{text}");
        }
    }

    #[test]
    fn short_forms() {
        assert_eq!(parse("1:05", 25.0), Some(65_000));
        assert_eq!(parse("1:02:05", 25.0), Some(3_725_000));
        assert_eq!(parse("90", 25.0), Some(90_000));
        assert_eq!(parse(".5", 25.0), Some(500));
        assert_eq!(parse("  7.5  ", 25.0), Some(7_500));
        // Minutes of M:SS are not limited to 59.
        assert_eq!(parse("75:00", 25.0), Some(4_500_000));
    }

    #[test]
    fn frames_count_at_the_clips_frame_rate() {
        assert_eq!(parse("00:01:05:12", 25.0), Some(65_480));
        assert_eq!(parse("00:00:01:12", 24.0), Some(1_500));
        assert_eq!(parse("00:00:00:25", 25.0), None);
        assert_eq!(parse("00:00:00:24", 25.0), Some(960));
    }

    #[test]
    fn drop_frame_skips_two_frame_numbers_each_minute_but_the_tenth() {
        let fps = 30_000.0 / 1001.0;
        // 00:00:59;29 is frame 1799, so 00:01:00;02 is frame 1800: 1800 / 29.97 fps = 60.06 s.
        assert_eq!(parse("00:01:00;02", fps), Some(60_060));
        assert_eq!(parse("00:01:00;00", fps), None);
        // Ten minutes are 17982 frames, an hour 107892: drop-frame keeps the clock within
        // milliseconds of the wall clock.
        assert_eq!(parse("00:10:00;00", fps), Some(599_999));
        assert_eq!(parse("01:00:00;00", fps), Some(3_599_996));
        // Without the drop, the same digits count 30 frames a second.
        assert_eq!(parse("00:00:01:00", fps), Some(1_001));
    }

    #[test]
    fn out_of_range_and_junk_are_refused() {
        for text in [
            "",
            "  ",
            "abc",
            "1:60",
            "1:75:00",
            "00:00:60",
            "-5",
            "1.2.3",
            "1::2",
            "12:",
            ":30",
            "1;2",
            "00:01:05.25:12",
        ] {
            assert_eq!(parse(text, 25.0), None, "{text:?}");
        }
        assert_eq!(parse("1:05", 0.0), None);
    }

    #[test]
    fn copy_then_paste_lands_on_the_same_moment() {
        for ms in [0, 1, 999, 1_000, 65_250, 3_725_007, 36_000_000] {
            assert_eq!(parse(&format(ms), 25.0), Some(ms));
        }
        assert_eq!(format(65_250), "00:01:05.250");
    }

    fn shown(style: TimecodeStyle, fps: f64, ms: u64) -> String {
        TimeFormat::new(style, Some(fps)).copied(ms)
    }

    #[test]
    fn whole_frame_rates_count_frames_at_the_nominal_rate() {
        // 1 h 1 min 5 s and 12 frames' worth of milliseconds at each rate.
        for (fps, frame_ms) in [
            (24.0, 500),
            (25.0, 480),
            (30.0, 400),
            (50.0, 240),
            (60.0, 200),
        ] {
            let ms = 3_665_000 + frame_ms;
            let want = format!(
                "01:01:05:{:02}",
                (frame_ms as f64 * fps / 1000.0).round() as u64
            );
            assert_eq!(shown(TimecodeStyle::Premiere, fps, ms), want, "{fps}");
            assert_eq!(shown(TimecodeStyle::Resolve, fps, ms), want, "{fps}");
        }
    }

    #[test]
    fn twenty_three_nine_seven_six_counts_twenty_four_frames_without_dropping() {
        let fps = 24_000.0 / 1001.0;
        assert_eq!(shown(TimecodeStyle::Premiere, fps, 0), "00:00:00:00");
        // Frame 24 is the first of second 1.
        assert_eq!(shown(TimecodeStyle::Premiere, fps, 1_001), "00:00:01:00");
        assert_eq!(
            shown(TimecodeStyle::Premiere, fps, 1_001 - 42),
            "00:00:00:23"
        );
    }

    #[test]
    fn twenty_nine_nine_seven_is_drop_frame_with_each_styles_separator() {
        let fps = 30_000.0 / 1001.0;
        // Frame 1799 is 00:00:59;29 and frame 1800 skips to 00:01:00;02.
        assert_eq!(shown(TimecodeStyle::Resolve, fps, 60_027), "00:00:59;29");
        assert_eq!(shown(TimecodeStyle::Resolve, fps, 60_060), "00:01:00;02");
        assert_eq!(shown(TimecodeStyle::Premiere, fps, 60_060), "00;01;00;02");
        // The tenth minute keeps its first frames.
        assert_eq!(shown(TimecodeStyle::Resolve, fps, 599_999), "00:10:00;00");
        assert_eq!(shown(TimecodeStyle::Resolve, fps, 3_599_996), "01:00:00;00");
    }

    #[test]
    fn fifty_nine_nine_four_drops_four_frames_a_minute() {
        let fps = 60_000.0 / 1001.0;
        assert_eq!(shown(TimecodeStyle::Resolve, fps, 60_060), "00:01:00;04");
        assert_eq!(shown(TimecodeStyle::Resolve, fps, 599_999), "00:10:00;00");
    }

    #[test]
    fn what_is_shown_is_read_back_to_the_same_frame() {
        for fps in [
            24_000.0 / 1001.0,
            24.0,
            25.0,
            30_000.0 / 1001.0,
            30.0,
            50.0,
            60_000.0 / 1001.0,
            60.0,
        ] {
            for style in [TimecodeStyle::Premiere, TimecodeStyle::Resolve] {
                for ms in [0, 1_000, 59_993, 60_060, 599_999, 3_725_007, 36_000_000] {
                    let text = shown(style, fps, ms);
                    let back = parse(&text, fps).unwrap_or_else(|| panic!("{text} at {fps}"));
                    // Within the frame the moment fell in.
                    assert!(
                        (back as f64 - ms as f64).abs() <= 1000.0 / fps,
                        "{text} {back} {ms}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_clip_without_a_steady_frame_rate_keeps_the_default_style() {
        for fps in [None, Some(0.0), Some(f64::NAN), Some(-25.0)] {
            let format = TimeFormat::new(TimecodeStyle::Premiere, fps);
            assert_eq!(format.effective_style(), TimecodeStyle::Default);
            assert_eq!(format.copied(65_250), "00:01:05.250");
        }
        let steady = TimeFormat::new(TimecodeStyle::Resolve, Some(25.0));
        assert_eq!(steady.effective_style(), TimecodeStyle::Resolve);
    }

    #[test]
    fn the_style_name_is_stable_and_unknown_names_are_the_default() {
        for style in TimecodeStyle::ALL {
            assert_eq!(TimecodeStyle::from_name(style.as_str()), style);
        }
        assert_eq!(TimecodeStyle::from_name("avid"), TimecodeStyle::Default);
    }
}
