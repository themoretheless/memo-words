//! Runtime configuration loaded from a simple `key = value` file.
//!
//! Lookup order: the path in `MEMO_CONFIG`, else `$HOME/.config/memo-words/config.conf`.
//! Unknown keys and unparseable values are ignored (the default is kept), so a
//! malformed file never prevents the app from starting. Every field defaults to
//! the original hard-coded behaviour, so no config file means no change.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Corner {
    fn parse(s: &str) -> Option<Self> {
        match s
            .trim()
            .to_ascii_lowercase()
            .replace(['_', ' '], "-")
            .as_str()
        {
            "top-left" => Some(Corner::TopLeft),
            "top-right" => Some(Corner::TopRight),
            "bottom-left" => Some(Corner::BottomLeft),
            "bottom-right" => Some(Corner::BottomRight),
            _ => None,
        }
    }
}

/// Upper bound for second-valued knobs (`interval_secs`, `jitter_secs`).
/// Anything past a day is meaningless for an ambient word timer, and clamping
/// also keeps the values safely inside `i64` range: `roll_interval` casts them
/// with `as i64`, so an unclamped value above `i64::MAX` would wrap negative,
/// make the jitter range `-j..=j` empty, and panic `rand`'s `random_range`.
const MAX_SECS: u64 = 86_400;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub interval_secs: u64,
    pub jitter_secs: u64,
    pub transcription_delay: f32,
    pub translation_delay: f32,
    pub fade_duration: f32,
    pub corner: Corner,
    pub speak: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interval_secs: 30,
            jitter_secs: 0,
            transcription_delay: 5.0,
            translation_delay: 10.0,
            fade_duration: 1.0,
            corner: Corner::BottomRight,
            speak: false,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        match read_config_file() {
            Some(text) => Config::default().merge_str(&text),
            None => Config::default(),
        }
    }

    /// Apply `key = value` lines onto `self`. Comments (`#`) and blank lines are
    /// skipped; unknown keys and unparseable values are ignored so a malformed
    /// file never changes a field from its default. Pure (no I/O) so it is unit
    /// testable without touching the environment or filesystem.
    fn merge_str(mut self, text: &str) -> Self {
        let cfg = &mut self;
        for line in text.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let (key, value) = (key.trim(), value.trim());
            match key {
                "interval_secs" => {
                    if let Ok(v) = value.parse::<u64>() {
                        cfg.interval_secs = v.clamp(1, MAX_SECS);
                    }
                }
                "jitter_secs" => {
                    if let Ok(v) = value.parse::<u64>() {
                        cfg.jitter_secs = v.min(MAX_SECS);
                    }
                }
                "transcription_delay" => {
                    if let Ok(v) = value.parse::<f32>() {
                        cfg.transcription_delay = v.max(0.0);
                    }
                }
                "translation_delay" => {
                    if let Ok(v) = value.parse::<f32>() {
                        cfg.translation_delay = v.max(0.0);
                    }
                }
                "fade_duration" => {
                    if let Ok(v) = value.parse::<f32>() {
                        cfg.fade_duration = v.max(0.01);
                    }
                }
                "corner" => {
                    if let Some(c) = Corner::parse(value) {
                        cfg.corner = c;
                    }
                }
                "speak" => {
                    cfg.speak = matches!(
                        value.to_ascii_lowercase().as_str(),
                        "true" | "1" | "yes" | "on"
                    );
                }
                _ => {}
            }
        }
        self
    }
}

fn read_config_file() -> Option<String> {
    if let Ok(path) = std::env::var("MEMO_CONFIG") {
        return std::fs::read_to_string(path).ok();
    }
    let home = std::env::var("HOME").ok()?;
    let path = format!("{home}/.config/memo-words/config.conf");
    std::fs::read_to_string(path).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_parse_accepts_separators_and_case() {
        assert_eq!(Corner::parse("top-left"), Some(Corner::TopLeft));
        assert_eq!(Corner::parse("Top_Left"), Some(Corner::TopLeft));
        assert_eq!(Corner::parse("BOTTOM RIGHT"), Some(Corner::BottomRight));
        assert_eq!(Corner::parse("middle"), None);
    }

    #[test]
    fn merge_str_parses_known_keys() {
        let cfg = Config::default().merge_str(
            "interval_secs = 45\njitter_secs = 7\ncorner = top-left\nspeak = yes\nfade_duration = 2.5",
        );
        assert_eq!(cfg.interval_secs, 45);
        assert_eq!(cfg.jitter_secs, 7);
        assert_eq!(cfg.corner, Corner::TopLeft);
        assert!(cfg.speak);
        assert_eq!(cfg.fade_duration, 2.5);
    }

    #[test]
    fn merge_str_ignores_comments_blanks_and_garbage() {
        let cfg = Config::default().merge_str(
            "# a comment\n\ninterval_secs = 12  # inline comment\nbogus_key = 9\ninterval_secs = not_a_number",
        );
        // Valid line applies; inline comment is stripped; the later unparseable
        // value leaves the field at its last good value; unknown key ignored.
        assert_eq!(cfg.interval_secs, 12);
        // Untouched fields keep defaults.
        assert_eq!(cfg.jitter_secs, Config::default().jitter_secs);
    }

    #[test]
    fn merge_str_clamps_out_of_range_values() {
        let cfg = Config::default().merge_str("interval_secs = 0\nfade_duration = 0");
        assert_eq!(cfg.interval_secs, 1); // clamped to >= 1
        assert!(cfg.fade_duration >= 0.01); // clamped to >= 0.01
    }

    #[test]
    fn merge_str_clamps_huge_second_values() {
        // 1e19 fits in u64 but exceeds i64::MAX; left unclamped it would wrap
        // to a negative i64 in roll_interval and panic rand. Clamp keeps both
        // knobs at the sane upper bound instead.
        let cfg = Config::default()
            .merge_str("interval_secs = 10000000000000000000\njitter_secs = 10000000000000000000");
        assert_eq!(cfg.interval_secs, 86_400);
        assert_eq!(cfg.jitter_secs, 86_400);
    }

    #[test]
    fn empty_input_keeps_defaults() {
        let cfg = Config::default().merge_str("");
        let def = Config::default();
        assert_eq!(cfg.interval_secs, def.interval_secs);
        assert_eq!(cfg.corner, def.corner);
        assert_eq!(cfg.speak, def.speak);
    }
}
