use crate::db::Word;
use crate::ui::{self, CardView};
use eframe::egui;
use muda::MenuEvent;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

const WORD_INTERVAL: Duration = Duration::from_secs(30);
// Repaint cadence while a card is animating in (word -> transcription ->
// translation fades). ~60 fps keeps the fades smooth.
const ANIM_FRAME: Duration = Duration::from_millis(16);
// The card is fully settled once the translation has finished fading in.
const ANIM_END: f32 = ui::TRANSLATION_DELAY + ui::FADE_DURATION;

// Idle window measured by the frame-counter benchmark (MEMO_BENCH=1).
const BENCH_SECS: u64 = 10;

pub struct App {
    words: Vec<Word>,
    shown: HashSet<usize>,
    current_idx: Option<usize>,
    shown_at: Option<Instant>,
    last_show: Instant,
    prev_width: f32,
    started: bool,
    quit_id: muda::MenuId,
    bench: bool,
    frames: Arc<AtomicUsize>,
}

impl App {
    pub fn new(words: Vec<Word>, quit_id: muda::MenuId) -> Self {
        Self {
            words,
            shown: HashSet::new(),
            current_idx: None,
            shown_at: None,
            last_show: Instant::now(),
            prev_width: ui::MIN_WIDTH,
            started: false,
            quit_id,
            bench: std::env::var("MEMO_BENCH").is_ok(),
            frames: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn next_word(&mut self) {
        if self.words.is_empty() {
            return;
        }

        let mut available: Vec<usize> = (0..self.words.len())
            .filter(|i| !self.shown.contains(i))
            .collect();

        if available.is_empty() {
            self.shown.clear();
            available = (0..self.words.len()).collect();
        }

        let idx = *available.choose(&mut thread_rng()).unwrap();
        self.shown.insert(idx);
        self.current_idx = Some(idx);
        self.shown_at = Some(Instant::now());
        self.last_show = Instant::now();
    }

    fn fill_screen(&self, ctx: &egui::Context) {
        if let Some(screen) = ctx.input(|i| i.viewport().monitor_size) {
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(0.0, 0.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(screen.x, screen.y)));
        }
    }

    // Wake the UI thread on tray-menu events. Without this the event loop
    // would sleep through idle and the Quit item would only register on the
    // next scheduled repaint (up to WORD_INTERVAL later).
    fn spawn_menu_watcher(ctx: &egui::Context, quit_id: muda::MenuId) {
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let rx = MenuEvent::receiver();
            while let Ok(event) = rx.recv() {
                if event.id() == &quit_id {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
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

    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if self.bench {
            self.frames.fetch_add(1, Ordering::Relaxed);
        }

        if !self.started {
            self.started = true;
            self.fill_screen(ctx);
            Self::spawn_menu_watcher(ctx, self.quit_id.clone());
            self.next_word();

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

        if self.last_show.elapsed() >= WORD_INTERVAL {
            self.next_word();
        }

        let elapsed = self
            .shown_at
            .map(|t| t.elapsed().as_secs_f32())
            .unwrap_or(0.0);

        let frame = egui::Frame::central_panel(&ctx.style()).fill(egui::Color32::TRANSPARENT);

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            if let Some(idx) = self.current_idx {
                let w = &self.words[idx];
                let view = CardView {
                    word: &w.word,
                    transcription: &w.transcription,
                    translation: &w.translation,
                    elapsed,
                    prev_width: self.prev_width,
                };
                let widget_w = view.compute_width(ui);
                self.prev_width = widget_w;
                view.paint(ui, widget_w);
            }
        });

        // Drive repaints by state: animate at ~60 fps while the card fades in,
        // then sleep until the next word is due. A static card costs no frames.
        if elapsed < ANIM_END {
            ctx.request_repaint_after(ANIM_FRAME);
        } else {
            let until_next = WORD_INTERVAL.saturating_sub(self.last_show.elapsed());
            ctx.request_repaint_after(until_next);
        }
    }
}
