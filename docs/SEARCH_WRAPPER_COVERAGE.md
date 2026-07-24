# dhruv_search C ABI Coverage

Scope: crate-root runtime/query APIs re-exported by `dhruv_search`
(74 functions; enumerated in `docs/SEARCH_RUNTIME_APIS.md`).

The search/event families are covered by unified operation entry points
(`dhruv_*_search_ex` request structs mirroring the `dhruv_search`
operations) rather than one export per crate-root function; per-moment
panchang/jyotish helpers keep direct exports. Functional coverage is
`74 / 74` when the gaps below are satisfied through other exports.

## Not Wrapped Directly

These crate-root runtime functions do not currently have a direct C export:

- `moon_sidereal_longitude_at`
- `masa_for_date_with_eop`
- `ayana_for_date_with_eop`
- `varsha_for_date_with_eop`
- `transit_body_ecliptic_lon_lat`

Functional coverage notes:
- `moon_sidereal_longitude_at` is obtainable from
  `dhruv_graha_longitudes` with sidereal config (Moon is one graha entry in that output).
- The `*_for_date_with_eop` variants are obtainable from
  `dhruv_panchang_compute_ex` (EOP handle + `DHRUV_PANCHANG_INCLUDE_MASA`/`_AYANA`/`_VARSHA` bits).
- `transit_body_ecliptic_lon_lat` splits into existing exports: plain
  bodies via `dhruv_body_ecliptic_lon_lat`, node longitudes via
  `dhruv_lunar_node_compute_ex` (node latitude is 0 by definition).

## Wrapped API Families

- Conjunction/aspect: `dhruv_conjunction_search_ex` (next/prev/range query
  modes covering `next_conjunction`, `prev_conjunction`,
  `search_conjunctions`; `TransitBody` codes 10007/10008 for Rahu/Ketu,
  `node_mode`, multi-angle `target_separations_deg`, optional sidereal
  echo config) plus `dhruv_conjunction_config_default` and
  `dhruv_body_ecliptic_lon_lat`
- Lunar phase: `dhruv_lunar_phase_search_ex` (kind + next/prev/range,
  covering `next_purnima`, `prev_purnima`, `next_amavasya`,
  `prev_amavasya`, `search_purnimas`, `search_amavasyas`)
- Grahan: `dhruv_grahan_search_ex` (kind + next/prev/range, covering the
  `*_chandra_grahan`/`*_surya_grahan` functions) plus the geometry
  accessors and `dhruv_grahan_config_effective`
- Sankranti/ingress: `dhruv_sankranti_search_ex` (target + next/prev/range;
  the request's `body_code` — 0 = Sun back-compat default, else a NAIF code
  or 10007/10008 — covers the generalized `next_ingress`, `prev_ingress`,
  `search_ingresses`, `next_specific_ingress`, `prev_specific_ingress` as
  well as the classical Sun wrappers) plus
  `dhruv_sankranti_config_default`
- Stationary/max-speed: `dhruv_motion_search_ex` (kind + next/prev/range
  with `body_code` incl. 10007/10008, `node_mode`, optional sidereal echo
  config, covering the `*_stationary`/`*_max_speed` functions) plus
  `dhruv_stationary_config_default`
- Panchang/time slices: `dhruv_masa_for_date`, `dhruv_ayana_for_date`,
  `dhruv_varsha_for_date`, `dhruv_nakshatra_for_date`, `dhruv_tithi_for_date`,
  `dhruv_karana_for_date`, `dhruv_yoga_for_date`, `dhruv_vaar_for_date`,
  `dhruv_hora_for_date`, `dhruv_ghatika_for_date`, `dhruv_panchang_compute_ex`
  (unified masked panchang, covering `panchang_for_date`),
  plus helper exports (`dhruv_elongation_at`, `dhruv_sidereal_sum_at`,
  `dhruv_tithi_at`, `dhruv_karana_at`, `dhruv_yoga_at`,
  `dhruv_vedic_day_sunrises`, `dhruv_vaar_from_sunrises`,
  `dhruv_hora_from_sunrises`, `dhruv_ghatika_from_sunrises`)
- Panchang range sweep: `dhruv_panchang_events` (handle-based, covering
  `panchang_events`) with per-kind `_count`/`_at` accessors, `_meta`, and
  `_free`
- Jyotish orchestrators: `dhruv_special_lagnas_for_date`,
  `dhruv_arudha_padas_for_date`, `dhruv_all_upagrahas_for_date`,
  `dhruv_graha_positions`, `dhruv_ashtakavarga_for_date`, `dhruv_core_bindus`,
  `dhruv_drishti`, `dhruv_graha_longitudes`, `dhruv_nakshatra_at`
- Amsha range ops: `dhruv_amsha_series` (handle-based, covering
  `amsha_series`) with `_point_count`/`_chart_count`/`_point_at`/`_chart_at`/
  `_free`, and `dhruv_amsha_lagna_events` (handle-based, covering
  `amsha_lagna_events`) with `_entry_count`/`_entry_info`/`_segment_count`/
  `_segment_at`/`_meta`/`_free`

## Amsha Parity Status

Amsha parity is tracked separately from the broad runtime-coverage count above.

Canonical C ABI amsha surface:

- `dhruv_amsha_longitude`
- `dhruv_amsha_rashi_info`
- `dhruv_amsha_longitudes`
- `dhruv_amsha_chart_for_date`
- `dhruv_amsha_series` (+ accessors/free)
- `dhruv_amsha_lagna_events` (+ accessors/meta/free)
- full-kundali amsha config/result fields

Current wrapper status:

| Surface | Status | Notes |
|---|---|---|
| `dhruv_rs` | complete | direct pure helpers, date-backed chart helpers, and full-kundali amsha config exposed |
| CLI | complete | `amsha`, `amsha-chart`, and `kundali` amsha selection/scope flags documented and implemented |
| Python | complete | direct amsha helpers plus full-kundali amsha selection/scope and optional section extraction |
| Go | complete | direct amsha helpers plus full-kundali amsha selection/scope and optional section extraction |
| Node | complete | direct amsha helpers plus full-kundali amsha selection/scope and optional section extraction |
| Elixir | complete | dedicated amsha scope plus full-kundali amsha selection/scope and richer result maps |

Reference:

- `docs/AMSHA_PARITY_CONTRACT.md`
