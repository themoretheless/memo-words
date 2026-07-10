//! Reveal choreography and exit-window boundaries.

use crate::config::Config;
use std::time::Duration;

const RECALL_REVEAL_FRACTION: f32 = 0.55;

pub fn effective_translation_delay(cfg: &Config) -> f32 {
    if cfg.recall_mode {
        (cfg.interval_secs as f32 * RECALL_REVEAL_FRACTION).max(cfg.translation_delay)
    } else {
        cfg.translation_delay
    }
}

pub fn example_delay(cfg: &Config) -> f32 {
    effective_translation_delay(cfg) + cfg.fade_duration
}

pub fn anim_end(cfg: &Config, has_example: bool) -> f32 {
    let mut last = cfg
        .transcription_delay
        .max(effective_translation_delay(cfg));
    if has_example {
        last = last.max(example_delay(cfg));
    }
    last + cfg.fade_duration
}

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
    fn recall_mode_delays_translation_but_never_advances_it() {
        let cfg = Config {
            translation_delay: 10.0,
            interval_secs: 30,
            recall_mode: true,
            ..Config::default()
        };
        assert_eq!(effective_translation_delay(&cfg), 16.5);

        let short = Config {
            interval_secs: 8,
            ..cfg
        };
        assert_eq!(effective_translation_delay(&short), 10.0);
    }

    #[test]
    fn normal_mode_keeps_configured_delay() {
        let cfg = Config {
            translation_delay: 10.0,
            recall_mode: false,
            ..Config::default()
        };
        assert_eq!(effective_translation_delay(&cfg), 10.0);
    }

    #[test]
    fn example_extends_animation_end() {
        let cfg = Config {
            transcription_delay: 5.0,
            translation_delay: 10.0,
            fade_duration: 1.0,
            ..Config::default()
        };
        assert_eq!(example_delay(&cfg), 11.0);
        assert_eq!(anim_end(&cfg, false), 11.0);
        assert_eq!(anim_end(&cfg, true), 12.0);
    }

    #[test]
    fn later_transcription_controls_animation_end() {
        let cfg = Config {
            transcription_delay: 15.0,
            translation_delay: 10.0,
            fade_duration: 1.0,
            ..Config::default()
        };
        assert_eq!(anim_end(&cfg, false), 16.0);
    }

    #[test]
    fn exit_window_is_disabled_at_zero() {
        let cfg = Config {
            exit_duration: 0.0,
            ..Config::default()
        };
        assert_eq!(exit_window(&cfg, Duration::from_secs(30)), Duration::ZERO);
    }

    #[test]
    fn exit_window_is_capped_at_half_the_interval() {
        let cfg = Config {
            exit_duration: 5.0,
            ..Config::default()
        };
        assert_eq!(
            exit_window(&cfg, Duration::from_secs(4)),
            Duration::from_secs(2)
        );
    }
}
