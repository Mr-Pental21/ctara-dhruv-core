# Python Wrapper (`python-open`)

Open-source Python bindings for `ctara-dhruv-core`, implemented against the
canonical C ABI (`dhruv_ffi_c`) via `cffi`.

## Status

- ABI target: `DHRUV_API_VERSION=89`
- Package root: `bindings/python-open`
- Runtime dependency: `cffi`
- Primary distribution: PyPI wheels plus sdist from unified `vX.Y.Z` tags

## End-User Docs

Usage-first documentation for this wrapper lives in
[`../../docs/end_user/python/README.md`](../../docs/end_user/python/README.md).

## Install

Published installs:

```bash
pip install ctara-dhruv
```

Local development from `bindings/python-open`:

```bash
pip install -e .
```

The shared `dhruv_ffi_c` library must be built from the repository root:

```bash
cargo build -p dhruv_ffi_c --release
```

Or use the repository helper to refresh the optimized local binaries, bundled
Python shared library, and local CLI/C ABI archives without cutting a release:

```bash
./scripts/ci/build_local_native_binaries.sh
```

Supported prebuilt wheel targets are Linux, macOS, and Windows x64 on the main
release matrix. Windows ARM64 remains best-effort until wheel support is proven
green in CI.

## Time-Based Upagraha Config

## Runtime SPK Replacement

Long-lived `Engine` handles can replace their active SPK set without being
recreated:

- `Engine.replace_spks(spk_paths)` returns a report with generation, active,
  loaded, and reused counts.
- `Engine.list_spks()` returns the active SPKs in query order.
- `ctara_dhruv.replace_spks(spk_paths)` applies the same replacement to the
  initialized module-level engine.

Replacement is all-or-nothing. Common kernels are reused when canonical path,
file size, and modified time match.

Dasha periods exposed by the Python wrapper now include `entity_name`, the
exact canonical Sanskrit entity name alongside the numeric entity fields.

Dasha `variation_config` dicts (accepted by `dasha_hierarchy`,
`dasha_snapshot`, `dasha_level0`, `dasha_level0_entity`, `dasha_children`,
`dasha_child_period`, and `dasha_complete_level`) support level-0 cycle
repetition:

- `cycles`: explicit level-0 whole-cycle repetition count. `0` (default) means
  the system default of one cycle. Wins over `min_span_years`.
- `min_span_years`: repeat whole cycles until level-0 coverage from birth
  reaches at least this many years; the final cycle completes past the target.
  `0.0` or negative (default) disables this.

Both options apply to nakshatra-based and Yogini dasha systems only; other
systems ignore them. The `order` field of returned periods is global across
cycles, so the cycle number of an entry is
`(order - 1) // sequence_len + 1`. The same two fields exist on the CFFI
dasha selection config used for embedded full-kundali dashas
(`config.dasha_config.cycles`, `config.dasha_config.min_span_years`).

The Python wrapper now also exposes `ctara_dhruv.gochar_events(...)` for grouped
Tajaka, Tithi Pravesha, and named transit-aspect event windows around a query
time. `transit_body_codes` accepts physical-body codes such as `499`, `599`,
`699`, `799`, `899`, `999`, plus `ctara_dhruv.GocharTransitBody.RAHU` and
`ctara_dhruv.GocharTransitBody.KETU`.

The Python wrapper exposes configurable time-based upagrahas through:

- `ctara_dhruv.vedic.time_upagraha_config_default()`
- `ctara_dhruv.vedic.all_upagrahas_for_date(..., upagraha_config=...)`
- `ctara_dhruv.kundali.core_bindus(..., bindus_config={"upagraha_config": ...})`
- `ctara_dhruv.kundali.full_kundali(..., config=...)`

Accepted dict values are:

- points: `"start"`, `"middle"`, `"end"`
- planets: `"rahu"`, `"saturn"`

## Panchang

`ctara_dhruv.panchang.panchang(...)` takes an optional `location` (default
`None`). Location-independent elements (tithi, karana, yoga, nakshatra, masa,
ayana, varsha) can be computed without one; requesting location-dependent
elements (vaar, hora, ghatika) without a location raises `DhruvError`.
`include_mask` accepts the `INCLUDE_*` constants including the convenience
masks `INCLUDE_LOCATION_INDEPENDENT` and `INCLUDE_LOCATION_DEPENDENT`.

