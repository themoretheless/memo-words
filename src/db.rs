use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Word {
    pub word: String,
    pub transcription: String,
    pub translation: String,
    #[serde(default)]
    pub frequency: i32,
}

/// Load words from MongoDB, falling back to a small built-in set whenever the
/// database is unavailable or empty. This keeps the overlay functional out of
/// the box, without requiring `brew services start mongodb-community` first.
pub fn load_words() -> Vec<Word> {
    let words = load_from_mongo();
    if words.is_empty() {
        eprintln!(
            "Using built-in fallback word set ({} words).",
            FALLBACK.len()
        );
        return fallback_words();
    }
    words
}

fn load_from_mongo() -> Vec<Word> {
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

fn fallback_words() -> Vec<Word> {
    FALLBACK
        .iter()
        .map(|&(word, transcription, translation, frequency)| Word {
            word: word.to_string(),
            transcription: transcription.to_string(),
            translation: translation.to_string(),
            frequency,
        })
        .collect()
}

// Curated common-word fallback used only when MongoDB has no data. The full
// deck still lives in MongoDB (see seed_words.js).
const FALLBACK: &[(&str, &str, &str, i32)] = &[
    ("the", "/ðə/", "определённый артикль", 1),
    ("be", "/biː/", "быть, являться", 2),
    ("to", "/tuː/", "к, в, до", 3),
    ("of", "/ɒv/", "из, от, о", 4),
    ("and", "/ænd/", "и, а", 5),
    ("a", "/eɪ/", "неопределённый артикль", 6),
    ("in", "/ɪn/", "в, внутри", 7),
    ("have", "/hæv/", "иметь, обладать", 9),
    ("it", "/ɪt/", "это, он/она/оно", 11),
    ("for", "/fɔːr/", "для, за, ради", 12),
    ("not", "/nɒt/", "не, нет", 13),
    ("on", "/ɒn/", "на, по", 14),
    ("with", "/wɪð/", "с, вместе с", 15),
    ("he", "/hiː/", "он", 16),
    ("as", "/æz/", "как, в качестве", 17),
    ("you", "/juː/", "ты, вы", 18),
    ("do", "/duː/", "делать", 19),
    ("at", "/æt/", "у, в, на", 20),
    ("this", "/ðɪs/", "этот, это", 21),
    ("but", "/bʌt/", "но, однако", 22),
    ("his", "/hɪz/", "его", 23),
    ("by", "/baɪ/", "посредством, у", 24),
    ("from", "/frɒm/", "из, от", 25),
    ("they", "/ðeɪ/", "они", 26),
    ("we", "/wiː/", "мы", 27),
    ("say", "/seɪ/", "говорить, сказать", 28),
    ("her", "/hɜːr/", "её, ей", 29),
    ("she", "/ʃiː/", "она", 30),
    ("will", "/wɪl/", "вспом. глагол будущего времени", 31),
    ("time", "/taɪm/", "время", 70),
    ("year", "/jɪər/", "год", 71),
    ("people", "/ˈpiːpl/", "люди", 72),
    ("way", "/weɪ/", "путь, способ", 73),
    ("day", "/deɪ/", "день", 74),
    ("man", "/mæn/", "мужчина, человек", 75),
    ("thing", "/θɪŋ/", "вещь, предмет", 76),
    ("woman", "/ˈwʊmən/", "женщина", 77),
    ("world", "/wɜːld/", "мир", 78),
    ("life", "/laɪf/", "жизнь", 79),
    ("hand", "/hænd/", "рука", 80),
];
