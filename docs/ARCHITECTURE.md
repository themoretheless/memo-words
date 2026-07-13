# Architecture

`memo-words` is a small macOS ambient vocabulary overlay built with Rust 2024,
eframe/egui, wgpu, muda, and tray-icon. This document is the canonical map of
the code after the July 2026 SOLID/DRY passes. `README.md` is the user guide;
`docs/RECOMMENDATION.md` is the canonical 500-item audit.

## Product boundary

The app paints one calm, transparent, always-on-top card in a screen corner.
The overlay is click-through and must not steal focus. It reveals a word, IPA,
translation, and example over time, then sleeps until the next event. The two
strongest constraints are therefore:

1. **Calm ambient behavior:** no persistent controls or dashboard chrome on the
   card itself.
2. **Zero idle work:** a settled card requests no animation frames until the
   next reveal/exit/menu event.

The tray is the active control surface. Future grading/preferences belong there
or in a companion window, not inside the click-through card.

## Dependency direction

Dependencies point from composition/adapters toward pure policy. Core timing,
deck, selection, and session state know nothing about egui, MongoDB, tray-icon,
or macOS subprocesses.

```text
main.rs (composition root)
  |-- fallback.rs ----------------> model.rs
  |-- source.rs facade
  |     |-- source/controller.rs -> source/report.rs
  |     |-- source/report.rs -----> model.rs
  |     |-- source/static_source.rs -> model.rs
  |     `-- source/mongo.rs ------> model.rs + mongodb
  |-- loading.rs -----------------> source.rs
  |-- diagnostics.rs -------------> config.rs + source/controller.rs
  |-- deck.rs --------------------> model.rs + selector.rs
  |-- platform.rs (Speaker)
  |-- tray.rs (icon + source menu view)
  `-- app.rs (eframe adapter)
       |-- deck.rs
       |-- source.rs (SourceController)
       |-- diagnostics.rs / tray.rs
       |-- session.rs
       |-- wake.rs (owned long-deadline worker)
       |-- timing.rs facade
       |     |-- timing/easing.rs
       |     |-- timing/pacing.rs
       |     |-- timing/timeline.rs
       |     `-- timing/repaint.rs
       |-- theme.rs
       `-- ui.rs facade
             |-- ui/foundation.rs
             |-- ui/text.rs
             |-- ui/surface.rs
             `-- ui/card.rs

config.rs facade
  |-- config/parser.rs
  |-- config/path.rs
  `-- config/tests.rs
```

## Read the project in small pieces

This order follows data and behavior from the smallest pure types outward. Each
step can be understood without reading the later ones.

| Step | File | Question it answers |
|---:|---|---|
| 1 | `src/model.rs` | What is a word record today? |
| 2 | `src/selector.rs` | How is a candidate chosen? |
| 3 | `src/deck.rs` | How are repeats and recaps controlled? |
| 4 | `src/session.rs` | What does start/pause/resume mean for time? |
| 5 | `src/timing/easing.rs` | How do fade/settle curves behave? |
| 6 | `src/timing/pacing.rs` | How does rarity change dwell time? |
| 7 | `src/timing/timeline.rs` | When do lines reveal and exit? |
| 8 | `src/timing/repaint.rs` | When should the UI request another frame? |
| 9 | `src/wake.rs` | How does a long idle deadline wake eframe exactly once? |
| 10 | `src/config.rs` | Which four configuration contracts exist? |
| 11 | `src/config/parser.rs` | How are 20 keys validated and clamped? |
| 12 | `src/config/path.rs` | Where is configuration found and read? |
| 13 | `src/theme.rs` | Which semantic tokens define the card? |
| 14 | `src/ui/foundation.rs` | How are transparent visuals and fonts registered? |
| 15 | `src/ui/text.rs` | How are lines measured, fitted, and painted? |
| 16 | `src/ui/surface.rs` | How is the optional surface sheen drawn? |
| 17 | `src/ui/card.rs` | How are content, timeline, and style composed? |
| 18 | `src/source/report.rs` | How are source outcomes, issues, and fallback represented? |
| 19 | `src/source/static_source.rs` | What is the deterministic in-memory adapter? |
| 20 | `src/source/mongo.rs` | How are Mongo rows mapped into a typed report? |
| 21 | `src/fallback.rs` | Which records guarantee offline startup? |
| 22 | `src/source.rs` | How are source contracts and fallback policy composed? |
| 23 | `src/loading.rs` | How does synchronous source work leave the UI thread? |
| 24 | `src/source/controller.rs` | How are attempts, health, retry, and active/pending decks owned? |
| 25 | `src/diagnostics.rs` | Which redacted support facts can be copied? |
| 26 | `src/platform.rs` | How is speech isolated from the app? |
| 27 | `src/tray.rs` | How are source state and commands presented compactly? |
| 28 | `src/app.rs` | How are source handoff, commands, timing, deck, and render joined? |
| 29 | `src/main.rs` | Which concrete adapters are wired at startup? |

