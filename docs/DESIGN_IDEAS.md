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
- **Symmetric exit settle** - `exit_duration` fades the whole card out before the
  next word instead of hard-cutting, so words leave the way they arrive. Off by
  default (hard cut), capped at half the interval, one `dim()` opacity multiplier
  threaded through every painted colour. (iOS notification dismissal, Things 3)
- **Interleaved spaced recap** - `recap_chance` occasionally re-shows a word from
  further back in the rotation (at least 5 cards ago) for spaced review instead of
  a fresh pick, refreshing its recency so it isn't repeated. Off by default, pure
  `deck.rs`, no UI or persistence. (Drops, Memrise spacing)
- **Per-line entrance settle** - `settle_px` drifts each line up a few points as
  it fades in, so the card eases together instead of appearing flat. Off by
  default, offsets only the drawn galley (no re-layout, no extra idle frames),
  the entrance complement to the exit fade. (Apple Dictionary reveal, Drops)
- **Accent rule** - `accent_color` (bare rrggbb hex) draws a short thin rounded
  bar under the headword, the first optional splash of colour in the otherwise
  monochrome card. Off by default, eases in with the word and fades with the
  exit. (Reeder rules, Things 3 underlines, Raycast accent)
- **Faux-vibrancy sheen** - `sheen` (0..1) paints a faint white-to-transparent
  gradient pooled at the top of the card so it reads like a lit material. Off by
  default, inset within the rounded corners and confined to the top, drawn as a
  4-vertex mesh over the fill, dimmed by the exit fade. (macOS NSVisualEffectView)
