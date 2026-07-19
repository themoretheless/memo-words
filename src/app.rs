use crate::command::AppCommand;
use crate::config::Config;
use crate::deck::Deck;
use crate::diagnostics;
use crate::platform::Speaker;
use crate::session::SessionClock;
use crate::source::SourceController;
use crate::store::{ProgressState, ProgressStore};
use crate::theme::Theme;
use crate::timing;
use crate::tray::{TrayFeedback, TrayMenu, TrayState};
use crate::ui::{CardContent, CardStyle, CardTimeline, CardView};
use crate::wake::WakeScheduler;
use eframe::egui;
use rand::RngExt;
use rand::rng;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

// Repaint cadence while a card is animating in (word -> transcription ->
// translation fades). ~60 fps keeps the fades smooth.
const ANIM_FRAME: Duration = Duration::from_millis(16);

// Idle window measured by the frame-counter benchmark (MEMO_BENCH=1).
const BENCH_SECS: u64 = 10;
const BENCH_WARMUP_SECS: u64 = 2;
const STATUS_FEEDBACK: Duration = Duration::from_secs(2);

// Dirty progress is flushed to disk at most this often (plus always on quit
// and drop), coalescing exposure counters into one small write instead of one
// per word swap. Flushing happens inside frames the app is already painting,
// so it never wakes an idle overlay.
const PROGRESS_FLUSH_INTERVAL: Duration = Duration::from_secs(60);

/// The eframe adapter (a humble object): it owns timing and rendering, consumes
/// domain-neutral commands, and delegates word rotation to `Deck`. It
/// deliberately holds no native menu IDs, selection, or word-storage logic.
pub struct App {
    deck: Deck,
    clock: SessionClock,
    prev_width: f32,
    theme: Theme,
    wake_scheduler: Option<WakeScheduler>,
    started: bool,
    command_rx: Receiver<AppCommand>,
    source: Option<SourceController>,
    tray_menu: Option<TrayMenu>,
    pending_words: Option<Vec<crate::model::Word>>,
    status_feedback_until: Option<Instant>,
    cfg: Config,
    bench: bool,
    frames: Arc<AtomicUsize>,
    // The TTS port. `App` depends on the capability, not the OS mechanism, so the
    // composition root chooses the real speaker and tests inject a double.
    speaker: Box<dyn Speaker>,
    // The persistence port and the in-memory learning state it loads/saves.
    // Same inversion as the speaker: the composition root picks file vs no-op.
    store: Box<dyn ProgressStore>,
    progress: ProgressState,
    progress_dirty: bool,
    last_progress_flush: Instant,
}

impl App {
    pub fn new(
        deck: Deck,
        command_rx: Receiver<AppCommand>,
        cfg: Config,
        source: Option<SourceController>,
        tray_menu: Option<TrayMenu>,
        speaker: Box<dyn Speaker>,
        store: Box<dyn ProgressStore>,
    ) -> Self {
        let now = Instant::now();
        let theme = Theme::from_config(&cfg);
        let progress = store.load();
        let app = Self {
            deck,
            clock: SessionClock::new(now, Duration::from_secs(cfg.timing.interval_secs)),
            prev_width: theme.metrics.min_width,
            theme,
            wake_scheduler: None,
            started: false,
            command_rx,
            source,
            tray_menu,
            pending_words: None,
            status_feedback_until: None,
            cfg,
            bench: std::env::var("MEMO_BENCH").is_ok(),
            frames: Arc::new(AtomicUsize::new(0)),
            speaker,
            store,
            progress,
            progress_dirty: false,
            last_progress_flush: now,
        };
        app.sync_tray();
        app
    }

