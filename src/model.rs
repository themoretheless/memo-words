//! The core word data model. Pure data with no I/O, database, or UI dependency,
//! so every other module (deck, selector, sources, rendering) depends only on
//! this small struct rather than on each other.

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Word {
    pub word: String,
    pub transcription: String,
    pub translation: String,
    #[serde(default)]
    pub frequency: i32,
    /// A short usage example. Optional: MongoDB documents without the field
    /// (and older data) deserialize to an empty string and simply show no
    /// example line.
    #[serde(default)]
    pub example: String,
}
