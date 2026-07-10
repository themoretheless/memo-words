use crate::config::Config;
use crate::deck::Deck;
use crate::platform::Speaker;
use crate::session::SessionClock;
use crate::theme::Theme;
use crate::timing;
use crate::ui::{CardContent, CardStyle, CardTimeline, CardView};
use eframe::egui;
use muda::MenuEvent;
use rand::RngExt;
use rand::rng;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

// Repaint cadence while a card is animating in (word -> transcription ->
// translation fades). ~60 fps keeps the fades smooth.
const ANIM_FRAME: Duration = Duration::from_millis(16);

// Idle window measured by the frame-counter benchmark (MEMO_BENCH=1).
const BENCH_SECS: u64 = 10;

#[derive(Clone)]
pub struct MenuIds {
    pub next: muda::MenuId,
    pub pause: muda::MenuId,
    pub quit: muda::MenuId,
}

/// The eframe adapter (a humble object): it owns the timing, tray-menu wiring,
/// and rendering, and delegates word rotation to `Deck`. It deliberately holds
/// no selection or word-storage logic of its own.
pub struct App {
    deck: Deck,
    clock: SessionClock,
    prev_width: f32,
    theme: Theme,
    started: bool,
    menu_ids: MenuIds,
    menu_tx: Option<Sender<muda::MenuId>>,
    menu_rx: Receiver<muda::MenuId>,
    cfg: Config,
    bench: bool,
    frames: Arc<AtomicUsize>,
    // The TTS port. `App` depends on the capability, not the OS mechanism, so the
    // composition root chooses the real speaker and tests inject a double.
    speaker: Box<dyn Speaker>,
}

impl App {
    pub fn new(deck: Deck, menu_ids: MenuIds, cfg: Config, speaker: Box<dyn Speaker>) -> Self {
        let (menu_tx, menu_rx) = std::sync::mpsc::channel();
        let now = Instant::now();
        let theme = Theme::from_config(&cfg);
        Self {
            deck,
            clock: SessionClock::new(now, Duration::from_secs(cfg.timing.interval_secs)),
            prev_width: theme.metrics.min_width,
            theme,
            started: false,
            menu_ids,
            menu_tx: Some(menu_tx),
            menu_rx,
            cfg,
            bench: std::env::var("MEMO_BENCH").is_ok(),
            frames: Arc::new(AtomicUsize::new(0)),
            speaker,
        }
    }

    /// Show the next word: advance the deck, reset the timers, roll a fresh
    /// interval, and speak it through the speaker port.
    fn advance(&mut self) {
        self.deck.advance();
        // Roll the interval against the new word's frequency so rare-word dwell
        // (if enabled) can stretch harder words. Unknown/absent rank reads as
        // rarest, matching the selector's convention.
        let frequency = self.deck.current().map(|w| w.frequency).unwrap_or(0);
        let now = Instant::now();
        let interval = self.roll_interval(frequency);
        self.clock.start_word(now, interval);

        // Always route through the speaker port; whether it makes a sound is the
        // composition root's choice of speaker (System vs Null), not App's job.
        if let Some(w) = self.deck.current() {
            self.speaker.speak(&w.word);
        }
    }

    // Time the current word stays up: the pure base interval from `timing`
    // (optionally stretched for rarer words), then jittered by +/- jitter_secs
    // here so the cadence doesn't feel metronomic. The jitter lives in `App`
    // because it needs an RNG; the deterministic part stays pure and tested in
    // `timing`. Clamped to >=1s.
    fn roll_interval(&self, frequency: i32) -> Duration {
        let base = timing::dwelled_base_secs(&self.cfg, frequency);
        if self.cfg.timing.jitter_secs == 0 {
            return Duration::from_secs(base.max(1) as u64);
        }
        let j = self.cfg.timing.jitter_secs as i64;
        let delta = rng().random_range(-j..=j);
        Duration::from_secs((base + delta).max(1) as u64)
    }

