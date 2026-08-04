# Node Wrapper (`node-open`)

Open-source Node.js bindings for `ctara-dhruv-core`, implemented against the canonical C ABI (`dhruv_ffi_c`).

## Status

- ABI target: `DHRUV_API_VERSION=88`
- Binding strategy: Native Node-API addon (`native/dhruv_node.cc`) over `crates/dhruv_ffi_c/include/dhruv.h`
- Package: `bindings/node-open`
- Primary distribution: npm package with bundled platform prebuilds from unified `vX.Y.Z` tags

## End-User Docs

Usage-first documentation for this wrapper lives in
[`../../docs/end_user/node/README.md`](../../docs/end_user/node/README.md).

## Install

Published installs:

```bash
npm install ctara-dhruv-node-open
```

The published npm tarball bundles native prebuilds for the required release
targets and loads them automatically at runtime.

## Prerequisites For Local Development

- Node.js 20+
- C++ compiler (Linux/macOS currently)
- Rust toolchain (`cargo`)

## Build

From `bindings/node-open`:

```bash
npm run build
```

This script builds `dhruv_ffi_c` in release mode, compiles `dhruv_node.node`, and copies the shared `dhruv_ffi_c` library next to the addon.

## Test

```bash
npm test
```

Integration tests skip gracefully when kernel files are absent.

## Quickstart

```js
const dhruv = require('./index');

dhruv.verifyAbi();

const engine = dhruv.Engine.create({
  spkPaths: ['/abs/path/to/de442s.bsp'],
  lskPath: '/abs/path/to/naif0012.tls',
  cacheCapacity: 64,
  strictValidation: false,
});

const result = engine.query({
  target: 301,
  observer: 399,
  frame: 1,
  epochTdbJd: 2451545.0,
});

console.log(result.state);
engine.close();
```

## Coverage

Public modules included in this wrapper:

- engine/config/LSK/EOP lifecycle
- runtime SPK replacement and listing through `engine.replaceSpks(...)` and
  `engine.listSpks()`
- time conversions, nutation, ayanamsha, and lunar-node APIs
- unified search APIs (conjunction/grahan/motion/lunar phase/sankranti/fixed longitude/gochar events), including Rahu/Ketu bodies, any-body sankranti, multi-angle conjunction sweeps, and optional sidereal echoes (see "Search Notes")
- panchang/date APIs (`compute_rise_set*`, `compute_all_events*`, `compute_bhavas*`, `lagna/mc/ramc`, `tithi`, `karana`, `yoga`, `nakshatra`, `vaar`, `hora`, `ghatika`, `masa`, `ayana`, `varsha`)
- unified panchang selection (`panchangComputeEx` with a `PANCHANG_INCLUDE`
  bitmask and optional `location`; the same mask drives
  `fullKundaliConfig.panchangIncludeMask`)
- range sweeps: `panchangEvents` (exact panchang element segments over a UTC
  range), `amshaSeries` (fixed-cadence slim varga charts), and
  `amshaLagnaEvents` (exact varga-lagna transitions), with the C ABI caps
  exported as `MAX_PANCHANG_EVENTS`, `MAX_AMSHA_SERIES_CELLS`, and
  `MAX_AMSHA_LAGNA_SEGMENTS`
- jyotish/rashi/nakshatra helpers (`grahaLongitudes`, longitude classifiers, special lagnas, arudha/upagraha date APIs)
- charakaraka date API (`charakarakaForDate`) with selectable schemes (`8`, `7-no-pitri`, `7-pk-merged-mk`, `mixed-parashara`)
- charakaraka ranking-change events (`charakarakaEvents` over a UTC range plus
  `nextCharakarakaEvent`/`prevCharakarakaEvent` point lookups), with the C ABI
  cap exported as `MAX_CHARAKARAKA_EVENTS` and trigger codes as
  `CHARAKARAKA_EVENT_TRIGGER`
- build identity helpers (`libraryVersion`, `buildGitHash`) alongside
  `apiVersion`
- extras/composable APIs (panchang intermediates, sphuta/special-lagna scalar helpers, ashtakavarga, drishti, graha positions, bindus, amsha)
- low-level graha relationship/combustion/dignity helpers in `extras`
- shadbala/vimsopaka/avastha and full-kundali summary
- dasha hierarchy and snapshot, with `entityName` on returned period objects for the exact canonical Sanskrit entity name
- dasha level-0 cycle repetition through `variationConfig.cycles` and `variationConfig.minSpanYears`
- tara catalog load/compute helpers plus low-level propagation/correction primitives

