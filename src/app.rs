use crate::db::Word;
use crate::ui::{self, CardView};
use eframe::egui;
use muda::MenuEvent;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashSet;
use std::time::{Duration, Instant};

const WORD_INTERVAL: Duration = Duration::from_secs(30);
const REPAINT_INTERVAL: Duration = Duration::from_millis(50);

pub struct App {
    words: Vec<Word>,
    shown: HashSet<usize>,
    current_idx: Option<usize>,
    shown_at: Option<Instant>,
    last_show: Instant,
    prev_width: f32,
    started: bool,
    quit_id: muda::MenuId,
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
            if event.id() == &self.quit_id {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
        }

        if self.last_show.elapsed() >= WORD_INTERVAL {
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
                };
                self.prev_width = view.compute_width(ui);
                view.paint(ui);
                ctx.request_repaint();
            }
        });
    }
}
