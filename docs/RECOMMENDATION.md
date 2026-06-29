# Recommendations: top 200 things still weak, wrong, or missing

This is the current top 200 audit for `memo-words`, synchronized with
`README.md`, `docs/ARCHITECTURE.md`, and the root compatibility aliases
`architecture.md` and `recommendation.md`.

Scope checked against `main` on 2026-06-29. The app is already small and
pleasant; this list is intentionally stricter than "is it usable today?" and is
ranked by product risk, learning value, correctness, maintainability, and release
confidence.

Severity key:

- **P0** - can crash, block startup, corrupt the core product promise, or hides a
  major user-facing capability gap.
- **P1** - high-leverage architecture, UX, data, or release issue.
- **P2** - important maintainability, quality, accessibility, or documentation
  debt.
- **P3** - polish, process, future-proofing, or lower-risk cleanup.

## Top 200

| # | Priority | Area | Problem | Recommended fix |
|---|----------|------|---------|-----------------|
| 1 | P0 | correctness | Startup can panic on tray icon creation, tray menu append, and icon RGBA conversion. | Return recoverable errors, log context, and run without tray when needed. |
| 2 | P0 | correctness | `MongoWordSource::load()` panics if `tokio::runtime::Runtime::new()` fails. | Convert runtime creation into a fallible branch and fall back to the built-in deck. |
| 3 | P0 | product | There is no persisted learning state, so progress, mastery, streaks, and review history vanish on restart. | Add a small `Store` trait with file or SQLite persistence before adding deeper SRS. |
| 4 | P0 | product | The app has recap, but not real spaced repetition because no due dates or grades survive sessions. | Model per-word exposure, ease, due time, and last result, then route selection through it. |
| 5 | P0 | startup | Word loading is synchronous before the first frame, so MongoDB/TLS/DNS can delay the overlay appearing. | Show the fallback immediately or load the primary source asynchronously with a status transition. |
| 6 | P0 | tests | There are no end-to-end tests for boot, first frame, tray wiring, and quit behavior. | Add a headless integration harness around `App` and a smoke launch for the bundle. |
| 7 | P0 | performance | The zero-idle promise is not enforced in CI; `MEMO_BENCH` is manual. | Add a benchmark gate that fails if settled FPS exceeds a tiny threshold. |
| 8 | P0 | display | The initial viewport is hard-coded to 3840x2160 before being resized once. | Query the monitor before creating the viewport, or create a small hidden window first. |
| 9 | P0 | display | Screen size is only re-read on first frame; hot-plug, resolution, scaling, and monitor changes are ignored. | Centralize a screen adapter and re-query on viewport or monitor events. |
| 10 | P0 | release | `Info.plist` still says `0.1.0` while `Cargo.toml` is `0.2.0`. | Generate or patch plist versions from Cargo metadata in CI. |
| 11 | P1 | build | MongoDB, Tokio, TLS, and DNS are in the default build even when the user only needs the fallback deck. | Feature-gate MongoDB and provide a small default/offline build. |
| 12 | P1 | config | MongoDB URI, database, and collection are hard-coded. | Add config/env keys for source selection and Mongo connection fields. |
| 13 | P1 | data | The full deck exists only as a `mongosh` JS seed file, not as a portable bundled data asset. | Store the deck as JSON/TOML/CSV and generate both Mongo seed and fallback from it. |
| 14 | P1 | data | `seed_words.js` duplicates `country` and `possible` with different ranks. | Add a seed validator that rejects duplicate words before insert. |
| 15 | P1 | data | The fallback and seed decks use different examples and sometimes different translations for the same word. | Single-source common records and generate fallback from the same canonical data. |
| 16 | P1 | data | The fallback deck has only 40 words, so offline use repeats quickly. | Embed a larger curated offline deck or ship the full deck as an asset. |
| 17 | P1 | data | Frequency ranks in fallback jump from the low 30s to 70-80, creating uneven weighting semantics. | Normalize fallback ranks or keep exact canonical ranks with documented intent. |
| 18 | P1 | model | `Word` has no stable ID, schema version, or source metadata. | Add `id`, `schema_version`, and optional `source` fields with migration rules. |
| 19 | P1 | model | `Word` cannot represent multiple senses, translations, or examples. | Introduce optional sense records while preserving the simple flat import path. |
| 20 | P1 | model | There is no part-of-speech or sense label, so ambiguous words are under-specified. | Add optional `part_of_speech` and `sense` fields and render them quietly. |
| 21 | P1 | source | Bad MongoDB documents are silently skipped during cursor deserialization. | Count and report skipped records with the field-level error. |
| 22 | P1 | source | Cursor advance errors are treated as "done" via `unwrap_or(false)`. | Distinguish EOF from driver error and log an actionable failure. |
| 23 | P1 | source | Startup does not report loaded word count, fallback count, or skipped count in a structured way. | Add a `LoadReport` returned by `WordSource`. |
| 24 | P1 | source | `WordSource::load()` is synchronous, forcing async backends to block internally. | Split into sync static sources and async primary sources, or load on a worker. |
| 25 | P1 | source | Mongo timeout is hard-coded to 2 seconds. | Make timeout configurable and document the startup trade-off. |
| 26 | P1 | platform | macOS config path uses `$HOME/.config`, not `~/Library/Application Support`. | Move config-path resolution into `platform.rs` with macOS-correct defaults. |
| 27 | P1 | platform | `platform.rs` only owns TTS; screen sizing, config path, and font loading remain scattered. | Expand platform ports for screen, paths, fonts, and system settings. |
| 28 | P1 | platform | Font loading uses one hard-coded macOS font path. | Probe multiple IPA-capable fonts, warn on failure, or bundle a small font. |
| 29 | P1 | platform | Missing font fallback is silent, so IPA can render as tofu without explanation. | Emit a warning and expose a font status in diagnostics. |
| 30 | P1 | platform | Non-macOS speech is a silent no-op even when `speak = true`. | Log once that speech is unsupported on the current target. |
| 31 | P1 | display | The card does not account for Dock, menu bar, notch, or usable screen frame. | Place against usable frame and add configurable margins. |
| 32 | P1 | display | Users cannot choose which monitor receives the overlay. | Add monitor selection by index/name with safe fallback. |
| 33 | P1 | display | Retina scale and unusual DPI are not visually regression-tested. | Add screenshot checks at common scale factors. |
| 34 | P1 | ux | The overlay is click-through, so there is no in-card reveal, grading, mute, or skip affordance. | Keep click-through as identity, but add tray or companion controls for active actions. |
| 35 | P1 | ux | Tray menu does not show current state beyond a static "Pause / Resume" label. | Reflect paused state, source status, and current word in the tray menu. |
| 36 | P1 | ux | There is no preferences UI for non-technical users. | Add a small tray-opened settings window that writes the config file. |
| 37 | P1 | ux | Config reload requires app restart. | Add a file watcher or a tray "Reload config" command. |
| 38 | P1 | ux | Unknown or malformed config keys are silently ignored. | Keep default-preserving behavior but surface warnings in stderr and diagnostics. |
| 39 | P1 | ux | `accent_color = #ff8800` looks natural but is parsed as a comment and turns accent off. | Accept quoted/hash colors or warn specifically when the value is empty after `#`. |
| 40 | P1 | ux | There is no default config file generation. | Add "write sample config" from CLI or tray. |
| 41 | P1 | a11y | The app does not honor macOS Reduce Motion. | Disable settle and exit motion when the system flag is enabled. |
| 42 | P1 | a11y | There is no font-scale setting. | Add `font_scale` with bounds and layout tests. |
| 43 | P1 | a11y | There is no enhanced contrast mode. | Add `enhanced_contrast` that raises opacity/text intensity while preserving hierarchy. |
| 44 | P1 | a11y | Card contrast is not measured against light or busy wallpapers. | Add sampled contrast heuristics or high-contrast presets. |
| 45 | P1 | a11y | Accent colors are not contrast-checked. | Warn or auto-adjust accents that disappear on the card fill. |
| 46 | P1 | a11y | The fixed 160px card height limits larger fonts and translated strings. | Make height responsive within a max, or introduce compact/comfortable density modes. |
| 47 | P1 | render | Example truncation is character-count based, not pixel/word aware. | Truncate by measured width and cut at word boundaries. |
| 48 | P1 | render | Headword, IPA, and translation do not have robust overflow policies. | Add measured shrink/truncate/wrap rules per line. |
| 49 | P1 | render | Long unbroken strings can still force ugly max-width behavior. | Add per-line elision after width measurement. |
| 50 | P1 | render | There are no visual snapshots for the card, fade states, corners, or examples. | Render fixed test states and compare screenshots or pixel bounds. |
| 51 | P1 | timing | Pause does not freeze the reveal timeline; elapsed time keeps moving. | Decide semantics and either freeze `shown_at` or document "pause rotation only". |
| 52 | P1 | timing | Resuming after a long pause shows an old fully-revealed card for a full fresh interval. | Optionally advance on resume or show a subtle paused state. |
| 53 | P1 | timing | `MEMO_BENCH` pins elapsed to 20s, which may not be settled for unusual config delays. | Pin to `anim_end + margin` based on the active config. |
| 54 | P1 | timing | `MEMO_BENCH` reports integer FPS, hiding fractional idle regressions. | Print frames, seconds, and floating FPS. |
| 55 | P1 | timing | Jitter is random and not injectable, making interval tests statistical. | Inject an RNG or deterministic jitter source into `App`. |
| 56 | P1 | timing | Rare-word dwell uses frequency rank as a proxy for learning difficulty. | Replace with learned difficulty after persistence exists. |
| 57 | P1 | timing | The difficulty factor saturates quickly, so very rare ranks cluster near max dwell. | Use a smoother transform such as log-rank buckets. |
| 58 | P1 | timing | Interval math rounds to whole seconds only. | Decide if sub-second config should be supported or explicitly rejected. |
| 59 | P1 | timing | Jitter can dominate short intervals. | Cap jitter to a fraction of the current interval. |
| 60 | P1 | timing | Exit fade, recall delay, and very short intervals have limited combinational tests. | Add property tests around timing invariants. |
| 61 | P1 | deck | `Deck` has no durable exposure count, last-seen time, or mastery. | Add a state-backed selector input. |
| 62 | P1 | deck | `WordSelector::choose(&self, ...)` prevents stateful selectors from mutating cleanly. | Pass a mutable selector or separate scoring from state updates. |
| 63 | P1 | deck | Recap chooses uniformly from older recent words, not from due or weak words. | Score recap candidates by due time, difficulty, and last result. |
| 64 | P1 | deck | Recap chance is global, not adaptive. | Tune recap probability by session length, misses, and deck size. |
| 65 | P1 | deck | Tiny decks with fewer than 3 words disable the recent window entirely. | Use a minimal repeat guard when deck size permits. |
| 66 | P1 | deck | There is no way to snooze, bury, or ignore a word. | Add user-level word state once persistence lands. |
| 67 | P1 | deck | There is no favorites or "difficult words" list. | Add tags backed by the same store. |
| 68 | P1 | deck | There is no themed or topical deck rotation. | Add optional tags and selector filters. |
| 69 | P1 | deck | There is no daily seen limit or decay window. | Add recent exposure decay across hours/days. |
| 70 | P1 | deck | The selector cannot explain why a word was chosen. | Return selection metadata for diagnostics and tests. |
| 71 | P1 | release | The app bundle is not signed or notarized. | Add signing/notarization steps for release builds. |
| 72 | P1 | release | Release ZIP has no checksum. | Publish SHA-256 alongside the artifact. |
| 73 | P1 | release | Release workflow does not validate the built app's plist version. | Add a plist check before creating the release. |
| 74 | P1 | release | CI builds the bundle but does not launch-smoke it. | Run a minimal startup smoke with `MEMO_BENCH=1`. |
| 75 | P1 | release | There is no release checklist tying Cargo version, changelog, plist, tag, and artifact. | Add a documented release checklist or script. |
| 76 | P1 | deps | There is no `rust-toolchain.toml`, so CI and local stable can drift. | Pin a stable toolchain intentionally. |
| 77 | P1 | deps | CI does not test a no-Mongo/minimal feature set because no such feature exists. | Add feature matrix after feature-gating Mongo. |
| 78 | P1 | deps | `deny.toml` skips license enforcement. | Either add an allow-list or document this as an accepted personal-app risk. |
| 79 | P1 | deps | Multiple transitive versions are allowed without review. | Periodically audit duplicates or document accepted duplicates. |
| 80 | P1 | deps | There is no dependency update cadence. | Add Dependabot/Renovate or a manual monthly update note. |
| 81 | P2 | tests | Unit tests cover core math, but there is no `tests/` integration suite. | Add integration tests for config/source/deck/app composition. |
| 82 | P2 | tests | Selector tests rely on random sampling and loose thresholds. | Use seeded RNG or test weight functions deterministically. |
| 83 | P2 | tests | There is no test for Mongo fallback load reports. | Mock a failing source and assert fallback/report behavior. |
| 84 | P2 | tests | There is no test for skipped malformed Mongo records. | Add deserialization fixtures and expected skip counts. |
| 85 | P2 | tests | There is no test that README config keys match `Config`. | Generate or assert the documented key set. |
| 86 | P2 | tests | There is no test that `config.example.conf` includes every config key. | Compare parsed keys against `Config` metadata. |
| 87 | P2 | tests | There is no link checker for Markdown docs. | Add a markdown link check in CI. |
| 88 | P2 | tests | There is no seed-data validation in CI. | Parse `seed_words.js` or canonical data and reject duplicates/missing fields. |
| 89 | P2 | tests | There is no fallback-data validation in CI. | Assert fallback words are unique, ranked, and have examples. |
| 90 | P2 | tests | There is no screenshot test for accent, sheen, exit fade, or recall mode. | Render deterministic states with a test painter or snapshot harness. |
| 91 | P2 | tests | There is no accessibility contrast test. | Compute contrast for theme presets and enhanced-contrast mode. |
| 92 | P2 | tests | There is no test for menu event handling. | Abstract menu events behind a channel adapter and inject commands. |
| 93 | P2 | tests | There is no test for pause/resume timing semantics. | Add deterministic clock tests around paused state. |
| 94 | P2 | tests | There is no fake clock; tests rely on `Instant::now()`. | Introduce a small clock trait or clock parameter for app timing. |
| 95 | P2 | tests | There is no test that speech is not invoked when disabled at the composition root. | Add a wiring test or split composition into a testable function. |
| 96 | P2 | tests | There is no test for non-macOS behavior. | Add cfg-level tests or CI jobs for compile-only non-mac targets. |
| 97 | P2 | tests | There is no property test for repaint scheduling around boundary times. | Use table/property cases for `elapsed`, `until_next`, and `exit_window`. |
| 98 | P2 | tests | There is no regression test for hard-coded viewport size. | Test native options construction after it is extracted. |
| 99 | P2 | tests | There is no bundle content test. | Inspect ZIP/app for executable, Info.plist, version, and permissions. |
| 100 | P2 | tests | There is no stored performance baseline history. | Emit machine-readable bench output and compare to a checked threshold. |
| 101 | P2 | architecture | `ui.rs` still mixes font loading, visual setup, text measurement, layout, and painting. | Split font/platform setup from `CardView` rendering. |
| 102 | P2 | architecture | `timing.rs` combines easing, pacing, reveal timeline, and repaint policy. | Keep it pure but split submodules if it grows further. |
| 103 | P2 | architecture | `theme.rs` is pure-ish but tied directly to egui color and shadow types. | Introduce theme tokens if non-egui rendering or tests need it. |
| 104 | P2 | architecture | `Config` is a flat 16-field bag consumed broadly. | Split into timing, appearance, behavior, source, and platform sub-configs. |
| 105 | P2 | architecture | Consumers receive the whole `Config` even when they need a few fields. | Pass narrow config views into timing/render/source code. |
| 106 | P2 | architecture | There is no validated config metadata layer. | Define keys, defaults, ranges, docs, and parsing in one table. |
| 107 | P2 | architecture | `main.rs` still constructs tray, source, deck, window, fonts, and speaker inline. | Extract a composition builder that returns fallible parts. |
| 108 | P2 | architecture | Native window options are not reusable or testable. | Move viewport construction into a function or platform adapter. |
| 109 | P2 | architecture | `App` still owns benchmark, menu, scheduling, speaker, and rendering orchestration. | Keep it as adapter, but split bench/menu helpers when adding more behavior. |
| 110 | P2 | architecture | Menu watcher uses the global `MenuEvent::receiver()` directly. | Wrap menu events behind an injectable adapter. |
| 111 | P2 | architecture | The menu watcher thread is not joined on shutdown. | Add a shutdown signal or make the thread lifecycle explicit. |
| 112 | P2 | architecture | Speech spawns one OS command per word without a queue. | Add a small speaker worker that can drop, serialize, or cancel speech. |
| 113 | P2 | architecture | There is no diagnostics object for startup state. | Carry source, config, font, tray, and screen diagnostics into logs/tray. |
| 114 | P2 | architecture | There is no error type taxonomy. | Add small error enums instead of scattered `eprintln!` and panics. |
| 115 | P2 | architecture | There is no app state model separate from egui adapter state. | Introduce a pure state reducer if interactions grow. |
| 116 | P2 | architecture | There is no module-boundary test to prevent UI deps entering core modules. | Add a lightweight dependency-boundary check. |
| 117 | P2 | architecture | Persistence is not represented in the architecture diagram yet. | Add store/source/selector flow once implemented. |
| 118 | P2 | architecture | Source loading and deck creation are one-shot only. | Design reloadable sources and deck replacement semantics. |
| 119 | P2 | architecture | There is no migration layer for future stored state. | Add versioned state and migration tests from day one. |
| 120 | P2 | architecture | There is no import/export boundary for custom decks. | Define deck file format before adding more sources. |
| 121 | P2 | config | Config format is a custom key-value parser with no quoting or escaping. | Either document the limits hard or switch to TOML. |
| 122 | P2 | config | Inline comments make color values with `#` surprising. | Support quoted values or parse color before stripping comments. |
| 123 | P2 | config | `MEMO_CONFIG` does not expand `~`. | Accept shell-expanded paths only in docs or implement expansion intentionally. |
| 124 | P2 | config | Non-UTF8 config files fail silently. | Report path and decode failure. |
| 125 | P2 | config | There are no CLI flags for one-off overrides. | Add a tiny CLI layer for config path, source, bench, and sample-config output. |
| 126 | P2 | config | There is no profile support for work/home modes. | Add named config profiles only after core keys are metadata-driven. |
| 127 | P2 | config | There are no config presets for common modes. | Add documented examples such as calm, recall, high-contrast, and speech. |
| 128 | P2 | config | The app does not create parent directories for config output because it never writes config. | When adding settings UI, make path creation explicit and safe. |
| 129 | P2 | config | There is no migration story for renamed keys. | Add aliases/deprecations in the metadata table. |
| 130 | P2 | config | Config validation cannot show all errors at once. | Parse into diagnostics instead of dropping bad values immediately. |
| 131 | P2 | ux | There is no first-run onboarding or "where did my words come from?" explanation in-app. | Add tray status and a README-linked first-run note. |
| 132 | P2 | ux | Users cannot tell whether they are on MongoDB or fallback deck while running. | Show source name and word count in tray. |
| 133 | P2 | ux | Users cannot trigger "reload words" after fixing MongoDB. | Add tray reload or periodic retry. |
| 134 | P2 | ux | Users cannot skip without opening tray. | Consider a global hotkey or menubar shortcut. |
| 135 | P2 | ux | Users cannot grade recall quality. | Add tray commands for forgot/hard/good once persistence exists. |
| 136 | P2 | ux | Users cannot reveal translation early without abandoning click-through. | Add tray "Reveal now" or temporary settings window. |
| 137 | P2 | ux | There is no session history. | Add a tray submenu or diagnostics window listing recent words. |
| 138 | P2 | ux | There is no session summary or progress readout. | Add daily counts after persistence exists. |
| 139 | P2 | ux | There is no idle/focus awareness. | Pause during fullscreen apps, DND, or configured focus windows. |
| 140 | P2 | ux | There is no sleep/wake handling. | Reset timers or advance cleanly after system wake. |
| 141 | P2 | ux | There is no battery/power-mode awareness. | Disable optional animation/speech on low power if requested. |
| 142 | P2 | ux | There is no time-of-day pacing. | Allow morning/day/evening interval profiles. |
| 143 | P2 | ux | There is no difficulty-band routing. | Let users focus on common, rare, new, or weak words. |
| 144 | P2 | ux | There is no deck progress indicator. | Add optional quiet progress text or tray-only progress. |
| 145 | P2 | ux | There is no paused visual state on the card. | Dim or add a tiny pause glyph only when paused. |
| 146 | P2 | ux | There is no "do not show over presentations" rule. | Add fullscreen/keynote/video detection hooks. |
| 147 | P2 | ux | There is no per-app exclusion list. | Let users pause over selected frontmost apps. |
| 148 | P2 | ux | There is no configurable margin. | Add `margin_x` and `margin_y` with sane bounds. |
| 149 | P2 | ux | Corner changes require config edit and restart. | Make corner a tray submenu or live-reload key. |
| 150 | P2 | ux | Speech has no rate, voice, or language option. | Add voice/rate config for macOS `say`. |
| 151 | P3 | render | The tray icon is a hand-drawn 22px raster with no dark/light/template variants. | Use a template icon or ship multiple sizes. |
| 152 | P3 | render | The card has one dark translucent visual identity only. | Add named theme presets before individual color overrides. |
| 153 | P3 | render | The theme has no UI token documentation. | Document spacing, sizes, intensities, and motion durations. |
| 154 | P3 | render | The accent and sheen are cosmetic options with little learning value. | Keep them opt-in and avoid expanding cosmetics before SRS. |
| 155 | P3 | render | Width transition duration lives in `ui.rs`, separate from timing docs. | Either move it to `timing.rs` or document it as layout-only motion. |
| 156 | P3 | render | Text line heights are measured every frame. | Cache stable galley metrics if profiling shows cost. |
| 157 | P3 | render | The card cannot show target-word emphasis inside examples. | Highlight the headword in the example when safe and subtle. |
| 158 | P3 | render | There is no monospace/IPA-specific rendering option. | Add optional IPA font family after font handling is centralized. |
| 159 | P3 | render | There is no grayscale-only mode for color-sensitive users. | Add a monochrome preset and keep accent disabled. |
| 160 | P3 | render | There is no text outline/halo option for extreme wallpapers. | Add an enhanced-contrast-only halo, not a default style. |
| 161 | P3 | content | Word list provenance and license are not documented. | Add attribution or replace with a documented source. |
| 162 | P3 | content | IPA conventions mix US and UK forms. | Pick a convention or store variants explicitly. |
| 163 | P3 | content | Russian translations are not marked by register, plurality, or context. | Add optional notes and review the top deck. |
| 164 | P3 | content | Example sentences are not CEFR-leveled. | Add a level field or keep examples deliberately simple. |
| 165 | P3 | content | Example sentences sometimes duplicate the target too plainly for recall. | Add alternate examples and rotate by mode. |
| 166 | P3 | content | There are no collocations, word families, or related-word hints. | Add optional metadata after schema versioning. |
| 167 | P3 | content | There is no phrase or multiword-expression support. | Let `Word` become a generic `CardItem` later if needed. |
| 168 | P3 | content | There is no custom user deck import path. | Add local JSON/TOML import before more remote sources. |
| 169 | P3 | content | There is no export of learned or ignored words. | Add export once the store exists. |
| 170 | P3 | content | There is no data normalization script. | Add a script that sorts, dedupes, validates, and formats the deck. |
| 171 | P3 | docs | `DESIGN_IDEAS.md` mixes shipped items, old rounds, backlog, and rationale in one long file. | Split shipped design notes from active backlog. |
| 172 | P3 | docs | Architecture docs do not show the future persistence/store flow. | Add a planned-state diagram after choosing the store boundary. |
| 173 | P3 | docs | Recommendation items have no owner, status, or target milestone. | Add status columns when this becomes an execution tracker. |
| 174 | P3 | docs | README has no screenshot or GIF of the overlay. | Add a real screenshot after visual design stabilizes. |
| 175 | P3 | docs | README assumes Homebrew MongoDB but does not explain alternatives. | Document fallback-only, local file, and Mongo options after source config lands. |
| 176 | P3 | docs | README does not say how to uninstall or clean config/state. | Add uninstall and data location notes. |
| 177 | P3 | docs | README platform scope is macOS-in-practice, but build deps imply more. | State supported vs compile-only platforms clearly. |
| 178 | P3 | docs | There is no CONTRIBUTING guide. | Add local dev, test, docs, and release workflow notes. |
| 179 | P3 | docs | There is no LICENSE file and Cargo package metadata is sparse. | Add license, authors, repository, and description metadata. |
| 180 | P3 | docs | There is no SECURITY or vulnerability reporting note. | Add a tiny security policy if publishing releases. |
| 181 | P3 | docs | Changelog does not describe every recent design/config addition in detail. | Update changelog during releases, not after the fact. |
| 182 | P3 | docs | Docs are not checked for stale audit counts or obsolete module names. | Add a simple docs grep/link check in CI. |
| 183 | P3 | docs | Root architecture aliases can confuse future readers if they duplicate canonical content. | Keep `architecture.md` as a tiny pointer to `docs/ARCHITECTURE.md`. |
| 184 | P3 | docs | Root aliases can drift if they grow beyond links. | Keep root docs as pointers to canonical `docs/` files. |
| 185 | P3 | process | There is no roadmap grouped by milestones. | Convert the top recommendations into 3-5 milestone themes. |
| 186 | P3 | process | There is no definition of done for UI changes. | Require tests, screenshot, docs, and performance check for visual work. |
| 187 | P3 | process | There is no design review checklist. | Add checks for contrast, motion, density, truncation, and calmness. |
| 188 | P3 | process | There is no release candidate smoke checklist. | Include bundle launch, tray, pause, next, config, fallback, Mongo, quit. |
| 189 | P3 | process | There is no issue template for feature vs bug vs content fixes. | Add templates if the repo becomes collaborative. |
| 190 | P3 | process | There is no data review checklist. | Validate duplicates, rank order, IPA convention, translation, example length. |
| 191 | P3 | process | There is no performance budget beyond "zero idle" prose. | Define budgets for idle FPS, startup time, binary size, and memory. |
| 192 | P3 | process | There is no binary size budget. | Track release artifact size and dependency growth. |
| 193 | P3 | process | There is no privacy note for local deck/state data. | Document exactly what is stored and where. |
| 194 | P3 | process | There is no crash/failure taxonomy for user reports. | Add labels for startup, source, render, tray, config, and release. |
| 195 | P3 | process | There is no automated formatting for Markdown and JS seed data. | Add markdown and data formatting checks. |
| 196 | P3 | process | There is no branch/PR phase naming convention in docs. | Add lightweight guidance for refactor, feature, docs, and release PRs. |
| 197 | P3 | process | There is no compatibility matrix for macOS versions. | Track tested macOS versions and chip architectures. |
| 198 | P3 | process | There is no documented backup/restore path for future learning state. | Design backup before users accumulate important progress. |
| 199 | P3 | process | There is no canonical "next 10" execution order tied to the audit. | Keep a short priority queue below this table and update it after each sprint. |
| 200 | P3 | process | The backlog can grow faster than the product improves. | Timebox ideation and ship from the highest-learning-value items first. |

## Best next 10 moves

1. Fix startup panics and graceful tray-less mode.
2. Fix `Info.plist` version generation from `Cargo.toml`.
3. Add the canonical deck validator and remove duplicate seed rows.
4. Add a persisted store primitive for exposure and review state.
5. Feature-gate MongoDB and ship an offline-first build path.
6. Move config path, screen sizing, and font loading into platform adapters.
7. Add CI gates for `MEMO_BENCH`, bundle contents, and Markdown links.
8. Add a screenshot or mock-painter visual test for the card states.
9. Add a settings/reload path so config changes do not require a restart.
10. Add accessibility knobs: font scale, reduced motion, and enhanced contrast.
