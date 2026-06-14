use crate::config::Config;
use crate::deck::Deck;
use crate::ui::{self, CardView};
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

// In recall mode the translation is held back to this fraction of the interval
// (capped below by the configured translation_delay) so there's a real window
// to recall the meaning before it's revealed. 0.55 lands the reveal just past
// the midpoint, leaving roughly the same span again to absorb the answer.
const RECALL_REVEAL_FRACTION: f32 = 0.55;

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
    shown_at: Option<Instant>,
    last_show: Instant,
    prev_width: f32,
    started: bool,
    menu_ids: MenuIds,
    menu_tx: Option<Sender<muda::MenuId>>,
    menu_rx: Receiver<muda::MenuId>,
    paused: bool,
    cfg: Config,
    word_interval: Duration,
    bench: bool,
    frames: Arc<AtomicUsize>,
}

impl App {
    pub fn new(deck: Deck, menu_ids: MenuIds, cfg: Config) -> Self {
        let (menu_tx, menu_rx) = std::sync::mpsc::channel();
        Self {
            deck,
            shown_at: None,
            last_show: Instant::now(),
            prev_width: ui::MIN_WIDTH,
            started: false,
            menu_ids,
            menu_tx: Some(menu_tx),
            menu_rx,
            paused: false,
            cfg,
            word_interval: Duration::from_secs(cfg.interval_secs),
            bench: std::env::var("MEMO_BENCH").is_ok(),
            frames: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Show the next word: advance the deck, reset the timers, roll a fresh
    /// interval, and speak it if configured.
    fn advance(&mut self) {
        self.deck.advance();
        self.shown_at = Some(Instant::now());
        self.last_show = Instant::now();
        self.word_interval = self.roll_interval();

        if self.cfg.speak
            && let Some(w) = self.deck.current()
        {
            speak_word(&w.word);
        }
    }

    // When the translation line starts fading in. Normally the configured
    // translation_delay; in recall mode it's pushed back to RECALL_REVEAL_FRACTION
    // of the interval so the meaning stays hidden long enough to recall it first.
    // Capped below by translation_delay, so enabling recall mode only ever delays
    // the reveal, never pulls it earlier than the configured value.
    fn effective_translation_delay(&self) -> f32 {
        if self.cfg.recall_mode {
            let late = self.cfg.interval_secs as f32 * RECALL_REVEAL_FRACTION;
            late.max(self.cfg.translation_delay)
        } else {
            self.cfg.translation_delay
        }
    }

    // When the example line starts fading in: just after the translation has
    // settled, so the lines reveal in sequence (word, transcription,
    // translation, example). Follows the effective translation delay, so recall
    // mode pushes the example back in lockstep with the meaning.
    fn example_delay(&self) -> f32 {
        self.effective_translation_delay() + self.cfg.fade_duration
    }

    // Elapsed time (seconds since the word appeared) at which the card is fully
    // settled and repaints can stop. The lines fade in at independent delays,
    // so the card isn't done until the LAST one finishes. Using only
    // translation_delay here meant a transcription_delay past the translation
    // fade window stopped repaints before the transcription ever rendered,
    // hiding it entirely. The example delay only counts when the current word
    // actually has one, so example-less words don't repaint longer for nothing.
    fn anim_end(&self, has_example: bool) -> f32 {
        let mut last = self
            .cfg
            .transcription_delay
            .max(self.effective_translation_delay());
        if has_example {
            last = last.max(self.example_delay());
        }
        last + self.cfg.fade_duration
    }

    // How long the card spends fading out before the next word. Zero (the
    // default) means a hard cut. Capped at half the current interval so the fade
    // never eats the whole word, however large exit_duration is configured.
    fn exit_window(&self) -> Duration {
        if self.cfg.exit_duration <= 0.0 {
            return Duration::ZERO;
        }
        let cap = self.word_interval.as_secs_f32() * 0.5;
        Duration::from_secs_f32(self.cfg.exit_duration.min(cap).max(0.0))
    }

    // Time the current word stays up: base interval optionally jittered by
    // +/- jitter_secs so the cadence doesn't feel metronomic. Clamped to >=1s.
    fn roll_interval(&self) -> Duration {
        let base = self.cfg.interval_secs as i64;
        if self.cfg.jitter_secs == 0 {
            return Duration::from_secs(base.max(1) as u64);
        }
        let j = self.cfg.jitter_secs as i64;
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
                self.shown_at = Some(Instant::now() - Duration::from_secs(20));
                self.last_show = Instant::now();
                let frames = self.frames.clone();
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_secs(BENCH_SECS));
                    let n = frames.load(Ordering::Relaxed);
                    eprintln!("BENCH frames={n} fps={}", n as u64 / BENCH_SECS);
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
                self.paused = !self.paused;
                // Reset the timer so resuming gives a full interval rather
                // than instantly advancing on leftover elapsed time.
                self.last_show = Instant::now();
            } else if id == self.menu_ids.next {
                self.advance();
            }
        }

