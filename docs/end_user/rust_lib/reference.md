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
- `ayanamsha_op`
- `lunar_node_op`
- `panchang_op`
- `tara_op`
- `charakaraka`
- `upagraha_op`
- `avastha_op`
- `full_kundali`
- `gochar_events`

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
