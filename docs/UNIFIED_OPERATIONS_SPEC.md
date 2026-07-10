# Unified Operations Specification

Status: Draft (implementation started)

## Purpose

Define one canonical operation model shared by:
- C ABI (`dhruv_ffi_c`)
- Rust facade (`dhruv_rs`)
- CLI (`dhruv_cli`)

This replaces function/command proliferation by direction (`next/prev/search`) and time base (`jd/utc`) with request structs and mode enums.

## Core Concepts

## QueryMode

- `Next`: find first event after a time
- `Prev`: find first event before a time
- `Range`: find all events in a window

## TimeInput

- `JdTdb(f64)`: Julian Date in TDB
- `Utc(UtcTime)`: UTC calendar timestamp

## Window Semantics

- `Next` / `Prev`: require `at` time
- `Range`: require `start` and `end` with `end > start`

## Family Operations

Each family gets one canonical request + result contract:

1. Conjunction
2. Grahan
3. Motion (stationary/max-speed)
4. Lunar phase (purnima/amavasya)
5. Sankranti
6. Ayanamsha
7. Tara
8. Panchang (with include mask)
9. Lunar node backend selection
10. Panchang events (range sweep, location-independent elements)
11. Amsha series (fixed-cadence slim varga charts)
12. Amsha lagna events (exact varga-lagna transitions)

## Conjunction (Implemented)

## Request

`ConjunctionOperation` fields:
- `body1: Body`
- `body2: Body`
- `config: ConjunctionConfig`
- `query: ConjunctionQuery`

`ConjunctionQuery` variants:
- `Next { at_jd_tdb: f64 }`
- `Prev { at_jd_tdb: f64 }`
- `Range { start_jd_tdb: f64, end_jd_tdb: f64 }`

## Result

`ConjunctionResult` variants:
- `Single(Option<ConjunctionEvent>)` for `Next` / `Prev`
- `Many(Vec<ConjunctionEvent>)` for `Range`

## Validation

- `ConjunctionConfig` must validate
- range requires `end_jd_tdb > start_jd_tdb`

## Adapter Policy

- `dhruv_search`: source-of-truth operation execution
- `dhruv_rs`: strongly typed wrappers over operation structs
- `dhruv_cli`: command args mapped into operation structs
- `dhruv_ffi_c`: `repr(C)` request structs mapped into same `dhruv_search` operations

## Migration Policy

No existing users assumed, so hard-breaking cleanup is allowed once replacement APIs are complete and tested.

Interim implementation may expose both legacy and canonical paths while parity tests are added.

## Grahan (Implemented)

## Request

`GrahanOperation` fields:
- `kind: GrahanKind` (`Chandra` or `Surya`)
- `config: GrahanConfig`
- `query: GrahanQuery`

`GrahanQuery` variants:
- `Next { at_jd_tdb }`
- `Prev { at_jd_tdb }`
- `Range { start_jd_tdb, end_jd_tdb }`

## Result

`GrahanResult` variants:
- `ChandraSingle(Option<ChandraGrahan>)`
- `ChandraMany(Vec<ChandraGrahan>)`
- `SuryaSingle(Option<SuryaGrahan>)`
- `SuryaMany(Vec<SuryaGrahan>)`

## Motion (Implemented)

## Request

`MotionOperation` fields:
- `body: Body`
- `kind: MotionKind` (`Stationary` or `MaxSpeed`)
- `config: StationaryConfig`
- `query: MotionQuery`

`MotionQuery` variants:
- `Next { at_jd_tdb }`
- `Prev { at_jd_tdb }`
- `Range { start_jd_tdb, end_jd_tdb }`

## Result

`MotionResult` variants:
- `StationarySingle(Option<StationaryEvent>)`
- `StationaryMany(Vec<StationaryEvent>)`
- `MaxSpeedSingle(Option<MaxSpeedEvent>)`
- `MaxSpeedMany(Vec<MaxSpeedEvent>)`

## Lunar Phase (Implemented)

## Request

`LunarPhaseOperation` fields:
- `kind: LunarPhaseKind` (`Amavasya` or `Purnima`)
- `query: LunarPhaseQuery`

`LunarPhaseQuery` variants:
- `Next { at_jd_tdb }`
- `Prev { at_jd_tdb }`
- `Range { start_jd_tdb, end_jd_tdb }`

