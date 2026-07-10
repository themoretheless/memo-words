use super::*;

#[test]
fn defaults_are_grouped_and_default_preserving() {
    let cfg = Config::default();
    assert_eq!(cfg.timing.interval_secs, 30);
    assert_eq!(cfg.appearance.corner, Corner::BottomRight);
    assert_eq!(cfg.appearance.theme, ThemePreset::Graphite);
    assert_eq!(cfg.accessibility.font_scale, 1.0);
    assert!(!cfg.learning.speak);
}

#[test]
fn parser_applies_every_supported_group() {
    let cfg = Config::default().merge_str(
        "interval_secs=45\njitter_secs=7\ntranscription_delay=2\ntranslation_delay=4\nfade_duration=.5\nexit_duration=.3\nrare_word_dwell=.4\ncorner=top-left\ncard_opacity=.6\ncorner_radius=20\nsettle_px=3\naccent_color=#5e9eff\nsheen=.2\ntheme=paper\nspeak=yes\nrecall_mode=on\nrecap_chance=.25\nfont_scale=1.2\nenhanced_contrast=true\nreduce_motion=1",
    );
    assert_eq!(cfg.timing.interval_secs, 45);
    assert_eq!(cfg.timing.jitter_secs, 7);
    assert_eq!(cfg.timing.transcription_delay, 2.0);
    assert_eq!(cfg.timing.translation_delay, 4.0);
    assert_eq!(cfg.timing.fade_duration, 0.5);
    assert_eq!(cfg.timing.exit_duration, 0.3);
    assert_eq!(cfg.timing.rare_word_dwell, 0.4);
    assert_eq!(cfg.appearance.corner, Corner::TopLeft);
    assert_eq!(cfg.appearance.card_opacity, 0.6);
    assert_eq!(cfg.appearance.corner_radius, 20.0);
    assert_eq!(cfg.appearance.settle_px, 3.0);
    assert_eq!(cfg.appearance.accent_color, Some([0x5e, 0x9e, 0xff]));
    assert_eq!(cfg.appearance.sheen, 0.2);
    assert_eq!(cfg.appearance.theme, ThemePreset::Paper);
    assert!(cfg.learning.speak);
    assert!(cfg.learning.recall_mode);
    assert_eq!(cfg.learning.recap_chance, 0.25);
    assert_eq!(cfg.accessibility.font_scale, 1.2);
    assert!(cfg.accessibility.enhanced_contrast);
    assert!(cfg.accessibility.reduce_motion);
}

#[test]
fn parser_accepts_hash_and_bare_accent_colors() {
    assert_eq!(
        Config::default()
            .merge_str("accent_color = #ff8800")
            .appearance
            .accent_color,
        Some([0xff, 0x88, 0x00])
    );
    assert_eq!(
        Config::default()
            .merge_str("accent_color = 00aaff # comment")
            .appearance
            .accent_color,
        Some([0x00, 0xaa, 0xff])
    );
}

#[test]
fn parser_ignores_unknown_and_malformed_values() {
    let cfg = Config::default()
        .merge_str("# comment\nunknown=9\ninterval_secs=nope\ncard_opacity=nan\nfont_scale=inf");
    let default = Config::default();
    assert_eq!(cfg.timing.interval_secs, default.timing.interval_secs);
    assert_eq!(cfg.appearance.card_opacity, default.appearance.card_opacity);
    assert_eq!(
        cfg.accessibility.font_scale,
        default.accessibility.font_scale
    );
}

#[test]
fn numeric_values_are_clamped_to_safe_ranges() {
    let cfg = Config::default().merge_str(
        "interval_secs=0\njitter_secs=10000000000000000000\nfade_duration=0\ncorner_radius=999\nsettle_px=-3\nrecap_chance=2\nfont_scale=9",
    );
    assert_eq!(cfg.timing.interval_secs, 1);
    assert_eq!(cfg.timing.jitter_secs, 86_400);
    assert_eq!(cfg.timing.fade_duration, 0.01);
    assert_eq!(cfg.appearance.corner_radius, 64.0);
    assert_eq!(cfg.appearance.settle_px, 0.0);
    assert_eq!(cfg.learning.recap_chance, 1.0);
    assert_eq!(cfg.accessibility.font_scale, 1.5);
}

#[test]
fn enum_parsers_accept_human_friendly_spelling() {
    assert_eq!(Corner::parse("BOTTOM RIGHT"), Some(Corner::BottomRight));
    assert_eq!(
        ThemePreset::parse("high_contrast"),
        Some(ThemePreset::HighContrast)
    );
    assert_eq!(ThemePreset::parse("light"), Some(ThemePreset::Paper));
    assert_eq!(ThemePreset::parse("neon"), None);
}
