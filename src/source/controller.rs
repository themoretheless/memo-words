//! Runtime ownership for one source attempt at a time.

use super::{LoadOutcome, LoadReport, SourceKind};
use crate::model::Word;
use std::fmt;
use std::sync::mpsc::{Receiver, TryRecvError};

pub type SourceLauncher = Box<dyn FnMut(u64) -> Receiver<LoadReport>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceHealth {
    Loading,
    Normal,
    Degraded,
    Retrying,
    Failed,
}

impl fmt::Display for SourceHealth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Loading => "loading",
            Self::Normal => "normal",
            Self::Degraded => "degraded",
            Self::Retrying => "retrying",
            Self::Failed => "failed",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeckSource {
    pub kind: SourceKind,
    pub words: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceStatus {
    pub health: SourceHealth,
    pub active: DeckSource,
    pub pending: Option<DeckSource>,
    pub attempt_id: u64,
}

pub struct SourceUpdate {
    pub words: Option<Vec<Word>>,
    pub report_received: bool,
}

pub struct SourceController {
    launcher: SourceLauncher,
    receiver: Option<Receiver<LoadReport>>,
    latest_report: Option<LoadReport>,
    status: SourceStatus,
    next_attempt_id: u64,
}

impl SourceController {
    pub fn new(fallback_words: usize, mut launcher: SourceLauncher) -> Self {
        let attempt_id = 1;
        let receiver = launcher(attempt_id);
        Self {
            launcher,
            receiver: Some(receiver),
            latest_report: None,
            status: SourceStatus {
                health: SourceHealth::Loading,
                active: DeckSource {
                    kind: SourceKind::Fallback,
                    words: fallback_words,
                },
                pending: None,
                attempt_id,
            },
            next_attempt_id: 2,
        }
    }

    pub fn status(&self) -> &SourceStatus {
        &self.status
    }

    pub fn latest_report(&self) -> Option<&LoadReport> {
        self.latest_report.as_ref()
    }

    pub fn can_reload(&self) -> bool {
        self.receiver.is_none() && self.status.pending.is_none()
    }

    pub fn reload(&mut self) -> bool {
        if !self.can_reload() {
            return false;
        }
        let attempt_id = self.next_attempt_id;
        self.next_attempt_id = self.next_attempt_id.saturating_add(1);
        self.status.health = SourceHealth::Retrying;
        self.status.attempt_id = attempt_id;
        self.receiver = Some((self.launcher)(attempt_id));
        true
    }

    pub fn poll(&mut self) -> Option<SourceUpdate> {
        let result = self.receiver.as_ref()?.try_recv();
        match result {
            Ok(mut report) => {
                self.receiver = None;
                if let Some(attempt) = &report.attempt {
                    self.status.attempt_id = attempt.id;
                }
                self.status.health = health_for(report.outcome);
                let words = if matches!(report.outcome, LoadOutcome::Loaded | LoadOutcome::Partial)
                    && !report.words.is_empty()
                {
                    self.status.pending = Some(DeckSource {
                        kind: report.active.unwrap_or(report.requested),
                        words: report.loaded,
                    });
                    Some(std::mem::take(&mut report.words))
                } else {
                    self.status.pending = None;
                    None
                };
                self.latest_report = Some(report);
                Some(SourceUpdate {
                    words,
                    report_received: true,
                })
            }
            Err(TryRecvError::Disconnected) => {
                self.receiver = None;
                self.status.health = SourceHealth::Failed;
                self.status.pending = None;
                Some(SourceUpdate {
                    words: None,
                    report_received: false,
                })
            }
            Err(TryRecvError::Empty) => None,
        }
    }

    pub fn activate_pending(&mut self) -> bool {
        let Some(pending) = self.status.pending.take() else {
            return false;
        };
        self.status.active = pending;
        true
    }
}

fn health_for(outcome: LoadOutcome) -> SourceHealth {
    match outcome {
        LoadOutcome::Loaded => SourceHealth::Normal,
        LoadOutcome::Partial | LoadOutcome::Fallback => SourceHealth::Degraded,
        LoadOutcome::Empty | LoadOutcome::Failed => SourceHealth::Failed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{LoadIssue, LoadIssueKind};
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::mpsc::{self, Sender};

    fn word(value: &str) -> Word {
        Word {
            word: value.into(),
            transcription: String::new(),
            translation: value.into(),
            frequency: 1,
            example: String::new(),
        }
    }

    #[test]
    fn successful_load_is_retained_then_activated_and_can_be_retried() {
        let senders = Rc::new(RefCell::new(Vec::<Sender<LoadReport>>::new()));
        let attempt_ids = Rc::new(RefCell::new(Vec::new()));
        let senders_for_launcher = senders.clone();
        let ids_for_launcher = attempt_ids.clone();
        let mut controller = SourceController::new(
            40,
            Box::new(move |attempt_id| {
                let (tx, rx) = mpsc::channel();
                senders_for_launcher.borrow_mut().push(tx);
                ids_for_launcher.borrow_mut().push(attempt_id);
                rx
            }),
        );

        assert_eq!(controller.status.health, SourceHealth::Loading);
        assert_eq!(controller.status.active.kind, SourceKind::Fallback);
        assert!(!controller.can_reload());
        senders.borrow()[0]
            .send(LoadReport::loaded(
                SourceKind::Mongo,
                vec![word("one"), word("two")],
            ))
            .unwrap();

        let update = controller.poll().unwrap();
        assert_eq!(update.words.unwrap().len(), 2);
        assert_eq!(controller.status.health, SourceHealth::Normal);
        assert_eq!(controller.status.active.kind, SourceKind::Fallback);
        assert_eq!(controller.status.pending.unwrap().kind, SourceKind::Mongo);
        assert_eq!(controller.latest_report().unwrap().loaded, 2);
        assert!(controller.latest_report().unwrap().words.is_empty());
        assert!(!controller.can_reload());

        assert!(controller.activate_pending());
        assert_eq!(controller.status.active.kind, SourceKind::Mongo);
        assert_eq!(controller.status.active.words, 2);
        assert!(controller.can_reload());
        assert!(controller.reload());
        assert_eq!(controller.status.health, SourceHealth::Retrying);
        assert!(!controller.reload());
        assert_eq!(&*attempt_ids.borrow(), &[1, 2]);

        let failed = LoadReport::failed(
            SourceKind::Mongo,
            LoadIssue::new(LoadIssueKind::Connection, "offline"),
        );
        senders.borrow()[1]
            .send(LoadReport::with_fallback(failed, vec![word("fallback")]))
            .unwrap();
        let update = controller.poll().unwrap();

        assert!(update.words.is_none());
        assert_eq!(controller.status.health, SourceHealth::Degraded);
        assert_eq!(controller.status.active.kind, SourceKind::Mongo);
        assert_eq!(controller.status.active.words, 2);
        assert_eq!(
            controller.latest_report().unwrap().outcome,
            LoadOutcome::Fallback
        );
    }

    #[test]
    fn disconnected_attempt_marks_failure_without_erasing_active_deck() {
        let mut controller = SourceController::new(
            40,
            Box::new(|_| {
                let (tx, rx) = mpsc::channel::<LoadReport>();
                drop(tx);
                rx
            }),
        );

        assert!(controller.poll().is_some());
        assert_eq!(controller.status.health, SourceHealth::Failed);
        assert_eq!(controller.status.active.kind, SourceKind::Fallback);
        assert_eq!(controller.status.active.words, 40);
        assert!(controller.can_reload());
    }
}
