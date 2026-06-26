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

/// Default card opacity. 77/255 reproduces the original hard-coded card fill
/// (premultiplied alpha 77) exactly, so the default look is unchanged.
pub const DEFAULT_CARD_OPACITY: f32 = 77.0 / 255.0;
/// Default card corner radius, matching the original hard-coded value.
pub const DEFAULT_CORNER_RADIUS: f32 = 16.0;
/// Upper bound for the corner radius; the card is 160px tall, so anything past
/// this stops looking like a rounded rectangle.
const MAX_CORNER_RADIUS: f32 = 64.0;
/// Upper bound for the exit-fade duration. It is a transition, not a dwell time;
/// anything past a few seconds stops reading as a fade. The app also caps it at
/// half the interval at runtime so the fade never eats the whole word.
const MAX_EXIT_DURATION: f32 = 10.0;
/// Upper bound for the entrance settle distance. Each line drifts up by at most
/// this many points; past a small nudge it stops reading as a settle and lines
/// start to visibly overlap their neighbours inside the fixed-height card.
const MAX_SETTLE_PX: f32 = 16.0;

/// Parse a `#rrggbb` or `rrggbb` hex colour into RGB bytes. Returns None for any
/// other shape (wrong length, non-hex, shorthand), so a malformed value just
/// leaves the accent off instead of changing it.
fn parse_hex_color(s: &str) -> Option<[u8; 3]> {
    let hex = s.trim();
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r, g, b])
}

