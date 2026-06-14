use crate::config::Corner;
use eframe::egui;
use std::sync::Arc;

const CORNER_RADIUS: f32 = 16.0;
const SCREEN_MARGIN: f32 = 40.0;
const INNER_MARGIN: f32 = 20.0;
const BG_COLOR: egui::Color32 = egui::Color32::from_rgba_premultiplied(9, 9, 9, 77);
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
pub const SUB_FONT_SIZE: f32 = 15.0;
// The example sentence is subordinate to the translation: a touch smaller and
// dimmer so it reads as supporting context, not the answer.
pub const EXAMPLE_FONT_SIZE: f32 = 13.0;
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
        let trans_w = measure_text_width(ui, self.transcription, SUB_FONT_SIZE);
        let transl_w = measure_text_width(ui, self.translation, SUB_FONT_SIZE);
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

        ui.painter()
            .add(SHADOW.as_shape(widget_rect, CORNER_RADIUS));
        ui.painter()
            .rect_filled(widget_rect, CORNER_RADIUS, BG_COLOR);
        ui.painter().rect_stroke(
            widget_rect,
            CORNER_RADIUS,
            egui::Stroke::new(1.0_f32, BORDER_COLOR),
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
            content_h +=
                6.0 * trans_ease + measure_text_height(ui, self.transcription, SUB_FONT_SIZE);
        }
        if transl_ease > 0.01 {
            content_h +=
                4.0 * transl_ease + measure_text_height(ui, self.translation, SUB_FONT_SIZE);
        }
        if show_example {
            content_h += 6.0 * example_ease + measure_text_height(ui, &example, EXAMPLE_FONT_SIZE);
        }
        let top_pad = ((inner.height() - content_h) / 2.0).max(0.0);

        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
        child.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.add_space(top_pad);
            centered_text(ui, self.word, WORD_FONT_SIZE, egui::Color32::WHITE);

            if trans_ease > 0.01 {
                ui.add_space(6.0 * trans_ease);
                let a = (trans_ease * 180.0) as u8;
                centered_text(
                    ui,
                    self.transcription,
                    SUB_FONT_SIZE,
                    egui::Color32::from_rgba_unmultiplied(180, 180, 180, a),
                );
            }

            if transl_ease > 0.01 {
                ui.add_space(4.0 * transl_ease);
                let a = (transl_ease * 140.0) as u8;
                centered_text(
                    ui,
                    self.translation,
                    SUB_FONT_SIZE,
                    egui::Color32::from_rgba_unmultiplied(140, 140, 140, a),
                );
            }

            if show_example {
                ui.add_space(6.0 * example_ease);
                // Dimmer than the translation so it reads as supporting context.
                let a = (example_ease * 110.0) as u8;
                centered_text(
                    ui,
                    &example,
                    EXAMPLE_FONT_SIZE,
                    egui::Color32::from_rgba_unmultiplied(150, 150, 150, a),
                );
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn truncate_example_never_exceeds_budget_at_zero() {
        // The result must never exceed max_chars, including the degenerate 0.
        assert_eq!(truncate_example("hello", 0), "");
        assert_eq!(truncate_example("", 0), "");
        // And at 1 the ellipsis alone is within budget.
        assert_eq!(truncate_example("hello", 1).chars().count(), 1);
    }
}
