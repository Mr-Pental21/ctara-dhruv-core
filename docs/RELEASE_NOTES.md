# Release Notes

## Unreleased

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