## Result

`LunarPhaseResult` variants:
- `Single(Option<LunarPhaseEvent>)`
- `Many(Vec<LunarPhaseEvent>)`

## Sankranti (Implemented)

## Request

`SankrantiOperation` fields:
- `target: SankrantiTarget` (`Any` or `SpecificRashi(Rashi)`)
- `config: SankrantiConfig`
- `query: SankrantiQuery`

`SankrantiQuery` variants:
- `Next { at_jd_tdb }`
- `Prev { at_jd_tdb }`
- `Range { start_jd_tdb, end_jd_tdb }`

## Result

`SankrantiResult` variants:
- `Single(Option<SankrantiEvent>)`
- `Many(Vec<SankrantiEvent>)`

## Ayanamsha (Implemented)

## Request

`AyanamshaOperation` fields:
- `system: AyanamshaSystem`
- `mode: AyanamshaMode` (`Mean`, `True`, `Unified`)
- `at_jd_tdb: f64`
- `use_nutation: bool` (for `Unified`)
- `delta_psi_arcsec: f64` (for `True`)

## Result

- `f64` (ayanamsha degrees)

## Node Backend (Implemented)

## Request

`NodeOperation` fields:
- `node: LunarNode` (`Rahu` or `Ketu`)
- `mode: NodeMode` (`Mean` or `True`)
- `backend: NodeBackend` (`Analytic` or `Engine`)
- `at_jd_tdb: f64`

## Result

- `f64` (node longitude in degrees)

## Panchang (Implemented)

## Request

`PanchangOperation` fields:
- `at_utc: UtcTime`
- `location: Option<GeoLocation>` (required only when a location-dependent
  element — vaar, hora, ghatika — is selected in `include_mask`)
- `riseset_config: RiseSetConfig`
- `sankranti_config: SankrantiConfig`
- `include_mask: u32` (bitset over `PANCHANG_INCLUDE_*`)

Include bits:
- `PANCHANG_INCLUDE_TITHI`
- `PANCHANG_INCLUDE_KARANA`
- `PANCHANG_INCLUDE_YOGA`
- `PANCHANG_INCLUDE_VAAR`
- `PANCHANG_INCLUDE_HORA`
- `PANCHANG_INCLUDE_GHATIKA`
- `PANCHANG_INCLUDE_NAKSHATRA`
- `PANCHANG_INCLUDE_MASA`
- `PANCHANG_INCLUDE_AYANA`
- `PANCHANG_INCLUDE_VARSHA`
- `PANCHANG_INCLUDE_ALL_CORE`
- `PANCHANG_INCLUDE_ALL_CALENDAR`
- `PANCHANG_INCLUDE_ALL`
- `PANCHANG_INCLUDE_LOCATION_INDEPENDENT` (tithi|karana|yoga|nakshatra|masa|ayana|varsha = `0x3C7`)
- `PANCHANG_INCLUDE_LOCATION_DEPENDENT` (vaar|hora|ghatika = `0x38`)

`panchang_include_bits(name)` resolves element names and group names (`all`,
`all_core`, `all_calendar`, `location_independent`, `location_dependent`),
case-insensitive.

Semantics:
- `include_mask` gates computation, not just output: unselected elements are
  never computed, and shared intermediates (Moon-Sun elongation for
  tithi/karana, sidereal sum for yoga, Moon sidereal longitude for nakshatra,
  the Vedic-day sunrise pair for vaar/hora/ghatika) are computed at most once
  and only when a selected element needs them. Masa/ayana/varsha are
  individually gated.
- `include_mask == 0` -> `InvalidConfig`.
- Location-dependent bit set with `location: None` ->
  `InvalidConfig("location required for vaar/hora/ghatika")`.

## Result

`PanchangResult` fields (all optional, controlled by `include_mask`):
- `tithi: Option<TithiInfo>`
- `karana: Option<KaranaInfo>`
- `yoga: Option<YogaInfo>`
- `vaar: Option<VaarInfo>`
- `hora: Option<HoraInfo>`
- `ghatika: Option<GhatikaInfo>`
- `nakshatra: Option<PanchangNakshatraInfo>`
- `masa: Option<MasaInfo>`
- `ayana: Option<AyanaInfo>`
- `varsha: Option<VarshaInfo>`

