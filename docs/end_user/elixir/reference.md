# Elixir Reference

This page summarizes the public Elixir wrapper from the `CtaraDhruv` modules,
using `bindings/elixir-open/lib/` as the source of truth.

## Public Modules

- `CtaraDhruv`
- `CtaraDhruv.Engine`
- `CtaraDhruv.Ephemeris`
- `CtaraDhruv.Time`
- `CtaraDhruv.Math`
- `CtaraDhruv.Vedic`
- `CtaraDhruv.Panchang`
- `CtaraDhruv.Search`
- `CtaraDhruv.Jyotish`
- `CtaraDhruv.Dasha`
- `CtaraDhruv.Tara`

## Exact Function Inventory

`CtaraDhruv`:

- `new_engine/1`

`CtaraDhruv.Engine`:

- `new/1`
- `close/1`
- `load_config/2`
- `clear_config/1`
- `load_eop/2`
- `clear_eop/1`
- `load_tara_catalog/2`
- `reset_tara_catalog/1`
- `replace_spks/2`
- `list_spks/1`

`replace_spks/2` atomically swaps the full active SPK set for a long-lived
engine. `list_spks/1` returns the active SPKs in query order.

`load_config/2` accepts either:

- a string path convenience form
- or a request map with optional `:path` and `:defaults_mode`

When `:path` is omitted, discovery mode is used.

`new/1` accepts optional `:cache_capacity` and `:strict_validation`. When they
are omitted, the Elixir wrapper aligns with the shared wrapper convention:
`cache_capacity: 256` and `strict_validation: true`.

`CtaraDhruv.Ephemeris`:

- `query/2`
- `body_ecliptic_lon_lat/2`
- `cartesian_to_spherical/1`

`CtaraDhruv.Time`:

- `utc_to_jd_tdb/2`
  Requests may include `time_policy`; results now include `diagnostics`.
- `jd_tdb_to_utc/2`
- `nutation/1`
- `nutation_utc/2`
- `approximate_local_noon/1`
- `ayanamsha_system_count/0`
- `reference_plane_default/1`

`CtaraDhruv.Math`:

- classifiers and canonical name lookups:
  `rashi_from_longitude/1`, `nakshatra_from_longitude/1`,
  `nakshatra28_from_longitude/1`, `rashi_from_tropical/1`,
  `nakshatra_from_tropical/1`, `nakshatra28_from_tropical/1`,
  `graha_name/1`, `yogini_name/1`, `rashi_name/1`, `nakshatra_name/1`,
  `nakshatra28_name/1`, `sphuta_name/1`, `upagraha_name/1`
  `graha_name/1` accepts either `%{index: n}` or `%{graha: value}` and returns
  canonical graha names such as `Mangal` and `Buddh`; `Mangala` is a Yogini
  dasha name returned by `yogini_name/1`, not a graha name.
- relationship, dignity, combustion, and lord helpers:
  `hora_lord/1`, `masa_lord/1`, `samvatsara_lord/1`,
  `exaltation_degree/1`, `debilitation_degree/1`, `moolatrikone_range/1`,
  `combustion_threshold/1`, `combust?/1`, `all_combustion_status/1`,
  `naisargika_maitri/1`, `tatkalika_maitri/1`, `panchadha_maitri/1`,
  `dignity_in_rashi/1`, `dignity_in_rashi_with_positions/1`,
  `node_dignity_in_rashi/1`, `natural_benefic_malefic/1`,
  `moon_benefic_nature/1`, `graha_gender/1`
- drishti, upagraha, sphuta, and ashtakavarga helpers:
  `graha_drishti/1`, `graha_drishti_matrix/1`, `sun_based_upagrahas/1`,
  `time_upagraha_jd/1`, `all_sphutas/1`, `calculate_ashtakavarga/1`,
  `calculate_bav/1`, `calculate_all_bav/1`, `calculate_sav/1`,
  `trikona_sodhana/1`, `ekadhipatya_sodhana/1`
- engine-free amsha helpers:
  `amsha_rashi_infos/1`, `amsha_variations/1`, `amsha_variations_many/1`

