# Release Notes

## Unreleased

- Time policy default changed to `hybrid-deltat` (future freeze enabled by default).
- Date-driven command paths in `dhruv_cli`, `dhruv_search`, and `dhruv_rs` now share policy-aware UTC->TDB handling by default.
- Added optional staleness warnings:
  - `--stale-lsk-threshold-days`
  - `--stale-eop-threshold-days`
- Added opt-in `--smh-future-family stephenson1997` for post-EOP future Delta-T asymptotic fallback under `hybrid-deltat` + `--no-freeze-future`, while preserving the default 100-year continuity blend.
