mod app;
mod config;
mod db;
mod deck;
mod selector;
mod tray;
mod ui;

use db::{MongoWordSource, StaticWordSource, WithFallback, Word, WordSource};
use deck::Deck;
use muda::{CheckMenuItem, Menu, MenuId, MenuItem};
use selector::FrequencyWeighted;
use tray_icon::TrayIconBuilder;

fn main() -> eframe::Result<()> {
    let cfg = config::Config::load();
    let bench = std::env::var("MEMO_BENCH").is_ok();

    // Composition root: pick the concrete word source here, behind the
    // WordSource trait, so nothing downstream depends on MongoDB directly.
    let source: Box<dyn WordSource> = if bench {
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

    let (menu_ids, menu_checks, menu) = if bench {
        (
            app::MenuIds {
                next: MenuId::from("next"),
                speak: MenuId::from("speak"),
                copy: MenuId::from("copy"),
                copy_recent: MenuId::from("copy_recent"),
                copy_summary: MenuId::from("copy_summary"),
                copy_anki: MenuId::from("copy_anki"),
                copy_health: MenuId::from("copy_health"),
                reset_session: MenuId::from("reset_session"),
                toggle_recall: MenuId::from("toggle_recall"),
                toggle_pace: MenuId::from("toggle_pace"),
                pause: MenuId::from("pause"),
                quit: MenuId::from("quit"),
            },
            None,
            None,
        )
    } else {
        let menu = Menu::new();
        let next_item = MenuItem::new("Next word", true, None);
        let speak_item = MenuItem::new("Speak current word", true, None);
        let copy_item = MenuItem::new("Copy current word", true, None);
        let recent_item = MenuItem::new("Copy recent words", true, None);
        let summary_item = MenuItem::new("Copy session summary", true, None);
        let anki_item = MenuItem::new("Copy session as Anki TSV", true, None);
        let health_item = MenuItem::new("Copy deck health report", true, None);
        let reset_item = MenuItem::new("Reset session", true, None);
        let recall_item = CheckMenuItem::new("Recall mode", true, cfg.recall_mode, None);
        let pace_item = CheckMenuItem::new("Pace indicator", true, cfg.pace_indicator, None);
        let pause_item = CheckMenuItem::new("Paused", true, false, None);
        let quit_item = MenuItem::new("Quit", true, None);
        let menu_ids = app::MenuIds {
            next: next_item.id().clone(),
            speak: speak_item.id().clone(),
            copy: copy_item.id().clone(),
            copy_recent: recent_item.id().clone(),
            copy_summary: summary_item.id().clone(),
            copy_anki: anki_item.id().clone(),
            copy_health: health_item.id().clone(),
            reset_session: reset_item.id().clone(),
            toggle_recall: recall_item.id().clone(),
            toggle_pace: pace_item.id().clone(),
            pause: pause_item.id().clone(),
            quit: quit_item.id().clone(),
        };
        let menu_checks = app::MenuChecks {
            pause: pause_item.clone(),
            recall: recall_item.clone(),
            pace: pace_item.clone(),
        };
        menu.append(&next_item).unwrap();
        menu.append(&speak_item).unwrap();
        menu.append(&copy_item).unwrap();
        menu.append(&recent_item).unwrap();
        menu.append(&summary_item).unwrap();
        menu.append(&anki_item).unwrap();
        menu.append(&health_item).unwrap();
        menu.append(&reset_item).unwrap();
        menu.append(&recall_item).unwrap();
        menu.append(&pace_item).unwrap();
        menu.append(&pause_item).unwrap();
        menu.append(&quit_item).unwrap();
        (menu_ids, Some(menu_checks), Some(menu))
    };

    let _tray = menu.map(|menu| {
        TrayIconBuilder::new()
            .with_icon(tray::create_icon())
            .with_tooltip("Memo Words")
            .with_menu(Box::new(menu))
            .build()
            .expect("Failed to build tray icon")
    });

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
            Ok(Box::new(app::App::new(
                deck,
                menu_ids.clone(),
                menu_checks.clone(),
                cfg,
            )))
        }),
    )
}
