# Panchang Selection & Range-Event Operations Plan

Status: all phases complete. Phase 1 (mask-driven lazy panchang selection)
and Phases 2–3 (`amsha_series`, `panchang_events`, `amsha_lagna_events`,
`next_amsha_boundary_longitude`, batched pure varga mapping) are implemented
across all surfaces (C ABI v77, CLI, dhruv_rs, Python, Go, Node, Elixir),
integration-tested (`crates/dhruv_search/tests/range_events.rs` plus
per-wrapper suites), and documented (`clean_room_range_events.md`, reference
and end-user docs). Follow-up (also complete): caller-cached calendar
context — `PanchangPrecomputed` (`known_masa`/`known_ayana`/`known_varsha`)
on the panchang request across all surfaces (C ABI v78); values are reused
only inside their `[start, end)` validity window.

## Goals

1. Per-element panchang selection that **skips computation** (not just output) for
   unselected elements, with named groups for location-independent and
   location-dependent elements. Location becomes optional when no
   location-dependent element is requested.
2. The same selection controls panchang embedding in `full_kundali` (and
   transitively `gochar_events` via its nested kundali config).
3. Batch/range operations: `amsha_series`, `panchang_events`,
   `amsha_lagna_events` — collapsing the many-calls-per-window pattern used by
   rectification and long-range precalculation into single range calls.

## Element inventory (current)

Location-independent (7): tithi, karana, yoga, nakshatra, masa, ayana, varsha.
Location-dependent (3, sunrise-anchored): vaar, hora, ghatika.
Not implemented anywhere (future candidates, out of scope): ritu, kalams
(rahu/yamaganda/gulika), muhurta, moonrise/moonset.

## Phase 1 — mask-driven lazy panchang + full_kundali mask

### Engine (`dhruv_search`, mirrored in `dhruv_vedic_ops`)

- `panchang_for_date` signature becomes:
  `panchang_for_date(engine, eop, utc, location: Option<&GeoLocation>, riseset_config, sankranti_config, include_mask: u32) -> Result<PanchangResult, SearchError>`
  - Computes **only** elements whose bit is set. Shared intermediates computed
    at most once and only when needed:
    - elongation → tithi | karana
    - sidereal sum → yoga
    - moon sidereal longitude → nakshatra
    - vedic-day sunrise pair → vaar | hora | ghatika
    - masa / ayana / varsha each individually gated (no all-or-nothing
      `include_calendar` bool).
  - Errors: mask == 0 → InvalidConfig; any location-dependent bit set with
    `location == None` → InvalidConfig.
- `PanchangInfo` is removed; `PanchangResult` (all fields `Option`) is the
  single result shape everywhere, including inside `FullKundaliResult`.
- New mask constants alongside the existing per-element bits:
  - `PANCHANG_INCLUDE_LOCATION_INDEPENDENT = TITHI|KARANA|YOGA|NAKSHATRA|MASA|AYANA|VARSHA`
  - `PANCHANG_INCLUDE_LOCATION_DEPENDENT = VAAR|HORA|GHATIKA`
  - Existing `ALL_CORE` / `ALL_CALENDAR` / `ALL` retained.
- `PanchangOperation.location: Option<GeoLocation>`; executor `panchang()`
  becomes a thin delegation to `panchang_for_date` (removes the current
  unreachable calendar-only per-element branch).

### full_kundali

- `FullKundaliConfig.include_panchang` / `.include_calendar` are **replaced**
  (not deprecated-and-kept, per repo consolidation policy) by
  `panchang_include_mask: u32` (default 0 = omit panchang).
- `FullKundaliResult.panchang: Option<PanchangResult>`.
- Internal masa/varsha context stash: populated when present in the result.

### dhruv_config

- `FullKundaliConfigPatch.panchang_include_mask: Option<PanchangIncludeInput>`
  where the input accepts an integer mask, a group name string
  (`"all"`, `"all_core"`, `"all_calendar"`, `"location_independent"`,
  `"location_dependent"`, `"none"`), or a list of element names.
- `resolve_full_kundali` layers it like other fields (provenance tracked).

### Surfaces

- C ABI: `DhruvPanchangComputeRequest` gains explicit location-present
  handling; `DhruvFullKundaliConfig.panchang_include_mask` replaces the two
  bools; per-element validity flags on the kundali panchang section; new
  `DHRUV_PANCHANG_INCLUDE_LOCATION_INDEPENDENT/_DEPENDENT` constants; bump
  `DHRUV_API_VERSION`.
- CLI: `panchang` gains `--elements` (comma list of element/group names,
  default `all`); location flags optional when the selection is
  location-independent. `kundali` replaces `--include-panchang`/
  `--include-calendar` with `--panchang-elements`.
- Wrappers: python/go/node follow the C ABI; elixir NIF accepts
  `include_mask` int or element/group name list, optional `location`, and
  `panchang_include_mask` in full-kundali config maps.
- Docs: reference + end_user docs for all six surfaces; config layering spec.

## Phase 2 — amsha_series + pure mapping util exposure

- `amsha_series(engine, eop, from_utc, to_utc, step_minutes, location, aya_config, bhava/riseset configs, amsha_requests, options)`
  mirroring `graha_positions_series` (same grid semantics). Per point, per
  requested amsha: lagna `AmshaEntry` always; grahas behind `include_grahas`.
  Cap: `points * amsha_requests <= MAX_AMSHA_SERIES_CELLS` (100_000).
- Expose existing pure `amsha_rashi_infos` / `amsha_longitudes` batched
  mapping (longitudes × amsha_requests) through the Elixir NIF `util_run`
  (already on the C ABI as `dhruv_amsha_longitudes`).

## Phase 3 — boundary-sweep range ops

Shared scaffolding in `search_util`: given a monotone-ish scalar `f(t)` and a
"next boundary value" function, produce a warm-started segment stream (each
root-find seeded from the previous boundary; bisection refinement reuses
`find_zero_crossing` internals).

- `panchang_events`: `from_utc/to_utc + include_mask (location-independent
  bits only in v1) + sankranti_config + max_events`. Response: per-kind
  vectors reusing the existing `*Info` structs (they already carry
  start/end), plus `next_from_utc: Option<UtcTime>` when truncated by
  `max_events` (hard ceiling 50_000). Karana sweep subsumes tithi boundaries
  (6° vs 12°); masa/ayana/varsha reuse new-moon/sankranti searches with
  warm-started cursors.
- `amsha_lagna_events`: `from_utc/to_utc + location + amsha_requests +
  configs`. Per amsha: `Vec<{rashi, rashi_index, start, end}>` segments.
  Implementation: varga-rashi transitions occur at fixed D1 longitudes; add a
  pure `next_amsha_boundary_longitude(d1_lon, amsha, variation)` in
  `dhruv_vedic_math`; ascendant is monotone increasing (~360°/day) so each
  boundary crossing is a seeded root-find. No sampling grid; exact segment
  boundaries (fixes D60 aliasing).
- New clean-room docs for both algorithms; both ops propagate to all surfaces
  per the standard checklist.

## Duplication caution

The panchang implementation is duplicated between `dhruv_search` and
`dhruv_vedic_ops` (see ABI_WRAPPER_DUPLICATION_FIX_PLAN.md). Phase 1 keeps the
mirror in sync. Phase 3 sweep code should live only in `dhruv_search`, with
`dhruv_vedic_ops` re-exporting, to avoid growing the duplication.