Repeated nearby calls can skip the expensive new-moon/sankranti searches by
feeding calendar values from a previous result back in via the optional
`known_masa` (`MasaInfo`), `known_ayana` (`AyanaInfo`), and `known_varsha`
(`VarshaInfo`) keyword arguments. A known value is reused verbatim only when
its element is selected in `include_mask` and the requested moment falls
inside its `[start, end)` window; stale values are silently ignored and the
element is recomputed.

The full-kundali panchang section is selected with
`config.panchang_include_mask` (same `INCLUDE_*` bits; `0` omits the section —
this replaces the former `include_panchang`/`include_calendar` flags), and
`FullKundaliResult.panchang` is a `PanchangResult` with per-element optional
fields, identical to the standalone panchang op.

`ctara_dhruv.panchang.panchang_events(...)` streams exact panchang element
segments over a UTC range for all ten kinds (tithi, karana, yoga, vaar,
hora, ghatika, nakshatra, masa, ayana, varsha). The location-dependent
kinds (vaar, hora, ghatika) require the optional `location` argument
(`GeoLocation`); requesting them without one raises
`InvalidSearchConfigError`. An optional `riseset_config` pointer tunes the
sunrise model for those kinds. Segments of one kind chain exactly
(`end == next.start`), including across Vedic-day rolls; vaar segments are
sunrise-to-sunrise Vedic days, hora/ghatika their 24/60 subdivisions. The
first segment may start before `from_utc` and the last may end after
`to_utc`. Sweeps are capped at `MAX_PANCHANG_EVENTS` (50,000) events; a
truncated result carries a `next_from` resume point (dedup on
`(kind, start)` when merging).

## Search: Nodes, Any-Body Sankranti, Multi-Angle Conjunctions

The `ctara_dhruv.search` conjunction, motion, and sankranti operations accept
the lunar node body codes `10007` (Rahu) and `10008` (Ketu) alongside NAIF
codes (conjunction `body1_code`/`body2_code`, motion `body_code`, sankranti
`body_code`). The `DhruvSankrantiConfig`, `DhruvConjunctionConfig`, and
`DhruvStationaryConfig` structs (and the dict form accepted where sankranti
configs are dict-based) carry a `node_mode` field selecting the node model
(`0` = mean, `1` = true; default `1`). Stationary search of Rahu/Ketu
requires the true node — the true node stations roughly weekly — and rejects
`node_mode=0` with an invalid-query/config error.

Sankranti search generalizes to any-body rashi ingress: `next_sankranti`,
`prev_sankranti`, `specific_sankranti`, and `search_sankrantis` take an
optional `body_code` (`0` = Sun, the classical default). `SankrantiEvent`
now carries `body_code`, `sidereal_longitude_deg`, `tropical_longitude_deg`,
and `is_retrograde` (retrograde bodies re-entering a rashi); the legacy
`sun_sidereal_longitude_deg` / `sun_tropical_longitude_deg` fields remain as
aliases for the tracked body's longitudes.

Fixed-longitude search (`next_fixed_longitude`, `prev_fixed_longitude`,
`search_fixed_longitudes`) finds when a moving body reaches a fixed
sidereal longitude (`target_longitude_deg`), optionally offset by
`angles_deg` (offsets added to the target mod 360; at most
`MAX_FIXED_LONGITUDE_ANGLES` = 16). `include_special_angles=True` also
searches the body's classical special-aspect angles (Mars 90/210,
Jupiter 120/240, Saturn 60/270) applied so the moving body casts that
aspect onto the target. The config is the sankranti config (struct or
dict); `body_code` defaults to the Sun. Range results are sorted by time
then angle, and a range crossing the ephemeris coverage edge returns the
events found up to the edge.

