# Design backlog

Design and UX ideas for the memo-words ambient overlay, thought through with a
designer's lens and borrowed from polished reference apps. The guiding constraint
is the product's core identity: a **calm, glanceable, click-through ambient**
surface, not an app you focus on. Ideas that fight that identity (heavy motion,
chrome, interactivity) are deliberately downranked.

Ranked by value-to-effort for this product. Status is updated as items ship.

| # | Idea | Inspired by | Value | Effort | Risk | Status |
|---|------|-------------|-------|--------|------|--------|
| 1 | **Depth & edge definition** — soft drop shadow + 1px hairline border so the translucent card reads as a floating surface and stays legible on busy/light wallpapers | macOS widgets, iOS notification materials | High | Low | Low | ✅ shipped |
| 2 | **Example sentence line** — a short usage example; the single biggest learning lever for vocab. Shipped end to end: card renders it (single dim line, fixed-height card, long examples ellipsized), the fallback deck and all 350 `seed_words.js` words now carry an `example`. The headword is now softly emphasized inside the example when present | Apple Dictionary, Drops context cards | High | Med | Med | ✅ shipped |
| 3 | **Adaptive contrast** — sample wallpaper luminance behind the card and tint the fill so it's always readable, light or dark desktop | macOS vibrancy / dynamic materials | High | High | Med | backlog |
| 4 | **Part-of-speech accent** — a small colored pill (noun / verb / adj …); color-coding aids recall and adds life without noise | Drops, Memrise | Med | Med (needs POS data) | Low | backlog |
| 5 | **Restrained entrance** — gentle scale 0.98→1.0 layered on the existing fade; presence without distraction (kept subtle for ambient) | iOS spring transitions | Med | Low | Med (motion can fight "calm") | backlog |
| 6 | **Pace indicator** — an optional faint underline showing progress through the current word interval; turns the cadence into a glanceable signal without adding text | timer apps, Things | Med | Med | Med (extra idle repaint when enabled) | ✅ shipped behind `pace_indicator` |
| 7 | **Typographic pass** — long words/translations/examples scale down within the card instead of spilling past the glass; example headword gets a subtle brighter treatment | Reeder, Things 3 | Med | Low | Low | ✅ shipped |
| 8 | **Theme & appearance config** — opacity + corner radius via the config file (default-preserving; accent color and light/dark deferred) | Raycast themes | Med | Med | Low | ✅ opacity + radius |
| 9 | **Glanceable session counter** — a faint "N words today" for quiet motivation, unobtrusive enough for ambient | Duolingo streaks | Low | Med | Med | ✅ shipped behind `session_counter` |
| 10 | **Reveal-on-recall** — `recall_mode` config holds the translation (and the example after it) back to ~55% of the interval, giving a real window to recall the meaning before the answer fades in. Off by default; never reveals earlier than `translation_delay` | Anki / flashcards | Med | Med | Low | ✅ shipped |

## Further backlog (beyond the current top 10)

- Etymology / word-family micro-line for deeper context.
- Optional subtle chime on a new word (off by default).
- Multi-monitor aware placement; smooth corner repositioning.
- Familiarity-based emphasis: shipped as `familiarity_dimming`; a cross-session
  familiarity model can follow.
- Hover-revealed speaker glyph hinting at the spoken pronunciation.

## Next top 10 editor-inspired ideas

These are intentionally borrowed from editors and productivity tools, but
filtered through the product rule: the overlay should stay calm, click-through,
and glanceable.

