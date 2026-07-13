# Go Wrapper (`go-open`)

Open-source Go bindings for `ctara-dhruv-core`, implemented against the canonical C ABI (`dhruv_ffi_c`).

## Status

- ABI target: `DHRUV_API_VERSION=80`
- Binding strategy: `cgo` over `crates/dhruv_ffi_c/include/dhruv.h`
- Package: `ctara-dhruv-core/bindings/go-open/dhruv`
- Distribution model: tagged Go module plus validated C ABI release artifacts

## End-User Docs

Usage-first documentation for this wrapper lives in
[`../../docs/end_user/go/README.md`](../../docs/end_user/go/README.md).

## Prerequisites

- Go (1.24+)
- Rust toolchain (`cargo`)

## Install

```bash
go get ctara-dhruv-core/bindings/go-open/dhruv@v0.1.0
```

The Go wrapper remains a source-consumed module. It expects a matching
`dhruv_ffi_c` library at build/runtime.

## Build Or Download The C ABI

From repository root:

```bash
cargo build -p dhruv_ffi_c --release
```

This produces:

- Linux: `target/release/libdhruv_ffi_c.so`
- macOS: `target/release/libdhruv_ffi_c.dylib`
- Windows: `target/release/dhruv_ffi_c.dll`

Release tags also publish C ABI bundles on GitHub Releases for the main
platform matrix. Linux and macOS consumers can point `CGO_LDFLAGS` and runtime
library paths at those bundles. Windows Go support remains source-build first
in the initial rollout.

## Run Tests

From `bindings/go-open`:

```bash
GOCACHE=/tmp/go-build go test ./...
```

Kernel-dependent tests auto-skip when required files are absent.

## Quickstart

See `examples/basic/main.go`.

```bash
export DHRUV_SPK_PATH=/abs/path/to/de442s.bsp
export DHRUV_LSK_PATH=/abs/path/to/naif0012.tls
cd bindings/go-open
GOCACHE=/tmp/go-build go run ./examples/basic
```

## Library Loading

The wrapper links against `target/release` by default via cgo linker flags.

If runtime loading fails:

- Linux: add `target/release` to `LD_LIBRARY_PATH`
- macOS: add `target/release` to `DYLD_LIBRARY_PATH`
- Windows: add `target/release` to `PATH`

## Coverage

Low-level coverage in `internal/cabi` maps all currently exported `dhruv_ffi_c`
symbols from `dhruv.h` (ABI v78).

Dasha periods returned through the Go wrapper now carry `EntityName`, the exact
canonical Sanskrit entity name alongside the numeric kind/index fields.

`DashaVariationConfig` and `DashaSelectionConfig` support level-0 cycle
repetition through `Cycles` (explicit whole-cycle count, 0 = system default)
and `MinSpanYears` (repeat whole cycles until level-0 coverage from birth
reaches at least that many years; the final cycle completes past the target,
0 disables). `Cycles` wins when both are set. These apply to nakshatra-based
and Yogini dasha systems only; other systems ignore them. The level-0 requests
(`DashaLevel0Request`, `DashaLevel0EntityRequest`) now accept a `Variation`
like the hierarchy/children requests. For a repeated entry, derive its cycle
as `(Order-1)/sequenceLen + 1`.

The public `dhruv` package includes wrappers for:

- engine/config/LSK/EOP lifecycle
- runtime SPK replacement and listing through `(*Engine).ReplaceSPKs` and
  `(*Engine).ListSPKs`
- unified ephemeris query requests with selectable JD-vs-UTC input and cartesian-vs-spherical output
- time conversion and nutation
- ayanamsha and lunar-node APIs
- riseset/bhava APIs
- unified search APIs (conjunction/grahan/motion/lunar phase/sankranti)
  with structured UTC on the high-level time-bearing result objects alongside JD
- grouped `gochar_events` return-chart and transit-aspect API with caller-named natal targets, including `GocharTransitRahu` and `GocharTransitKetu` alongside physical-body codes
- panchang and calendar date APIs
- range-sweep APIs: `(*Engine).AmshaSeries` (fixed-cadence slim varga
  charts), `(*Engine).PanchangEvents` (exact panchang segments for all ten
  elements, with an optional observer location for vaar/hora/ghatika and
  truncation/resume metadata), and
  `(*Engine).AmshaLagnaEvents` (exact varga-lagna rashi transitions)