Conjunction searches additionally accept `target_separations` (a list of up
to `MAX_CONJUNCTION_TARGETS` = 16 target angles swept in one pass; events
echo the matched angle as `target_separation_deg`) and `sidereal_config` (a
`DhruvSankrantiConfig` struct or dict). Motion searches accept the same
`sidereal_config`. When it is set, conjunction events carry
`body1_sidereal_longitude_deg` / `body2_sidereal_longitude_deg` and
`body1_rashi_index` / `body2_rashi_index`, and stationary/max-speed events
carry `sidereal_longitude_deg` / `rashi_index` (all `None` otherwise, with
`has_sidereal` flagging presence).

## Amsha Surface

Direct amsha helpers:

- `ctara_dhruv.amsha.amsha_longitude`
- `ctara_dhruv.amsha.amsha_rashi_info`
- `ctara_dhruv.amsha.amsha_longitudes`
- `ctara_dhruv.amsha.amsha_chart_for_date`
- `ctara_dhruv.amsha.amsha_variations`
- `ctara_dhruv.amsha.amsha_variations_many`

Amsha range operations:

- `ctara_dhruv.amsha.amsha_series` — fixed-cadence slim varga charts
  (`AmshaSeriesPoint` list; lagna always, grahas with `include_grahas`;
  points x unique requests capped at `MAX_AMSHA_SERIES_CELLS` = 100,000).
- `ctara_dhruv.amsha.amsha_lagna_events` — exact varga-lagna rashi
  transition segments per unique request, chaining `end == next.start`
  (total segments capped at `MAX_AMSHA_LAGNA_SEGMENTS` = 50,000; truncated
  results carry a `next_from` resume point).

Charakaraka range operations:

- `ctara_dhruv.kundali.charakaraka_events` — every chara-karaka
  ranking-change event in a UTC range for a scheme, each carrying
  `trigger`/`trigger_name`, `changed_roles`, and full `before`/`after`
  rankings (capped at `MAX_CHARAKARAKA_EVENTS` = 50,000; truncated results
  carry a `next_from` resume point).
- `ctara_dhruv.kundali.next_charakaraka_event` /
  `ctara_dhruv.kundali.prev_charakaraka_event` — first ranking change
  strictly after / last strictly before a UTC instant (`None` when none is
  found before the coverage edge).

Build identity helpers (process-wide): `ctara_dhruv.library_version()` and
`ctara_dhruv.build_git_hash()` return the native library's version string
and build git hash (`"unknown"` outside a git checkout).

`GrahaLongitudesConfig` (and the dict form accepted by
`graha_longitudes`) now carries `node_mode` selecting the Rahu/Ketu node
model (`0` = mean, `1` = true; default `1`), matching the sankranti-config
field of the same name.

Embedded amsha support:

- `ctara_dhruv.kundali.full_kundali_config_default`
- `ctara_dhruv.kundali.full_kundali`

Relevant full-kundali config fields:

- `config.include_amshas`
- `config.amsha_selection`
- `config.amsha_scope`

Standalone bala helpers also accept `amsha_selection`:

- `ctara_dhruv.shadbala.shadbala`
- `ctara_dhruv.shadbala.vimsopaka`
- `ctara_dhruv.shadbala.balas`
- `ctara_dhruv.shadbala.avastha`

Optional amsha chart sections extracted by the wrapper:

- `bhava_cusps`
- `arudha_padas`
- `upagrahas`
- `sphutas`
- `special_lagnas`
- `outer_planets`

Graha-position and graha-longitude outputs keep their traditional navagraha
lists at length 9 and expose Uranus, Neptune, and Pluto separately as
`outer_planets`. Outer planets are positional display entities only; they are
not inputs to bala, avastha, dasha, drishti, or lordship calculations.

## Equatorial Output

`ctara_dhruv.kundali.graha_positions` accepts `config["include_equatorial"]`
(0/1; also `graha_positions_config.include_equatorial` on the full-kundali
config). When enabled, each entry reports `equatorial_valid=True` with
geocentric `right_ascension_deg` ([0, 360)), `declination_deg`
([-90, +90]), and `ecliptic_latitude_deg` in degrees — equinox of date,
nutation applied per the request's `use_nutation` flag, geometric (no
light-time or aberration). Lagna and Rahu/Ketu report ecliptic latitude
exactly 0. The result additionally carries `earth_orientation_valid`,
`gmst_deg`, and `gast_deg` (Greenwich mean/apparent sidereal time in degrees
at the request instant).

