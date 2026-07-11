# Recommendation register: 500 findings and ideas

This is the canonical, current audit for `memo-words`. It combines concrete
defects, product gaps, maintainability risks, UX improvements, and future ideas.
It was re-checked against the source after the July 2026 passes that split
session/timing/rendering/source responsibilities, added themes and accessibility
behavior, and moved remote loading behind fallback-first startup.

The list intentionally has one source of truth. `README.md`,
`docs/ARCHITECTURE.md`, `architecture.md`, and `recommendation.md` summarize and
link here instead of copying 500 rows and drifting out of sync.

Priority: **P0** blocks the product promise or release confidence; **P1** is a
high-leverage correctness/product issue; **P2** is important quality debt;
**P3** is a useful refinement or longer-horizon idea. Kind distinguishes an
observed **Problem/Risk** from an **Improvement/Idea**.

## Coverage

| Range | Area | Count |
|---|---|---:|
| 1-25 | Product and user value | 25 |
| 26-50 | Learning science | 25 |
| 51-75 | Domain model | 25 |
| 76-100 | Deck and content quality | 25 |
| 101-125 | Sources and import | 25 |
| 126-150 | Persistence and state | 25 |
| 151-175 | Selection and scheduling | 25 |
| 176-200 | Timing and session behavior | 25 |
| 201-225 | Visual design and layout | 25 |
| 226-250 | Accessibility | 25 |
| 251-275 | Tray, preferences, and interaction | 25 |
| 276-300 | Platform and display integration | 25 |
| 301-325 | Performance and resource use | 25 |
| 326-350 | Reliability and error handling | 25 |
| 351-375 | Architecture, SOLID, and DRY | 25 |
| 376-400 | Testing and quality assurance | 25 |
| 401-425 | Security and privacy | 25 |
| 426-450 | Build, release, and supply chain | 25 |
| 451-475 | Documentation and developer experience | 25 |
| 476-500 | Diagnostics, roadmap, and ecosystem | 25 |

## 1. Product and user value

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 1 | P0 | Problem | Learning progress disappears when the process exits. | Add a versioned local progress store before deeper learning features. |
| 2 | P0 | Problem | Recap is random in-memory resurfacing, not durable spaced repetition. | Persist due dates and route review selection through an SRS policy. |
| 3 | P0 | Risk | Fallback-first startup has no release gate proving a timely first visible card. | Add an app-bundle smoke assertion with a cold-start latency budget. |
| 4 | P1 | Problem | Non-technical users must hand-edit a config file. | Add a small tray-opened preferences window with validation and save feedback. |
| 5 | P1 | Problem | The product never explains whether MongoDB or fallback data is active. | Show source state, word count, and last load result in diagnostics/tray. |
| 6 | P1 | Risk | A tray creation failure leaves a click-through overlay with no direct controls. | Provide a fallback hotkey or small non-overlay control window. |
| 7 | P1 | Problem | Users cannot choose a target monitor. | Add monitor selection with a stable name/index fallback strategy. |
| 8 | P1 | Problem | Offline use is limited to a small 40-word fallback deck. | Bundle a curated portable deck and treat MongoDB as optional enrichment. |
| 9 | P1 | Problem | English-to-Russian is hard-coded as the only learning direction. | Model source and target languages and expose them in deck metadata. |
| 10 | P1 | Risk | Click-through identity prevents grading, reveal, skip, and mute on the card. | Keep the ambient card passive but provide active controls in tray/preferences. |
| 11 | P1 | Improvement | There is no user-defined daily or weekly learning goal. | Add optional exposure/review goals without turning the overlay into a dashboard. |
| 12 | P1 | Improvement | There is no focused study/session mode alongside ambient mode. | Add a temporary active-review window backed by the same deck and scheduler. |
| 13 | P1 | Improvement | The app cannot auto-pause during Focus, meetings, presentations, or fullscreen work. | Add opt-in system-aware quiet rules and a visible override state. |
| 14 | P1 | Problem | Users cannot inspect recently shown words. | Add a lightweight history submenu or companion window. |
| 15 | P1 | Improvement | No progress summary shows exposure, recall, or retention trends. | Build local-only summaries after persistence exists. |
| 16 | P1 | Problem | Only one deck/source configuration can be active. | Add named deck profiles and an explicit active profile. |
| 17 | P1 | Problem | Personal CSV/JSON word lists have no supported import path. | Define a portable deck schema and import validator. |
| 18 | P1 | Risk | Source failure silently changes the product from full deck to fallback behavior. | Surface fallback activation once, without recurring notifications. |
| 19 | P2 | Improvement | The cadence cannot follow working hours or a study schedule. | Add quiet hours and per-period interval profiles. |
| 20 | P2 | Risk | High-frequency ambient exposure can become wallpaper and lose attention value. | Add fatigue-aware cadence and optional session caps. |
| 21 | P2 | Problem | Speech cannot be muted immediately without editing config and restarting. | Add a tray speech toggle and serialize speech through one worker. |
| 22 | P2 | Problem | Users receive no in-app indication that a newer release exists. | Add a privacy-preserving, opt-in release check. |
| 23 | P2 | Problem | There is no user-facing diagnostics export for support. | Add a redacted diagnostics report with config/source/runtime state. |
| 24 | P2 | Improvement | The card cannot temporarily hide without pausing learning state. | Add separate Hide/Show and Pause/Resume commands. |
| 25 | P3 | Idea | Product decisions have no privacy-safe evidence about feature usefulness. | Define opt-in local counters first; never add telemetry by default. |

## 2. Learning science

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 26 | P0 | Problem | The system records no per-word exposure count. | Persist exposure events keyed by a stable word identity. |
| 27 | P0 | Problem | A learner cannot grade recall quality. | Add Again/Hard/Good/Easy actions to the active control surface. |
| 28 | P0 | Problem | There is no concept of a due review. | Store `due_at` and select overdue cards before fresh exposure. |
| 29 | P1 | Problem | Frequency rank is treated as a proxy for learner difficulty. | Separate corpus frequency from personal difficulty. |
| 30 | P1 | Problem | The same reveal schedule applies to new and familiar words. | Adapt reveal delay using exposure and recall history. |
| 31 | P1 | Problem | Recall mode uses a fixed 55 percent interval heuristic. | Make the hold duration a tested policy derived from card state. |
| 32 | P1 | Improvement | Lapses are not tracked or treated differently from first exposure. | Add lapse count, relearning steps, and short retry intervals. |
| 33 | P1 | Improvement | Persistently failed words cannot become leeches. | Add configurable leech thresholds and suspend/inspect actions. |
| 34 | P1 | Improvement | New and review words have no explicit mixing policy. | Add daily limits and a deterministic new/review ratio. |
| 35 | P1 | Improvement | Desired retention cannot be tuned. | Introduce a scheduler parameter with conservative defaults. |
| 36 | P1 | Risk | Closely related words can appear near each other and create interference. | Add semantic-family separation when metadata becomes available. |
| 37 | P1 | Improvement | One example sentence is repeated forever. | Support multiple examples and rotate them by exposure. |
| 38 | P1 | Idea | Recognition is trained more than production. | Add optional reverse cards and cloze prompts in active mode. |
| 39 | P1 | Improvement | Audio is output-only and never becomes a listening prompt. | Add word-hidden audio recall cards to active mode. |
| 40 | P1 | Problem | Pronunciation success is not captured. | Allow optional self-rating or speech feedback outside the overlay. |
| 41 | P2 | Improvement | The scheduler has no forgetting-curve model. | Start with a transparent Leitner/SM-2 policy, then evaluate alternatives. |
| 42 | P2 | Improvement | Review intervals cannot be capped for difficult material. | Add minimum/maximum interval policy values. |
| 43 | P2 | Risk | A long example may reveal the meaning before the translation line. | Audit examples for accidental answer leakage and mark cloze spans. |
| 44 | P2 | Improvement | Context difficulty is not considered when choosing examples. | Tag examples by complexity and match them to learner level. |
| 45 | P2 | Improvement | Morphological families are invisible. | Store lemma, inflections, and family links for reinforcement. |
| 46 | P2 | Improvement | Part of speech does not influence pacing or examples. | Add POS-aware senses and context selection. |
| 47 | P2 | Risk | Frequent passive exposures may be counted as learning without attention. | Distinguish displayed, acknowledged, and graded events. |
| 48 | P2 | Idea | There is no delayed test after initial exposure. | Schedule a first recall check after a configurable minimum lag. |
| 49 | P3 | Idea | The app cannot balance semantic, phonetic, and orthographic difficulty. | Add separate difficulty dimensions once evidence exists. |
| 50 | P3 | Idea | No experiment framework compares learning policies locally. | Add deterministic policy simulation over anonymized local history. |

