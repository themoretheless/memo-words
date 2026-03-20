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

// Animation timings (seconds)
pub const TRANSCRIPTION_DELAY: f32 = 5.0;
pub const TRANSLATION_DELAY: f32 = 10.0;
pub const FADE_DURATION: f32 = 1.0;
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

pub fn fade_factor(elapsed: f32, delay: f32) -> f32 {
    smoothstep((elapsed - delay) / FADE_DURATION)
}

pub fn measure_text_width(ui: &egui::Ui, text: &str, size: f32) -> f32 {
    ui.painter()
        .layout_no_wrap(text.into(), egui::FontId::proportional(size), egui::Color32::WHITE)
        .rect
        .width()
}

pub fn centered_text(ui: &mut egui::Ui, text: &str, size: f32, color: egui::Color32) {
    let galley = ui.painter().layout_no_wrap(
        text.into(),
        egui::FontId::proportional(size),
        color,
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
}

impl<'a> CardView<'a> {
    pub fn compute_width(&self, ui: &egui::Ui) -> f32 {
        let word_w = measure_text_width(ui, self.word, WORD_FONT_SIZE);
        let trans_w = measure_text_width(ui, self.transcription, SUB_FONT_SIZE);
        let transl_w = measure_text_width(ui, self.translation, SUB_FONT_SIZE);

        let trans_ease = fade_factor(self.elapsed, TRANSCRIPTION_DELAY);
        let transl_ease = fade_factor(self.elapsed, TRANSLATION_DELAY);

        let initial = smoothstep(self.elapsed / WIDTH_TRANSITION);
        let from = self.prev_width - 2.0 * INNER_MARGIN;
        let w0 = from + (word_w - from) * initial;
        let w1 = word_w.max(trans_w);
        let w2 = w1.max(transl_w);
        let content = w0 + (w1 - w0) * trans_ease + (w2 - w1) * transl_ease;

        (content + 2.0 * INNER_MARGIN).clamp(MIN_WIDTH, MAX_WIDTH)
    }

    pub fn paint(&self, ui: &mut egui::Ui) {
        let screen = ui.max_rect();
        let widget_w = self.compute_width(ui);

        let widget_rect = egui::Rect::from_min_size(
            egui::pos2(
                screen.max.x - widget_w - SCREEN_MARGIN,
                screen.max.y - WIDGET_HEIGHT - SCREEN_MARGIN,
            ),
            egui::vec2(widget_w, WIDGET_HEIGHT),
        );

        ui.painter().rect_filled(widget_rect, CORNER_RADIUS, BG_COLOR);

        let inner = widget_rect.shrink(INNER_MARGIN);
        let trans_ease = fade_factor(self.elapsed, TRANSCRIPTION_DELAY);
        let transl_ease = fade_factor(self.elapsed, TRANSLATION_DELAY);

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