- **Rare-word dwell** - `rare_word_dwell` (0..1) stretches the display interval of
  less common words (by up to 1+strength as the frequency rank rises) so harder
  vocab lingers longer for more exposure, while common words keep the base
  interval. Off by default, pure interval math in `app.rs`, no persistence. First
  round-3 ship-now item landed. (Duolingo adaptive spacing; round 3 #4)
- **Coordinated theme presets** - `graphite`, `midnight`, `paper`, and
  `high-contrast` now define surface, border, shadow, semantic text colours,
  typography, and geometry as one token system. Graphite preserves the original
  default. (Raycast, Linear, macOS materials)
- **Accessibility design controls** - `font_scale` scales text and card geometry
  together; `enhanced_contrast` raises surface/border/text visibility; and
  `reduce_motion` disables settle, exit fade, and width morphing while retaining
  opacity reveal. System preference auto-detection remains backlog. (macOS
  accessibility, WCAG motion guidance)
- **State-aware tray commands** - native menu IDs are translated into five typed
  application commands; `Pause`/`Resume` and `Reload words`/`Retry source` always
  name the next effect. One compact state model updates text and availability
  without adding card chrome or idle frames. (VS Code command enablement,
  macOS menu conventions)

## Top 10 (round 2)

Ranked by value-to-effort, all vetted against the hard constraints (calm,
ambient, click-through, zero idle cost) by an adversarial review pass that read
the actual source. Notes record the feasibility findings.

| # | Idea | Inspired by | Value | Effort | Risk | Status |
|---|------|-------------|-------|--------|------|--------|
| 1 | **Symmetric exit settle** - fade the current card out before advancing, so words leave the way they arrive instead of hard-cutting. Alpha-only via one `dim()` multiplier, off by default, capped at half the interval | iOS notification dismissal, Things 3 rows | High | Low-Med | Low | ✅ shipped |
| 2 | **Answer-first type hierarchy** - the meaning outranks the IPA in size and brightness | Things 3, Reeder | High | Low | Low | ✅ shipped |
| 3 | **Interleaved spaced recap** - `recap_chance` re-shows an earlier word (>= 5 cards back) for spaced review, refreshing its recency. Off by default, pure `deck.rs`, no persistence | Drops, Memrise spacing | Med | Low | Low | ✅ shipped |
| 4 | **Faux-vibrancy material** - `sheen` (0..1) paints a faint top gradient as a 4-vertex mesh, inset within the rounded corners and confined to the top. Default-off, kept subtle. Top inner-highlight stroke deferred (the gradient alone carries the cue) | macOS NSVisualEffectView, Dynamic Island | Med | Med | Low | ✅ shipped |
| 5 | **Spaced-repetition selection (Leitner)** - prefer due words on an expanding schedule instead of pure frequency weighting; the highest learning lever. Blocked on two real gaps: `choose()` is `&self` (can't mutate box state) and there is no persistence layer at all, so it must add a read/write state file. Unlocks #8 too | Anki / SuperMemo, Memrise | High | High | Med | backlog (needs persistence) |
| 6 | **Exit collapse** - the width-collapse increment on top of #1: ease width back toward `MIN_WIDTH` as the card fades out. Do #1 first; this is its superset | iOS Live Activities | Med | Med | Med | backlog (after #1) |
| 7 | **Named theme presets** - `theme = graphite/midnight/paper/high-contrast` selects a coordinated semantic palette and geometry system; Graphite is default-preserving | Raycast, Linear, Arc | Med | Med | Low | ✅ shipped |
| 8 | **Familiarity-adaptive reveal pacing** - scale per-line delays by how often a word has been seen (new words reveal sooner, known words hold on the headword as a recognition test). Cheap once stats exist, but a no-op until #5 lands persistence | Anki graduating intervals, Duolingo | Med | Med | Low | backlog (after persistence) |
| 9 | **Static accent rule** - `accent_color` draws a short thin rounded bar under the headword (default off). The lightweight precursor to #7's accent key. Carries no information and adds the first chroma to a monochrome design, so opt-in | Reeder rules, Things 3 underlines | Med | Low | Low | ✅ shipped |
| 10 | **Per-line vertical settle** - `settle_px` drifts each line up a few px as it fades in, offsetting only the galley draw pos (cache-stable, no extra idle frames). Off by default | Apple Dictionary reveal, Drops stagger | Med | Low | Med | ✅ shipped |

## Further backlog (beyond the current top 10)

Round 2 is fully resolved: ranks 1-4, 9, 10 shipped; ranks 5-8 are deliberately
parked because they need infrastructure that doesn't exist yet (a persistence
layer for #5/#8) or a prerequisite refactor (#6 builds on the exit fade, #7 needs
the card colours promoted to config keys first). The next high-leverage move is
the persistence primitive, which unlocks the two highest learning-value ideas.

- Persistence layer (read/write state file) - the shared prerequisite for #5 and
  #8 and for any cross-session stat; worth building as its own primitive.
- Target-word emphasis within the example sentence (brighter/bold the headword).
- Etymology / word-family micro-line for deeper context.
- Optional subtle chime on a new word (off by default).
- Multi-monitor aware placement; smooth corner repositioning.
- Hover-revealed speaker glyph (out of scope while the window is click-through).

## Round 3 (wide sweep, curated)

Generated by an 8-lens ideation pass (code editors, design tools, learning apps,
macOS system UI, menubar/launcher apps, typography & motion, accessibility,
content-selection) followed by an adversarial curation pass. **128 raw ideas
collapsed to 46 viable**: 2 were genuinely impossible under click-through (a
multi-card queue preview and live keystroke hints both need input the overlay
can't take), and ~80 were near-duplicates merged into bundles. This is the honest
count, not padded to a round number.

Where the value actually sits: **accessibility (~30%)** - font scaling, contrast,
motion respect, position awareness, colour-blind safety; **content pacing (~30%)**
- time-of-day intervals, decay windows, difficulty-aware dwell, themed rotation;
**UI/system polish (~25%)** - easing presets, typography, Focus-mode pausing, tray
richness. The remaining ~15% is metadata enrichment (blocked on data work) and
admitted cosmetics (haptics, tray spin, audio chime). The biggest non-cosmetic gap
is still the one round 2 hit: real per-word memory (SRS / adaptive pacing) needs a
persistence layer that does not exist yet.

### Ship-now top 12 (round 3)

All ready or config-only, all default-preserving, all provable by tests + opt-in
defaults without pixel checks. Ordered by value-to-effort.

| # | Idea | Inspired by | Viability | Effort | Value |
|---|------|-------------|-----------|--------|-------|
| 1 | **Focus/DND auto-pause** - pause rotation while macOS Focus/Do-Not-Disturb or a fullscreen app is active; resume after. `respect_dnd` (default on) | macOS Focus modes | config-only | M | High |
| 2 | **Idle-repaint abort gate** - an owned wake worker now schedules long deadlines without repeated egui callbacks; reviewed release smoke reports 0.00 settled FPS | VS Code idle timer | shipped | S | High |
| 3 | **Font scale knob** - `font_scale` (0.8..1.5) scales type, spacing, width limits, and card height together | Raycast text size | shipped | S | High |
| 4 | **Difficulty-aware dwell** - rare words hold longer, common words pass quicker; `contextual_jitter` (default off) | Duolingo spacing | ready | M | High |
| 5 | **Entrance easing presets** - `entrance_curve` (spring / ease-in-out / linear / smooth) as pure preset functions, no extra idle cost | Framer motion | config-only | M | High |
| 6 | **Reduced-motion honor** - explicit `reduce_motion` is shipped; reading the macOS system flag remains | WCAG / macOS a11y | partial | M | High |
| 7 | **Position awareness** - query the usable screen frame (dock/notch) once and nudge the corner so the card never overlaps; `margin_x/y` overrides | NSScreen, Raycast | config-only | M | High |
| 8 | **Themed deck rotation** - optional `theme_tag` on words + `theme_rotation_mode` (daily/weekly); selector filters to the active theme | Duolingo themes | config-only | M | High |
| 9 | **Time-of-day pacing** - `morning/afternoon/evening_interval_secs`, chosen by the system clock at each roll | habit / circadian apps | config-only | S | High |
| 10 | **Recently-seen decay** - in-memory throttle of words shown in the last N hours via `decay_window_hours` + `decay_strength` (no disk) | Anki recent-avoidance | ready | M | High |
| 11 | **Enhanced-contrast toggle** - shipped as an explicit config control; system Increase Contrast sync and numeric WCAG tests remain | macOS Increase Contrast | partial | S | High |
| 12 | **Mini flashcard reveal** - generalises recall mode: translation starts hidden, fades in after a dwell/timeout; `translation_hide_mode` | Anki reveal | config-only | M | High |

### Full curated bank (46)

The complete deduped backlog, ranked. Viability: **ready** = shippable now on
tests+opt-in; **config** = a new default-preserving knob; **persist** = needs the
disk state layer first; **cosmetic** = works but near-zero learning value.

| # | Idea | Viability | Eff | Val |
|---|------|-----------|-----|-----|
| 1 | Focus/DND auto-pause | config | M | High |
| 2 | Idle-repaint abort gate | shipped | S | High |
| 3 | Font scale knob | shipped | S | High |
| 4 | Difficulty-aware dwell | ready | M | High |
| 5 | Entrance easing presets | config | M | High |
| 6 | Reduced-motion honor | partial | M | High |
| 7 | Position awareness (dock/notch) | config | M | High |
| 8 | Themed deck rotation | config | M | High |
| 9 | Time-of-day pacing | config | S | High |
| 10 | Recently-seen decay window | ready | M | High |
| 11 | Avoid-seen-today filter | config | M | High |
| 12 | Enhanced-contrast toggle | partial | S | High |
| 13 | Deck progress badge (47/350) | config | S | Med |
| 14 | Frequency-tier glyph (dots/stars) | ready | S | Med |
| 15 | Pause indicator (dim when paused) | ready | M | Med |
| 16 | Adaptive interval system (stack modifiers) | config | M | Med |
| 17 | Interval floor/ceiling clamp | ready | S | Med |
| 18 | Part-of-speech indicator | ready | S | Med |
| 19 | Related-word hints line | ready | S | Med |
| 20 | Example-line enhancements (rotate/label) | config | M | Med |
| 21 | Session history in tray submenu | cosmetic | M | Med |
| 22 | Mini flashcard reveal | config | M | High |
| 23 | Smart recap targeting (easier first) | ready | M | High |
| 24 | Typography system (tracking/line-height) | ready | M | Med |
| 25 | Session-length fatigue compensation | ready | M | Med |
| 26 | System-integration bundle (wallpaper/size) | config | M | Med |
| 27 | Selector strategies (balanced / min-rank) | ready | M | Med |
| 28 | Monospace IPA rendering | ready | S | Med |
| 29 | Accent-system expansion (fade / palette) | config | M | Med |
| 30 | Metadata enrichment (POS / collocations) | persist | L | High |
| 31 | Pronunciation cue + variant carousel | config | M | Med |
| 32 | Grayscale-only mode (colour-blind safe) | config | S | Med |
| 33 | Dwell-time reveal (motor-delay a11y) | ready | M | Med |
| 34 | US/UK/IPA transcription carousel | config | M | Med |
| 35 | Tray quick-toggle + timing presets | ready | M | Med |
| 36 | Dynamic entrance style (slide / float) | config | M | Med |
| 37 | Night mode (light card) | config | M | Med |
| 38 | Text outline / halo for contrast | config | M | Med |
| 39 | Haptic tap on advance | config | M | Low |
| 40 | Accent contrast WCAG warning | config | S | Med |
| 41 | Current-word info window from tray | ready | L | Med |
| 42 | Tray icon heartbeat on advance | ready | M | Low |
| 43 | Card-width-responsive font scale | ready | M | Low |
| 44 | Flashcard mode (word only, no example) | ready | S | Med |
| 45 | Soft audio chime on new word | ready | S | Low |
| 46 | Difficulty-band routing | config | S | Med |

Dropped as impossible under click-through: a peek-ahead multi-card queue, and live
keystroke/typing hints (both need pointer or keyboard input the overlay can't
receive). Roughly 80 raw ideas were merged into the bundles above rather than
listed separately (six accent variants, five session-counters, eight metadata
ideas, fourteen macOS-integration ideas, and so on).

> This file is the running design backlog for an iterative improvement loop: each
> round ships the highest value-to-effort item still on-concept and regenerates a
> fresh idea bank. It is curated, not a quota: items are dropped rather than forced
> when they would harm the calm-ambient identity or depend on infrastructure that
> does not exist yet. Round 3's wide sweep confirms the cheap on-concept wins now
> cluster in accessibility and pacing (config-only, shippable), while the deepest
> learning levers remain gated on a persistence primitive.

## Top 10 (round 4: calm control surface)

Ten new ideas derived from the current typed-command boundary and editor control
patterns. They are ranked by user value, but every idea must keep the overlay
click-through, preserve predictable native-menu ordering, and add zero idle work.

| # | Idea | Inspired by | Value | Effort | UX guardrail |
|---:|---|---|:---:|:---:|---|
| 1 | **Undo last skip** - keep one in-memory previous-card slot so an accidental Next can be reversed | editor navigation undo | High | M | One step only; never imply durable history |
| 2 | **Hold one more interval** - extend only the current card once without changing global cadence | pinned editor previews | High | S | Automatically clears on advance |
| 3 | **Pause after this card** - queue pause at the natural card boundary instead of freezing mid-reveal | breakpoint stop-after semantics | Med | M | Status must clearly say the pause is queued |
| 4 | **Safe retry reason** - show a short redacted issue kind such as connection or decode in source details | editor Problems panels | High | S | Never place backend messages or endpoints in menu text |
| 5 | **Use built-in deck for this session** - explicitly lock to fallback until restart | editor restricted/offline modes | Med | M | Temporary and visibly reversible; do not mutate config |
| 6 | **Deck-change summary** - after reload, report added/removed/changed counts before the next-card handoff | source-control diff summaries | Med | M | Counts only; no modal confirmation in normal flow |
| 7 | **External-config change indicator** - detect an edited config and offer Apply or Keep current settings | editor file-change conflict bars | High | L | Validate the whole candidate before exposing Apply |
| 8 | **Repeat pronunciation** - replay the current word without advancing it | editor Run Again commands | Med | M | Requires the bounded speech worker first |
| 9 | **Command acknowledgement line** - briefly reuse the disabled status row for Paused, Reload started, or Word advanced | IDE status bars | Med | S | One message, two seconds, no notification/toast stack |
| 10 | **Top-level action budget** - cap the root tray at seven actions and move future low-frequency tools into stable submenus | command palettes and editor menus | High | S | Never reorder commands by usage; preserve muscle memory |
