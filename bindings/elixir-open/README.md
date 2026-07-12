# Elixir Wrapper (`elixir-open`)

Open-source Elixir bindings for `ctara-dhruv-core`, implemented as a Rustler
NIF that calls the in-repo Rust crates directly.

## Status

- OTP app: `:ctara_dhruv`
- Binding strategy: source-built Rustler NIF (`native/dhruv_elixir_nif`)
- Package root: `bindings/elixir-open`
- Build mode: Hex package with source-built NIFs, no precompiled NIFs yet

## End-User Docs

Usage-first documentation for this wrapper lives in
[`../../docs/end_user/elixir/README.md`](../../docs/end_user/elixir/README.md).

## Install

Published installs use Hex:

```elixir
{:ctara_dhruv, "~> 0.1.0"}
```

The package is published from unified `vX.Y.Z` tags, but the Rustler NIF is
still compiled from source during `mix deps.compile`.

## Prerequisites

- Elixir 1.19+
- Erlang/OTP 28+
- Rust toolchain (`cargo`)

## Build

From `bindings/elixir-open`:

```bash
mix deps.get
mix test
```

Rustler compiles the NIF automatically during Mix compilation.

## Test

```bash
mix test
```

The ExUnit suite runs wrapper smoke coverage across the native families. Tests
that require SPK/LSK/EOP/tara data skip gracefully when those files are absent.

## Benchmark

```bash
mix run bench/all_functions.exs
```

Optional environment knobs:

- `DHRUV_BENCH_ITERATIONS=3`
- `DHRUV_BENCH_WARMUP=1`
- `DHRUV_BENCH_FILTER='Jyotish|Dasha'`

## Quickstart

```elixir
alias CtaraDhruv.{Engine, Ephemeris, Time}

{:ok, engine} =
  Engine.new(%{
    spk_paths: ["/abs/path/to/de442s.bsp"],
    lsk_path: "/abs/path/to/naif0012.tls",
    cache_capacity: 64,
    strict_validation: false
  })

{:ok, state} =
  Ephemeris.query(engine, %{
    target: :mars,
    observer: :solar_system_barycenter,
    frame: :eclip_j2000,
    epoch_tdb_jd: 2_451_545.0
  })

{:ok, time_result} =
  Time.utc_to_jd_tdb(engine, %{
    utc: %{year: 2024, month: 1, day: 1, hour: 12, minute: 0, second: 0.0},
    time_policy: %{mode: :hybrid_delta_t}
  })

IO.inspect(state)
IO.inspect(time_result.jd_tdb)
IO.inspect(time_result.diagnostics)

:ok = Engine.close(engine)
```

## Sidereal Chart Output

## Runtime SPK Replacement

Long-lived `CtaraDhruv.Engine` handles expose:

- `CtaraDhruv.Engine.replace_spks(engine, spk_paths)`
- `CtaraDhruv.Engine.list_spks(engine)`

Replacement swaps the full active SPK set atomically. Shared kernels are reused
when canonical path, file size, and modified time match.

The search surface now includes `CtaraDhruv.Search.gochar_events/2` for grouped
Tajaka, Tithi Pravesha, and named transit-aspect windows around a query time.
`transit_bodies` now accepts Rahu, Ketu, and the outer planets on the same
surface as the classical body names/codes.

The direct Vedic bhava surface is tropical unless you provide a
`sankranti_config`. The Elixir wrapper now exposes convenience arities for that
explicitly:

```elixir
alias CtaraDhruv.{Jyotish, Vedic}

location = %{latitude_deg: 28.6139, longitude_deg: 77.2090, altitude_m: 0.0}
utc = %{year: 2015, month: 1, day: 15, hour: 6, minute: 0, second: 0.0}
sidereal = %{ayanamsha_system: :lahiri, use_nutation: false}

{:ok, lagna} = Vedic.lagna(engine, %{utc: utc, location: location}, sidereal)
{:ok, bhavas} = Vedic.bhavas(engine, %{utc: utc, location: location}, sidereal)

{:ok, chart} =
  Jyotish.full_kundali(
    engine,
    %{utc: utc, location: location},
    sidereal
  )
```

