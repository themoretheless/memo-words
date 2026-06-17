use crate::config::{Corner, Rgb};
use eframe::egui;
use std::sync::Arc;

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

pub fn accent_rgba(rgb: Rgb, alpha: u8) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(rgb.r, rgb.g, rgb.b, alpha)
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
const HIGH_CONTRAST_BORDER_COLOR: egui::Color32 =
    egui::Color32::from_rgba_premultiplied(48, 48, 48, 48);

pub const MIN_WIDTH: f32 = 150.0;
pub const MAX_WIDTH: f32 = 600.0;
pub const WIDGET_HEIGHT: f32 = 160.0;
pub const WORD_FONT_SIZE: f32 = 32.0;
pub const WORD_MIN_FONT_SIZE: f32 = 22.0;
pub const SUB_FONT_SIZE: f32 = 15.0;
pub const SUB_MIN_FONT_SIZE: f32 = 12.0;
// The example sentence is subordinate to the translation: a touch smaller and
// dimmer so it reads as supporting context, not the answer.
pub const EXAMPLE_FONT_SIZE: f32 = 13.0;
pub const EXAMPLE_MIN_FONT_SIZE: f32 = 11.0;
pub const COUNTER_FONT_SIZE: f32 = 11.0;
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

pub fn contrast_alpha(progress: f32, normal: u8, high: u8, high_contrast: bool) -> u8 {
    let max = if high_contrast { high } else { normal };
    (progress.clamp(0.0, 1.0) * f32::from(max)) as u8
}

pub fn familiarity_factor(seen_count: usize, enabled: bool) -> f32 {
    if !enabled || seen_count <= 1 {
        return 1.0;
    }

    (1.0 - 0.08 * (seen_count - 1) as f32).max(0.72)
}

pub fn apply_alpha_factor(alpha: u8, factor: f32) -> u8 {
    (f32::from(alpha) * factor.clamp(0.0, 1.0)).round() as u8
}