## 3. Domain model

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 51 | P0 | Problem | `Word` has no stable ID, so durable history cannot join safely. | Add source-qualified IDs with migration rules. |
| 52 | P1 | Problem | The schema has no version field. | Version deck records and validate supported versions at import. |
| 53 | P1 | Problem | A word can hold only one translation. | Model one or more senses with labeled translations. |
| 54 | P1 | Problem | A word can hold only one example sentence. | Store an ordered collection of examples per sense. |
| 55 | P1 | Problem | Part of speech is absent. | Add an optional normalized POS enum plus display label. |
| 56 | P1 | Problem | Source language and target language are implicit. | Put language tags in deck and sense metadata. |
| 57 | P1 | Problem | Transcription has no notation or locale metadata. | Store IPA/system and pronunciation locale explicitly. |
| 58 | P1 | Problem | UK/US pronunciation variants cannot coexist. | Add pronunciation records keyed by locale. |
| 59 | P1 | Problem | Frequency is a bare `i32` with unclear corpus semantics. | Wrap it in a validated rank type with corpus metadata. |
| 60 | P1 | Risk | `frequency <= 0` silently means rarest in selection. | Represent unknown rank as `Option` and make policy explicit. |
| 61 | P1 | Problem | Word text is not normalized at the domain boundary. | Trim and Unicode-normalize imported fields. |
| 62 | P1 | Problem | Empty word/translation values are representable. | Introduce validated constructors or import validation errors. |
| 63 | P1 | Improvement | Lemma and displayed form are conflated. | Separate lemma, surface form, and inflection metadata. |
| 64 | P1 | Improvement | Homographs cannot be distinguished. | Key records by sense/source ID, not display spelling. |
| 65 | P1 | Improvement | Synonyms and antonyms have no representation. | Add optional semantic relation records. |
| 66 | P2 | Improvement | Difficulty and mastery have no domain types. | Add bounded value objects instead of free floats. |
| 67 | P2 | Improvement | Tags/themes cannot be attached to words. | Add normalized tags with deck-level definitions. |
| 68 | P2 | Improvement | Content provenance and license are absent per record. | Add source URL/name/license fields. |
| 69 | P2 | Risk | `example` defaults to empty string, mixing missing with intentionally blank. | Use `Option<String>` or a validated examples collection. |
| 70 | P2 | Improvement | Notes and mnemonics cannot be stored. | Add optional learner-owned annotations outside immutable deck content. |
| 71 | P2 | Improvement | Audio assets cannot be referenced. | Add optional local/remote pronunciation asset descriptors. |
| 72 | P2 | Improvement | Image cues have no domain representation. | Add optional media references without coupling the core to rendering. |
| 73 | P2 | Risk | Serde deserialization accepts unbounded string lengths. | Enforce import limits before records enter the deck. |
| 74 | P3 | Idea | Collocations are not modeled. | Add typed phrase links with examples when content supports them. |
| 75 | P3 | Idea | Etymology cannot be attached without overloading examples. | Add optional structured enrichment blocks. |

## 4. Deck and content quality

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 76 | P0 | Problem | The fallback and Mongo seed are separate sources of truth. | Generate both from one canonical deck asset. |
| 77 | P1 | Risk | Duplicate spellings can enter the seed with conflicting ranks. | Add a CI validator for normalized duplicate IDs and spellings. |
| 78 | P1 | Risk | Shared fallback/seed records can disagree in translation or example. | Compare generated subsets byte-for-byte in tests. |
| 79 | P1 | Problem | The fallback deck is too small for sustained offline use. | Bundle the full reviewed starter deck. |
| 80 | P1 | Problem | Content quality has no automated lint step. | Validate required fields, lengths, Unicode, rank, and duplicates. |
| 81 | P1 | Risk | Example sentences may not match the intended sense. | Add editorial sense checks and reviewer ownership. |
| 82 | P1 | Risk | Russian translations may be inconsistent in style or aspect. | Define translation guidelines and run a terminology review. |
| 83 | P1 | Risk | IPA strings are not validated. | Add lightweight delimiter/symbol validation and manual exceptions. |
| 84 | P1 | Problem | There is no content schema documentation for contributors. | Publish a deck format specification with valid examples. |
| 85 | P1 | Improvement | Frequency ranks have no named corpus or revision. | Record corpus source, date, and ranking method. |
| 86 | P1 | Improvement | Words cannot be grouped by learner level. | Add CEFR or project-specific level tags with provenance. |
| 87 | P1 | Improvement | Offensive, sensitive, or domain-specific terms cannot be flagged. | Add content labels and opt-in filters. |
| 88 | P1 | Risk | Seed updates have no migration or diff report. | Generate added/changed/removed record summaries in CI. |
| 89 | P2 | Improvement | Examples do not highlight the target form. | Store target spans and render semantic emphasis. |
| 90 | P2 | Improvement | Examples cannot include alternate forms of the target. | Link example spans to lemma/inflection metadata. |
| 91 | P2 | Risk | Long translations are not reviewed against card constraints. | Add width-budget linting using representative font metrics. |
| 92 | P2 | Risk | Very short or context-free examples provide little learning value. | Add minimum editorial quality rules, not only length checks. |
| 93 | P2 | Improvement | Punctuation and capitalization style are not normalized. | Define per-field normalization with explicit exceptions. |
| 94 | P2 | Improvement | Content changes cannot cite a reviewer. | Add review metadata in the source asset or pull request process. |
| 95 | P2 | Risk | Homographs may collapse under spelling-only duplicate checks. | Validate stable IDs while allowing distinct labeled senses. |
| 96 | P2 | Improvement | There is no small beginner deck separate from the full list. | Ship curated presets generated from the canonical asset. |
| 97 | P2 | Improvement | Users cannot exclude known words before first run. | Support ignore lists and bulk known-word import. |
| 98 | P2 | Improvement | Deck licenses are not represented in import/export. | Carry provenance and license metadata with every deck. |
| 99 | P3 | Idea | Content has no topical bundles. | Add optional travel/work/academic tag collections. |
| 100 | P3 | Idea | There is no reproducible content quality score. | Define transparent lint/editorial metrics and publish reports. |

## 5. Sources and import

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 101 | P0 | Problem | `LoadReport` has no elapsed time, attempt ID, or completion timestamp. | Add bounded attempt metadata so diagnostics can order and compare loads. |
| 102 | P0 | Problem | The background source runner is one-shot and cannot express cancellation, retry, or progress. | Define an owned source-load lifecycle before adding reload controls. |
| 103 | P1 | Risk | Mongo records are consumed without a deterministic sort or result bound. | Define stable ordering and a validated maximum deck size. |
| 104 | P1 | Problem | Decode reporting keeps only three messages and no stable document identity. | Retain redacted record IDs/field paths plus aggregate counts. |
| 105 | P1 | Problem | Mongo URI is hard-coded. | Support environment/config URI with redaction in diagnostics. |
| 106 | P1 | Problem | Database name is hard-coded. | Add a validated `mongo_database` setting. |
| 107 | P1 | Problem | Collection name is hard-coded. | Add a validated `mongo_collection` setting. |
| 108 | P1 | Problem | Connection and selection timeouts are fixed in code. | Expose conservative bounded timeout settings. |
| 109 | P1 | Risk | The latest source report is discarded after logging and deck handoff. | Retain redacted source state for diagnostics and retry decisions. |
| 110 | P1 | Improvement | The app cannot reload words without restart. | Trigger the existing atomic deck replacement through an owned retry command. |
| 111 | P1 | Risk | Remote loading has no cancellation during quit or source change. | Pass a cancellation token through asynchronous loaders. |
| 112 | P1 | Risk | Repeated failures have no retry/backoff policy. | Add bounded exponential backoff with manual Retry. |
| 113 | P1 | Problem | There is no local JSON/CSV/TOML source adapter. | Implement a validated file source before adding more remote backends. |
| 114 | P1 | Improvement | Multiple sources cannot be composed by explicit priority. | Generalize fallback into a source chain with named outcomes. |
| 115 | P1 | Risk | Records from combined sources are not deduplicated. | Merge by stable source-qualified ID with conflict rules. |
| 116 | P1 | Risk | Source size is unbounded before allocation into the deck. | Add record and byte limits with clear failure messages. |
| 117 | P1 | Improvement | Last successful remote data is not cached. | Cache a validated snapshot and fall back to it before the tiny deck. |
| 118 | P1 | Risk | Credentials embedded in URI could leak through future logs. | Redact userinfo and sensitive query parameters centrally. |
| 119 | P2 | Improvement | TLS/auth setup is not documented or represented. | Add secure connection examples and typed options. |
| 120 | P2 | Improvement | A source cannot declare language, license, or schema capabilities. | Add source/deck metadata to load results. |
| 121 | P2 | Risk | Source data is accepted without Unicode/control-character screening. | Normalize and reject unsafe control characters at import. |
| 122 | P2 | Improvement | Import errors cannot point to row/document identity consistently. | Define a location type for file rows and database IDs. |
| 123 | P2 | Improvement | Import has no dry-run command. | Add validation-only CLI output with counts and errors. |
| 124 | P3 | Idea | Online dictionary enrichment has no extension boundary. | Define an enrichment port separate from the base word source. |
| 125 | P3 | Idea | Source freshness cannot be scheduled or displayed. | Store fetch timestamps and optional refresh policy. |