`ctara_dhruv.kundali.graha_positions_series` samples the same op at a
fixed cadence: it takes `from_utc`, `to_utc`, and `step_minutes` instead
of a single instant (endpoints inclusive when on the grid, at most
10,000 points) and returns a list of `GrahaPositionsPoint` values, each
with `utc`, `jd_utc`, and a `positions` value of the single-epoch shape.

Grahan results also carry apparent equatorial coordinates at greatest
grahan: `ChandraGrahanResult.moon_right_ascension_deg` /
`moon_declination_deg` and `SuryaGrahanResult.sun_right_ascension_deg` /
`sun_declination_deg` (degrees, equinox of date, nutation applied).

The grahan config additionally exposes the surya field products
(`include_local_grid`/`local_grid_step_deg`, `include_isolines` with
`duration_isoline_fractions`/`magnitude_isoline_levels`, and
`include_central_corridor`); `SuryaGrahanResult` then carries `centrality`,
`local_grid` (`SuryaLocalGridSample`), `isolines` (`SuryaIsolines`), and
`central_corridor` (`SuryaRingSetLevel` segments), and
`search.grahan_config_effective(config)` echoes the clamped/sanitized
configuration for cache identity. `include_contact_footprints` /
`include_umbra_footprints` add `contact_footprints` and `umbra_footprints`;
sampled footprints carry `contains_pole`.

## Example

```python
import ctara_dhruv
from ctara_dhruv.engine import engine, lsk, eop
from ctara_dhruv.amsha import amsha_chart_for_date
from ctara_dhruv.kundali import full_kundali, full_kundali_config_default

ctara_dhruv.init(
    ["../../kernels/data/de442s.bsp"],
    "../../kernels/data/naif0012.tls",
    "../../kernels/data/finals2000A.all",
)

chart = amsha_chart_for_date(
    engine(), lsk(), eop(),
    jd_utc=(2024, 1, 15, 6, 0, 0.0),
    location=(28.6139, 77.2090),
    amsha_code=9,
    scope={
        "include_sphutas": 1,
        "include_special_lagnas": 1,
    },
)

cfg = full_kundali_config_default()
cfg.include_sphutas = 1
cfg.include_special_lagnas = 1
cfg.include_amshas = 1
cfg.amsha_selection.count = 1
cfg.amsha_selection.codes[0] = 9
cfg.amsha_scope.include_sphutas = 1
cfg.amsha_scope.include_special_lagnas = 1

kundali = full_kundali(
    engine(), lsk(), eop(),
    jd_utc=(2024, 1, 15, 6, 0, 0.0),
    location=(28.6139, 77.2090),
    config=cfg,
)
```

For embedded amsha sections in `full_kundali`, remember that scoped amsha
sub-sections depend on the corresponding root full-kundali sections also being
enabled. Returned `kundali.amshas` charts now reflect the full resolved amsha
set used by the call: explicit `config.amsha_selection` first, then any
internally required amshas for shadbala, vimsopaka, or avastha.
Variation codes are amsha-scoped; use `amsha_variations*` to discover the
valid codes, names, labels, and defaults for a given amsha.

## Low-Level Helper Coverage

`ctara_dhruv.vedic` now also exposes the intended low-level helper family:

- graha relationship and dignity helpers such as `naisargika_maitri`,
  `tatkalika_maitri`, `panchadha_maitri`, `dignity_in_rashi`, and
  `node_dignity_in_rashi`
- combustion helpers such as `combustion_threshold`, `is_combust`, and
  `all_combustion_status`
- classification/lord helpers such as `natural_benefic_malefic`,
  `moon_benefic_nature`, `graha_gender`, `hora_lord`, `masa_lord`, and
  `samvatsara_lord`

`ctara_dhruv.tara` also exposes the low-level tara primitives:

- `propagate_position`
- `apply_aberration`
- `apply_light_deflection`
- `galactic_anticenter_icrs`
