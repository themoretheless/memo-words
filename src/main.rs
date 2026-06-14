mod app;
mod config;
mod db;
mod deck;
mod selector;
mod tray;
mod ui;

use db::{MongoWordSource, StaticWordSource, WithFallback, Word, WordSource};
use deck::Deck;
use muda::{Menu, MenuItem};
use selector::FrequencyWeighted;
use tray_icon::TrayIconBuilder;

fn main() -> eframe::Result<()> {
    let cfg = config::Config::load();

    // Composition root: pick the concrete word source here, behind the
    // WordSource trait, so nothing downstream depends on MongoDB directly.
    let source: Box<dyn WordSource> = if std::env::var("MEMO_BENCH").is_ok() {
        Box::new(StaticWordSource(vec![Word {
            word: "benchmark".into(),
            transcription: "/ˈbentʃmɑːk/".into(),
            translation: "эталонный тест".into(),
            frequency: 1,
            example: "We ran the benchmark twice.".into(),
        }]))
    } else {
        Box::new(WithFallback(MongoWordSource::default()))
    };
    let deck = Deck::new(source.load(), Box::new(FrequencyWeighted));

    let menu = Menu::new();
    let next_item = MenuItem::new("Next word", true, None);
    let pause_item = MenuItem::new("Pause / Resume", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    let menu_ids = app::MenuIds {
        next: next_item.id().clone(),
        pause: pause_item.id().clone(),
        quit: quit_item.id().clone(),
    };
    menu.append(&next_item).unwrap();
    menu.append(&pause_item).unwrap();
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
            Ok(Box::new(app::App::new(deck, menu_ids.clone(), cfg)))
        }),
    )
}
