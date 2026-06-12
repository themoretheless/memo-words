use crate::config::Config;
use crate::db::Word;
use crate::ui::{self, CardView};
use eframe::egui;
use muda::MenuEvent;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

const REPAINT_INTERVAL: Duration = Duration::from_millis(50);

#[derive(Clone)]
pub struct MenuIds {
    pub next: muda::MenuId,
    pub pause: muda::MenuId,
    pub quit: muda::MenuId,
}

pub struct App {
    words: Vec<Word>,
    recent: VecDeque<usize>,
    recent_cap: usize,
    current_idx: Option<usize>,
    shown_at: Option<Instant>,
    last_show: Instant,
    prev_width: f32,
    started: bool,
    menu_ids: MenuIds,
    paused: bool,
    cfg: Config,
    word_interval: Duration,
}

impl App {
    pub fn new(words: Vec<Word>, menu_ids: MenuIds, cfg: Config) -> Self {
        // Sliding window of recently shown words: avoids short-term repeats
        // while still letting frequent words recur over time. Sized to ~a
        // third of the deck, capped so large decks stay varied and small
        // decks don't exclude everything.
        let recent_cap = (words.len() / 3).min(100);
        Self {
            words,
            recent: VecDeque::new(),
            recent_cap,
            current_idx: None,
            shown_at: None,
            last_show: Instant::now(),
            prev_width: ui::MIN_WIDTH,
            started: false,
            menu_ids,
            paused: false,
            cfg,
            word_interval: Duration::from_secs(cfg.interval_secs),
        }
    }

    fn next_word(&mut self) {
        if self.words.is_empty() {
            return;
        }

        let available: Vec<usize> = (0..self.words.len())
            .filter(|i| !self.recent.contains(i))
            .collect();

        // `frequency` is a rank (1 = most common), so weight by its inverse:
        // common words surface more often, rarer ones still appear. Rank <= 0
        // (missing data) falls back to the rarest tier.
        let rng = &mut thread_rng();
        let idx = available
            .choose_weighted(rng, |&i| 1.0 / self.words[i].frequency.max(1) as f64)
            .copied()
            .unwrap();

        if self.recent_cap > 0 {
            self.recent.push_back(idx);
            while self.recent.len() > self.recent_cap {
                self.recent.pop_front();
            }
        }
        self.current_idx = Some(idx);
        self.shown_at = Some(Instant::now());
        self.last_show = Instant::now();
        self.word_interval = self.roll_interval();

        if self.cfg.speak {
            speak_word(&self.words[idx].word);
        }
    }

    // Time the current word stays up: base interval optionally jittered by
    // +/- jitter_secs so the cadence doesn't feel metronomic. Clamped to >=1s.
    fn roll_interval(&self) -> Duration {
        let base = self.cfg.interval_secs as i64;
        if self.cfg.jitter_secs == 0 {
            return Duration::from_secs(base.max(1) as u64);
        }
        let j = self.cfg.jitter_secs as i64;
        let delta = thread_rng().gen_range(-j..=j);
        Duration::from_secs((base + delta).max(1) as u64)
    }

    fn fill_screen(&self, ctx: &egui::Context) {
        if let Some(screen) = ctx.input(|i| i.viewport().monitor_size) {
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(0.0, 0.0)));
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(screen.x, screen.y)));
        }
    }
}

impl eframe::App for App {
    fn clear_color(&self, _: &egui::Visuals) -> [f32; 4] {
        [0.0; 4]
    }

    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if !self.started {
            self.started = true;
            self.fill_screen(ctx);
            self.next_word();
        }

        if let Ok(event) = MenuEvent::receiver().try_recv() {
            let id = event.id();
            if id == &self.menu_ids.quit {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            } else if id == &self.menu_ids.pause {
                self.paused = !self.paused;
                // Reset the timer so resuming gives a full interval rather
                // than instantly advancing on leftover elapsed time.
                self.last_show = Instant::now();
            } else if id == &self.menu_ids.next {
                self.next_word();
            }
        }

        if !self.paused && self.last_show.elapsed() >= self.word_interval {
            self.next_word();
        }

        ctx.request_repaint_after(REPAINT_INTERVAL);

        let frame = egui::Frame::central_panel(&ctx.style()).fill(egui::Color32::TRANSPARENT);

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            if let (Some(idx), Some(shown_at)) = (self.current_idx, self.shown_at) {
                let w = &self.words[idx];
                let view = CardView {
                    word: &w.word,
                    transcription: &w.transcription,
                    translation: &w.translation,
                    elapsed: shown_at.elapsed().as_secs_f32(),
                    prev_width: self.prev_width,
                    transcription_delay: self.cfg.transcription_delay,
                    translation_delay: self.cfg.translation_delay,
                    fade_duration: self.cfg.fade_duration,
                    corner: self.cfg.corner,
                };
                self.prev_width = view.compute_width(ui);
                view.paint(ui);
                ctx.request_repaint();
            }
        });
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
