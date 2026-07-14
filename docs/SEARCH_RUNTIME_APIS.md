# dhruv_search Runtime API (Query Functions Only)

This is the runtime/query surface of `dhruv_search` re-exported from `crates/dhruv_search/src/lib.rs`.

Total runtime functions documented here: **68**.

## Conjunction / Aspect (4)

| Function | Inputs | Output | What it does |
|---|---|---|---|
| `body_ecliptic_lon_lat` | `engine`, `body`, `jd_tdb` | `Result<(f64, f64), SearchError>` | Geocentric ecliptic lon/lat (degrees) for a body. |
| `next_conjunction` | `engine`, `body1`, `body2`, `jd_tdb`, `config` | `Result<Option<ConjunctionEvent>, SearchError>` | Next event where separation reaches target aspect angle. |
| `prev_conjunction` | `engine`, `body1`, `body2`, `jd_tdb`, `config` | `Result<Option<ConjunctionEvent>, SearchError>` | Previous event where separation reaches target angle. |
| `search_conjunctions` | `engine`, `body1`, `body2`, `jd_start`, `jd_end`, `config` | `Result<Vec<ConjunctionEvent>, SearchError>` | All target-separation events in a range. |

## Lunar Phase (6)

| Function | Inputs | Output | What it does |
|---|---|---|---|
| `next_purnima` | `engine`, `utc` | `Result<Option<LunarPhaseEvent>, SearchError>` | Next full moon after UTC instant. |
| `prev_purnima` | `engine`, `utc` | `Result<Option<LunarPhaseEvent>, SearchError>` | Previous full moon before UTC instant. |
| `next_amavasya` | `engine`, `utc` | `Result<Option<LunarPhaseEvent>, SearchError>` | Next new moon after UTC instant. |
| `prev_amavasya` | `engine`, `utc` | `Result<Option<LunarPhaseEvent>, SearchError>` | Previous new moon before UTC instant. |
| `search_purnimas` | `engine`, `start`, `end` | `Result<Vec<LunarPhaseEvent>, SearchError>` | All full moons in UTC range. |
| `search_amavasyas` | `engine`, `start`, `end` | `Result<Vec<LunarPhaseEvent>, SearchError>` | All new moons in UTC range. |

## Grahan (7)

| Function | Inputs | Output | What it does |
|---|---|---|---|
| `next_chandra_grahan` | `engine`, `jd_tdb`, `config` | `Result<Option<ChandraGrahan>, SearchError>` | Next lunar eclipse after `jd_tdb`. |
| `prev_chandra_grahan` | `engine`, `jd_tdb`, `config` | `Result<Option<ChandraGrahan>, SearchError>` | Previous lunar eclipse before `jd_tdb`. |
| `search_chandra_grahan` | `engine`, `jd_start`, `jd_end`, `config` | `Result<Vec<ChandraGrahan>, SearchError>` | All lunar eclipses in range. |
| `besselian_elements_at` | `engine`, `eop`, `jd_tdb` | `Result<BesselianElements, SearchError>` | Instantaneous ephemeris-derived shadow elements. |
| `next_surya_grahan` | `engine`, `eop`, `jd_tdb`, `location`, `config` | `Result<Option<SuryaGrahan>, SearchError>` | Next geographic solar eclipse and optional local circumstances. |
| `prev_surya_grahan` | `engine`, `eop`, `jd_tdb`, `location`, `config` | `Result<Option<SuryaGrahan>, SearchError>` | Previous geographic solar eclipse. |
| `search_surya_grahan` | `engine`, `eop`, `jd_start`, `jd_end`, `location`, `config` | `Result<Vec<SuryaGrahan>, SearchError>` | Solar eclipses in a range, optionally including paths and footprints. |

Surya field products (all opt-in via `GrahanConfig`): `include_local_grid`
(+ `local_grid_step_deg`, clamped to [0.5, 10]) fills `SuryaGrahan.local_grid`
with per-cell local circumstances (local max magnitude/obscuration/time,
Sun-up-clipped first/last contacts, summed visible duration) at cell centers
`lat = -90 + (i + 0.5)·step`, `lon = -180 + (j + 0.5)·step`;
`include_isolines` (+ `duration_isoline_fractions` of the C1–C4 span and
`magnitude_isoline_levels`) fills `SuryaGrahan.isolines` with closed, ordered,
antimeridian-safe rings (`SuryaIsolineRing { boundary, contains_pole }`):
the level-0 `visibility_boundary` plus duration/magnitude level sets;
`include_central_corridor` fills `SuryaGrahan.central_corridor` with the
swept umbral/antumbral outline as `{ grahan_type, rings }` segments (hybrid
events return separate annular and total segments; rounded end caps come
from the exact swept level set on a track-aligned grid). Every event also
reports `centrality` (`Full | Partial | None`; `Partial` marks grazing
events whose center line misses Earth — one-sided limits, closed corridor).
`GrahanConfig::effective()` returns the clamped/sanitized configuration
actually applied; cache keys should be built against that echo.

