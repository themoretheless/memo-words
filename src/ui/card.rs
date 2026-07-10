//! Card composition: geometry, reveal-aware width, and semantic line hierarchy.

use super::surface::paint_sheen;
use super::text::{centered_line, measure_height, measure_width, truncate_example};
use crate::config::Corner;
use crate::theme::{
    BORDER_COLOR, EXAMPLE_FONT_SIZE, EXAMPLE_INTENSITY, SHADOW, TRANSCRIPTION_FONT_SIZE,
    TRANSCRIPTION_INTENSITY, TRANSLATION_FONT_SIZE, TRANSLATION_INTENSITY, WORD_FONT_SIZE, card_bg,
    dim, faded_line,
};
use crate::timing::{fade_factor, settle_offset, smoothstep};
use eframe::egui;

const SCREEN_MARGIN: f32 = 40.0;
const INNER_MARGIN: f32 = 20.0;
pub const MIN_WIDTH: f32 = 150.0;
const MAX_WIDTH: f32 = 600.0;
const WIDGET_HEIGHT: f32 = 160.0;
const ACCENT_RULE_WIDTH: f32 = 28.0;
const ACCENT_RULE_THICKNESS: f32 = 2.0;
const ACCENT_RULE_GAP: f32 = 4.0;
const EXAMPLE_MAX_CHARS: usize = 64;
const WIDTH_TRANSITION: f32 = 0.5;

pub struct CardContent<'a> {
    pub word: &'a str,
    pub transcription: &'a str,
    pub translation: &'a str,
    pub example: &'a str,
}

pub struct CardTimeline {
    pub elapsed: f32,
    pub transcription_delay: f32,
    pub translation_delay: f32,
    pub example_delay: f32,
    pub fade_duration: f32,
}

pub struct CardStyle {
    pub corner: Corner,
    pub card_opacity: f32,
    pub corner_radius: f32,
    pub exit_alpha: f32,
    pub settle_px: f32,
    pub accent: Option<egui::Color32>,
    pub sheen: f32,
}

pub struct CardView<'a> {
    pub content: CardContent<'a>,
    pub timeline: CardTimeline,
    pub style: CardStyle,
    pub prev_width: f32,
}

