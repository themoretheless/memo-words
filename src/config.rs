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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    fn parse(s: &str) -> Option<Self> {
        let hex = s.trim().strip_prefix('#').unwrap_or(s.trim());
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Self { r, g, b })
    }
}

/// Upper bound for second-valued knobs (`interval_secs`, `jitter_secs`).
/// Anything past a day is meaningless for an ambient word timer, and clamping
/// also keeps the values safely inside `i64` range: `roll_interval` casts them
/// with `as i64`, so an unclamped value above `i64::MAX` would wrap negative,
/// make the jitter range `-j..=j` empty, and panic `rand`'s `random_range`.
const MAX_SECS: u64 = 86_400;

/// Default card opacity. 77/255 reproduces the original hard-coded card fill
/// (premultiplied alpha 77) exactly, so the default look is unchanged.
pub const DEFAULT_CARD_OPACITY: f32 = 77.0 / 255.0;
/// Default card corner radius, matching the original hard-coded value.
pub const DEFAULT_CORNER_RADIUS: f32 = 16.0;
/// Default outer card margin from the chosen screen edges.
pub const DEFAULT_SCREEN_MARGIN: f32 = 40.0;
/// Default accent: near-white, matching the existing subtle highlight.
pub const DEFAULT_ACCENT_COLOR: Rgb = Rgb {
    r: 230,
    g: 230,
    b: 230,
};
/// Upper bound for the corner radius; the card is 160px tall, so anything past
/// this stops looking like a rounded rectangle.
const MAX_CORNER_RADIUS: f32 = 64.0;
/// Max practical edge margin for an ambient corner card.
const MAX_SCREEN_MARGIN: f32 = 240.0;

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub interval_secs: u64,
    pub jitter_secs: u64,
    pub transcription_delay: f32,
    pub translation_delay: f32,
    pub fade_duration: f32,
    pub corner: Corner,
    pub speak: bool,
    /// Card background opacity, 0.0 (invisible) to 1.0 (opaque).
    pub card_opacity: f32,
    /// Card corner radius in points.
    pub corner_radius: f32,
    /// Outer margin from the selected screen corner.
    pub screen_margin: f32,
    /// Active-recall mode: hold the translation back until late in the word's
    /// display so there's a real window to recall the meaning first.
    pub recall_mode: bool,
    /// Draw a faint underline showing how far through the current word's
    /// interval we are. Off by default so the idle overlay can keep sleeping.
    pub pace_indicator: bool,
    /// Subtle accent used for example-word emphasis and the pace indicator.
    pub accent_color: Rgb,
    /// Draw a tiny count of words shown this session.
    pub session_counter: bool,
    /// Stronger material and text contrast for bright or busy wallpapers.
    pub high_contrast: bool,
    /// Slightly dim words that have already appeared in this app session.
    pub familiarity_dimming: bool,
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
            card_opacity: DEFAULT_CARD_OPACITY,
            corner_radius: DEFAULT_CORNER_RADIUS,
            screen_margin: DEFAULT_SCREEN_MARGIN,
            recall_mode: false,
            pace_indicator: false,
            accent_color: DEFAULT_ACCENT_COLOR,
            session_counter: false,
            high_contrast: false,
            familiarity_dimming: false,
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
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = if key == "accent_color" {
                value.split_whitespace().next().unwrap_or("").trim()
            } else {
                value.split('#').next().unwrap_or("").trim()
            };
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
                "card_opacity" => {
                    if let Ok(v) = value.parse::<f32>() {
                        cfg.card_opacity = v.clamp(0.0, 1.0);
                    }
                }
                "corner_radius" => {
                    if let Ok(v) = value.parse::<f32>() {
                        cfg.corner_radius = v.clamp(0.0, MAX_CORNER_RADIUS);
                    }
                }
                "screen_margin" => {
                    if let Ok(v) = value.parse::<f32>() {
                        cfg.screen_margin = v.clamp(0.0, MAX_SCREEN_MARGIN);
                    }
                }
                "recall_mode" => {
                    cfg.recall_mode = matches!(
                        value.to_ascii_lowercase().as_str(),
                        "true" | "1" | "yes" | "on"
                    );
                }
                "pace_indicator" => {
                    cfg.pace_indicator = matches!(
                        value.to_ascii_lowercase().as_str(),
                        "true" | "1" | "yes" | "on"
                    );
                }
                "accent_color" => {
                    if let Some(rgb) = Rgb::parse(value) {
                        cfg.accent_color = rgb;
                    }
                }
                "session_counter" => {
                    cfg.session_counter = matches!(
                        value.to_ascii_lowercase().as_str(),
                        "true" | "1" | "yes" | "on"
                    );
                }
                "high_contrast" => {
                    cfg.high_contrast = matches!(
                        value.to_ascii_lowercase().as_str(),
                        "true" | "1" | "yes" | "on"
                    );
                }
                "familiarity_dimming" => {
                    cfg.familiarity_dimming = matches!(
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
    fn rgb_parse_accepts_hash_or_plain_hex() {
        assert_eq!(
            Rgb::parse("#7FD1AE"),
            Some(Rgb {
                r: 127,
                g: 209,
                b: 174
            })
        );
        assert_eq!(
            Rgb::parse("7fd1ae"),
            Some(Rgb {
                r: 127,
                g: 209,
                b: 174
            })
        );
    }

    #[test]
    fn rgb_parse_rejects_invalid_values() {
        assert_eq!(Rgb::parse("#123"), None);
        assert_eq!(Rgb::parse("#zzzzzz"), None);
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

    #[test]
    fn merge_str_parses_appearance_keys() {
        let cfg = Config::default()
            .merge_str("card_opacity = 0.5\ncorner_radius = 24\nscreen_margin = 64");
        assert_eq!(cfg.card_opacity, 0.5);
        assert_eq!(cfg.corner_radius, 24.0);
        assert_eq!(cfg.screen_margin, 64.0);
    }

    #[test]
    fn merge_str_clamps_appearance_keys() {
        let cfg = Config::default()
            .merge_str("card_opacity = 5\ncorner_radius = 999\nscreen_margin = 999");
        assert_eq!(cfg.card_opacity, 1.0); // clamped to <= 1.0
        assert_eq!(cfg.corner_radius, 64.0); // clamped to <= MAX_CORNER_RADIUS
        assert_eq!(cfg.screen_margin, 240.0); // clamped to <= MAX_SCREEN_MARGIN
        let cfg = Config::default()
            .merge_str("card_opacity = -1\ncorner_radius = -5\nscreen_margin = -5");
        assert_eq!(cfg.card_opacity, 0.0); // clamped to >= 0.0
        assert_eq!(cfg.corner_radius, 0.0); // clamped to >= 0.0
        assert_eq!(cfg.screen_margin, 0.0); // clamped to >= 0.0
    }

    #[test]
    fn merge_str_parses_recall_mode() {
        assert!(Config::default().merge_str("recall_mode = on").recall_mode);
        assert!(
            Config::default()
                .merge_str("recall_mode = true")
                .recall_mode
        );
        assert!(Config::default().merge_str("recall_mode = 1").recall_mode);
        // Off by default and for anything that isn't a truthy token.
        assert!(!Config::default().recall_mode);
        assert!(
            !Config::default()
                .merge_str("recall_mode = false")
                .recall_mode
        );
        assert!(
            !Config::default()
                .merge_str("recall_mode = nope")
                .recall_mode
        );
    }

    #[test]
    fn merge_str_parses_pace_indicator() {
        assert!(
            Config::default()
                .merge_str("pace_indicator = on")
                .pace_indicator
        );
        assert!(
            Config::default()
                .merge_str("pace_indicator = yes")
                .pace_indicator
        );
        assert!(!Config::default().pace_indicator);
        assert!(
            !Config::default()
                .merge_str("pace_indicator = false")
                .pace_indicator
        );
    }

    #[test]
    fn merge_str_parses_accent_color() {
        let cfg = Config::default().merge_str("accent_color = #7FD1AE");
        assert_eq!(
            cfg.accent_color,
            Rgb {
                r: 127,
                g: 209,
                b: 174
            }
        );

        let cfg = Config::default().merge_str("accent_color = nope");
        assert_eq!(cfg.accent_color, DEFAULT_ACCENT_COLOR);
    }

    #[test]
    fn merge_str_parses_session_counter() {
        assert!(
            Config::default()
                .merge_str("session_counter = on")
                .session_counter
        );
        assert!(!Config::default().session_counter);
        assert!(
            !Config::default()
                .merge_str("session_counter = off")
                .session_counter
        );
    }

    #[test]
    fn merge_str_parses_high_contrast() {
        assert!(
            Config::default()
                .merge_str("high_contrast = yes")
                .high_contrast
        );
        assert!(!Config::default().high_contrast);
        assert!(
            !Config::default()
                .merge_str("high_contrast = false")
                .high_contrast
        );
    }

    #[test]
    fn merge_str_parses_familiarity_dimming() {
        assert!(
            Config::default()
                .merge_str("familiarity_dimming = true")
                .familiarity_dimming
        );
        assert!(!Config::default().familiarity_dimming);
        assert!(
            !Config::default()
                .merge_str("familiarity_dimming = no")
                .familiarity_dimming
        );
    }
}
