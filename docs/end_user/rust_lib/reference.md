# Rust Library Reference

This page summarizes the intended public `dhruv_rs` surface from
`crates/dhruv_rs/src/`.

## Primary API Styles

- Explicit reusable context ownership via `DhruvContext`
- Request-based operation APIs in `ops.rs`
- Amsha helpers in `amsha.rs`

`dhruv_rs` should be used through explicit `DhruvContext` ownership rather than
global singleton state. A `DhruvContext` owns an initialized engine and is
meant to be reused across many operations, not recreated for every call.

## Context APIs

Core public types:

- `DhruvContext`
- `UtcDate`
- `DhruvError`

`DhruvContext` methods:

- `DhruvContext::new`
- `DhruvContext::with_resolver`
- `DhruvContext::engine`
- `DhruvContext::resolver`
- `DhruvContext::set_resolver`
- `DhruvContext::set_time_conversion_policy`
- `DhruvContext::time_conversion_policy`

## Request-Based Ops API

Public request/query types in `ops.rs`:

- `TimeInput`
- `TransitBody` (with `TRANSIT_CODE_RAHU` = 10007, `TRANSIT_CODE_KETU` = 10008)
- `ConjunctionRequestQuery`, `ConjunctionRequest`
- `GrahanRequestQuery`, `GrahanRequest`
- `MotionRequestQuery`, `MotionRequest`
- `LunarPhaseRequestQuery`, `LunarPhaseRequest`
- `SankrantiRequestQuery`, `SankrantiRequest`
- `AyanamshaRequestMode`, `AyanamshaRequest`
- `NodeRequest`
- `PanchangRequest`
- `TaraRequest`
- `CharakarakaRequest`
- `UpagrahaRequest`
- `AvasthaTarget`, `AvasthaRequest`, `AvasthaResult`
- `FullKundaliRequest`
- `GocharEventsRequest`
- `GocharTransitBody`

Request-driven functions:

- `conjunction`
- `grahan`
- `motion`
- `lunar_phase`
- `sankranti`
- `fixed_longitude`
- `ayanamsha_op`
- `lunar_node_op`
- `panchang_op`
- `tara_op`
- `charakaraka`
- `upagraha_op`
- `avastha_op`
- `full_kundali`
- `gochar_events`

The conjunction, motion, and sankranti searches track `TransitBody` values:
any plain `Body` plus `Rahu`/`Ketu` (from the lunar-node model selected by
the relevant config's `node_mode`; default true/osculating).
`TransitBody: From<Body>`, so plain-body call sites use `Body::Sun.into()`.

- `SankrantiRequest.body` selects the ingress body: the search finds when
  that body's sidereal longitude enters a rashi (Sun = classical
  sankranti; Earth rejected). `SankrantiEvent` reports `body`, `rashi`,
  `rashi_index`, `sidereal_longitude_deg`, `tropical_longitude_deg`, and
  `is_retrograde` (retrograde re-entry into the preceding rashi; always
  false for the Sun). `SankrantiConfig::for_body(system, use_nutation,
  body)` picks the per-body scan step.
- `ConjunctionRequest` takes `body1`/`body2: TransitBody`, an optional
  `target_separations_deg: Vec<f64>` multi-angle sweep (each event carries
  the matched angle in `target_separation_deg`), and an optional
  `sankranti_config` that adds sidereal longitude and rashi-index echoes
  for both bodies to each event. The next/prev scan window is pair-aware,
  so slow pairs such as Jupiter-Saturn are found.
- `MotionRequest.body` is a `TransitBody`; true-node Rahu/Ketu stationary
  search is supported (`StationaryConfig::lunar_node()` preset with a
  0.25-day step; the mean node never stations and is rejected with an
  invalid-config error). The optional `sankranti_config` adds
  `sidereal_longitude_deg`/`rashi_index` echoes to events.
- `LunarPhaseRequest` accepts an optional `sankranti_config` that adds
  Sun/Moon sidereal longitude and rashi-index echoes to each event.