## 6. Persistence and state

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 126 | P0 | Problem | There is no `Store` port in the domain. | Define minimal load/save transaction interfaces before choosing storage. |
| 127 | P0 | Risk | Adding persistence without stable IDs would misattach history. | Land ID/version migrations before recording progress. |
| 128 | P1 | Improvement | State has no versioned envelope. | Include schema version, app version, and migration metadata. |
| 129 | P1 | Risk | There is no atomic-write strategy. | Write temp, fsync where appropriate, then rename atomically. |
| 130 | P1 | Risk | Corrupted state has no recovery path. | Validate, quarantine the bad file, and restore the last backup. |
| 131 | P1 | Improvement | Storage location is not abstracted by platform. | Add a platform path provider for state, cache, config, and logs. |
| 132 | P1 | Risk | Multiple app instances could race on one state file. | Add single-instance policy or explicit locking. |
| 133 | P1 | Improvement | There is no transaction boundary for grade plus schedule update. | Persist each review event and derived state atomically. |
| 134 | P1 | Improvement | Exposure history has no event schema. | Define immutable events with timestamps and source IDs. |
| 135 | P1 | Risk | Wall-clock changes could invalidate due dates. | Store UTC instants and test clock rollback/forward behavior. |
| 136 | P1 | Improvement | User settings and learning state have no backup/export path. | Add portable, versioned export with secrets excluded. |
| 137 | P1 | Improvement | Importing state has no conflict policy. | Offer replace/merge preview with deterministic rules. |
| 138 | P1 | Risk | Future migrations could silently drop unknown fields. | Round-trip fixtures and retain forward-compatible extensions where possible. |
| 139 | P2 | Improvement | State growth has no retention/compaction policy. | Compact derived history while preserving auditable review events. |
| 140 | P2 | Improvement | No last-known-good snapshot is retained. | Rotate a small bounded set of backups. |
| 141 | P2 | Risk | Sensitive local history could inherit permissive file modes. | Create state/config files with user-only permissions. |
| 142 | P2 | Improvement | Reset actions are not scoped. | Separate reset session, progress, settings, and all-data commands. |
| 143 | P2 | Risk | A reset could be irreversible and accidental. | Require confirmation and offer a timestamped backup. |
| 144 | P2 | Improvement | State writes cannot be deferred/coalesced intelligently. | Batch low-risk counters while flushing grades immediately. |
| 145 | P2 | Improvement | Persistence health cannot be inspected. | Report path, schema, last save, backup, and migration status. |
| 146 | P2 | Idea | SQLite/file choice has no measured decision record. | Benchmark complexity, durability, size, and migration needs in an ADR. |
| 147 | P2 | Idea | Cross-device sync has no conflict-ready event model. | Keep immutable event IDs if sync is a future goal. |
| 148 | P3 | Idea | User annotations cannot be separated from imported deck content. | Store overlays keyed by stable card ID. |
| 149 | P3 | Idea | State cannot support multiple learner profiles. | Namespace progress by explicit local profile. |
| 150 | P3 | Idea | There is no privacy-preserving data deletion report. | Show which files were removed and which external sources remain untouched. |

## 7. Selection and scheduling

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 151 | P0 | Problem | Selection has no due-card policy. | Add a scheduler port that returns due reviews before new cards. |
| 152 | P1 | Risk | Weighted random behavior is not reproducible for debugging. | Inject an RNG/seed and record it in deterministic tests. |
| 153 | P1 | Problem | Frequency weighting can overexpose already familiar common words. | Combine corpus rank with personal mastery. |
| 154 | P1 | Problem | Recap chance is global rather than state-dependent. | Derive recap demand from overdue and weak cards. |
| 155 | P1 | Risk | The recent window uses deck size heuristics, not learning goals. | Make repeat spacing an explicit policy with tests. |
| 156 | P1 | Improvement | New-card order cannot be sequential, shuffled, or tagged. | Add named selection strategies behind the existing port. |
| 157 | P1 | Improvement | There is no daily new-card limit. | Track study day boundaries and cap introductions. |
| 158 | P1 | Improvement | There is no daily review limit or overflow policy. | Expose a bounded queue with transparent carryover. |
| 159 | P1 | Risk | Empty and one-card decks are safe but offer no explanatory state. | Surface an empty/single-card source status to controls. |
| 160 | P1 | Improvement | Suspended or ignored cards cannot be excluded. | Add scheduler-level eligibility filters. |
| 161 | P1 | Improvement | Difficulty bands cannot be balanced. | Add optional quota/stratified selection by level. |
| 162 | P1 | Risk | Random recap can choose a card with no pedagogical reason. | Score candidates by due time, lapses, and recency. |
| 163 | P1 | Improvement | The scheduler cannot avoid same-family interference. | Accept relation metadata and minimum family spacing. |
| 164 | P2 | Improvement | No deterministic tie-breaker is specified. | Order equal scores by stable card ID. |
| 165 | P2 | Improvement | Selection cannot explain why a card appeared. | Return a small reason enum for diagnostics and tests. |
| 166 | P2 | Risk | Strategy changes could radically reorder reviews after upgrade. | Version scheduler parameters and migration behavior. |
| 167 | P2 | Improvement | Session fatigue does not lower introduction rate. | Add active-session exposure caps and recovery windows. |
| 168 | P2 | Improvement | Time-of-day cannot influence pacing or card difficulty. | Add an opt-in schedule policy, not hidden heuristics. |
| 169 | P2 | Improvement | Speech-enabled cards are not spaced to prevent audio overlap. | Let speaker availability influence advance handling. |
| 170 | P2 | Risk | Unknown frequency receives an extreme rarity interpretation. | Use an explicit neutral/default rank policy. |
| 171 | P2 | Improvement | The recent set/window invariants are internal only. | Expose policy tests across deck sizes and recap settings. |
| 172 | P3 | Idea | A user cannot prioritize tagged goals temporarily. | Add bounded boosts that decay after a session. |
| 173 | P3 | Idea | There is no scheduler simulation report. | Run synthetic histories to inspect interval and workload distributions. |
| 174 | P3 | Idea | Selection cannot optimize context diversity. | Score examples/topics to avoid repetitive contexts. |
| 175 | P3 | Idea | There is no pluggable research scheduler boundary. | Keep scheduling traits pure and serialize policy identity. |

## 8. Timing and session behavior

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 176 | P1 | Risk | Interval, reveal delays, and fades can form incoherent timelines. | Validate cross-field relationships and report adjustments. |
| 177 | P1 | Risk | Translation can be configured to reveal after the word swaps. | Clamp or warn when reveal exceeds effective dwell. |
| 178 | P1 | Risk | Example reveal can also occur after the swap. | Derive a bounded timeline from one validated policy. |
| 179 | P1 | Improvement | Pause state is not reflected in tray text. | Update the control label and optional status line immediately. |
| 180 | P1 | Risk | System sleep/wake semantics are undefined. | Decide whether elapsed wall time advances or freezes and test it. |
| 181 | P1 | Risk | Clock changes are untested for any future persisted schedule. | Separate monotonic presentation time from UTC review time. |
| 182 | P1 | Improvement | Next Word during pause has no documented timer semantics. | Specify and test whether pause remains and the new card is frozen. |
| 183 | P1 | Improvement | Repeated Next commands can spawn overlapping speech. | Serialize/cancel speech and debounce only where appropriate. |
| 184 | P1 | Risk | A one-second minimum interval can make reveal settings meaningless. | Add a timeline validator or minimum derived from reveal requirements. |
| 185 | P2 | Improvement | Jitter uses a uniform distribution only. | Offer named cadence policies with deterministic tests. |
| 186 | P2 | Risk | Very large jitter often clamps at one second and skews the distribution. | Clamp jitter relative to base interval or sample within valid bounds. |
| 187 | P2 | Improvement | Rare-word dwell rounds to whole seconds. | Preserve sub-second duration math until the final `Duration`. |
| 188 | P2 | Improvement | Exit fade is capped silently at half the interval. | Report effective values in diagnostics/preferences. |
| 189 | P2 | Improvement | Reduced motion is config-only, not system-aware. | Combine explicit override with macOS accessibility preference. |
| 190 | P2 | Improvement | Reveal opacity animation has no reduced-transparency option. | Add a separate instant-reveal setting if users need it. |
| 191 | P2 | Risk | Width transition duration is a fixed design token. | Keep it theme-owned but expose tested preset values if needed. |
| 192 | P2 | Improvement | Session duration and exposure count are not tracked. | Add local session summaries once persistence exists. |
| 193 | P2 | Improvement | Quiet hours cannot defer due reviews gracefully. | Resume with a bounded catch-up policy rather than a burst. |
| 194 | P2 | Risk | Benchmark warm-up and measurement durations are hard-coded. | Make both explicit options and include them in structured output. |
| 195 | P2 | Improvement | Benchmark output lacks elapsed milliseconds and configuration identity. | Emit a structured one-line report. |
| 196 | P2 | Risk | `Instant` transitions are tested but App still calls `Instant::now()` directly. | Inject a clock into orchestration tests. |
| 197 | P3 | Idea | Different card types cannot have different reveal choreography. | Make timeline policy depend on prompt type. |
| 198 | P3 | Idea | The app cannot align cadence to natural work breaks. | Add opt-in Pomodoro/calendar-aware scheduling adapters. |
| 199 | P3 | Idea | No gentle catch-up exists after a long pause. | Resume at normal pace and prioritize due cards without rapid firing. |
| 200 | P3 | Idea | Timing policies have no visual simulator. | Add a developer preview that scrubs elapsed time deterministically. |