Notes:

- `Vedic.lagna/2`, `Vedic.mc/2`, and `Vedic.bhavas/2` stay tropical when
  `:sankranti_config` is omitted.
- `Jyotish.full_kundali/3` applies the supplied ayanamsha to the full chart,
  including returned `bhava_cusps`.
- `full_kundali` now includes `graha_positions.lagna` by default.

## Equatorial Graha Output

`Jyotish.graha_positions/2` (and the embedded `full_kundali`
`graha_positions` block) can also report geocentric equatorial coordinates.
Enable it via `graha_positions_config`:

```elixir
{:ok, positions} =
  Jyotish.graha_positions(engine, %{
    utc: utc,
    location: location,
    graha_positions_config: %{include_equatorial: true}
  })
```

Each entry then carries `equatorial_valid`, `right_ascension_deg`,
`declination_deg`, and `ecliptic_latitude_deg` — degrees, equinox of date,
nutation per the request's `use_nutation` flag, geometric (no light-time or
aberration). Lagna and Rahu/Ketu report ecliptic latitude exactly `0.0`. The
result additionally carries `earth_orientation_valid`, `gmst_deg`, and
`gast_deg` (Greenwich mean/apparent sidereal time in degrees), populated when
equatorial output is requested.

`CtaraDhruv.Jyotish.graha_positions_series/2` samples the same op at a
fixed cadence: pass `:from_utc`, `:to_utc`, and `:step_minutes` instead
of `:utc` (endpoints inclusive when on the grid, at most 10,000 points).
The result is `%{points: [%{utc: ..., jd_utc: ..., positions: ...}]}`
where each `positions` value has the single-epoch shape.

Grahan results also carry apparent equatorial coordinates at greatest
grahan: `moon_right_ascension_deg`/`moon_declination_deg` on chandra
grahan results and `sun_right_ascension_deg`/`sun_declination_deg` on
surya grahan results (degrees, equinox of date, nutation applied).

## Amsha Notes

The Elixir wrapper exposes amsha-related behavior through
`CtaraDhruv.Jyotish`.

Dedicated amsha requests:

```elixir
{:ok, result} =
  Jyotish.amsha(engine, %{
    utc: utc,
    location: location,
    amsha_requests: [%{code: 9}, %{code: 2, variation: 1}],
    amsha_scope: %{
      include_bhava_cusps: true,
      include_arudha_padas: true,
      include_upagrahas: true,
      include_sphutas: true,
      include_special_lagnas: true,
      include_outer_planets: true
    }
  })
```

Batch sweeps over a time window (e.g. birth-time rectification or long-range
precalculation) replace loops of per-moment calls:

```elixir
# Slim varga charts on a fixed grid; add include_grahas: true for the nine
# graha entries per chart. Cap: 100,000 cells (points x unique requests).
{:ok, series} =
  Jyotish.amsha_series(engine, %{
    from_utc: from_utc,
    to_utc: to_utc,
    step_minutes: 5,
    location: location,
    amsha_requests: [%{code: 1}, %{code: 9}]
  })
# => %{"points" => [%{"utc" => ..., "jd_utc" => ...,
#      "charts" => [%{"amsha" => ..., "variation_code" => ...,
#                     "lagna" => ..., "grahas" => [...] | nil}]}]}

# Exact varga-lagna rashi segments (no sampling grid). Optional
# :max_segments (0 = 50,000 ceiling); on truncation resume from
# "next_from_utc".
{:ok, events} =
  Jyotish.amsha_lagna_events(engine, %{
    from_utc: from_utc,
    to_utc: to_utc,
    location: location,
    amsha_requests: [%{code: 9}]
  })
# => %{"entries" => [%{"amsha" => ..., "variation_code" => ...,
#      "segments" => [%{"rashi" => ..., "rashi_index" => ...,
#                       "start" => ..., "end" => ...}]}],
#      "truncated" => false, "next_from_utc" => nil}
```