- `FixedLongitudeRequest` finds when the moving `body` reaches the fixed
  sidereal `target_longitude_deg`, optionally offset by
  `target_angles_deg` (offsets added mod 360; empty = conjunction only).
  `include_special_angles` also searches the body's classical
  special-aspect angles (Mars 90/210, Jupiter 120/240, Saturn 60/270)
  applied so the moving body casts that aspect onto the target. The
  optional `config` is a `SankrantiConfig` (context default when `None`).
  `FixedLongitudeResult` mirrors `SankrantiResult`
  (`Single(Option<FixedLongitudeEvent>)` / `Many(Vec<...>)`); events
  carry the matched longitude, sidereal + tropical longitudes, and the
  root residual. A range crossing the ephemeris coverage edge returns
  partial results; next/prev scans are bounded per body.

`ConjunctionConfig`, `StationaryConfig`, and `SankrantiConfig` each carry
`node_mode: NodeMode` (also settable through layered config as
`node_mode = "mean" | "true"` under `[operations.conjunction]`,
`[operations.stationary]`, and `[operations.sankranti]`).

Solar `GrahanRequest` accepts an optional observer location; `GrahanConfig`
controls optional path and footprint sampling plus the field products
(`include_local_grid`/`local_grid_step_deg`, `include_isolines` with
`duration_isoline_fractions`/`magnitude_isoline_levels`, and
`include_central_corridor`, `include_contact_footprints`,
`include_umbra_footprints`). `SuryaGrahan` then carries `centrality`,
`local_grid`, `isolines`, `central_corridor`, `contact_footprints`, and
`umbra_footprints`; sampled footprints carry `contains_pole` and, with
`instantaneous_magnitude_levels`, per-timestamp `magnitude_rings`;
`GrahanConfig::effective()` echoes the clamped/sanitized configuration for
cache identity. See `docs/end_user/solar_eclipse_visibility.md` for the
result fields and map use.

High-level time-bearing search results default to structured Gregorian UTC on
their main result types while retaining numeric JD/TDB alongside UTC where the
numeric transport remains part of the public contract.

The corresponding high-level search request types use `TimeInput`, so the same
main operations accept either structured Gregorian UTC or numeric JD/TDB
without separate `*_utc` entrypoints.

## Common Public Types And Configs

Frequently used config and result families re-exported from `dhruv_rs::*`:

- `EngineConfig`
- `GeoLocation`
- `EopKernel`
- `RiseSetConfig`
- `BhavaConfig`
- `SankrantiConfig`
- `ConjunctionConfig`
- `GrahanConfig`
- `StationaryConfig`
- `TimeConversionPolicy`
- `TimeConversionOptions`
- `Graha`
- `AyanamshaSystem`
- `NodeDignityPolicy`
- `GrahaPositionsConfig`
  Defaults `include_outer_planets=true`; returned `grahas` stay the 9
  navagrahas and `outer_planets` carries `[Uranus, Neptune, Pluto]`.
  `basic_states_config` controls optional `basic_states` and
  `sensitive_point_distances` output on entries.
- `GrahaLongitudesConfig`
  Defaults `include_outer_planets=true`; `graha_longitudes` returns the 9
  navagraha `longitudes` plus sibling `outer_planets`. Use
  `.with_outer_planets(false)` for navagraha-only Rust calls.