## Module responsibilities

### Pure domain and policy

| Module | One responsibility |
|---|---|
| `model.rs` | Serializable `Word` data. |
| `selector.rs` | `WordSelector` strategy and frequency-weighted implementation. |
| `deck.rs` | Current word, recent-window invariants, and optional recap rotation. |
| `session.rs` | Presentation-clock state; pause freezes reveal and interval clocks. |
| `timing/easing.rs` | Stateless visual interpolation functions. |
| `timing/pacing.rs` | Difficulty and dwell-duration calculation. |
| `timing/timeline.rs` | Reveal/animation/exit boundaries. |
| `timing/repaint.rs` | Pure next-repaint decision and zero-idle guard. |

`timing.rs` is only a facade. Callers depend on stable timing concepts without
knowing the implementation file layout.

### Configuration

`Config` is split by consumer need instead of exposing one bag of unrelated
fields:

| Group | Owns |
|---|---|
| `TimingConfig` | interval, jitter, reveal/fade/exit timing, rare-word dwell |
| `AppearanceConfig` | corner, opacity, radius, settle, accent, sheen, theme |
| `LearningConfig` | speech, recall mode, recap chance |
| `AccessibilityConfig` | font scale, enhanced contrast, reduced motion |

`config/parser.rs` is pure. It parses and clamps values, rejects non-finite
floats, accepts both `rrggbb` and `#rrggbb`, and leaves the previous/default
value intact when input is invalid. `config/path.rs` is the small filesystem/env
adapter. The current macOS path remains a known issue in audit item #282.

### Design and rendering

`theme.rs` is the semantic design system, not a list of arbitrary literals. A
`Theme` contains:

- a surface palette and minimum readable opacity;
- semantic word/translation/transcription/example colors;
- an answer-first type scale;
- coordinated geometry tokens for margin, width, height, and accent;
- shadow and sheen tokens.

The presets are `graphite` (default-preserving), `midnight`, `paper`, and
`high-contrast`. Font scaling changes typography, geometry, and card height as
one system. Enhanced contrast raises surface/border/text visibility. Reduced
motion disables positional settle, exit fade, and width morphing while retaining
the gentle opacity reveal.

The UI facade divides work by reason to change:

| Module | One responsibility |
|---|---|
| `ui/foundation.rs` | One-time transparent visuals and font registration. |
| `ui/text.rs` | Text truncation, measurement, fitting, and centered painting. |
| `ui/surface.rs` | Optional material sheen mesh. |
| `ui/card.rs` | Card geometry and composition from content/timeline/style. |

`CardView` receives three explicit contracts:

- `CardContent`: the four user-facing strings;
- `CardTimeline`: elapsed time, reveal delays, fade, motion preference;
- `CardStyle`: placement, surface controls, accent, and semantic theme.

This avoids a 20-field flat render interface and makes future layout tests
smaller.

### Adapters and composition

| Module | One responsibility today |
|---|---|
| `source/report.rs` | Typed outcome, issue, active-source, count, and fallback metadata. |
| `source/static_source.rs` | Deterministic in-memory source for tests and benchmarks. |
| `source/mongo.rs` | Mongo query/cursor/decode mapping and bounded diagnostics. |
| `source/controller.rs` | One active attempt, retained report, health, retry, and deck activation state. |
| `source.rs` | Stable `WordSource` facade and fallback decorator. |
| `fallback.rs` | Built-in offline records. |
| `loading.rs` | Timed background source execution and completion notification. |
| `diagnostics.rs` | Redacted support report without raw backend error messages. |
| `platform.rs` | `Speaker` port with macOS and null adapters. |
| `tray.rs` | Procedural icon plus compact source status/reload/diagnostics controls. |
| `wake.rs` | Owned/cancellable worker for long repaint deadlines. |
| `app.rs` | eframe lifecycle, menu polling, deck advance, render orchestration. |
| `main.rs` | Choose concrete source/speaker, construct tray/window, run eframe. |