pub fn contrast_border(high_contrast: bool) -> egui::Color32 {
    if high_contrast {
        HIGH_CONTRAST_BORDER_COLOR
    } else {
        BORDER_COLOR
    }
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

pub fn scale_font_size(measured_width: f32, base_size: f32, min_size: f32, max_width: f32) -> f32 {
    if measured_width <= 0.0 || max_width <= 0.0 || measured_width <= max_width {
        return base_size;
    }

    (base_size * max_width / measured_width).clamp(min_size, base_size)
}

pub fn fit_font_size(
    ui: &egui::Ui,
    text: &str,
    base_size: f32,
    min_size: f32,
    max_width: f32,
) -> f32 {
    let measured = measure_text_width(ui, text, base_size);
    scale_font_size(measured, base_size, min_size, max_width)
}

pub fn truncate_to_width(ui: &egui::Ui, text: &str, size: f32, max_width: f32) -> String {
    if max_width <= 0.0 {
        return String::new();
    }
    if measure_text_width(ui, text, size) <= max_width {
        return text.to_string();
    }

    let ellipsis = "…";
    if measure_text_width(ui, ellipsis, size) > max_width {
        return String::new();
    }

    let mut out = String::new();
    for ch in text.chars() {
        let candidate = format!("{out}{ch}{ellipsis}");
        if measure_text_width(ui, &candidate, size) > max_width {
            break;
        }
        out.push(ch);
    }

    format!("{}{}", out.trim_end(), ellipsis)
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

fn is_left_word_boundary(text: &str, idx: usize) -> bool {
    if idx == 0 {
        return true;
    }

    text[..idx]
        .chars()
        .next_back()
        .is_none_or(|c| !c.is_ascii_alphanumeric())
}

fn is_right_word_boundary(text: &str, idx: usize) -> bool {
    if idx == text.len() {
        return true;
    }

    text[idx..]
        .chars()
        .next()
        .is_none_or(|c| !c.is_ascii_alphanumeric())
}

pub fn find_word_span(text: &str, word: &str) -> Option<(usize, usize)> {
    let needle = word.trim();
    if needle.is_empty() {
        return None;
    }

    let lower_text = text.to_ascii_lowercase();
    let lower_needle = needle.to_ascii_lowercase();
    lower_text
        .match_indices(&lower_needle)
        .find_map(|(start, _)| {
            let end = start + lower_needle.len();
            is_left_word_boundary(text, start).then_some(())?;
            is_right_word_boundary(text, end).then_some((start, end))
        })
}

pub fn centered_example_text(
    ui: &mut egui::Ui,
    text: &str,
    word: &str,
    size: f32,
    color: egui::Color32,
    highlight: egui::Color32,
) {
    let Some((start, end)) = find_word_span(text, word) else {
        centered_text(ui, text, size, color);
        return;
    };

    let segments = [
        (&text[..start], color),
        (&text[start..end], highlight),
        (&text[end..], color),
    ];
    let galleys: Vec<_> = segments
        .iter()
        .map(|(segment, _)| {
            ui.painter().layout_no_wrap(
                (*segment).into(),
                egui::FontId::proportional(size),
                egui::Color32::PLACEHOLDER,
            )
        })
        .collect();

    let text_w: f32 = galleys.iter().map(|g| g.rect.width()).sum();
    let text_h = galleys.iter().map(|g| g.rect.height()).fold(0.0, f32::max);
    let avail = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(avail, text_h), egui::Sense::hover());
    let mut x = rect.min.x + (avail - text_w) / 2.0;
    for ((_, color), galley) in segments.iter().zip(galleys) {
        let w = galley.rect.width();
        let y = rect.min.y + (text_h - galley.rect.height()) / 2.0;
        ui.painter().galley(egui::pos2(x, y), galley, *color);
        x += w;
    }
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
    pub interval_progress: f32,
    pub show_pace_indicator: bool,
    pub accent_color: Rgb,
    pub session_count: usize,
    pub show_session_counter: bool,
    pub high_contrast: bool,
    pub screen_margin: f32,
    pub familiarity_factor: f32,
    pub show_placement_preview: bool,
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
        let margin = self.screen_margin.max(0.0);

        let x = match self.corner {
            Corner::TopLeft | Corner::BottomLeft => screen.min.x + margin,
            Corner::TopRight | Corner::BottomRight => screen.max.x - widget_w - margin,
        };
        let y = match self.corner {
            Corner::TopLeft | Corner::TopRight => screen.min.y + margin,
            Corner::BottomLeft | Corner::BottomRight => screen.max.y - WIDGET_HEIGHT - margin,
        };

        let widget_rect =
            egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(widget_w, WIDGET_HEIGHT));

        if self.show_placement_preview {
            self.paint_placement_preview(ui, screen, widget_rect, margin);
        }

        ui.painter()
            .add(SHADOW.as_shape(widget_rect, self.corner_radius));
        ui.painter()
            .rect_filled(widget_rect, self.corner_radius, card_bg(self.card_opacity));
        ui.painter().rect_stroke(
            widget_rect,
            self.corner_radius,
            egui::Stroke::new(1.0_f32, contrast_border(self.high_contrast)),
            egui::StrokeKind::Inside,
        );

        let inner = widget_rect.shrink(INNER_MARGIN);
        if self.show_pace_indicator {
            self.paint_pace_indicator(ui, widget_rect);
        }
        if self.show_session_counter {
            self.paint_session_counter(ui, widget_rect);
        }

        let (trans_ease, transl_ease, example_ease) = self.eases();
        let example = self.example_text();
        let content_w = inner.width().max(1.0);
        let word_size = fit_font_size(ui, self.word, WORD_FONT_SIZE, WORD_MIN_FONT_SIZE, content_w);
        let word = truncate_to_width(ui, self.word, word_size, content_w);
        let transcription_size = fit_font_size(
            ui,
            self.transcription,
            SUB_FONT_SIZE,
            SUB_MIN_FONT_SIZE,
            content_w,
        );
        let transcription =
            truncate_to_width(ui, self.transcription, transcription_size, content_w);
        let translation_size = fit_font_size(
            ui,
            self.translation,
            SUB_FONT_SIZE,
            SUB_MIN_FONT_SIZE,
            content_w,
        );
        let translation = truncate_to_width(ui, self.translation, translation_size, content_w);
        let example_size = fit_font_size(
            ui,
            &example,
            EXAMPLE_FONT_SIZE,
            EXAMPLE_MIN_FONT_SIZE,
            content_w,
        );
        let example = truncate_to_width(ui, &example, example_size, content_w);
        let show_example = !example.is_empty() && example_ease > 0.01;

        // Budget the REAL rendered row heights (what centered_text allocates),
        // not the nominal font sizes, so the block is centered accurately.
        let mut content_h = measure_text_height(ui, &word, word_size);
        if trans_ease > 0.01 {
            content_h +=
                6.0 * trans_ease + measure_text_height(ui, &transcription, transcription_size);
        }
        if transl_ease > 0.01 {
            content_h +=
                4.0 * transl_ease + measure_text_height(ui, &translation, translation_size);
        }
        if show_example {
            content_h += 6.0 * example_ease + measure_text_height(ui, &example, example_size);
        }
        let top_pad = ((inner.height() - content_h) / 2.0).max(0.0);

        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
        child.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.add_space(top_pad);
            centered_text(
                ui,
                &word,
                word_size,
                egui::Color32::from_white_alpha(apply_alpha_factor(255, self.familiarity_factor)),
            );

            if trans_ease > 0.01 {
                ui.add_space(6.0 * trans_ease);
                let a = apply_alpha_factor(
                    contrast_alpha(trans_ease, 180, 224, self.high_contrast),
                    self.familiarity_factor,
                );
                centered_text(
                    ui,
                    &transcription,
                    transcription_size,
                    egui::Color32::from_rgba_unmultiplied(180, 180, 180, a),
                );
            }

            if transl_ease > 0.01 {
                ui.add_space(4.0 * transl_ease);
                let a = apply_alpha_factor(
                    contrast_alpha(transl_ease, 140, 196, self.high_contrast),
                    self.familiarity_factor,
                );
                centered_text(
                    ui,
                    &translation,
                    translation_size,
                    egui::Color32::from_rgba_unmultiplied(140, 140, 140, a),
                );
            }

            if show_example {
                ui.add_space(6.0 * example_ease);
                // Dimmer than the translation so it reads as supporting context.
                let a = apply_alpha_factor(
                    contrast_alpha(example_ease, 110, 168, self.high_contrast),
                    self.familiarity_factor,
                );
                centered_example_text(
                    ui,
                    &example,
                    self.word,
                    example_size,
                    egui::Color32::from_rgba_unmultiplied(150, 150, 150, a),
                    accent_rgba(self.accent_color, a),
                );
            }
        });
    }

    fn paint_pace_indicator(&self, ui: &mut egui::Ui, widget_rect: egui::Rect) {
        let progress = self.interval_progress.clamp(0.0, 1.0);
        let inset = 18.0_f32.min(widget_rect.width() / 4.0);
        let track = egui::Rect::from_min_max(
            egui::pos2(widget_rect.min.x + inset, widget_rect.max.y - 10.0),
            egui::pos2(widget_rect.max.x - inset, widget_rect.max.y - 8.0),
        );
        ui.painter().rect_filled(
            track,
            1.0,
            egui::Color32::from_rgba_unmultiplied(
                255,
                255,
                255,
                if self.high_contrast { 34 } else { 16 },
            ),
        );
        if progress > 0.0 {
            let fill = egui::Rect::from_min_max(
                track.min,
                egui::pos2(track.min.x + track.width() * progress, track.max.y),
            );
            ui.painter().rect_filled(
                fill,
                1.0,
                accent_rgba(self.accent_color, if self.high_contrast { 120 } else { 72 }),
            );
        }
    }

    fn paint_session_counter(&self, ui: &mut egui::Ui, widget_rect: egui::Rect) {
        let text = self.session_count.to_string();
        let galley = ui.painter().layout_no_wrap(
            text,
            egui::FontId::proportional(COUNTER_FONT_SIZE),
            egui::Color32::PLACEHOLDER,
        );
        let pos = egui::pos2(
            widget_rect.max.x - 18.0 - galley.rect.width(),
            widget_rect.max.y - 28.0,
        );
        ui.painter().galley(
            pos,
            galley,
            accent_rgba(self.accent_color, if self.high_contrast { 136 } else { 88 }),
        );
    }

    fn paint_placement_preview(
        &self,
        ui: &mut egui::Ui,
        screen: egui::Rect,
        widget_rect: egui::Rect,
        margin: f32,
    ) {
        let safe = screen.shrink(margin);
        let guide = accent_rgba(self.accent_color, if self.high_contrast { 96 } else { 64 });
        ui.painter().rect_stroke(
            safe,
            0.0,
            egui::Stroke::new(1.0_f32, guide),
            egui::StrokeKind::Inside,
        );
        ui.painter().rect_stroke(
            widget_rect.expand(4.0),
            self.corner_radius + 4.0,
            egui::Stroke::new(1.0_f32, guide),
            egui::StrokeKind::Outside,
        );

        let notch = 18.0;
        for corner in [
            safe.left_top(),
            safe.right_top(),
            safe.left_bottom(),
            safe.right_bottom(),
        ] {
            let x_dir = if corner.x <= safe.center().x {
                1.0
            } else {
                -1.0
            };
            let y_dir = if corner.y <= safe.center().y {
                1.0
            } else {
                -1.0
            };
            ui.painter().line_segment(
                [corner, egui::pos2(corner.x + notch * x_dir, corner.y)],
                egui::Stroke::new(1.0_f32, guide),
            );
            ui.painter().line_segment(
                [corner, egui::pos2(corner.x, corner.y + notch * y_dir)],
                egui::Stroke::new(1.0_f32, guide),
            );
        }
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
    fn accent_rgba_uses_configured_rgb_and_alpha() {
        let color = accent_rgba(Rgb { r: 1, g: 2, b: 3 }, 255);
        assert_eq!(color.r(), 1);
        assert_eq!(color.g(), 2);
        assert_eq!(color.b(), 3);
        assert_eq!(color.a(), 255);
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
    fn contrast_alpha_uses_high_contrast_max_when_enabled() {
        assert_eq!(contrast_alpha(1.0, 100, 200, false), 100);
        assert_eq!(contrast_alpha(1.0, 100, 200, true), 200);
        assert_eq!(contrast_alpha(2.0, 100, 200, true), 200);
    }

    #[test]
    fn familiarity_factor_only_dims_repeated_words_when_enabled() {
        assert_eq!(familiarity_factor(1, false), 1.0);
        assert_eq!(familiarity_factor(5, false), 1.0);
        assert_eq!(familiarity_factor(1, true), 1.0);
        assert!((familiarity_factor(3, true) - 0.84).abs() < 1e-6);
        assert_eq!(familiarity_factor(99, true), 0.72);
    }

    #[test]
    fn apply_alpha_factor_scales_and_clamps() {
        assert_eq!(apply_alpha_factor(200, 0.5), 100);
        assert_eq!(apply_alpha_factor(200, 2.0), 200);
        assert_eq!(apply_alpha_factor(200, -1.0), 0);
    }

    #[test]
    fn contrast_border_switches_color() {
        assert_eq!(contrast_border(false), BORDER_COLOR);
        assert_eq!(contrast_border(true), HIGH_CONTRAST_BORDER_COLOR);
    }

    #[test]
    fn scale_font_size_keeps_fitting_text_unchanged() {
        assert_eq!(scale_font_size(80.0, 32.0, 22.0, 100.0), 32.0);
        assert_eq!(scale_font_size(0.0, 32.0, 22.0, 100.0), 32.0);
    }

    #[test]
    fn scale_font_size_reduces_oversized_text() {
        assert_eq!(scale_font_size(200.0, 32.0, 22.0, 100.0), 22.0);
        assert_eq!(scale_font_size(150.0, 30.0, 10.0, 100.0), 20.0);
    }

    #[test]
    fn find_word_span_matches_whole_words_case_insensitively() {
        assert_eq!(
            find_word_span("The keys are in my bag.", "the"),
            Some((0, 3))
        );
        assert_eq!(
            find_word_span("She wants to be a doctor.", "be"),
            Some((13, 15))
        );
    }

    #[test]
    fn find_word_span_ignores_substrings_inside_words() {
        assert_eq!(find_word_span("She speaks three languages.", "he"), None);
        assert_eq!(find_word_span("Travel around the world.", "a"), None);
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
