# Release Notes

## Unreleased

- Reworded the LSK leap-second warnings, which told users to do something
  impossible. Both the `TimeWarning::LskFutureFrozen` message and the CLI's
  `--stale-lsk-threshold-days` warning ended with "update `naif0012.tls`",
  implying the kernel was out of date. It is not: `naif0012.tls` is still
  NAIF's current release, its table ends at 2017-01-01 because that is the
  most recent leap second, and IERS has announced none since (Bulletin C 72,
  July 2026). The held `DELTA_AT=37s` is correct, not a degraded fallback.
  The messages now say so and point at Bulletin C instead of a re-download.
  Text only — the warning variants, their fields, the C ABI
  `DHRUV_TIME_WARNING_LSK_FUTURE_FROZEN` kind, and the Elixir
  `lsk_future_frozen` payload are unchanged, as is the (correct) EOP
  staleness advice.
- Fix: `gochar_events` missed some Rahu/Ketu transit-to-natal contacts.
  It carried its own per-body coarse-scan step table that had drifted from
  `TransitBody::default_ingress_step_days`, sampling the nodes every 2 days
  where every other scan path uses 1. The true node stations roughly weekly,
  and each direct excursion re-crosses longitudes it just passed, so a
  contact near a station is a pair of crossings often less than two days
  apart; a 2-day step can span the pair, see no sign change, and drop both.
  Where the node reaches a natal longitude only via such an excursion, the
  contact was reported as not happening at all. Measured against a
  0.0625-day reference over 2020-2024, the 2-day step lost every contact
  for ~0.8% of longitudes sampled near a node station (and ~4.6% of them
  had some contact missing); the 1-day step loses none. The duplicate table
  is gone — gochar now shares `TransitBody::default_ingress_step_days` with
  the ingress and fixed-longitude scans, so the paths cannot disagree
  again. Node scans cost ~1.9x more; on a 35-day window with Jupiter,
  Saturn, Rahu and Ketu this is under 1 ms per `gochar_events` call. No API
  change; affected node contacts now appear in results that previously
  omitted them.
- New search op `fixed_longitude`: when does a MOVING transit body
  next/previously reach a FIXED sidereal longitude, optionally offset by
  an angle set. Promotes the root-find that already powered gochar
  transit-to-natal aspect timing to a first-class op, so timeline
  consumers no longer need windowed `gochar_events` sweeps. Any
  `TransitBody` (plain bodies + Rahu/Ketu via `node_mode`); the longitude
  model and numerical knobs are the existing `SankrantiConfig` (frame,
  ayanamsha, step/convergence/max-iterations). `target_angles_deg` are
  offsets added to the target (mod 360, empty = conjunction only);
  `include_special_angles` additionally searches the body's classical
  special-aspect angles (Mars 90/210, Jupiter 120/240, Saturn 60/270)
  applied so the moving body casts that aspect onto the target. Events
  carry `{utc, jd_tdb, body, target/angle/matched longitude, sidereal +
  tropical longitude, actual_separation_deg}`. next/prev scans are
  bounded per body at 13 x `ingress_max_scan_days` (a specific longitude
  can take most of a zodiac lap, incl. retrograde loitering); range mode
  returns partial results when the window crosses the ephemeris coverage
  edge. Purely additive.
  - Core: `dhruv_search::fixed_longitude` (`FixedLongitudeOperation` /
    `FixedLongitudeQuery` / `FixedLongitudeResult`, plus split
    `next_fixed_longitude` / `prev_fixed_longitude` /
    `search_fixed_longitudes`); `dhruv_rs::fixed_longitude`
    (`FixedLongitudeRequest`).
  - C ABI (v88): `dhruv_fixed_longitude_search`
    (`DhruvFixedLongitudeRequest` / `DhruvFixedLongitudeEvent`,
    `DHRUV_FIXED_LONGITUDE_QUERY_MODE_*`,
    `DHRUV_MAX_FIXED_LONGITUDE_ANGLES = 16`).
  - Python (`next_fixed_longitude` / `prev_fixed_longitude` /
    `search_fixed_longitudes` in the search module), Go
    (`(*Engine).FixedLongitudeSearch`), Node (`fixedLongitudeSearch`).
  - Elixir `search_run` gains op `"fixed_longitude"` (semantic `mode`
    next/prev/range, `body`, `target_longitude_deg`, `target_angles_deg`,
    `include_special_angles`, `sankranti_config` +
    `config.step_size_days/max_iterations/convergence_days` overrides,
    per-body default step) via `CtaraDhruv.Search.fixed_longitude/2`.
  - CLI: new `fixed-longitude` subcommand.

