//! Small, stateless easing primitives shared by layout and painting.

/// Smooth Hermite ease over `0..=1` (clamped), the basis for every fade.
pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Fade-in progress of a line at `elapsed`.
pub fn fade_factor(elapsed: f32, delay: f32, fade_duration: f32) -> f32 {
    smoothstep((elapsed - delay) / fade_duration)
}

/// Whole-card opacity multiplier during the exit window.
///
/// Pausing intentionally restores full opacity: a paused learning card should
/// remain readable even if the pause happened midway through its exit.
pub fn exit_alpha(until_next: f32, exit_duration: f32, paused: bool) -> f32 {
    if paused || exit_duration <= 0.0 || until_next >= exit_duration {
        return 1.0;
    }
    let progress = 1.0 - (until_next.max(0.0) / exit_duration);
    1.0 - smoothstep(progress)
}

/// Vertical entrance offset for a line at fade progress `ease`.
pub fn settle_offset(settle_px: f32, ease: f32) -> f32 {
    settle_px * (1.0 - ease.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoothstep_clamps_and_eases() {
        assert_eq!(smoothstep(-1.0), 0.0);
        assert_eq!(smoothstep(0.0), 0.0);
        assert_eq!(smoothstep(1.0), 1.0);
        assert_eq!(smoothstep(2.0), 1.0);
        assert!((smoothstep(0.5) - 0.5).abs() < 1e-6);
        assert!(smoothstep(0.25) < smoothstep(0.75));
    }

    #[test]
    fn fade_factor_obeys_delay_and_duration() {
        let (delay, fade) = (5.0, 1.0);
        assert_eq!(fade_factor(0.0, delay, fade), 0.0);
        assert_eq!(fade_factor(delay, delay, fade), 0.0);
        assert_eq!(fade_factor(delay + fade, delay, fade), 1.0);
        assert!((fade_factor(delay + 0.5 * fade, delay, fade) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn exit_alpha_is_full_before_or_without_window() {
        assert_eq!(exit_alpha(5.0, 0.0, false), 1.0);
        assert_eq!(exit_alpha(2.0, 0.5, false), 1.0);
        assert_eq!(exit_alpha(0.5, 0.5, false), 1.0);
    }

    #[test]
    fn exit_alpha_eases_to_zero() {
        assert!((exit_alpha(0.25, 0.5, false) - 0.5).abs() < 1e-6);
        assert_eq!(exit_alpha(0.0, 0.5, false), 0.0);
    }

    #[test]
    fn exit_alpha_is_readable_while_paused() {
        assert_eq!(exit_alpha(0.0, 0.5, true), 1.0);
        assert_eq!(exit_alpha(0.25, 0.5, true), 1.0);
    }

    #[test]
    fn settle_offset_drifts_to_rest() {
        assert_eq!(settle_offset(0.0, 0.5), 0.0);
        assert_eq!(settle_offset(6.0, 0.0), 6.0);
        assert_eq!(settle_offset(6.0, 1.0), 0.0);
        assert!((settle_offset(6.0, 0.5) - 3.0).abs() < 1e-6);
    }
}
