//! One frame back or forward (#162): where to seek from the frame on screen.
//!
//! Steps go by the timestamps of the decoded frames, not by an assumed `1 / fps`, so a
//! variable-frame-rate clip (an iPhone recording) steps to its real previous or next frame.
//!
//! A step forward asks GStreamer for the next decoded frame (a step event): cheap even in a clip
//! with long groups of pictures, where every seek decodes from the previous keyframe. A step back
//! has to seek. An accurate seek clips the frame it lands in to the seek's edge: played forward,
//! the frame's timestamp becomes the seek target (its end stays true); played backward, its end
//! becomes the seek's stop (its start stays true). So a step back seeks backward to stop at the
//! true start of the frame on screen, which shows the frame before it, whole.

use std::time::Duration;

use crate::features::video_controls::FrameStep;

/// The frame on screen, in stream time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShownFrame {
    /// Its timestamp. True unless `!start_exact`.
    pub start: Duration,
    /// How long it shows from `start`; `None` when the decoder did not say.
    pub duration: Option<Duration>,
    /// False when a forward seek may have clipped its start to the seek target: its true start
    /// is then earlier.
    pub start_exact: bool,
    /// Shown by a backward seek (a step back): the pipeline runs backward until a seek turns it
    /// forward again.
    pub reversed: bool,
}

impl ShownFrame {
    /// Where it ends, which is always true: the next frame's start.
    pub fn end(self) -> Option<Duration> {
        self.duration.map(|duration| self.start + duration)
    }
}

/// What a step does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepTarget {
    /// An accurate forward seek to this time: the frame starting there. Only after a step back,
    /// when the pipeline runs backward and a step event would go backward too.
    Forward(Duration),
    /// An accurate backward seek stopping at this time: the frame just before it.
    Backward(Duration),
    /// A step event: the next decoded frame.
    NextBuffer,
    /// At the first or last frame: nothing.
    Stay,
}

/// Where a step from `frame` goes in a clip `clip` long.
///
/// Back from a frame whose start is not known exactly, the step first shows that same frame
/// from its true start (a backward seek stopping at its end); a second step from there goes to
/// the frame before.
pub fn step_target(frame: ShownFrame, step: FrameStep, clip: Duration) -> StepTarget {
    match step {
        FrameStep::Forward => match frame.end() {
            Some(end) if end >= clip => StepTarget::Stay,
            Some(end) if frame.reversed => StepTarget::Forward(end),
            None if frame.reversed || frame.start >= clip => StepTarget::Stay,
            _ => StepTarget::NextBuffer,
        },
        FrameStep::Back if !frame.start_exact => match frame.end() {
            Some(end) => StepTarget::Backward(end),
            None => StepTarget::Backward(frame.start),
        },
        FrameStep::Back if frame.start.is_zero() => StepTarget::Stay,
        FrameStep::Back => StepTarget::Backward(frame.start),
    }
}

/// A frame a step event showed: its timestamp is its true start. The sink reports a segment
/// starting there, as after a forward seek that may have clipped it, so it would read as
/// inexact and a step back from it would first seek to find its start.
pub fn stepped_onto(frame: ShownFrame) -> ShownFrame {
    ShownFrame {
        start_exact: true,
        ..frame
    }
}

/// Whether a step from `from` that showed `to` only found `from`'s true start (the first half
/// of a step back): the same frame, so the step goes on.
pub fn only_found_the_start(from: ShownFrame, to: ShownFrame) -> bool {
    !from.start_exact && to.start_exact && to.end().is_some() && to.end() == from.end()
}

