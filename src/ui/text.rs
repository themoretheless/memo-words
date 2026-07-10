//! Text measurement, elision, and centered line painting.

use eframe::egui;

pub(super) fn truncate_example(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let kept: String = s.chars().take(max_chars - 1).collect();
    format!("{}…", kept.trim_end())
}

pub(super) fn measure_width(ui: &egui::Ui, text: &str, size: f32) -> f32 {
    layout(ui, text, size).rect.width()
}

pub(super) fn measure_height(ui: &egui::Ui, text: &str, size: f32) -> f32 {
    layout(ui, text, size).rect.height()
}

pub(super) fn centered_line(
    ui: &mut egui::Ui,
    text: &str,
    size: f32,
    color: egui::Color32,
    y_offset: f32,
) {
    let avail = ui.available_width();
    let measured = layout(ui, text, size);
    let fitted_size = fit_font_size(size, measured.rect.width(), avail);
    let galley = if fitted_size < size {
        layout(ui, text, fitted_size)
    } else {
        measured
    };
    let text_w = galley.rect.width();
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(avail, galley.rect.height()),
        egui::Sense::hover(),
    );
    let x = rect.min.x + (avail - text_w) / 2.0;
    ui.painter()
        .galley(egui::pos2(x, rect.min.y + y_offset), galley, color);
}

fn fit_font_size(size: f32, measured_width: f32, available_width: f32) -> f32 {
    if measured_width <= available_width || measured_width <= 0.0 {
        size
    } else {
        size * (available_width.max(0.0) / measured_width)
    }
}

fn layout(ui: &egui::Ui, text: &str, size: f32) -> std::sync::Arc<egui::Galley> {
    // A stable placeholder color keeps the galley-cache key unchanged while
    // fade alpha changes every frame. The actual color is supplied at paint.
    ui.painter().layout_no_wrap(
        text.into(),
        egui::FontId::proportional(size),
        egui::Color32::PLACEHOLDER,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_example_is_untouched() {
        assert_eq!(truncate_example("Have a nice day!", 64), "Have a nice day!");
        assert_eq!(truncate_example("", 64), "");
    }

    #[test]
    fn long_example_uses_an_ellipsis_within_budget() {
        let out = truncate_example(
            "This is a rather long example sentence that should be cut.",
            20,
        );
        assert!(out.ends_with('…'));
        assert_eq!(out.chars().count(), 20);
    }

    #[test]
    fn truncation_respects_unicode_boundaries() {
        let out = truncate_example("Это довольно длинный пример предложения", 10);
        assert_eq!(out.chars().count(), 10);
        assert!(out.ends_with('…'));
    }

    #[test]
    fn zero_and_one_character_budgets_are_safe() {
        assert_eq!(truncate_example("hello", 0), "");
        assert_eq!(truncate_example("hello", 1).chars().count(), 1);
    }

    #[test]
    fn oversized_lines_scale_to_the_available_width() {
        assert_eq!(fit_font_size(20.0, 200.0, 100.0), 10.0);
        assert_eq!(fit_font_size(20.0, 80.0, 100.0), 20.0);
        assert_eq!(fit_font_size(20.0, 0.0, 100.0), 20.0);
    }
}