- Ephemeris coverage misses now surface as the typed
  `EngineError::EpochOutOfRange` instead of `EngineError::Internal`
  (fix). The variant existed but was never constructed, so every
  coverage-edge tolerance in the search scans (sankranti, conjunction,
  stationary, charakaraka events — and the whole `gochar_events` op) was
  dead against real kernels: open-ended scans and window sweeps near the
  kernel edge errored out instead of ending with "no event" / partial
  results. With the fix, `gochar_events` windows overlapping the coverage
  edge return the events found up to the edge, and the C ABI now actually
  returns the long-documented `DHRUV_STATUS_EPOCH_OUT_OF_RANGE` (6) for
  out-of-coverage queries where it previously returned
  `DHRUV_STATUS_INTERNAL` (255). Range searches that require the full
  window in coverage (e.g. sankranti range) still error, now with the
  typed status.

- New range op `charakaraka_events`: the exact moments the chara-karaka
  ranking changes over a UTC range, per scheme (all four
  `CharakarakaScheme` values) and sidereal config, plus
  `next_charakaraka_event` / `prev_charakaraka_event`. Boundaries are
  root-found — rashi ingresses, pairwise degree-in-rashi crossings (with
  Rahu's reversed count expressed as the sum condition
  `d_Rahu + d_other = 30`, which a separation search cannot express), and,
  for `MixedParashara`, the integer-degree bin boundaries that flip the
  8↔7 mode (`scheme_mode_change` trigger with `used_eight_karakas`
  before/after). Events carry before/after rankings in the per-moment
  result shape, `changed_roles`, and a trigger discriminator; only actual
  ranking changes are emitted, simultaneous roots consolidate into one
  event, and range mode has the shared `max_events`/`truncated`/
  `next_from_utc` continuation contract (50,000 ceiling). The ranking
  order is now a documented contract: effective degree desc, then raw
  degrees-in-rashi desc, then graha index asc. Purely additive.
  - C ABI (v87): `dhruv_charakaraka_events` (+`_count`/`_at`/`_meta`/
    `_free`, `DhruvCharakarakaEventsHandle`,
    `DhruvCharakarakaChangeEvent` with `changed_roles_mask` bit-per-role,
    `DHRUV_MAX_CHARAKARAKA_EVENTS`, `DHRUV_CHARAKARAKA_TRIGGER_*`) and
    `dhruv_next_charakaraka_event` / `dhruv_prev_charakaraka_event`.
  - Python (`charakaraka_events` / `next_charakaraka_event` /
    `prev_charakaraka_event` in the kundali module), Go
    (`(*Engine).CharakarakaEvents` / `NextCharakarakaEvent` /
    `PrevCharakarakaEvent`) and Node (`charakarakaEvents` /
    `nextCharakarakaEvent` / `prevCharakarakaEvent`) wrap the handles.
  - Elixir `search_run` gains op `"charakaraka_events"` (semantic `mode`
    next/prev/range, `charakaraka_config.scheme`, `max_events`,
    `sankranti_config`); events echo `ranking_before`/`ranking_after`
    graha lists plus full `before`/`after` snapshots.
  - CLI: new `charakaraka-events` subcommand.

- Jyotish graha-longitude paths now honor `node_mode`.
  `GrahaLongitudesConfig` gains a `node_mode` field (default true node —
  unchanged behavior unless a caller explicitly selects the mean node),
  and the three previously hard-coded true-node call sites in
  `dhruv_search::jyotish` use it. Per-moment `charakaraka_for_date` and
  the new `charakaraka_events` therefore both honor
  `sankranti_config.node_mode` and stay in parity for any setting —
  previously the per-moment op accepted the field but silently ignored it
  for Rahu/Ketu. Callers who explicitly passed `node_mode = mean` to
  jyotish ops now get mean-node Rahu/Ketu as requested (fix). The C
  `DhruvGrahaLongitudesConfig` gains a trailing `node_mode` field
  (`DHRUV_NODE_MODE_*`; part of the v87 bump).