The 500-item audit intentionally calls out the remaining adapter debt: source
attempts have manual retry but no cancellation/backoff/progress, platform
path/font/display logic is still scattered, `App` still sees raw muda IDs and
creates RNG/time directly, diagnostics cover only selected config/capabilities,
and tray/worker shutdown lifecycles are not owned explicitly.

## Runtime flow

### Startup

1. `main` loads the grouped config.
2. Normal mode immediately builds `Deck` from fallback words; benchmark mode
   uses one deterministic static record and never starts Mongo.
3. It creates tray/menu IDs and the full-screen transparent viewport.
4. Inside eframe setup it gives `SourceController` a launcher that runs
   `WithFallback<MongoWordSource>` through the timed `loading` worker.
5. It selects `SystemSpeaker` or `NullSpeaker` and constructs `App`.
6. The first `App` advance renders fallback without waiting for remote I/O.
7. Worker completion requests one repaint. `SourceController` retains the
   report and health while `App` queues usable remote words or keeps the active
   deck on failure.
8. The tray shows a disabled status row and enables Reload only when no attempt
   or pending handoff is active.
9. The next normal timer/menu advance installs queued words, then controller
   marks that source active. The visible card never changes midway.

### Each frame

1. `App` polls `SourceController` after its initial fallback advance.
2. It consumes menu events, including safe reload and redacted diagnostics copy.
3. `SessionClock` decides whether the current word is due to advance.
4. `timing` derives reveal delays, exit window, and repaint sleep.
5. `Theme` and config create `CardContent`, `CardTimeline`, and `CardStyle`.
6. `CardView` measures once for width, paints surface/content, and returns.
7. `timing::repaint_after` chooses the delay; 16ms animation stays in egui,
   while `WakeScheduler` owns long idle/exit/pause deadlines independently of
   eframe pass invalidation.

### Pause invariant

`SessionClock` stores `paused_at`. While paused, `elapsed()` and `until_next()`
use that frozen instant. On resume, both underlying anchors shift by the pause
duration. Therefore reveal state and next-word timing continue from the exact
active-time position. `repaint_after` checks pause before animation so a frozen
mid-fade frame cannot request 60 FPS forever.

## SOLID/DRY map

- **Single responsibility:** timing, session state, config path/parsing, text,
  surface, and card composition live in separate modules.
- **Open/closed:** word selection and speech use ports; new strategies/adapters
  should not modify deck/application policy.
- **Liskov substitution:** `NullSpeaker` and `StaticWordSource` let tests and
  benchmark mode replace OS/remote behavior without changing callers.
- **Source boundary:** `WordSource` returns one typed `LoadReport`; `loading`
  times background execution, `WithFallback` handles availability policy, and
  `SourceController` owns runtime lifecycle without knowing Mongo or egui.
- **Interface segregation:** config is grouped by timing/appearance/learning/
  accessibility; rendering receives content/timeline/style contracts.
- **Dependency inversion:** `App` depends on `Speaker`; `Deck` depends on
  `WordSelector`; composition chooses concrete implementations.
- **DRY timing:** scheduler and renderer share the `timing` facade.
- **DRY design:** semantic theme tokens are the only card palette/type/geometry
  source.
- **DRY pause:** `SessionClock` is the only presentation-clock authority.
- **DRY recommendations:** all 500 audit rows live only in
  `docs/RECOMMENDATION.md`.

Known DRY violations remain: fallback and seed content are duplicated; config
metadata is repeated across parser/example/README; source/platform errors use
scattered `eprintln!`. These are tracked as #76-80, #363-364, and #338/#476.

## Verification

The current suite has 77 unit tests covering config groups/parser,
deck/selector/replacement, session pause semantics, timing modules, themes,
text helpers, timed source reports, retry/activation state, redaction, tray
labels, and the background handoff. The standard local gate is:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

There are still no end-to-end app, visual snapshot, controlled Mongo adapter,
bundle smoke, or CI performance tests. Audit items #376-400 define that missing
layer.

## Audit relationship

The canonical audit has exactly 500 rows across 20 areas, 25 rows per area. This
architecture document explains ownership and dependency boundaries; it does not
copy those rows. Start with the "Best next 20 moves" at the end of
[`RECOMMENDATION.md`](RECOMMENDATION.md), then follow item numbers back to the
relevant modules above.
