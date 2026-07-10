//! Reveal choreography and exit-window boundaries.

use crate::config::Config;
use std::time::Duration;

const RECALL_REVEAL_FRACTION: f32 = 0.55;

pub fn effective_translation_delay(cfg: &Config) -> f32 {
    if cfg.learning.recall_mode {
        (cfg.timing.interval_secs as f32 * RECALL_REVEAL_FRACTION).max(cfg.timing.translation_delay)
    } else {
        cfg.timing.translation_delay
    }
}

pub fn example_delay(cfg: &Config) -> f32 {
    effective_translation_delay(cfg) + cfg.timing.fade_duration
}

pub fn anim_end(cfg: &Config, has_example: bool) -> f32 {
    let mut last = cfg
        .timing
        .transcription_delay
        .max(effective_translation_delay(cfg));
    if has_example {
        last = last.max(example_delay(cfg));
    }
    last + cfg.timing.fade_duration
}

pub fn exit_window(cfg: &Config, word_interval: Duration) -> Duration {
    if cfg.accessibility.reduce_motion || cfg.timing.exit_duration <= 0.0 {
        return Duration::ZERO;
    }
    let cap = word_interval.as_secs_f32() * 0.5;
    Duration::from_secs_f32(cfg.timing.exit_duration.min(cap).max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recall_mode_delays_translation_but_never_advances_it() {
        let mut cfg = Config::default();
        cfg.timing.translation_delay = 10.0;
        cfg.timing.interval_secs = 30;
        cfg.learning.recall_mode = true;
        assert_eq!(effective_translation_delay(&cfg), 16.5);

        let mut short = cfg;
        short.timing.interval_secs = 8;
        assert_eq!(effective_translation_delay(&short), 10.0);
    }

    #[test]
    fn normal_mode_keeps_configured_delay() {
        let mut cfg = Config::default();
        cfg.timing.translation_delay = 10.0;
        cfg.learning.recall_mode = false;
        assert_eq!(effective_translation_delay(&cfg), 10.0);
    }

    #[test]
    fn example_extends_animation_end() {
        let mut cfg = Config::default();
        cfg.timing.transcription_delay = 5.0;
        cfg.timing.translation_delay = 10.0;
        cfg.timing.fade_duration = 1.0;
        assert_eq!(example_delay(&cfg), 11.0);
        assert_eq!(anim_end(&cfg, false), 11.0);
        assert_eq!(anim_end(&cfg, true), 12.0);
    }

    #[test]
    fn later_transcription_controls_animation_end() {
        let mut cfg = Config::default();
        cfg.timing.transcription_delay = 15.0;
        cfg.timing.translation_delay = 10.0;
        cfg.timing.fade_duration = 1.0;
        assert_eq!(anim_end(&cfg, false), 16.0);
    }

    #[test]
    fn exit_window_is_disabled_at_zero() {
        let mut cfg = Config::default();
        cfg.timing.exit_duration = 0.0;
        assert_eq!(exit_window(&cfg, Duration::from_secs(30)), Duration::ZERO);
    }

    #[test]
    fn exit_window_is_capped_at_half_the_interval() {
        let mut cfg = Config::default();
        cfg.timing.exit_duration = 5.0;
        assert_eq!(
            exit_window(&cfg, Duration::from_secs(4)),
            Duration::from_secs(2)
        );
    }

    #[test]
    fn reduced_motion_disables_exit_animation() {
        let mut cfg = Config::default();
        cfg.timing.exit_duration = 1.0;
        cfg.accessibility.reduce_motion = true;
        assert_eq!(exit_window(&cfg, Duration::from_secs(30)), Duration::ZERO);
    }
}
