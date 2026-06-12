use crate::db::Word;
use rand::rng;
use rand::seq::IndexedRandom;

/// Strategy for choosing the next word out of a candidate set. Implementations
/// encapsulate the selection policy (uniform, frequency-weighted, or a future
/// spaced-repetition scheme) so the rest of the app stays closed to that
/// change (Open/Closed) and depends only on this interface.
pub trait WordSelector {
    /// Pick one index out of `candidates` (indices into `words`).
    /// `candidates` is guaranteed non-empty by the caller.
    fn choose(&self, candidates: &[usize], words: &[Word]) -> usize;
}

/// Weights by inverse frequency rank (1 = most common) so common words surface
/// more often; missing/zero ranks fall back to the rarest tier.
pub struct FrequencyWeighted;

impl WordSelector for FrequencyWeighted {
    fn choose(&self, candidates: &[usize], words: &[Word]) -> usize {
        *candidates
            .choose_weighted(&mut rng(), |&i| 1.0 / words[i].frequency.max(1) as f64)
            .expect("candidates non-empty")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words() -> Vec<Word> {
        vec![
            Word {
                word: "common".into(),
                transcription: String::new(),
                translation: String::new(),
                frequency: 1,
            },
            Word {
                word: "rare".into(),
                transcription: String::new(),
                translation: String::new(),
                frequency: 100,
            },
        ]
    }

    #[test]
    fn frequency_weighted_favours_the_common_word() {
        let w = words();
        let sel = FrequencyWeighted;
        let mut common = 0;
        for _ in 0..2000 {
            if sel.choose(&[0, 1], &w) == 0 {
                common += 1;
            }
        }
        // Rank 1 has 100x the weight of rank 100, so it should dominate well
        // past a coin flip. Loose bound to stay non-flaky.
        assert!(common > 1500, "common chosen {common}/2000");
    }
}
