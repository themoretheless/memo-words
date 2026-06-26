//! The built-in fallback deck: a curated common-word set used only when the
//! primary source (MongoDB) has no data, so the overlay always has words to show
//! offline. The full deck lives in MongoDB (see `seed_words.js`); this is the
//! minimal embedded set. Kept in its own module so the data table doesn't clutter
//! the source logic. Fields: word, IPA, translation, frequency rank, example.

use crate::model::Word;

/// Build owned `Word`s from the embedded `FALLBACK` table.
pub fn fallback_words() -> Vec<Word> {
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

/// Curated common-word fallback used only when MongoDB has no data. Fields:
/// word, IPA, translation, frequency rank, short example sentence.
pub const FALLBACK: &[(&str, &str, &str, i32, &str)] = &[
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