        if !self.paused && self.last_show.elapsed() >= self.word_interval {
            self.advance();
        }

        let elapsed = self
            .shown_at
            .map(|t| t.elapsed().as_secs_f32())
            .unwrap_or(0.0);

        // Seconds until the next word, and the whole-card opacity for the exit
        // fade as the swap approaches (1.0 = fully shown, no fade by default).
        let exit_window = self.exit_window();
        let until_next = self.word_interval.saturating_sub(self.last_show.elapsed());
        let exit_alpha = crate::ui::exit_alpha(until_next.as_secs_f32(), exit_window.as_secs_f32());

        // Read-only borrow of the deck for rendering; defer the prev_width
        // write until that borrow ends.
        let mut new_prev_width = None;
        let mut has_example = false;
        let translation_delay = self.effective_translation_delay();
        let example_delay = self.example_delay();
        if let Some(w) = self.deck.current() {
            has_example = !w.example.trim().is_empty();
            let view = CardView {
                word: &w.word,
                transcription: &w.transcription,
                translation: &w.translation,
                example: &w.example,
                elapsed,
                prev_width: self.prev_width,
                transcription_delay: self.cfg.transcription_delay,
                translation_delay,
                example_delay,
                fade_duration: self.cfg.fade_duration,
                corner: self.cfg.corner,
                card_opacity: self.cfg.card_opacity,
                corner_radius: self.cfg.corner_radius,
                exit_alpha,
            };
            let widget_w = view.compute_width(ui);
            view.paint(ui, widget_w);
            new_prev_width = Some(widget_w);
        }
        if let Some(w) = new_prev_width {
            self.prev_width = w;
        }

        // Drive repaints by state: animate at ~60 fps while the card fades in or
        // out, sleep long while paused (a menu event wakes us), otherwise sleep
        // until the exit fade should begin. A static card costs no frames.
        let anim_end = self.anim_end(has_example);
        if elapsed < anim_end {
            ctx.request_repaint_after(ANIM_FRAME);
        } else if self.paused {
            ctx.request_repaint_after(Duration::from_secs(3600));
        } else if until_next <= exit_window {
            // Inside the exit window: animate the fade-out until the swap.
            ctx.request_repaint_after(ANIM_FRAME);
        } else {
            // Sleep until the exit fade should start (or the swap, if disabled).
            ctx.request_repaint_after(until_next.saturating_sub(exit_window));
        }
    }
}

// Pronounce the word out loud. Fire-and-forget so the UI thread never blocks
// on the TTS process. macOS only (uses `say`); a no-op elsewhere.
#[cfg(target_os = "macos")]
fn speak_word(word: &str) {
    // Run on a detached thread that waits on the child, so finished `say`
    // processes are reaped instead of piling up as zombies over a session.
    let word = word.to_string();
    std::thread::spawn(move || {
        let _ = std::process::Command::new("say").arg(word).status();
    });
}