- New build-identity API for precalc provenance: library version + git
  build hash (internal crate `dhruv_build_info`; hash falls back to
  `"unknown"` outside a git checkout).
  - C ABI (v87): `dhruv_library_version()` / `dhruv_build_git_hash()`
    (static NUL-terminated strings).
  - Python (`library_version` / `build_git_hash`), Go
    (`LibraryVersion()` / `BuildGitHash()`), Node (`libraryVersion()` /
    `buildGitHash()`), Elixir `util_run` op `"build_info"`
    (`%{"version", "git_hash"}`), CLI `build-info` subcommand, and
    `dhruv_rs::build_version` / `build_git_hash`.

- Exposed the library's own display vocabulary for divisional charts across
  every public surface. `Amsha::sanskrit_name()` ("Rashi", "Hora", "Drekkana",
  "Navamsha", ...) already existed in `dhruv_vedic_math` but never crossed the
  ABI boundary, so consumers hand-maintained their own D-number to
  display-name tables. Purely additive — no existing field, column or key
  changes meaning.
  - C ABI (v86): new `dhruv_amsha_sanskrit_name(uint16_t amsha_code)` keyed by
    the D-number in `DhruvAmshaChart.amsha_code`, returning NUL-terminated
    UTF-8 (or `NULL` for a code outside the 34 supported amshas). It reads
    from a `CStr` table rather than the Rust `&'static str`, which is not
    NUL-terminated; a test walks every `ALL_AMSHAS` entry through the C
    accessor and compares against `sanskrit_name()`, so a stale table entry
    fails loudly instead of handing C the wrong name.
  - Python (`amsha_sanskrit_name`), Go (`AmshaSanskritName`) and Node
    (`amshaSanskritName`) expose the accessor, and their amsha chart and
    series-chart results carry the resolved name (`sanskrit_name` /
    `SanskritName` / `sanskritName`).
  - Elixir amsha chart and series-chart maps gain `"sanskrit_name"` alongside
    the code-derived `"amsha"` key (`"d9"`).
  - CLI `--format tsv` on `amsha` and `amsha-variations` gains a trailing
    `sanskrit_name` column, appended last so existing column positions are
    unchanged. The text format is untouched — its `D9_Navamsha` chart label
    already spells the name.

- Amsha chart entries now carry their own identity. Each `AmshaEntry` gains a
  `point` (`AmshaPoint { family, index }`) whose `name()`/`key()` resolve to
  the library's existing vocabulary and to a stable snake_case identifier, so
  consumers no longer have to recover a point's identity from its array
  position. Entries also gain `nakshatra`, `nakshatra_index`, `pada`, and
  `rashi_bhava_number` (whole-sign bhava from the varga lagna; a varga
  transform is not monotonic, so there is deliberately no cusp-based
  `bhava_number` inside a varga). **The sections stay arrays on every
  surface** — the keys are purely additive.
  - C ABI (v85): `DhruvAmshaEntry` gains `nakshatra_index`, `pada` and
    `rashi_bhava_number` in the three padding bytes it already carried, so the
    struct size and every pre-existing field offset are unchanged (pinned by a
    compile-time layout assertion). Point names are *not* repeated per entry —
    they are compile-time constants of (family, index) — and are queried
    instead via new `dhruv_amsha_point_count` / `dhruv_amsha_point_name` /
    `dhruv_amsha_point_key` with the `DHRUV_AMSHA_POINT_FAMILY_*` codes.
  - Elixir/Node/Python/Go amsha entries carry `name` (stable key),
    `display_name`, `family` and `point_index` alongside the new nakshatra and
    bhava fields; Python, Go and Node also expose the point accessors
    directly. The CLI now labels upagrahas, sphutas and special lagnas from
    each entry's own identity instead of hand-maintained tables (output text
    unchanged).
  - New tests pin the emitted names against the order in which the *named*
    source fields are flattened into the positional arrays, so a reorder fails
    loudly instead of silently relabelling every downstream point.
