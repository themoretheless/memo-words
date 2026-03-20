mod app;
mod db;
mod tray;
mod ui;

use muda::{Menu, MenuItem};
use tray_icon::TrayIconBuilder;

fn main() -> eframe::Result<()> {
    let words = db::load_words();

    let menu = Menu::new();
    let quit_item = MenuItem::new("Quit", true, None);
    let quit_id = quit_item.id().clone();
    menu.append(&quit_item).unwrap();

    let _tray = TrayIconBuilder::new()
        .with_icon(tray::create_icon())
        .with_tooltip("Memo Words")
        .with_menu(Box::new(menu))
        .build()
        .expect("Failed to build tray icon");

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([3840.0, 2160.0])
            .with_position([0.0, 0.0])
            .with_decorations(false)
            .with_always_on_top()
            .with_transparent(true)
            .with_mouse_passthrough(true),
        ..Default::default()
    };

    eframe::run_native(
        "Memo Words",
        options,
        Box::new(|cc| {
            ui::setup_visuals(&cc.egui_ctx);
            ui::load_fonts(&cc.egui_ctx);
            Ok(Box::new(app::App::new(words, quit_id.clone())))
        }),
    )
}
