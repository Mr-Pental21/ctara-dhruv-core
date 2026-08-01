# dhruv_search API Inventory

This document lists the public function surface used by `dhruv_search` callers,
with inputs, outputs, and behavior.

Notes:
- Most operational APIs return `Result<..., SearchError>`.
- Time arguments are either `UtcTime` (UTC-facing API) or `f64` Julian Date TDB (`jd_tdb`).
- Many input/output types come from `dhruv_core`, `dhruv_time`, and `dhruv_vedic_base`.

## Related Docs

- `docs/clean_room_conjunction.md`
- `docs/clean_room_ingress.md`
- `docs/clean_room_grahan.md`
- `docs/clean_room_stationary.md`
- `docs/clean_room_panchang.md`
- `docs/clean_room_tithi_karana_yoga.md`
- `docs/clean_room_ashtakavarga.md`
- `docs/clean_room_drishti.md`
- `docs/clean_room_upagraha.md`
- C ABI mapping (for wrapper parity): `docs/C_ABI_REFERENCE.md`

## Error Type

`SearchError` (`crates/dhruv_search/src/error.rs`) has:
- `Engine(EngineError)`
- `InvalidConfig(&'static str)`
- `NoConvergence(&'static str)`

## Transit Body Selector

Source: `crates/dhruv_search/src/transit_body.rs`

