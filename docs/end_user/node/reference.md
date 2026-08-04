# Node.js Reference

This page summarizes the public Node wrapper exported from
`bindings/node-open/src/index.js`.

## Exported Module Families

- `engine`
- `time`
- `search`
- `panchang`
- `jyotish`
- `extras`
- `shadbala`
- `dasha`
- `tara`

## Exact Export Inventory

`engine.js` exports:

- `Config`
- `Engine`
- `EOP`
- `LSK`
- `apiVersion`
- `verifyAbi`
- `clearActiveConfig`
- `queryOnce`
- `cartesianToSpherical`
- `QUERY_OUTPUT`
- `QUERY_TIME`

`Engine` instances also expose `replaceSpks(spkPaths)` and `listSpks()` for
copy-on-write SPK replacement on long-lived handles. Replacement is
all-or-nothing and list output preserves active query order.

`time.js` exports:

- `utcToTdbJd`
  `utcToTdbJd` now accepts a request object with `utc` plus optional `timePolicy`, and returns `{ jdTdb, diagnostics }`.
- `jdTdbToUtc`
- `nutationIau2000b`
- `nutationIau2000bUtc`
- `approximateLocalNoonJd`
- `ayanamshaSystemCount`
- `referencePlaneDefault`
- `ayanamshaComputeEx`
- `lunarNodeCount`
- `lunarNodeDeg`
- `lunarNodeDegWithEngine`
- `lunarNodeDegUtc`
- `lunarNodeDegUtcWithEngine`
- `lunarNodeComputeEx`
- `riseSetConfigDefault`
- `bhavaConfigDefault`
- `sankrantiConfigDefault`

`search.js` exports:

- `conjunctionConfigDefault`
- `grahanConfigDefault`
- `stationaryConfigDefault`
- `gocharEventsConfigDefault`
- `conjunctionSearch`
- `grahanSearch`

Solar grahan requests may include `location` and path sampling config. Solar
results include obscuration, gamma, Besselian elements, greatest location,
materialized `path` and `footprints` arrays, and complete local circumstances.
The config additionally accepts `includeLocalGrid`/`localGridStepDeg`,
`includeIsolines` with `durationIsolineFractions`/`magnitudeIsolineLevels`,
and `includeCentralCorridor`; results then carry `centrality`, `localGrid`,
`isolines` (`visibilityBoundary`/`durationIsolines`/`magnitudeIsolines` with
`containsPole`-tagged rings), and `centralCorridor.segments`. With
`includeContactFootprints`/`includeUmbraFootprints`, results also carry
`contactFootprints` and `umbraFootprints`; every `footprints` entry carries
`containsPole`, and with `instantaneousMagnitudeLevels` both footprint
kinds carry `magnitudeRings`. See
`docs/end_user/solar_eclipse_visibility.md`.
- `motionSearch`
- `lunarPhaseSearch`
- `sankrantiSearch`
- `fixedLongitudeSearch`
- `gocharEvents`

For range searches (`queryMode: 2`), these functions auto-expand their
internal buffers until the full result set is returned. The optional third
argument is only the initial internal chunk size, not a public result limit.

High-level time-bearing search results expose structured UTC on the main
result objects. Conjunction, grahan, stationary, and max-speed results now
include UTC alongside JD/TDB; sankranti and lunar-phase results remain UTC-first.

The same request objects accept `atUtc` / `startUtc` / `endUtc` alongside
`atJdTdb` / `startJdTdb` / `endJdTdb`, so UTC input stays on the main search
functions instead of introducing `*UtcSearch` variants.

