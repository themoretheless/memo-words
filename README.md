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
- Appearance is a coordinated system rather than isolated decoration: four
  theme presets, proportional text/card scaling, enhanced contrast, reduced
  motion, opacity/radius, accent, and sheen. Graphite at scale 1.0 preserves the
  original defaults.
- Active recall, spaced recap, speech, jitter, and extra dwell for rarer words
  are optional and default-preserving.
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

The first card always comes from a small built-in set, so opening the window
never waits for MongoDB. In the background the app tries
`mongodb://localhost:27017`, reading the `words` collection of the
`english_words` database. A usable remote deck is queued and takes over on the
next normal word change, avoiding an abrupt mid-card swap. Partially readable
data uses its valid rows and reports skipped records. If MongoDB is unavailable
or yields no valid words, the built-in deck remains active. The latest typed
report, attempt timing, effective source, and word counts remain available to
the tray diagnostics. A failed reload never replaces a working deck.

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

All 20 keys are listed below. Every key has a safe default; `graphite`,
`font_scale = 1.0`, and disabled accessibility overrides preserve the original
look and behavior. `config.example.conf` is the complete annotated example.

**Timing**

| Key                   | Default        | Meaning                                                        |
|-----------------------|----------------|----------------------------------------------------------------|
| `interval_secs`       | `30`           | Seconds each word stays up (1..86400).                         |
| `jitter_secs`         | `0`            | Random +/- variation added to the interval, so it isn't metronomic. |
| `transcription_delay` | `5.0`          | Seconds before the transcription fades in (clamped to >= 0).   |
| `translation_delay`   | `10.0`         | Seconds before the translation fades in (clamped to >= 0).     |
| `fade_duration`       | `1.0`          | Fade-in duration in seconds (clamped to >= 0.01).              |
| `exit_duration`       | `0.0`          | Seconds the card fades out before a swap (0.0..10.0, capped to half the interval). |
| `rare_word_dwell`     | `0.0`          | Extra display time for rarer words (0.0..1.0); the interval is multiplied by up to `1 + value` as the frequency rank rises. `0` = off. |

**Appearance**

| Key             | Default        | Meaning                                                              |
|-----------------|----------------|----------------------------------------------------------------------|
| `corner`        | `bottom-right` | Card position: `top-left`, `top-right`, `bottom-left`, `bottom-right`. |
| `card_opacity`  | `0.30`         | Requested background opacity (0.0..1.0); readable presets may apply a floor. |
| `corner_radius` | `16.0`         | Card corner radius in points (0.0..64.0).                            |
| `settle_px`     | `0.0`          | Points each line drifts up as it fades in (0.0..16.0). `0` = off.    |
| `accent_color`  | _(unset)_      | Accent rule as `rrggbb` or `#rrggbb`. Unset = no rule.               |
| `sheen`         | `0.0`          | Strength of a faint top material highlight (0.0..1.0). `0` = off.    |
| `theme`         | `graphite`     | Coordinated palette: `graphite`, `midnight`, `paper`, `high-contrast`. |

**Learning**

| Key            | Default | Meaning                                                              |
|----------------|---------|----------------------------------------------------------------------|
| `recall_mode`  | `false` | Hold the translation back to ~55% of the interval for active recall. `true`/`1`/`yes`/`on`. |
| `recap_chance` | `0.0`   | Probability (0.0..1.0) that a swap re-shows an earlier word for spaced review instead of a fresh one. |
| `speak`        | `false` | Speak each word aloud (macOS `say`). `true`/`1`/`yes`/`on` enable it. |

**Accessibility**

| Key                 | Default | Meaning                                                             |
|---------------------|---------|---------------------------------------------------------------------|
| `font_scale`        | `1.0`   | Scale text, spacing, width limits, and card height together (0.8..1.5). |
| `enhanced_contrast` | `false` | Raise surface, border, and secondary-text contrast while preserving hierarchy. |
| `reduce_motion`     | `false` | Disable settle, exit fade, and width morphing; opacity reveal remains. |

Example `config.conf`:

```ini
interval_secs = 20
jitter_secs = 5
corner = top-right
card_opacity = 0.45
exit_duration = 0.4
theme = midnight
font_scale = 1.15
enhanced_contrast = true
recall_mode = true
speak = true
```

## Tray menu

- **Source: ...** - a quiet, disabled status row showing loading, pending,
  active, degraded, or failed source state and the effective word count.
- **Next word** - skip to a new word immediately.
- **Pause** / **Resume** - the label always names the next action and changes as
  soon as the presentation clock is paused or resumed.
- **Reload words** / **Retry source** - refresh MongoDB without restarting; the
  recovery verb appears after a failed/fallback result. The action is disabled
  while an attempt or safe deck handoff is already in progress.
- **Copy diagnostics** - copy a redacted app/runtime/config/source report. Raw
  backend error messages are deliberately excluded.
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

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) - SOLID/DRY module map,
  dependency direction, runtime flow, and a 30-step small-piece reading order.
- [`docs/RECOMMENDATION.md`](docs/RECOMMENDATION.md) - the canonical 500-item
  register: 20 product/engineering areas with 25 findings and actions each.
- [`docs/DESIGN_IDEAS.md`](docs/DESIGN_IDEAS.md) - the running design backlog.

Compatibility aliases are kept at the repository root for quick lookup:
[`architecture.md`](architecture.md) and [`recommendation.md`](recommendation.md).
The canonical documents live in `docs/`.

For a quick code tour, start with `model.rs`, `selector.rs`, `deck.rs`, and
`session.rs`; then read the small timing, config, UI, source, loading,
diagnostics, command, and tray modules before opening `app.rs` and `main.rs`. The
architecture document explains what question each file answers.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
