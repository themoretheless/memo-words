# Recommendations: what is weak, wrong, or unfinished

A critical self-audit of memo-words, including the work done in the recent feature
+ refactor sprint. It was produced by an 8-lens adversarial pass (correctness,
architecture, refactor-debt, tests, deps/build, product/UX, docs/process,
data/content); each candidate was verified against the actual code before being
listed. The raw pass produced 79 confirmed findings; after de-duplication (the
lenses converged heavily - the stale README alone surfaced six times) this is the
ranked top ~50 of distinct issues.

Honesty notes:
- **Nothing here is truly "critical"** for a personal macOS overlay. Severity is
  calibrated for what this app is, not an enterprise service.
- "**Origin**" marks whether an issue was introduced or left during the recent
  sprint (`session`) or predates it (`pre-existing`).
- Two raw findings were rejected on verification: a claimed `random_range(0..0)`
  panic in `try_recap` (guarded by an `if pool == 0` early return, so it cannot
  fire), and a claim that the example line "breaks the recall illusion" (the
  example is delayed in lockstep with the translation, and it is English context,
  not the hidden Russian meaning - no early reveal).

## High

| # | Issue | Area | Origin | Fix |
|---|-------|------|--------|-----|
| 1 | **Startup panics.** `main.rs` uses `.expect()` on the tray-icon build and `.unwrap()` on three `menu.append()` calls; `tray.rs` `.expect()`s the icon; `source.rs` `.expect()`s `tokio::runtime::Runtime::new()`. Any failure crashes before the first frame, though a missing tray is merely cosmetic. | correctness | pre-existing | Degrade gracefully: tray/menu failures log and continue; runtime failure falls back to the built-in deck. |
| 2 | **README is stale.** It documents 7 of the 16 config keys and shows a Word schema without the `example` field, so ~56% of features are undiscoverable. | docs | session | Full config table + updated schema (done in this PR's README sync). |
| 3 | **No persistence layer.** Word progress, streaks, and mastery are lost on every restart, so real spaced-repetition (SM-2/Leitner) is impossible and `recall_mode`/`rare_word_dwell` cannot adapt to the learner. The fundamental product gap. | architecture | pre-existing | A `Store` trait + a small file/SQLite store, injected into `Deck`; SRS becomes a `WordSelector`. |
| 4 | **Info.plist version drift.** `Info.plist` declares `0.1.0` while `Cargo.toml` is `0.2.0`, so the bundled `.app` shows a stale version. (Verified.) | deps-build | session | Inject the Cargo version into `Info.plist` at bundle time in CI. |
| 5 | **No integration tests.** `tests/` is empty; app boot, the menu-watcher thread, the first frame, and the zero-idle invariant are never tested end-to-end - only unit logic is. | tests | pre-existing | Add an integration test booting `App` with `NullSpeaker`, plus a `MEMO_BENCH`-based idle-cost gate. |

## Medium

| # | Issue | Area | Origin | Fix |
|---|-------|------|--------|-----|
| 6 | **MongoDB + tokio + TLS/DNS in the default build** (~2MB, large compile + attack surface) for a tool that falls back to 40 words. | deps-build | pre-existing | Feature-gate `mongo` (default on); allow `--no-default-features` for a minimal build. |
| 7 | **Per-call tokio Runtime** in `MongoWordSource::load()` - fine once at startup, a landmine for any reload, and `.expect()`-panics. | correctness | pre-existing | A `once_cell` runtime or a graceful fallback. |
| 8 | **Multi-monitor / DPI fragility.** The window is created at a hard-coded 3840x2160 and resized once to a single monitor's size; wrong on non-primary, ultrawide, or hot-plugged displays, with no re-query. | correctness | session (left) | Query the active screen before creating the viewport; handle display changes. |
| 9 | **4K surface over-allocation.** That same 3840x2160 startup surface wastes VRAM on integrated GPUs before being shrunk. | deps-build | session | Size the viewport to the monitor up front. |
| 10 | **Hard-coded macOS font path** with silent fallback. If `Arial Unicode.ttf` is absent, IPA renders as tofu with no warning. | correctness | pre-existing | Warn on failure; try fallback fonts; consider embedding an IPA-capable font. |
| 11 | **Tiny decks repeat.** A custom deck of <3 words gets `recent_cap = 0`, silently disabling the no-repeat guard. | correctness | pre-existing | `(len/3).max(1).min(100)`. (The claimed `random_range` panic is a false alarm - guarded.) |
| 12 | **Platform abstraction left half-done.** The refactor created `platform.rs` for exactly this but only moved `Speaker`; the font path, `fill_screen`, and config path are still OS-specific code in `ui`/`app`/`config`. | refactor-debt | session | Finish: `platform::FontLoader`, `ScreenAdapter`, `config_path()`. |
| 13 | **Wrong config dir on macOS.** Config path hard-codes `$HOME/.config/...` (Linux XDG) rather than `~/Library/Application Support/...`. | architecture | session (left) | Platform-specific path via `platform.rs`. |
| 14 | **`accent_color` `#` trap.** Since `#` starts a comment, the natural `accent_color = #ff8800` is silently truncated and the accent stays off; users must write bare `ff8800`. | product-ux | pre-existing | Document prominently and/or special-case hex parsing. |
| 15 | **"Single source of truth" overclaim.** `timing.rs` says the timeline is defined once, but the card's width-transition easing (`WIDTH_TRANSITION = 0.5s`) still lives untested in `ui.rs` and can drift from `anim_end`. | refactor-debt | session | Move `WIDTH_TRANSITION` into `timing` (and test it) or document the exception. |
| 16 | **CHANGELOG omits the sprint** (non-finite guard, rare-word dwell, the 4-PR refactor). | docs | session | Add the entries. |
| 17 | **Docs drift.** `config.example.conf` documents 16 keys; the README documents 7 - they will keep diverging. | docs | session | Single-source the table (generate from `Config::default`). |
| 18 | **No architecture/module guide** and `DESIGN_IDEAS.md` sprawls (~200 lines). | docs | session | `ARCHITECTURE.md` (added in this PR). |
| 19 | **PR cadence without phase markers.** A flurry of tiny PRs with no grouping makes the history hard to read as an arc. | docs-process | session | Group related changes or annotate phases. |
| 20 | **Zero-idle invariant has no automated gate.** It is proven only by unit logic plus a manual `MEMO_BENCH` print; a regression that wakes the loop at 60fps would pass CI. | tests | session | Make `MEMO_BENCH` a CI gate (fail if idle fps exceeds a threshold). |
| 21 | **No first-frame test.** `fill_screen`, the menu-watcher spawn, the first `advance()`, and speaker routing on frame 1 are untested. | tests | session | A headless first-frame test. |
| 22 | **Fallback vs seed examples diverge** (~23 of 40 words show different example sentences than the same word from MongoDB). | data-content | session | Single-source the example data. |
| 23 | **Deck/seed mismatches.** `her` is in the 40-word fallback but missing from the 350-word seed; `country` and `possible` are duplicated in the seed. (Verified.) | data-content | pre-existing | Reconcile and de-duplicate. |
| 24 | **Inconsistent IPA.** Transcriptions mix British and American conventions across the deck. | data-content | pre-existing | Pick one standard or add a variant field. |
| 25 | **wgpu for a static card.** The wgpu backend pulls a heavier graphics stack than a flat card needs, with no compile-time choice of `glow`. | deps-build | pre-existing | Optional `glow` feature for minimal builds (note the hi-DPI caveat). |
| 26 | **Size-over-speed release profile.** `opt-level = "z"` + `strip` optimise binary size, but for an idle-cost-sensitive app `"s"`/`"3"` may render more cheaply. | deps-build | session | Re-evaluate `opt-level`; keep symbol stripping to CI artifacts. |

## Low / nit

| # | Issue | Area | Origin | Fix |
|---|-------|------|--------|-----|
| 27 | `timing.rs` is a ~440-line grab-bag of four concerns (easing, pacing, reveal, schedule). | refactor-debt | session | Optional feature sub-split if it grows. |
| 28 | `Config` is a 16-field flat bag; every consumer depends on the whole struct (no interface segregation). | architecture | session | Sub-configs (`TimingConfig`/`AppearanceConfig`/...). |
| 29 | `Theme` is loose consts, not a validated struct; no light mode despite the groundwork. | refactor-debt | session | A `Theme` struct with `dark()`/`light()` presets. |
| 30 | Doc comments say delays are "Floored at 0.0" but the code clamps with `.max` (i.e. "clamped to >= 0.0"). | refactor-debt | session | Correct the wording. |
| 31 | The menu-watcher thread is never joined or cancelled; it is abandoned mid-`recv` on quit. | tests | pre-existing | A shutdown signal or a `Drop`-based stop. |
| 32 | `try_recap`'s pool math is "accidentally correct but fragile" (relies on `random_range` exclusivity); intent is not obvious. | correctness | session | Clarify with a comment or `0..=pool-1`. |
| 33 | No `LICENSE` file; `Cargo.toml` lacks `authors`/`license`/`repository` metadata. | docs-process | pre-existing | Add them. |
| 34 | No onboarding/first-run guidance; loading the full deck needs a manual `mongosh` seed. | product-ux | pre-existing | A first-run note / bundled data. |
| 35 | The `Word` model lacks part-of-speech / sense tags, so ambiguous words (`lead`, `bow`) are unqualified. | data-content | pre-existing | Optional POS/sense fields. |
| 36 | Example truncation (64 chars + ellipsis) cuts mid-clause silently. | data-content | session (left) | Truncate at a word/clause boundary. |
| 37 | `seed_words.js` has no schema validation or documentation; malformed entries fail silently at load. | data-content | pre-existing | A validator / documented schema. |
| 38 | **Click-through caps the learning model** at passive exposure - no grading or reveal-on-demand is possible in-card. Deliberate, but a real ceiling. | product-ux | by design | Accept, or add a tray-driven grade/reveal surface. |
| 39 | The cosmetic sprint features (accent rule, sheen) add near-zero learning value; they shipped because the design loop asked for visual polish. | product-ux | session | Keep them opt-in; prioritise learning features. |
| 40 | The autonomous design-loop risks churn/padding - many default-off, visually-unverifiable knobs. | docs-process | session | Keep curating; resist padding to a counter. |
| 41 | CI builds but does not bundle/verify the `.app` or run anything in `tests/`. | deps-build | pre-existing | Add bundle + `cargo test --test '*'` steps. |
| 42 | The exit fade and line fades are never render-verified (no snapshot/mock-painter test). | tests | session | A mock-painter test asserting alpha decays to 0 at the swap. |
| 43 | The 40-word fallback gives only a ~12-word no-repeat window, so a long offline session repeats noticeably. | data-content | pre-existing | A larger bundled deck, or accept. |
| 44 | Config is read once at startup; there is no live reload on file change. | architecture | pre-existing | Optional file-watch reload, or document the restart requirement. |
| 45 | The full 350-word deck exists only as a `mongosh` JS seed; there is no portable bundled data file, so the rich deck requires MongoDB + a manual step. | data-content | pre-existing | Bundle the full deck as embedded data (ties to #3/#6). |
| 46 | Frequency ranks are sparse/uneven (1-31 then jumping to 70-80 in the fallback), so weighting may be lumpier than intended. | data-content | pre-existing | Verify/normalise the rank distribution. |
| 47 | `MEMO_BENCH` is a manual measurement, not a stored baseline, so perf trends are invisible. | tests | session | Emit machine-readable output and track it. |
| 48 | The `speak` flag's gate moved to `main` in the refactor; a reader of `app.rs`/`config.rs` no longer sees where it is consumed. | refactor-debt | session | A one-line doc pointer. |
| 49 | macOS-only in practice (the `say` speaker, the font path, the implicit `~/.config` path), with no stated cross-platform position. | product-ux | pre-existing | State the platform scope explicitly; gate per-OS bits. |
| 50 | README "Build and run" does not mention the heavy default deps or the Mongo-vs-fallback trade-off a user is opting into. | docs | session | Note it (partly addressed in the README sync). |

## Suggested priority order

If only a handful are tackled, do these first - they are the highest value per unit
effort and several are quick:

1. **#4 Info.plist version** (one-line CI fix, verified-wrong today).
2. **#1 startup panics** (small, removes the only crash-on-boot paths).
3. **#2 / #16 / #17 docs sync** (cheap, and #2 is largely done in this PR).
4. **#11 tiny-deck window** (one-character fix, removes a real repeat).
5. **#3 persistence layer** (the one big *capability* unlock; everything that makes
   this a real learning tool depends on it).
6. **#6 feature-gate MongoDB** (halves the default binary; the trait seam already
   exists, so it is mostly wiring).