/// Parse a finite f32, rejecting NaN and the infinities. A non-finite value would
/// otherwise slip past the clamps and poison the field: `f32::clamp` returns NaN
/// for a NaN input, and `inf.max(0.0)` stays infinite. A NaN opacity renders the
/// card invisible, and an infinite delay makes `anim_end` infinite so the app
/// would repaint at 60fps forever (the zero-idle invariant relies on a finite
/// end). Dropping the value here keeps the field at its default instead.
fn parse_finite_f32(s: &str) -> Option<f32> {
    s.parse::<f32>().ok().filter(|v| v.is_finite())
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub interval_secs: u64,
    pub jitter_secs: u64,
    /// Seconds before the transcription line starts fading in. Floored at 0.0;
    /// non-finite values are rejected.
    pub transcription_delay: f32,
    /// Seconds before the translation line starts fading in. Floored at 0.0;
    /// non-finite values are rejected. In recall mode the effective reveal is
    /// pushed later (see `App::effective_translation_delay`).
    pub translation_delay: f32,
    /// Seconds each line takes to fade in. Floored at 0.01; non-finite rejected.
    pub fade_duration: f32,
    pub corner: Corner,
    pub speak: bool,
    /// Card background opacity, 0.0 (invisible) to 1.0 (opaque).
    pub card_opacity: f32,
    /// Card corner radius in points (0.0..=64.0). Past that the rounded
    /// rectangle degrades against the fixed 160px card height.
    pub corner_radius: f32,
    /// Active-recall mode: hold the translation back until late in the word's
    /// display so there's a real window to recall the meaning first.
    pub recall_mode: bool,
    /// Seconds the card takes to fade out before the next word (0.0..=10.0, and
    /// further capped at runtime to half the interval). 0 = hard cut (the
    /// original behaviour); a small value softens the swap.
    pub exit_duration: f32,
    /// Probability (0.0..=1.0) that a word swap re-shows an earlier word instead
    /// of a fresh one, for spaced review. 0 = off (always a fresh word).
    pub recap_chance: f32,
    /// Points each line drifts up from as it fades in (0.0..=16.0, a gentle
    /// entrance settle). 0 = off (lines fade in place, the original behaviour).
    pub settle_px: f32,
    /// Optional accent colour (RGB) for a thin rule under the headword. None
    /// (the default) draws no rule, keeping the card monochrome.
    pub accent_color: Option<[u8; 3]>,
    /// Strength (0.0..=1.0) of a faint top sheen, a faux-vibrancy highlight that
    /// makes the card read like a lit material. 0 = off (flat fill).
    pub sheen: f32,
    /// Extra dwell for rarer words (0.0..=1.0). At strength `s` a word's display
    /// interval is multiplied by up to `(1 + s)` as its frequency rank rises, so
    /// harder words linger longer for more exposure while common words keep the
    /// base interval. 0 = off (every word uses the same interval).
    pub rare_word_dwell: f32,
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
            recall_mode: false,
            exit_duration: 0.0,
            recap_chance: 0.0,
            settle_px: 0.0,
            accent_color: None,
            sheen: 0.0,
            rare_word_dwell: 0.0,
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
                        cfg.jitter_secs = v.clamp(0, MAX_SECS);
                    }
                }
                "transcription_delay" => {
                    if let Some(v) = parse_finite_f32(value) {
                        cfg.transcription_delay = v.max(0.0);
                    }
                }
                "translation_delay" => {
                    if let Some(v) = parse_finite_f32(value) {
                        cfg.translation_delay = v.max(0.0);
                    }
                }
                "fade_duration" => {
                    if let Some(v) = parse_finite_f32(value) {
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
                    if let Some(v) = parse_finite_f32(value) {
                        cfg.card_opacity = v.clamp(0.0, 1.0);
                    }
                }
                "corner_radius" => {
                    if let Some(v) = parse_finite_f32(value) {
                        cfg.corner_radius = v.clamp(0.0, MAX_CORNER_RADIUS);
                    }
                }
                "recall_mode" => {
                    cfg.recall_mode = matches!(
                        value.to_ascii_lowercase().as_str(),
                        "true" | "1" | "yes" | "on"
                    );
                }
                "exit_duration" => {
                    if let Some(v) = parse_finite_f32(value) {
                        cfg.exit_duration = v.clamp(0.0, MAX_EXIT_DURATION);
                    }
                }
                "recap_chance" => {
                    if let Some(v) = parse_finite_f32(value) {
                        cfg.recap_chance = v.clamp(0.0, 1.0);
                    }
                }
                "settle_px" => {
                    if let Some(v) = parse_finite_f32(value) {
                        cfg.settle_px = v.clamp(0.0, MAX_SETTLE_PX);
                    }
                }
                "accent_color" => {
                    if let Some(rgb) = parse_hex_color(value) {
                        cfg.accent_color = Some(rgb);
                    }
                }
                "sheen" => {
                    if let Some(v) = parse_finite_f32(value) {
                        cfg.sheen = v.clamp(0.0, 1.0);
                    }
                }
                "rare_word_dwell" => {
                    if let Some(v) = parse_finite_f32(value) {
                        cfg.rare_word_dwell = v.clamp(0.0, 1.0);
                    }
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
    fn merge_str_rejects_non_finite_floats() {
        // nan/inf must never reach a field. f32::clamp returns NaN for a NaN
        // input (so a clamp arm would store NaN -> invisible card), and
        // inf.max(0.0) stays infinite (so a max arm would store inf, making
        // anim_end infinite and pinning the app at 60fps). Every float key must
        // drop a non-finite value and keep its default. We assert the WHOLE
        // config stays finite after each poisoned line, so a future field that
        // forgets the guard is caught too.
        let def = Config::default();
        let float_keys = [
            "transcription_delay",
            "translation_delay",
            "fade_duration",
            "card_opacity",
            "corner_radius",
            "exit_duration",
            "recap_chance",
            "settle_px",
            "sheen",
            "rare_word_dwell",
        ];
        for key in float_keys {
            for bad in ["nan", "NaN", "inf", "-inf", "infinity"] {
                let cfg = Config::default().merge_str(&format!("{key} = {bad}"));
                assert!(cfg.transcription_delay.is_finite());
                assert!(cfg.translation_delay.is_finite());
                assert!(cfg.fade_duration.is_finite());
                assert!(cfg.card_opacity.is_finite());
                assert!(cfg.corner_radius.is_finite());
                assert!(cfg.exit_duration.is_finite());
                assert!(cfg.recap_chance.is_finite());
                assert!(cfg.settle_px.is_finite());
                assert!(cfg.sheen.is_finite());
                assert!(cfg.rare_word_dwell.is_finite());
                // The poisoned key specifically must equal its default.
                let same = match key {
                    "transcription_delay" => cfg.transcription_delay == def.transcription_delay,
                    "translation_delay" => cfg.translation_delay == def.translation_delay,
                    "fade_duration" => cfg.fade_duration == def.fade_duration,
                    "card_opacity" => cfg.card_opacity == def.card_opacity,
                    "corner_radius" => cfg.corner_radius == def.corner_radius,
                    "exit_duration" => cfg.exit_duration == def.exit_duration,
                    "recap_chance" => cfg.recap_chance == def.recap_chance,
                    "settle_px" => cfg.settle_px == def.settle_px,
                    "sheen" => cfg.sheen == def.sheen,
                    "rare_word_dwell" => cfg.rare_word_dwell == def.rare_word_dwell,
                    _ => unreachable!(),
                };
                assert!(same, "{key} = {bad} changed the field from its default");
            }
        }
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
        let cfg = Config::default().merge_str("card_opacity = 0.5\ncorner_radius = 24");
        assert_eq!(cfg.card_opacity, 0.5);
        assert_eq!(cfg.corner_radius, 24.0);
    }

    #[test]
    fn merge_str_clamps_appearance_keys() {
        let cfg = Config::default().merge_str("card_opacity = 5\ncorner_radius = 999");
        assert_eq!(cfg.card_opacity, 1.0); // clamped to <= 1.0
        assert_eq!(cfg.corner_radius, 64.0); // clamped to <= MAX_CORNER_RADIUS
        let cfg = Config::default().merge_str("card_opacity = -1\ncorner_radius = -5");
        assert_eq!(cfg.card_opacity, 0.0); // clamped to >= 0.0
        assert_eq!(cfg.corner_radius, 0.0); // clamped to >= 0.0
    }

    #[test]
    fn merge_str_parses_and_clamps_sheen() {
        assert_eq!(Config::default().sheen, 0.0); // off by default
        assert_eq!(Config::default().merge_str("sheen = 0.5").sheen, 0.5);
        assert_eq!(Config::default().merge_str("sheen = 9").sheen, 1.0);
        assert_eq!(Config::default().merge_str("sheen = -2").sheen, 0.0);
    }

    #[test]
    fn merge_str_parses_and_clamps_rare_word_dwell() {
        assert_eq!(Config::default().rare_word_dwell, 0.0); // off by default
        assert_eq!(
            Config::default()
                .merge_str("rare_word_dwell = 0.5")
                .rare_word_dwell,
            0.5
        );
        // Out-of-range clamps into 0.0..=1.0.
        assert_eq!(
            Config::default()
                .merge_str("rare_word_dwell = 3")
                .rare_word_dwell,
            1.0
        );
        assert_eq!(
            Config::default()
                .merge_str("rare_word_dwell = -1")
                .rare_word_dwell,
            0.0
        );
    }

    #[test]
    fn merge_str_parses_accent_color() {
        assert_eq!(Config::default().accent_color, None); // off by default
        // Bare hex; '#' starts a comment in this format, so the hash is omitted.
        assert_eq!(
            Config::default()
                .merge_str("accent_color = ff8800")
                .accent_color,
            Some([0xff, 0x88, 0x00])
        );
        assert_eq!(
            Config::default()
                .merge_str("accent_color = 00AAff")
                .accent_color,
            Some([0x00, 0xaa, 0xff])
        );
        // A leading '#' is stripped as a comment, leaving an empty value, so the
        // accent stays off rather than turning on.
        assert_eq!(
            Config::default()
                .merge_str("accent_color = #ff8800")
                .accent_color,
            None
        );
        // Garbage and shorthand leave it off (unchanged default).
        assert_eq!(
            Config::default()
                .merge_str("accent_color = nope")
                .accent_color,
            None
        );
        assert_eq!(
            Config::default()
                .merge_str("accent_color = fff")
                .accent_color,
            None
        );
    }

    #[test]
    fn merge_str_parses_and_clamps_settle_px() {
        assert_eq!(Config::default().settle_px, 0.0); // off by default
        assert_eq!(Config::default().merge_str("settle_px = 4").settle_px, 4.0);
        assert_eq!(
            Config::default().merge_str("settle_px = 999").settle_px,
            16.0
        );
        assert_eq!(Config::default().merge_str("settle_px = -3").settle_px, 0.0);
    }

    #[test]
    fn merge_str_parses_and_clamps_recap_chance() {
        assert_eq!(Config::default().recap_chance, 0.0); // off by default
        assert_eq!(
            Config::default()
                .merge_str("recap_chance = 0.2")
                .recap_chance,
            0.2
        );
        // Out-of-range clamps into 0.0..=1.0.
        assert_eq!(
            Config::default().merge_str("recap_chance = 2").recap_chance,
            1.0
        );
        assert_eq!(
            Config::default()
                .merge_str("recap_chance = -1")
                .recap_chance,
            0.0
        );
    }

    #[test]
    fn merge_str_parses_and_clamps_exit_duration() {
        assert_eq!(Config::default().exit_duration, 0.0); // off by default
        assert_eq!(
            Config::default()
                .merge_str("exit_duration = 0.4")
                .exit_duration,
            0.4
        );
        // Negative clamps to 0 (off); absurdly large clamps to the cap.
        assert_eq!(
            Config::default()
                .merge_str("exit_duration = -1")
                .exit_duration,
            0.0
        );
        assert_eq!(
            Config::default()
                .merge_str("exit_duration = 999")
                .exit_duration,
            10.0
        );
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
}