Change 6 additions: every sampled `footprints[]` entry carries
`contains_pole` (decided on the sphere by the geometry producer).
`include_contact_footprints` fills `SuryaGrahan.contact_footprints` with
the instantaneous Sun-up-clipped visibility ring at each contact the event
actually has (`C1 | C2 | Greatest | C3 | C4`; the ring may be empty at
exact C1/C4 tangency — fall back to the nearest sampled footprint).
`include_umbra_footprints` fills `SuryaGrahan.umbra_footprints` with the
true instantaneous umbral/antumbral outlines (`grahan_type` per moment) at
every path timestamp plus the C2/greatest/C3 moments; partial events
return none.

`instantaneous_magnitude_levels` (non-empty list; values outside (0, 1.5]
dropped, sorted, deduplicated, capped at 16) adds
`magnitude_rings: Vec<SuryaMagnitudeRing>` (`{level, boundary,
contains_pole}`) to every sampled footprint and contact footprint: the
instantaneous iso-magnitude contour at that timestamp, terminator-clipped
like the visibility products, so the rings nest per timestamp (umbra ⊆
higher levels ⊆ lower levels ⊆ penumbral boundary). Levels the moment's
maximum magnitude does not reach are omitted.

## Sankranti (5)

| Function | Inputs | Output | What it does |
|---|---|---|---|
| `next_sankranti` | `engine`, `utc`, `config` | `Result<Option<SankrantiEvent>, SearchError>` | Next Sun entry into any rashi. |
| `prev_sankranti` | `engine`, `utc`, `config` | `Result<Option<SankrantiEvent>, SearchError>` | Previous Sun entry into any rashi. |
| `search_sankrantis` | `engine`, `start`, `end`, `config` | `Result<Vec<SankrantiEvent>, SearchError>` | All sankrantis in UTC range. |
| `next_specific_sankranti` | `engine`, `utc`, `rashi`, `config` | `Result<Option<SankrantiEvent>, SearchError>` | Next Sun entry into a chosen rashi. |
| `prev_specific_sankranti` | `engine`, `utc`, `rashi`, `config` | `Result<Option<SankrantiEvent>, SearchError>` | Previous Sun entry into a chosen rashi. |

## Stationary / Max-Speed (6)

| Function | Inputs | Output | What it does |
|---|---|---|---|
| `next_stationary` | `engine`, `body`, `jd_tdb`, `config` | `Result<Option<StationaryEvent>, SearchError>` | Next stationary point after `jd_tdb`. |
| `prev_stationary` | `engine`, `body`, `jd_tdb`, `config` | `Result<Option<StationaryEvent>, SearchError>` | Previous stationary point before `jd_tdb`. |
| `search_stationary` | `engine`, `body`, `jd_start`, `jd_end`, `config` | `Result<Vec<StationaryEvent>, SearchError>` | All stationary points in range. |
| `next_max_speed` | `engine`, `body`, `jd_tdb`, `config` | `Result<Option<MaxSpeedEvent>, SearchError>` | Next speed extremum after `jd_tdb`. |
| `prev_max_speed` | `engine`, `body`, `jd_tdb`, `config` | `Result<Option<MaxSpeedEvent>, SearchError>` | Previous speed extremum before `jd_tdb`. |
| `search_max_speed` | `engine`, `body`, `jd_start`, `jd_end`, `config` | `Result<Vec<MaxSpeedEvent>, SearchError>` | All speed extrema in range. |

## Panchang (26)