## 9. Visual design and layout

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 201 | P1 | Problem | Example truncation is character-count based, not measured-width based. | Elide at word boundaries using the active font and available width. |
| 202 | P1 | Risk | Long primary lines can shrink to an unreadably small size to fit. | Define per-line minimum sizes, then ellipsize or expand within safe bounds. |
| 203 | P1 | Problem | Headword, translation, IPA, and example share one generic overflow behavior. | Define explicit shrink/wrap/elide policy for each semantic line. |
| 204 | P1 | Risk | Card height responds to font scale but not unusually tall content. | Measure final content and clamp a responsive height range. |
| 205 | P1 | Problem | No visual snapshots protect themes, corners, or reveal states. | Add deterministic render fixtures and pixel/bounds comparison. |
| 206 | P1 | Risk | Graphite contrast is not evaluated against the actual wallpaper. | Add optional high-contrast auto mode or conservative surface floor. |
| 207 | P1 | Risk | Paper forces a high opacity that may surprise users who set opacity lower. | Show effective opacity in preferences and document preset floors. |
| 208 | P1 | Improvement | Theme choices have no preview surface. | Add a live preferences preview with representative long content. |
| 209 | P1 | Risk | User accent colors are not checked against each theme. | Compute contrast and auto-adjust or warn. |
| 210 | P1 | Improvement | Custom semantic theme tokens cannot be supplied. | Support a validated advanced theme file after presets stabilize. |
| 211 | P2 | Risk | The UI does not account for Dock/menu-bar/notch safe areas. | Position within the usable screen frame. |
| 212 | P2 | Improvement | Horizontal and vertical margins cannot be customized. | Add bounded per-axis margins with preview. |
| 213 | P2 | Improvement | The target word is not emphasized inside its example. | Paint matched spans with semantic weight, not raw substring hacks. |
| 214 | P2 | Improvement | Part-of-speech/sense context has no visual slot. | Reserve a quiet metadata line or label when the model supports it. |
| 215 | P2 | Improvement | Paused state has no subtle visual affordance. | Add an optional low-salience pause glyph/status outside learning content. |
| 216 | P2 | Improvement | Fallback/source-degraded state is visually invisible. | Surface it in controls, not as persistent card decoration. |
| 217 | P2 | Risk | Theme tests assert alpha/hierarchy but not actual WCAG contrast ratios. | Add luminance/contrast tests for every semantic token. |
| 218 | P2 | Risk | Text shrink changes size without adjusting measured vertical centering. | Use one fitted layout result for both height budgeting and paint. |
| 219 | P2 | Risk | Maximum card width grows with font scale up to a large desktop value. | Use viewport- and reading-length-aware width constraints. |
| 220 | P2 | Improvement | The example is always one line even when scale and viewport allow two. | Add an optional two-line comfortable layout with bounded height. |
| 221 | P2 | Improvement | There is no compact layout preset. | Add compact/default/comfortable density tokens as coordinated systems. |
| 222 | P2 | Improvement | Theme surface, typography, and motion cannot be previewed at timeline states. | Build a scrubbed design harness for 0/reveal/settled/exit frames. |
| 223 | P3 | Idea | Frequency or review state has no quiet visual cue. | Test a restrained semantic cue and reject it if it adds dashboard noise. |
| 224 | P3 | Idea | Card placement cannot avoid visually busy wallpaper regions. | Explore optional contrast sampling with privacy-safe local pixels. |
| 225 | P3 | Idea | There is no documented visual acceptance checklist. | Record hierarchy, fit, contrast, motion, and safe-area criteria. |

## 10. Accessibility

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 226 | P1 | Problem | Reduced motion is a config toggle, not synced with macOS settings. | Query the system preference with explicit override semantics. |
| 227 | P1 | Problem | Enhanced contrast is config-only. | Respect macOS Increase Contrast unless the user overrides it. |
| 228 | P1 | Problem | The app does not follow light/dark system appearance. | Add an Auto theme that listens for appearance changes. |
| 229 | P1 | Risk | Font loading probes one hard-coded supplemental font path. | Use a platform font resolver with several IPA/Cyrillic-capable fallbacks. |
| 230 | P1 | Risk | Missing IPA glyphs fail silently. | Detect missing coverage and report a diagnostics warning. |
| 231 | P1 | Problem | A click-through overlay exposes no VoiceOver interaction model. | Keep content passive but make tray/preferences fully accessible. |
| 232 | P1 | Problem | There are no global keyboard commands for pause, next, hide, or speech. | Add configurable, conflict-aware shortcuts. |
| 233 | P1 | Risk | Semantic text contrast is not tested numerically. | Add contrast-ratio tests across all presets and contrast modes. |
| 234 | P1 | Risk | Accent contrast is unconstrained. | Enforce a minimum contrast against the active surface. |
| 235 | P1 | Risk | Automatic text fitting can produce text below an accessible minimum. | Prefer layout growth/elision once the minimum readable size is reached. |
| 236 | P2 | Improvement | Font scale is capped at 1.5 with no rationale or larger layout. | Validate a larger accessibility range or document the cap. |
| 237 | P2 | Improvement | There is no reduced-transparency preference. | Offer an opaque surface mode independent of enhanced contrast. |
| 238 | P2 | Improvement | There is no instant-reveal/no-fade option. | Separate motion and opacity preferences. |
| 239 | P2 | Improvement | Line spacing cannot be increased independently. | Add a comfortable density preset rather than isolated spacing knobs. |
| 240 | P2 | Improvement | A dyslexia-friendly font option is unavailable. | Support user-selected fonts with coverage validation. |
| 241 | P2 | Problem | UI/control labels are English-only. | Introduce localized resources and retain deck-language independence. |
| 242 | P2 | Risk | Right-to-left content/layout is untested. | Add bidi fixtures and semantic alignment rules. |
| 243 | P2 | Risk | Mixed Cyrillic/Latin/IPA baselines are not visually regression-tested. | Add representative multilingual snapshots. |
| 244 | P2 | Improvement | Speech voice/rate cannot be selected for accessibility. | Expose bounded system voice and rate choices. |
| 245 | P2 | Risk | Rapid manual Next actions can create an audio accessibility problem. | Cancel/queue speech and expose mute feedback. |
| 246 | P2 | Improvement | There is no high-visibility focusable status window. | Add an optional accessible companion panel. |
| 247 | P2 | Improvement | Color is the only potential accent distinction. | Keep all meaning available through text/shape as well. |
| 248 | P3 | Idea | No accessibility audit is part of release criteria. | Add keyboard, VoiceOver, contrast, motion, and scaling checks. |
| 249 | P3 | Idea | User-specific reading speed cannot tune reveals. | Add a simple reading pace preset grounded in measured timelines. |
| 250 | P3 | Idea | The app cannot announce new words through assistive channels selectively. | Explore opt-in VoiceOver announcements without stealing focus. |