Variation discovery helpers live on `CtaraDhruv.Math`:

```elixir
{:ok, d2_catalog} = CtaraDhruv.Math.amsha_variations(%{amsha_code: 2})
{:ok, catalogs} = CtaraDhruv.Math.amsha_variations_many(%{amsha_codes: [2, 9]})
```

Engine-free batched varga mapping also lives on `CtaraDhruv.Math`: given
sidereal longitudes and amsha requests it returns, per longitude, one map per
request with `amsha_longitude`, `rashi`, `rashi_index`, `degrees_in_rashi`,
and `dms`:

```elixir
{:ok, %{"entries" => [[d1_info, d9_info] | _]}} =
  CtaraDhruv.Math.amsha_rashi_infos(%{
    longitudes: [123.5, 245.0],
    amsha_requests: [%{code: 1}, %{code: 9}]
  })
```

Embedded amsha configuration in `full_kundali`:

```elixir
{:ok, chart} =
  Jyotish.full_kundali(engine, %{
    utc: utc,
    location: location,
    full_kundali_config: %{
      include_amshas: true,
      amsha_selection: [%{code: 9}],
      amsha_scope: %{include_sphutas: true, include_special_lagnas: true}
    }
  })
```

Result maps may now include these optional amsha chart keys when requested and
available:

- `:bhava_cusps`
- `:arudha_padas`
- `:upagrahas`
- `:sphutas`
- `:special_lagnas`
- `:outer_planets`

Graha-position and longitude maps keep navagraha lists at length 9 and expose
Uranus, Neptune, and Pluto as sibling `:outer_planets` sections. Outer planets
are positional display entities only and do not participate in bala, avastha,
dasha, drishti, or lordship calculations.

Standalone bala request maps also accept `:amsha_selection` with the same
`[%{code: ..., variation: ...}]` shape used by `full_kundali`. Embedded
`full_kundali` `:amshas` results now include the full resolved amsha union used
internally by the call. Variation codes remain numeric on input, but each code
is now interpreted in the namespace of that request's amsha code.

## Dasha Cycle Options

Standalone dasha request maps accept an optional `:variation` map with
`:level_methods`, `:yogini_scheme`, `:use_abhijit`, `:cycles`, and
`:min_span_years`. `full_kundali_config[:dasha_config]` accepts the same
`:cycles` and `:min_span_years` keys alongside `:systems`, `:max_level`,
`:max_levels`, and `:snapshot_utc`.

- `:cycles` (integer >= 1): explicit level-0 whole-cycle repetition count.
  When omitted, the system default applies. Wins over `:min_span_years` when
  both are set.
- `:min_span_years` (positive number): repeat whole mahadasha cycles until
  level-0 coverage from birth reaches at least that many years; the final
  cycle completes past the target.

Both options apply to nakshatra-based and Yogini dasha systems only; other
systems ignore them. A period's cycle number is
`(order - 1) / sequence_len + 1`, where `order` is global across cycles.

## Coverage

Public modules included in this wrapper:

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

Each module returns `{:ok, value}` or
`{:error, %CtaraDhruv.Error{kind, message, details}}`. The only long-lived
wrapper-owned struct is `%CtaraDhruv.Engine{}`.

`CtaraDhruv.Time` now includes the intended helper subset
(`nutation_utc/2`, `approximate_local_noon/1`, `ayanamsha_system_count/0`,
`reference_plane_default/1`), `CtaraDhruv.Panchang` includes the composable
intermediate helpers, `CtaraDhruv.Math` covers the pure helper surface, and
`CtaraDhruv.Tara` exposes the low-level propagation/correction primitives in
addition to the main request/config compute API.

## Panchang Element Selection

