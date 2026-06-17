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
    review_boosts: HashSet<String>,
    review_weighting: bool,
    current: Option<usize>,
}

impl Deck {
    #[cfg(test)]
    pub fn new(words: Vec<Word>, selector: Box<dyn WordSelector>) -> Self {
        Self::with_options(words, selector, 0, false)
    }

    pub fn with_options(
        words: Vec<Word>,
        selector: Box<dyn WordSelector>,
        cooldown_words: usize,
        review_weighting: bool,
    ) -> Self {
        let recent_cap = resolve_recent_cap(words.len(), cooldown_words);
        Self {
            words,
            selector,
            recent: VecDeque::new(),
            recent_set: HashSet::new(),
            recent_cap,
            review_boosts: HashSet::new(),
            review_weighting,
            current: None,
        }
    }

    pub fn current(&self) -> Option<&Word> {
        self.current.map(|i| &self.words[i])
    }

    pub fn words(&self) -> &[Word] {
        &self.words
    }

    pub fn recent_cap(&self) -> usize {
        self.recent_cap
    }

    pub fn set_cooldown_words(&mut self, cooldown_words: usize) {
        self.recent_cap = resolve_recent_cap(self.words.len(), cooldown_words);
        while self.recent.len() > self.recent_cap {
            if let Some(old) = self.recent.pop_front() {
                self.recent_set.remove(&old);
            }
        }
        if self.recent_cap == 0 {
            self.recent.clear();
            self.recent_set.clear();
        }
    }

    pub fn set_review_weighting(&mut self, enabled: bool) {
        self.review_weighting = enabled;
        if !enabled {
            self.review_boosts.clear();
        }
    }

    pub fn boost_word_for_review(&mut self, word: &str) {
        if self.review_weighting {
            let key = normalize_word_key(word);
            if !key.is_empty() {
                self.review_boosts.insert(key);
            }
        }
    }

    pub fn boost_current_for_review(&mut self) {
        if let Some(word) = self.current().map(|w| w.word.clone()) {
            self.boost_word_for_review(&word);
        }
    }

    #[cfg(test)]
    pub(crate) fn is_review_boosted(&self, word: &str) -> bool {
        self.review_boosts.contains(&normalize_word_key(word))
    }

    /// Select a fresh word via the strategy, excluding the recent window.
    /// No-op on an empty deck.
    pub fn advance(&mut self) {
        if self.words.is_empty() {
            return;
        }
        let mut candidates: Vec<usize> = (0..self.words.len())
            .filter(|i| !self.recent_set.contains(i))
            .collect();
        if candidates.is_empty() {
            candidates.extend(0..self.words.len());
        }
        if self.review_weighting {
            let boosted = candidates.clone();
            for idx in boosted {
                if self
                    .review_boosts
                    .contains(&normalize_word_key(&self.words[idx].word))
                {
                    candidates.extend([idx, idx]);
                }
            }
        }
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

fn normalize_word_key(word: &str) -> String {
    word.trim().to_ascii_lowercase()
}

fn resolve_recent_cap(word_count: usize, cooldown_words: usize) -> usize {
    // Window sized to ~a third of the deck, capped so large decks stay varied
    // and small decks (< 3 words) don't exclude every candidate.
    if cooldown_words == 0 {
        (word_count / 3).min(100)
    } else {
        cooldown_words.min(word_count.saturating_sub(1))
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

    fn deck_with_options(n: usize, cooldown_words: usize, review_weighting: bool) -> Deck {
        let words = (0..n)
            .map(|i| Word {
                word: format!("w{i}"),
                transcription: String::new(),
                translation: String::new(),
                frequency: 1,
                example: String::new(),
            })
            .collect();
        Deck::with_options(
            words,
            Box::new(FrequencyWeighted),
            cooldown_words,
            review_weighting,
        )
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
    fn configured_cooldown_overrides_auto_window() {
        let d = deck_with_options(30, 7, false);
        assert_eq!(d.recent_cap(), 7);
    }

    #[test]
    fn configured_cooldown_leaves_one_candidate_for_small_decks() {
        let d = deck_with_options(2, 99, false);
        assert_eq!(d.recent_cap(), 1);
    }

    #[test]
    fn review_boosting_records_requested_words_only_when_enabled() {
        let mut d = deck_with_options(5, 0, true);
        d.boost_word_for_review(" W1 ");
        assert!(d.is_review_boosted("w1"));

        let mut disabled = deck_with_options(5, 0, false);
        disabled.boost_word_for_review("w1");
        assert!(!disabled.is_review_boosted("w1"));
    }

    #[test]
    fn set_cooldown_words_trims_existing_recent_window() {
        let mut d = deck_with_options(30, 10, false);
        for _ in 0..10 {
            d.advance();
        }

        d.set_cooldown_words(3);

        assert_eq!(d.recent_cap(), 3);
        assert!(d.recent.len() <= 3);
        assert_eq!(d.recent.len(), d.recent_set.len());
    }

    #[test]
    fn disabling_review_weighting_clears_boosts() {
        let mut d = deck_with_options(5, 0, true);
        d.boost_word_for_review("w1");
        d.set_review_weighting(false);

        assert!(!d.is_review_boosted("w1"));
    }
}
