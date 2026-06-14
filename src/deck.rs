use crate::db::Word;
use crate::selector::WordSelector;
use rand::RngExt;
use rand::rng;
use std::collections::{HashSet, VecDeque};

/// A spaced recap re-shows a word at least this many positions back in the
/// recent window, so it reads as a deliberate review rather than an obvious
/// short-term repeat. The window must be longer than this to recap at all.
const RECAP_MIN_LAG: usize = 5;

/// Owns the word list and the rotation policy: a sliding window of recently
/// shown words (to avoid short-term repeats) plus a pluggable selection
/// strategy. Knows nothing about UI, timing, audio, or persistence (SRP), which
/// also makes it unit-testable without egui or a database.
pub struct Deck {
    words: Vec<Word>,
    selector: Box<dyn WordSelector>,
    recent: VecDeque<usize>,
    recent_set: HashSet<usize>,
    recent_cap: usize,
    current: Option<usize>,
    recap_chance: f32,
}

impl Deck {
    pub fn new(words: Vec<Word>, selector: Box<dyn WordSelector>) -> Self {
        // Window sized to ~a third of the deck, capped so large decks stay
        // varied and small decks (< 3 words) don't exclude every candidate.
        let recent_cap = (words.len() / 3).min(100);
        Self {
            words,
            selector,
            recent: VecDeque::new(),
            recent_set: HashSet::new(),
            recent_cap,
            current: None,
            recap_chance: 0.0,
        }
    }

    /// Set the probability (0.0..=1.0) that a swap re-shows an earlier word for
    /// spaced review instead of picking a fresh one. 0 (the default) is off.
    pub fn with_recap_chance(mut self, chance: f32) -> Self {
        self.recap_chance = chance.clamp(0.0, 1.0);
        self
    }

    pub fn current(&self) -> Option<&Word> {
        self.current.map(|i| &self.words[i])
    }

    /// Advance to the next word. Usually a fresh pick via the strategy (excluding
    /// the recent window); with probability `recap_chance` it instead re-shows an
    /// earlier word for spaced review. No-op on an empty deck.
    pub fn advance(&mut self) {
        if self.words.is_empty() {
            return;
        }
        // Spaced recap: roll the dice, and only if there's an old-enough word in
        // the window. Otherwise fall through to a normal fresh pick.
        if self.recap_chance > 0.0
            && rng().random_range(0.0_f32..1.0) < self.recap_chance
            && self.try_recap()
        {
            return;
        }

        let candidates: Vec<usize> = (0..self.words.len())
            .filter(|i| !self.recent_set.contains(i))
            .collect();
        let idx = self.selector.choose(&candidates, &self.words);

        if self.recent_cap > 0 {
            self.recent.push_back(idx);
            self.recent_set.insert(idx);
            while self.recent.len() > self.recent_cap {
                if let Some(old) = self.recent.pop_front() {
                    self.recent_set.remove(&old);
                }
            }
        }
        self.current = Some(idx);
    }

