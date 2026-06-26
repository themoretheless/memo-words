//! Where words are loaded from. The `WordSource` trait (DIP) decouples the app
//! from any concrete backend, so the composition root in `main` wires in MongoDB,
//! a static set, or a fallback-wrapped source, and tests use an in-memory one
//! without a database. The concrete backends live here; the data model lives in
//! [`crate::model`] and the built-in deck in [`crate::fallback`].

use crate::fallback::{FALLBACK, fallback_words};
use crate::model::Word;
use std::time::Duration;

/// A place words can be loaded from. Decoupling the app from MongoDB behind
/// this trait (DIP) lets `main` wire in any concrete source and lets tests use
/// an in-memory one without a database.
pub trait WordSource {
    fn load(&self) -> Vec<Word>;
}

/// Loads from a MongoDB collection, returning an empty vec on any failure so a
/// caller can fall back gracefully.
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
    fn load(&self) -> Vec<Word> {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            let mut options = match mongodb::options::ClientOptions::parse(&self.uri).await {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("MongoDB connection string invalid: {e}");
                    return Vec::new();
                }
            };
            // Fail over to the fallback deck within a couple of seconds instead
            // of blocking the UI for the driver's default 30s server-selection
            // timeout when no MongoDB is reachable (the common "not running"
            // case). The card should appear promptly, not after a long stall.
            options.server_selection_timeout = Some(Duration::from_secs(2));
            options.connect_timeout = Some(Duration::from_secs(2));

            let client = match mongodb::Client::with_options(options) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("MongoDB client init failed: {e}");
                    return Vec::new();
                }
            };

            let collection = client
                .database(&self.database)
                .collection::<Word>(&self.collection);

            let mut cursor = match collection.find(mongodb::bson::doc! {}).await {
                Ok(c) => c,
                Err(e) => {
                    // A real connection failure (server down/unreachable)
                    // surfaces here, not at parse time, so the actionable hint
                    // belongs on this branch.
                    eprintln!("Failed to query words: {e}");
                    eprintln!(
                        "Is MongoDB running? Start it: brew services start mongodb-community"
                    );
                    return Vec::new();
                }
            };

            let mut words = Vec::new();
            while cursor.advance().await.unwrap_or(false) {
                if let Ok(word) = cursor.deserialize_current() {
                    words.push(word);
                }
            }

            if words.is_empty() {
                eprintln!("No words found. Run: mongosh english_words seed_words.js");
            }

            words
        })
    }
}

/// A fixed in-memory word set. Used for the benchmark harness and tests, and as
/// the built-in fallback deck.
pub struct StaticWordSource(pub Vec<Word>);

impl WordSource for StaticWordSource {
    fn load(&self) -> Vec<Word> {
        self.0.clone()
    }
}

/// Wraps a primary source and substitutes the built-in fallback deck whenever
/// the primary yields nothing (Decorator). Keeps the "Mongo, else fallback"
/// policy out of `main` and composable with any `WordSource`.
pub struct WithFallback<S: WordSource>(pub S);

impl<S: WordSource> WordSource for WithFallback<S> {
    fn load(&self) -> Vec<Word> {
        let words = self.0.load();
        if words.is_empty() {
            eprintln!(
                "Using built-in fallback word set ({} words).",
                FALLBACK.len()
            );
            return fallback_words();
        }
        words
    }
}
