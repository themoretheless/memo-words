//! Pure choreography and pacing math: the single source of truth for *when*
//! things happen on the card. No egui, no `App` state, no I/O, just functions
//! over elapsed time and the relevant [`Config`] values. Both the repaint
//! scheduler (`app`) and the renderer (`ui`) consume these, so the timeline is
//! defined once instead of being re-derived in two places, and the whole thing
//! is unit-testable without constructing a window or an `App`.

use crate::config::Config;
use std::time::Duration;

/// In recall mode the translation is held back to this fraction of the interval
/// (capped below by the configured `translation_delay`) so there's a real window
/// to recall the meaning before it's revealed. 0.55 lands the reveal just past
/// the midpoint, leaving roughly the same span again to absorb the answer.
const RECALL_REVEAL_FRACTION: f32 = 0.55;

// --- Easing primitives ---------------------------------------------------

/// Smooth Hermite ease over `0..=1` (clamped), the basis for every fade.
pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Fade-in progress (0..1) of a line at `elapsed`, given when it starts (`delay`)
/// and how long it fades (`fade_duration`).
pub fn fade_factor(elapsed: f32, delay: f32, fade_duration: f32) -> f32 {
    smoothstep((elapsed - delay) / fade_duration)
}

/// Whole-card opacity multiplier (1..0) for the exit fade. `until_next` is the
/// seconds left before the next word; once it drops below `exit_duration` the
/// card eases out, reaching 0 exactly at the swap. With `exit_duration <= 0` the
/// card never fades.
pub fn exit_alpha(until_next: f32, exit_duration: f32) -> f32 {
    if exit_duration <= 0.0 || until_next >= exit_duration {
        return 1.0;
    }
    let progress = 1.0 - (until_next.max(0.0) / exit_duration);
    1.0 - smoothstep(progress)
}

/// Vertical entrance offset for a line at fade progress `ease`: it starts
/// `settle_px` points low and rises to rest (0) as it fades in. `settle_px == 0`
/// is always 0 (the entrance is off).
pub fn settle_offset(settle_px: f32, ease: f32) -> f32 {
    settle_px * (1.0 - ease.clamp(0.0, 1.0))
}

// --- Pacing --------------------------------------------------------------

/// How "rare" a word is on a 0.0..=1.0 scale from its frequency rank (1 = most
/// common). Rank 1 -> 0.0 (no extra dwell); rarer ranks rise toward 1.0. An
/// unknown/absent rank (<= 0, e.g. a MongoDB doc without the field) reads as
/// rarest (1.0), matching `selector::weight`'s convention.
pub fn difficulty_factor(frequency: i32) -> f32 {
    if frequency >= 1 {
        1.0 - 1.0 / frequency as f32
    } else {
        1.0
    }
}

/// The base display interval in whole seconds for a word, before jitter: the
/// configured `interval_secs` optionally stretched for rarer words by
/// `rare_word_dwell` so harder vocab lingers longer. The random jitter is applied
/// by the caller (it needs an RNG), keeping this part pure and testable.
pub fn dwelled_base_secs(cfg: &Config, frequency: i32) -> i64 {
    let mult = 1.0 + cfg.rare_word_dwell * difficulty_factor(frequency);
    (cfg.interval_secs as f64 * mult as f64).round() as i64
}

// --- Reveal timeline -----------------------------------------------------

/// When the translation line starts fading in. Normally `translation_delay`; in
/// recall mode it's pushed back to `RECALL_REVEAL_FRACTION` of the interval so
/// the meaning stays hidden long enough to recall it first. Capped below by
/// `translation_delay`, so recall mode only ever delays the reveal.
pub fn effective_translation_delay(cfg: &Config) -> f32 {
    if cfg.recall_mode {
        let late = cfg.interval_secs as f32 * RECALL_REVEAL_FRACTION;
        late.max(cfg.translation_delay)
    } else {
        cfg.translation_delay
    }
}

/// When the example line starts fading in: just after the translation settles,
/// so the lines reveal in sequence. Follows the effective translation delay, so
/// recall mode pushes the example back in lockstep with the meaning.
pub fn example_delay(cfg: &Config) -> f32 {
    effective_translation_delay(cfg) + cfg.fade_duration
}