| Function | Inputs | Output | What it does |
|---|---|---|---|
| `masa_for_date` | `engine`, `utc`, `sankranti_config` | `Result<MasaInfo, SearchError>` | Amanta month + adhika flag + boundaries. |
| `masa_for_date_with_eop` | `engine`, `eop`, `utc`, `sankranti_config` | `Result<MasaInfo, SearchError>` | EOP-aware variant of `masa_for_date`. |
| `ayana_for_date` | `engine`, `utc`, `sankranti_config` | `Result<AyanaInfo, SearchError>` | Ayana + start/end transitions. |
| `ayana_for_date_with_eop` | `engine`, `eop`, `utc`, `sankranti_config` | `Result<AyanaInfo, SearchError>` | EOP-aware variant of `ayana_for_date`. |
| `varsha_for_date` | `engine`, `utc`, `sankranti_config` | `Result<VarshaInfo, SearchError>` | Samvatsara + Vedic year boundaries. |
| `varsha_for_date_with_eop` | `engine`, `eop`, `utc`, `sankranti_config` | `Result<VarshaInfo, SearchError>` | EOP-aware variant of `varsha_for_date`. |
| `elongation_at` | `engine`, `jd_tdb` | `Result<f64, SearchError>` | `(Moon_lon - Sun_lon) mod 360`. |
| `sidereal_sum_at` | `engine`, `jd_tdb`, `sankranti_config` | `Result<f64, SearchError>` | `(Moon_sid + Sun_sid) mod 360`. |
| `moon_sidereal_longitude_at` | `engine`, `jd_tdb`, `sankranti_config` | `Result<f64, SearchError>` | Moon sidereal longitude. |
| `nakshatra_for_date` | `engine`, `utc`, `sankranti_config` | `Result<PanchangNakshatraInfo, SearchError>` | Moon nakshatra/pada + boundaries. |
| `nakshatra_at` | `engine`, `jd_tdb`, `moon_sidereal_deg`, `sankranti_config` | `Result<PanchangNakshatraInfo, SearchError>` | Same using precomputed Moon sidereal longitude. |
| `tithi_for_date` | `engine`, `utc` | `Result<TithiInfo, SearchError>` | Tithi with paksha and boundaries. |
| `tithi_at` | `engine`, `jd_tdb`, `elongation_deg` | `Result<TithiInfo, SearchError>` | Same using precomputed elongation. |
| `karana_for_date` | `engine`, `utc` | `Result<KaranaInfo, SearchError>` | Karana with boundaries. |
| `karana_at` | `engine`, `jd_tdb`, `elongation_deg` | `Result<KaranaInfo, SearchError>` | Same using precomputed elongation. |
| `yoga_for_date` | `engine`, `utc`, `sankranti_config` | `Result<YogaInfo, SearchError>` | Yoga with boundaries. |
| `yoga_at` | `engine`, `jd_tdb`, `sidereal_sum_deg`, `sankranti_config` | `Result<YogaInfo, SearchError>` | Same using precomputed sidereal sum. |
| `vedic_day_sunrises` | `engine`, `eop`, `utc`, `location`, `riseset_config` | `Result<(f64, f64), SearchError>` | Sunrise and next-sunrise JD bounds for Vedic day. |
| `vaar_for_date` | `engine`, `eop`, `utc`, `location`, `riseset_config` | `Result<VaarInfo, SearchError>` | Vedic weekday from sunrise boundaries. |
| `vaar_from_sunrises` | `sunrise_jd`, `next_sunrise_jd`, `lsk` | `VaarInfo` | Weekday from sunrise pair (pure arithmetic). |
| `hora_for_date` | `engine`, `eop`, `utc`, `location`, `riseset_config` | `Result<HoraInfo, SearchError>` | Planetary hour with boundaries. |
| `hora_from_sunrises` | `jd_tdb`, `sunrise_jd`, `next_sunrise_jd`, `lsk` | `HoraInfo` | Hora from sunrise pair (pure arithmetic). |
| `ghatika_for_date` | `engine`, `eop`, `utc`, `location`, `riseset_config` | `Result<GhatikaInfo, SearchError>` | Ghatika with boundaries. |
| `ghatika_from_sunrises` | `jd_tdb`, `sunrise_jd`, `next_sunrise_jd`, `lsk` | `GhatikaInfo` | Ghatika from sunrise pair (pure arithmetic). |
| `panchang_for_date` | `engine`, `eop`, `utc`, `location: Option<&GeoLocation>`, `riseset_config`, `sankranti_config`, `include_mask: u32`, `known: &PanchangPrecomputed` | `Result<PanchangResult, SearchError>` | Combined panchang; `include_mask` gates which elements are computed, `location` required only for vaar/hora/ghatika, `known` reuses caller-cached calendar values inside their windows. |
| `panchang_events` | `engine`, `eop`, `from_utc`, `to_utc`, `include_mask`, `location: Option<&GeoLocation>`, `riseset_config`, `sankranti_config`, `max_events` | `Result<PanchangEventsResult, SearchError>` | Exact element segments over a range (all ten elements; location required only for vaar/hora/ghatika); segments chain exactly; `max_events` 0 = 50,000 ceiling with `truncated`/`next_from_utc` resume. |

## Jyotish Orchestration (13)

