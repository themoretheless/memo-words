use super::{LoadIssue, LoadIssueKind, LoadReport, SourceKind, WordSource};
use crate::model::Word;
use std::time::Duration;

pub struct MongoWordSource {
    pub uri: String,
    pub database: String,
    pub collection: String,
}

impl Default for MongoWordSource {
    fn default() -> Self {
        Self {
            uri: "mongodb://localhost:27017".to_string(),
            database: "english_words".to_string(),
            collection: "words".to_string(),
        }
    }
}

impl WordSource for MongoWordSource {
    fn load(&self) -> LoadReport {
        let runtime = match tokio::runtime::Runtime::new() {
            Ok(runtime) => runtime,
            Err(error) => {
                return failed(LoadIssueKind::Runtime, error);
            }
        };

        runtime.block_on(async {
            let mut options = match mongodb::options::ClientOptions::parse(&self.uri).await {
                Ok(options) => options,
                Err(error) => return failed(LoadIssueKind::Configuration, error),
            };
            options.server_selection_timeout = Some(Duration::from_secs(2));
            options.connect_timeout = Some(Duration::from_secs(2));

            let client = match mongodb::Client::with_options(options) {
                Ok(client) => client,
                Err(error) => return failed(LoadIssueKind::Connection, error),
            };
            let collection = client
                .database(&self.database)
                .collection::<Word>(&self.collection);
            let mut cursor = match collection.find(mongodb::bson::doc! {}).await {
                Ok(cursor) => cursor,
                Err(error) => return failed(LoadIssueKind::Query, error),
            };

            let mut words = Vec::new();
            let mut skipped = 0;
            let mut issues = Vec::new();
            loop {
                match cursor.advance().await {
                    Ok(true) => match cursor.deserialize_current() {
                        Ok(word) => words.push(word),
                        Err(error) => {
                            skipped += 1;
                            if issues.len() < 3 {
                                issues
                                    .push(LoadIssue::new(LoadIssueKind::Decode, error.to_string()));
                            }
                        }
                    },
                    Ok(false) => break,
                    Err(error) => {
                        issues.push(LoadIssue::new(LoadIssueKind::Cursor, error.to_string()));
                        break;
                    }
                }
            }

            LoadReport::from_parts(SourceKind::Mongo, words, skipped, issues)
        })
    }
}

fn failed(kind: LoadIssueKind, error: impl std::fmt::Display) -> LoadReport {
    LoadReport::failed(SourceKind::Mongo, LoadIssue::new(kind, error.to_string()))
}