`TransitBody` (`Body(Body) | Rahu | Ketu`) is the shared body selector for
the ingress, conjunction, motion, and gochar-events searches. Rahu/Ketu use
wire codes `TRANSIT_CODE_RAHU` = 10007 and `TRANSIT_CODE_KETU` = 10008 and
are computed from the lunar-node model (mean or true, per the search
config's `node_mode`). `GocharTransitBody` is an alias of `TransitBody`,
and `TransitBody: From<Body>` so `Body::Sun.into()` works at call sites.

| Function | Inputs | Output | What it does |
|---|---|---|---|
| `TransitBody::code` | `self` | `i32` | NAIF body code, or 10007/10008 for Rahu/Ketu. |
| `TransitBody::from_code` | `code` | `Option<TransitBody>` | Decodes a NAIF or node wire code. |
| `TransitBody::name` | `self` | `&'static str` | Display name (`"Sun"`, `"Rahu"`, ...). |
| `TransitBody::lunar_node` | `self` | `Option<LunarNode>` | The node for Rahu/Ketu variants, `None` for plain bodies. |
| `TransitBody::body` | `self` | `Option<Body>` | The wrapped plain body, `None` for Rahu/Ketu. |
| `TransitBody::default_ingress_step_days` | `self` | `f64` | Per-body coarse-scan step for rashi-ingress search (Moon 0.25, Mercury/Venus 0.5, Sun/Mars 1.0, Jupiter/Saturn 2.0, Uranus/Neptune/Pluto 5.0, Rahu/Ketu 1.0). |
| `TransitBody::ingress_max_scan_days` | `self` | `f64` | Scan ceiling for next/prev any-rashi ingress search (Moon 40, Sun 400, Mercury 500, Venus 700, Mars/Jupiter 1500, Saturn 2000, Uranus 4000, Neptune 7000, Pluto 13000, Rahu/Ketu 800). |

## Conjunction APIs

Source: `crates/dhruv_search/src/conjunction.rs`, `crates/dhruv_search/src/conjunction_types.rs`

| Function | Inputs | Output | What it does |
|---|---|---|---|
| `body_ecliptic_lon_lat` | `engine`, `body: Body`, `jd_tdb` | `Result<(f64, f64), SearchError>` | Queries geocentric ecliptic longitude/latitude of a plain body (degrees). |
| `transit_body_ecliptic_lon_lat` | `engine`, `body: TransitBody`, `jd_tdb`, `node_mode` | `Result<(f64, f64), SearchError>` | Same for a transit body; Rahu/Ketu use the lunar-node model with latitude 0. |
| `next_conjunction` | `engine`, `body1: TransitBody`, `body2: TransitBody`, `jd_tdb`, `config` | `Result<Option<ConjunctionEvent>, SearchError>` | Finds next event where body separation hits target angle in `config`. Scan ceiling is pair-aware: `max(800 d, 1.3 x mean synodic estimate)`, so slow pairs (Jupiter-Saturn, node-Saturn) are found; near-equal-rate pairs (Sun with Mercury/Venus) keep 800 d. A mid-scan engine error ends the scan with `Ok(None)`. |
| `prev_conjunction` | `engine`, `body1: TransitBody`, `body2: TransitBody`, `jd_tdb`, `config` | `Result<Option<ConjunctionEvent>, SearchError>` | Finds previous target-separation event (same pair-aware ceiling and error policy). |
| `search_conjunctions` | `engine`, `body1: TransitBody`, `body2: TransitBody`, `jd_start`, `jd_end`, `config` | `Result<Vec<ConjunctionEvent>, SearchError>` | Finds all target-separation events in range (bounded by the range itself). |
| `ConjunctionConfig::conjunction` | `step_size_days` | `ConjunctionConfig` | Factory for 0 degree separation search (true node model by default). |
| `ConjunctionConfig::opposition` | `step_size_days` | `ConjunctionConfig` | Factory for 180 degree separation search. |
| `ConjunctionConfig::aspect` | `target_deg`, `step_size_days` | `ConjunctionConfig` | Factory for arbitrary aspect angle search. |

`ConjunctionConfig` carries `node_mode: NodeMode` (default true/osculating),
used when either body is Rahu/Ketu. `ConjunctionEvent` carries `body1`/`body2`
(`TransitBody`), the matched `target_separation_deg`, and optional sidereal
echo fields (`body{1,2}_sidereal_longitude_deg`, `body{1,2}_rashi_index`)
populated only by the operation layer when a sankranti config is supplied.

## Lunar Phase APIs

Source: `crates/dhruv_search/src/lunar_phase.rs`, `crates/dhruv_search/src/lunar_phase_types.rs`

| Function | Inputs | Output | What it does |
|---|---|---|---|
| `next_purnima` | `engine`, `utc` | `Result<Option<LunarPhaseEvent>, SearchError>` | Next full moon after UTC instant. |
| `prev_purnima` | `engine`, `utc` | `Result<Option<LunarPhaseEvent>, SearchError>` | Previous full moon before UTC instant. |
| `next_amavasya` | `engine`, `utc` | `Result<Option<LunarPhaseEvent>, SearchError>` | Next new moon after UTC instant. |
| `prev_amavasya` | `engine`, `utc` | `Result<Option<LunarPhaseEvent>, SearchError>` | Previous new moon before UTC instant. |
| `search_purnimas` | `engine`, `start`, `end` | `Result<Vec<LunarPhaseEvent>, SearchError>` | All full moons in UTC range. |
| `search_amavasyas` | `engine`, `start`, `end` | `Result<Vec<LunarPhaseEvent>, SearchError>` | All new moons in UTC range. |
| `LunarPhase::name` | `self` | `&'static str` | Returns display name (`"Amavasya"` or `"Purnima"`). |

`LunarPhaseEvent` carries optional sidereal echo fields
(`moon_sidereal_longitude_deg`, `sun_sidereal_longitude_deg`,
`moon_rashi_index`, `sun_rashi_index`) populated only by the operation layer
when a sankranti config is supplied.

## Grahan (Eclipse) APIs

Source: `crates/dhruv_search/src/grahan.rs`, `crates/dhruv_search/src/grahan_types.rs`

| Function | Inputs | Output | What it does |
|---|---|---|---|
| `next_chandra_grahan` | `engine`, `jd_tdb`, `config` | `Result<Option<ChandraGrahan>, SearchError>` | Next lunar eclipse candidate after `jd_tdb`, classified + contacts. |
| `prev_chandra_grahan` | `engine`, `jd_tdb`, `config` | `Result<Option<ChandraGrahan>, SearchError>` | Previous lunar eclipse before `jd_tdb`. |
| `search_chandra_grahan` | `engine`, `jd_start`, `jd_end`, `config` | `Result<Vec<ChandraGrahan>, SearchError>` | All lunar eclipses in range. |
| `besselian_elements_at` | `engine`, `eop`, `jd_tdb` | `Result<BesselianElements, SearchError>` | Ephemeris-derived instantaneous solar-shadow elements. |
| `next_surya_grahan` | `engine`, `eop`, `jd_tdb`, `location`, `config` | `Result<Option<SuryaGrahan>, SearchError>` | Next solar eclipse with optional geographic path and local circumstances. |
| `prev_surya_grahan` | `engine`, `eop`, `jd_tdb`, `location`, `config` | `Result<Option<SuryaGrahan>, SearchError>` | Previous solar eclipse with optional geographic products. |
| `search_surya_grahan` | `engine`, `eop`, `jd_start`, `jd_end`, `location`, `config` | `Result<Vec<SuryaGrahan>, SearchError>` | All solar eclipses in range. |
| `GeoLocation::new` | `latitude_deg`, `longitude_deg`, `altitude_m` | `GeoLocation` | Constructor for grahan location struct. |
| `GeoLocation::latitude_rad` | `self` | `f64` | Latitude in radians. |
| `GeoLocation::longitude_rad` | `self` | `f64` | Longitude in radians. |

## Sankranti / Rashi-Ingress APIs

Source: `crates/dhruv_search/src/sankranti.rs`, `crates/dhruv_search/src/sankranti_types.rs`

The engine is a general rashi-ingress search for any `TransitBody` (Sun
through Pluto plus Rahu/Ketu; Earth rejected): coarse scan of the sidereal
rashi index plus bisection on the crossed cusp, so retrograde re-ingresses
are first-class events (`SankrantiEvent.is_retrograde`). The classical
`*_sankranti` functions are Sun wrappers over the same engine.
`SankrantiEvent` carries `body`, `rashi`, `rashi_index`,
`sidereal_longitude_deg`, `tropical_longitude_deg`, and `is_retrograde`.
A mid-scan engine error (ephemeris coverage edge) ends next/prev scans with
`Ok(None)` and range sweeps with the events found so far; the first sample
still propagates errors. See `docs/clean_room_ingress.md`.

| Function | Inputs | Output | What it does |
|---|---|---|---|
| `next_ingress` | `engine`, `body: TransitBody`, `utc`, `config` | `Result<Option<SankrantiEvent>, SearchError>` | Next rashi ingress of `body` after UTC time (scan bounded by `TransitBody::ingress_max_scan_days`). |
| `prev_ingress` | `engine`, `body: TransitBody`, `utc`, `config` | `Result<Option<SankrantiEvent>, SearchError>` | Previous rashi ingress of `body` before UTC time. |
| `search_ingresses` | `engine`, `body: TransitBody`, `start`, `end`, `config` | `Result<Vec<SankrantiEvent>, SearchError>` | All rashi ingresses of `body` in UTC range (scan bounded by the range). |
| `next_specific_ingress` | `engine`, `body: TransitBody`, `utc`, `rashi`, `config` | `Result<Option<SankrantiEvent>, SearchError>` | Next time `body` enters a specific rashi. |
| `prev_specific_ingress` | `engine`, `body: TransitBody`, `utc`, `rashi`, `config` | `Result<Option<SankrantiEvent>, SearchError>` | Previous time `body` entered a specific rashi. |
| `next_sankranti` | `engine`, `utc`, `config` | `Result<Option<SankrantiEvent>, SearchError>` | Sun wrapper: next Sun-entry into any rashi. |
| `prev_sankranti` | `engine`, `utc`, `config` | `Result<Option<SankrantiEvent>, SearchError>` | Sun wrapper: previous Sun-entry into any rashi. |
| `search_sankrantis` | `engine`, `start`, `end`, `config` | `Result<Vec<SankrantiEvent>, SearchError>` | Sun wrapper: all sankrantis in UTC range. |
| `next_specific_sankranti` | `engine`, `utc`, `rashi`, `config` | `Result<Option<SankrantiEvent>, SearchError>` | Sun wrapper: next entry into a specific rashi. |
| `prev_specific_sankranti` | `engine`, `utc`, `rashi`, `config` | `Result<Option<SankrantiEvent>, SearchError>` | Sun wrapper: previous entry into a specific rashi. |
| `SankrantiConfig::new` | `ayanamsha_system`, `use_nutation` | `SankrantiConfig` | Constructor with default scan parameters (1.0-day step, true node model). |
| `SankrantiConfig::new_with_model` | `ayanamsha_system`, `use_nutation`, `precession_model` | `SankrantiConfig` | Constructor with an explicit precession model. |
| `SankrantiConfig::for_body` | `ayanamsha_system`, `use_nutation`, `body: TransitBody` | `SankrantiConfig` | Constructor using the per-body coarse-scan step (`TransitBody::default_ingress_step_days`). |
| `SankrantiConfig::default_lahiri` | none | `SankrantiConfig` | Factory using Lahiri ayanamsha. |
| `SankrantiConfig::validate` | `&self` | `Result<(), &'static str>` | Validates search parameter ranges. |

`SankrantiConfig` carries `node_mode: NodeMode` (default true/osculating),
used when the ingress body is Rahu/Ketu.

## Stationary and Max-Speed APIs

Source: `crates/dhruv_search/src/stationary.rs`, `crates/dhruv_search/src/stationary_types.rs`

| Function | Inputs | Output | What it does |
|---|---|---|---|
| `next_stationary` | `engine`, `body: TransitBody`, `jd_tdb`, `config` | `Result<Option<StationaryEvent>, SearchError>` | Next station (velocity sign-crossing) after `jd_tdb`. A mid-scan engine error ends the scan with `Ok(None)`. |
| `prev_stationary` | `engine`, `body: TransitBody`, `jd_tdb`, `config` | `Result<Option<StationaryEvent>, SearchError>` | Previous station before `jd_tdb`. |
| `search_stationary` | `engine`, `body: TransitBody`, `jd_start`, `jd_end`, `config` | `Result<Vec<StationaryEvent>, SearchError>` | All stations in range. |
| `next_max_speed` | `engine`, `body: TransitBody`, `jd_tdb`, `config` | `Result<Option<MaxSpeedEvent>, SearchError>` | Next local speed extremum after `jd_tdb`. |
| `prev_max_speed` | `engine`, `body: TransitBody`, `jd_tdb`, `config` | `Result<Option<MaxSpeedEvent>, SearchError>` | Previous speed extremum before `jd_tdb`. |
| `search_max_speed` | `engine`, `body: TransitBody`, `jd_start`, `jd_end`, `config` | `Result<Vec<MaxSpeedEvent>, SearchError>` | All speed extrema in range. |
| `StationaryConfig::inner_planet` | none | `StationaryConfig` | Preset config for inner planets (1-day step). |
| `StationaryConfig::outer_planet` | none | `StationaryConfig` | Preset config for outer planets (2-day step). |
| `StationaryConfig::lunar_node` | none | `StationaryConfig` | Preset config for Rahu/Ketu (0.25-day step; the true node stations roughly weekly). |

`StationaryConfig` carries `node_mode: NodeMode` (default true/osculating).
Stationary search accepts Rahu/Ketu only with the true node model — the mean
node is always retrograde, so `node_mode = Mean` is rejected with
`InvalidConfig` (Sun, Moon, and Earth remain rejected as before). Max-speed
search accepts both node models. `StationaryEvent`/`MaxSpeedEvent` carry
`body: TransitBody` and optional `sidereal_longitude_deg`/`rashi_index`
echoes populated only by the operation layer when a sankranti config is
supplied.

## Panchang APIs

Source: `crates/dhruv_search/src/panchang.rs`

| Function | Inputs | Output | What it does |
|---|---|---|---|
| `masa_for_date` | `engine`, `utc`, `sankranti_config` | `Result<MasaInfo, SearchError>` | Computes amanta lunar month + adhika flag + boundaries. |
| `masa_for_date_with_eop` | `engine`, `eop`, `utc`, `sankranti_config` | `Result<MasaInfo, SearchError>` | EOP-aware variant of `masa_for_date` used by assembled workflows. |
| `ayana_for_date` | `engine`, `utc`, `sankranti_config` | `Result<AyanaInfo, SearchError>` | Computes current ayana and its start/end transitions. |
| `ayana_for_date_with_eop` | `engine`, `eop`, `utc`, `sankranti_config` | `Result<AyanaInfo, SearchError>` | EOP-aware variant of `ayana_for_date` used by assembled workflows. |
| `varsha_for_date` | `engine`, `utc`, `sankranti_config` | `Result<VarshaInfo, SearchError>` | Computes samvatsara position and Vedic year boundaries. |
| `varsha_for_date_with_eop` | `engine`, `eop`, `utc`, `sankranti_config` | `Result<VarshaInfo, SearchError>` | EOP-aware variant of `varsha_for_date` used by assembled workflows. |
| `elongation_at` | `engine`, `jd_tdb` | `Result<f64, SearchError>` | Computes `(Moon_lon - Sun_lon) mod 360` (tropical). |
| `sidereal_sum_at` | `engine`, `jd_tdb`, `sankranti_config` | `Result<f64, SearchError>` | Computes `(Moon_sid + Sun_sid) mod 360`. |
| `moon_sidereal_longitude_at` | `engine`, `jd_tdb`, `sankranti_config` | `Result<f64, SearchError>` | Computes Moon sidereal longitude. |
| `nakshatra_for_date` | `engine`, `utc`, `sankranti_config` | `Result<PanchangNakshatraInfo, SearchError>` | Computes current nakshatra/pada with start/end. |
| `nakshatra_at` | `engine`, `jd_tdb`, `moon_sidereal_deg`, `sankranti_config` | `Result<PanchangNakshatraInfo, SearchError>` | Same as above using precomputed Moon sidereal longitude. |
| `tithi_for_date` | `engine`, `utc` | `Result<TithiInfo, SearchError>` | Computes tithi + paksha + start/end. |
| `tithi_at` | `engine`, `jd_tdb`, `elongation_deg` | `Result<TithiInfo, SearchError>` | Same as above using precomputed elongation. |
| `karana_for_date` | `engine`, `utc` | `Result<KaranaInfo, SearchError>` | Computes karana with start/end. |
| `karana_at` | `engine`, `jd_tdb`, `elongation_deg` | `Result<KaranaInfo, SearchError>` | Same as above using precomputed elongation. |
| `yoga_for_date` | `engine`, `utc`, `sankranti_config` | `Result<YogaInfo, SearchError>` | Computes yoga with start/end. |
| `yoga_at` | `engine`, `jd_tdb`, `sidereal_sum_deg`, `sankranti_config` | `Result<YogaInfo, SearchError>` | Same as above using precomputed sidereal sum. |
| `vedic_day_sunrises` | `engine`, `eop`, `utc`, `location`, `riseset_config` | `Result<(f64, f64), SearchError>` | Returns sunrise and next-sunrise JD bounds for the Vedic day. |
| `vaar_for_date` | `engine`, `eop`, `utc`, `location`, `riseset_config` | `Result<VaarInfo, SearchError>` | Computes Vedic weekday with sunrise boundaries. |
| `vaar_from_sunrises` | `sunrise_jd`, `next_sunrise_jd`, `lsk` | `VaarInfo` | Pure arithmetic weekday result from sunrise pair. |
| `hora_for_date` | `engine`, `eop`, `utc`, `location`, `riseset_config` | `Result<HoraInfo, SearchError>` | Computes planetary hour with start/end. |
| `hora_from_sunrises` | `jd_tdb`, `sunrise_jd`, `next_sunrise_jd`, `lsk` | `HoraInfo` | Pure arithmetic hora classification from sunrise pair. |
| `ghatika_for_date` | `engine`, `eop`, `utc`, `location`, `riseset_config` | `Result<GhatikaInfo, SearchError>` | Computes ghatika number (1..60) with start/end. |
| `ghatika_from_sunrises` | `jd_tdb`, `sunrise_jd`, `next_sunrise_jd`, `lsk` | `GhatikaInfo` | Pure arithmetic ghatika classification from sunrise pair. |
| `panchang_for_date` | `engine`, `eop`, `utc`, `location: Option<&GeoLocation>`, `riseset_config`, `sankranti_config`, `include_mask: u32`, `known: &PanchangPrecomputed` | `Result<PanchangResult, SearchError>` | Combined panchang for one moment. `include_mask` (`PANCHANG_INCLUDE_*` bits) gates computation, sharing intermediates across selected elements; `location` required only for vaar/hora/ghatika; `known` reuses caller-cached masa/ayana/varsha inside their validity windows. |
| `panchang_events` | `engine`, `eop`, `from_utc`, `to_utc`, `include_mask: u32`, `location: Option<&GeoLocation>`, `riseset_config`, `sankranti_config`, `max_events: u32` | `Result<PanchangEventsResult, SearchError>` | Streams exact element segments overlapping the range (warm-seeded boundary sweep; source `crates/dhruv_search/src/panchang_events.rs`). All ten elements supported; `location` required only when a location-dependent element (vaar, hora, ghatika) is selected — sunrise-anchored kinds cost one sunrise search per Vedic day. Per-kind `Vec`s of the per-moment `*Info` structs; consecutive segments chain exactly; `max_events` 0 = `MAX_PANCHANG_EVENTS` (50,000) ceiling, with `truncated`/`next_from_utc` resume. |

## Jyotish Orchestration APIs

Source: `crates/dhruv_search/src/jyotish.rs`, `crates/dhruv_search/src/jyotish_types.rs`

| Function | Inputs | Output | What it does |
|---|---|---|---|
| `graha_longitudes` | `engine`, `jd_tdb`, `config` | `Result<GrahaLongitudes, SearchError>` | Computes 9 graha longitudes on the requested reference plane. `config.kind` selects sidereal vs tropical/reference-plane output, while `ayanamsha_system`, `use_nutation`, `precession_model`, `reference_plane`, and `node_mode` (Rahu/Ketu lunar-node model, default true node) carry the remaining variations. |
| `moving_osculating_apogees` | `engine`, `jd_tdb`, `config`, `grahas` | `Result<MovingOsculatingApogees, SearchError>` | Batch heliocentric moving osculating apogee endpoint for Mangal, Buddh, Guru, Shukra, and Shani. Returns entries in caller order with sidereal longitude, ayanamsha, and reference-plane longitude. |
| `moving_osculating_apogees_for_date` | `engine`, `eop`, `utc`, `config`, `grahas` | `Result<MovingOsculatingApogees, SearchError>` | UTC-date helper for moving osculating apogees using the same sidereal config semantics as graha longitudes. |
| `special_lagnas_for_date` | `engine`, `eop`, `utc`, `location`, `riseset_config`, `aya_config` | `Result<AllSpecialLagnas, SearchError>` | Computes all special lagnas via engine + pure math orchestration. |
| `arudha_padas_for_date` | `engine`, `eop`, `utc`, `location`, `bhava_config`, `aya_config` | `Result<[ArudhaResult; 12], SearchError>` | Computes arudha padas for all 12 houses. |
| `all_upagrahas_for_date` | `engine`, `eop`, `utc`, `location`, `riseset_config`, `aya_config` | `Result<AllUpagrahas, SearchError>` | Computes all 11 upagrahas (time-based and sun-based). |
| `graha_positions` | `engine`, `eop`, `utc`, `location`, `bhava_config`, `aya_config`, `config` | `Result<GrahaPositions, SearchError>` | Central graha position API with optional lagna/nakshatra/bhava/outer planets. |
| `amsha_series` | `engine`, `eop`, `from_utc`, `to_utc`, `step_minutes`, `location`, `aya_config`, `amsha_requests`, `include_grahas` | `Result<AmshaSeries, SearchError>` | Fixed-cadence slim varga charts over a range (source `crates/dhruv_search/src/jyotish.rs`). Grid semantics match `graha_positions_series`; varga lagna always, nine grahas when `include_grahas`; charts in request order; `points * unique_requests` capped at `MAX_AMSHA_SERIES_CELLS` (100,000). |
| `amsha_lagna_events` | `engine`, `eop`, `from_utc`, `to_utc`, `location`, `aya_config`, `amsha_requests`, `max_segments` | `Result<AmshaLagnaEventsResult, SearchError>` | Exact varga-lagna rashi segments via fixed division-boundary longitudes + root-finding on the monotone ascendant — no sampling grid, no D60 aliasing (source `crates/dhruv_search/src/amsha_events.rs`). One entry per unique request; `max_segments` 0 = `MAX_AMSHA_LAGNA_SEGMENTS` (50,000) ceiling, with `truncated`/`next_from_utc` resume. |
| `charakaraka_events` | `engine`, `eop`, `from_utc`, `to_utc`, `aya_config`, `scheme`, `max_events` | `Result<CharakarakaEventsResult, SearchError>` | Chara-karaka ranking-change events via root-found lattice crossings — ingresses, pairwise degree-in-rashi crossings (Rahu reversed: sum condition `d_Rahu + d_other = 30`), and MixedParashara integer-bin mode flips (source `crates/dhruv_search/src/charakaraka_events.rs`). Emits only actual changes (before/after evaluated at ±probe), consolidating simultaneous roots; honors `aya_config.node_mode` on the same longitude path as `charakaraka_for_date`; `max_events` 0 = `MAX_CHARAKARAKA_EVENTS` (50,000) ceiling, with `truncated`/`next_from_utc` resume. |
| `next_charakaraka_event` / `prev_charakaraka_event` | `engine`, `eop`, `at_utc`, `aya_config`, `scheme` | `Result<Option<CharakarakaChangeEvent>, SearchError>` | First ranking change strictly after / last strictly before `at_utc`; `None` only at the ephemeris coverage edge. |
| `ashtakavarga_for_date` | `engine`, `eop`, `utc`, `location`, `aya_config` | `Result<AshtakavargaResult, SearchError>` | Computes full ashtakavarga (BAV/SAV/sodhana) for date/location. |
| `core_bindus` | `engine`, `eop`, `utc`, `location`, `bhava_config`, `riseset_config`, `aya_config`, `config` | `Result<BindusResult, SearchError>` | Computes curated bindu points (arudha set + lagnas + gulika/maandi etc.). |
| `drishti_for_date` | `engine`, `eop`, `utc`, `location`, `bhava_config`, `riseset_config`, `aya_config`, `config` | `Result<DrishtiResult, SearchError>` | Computes graha drishti matrix and optional bhava/lagna/bindu projections. |
| `GrahaLongitudes::longitude` | `&self`, `graha` | `f64` | Reads one graha sidereal longitude from stored array. |
| `GrahaLongitudes::rashi_index` | `&self`, `graha` | `u8` | Computes 0-based rashi index for one graha. |
| `GrahaLongitudes::all_rashi_indices` | `&self` | `[u8; 9]` | Computes rashi indices for all 9 grahas. |
| `GrahaEntry::sentinel` | none | `GrahaEntry` | Returns sentinel/zeroed entry used when optional fields are not requested. |

## API Surface Included Via Re-exports

The crate root (`crates/dhruv_search/src/lib.rs`) re-exports all operational search/orchestration functions above, plus the main input/output structs and enums. This lets callers use `dhruv_search::...` directly.
