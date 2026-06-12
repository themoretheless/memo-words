# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
