use crate::db::Word;
use crate::selector::WordSelector;
use std::collections::{HashSet, VecDeque};

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
        }
    }

    pub fn current(&self) -> Option<&Word> {
        self.current.map(|i| &self.words[i])
    }

    /// Select a fresh word via the strategy, excluding the recent window.
    /// No-op on an empty deck.
    pub fn advance(&mut self) {
        if self.words.is_empty() {
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
}
