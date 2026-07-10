# `dhruv_rs` — Rust Wrapper

## Purpose

`dhruv_rs` is the context-first Rust facade over the ctara-dhruv crates.

The intended public shape is:

- explicit reusable `DhruvContext`
- typed request structs in `ops.rs`
- selected re-exported config/result/helper types that are intentionally part
  of the high-level Rust contract

For usage-first end-user docs, start with
[`docs/end_user/rust_lib/README.md`](end_user/rust_lib/README.md).

## Quick Start

```rust
use std::path::PathBuf;
use dhruv_rs::*;

let engine_config = EngineConfig::with_single_spk(
    PathBuf::from("kernels/data/de442s.bsp"),
    PathBuf::from("kernels/data/naif0012.tls"),
    256,
    true,
);

let ctx = DhruvContext::new(engine_config).expect("context init");
let eop = EopKernel::load("kernels/data/finals2000A.all").expect("eop");

let request = UpagrahaRequest {
    at: TimeInput::Utc(UtcDate::new(2024, 1, 15, 12, 0, 0.0)),
    location: GeoLocation::new(28.6139, 77.2090, 0.0),
    riseset_config: Some(RiseSetConfig::default()),
    sankranti_config: Some(SankrantiConfig::default_lahiri()),
    upagraha_config: Some(TimeUpagrahaConfig::default()),
};

let upagrahas = upagraha_op(&ctx, &eop, &request).expect("upagraha");
assert!(upagrahas.gulika >= 0.0 && upagrahas.gulika < 360.0);
```

## Public API Shape

### Context Lifecycle

- `DhruvContext::new(config)`
- `DhruvContext::with_resolver(config, resolver)`
- `DhruvContext::engine()`
- `DhruvContext::replace_spk_paths(...)`
- `DhruvContext::spk_infos()`
- `DhruvContext::resolver()`
- `DhruvContext::set_resolver(...)`
- `DhruvContext::set_time_conversion_policy(...)`
- `DhruvContext::time_conversion_policy()`

`replace_spk_paths` swaps the full SPK set for long-lived contexts without
recreating the context. Matching already-loaded kernels are reused by path
metadata, failed replacements leave the active set unchanged, and the LSK path
remains fixed for the lifetime of the context.

### Request-Based Ops

- search/event requests:
  `ConjunctionRequest`, `GrahanRequest`, `MotionRequest`,
  `LunarPhaseRequest`, `SankrantiRequest`
- scalar/value requests:
  `AyanamshaRequest`, `NodeRequest`
- assembled workflow requests:
  `PanchangRequest`, `TaraRequest`, `CharakarakaRequest`,
  `UpagrahaRequest`, `AvasthaRequest`, `FullKundaliRequest`

The corresponding entrypoints are:

- `conjunction`
- `grahan`
- `motion`
- `lunar_phase`
- `sankranti`
- `ayanamsha_op`
- `lunar_node_op`
- `panchang_op`
- `tara_op`
- `charakaraka`
- `upagraha_op`
- `avastha_op`
- `full_kundali`

High-level request-based search ops use one main request shape per feature.
Their `TimeInput` fields accept either structured Gregorian UTC or numeric
JD/TDB transport without splitting the public API into `*_utc` or similar
variant entrypoints.

`PanchangRequest.include_mask` (`PANCHANG_INCLUDE_*` bits) gates computation:
only selected elements are computed, and `panchang_op` returns a
`PanchangResult` whose fields are `Option`, populated only for selected
elements. `PanchangRequest.location` is `Option<GeoLocation>` and is required
only when a location-dependent element (vaar, hora, ghatika) is selected.
`dhruv_rs` re-exports the per-element `PANCHANG_INCLUDE_*` constants, the
group masks `PANCHANG_INCLUDE_ALL`, `PANCHANG_INCLUDE_ALL_CORE`,
`PANCHANG_INCLUDE_ALL_CALENDAR`, `PANCHANG_INCLUDE_LOCATION_INDEPENDENT`,
`PANCHANG_INCLUDE_LOCATION_DEPENDENT`, and the name resolver
`panchang_include_bits(name)` (element and group names, case-insensitive).
`PanchangRequest.known: PanchangPrecomputed` carries caller-cached
masa/ayana/varsha values from a previous result; each is reused verbatim
(skipping its new-moon/sankranti searches) only when its element is selected
and the requested moment lies inside the value's `[start, end)` window —
stale values are silently recomputed. Use `PanchangPrecomputed::default()`
when nothing is cached.
`FullKundaliConfig.panchang_include_mask: u32` (default 0 = omit the panchang
section) replaces the former `include_panchang`/`include_calendar` booleans;
`FullKundaliResult.panchang` is `Option<PanchangResult>` containing only the
selected elements.

