# Recommendations: top 50 things done poorly or incorrectly

This is the current top 50 audit for `memo-words`, synchronized with
`README.md`, `docs/ARCHITECTURE.md`, and the root compatibility aliases
`architecture.md` and `recommendation.md`. It replaces the previous top-200
version: re-verified line by line against the code at this commit rather than
carried forward, trimmed to the 50 sharpest problems, and folding in newly
found defects. The broader backlog of not-yet-done ideas and suggestions
lives in [`docs/BACKLOG.md`](BACKLOG.md); this file is specifically about
things that are already built but weak, wrong, or misleading.

Scope checked against `main` on 2026-07-01. Ranked by product risk, learning
value, correctness, maintainability, and release confidence.

Severity key:

- **P0** - can crash, block startup, corrupt the core product promise, or hides a
  major user-facing capability gap.
- **P1** - high-leverage architecture, UX, data, or release issue.
- **P2** - important maintainability, quality, accessibility, or documentation
  debt.

## Fixed since the 2026-06-29 audit

Verified against current source; kept here for traceability rather than
silently dropped.

| Was # | Fix |
|-------|-----|
| 1 | Startup no longer panics on tray-menu append or tray-icon build failure; both log and continue without a tray (`main.rs`, `tray.rs::create_icon` now returns `Option`). |
| 2 | `MongoWordSource::load()` no longer panics if `tokio::runtime::Runtime::new()` fails; it logs and returns an empty vec so `WithFallback` substitutes the built-in deck. |
| 10 | `Info.plist` version synced to `0.2.0` (was `0.1.0`, Cargo.toml was already `0.2.0`). |
| 65 | Tiny decks (2 words) no longer repeat the same word back-to-back: `recent_cap` is now at least 1 for any 2+ word deck. |

## Top 50

