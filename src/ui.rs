use crate::config::Corner;
use eframe::egui;
use std::sync::Arc;

const SCREEN_MARGIN: f32 = 40.0;
const INNER_MARGIN: f32 = 20.0;
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
// A soft drop shadow grounds the translucent card as a floating surface (the
// macOS-widget / iOS-notification look) and lifts it off busy wallpapers.
const SHADOW: egui::epaint::Shadow = egui::epaint::Shadow {
    offset: [0, 6],
    blur: 24,
    spread: 0,
    color: egui::Color32::from_black_alpha(60),
};
// A 1px hairline defines the card edge against light backgrounds, where the
// dark translucent fill alone would otherwise wash out. Premultiplied white at
// ~7% alpha (rgb == alpha == 18).
const BORDER_COLOR: egui::Color32 = egui::Color32::from_rgba_premultiplied(18, 18, 18, 18);

pub const MIN_WIDTH: f32 = 150.0;
pub const MAX_WIDTH: f32 = 600.0;
pub const WIDGET_HEIGHT: f32 = 160.0;
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
const TRANSLATION_INTENSITY: u8 = 215;
const TRANSCRIPTION_INTENSITY: u8 = 145;
const EXAMPLE_INTENSITY: u8 = 120;

// Compile-time guard that the type hierarchy stays answer-first: the payoff (the
// translation) must outrank the phonetic aid (the transcription) in both size
// and brightness, the headword tops everything, and the example is faintest and
// smallest. The brightness order is the regression guard, the IPA used to render
// brighter than the meaning. Tripping any of these fails the build, not a test.
const _: () = {
    assert!(WORD_FONT_SIZE > TRANSLATION_FONT_SIZE);
    assert!(TRANSLATION_FONT_SIZE > TRANSCRIPTION_FONT_SIZE);
    assert!(TRANSCRIPTION_FONT_SIZE >= EXAMPLE_FONT_SIZE);
    assert!(TRANSLATION_INTENSITY < 255);
    assert!(TRANSLATION_INTENSITY > TRANSCRIPTION_INTENSITY);
    assert!(TRANSCRIPTION_INTENSITY > EXAMPLE_INTENSITY);
};

/// A greyscale line colour at the given fade progress. RGB is fixed at
/// `intensity`; alpha ramps to `intensity` at full fade, so a brighter intensity
/// is both lighter and more opaque, ranking the line in the hierarchy.
fn faded_line(intensity: u8, ease: f32) -> egui::Color32 {
    let a = (ease * intensity as f32) as u8;
    egui::Color32::from_rgba_unmultiplied(intensity, intensity, intensity, a)
}
// Kept to a single line that fits the fixed-height card; longer examples are
// truncated with an ellipsis rather than wrapping and overflowing.
pub const EXAMPLE_MAX_CHARS: usize = 64;

// Animation timings (seconds). Fade delays/duration are configurable per
// CardView; the width transition stays fixed.
pub const WIDTH_TRANSITION: f32 = 0.5;

pub fn setup_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::TRANSPARENT;
    visuals.window_fill = egui::Color32::TRANSPARENT;
    ctx.set_visuals(visuals);
}

pub fn load_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(data) = std::fs::read("/System/Library/Fonts/Supplemental/Arial Unicode.ttf") {
        fonts.font_data.insert(
            "arial_unicode".into(),
            Arc::new(egui::FontData::from_owned(data)),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .push("arial_unicode".into());
    }
    ctx.set_fonts(fonts);
}

pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn fade_factor(elapsed: f32, delay: f32, fade_duration: f32) -> f32 {
    smoothstep((elapsed - delay) / fade_duration)
}

