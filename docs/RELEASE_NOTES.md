# Release Notes

## Unreleased

- Added three range operations across `dhruv_search`/`dhruv_rs`, the C ABI
  (v77), the CLI, and wrappers: `amsha_series` (fixed-cadence slim varga
  charts), `panchang_events` (exact boundary sweep over location-independent
  panchang elements), and `amsha_lagna_events` (exact varga-lagna rashi
  transitions, no sampling grid). New pure helper
  `next_amsha_boundary_longitude` in `dhruv_vedic_base::amsha`. The
  inclusive-grid slack in `graha_positions_series`/`amsha_series` was widened
  so endpoints exactly on the grid are reliably included.
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