## Search Notes

The unified search APIs accept the lunar nodes as first-class bodies: pass
`10007` (Rahu) or `10008` (Ketu) as `body1Code`/`body2Code` in
`conjunctionSearch`, as `bodyCode` in `motionSearch`, and as `bodyCode` in
`sankrantiSearch`. The sankranti, conjunction, and stationary configs carry a
`nodeMode` field (`0` = mean node, `1` = true node; default `1`). Stationary
search of Rahu/Ketu requires the true node (`nodeMode: 1`); with the mean node
it fails with an invalid-query/config error. The optional `grahaLongitudes`/
`movingOsculatingApogeesForDate` config object accepts the same `nodeMode`
field for its Rahu/Ketu longitudes (default `1` = true node).

`sankrantiSearch` requests take an optional `bodyCode` (default `0` = the Sun,
the classical sankranti) to search rashi ingresses of any body, including
retrograde re-ingresses. Events keep `sunSiderealLongitudeDeg`/
`sunTropicalLongitudeDeg` as legacy aliases for the tracked body and add
`bodyCode`, `siderealLongitudeDeg`, `tropicalLongitudeDeg`, and `isRetrograde`
(`true` when the boundary was crossed in retrograde motion).

`fixedLongitudeSearch(engine, request, capacity?)` finds when a moving body
reaches a fixed sidereal longitude — the root-find behind transit-to-natal
timeline search. `request.queryMode` is `0` (next), `1` (prev), or `2`
(range); `targetLongitudeDeg` is required; optional `targetAnglesDeg`
(offsets added to the target mod 360, at most 16; absent = conjunction
only), `includeSpecialAngles` (the body's classical special aspects —
Mars 90/210, Jupiter 120/240, Saturn 60/270 — cast onto the target),
`bodyCode` (absent/`0` = Sun), and `config` (sankranti config object).
Events carry `matchedLongitudeDeg` (target + angle), `angleDeg`,
`siderealLongitudeDeg`, `tropicalLongitudeDeg`, and
`actualSeparationDeg`; a range crossing the ephemeris coverage edge
returns the events found up to the edge.

`conjunctionSearch` requests accept an optional `targetSeparationsDeg` array
(up to 16 angles) to sweep several separation angles in one range search;
omitted or empty, the single `config.targetSeparationDeg` is used. Each event
reports the matched angle as `targetSeparationDeg`.

`conjunctionSearch` and `motionSearch` requests accept an optional
`siderealConfig` (a sankranti config object, e.g. from
`sankrantiConfigDefault()`). When present, events carry `hasSidereal: true`
plus sidereal echoes: `body1SiderealLongitudeDeg`/`body2SiderealLongitudeDeg`
and `body1RashiIndex`/`body2RashiIndex` on conjunction events,
`siderealLongitudeDeg`/`rashiIndex` on stationary and max-speed events.

## Panchang Selection

`panchangComputeEx(engine, eop, lsk, request)` computes any subset of panchang
elements in one call. `request.includeMask` is a bitmask built from the
exported `PANCHANG_INCLUDE` object (`TITHI`, `KARANA`, `YOGA`, `VAAR`, `HORA`,
`GHATIKA`, `NAKSHATRA`, `MASA`, `AYANA`, `VARSHA`, plus the combinations
`ALL_CORE`, `ALL_CALENDAR`, `ALL`, `LOCATION_INDEPENDENT`, and
`LOCATION_DEPENDENT`).

`request.location` is optional. It is required only for the
location-dependent elements (`VAAR`, `HORA`, `GHATIKA`); requesting those bits
without a location fails with `STATUS.INVALID_SEARCH_CONFIG`. When
`includeMask` is omitted it defaults to `PANCHANG_INCLUDE.ALL` with a location
and `PANCHANG_INCLUDE.LOCATION_INDEPENDENT` without one. `riseSetConfig` and
`sankrantiConfig` default to the library defaults when omitted.

Repeated nearby calls can skip the expensive new-moon/sankranti searches by
feeding calendar values from a previous result back through the optional
request properties `knownMasa`, `knownAyana`, and `knownVarsha` (the same
shapes the result emits as `masa`, `ayana`, and `varsha`). A known value is
reused verbatim only when its element is selected in `includeMask` and the
requested moment falls inside its `[start, end)` window; stale or invalid
values are silently ignored and recomputed.

The result carries per-element `*Valid` flags (`tithiValid`, `vaarValid`,
`masaValid`, ...) alongside the element payloads.

