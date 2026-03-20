use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Word {
    pub word: String,
    pub transcription: String,
    pub translation: String,
    #[serde(default)]
    pub frequency: i32,
}

pub fn load_words() -> Vec<Word> {
    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    rt.block_on(async {
        let client = match mongodb::Client::with_uri_str("mongodb://localhost:27017").await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("MongoDB connection failed: {e}");
                eprintln!("Start MongoDB: brew services start mongodb-community");
                return Vec::new();
            }
        };

        let collection = client.database("english_words").collection::<Word>("words");

        let mut cursor = match collection.find(mongodb::bson::doc! {}).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to query words: {e}");
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