## Panchang Events (Implemented)

Range sweep over panchang element boundaries. Inputs are plain function
parameters (`dhruv_search::panchang_events`), mirrored as handle-based C ABI
accessors, a CLI subcommand, and wrapper ops.

## Request

- `from_utc: UtcTime`, `to_utc: UtcTime` (`to_utc > from_utc`)
- `include_mask: u32` — non-zero, restricted to
  `PANCHANG_INCLUDE_LOCATION_INDEPENDENT` bits
  (tithi|karana|yoga|nakshatra|masa|ayana|varsha)
- `config: SankrantiConfig`
- `max_events: u32` (`0` = ceiling `MAX_PANCHANG_EVENTS` = 50,000)

## Result

`PanchangEventsResult` fields:
- per-kind `Vec`s of the existing per-moment info structs: `tithi:
  Vec<TithiInfo>`, `karana: Vec<KaranaInfo>`, `yoga: Vec<YogaInfo>`,
  `nakshatra: Vec<PanchangNakshatraInfo>`, `masa: Vec<MasaInfo>`,
  `ayana: Vec<AyanaInfo>`, `varsha: Vec<VarshaInfo>`
- `truncated: bool`
- `next_from_utc: Option<UtcTime>` (resume point; dedup on `(kind, start)`)

Semantics: consecutive segments of one kind chain exactly (`end ==
next.start`); the first segment may start before `from_utc` and the last may
end after `to_utc`; nakshatra `pada` is the segment-start pada (always 1).

## Amsha Series (Implemented)

Fixed-cadence sampling of slim varga charts
(`dhruv_search::amsha_series`).

## Request

- `from_utc: UtcTime`, `to_utc: UtcTime`, `step_minutes: u32` (>= 1) — grid
  semantics identical to `graha_positions_series`
- `location: GeoLocation`
- `aya_config: SankrantiConfig`
- `amsha_requests: &[AmshaRequest]` (non-empty, usual amsha batch rules)
- `include_grahas: bool`

## Result

`AmshaSeries { points: Vec<AmshaSeriesPoint> }`; each point carries `utc`,
`jd_utc`, and `charts: Vec<AmshaSeriesChart>` in request order. Each chart
has `amsha`, `variation_code`, `lagna: AmshaEntry`, and `grahas:
Option<[AmshaEntry; 9]>` (present when `include_grahas`).

Validation: `points * unique_requests` capped at `MAX_AMSHA_SERIES_CELLS`
(100,000).

## Amsha Lagna Events (Implemented)

Exact varga-lagna rashi segments (`dhruv_search::amsha_lagna_events`) via
fixed division-boundary longitudes and root-finding on the monotone
ascendant — no sampling grid, so fast vargas (e.g. D60) cannot alias.

## Request

- `from_utc: UtcTime`, `to_utc: UtcTime`
- `location: GeoLocation`
- `aya_config: SankrantiConfig`
- `amsha_requests: &[AmshaRequest]` (non-empty; duplicates collapsed)
- `max_segments: u32` (`0` = ceiling `MAX_AMSHA_LAGNA_SEGMENTS` = 50,000)

## Result

`AmshaLagnaEventsResult` fields:
- `entries: Vec<AmshaLagnaEvents>` — one per unique request, in request
  order; each carries `amsha`, `variation_code`, and `segments:
  Vec<AmshaLagnaSegment>` (`rashi`, `rashi_index`, `start`, `end`)
- `truncated: bool`
- `next_from_utc: Option<UtcTime>`

Semantics: the first segment starts at `from_utc` (clipped); each segment's
`end` is an exact root-found transition; the last segment's `end` is the
first transition at or after `to_utc`.

## Tara (Implemented)

## Request

`TaraOperation` fields:
- `star: TaraId`
- `output: TaraOutputKind` (`Equatorial`, `Ecliptic`, `Sidereal`)
- `at_jd_tdb: f64`
- `ayanamsha_deg: f64` (used for `Sidereal`)
- `config: TaraConfig`
- `earth_state: Option<EarthState>`

## Result

`TaraResult` variants:
- `Equatorial(EquatorialPosition)`
- `Ecliptic(SphericalCoords)`
- `Sidereal(f64)`
