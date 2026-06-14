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
| 2 | **Example sentence line** — a short usage example; the single biggest learning lever for vocab. Shipped end to end: card renders it (single dim line, fixed-height card, long examples ellipsized), the fallback deck and all 350 `seed_words.js` words now carry an `example`. Target-word emphasis still deferred | Apple Dictionary, Drops context cards | High | Med | Med | ✅ shipped |
| 3 | **Adaptive contrast** — sample wallpaper luminance behind the card and tint the fill so it's always readable, light or dark desktop | macOS vibrancy / dynamic materials | High | High | Med | backlog |
| 4 | **Part-of-speech accent** — a small colored pill (noun / verb / adj …); color-coding aids recall and adds life without noise | Drops, Memrise | Med | Med (needs POS data) | Low | backlog |
| 5 | **Restrained entrance** — gentle scale 0.98→1.0 layered on the existing fade; presence without distraction (kept subtle for ambient) | iOS spring transitions | Med | Low | Med (motion can fight "calm") | backlog |
| 6 | **Pace indicator** — a faint progress ring/underline counting down to the next word; turns the cadence into a glanceable signal | timer apps, Things | Med | Med | Med (added visual element) | backlog |
| 7 | **Typographic pass** — tune sizes/weights/vertical rhythm; give the IPA a distinct, dimmer, more "phonetic" treatment | Reeder, Things 3 | Med | Low | Low | backlog |
| 8 | **Theme & appearance config** — opacity + corner radius via the config file (default-preserving; accent color and light/dark deferred) | Raycast themes | Med | Med | Low | ✅ opacity + radius |
| 9 | **Glanceable session counter** — a faint "N words today" for quiet motivation, unobtrusive enough for ambient | Duolingo streaks | Low | Med | Med | backlog |
| 10 | **Reveal-on-recall** — `recall_mode` config holds the translation (and the example after it) back to ~55% of the interval, giving a real window to recall the meaning before the answer fades in. Off by default; never reveals earlier than `translation_delay` | Anki / flashcards | Med | Med | Low | ✅ shipped |

## Further backlog (beyond the current top 10)

- Target-word emphasis within the example sentence (brighter/bold the headword).
- Etymology / word-family micro-line for deeper context.
- Optional subtle chime on a new word (off by default).
- Multi-monitor aware placement; smooth corner repositioning.
- Familiarity-based emphasis: words seen more often render smaller/dimmer.
- Hover-revealed speaker glyph hinting at the spoken pronunciation.

> This file is the running design backlog for an iterative improvement loop:
> each round ships the highest value-to-effort item still on-concept and proposes
> fresh ideas. It is curated, not a quota — items are dropped rather than forced
> when they would harm the calm-ambient identity.
