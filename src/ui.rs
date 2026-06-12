use crate::config::Corner;
use eframe::egui;
use std::sync::Arc;

const CORNER_RADIUS: f32 = 16.0;
const SCREEN_MARGIN: f32 = 40.0;
const INNER_MARGIN: f32 = 20.0;
const BG_COLOR: egui::Color32 = egui::Color32::from_rgba_premultiplied(9, 9, 9, 77);

pub const MIN_WIDTH: f32 = 150.0;
pub const MAX_WIDTH: f32 = 600.0;
pub const WIDGET_HEIGHT: f32 = 160.0;
pub const WORD_FONT_SIZE: f32 = 32.0;
pub const SUB_FONT_SIZE: f32 = 15.0;

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
        fonts.font_data.insert("arial_unicode".into(), Arc::new(egui::FontData::from_owned(data)));
        fonts.families
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

pub fn measure_text_width(ui: &egui::Ui, text: &str, size: f32) -> f32 {
    ui.painter()
        .layout_no_wrap(text.into(), egui::FontId::proportional(size), egui::Color32::WHITE)
        .rect
        .width()
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
    let (rect, _) = ui.allocate_exact_size(egui::vec2(avail, galley.rect.height()), egui::Sense::hover());
    let x = rect.min.x + (avail - text_w) / 2.0;
    ui.painter().galley(egui::pos2(x, rect.min.y), galley, color);
}

pub struct CardView<'a> {
    pub word: &'a str,
    pub transcription: &'a str,
    pub translation: &'a str,
    pub elapsed: f32,
    pub prev_width: f32,
    pub transcription_delay: f32,
    pub translation_delay: f32,
    pub fade_duration: f32,
    pub corner: Corner,
}

impl<'a> CardView<'a> {
    pub fn compute_width(&self, ui: &egui::Ui) -> f32 {
        let word_w = measure_text_width(ui, self.word, WORD_FONT_SIZE);
        let trans_w = measure_text_width(ui, self.transcription, SUB_FONT_SIZE);
        let transl_w = measure_text_width(ui, self.translation, SUB_FONT_SIZE);

        let trans_ease = fade_factor(self.elapsed, self.transcription_delay, self.fade_duration);
        let transl_ease = fade_factor(self.elapsed, self.translation_delay, self.fade_duration);

        let initial = smoothstep(self.elapsed / WIDTH_TRANSITION);
        let from = self.prev_width - 2.0 * INNER_MARGIN;
        let w0 = from + (word_w - from) * initial;
        let w1 = word_w.max(trans_w);
        let w2 = w1.max(transl_w);
        let content = w0 + (w1 - w0) * trans_ease + (w2 - w1) * transl_ease;

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
            Corner::BottomLeft | Corner::BottomRight => screen.max.y - WIDGET_HEIGHT - SCREEN_MARGIN,
        };

        let widget_rect = egui::Rect::from_min_size(
            egui::pos2(x, y),
            egui::vec2(widget_w, WIDGET_HEIGHT),
        );

        ui.painter().rect_filled(widget_rect, CORNER_RADIUS, BG_COLOR);

        let inner = widget_rect.shrink(INNER_MARGIN);
        let trans_ease = fade_factor(self.elapsed, self.transcription_delay, self.fade_duration);
        let transl_ease = fade_factor(self.elapsed, self.translation_delay, self.fade_duration);

        let mut content_h = WORD_FONT_SIZE;
        if trans_ease > 0.01 { content_h += 6.0 * trans_ease + SUB_FONT_SIZE; }
        if transl_ease > 0.01 { content_h += 4.0 * transl_ease + SUB_FONT_SIZE; }
        let top_pad = ((inner.height() - content_h) / 2.0).max(0.0);

        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
        child.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.add_space(top_pad);
            centered_text(ui, self.word, WORD_FONT_SIZE, egui::Color32::WHITE);

            if trans_ease > 0.01 {
                ui.add_space(6.0 * trans_ease);
                let a = (trans_ease * 180.0) as u8;
                centered_text(ui, self.transcription, SUB_FONT_SIZE,
                    egui::Color32::from_rgba_unmultiplied(180, 180, 180, a));
            }

            if transl_ease > 0.01 {
                ui.add_space(4.0 * transl_ease);
                let a = (transl_ease * 140.0) as u8;
                centered_text(ui, self.translation, SUB_FONT_SIZE,
                    egui::Color32::from_rgba_unmultiplied(140, 140, 140, a));
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
}