### Range / Series Ops (re-exported from `dhruv_search`)

`dhruv_rs` re-exports the range operations directly from `dhruv_search`:

- `graha_positions_series(engine, eop, from_utc, to_utc, step_minutes, ...)` —
  fixed-cadence graha positions (`GrahaPositionsSeries` /
  `GrahaPositionsPoint`, cap `MAX_GRAHA_POSITIONS_SERIES_POINTS`).
- `amsha_series(engine, eop, from_utc, to_utc, step_minutes, location,
  aya_config, amsha_requests, include_grahas)` — fixed-cadence slim varga
  charts. Grid semantics match `graha_positions_series`; the varga lagna is
  always computed, the nine grahas are added when `include_grahas` is set;
  charts are in request order. Types: `AmshaSeries`, `AmshaSeriesPoint`,
  `AmshaSeriesChart`; cap `MAX_AMSHA_SERIES_CELLS` (100,000 = points x unique
  requests).
- `panchang_events(engine, eop, from_utc, to_utc, include_mask, config,
  max_events)` — exact boundary sweep over location-independent panchang
  elements only (`include_mask` restricted to
  `PANCHANG_INCLUDE_LOCATION_INDEPENDENT` bits). Returns
  `PanchangEventsResult` with per-kind `Vec`s of the existing per-moment
  `*Info` structs plus `truncated`/`next_from_utc`; cap `MAX_PANCHANG_EVENTS`
  (50,000, `max_events = 0` selects the ceiling).
- `amsha_lagna_events(engine, eop, from_utc, to_utc, location, aya_config,
  amsha_requests, max_segments)` — exact varga-lagna rashi segments via
  root-found division-boundary crossings (no sampling grid). Returns
  `AmshaLagnaEventsResult` with one `AmshaLagnaEvents` entry per unique
  request and `AmshaLagnaSegment` items plus `truncated`/`next_from_utc`;
  cap `MAX_AMSHA_LAGNA_SEGMENTS` (50,000).

The pure boundary helper `next_amsha_boundary_longitude(sidereal_lon, amsha,
variation)` lives in `dhruv_vedic_base::amsha` (re-exported from
`dhruv_vedic_math`) and returns the next longitude at which the varga rashi
changes.

### Re-Export Policy

`dhruv_rs` intentionally re-exports a selected set of high-level config/result
types so callers can stay on the facade for common workflows. It is not meant
to be a full umbrella crate for every low-level Rust API in the workspace.

Low-level engine, time, frame, and extension-trait surfaces that are not part
of the stable high-level contract should be used from their source crates:

- `dhruv_core`
- `dhruv_time`
- `dhruv_frames`
- `dhruv_search`
- `dhruv_vedic_base`

## Configuration Rules

- Invocation-specific data belongs in the request or context:
  UTC vs JD(TDB), locations, target graha selection, range bounds, and other
  per-call inputs.
- Behavior and policy knobs belong in config structs:
  `SankrantiConfig`, `RiseSetConfig`, `FullKundaliConfig`,
  `TimeUpagrahaConfig`, and similar families.
- If a `DhruvContext` has a `ConfigResolver`, omitted config fields are resolved
  from layered config before built-in defaults.

## Notes

- `dhruv_rs` does not use public global singleton APIs.
- Reusable `DhruvContext` ownership is the intended replacement for process-wide
  wrapper state.
