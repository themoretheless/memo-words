//! Presentation-clock state for one ambient learning session.
//!
//! `App` owns rendering and adapters; this type owns only time semantics. Taking
//! `Instant` as an argument keeps transitions deterministic in unit tests.

use std::time::{Duration, Instant};

pub struct SessionClock {
    shown_at: Option<Instant>,
    last_show: Instant,
    word_interval: Duration,
    paused_at: Option<Instant>,
}

impl SessionClock {
    pub fn new(now: Instant, word_interval: Duration) -> Self {
        Self {
            shown_at: None,
            last_show: now,
            word_interval,
            paused_at: None,
        }
    }

    pub fn start_word(&mut self, now: Instant, word_interval: Duration) {
        self.shown_at = Some(now);
        self.last_show = now;
        self.word_interval = word_interval;
    }

    pub fn toggle_pause(&mut self, now: Instant) {
        if let Some(started) = self.paused_at.take() {
            let paused_for = now.saturating_duration_since(started);
            self.shown_at = self
                .shown_at
                .and_then(|shown| shown.checked_add(paused_for));
            self.last_show = self.last_show.checked_add(paused_for).unwrap_or(now);
        } else {
            self.paused_at = Some(now);
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused_at.is_some()
    }

    pub fn is_due(&self, now: Instant) -> bool {
        !self.is_paused() && now.saturating_duration_since(self.last_show) >= self.word_interval
    }

    pub fn elapsed(&self, now: Instant) -> Duration {
        let effective_now = self.paused_at.unwrap_or(now);
        self.shown_at
            .map(|shown| effective_now.saturating_duration_since(shown))
            .unwrap_or(Duration::ZERO)
    }

    pub fn until_next(&self, now: Instant) -> Duration {
        let effective_now = self.paused_at.unwrap_or(now);
        self.word_interval
            .saturating_sub(effective_now.saturating_duration_since(self.last_show))
    }

    pub fn word_interval(&self) -> Duration {
        self.word_interval
    }

    pub fn pin_elapsed(&mut self, now: Instant, elapsed: Duration) {
        self.shown_at = now.checked_sub(elapsed).or(Some(now));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pause_freezes_reveal_and_interval_clocks() {
        let t0 = Instant::now();
        let mut clock = SessionClock::new(t0, Duration::from_secs(30));
        clock.start_word(t0, Duration::from_secs(30));
        let pause = t0 + Duration::from_secs(4);
        clock.toggle_pause(pause);

        let much_later = pause + Duration::from_secs(100);
        assert_eq!(clock.elapsed(much_later), Duration::from_secs(4));
        assert_eq!(clock.until_next(much_later), Duration::from_secs(26));
        assert!(!clock.is_due(much_later));
    }

    #[test]
    fn resume_continues_from_the_frozen_instant() {
        let t0 = Instant::now();
        let mut clock = SessionClock::new(t0, Duration::from_secs(30));
        clock.start_word(t0, Duration::from_secs(30));
        clock.toggle_pause(t0 + Duration::from_secs(4));
        clock.toggle_pause(t0 + Duration::from_secs(104));

        let after_resume = t0 + Duration::from_secs(109);
        assert_eq!(clock.elapsed(after_resume), Duration::from_secs(9));
        assert_eq!(clock.until_next(after_resume), Duration::from_secs(21));
    }

    #[test]
    fn due_only_after_active_interval() {
        let t0 = Instant::now();
        let mut clock = SessionClock::new(t0, Duration::from_secs(2));
        clock.start_word(t0, Duration::from_secs(2));
        assert!(!clock.is_due(t0 + Duration::from_secs(1)));
        assert!(clock.is_due(t0 + Duration::from_secs(2)));
    }
}