`BhavaConfig` defaults `use_rashi_bhava_for_bala_avastha=true`,
`include_rashi_bhava_results=true`, `include_special_bhavabala_rules=true`, and
`include_node_aspects_for_drik_bala=false`. Set
`include_node_aspects_for_drik_bala=true` when Shadbala Drik Bala and
Bhava Bala Drishti Bala should include Rahu/Ketu incoming aspects;
standalone drishti matrices are unaffected.
`divide_guru_buddh_drishti_by_4_for_drik_bala=true` by default; set it to
`false` to add Guru/Buddh incoming aspects at full signed strength instead of
through the divided Drik Bala balance. `chandra_benefic_rule` defaults to
`ChandraBeneficRule::Brightness72`, where Chandra is benefic when its smaller
angular distance from Surya is at least 72 degrees. Set it to
`ChandraBeneficRule::Waxing180` for the prior waxing-arc rule where Chandra is
benefic when `normalize_360(Chandra - Surya) <= 180`. The same Chandra rule
is used by Buddh's association-based benefic/malefic classification in
Shadbala Drik Bala and Bhava Bala Drishti Bala.
`sayanadi_ghatika_rounding` defaults to `SayanadiGhatikaRounding::Floor`; use
`SayanadiGhatikaRounding::Ceil` to count the current partial ghatika.
- `BindusConfig`
- `DrishtiConfig`
- `TimeUpagrahaConfig`
- `TimeUpagrahaPoint`
- `GulikaMaandiPlanet`
- `FullKundaliConfig`
- `FullKundaliResult`
- `GocharEventsConfig`
- `GocharEventsResult`
- `GocharReference`
- `GocharTransitBody`
- `NatalTargetKind`
- `NatalTargetLongitude`
- `TajakaReturnBasis`
- `TajakaReturnEvent`
- `TithiPraveshaEvent`
- `TransitToNatalConjunctionEvent`
- `DashaSnapshotTime`
- `DashaTimeExt`
- `DashaSnapshotTimeExt`
- `AllUpagrahas`
- `AllGrahaAvasthas`
- `GrahaAvasthas`
- `DashaVariationConfig`
- `TaraConfig`

## Selected Direct Re-Exports

`dhruv_rs` still re-exports a selected set of lower-level helpers and result
types for Rust callers, including:

- amsha helpers such as `amsha_longitude`, `amsha_chart_for_date`, and
  `amsha_charts_for_date`
  Amsha chart `grahas` stay length 9; transformed outer planet entries are
  returned in the sibling `outer_planets` section when enabled.
  Every `AmshaEntry` carries a `point: AmshaPoint { family, index }` whose
  `name()` and `key()` resolve its identity, so callers never index a section
  to find out what a value is. Entries also carry `nakshatra`,
  `nakshatra_index`, `pada`, and `rashi_bhava_number` (whole-sign bhava from
  the varga lagna; a varga transform is not monotonic, so `bhava_cusps` are
  not ordered house boundaries and there is no cusp-based bhava inside a
  varga). `AmshaPointFamily` and `ALL_AMSHA_POINT_FAMILIES` enumerate the
  families and their canonical order.
  `Amsha::sanskrit_name()` gives the library's display name for a divisional
  chart (`"Navamsha"`, `"Drekkana"`, ...), and `Amsha::name()` the prefixed
  form (`"D9_Navamsha"`); both are reachable through the `Amsha` re-export.
- full-kundali, shadbala, vimsopaka, and dasha result/config families
- pure jyotish math helpers such as `calculate_ashtakavarga`,
  `calculate_bhava_bala`, `calculate_bav`, `calculate_sav`, and
  `calculate_all_bav`

The standalone shadbala, vimsopaka, balas, and avastha surfaces now share
`AmshaSelectionConfig`, and embedded `full_kundali(...).amshas` returns the
resolved amsha union used by the call.

`full_kundali(...)` also forwards `graha_positions_config.basic_states_config`,
and sensitive-point distance mode may return bhava-cusp distance arrays.

`GrahaAvasthas.deeptadi` is the primary compatibility Deeptadi state.
`GrahaAvasthas.deeptadi_states` is the authoritative full set of Deeptadi
states satisfied by that graha.
`GrahaAvasthas.lajjitadi` is `None` when no Lajjitadi condition applies;
`GrahaAvasthas.lajjitadi_states` is the authoritative full set of Lajjitadi
states satisfied by that graha.

Rahu owns Kumbha and Ketu owns Vrischika for node dignity in the default
sign-lord-based policy. Chara-style dasha period selection uses dual lordship
for Kumbha (`Shani`/`Rahu`) and Vrischika (`Mangal`/`Ketu`); ordinary primary
rashi-lord helpers remain visible-lord based.

## Dasha Level-0 Cycle Repetition

`DashaVariationConfig` carries two optional level-0 cycle knobs for
nakshatra-based and Yogini systems (other systems ignore them):

- `cycles: Option<u8>` — emit exactly N whole mahadasha cycles; wins over
  `min_span_years`.
