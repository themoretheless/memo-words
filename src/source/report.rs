//! Typed source outcomes retained across fallback and background loading.

use crate::model::Word;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Mongo,
    Static,
    Fallback,
}

impl fmt::Display for SourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mongo => f.write_str("MongoDB"),
            Self::Static => f.write_str("static"),
            Self::Fallback => f.write_str("built-in fallback"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadOutcome {
    Loaded,
    Partial,
    Empty,
    Failed,
    Fallback,
}

impl fmt::Display for LoadOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Loaded => "loaded",
            Self::Partial => "partial",
            Self::Empty => "empty",
            Self::Failed => "failed",
            Self::Fallback => "fallback",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadIssueKind {
    Runtime,
    Configuration,
    Connection,
    Query,
    Cursor,
    Decode,
    Empty,
}

impl fmt::Display for LoadIssueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Runtime => "runtime",
            Self::Configuration => "configuration",
            Self::Connection => "connection",
            Self::Query => "query",
            Self::Cursor => "cursor",
            Self::Decode => "decode",
            Self::Empty => "empty",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadIssue {
    pub kind: LoadIssueKind,
    pub message: String,
}

impl LoadIssue {
    pub fn new(kind: LoadIssueKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadReport {
    pub requested: SourceKind,
    pub active: Option<SourceKind>,
    pub outcome: LoadOutcome,
    pub words: Vec<Word>,
    pub skipped: usize,
    pub issues: Vec<LoadIssue>,
}

impl LoadReport {
    pub fn loaded(source: SourceKind, words: Vec<Word>) -> Self {
        Self::from_parts(source, words, 0, Vec::new())
    }

    pub fn failed(source: SourceKind, issue: LoadIssue) -> Self {
        Self::from_parts(source, Vec::new(), 0, vec![issue])
    }

    pub(crate) fn from_parts(
        source: SourceKind,
        words: Vec<Word>,
        skipped: usize,
        issues: Vec<LoadIssue>,
    ) -> Self {
        let (active, outcome) = if words.is_empty() {
            if issues.is_empty() {
                (None, LoadOutcome::Empty)
            } else {
                (None, LoadOutcome::Failed)
            }
        } else if skipped > 0 || !issues.is_empty() {
            (Some(source), LoadOutcome::Partial)
        } else {
            (Some(source), LoadOutcome::Loaded)
        };
        Self {
            requested: source,
            active,
            outcome,
            words,
            skipped,
            issues,
        }
    }

    pub fn with_fallback(mut primary: Self, words: Vec<Word>) -> Self {
        if primary.issues.is_empty() {
            primary.issues.push(LoadIssue::new(
                LoadIssueKind::Empty,
                "primary source returned no words",
            ));
        }
        primary.active = Some(SourceKind::Fallback);
        primary.outcome = LoadOutcome::Fallback;
        primary.words = words;
        primary
    }

    pub fn is_usable(&self) -> bool {
        self.active.is_some() && !self.words.is_empty()
    }
}

impl fmt::Display for LoadReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "requested={}, active={}, outcome={}, loaded={}, skipped={}, issues={}",
            self.requested,
            self.active
                .map(|source| source.to_string())
                .unwrap_or_else(|| "none".to_string()),
            self.outcome,
            self.words.len(),
            self.skipped,
            self.issues.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word() -> Word {
        Word {
            word: "test".into(),
            transcription: String::new(),
            translation: "test".into(),
            frequency: 1,
            example: String::new(),
        }
    }

    #[test]
    fn parts_distinguish_loaded_partial_empty_and_failed() {
        assert_eq!(
            LoadReport::from_parts(SourceKind::Static, vec![word()], 0, vec![]).outcome,
            LoadOutcome::Loaded
        );
        assert_eq!(
            LoadReport::from_parts(SourceKind::Static, vec![word()], 1, vec![]).outcome,
            LoadOutcome::Partial
        );
        assert_eq!(
            LoadReport::from_parts(SourceKind::Static, vec![], 0, vec![]).outcome,
            LoadOutcome::Empty
        );
        assert_eq!(
            LoadReport::failed(
                SourceKind::Static,
                LoadIssue::new(LoadIssueKind::Decode, "bad")
            )
            .outcome,
            LoadOutcome::Failed
        );
    }

    #[test]
    fn fallback_preserves_words_and_marks_the_degraded_outcome() {
        let primary = LoadReport::failed(
            SourceKind::Mongo,
            LoadIssue::new(LoadIssueKind::Connection, "offline"),
        );
        let report = LoadReport::with_fallback(primary, vec![word()]);

        assert!(report.is_usable());
        assert_eq!(report.outcome, LoadOutcome::Fallback);
    }
}
