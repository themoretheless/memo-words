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
| `card_opacity`        | `0.3`          | Card background opacity, from `0.0` (invisible) to `1.0` (opaque). |
| `corner_radius`       | `16`           | Card corner radius in points (max 64).                         |
| `screen_margin`       | `40`           | Card distance from the selected screen edges, in points.        |
| `recall_mode`         | `false`        | Hold the translation back until late in the interval for active recall. |
| `pace_indicator`      | `false`        | Draw a faint progress underline for the current word interval. |
| `accent_color`        | `#E6E6E6`      | Subtle accent for example emphasis and the pace indicator: hex, `plain`, `mint`, `sky`, or `rose`. |
| `session_counter`     | `false`        | Draw a tiny count of words shown in the current session. |
| `high_contrast`       | `false`        | Strengthen card material, border, and secondary text alphas. |
| `familiarity_dimming` | `false`        | Slightly dim words already seen in the current session. |
| `focus_hours`         | `off`          | Comma-separated local quiet windows like `09:00-17:00,22:00-01:00`; rotation pauses inside them. |
| `daily_rollup`        | `false`        | Append shown words to a dated local Markdown rollup. |
| `quiet_review_weighting` | `false`     | Gently boost copied/recently revisited words in future rotation. |
| `placement_preview`   | `false`        | Draw temporary safe-area guides for corner/margin tuning. |
| `preferences_open`    | `false`        | Open the small preferences window on launch. |
| `session_frozen`      | `false`        | Start with the current word frozen while tray copy/export actions stay available. |
| `cooldown_words`      | `0`            | Override the recent-word cooldown window; `0` keeps the automatic deck-sized window. |

Example `config.conf`:

```ini
interval_secs = 20
jitter_secs = 5
corner = top-right
screen_margin = 56
speak = true
pace_indicator = true
accent_color = mint
session_counter = true
high_contrast = true
familiarity_dimming = true
focus_hours = 09:00-17:00
daily_rollup = true
quiet_review_weighting = true
cooldown_words = 25
```

## Tray menu

- **Next word** - skip to a new word immediately.
- **Speak current word** - pronounce the visible word once.
- **Copy current word** - copy the visible word, IPA, translation, and example.
- **Copy recent words** - copy the five most recently shown words as Markdown.
- **Copy session summary** - copy a Markdown recap of words shown this session.
- **Copy session as Anki TSV** - copy tab-separated `word`, `translation`,
  `transcription`, and `example` columns for flashcard import.
- **Copy deck health report** - copy a Markdown data-quality report grouped by
  errors, warnings, and info.
- **Reset session** - clear the current session history, count, and familiarity
  dimming state without restarting the app.
- **Recall mode** - checked when the delayed-translation reveal is enabled for
  the current session; changing it restarts the current card's reveal.
- **Pace indicator** - checked when the faint progress underline is visible for
  the current session.
- **Placement preview** - checked when safe-area/corner guides are visible for
  tuning layout.
- **Preferences** - opens a small settings window and temporarily makes the
  overlay clickable; closing it returns the overlay to click-through mode.
- **Freeze current word** - stops automatic rotation on the visible word while
  keeping copy/export/speak actions available.
- **Paused** - checked when automatic word rotation is stopped.
- **Quit** - exit the app.

## Environment variables

- `MEMO_CONFIG` - path to a config file (overrides the default location).
- `MEMO_ROLLUP` - path for `daily_rollup` output; defaults to
  `$HOME/.config/memo-words/daily-rollup.md`.
- `MEMO_DRY_RUN` - validate the loaded deck, print the Markdown health report,
  and exit without opening the overlay. Equivalent CLI flags:
  `--dry-run` or `--validate-deck`.
- `MEMO_BENCH` - benchmark mode: shows one pinned card, counts rendered frames
  over a fixed idle window, prints `BENCH frames=N fps=N active_repaints=N` to
  stderr, and exits. `active_repaints` counts app-requested animation/progress
  repaints, separating them from platform-driven window frames.

## Platform notes

- Word pronunciation (`speak`) uses the macOS `say` command and is a no-op on
  other platforms.
- Fonts: the app loads `Arial Unicode` from the standard macOS system font path
  for full Unicode (IPA transcription) coverage.
