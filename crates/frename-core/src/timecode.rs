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
}
