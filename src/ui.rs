use crate::config::Corner;
use crate::theme::{
    BORDER_COLOR, EXAMPLE_FONT_SIZE, EXAMPLE_INTENSITY, SHADOW, SHEEN_HEIGHT_FRAC, SHEEN_MAX_ALPHA,
    TRANSCRIPTION_FONT_SIZE, TRANSCRIPTION_INTENSITY, TRANSLATION_FONT_SIZE, TRANSLATION_INTENSITY,
    WORD_FONT_SIZE, card_bg, dim, faded_line,
};
use crate::timing::{fade_factor, settle_offset, smoothstep};
use eframe::egui;
use std::sync::Arc;

const SCREEN_MARGIN: f32 = 40.0;
const INNER_MARGIN: f32 = 20.0;

pub const MIN_WIDTH: f32 = 150.0;
pub const MAX_WIDTH: f32 = 600.0;
pub const WIDGET_HEIGHT: f32 = 160.0;

// The optional accent rule under the headword: a short, thin, rounded bar. Kept
// small and tight to the word so it reads as an underline, not a divider, and
// so it barely adds to the content height inside the fixed card.
const ACCENT_RULE_WIDTH: f32 = 28.0;
const ACCENT_RULE_THICKNESS: f32 = 2.0;
const ACCENT_RULE_GAP: f32 = 4.0;

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

/// Paint a faint top-to-transparent vertical sheen inside the card, a faux
/// material highlight. `strength` (0..1) scales the highlight alpha, `fade` is
/// the exit-fade multiplier. The gradient is inset horizontally by the corner
/// radius and confined to the top, so it never spills past the rounded corners
/// and fades out before the lower lines. A `strength` of 0 draws nothing.
fn paint_sheen(ui: &egui::Ui, rect: egui::Rect, corner_radius: f32, strength: f32, fade: f32) {
    if strength <= 0.0 {
        return;
    }
    let top_alpha = (SHEEN_MAX_ALPHA as f32 * strength.clamp(0.0, 1.0)) as u8;
    let top = dim(egui::Color32::from_white_alpha(top_alpha), fade);
    let bottom = egui::Color32::TRANSPARENT;
    let inset = corner_radius.min(rect.width() / 2.0);
    let left = rect.min.x + inset;
    let right = rect.max.x - inset;
    let y_top = rect.min.y + 1.0;
    let y_bot = rect.min.y + rect.height() * SHEEN_HEIGHT_FRAC;
    let mut mesh = egui::epaint::Mesh::default();
    mesh.colored_vertex(egui::pos2(left, y_top), top);
    mesh.colored_vertex(egui::pos2(right, y_top), top);
    mesh.colored_vertex(egui::pos2(right, y_bot), bottom);
    mesh.colored_vertex(egui::pos2(left, y_bot), bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    ui.painter().add(egui::Shape::mesh(mesh));
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

pub fn centered_text(
    ui: &mut egui::Ui,
    text: &str,
    size: f32,
    color: egui::Color32,
    y_offset: f32,
) {
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
    // `y_offset` shifts only the drawn galley, not the allocated slot, so the
    // line drifts inside its row (the entrance settle) without re-laying-out the
    // block or moving its neighbours.
    ui.painter()
        .galley(egui::pos2(x, rect.min.y + y_offset), galley, color);
}

/// Draw a short, thin, rounded accent rule centred in its own row, used as a
/// subtle underline beneath the headword. `y_offset` matches the headword's
/// entrance settle so the rule drifts with it; the colour carries the fade and
/// exit-alpha already applied by the caller.
fn centered_rule(ui: &mut egui::Ui, width: f32, color: egui::Color32, y_offset: f32) {
    let avail = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(avail, ACCENT_RULE_THICKNESS),
        egui::Sense::hover(),
    );
    let x = rect.min.x + (avail - width) / 2.0;
    let bar = egui::Rect::from_min_size(
        egui::pos2(x, rect.min.y + y_offset),
        egui::vec2(width, ACCENT_RULE_THICKNESS),
    );
    ui.painter()
        .rect_filled(bar, ACCENT_RULE_THICKNESS / 2.0, color);
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
    /// Points each line drifts up from as it fades in. 0 leaves lines in place.
    pub settle_px: f32,
    /// Optional accent colour for a thin rule under the headword. None = no rule.
    pub accent: Option<egui::Color32>,
    /// Strength (0..1) of the top sheen highlight. 0 leaves the fill flat.
    pub sheen: f32,
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
        // Sheen sits over the fill but under the border and text.
        paint_sheen(ui, widget_rect, self.corner_radius, self.sheen, e);
        ui.painter().rect_stroke(
            widget_rect,
            self.corner_radius,
            egui::Stroke::new(1.0_f32, dim(BORDER_COLOR, e)),
            egui::StrokeKind::Inside,
        );

        let inner = widget_rect.shrink(INNER_MARGIN);
        let (trans_ease, transl_ease, example_ease) = self.eases();
        // The headword has no fade, so it settles on the same curve that grows
        // the card width on entrance.
        let word_ease = smoothstep(self.elapsed / WIDTH_TRANSITION);
        let example = self.example_text();
        let show_example = !example.is_empty() && example_ease > 0.01;

        // Budget the REAL rendered row heights (what centered_text allocates),
        // not the nominal font sizes, so the block is centered accurately.
        let mut content_h = measure_text_height(ui, self.word, WORD_FONT_SIZE);
        if self.accent.is_some() {
            content_h += ACCENT_RULE_GAP + ACCENT_RULE_THICKNESS;
        }
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
            centered_text(
                ui,
                self.word,
                WORD_FONT_SIZE,
                dim(egui::Color32::WHITE, e),
                settle_offset(self.settle_px, word_ease),
            );

            if let Some(accent) = self.accent {
                ui.add_space(ACCENT_RULE_GAP);
                // Ease the rule in with the headword's curve and carry the exit
                // fade, so it appears and leaves with the rest of the card.
                centered_rule(
                    ui,
                    ACCENT_RULE_WIDTH,
                    dim(accent, word_ease * e),
                    settle_offset(self.settle_px, word_ease),
                );
            }

            if trans_ease > 0.01 {
                ui.add_space(6.0 * trans_ease);
                centered_text(
                    ui,
                    self.transcription,
                    TRANSCRIPTION_FONT_SIZE,
                    dim(faded_line(TRANSCRIPTION_INTENSITY, trans_ease), e),
                    settle_offset(self.settle_px, trans_ease),
                );
            }

            if transl_ease > 0.01 {
                ui.add_space(4.0 * transl_ease);
                centered_text(
                    ui,
                    self.translation,
                    TRANSLATION_FONT_SIZE,
                    dim(faded_line(TRANSLATION_INTENSITY, transl_ease), e),
                    settle_offset(self.settle_px, transl_ease),
                );
            }

            if show_example {
                ui.add_space(6.0 * example_ease);
                centered_text(
                    ui,
                    &example,
                    EXAMPLE_FONT_SIZE,
                    dim(faded_line(EXAMPLE_INTENSITY, example_ease), e),
                    settle_offset(self.settle_px, example_ease),
                );
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