| Function | Inputs | Output | What it does |
|---|---|---|---|
| `graha_longitudes` | `engine`, `jd_tdb`, `config` | `Result<GrahaLongitudes, SearchError>` | 9 graha longitudes on the selected plane; `config.kind` chooses sidereal vs tropical/reference-plane output and config carries model/ayanamsha choices. |
| `moving_osculating_apogees` | `engine`, `jd_tdb`, `config`, `grahas` | `Result<MovingOsculatingApogees, SearchError>` | Moving heliocentric osculating apogees for Mangal, Buddh, Guru, Shukra, and Shani. |
| `moving_osculating_apogees_for_date` | `engine`, `eop`, `utc`, `config`, `grahas` | `Result<MovingOsculatingApogees, SearchError>` | UTC-date moving apogee helper using the same sidereal config semantics. |
| `special_lagnas_for_date` | `engine`, `eop`, `utc`, `location`, `riseset_config`, `aya_config` | `Result<AllSpecialLagnas, SearchError>` | Computes all special lagnas. |
| `arudha_padas_for_date` | `engine`, `eop`, `utc`, `location`, `bhava_config`, `aya_config` | `Result<[ArudhaResult; 12], SearchError>` | Computes arudha padas for 12 houses. |
| `all_upagrahas_for_date` | `engine`, `eop`, `utc`, `location`, `riseset_config`, `aya_config` | `Result<AllUpagrahas, SearchError>` | Computes all 11 upagrahas. |
| `graha_positions` | `engine`, `eop`, `utc`, `location`, `bhava_config`, `aya_config`, `config` | `Result<GrahaPositions, SearchError>` | Extended graha-position API. `config.include_equatorial` adds per-entry geocentric RA/declination/ecliptic latitude (equinox of date) and result-level `gmst_deg`/`gast_deg`. |
| `graha_positions_series` | `engine`, `eop`, `from_utc`, `to_utc`, `step_minutes`, `location`, `bhava_config`, `aya_config`, `config` | `Result<GrahaPositionsSeries, SearchError>` | Fixed-cadence samples of `graha_positions` over a range (endpoints inclusive on the grid, max 10,000 points). |
| `amsha_series` | `engine`, `eop`, `from_utc`, `to_utc`, `step_minutes`, `location`, `aya_config`, `amsha_requests`, `include_grahas` | `Result<AmshaSeries, SearchError>` | Fixed-cadence slim varga charts (grid semantics match `graha_positions_series`); varga lagna always, nine grahas optional; `points * unique_requests` capped at 100,000. |
| `amsha_lagna_events` | `engine`, `eop`, `from_utc`, `to_utc`, `location`, `aya_config`, `amsha_requests`, `max_segments` | `Result<AmshaLagnaEventsResult, SearchError>` | Exact varga-lagna rashi segments via root-found division-boundary crossings (no sampling grid); one entry per unique request; `max_segments` 0 = 50,000 ceiling with `truncated`/`next_from_utc` resume. |
| `ashtakavarga_for_date` | `engine`, `eop`, `utc`, `location`, `aya_config` | `Result<AshtakavargaResult, SearchError>` | Full ashtakavarga result. |
| `core_bindus` | `engine`, `eop`, `utc`, `location`, `bhava_config`, `riseset_config`, `aya_config`, `config` | `Result<BindusResult, SearchError>` | Curated bindu/sensitive points set. |
| `drishti_for_date` | `engine`, `eop`, `utc`, `location`, `bhava_config`, `riseset_config`, `aya_config`, `config` | `Result<DrishtiResult, SearchError>` | Graha drishti matrix (+ optional projections). |

## Forecast / Returns (1)

| Function | Inputs | Output | What it does |
|---|---|---|---|
| `gochar_events` | `engine`, `operation` | `Result<GocharEventsResult, SearchError>` | Grouped yearly/monthly Tajaka returns, yearly/monthly Tithi Pravesha returns, and transit conjunction/opposition/special-aspect searches from physical bodies plus Rahu/Ketu to caller-supplied natal target longitudes around a query time. |

## Related Detailed Docs

- Full inventory (includes helper methods): `docs/SEARCH_API_INVENTORY.md`
- Clean-room provenance: `docs/clean_room_conjunction.md`, `docs/clean_room_grahan.md`, `docs/clean_room_solar_eclipse_visibility.md`, `docs/clean_room_stationary.md`, `docs/clean_room_panchang.md`, `docs/clean_room_tithi_karana_yoga.md`, `docs/clean_room_ashtakavarga.md`, `docs/clean_room_drishti.md`, `docs/clean_room_upagraha.md`, `docs/clean_room_gochar_events.md`
