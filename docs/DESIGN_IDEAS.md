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
| 9 | **Glanceable session counter** — a faint "N words today" for quiet motivation, unobtrusive enough for ambient | Duolingo streaks | Low | Med | Med | backlog |
| 10 | **Reveal-on-recall** — `recall_mode` config holds the translation (and the example after it) back to ~55% of the interval, giving a real window to recall the meaning before the answer fades in. Off by default; never reveals earlier than `translation_delay` | Anki / flashcards | Med | Med | Low | ✅ shipped |

## Further backlog (beyond the current top 10)

- Etymology / word-family micro-line for deeper context.
- Optional subtle chime on a new word (off by default).
- Multi-monitor aware placement; smooth corner repositioning.
- Familiarity-based emphasis: words seen more often render smaller/dimmer.
- Hover-revealed speaker glyph hinting at the spoken pronunciation.

## Next top 10 editor-inspired ideas

These are intentionally borrowed from editors and productivity tools, but
filtered through the product rule: the overlay should stay calm, click-through,
and glanceable.

| # | Idea | Inspired by | Why it fits |
|---|------|-------------|-------------|
| 1 | **Quick preferences panel** — a tiny tray-opened settings window for interval, corner, opacity, recall, and pace indicator | Raycast, VS Code settings UI | Makes tuning visual feel immediate without editing config files |
| 2 | **Command-style tray actions** — Next, Pause, Speak current word, Copy current word, Copy session summary, Toggle recall mode, and Toggle pace indicator are shipped | command palettes | Gives power-user control while keeping the overlay itself passive |
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
| 3 | **Recently shown submenu** — last 5 words with copy/speak actions | editor recent files | Gives recovery when a word disappears too fast |
| 4 | **Deck lint command** — shipped as a Markdown health report for duplicates, missing fields, and invalid frequencies | problems panel | Useful before growing the Mongo deck further |
| 5 | **Accent color config** — shipped as `accent_color` for example emphasis and progress fill | theme settings | Adds personality without turning the card into a theme park |
| 6 | **High-contrast preset** — shipped as `high_contrast`, strengthening fill, border, and secondary text alphas | accessibility themes | Helps on bright or busy wallpapers |
| 7 | **Session word counter** — shipped as optional `session_counter`, a tiny status-style count on the card | editor status bar | Gives motivation with almost no extra visual weight |
| 8 | **Focus-hours config** — pause automatically during configured time windows | focus modes | Keeps the overlay respectful during deep work or calls |
| 9 | **Import preview command** — validate seed data before writing MongoDB | import previews | Prevents bad deck data from reaching the app |
| 10 | **Safe margins** — shipped global `screen_margin`; per-monitor/per-corner margins can follow | editor layout settings | Makes placement feel deliberate across different desktops |

## Next top 10, round 3

After the tray/export/status pass, the next strongest ideas lean into quality,
review, and respectful automation.

| # | Idea | Inspired by | Why it fits |
|---|------|-------------|-------------|
| 1 | **Recently shown recovery** — shipped as "Copy recent words" for the last 5 items; a dynamic submenu can follow | recent files | Recover a missed word without adding controls to the card |
| 2 | **Seen-frequency dimming** — shipped as optional `familiarity_dimming`; repeated session words become slightly quieter | editor diagnostics fading | Lets novelty stand out while keeping the same rotation model |
| 3 | **Smart cooldown by word** — prevent the same word from returning for a longer configurable span | debounce/cooldown controls | Improves variety beyond the current recent window |
| 4 | **Focus-hours config** — auto-pause during configured time windows | focus modes | Respectful during meetings or deep work |
| 5 | **High-contrast preset** — shipped as `high_contrast` for stronger material/text/border | accessibility themes | Better legibility on bright wallpapers |
| 6 | **Deck health severity levels** — group report items as error/warning/info | problems panel | Makes the health report easier to scan |
| 7 | **Import dry-run script** — validate `seed_words.js` shape before writing MongoDB | import preview | Prevents data cleanup from happening after the fact |
| 8 | **Daily rollup file** — append shown words to a local Markdown log | editor activity logs | Gives passive learning a durable trail |
| 9 | **Corner safe-area preview** — temporary non-click-through preview mode for placement tuning | layout inspectors | Helps set margins/corners without guessing |
| 10 | **Minimal preferences window** — tray-opened settings surface for the most-used config keys | settings UI | Removes config-file friction while keeping the card calm |

## Next top 10, round 4

After shipping high contrast, safe margin tuning, recent recovery, and
familiarity dimming, the next editor-inspired opportunities are mostly about
review loops, validation, and making settings less file-centric.

| # | Idea | Inspired by | Why it fits |
|---|------|-------------|-------------|
| 1 | **Import dry-run command** — validate seed/Mongo word data before loading | import previews | Catches bad data before it reaches the ambient surface |
| 2 | **Deck health severity levels** — errors/warnings/info in the copied report | problems panel | Makes data cleanup less flat and easier to prioritize |
| 3 | **Focus-hours config** — auto-pause during configured local time windows | focus modes | Keeps the overlay respectful during meetings and deep work |
| 4 | **Quiet review queue** — slightly boost words copied/recently revisited | review queues | Turns user intent into memory weight without a quiz UI |
| 5 | **Daily Markdown rollup** — optional local log of shown words by date | activity logs | Gives passive learning a durable trail for later review |
| 6 | **Minimal preferences window** — tray-opened controls for common config keys | settings UI | Removes config-file friction for visual tuning |
| 7 | **Placement preview mode** — temporary visible safe-area guides | layout inspectors | Helps tune corner/margin without guessing from config |
| 8 | **Accent swatches** — curated calm accent presets, still one accent at a time | theme pickers | Avoids arbitrary noisy colors while allowing personality |
| 9 | **Deck source badge in health report** — Mongo/fallback/bench source metadata | status bars | Makes reports easier to interpret when debugging data |
| 10 | **Session reset action** — clear counter/history without restarting the app | command palettes | Useful after changing decks or starting a new study block |

> This file is the running design backlog for an iterative improvement loop:
> each round ships the highest value-to-effort item still on-concept and proposes
> fresh ideas. It is curated, not a quota — items are dropped rather than forced
> when they would harm the calm-ambient identity.
