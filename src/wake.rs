//! One owned worker for long repaint deadlines.
//!
//! eframe invalidates delayed repaint requests after unrelated UI passes. This
//! worker keeps the absolute deadline outside that pass counter, while short
//! animation-frame requests continue to use egui directly.

use eframe::egui;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

enum Command {
    Schedule(Instant),
    Cancel,
    Stop,
}

pub struct WakeScheduler {
    tx: Sender<Command>,
    worker: Option<JoinHandle<()>>,
    scheduled_for: Option<Instant>,
}

impl WakeScheduler {
    pub fn new(ctx: &egui::Context) -> Self {
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        let worker = std::thread::spawn(move || run(rx, ctx));
        Self {
            tx,
            worker: Some(worker),
            scheduled_for: None,
        }
    }

    pub fn schedule(&mut self, now: Instant, delay: Duration, tolerance: Duration) {
        let target = now.checked_add(delay).unwrap_or(now);
        if self.scheduled_for.is_some_and(|existing| {
            now < existing && deadline_difference(existing, target) <= tolerance
        }) {
            return;
        }
        self.scheduled_for = Some(target);
        let _ = self.tx.send(Command::Schedule(target));
    }

    pub fn cancel(&mut self) {
        if self.scheduled_for.take().is_some() {
            let _ = self.tx.send(Command::Cancel);
        }
    }
}

impl Drop for WakeScheduler {
    fn drop(&mut self) {
        let _ = self.tx.send(Command::Stop);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn run(rx: Receiver<Command>, ctx: egui::Context) {
    let mut deadline: Option<Instant> = None;
    loop {
        let command = match deadline {
            Some(target) => match target.checked_duration_since(Instant::now()) {
                Some(wait) => match rx.recv_timeout(wait) {
                    Ok(command) => command,
                    Err(RecvTimeoutError::Timeout) => {
                        deadline = None;
                        ctx.request_repaint();
                        continue;
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                },
                None => {
                    deadline = None;
                    ctx.request_repaint();
                    continue;
                }
            },
            None => match rx.recv() {
                Ok(command) => command,
                Err(_) => break,
            },
        };

        match command {
            Command::Schedule(target) => deadline = Some(target),
            Command::Cancel => deadline = None,
            Command::Stop => break,
        }
    }
}

fn deadline_difference(a: Instant, b: Instant) -> Duration {
    if a >= b {
        a.duration_since(b)
    } else {
        b.duration_since(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deadline_difference_is_symmetric() {
        let now = Instant::now();
        let later = now + Duration::from_secs(3);
        assert_eq!(deadline_difference(now, later), Duration::from_secs(3));
        assert_eq!(deadline_difference(later, now), Duration::from_secs(3));
    }

    #[test]
    fn scheduler_wakes_egui_at_the_deadline() {
        let ctx = egui::Context::default();
        let (tx, rx) = mpsc::channel();
        ctx.set_request_repaint_callback(move |_| {
            let _ = tx.send(());
        });

        let mut scheduler = WakeScheduler::new(&ctx);
        scheduler.schedule(Instant::now(), Duration::from_millis(10), Duration::ZERO);
        assert!(rx.recv_timeout(Duration::from_secs(1)).is_ok());
    }
}