#[cfg(not(target_os = "macos"))]
fn speak_word(_word: &str) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Word;
    use crate::selector::FrequencyWeighted;

    fn test_app(n: usize, cfg: Config) -> App {
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
        App::new(deck, ids, cfg)
    }

    #[test]
    fn roll_interval_no_jitter_is_exact() {
        let cfg = Config {
            interval_secs: 30,
            jitter_secs: 0,
            ..Config::default()
        };
        let app = test_app(5, cfg);
        assert_eq!(app.roll_interval(), Duration::from_secs(30));
    }

    #[test]
    fn roll_interval_with_jitter_stays_in_range() {
        let cfg = Config {
            interval_secs: 30,
            jitter_secs: 5,
            ..Config::default()
        };
        let app = test_app(5, cfg);
        for _ in 0..1000 {
            let s = app.roll_interval().as_secs();
            assert!((25..=35).contains(&s), "interval {s} out of [25,35]");
        }
    }

    #[test]
    fn roll_interval_never_below_one_second() {
        let cfg = Config {
            interval_secs: 2,
            jitter_secs: 10,
            ..Config::default()
        };
        let app = test_app(5, cfg);
        for _ in 0..1000 {
            assert!(app.roll_interval() >= Duration::from_secs(1));
        }
    }

    #[test]
    fn anim_end_uses_the_later_fade() {
        let cfg = Config {
            transcription_delay: 5.0,
            translation_delay: 10.0,
            fade_duration: 1.0,
            ..Config::default()
        };
        let app = test_app(5, cfg);
        assert_eq!(app.anim_end(false), 11.0);
    }

    #[test]
    fn anim_end_covers_a_late_transcription_fade() {
        // transcription_delay past the translation fade window must still be
        // covered, otherwise the transcription line never gets painted.
        let cfg = Config {
            transcription_delay: 15.0,
            translation_delay: 10.0,
            fade_duration: 1.0,
            ..Config::default()
        };
        let app = test_app(5, cfg);
        assert_eq!(app.anim_end(false), 16.0);
    }

    #[test]
    fn anim_end_extends_for_the_example_line() {
        // With an example present, repaints must run until its fade (which
        // starts after the translation settles) finishes.
        let cfg = Config {
            transcription_delay: 5.0,
            translation_delay: 10.0,
            fade_duration: 1.0,
            ..Config::default()
        };
        let app = test_app(5, cfg);
        // example_delay = 10 + 1 = 11; end = 11 + 1 = 12.
        assert_eq!(app.anim_end(true), 12.0);
        // Without an example the end is unchanged.
        assert_eq!(app.anim_end(false), 11.0);
    }

    #[test]
    fn effective_translation_delay_unchanged_without_recall() {
        let cfg = Config {
            translation_delay: 10.0,
            interval_secs: 30,
            recall_mode: false,
            ..Config::default()
        };
        let app = test_app(5, cfg);
        assert_eq!(app.effective_translation_delay(), 10.0);
    }

    #[test]
    fn recall_mode_delays_translation_to_late_in_interval() {
        let cfg = Config {
            translation_delay: 10.0,
            interval_secs: 30,
            recall_mode: true,
            ..Config::default()
        };
        let app = test_app(5, cfg);
        // 30 * 0.55 = 16.5, later than the 10s default, so the late reveal wins.
        assert_eq!(app.effective_translation_delay(), 16.5);
    }

    #[test]
    fn recall_mode_never_earlier_than_configured_delay() {
        // Short interval: 8 * 0.55 = 4.4, earlier than the 10s translation_delay.
        // Recall mode must not pull the reveal in ahead of the configured value.
        let cfg = Config {
            translation_delay: 10.0,
            interval_secs: 8,
            recall_mode: true,
            ..Config::default()
        };
        let app = test_app(5, cfg);
        assert_eq!(app.effective_translation_delay(), 10.0);
    }

    #[test]
    fn recall_mode_pushes_example_and_anim_end() {
        let cfg = Config {
            transcription_delay: 5.0,
            translation_delay: 10.0,
            fade_duration: 1.0,
            interval_secs: 30,
            recall_mode: true,
            ..Config::default()
        };
        let app = test_app(5, cfg);
        // translation at 16.5, example at 17.5, repaints end at 18.5.
        assert_eq!(app.effective_translation_delay(), 16.5);
        assert_eq!(app.example_delay(), 17.5);
        assert_eq!(app.anim_end(true), 18.5);
    }

    #[test]
    fn exit_window_is_zero_when_disabled() {
        let cfg = Config {
            exit_duration: 0.0,
            interval_secs: 30,
            ..Config::default()
        };
        let app = test_app(5, cfg);
        assert_eq!(app.exit_window(), Duration::ZERO);
    }

    #[test]
    fn exit_window_used_as_is_when_it_fits() {
        // 0.4s fits well within half of a 30s interval, so it's used unchanged.
        let cfg = Config {
            exit_duration: 0.4,
            interval_secs: 30,
            jitter_secs: 0,
            ..Config::default()
        };
        let app = test_app(5, cfg);
        assert_eq!(app.exit_window(), Duration::from_secs_f32(0.4));
    }

    #[test]
    fn exit_window_capped_at_half_a_short_interval() {
        // A 5s exit on a 4s interval would eat the whole word; cap at half = 2s.
        let cfg = Config {
            exit_duration: 5.0,
            interval_secs: 4,
            jitter_secs: 0,
            ..Config::default()
        };
        let app = test_app(5, cfg);
        assert_eq!(app.exit_window(), Duration::from_secs_f32(2.0));
    }
}