- `min_span_years: Option<f64>` — append whole cycles until level-0
  coverage from birth reaches at least N years; the final cycle completes
  even if it overshoots.

The variation config is accepted by the level-0 entrypoints
(`dasha_level0_for_birth`, `dasha_level0_entity_for_birth`,
`dasha_level0_with_inputs`, `dasha_level0_entity_with_inputs`) in addition
to the hierarchy/snapshot/children surfaces. In full-kundali requests the
same knobs live on `DashaSelectionConfig` (`cycles: u8`, 0 = system
default; `min_span_years: f64`, 0.0 = disabled). A period's cycle number
is `(order - 1) / sequence_len + 1` — `order` is global across cycles.

## Panchang Element Selection

`PanchangRequest.include_mask` carries `PANCHANG_INCLUDE_*` bits that gate
computation: only the selected elements are computed, and `panchang_op`
returns a `PanchangResult` whose fields are all `Option`, populated only for
selected elements. `PanchangRequest.location` is `Option<GeoLocation>` and is
needed only when a location-dependent element (vaar, hora, ghatika) is
selected; location-independent selections require no location.

Per-element constants (`PANCHANG_INCLUDE_TITHI`, ..., `PANCHANG_INCLUDE_VARSHA`),
group masks (`PANCHANG_INCLUDE_ALL`, `PANCHANG_INCLUDE_ALL_CORE`,
`PANCHANG_INCLUDE_ALL_CALENDAR`, `PANCHANG_INCLUDE_LOCATION_INDEPENDENT`,
`PANCHANG_INCLUDE_LOCATION_DEPENDENT`), and the name resolver
`panchang_include_bits("tithi" | "all" | "location_independent" | ...)` are
re-exported from `dhruv_rs`.

```rust
use dhruv_rs::{
    PANCHANG_INCLUDE_LOCATION_INDEPENDENT, PanchangPrecomputed, PanchangRequest, TimeInput,
    panchang_op,
};

let request = PanchangRequest {
    at: TimeInput::Utc(utc),
    location: None, // not needed: no vaar/hora/ghatika selected
    riseset_config: None,
    sankranti_config: None,
    include_mask: PANCHANG_INCLUDE_LOCATION_INDEPENDENT,
    known: PanchangPrecomputed::default(),
};
let result = panchang_op(&ctx, &request, &eop)?;
assert!(result.tithi.is_some());
assert!(result.vaar.is_none()); // not selected, never computed

// Repeated nearby calls can feed calendar values back: each known value is
// reused verbatim (its new-moon/sankranti searches skipped) only while the
// requested moment stays inside its [start, end) window; stale values are
// silently recomputed.
let next_request = PanchangRequest {
    at: TimeInput::Utc(next_day),
    known: PanchangPrecomputed {
        masa: result.masa,
        ayana: result.ayana,
        varsha: result.varsha,
    },
    ..request
};
```

In full-kundali requests, `FullKundaliConfig.panchang_include_mask: u32`
(default 0 = omit the panchang section) selects the elements the same way;
it replaces the former `include_panchang`/`include_calendar` booleans, and
`FullKundaliResult.panchang` is `Option<PanchangResult>` containing only the
selected elements.

## Panchang Events Over a Range

`panchang_events` (re-exported from `dhruv_search`) streams every element
segment overlapping a UTC range in one call, with exact boundary times. All
ten elements are supported; a location (with a rise/set config) is required
only when a location-dependent element (vaar, hora, ghatika) is selected —
those kinds cost one sunrise search per Vedic day, with hora/ghatika
subdivisions computed arithmetically. Results are per-kind `Vec`s of the
same `*Info` structs the per-moment ops use. Consecutive segments of one
kind chain exactly (`end == next.start`), including across Vedic-day rolls;
the first segment may start before `from_utc` and the last may end after
`to_utc`. `max_events` caps the total segments across all kinds (`0`
selects the `MAX_PANCHANG_EVENTS` = 50,000 ceiling); a capped result sets
`truncated` and `next_from_utc` for resuming (deduplicate on
`(kind, start)`). Nakshatra events report the segment-start pada (always 1).

