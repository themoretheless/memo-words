# Architecture

memo-words is a small macOS ambient overlay built on Rust 2024 + eframe (egui
0.34, wgpu). This document describes how the code is organised after the modular
refactor (PRs #41-#44, with startup-hardening fixes since). It is the map;
`README.md` is the user guide and `RECOMMENDATION.md` is the top 50
known-issues / improvement list. Root files `architecture.md` and
`recommendation.md` are compatibility aliases that point back to the canonical
docs.

## The product in one paragraph

A transparent, always-on-top, **click-through** window paints a single small card
in a screen corner. Every ~30s it shows one English word; its IPA transcription,
Russian translation, and a short example sentence then fade in on a timed
sequence. It is meant to teach passively while you work, so it must stay calm and
cost **zero idle CPU/GPU** when nothing is animating.

## Layering

The code is split into a pure domain core (no egui, no OS), a render shell (egui),
OS/IO adapters, and a thin composition root. Dependencies point inward: the core
knows nothing about egui, the database, or the OS.

```
                       main.rs  (composition root: wire concretes, run eframe)
                          |
                  app.rs  (eframe adapter: loop, scheduling, menu, bench)
                 /     |          \              \
        ui.rs+theme.rs |        platform.rs       deck.rs
        (render shell) |        (Speaker port)     |
                       |                           selector.rs
                    timing.rs  <----- pure domain core ----->  model.rs
                 (choreography +                               source.rs / fallback.rs
                  pacing + schedule)                           (word loading)
                       ^
                       |
                    config.rs  (key=value file -> Config)
```

### Pure domain core (no egui, no OS - fully unit-testable)

| Module | Responsibility |
|--------|----------------|
| `model.rs` | The `Word` data type (serde). Importable without tokio/mongodb. |
| `deck.rs` | Word rotation: a recent-window (no short-term repeats) plus an interleaved spaced **recap**, behind a pluggable selector. Pure, no UI/IO. |
| `selector.rs` | `WordSelector` trait + `FrequencyWeighted` (1/rank; unknown rank = rarest). |
| `timing.rs` | All choreography and pacing math: easing (`smoothstep`, `fade_factor`, `settle_offset`), pacing (`difficulty_factor`, `dwelled_base_secs`), the reveal timeline (`effective_translation_delay`, `example_delay`, `anim_end`, `exit_window`), and the repaint decision (`repaint_after`). The single source of truth for *when* things happen. |

`timing.rs` is consumed by **both** the scheduler (`app`) and the renderer (`ui`),
so the timeline is defined once and is testable without a window.

### Word sources (adapters over the model)

| Module | Responsibility |
|--------|----------------|
| `source.rs` | `WordSource` trait + `MongoWordSource`, `StaticWordSource`, and the `WithFallback` decorator (Mongo, else the built-in deck). |
| `fallback.rs` | The 40-word built-in deck used when MongoDB has no data, so the app always works offline. |

### Render shell (egui)

| Module | Responsibility |
|--------|----------------|
| `theme.rs` | The card's visual identity: tint/border/shadow/sheen colours, the type-size scale, per-line brightness intensities, the colour helpers (`card_bg`, `faded_line`, `dim`), and a **compile-time hierarchy guard**. |
| `ui.rs` | `CardView`: layout geometry, text measurement/truncation, and the egui painter. Consumes `timing` easing and `theme` values. |

### Adapters and root

| Module | Responsibility |
|--------|----------------|
| `platform.rs` | OS ports. Today a `Speaker` (TTS) with `SystemSpeaker` (macOS `say`) and `NullSpeaker`. Intended home for monitor sizing and config paths next. |
| `tray.rs` | The procedural tray icon. |
| `config.rs` | The `Config` struct + a zero-dependency, default-preserving `key=value` parser. |
| `app.rs` | The `eframe::App` adapter: the per-frame loop, repaint scheduling (applying `timing::repaint_after`), tray-menu wiring + watcher thread, the `MEMO_BENCH` harness, and the RNG jitter. |
| `main.rs` | Composition root: load config, choose the `WordSource` and `Speaker`, build the window, run eframe. |

## Key invariants

- **Zero idle cost.** A fully-settled, unpaused card requests no frames. The
  decision lives in the pure `timing::repaint_after` (animate while fading in or
  during the exit fade; otherwise sleep until the next event), and is unit-tested
  directly. `MEMO_BENCH=1` measures it empirically.
- **Default-preserving config.** Every key defaults to the original behaviour;
  unknown keys and unparseable/non-finite values are ignored, so a malformed file
  never changes a field or stops startup.
- **Answer-first type hierarchy.** Headword > translation > transcription >
  example in both size and brightness, locked by a compile-time assertion in
  `theme.rs` (the build fails if the ordering regresses).
- **Fixed card.** The card is a fixed ~160px tall; content is laid out to fit and
  long examples are truncated to one line.
- **Click-through.** The window is mouse-passthrough: there is no in-card input.
  The only interaction surfaces are the tray menu and the config file. This is a
  deliberate identity constraint and it caps the learning model at passive
  exposure (see `RECOMMENDATION.md`).

## Data flow per frame

1. `app::ui()` polls the tray-menu channel and, if the interval elapsed, calls
   `advance()`.
2. `advance()` rotates the `Deck`, rolls the next interval
   (`timing::dwelled_base_secs` + jitter), and routes the word to the `Speaker`.
3. The current `Word` is rendered by `CardView` using `timing` eases and `theme`
   colours.
4. `app` asks `timing::repaint_after(...)` how long to sleep and calls
   `ctx.request_repaint_after(...)`. A settled card sleeps until the next word.

## Testing

Each core module carries unit tests (66 total). The pure core (`timing`, `deck`,
`selector`, `model`) is fully testable with no egui or OS. Visual output is not
pixel-verified in CI; geometry and choreography are proven by arithmetic, unit
tests, and the compile-time hierarchy guard instead. There are currently **no**
integration/end-to-end tests (see `RECOMMENDATION.md`).