/// Whole-card opacity multiplier (1..0) for the exit fade. `until_next` is the
/// seconds left before the next word; once it drops below `exit_duration` the
/// card eases out, reaching 0 exactly at the swap, so the word leaves softly
/// instead of hard-cutting. With `exit_duration <= 0` the card never fades.
pub fn exit_alpha(until_next: f32, exit_duration: f32) -> f32 {
    if exit_duration <= 0.0 || until_next >= exit_duration {
        return 1.0;
    }
    let progress = 1.0 - (until_next.max(0.0) / exit_duration);
    1.0 - smoothstep(progress)
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

/// Clamp an example sentence to a single line's worth of characters, appending
/// an ellipsis if it was cut. Counts by `char` so multibyte text never splits a
/// codepoint, and the result never exceeds `max_chars` chars. Returns the input
/// untouched when it already fits.
pub fn truncate_example(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    // Reserve one char for the ellipsis so the total stays within budget.
    let kept: String = s.chars().take(max_chars - 1).collect();
    format!("{}…", kept.trim_end())
}

pub fn measure_text_width(ui: &egui::Ui, text: &str, size: f32) -> f32 {
    ui.painter()
        .layout_no_wrap(
            text.into(),
            egui::FontId::proportional(size),
            egui::Color32::WHITE,
        )
        .rect
        .width()
}

/// Real rendered height of one line, matching exactly what `centered_text`
/// allocates (`galley.rect.height()`). Used so vertical centering budgets the
/// true row height (ascent + descent + line gap), not the nominal font size.
pub fn measure_text_height(ui: &egui::Ui, text: &str, size: f32) -> f32 {
    ui.painter()
        .layout_no_wrap(
            text.into(),
            egui::FontId::proportional(size),
            egui::Color32::PLACEHOLDER,
        )
        .rect
        .height()
}

pub fn centered_text(ui: &mut egui::Ui, text: &str, size: f32, color: egui::Color32) {
    // Lay out with PLACEHOLDER so the galley's cache key (which epaint hashes
    // including color) stays constant as the fade alpha changes each frame.
    // The real color is applied at draw time below, turning per-frame fade
    // re-layouts into galley-cache hits. Geometry is color-independent.
    let galley = ui.painter().layout_no_wrap(
        text.into(),
        egui::FontId::proportional(size),
        egui::Color32::PLACEHOLDER,
    );
    let text_w = galley.rect.width();
    let avail = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(avail, galley.rect.height()),
        egui::Sense::hover(),
    );
    let x = rect.min.x + (avail - text_w) / 2.0;
    ui.painter()
        .galley(egui::pos2(x, rect.min.y), galley, color);
}

pub struct CardView<'a> {
    pub word: &'a str,
    pub transcription: &'a str,
    pub translation: &'a str,
    pub example: &'a str,
    pub elapsed: f32,
    pub prev_width: f32,
    pub transcription_delay: f32,
    pub translation_delay: f32,
    pub example_delay: f32,
    pub fade_duration: f32,
    pub corner: Corner,
    pub card_opacity: f32,
    pub corner_radius: f32,
    /// Whole-card opacity multiplier (1.0 = fully shown). Drives the exit fade;
    /// 1.0 leaves every colour untouched.
    pub exit_alpha: f32,
}

impl<'a> CardView<'a> {
    /// The example text actually shown: empty if absent (or whitespace-only),
    /// else trimmed and truncated to a single line. Computed once so width and
    /// paint agree.
    fn example_text(&self) -> String {
        let trimmed = self.example.trim();
        if trimmed.is_empty() {
            String::new()
        } else {
            truncate_example(trimmed, EXAMPLE_MAX_CHARS)
        }
    }

    /// Fade-in progress (0..1) of the transcription, translation, and example
    /// lines at the current elapsed time. Shared by width and painting.
    fn eases(&self) -> (f32, f32, f32) {
        (
            fade_factor(self.elapsed, self.transcription_delay, self.fade_duration),
            fade_factor(self.elapsed, self.translation_delay, self.fade_duration),
            fade_factor(self.elapsed, self.example_delay, self.fade_duration),
        )
    }