`panchangEvents(engine, eop, fromUtc, toUtc, includeMask, sankrantiConfig,
maxEvents, location, riseSetConfig)` streams the exact element segments
overlapping the range (no sampling). `includeMask` may combine any
`PANCHANG_INCLUDE` element bits; the location-dependent bits (`VAAR`, `HORA`,
`GHATIKA`) additionally require `location` and fail with
`STATUS.INVALID_SEARCH_CONFIG` without one. `riseSetConfig` is read only for
those elements (`null` selects the library defaults); both optional
parameters are appended after `maxEvents` so existing positional callers
keep working. Vaar segments are sunrise-to-sunrise Vedic days, hora/ghatika
their 24/60 subdivisions. `maxEvents` of `0` selects the
`MAX_PANCHANG_EVENTS` ceiling. Segments of one kind chain exactly, including
across Vedic-day rolls; on truncation (`truncated: true`) resume from
`nextFromUtc` and deduplicate on `(kind, start)`. The result carries
`tithis`, `karanas`, `yogas`, `nakshatras`, `vaars`, `horas`, `ghatikas`,
`masas`, `ayanas`, and `varshas` arrays.

`fullKundaliConfig` selects its embedded panchang section with the same mask
through `panchangIncludeMask` (`0` omits the section; it replaces the former
`includePanchang`/`includeCalendar` booleans). The embedded
`fullKundaliForDate(...).panchang` result uses the same per-element `*Valid`
shape as `panchangComputeEx`.

## Charakaraka Events

`charakarakaEvents(engine, eop, fromUtc, toUtc, options)` finds every
chara-karaka ranking change in `[fromUtc, toUtc]`. `options` accepts `scheme`
(the `charakarakaForDate` codes or names, default `'eight'`),
`sankrantiConfig` (the sidereal longitude policy, including `nodeMode`;
library defaults when omitted), and `maxEvents` (`0` selects the
`MAX_CHARAKARAKA_EVENTS` ceiling). The result is `{ events, truncated,
nextFromUtc }`; on truncation resume from `nextFromUtc` and deduplicate on
the event time. Each event carries:

- `at` (UTC object) and `jdTdb`
- `trigger`/`triggerName`: `CHARAKARAKA_EVENT_TRIGGER` code plus
  `"degree_crossing"`, `"rashi_ingress"`, or `"scheme_mode_change"`
- `changedRoles`: the `CHARAKARAKA_ROLE` codes whose assigned graha changed
- `before`/`after`: full rankings in the `charakarakaForDate` result shape

`nextCharakarakaEvent(engine, eop, atUtc, options)` and
`prevCharakarakaEvent(engine, eop, atUtc, options)` return the first change
strictly after / last change strictly before `atUtc` (same `options` without
`maxEvents`), or `null` when none is found before the coverage edge.

## Time-Based Upagraha Config

The Node wrapper accepts an optional `upagrahaConfig` object in:

- `jyotish.allUpagrahasForDate(...)`
- `extras.timeUpagrahaJd(...)`
- `extras.timeUpagrahaJdUtc(...)`
- `extras.coreBindusForDate(...)`
- `shadbala.fullKundaliForDate(...)`

Object fields are:

- `gulikaPoint`, `maandiPoint`, `otherPoint`
- `gulikaPlanet`, `maandiPlanet`

Value mappings are numeric:

- points: `0=start`, `1=middle`, `2=end`
- planets: `0=rahu`, `1=saturn`

## Dasha Variation Config

Dasha request functions (`dashaHierarchy`, `dashaSnapshot`, `dashaLevel0`,
`dashaLevel0Entity`, `dashaChildren`, `dashaChildPeriod`, `dashaCompleteLevel`)
accept an optional `variationConfig` object with:

- `levelMethods`
- `yoginiScheme`
- `useAbhijit`
- `cycles`: explicit level-0 whole-cycle repetition count; `0` (default) keeps
  the system default. Takes precedence over `minSpanYears`.
- `minSpanYears`: repeat whole level-0 cycles until coverage from birth reaches
  at least this many years; the final cycle completes past the target. `0` or
  negative (default) disables it.

`cycles` and `minSpanYears` apply to nakshatra-based and Yogini dasha systems
only; other systems ignore them. For repeated cycles, each period's cycle can
be derived from its `order` as `cycle = floor((order - 1) / sequenceLength) + 1`.
The same two fields are also accepted on `fullKundaliConfig.dashaConfig`.

## Amsha Notes

The Node wrapper exposes the amsha family through:

- `amshaLongitude`
- `amshaRashiInfo`
- `amshaLongitudes`
- `amshaChartForDate`
- `amshaSeries`
- `amshaLagnaEvents`
- `amshaVariations`
- `amshaVariationsMany`
- `fullKundaliConfigDefault`
- `fullKundaliForDate`

`amshaSeries(engine, eop, fromUtc, toUtc, stepMinutes, location, amshaCodes,
variationCodes, includeGrahas, sankrantiConfig)` samples slim varga charts at
a fixed cadence (points x unique requests capped at
`MAX_AMSHA_SERIES_CELLS`). `amshaLagnaEvents(engine, eop, fromUtc, toUtc,
location, amshaCodes, variationCodes, maxSegments, sankrantiConfig)` streams
exact varga-lagna rashi transition segments per unique request (capped at
`MAX_AMSHA_LAGNA_SEGMENTS`; on truncation resume from `nextFromUtc`). For
both, `variationCodes` may be `null` to use each amsha's default variation.

`fullKundaliConfigDefault()` returns amsha config fields as:

- `amshaScope`
- `amshaSelection`

The standalone bala helpers accept the same `amshaSelection` object:

- `shadbalaForDate(..., amshaSelection)`
- `vimsopakaForDate(..., amshaSelection)`
- `balasForDate(..., amshaSelection)`
- `avasthaForDate(..., amshaSelection)`

Direct amsha charts and embedded `fullKundaliForDate(...).amshas` now include
the optional scoped arrays when requested and available:

- `bhavaCusps`
- `arudhaPadas`
- `upagrahas`
- `sphutas`
- `specialLagnas`
- `outerPlanets`

Graha-position and longitude outputs keep their traditional navagraha arrays at
length 9 and expose Uranus, Neptune, and Pluto separately as `outerPlanets`.
Outer planets are positional display entities only; they are not used in bala,
avastha, dasha, drishti, or lordship calculations.

Graha positions support optional equatorial output. Set
`includeEquatorial: true` on the graha positions config (standalone
`grahaPositionsForDate` or `fullKundaliConfig.grahaPositionsConfig`). Each
entry then reports `equatorialValid`, `rightAscensionDeg`, `declinationDeg`,
and `eclipticLatitudeDeg`: geocentric coordinates in degrees, equinox of date,
nutation applied per the request's `useNutation` flag, geometric (no
light-time or aberration). Lagna and Rahu/Ketu report ecliptic latitude
exactly `0`. The positions result also carries `earthOrientationValid` with
`gmstDeg` and `gastDeg` (Greenwich mean/apparent sidereal time in degrees).

`grahaPositionsSeriesForDate(engine, eop, fromUtc, toUtc, stepMinutes,
location, bhavaConfig, ayanamshaSystem, useNutation, config)` samples the
same op at a fixed cadence (endpoints inclusive when on the grid, at most
10,000 points) and returns an array of points, each carrying `utc`,
`jdUtc`, and a `positions` object of the single-epoch shape.

Grahan results also carry apparent equatorial coordinates at greatest
grahan: `moonRightAscensionDeg`/`moonDeclinationDeg` on chandra grahan
results and `sunRightAscensionDeg`/`sunDeclinationDeg` on surya grahan
results (degrees, equinox of date, nutation applied).

The grahan config additionally accepts the surya field products
(`includeLocalGrid`/`localGridStepDeg`, `includeIsolines` with
`durationIsolineFractions`/`magnitudeIsolineLevels`, and
`includeCentralCorridor`); surya results then carry `centrality`,
`localGrid`, `isolines` (rings tagged with `containsPole`), and
`centralCorridor.segments`. `includeContactFootprints`/
`includeUmbraFootprints` add `contactFootprints` and `umbraFootprints`;
sampled `footprints` entries carry `containsPole`.

Embedded `fullKundaliForDate(...).amshas` now returns the full resolved amsha
union used by the call, not only the explicitly requested subset. Numeric
`variationCode` values are interpreted per amsha; use `amshaVariations*` to
discover the valid codes and names for a given amsha.

## Library Loading

- Optional addon override: `DHRUV_NODE_ADDON_PATH=/abs/path/to/dhruv_node.node`
- Published packages load bundled prebuilds from `prebuilds/<platform>-<arch>/`.
- Local development builds still use `build/Release/`.

## Notes

- Windows build path is implemented in `scripts/build-addon.mjs` using MSVC `cl`.
- If `cl`/`node.lib` discovery differs in your environment, set `NODE_INCLUDE_DIR` and use the package scripts from a Developer Command Prompt.