impl CardView<'_> {
    fn example_text(&self) -> String {
        let trimmed = self.content.example.trim();
        if trimmed.is_empty() {
            String::new()
        } else {
            truncate_example(trimmed, EXAMPLE_MAX_CHARS)
        }
    }

    fn eases(&self) -> (f32, f32, f32) {
        let t = &self.timeline;
        (
            fade_factor(t.elapsed, t.transcription_delay, t.fade_duration),
            fade_factor(t.elapsed, t.translation_delay, t.fade_duration),
            fade_factor(t.elapsed, t.example_delay, t.fade_duration),
        )
    }

    pub fn compute_width(&self, ui: &egui::Ui) -> f32 {
        let word_w = measure_width(ui, self.content.word, WORD_FONT_SIZE);
        let transcription_w =
            measure_width(ui, self.content.transcription, TRANSCRIPTION_FONT_SIZE);
        let translation_w = measure_width(ui, self.content.translation, TRANSLATION_FONT_SIZE);
        let example_w = measure_width(ui, &self.example_text(), EXAMPLE_FONT_SIZE);
        let (transcription_ease, translation_ease, example_ease) = self.eases();

        let initial = smoothstep(self.timeline.elapsed / WIDTH_TRANSITION);
        let from = self.prev_width - 2.0 * INNER_MARGIN;
        let w0 = from + (word_w - from) * initial;
        let w1 = word_w.max(transcription_w);
        let w2 = w1.max(translation_w);
        let w3 = w2.max(example_w);
        let content = w0
            + (w1 - w0) * transcription_ease
            + (w2 - w1) * translation_ease
            + (w3 - w2) * example_ease;

        (content + 2.0 * INNER_MARGIN).clamp(MIN_WIDTH, MAX_WIDTH)
    }

    pub fn paint(&self, ui: &mut egui::Ui, widget_width: f32) {
        let rect = self.widget_rect(ui.max_rect(), widget_width);
        self.paint_surface(ui, rect);
        self.paint_content(ui, rect);
    }

    fn widget_rect(&self, screen: egui::Rect, width: f32) -> egui::Rect {
        let x = match self.style.corner {
            Corner::TopLeft | Corner::BottomLeft => screen.min.x + SCREEN_MARGIN,
            Corner::TopRight | Corner::BottomRight => screen.max.x - width - SCREEN_MARGIN,
        };
        let y = match self.style.corner {
            Corner::TopLeft | Corner::TopRight => screen.min.y + SCREEN_MARGIN,
            Corner::BottomLeft | Corner::BottomRight => {
                screen.max.y - WIDGET_HEIGHT - SCREEN_MARGIN
            }
        };
        egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(width, WIDGET_HEIGHT))
    }

    fn paint_surface(&self, ui: &egui::Ui, rect: egui::Rect) {
        let style = &self.style;
        let mut shadow = SHADOW;
        shadow.color = dim(SHADOW.color, style.exit_alpha);
        ui.painter().add(shadow.as_shape(rect, style.corner_radius));
        ui.painter().rect_filled(
            rect,
            style.corner_radius,
            dim(card_bg(style.card_opacity), style.exit_alpha),
        );
        paint_sheen(ui, rect, style.corner_radius, style.sheen, style.exit_alpha);
        ui.painter().rect_stroke(
            rect,
            style.corner_radius,
            egui::Stroke::new(1.0_f32, dim(BORDER_COLOR, style.exit_alpha)),
            egui::StrokeKind::Inside,
        );
    }

    fn paint_content(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let inner = rect.shrink(INNER_MARGIN);
        let (transcription_ease, translation_ease, example_ease) = self.eases();
        let word_ease = smoothstep(self.timeline.elapsed / WIDTH_TRANSITION);
        let example = self.example_text();
        let show_example = !example.is_empty() && example_ease > 0.01;
        let top_pad = self.top_padding(
            ui,
            inner.height(),
            &example,
            transcription_ease,
            translation_ease,
            example_ease,
        );
        let style = &self.style;
        let exit = style.exit_alpha;

        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
        child.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.add_space(top_pad);
            centered_line(
                ui,
                self.content.word,
                WORD_FONT_SIZE,
                dim(egui::Color32::WHITE, exit),
                settle_offset(style.settle_px, word_ease),
            );

            if let Some(accent) = style.accent {
                ui.add_space(ACCENT_RULE_GAP);
                centered_rule(
                    ui,
                    ACCENT_RULE_WIDTH,
                    dim(accent, word_ease * exit),
                    settle_offset(style.settle_px, word_ease),
                );
            }

            if transcription_ease > 0.01 {
                ui.add_space(6.0 * transcription_ease);
                centered_line(
                    ui,
                    self.content.transcription,
                    TRANSCRIPTION_FONT_SIZE,
                    dim(
                        faded_line(TRANSCRIPTION_INTENSITY, transcription_ease),
                        exit,
                    ),
                    settle_offset(style.settle_px, transcription_ease),
                );
            }

            if translation_ease > 0.01 {
                ui.add_space(4.0 * translation_ease);
                centered_line(
                    ui,
                    self.content.translation,
                    TRANSLATION_FONT_SIZE,
                    dim(faded_line(TRANSLATION_INTENSITY, translation_ease), exit),
                    settle_offset(style.settle_px, translation_ease),
                );
            }

            if show_example {
                ui.add_space(6.0 * example_ease);
                centered_line(
                    ui,
                    &example,
                    EXAMPLE_FONT_SIZE,
                    dim(faded_line(EXAMPLE_INTENSITY, example_ease), exit),
                    settle_offset(style.settle_px, example_ease),
                );
            }
        });
    }

    fn top_padding(
        &self,
        ui: &egui::Ui,
        available_height: f32,
        example: &str,
        transcription_ease: f32,
        translation_ease: f32,
        example_ease: f32,
    ) -> f32 {
        let mut height = measure_height(ui, self.content.word, WORD_FONT_SIZE);
        if self.style.accent.is_some() {
            height += ACCENT_RULE_GAP + ACCENT_RULE_THICKNESS;
        }
        if transcription_ease > 0.01 {
            height += 6.0 * transcription_ease
                + measure_height(ui, self.content.transcription, TRANSCRIPTION_FONT_SIZE);
        }
        if translation_ease > 0.01 {
            height += 4.0 * translation_ease
                + measure_height(ui, self.content.translation, TRANSLATION_FONT_SIZE);
        }
        if !example.is_empty() && example_ease > 0.01 {
            height += 6.0 * example_ease + measure_height(ui, example, EXAMPLE_FONT_SIZE);
        }
        ((available_height - height) / 2.0).max(0.0)
    }
}

fn centered_rule(ui: &mut egui::Ui, width: f32, color: egui::Color32, y_offset: f32) {
    let available = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(available, ACCENT_RULE_THICKNESS),
        egui::Sense::hover(),
    );
    let x = rect.min.x + (available - width) / 2.0;
    let bar = egui::Rect::from_min_size(
        egui::pos2(x, rect.min.y + y_offset),
        egui::vec2(width, ACCENT_RULE_THICKNESS),
    );
    ui.painter()
        .rect_filled(bar, ACCENT_RULE_THICKNESS / 2.0, color);
}
