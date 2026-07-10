//! Pure `key = value` parsing and validation.

use super::{Config, Corner, ThemePreset};

const MAX_SECS: u64 = 86_400;
const MAX_CORNER_RADIUS: f32 = 64.0;
const MAX_EXIT_DURATION: f32 = 10.0;
const MAX_SETTLE_PX: f32 = 16.0;
const MIN_FONT_SCALE: f32 = 0.8;
const MAX_FONT_SCALE: f32 = 1.5;

pub(super) fn merge(mut cfg: Config, text: &str) -> Config {
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, raw_value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = config_value(key, raw_value);
        apply(&mut cfg, key, value);
    }
    cfg
}

fn config_value<'a>(key: &str, raw: &'a str) -> &'a str {
    let value = raw.trim();
    // Hex colors conventionally begin with '#'. Accept that natural form even
    // though '#' starts comments elsewhere in this deliberately tiny format.
    if key == "accent_color" && value.starts_with('#') {
        return value.split_whitespace().next().unwrap_or(value);
    }
    value.split('#').next().unwrap_or("").trim()
}

fn apply(cfg: &mut Config, key: &str, value: &str) {
    match key {
        "interval_secs" => set_u64(value, 1, MAX_SECS, &mut cfg.timing.interval_secs),
        "jitter_secs" => set_u64(value, 0, MAX_SECS, &mut cfg.timing.jitter_secs),
        "transcription_delay" => set_f32(value, 0.0, f32::MAX, &mut cfg.timing.transcription_delay),
        "translation_delay" => set_f32(value, 0.0, f32::MAX, &mut cfg.timing.translation_delay),
        "fade_duration" => set_f32(value, 0.01, f32::MAX, &mut cfg.timing.fade_duration),
        "exit_duration" => set_f32(value, 0.0, MAX_EXIT_DURATION, &mut cfg.timing.exit_duration),
        "rare_word_dwell" => set_f32(value, 0.0, 1.0, &mut cfg.timing.rare_word_dwell),
        "corner" => set_parsed(Corner::parse(value), &mut cfg.appearance.corner),
        "card_opacity" => set_f32(value, 0.0, 1.0, &mut cfg.appearance.card_opacity),
        "corner_radius" => set_f32(
            value,
            0.0,
            MAX_CORNER_RADIUS,
            &mut cfg.appearance.corner_radius,
        ),
        "settle_px" => set_f32(value, 0.0, MAX_SETTLE_PX, &mut cfg.appearance.settle_px),
        "accent_color" => {
            if let Some(color) = parse_hex_color(value) {
                cfg.appearance.accent_color = Some(color);
            }
        }
        "sheen" => set_f32(value, 0.0, 1.0, &mut cfg.appearance.sheen),
        "theme" => set_parsed(ThemePreset::parse(value), &mut cfg.appearance.theme),
        "speak" => cfg.learning.speak = parse_bool(value),
        "recall_mode" => cfg.learning.recall_mode = parse_bool(value),
        "recap_chance" => set_f32(value, 0.0, 1.0, &mut cfg.learning.recap_chance),
        "font_scale" => set_f32(
            value,
            MIN_FONT_SCALE,
            MAX_FONT_SCALE,
            &mut cfg.accessibility.font_scale,
        ),
        "enhanced_contrast" => cfg.accessibility.enhanced_contrast = parse_bool(value),
        "reduce_motion" => cfg.accessibility.reduce_motion = parse_bool(value),
        _ => {}
    }
}

fn parse_bool(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "true" | "1" | "yes" | "on"
    )
}

fn parse_hex_color(value: &str) -> Option<[u8; 3]> {
    let hex = value.trim().strip_prefix('#').unwrap_or(value.trim());
    if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    Some([
        u8::from_str_radix(&hex[0..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..6], 16).ok()?,
    ])
}

fn set_u64(value: &str, min: u64, max: u64, target: &mut u64) {
    if let Ok(parsed) = value.parse::<u64>() {
        *target = parsed.clamp(min, max);
    }
}

fn set_f32(value: &str, min: f32, max: f32, target: &mut f32) {
    if let Some(parsed) = value.parse::<f32>().ok().filter(|value| value.is_finite()) {
        *target = parsed.clamp(min, max);
    }
}

fn set_parsed<T>(value: Option<T>, target: &mut T) {
    if let Some(value) = value {
        *target = value;
    }
}
