use super::{LoadReport, SourceKind, WordSource};
use crate::model::Word;

/// A fixed in-memory source used by tests and the benchmark harness.
pub struct StaticWordSource(pub Vec<Word>);

impl WordSource for StaticWordSource {
    fn load(&self) -> LoadReport {
        LoadReport::loaded(SourceKind::Static, self.0.clone())
    }
}
