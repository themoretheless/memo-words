//! Word-source facade and fallback policy.

mod controller;
mod mongo;
mod report;
mod static_source;

use crate::fallback::fallback_words;

#[cfg(test)]
pub use controller::DeckSource;
pub use controller::{SourceController, SourceHealth, SourceStatus};
pub use mongo::MongoWordSource;
pub use report::{LoadIssue, LoadIssueKind, LoadOutcome, LoadReport, SourceKind};
pub use static_source::StaticWordSource;

/// A synchronous source capability. Slow implementations are executed by the
/// background loader; the domain-facing contract remains small and testable.
pub trait WordSource: Send + 'static {
    fn load(&self) -> LoadReport;
}

/// Decorates any primary source with the built-in offline deck while retaining
/// the primary attempt, outcome, and issues in the returned report.
pub struct WithFallback<S: WordSource>(pub S);

impl<S: WordSource> WordSource for WithFallback<S> {
    fn load(&self) -> LoadReport {
        let report = self.0.load();
        if report.is_usable() {
            report
        } else {
            LoadReport::with_fallback(report, fallback_words())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FailedSource;

    impl WordSource for FailedSource {
        fn load(&self) -> LoadReport {
            LoadReport::failed(
                SourceKind::Mongo,
                LoadIssue::new(LoadIssueKind::Connection, "offline"),
            )
        }
    }

    #[test]
    fn fallback_preserves_primary_failure_context() {
        let report = WithFallback(FailedSource).load();
        assert_eq!(report.requested, SourceKind::Mongo);
        assert_eq!(report.active, Some(SourceKind::Fallback));
        assert_eq!(report.outcome, LoadOutcome::Fallback);
        assert_eq!(report.issues[0].kind, LoadIssueKind::Connection);
        assert!(!report.words.is_empty());
    }
}