| # | Priority | Area | Problem | Recommended fix |
|---|----------|------|---------|-----------------|
| 1 | P0 | product | There is no persisted learning state, so progress, mastery, streaks, and review history vanish on restart. | Add a small `Store` trait with file or SQLite persistence before adding deeper SRS. |
| 2 | P0 | product | The app has recap, but not real spaced repetition because no due dates or grades survive sessions. | Model per-word exposure, ease, due time, and last result, then route selection through it. |
| 3 | P0 | startup | Word loading is synchronous before the first frame (`main.rs` calls `source.load()` before `eframe::run_native`), so MongoDB/TLS/DNS can delay the overlay appearing. | Show the fallback immediately or load the primary source asynchronously with a status transition. |
| 4 | P0 | tests | There are no end-to-end tests for boot, first frame, tray wiring, and quit behavior; no `tests/` integration directory exists. | Add a headless integration harness around `App` and a smoke launch for the bundle. |
| 5 | P0 | performance | The zero-idle promise is not enforced in CI; `ci.yml` never runs `MEMO_BENCH=1`. | Add a benchmark gate that fails if settled FPS exceeds a tiny threshold. |
| 6 | P0 | display | The initial viewport is still hard-coded to `[3840.0, 2160.0]` in `main.rs` before the real monitor size is known. | Query the monitor before creating the viewport, or create a small hidden window first. |
| 7 | P0 | display | Screen size is only read once, in `App::fill_screen`, gated by `!self.started`; hot-plug, resolution, scaling, and monitor changes are never re-applied. | Centralize a screen adapter and re-query on viewport or monitor events. |
| 8 | P1 | correctness | Pausing can leave the card invisible instead of frozen: `until_next` (and therefore `exit_alpha`) is derived from `last_show.elapsed()`, which keeps growing while paused. If `exit_duration` is configured and the app stays paused longer than one `word_interval`, `timing::exit_alpha` evaluates to `0.0`; any repaint that fires during that pause (screen wake, Space switch, window expose) paints the card fully transparent, not paused-and-visible, until Resume. | While `self.paused`, freeze `until_next`/`exit_alpha` at their pre-pause value, or short-circuit to `exit_alpha = 1.0`, so "paused" always means "frozen," never "faded out." |
| 9 | P1 | release | There is no `LICENSE` file at the repo root and `Cargo.toml` has no `license`/`authors`/`repository` metadata, even though `.github/workflows/release.yml` publishes public GitHub Releases of the built `.app`. | Add a license file and matching Cargo.toml metadata before the next tagged release. |
| 10 | P2 | release | `deny.toml` waives the license check with the comment "a personal, unpublished app," but the project ships public GitHub Releases via `softprops/action-gh-release` - the stated justification no longer matches what the repo actually does. | Either add a real license allow-list now that the app is publicly distributed, or rewrite the comment to state the actual accepted risk honestly. |
| 11 | P1 | build | MongoDB, Tokio, TLS, and DNS are in the default build even when the user only needs the fallback deck. | Feature-gate MongoDB and provide a small default/offline build. |
| 12 | P1 | config | MongoDB URI, database, and collection are hard-coded in `MongoWordSource::default()` with no config keys to override them. | Add config/env keys for source selection and Mongo connection fields. |
| 13 | P1 | data | The full deck exists only as a `mongosh` JS seed file, not as a portable bundled data asset. | Store the deck as JSON/TOML/CSV and generate both the Mongo seed and the fallback from it. |
| 14 | P1 | data | `seed_words.js` still duplicates `country` (lines 169 and 205, ranks 159 vs 195) and `possible` (lines 236 and 313, ranks 225 vs 302) with different ranks. | Add a seed validator that rejects duplicate words before insert. |
| 15 | P1 | data | The fallback and seed decks disagree for shared words: e.g. `time` is rank 54 in the seed vs rank 70 in the fallback with a different example sentence, and `people`'s fallback translation drops a synonym present in the seed. | Single-source common records and generate the fallback from the same canonical data. |
| 16 | P1 | data | The fallback deck has only 40 words, so offline use repeats quickly. | Embed a larger curated offline deck or ship the full deck as an asset. |
| 17 | P1 | data | Fallback frequency ranks jump from 31 (`will`) straight to 70 (`time`), creating uneven weighting semantics. | Normalize fallback ranks or keep exact canonical ranks with documented intent. |
| 18 | P1 | model | `Word` has no stable ID, schema version, or source metadata. | Add `id`, `schema_version`, and optional `source` fields with migration rules. |
| 19 | P1 | model | `Word` cannot represent multiple senses, translations, or examples. | Introduce optional sense records while preserving the simple flat import path. |
| 20 | P1 | model | There is no part-of-speech or sense label, so ambiguous words are under-specified. | Add optional `part_of_speech` and `sense` fields and render them quietly. |
| 21 | P1 | source | Bad MongoDB documents are silently dropped: `if let Ok(word) = cursor.deserialize_current() { words.push(word) }` discards the `Err` case with no count or log. | Count and report skipped records with the field-level error. |
| 22 | P1 | source | `cursor.advance().await.unwrap_or(false)` treats a real driver error identically to a clean EOF. | Distinguish EOF from driver error and log an actionable failure. |
| 23 | P1 | source | Startup does not report loaded word count, fallback count, or skipped count in a structured way. | Add a `LoadReport` returned by `WordSource`. |
| 24 | P1 | source | `WordSource::load()` is synchronous, forcing the Mongo backend to build its own Tokio runtime and block on it. | Split into sync static sources and async primary sources, or load on a worker. |
| 25 | P1 | source | Mongo `server_selection_timeout`/`connect_timeout` are hard-coded to 2 seconds. | Make the timeout configurable and document the startup trade-off. |
| 26 | P1 | platform | Config path resolution (`config.rs::read_config_file`) uses `$HOME/.config/memo-words/config.conf`, not the macOS-correct `~/Library/Application Support`. | Move config-path resolution into `platform.rs` with macOS-correct defaults. |
| 27 | P1 | platform | `platform.rs` only owns TTS; screen sizing, config path, and font loading remain scattered across `main.rs`, `app.rs`, and `ui.rs`. | Expand platform ports for screen, paths, fonts, and system settings. |
| 28 | P1 | platform | `ui::load_fonts` probes exactly one hard-coded path, `/System/Library/Fonts/Supplemental/Arial Unicode.ttf`. | Probe multiple IPA-capable fonts, warn on failure, or bundle a small font. |
| 29 | P1 | platform | When that font is missing, `load_fonts` silently falls through to egui's default font, so IPA can render as tofu with zero explanation. | Emit a warning and expose a font status in diagnostics. |
| 30 | P1 | platform | `SystemSpeaker::speak` is a silent no-op on non-macOS targets even when `speak = true` in config. | Log once that speech is unsupported on the current target. |
| 31 | P1 | display | The card is placed with a flat `SCREEN_MARGIN` from the raw monitor edges; it does not account for the Dock, menu bar, or notch. | Place against the usable screen frame and add configurable margins. |
| 32 | P1 | display | Users cannot choose which monitor receives the overlay; `fill_screen` always uses `viewport().monitor_size`, i.e. whichever monitor eframe picked. | Add monitor selection by index/name with safe fallback. |
| 33 | P1 | ux | The overlay is click-through by design, so there is no in-card reveal, grading, mute, or skip affordance. | Keep click-through as identity, but add tray or companion controls for active actions. |
| 34 | P1 | ux | The tray "Pause / Resume" `MenuItem` label is a static string; it is never rewritten to reflect the actual paused state. | Reflect paused state, source status, and current word in the tray menu. |
| 35 | P1 | ux | There is no preferences UI for non-technical users; every setting requires hand-editing `config.conf`. | Add a small tray-opened settings window that writes the config file. |
| 36 | P1 | ux | Config is only read once at startup (`Config::load()` in `main`); changing the file requires a full app restart. | Add a file watcher or a tray "Reload config" command. |
| 37 | P1 | ux | Unknown or malformed config keys are silently ignored with no feedback path. | Keep default-preserving behavior but surface warnings in stderr and diagnostics. |
| 38 | P1 | ux | `accent_color = #ff8800` looks natural but is parsed as a comment (the `#` truncates the line first), silently leaving the accent off. This is exercised by a test, so it is documented behavior, not an oversight - but it is still a real footgun for anyone editing the file by hand. | Accept quoted/hash colors, or warn specifically when the value is empty after stripping `#`. |
| 39 | P1 | ux | There is no way to generate a default config file; users must copy `config.example.conf` by hand. | Add "write sample config" from CLI or tray. |
| 40 | P1 | a11y | The app does not check or honor macOS Reduce Motion; `settle_px` and `exit_duration` animate regardless of the system setting. | Disable settle and exit motion when the system flag is enabled. |
| 41 | P1 | a11y | There is no font-scale setting; `WORD_FONT_SIZE`/`TRANSLATION_FONT_SIZE`/etc. in `theme.rs` are fixed constants. | Add `font_scale` with bounds and layout tests. |
| 42 | P1 | a11y | There is no enhanced contrast mode. | Add `enhanced_contrast` that raises opacity/text intensity while preserving hierarchy. |
| 43 | P1 | a11y | Card contrast is not measured against light or busy wallpapers; the fixed dark tint (30,30,30) has no light-wallpaper counterpart. | Add sampled contrast heuristics or high-contrast presets. |
| 44 | P1 | a11y | Accent colors are user-supplied hex with no contrast check against the card fill. | Warn or auto-adjust accents that disappear on the card fill. |
| 45 | P1 | a11y | `WIDGET_HEIGHT` is a fixed 160px constant, which limits larger fonts and longer translated strings. | Make height responsive within a max, or introduce compact/comfortable density modes. |
| 46 | P1 | render | `truncate_example` truncates by `char` count (`EXAMPLE_MAX_CHARS = 64`), not by measured pixel width, so the same character budget looks very different across fonts and scripts. | Truncate by measured width and cut at word boundaries. |
| 47 | P1 | render | The headword, IPA, and translation lines have no overflow policy at all (`centered_text` never wraps or truncates them), unlike the example line. | Add measured shrink/truncate/wrap rules per line. |
| 48 | P1 | render | There are no visual snapshots for the card, fade states, corners, or examples; correctness relies entirely on arithmetic unit tests. | Render fixed test states and compare screenshots or pixel bounds. |
| 49 | P1 | timing | Resuming after a long pause shows an old, fully-revealed card: `shown_at` is never reset on pause/resume, only `last_show` is, so `elapsed` (and thus every fade) is already long past `anim_end` the instant you resume. | Decide semantics and either freeze `shown_at` while paused or document "pause rotation only." |
| 50 | P1 | timing | `MEMO_BENCH` pins `shown_at` to a fixed `Instant::now() - Duration::from_secs(20)` and reports `n as u64 / BENCH_SECS` (integer division), which hides fractional idle-FPS regressions and can measure an unsettled card if config delays push `anim_end` past 20s. | Pin to `anim_end + margin` based on the active config, and print floating-point FPS. |

## Best next 10 moves

1. Freeze `exit_alpha`/`until_next` while paused (#8) - small, high-confidence correctness fix.
2. Add `LICENSE` + Cargo.toml metadata, and reconcile `deny.toml`'s stale rationale (#9, #10) - cheap, removes real legal ambiguity around the public release pipeline.
3. Fix startup panics' remaining sibling: make word loading non-blocking or fallback-first (#3).
4. Add the canonical deck validator and remove the `country`/`possible` duplicate seed rows (#14).
5. Add a persisted store primitive for exposure and review state (#1, #2).
6. Feature-gate MongoDB and ship an offline-first build path (#11).
7. Move config path, screen sizing, and font loading into platform adapters (#26, #27, #28).
8. Add CI gates for `MEMO_BENCH`, bundle contents, and Markdown links (#5).
9. Add a settings/reload path so config changes do not require a restart (#36).
10. Add accessibility knobs: font scale, reduced motion, and enhanced contrast (#40, #41, #42).
