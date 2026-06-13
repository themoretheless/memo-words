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
            .choose_weighted(&mut rng(), |&i| weight(words[i].frequency))
            .expect("candidates non-empty")
    }
}

/// Selection weight for a frequency rank. Rank 1 (most common) gets the
/// largest weight and weight falls off as 1/rank. A missing, zero, or negative
/// rank means "frequency unknown" (e.g. a MongoDB document without the field,
/// which `#[serde(default)]` fills with 0) and is treated as the rarest tier.
/// The old `1.0 / frequency.max(1)` did the opposite: it clamped rank 0 up to
/// 1, handing unranked words the *maximum* weight and surfacing them most
/// often, contradicting the doc above.
fn weight(frequency: i32) -> f64 {
    if frequency >= 1 {
        1.0 / frequency as f64
    } else {
        1.0 / f64::from(i32::MAX)
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

    #[test]
    fn missing_frequency_is_treated_as_rarest_not_most_common() {
        // A word whose frequency field was absent in MongoDB deserializes to 0.
        // It must surface far less than a real rank-1 word, not more.
        let w = vec![
            Word {
                word: "common".into(),
                transcription: String::new(),
                translation: String::new(),
                frequency: 1,
            },
            Word {
                word: "unknown".into(),
                transcription: String::new(),
                translation: String::new(),
                frequency: 0,
            },
        ];
        let sel = FrequencyWeighted;
        let mut common = 0;
        for _ in 0..2000 {
            if sel.choose(&[0, 1], &w) == 0 {
                common += 1;
            }
        }
        // weight(0) is ~1/i32::MAX, so the rank-1 word wins virtually always.
        assert!(common > 1990, "common chosen {common}/2000");
    }

    #[test]
    fn weight_ranks_correctly() {
        // Most common (rank 1) outweighs rarer (rank 100) outweighs unknown (0).
        assert!(weight(1) > weight(100));
        assert!(weight(100) > weight(0));
        assert!(weight(0) > 0.0); // must stay positive for choose_weighted
        assert_eq!(weight(0), weight(-5)); // negatives are "unknown" too
    }
}
