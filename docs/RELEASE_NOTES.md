# Release Notes

## Unreleased

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
