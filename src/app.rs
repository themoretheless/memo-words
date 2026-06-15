use crate::config::Config;
use crate::db::Word;
use crate::deck::Deck;
use crate::ui::{self, CardView};
use eframe::egui;
use muda::MenuEvent;
use rand::RngExt;
use rand::rng;
use std::collections::BTreeMap;
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
    pub speak: muda::MenuId,
    pub copy: muda::MenuId,
    pub copy_summary: muda::MenuId,
    pub copy_anki: muda::MenuId,
    pub copy_health: muda::MenuId,
    pub toggle_recall: muda::MenuId,
    pub toggle_pace: muda::MenuId,
    pub pause: muda::MenuId,
    pub quit: muda::MenuId,
}

#[derive(Clone)]
pub struct MenuChecks {
    pub pause: muda::CheckMenuItem,
    pub recall: muda::CheckMenuItem,
    pub pace: muda::CheckMenuItem,
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
    menu_checks: Option<MenuChecks>,
    menu_tx: Option<Sender<muda::MenuId>>,
    menu_rx: Receiver<muda::MenuId>,
    paused: bool,
    cfg: Config,
    word_interval: Duration,
    session_started: Instant,
    session_words: Vec<Word>,
    bench: bool,
    frames: Arc<AtomicUsize>,
    active_repaints: Arc<AtomicUsize>,
}