| # | Idea | Inspired by | Why it fits |
|---|------|-------------|-------------|
| 1 | **Quick preferences panel** — a tiny tray-opened settings window for interval, corner, opacity, recall, and pace indicator | Raycast, VS Code settings UI | Makes tuning visual feel immediate without editing config files |
| 2 | **Command-style tray actions** — Next, Pause, Speak current word, Copy current word, Copy recent words, Copy session summary, Reset session, Toggle recall mode, and Toggle pace indicator are shipped | command palettes | Gives power-user control while keeping the overlay itself passive |
| 3 | **Deck focus presets** — Common words, verbs, business English, travel, confusing pairs | Notion database views | Lets the same surface serve different learning moments |
| 4 | **Quiet review queue** — words missed or skipped more often get a slightly higher future weight | Anki, SuperMemo | Adds memory logic without turning the app into a quiz |
| 5 | **Session summary export** — tray action copies the current session as Markdown; file export and daily rollups can follow | Obsidian, editors with export commands | Lets users review later in their own notes |
| 6 | **Appearance themes** — glass, paper, high-contrast, and minimal monochrome presets | editor theme pickers | Useful for different wallpapers and vision needs |
| 7 | **Per-word notes** — optional short note/source phrase rendered only in review/export, not on the ambient card | margin notes | Captures context without cluttering the overlay |
| 8 | **Smart silence windows** — pause during presentations, screen sharing, or focus hours | Focus modes, calendar-aware tools | Prevents the overlay from appearing at awkward moments |
| 9 | **Multi-monitor placement profiles** — choose monitor + corner per setup | editor workspace layouts | Makes the card predictable across desk/laptop setups |
| 10 | **Data health inspector** — show missing IPA/translation/example counts and duplicate words | lint panels | Keeps the deck clean as the word database grows |

## Next top 10, round 2

Another pass from an editor/product-design lens, after shipping copy/export
actions. These are ordered by how much they preserve the calm overlay while
adding leverage.

| # | Idea | Inspired by | Why it fits |
|---|------|-------------|-------------|
| 1 | **Checked menu state** — Pause, Recall, and Pace are now real checked menu items | editor status menus | Users can read mode state without waiting for the next card |
| 2 | **Copy as Anki TSV** — shipped as a tray action with word, translation, transcription, and example columns | Anki importers | Makes the ambient session actionable in study tools |
| 3 | **Recently shown recovery** — shipped as Copy recent words for the last 5 shown items; a dynamic submenu can follow | editor recent files | Gives recovery when a word disappears too fast |
| 4 | **Deck lint command** — shipped as a Markdown health report for duplicates, missing fields, and invalid frequencies | problems panel | Useful before growing the Mongo deck further |
| 5 | **Accent color config** — shipped as `accent_color` for example emphasis and progress fill | theme settings | Adds personality without turning the card into a theme park |
| 6 | **High-contrast preset** — shipped as `high_contrast`, strengthening fill, border, and secondary text alphas | accessibility themes | Helps on bright or busy wallpapers |
| 7 | **Session word counter** — shipped as optional `session_counter`, a tiny status-style count on the card | editor status bar | Gives motivation with almost no extra visual weight |
| 8 | **Focus-hours config** — shipped as `focus_hours` local quiet windows | focus modes | Keeps the overlay respectful during deep work or calls |
| 9 | **Import preview command** — shipped as `MEMO_DRY_RUN=1` / `--dry-run` deck validation | import previews | Prevents bad deck data from reaching the app |
| 10 | **Safe margins** — shipped global `screen_margin`; per-monitor/per-corner margins can follow | editor layout settings | Makes placement feel deliberate across different desktops |

## Next top 10, round 3

After the tray/export/status pass, the next strongest ideas lean into quality,
review, and respectful automation.