## 11. Tray, preferences, and interaction

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 251 | P1 | Problem | The Pause/Resume menu label never reflects current state. | Set the item text to the next available action. |
| 252 | P1 | Problem | Current word/source/deck state is absent from the tray. | Add concise disabled status rows or a diagnostics window. |
| 253 | P1 | Problem | Config changes require restart. | Add validated reload and atomic apply behavior. |
| 254 | P1 | Problem | There is no preferences window. | Build a compact, keyboard-accessible settings surface. |
| 255 | P1 | Risk | Tray event watcher lifetime is detached and implicit. | Own a shutdown signal and join/terminate policy. |
| 256 | P1 | Improvement | Hide and Pause are conflated by the lack of Hide. | Add independent visibility and learning-clock controls. |
| 257 | P1 | Improvement | Speech has no tray toggle. | Add checked mute/speak state with immediate effect. |
| 258 | P1 | Improvement | Corner changes require file edit and restart. | Add a Corner submenu with checked state. |
| 259 | P1 | Improvement | Theme changes require file edit and restart. | Add a Theme submenu or preferences preview. |
| 260 | P1 | Improvement | Interval changes are inaccessible during use. | Offer a few named cadence presets plus Custom in preferences. |
| 261 | P1 | Improvement | Reload Words is unavailable. | Add reload with progress, cancellation, and result feedback. |
| 262 | P1 | Improvement | There is no Retry Source action after fallback. | Add one-shot retry without restarting the overlay. |
| 263 | P2 | Risk | Menu commands are compared through raw external IDs in `App`. | Map them once to a small application command enum. |
| 264 | P2 | Improvement | Menu state cannot be unit-tested without muda IDs. | Put command interpretation behind an adapter. |
| 265 | P2 | Improvement | Next Word has no optional keyboard shortcut display. | Register shortcuts and let native menu rendering show them. |
| 266 | P2 | Improvement | Quit offers no state-flush feedback once persistence exists. | Flush safely and report unrecoverable save failure. |
| 267 | P2 | Improvement | There is no Open Config/Open Data Folder command. | Add platform-resolved paths to an Advanced submenu. |
| 268 | P2 | Improvement | Invalid settings have no inline explanation. | Validate per field and preserve the last valid applied config. |
| 269 | P2 | Improvement | Defaults cannot be restored by section. | Add scoped reset with preview and confirmation. |
| 270 | P2 | Improvement | The sample config cannot be generated from the running app. | Add Export Settings or Write Example Config. |
| 271 | P2 | Risk | A dense preferences window could violate the calm product identity. | Use restrained grouped settings and progressive disclosure. |
| 272 | P3 | Idea | Recent words could clutter the top-level tray. | Keep history in a submenu/companion panel with a strict limit. |
| 273 | P3 | Idea | There is no temporary Snooze command. | Add 15m/1h/Until Tomorrow without changing long-term settings. |
| 274 | P3 | Idea | No menu action copies the current word/details. | Add Copy Current Word in the tray. |
| 275 | P3 | Idea | Controls cannot expose why a word was selected. | Add a developer/diagnostics explanation, not card chrome. |

## 12. Platform and display integration

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 276 | P0 | Problem | Initial viewport size is hard-coded to 3840x2160. | Resolve the target monitor before showing or resize from a hidden bootstrap. |
| 277 | P0 | Problem | Monitor size is applied only on first UI start. | React to monitor, scale, and viewport changes. |
| 278 | P1 | Problem | Usable screen frame is ignored. | Account for Dock, menu bar, notch, and reserved regions. |
| 279 | P1 | Problem | Multi-monitor selection is unavailable. | Persist a monitor identity with robust fallback. |
| 280 | P1 | Risk | Display hot-plug can leave the overlay off-screen or mis-sized. | Re-evaluate placement on topology events. |
| 281 | P1 | Risk | Changing display scale/rotation is untested. | Add integration checks and recompute geometry. |
| 282 | P1 | Problem | Config path uses an XDG-style location on macOS. | Resolve Application Support/Preferences paths through `platform`. |
| 283 | P1 | Problem | Font lookup lives in the render module. | Move font discovery behind a platform capability. |
| 284 | P1 | Risk | Speech is implemented as `say` subprocesses spawned per word. | Add a long-lived speaker worker with cancel/replace semantics. |
| 285 | P1 | Problem | Non-macOS speech failure is silent. | Report unsupported capability once and disable the control. |
| 286 | P1 | Improvement | System voice and locale are not selected explicitly. | Resolve a matching voice and expose validated overrides. |
| 287 | P1 | Improvement | Focus/Do Not Disturb is not observed. | Add an opt-in platform quiet-state port. |
| 288 | P1 | Improvement | System appearance changes are not observed. | Subscribe to appearance and accessibility notifications. |
| 289 | P1 | Risk | Spaces/fullscreen behavior is undocumented and untested. | Define collection behavior and test across macOS Spaces. |
| 290 | P1 | Risk | Stage Manager interactions are untested. | Verify overlay level, click-through, and monitor placement. |
| 291 | P2 | Improvement | Launch at Login is unsupported. | Add an opt-in ServiceManagement integration. |
| 292 | P2 | Risk | Multiple instances can create duplicate overlays and tray items. | Add a single-instance guard with activation behavior. |
| 293 | P2 | Improvement | App activation policy/menu-bar-only behavior is not explicit. | Configure and document the intended LSUIElement lifecycle. |
| 294 | P2 | Risk | Screen capture/presentation privacy behavior is undefined. | Add hide-on-share or manual presentation mode. |
| 295 | P2 | Improvement | Power/battery state cannot influence cadence. | Add an opt-in low-power policy. |
| 296 | P2 | Improvement | Wake-from-sleep source refresh is absent. | Revalidate monitor/source state after wake. |
| 297 | P2 | Risk | Retina/non-Retina transitions may expose geometry assumptions. | Test logical points across scale factors. |
| 298 | P3 | Idea | Wallpaper contrast adaptation has no platform boundary. | Add a privacy-preserving local sampler port if justified. |
| 299 | P3 | Idea | The app cannot follow the active display. | Offer follow-active-monitor as an explicit mode. |
| 300 | P3 | Idea | Platform-specific capabilities lack a feature matrix. | Document supported/degraded behavior per OS. |

## 13. Performance and resource use

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 301 | P0 | Risk | The zero-idle promise is not enforced in CI. | Run a stable benchmark/scheduler assertion with a regression threshold. |
| 302 | P1 | Risk | A full-screen transparent viewport is expensive by construction. | Measure compositor/GPU cost and explore a card-sized click-through window. |
| 303 | P1 | Risk | The initial 3840x2160 surface may exceed the actual display. | Bootstrap with real monitor geometry. |
| 304 | P1 | Problem | MongoDB, Tokio, DNS, and TLS are unconditional dependencies. | Feature-gate remote sources and ship an offline build profile. |
| 305 | P1 | Risk | Mongo loading creates a Tokio runtime for a single synchronous call. | Own one async runtime/worker or remove it from offline builds. |
| 306 | P1 | Risk | Every spoken word can create a new thread and subprocess. | Use one bounded speaker worker with latest-wins cancellation. |
| 307 | P1 | Risk | Rapid Next actions can accumulate speech processes. | Cancel the previous process before starting the next utterance. |
| 308 | P1 | Improvement | Font file I/O occurs during UI startup. | Resolve/cache font bytes before creating the visible viewport. |
| 309 | P1 | Risk | Text is measured repeatedly for width and vertical budget during animation. | Cache fitted galleys/layout metrics per card/theme/scale. |
| 310 | P1 | Risk | Example text is trimmed/truncated into a new `String` repeatedly. | Precompute rendered card content on word change. |
| 311 | P1 | Improvement | Card width recomputes all line measurements every animated frame. | Cache target widths and interpolate numeric stages. |
| 312 | P1 | Improvement | Theme/layout values are copied and recomputed only once, but this is implicit. | Keep a documented immutable render spec per config revision. |
| 313 | P2 | Risk | Benchmark has a warm-up but reports no per-second distribution or wake reason. | Report measurement metadata and optional wake diagnostics. |
| 314 | P2 | Problem | Benchmark has no automated pass/fail threshold. | Store a conservative baseline and alert on large regressions. |
| 315 | P2 | Improvement | No frame-time histogram is available during animation. | Add opt-in developer timing counters. |
| 316 | P2 | Improvement | CPU, GPU, memory, wakeups, and battery are not profiled together. | Define a release performance checklist using macOS Instruments. |
| 317 | P2 | Risk | Release `opt-level = z` favors size without measured runtime impact. | Benchmark `z`, `s`, and `3` for startup/frame cost and binary size. |
| 318 | P2 | Risk | WGPU backend choice is not measured or documented. | Record backend/device information in diagnostics and test supported Macs. |
| 319 | P2 | Improvement | Source snapshots are fully materialized before deck creation. | Add bounded loading and evaluate streaming only if decks become large. |
| 320 | P2 | Improvement | Selector allocations/weights are not benchmarked on large decks. | Add criterion-style pure benchmarks for 1k/10k/100k cards. |
| 321 | P2 | Risk | Recent-set sizing may waste memory for very large decks. | Bound policy and benchmark memory per deck size. |
| 322 | P2 | Improvement | No startup latency budget is defined. | Measure cold/warm launch to first visible fallback card. |
| 323 | P2 | Improvement | No binary-size budget protects the small ambient utility identity. | Track compressed/uncompressed artifacts per release. |
| 324 | P3 | Idea | Low Power Mode cannot lower animation cadence. | Add an opt-in 30 FPS animation cadence under constrained power. |
| 325 | P3 | Idea | Static settled content could avoid a full-screen surface entirely. | Prototype a dynamically resized card viewport and compare behavior. |

