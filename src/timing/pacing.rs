//! Difficulty and display-duration math.

use crate::config::Config;

/// Rarity on a `0.0..=1.0` scale from frequency rank (`1` is most common).
pub fn difficulty_factor(frequency: i32) -> f32 {
    if frequency >= 1 {
        1.0 - 1.0 / frequency as f32
    } else {
        1.0
    }
}

/// Display interval before random jitter is applied by the application shell.
pub fn dwelled_base_secs(cfg: &Config, frequency: i32) -> i64 {
    let mult = 1.0 + cfg.rare_word_dwell * difficulty_factor(frequency);
    (cfg.interval_secs as f64 * mult as f64).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn difficulty_factor_ranks_rarity() {
        assert_eq!(difficulty_factor(1), 0.0);
        assert_eq!(difficulty_factor(2), 0.5);
        assert_eq!(difficulty_factor(0), 1.0);
        assert!(difficulty_factor(100) > difficulty_factor(2));
    }

    #[test]
    fn dwell_off_keeps_uniform_timing() {
        let cfg = Config {
            interval_secs: 30,
            rare_word_dwell: 0.0,
            ..Config::default()
        };
        assert_eq!(dwelled_base_secs(&cfg, 1), 30);
        assert_eq!(dwelled_base_secs(&cfg, 1000), 30);
    }

    #[test]
    fn dwell_stretches_rarer_words() {
        let cfg = Config {
            interval_secs: 30,
            rare_word_dwell: 1.0,
            ..Config::default()
        };
        assert_eq!(dwelled_base_secs(&cfg, 1), 30);
        assert_eq!(dwelled_base_secs(&cfg, 2), 45);
        assert_eq!(dwelled_base_secs(&cfg, 0), 60);
    }
}
