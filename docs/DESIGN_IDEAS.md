# Design backlog

Design and UX ideas for the memo-words ambient overlay, thought through with a
designer's lens and borrowed from polished reference apps. The guiding constraint
is the product's core identity: a **calm, glanceable, click-through ambient**
surface, not an app you focus on. The window is literally mouse-passthrough, so
anything needing hover, click, or input is out of scope; ideas that fight the
calm-ambient identity (heavy motion, chrome, interactivity) are downranked.

## Shipped

- **Depth & edge definition** - soft drop shadow + 1px hairline border so the
  translucent card reads as a floating surface and stays legible on busy/light
  wallpapers. (macOS widgets, iOS notification materials)
- **Example sentence line** - a short usage example, the single biggest learning
  lever for vocab. End to end: card renders it (single line, fixed-height card,
  long examples ellipsized), the fallback deck and all 350 `seed_words.js` words
  carry an `example`. (Apple Dictionary, Drops)
- **Appearance config** - `card_opacity` + `corner_radius` via the config file,
  default-preserving. (Raycast themes; accent and light/dark still deferred)
- **Reveal-on-recall** - `recall_mode` holds the translation (and the example
  after it) back to ~55% of the interval, giving a real window to recall the
  meaning before the answer fades in. Off by default, never earlier than
  `translation_delay`. (Anki / flashcards)
- **Answer-first type hierarchy** - re-tiered the lines by size and brightness so
  the translation (the payoff) outranks the phonetic transcription. The IPA used
  to render brighter than the meaning; now the order is headword > meaning > IPA >
  example, locked by a compile-time assertion. (Things 3, Reeder, Apple Dictionary)

## Top 10 (round 2)

Ranked by value-to-effort, all vetted against the hard constraints (calm,
ambient, click-through, zero idle cost) by an adversarial review pass that read
the actual source. Notes record the feasibility findings.

| # | Idea | Inspired by | Value | Effort | Risk | Status |
|---|------|-------------|-------|--------|------|--------|
| 1 | **Symmetric exit settle** - fade the current card out before advancing, so words leave the way they arrive instead of hard-cutting. `advance()` currently hard-cuts while entrances fade; this fills the missing half. Alpha-only (no geometry recompute), extra frames run only during the short exit | iOS notification dismissal, Things 3 rows | High | Low-Med | Low | next |
| 2 | **Answer-first type hierarchy** - the meaning outranks the IPA in size and brightness | Things 3, Reeder | High | Low | Low | ✅ shipped |
| 3 | **Interleaved spaced recap** - occasionally re-show a word seen ~10-20 cards ago (pick from inside the recent ring at an older offset). Pure Rust in `deck.rs`, no UI; guard small decks and carve out the recent-window test invariant | Drops, Memrise spacing | Med | Low | Low | backlog |
| 4 | **Faux-vibrancy material** - painter-faked top sheen gradient + 1px inner top highlight, mimicking macOS HUD/sidebar materials. Static, default-off, deliberately subtle (a bright highlight reads glossy and overlaps the shadow+border depth already shipped) | macOS NSVisualEffectView, Dynamic Island | Med | Med | Low | backlog |
| 5 | **Spaced-repetition selection (Leitner)** - prefer due words on an expanding schedule instead of pure frequency weighting; the highest learning lever. Blocked on two real gaps: `choose()` is `&self` (can't mutate box state) and there is no persistence layer at all, so it must add a read/write state file. Unlocks #8 too | Anki / SuperMemo, Memrise | High | High | Med | backlog (needs persistence) |
| 6 | **Exit collapse** - the width-collapse increment on top of #1: ease width back toward `MIN_WIDTH` as the card fades out. Do #1 first; this is its superset | iOS Live Activities | Med | Med | Med | backlog (after #1) |
| 7 | **Named theme presets** - a `theme =` key (graphite, mono, midnight, paper) applying a vetted palette before per-key overrides. Needs raw color keys (`CARD_TINT`, border, per-line alphas) promoted to config first, then presets on top; the single-pass merge makes base-key order a footgun to test | Raycast, Linear, Arc | Med | Med | Low | backlog (after color keys) |
| 8 | **Familiarity-adaptive reveal pacing** - scale per-line delays by how often a word has been seen (new words reveal sooner, known words hold on the headword as a recognition test). Cheap once stats exist, but a no-op until #5 lands persistence | Anki graduating intervals, Duolingo | Med | Med | Low | backlog (after persistence) |
| 9 | **Static accent rule** - a thin tinted hairline under the headword, one `accent_color` (default off). Pure static color, the lightweight precursor to #7's accent key. Carries no information, so low priority, and it adds the first chroma to a monochrome design | Reeder rules, Things 3 underlines | Med | Low | Low | backlog |
| 10 | **Per-line vertical settle** - each line drifts up a few px from a small offset as it fades in, driven by its existing ease. Offset the galley draw pos only (keeps the cache key stable); best bundled with the backlogged whole-card 0.98->1.0 scale entrance, not shipped alone | Apple Dictionary reveal, Drops stagger | Med | Low | Med | backlog |

## Further backlog (beyond the current top 10)

- Persistence layer (read/write state file) - the shared prerequisite for #5 and
  #8 and for any cross-session stat; worth building as its own primitive.
- Target-word emphasis within the example sentence (brighter/bold the headword).
- Etymology / word-family micro-line for deeper context.
- Optional subtle chime on a new word (off by default).
- Multi-monitor aware placement; smooth corner repositioning.
- Hover-revealed speaker glyph (out of scope while the window is click-through).

> This file is the running design backlog for an iterative improvement loop: each
> round ships the highest value-to-effort item still on-concept and regenerates a
> fresh top 10. It is curated, not a quota: items are dropped rather than forced
> when they would harm the calm-ambient identity or depend on infrastructure that
> does not exist yet.