`fixedLongitudeSearch(engine, request, capacity?)` finds when a moving
body reaches a fixed sidereal longitude. The request takes `queryMode`
(0=next, 1=prev, 2=range), `targetLongitudeDeg`, optional
`targetAnglesDeg` (offsets added to the target mod 360; absent =
conjunction only; at most 16), optional `includeSpecialAngles` (the
body's classical special aspects — Mars 90/210, Jupiter 120/240, Saturn
60/270 — cast onto the target), optional `bodyCode` (absent/0 = Sun,
NAIF codes, 10007/10008 for Rahu/Ketu), optional `config` (sankranti
config shape), and the usual time inputs. Next/prev return
`{found, event}`; range returns `{events}` sorted by time then angle.
Events report the matched longitude (target + angle), sidereal +
tropical longitudes, and the root residual (`actualSeparationDeg`). A
range crossing the ephemeris coverage edge returns the events found up
to the edge.

`gocharEvents(engine, eop, request)` returns grouped `yearlyTajaka`,
`yearlyTithiPravesha`, `monthlyTajaka`, `monthlyTithiPravesha`, and
`transitEvents`, with caller-supplied natal target names preserved on emitted
transit aspect rows. `gocharEventsConfigDefault()` exposes the typed defaults
for Tajaka basis, yearly/monthly window sizes, transit window, search steps,
and optional embedded return charts. `request.transitBodyCodes` accepts
physical-body codes such as `499`, `599`, `699`, `799`, `899`, `999`, plus
`GOCHAR_TRANSIT_BODY.RAHU` and `GOCHAR_TRANSIT_BODY.KETU`.

`panchang.js` exports:

- `bhavaSystemCount`
- `computeRiseSet`
- `computeAllEvents`
- `computeRiseSetUtc`
- `computeAllEventsUtc`
- `computeBhavas`
- `computeBhavasUtc`
- `lagnaDeg`
- `mcDeg`
- `ramcDeg`
- `lagnaDegUtc`
- `mcDegUtc`
- `ramcDegUtc`
- `riseSetResultToUtc`
- `tithiForDate`
- `karanaForDate`
- `yogaForDate`
- `nakshatraForDate`
- `vaarForDate`
- `horaForDate`
- `ghatikaForDate`
- `masaForDate`
- `ayanaForDate`
- `varshaForDate`
- `panchangComputeEx`
- `panchangEvents`
- `PANCHANG_INCLUDE`
- `MAX_PANCHANG_EVENTS`

`panchangComputeEx(engine, eop, lsk, request)` computes any subset of panchang
elements in one call. `request.includeMask` is a bitmask built from
`PANCHANG_INCLUDE` (`TITHI`, `KARANA`, `YOGA`, `VAAR`, `HORA`, `GHATIKA`,
`NAKSHATRA`, `MASA`, `AYANA`, `VARSHA`, plus `ALL_CORE`, `ALL_CALENDAR`,
`ALL`, `LOCATION_INDEPENDENT`, `LOCATION_DEPENDENT`). `request.location` is
optional and only needed for the location-dependent elements (`VAAR`, `HORA`,
`GHATIKA`); requesting those bits without a location fails with
`STATUS.INVALID_SEARCH_CONFIG`. When `includeMask` is omitted it defaults to
`PANCHANG_INCLUDE.ALL` with a location and
`PANCHANG_INCLUDE.LOCATION_INDEPENDENT` without one. `riseSetConfig` and
`sankrantiConfig` fall back to library defaults when omitted. The optional
request properties `knownMasa`, `knownAyana`, and `knownVarsha` accept
caller-cached calendar values from a previous result (the same shapes the
result emits as `masa`, `ayana`, and `varsha`). A known value is reused
verbatim only when its element is selected in `includeMask` and the requested
moment falls inside its `[start, end)` window; stale or invalid values are
silently ignored and recomputed. Feeding these back lets repeated nearby
calls skip the expensive new-moon/sankranti searches. The result carries
per-element `*Valid` flags (`tithiValid`, `vaarValid`, `masaValid`, ...)
alongside the element payloads.

