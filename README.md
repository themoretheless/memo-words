# Memo Words

An ambient vocabulary overlay for the desktop. A small, transparent,
always-on-top, click-through card appears in a screen corner every ~30 seconds
showing an English word; after a few seconds its transcription fades in, then
its translation. It is meant to live quietly on top of your other windows and
teach you words passively while you work.

## How it works

- A frameless, transparent, click-through window covers the screen and paints a
  single card in one corner. It never steals focus or intercepts the mouse.
- Each word is shown, then its transcription fades in (default at 5s) and its
  translation fades in (default at 10s). The next word appears after the
  interval (default 30s).
- Words are chosen weighted by frequency rank (common words appear more often),
  while a sliding window of recently shown words prevents short-term repeats.
- The overlay only repaints while a card is animating; once a card has fully
  settled it sleeps until the next word is due, so an idle overlay costs almost
  no CPU/GPU.

## Build and run

Requires a Rust toolchain (edition 2024).

```sh
cargo run --release
```

The app starts in the system tray with a small "W" icon.

## Words

On startup the app tries MongoDB at `mongodb://localhost:27017`, reading the
`words` collection of the `english_words` database. If MongoDB is unavailable
or the collection is empty, it falls back to a small built-in set of common
words, so it always works out of the box.

Each word document has the shape:

```json
{ "word": "time", "transcription": "/taɪm/", "translation": "время", "frequency": 70 }
```

`frequency` is a rank: `1` is the most common word, higher numbers are rarer.

To load the full deck into MongoDB:

```sh
brew services start mongodb-community
mongosh english_words seed_words.js
```

## Configuration

Optional config file. Lookup order:

1. the path in the `MEMO_CONFIG` environment variable, else
2. `$HOME/.config/memo-words/config.conf`

Format is simple `key = value` lines; `#` starts a comment. Unknown keys and
unparseable values are ignored, so a malformed file never stops the app from
starting. Every key defaults to the original behaviour, so no config file means
no change.

| Key                   | Default        | Meaning                                                        |
|-----------------------|----------------|----------------------------------------------------------------|
| `interval_secs`       | `30`           | Seconds each word stays up (min 1).                            |
| `jitter_secs`         | `0`            | Random +/- variation added to the interval, so it isn't metronomic. |
| `transcription_delay` | `5.0`          | Seconds before the transcription fades in.                     |
| `translation_delay`   | `10.0`         | Seconds before the translation fades in.                       |
| `fade_duration`       | `1.0`          | Fade-in duration in seconds (min 0.01).                        |
| `corner`              | `bottom-right` | Card position: `top-left`, `top-right`, `bottom-left`, `bottom-right`. |
| `speak`               | `false`        | Speak each word aloud (macOS `say`). `true`/`1`/`yes`/`on` enable it. |

Example `config.conf`:

```ini
interval_secs = 20
jitter_secs = 5
corner = top-right
speak = true
```

## Tray menu

- **Next word** - skip to a new word immediately.
- **Pause / Resume** - stop/restart advancing to new words.
- **Quit** - exit the app.

## Environment variables

- `MEMO_CONFIG` - path to a config file (overrides the default location).
- `MEMO_BENCH` - benchmark mode: shows one pinned card, counts rendered frames
  over a fixed idle window, prints `BENCH frames=N fps=N` to stderr, and exits.
  Used to measure idle repaint cost.

## Platform notes

- Word pronunciation (`speak`) uses the macOS `say` command and is a no-op on
  other platforms.
- Fonts: the app loads `Arial Unicode` from the standard macOS system font path
  for full Unicode (IPA transcription) coverage.