| # | Idea | Inspired by | Why it fits |
|---|------|-------------|-------------|
| 1 | **Recently shown recovery** — shipped as "Copy recent words" for the last 5 items; a dynamic submenu can follow | recent files | Recover a missed word without adding controls to the card |
| 2 | **Seen-frequency dimming** — shipped as optional `familiarity_dimming`; repeated session words become slightly quieter | editor diagnostics fading | Lets novelty stand out while keeping the same rotation model |
| 3 | **Smart cooldown by word** — shipped as `cooldown_words` for configurable recent-window length | debounce/cooldown controls | Improves variety beyond the current recent window |
| 4 | **Focus-hours config** — shipped as auto-pause during configured local time windows | focus modes | Respectful during meetings or deep work |
| 5 | **High-contrast preset** — shipped as `high_contrast` for stronger material/text/border | accessibility themes | Better legibility on bright wallpapers |
| 6 | **Deck health severity levels** — shipped as error/warning/info sections in the copied report | problems panel | Makes the health report easier to scan |
| 7 | **Import dry-run script** — shipped as dry-run deck validation before opening the UI | import preview | Prevents data cleanup from happening after the fact |
| 8 | **Daily rollup file** — shipped as optional `daily_rollup` Markdown append | editor activity logs | Gives passive learning a durable trail |
| 9 | **Corner safe-area preview** — shipped as `placement_preview` guides | layout inspectors | Helps set margins/corners without guessing |
| 10 | **Minimal preferences window** — shipped as a tray-opened settings surface | settings UI | Removes config-file friction while keeping the card calm |

## Next top 10, round 4

After shipping high contrast, safe margin tuning, recent recovery, and
familiarity dimming, the next editor-inspired opportunities are mostly about
review loops, validation, and making settings less file-centric.

| # | Idea | Inspired by | Why it fits |
|---|------|-------------|-------------|
| 1 | **Import dry-run command** — shipped as `MEMO_DRY_RUN=1` / `--dry-run` | import previews | Catches bad data before it reaches the ambient surface |
| 2 | **Deck health severity levels** — shipped as errors/warnings/info in the copied report | problems panel | Makes data cleanup less flat and easier to prioritize |
| 3 | **Focus-hours config** — shipped as `focus_hours` | focus modes | Keeps the overlay respectful during meetings and deep work |
| 4 | **Quiet review queue** — shipped as `quiet_review_weighting` for copied/revisited words | review queues | Turns user intent into memory weight without a quiz UI |
| 5 | **Daily Markdown rollup** — shipped as `daily_rollup` | activity logs | Gives passive learning a durable trail for later review |
| 6 | **Minimal preferences window** — shipped as tray-opened controls for common config keys | settings UI | Removes config-file friction for visual tuning |
| 7 | **Placement preview mode** — shipped as visible safe-area guides | layout inspectors | Helps tune corner/margin without guessing from config |
| 8 | **Accent swatches** — shipped as curated calm accent presets, still one accent at a time | theme pickers | Avoids arbitrary noisy colors while allowing personality |
| 9 | **Deck source badge in health report** — shipped as Mongo/fallback/bench source metadata | status bars | Makes reports easier to interpret when debugging data |
| 10 | **Session reset action** — shipped as Reset session, clearing history/count/familiarity without restart | command palettes | Useful after changing decks or starting a new study block |

## Next top 10, round 5

After session reset, the strongest remaining ideas are less about adding visible
chrome and more about making the learning loop durable, inspectable, and easy
to tune.

| # | Idea | Inspired by | Why it fits |
|---|------|-------------|-------------|
| 1 | **Focus-hours config** — shipped as auto-pause during configured local time windows | focus modes | Prevents the overlay from intruding during calls or deep work |
| 2 | **Deck health severity levels** — shipped as copied report sections for errors, warnings, and info | problems panel | Makes cleanup priority obvious at a glance |
| 3 | **Daily Markdown rollup** — shipped as shown words appended to a dated local note | activity logs | Turns ambient exposure into reviewable notes |
| 4 | **Import dry-run command** — shipped as dry-run deck validation | import previews | Keeps bad rows out of the deck before they become UI issues |
| 5 | **Quiet review weighting** — shipped as boost for copied/recently revisited words | review queues | Uses user intent as a study signal without quizzes |
| 6 | **Accent swatch names** — shipped as `accent_color = mint/sky/rose/plain` presets | theme palettes | Makes visual tuning friendlier than raw hex for most users |
| 7 | **Placement preview mode** — shipped as temporary visible frame and safe-area guides | layout inspectors | Lets users tune corner/margin confidently |
| 8 | **Deck source metadata** — shipped as source/cooldown/review state in the health report | status bars | Makes fallback-vs-Mongo state easy to diagnose |
| 9 | **Session freeze action** — shipped as Freeze current word | editor read-only modes | Useful while manually reviewing a word |
| 10 | **Preferences window MVP** — shipped as an egui settings window for common toggles | settings UI | Removes config-file friction while preserving the passive card |

