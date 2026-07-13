mod app;
mod config;
mod deck;
mod diagnostics;
mod fallback;
mod loading;
mod model;
mod platform;
mod selector;
mod session;
mod source;
mod theme;
mod timing;
mod tray;
mod ui;
mod wake;

use deck::Deck;
use model::Word;
use muda::{Menu, MenuItem, PredefinedMenuItem};
use selector::FrequencyWeighted;
use source::{MongoWordSource, SourceController, StaticWordSource, WithFallback, WordSource};
use tray_icon::TrayIconBuilder;

fn main() -> eframe::Result<()> {
    let cfg = config::Config::load();
    let bench = std::env::var("MEMO_BENCH").is_ok();

    // The normal app always has a usable deck before the window is created.
    // Benchmark mode stays deterministic and never starts the remote adapter.
    let initial_words = if bench {
        StaticWordSource(vec![Word {
            word: "benchmark".into(),
            transcription: "/ˈbentʃmɑːk/".into(),
            translation: "эталонный тест".into(),
            frequency: 1,
            example: "We ran the benchmark twice.".into(),
        }])
        .load()
        .words
    } else {
        fallback::fallback_words()
    };
    let initial_word_count = initial_words.len();
    let deck = Deck::new(initial_words, Box::new(FrequencyWeighted))
        .with_recap_chance(cfg.learning.recap_chance);

    let menu = Menu::new();
    let source_menu = tray::SourceMenu::new(bench);
    let next_item = MenuItem::new("Next word", true, None);
    let pause_item = MenuItem::new("Pause / Resume", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    let separator_1 = PredefinedMenuItem::separator();
    let separator_2 = PredefinedMenuItem::separator();
    let separator_3 = PredefinedMenuItem::separator();
    let menu_ids = app::MenuIds {
        next: next_item.id().clone(),
        pause: pause_item.id().clone(),
        reload: source_menu.reload.id().clone(),
        diagnostics: source_menu.diagnostics.id().clone(),
        quit: quit_item.id().clone(),
    };
    // Tray failures are not fatal: the overlay still cycles words on the timer
    // without a tray menu, so log and continue rather than crash on startup.
    if let Err(e) = menu.append_items(&[
        &source_menu.status,
        &separator_1,
        &next_item,
        &pause_item,
        &separator_2,
        &source_menu.reload,
        &source_menu.diagnostics,
        &separator_3,
        &quit_item,
    ]) {
        eprintln!("memo-words: could not build the complete tray menu: {e}");
    }

    let mut tray_builder = TrayIconBuilder::new()
        .with_tooltip("Memo Words")
        .with_menu(Box::new(menu));
    if let Some(icon) = tray::create_icon() {
        tray_builder = tray_builder.with_icon(icon);
    }
    let _tray = match tray_builder.build() {
        Ok(tray) => Some(tray),
        Err(e) => {
            eprintln!("memo-words: tray icon unavailable ({e}); continuing without it.");
            None
        }
    };

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
            let source = if bench {
                None
            } else {
                let source_ctx = cc.egui_ctx.clone();
                Some(SourceController::new(
                    initial_word_count,
                    Box::new(move |attempt_id| {
                        let wake_ctx = source_ctx.clone();
                        loading::spawn(
                            attempt_id,
                            WithFallback(MongoWordSource::default()),
                            move || wake_ctx.request_repaint(),
                        )
                    }),
                ))
            };
            // Pick the TTS adapter at the composition root: speak aloud only when
            // configured, otherwise a no-op speaker. App just routes to the port.
            let speaker: Box<dyn platform::Speaker> = if cfg.learning.speak {
                Box::new(platform::SystemSpeaker)
            } else {
                Box::new(platform::NullSpeaker)
            };
            Ok(Box::new(app::App::new(
                deck,
                menu_ids.clone(),
                cfg,
                source,
                Some(source_menu.clone()),
                speaker,
            )))
        }),
    )
}