`panchangEvents(engine, eop, fromUtc, toUtc, includeMask, sankrantiConfig,
maxEvents, location, riseSetConfig)` streams the exact panchang element
segments overlapping `[fromUtc, toUtc]` without a sampling grid.
`includeMask` may combine any `PANCHANG_INCLUDE` element bits (`TITHI`,
`KARANA`, `YOGA`, `VAAR`, `HORA`, `GHATIKA`, `NAKSHATRA`, `MASA`, `AYANA`,
`VARSHA`; it defaults to `PANCHANG_INCLUDE.LOCATION_INDEPENDENT`). The
location-dependent bits (`VAAR`, `HORA`, `GHATIKA`, grouped as
`PANCHANG_INCLUDE.LOCATION_DEPENDENT`) additionally require `location`
(`{ latitudeDeg, longitudeDeg, altitudeM }`); requesting them with `location`
null fails with `STATUS.INVALID_SEARCH_CONFIG`, as do a zero mask or unknown
bits. `riseSetConfig` is read only for those elements; `null` selects the
library defaults. The optional `location` and `riseSetConfig` parameters are
appended after `maxEvents`, so existing positional callers keep working.
`maxEvents` caps the total segments across all requested kinds; `0` selects
the hard ceiling `MAX_PANCHANG_EVENTS` (50,000). The result is:

```js
{
  tithis: [{ tithiIndex, paksha, tithiInPaksha, start, end }, ...],
  karanas: [{ karanaIndex, karanaNameIndex, start, end }, ...],
  yogas: [{ yogaIndex, start, end }, ...],
  nakshatras: [{ nakshatraIndex, pada, start, end }, ...],
  vaars: [{ vaarIndex, start, end }, ...],
  horas: [{ horaIndex, horaPosition, start, end }, ...],
  ghatikas: [{ value, start, end }, ...],
  masas: [{ masaIndex, adhika, start, end }, ...],
  ayanas: [{ ayana, start, end }, ...],
  varshas: [{ samvatsaraIndex, order, start, end }, ...],
  truncated: false,
  nextFromUtc: null,
}
```