`CtaraDhruv.Panchang.daily/2` accepts an optional `:include_mask` selecting
which panchang elements to compute: an integer `PANCHANG_INCLUDE_*` bitmask, a
single name, or a list of names OR-ed together. Accepted names
(case-insensitive) are the ten elements (`tithi`, `karana`, `yoga`,
`nakshatra`, `vaar`, `hora`, `ghatika`, `masa`, `ayana`, `varsha`) and the
groups `all`, `all_core`, `all_calendar`, `location_independent`,
`location_dependent`, plus `none`. When omitted, the core elements
(`all_core`) are computed, matching the historical `daily` behavior; calendar
elements (`masa`, `ayana`, `varsha`) are opt-in.
The result map carries `nil` for elements not selected.

`:location` is optional for `daily/2`; it is only required when a
location-dependent element (`vaar`, `hora`, `ghatika`) is selected.

`daily/2` also accepts optional `:known_masa`, `:known_ayana`, and
`:known_varsha` fields carrying a previously returned masa/ayana/varsha map
verbatim (same shape as the `daily`/`events` results: element name or index,
`adhika`/`order` where applicable, and `start`/`end` UTC maps). Calendar
elements are interval-valid (masa ~a month, ayana ~half a year, varsha ~a
year), so loops over nearby dates can feed the previous values back and skip
the expensive new-moon/sankranti searches. A known value is reused (echoed
verbatim in the result) only when its element is selected in the include mask
and the requested moment lies inside its `[start, end)` window; otherwise it
is silently ignored and recomputed, so stale values can never corrupt a
result. Unknown element names, however, are rejected with `invalid_request`.

```elixir
{:ok, day1} = CtaraDhruv.Panchang.daily(engine, %{utc: utc, include_mask: [:tithi, :masa, :ayana, :varsha]})

{:ok, day2} =
  CtaraDhruv.Panchang.daily(engine, %{
    utc: next_day,
    include_mask: [:tithi, :masa, :ayana, :varsha],
    known_masa: day1.masa,
    known_ayana: day1.ayana,
    known_varsha: day1.varsha
  })
```

`full_kundali_config` selects embedded panchang output with
`:panchang_include_mask`, accepting the same integer/name/list forms
(default `0` omits panchang). It replaces the former
`:include_panchang`/`:include_calendar` booleans.

`CtaraDhruv.Panchang.events/2` streams exact element boundaries over a range
instead of computing one moment at a time: pass `:from_utc` and `:to_utc`,
an optional `:include_mask` accepting any element names or bits (including
`"all"`), and an optional `:max_events` cap (`0` selects the 50,000
ceiling). The default mask is unchanged: when `:include_mask` is omitted,
the location-independent elements (`tithi`, `karana`, `yoga`, `nakshatra`,
`masa`, `ayana`, `varsha`) are selected. The location-dependent elements
(`vaar`, `hora`, `ghatika`) are now supported too; selecting any of them
requires the optional `:location` (an optional `:riseset_config` tunes the
underlying sunrise searches), otherwise the call fails with a
`:search_error` (`"location required for vaar/hora/ghatika"`). The result
carries one list per kind in the per-moment shapes (empty lists for
unselected kinds) plus `"truncated"` and `"next_from_utc"`; consecutive
segments of one kind chain exactly (`end == next start`), including vaar,
hora, and ghatika across Vedic-day rolls. On truncation resume from
`next_from_utc` and deduplicate on `{kind, start}`.

## Time-Based Upagraha Config

The Elixir wrapper accepts `:upagraha_config` maps for:

- `CtaraDhruv.Jyotish.upagrahas/2`
- `CtaraDhruv.Jyotish.bindus/2`
- `CtaraDhruv.Jyotish.full_kundali/2`

Supported keys are:

- `:gulika_point`, `:maandi_point`, `:other_point`
- `:gulika_planet`, `:maandi_planet`

Accepted values are strings or atoms matching:

- points: `start`, `middle`, `end`
- planets: `rahu`, `saturn`

## Notes

- The wrapper keeps the NIF boundary private in `CtaraDhruv.Native`.
- Most results are returned as atom-keyed maps and lists rather than large
  Elixir struct graphs.
- The default tara catalog is the embedded Rust catalog; loading a JSON catalog
  from disk is optional.