impl App {
    pub fn new(
        deck: Deck,
        menu_ids: MenuIds,
        menu_checks: Option<MenuChecks>,
        cfg: Config,
    ) -> Self {
        let (menu_tx, menu_rx) = std::sync::mpsc::channel();
        Self {
            deck,
            shown_at: None,
            last_show: Instant::now(),
            prev_width: ui::MIN_WIDTH,
            started: false,
            menu_ids,
            menu_checks,
            menu_tx: Some(menu_tx),
            menu_rx,
            paused: false,
            cfg,
            word_interval: Duration::from_secs(cfg.interval_secs),
            session_started: Instant::now(),
            session_words: Vec::new(),
            bench: std::env::var("MEMO_BENCH").is_ok(),
            frames: Arc::new(AtomicUsize::new(0)),
            active_repaints: Arc::new(AtomicUsize::new(0)),
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

        if let Some(w) = self.deck.current() {
            self.session_words.push(w.clone());
        }
    }

    fn speak_current_word(&self) {
        if let Some(w) = self.deck.current() {
            speak_word(&w.word);
        }
    }

    fn copy_current_word(&self, ctx: &egui::Context) {
        if let Some(w) = self.deck.current() {
            ctx.copy_text(format_word_clipboard_text(w));
        }
    }

    fn copy_session_summary(&self, ctx: &egui::Context) {
        ctx.copy_text(format_session_summary(
            &self.session_words,
            self.session_started.elapsed(),
        ));
    }

    fn copy_session_as_anki(&self, ctx: &egui::Context) {
        ctx.copy_text(format_session_anki_tsv(&self.session_words));
    }

    fn copy_deck_health_report(&self, ctx: &egui::Context) {
        ctx.copy_text(format_deck_health_report(self.deck.words()));
    }

    fn restart_current_word_timing(&mut self) {
        let now = Instant::now();
        self.shown_at = Some(now);
        self.last_show = now;
        self.word_interval = self.roll_interval();
    }

    fn toggle_recall_mode(&mut self) {
        self.cfg.recall_mode = !self.cfg.recall_mode;
        if let Some(checks) = &self.menu_checks {
            checks.recall.set_checked(self.cfg.recall_mode);
        }
        // Recall changes reveal timing, so restart the current card's reveal
        // sequence instead of making visible lines pop in/out mid-word.
        self.restart_current_word_timing();
    }

    fn toggle_pace_indicator(&mut self) {
        self.cfg.pace_indicator = !self.cfg.pace_indicator;
        if let Some(checks) = &self.menu_checks {
            checks.pace.set_checked(self.cfg.pace_indicator);
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
            if !self.bench
                && let Some(tx) = self.menu_tx.take()
            {
                Self::spawn_menu_watcher(&ctx, tx);
            }
            self.advance();

            if self.bench {
                // Pin the card in its fully-settled, static state so the whole
                // window measures idle cost, then close after BENCH_SECS.
                self.shown_at = Some(Instant::now() - Duration::from_secs(20));
                self.last_show = Instant::now();
                let frames = self.frames.clone();
                let active_repaints = self.active_repaints.clone();
                let ctx = ctx.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_secs(BENCH_SECS));
                    let n = frames.load(Ordering::Relaxed);
                    let active = active_repaints.load(Ordering::Relaxed);
                    eprintln!(
                        "BENCH frames={n} fps={} active_repaints={active}",
                        n as u64 / BENCH_SECS
                    );
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
                if let Some(checks) = &self.menu_checks {
                    checks.pause.set_checked(self.paused);
                }
                // Reset the timer so resuming gives a full interval rather
                // than instantly advancing on leftover elapsed time.
                self.last_show = Instant::now();
            } else if id == self.menu_ids.next {
                self.advance();
            } else if id == self.menu_ids.speak {
                self.speak_current_word();
            } else if id == self.menu_ids.copy {
                self.copy_current_word(&ctx);
            } else if id == self.menu_ids.copy_summary {
                self.copy_session_summary(&ctx);
            } else if id == self.menu_ids.copy_anki {
                self.copy_session_as_anki(&ctx);
            } else if id == self.menu_ids.copy_health {
                self.copy_deck_health_report(&ctx);
            } else if id == self.menu_ids.toggle_recall {
                self.toggle_recall_mode();
            } else if id == self.menu_ids.toggle_pace {
                self.toggle_pace_indicator();
            }
        }

        if !self.paused && self.last_show.elapsed() >= self.word_interval {
            self.advance();
        }

        let elapsed = self
            .shown_at
            .map(|t| t.elapsed().as_secs_f32())
            .unwrap_or(0.0);

        // Read-only borrow of the deck for rendering; defer the prev_width
        // write until that borrow ends.
        let mut new_prev_width = None;
        let mut has_example = false;
        let translation_delay = self.effective_translation_delay();
        let example_delay = self.example_delay();
        let interval_secs = self.word_interval.as_secs_f32();
        let interval_progress = if interval_secs > 0.0 {
            self.last_show.elapsed().as_secs_f32() / interval_secs
        } else {
            1.0
        };
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
                interval_progress,
                show_pace_indicator: self.cfg.pace_indicator,
                accent_color: self.cfg.accent_color,
                session_count: self.session_words.len(),
                show_session_counter: self.cfg.session_counter,
            };
            let widget_w = view.compute_width(ui);
            view.paint(ui, widget_w);
            new_prev_width = Some(widget_w);
        }
        if let Some(w) = new_prev_width {
            self.prev_width = w;
        }

        // Drive repaints by state: animate at ~60 fps while the card fades in,
        // sleep long while paused (a menu event wakes us), otherwise sleep until
        // the next word is due. A static card costs no frames.
        let anim_end = self.anim_end(has_example);
        if elapsed < anim_end {
            if self.bench {
                self.active_repaints.fetch_add(1, Ordering::Relaxed);
            }
            ctx.request_repaint_after(ANIM_FRAME);
        } else if self.paused {
            ctx.request_repaint_after(Duration::from_secs(3600));
        } else {
            let until_next = self.word_interval.saturating_sub(self.last_show.elapsed());
            if self.cfg.pace_indicator && until_next > Duration::ZERO {
                if self.bench {
                    self.active_repaints.fetch_add(1, Ordering::Relaxed);
                }
                ctx.request_repaint_after(until_next.min(Duration::from_secs(1)));
            } else {
                ctx.request_repaint_after(until_next);
            }
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

fn format_word_clipboard_text(w: &Word) -> String {
    let mut lines = vec![
        w.word.clone(),
        w.transcription.clone(),
        w.translation.clone(),
    ];
    if !w.example.trim().is_empty() {
        lines.push(w.example.trim().to_string());
    }
    lines.join("\n")
}

fn format_session_summary(words: &[Word], elapsed: Duration) -> String {
    let mut out = String::from("# Memo Words session\n\n");
    out.push_str(&format!("Duration: {}\n", format_duration(elapsed)));
    out.push_str(&format!("Words shown: {}\n\n", words.len()));

    if words.is_empty() {
        out.push_str("_No words shown yet._\n");
        return out;
    }

    for (i, w) in words.iter().enumerate() {
        out.push_str(&format!(
            "{}. **{}** - {} - {}\n",
            i + 1,
            w.word,
            w.transcription,
            w.translation
        ));
        if !w.example.trim().is_empty() {
            out.push_str(&format!("   Example: {}\n", w.example.trim()));
        }
    }
    out
}

fn format_session_anki_tsv(words: &[Word]) -> String {
    let mut out = String::from("word\ttranslation\ttranscription\texample\n");
    for w in words {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\n",
            tsv_cell(&w.word),
            tsv_cell(&w.translation),
            tsv_cell(&w.transcription),
            tsv_cell(w.example.trim())
        ));
    }
    out
}