/// The playhead for a frame: its start rounded up to a whole millisecond, so a marker set there
/// (markers are whole milliseconds) seeks back onto this frame, not the one before; rounded
/// down instead when rounding up would leave the frame.
pub fn frame_position(frame: ShownFrame) -> Duration {
    let nanos = frame.start.as_nanos();
    let up = Duration::from_millis(u64::try_from(nanos.div_ceil(1_000_000)).unwrap_or(u64::MAX));
    match frame.end() {
        Some(end) if up >= end => {
            Duration::from_millis(u64::try_from(nanos / 1_000_000).unwrap_or(u64::MAX))
        }
        _ => up,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A clip as a decoder shows it after accurate seeks: the start of each frame, the last one
    /// ending at `end`.
    struct Clip {
        starts: Vec<Duration>,
        end: Duration,
    }

    impl Clip {
        fn bounds(&self, index: usize) -> (Duration, Duration) {
            let next = self.starts.get(index + 1).copied().unwrap_or(self.end);
            (self.starts[index], next)
        }

        /// A forward seek to `target`: the frame holding it, its start clipped to the target.
        fn forward(&self, target: Duration) -> ShownFrame {
            let index = self
                .starts
                .iter()
                .rposition(|&start| start <= target)
                .unwrap_or(0);
            let (start, end) = self.bounds(index);
            let shown = start.max(target);
            ShownFrame {
                start: shown,
                duration: Some(end - shown),
                start_exact: shown != target,
                reversed: false,
            }
        }

        /// A backward seek stopping at `stop`: the frame just before it, its end clipped to the
        /// stop; `None` when there is none (the first frame).
        fn backward(&self, stop: Duration) -> Option<ShownFrame> {
            let index = self.starts.iter().rposition(|&start| start < stop)?;
            let (start, end) = self.bounds(index);
            Some(ShownFrame {
                start,
                duration: Some(end.min(stop) - start),
                start_exact: true,
                reversed: true,
            })
        }

        /// A step event from `frame`: the next frame, whole, reported with a segment starting
        /// at it (so inexact), and marked exact the way the player does.
        fn next(&self, frame: ShownFrame) -> ShownFrame {
            let (start, end) = self.bounds(self.index_of(frame) + 1);
            stepped_onto(ShownFrame {
                start,
                duration: Some(end - start),
                start_exact: false,
                reversed: false,
            })
        }

        /// One step as the player takes it, including the second half of a step back.
        fn step(&self, frame: ShownFrame, step: FrameStep) -> ShownFrame {
            let shown = match step_target(frame, step, self.end) {
                StepTarget::Forward(target) => self.forward(target),
                StepTarget::Backward(stop) => self.backward(stop).unwrap_or(frame),
                StepTarget::NextBuffer => self.next(frame),
                StepTarget::Stay => frame,
            };
            if only_found_the_start(frame, shown) {
                return self.step(shown, step);
            }
            shown
        }

        /// The index of the frame on screen, by its end (always true).
        fn index_of(&self, frame: ShownFrame) -> usize {
            let end = frame.end().expect("a length");
            (0..self.starts.len())
                .find(|&index| self.bounds(index).1 == end)
                .expect("a frame ends there")
        }
    }

    fn nanos(n: u64) -> Duration {
        Duration::from_nanos(n)
    }

    /// 29.97 fps: timestamps that never fall on whole milliseconds.
    fn constant() -> Clip {
        let starts = (0..30u64).map(|i| nanos(i * 1_001_000_000 / 30)).collect();
        Clip {
            starts,
            end: nanos(30 * 1_001_000_000 / 30),
        }
    }

    /// An iPhone-style variable rate: frames of 16, 33 and 50 ms mixed.
    fn variable() -> Clip {
        let lengths = [
            33_366_667u64,
            16_683_333,
            50_050_000,
            16_683_334,
            33_366_666,
        ];
        let mut starts = vec![Duration::ZERO];
        for length in lengths {
            let last = *starts.last().expect("a start");
            starts.push(last + nanos(length));
        }
        let end = starts.pop().expect("an end");
        Clip { starts, end }
    }

    /// From a frame a seek landed in mid-way (a marker, the bar): forward to the last frame,
    /// back to the first, and every step moves exactly one frame.
    fn walk(clip: &Clip, from: usize) {
        let (start, end) = clip.bounds(from);
        let mut frame = clip.forward(start + (end - start) / 2);
        assert!(!frame.start_exact);
        let last = clip.starts.len() - 1;
        for expected in from + 1..=last {
            frame = clip.step(frame, FrameStep::Forward);
            assert_eq!(clip.index_of(frame), expected);
        }
        assert_eq!(
            clip.step(frame, FrameStep::Forward),
            frame,
            "the last stays"
        );
        for expected in (0..last).rev() {
            frame = clip.step(frame, FrameStep::Back);
            assert_eq!(clip.index_of(frame), expected);
            assert_eq!(
                frame.start, clip.starts[expected],
                "a step back finds the start"
            );
        }
        assert_eq!(clip.step(frame, FrameStep::Back), frame, "the first stays");
        // Back from a frame a seek clipped: the frame before it, in one step.
        let mut frame = clip.forward(start + (end - start) / 2);
        if from > 0 {
            frame = clip.step(frame, FrameStep::Back);
            assert_eq!(clip.index_of(frame), from - 1);
        }
    }

    #[test]
    fn steps_visit_every_frame_of_a_constant_rate_clip_and_stop_at_the_ends() {
        walk(&constant(), 0);
        walk(&constant(), 7);
    }

    #[test]
    fn steps_visit_every_frame_of_a_variable_rate_clip_and_stop_at_the_ends() {
        walk(&variable(), 0);
        walk(&variable(), 3);
    }

    #[test]
    fn forward_is_a_step_event_unless_a_step_back_left_the_pipeline_running_backward() {
        let frame = ShownFrame {
            start: Duration::from_secs(1),
            duration: Some(Duration::from_millis(40)),
            start_exact: false,
            reversed: false,
        };
        let clip = Duration::from_secs(5);
        assert_eq!(
            step_target(frame, FrameStep::Forward, clip),
            StepTarget::NextBuffer
        );
        let reversed = ShownFrame {
            reversed: true,
            start_exact: true,
            ..frame
        };
        assert_eq!(
            step_target(reversed, FrameStep::Forward, clip),
            StepTarget::Forward(Duration::from_millis(1_040))
        );
    }

    /// After a step forward, a step back is one backward seek, not two.
    #[test]
    fn back_after_a_step_forward_seeks_once() {
        let clip = constant();
        let shown = clip.next(clip.forward(clip.starts[3]));
        assert_eq!(
            step_target(shown, FrameStep::Back, clip.end),
            StepTarget::Backward(clip.starts[4])
        );
        let reported = ShownFrame {
            start_exact: false,
            ..shown
        };
        assert_eq!(
            step_target(reported, FrameStep::Back, clip.end),
            StepTarget::Backward(clip.bounds(4).1),
            "as the sink reports it, the frame would first be found again"
        );
    }

    #[test]
    fn a_frame_of_unknown_length_steps_forward_by_the_next_decoded_frame() {
        let frame = ShownFrame {
            start: Duration::from_secs(1),
            duration: None,
            start_exact: true,
            reversed: false,
        };
        let clip = Duration::from_secs(5);
        assert_eq!(
            step_target(frame, FrameStep::Forward, clip),
            StepTarget::NextBuffer
        );
        let reversed = ShownFrame {
            reversed: true,
            ..frame
        };
        assert_eq!(
            step_target(reversed, FrameStep::Forward, clip),
            StepTarget::Stay
        );
        assert_eq!(
            step_target(frame, FrameStep::Back, clip),
            StepTarget::Backward(frame.start)
        );
    }

    #[test]
    fn the_playhead_of_a_frame_seeks_back_onto_that_frame() {
        let clip = constant();
        for index in 0..clip.starts.len() {
            let (start, end) = clip.bounds(index);
            for frame in [clip.forward(start), clip.forward(end - nanos(50_000))] {
                let position = frame_position(frame);
                assert!(position >= start && position < end, "{frame:?}");
                assert_eq!(clip.index_of(clip.forward(position)), index);
            }
        }
    }
}