Arrays for kinds not present in `includeMask` are empty. Consecutive segments
of one kind chain exactly (`end` equals the next segment's `start`); the
first segment of each kind may start before `fromUtc` and the last may end
after `toUtc`. Vaar segments are sunrise-to-sunrise Vedic days and hora and
ghatika segments their 24 and 60 subdivisions; their chaining holds across
Vedic-day rolls too. When the cap is hit, `truncated` is `true` and
`nextFromUtc` carries the resume point: call `panchangEvents` again from
`nextFromUtc` and deduplicate merged segments on `(kind, start)` (resumed
sweeps re-solve boundaries, so match starts with a small tolerance).

`jyotish.js` exports:

- `grahaLongitudes`
  Accepts an optional config object with `kind`, `ayanamshaSystem`, `useNutation`, `precessionModel`, and `referencePlane`.
- `specialLagnasForDate`
- `arudhaPadasForDate`
- `allUpagrahasForDate`
- `charakarakaForDate`
- `CHARAKARAKA_SCHEME`
- `CHARAKARAKA_ROLE`
- `rashiCount`
- `nakshatraCount`
- `rashiFromLongitude`
- `nakshatraFromLongitude`
- `nakshatra28FromLongitude`
- `rashiFromTropical`
- `nakshatraFromTropical`
- `nakshatra28FromTropical`
- `rashiFromTropicalUtc`
- `nakshatraFromTropicalUtc`
- `nakshatra28FromTropicalUtc`
- `degToDms`
- `tithiFromElongation`
- `karanaFromElongation`
- `yogaFromSum`
- `samvatsaraFromYear`
- `rashiName`
- `nakshatraName`
- `nakshatra28Name`
- `masaName`
- `ayanaName`
- `samvatsaraName`
- `tithiName`
- `karanaName`
- `yogaName`
- `vaarName`
- `horaName`
- `grahaName`
- `yoginiName`
- `sphutaName`
- `specialLagnaName`
- `arudhaPadaName`
- `upagrahaName`
- `vaarFromJd`
- `masaFromRashiIndex`
- `ayanaFromSiderealLongitude`
- `nthRashiFrom`
- `rashiLord`
- `horaAt`

`extras.js` exports:

- panchang intermediates:
  - `elongationAt`
  - `siderealSumAt`
  - `vedicDaySunrises`
  - `bodyEclipticLonLat`
  - `tithiAt`
  - `karanaAt`
  - `yogaAt`
  - `vaarFromSunrises`
  - `horaFromSunrises`
  - `ghatikaFromSunrises`
  - `nakshatraAt`
  - `ghatikaFromElapsed`
  - `ghatikasSinceSunrise`
- sphuta and special-lagna helpers:
  - `allSphutas`
  - `bhriguBindu`
  - `pranaSphuta`
  - `dehaSphuta`
  - `mrityuSphuta`
  - `tithiSphuta`
  - `yogaSphuta`
  - `yogaSphutaNormalized`
  - `rahuTithiSphuta`
  - `kshetraSphuta`
  - `beejaSphuta`
  - `trisphuta`
  - `chatussphuta`
  - `panchasphuta`
  - `sookshmaTrisphuta`
  - `avayogaSphuta`
  - `kunda`
  - `bhavaLagna`
  - `horaLagna`
  - `ghatiLagna`
  - `vighatiLagna`
  - `varnadaLagna`
  - `sreeLagna`
  - `pranapadaLagna`
  - `induLagna`
  - `arudhaPada`
  - `sunBasedUpagrahas`
- time-based upagraha helpers:
  - `timeUpagrahaJd`
  - `timeUpagrahaJdUtc`
- ashtakavarga, drishti, and charts:
  - `calculateAshtakavarga`
  - `calculateBav`
  - `calculateAllBav`
  - `calculateSav`
  - `trikonaSodhana`
  - `ekadhipatyaSodhana`
  - `ashtakavargaForDate`
  - `grahaDrishti`
  - `grahaDrishtiMatrixForLongitudes`
  - `drishtiForDate`
  - `grahaPositionsForDate`
  - `coreBindusForDate`
  - `amshaLongitude`
  - `amshaRashiInfo`
  - `amshaLongitudes`
  - `amshaChartForDate`
  - `amshaSeries`
  - `amshaLagnaEvents`
  - `amshaPointCount` / `amshaPointName` / `amshaPointKey`
  - `amshaSanskritName(amshaCode)` — the library's display name for a D-number
    (`'Navamsha'` for `9`), or `null` for an unsupported code. Every amsha
    chart and series chart also carries it as `sanskritName`, so a consumer
    does not need its own D-number to display-name table.
  - `AMSHA_POINT_FAMILY`
  - `MAX_AMSHA_SERIES_CELLS`
  - `MAX_AMSHA_LAGNA_SEGMENTS`

  Every entry inside an amsha chart identifies itself: `name` is a stable
  snake_case key (`'sree_lagna'`, `'gulika'`, `'a1'`, `'bhava_3'`,
  `'surya'`), `displayName` is the readable form, and `family` / `pointIndex`
  address the point. Entries also carry `nakshatraIndex`, `pada`, and
  `rashiBhavaNumber` (whole-sign bhava from the varga lagna; a varga
  transform is not monotonic, so `bhavaCusps` are not ordered house
  boundaries and there is no cusp-based bhava inside a varga). All chart
  sections stay **arrays** in the canonical order — prefer reading `name`
  over the array index.
- graha relationship, combustion, dignity, and classification helpers:
  - `horaLord`
  - `masaLord`
  - `samvatsaraLord`
  - `exaltationDegree`
  - `debilitationDegree`
  - `moolatrikoneRange`
  - `combustionThreshold`
  - `isCombust`
  - `allCombustionStatus`
  - `naisargikaMaitri`
  - `tatkalikaMaitri`
  - `panchadhaMaitri`
  - `dignityInRashi`
  - `dignityInRashiWithPositions`
  - `nodeDignityInRashi`
  - `naturalBeneficMalefic`
  - `moonBeneficNature`
  - `grahaGender`

`shadbala.js` exports:

- `calculateBhavaBala`
- `shadbalaForDate`
- `bhavaBalaForDate`
- `vimsopakaForDate`
- `balasForDate`
- `avasthaForDate`
- `fullKundaliConfigDefault`
- `fullKundaliForDate`
- `fullKundaliSummaryForDate`

`jyotish.js` also exports `movingOsculatingApogeesForDate(engine, eop, utc,
grahas, config)`, where `grahas` is an array of graha indices. Supported
indices are 2..6 (`Mangal,Buddh,Guru,Shukra,Shani`).

`shadbalaForDate`, `vimsopakaForDate`, `balasForDate`, and `avasthaForDate`
accept an `amshaSelection` object aligned with `fullKundaliConfigDefault()`.
Embedded `fullKundaliForDate(...).amshas` returns the resolved amsha union used
internally by the call. Use `amshaVariations` and `amshaVariationsMany` to
discover valid per-amsha variation codes and names.

Avastha entries expose `deeptadi` as the primary compatibility index and
`deeptadiStates` / `deeptadiMask` as the full set of Deeptadi states that apply
to the graha. They also expose `lajjitadi`, `lajjitadiStates`, and
`lajjitadiMask`; `lajjitadi` is `null` when no Lajjitadi condition applies.

`dasha.js` exports:

- `DashaHierarchy`
- `dashaSelectionConfigDefault`
- `dashaVariationConfigDefault`
- `dashaHierarchy`
- `dashaSnapshot`
- `dashaLevel0`
- `dashaLevel0Entity`
- `dashaChildren`
- `dashaChildPeriod`
- `dashaCompleteLevel`

Node dasha calls use one request-driven surface per feature. The same functions
accept either:

- `birthUtc` plus `location` for engine-derived inputs
- `birthJd` plus `inputs` for precomputed raw dasha inputs

`dashaSnapshot` similarly accepts either `queryUtc` or `queryJd`.

All dasha request functions accept an optional `variationConfig` object
(`levelMethods`, `yoginiScheme`, `useAbhijit`, `cycles`, `minSpanYears`).
`cycles` sets an explicit level-0 whole-cycle repetition count (`0` = system
default) and wins over `minSpanYears`. `minSpanYears` repeats whole level-0
cycles until coverage from birth reaches at least that many years, with the
final cycle completing past the target (`0` or negative = disabled). Both
apply to nakshatra-based and Yogini systems only; other systems ignore them.
With repeated cycles, a period's cycle number is
`floor((order - 1) / sequenceLength) + 1`.

Returned dasha period objects include `entityName`, the exact canonical
Sanskrit entity name, plus `startUtc` / `endUtc` alongside `startJd` /
`endJd`. Dasha snapshots expose `queryUtc` alongside `queryJd`.

Chara-style dasha periods use dual lordship for Kumbha (`Shani`/`Rahu`) and
Vrischika (`Mangal`/`Ketu`). Rahu owns Kumbha and Ketu owns Vrischika for the
default sign-lord-based node dignity policy.

`tara.js` exports:

- `TaraCatalog`
- `propagatePosition`
- `applyAberration`
- `applyLightDeflection`
- `galacticAnticenterIcrs`

## Public Config Objects

Common config objects:

- rise-set config
- bhava config
- sankranti config
- search configs
- drishti config
- graha positions config
  supports nested `basicStatesConfig.includeBasicStates` and
  `basicStatesConfig.includeSensitivePointDistances`; graha entries then expose
  `basicStates` and `sensitivePointDistances`.
  Also supports `includeEquatorial`: each entry then exposes
  `equatorialValid`, `rightAscensionDeg`, `declinationDeg`, and
  `eclipticLatitudeDeg` (geocentric, degrees, equinox of date, nutation per
  the request's `useNutation` flag, geometric without light-time or
  aberration; lagna and Rahu/Ketu report ecliptic latitude exactly `0`), and
  the positions result exposes `earthOrientationValid`, `gmstDeg`, and
  `gastDeg` (Greenwich mean/apparent sidereal time in degrees)
- bindus config
- full-kundali config
  forwards the same nested graha-position settings and may return
  `bhavaCuspSensitivePointDistances` plus
  `rashiBhavaCuspSensitivePointDistances`.
  Selects its embedded panchang section with `panchangIncludeMask`, a
  `PANCHANG_INCLUDE` bitmask (`0` omits the section; it replaces the former
  `includePanchang`/`includeCalendar` booleans). The embedded
  `fullKundaliForDate(...).panchang` result uses the same per-element
  `*Valid` shape as `panchangComputeEx`.
- dasha selection and variation configs

- `grahaPositionsSeriesForDate(engine, eop, fromUtc, toUtc, stepMinutes,
  location, bhavaConfig, ayanamshaSystem, useNutation, config)` —
  fixed-cadence sampling of the same op (endpoints inclusive on the grid,
  max 10,000 points); returns an array of `{ utc, jdUtc, positions }`.

- `amshaSeries(engine, eop, fromUtc, toUtc, stepMinutes, location,
  amshaCodes, variationCodes = null, includeGrahas = true,
  sankrantiConfig = default)` — fixed-cadence slim varga charts: one point
  per `stepMinutes` starting at `fromUtc` (endpoints inclusive when on the
  grid). `amshaCodes` is a non-empty array of amsha codes; `variationCodes`
  is `null` (each amsha's default variation) or a parallel array of variation
  codes. Returns an array of points
  `{ utc, jdUtc, charts: [{ amshaCode, variationCode, lagna, grahas }] }`
  with one chart per request, in request order (duplicates repeated).
  `lagna` and each `grahas` element use the amsha-entry shape
  (`siderealLongitude`, `rashiIndex`, `dmsDegrees`, `dmsMinutes`,
  `dmsSeconds`, `degreesInRashi`); `grahas` is `null` unless `includeGrahas`
  is `true`. Rejects `stepMinutes === 0`, reversed ranges, empty or invalid
  request lists, and grids whose points x unique requests exceed
  `MAX_AMSHA_SERIES_CELLS` (100,000). Narrow the range, increase the step,
  or split the request list to stay under the cap.

- `amshaLagnaEvents(engine, eop, fromUtc, toUtc, location, amshaCodes,
  variationCodes = null, maxSegments = 0, sankrantiConfig = default)` —
  exact varga-lagna rashi segments overlapping `[fromUtc, toUtc]` (exact
  transition boundaries, no sampling grid). One entry per unique
  `(amshaCode, variationCode)` request (duplicates collapsed), in request
  order. `maxSegments` caps the total segments across all amshas; `0`
  selects the hard ceiling `MAX_AMSHA_LAGNA_SEGMENTS` (50,000). Returns:

  ```js
  {
    entries: [{ amshaCode, variationCode,
                segments: [{ rashiIndex, start, end }, ...] }],
    truncated: false,
    nextFromUtc: null,
  }
  ```

  Per entry, segments chain exactly (`end` equals the next segment's
  `start`); the first segment starts at `fromUtc` and the last ends at the
  first transition at or after `toUtc`. When truncated, resume from
  `nextFromUtc` and deduplicate merged segments on their `start`.

- `charakarakaEvents(engine, eop, fromUtc, toUtc, options)` — the exact
  moments the chara-karaka ranking changes over the range. `options`
  takes `scheme` (same names/codes as `charakarakaForDate`),
  `sankrantiConfig` (rankings are sidereal and honor `nodeMode`, the same
  longitude path as `charakarakaForDate`), and `maxEvents` (`0` = the
  `MAX_CHARAKARAKA_EVENTS` = 50,000 ceiling). Returns
  `{ events, truncated, nextFromUtc }`; each event is
  `{ at, jdTdb, trigger, triggerName, changedRoles, before, after }` where
  `triggerName` is `'degree_crossing'` (pairwise crossings including
  Rahu's reversed-count sum condition `d_Rahu + d_other = 30`),
  `'rashi_ingress'`, or `'scheme_mode_change'` (the MixedParashara 8↔7
  flip — compare `before.usedEightKarakas`/`after.usedEightKarakas`), and
  `before`/`after` are in the `charakarakaForDate` result shape (entry
  order is the documented ranking contract: effective degree desc, then
  raw degrees-in-rashi desc, then graha index asc). Only actual ranking
  changes are emitted; on truncation the seam event is re-found — dedupe
  on `at`. Companions `nextCharakarakaEvent(engine, eop, atUtc, options)`
  / `prevCharakarakaEvent(...)` return the single neighboring change (or
  `null` at the ephemeris coverage edge).

- `libraryVersion()` / `buildGitHash()` (from `engine.js`) — build
  identity strings for provenance (`buildGitHash()` is `'unknown'`
  outside a git checkout).

Grahan results also carry apparent equatorial coordinates at greatest
grahan: `moonRightAscensionDeg`/`moonDeclinationDeg` on chandra grahan
results and `sunRightAscensionDeg`/`sunDeclinationDeg` on surya grahan
results (degrees, equinox of date, nutation applied).

`fullKundaliConfig.dashaConfig` supports:

- `systems`
- `maxLevels`
- `maxLevel`
- `cycles` (level-0 whole-cycle repetition count, `0` = system default)
- `minSpanYears` (repeat level-0 cycles to cover at least N years from birth)
- `snapshotUtc`

Time-based upagraha config object:

- `gulikaPoint`
- `maandiPoint`
- `otherPoint`
- `gulikaPlanet`
- `maandiPlanet`

Value mapping:

- points: `0=start`, `1=middle`, `2=end`
- planets: `0=rahu`, `1=saturn`

Other public enum objects:

- `CHARAKARAKA_SCHEME`
- `CHARAKARAKA_ROLE`

`bindusConfig` and `fullKundaliConfig` both accept nested upagraha config.

For build/runtime notes, see [`bindings/node-open/README.md`](../../../bindings/node-open/README.md).

## Rashi-Bhava Bhava Config

`bhavaConfigDefault()` includes `useRashiBhavaForBalaAvastha`, `includeRashiBhavaResults`, and `includeSpecialBhavaBalaRules`, all defaulting to `true`. `includeSpecialBhavaBalaRules=false` keeps Bhava Bala occupation/rising fields visible but excludes them from totals. It also includes `includeNodeAspectsForDrikBala`, defaulting to `false`, which controls whether Rahu/Ketu incoming aspects contribute to Shadbala Drik Bala and Bhava Bala Drishti Bala. `divideGuruBuddhDrishtiBy4ForDrikBala` defaults to `true`; set it to `false` to add Guru/Buddh incoming aspects at full signed strength instead of through the divided Drik Bala balance. `chandraBeneficRule` defaults to `CHANDRA_BENEFIC_RULE.BRIGHTNESS_72`; set it to `CHANDRA_BENEFIC_RULE.WAXING_180` for the 0..=180-degree waxing arc rule. `sayanadiGhatikaRounding` defaults to `0` for floor; set it to `1` for ceil. Existing fields keep configured bhava-system meaning; sibling fields such as `rashiBhavaCusps`, `rashiBhavaNumber`, and `grahaToRashiBhava` expose the rashi-bhava/equal-house basis.