    fn fill_screen(&self, ctx: &egui::Context) {
        if let Some(screen) = ctx.input(|i| i.viewport().monitor_size) {
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(0.0, 0.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                screen.x, screen.y,
            )));
        }
    }

    // Forward tray-menu events to the UI thread and wake it. The UI loop sleeps
    // through idle (see repaint scheduling), so it can't poll the menu itself;
    // this thread blocks on the menu channel and pings the UI when a click
    // arrives, which then handles quit/pause/next with full access to state.
    fn spawn_menu_watcher(ctx: &egui::Context, tx: Sender<muda::MenuId>) {
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let rx = MenuEvent::receiver();
            while let Ok(event) = rx.recv() {
                if tx.send(event.id().clone()).is_err() {
                    break;
                }
                ctx.request_repaint();
            }
        });
    }
}

impl eframe::App for App {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }

    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        // eframe 0.34 hands us a root Ui with no margin or background, which is
        // exactly what this absolutely-positioned overlay wants. Grab a Context
        // handle for viewport commands, repaint scheduling, and threads.
        let ctx = ui.ctx().clone();

        if self.bench {
            self.frames.fetch_add(1, Ordering::Relaxed);
        }

        if !self.started {
            self.started = true;
            self.fill_screen(&ctx);
            if let Some(tx) = self.menu_tx.take() {
                Self::spawn_menu_watcher(&ctx, tx);
            }
            self.advance();

            if self.bench {
                // Pin the card in its fully-settled, static state so the whole
                // window measures idle cost, then close after BENCH_SECS.
                let has_example = self
                    .deck
                    .current()
                    .is_some_and(|word| !word.example.trim().is_empty());
                let settled = timing::anim_end(&self.cfg, has_example) + 1.0;
                self.clock
                    .pin_elapsed(Instant::now(), Duration::from_secs_f32(settled.max(0.0)));
                let frames = self.frames.clone();
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_secs(BENCH_SECS));
                    let n = frames.load(Ordering::Relaxed);
                    eprintln!("BENCH frames={n} fps={:.2}", n as f64 / BENCH_SECS as f64);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    ctx.request_repaint();
                });
            }
        }

        while let Ok(id) = self.menu_rx.try_recv() {
            if id == self.menu_ids.quit {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            } else if id == self.menu_ids.pause {
                self.clock.toggle_pause(Instant::now());
            } else if id == self.menu_ids.next {
                self.advance();
            }
        }

        let now = Instant::now();
        if self.clock.is_due(now) {
            self.advance();
        }

        let now = Instant::now();
        let elapsed = self.clock.elapsed(now).as_secs_f32();

        // Seconds until the next word, and the whole-card opacity for the exit
        // fade as the swap approaches (1.0 = fully shown, no fade by default).
        let exit_window = timing::exit_window(&self.cfg, self.clock.word_interval());
        let until_next = self.clock.until_next(now);
        let paused = self.clock.is_paused();
        let exit_alpha =
            timing::exit_alpha(until_next.as_secs_f32(), exit_window.as_secs_f32(), paused);
        let accent = self
            .cfg
            .appearance
            .accent_color
            .map(|[r, g, b]| egui::Color32::from_rgb(r, g, b));

        // Read-only borrow of the deck for rendering; defer the prev_width
        // write until that borrow ends.
        let mut new_prev_width = None;
        let mut has_example = false;
        let translation_delay = timing::effective_translation_delay(&self.cfg);
        let example_delay = timing::example_delay(&self.cfg);
        if let Some(w) = self.deck.current() {
            has_example = !w.example.trim().is_empty();
            let view = CardView {
                content: CardContent {
                    word: &w.word,
                    transcription: &w.transcription,
                    translation: &w.translation,
                    example: &w.example,
                },
                timeline: CardTimeline {
                    elapsed,
                    transcription_delay: self.cfg.timing.transcription_delay,
                    translation_delay,
                    example_delay,
                    fade_duration: self.cfg.timing.fade_duration,
                    reduce_motion: self.cfg.accessibility.reduce_motion,
                },
                style: CardStyle {
                    corner: self.cfg.appearance.corner,
                    card_opacity: self.cfg.appearance.card_opacity,
                    corner_radius: self.cfg.appearance.corner_radius,
                    exit_alpha,
                    settle_px: if self.cfg.accessibility.reduce_motion {
                        0.0
                    } else {
                        self.cfg.appearance.settle_px
                    },
                    accent,
                    sheen: self.cfg.appearance.sheen,
                    theme: self.theme,
                },
                prev_width: self.prev_width,
            };
            let widget_w = view.compute_width(ui);
            view.paint(ui, widget_w);
            new_prev_width = Some(widget_w);
        }
        if let Some(w) = new_prev_width {
            self.prev_width = w;
        }

        // Drive repaints by state (the zero-idle invariant: a settled card costs
        // no frames). The decision is a pure function in `timing` so it can be
        // tested without egui; here we just apply it.
        let anim_end = timing::anim_end(&self.cfg, has_example);
        ctx.request_repaint_after(timing::repaint_after(
            elapsed,
            anim_end,
            paused,
            until_next,
            exit_window,
            ANIM_FRAME,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Word;
    use crate::platform::NullSpeaker;
    use crate::selector::FrequencyWeighted;
    use std::sync::Mutex;

    fn test_app(n: usize, cfg: Config) -> App {
        test_app_with_speaker(n, cfg, Box::new(NullSpeaker))
    }

    fn test_app_with_speaker(n: usize, cfg: Config, speaker: Box<dyn Speaker>) -> App {
        let words = (0..n)
            .map(|i| Word {
                word: format!("w{i}"),
                transcription: String::new(),
                translation: String::new(),
                frequency: i as i32 + 1,
                example: String::new(),
            })
            .collect();
        let deck = Deck::new(words, Box::new(FrequencyWeighted));
        let ids = MenuIds {
            next: muda::MenuId::from("next"),
            pause: muda::MenuId::from("pause"),
            quit: muda::MenuId::from("quit"),
        };
        App::new(deck, ids, cfg, speaker)
    }

    // A test double that records the words it is asked to speak, so we can assert
    // the speak-on-advance behaviour without invoking a real TTS process.
    struct RecordingSpeaker(Arc<Mutex<Vec<String>>>);

    impl Speaker for RecordingSpeaker {
        fn speak(&self, word: &str) {
            self.0.lock().unwrap().push(word.to_string());
        }
    }

    #[test]
    fn advance_routes_each_word_to_the_speaker_port() {
        // App always speaks through its injected port (the audible/silent choice
        // is the composition root's, via System vs Null speaker). Here a recording
        // double proves advance() hands the current word to the speaker, and that
        // each advance speaks exactly once, without any real TTS process.
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut app = test_app_with_speaker(
            5,
            Config::default(),
            Box::new(RecordingSpeaker(log.clone())),
        );
        app.advance();
        assert_eq!(log.lock().unwrap().len(), 1, "one word spoken per advance");
        app.advance();
        assert_eq!(log.lock().unwrap().len(), 2);
        // The recorded words are the deck's current words (non-empty).
        assert!(log.lock().unwrap().iter().all(|w| !w.is_empty()));
    }

    #[test]
    fn roll_interval_no_jitter_is_exact() {
        let mut cfg = Config::default();
        cfg.timing.interval_secs = 30;
        cfg.timing.jitter_secs = 0;
        let app = test_app(5, cfg);
        assert_eq!(app.roll_interval(1), Duration::from_secs(30));
    }

    #[test]
    fn roll_interval_with_jitter_stays_in_range() {
        let mut cfg = Config::default();
        cfg.timing.interval_secs = 30;
        cfg.timing.jitter_secs = 5;
        let app = test_app(5, cfg);
        for _ in 0..1000 {
            let s = app.roll_interval(1).as_secs();
            assert!((25..=35).contains(&s), "interval {s} out of [25,35]");
        }
    }

    #[test]
    fn roll_interval_never_below_one_second() {
        let mut cfg = Config::default();
        cfg.timing.interval_secs = 2;
        cfg.timing.jitter_secs = 10;
        let app = test_app(5, cfg);
        for _ in 0..1000 {
            assert!(app.roll_interval(1) >= Duration::from_secs(1));
        }
    }
}