    pub fn compute_width(&self, ui: &egui::Ui) -> f32 {
        let word_w = measure_text_width(ui, self.word, WORD_FONT_SIZE);
        let trans_w = measure_text_width(ui, self.transcription, TRANSCRIPTION_FONT_SIZE);
        let transl_w = measure_text_width(ui, self.translation, TRANSLATION_FONT_SIZE);
        let example = self.example_text();
        let example_w = measure_text_width(ui, &example, EXAMPLE_FONT_SIZE);

        let (trans_ease, transl_ease, example_ease) = self.eases();

        let initial = smoothstep(self.elapsed / WIDTH_TRANSITION);
        let from = self.prev_width - 2.0 * INNER_MARGIN;
        let w0 = from + (word_w - from) * initial;
        let w1 = word_w.max(trans_w);
        let w2 = w1.max(transl_w);
        let w3 = w2.max(example_w);
        let content =
            w0 + (w1 - w0) * trans_ease + (w2 - w1) * transl_ease + (w3 - w2) * example_ease;

        (content + 2.0 * INNER_MARGIN).clamp(MIN_WIDTH, MAX_WIDTH)
    }

    pub fn paint(&self, ui: &mut egui::Ui, widget_w: f32) {
        let screen = ui.max_rect();

        let x = match self.corner {
            Corner::TopLeft | Corner::BottomLeft => screen.min.x + SCREEN_MARGIN,
            Corner::TopRight | Corner::BottomRight => screen.max.x - widget_w - SCREEN_MARGIN,
        };
        let y = match self.corner {
            Corner::TopLeft | Corner::TopRight => screen.min.y + SCREEN_MARGIN,
            Corner::BottomLeft | Corner::BottomRight => {
                screen.max.y - WIDGET_HEIGHT - SCREEN_MARGIN
            }
        };

        let widget_rect =
            egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(widget_w, WIDGET_HEIGHT));

        // Whole-card opacity for the exit fade. 1.0 leaves every colour
        // untouched, so this is a no-op outside the exit window.
        let e = self.exit_alpha;
        let mut shadow = SHADOW;
        shadow.color = dim(SHADOW.color, e);
        ui.painter()
            .add(shadow.as_shape(widget_rect, self.corner_radius));
        ui.painter().rect_filled(
            widget_rect,
            self.corner_radius,
            dim(card_bg(self.card_opacity), e),
        );
        ui.painter().rect_stroke(
            widget_rect,
            self.corner_radius,
            egui::Stroke::new(1.0_f32, dim(BORDER_COLOR, e)),
            egui::StrokeKind::Inside,
        );

        let inner = widget_rect.shrink(INNER_MARGIN);
        let (trans_ease, transl_ease, example_ease) = self.eases();
        let example = self.example_text();
        let show_example = !example.is_empty() && example_ease > 0.01;

        // Budget the REAL rendered row heights (what centered_text allocates),
        // not the nominal font sizes, so the block is centered accurately.
        let mut content_h = measure_text_height(ui, self.word, WORD_FONT_SIZE);
        if trans_ease > 0.01 {
            content_h += 6.0 * trans_ease
                + measure_text_height(ui, self.transcription, TRANSCRIPTION_FONT_SIZE);
        }
        if transl_ease > 0.01 {
            content_h += 4.0 * transl_ease
                + measure_text_height(ui, self.translation, TRANSLATION_FONT_SIZE);
        }
        if show_example {
            content_h += 6.0 * example_ease + measure_text_height(ui, &example, EXAMPLE_FONT_SIZE);
        }
        let top_pad = ((inner.height() - content_h) / 2.0).max(0.0);

        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
        child.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.add_space(top_pad);
            centered_text(ui, self.word, WORD_FONT_SIZE, dim(egui::Color32::WHITE, e));

            if trans_ease > 0.01 {
                ui.add_space(6.0 * trans_ease);
                centered_text(
                    ui,
                    self.transcription,
                    TRANSCRIPTION_FONT_SIZE,
                    dim(faded_line(TRANSCRIPTION_INTENSITY, trans_ease), e),
                );
            }