- panchang/classifier/math helper APIs
- graha longitude and jyotish date APIs
- shadbala, vimsopaka, and avastha date APIs
- drishti, ashtakavarga, core bindus, and amsha APIs
- dasha hierarchy/snapshot APIs
- full-kundali summary and full-result APIs, including root sphutas and dasha hierarchies
- tara catalog and compute APIs
- low-level graha relationship/combustion/dignity helpers
- low-level tara propagation and correction primitives

## Panchang Selection

`(*Engine).PanchangComputeEx` selects elements through
`PanchangComputeRequest.IncludeMask` using the `PanchangInclude*` bit
constants (`Tithi`, `Karana`, `Yoga`, `Vaar`, `Hora`, `Ghatika`, `Nakshatra`,
`Masa`, `Ayana`, `Varsha`) and the groups `PanchangIncludeAllCore`,
`PanchangIncludeAllCalendar`, `PanchangIncludeAll`,
`PanchangIncludeLocationIndependent`, and `PanchangIncludeLocationDependent`.

Location is optional: set `PanchangComputeRequest.HasLocation` (plus
`Location`) only when the mask requests location-dependent elements (vaar,
hora, ghatika). Those bits without `HasLocation` fail with an
invalid-search-config error; location-independent masks need no location.

Repeated nearby calls can skip the expensive new-moon/sankranti searches by
feeding calendar values from a previous result back through the optional
request fields `KnownMasa`, `KnownAyana`, and `KnownVarsha` (`*MasaInfo`,
`*AyanaInfo`, `*VarshaInfo`; nil means absent). A known value is reused
verbatim only when its element is selected in `IncludeMask` and the requested
moment falls inside its `[Start, End)` window; stale or invalid values are
silently ignored and recomputed.

`FullKundaliConfig.PanchangIncludeMask` selects the embedded panchang section
of `FullKundaliForDate` with the same bits (0 omits the section, replacing the
former `IncludePanchang`/`IncludeCalendar` booleans). The result section
`FullKundaliResult.Panchang` is a `*PanchangOperationResult` with the same
per-element `*Valid` flags as the standalone call.

## Range Sweeps

Three engine methods sweep a UTC range instead of a single epoch:

- `(*Engine).AmshaSeries(eop, fromUTC, toUTC, stepMinutes, loc, sankrantiCfg,
  requests, includeGrahas)` returns `[]AmshaSeriesPoint` on the same grid as
  `GrahaPositionsSeriesForDate` (one point per `stepMinutes` starting at
  `fromUTC`, endpoints inclusive when on the grid). Each point carries one
  slim `AmshaSeriesChart` per `AmshaRequest{AmshaCode, VariationCode}` in
  request order (duplicates repeated); the varga lagna is always present and
  `Grahas`/`GrahasValid` are filled when `includeGrahas` is true. Points times
  unique requests must stay within `MaxAmshaSeriesCells` (100,000).
- `(*Engine).PanchangEvents(eop, fromUTC, toUTC, includeMask, location,
  risesetCfg, sankrantiCfg, maxEvents)` returns `PanchangEventsResult` with
  per-kind slices (`Tithis`, `Karanas`, `Yogas`, `Nakshatras`, `Vaars`,
  `Horas`, `Ghatikas`, `Masas`, `Ayanas`, `Varshas`). Any combination of
  `PanchangInclude*` element bits is allowed; the location-dependent bits
  (vaar, hora, ghatika) require a non-nil `location *GeoLocation` and fail
  with an invalid-search-config error otherwise. `risesetCfg *RiseSetConfig`
  is read only for those elements (nil selects the library defaults). Vaar
  segments are sunrise-to-sunrise Vedic days, hora/ghatika their 24/60
  subdivisions. Segments of a kind chain exactly (`End` == next `Start`),
  including across Vedic-day rolls; the first may start before `fromUTC` and
  the last may end after `toUTC`. `maxEvents` caps the total events across
  kinds (0 selects `MaxPanchangEvents`, 50,000).
- `(*Engine).AmshaLagnaEvents(eop, fromUTC, toUTC, loc, sankrantiCfg,
  requests, maxSegments)` returns `AmshaLagnaEventsResult` with one
  `AmshaLagnaEntry` per unique request (duplicates collapsed) holding exact
  `AmshaLagnaSegment` rashi transitions. `maxSegments` caps total segments
  (0 selects `MaxAmshaLagnaSegments`, 50,000).

