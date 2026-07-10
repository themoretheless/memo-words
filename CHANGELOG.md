# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Example sentence line on the card, with the deck and fallback carrying an
  optional `example` field.
- Many opt-in, default-preserving config keys: `card_opacity`, `corner_radius`,
  `recall_mode` (hold the translation back for active recall), `exit_duration`
  (exit fade), `recap_chance` (interleaved spaced recap), `settle_px` (entrance
  settle), `accent_color` (accent rule), `sheen` (faux-vibrancy), and
  `rare_word_dwell` (longer dwell for rarer words).
- Soft drop shadow + hairline border on the card, and an answer-first type
  hierarchy (translation outranks the IPA), guarded at compile time.
- Coordinated `graphite`, `midnight`, `paper`, and `high-contrast` themes plus
  `font_scale`, `enhanced_contrast`, and `reduce_motion` accessibility controls.

### Changed
- Decomposed the code into cohesive, loosely-coupled modules: `model`, `source`,
  `fallback`, a pure `timing` (choreography/pacing/scheduling shared by the
  scheduler and renderer), `theme`, and `platform`, leaving `app` a thin eframe
  adapter. The repaint-scheduling decision is now a pure, unit-tested function.
- Text-to-speech is behind a `Speaker` port chosen at the composition root.
- Split configuration by timing, appearance, learning, and accessibility;
  decomposed timing and card rendering into narrow modules, and moved pause
  semantics into a tested session clock.

### Fixed
- Reject non-finite (`nan`/`inf`) config values so they can't poison a field
  (e.g. an infinite delay that would pin the app at 60fps).
- Start up gracefully: a tray, menu, or tokio-runtime failure now logs and
  continues (or falls back to the built-in deck) instead of panicking.
- Tiny decks (2 words) no longer repeat the same word back-to-back.
- `Info.plist` version synced to 0.2.0 (was 0.1.0).
- Pause now freezes both reveal and interval clocks without idle animation;
  natural `accent_color = #rrggbb` values are accepted.

### Docs
- Added `docs/ARCHITECTURE.md` and `docs/RECOMMENDATION.md` (a critical audit),
  and synced the README config table (all keys) and Word schema.

## [0.2.0]

### Added
- Frequency-weighted word selection: common words (lower rank) surface more
  often, rarer ones still appear, via a sliding recent-window that avoids
  short-term repeats.
- Configuration file (`~/.config/memo-words/config.conf`, or `MEMO_CONFIG`) for
  interval, jitter, fade timings, corner, and speech.
- Random jitter around the word interval so the cadence isn't metronomic.
- Optional spoken pronunciation of each word (macOS `say`).
- Tray menu: "Next word" and "Pause / Resume" in addition to "Quit".
- Built-in fallback word set so the overlay works without MongoDB.

### Changed
- Stop continuous repainting when the card is idle: repaint at ~60fps only
  while the card fades in, then sleep until the next word. Large CPU/GPU/battery
  reduction on an ambient overlay (~1100 -> ~100 frames per 10s idle window).
- Compute the card width once per frame instead of twice.
- Keep the text galley cache key stable across fade frames to avoid per-frame
  re-layout.
- Track the recent-word window with a `HashSet` for O(1) membership.
- Refactor into decoupled modules: a `WordSource` trait (Mongo / static /
  fallback), a `WordSelector` strategy, and a `Deck` that owns word rotation,
  leaving `App` a thin eframe adapter.

### Dependencies
- eframe 0.31 -> 0.34, rand 0.8 -> 0.10, tray-icon 0.19 -> 0.24,
  muda 0.15 -> 0.19, plus a refreshed lockfile.

### Tooling
- CI workflow running `cargo fmt --check`, `cargo clippy --all-targets
  -D warnings`, and `cargo test`.
- Unit tests for config parsing, deck rotation, and word selection.

## [0.1.1]

- Initial tracked release: ambient always-on-top overlay that shows a word from
  MongoDB every 30s, fading in transcription then translation, with a tray icon.