fn tsv_cell(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace('\t', " ")
}

fn format_deck_health_report(words: &[Word]) -> String {
    let mut missing_transcription = 0;
    let mut missing_translation = 0;
    let mut missing_example = 0;
    let mut invalid_frequency = 0;
    let mut counts = BTreeMap::<String, usize>::new();

    for w in words {
        if w.transcription.trim().is_empty() {
            missing_transcription += 1;
        }
        if w.translation.trim().is_empty() {
            missing_translation += 1;
        }
        if w.example.trim().is_empty() {
            missing_example += 1;
        }
        if w.frequency <= 0 {
            invalid_frequency += 1;
        }

        let key = w.word.trim().to_ascii_lowercase();
        if !key.is_empty() {
            *counts.entry(key).or_default() += 1;
        }
    }

    let duplicates: Vec<_> = counts
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|(word, count)| (word.as_str(), *count))
        .collect();

    let mut out = String::from("# Memo Words deck health\n\n");
    out.push_str(&format!("Words: {}\n", words.len()));
    out.push_str(&format!("Missing transcription: {missing_transcription}\n"));
    out.push_str(&format!("Missing translation: {missing_translation}\n"));
    out.push_str(&format!("Missing example: {missing_example}\n"));
    out.push_str(&format!("Invalid frequency: {invalid_frequency}\n"));
    out.push_str(&format!("Duplicate words: {}\n", duplicates.len()));

    if !duplicates.is_empty() {
        out.push_str("\nDuplicates:\n");
        for (word, count) in duplicates {
            out.push_str(&format!("- {word} x{count}\n"));
        }
    }

    out
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

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
            speak: muda::MenuId::from("speak"),
            copy: muda::MenuId::from("copy"),
            copy_summary: muda::MenuId::from("copy_summary"),
            copy_anki: muda::MenuId::from("copy_anki"),
            copy_health: muda::MenuId::from("copy_health"),
            toggle_recall: muda::MenuId::from("toggle_recall"),
            toggle_pace: muda::MenuId::from("toggle_pace"),
            pause: muda::MenuId::from("pause"),
            quit: muda::MenuId::from("quit"),
        };
        let checks = MenuChecks {
            pause: muda::CheckMenuItem::with_id("pause", "Paused", true, false, None),
            recall: muda::CheckMenuItem::with_id(
                "toggle_recall",
                "Recall mode",
                true,
                cfg.recall_mode,
                None,
            ),
            pace: muda::CheckMenuItem::with_id(
                "toggle_pace",
                "Pace indicator",
                true,
                cfg.pace_indicator,
                None,
            ),
        };
        App::new(deck, ids, Some(checks), cfg)
    }

    #[test]
    fn toggle_recall_mode_flips_config_and_restarts_timing() {
        let mut app = test_app(5, Config::default());
        app.advance();
        app.shown_at = Some(Instant::now() - Duration::from_secs(20));
        app.last_show = Instant::now() - Duration::from_secs(20);

        app.toggle_recall_mode();

        assert!(app.cfg.recall_mode);
        assert!(app.menu_checks.as_ref().unwrap().recall.is_checked());
        assert!(app.shown_at.unwrap().elapsed() < Duration::from_secs(1));
        assert!(app.last_show.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn toggle_pace_indicator_flips_config() {
        let mut app = test_app(5, Config::default());
        assert!(!app.cfg.pace_indicator);
        app.toggle_pace_indicator();
        assert!(app.cfg.pace_indicator);
        assert!(app.menu_checks.as_ref().unwrap().pace.is_checked());
        app.toggle_pace_indicator();
        assert!(!app.cfg.pace_indicator);
        assert!(!app.menu_checks.as_ref().unwrap().pace.is_checked());
    }

    #[test]
    fn advance_records_words_in_session_history() {
        let mut app = test_app(5, Config::default());
        assert!(app.session_words.is_empty());
        app.advance();
        assert_eq!(app.session_words.len(), 1);
        assert_eq!(app.session_words[0].word, app.deck.current().unwrap().word);
    }

    #[test]
    fn format_word_clipboard_text_includes_example_when_present() {
        let word = Word {
            word: "time".into(),
            transcription: "/taɪm/".into(),
            translation: "time".into(),
            frequency: 70,
            example: "What time is it?".into(),
        };

        assert_eq!(
            format_word_clipboard_text(&word),
            "time\n/taɪm/\ntime\nWhat time is it?"
        );
    }

    #[test]
    fn format_session_summary_lists_words_as_markdown() {
        let words = vec![Word {
            word: "time".into(),
            transcription: "/taɪm/".into(),
            translation: "time".into(),
            frequency: 70,
            example: "What time is it?".into(),
        }];

        let summary = format_session_summary(&words, Duration::from_secs(65));

        assert!(summary.contains("# Memo Words session"));
        assert!(summary.contains("Duration: 1m 5s"));
        assert!(summary.contains("Words shown: 1"));
        assert!(summary.contains("1. **time** - /taɪm/ - time"));
        assert!(summary.contains("Example: What time is it?"));
    }

    #[test]
    fn format_session_anki_tsv_uses_flat_tab_separated_rows() {
        let words = vec![Word {
            word: "look up".into(),
            transcription: "/lʊk ʌp/".into(),
            translation: "search".into(),
            frequency: 100,
            example: "Look up the word.\nThen write it down.".into(),
        }];

        let tsv = format_session_anki_tsv(&words);

        assert_eq!(
            tsv,
            "word\ttranslation\ttranscription\texample\nlook up\tsearch\t/lʊk ʌp/\tLook up the word. Then write it down.\n"
        );
    }

    #[test]
    fn format_deck_health_report_counts_missing_fields_and_duplicates() {
        let words = vec![
            Word {
                word: "Time".into(),
                transcription: "/taɪm/".into(),
                translation: "time".into(),
                frequency: 70,
                example: "What time is it?".into(),
            },
            Word {
                word: "time".into(),
                transcription: String::new(),
                translation: String::new(),
                frequency: 0,
                example: String::new(),
            },
        ];

        let report = format_deck_health_report(&words);

        assert!(report.contains("Words: 2"));
        assert!(report.contains("Missing transcription: 1"));
        assert!(report.contains("Missing translation: 1"));
        assert!(report.contains("Missing example: 1"));
        assert!(report.contains("Invalid frequency: 1"));
        assert!(report.contains("Duplicate words: 1"));
        assert!(report.contains("- time x2"));
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
}
