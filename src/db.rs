use serde::Deserialize;
use std::time::Duration;

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

fn fallback_words() -> Vec<Word> {
    FALLBACK
        .iter()
        .map(
            |&(word, transcription, translation, frequency, example)| Word {
                word: word.to_string(),
                transcription: transcription.to_string(),
                translation: translation.to_string(),
                frequency,
                example: example.to_string(),
            },
        )
        .collect()
}

// Curated common-word fallback used only when MongoDB has no data. The full
// deck still lives in MongoDB (see seed_words.js). Fields: word, IPA,
// translation, frequency rank, short example sentence.
const FALLBACK: &[(&str, &str, &str, i32, &str)] = &[
    (
        "the",
        "/ðə/",
        "определённый артикль",
        1,
        "Close the door, please.",
    ),
    (
        "be",
        "/biː/",
        "быть, являться",
        2,
        "She wants to be a doctor.",
    ),
    ("to", "/tuː/", "к, в, до", 3, "I'm going to the store."),
    ("of", "/ɒv/", "из, от, о", 4, "A cup of coffee, please."),
    ("and", "/ænd/", "и, а", 5, "Bread and butter."),
    (
        "a",
        "/eɪ/",
        "неопределённый артикль",
        6,
        "I saw a cat outside.",
    ),
    ("in", "/ɪn/", "в, внутри", 7, "The keys are in my bag."),
    (
        "have",
        "/hæv/",
        "иметь, обладать",
        9,
        "Do you have a minute?",
    ),
    ("it", "/ɪt/", "это, он/она/оно", 11, "It is raining today."),
    (
        "for",
        "/fɔːr/",
        "для, за, ради",
        12,
        "This gift is for you.",
    ),
    ("not", "/nɒt/", "не, нет", 13, "She is not at home."),
    ("on", "/ɒn/", "на, по", 14, "The book is on the table."),
    ("with", "/wɪð/", "с, вместе с", 15, "Come with me."),
    ("he", "/hiː/", "он", 16, "He plays the guitar."),
    ("as", "/æz/", "как, в качестве", 17, "She works as a nurse."),
    ("you", "/juː/", "ты, вы", 18, "Can you help me?"),
    ("do", "/duː/", "делать", 19, "Do your homework now."),
    ("at", "/æt/", "у, в, на", 20, "We met at the station."),
    ("this", "/ðɪs/", "этот, это", 21, "This is my house."),
    ("but", "/bʌt/", "но, однако", 22, "It's small but cozy."),
    ("his", "/hɪz/", "его", 23, "That is his car."),
    (
        "by",
        "/baɪ/",
        "посредством, у",
        24,
        "We travelled by train.",
    ),
    ("from", "/frɒm/", "из, от", 25, "She is from Spain."),
    ("they", "/ðeɪ/", "они", 26, "They live next door."),
    ("we", "/wiː/", "мы", 27, "We are good friends."),
    ("say", "/seɪ/", "говорить, сказать", 28, "What did you say?"),
    ("her", "/hɜːr/", "её, ей", 29, "I gave her the book."),
    ("she", "/ʃiː/", "она", 30, "She speaks three languages."),
    (
        "will",
        "/wɪl/",
        "вспом. глагол будущего времени",
        31,
        "I will call you tomorrow.",
    ),
    ("time", "/taɪm/", "время", 70, "What time is it?"),
    ("year", "/jɪər/", "год", 71, "See you next year."),
    (
        "people",
        "/ˈpiːpl/",
        "люди",
        72,
        "Many people came to the party.",
    ),
    ("way", "/weɪ/", "путь, способ", 73, "This is the right way."),
    ("day", "/deɪ/", "день", 74, "Have a nice day!"),
    (
        "man",
        "/mæn/",
        "мужчина, человек",
        75,
        "The man is reading.",
    ),
    (
        "thing",
        "/θɪŋ/",
        "вещь, предмет",
        76,
        "One more thing, please.",
    ),
    ("woman", "/ˈwʊmən/", "женщина", 77, "The woman is a pilot."),
    ("world", "/wɜːld/", "мир", 78, "Travel around the world."),
    ("life", "/laɪf/", "жизнь", 79, "Life is beautiful."),
    ("hand", "/hænd/", "рука", 80, "Raise your hand to answer."),
];