            if transl_ease > 0.01 {
                ui.add_space(4.0 * transl_ease);
                centered_text(
                    ui,
                    self.translation,
                    TRANSLATION_FONT_SIZE,
                    dim(faded_line(TRANSLATION_INTENSITY, transl_ease), e),
                );
            }

            if show_example {
                ui.add_space(6.0 * example_ease);
                centered_text(
                    ui,
                    &example,
                    EXAMPLE_FONT_SIZE,
                    dim(faded_line(EXAMPLE_INTENSITY, example_ease), e),
                );
            }
        });
    }
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
    fn smoothstep_clamps_and_eases() {
        assert_eq!(smoothstep(-1.0), 0.0);
        assert_eq!(smoothstep(0.0), 0.0);
        assert_eq!(smoothstep(1.0), 1.0);
        assert_eq!(smoothstep(2.0), 1.0);
        // Symmetric ease passes through 0.5 at the midpoint.
        assert!((smoothstep(0.5) - 0.5).abs() < 1e-6);
        // Monotonic.
        assert!(smoothstep(0.25) < smoothstep(0.75));
    }

    #[test]
    fn fade_factor_is_zero_before_delay_and_one_after() {
        let (delay, fade) = (5.0, 1.0);
        assert_eq!(fade_factor(0.0, delay, fade), 0.0);
        assert_eq!(fade_factor(delay, delay, fade), 0.0);
        assert_eq!(fade_factor(delay + fade, delay, fade), 1.0);
        assert_eq!(fade_factor(delay + 10.0, delay, fade), 1.0);
        // Halfway through the fade window is mid-ease.
        assert!((fade_factor(delay + 0.5 * fade, delay, fade) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn truncate_example_leaves_short_text_untouched() {
        assert_eq!(truncate_example("Have a nice day!", 64), "Have a nice day!");
        assert_eq!(truncate_example("", 64), "");
    }

    #[test]
    fn truncate_example_cuts_long_text_with_ellipsis() {
        let long = "This is a rather long example sentence that should be cut.";
        let out = truncate_example(long, 20);
        assert!(out.ends_with('…'));
        // Ellipsis counts as one of the kept chars.
        assert_eq!(out.chars().count(), 20);
    }

    #[test]
    fn truncate_example_respects_char_boundaries() {
        // Multibyte (Cyrillic) text must not split a codepoint.
        let s = "Это довольно длинный пример предложения для проверки.";
        let out = truncate_example(s, 10);
        assert_eq!(out.chars().count(), 10);
        assert!(out.ends_with('…'));
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
    fn exit_alpha_off_and_before_window_is_full() {
        // Disabled (duration 0) stays fully visible regardless of time left.
        assert_eq!(exit_alpha(5.0, 0.0), 1.0);
        assert_eq!(exit_alpha(0.0, 0.0), 1.0);
        // Outside the window (more time left than the fade) is fully visible.
        assert_eq!(exit_alpha(2.0, 0.5), 1.0);
        assert_eq!(exit_alpha(0.5, 0.5), 1.0); // exactly at the window edge
    }

    #[test]
    fn exit_alpha_eases_to_zero_at_the_swap() {
        let dur = 0.5;
        // Midpoint of the window: 1 - smoothstep(0.5) = 0.5.
        assert!((exit_alpha(0.25, dur) - 0.5).abs() < 1e-6);
        // At the swap the card is fully faded out.
        assert_eq!(exit_alpha(0.0, dur), 0.0);
        // Monotone: less time left means more faded (lower alpha).
        assert!(exit_alpha(0.1, dur) < exit_alpha(0.4, dur));
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

    #[test]
    fn truncate_example_never_exceeds_budget_at_zero() {
        // The result must never exceed max_chars, including the degenerate 0.
        assert_eq!(truncate_example("hello", 0), "");
        assert_eq!(truncate_example("", 0), "");
        // And at 1 the ellipsis alone is within budget.
        assert_eq!(truncate_example("hello", 1).chars().count(), 1);
    }
}