- Fixed `dhruv_special_lagna_name`, `dhruv_arudha_pada_name` and
  `dhruv_sphuta_name` returning pointers to Rust string literals, which are not
  NUL-terminated — C callers read past the end of the name. They now return
  proper NUL-terminated C strings, matching `dhruv_rashi_name` and
  `dhruv_upagraha_name`.
- Clarified `docs/clean_room_special_lagnas.md`: its numbered sections group
  the lagnas by derivation category and are not serialisation indices. The
  canonical order (Sree at index 5, Pranapada at 6) is now stated explicitly
  alongside them. No behavior change — the code was already correct and
  self-consistent across `ALL_SPECIAL_LAGNAS`, `SpecialLagna::index()`,
  `AllSpecialLagnas`, the wire arrays, `DhruvSpecialLagnas`, and
  `dhruv_special_lagna_name`.

- Generalized the sankranti search to any-body rashi-ingress search and
  added Rahu/Ketu to the conjunction and motion searches via the shared
  `TransitBody` selector (codes 10007/10008), across `dhruv_search`,
  `dhruv_rs`, the C ABI (v84), the CLI, and wrappers. New `dhruv_search`
  functions `next_ingress`/`prev_ingress`/`search_ingresses`/
  `next_specific_ingress`/`prev_specific_ingress` (classic `*_sankranti`
  functions are now Sun wrappers); retrograde re-ingresses are reported as
  events (`SankrantiEvent` gains `body`/`is_retrograde`). New `node_mode`
  config knob ("mean"|"true", recommended default "true") on the
  conjunction/stationary/sankranti configs and the corresponding
  `[operations.*]` sections; true-node stationary search is supported
  (mean node rejected). Conjunction operations add multi-angle
  `target_separations_deg` sweeps, and conjunction/motion/lunar-phase
  operations accept an optional sidereal config that adds sidereal
  longitude and rashi-index echo fields to events. The next/prev
  conjunction scan window is now pair-aware
  (`max(800 d, 1.3 x mean synodic estimate)`), fixing slow pairs such as
  Jupiter-Saturn that previously returned no event. Behavior change:
  Saturn's special transit-aspect angles in `gochar_events` were corrected
  from [90, 270] to [60, 270] (classical 3rd/10th drishti).
- Added three range operations across `dhruv_search`/`dhruv_rs`, the C ABI
  (v77-v79), the CLI, and wrappers: `amsha_series` (fixed-cadence slim varga
  charts), `panchang_events` (exact boundary sweep over all ten panchang
  elements; a location is required only for the sunrise-anchored
  vaar/hora/ghatika kinds), and `amsha_lagna_events` (exact varga-lagna
  rashi transitions, no sampling grid). New pure helper
  `next_amsha_boundary_longitude` in `dhruv_vedic_base::amsha`. The panchang
  request additionally accepts caller-cached calendar context
  (`known_masa`/`known_ayana`/`known_varsha`), reused only inside each
  value's validity window. The inclusive-grid slack in
  `graha_positions_series`/`amsha_series` was widened so endpoints exactly
  on the grid are reliably included.
- Added unified `vX.Y.Z` release automation across Python, Node, Go verification,
  Elixir, `dhruv_rs`, CLI, and the C ABI.
- Added GitHub Release packaging for CLI and C ABI bundles, npm prebuild
  packaging, PyPI publish, and Hex publish wiring.
- Expanded CI coverage to explicitly include Linux/macOS/Windows required
  targets plus best-effort Windows ARM64 jobs.
- Time policy default changed to `hybrid-deltat` (future freeze enabled by default).
- Date-driven command paths in `dhruv_cli`, `dhruv_search`, and `dhruv_rs` now share policy-aware UTC->TDB handling by default.
- Added optional staleness warnings:
  - `--stale-lsk-threshold-days`
  - `--stale-eop-threshold-days`
- Added model-agnostic future Delta-T transition strategies:
  - `legacy-tt-utc-blend` (default frozen-compatible behavior).
  - `bridge-modern-endpoint` (100-year bridge to selected asymptotic family).
- Removed user-facing `--no-freeze-future`; use `--future-delta-t-transition` instead.
- Added `stephenson1997` support in `--smh-future-family` under bridge strategy.
- Added `stephenson2016` support in `--smh-future-family` under bridge strategy, using:
  - `ΔT = -320.0 + 32.5 * ((year - 1825.0) / 100.0)^2`
