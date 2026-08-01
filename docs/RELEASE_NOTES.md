# Release Notes

## Unreleased

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
