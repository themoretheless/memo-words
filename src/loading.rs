//! One-shot background execution for synchronous word-source adapters.

use crate::source::{LoadReport, WordSource};
use std::sync::mpsc::{self, Receiver};
use std::time::{Instant, SystemTime};

/// Run a source away from the UI thread and notify the caller after the result
/// is available. The callback keeps this adapter independent from egui while
/// still allowing the composition root to wake a sleeping UI.
pub fn spawn<S, F>(attempt_id: u64, source: S, on_complete: F) -> Receiver<LoadReport>
where
    S: WordSource,
    F: FnOnce() + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let started_at = Instant::now();
        let mut report = source.load();
        report.complete_attempt(attempt_id, started_at.elapsed(), SystemTime::now());
        if tx.send(report).is_ok() {
            on_complete();
        }
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Word;
    use crate::source::{LoadOutcome, StaticWordSource};
    use std::time::Duration;

    #[test]
    fn source_runs_in_background_and_notifies_after_delivery() {
        let word = Word {
            word: "ready".into(),
            transcription: String::new(),
            translation: "ready".into(),
            frequency: 1,
            example: String::new(),
        };
        let (wake_tx, wake_rx) = mpsc::channel();
        let report_rx = spawn(7, StaticWordSource(vec![word]), move || {
            wake_tx.send(()).unwrap();
        });

        let report = report_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        wake_rx.recv_timeout(Duration::from_secs(1)).unwrap();

        assert_eq!(report.outcome, LoadOutcome::Loaded);
        assert_eq!(report.words[0].word, "ready");
        assert_eq!(report.attempt.unwrap().id, 7);
    }
}