    /// Show the next word: advance the deck, reset the timers, roll a fresh
    /// interval, and speak it through the speaker port.
    fn advance(&mut self) {
        if let Some(scheduler) = &mut self.wake_scheduler {
            scheduler.cancel();
        }
        if let Some(words) = self.pending_words.take()
            && self.deck.replace_words(words)
        {
            self.prev_width = self.theme.metrics.min_width;
            if let Some(source) = &mut self.source {
                source.activate_pending();
            }
            self.sync_tray();
        }
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
            self.progress
                .record_exposure(&w.word, crate::store::unix_now());
            self.progress_dirty = true;
        }
    }

    /// Write dirty progress through the store port, unconditionally. Failures
    /// are logged, not fatal: a broken disk must never take down the overlay.
    fn flush_progress(&mut self) {
        if !self.progress_dirty {
            return;
        }
        if let Err(e) = self.store.save(&self.progress) {
            eprintln!("memo-words: could not save learning progress: {e}");
        } else {
            self.progress_dirty = false;
        }
        self.last_progress_flush = Instant::now();
    }

    /// Flush on the coalescing cadence. Called from frames the app is already
    /// painting, so it adds no wakeups to an idle overlay.
    fn maybe_flush_progress(&mut self, now: Instant) {
        if self.progress_dirty
            && now.saturating_duration_since(self.last_progress_flush) >= PROGRESS_FLUSH_INTERVAL
        {
            self.flush_progress();
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

    fn poll_source(&mut self) {
        let update = self.source.as_mut().and_then(SourceController::poll);
        let Some(update) = update else {
            return;
        };
        if let Some(words) = update.words {
            self.pending_words = Some(words);
        }
        if update.report_received {
            if let Some(report) = self
                .source
                .as_ref()
                .and_then(SourceController::latest_report)
            {
                eprintln!("memo-words: word source {report}");
                for issue in &report.issues {
                    eprintln!("memo-words: source {}: {}", issue.kind, issue.message);
                }
            }
        } else {
            eprintln!("memo-words: background word source disconnected without a report");
        }
        self.status_feedback_until = None;
        self.sync_tray();
    }

    fn reload_source(&mut self) {
        if self.source.as_mut().is_some_and(SourceController::reload) {
            self.status_feedback_until = None;
            self.sync_tray();
        }
    }

    fn copy_diagnostics(&mut self, ctx: &egui::Context) {
        ctx.copy_text(diagnostics::build(
            &self.cfg,
            self.source.as_ref(),
            &self.progress,
        ));
        self.status_feedback_until = Some(Instant::now() + STATUS_FEEDBACK);
        self.sync_tray();
    }

    fn restore_tray_status(&mut self, now: Instant) {
        if self
            .status_feedback_until
            .is_some_and(|deadline| now >= deadline)
        {
            self.status_feedback_until = None;
            self.sync_tray();
        }
    }

    fn sync_tray(&self) {
        let Some(menu) = &self.tray_menu else {
            return;
        };
        let feedback = self
            .status_feedback_until
            .map(|_| TrayFeedback::DiagnosticsCopied);
        if let Some(source) = &self.source {
            menu.sync(TrayState::runtime(
                self.clock.is_paused(),
                source.status(),
                source.can_reload(),
                source.retry_recommended(),
                feedback,
            ));
        } else {
            menu.sync(TrayState::benchmark(self.clock.is_paused(), feedback));
        }
    }

    fn handle_command(&mut self, command: AppCommand, ctx: &egui::Context) -> bool {
        match command {
            AppCommand::Quit => {
                // Flush before the close command; Drop is the backstop for
                // exits that never pass through here (window manager close).
                self.flush_progress();
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                false
            }
            AppCommand::TogglePause => {
                self.clock.toggle_pause(Instant::now());
                if let Some(scheduler) = &mut self.wake_scheduler {
                    scheduler.cancel();
                }
                self.sync_tray();
                true
            }
            AppCommand::NextWord => {
                self.advance();
                true
            }
            AppCommand::ReloadSource => {
                self.reload_source();
                true
            }
            AppCommand::CopyDiagnostics => {
                self.copy_diagnostics(ctx);
                true
            }
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // eframe drops the App on every orderly shutdown path, so this is the
        // last-chance flush for exits that skip the Quit command.
        self.flush_progress();
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
        if self.wake_scheduler.is_none() {
            self.wake_scheduler = Some(WakeScheduler::new(&ctx));
        }
        self.restore_tray_status(Instant::now());

        if self.bench {
            self.frames.fetch_add(1, Ordering::Relaxed);
        }

        if !self.started {
            self.started = true;
            self.fill_screen(&ctx);
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
                    // Exclude window creation, font upload, and initial viewport
                    // settling from the idle measurement.
                    std::thread::sleep(Duration::from_secs(BENCH_WARMUP_SECS));
                    frames.store(0, Ordering::Relaxed);
                    std::thread::sleep(Duration::from_secs(BENCH_SECS));
                    let n = frames.load(Ordering::Relaxed);
                    eprintln!("BENCH frames={n} fps={:.2}", n as f64 / BENCH_SECS as f64);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    ctx.request_repaint();
                });
            }
        }

        // Poll after the initial advance so even an immediately available
        // remote result cannot replace the fallback before its first frame.
        // Loaded words are queued until the next normal advance.
        self.poll_source();

        while let Ok(command) = self.command_rx.try_recv() {
            if !self.handle_command(command, &ctx) {
                return;
            }
        }

        let now = Instant::now();
        if self.clock.is_due(now) {
            self.advance();
        }
        self.maybe_flush_progress(now);

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
        let mut repaint_after = timing::repaint_after(
            elapsed,
            anim_end,
            paused,
            until_next,
            exit_window,
            ANIM_FRAME,
        );
        if let Some(deadline) = self.status_feedback_until {
            repaint_after = repaint_after.min(deadline.saturating_duration_since(now));
        }
        if repaint_after <= ANIM_FRAME {
            if let Some(scheduler) = &mut self.wake_scheduler {
                scheduler.cancel();
            }
            ctx.request_repaint_after(repaint_after);
        } else if let Some(scheduler) = &mut self.wake_scheduler {
            scheduler.schedule(now, repaint_after, ANIM_FRAME.saturating_mul(2));
        }
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
        test_app_with_ports(n, cfg, speaker, Box::new(crate::store::NullProgressStore))
    }

    fn test_app_with_ports(
        n: usize,
        cfg: Config,
        speaker: Box<dyn Speaker>,
        store: Box<dyn ProgressStore>,
    ) -> App {
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
        let (_command_tx, command_rx) = std::sync::mpsc::channel();
        App::new(deck, command_rx, cfg, None, None, speaker, store)
    }

    // A test double that records the words it is asked to speak, so we can assert
    // the speak-on-advance behaviour without invoking a real TTS process.
    struct RecordingSpeaker(Arc<Mutex<Vec<String>>>);

    impl Speaker for RecordingSpeaker {
        fn speak(&self, word: &str) {
            self.0.lock().unwrap().push(word.to_string());
        }
    }

    // A store double that counts saves and keeps the last saved state, so tests
    // can assert flush behaviour without touching the filesystem.
    #[derive(Clone, Default)]
    struct RecordingStore {
        saves: Arc<Mutex<Vec<ProgressState>>>,
    }

    impl ProgressStore for RecordingStore {
        fn load(&self) -> ProgressState {
            ProgressState::default()
        }

        fn save(&self, state: &ProgressState) -> std::io::Result<()> {
            self.saves.lock().unwrap().push(state.clone());
            Ok(())
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

    #[test]
    fn advance_records_one_exposure_per_shown_word() {
        let mut app = test_app(3, Config::default());
        app.advance();
        app.advance();
        app.advance();
        assert_eq!(app.progress.total_exposures(), 3);
        assert!(
            app.progress_dirty,
            "recorded exposure must mark state dirty"
        );
        // Every exposure belongs to a real deck word.
        assert!(app.progress.words.keys().all(|w| w.starts_with('w')));
    }

    #[test]
    fn quit_flushes_progress_through_the_store_port() {
        let store = RecordingStore::default();
        let mut app = test_app_with_ports(
            3,
            Config::default(),
            Box::new(NullSpeaker),
            Box::new(store.clone()),
        );
        app.advance();
        assert!(store.saves.lock().unwrap().is_empty(), "no eager saves");

        let ctx = egui::Context::default();
        assert!(!app.handle_command(AppCommand::Quit, &ctx));
        let saves = store.saves.lock().unwrap();
        assert_eq!(saves.len(), 1, "quit must flush exactly once");
        assert_eq!(saves[0].total_exposures(), 1);
        assert!(!app.progress_dirty);
    }

    #[test]
    fn drop_flushes_dirty_progress_as_a_backstop() {
        let store = RecordingStore::default();
        {
            let mut app = test_app_with_ports(
                3,
                Config::default(),
                Box::new(NullSpeaker),
                Box::new(store.clone()),
            );
            app.advance();
            app.advance();
        } // App dropped here without a Quit command.
        let saves = store.saves.lock().unwrap();
        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].total_exposures(), 2);
    }

    #[test]
    fn clean_state_is_not_rewritten_on_drop() {
        let store = RecordingStore::default();
        {
            let _app = test_app_with_ports(
                3,
                Config::default(),
                Box::new(NullSpeaker),
                Box::new(store.clone()),
            );
        } // Dropped with nothing recorded.
        assert!(
            store.saves.lock().unwrap().is_empty(),
            "a clean state must not be rewritten"
        );
    }

    #[test]
    fn pause_command_toggles_session_state() {
        let mut app = test_app(1, Config::default());
        let ctx = egui::Context::default();

        assert!(!app.clock.is_paused());
        assert!(app.handle_command(AppCommand::TogglePause, &ctx));
        assert!(app.clock.is_paused());
        assert!(app.handle_command(AppCommand::TogglePause, &ctx));
        assert!(!app.clock.is_paused());
    }

    #[test]
    fn loaded_source_waits_for_the_next_advance_before_replacing_fallback() {
        let mut app = test_app(1, Config::default());
        let (tx, source) = test_source_controller(1);
        app.source = Some(source);
        app.advance();
        assert_eq!(app.deck.current().unwrap().word, "w0");

        tx.send(crate::source::LoadReport::loaded(
            crate::source::SourceKind::Mongo,
            vec![Word {
                word: "remote".into(),
                transcription: String::new(),
                translation: String::new(),
                frequency: 1,
                example: String::new(),
            }],
        ))
        .unwrap();
        app.poll_source();

        assert_eq!(app.deck.current().unwrap().word, "w0");
        app.advance();
        assert_eq!(app.deck.current().unwrap().word, "remote");
        assert_eq!(
            app.source.as_ref().unwrap().status().active.kind,
            crate::source::SourceKind::Mongo
        );
    }

    #[test]
    fn fallback_source_report_does_not_queue_a_duplicate_deck() {
        let mut app = test_app(1, Config::default());
        let (tx, source) = test_source_controller(1);
        app.source = Some(source);
        let primary = crate::source::LoadReport::failed(
            crate::source::SourceKind::Mongo,
            crate::source::LoadIssue::new(crate::source::LoadIssueKind::Connection, "offline"),
        );
        let report =
            crate::source::LoadReport::with_fallback(primary, crate::fallback::fallback_words());

        tx.send(report).unwrap();
        app.poll_source();

        assert!(app.pending_words.is_none());
    }

    fn test_source_controller(
        fallback_words: usize,
    ) -> (
        std::sync::mpsc::Sender<crate::source::LoadReport>,
        SourceController,
    ) {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut receiver = Some(rx);
        let source = SourceController::new(
            fallback_words,
            Box::new(move |_| receiver.take().expect("test starts one source attempt")),
        );
        (tx, source)
    }
}