/// Elapsed seconds at which the card is fully settled and repaints can stop. The
/// lines fade in at independent delays, so the card isn't done until the LAST one
/// finishes; the example delay only counts when the current word has one.
pub fn anim_end(cfg: &Config, has_example: bool) -> f32 {
    let mut last = cfg
        .transcription_delay
        .max(effective_translation_delay(cfg));
    if has_example {
        last = last.max(example_delay(cfg));
    }
    last + cfg.fade_duration
}

/// How long the card spends fading out before the next word. Zero (the default)
/// means a hard cut. Capped at half `word_interval` so the fade never eats the
/// whole word, however large `exit_duration` is configured.
pub fn exit_window(cfg: &Config, word_interval: Duration) -> Duration {
    if cfg.exit_duration <= 0.0 {
        return Duration::ZERO;
    }
    let cap = word_interval.as_secs_f32() * 0.5;
    Duration::from_secs_f32(cfg.exit_duration.min(cap).max(0.0))
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
    fn fade_factor_is_zero_before_delay_and_one_after() {
        let (delay, fade) = (5.0, 1.0);
        assert_eq!(fade_factor(0.0, delay, fade), 0.0);
        assert_eq!(fade_factor(delay, delay, fade), 0.0);
        assert_eq!(fade_factor(delay + fade, delay, fade), 1.0);
        assert_eq!(fade_factor(delay + 10.0, delay, fade), 1.0);
        assert!((fade_factor(delay + 0.5 * fade, delay, fade) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn exit_alpha_off_and_before_window_is_full() {
        assert_eq!(exit_alpha(5.0, 0.0), 1.0);
        assert_eq!(exit_alpha(0.0, 0.0), 1.0);
        assert_eq!(exit_alpha(2.0, 0.5), 1.0);
        assert_eq!(exit_alpha(0.5, 0.5), 1.0); // exactly at the window edge
    }

    #[test]
    fn exit_alpha_eases_to_zero_at_the_swap() {
        let dur = 0.5;
        assert!((exit_alpha(0.25, dur) - 0.5).abs() < 1e-6);
        assert_eq!(exit_alpha(0.0, dur), 0.0);
        assert!(exit_alpha(0.1, dur) < exit_alpha(0.4, dur));
    }

    #[test]
    fn settle_offset_drifts_from_full_to_zero() {
        assert_eq!(settle_offset(0.0, 0.0), 0.0);
        assert_eq!(settle_offset(0.0, 0.5), 0.0);
        assert_eq!(settle_offset(6.0, 0.0), 6.0);
        assert_eq!(settle_offset(6.0, 1.0), 0.0);
        assert!((settle_offset(6.0, 0.5) - 3.0).abs() < 1e-6);
        assert!(settle_offset(6.0, 0.75) < settle_offset(6.0, 0.25));
    }

    #[test]
    fn difficulty_factor_ranks_rarity() {
        assert_eq!(difficulty_factor(1), 0.0);
        assert_eq!(difficulty_factor(2), 0.5);
        assert_eq!(difficulty_factor(0), 1.0);
        assert_eq!(difficulty_factor(-5), 1.0);
        assert!(difficulty_factor(100) > difficulty_factor(2));
        for f in [-3, 0, 1, 2, 10, 100, 1000] {
            let v = difficulty_factor(f);
            assert!((0.0..=1.0).contains(&v), "factor {v} out of [0,1] for {f}");
        }
    }

    #[test]
    fn dwelled_base_off_keeps_uniform() {
        let cfg = Config {
            interval_secs: 30,
            rare_word_dwell: 0.0,
            ..Config::default()
        };
        assert_eq!(dwelled_base_secs(&cfg, 1), 30);
        assert_eq!(dwelled_base_secs(&cfg, 1000), 30);
        assert_eq!(dwelled_base_secs(&cfg, 0), 30);
    }

    #[test]
    fn dwelled_base_stretches_rarer_words() {
        let cfg = Config {
            interval_secs: 30,
            rare_word_dwell: 1.0,
            ..Config::default()
        };
        assert_eq!(dwelled_base_secs(&cfg, 1), 30); // common: unchanged
        assert_eq!(dwelled_base_secs(&cfg, 2), 45); // 30 * 1.5
        assert_eq!(dwelled_base_secs(&cfg, 0), 60); // rarest: 30 * 2.0
        assert!(dwelled_base_secs(&cfg, 2) > dwelled_base_secs(&cfg, 1));
    }

    #[test]
    fn effective_translation_delay_unchanged_without_recall() {
        let cfg = Config {
            translation_delay: 10.0,
            interval_secs: 30,
            recall_mode: false,
            ..Config::default()
        };
        assert_eq!(effective_translation_delay(&cfg), 10.0);
    }

    #[test]
    fn recall_mode_delays_translation_to_late_in_interval() {
        let cfg = Config {
            translation_delay: 10.0,
            interval_secs: 30,
            recall_mode: true,
            ..Config::default()
        };
        // 30 * 0.55 = 16.5, later than the 10s default, so the late reveal wins.
        assert_eq!(effective_translation_delay(&cfg), 16.5);
    }

    #[test]
    fn recall_mode_never_earlier_than_configured_delay() {
        // Short interval: 8 * 0.55 = 4.4, earlier than the 10s translation_delay.
        let cfg = Config {
            translation_delay: 10.0,
            interval_secs: 8,
            recall_mode: true,
            ..Config::default()
        };
        assert_eq!(effective_translation_delay(&cfg), 10.0);
    }

    #[test]
    fn recall_mode_pushes_example_and_anim_end() {
        let cfg = Config {
            transcription_delay: 5.0,
            translation_delay: 10.0,
            fade_duration: 1.0,
            interval_secs: 30,
            recall_mode: true,
            ..Config::default()
        };
        // translation at 16.5, example at 17.5, repaints end at 18.5.
        assert_eq!(effective_translation_delay(&cfg), 16.5);
        assert_eq!(example_delay(&cfg), 17.5);
        assert_eq!(anim_end(&cfg, true), 18.5);
    }

    #[test]
    fn anim_end_uses_the_later_fade() {
        let cfg = Config {
            transcription_delay: 5.0,
            translation_delay: 10.0,
            fade_duration: 1.0,
            ..Config::default()
        };
        assert_eq!(anim_end(&cfg, false), 11.0);
    }

    #[test]
    fn anim_end_covers_a_late_transcription_fade() {
        let cfg = Config {
            transcription_delay: 15.0,
            translation_delay: 10.0,
            fade_duration: 1.0,
            ..Config::default()
        };
        assert_eq!(anim_end(&cfg, false), 16.0);
    }

    #[test]
    fn anim_end_extends_for_the_example_line() {
        let cfg = Config {
            transcription_delay: 5.0,
            translation_delay: 10.0,
            fade_duration: 1.0,
            ..Config::default()
        };
        // example_delay = 10 + 1 = 11; end = 11 + 1 = 12.
        assert_eq!(anim_end(&cfg, true), 12.0);
        assert_eq!(anim_end(&cfg, false), 11.0);
    }

    #[test]
    fn exit_window_is_zero_when_disabled() {
        let cfg = Config {
            exit_duration: 0.0,
            interval_secs: 30,
            ..Config::default()
        };
        assert_eq!(exit_window(&cfg, Duration::from_secs(30)), Duration::ZERO);
    }

    #[test]
    fn exit_window_used_as_is_when_it_fits() {
        // 0.4s fits well within half of a 30s interval, so it's used unchanged.
        let cfg = Config {
            exit_duration: 0.4,
            interval_secs: 30,
            ..Config::default()
        };
        assert_eq!(
            exit_window(&cfg, Duration::from_secs(30)),
            Duration::from_secs_f32(0.4)
        );
    }

    #[test]
    fn exit_window_capped_at_half_a_short_interval() {
        // A 5s exit on a 4s interval would eat the whole word; cap at half = 2s.
        let cfg = Config {
            exit_duration: 5.0,
            interval_secs: 4,
            ..Config::default()
        };
        assert_eq!(
            exit_window(&cfg, Duration::from_secs(4)),
            Duration::from_secs_f32(2.0)
        );
    }
}