```rust
use dhruv_rs::{
    PANCHANG_INCLUDE_GHATIKA, PANCHANG_INCLUDE_MASA, PANCHANG_INCLUDE_TITHI, RiseSetConfig,
    panchang_events,
};

// Location-independent kinds need no location at all.
let events = panchang_events(
    ctx.engine(),
    &eop,
    &from_utc,
    &to_utc,
    PANCHANG_INCLUDE_TITHI | PANCHANG_INCLUDE_MASA,
    None,
    &RiseSetConfig::default(),
    &SankrantiConfig::default_lahiri(),
    0, // library ceiling
)?;
for tithi in &events.tithi {
    println!("{:?}: {:?} -> {:?}", tithi.tithi, tithi.start, tithi.end);
}
assert!(!events.truncated);

// Sunrise-anchored kinds (e.g. ghatika lanes) take a location.
let lanes = panchang_events(
    ctx.engine(),
    &eop,
    &from_utc,
    &to_utc,
    PANCHANG_INCLUDE_GHATIKA,
    Some(&location),
    &RiseSetConfig::default(),
    &SankrantiConfig::default_lahiri(),
    0,
)?;
assert_eq!(lanes.ghatika.first().map(|g| g.value % 60 + 1), lanes.ghatika.get(1).map(|g| g.value));
```

## Amsha Series and Amsha Lagna Events

`amsha_series` samples slim varga charts at a fixed cadence over
`[from_utc, to_utc]` (grid semantics identical to `graha_positions_series`).
The varga lagna is always computed per requested amsha; pass
`include_grahas = true` to add the nine graha varga entries. Charts come back
in request order; the grid is capped at `MAX_AMSHA_SERIES_CELLS` = 100,000
cells (points x unique requests).

```rust
use dhruv_rs::amsha_series;
use dhruv_vedic_base::{Amsha, AmshaRequest};

let series = amsha_series(
    ctx.engine(),
    &eop,
    &from_utc,
    &to_utc,
    60, // step_minutes
    &location,
    &SankrantiConfig::default_lahiri(),
    &[AmshaRequest::new(Amsha::D9), AmshaRequest::new(Amsha::D10)],
    true, // include_grahas
)?;
for point in &series.points {
    let d9 = &point.charts[0];
    println!("{:?}: D9 lagna in rashi {}", point.utc, d9.lagna.rashi_index);
}
```

`amsha_lagna_events` returns the exact varga-lagna rashi segments over a
range by root-finding the times the ascendant crosses fixed division-boundary
longitudes — no sampling grid, so fast vargas such as D60 cannot alias
between samples. One entry per unique request; the first segment starts at
`from_utc` and each segment `end` is an exact transition. `max_segments`
caps totals across all amshas (`0` = `MAX_AMSHA_LAGNA_SEGMENTS` = 50,000),
with `truncated`/`next_from_utc` for resuming.

```rust
use dhruv_rs::amsha_lagna_events;
use dhruv_vedic_base::{Amsha, AmshaRequest};

let result = amsha_lagna_events(
    ctx.engine(),
    &eop,
    &from_utc,
    &to_utc,
    &location,
    &SankrantiConfig::default_lahiri(),
    &[AmshaRequest::new(Amsha::D60)],
    0, // library ceiling
)?;
for segment in &result.entries[0].segments {
    println!("{:?} from {:?} to {:?}", segment.rashi, segment.start, segment.end);
}
```

The pure helper `next_amsha_boundary_longitude(sidereal_lon, amsha,
variation)` in `dhruv_vedic_base::amsha` returns the next longitude (as
`sidereal_lon + delta`, `delta > 0`) at which the varga rashi changes — an
exact division boundary, convenient for monotone trackers.

