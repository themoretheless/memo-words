# Memo Words

An ambient vocabulary overlay for the desktop. A small, transparent,
always-on-top, click-through card appears in a screen corner every ~30 seconds
showing an English word; its IPA transcription, Russian translation, and a short
example sentence then fade in on a timed sequence. It is meant to live quietly on
top of your other windows and teach you words passively while you work.

## How it works

- A frameless, transparent, click-through window covers the screen and paints a
  single card in one corner. It never steals focus or intercepts the mouse.
- Each word is shown, then its transcription fades in (default at 5s), its
  translation at 10s, and a short example sentence just after. The next word
  appears after the interval (default 30s, optionally jittered).
- Words are chosen weighted by frequency rank (common words appear more often),
  while a sliding window of recently shown words prevents short-term repeats.
- Many aspects are optional and default-preserving: card opacity and corner
  radius, an exit fade, an entrance settle, an accent rule, a faux-vibrancy
  sheen, an active-recall mode, spaced recap, and extra dwell for rarer words.
  All are off or unchanged unless you opt in (see the configuration table).
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
{
  "word": "time",
  "transcription": "/taɪm/",
  "translation": "время",
  "frequency": 70,
  "example": "What time is it?"
}
```

`frequency` is a rank: `1` is the most common word, higher numbers are rarer.
`example` is optional - documents without it simply show no example line.

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

All 16 keys are listed below. Every one defaults to the original behaviour, so an
absent key changes nothing. `config.example.conf` is a complete annotated example.

**Timing**

| Key                   | Default        | Meaning                                                        |
|-----------------------|----------------|----------------------------------------------------------------|
| `interval_secs`       | `30`           | Seconds each word stays up (1..86400).                         |
| `jitter_secs`         | `0`            | Random +/- variation added to the interval, so it isn't metronomic. |
| `transcription_delay` | `5.0`          | Seconds before the transcription fades in (clamped to >= 0).   |
| `translation_delay`   | `10.0`         | Seconds before the translation fades in (clamped to >= 0).     |
| `fade_duration`       | `1.0`          | Fade-in duration in seconds (clamped to >= 0.01).              |
| `rare_word_dwell`     | `0.0`          | Extra display time for rarer words (0.0..1.0); the interval is multiplied by up to `1 + value` as the frequency rank rises. `0` = off. |

**Appearance**

| Key             | Default        | Meaning                                                              |
|-----------------|----------------|----------------------------------------------------------------------|
| `corner`        | `bottom-right` | Card position: `top-left`, `top-right`, `bottom-left`, `bottom-right`. |
| `card_opacity`  | `0.30`         | Card background opacity (0.0 invisible .. 1.0 opaque).               |
| `corner_radius` | `16.0`         | Card corner radius in points (0.0..64.0).                           |
| `settle_px`     | `0.0`          | Points each line drifts up as it fades in (0.0..16.0). `0` = off.    |
| `accent_color`  | _(unset)_      | A thin accent rule under the headword, as bare hex `rrggbb` (no `#`, which starts a comment). Unset = no rule. |
| `sheen`         | `0.0`          | Strength of a faint top "lit material" highlight (0.0..1.0). `0` = off. |
| `exit_duration` | `0.0`          | Seconds the card fades out before the next word (0.0..10.0, capped to half the interval). `0` = hard cut. |

**Behaviour**

| Key            | Default | Meaning                                                              |
|----------------|---------|----------------------------------------------------------------------|
| `recall_mode`  | `false` | Hold the translation back to ~55% of the interval for active recall. `true`/`1`/`yes`/`on`. |
| `recap_chance` | `0.0`   | Probability (0.0..1.0) that a swap re-shows an earlier word for spaced review instead of a fresh one. |
| `speak`        | `false` | Speak each word aloud (macOS `say`). `true`/`1`/`yes`/`on` enable it. |

Example `config.conf`:

```ini
interval_secs = 20
jitter_secs = 5
corner = top-right
card_opacity = 0.45
exit_duration = 0.4
recall_mode = true
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

- The app targets macOS in practice: pronunciation (`speak`) uses the macOS `say`
  command (a no-op elsewhere), and the font is loaded from a macOS system path.
- Fonts: the app loads `Arial Unicode` from the standard macOS system font path
  for full Unicode (IPA transcription) coverage. If that font is missing it falls
  back to egui's default, which may not cover all IPA glyphs.
- Translations are Russian.

## Documentation

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) - module layout, layering, and the
  key invariants (zero idle cost, default-preserving config, the fixed card).
- [`docs/RECOMMENDATION.md`](docs/RECOMMENDATION.md) - the current top 200 audit
  of known weaknesses and prioritised improvements.
- [`docs/DESIGN_IDEAS.md`](docs/DESIGN_IDEAS.md) - the running design backlog.

Compatibility aliases are kept at the repository root for quick lookup:
[`architecture.md`](architecture.md) and [`recommendation.md`](recommendation.md).
The canonical documents live in `docs/`.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