    /// Re-show a word from the older part of the recent window (at least
    /// `RECAP_MIN_LAG` back, so it isn't an obvious repeat) and refresh its
    /// recency by moving it to the newest slot. Returns false (so the caller does
    /// a normal pick) when the window is too short to hold an old-enough word.
    /// The word stays in `recent_set`, so the window invariant and its length are
    /// preserved, this only reorders one existing entry.
    fn try_recap(&mut self) -> bool {
        let pool = self.recent.len().saturating_sub(RECAP_MIN_LAG);
        if pool == 0 {
            return false;
        }
        let p = rng().random_range(0..pool);
        if let Some(idx) = self.recent.remove(p) {
            self.recent.push_back(idx);
            self.current = Some(idx);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selector::FrequencyWeighted;

    fn deck(n: usize) -> Deck {
        let words = (0..n)
            .map(|i| Word {
                word: format!("w{i}"),
                transcription: String::new(),
                translation: String::new(),
                frequency: 1,
                example: String::new(),
            })
            .collect();
        Deck::new(words, Box::new(FrequencyWeighted))
    }

    #[test]
    fn empty_deck_advance_is_noop() {
        let mut d = deck(0);
        d.advance();
        assert!(d.current().is_none());
    }

    #[test]
    fn advance_sets_a_current_word() {
        let mut d = deck(5);
        assert!(d.current().is_none());
        d.advance();
        assert!(d.current().is_some());
    }

    #[test]
    fn recent_window_avoids_short_term_repeats() {
        // 30 words -> recent_cap 10. Across 10 consecutive picks, none repeats.
        let mut d = deck(30);
        let mut seen = Vec::new();
        for _ in 0..10 {
            d.advance();
            let w = d.current().unwrap().word.clone();
            assert!(!seen.contains(&w), "repeat within window: {w}");
            seen.push(w);
        }
    }

    #[test]
    fn recent_set_stays_in_sync_and_bounded() {
        let mut d = deck(30);
        for _ in 0..100 {
            d.advance();
            assert_eq!(d.recent.len(), d.recent_set.len());
            assert!(d.recent.len() <= d.recent_cap);
        }
    }

    #[test]
    fn tiny_deck_has_no_window_and_still_advances() {
        // 2 words -> recent_cap 0; repeats are allowed but it must not panic
        // or stall.
        let mut d = deck(2);
        for _ in 0..10 {
            d.advance();
            assert!(d.current().is_some());
        }
    }

    #[test]
    fn with_recap_chance_clamps() {
        let d = deck(30).with_recap_chance(5.0);
        assert_eq!(d.recap_chance, 1.0);
        let d = deck(30).with_recap_chance(-1.0);
        assert_eq!(d.recap_chance, 0.0);
    }

    #[test]
    fn recap_off_never_repeats_within_window() {
        // recap_chance 0 (default) must keep the no-repeat-within-window
        // guarantee: 300 words -> cap 100, so 100 consecutive picks are unique.
        let mut d = deck(300);
        let mut seen = Vec::new();
        for _ in 0..100 {
            d.advance();
            let i = d.current.unwrap();
            assert!(!seen.contains(&i), "repeat within window at recap_chance 0");
            seen.push(i);
        }
    }

    #[test]
    fn recap_always_reshows_an_earlier_word() {
        // With chance 1.0 and a full window, once enough history exists every
        // swap re-shows a word already in the recent set (a genuine recap).
        let mut d = deck(300).with_recap_chance(1.0);
        // Prime the window past RECAP_MIN_LAG so a recap pool exists.
        for _ in 0..20 {
            d.advance();
        }
        let mut recaps = 0;
        for _ in 0..30 {
            let before: std::collections::HashSet<usize> = d.recent_set.clone();
            d.advance();
            if before.contains(&d.current.unwrap()) {
                recaps += 1;
            }
        }
        assert!(recaps > 0, "expected at least one recap with chance 1.0");
    }

    #[test]
    fn recap_keeps_window_in_sync_and_bounded() {
        // The recap path reorders the deque; the set and length invariants must
        // survive a long run with recaps firing every swap.
        let mut d = deck(60).with_recap_chance(1.0);
        for _ in 0..500 {
            d.advance();
            assert_eq!(d.recent.len(), d.recent_set.len());
            assert!(d.recent.len() <= d.recent_cap);
        }
    }

    #[test]
    fn recap_noop_until_window_is_long_enough() {
        // A short window (<= RECAP_MIN_LAG) has no old-enough word, so even at
        // chance 1.0 the first few advances must fall back to fresh picks and
        // still set a current word without panicking.
        let mut d = deck(18).with_recap_chance(1.0); // cap = 6
        for _ in 0..3 {
            d.advance();
            assert!(d.current().is_some());
        }
    }
}
