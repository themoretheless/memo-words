//! The card's visual identity: colours, the type scale, and the per-line
//! brightness ranking, plus the small colour-derivation helpers. Split out of
//! `ui` so the look lives in one cohesive place and the renderer depends on named
//! theme values instead of scattered literals. Pure data and colour math; its
//! only egui coupling is the `Color32`/`Shadow` types it produces.

use eframe::egui;

// The dark glass tint of the card (unmultiplied RGB); alpha comes from the
// configurable opacity. At the default opacity this reproduces the original
// hard-coded fill (premultiplied 9,9,9 @ alpha 77) exactly.
const CARD_TINT: (u8, u8, u8) = (30, 30, 30);

/// Card background colour for a given opacity (0.0..=1.0).
pub fn card_bg(opacity: f32) -> egui::Color32 {
    let (r, g, b) = CARD_TINT;
    let a = (opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}

/// A soft drop shadow grounds the translucent card as a floating surface (the
/// macOS-widget / iOS-notification look) and lifts it off busy wallpapers.
pub const SHADOW: egui::epaint::Shadow = egui::epaint::Shadow {
    offset: [0, 6],
    blur: 24,
    spread: 0,
    color: egui::Color32::from_black_alpha(60),
};

/// A 1px hairline defines the card edge against light backgrounds, where the
/// dark translucent fill alone would otherwise wash out. Premultiplied white at
/// ~7% alpha (rgb == alpha == 18).
pub const BORDER_COLOR: egui::Color32 = egui::Color32::from_rgba_premultiplied(18, 18, 18, 18);

// Optional faux-vibrancy sheen: a faint white-to-transparent vertical gradient
// pooled in the top of the card, so it reads like a lit material. Kept very
// faint at full strength (a bright highlight reads glossy), and confined to the
// top portion so the lower lines stay on the flat fill.
pub const SHEEN_MAX_ALPHA: u8 = 30;
pub const SHEEN_HEIGHT_FRAC: f32 = 0.55;

pub const WORD_FONT_SIZE: f32 = 32.0;
// Type hierarchy answers a learner's question, so it ranks the lines by what
// matters: the headword is largest, then the meaning (the payoff), then the
// phonetic transcription (a pronunciation aid), then the example (context). The
// translation must beat the IPA in both size and brightness; previously the IPA
// was the brighter of the two, which let phonetics outrank the answer.
pub const TRANSLATION_FONT_SIZE: f32 = 18.0;
pub const TRANSCRIPTION_FONT_SIZE: f32 = 14.0;
// The example sentence is subordinate to the translation: a touch smaller and
// dimmer so it reads as supporting context, not the answer.
pub const EXAMPLE_FONT_SIZE: f32 = 13.0;

// Per-line greyscale level, used as BOTH the RGB value and the fully-faded alpha
// cap, so on the dark card a single number ranks each line's perceived
// brightness. The headword is pure white (255); among the rest the meaning wins,
// the transcription is a dim caption, and the example is faintest. Monotone
// 255 > translation > transcription > example keeps the hierarchy answer-first.
pub const TRANSLATION_INTENSITY: u8 = 215;
pub const TRANSCRIPTION_INTENSITY: u8 = 145;
pub const EXAMPLE_INTENSITY: u8 = 120;

// Compile-time guard that the type hierarchy stays answer-first: the payoff (the
// translation) must outrank the phonetic aid (the transcription) in both size
// and brightness, the headword tops everything, and the example is faintest and
// smallest. The brightness order is the regression guard, the IPA used to render
// brighter than the meaning. Tripping any of these fails the build, not a test.
const _: () = {
    assert!(WORD_FONT_SIZE > TRANSLATION_FONT_SIZE);
    assert!(TRANSLATION_FONT_SIZE > TRANSCRIPTION_FONT_SIZE);
    assert!(TRANSCRIPTION_FONT_SIZE > EXAMPLE_FONT_SIZE);
    assert!(TRANSLATION_INTENSITY < 255);
    assert!(TRANSLATION_INTENSITY > TRANSCRIPTION_INTENSITY);
    assert!(TRANSCRIPTION_INTENSITY > EXAMPLE_INTENSITY);
};

/// A greyscale line colour at the given fade progress. RGB is fixed at
/// `intensity`; alpha ramps to `intensity` at full fade, so a brighter intensity
/// is both lighter and more opaque, ranking the line in the hierarchy.
pub fn faded_line(intensity: u8, ease: f32) -> egui::Color32 {
    let a = (ease * intensity as f32) as u8;
    egui::Color32::from_rgba_unmultiplied(intensity, intensity, intensity, a)
}

/// Scale a colour's overall opacity by `factor` (0..1). Works on any `Color32`
/// by scaling all four premultiplied channels, so text, fill, shadow, and border
/// all fade uniformly toward transparent. `factor == 1.0` returns the colour
/// unchanged, so the exit fade is a no-op when the card is not leaving.
pub fn dim(color: egui::Color32, factor: f32) -> egui::Color32 {
    let f = factor.clamp(0.0, 1.0);
    let scale = |c: u8| (c as f32 * f) as u8;
    egui::Color32::from_rgba_premultiplied(
        scale(color.r()),
        scale(color.g()),
        scale(color.b()),
        scale(color.a()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_bg_default_matches_original_fill() {
        // The default opacity must reproduce the original hard-coded card fill
        // (premultiplied 9,9,9 @ alpha 77), so the look is unchanged by default.
        let original = egui::Color32::from_rgba_premultiplied(9, 9, 9, 77);
        assert_eq!(card_bg(crate::config::DEFAULT_CARD_OPACITY), original);
    }

    #[test]
    fn card_bg_scales_and_clamps_alpha() {
        assert_eq!(card_bg(0.0).a(), 0);
        assert_eq!(card_bg(1.0).a(), 255);
        assert_eq!(card_bg(5.0).a(), 255); // out-of-range clamps
        assert_eq!(card_bg(-1.0).a(), 0);
    }

    #[test]
    fn faded_line_ramps_alpha_and_fixes_rgb() {
        // Unfaded: fully transparent, RGB still at the intensity level.
        let c0 = faded_line(200, 0.0);
        assert_eq!(c0.a(), 0);
        // Fully faded: alpha reaches the intensity, RGB matches it.
        let c1 = faded_line(200, 1.0);
        assert_eq!(
            c1,
            egui::Color32::from_rgba_unmultiplied(200, 200, 200, 200)
        );
        // A brighter intensity is more opaque than a dimmer one at equal ease,
        // so perceived brightness tracks the single intensity number.
        assert!(faded_line(215, 1.0).a() > faded_line(120, 1.0).a());
    }

    #[test]
    fn dim_scales_opacity_uniformly() {
        let c = egui::Color32::from_rgba_unmultiplied(200, 100, 50, 255);
        assert_eq!(dim(c, 1.0), c); // identity at full
        assert_eq!(dim(c, 0.0), egui::Color32::TRANSPARENT); // gone at zero
        // Halving scales every premultiplied channel by ~half.
        let half = dim(c, 0.5);
        assert_eq!(half.a(), c.a() / 2);
        assert_eq!(half.r(), c.r() / 2);
    }
}