## 14. Reliability and error handling

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 326 | P0 | Problem | A source worker panic/disconnect becomes generic stderr instead of a typed report. | Catch worker termination and deliver a structured internal failure outcome. |
| 327 | P1 | Problem | Config read failure is indistinguishable from no config. | Return load status and show path/parse warnings. |
| 328 | P1 | Problem | Unknown and malformed config values are silently ignored. | Preserve safe defaults but report line-numbered warnings. |
| 329 | P1 | Risk | Theme/config cross-field inconsistencies are not validated. | Add a post-parse validation report with effective values. |
| 330 | P1 | Problem | Font failure is silent. | Report missing paths/coverage and chosen fallback. |
| 331 | P1 | Risk | Speech subprocess spawn/wait failures are ignored. | Return speaker outcomes and surface a once-per-session warning. |
| 332 | P1 | Risk | Tray/menu partial failure can leave commands missing silently except stderr. | Build a structured startup capability report. |
| 333 | P1 | Risk | Detached worker panics are not observed. | Wrap workers, report failure, and own lifecycle handles. |
| 334 | P1 | Risk | Menu watcher has no explicit shutdown path. | Add cancellation and deterministic termination. |
| 335 | P1 | Risk | Benchmark thread sleeps independently of app lifecycle. | Drive benchmark completion through owned state/timers. |
| 336 | P1 | Risk | Selector assumes non-empty candidates with `expect`. | Encode non-empty input or return `Option` through the strategy. |
| 337 | P1 | Improvement | Startup failures have no common severity taxonomy. | Define recoverable/degraded/fatal error categories. |
| 338 | P1 | Improvement | stderr messages lack stable context fields. | Add a tiny structured logging facade. |
| 339 | P1 | Risk | There is no panic hook or crash diagnostics guidance. | Install a redacted panic report path and document recovery. |
| 340 | P2 | Risk | No watchdog detects a stalled source/speaker worker. | Add bounded timeouts and health state, not a busy monitor. |
| 341 | P2 | Improvement | Effective fallback source is not retained in application state. | Store a source status object alongside the deck. |
| 342 | P2 | Risk | Reloading future config could partially apply changes. | Parse/validate a complete candidate, then swap atomically. |
| 343 | P2 | Risk | Future deck replacement could invalidate current indices/history. | Replace by stable IDs and explicit transition rules. |
| 344 | P2 | Improvement | Errors cannot be copied/exported from the UI. | Add a redacted diagnostics action. |
| 345 | P2 | Risk | System sleep and display reconfiguration recovery are untested. | Add lifecycle event handling and manual test cases. |
| 346 | P2 | Improvement | There is no safe mode after repeated startup failure. | Start fallback-only with defaults and expose the degraded reason. |
| 347 | P2 | Risk | Partial/corrupt bundled deck behavior is undefined. | Validate embedded assets at build and startup. |
| 348 | P2 | Improvement | Multiple identical warnings could become noisy after reload support. | Deduplicate warnings by cause and config revision. |
| 349 | P3 | Idea | Recovery actions are not modeled as commands. | Attach Retry/Open Path/Reset actions to typed diagnostics. |
| 350 | P3 | Idea | No reliability SLO exists for this ambient utility. | Define first-card success, idle stability, and clean-quit targets. |

## 15. Architecture, SOLID, and DRY

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 351 | P1 | Problem | `App` still owns menu polling, benchmark logic, screen setup, cadence, and rendering. | Extract application commands, benchmark adapter, and viewport controller. |
| 352 | P1 | Problem | `main.rs` constructs tray menus procedurally. | Move tray creation into an adapter returning command IDs/handle. |
| 353 | P1 | Problem | `platform.rs` contains only speech while paths/fonts/display logic is scattered. | Define narrow platform ports by capability. |
| 354 | P1 | Problem | `WordSource` expresses outcomes and metadata but has no cancellation contract. | Add a small cancellation/lifecycle port before supporting repeated loads. |
| 355 | P1 | Risk | `App` depends directly on `muda::MenuId`. | Translate adapter events into a domain-neutral command enum. |
| 356 | P1 | Risk | `App` creates randomness directly with `rand::rng()`. | Inject an interval-jitter policy or RNG facade. |
| 357 | P1 | Risk | `App` calls `Instant::now()` directly despite a testable session clock. | Inject a clock at the orchestration boundary. |
| 358 | P1 | Risk | `Theme` is coupled to egui color/shadow types. | Keep render tokens in the render layer and isolate pure palette math if reused. |
| 359 | P1 | Improvement | Theme preset construction and token behavior share one file. | Split palette presets from semantic token/model tests when it grows further. |
| 360 | P1 | Risk | `CardView` still performs geometry, fitting, composition, and painting. | Extract a pure `CardLayout` result consumed by the painter. |
| 361 | P1 | Improvement | Fitted text is computed separately from vertical budgeting. | Produce one reusable line-layout object per semantic line. |
| 362 | P1 | Risk | `Config` parsing knows every key through one large match. | Delegate each group to a small parser while retaining one registry. |
| 363 | P1 | Improvement | Config key metadata is duplicated across parser, example, and README. | Define a declarative key registry or verification test. |
| 364 | P1 | Improvement | Fallback records are handwritten separately from the seed. | Generate adapters from one canonical data source. |
| 365 | P2 | Risk | `Deck` owns both rotation history and recap policy details. | Separate eligibility/history from scheduling policy if SRS lands. |
| 366 | P2 | Improvement | Boxed selector dispatch is used where generic/static composition may suffice. | Benchmark and choose based on extension needs, documenting the tradeoff. |
| 367 | P2 | Risk | Module boundaries are documented but not dependency-tested. | Add a check that core modules do not import UI/OS/database crates. |
| 368 | P2 | Improvement | No library crate exposes the pure core for integration tests/tools. | Move domain modules to `lib.rs`, keep `main.rs` as composition root. |
| 369 | P2 | Improvement | Application commands have no central enum/state transition function. | Introduce command handling independent of tray transport. |
| 370 | P2 | Risk | Worker lifecycle ownership is implicit across App/platform/source. | Add explicit handles, cancellation, and shutdown ordering. |
| 371 | P2 | Improvement | Error presentation policy is duplicated through `eprintln!`. | Centralize reporting while preserving typed errors. |
| 372 | P2 | Improvement | Architecture decisions live mainly in prose and commit history. | Add ADRs for overlay shape, source fallback, storage, and scheduling. |
| 373 | P2 | Risk | Public module interfaces have few compile-time visibility boundaries. | Prefer `pub(crate)`/private APIs and boundary tests. |
| 374 | P3 | Idea | Feature work lacks vertical slice templates. | Document the path model -> policy -> adapter -> UI -> tests -> docs. |
| 375 | P3 | Idea | Dependency direction is not visualized automatically. | Generate/check a small module dependency graph in CI. |

