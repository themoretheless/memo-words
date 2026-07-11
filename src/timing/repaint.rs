//! Repaint scheduling, isolated because it protects the zero-idle invariant.

use std::time::Duration;

const PAUSED_SLEEP: Duration = Duration::from_secs(3600);

pub fn repaint_after(
    elapsed: f32,
    anim_end: f32,
    paused: bool,
    until_next: Duration,
    exit_window: Duration,
    anim_frame: Duration,
) -> Duration {
    // Pause wins over animation. SessionClock freezes elapsed while paused, so
    // animating first would request 60 fps forever at the same frozen frame.
    if paused {
        PAUSED_SLEEP
    } else if elapsed < anim_end || until_next <= exit_window {
        anim_frame
    } else {
        until_next.saturating_sub(exit_window)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: Duration = Duration::from_millis(16);

    #[test]
    fn fading_content_uses_frame_cadence() {
        assert_eq!(
            repaint_after(
                1.0,
                11.0,
                false,
                Duration::from_secs(29),
                Duration::ZERO,
                FRAME
            ),
            FRAME
        );
    }

    #[test]
    fn settled_content_sleeps_until_next_event() {
        let until = Duration::from_secs(29);
        assert_eq!(
            repaint_after(20.0, 11.0, false, until, Duration::ZERO, FRAME),
            until
        );
    }

    #[test]
    fn pause_sleeps_even_when_animation_was_in_progress() {
        let delay = repaint_after(
            1.0,
            11.0,
            true,
            Duration::from_secs(29),
            Duration::ZERO,
            FRAME,
        );
        assert!(delay >= Duration::from_secs(3600));
    }

    #[test]
    fn exit_window_uses_frame_cadence() {
        assert_eq!(
            repaint_after(
                20.0,
                11.0,
                false,
                Duration::from_millis(300),
                Duration::from_millis(500),
                FRAME,
            ),
            FRAME
        );
    }

    #[test]
    fn settled_content_sleeps_until_exit_window() {
        let until = Duration::from_secs(29);
        let exit = Duration::from_millis(500);
        assert_eq!(
            repaint_after(20.0, 11.0, false, until, exit, FRAME),
            until.saturating_sub(exit)
        );
    }
}
