//! Card-surface decoration kept separate from content layout.

use crate::theme::dim;
use eframe::egui;

pub(super) fn paint_sheen(
    ui: &egui::Ui,
    rect: egui::Rect,
    corner_radius: f32,
    strength: f32,
    fade: f32,
    max_alpha: u8,
    height_fraction: f32,
) {
    if strength <= 0.0 {
        return;
    }
    let top_alpha = (max_alpha as f32 * strength.clamp(0.0, 1.0)) as u8;
    let top = dim(egui::Color32::from_white_alpha(top_alpha), fade);
    let bottom = egui::Color32::TRANSPARENT;
    let inset = corner_radius.min(rect.width() / 2.0);
    let left = rect.min.x + inset;
    let right = rect.max.x - inset;
    let y_top = rect.min.y + 1.0;
    let y_bottom = rect.min.y + rect.height() * height_fraction;
    let mut mesh = egui::epaint::Mesh::default();
    mesh.colored_vertex(egui::pos2(left, y_top), top);
    mesh.colored_vertex(egui::pos2(right, y_top), top);
    mesh.colored_vertex(egui::pos2(right, y_bottom), bottom);
    mesh.colored_vertex(egui::pos2(left, y_bottom), bottom);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    ui.painter().add(egui::Shape::mesh(mesh));
}