## 16. Testing and quality assurance

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 376 | P0 | Problem | There are no end-to-end startup/tray/quit tests. | Add a macOS smoke harness around the built app bundle. |
| 377 | P0 | Problem | The first visible frame is not tested. | Verify fallback content appears within a startup budget. |
| 378 | P0 | Problem | Idle repaint behavior is not a CI gate. | Add deterministic scheduler tests plus a platform benchmark job. |
| 379 | P1 | Problem | There are no visual regression snapshots. | Render representative themes/scales/timeline states and compare outputs. |
| 380 | P1 | Problem | Mongo success/failure/malformed-cursor paths lack adapter tests. | Use a controlled test backend or extract cursor mapping. |
| 381 | P1 | Problem | Config path precedence is not tested with isolated environment/filesystem. | Inject path/env providers and test all lookup outcomes. |
| 382 | P1 | Risk | App pause tests cover `SessionClock`, not the full command/render path. | Add orchestration tests with fake clock and commands. |
| 383 | P1 | Risk | Next/Pause/Quit command mapping is not unit-tested independently. | Introduce a command enum and adapter tests. |
| 384 | P1 | Problem | Tray partial-failure behavior has no tests. | Abstract tray construction and simulate failed menu/icon steps. |
| 385 | P1 | Problem | Font fallback/coverage behavior has no tests. | Inject font candidates and verify selected coverage/reporting. |
| 386 | P1 | Risk | Long word/translation fitting has only arithmetic helper coverage. | Render extreme multilingual fixtures and assert bounds. |
| 387 | P1 | Risk | Card placement is not tested on tiny/portrait/ultrawide viewports. | Add geometry tests for representative screens and scales. |
| 388 | P1 | Risk | Theme contrast is not numerically tested. | Compute ratios for all semantic tokens/presets. |
| 389 | P1 | Problem | Release bundle contents are not smoke-tested. | Inspect plist, binary, icon, permissions, and launch result. |
| 390 | P1 | Risk | Seed/fallback consistency has no CI validator. | Generate both and fail on duplicate/drift. |
| 391 | P2 | Improvement | No property tests exercise parser numeric/Unicode edge cases. | Add bounded property tests for finite/clamped outcomes. |
| 392 | P2 | Improvement | Deck invariants are tested with examples but not generated sequences. | Property-test no-repeat/window consistency across sizes/seeds. |
| 393 | P2 | Improvement | Scheduler distribution tests can be flaky without fixed RNG seeds. | Inject deterministic RNG everywhere under test. |
| 394 | P2 | Risk | Thread/process shutdown is untested. | Add bounded lifecycle tests with fake workers. |
| 395 | P2 | Improvement | No mutation testing estimates assertion strength. | Run targeted mutation checks on parser/timing/deck modules. |
| 396 | P2 | Improvement | CI verifies audit/config counts but not Markdown link targets. | Add a local-link checker to the existing documentation invariant step. |
| 397 | P2 | Improvement | Test count dropped during refactor without a coverage metric. | Track behavior coverage/critical cases rather than raw count alone. |
| 398 | P2 | Risk | CI covers only the latest macOS runner. | Add a small supported-version matrix where practical. |
| 399 | P3 | Idea | No manual exploratory checklist covers Spaces/Stage Manager/Focus. | Maintain a release QA matrix for platform behaviors. |
| 400 | P3 | Idea | There is no canary/beta channel. | Add prerelease artifacts before risky scheduler/storage migrations. |

## 17. Security and privacy

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 401 | P0 | Risk | Remote Mongo configuration has no secure credential/TLS workflow. | Add secure URI/env support, TLS guidance, and redaction. |
| 402 | P1 | Risk | Future URI diagnostics could expose passwords or tokens. | Centralize secret-aware formatting before adding structured logs. |
| 403 | P1 | Problem | The project has no `SECURITY.md`. | Publish supported versions and a private reporting path. |
| 404 | P1 | Risk | Config/state file permissions are not enforced. | Create sensitive files with user-only permissions and warn otherwise. |
| 405 | P1 | Risk | Untrusted source strings have no control-character policy. | Reject or escape bidi overrides, terminal controls, and nulls. |
| 406 | P1 | Risk | Imported string/record sizes are unbounded. | Enforce conservative field, record, and deck limits. |
| 407 | P1 | Risk | Unicode normalization is absent, enabling deceptive duplicate IDs. | Normalize identifiers and detect confusable collisions. |
| 408 | P1 | Risk | Source provenance is not retained. | Attach source and license metadata to every imported record. |
| 409 | P1 | Risk | Speech receives arbitrary imported word text as a process argument. | Validate length/control characters and use explicit option termination if supported. |
| 410 | P1 | Improvement | There is no threat model for local files, MongoDB, release assets, and updates. | Add a short scoped threat-model document. |
| 411 | P1 | Risk | Future diagnostics export could include secrets or personal history. | Define a redaction schema and test it before shipping export. |
| 412 | P1 | Improvement | Privacy behavior is not summarized for users. | State clearly: local config, optional MongoDB, no telemetry, no cloud sync. |
| 413 | P1 | Risk | Dependency actions are pinned to mutable major tags, not commit SHAs. | Pin third-party GitHub Actions and automate updates. |
| 414 | P1 | Risk | License checks are intentionally skipped in `cargo-deny`. | Add an allow-list or a separate license inventory report. |
| 415 | P1 | Risk | Ignored RustSec advisories can become relevant if target scope expands. | Document target conditions and periodically re-evaluate each ignore. |
| 416 | P2 | Improvement | No SBOM accompanies releases. | Publish SPDX/CycloneDX metadata with artifacts. |
| 417 | P2 | Risk | There is no checksum/signature verification path for downloads. | Publish SHA-256 checksums and signed/notarized artifacts. |
| 418 | P2 | Risk | A future auto-updater has no trust design. | Require signed manifests, pinned origin, rollback, and explicit policy. |
| 419 | P2 | Improvement | Data deletion scope is undocumented. | Document config/cache/state/database responsibilities separately. |
| 420 | P2 | Risk | Imported examples may contain misleading bidi/invisible characters. | Make validation errors identify escaped code points. |
| 421 | P2 | Improvement | Clipboard export has no privacy policy because it is not yet designed. | Treat clipboard actions as explicit and never automatic. |
| 422 | P2 | Idea | Sandboxing/notarization entitlements are not planned. | Minimize entitlements and document why each is needed. |
| 423 | P3 | Idea | A security regression test corpus does not exist. | Add malicious config/deck fixtures and expected safe failures. |
| 424 | P3 | Idea | No dependency review policy exists for new crates. | Require maintenance, license, source, and necessity checks. |
| 425 | P3 | Idea | Privacy-sensitive integrations have no consent framework. | Define explicit opt-in boundaries before calendar/cloud features. |

## 18. Build, release, and supply chain

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 426 | P0 | Problem | Release artifacts are not code-signed or notarized. | Add Apple signing/notarization before broader distribution. |
| 427 | P0 | Problem | The release workflow does not launch-smoke the built app. | Run a bounded benchmark/smoke test on the assembled bundle. |
| 428 | P1 | Risk | Cargo and Info.plist versions can drift manually. | Generate/check plist version from Cargo metadata in CI. |
| 429 | P1 | Risk | Release tag existence check may run with shallow tag history. | Fetch tags explicitly or query GitHub before creating a release. |
| 430 | P1 | Problem | Release ZIP has no published checksum. | Generate and upload SHA-256 alongside it. |
| 431 | P1 | Improvement | No SBOM or dependency inventory is published. | Generate one from the locked release graph. |
| 432 | P1 | Risk | GitHub Actions are pinned to moving tags. | Pin SHAs with update automation. |
| 433 | P1 | Risk | Rust toolchain is floating `stable`. | Pin a tested toolchain and update deliberately. |
| 434 | P1 | Risk | Universal binary slices are built but not verified after `lipo`. | Inspect architectures and launch each supported path where possible. |
| 435 | P1 | Risk | Bundle metadata/resources are not validated. | Add plist parsing and bundle-structure assertions. |
| 436 | P1 | Improvement | Release notes rely only on generated GitHub text. | Curate user-facing changes, migrations, and known issues from CHANGELOG. |
| 437 | P1 | Risk | No reproducible-build guidance exists. | Pin toolchain/dependencies and record build environment. |
| 438 | P2 | Improvement | The project has no Homebrew cask/tap. | Add distribution only after signing, checksums, and stable releases. |
| 439 | P2 | Improvement | There is no prerelease channel. | Publish signed prereleases for storage/scheduler migrations. |
| 440 | P2 | Risk | Release workflow is manual but lacks a checklist gate. | Add version/changelog/tests/signing/smoke/checksum checks. |
| 441 | P2 | Improvement | Binary size and startup time are not compared per release. | Attach simple metrics to release CI. |
| 442 | P2 | Risk | Build scripts manually assemble the app bundle. | Evaluate cargo-bundle/cargo-dist while preserving explicit control. |
| 443 | P2 | Improvement | Offline/minimal builds are unavailable. | Feature-gate MongoDB/Tokio and test both feature sets. |
| 444 | P2 | Risk | `cargo deny` does not check licenses in CI. | Enable a maintained allow-list or equivalent report. |
| 445 | P2 | Improvement | CI validates audit counts and aliases, but not every Markdown link target. | Extend the documentation check with local-link validation. |
| 446 | P2 | Risk | Cache keys do not include the pinned toolchain because none exists. | Include toolchain and target dimensions. |
| 447 | P2 | Improvement | No changelog consistency check ensures current version sections. | Add release metadata validation. |
| 448 | P3 | Idea | There is no provenance attestation for artifacts. | Add GitHub artifact attestations/SLSA metadata. |
| 449 | P3 | Idea | Release rollback is undocumented. | Define yank/deprecate/reissue behavior for bad bundles. |
| 450 | P3 | Idea | No long-term support policy exists. | State supported macOS/app versions and update cadence. |