## Next top 10, round 6

After shipping grouped deck health and accent swatches, the next valuable ideas
are mostly about reducing setup friction and making passive learning leave a
useful trail outside the overlay.

| # | Idea | Inspired by | Why it fits |
|---|------|-------------|-------------|
| 1 | **Focus-hours config** — shipped as local quiet windows that auto-pause rotation | focus modes | Keeps the overlay out of calls, recordings, and deep work |
| 2 | **Daily Markdown rollup** — shipped as shown words appended to a dated local note | editor activity logs | Makes passive exposure reviewable without changing the card |
| 3 | **Import dry-run command** — shipped as `MEMO_DRY_RUN=1`, `--dry-run`, or `--validate-deck` | import previews | Catches bad fields before they reach the UI |
| 4 | **Quiet review weighting** — shipped as a gentle boost for copied/recently revisited words | review queues | Treats user intent as a study signal without adding quizzes |
| 5 | **Placement preview mode** — shipped as temporary frame and safe-area guides | layout inspectors | Makes corner/margin tuning visual and reversible |
| 6 | **Deck source metadata** — shipped as Mongo/static/fallback/bench source in reports | status bars | Makes data debugging obvious when the app falls back |
| 7 | **Minimal preferences window** — shipped as tray-opened egui controls for common settings | settings UI | Removes config-file friction for appearance and timing |
| 8 | **Session freeze action** — shipped as stop-rotation while keeping copy/export enabled | read-only modes | Supports manual review without losing the current word |
| 9 | **Smart cooldown config** — shipped as `cooldown_words` | debounce controls | Improves variety while preserving the current selector model |
| 10 | **Accent contrast guard** — shipped as clamping for accents that are too dim/bright | theme diagnostics | Keeps custom colors calm and legible across wallpapers |

## Next top 10, round 7

After the full setup/review/tuning pass, the next ideas move toward richer deck
quality, better persistence, and multi-context polish while still keeping the
card itself quiet.

| # | Idea | Inspired by | Why it fits |
|---|------|-------------|-------------|
| 1 | **Seed file validator** — parse `seed_words.js` directly and report row/field issues before Mongo import | import linters | Catches source-file mistakes even when Mongo is empty or unavailable |
| 2 | **Rollup de-dup mode** — collapse repeated words per day with counts | activity logs | Keeps daily notes compact during long sessions |
| 3 | **Review boost decay** — let copied-word boosts fade after N appearances | spaced repetition queues | Prevents one copied word from staying boosted forever |
| 4 | **Deck source tray label** — show current source in the tray tooltip | status bars | Makes fallback state visible without opening a report |
| 5 | **Preferences persistence** — write changed settings back to config on demand | settings editors | Makes the new preferences window useful beyond one session |
| 6 | **Focus-hours day presets** — weekday/weekend quiet windows | calendar focus modes | Matches real schedules without needing external calendar access |
| 7 | **Rollup open action** — tray action to open the Markdown rollup file | editor file commands | Connects passive capture to review with one action |
| 8 | **Deck health score** — summarize report severity as a single status line | diagnostics panels | Makes dry-run output fast to scan |
| 9 | **Per-corner margin config** — separate horizontal/vertical safe margins | layout inspectors | Handles notches, docks, and unusual monitor setups |
| 10 | **Accent accessibility report** — include guarded/custom accent luma in health/preferences | theme diagnostics | Helps tune appearance without visual guessing |

> This file is the running design backlog for an iterative improvement loop:
> each round ships the highest value-to-effort item still on-concept and proposes
> fresh ideas. It is curated, not a quota — items are dropped rather than forced
> when they would harm the calm-ambient identity.