`charakaraka_events` returns the exact moments the chara-karaka ranking
changes over a range, per scheme (`Eight`, `SevenNoPitri`,
`SevenPkMergedMk`, `MixedParashara`). Boundaries are root-found: rashi
ingresses, pairwise degree-in-rashi crossings (Rahu counts reversed, so a
Rahu crossing is the sum condition `d_Rahu + d_other = 30`), and — for
`MixedParashara` — the integer-degree bin boundaries that flip the 8↔7
mode (`SchemeModeChange` trigger, `used_eight_karakas` flips). Each
`CharakarakaChangeEvent` carries `utc`/`jd_tdb`, a trigger, `before`/
`after` rankings in the per-moment `CharakarakaResult` shape (the entry
order is the documented contract: effective degree desc, then raw
degrees-in-rashi desc, then graha index asc), and `changed_roles`. Only
actual ranking changes are emitted; rankings honor
`aya_config.node_mode` on the same longitude path as the per-moment op.
`max_events` caps output (`0` = `MAX_CHARAKARAKA_EVENTS` = 50,000) with
`truncated`/`next_from_utc` for resuming (the seam event is re-found —
deduplicate on the event time). `next_charakaraka_event` /
`prev_charakaraka_event` return the single neighboring change around
`at_utc`.

```rust
use dhruv_rs::{charakaraka_events, CharakarakaEventTrigger};
use dhruv_vedic_base::CharakarakaScheme;

let result = charakaraka_events(
    ctx.engine(),
    &eop,
    &from_utc,
    &to_utc,
    &SankrantiConfig::default_lahiri(),
    CharakarakaScheme::Eight,
    0, // library ceiling
)?;
for event in &result.events {
    let atma_after = &event.after.entries[0];
    println!(
        "{:?} {:?}: AK -> {:?} (changed: {:?})",
        event.utc, event.trigger, atma_after.graha, event.changed_roles
    );
}
```

`build_version()` and `build_git_hash()` identify the running build for
precalc provenance (`git_hash` is `"unknown"` outside a git checkout).

## Equatorial Output on Graha Positions

`GrahaPositionsConfig.include_equatorial` (default false) adds per-entry
geocentric equatorial data to `GrahaEntry`: `equatorial_valid`,
`right_ascension_deg` (0..360), `declination_deg` (−90..+90), and
`ecliptic_latitude_deg`. The `GrahaPositions` result additionally carries
`earth_orientation_valid`, `gmst_deg`, and `gast_deg` (Greenwich
mean/apparent sidereal time, degrees).

Conventions: equinox of date; nutation in longitude and true obliquity are
applied when the request's `use_nutation` flag is set (pair apparent RA
with `gast_deg`, mean RA with `gmst_deg`); geocentric, geometric positions
(no light-time or aberration); lagna and the lunar nodes lie on the
ecliptic so their `ecliptic_latitude_deg` is exactly 0; outer planets
carry true latitudes. Also available through `full_kundali` via
`graha_positions_config.include_equatorial`. See
`docs/clean_room_equatorial_output.md` for provenance.

`graha_positions_series(engine, eop, from_utc, to_utc, step_minutes,
location, bhava_config, aya_config, config)` samples the same op at a
fixed cadence (endpoints inclusive on the grid; at most
`MAX_GRAHA_POSITIONS_SERIES_POINTS` = 10,000 points). Each
`GrahaPositionsPoint` carries `utc`, `jd_utc`, and a `positions` value
with the identical single-epoch shape, including per-point
`gmst_deg`/`gast_deg` when equatorial output is enabled. The graha
positions family (`graha_positions`, `graha_positions_series`, and their
config/result types) is re-exported from `dhruv_rs`.

Grahan results also carry apparent equatorial coordinates at greatest
grahan: `ChandraGrahan.moon_right_ascension_deg`/`moon_declination_deg`
and `SuryaGrahan.sun_right_ascension_deg`/`sun_declination_deg`
(degrees, true equator/equinox of date, IAU 2000B nutation applied).

For low-level engine, time, frame, and extension-trait surfaces that are not
explicitly re-exported here, depend on the source crates directly:

- `dhruv_core`
- `dhruv_time`
- `dhruv_frames`
- `dhruv_search`
- `dhruv_vedic_base`

## Notes

- Use request/context attributes for invocation-specific inputs such as UTC vs
  JD(TDB), locations, and per-call selectors.
- Use config objects for behavior and policy knobs.
- `dhruv_rs` no longer carries public singleton or convenience-wrapper layers.