The two event sweeps report truncation on the result: when `Truncated` is
true, call again with `fromUTC = *NextFromUTC` and drop resumed events whose
(kind, `Start`) you already collected.

## Time-Based Upagraha Config

The Go wrapper exposes configurable time-based upagrahas through:

- `TimeUpagrahaConfigDefault()`
- `(*Engine).AllUpagrahasForDateWithConfig(...)`
- `BindusConfig.UpagrahaConfig`
- `FullKundaliConfig.UpagrahaConfig`

Public value constants are:

- `UpagrahaPointStart`, `UpagrahaPointMiddle`, `UpagrahaPointEnd`
- `GulikaMaandiPlanetRahu`, `GulikaMaandiPlanetSaturn`

## Amsha Notes

The Go wrapper exposes the amsha surface through:

- `AmshaLongitude`
- `AmshaRashiInfo`
- `AmshaLongitudes`
- `AmshaVariations`
- `AmshaVariationsMany`
- `(*Engine).AmshaChartForDate`
- `FullKundaliConfig.AmshaSelection`
- `FullKundaliConfig.AmshaScope`

Standalone bala helpers take the same selection shape:

- `(*Engine).ShadbalaForDate(..., amshaSelection)`
- `(*Engine).VimsopakaForDate(..., amshaSelection)`
- `(*Engine).BalasForDate(..., amshaSelection)`
- `(*Engine).AvasthaForDate(..., amshaSelection)`

`AmshaChart` now carries optional scoped sections directly:

- `BhavaCusps`
- `ArudhaPadas`
- `Upagrahas`
- `Sphutas`
- `SpecialLagnas`
- `OuterPlanets`

Graha-position and longitude results keep their traditional `Grahas` /
`Longitudes` arrays at length 9 and expose Uranus, Neptune, and Pluto through
separate `OuterPlanets` fields. These outer planets are positional display
entities only and are not used by bala, avastha, dasha, drishti, or lordship
calculations.

Setting `GrahaPositionsConfig.IncludeEquatorial` adds geocentric equatorial
output: each entry carries `EquatorialValid`, `RightAscensionDeg` ([0, 360)),
`DeclinationDeg` ([-90, +90]), and `EclipticLatitudeDeg` (equinox of date,
nutation per the request's `useNutation` flag, geometric with no
light-time/aberration correction; lagna and Rahu/Ketu report ecliptic latitude
exactly 0). The `GrahaPositions` result also carries `EarthOrientationValid`,
`GmstDeg`, and `GastDeg` (Greenwich mean/apparent sidereal time in degrees).

`Engine.GrahaPositionsSeriesForDate(eop, fromUTC, toUTC, stepMinutes, ...)`
samples the same op at a fixed cadence (endpoints inclusive when they fall on
the grid, at most 10,000 points; stepMinutes must be >= 1 and toUTC after
fromUTC). Each `GrahaPositionsPoint` carries `Utc`, `JdUtc`, and a `Positions`
value with the identical single-epoch shape.

Grahan results also carry apparent equatorial coordinates at greatest
grahan: `ChandraGrahanResult.MoonRightAscensionDeg`/`MoonDeclinationDeg`
and `SuryaGrahanResult.SunRightAscensionDeg`/`SunDeclinationDeg`
(degrees, equinox of date, nutation applied).

`GrahanConfig` additionally exposes the surya field products
(`IncludeLocalGrid`/`LocalGridStepDeg`, `IncludeIsolines` with
`DurationIsolineFractions`/`MagnitudeIsolineLevels`, and
`IncludeCentralCorridor`); `SuryaGrahanResult` then carries `Centrality`,
`LocalGrid`, `Isolines`, and `CentralCorridor`, and
`GrahanConfigEffective` echoes the clamped/sanitized configuration for
cache identity.

For embedded amsha charts in `FullKundaliForDate`, the relevant root sections
must also be enabled in the full-kundali config, or the wrapper caller must use
a higher-level helper that promotes those dependencies. Returned
`FullKundaliResult.Amshas` now contains the resolved union of explicit
`AmshaSelection` and any internally required bala/avastha amshas. Numeric
variation codes are amsha-scoped; use `AmshaVariations` or
`AmshaVariationsMany` to discover valid codes and names for each amsha.