## 19. Documentation and developer experience

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 451 | P1 | Problem | README has no screenshot or short visual demo. | Add a representative capture for each major theme, without marketing clutter. |
| 452 | P1 | Problem | There is no contributor setup guide beyond basic Cargo commands. | Add macOS prerequisites, Mongo setup, checks, and bundle workflow. |
| 453 | P1 | Problem | The portable deck format is undocumented because it does not exist. | Specify schema, validation, examples, and versioning. |
| 454 | P1 | Problem | There is no troubleshooting guide. | Cover tray absence, fonts, Mongo fallback, permissions, and display placement. |
| 455 | P1 | Problem | There is no privacy/security overview. | Add concise user-facing data-flow and reporting docs. |
| 456 | P1 | Improvement | Architecture has no decision records. | Add ADRs for click-through overlay, source fallback, timing, storage, and themes. |
| 457 | P1 | Risk | Root alias files can look like independent editable documents. | Keep explicit canonical pointers and verify them in CI. |
| 458 | P1 | Improvement | The register has a best-next list but no owned current milestone/WIP slice. | Derive a small active milestone with owner and outcome from the 500 items. |
| 459 | P1 | Improvement | Modules have no consistent "why/read next" rustdoc. | Add short responsibility/dependency comments at facades. |
| 460 | P1 | Improvement | No diagram shows source -> deck -> app -> timeline -> card flow. | Keep one small architecture diagram synced with modules. |
| 461 | P2 | Problem | There is no `CONTRIBUTING.md`. | Document issue/branch/test/PR expectations. |
| 462 | P2 | Improvement | There are no issue/PR templates. | Add focused bug, feature, and release checklists. |
| 463 | P2 | Improvement | No glossary defines dwell, recap, reveal, exposure, and review. | Add a short learning/product glossary. |
| 464 | P2 | Improvement | Config keys are documented manually in several places. | Generate or verify tables from one metadata registry. |
| 465 | P2 | Risk | Design backlog can go stale when features ship. | Mark shipped IDs automatically or fold it into the canonical register. |
| 466 | P2 | Risk | Historical backlog can conflict with the current audit. | Label it historical and link to this canonical register. |
| 467 | P2 | Improvement | No developer recipe explains adding a config key end-to-end. | Document type, parser, tests, example, README, and changelog steps. |
| 468 | P2 | Improvement | No recipe explains adding a word source. | Document report/error/cancellation/fallback expectations. |
| 469 | P2 | Improvement | No recipe explains adding a theme safely. | Require hierarchy, contrast, scale, and snapshot checks. |
| 470 | P2 | Improvement | Public release installation/quarantine behavior is not explained. | Document current warning and future notarized path honestly. |
| 471 | P2 | Improvement | CHANGELOG Unreleased mixes many cycles without issue links. | Group by release intent and link material changes. |
| 472 | P2 | Improvement | No support matrix states macOS/font/Mongo capabilities. | Add a compact matrix with degraded behavior. |
| 473 | P3 | Idea | Pure modules lack runnable examples. | Add small examples for config parsing, scheduling, and deck loading. |
| 474 | P3 | Idea | Documentation has no automated style/lint policy. | Add lightweight Markdown formatting/link checks. |
| 475 | P3 | Idea | No roadmap explains what will deliberately remain out of scope. | State calm/click-through constraints and rejected feature classes. |

## 20. Diagnostics, roadmap, and ecosystem

| # | Pri | Kind | Finding | Recommended action |
|---:|:---:|---|---|---|
| 476 | P1 | Problem | Logs are ad hoc `eprintln!` messages. | Add structured levels/context with no default file logging. |
| 477 | P1 | Problem | Diagnostics cannot report effective config values. | Expose redacted parsed/effective settings and adjustments. |
| 478 | P1 | Problem | Source counts/skips/fallback causes are logged but not retained or exposed. | Store the latest redacted `LoadReport` in application health state. |
| 479 | P1 | Problem | Runtime/app/platform versions are absent from support output. | Include app, Rust target, macOS, GPU backend, and schema versions. |
| 480 | P1 | Improvement | Benchmark output is human-only text. | Add optional JSON for CI while preserving one-line text. |
| 481 | P1 | Improvement | There is no diagnostics command in the tray. | Add View/Copy Diagnostics with redaction. |
| 482 | P1 | Improvement | No health state distinguishes normal, degraded, retrying, and failed. | Model a small application health enum. |
| 483 | P1 | Risk | Roadmap priorities can become a flat 500-item queue. | Select one milestone with explicit outcomes and WIP limits. |
| 484 | P1 | Improvement | Completed audit items have no durable traceability. | Link fixes to commits/PRs and archive them outside the open 500. |
| 485 | P1 | Improvement | There is no stable plugin/integration boundary. | Define import/export and enrichment ports before SDK ambitions. |
| 486 | P1 | Idea | Anki import/export is unavailable. | Start with transparent CSV/JSON interchange, then evaluate APKG. |
| 487 | P1 | Idea | Learning history cannot be exported to other tools. | Add a documented, privacy-safe event export. |
| 488 | P2 | Idea | Wiktionary/dictionary enrichment is unavailable. | Add opt-in enrichment with caching, attribution, and rate limits. |
| 489 | P2 | Idea | Cloud sync is unavailable and architecturally premature. | First ship stable IDs, local events, migrations, and conflict tests. |
| 490 | P2 | Idea | iCloud/Syncthing-friendly storage is not evaluated. | Document atomic file behavior before recommending sync tools. |
| 491 | P2 | Idea | A companion progress view does not exist. | Build it only after meaningful persisted metrics exist. |
| 492 | P2 | Idea | Calendar/Focus integrations have no consent and failure model. | Keep them opt-in adapters with local-only defaults. |
| 493 | P2 | Improvement | No issue labels map to the 20 audit areas. | Add area/priority/kind labels for actionable tracking. |
| 494 | P2 | Improvement | No milestone maps recommendations to releases. | Create small outcome-based milestones, not 500 individual promises. |
| 495 | P2 | Idea | No extension can provide custom selection policy. | Keep a stable pure scheduler interface before exposing plugins. |
| 496 | P3 | Idea | Deck sharing/discovery is unavailable. | Define signing/provenance/moderation before any marketplace. |
| 497 | P3 | Idea | No local simulation compares scheduler workload. | Build a developer tool over synthetic/anonymized events. |
| 498 | P3 | Idea | No opt-in crash reporting strategy exists. | Prefer local reports; require explicit consent for remote submission. |
| 499 | P3 | Idea | No deprecation policy exists for config/deck schemas. | Version, warn, migrate, and publish removal timelines. |
| 500 | P3 | Idea | Success criteria for the product are not written down. | Define calmness, learning value, reliability, privacy, and resource budgets. |

## Best next 20 moves

1. Retain `LoadReport`, expose quiet source health, and add owned retry (#5, #18, #101-110, #341, #478, #482).
2. Add stable card IDs plus a versioned local `Store` (#1, #51, #126-138).
3. Define review events and a simple due-card scheduler (#2, #26-35, #151).
4. Replace the hard-coded viewport with monitor/usable-frame handling (#276-281).
5. Add a tray-opened preferences surface and live theme preview (#4, #208, #254).
6. Translate raw `MenuId` values to application commands (#263-264, #355, #369).
7. Move platform paths/fonts/display/speech behind focused ports (#282-288, #353).
8. Serialize and cancel speech through one worker (#183, #245, #284, #306-307).
9. Generate fallback and Mongo seed from one validated deck asset (#76-80, #390).
10. Add text-layout snapshots for long multilingual content (#202-205, #386-388).
11. Enforce contrast ratios and system accessibility preferences (#226-239).
12. Add startup/bundle smoke tests and an idle benchmark gate (#301, #376-389).
13. Sign, notarize, checksum, and validate release artifacts (#426-435).
14. Feature-gate MongoDB/Tokio for a small offline build (#304, #443).
15. Add secure Mongo configuration and redacted diagnostics (#401-412, #476-482).
16. Introduce a declarative config-key registry to stop doc drift (#363, #464, #467).
17. Move pure domain modules into a library crate (#367-368).
18. Add quiet hours, Snooze, and Focus-aware pausing (#13, #19, #273, #287).
19. Publish contributor, troubleshooting, privacy, and deck-format docs (#451-475).
20. Select one small milestone from this register and cap work in progress (#483-494).
