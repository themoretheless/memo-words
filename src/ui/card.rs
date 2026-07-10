//! Card composition: geometry, reveal-aware width, and semantic hierarchy.

use super::surface::paint_sheen;
use super::text::{centered_line, measure_height, measure_width, truncate_example};
use crate::config::Corner;
use crate::theme::{Theme, dim, faded};
use crate::timing::{fade_factor, settle_offset, smoothstep};
use eframe::egui;

const EXAMPLE_MAX_CHARS: usize = 64;

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
    pub reduce_motion: bool,
}

pub struct CardStyle {
    pub corner: Corner,
    pub card_opacity: f32,
    pub corner_radius: f32,
    pub exit_alpha: f32,
    pub settle_px: f32,
    pub accent: Option<egui::Color32>,
    pub sheen: f32,
    pub theme: Theme,
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
        let timeline = &self.timeline;
        (
            fade_factor(
                timeline.elapsed,
                timeline.transcription_delay,
                timeline.fade_duration,
            ),
            fade_factor(
                timeline.elapsed,
                timeline.translation_delay,
                timeline.fade_duration,
            ),
            fade_factor(
                timeline.elapsed,
                timeline.example_delay,
                timeline.fade_duration,
            ),
        )
    }

    pub fn compute_width(&self, ui: &egui::Ui) -> f32 {
        let fonts = self.style.theme.type_scale;
        let metrics = self.style.theme.metrics;
        let word_width = measure_width(ui, self.content.word, fonts.word);
        let transcription_width =
            measure_width(ui, self.content.transcription, fonts.transcription);
        let translation_width = measure_width(ui, self.content.translation, fonts.translation);
        let example_width = measure_width(ui, &self.example_text(), fonts.example);
        let (transcription_ease, translation_ease, example_ease) = self.eases();

        let initial = smoothstep(self.timeline.elapsed / metrics.width_transition);
        let from = self.prev_width - 2.0 * metrics.inner_margin;
        let word_stage = from + (word_width - from) * initial;
        let transcription_stage = word_width.max(transcription_width);
        let translation_stage = transcription_stage.max(translation_width);
        let final_stage = translation_stage.max(example_width);
        let content_width = if self.timeline.reduce_motion {
            final_stage
        } else {
            word_stage
                + (transcription_stage - word_stage) * transcription_ease
                + (translation_stage - transcription_stage) * translation_ease
                + (final_stage - translation_stage) * example_ease
        };

        let viewport_max =
            (ui.max_rect().width() - 2.0 * metrics.screen_margin).max(metrics.min_width);
        let max_width = metrics.max_width.min(viewport_max);
        (content_width + 2.0 * metrics.inner_margin).clamp(metrics.min_width, max_width)
    }

    pub fn paint(&self, ui: &mut egui::Ui, widget_width: f32) {
        let rect = self.widget_rect(ui.max_rect(), widget_width);
        self.paint_surface(ui, rect);
        self.paint_content(ui, rect);
    }

    fn widget_rect(&self, screen: egui::Rect, width: f32) -> egui::Rect {
        let metrics = self.style.theme.metrics;
        let x = match self.style.corner {
            Corner::TopLeft | Corner::BottomLeft => screen.min.x + metrics.screen_margin,
            Corner::TopRight | Corner::BottomRight => screen.max.x - width - metrics.screen_margin,
        };
        let y = match self.style.corner {
            Corner::TopLeft | Corner::TopRight => screen.min.y + metrics.screen_margin,
            Corner::BottomLeft | Corner::BottomRight => {
                screen.max.y - metrics.height - metrics.screen_margin
            }
        };
        egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(width, metrics.height))
    }

    fn paint_surface(&self, ui: &egui::Ui, rect: egui::Rect) {
        let style = &self.style;
        let theme = style.theme;
        let mut shadow = theme.shadow;
        shadow.color = dim(theme.shadow.color, style.exit_alpha);
        ui.painter().add(shadow.as_shape(rect, style.corner_radius));
        ui.painter().rect_filled(
            rect,
            style.corner_radius,
            dim(theme.card_background(style.card_opacity), style.exit_alpha),
        );
        paint_sheen(
            ui,
            rect,
            style.corner_radius,
            style.sheen,
            style.exit_alpha,
            theme.sheen_max_alpha,
            theme.sheen_height_fraction,
        );
        ui.painter().rect_stroke(
            rect,
            style.corner_radius,
            egui::Stroke::new(1.0_f32, dim(theme.border, style.exit_alpha)),
            egui::StrokeKind::Inside,
        );
    }

    fn paint_content(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let theme = self.style.theme;
        let fonts = theme.type_scale;
        let metrics = theme.metrics;
        let inner = rect.shrink(metrics.inner_margin);
        let (transcription_ease, translation_ease, example_ease) = self.eases();
        let word_ease = if self.timeline.reduce_motion {
            1.0
        } else {
            smoothstep(self.timeline.elapsed / metrics.width_transition)
        };
        let example = self.example_text();
        let show_example = !example.is_empty() && example_ease > 0.01;
        let top_padding = self.top_padding(
            ui,
            inner.height(),
            &example,
            transcription_ease,
            translation_ease,
            example_ease,
        );
        let style = &self.style;
        let exit = style.exit_alpha;
        let spacing_scale = fonts.word / 32.0;

        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(inner));
        child.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.add_space(top_padding);
            centered_line(
                ui,
                self.content.word,
                fonts.word,
                dim(theme.text.word, exit),
                settle_offset(style.settle_px, word_ease),
            );

            if let Some(accent) = style.accent {
                ui.add_space(metrics.accent_gap);
                centered_rule(
                    ui,
                    metrics.accent_width,
                    metrics.accent_thickness,
                    dim(accent, word_ease * exit),
                    settle_offset(style.settle_px, word_ease),
                );
            }

            if transcription_ease > 0.01 {
                ui.add_space(6.0 * spacing_scale * transcription_ease);
                centered_line(
                    ui,
                    self.content.transcription,
                    fonts.transcription,
                    dim(faded(theme.text.transcription, transcription_ease), exit),
                    settle_offset(style.settle_px, transcription_ease),
                );
            }

            if translation_ease > 0.01 {
                ui.add_space(4.0 * spacing_scale * translation_ease);
                centered_line(
                    ui,
                    self.content.translation,
                    fonts.translation,
                    dim(faded(theme.text.translation, translation_ease), exit),
                    settle_offset(style.settle_px, translation_ease),
                );
            }

            if show_example {
                ui.add_space(6.0 * spacing_scale * example_ease);
                centered_line(
                    ui,
                    &example,
                    fonts.example,
                    dim(faded(theme.text.example, example_ease), exit),
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
        let fonts = self.style.theme.type_scale;
        let metrics = self.style.theme.metrics;
        let spacing_scale = fonts.word / 32.0;
        let mut height = measure_height(ui, self.content.word, fonts.word);
        if self.style.accent.is_some() {
            height += metrics.accent_gap + metrics.accent_thickness;
        }
        if transcription_ease > 0.01 {
            height += 6.0 * spacing_scale * transcription_ease
                + measure_height(ui, self.content.transcription, fonts.transcription);
        }
        if translation_ease > 0.01 {
            height += 4.0 * spacing_scale * translation_ease
                + measure_height(ui, self.content.translation, fonts.translation);
        }
        if !example.is_empty() && example_ease > 0.01 {
            height +=
                6.0 * spacing_scale * example_ease + measure_height(ui, example, fonts.example);
        }
        ((available_height - height) / 2.0).max(0.0)
    }
}

fn centered_rule(
    ui: &mut egui::Ui,
    width: f32,
    thickness: f32,
    color: egui::Color32,
    y_offset: f32,
) {
    let available = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(available, thickness), egui::Sense::hover());
    let x = rect.min.x + (available - width) / 2.0;
    let bar = egui::Rect::from_min_size(
        egui::pos2(x, rect.min.y + y_offset),
        egui::vec2(width, thickness),
    );
    ui.painter().rect_filled(bar, thickness / 2.0, color);
}