`amsha_rashi_infos/1` is an engine-free batched varga mapping. The request
takes `:longitudes` (list of sidereal longitudes in degrees) and
`:amsha_requests` (`[%{code: 9}, %{code: 2, variation: 1}, ...]`); the result
carries `:entries` — one list per input longitude with one map per request
carrying `:amsha_longitude`, `:rashi`, `:rashi_index`, `:degrees_in_rashi`,
and `:dms`:

```elixir
{:ok, %{entries: [[d9, d10]]}} =
  CtaraDhruv.Math.amsha_rashi_infos(%{
    longitudes: [123.456],
    amsha_requests: [%{code: 9}, %{code: 10}]
  })

d9.rashi_index  # 0-based varga rashi of the first longitude in D9
```

`CtaraDhruv.Vedic`:

- `ayanamsha/2`
- `lunar_node/2`
- `rise_set/2`
- `all_events/2`
- `lagna/2`
- `lagna/3`
- `mc/2`
- `mc/3`
- `ramc/2`
- `bhavas/2`
- `bhavas/3`

`CtaraDhruv.Panchang`:

- `tithi/2`
- `karana/2`
- `yoga/2`
- `nakshatra/2`
- `vaar/2`
- `hora/2`
- `ghatika/2`
- `masa/2`
- `ayana/2`
- `varsha/2`
- `daily/2`
- `elongation_at/2`
- `sidereal_sum_at/2`
- `vedic_day_sunrises/2`
- `body_ecliptic_lon_lat/2`
- `tithi_at/2`
- `karana_at/2`
- `yoga_at/2`
- `nakshatra_at/2`
- `vaar_from_sunrises/2`
- `hora_from_sunrises/2`
- `ghatika_from_sunrises/2`
- `ghatika_from_elapsed/1`
- `ghatikas_since_sunrise/1`
- `events/2`

`daily/2` accepts an optional `:include_mask` selecting which panchang
elements are computed: an integer mask, a single element/group name string
(`"tithi"`, `"all"`, `"all_core"`, `"all_calendar"`,
`"location_independent"`, `"location_dependent"`, ...), or a list of such
names OR-ed together. When omitted it defaults to the core elements
(`all_core`), preserving the historical `daily` payload; add calendar
elements (masa, ayana, varsha) via the mask. `:location` is optional and
required only when the selection includes a location-dependent element
(vaar, hora, ghatika). Only selected elements are computed and returned.

`daily/2` also accepts optional `:known_masa`, `:known_ayana`, and
`:known_varsha` — the calendar maps a previous `daily`/`events` result
returned, fed back verbatim. A known value is reused (its
new-moon/sankranti searches skipped) only when its element is selected and
the requested moment lies inside the value's `start`/`end` window; stale
values are silently recomputed. Unknown enum names are rejected with
`:invalid_request`.

`events/2` streams exact panchang element segments over a UTC range. The
request takes `:from_utc` and `:to_utc`, an optional `:include_mask` (any
element/group names or bits — integer mask, name, or list of names/atoms,
like `Panchang.daily/2`; defaults to `location_independent`), an optional
`:location` (required only when the selection includes `vaar`, `hora`, or
`ghatika` — otherwise the call fails with a `search_error` naming the
missing location), and an optional `:max_events` cap (default `0` selects
the 50,000 ceiling). The
result carries per-kind lists in the same shape as the per-moment ops
(unselected kinds are `[]`) plus `:truncated` and `:next_from_utc`;
consecutive segments of one kind chain exactly (`end == next start`). On
truncation, resume from `:next_from_utc` and deduplicate on `{kind, start}`:

```elixir
{:ok, events} =
  CtaraDhruv.Panchang.events(engine, %{
    from_utc: %{year: 2026, month: 4, day: 1, hour: 0, minute: 0, second: 0.0},
    to_utc: %{year: 2026, month: 5, day: 1, hour: 0, minute: 0, second: 0.0},
    include_mask: [:tithi, :masa]
  })

[first_tithi | _] = events.tithi   # %{tithi: ..., start: ..., end: ...}
[masa | _] = events.masa
refute events.truncated
```

`CtaraDhruv.Jyotish`:

- `graha_longitudes/2`
  Returns the 9 navagrahas in `:grahas` / `:longitudes` and Uranus, Neptune,
  and Pluto in sibling `:outer_planets`.
- `moving_osculating_apogees/2`
- `graha_positions/2`
  request `:graha_positions_config` may include nested
  `:basic_states_config` with `:include_basic_states` and
  `:include_sensitive_point_distances`, plus `:include_equatorial` (default
  `false`).
  Keeps `:grahas` as the 9 navagrahas and exposes positional-only outer grahas
  separately as `:outer_planets`.
  With `:include_equatorial` enabled, each entry carries
  `:equatorial_valid`, `:right_ascension_deg`, `:declination_deg`, and
  `:ecliptic_latitude_deg` — geocentric coordinates in degrees, equinox of
  date, nutation per the request's `:use_nutation` flag, geometric (no
  light-time or aberration). Lagna and Rahu/Ketu report ecliptic latitude
  exactly `0.0`. The result also carries `:earth_orientation_valid`,
  `:gmst_deg`, and `:gast_deg` (Greenwich mean/apparent sidereal time in
  degrees), populated when equatorial output is requested. The same fields
  appear in the embedded `full_kundali` `:graha_positions` block.
- `special_lagnas/2`
- `arudha/2`
- `upagrahas/2`
- `bindus/2`
- `ashtakavarga/2`
- `drishti/2`
- `charakaraka/2`
- `shadbala/2`
- `bhavabala/2`
- `vimsopaka/2`
- `balas/2`
- `avastha/2`
- `full_kundali/2`
- `full_kundali/3`
  nested `:graha_positions_config[:basic_states_config]` flows through to the
  same entry/state outputs, and sensitive-point distance mode can also surface
  bhava-cusp distance arrays
- `amsha/2`
- `amsha_series/2`
- `amsha_lagna_events/2`

- `CtaraDhruv.Jyotish.graha_positions_series/2` — fixed-cadence sampling
  of `graha_positions/2` via `:from_utc`, `:to_utc`, `:step_minutes`
  (endpoints inclusive on the grid, max 10,000 points); returns
  `%{points: [%{utc: ..., jd_utc: ..., positions: ...}]}`.

`amsha_series/2` samples slim varga charts at a fixed cadence over
`[from_utc, to_utc]`, with the same grid semantics as
`graha_positions_series/2`. The request takes `:from_utc`, `:to_utc`,
`:step_minutes`, `:location`, and `:amsha_requests`
(`[%{code: 9}, %{code: 2, variation: 1}, ...]`), plus an optional
`:include_grahas` boolean (default `false`) that adds the nine graha varga
entries per chart. At most 100,000 cells (points x unique requests). Each
point carries `:utc`, `:jd_utc`, and `:charts` in request order; each chart
carries `:amsha`, `:variation_code`, `:lagna`, and `:grahas` (`nil` unless
requested), with entries in the single-epoch `amsha/2` entry shape:

```elixir
{:ok, series} =
  CtaraDhruv.Jyotish.amsha_series(engine, %{
    from_utc: %{year: 2026, month: 4, day: 17, hour: 0, minute: 0, second: 0.0},
    to_utc: %{year: 2026, month: 4, day: 18, hour: 0, minute: 0, second: 0.0},
    step_minutes: 60,
    location: %{latitude_deg: 28.6139, longitude_deg: 77.2090, altitude_m: 0.0},
    amsha_requests: [%{code: 9}, %{code: 10}],
    include_grahas: true
  })

[point | _] = series.points
[d9_chart, _d10_chart] = point.charts
d9_chart.lagna.rashi_index
```

`amsha_lagna_events/2` returns exact varga-lagna rashi segments over
`[from_utc, to_utc]` (root-found transitions, no sampling grid, so fast
vargas such as D60 cannot alias). The request takes `:from_utc`, `:to_utc`,
`:location`, and `:amsha_requests`, plus an optional `:max_segments` cap
(default `0` selects the 50,000 ceiling). The result carries `:entries`
(one per unique request, each with `:amsha`, `:variation_code`, and
`:segments` maps of `:rashi`, `:rashi_index`, `:start`, `:end`),
`:truncated`, and `:next_from_utc`. On truncation resume from
`:next_from_utc`:

```elixir
{:ok, events} =
  CtaraDhruv.Jyotish.amsha_lagna_events(engine, %{
    from_utc: %{year: 2026, month: 4, day: 17, hour: 0, minute: 0, second: 0.0},
    to_utc: %{year: 2026, month: 4, day: 18, hour: 0, minute: 0, second: 0.0},
    location: %{latitude_deg: 28.6139, longitude_deg: 77.2090, altitude_m: 0.0},
    amsha_requests: [%{code: 60}]
  })

[d60_entry] = events.entries
[segment | _] = d60_entry.segments  # %{rashi: ..., rashi_index: ..., start: ..., end: ...}
```

Grahan results also carry apparent equatorial coordinates at greatest
grahan: `moon_right_ascension_deg`/`moon_declination_deg` on chandra
grahan results and `sun_right_ascension_deg`/`sun_declination_deg` on
surya grahan results (degrees, equinox of date, nutation applied).

Solar grahan requests also accept `location` and `config.include_path`,
`path_step_minutes`, and `boundary_step_deg`. Results include Besselian
elements, greatest location, map path/footprint samples, and local visibility
circumstances. The config additionally accepts `include_local_grid` /
`local_grid_step_deg`, `include_isolines` with
`duration_isoline_fractions` / `magnitude_isoline_levels`, and
`include_central_corridor`; surya results then carry `centrality`,
`local_grid`, `isolines` (`visibility_boundary`, `duration_isolines`,
`magnitude_isolines` with `%{boundary: [...], contains_pole: ...}` rings),
and `central_corridor.segments`. The grahan response envelope echoes the
effective (clamped/sanitized) configuration under `effective_config` for
cache identity. See `docs/end_user/solar_eclipse_visibility.md`.

`CtaraDhruv.Search`:

- `conjunction/2`
- `grahan/2`
- `lunar_phase/2`
- `sankranti/2`
- `motion/2`

High-level search results carry structured UTC on the main event payloads.
Conjunction, grahan, and motion results now include UTC alongside JD/TDB where
numeric JD is still exposed; sankranti and lunar-phase results remain UTC-first.

The same search request maps accept `:at_utc`, `:start_utc`, and `:end_utc`
alongside `:at_jd_tdb`, `:start_jd_tdb`, and `:end_jd_tdb`, keeping UTC input
on the main operations instead of splitting out separate UTC-specific APIs.

`CtaraDhruv.Dasha`:

- `hierarchy/2`
- `snapshot/2`
- `level0/2`
- `level0_entity/2`
- `children/2`
- `child_period/2`
- `complete_level/2`

Dasha request maps accept an optional `:variation` map with `:level_methods`,
`:yogini_scheme`, `:use_abhijit`, `:cycles`, and `:min_span_years`.
`:cycles` (integer >= 1) sets an explicit level-0 whole-cycle repetition count
(system default when omitted) and wins over `:min_span_years` when both are
set. `:min_span_years` (positive number) repeats whole mahadasha cycles until
level-0 coverage from birth reaches at least that many years; the final cycle
completes past the target. Both apply to nakshatra-based and Yogini dasha
systems only; other systems ignore them. A period's cycle number is
`(order - 1) / sequence_len + 1`, where `order` is global across cycles.

Returned dasha entity maps include `:name` with the exact canonical Sanskrit
entity name.

Chara-style dasha periods use dual lordship for Kumbha (`Shani`/`Rahu`) and
Vrischika (`Mangal`/`Ketu`). Rahu owns Kumbha and Ketu owns Vrischika for the
default sign-lord-based node dignity policy.

`CtaraDhruv.Tara`:

- `compute/2`
- `catalog_info/1`
- `propagate_position/1`
- `apply_aberration/1`
- `apply_light_deflection/1`
- `galactic_anticenter_icrs/0`

## Request Config Maps

Common request maps can include:

- `utc`
- `location`
- `sankranti_config`
- `riseset_config`
- `bhava_config`
- `time_policy`

Chart-related config maps:

- `graha_positions_config`
- `bindus_config`
- `drishti_config`
- `full_kundali_config`
- `amsha_scope`
- `amsha_selection`
- search request maps with `:op`-specific fields for conjunction, grahan, lunar phase, sankranti, and motion
- dasha request maps for hierarchy, snapshot, `level0`, `level0_entity`, `children`, `child_period`, and `complete_level` queries

Standalone `:shadbala`, `:vimsopaka`, `:balas`, and `:avastha` jyotish request
maps accept `:amsha_selection`. Embedded `:full_kundali` `:amshas` results now
return the resolved amsha union used by the call. Use
`CtaraDhruv.Math.amsha_variations/1` and
`CtaraDhruv.Math.amsha_variations_many/1` to discover valid variation codes,
names, labels, and defaults for each amsha.

Avastha result maps expose `:deeptadi` as the primary compatibility value and
`:deeptadi_states` / `:deeptadi_mask` as the full set of Deeptadi states that
apply to the graha. They also expose `:lajjitadi`, `:lajjitadi_states`, and
`:lajjitadi_mask`; `:lajjitadi` is `nil` when no Lajjitadi condition applies.

`bhava_config` maps may include `:chandra_benefic_rule`. The default is the
72-degree brightness rule (`:brightness_72`, `"brightness-72"`, or `0`), where
Chandra is benefic when its smaller angular distance from Surya is at least
72 degrees. Use `:waxing_180`, `"waxing-180"`, or `1` for the prior waxing-arc
rule where Chandra is benefic when `normalize_360(Chandra - Surya) <= 180`.
The same Chandra rule is used by Buddh's association-based benefic/malefic
classification in Shadbala Drik Bala and Bhava Bala Drishti Bala.
They may also include `:sayanadi_ghatika_rounding`; default `:floor`/`0` uses
completed ghatikas, while `:ceil`/`1` counts the current partial ghatika.

Time-based upagraha config map:

- `gulika_point`
- `maandi_point`
- `other_point`
- `gulika_planet`
- `maandi_planet`

Accepted enum-style string values:

- points: `"start"`, `"middle"`, `"end"`
- planets: `"rahu"`, `"saturn"`

## Wrapper Behavior Notes

- `bindus_config` and `full_kundali_config` can both carry `upagraha_config`.
- `full_kundali/3`, `lagna/3`, `mc/3`, and `bhavas/3` are convenience arities that inject `:sankranti_config`.
- Enum-like strings are normalized through the NIF boundary and usually come back as atoms in results.
- `full_kundali_config[:dasha_config]` supports `:systems`, `:max_level`,
  `:max_levels`, `:snapshot_utc`, `:cycles`, and `:min_span_years`. The
  `:cycles` / `:min_span_years` semantics match the standalone dasha
  `:variation` map.
- `full_kundali_config[:panchang_include_mask]` selects the panchang elements
  in full-kundali output (integer mask, element/group name string, or list of
  names — the same forms as `Panchang.daily/2` `:include_mask`; `"none"` or
  omitted = no panchang section). It replaces the former `:include_panchang` /
  `:include_calendar` keys, and the result's panchang block contains only the
  selected elements.
- Dasha period results now include `:start_utc` / `:end_utc` alongside
  `:start_jd` / `:end_jd`, and dasha snapshots include `:query_utc`
  alongside `:query_jd`.

For build/runtime notes, see [`bindings/elixir-open/README.md`](../../../bindings/elixir-open/README.md).

## Rashi-Bhava Bhava Config

Elixir bhava config maps accept `:use_rashi_bhava_for_bala_avastha`, `:include_rashi_bhava_results`, and `:include_special_bhavabala_rules`; all default to `true`. They also accept `:include_node_aspects_for_drik_bala`, defaulting to `false`, to include Rahu/Ketu incoming aspects in Shadbala Drik Bala and Bhava Bala Drishti Bala. `:divide_guru_buddh_drishti_by_4_for_drik_bala` defaults to `true`; set it to `false` to add Guru/Buddh incoming aspects at full signed strength instead of through the divided Drik Bala balance. Existing bhava fields keep configured-system meaning. Rashi-bhava sibling keys such as `:rashi_bhava_cusps`, `:rashi_bhava_number`, and `:graha_to_rashi_bhava` expose the equal-house/whole-sign companion basis.
