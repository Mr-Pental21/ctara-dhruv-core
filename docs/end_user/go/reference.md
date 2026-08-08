# Go Reference

This page summarizes the public Go wrapper from the `dhruv` package, using
`bindings/go-open/dhruv/` as the source of truth.

## Primary Public Types

Lifecycle and handles:

- `Engine`
- `LSK`
- `EOP`
- `Config`
- `TaraCatalog`
- `SpkReplaceReport`
- `LoadedSPKInfo`

Core inputs and configs:

- `EngineConfig`
- `ConfigLoadOptions`
- `Query`
- `QueryRequest`
- `QueryResult`
- `UtcTime`
- `GeoLocation`
- `RiseSetConfig`
- `BhavaConfig`
- `SankrantiConfig`
- `ConjunctionConfig`
- `GrahanConfig`
- `StationaryConfig`
- `GrahaPositionsConfig`
  `IncludeOuterPlanets` defaults on through the high-level defaults. Returned
  `Grahas` stay the 9 navagrahas; `OuterPlanets` is `[Uranus, Neptune, Pluto]`.
  `BasicStatesConfig` controls optional `BasicStates` and
  `SensitivePointDistances` output on entries. `IncludeEquatorial` adds
  geocentric `RightAscensionDeg`/`DeclinationDeg`/`EclipticLatitudeDeg`
  (with `EquatorialValid`) per entry — equinox of date, nutation per the
  request's `useNutation` flag, geometric (no light-time/aberration); lagna
  and Rahu/Ketu report ecliptic latitude exactly 0 — plus
  `EarthOrientationValid`, `GmstDeg`, and `GastDeg` (Greenwich mean/apparent
  sidereal time in degrees) on the `GrahaPositions` result.
- `BindusConfig`
- `DrishtiConfig`
- `TimeUpagrahaConfig`
- `AmshaChartScope`
- `AmshaSelectionConfig`
- `FullKundaliConfig`
  forwards `GrahaPositionsConfig.BasicStatesConfig`, and full-kundali results
  may also include bhava-cusp sensitive-point distance arrays
- `DashaSelectionConfig`
- `DashaVariationConfig`
- `TaraConfig`

Query request constants:

- `QueryTimeJDTDB`
- `QueryTimeUTC`
- `QueryOutputCartesian`
- `QueryOutputSpherical`
- `QueryOutputBoth`

Upagraha config constants:

- `UpagrahaPointStart`
- `UpagrahaPointMiddle`
- `UpagrahaPointEnd`
- `GulikaMaandiPlanetRahu`
- `GulikaMaandiPlanetSaturn`

Panchang selection constants (bits for `PanchangComputeRequest.IncludeMask`
and `FullKundaliConfig.PanchangIncludeMask`):

- `PanchangIncludeTithi`, `PanchangIncludeKarana`, `PanchangIncludeYoga`,
  `PanchangIncludeVaar`, `PanchangIncludeHora`, `PanchangIncludeGhatika`,
  `PanchangIncludeNakshatra`, `PanchangIncludeMasa`, `PanchangIncludeAyana`,
  `PanchangIncludeVarsha`
- Groups: `PanchangIncludeAllCore` (tithi through nakshatra),
  `PanchangIncludeAllCalendar` (masa/ayana/varsha), `PanchangIncludeAll`,
  `PanchangIncludeLocationIndependent` (everything except vaar/hora/ghatika),
  and `PanchangIncludeLocationDependent` (vaar/hora/ghatika)

Range-sweep types and caps:

- `AmshaRequest`, `AmshaSeriesChart`, `AmshaSeriesPoint` (for
  `(*Engine).AmshaSeries`)
- `PanchangEventsResult` (for `(*Engine).PanchangEvents`)
- `AmshaLagnaSegment`, `AmshaLagnaEntry`, `AmshaLagnaEventsResult` (for
  `(*Engine).AmshaLagnaEvents`)
- `CharakarakaChangeEvent`, `CharakarakaEventsResult` (for
  `(*Engine).CharakarakaEvents`)
- Hard ceilings: `MaxAmshaSeriesCells` (100,000 points times unique
  requests), `MaxPanchangEvents` (50,000 events), `MaxAmshaLagnaSegments`
  (50,000 segments), `MaxCharakarakaEvents` (50,000 events). The event
  sweeps select their ceiling when the caller passes a cap of 0 and report
  overflow through `Truncated`/`NextFromUTC` instead of failing.

## Package-Level Function Inventory

Lifecycle and runtime:

- `APIVersion`
- `VerifyABI`
- `LoadConfig`
- `ConfigLoadOptionsDefault`
- `ClearActiveConfig`
- `NewEngine`
- `LoadLSK`
- `LoadEOP`
- `LoadTaraCatalog`
- `QueryOnce`
- `CartesianToSpherical`

Long-lived engines expose runtime SPK replacement through
`(*Engine).ReplaceSPKs(spkPaths)` and active-set introspection through
`(*Engine).ListSPKs()`. Replacement is all-or-nothing and preserves the old set
on failure.

Default-config helpers:

- `RiseSetConfigDefault`
- `BhavaConfigDefault`
- `SankrantiConfigDefault`
- `ConjunctionConfigDefault`
- `GrahanConfigDefault`
- `StationaryConfigDefault`
- `DashaSelectionConfigDefault`
- `DashaVariationConfigDefault`
- `FullKundaliConfigDefault`
- `TimeUpagrahaConfigDefault`

Go config loading uses the same main request shape as the C ABI:

- `LoadConfig(ConfigLoadOptions)`
- `ConfigLoadOptions.Path` is nullable for discovery mode
- `ConfigLoadOptions.DefaultsMode` selects recommended defaults vs none
- `ConfigLoadOptionsDefault()` returns discovery mode with recommended defaults

Time and ayanamsha:

- `UTCToTdbJD`
  `UTCToTdbJD` now takes `(*LSK, *EOP, UtcToTdbRequest)` and returns `UtcToTdbResult`, including typed diagnostics.
- `JdTdbToUTC`
- `NutationIau2000b`
- `NutationIau2000bUTC`
- `ApproximateLocalNoonJD`
- `AyanamshaSystemCount`
- `ReferencePlaneDefault`
- `AyanamshaComputeEx`
- `LunarNodeCount`
- `LunarNodeDeg`
- `LunarNodeDegUTC`
- `LunarNodeComputeEx`

Classifiers, names, and pure helpers:

- `DegToDms`
- `RashiFromLongitude`
- `NakshatraFromLongitude`
- `Nakshatra28FromLongitude`
- `RashiFromTropical`
- `NakshatraFromTropical`
- `Nakshatra28FromTropical`
- `RashiFromTropicalUTC`
- `NakshatraFromTropicalUTC`
- `Nakshatra28FromTropicalUTC`
- `RashiCount`
- `NakshatraCount`
- `RashiName`
- `NakshatraName`
- `Nakshatra28Name`
- `MasaName`
- `AyanaName`
- `SamvatsaraName`
- `TithiName`
- `KaranaName`
- `YogaName`
- `VaarName`
- `HoraName`
- `GrahaName`
- `YoginiName`
- `SphutaName`
- `SpecialLagnaName`
- `ArudhaPadaName`
- `UpagrahaName`
- `TithiFromElongation`
- `KaranaFromElongation`
- `YogaFromSum`
- `VaarFromJD`
- `MasaFromRashiIndex`
- `AyanaFromSiderealLongitude`
- `NthRashiFrom`
- `RashiLord`
- `HoraAt`
- `SamvatsaraFromYear`
- `RiseSetResultToUTC`
- `VaarFromSunrises`
- `HoraFromSunrises`
- `GhatikaFromSunrises`
- `GhatikaFromElapsed`
- `GhatikasSinceSunrise`
- `HoraLord`
- `MasaLord`
- `SamvatsaraLord`
- `ExaltationDegree`
- `DebilitationDegree`
- `MoolatrikoneRange`
- `CombustionThreshold`
- `IsCombust`
- `AllCombustionStatus`
- `NaisargikaMaitri`
- `TatkalikaMaitri`
- `PanchadhaMaitri`
- `DignityInRashi`
- `DignityInRashiWithPositions`
- `NodeDignityInRashi`
- `NaturalBeneficMalefic`
- `MoonBeneficNature`
- `GrahaGender`

Pure sphuta, special-lagna, and upagraha helpers:

- `AllSphutas`
- `BhriguBindu`
- `PranaSphuta`
- `DehaSphuta`
- `MrityuSphuta`
- `TithiSphuta`
- `YogaSphuta`
- `YogaSphutaNormalized`
- `RahuTithiSphuta`
- `KshetraSphuta`
- `BeejaSphuta`
- `Trisphuta`
- `Chatussphuta`
- `Panchasphuta`
- `SookshmaTrisphuta`
- `AvayogaSphuta`
- `Kunda`
- `BhavaLagna`
- `HoraLagna`
- `GhatiLagna`
- `VighatiLagna`
- `VarnadaLagna`
- `SreeLagna`
- `PranapadaLagna`
- `InduLagna`
- `ArudhaPada`
- `TimeUpagrahaJD`
- `TimeUpagrahaJDWithConfig`

Pure ashtakavarga and drishti helpers:

- `CalculateAshtakavarga`
- `CalculateBAV`
- `CalculateAllBAV`
- `CalculateSAV`
- `TrikonaSodhana`
- `EkadhipatyaSodhana`
- `GrahaDrishti`
- `GrahaDrishtiMatrixForLongitudes`

Amsha helpers:

- `AmshaLongitude`
- `AmshaRashiInfo`
- `AmshaLongitudes`

## Engine Method Inventory

Ephemeris and node helpers:

- `(*Engine).Query`
- `(*Engine).LunarNodeDegWithEngine`
- `(*Engine).LunarNodeDegUTCWithEngine`

Go uses one main query request surface. `QueryRequest` carries JD-vs-UTC input
and cartesian-vs-spherical output selection instead of separate `QueryUTC` or
`QueryUTCSpherical` entrypoints.

Go dasha period results expose structured `StartUTC` / `EndUTC` alongside
`StartJD` / `EndJD`. Dasha snapshots expose `QueryUTC` alongside `QueryJD`.

Chara-style dasha periods use dual lordship for Kumbha (`Shani`/`Rahu`) and
Vrischika (`Mangal`/`Ketu`). Rahu owns Kumbha and Ketu owns Vrischika for the
default sign-lord-based node dignity policy.

Go high-level search/event results follow the same rule: conjunction, grahan,
stationary, and max-speed results expose structured UTC alongside their
existing JD/TDB fields, while sankranti and lunar-phase results remain UTC-first.

The corresponding Go search request structs carry `AtUTC` / `StartUTC` /
`EndUTC` alongside `AtJdTdb` / `StartJdTdb` / `EndJdTdb`, with one shared
request surface per feature instead of separate UTC-specific methods.

Go range-search methods auto-expand their internal buffers until the full
result set is returned. The optional final argument is only the initial
internal chunk size, not a public truncation cap.

Panchang and vedic basics:

- `(*Engine).ComputeRiseSet`
- `(*Engine).ComputeAllEvents`
- `(*Engine).ComputeRiseSetUTC`
- `(*Engine).ComputeAllEventsUTC`
- `(*Engine).ComputeBhavas`
- `(*Engine).ComputeBhavasUTC`
- `(*Engine).LagnaDeg`
- `(*Engine).LagnaDegWithConfig`
- `(*Engine).MCDeg`
- `(*Engine).MCDegWithConfig`
- `(*Engine).RAMCDeg`
- `(*Engine).LagnaDegUTC`
- `(*Engine).LagnaDegUTCWithConfig`
- `(*Engine).MCDegUTC`
- `(*Engine).MCDegUTCWithConfig`
- `(*Engine).RAMCDegUTC`
- `(*Engine).TithiForDate`
- `(*Engine).KaranaForDate`
- `(*Engine).YogaForDate`
- `(*Engine).NakshatraForDate`
- `(*Engine).VaarForDate`
- `(*Engine).HoraForDate`
- `(*Engine).GhatikaForDate`
- `(*Engine).MasaForDate`
- `(*Engine).AyanaForDate`
- `(*Engine).VarshaForDate`
- `(*Engine).PanchangComputeEx`
  Computes the elements selected by `PanchangComputeRequest.IncludeMask`
  (`PanchangInclude*` bits) in one call; each result element carries its own
  `*Valid` flag. Location is optional: set `HasLocation` and `Location` only
  when the mask includes location-dependent elements
  (`PanchangIncludeLocationDependent`, i.e. vaar/hora/ghatika); requesting
  those bits without `HasLocation` fails with an invalid-search-config error.
  The optional request fields `KnownMasa`, `KnownAyana`, and `KnownVarsha`
  (`*MasaInfo`, `*AyanaInfo`, `*VarshaInfo`; nil means absent) accept
  caller-cached calendar values from a previous result. A known value is
  reused verbatim only when its element is selected in `IncludeMask` and the
  requested moment falls inside its `[Start, End)` window; stale or invalid
  values are silently ignored and recomputed. Feeding these back lets
  repeated nearby calls skip the expensive new-moon/sankranti searches.
- `(*Engine).PanchangEvents(eop, fromUTC, toUTC, includeMask, location, risesetCfg, sankrantiCfg, maxEvents)`
  Range sweep returning `PanchangEventsResult` with exact per-kind segment
  slices (`Tithis`, `Karanas`, `Yogas`, `Nakshatras`, `Vaars`, `Horas`,
  `Ghatikas`, `Masas`, `Ayanas`, `Varshas`) overlapping `[fromUTC, toUTC]`.
  The mask must be non-zero; any combination of `PanchangInclude*` element
  bits is allowed. The location-dependent bits (vaar/hora/ghatika,
  `PanchangIncludeLocationDependent`) require a non-nil
  `location *GeoLocation` and fail with an invalid-search-config error
  otherwise; `risesetCfg *RiseSetConfig` is read only for those elements
  (nil selects the library defaults). Vaar segments are sunrise-to-sunrise
  Vedic days, hora/ghatika their 24/60 subdivisions. Consecutive segments of
  one kind chain exactly (`End` == next `Start`), including across Vedic-day
  rolls; the first segment of each kind may start before `fromUTC` and the
  last may end after `toUTC`. `maxEvents` caps total events across all kinds
  (0 selects `MaxPanchangEvents` = 50,000). When `Truncated` is true,
  `NextFromUTC` (a `*UtcTime`, nil when not truncated) is the resume point:
  call again from `*NextFromUTC` and drop resumed events whose (kind,
  `Start`) was already collected.
- `(*Engine).ElongationAt`
- `(*Engine).SiderealSumAt`
- `(*Engine).VedicDaySunrises`
- `(*Engine).BodyEclipticLonLat`
- `(*Engine).TithiAt`
- `(*Engine).KaranaAt`
- `(*Engine).YogaAt`
- `(*Engine).NakshatraAt`

Jyotish and charts:

- `(*Engine).GrahaLongitudes`
  Uses `GrahaLongitudesConfig` with `GrahaLongitudeKindSidereal` or `GrahaLongitudeKindTropical`, plus optional `PrecessionModel*` and `ReferencePlane*` choices.
- `(*Engine).MovingOsculatingApogeesForDate`
  Returns moving heliocentric osculating apogees for graha indices 2..6
  (`Mangal,Buddh,Guru,Shukra,Shani`) with sidereal longitude, ayanamsha, and
  reference-plane longitude.
- `(*Engine).SpecialLagnasForDate`
- `(*Engine).ArudhaPadasForDate`
- `(*Engine).AllUpagrahasForDate`
- `(*Engine).AllUpagrahasForDateWithConfig`
- `(*Engine).CharakarakaForDate`
- `(*Engine).GrahaPositionsForDate`
  Returns outer planets in a sibling field without changing the 9-graha list.
  With `GrahaPositionsConfig.IncludeEquatorial`, entries also carry geocentric
  equatorial coordinates and the result carries Greenwich sidereal time.
- `(*Engine).CoreBindusForDate`
- `(*Engine).DrishtiForDate`
- `(*Engine).AshtakavargaForDate`
- `(*Engine).FullKundaliForDateSummary`
- `(*Engine).FullKundaliForDate`
- `(*Engine).TimeUpagrahaJDUTC`
- `(*Engine).TimeUpagrahaJDUTCWithConfig`

- `Engine.GrahaPositionsSeriesForDate(eop, fromUTC, toUTC, stepMinutes, ...)`
  samples the same op at a fixed cadence (endpoints inclusive when on the
  grid, at most 10,000 points; `stepMinutes >= 1`, `toUTC` after `fromUTC`).
  Each `GrahaPositionsPoint` carries `Utc`, `JdUtc`, and a `Positions`
  value with the identical single-epoch shape.

Grahan results also carry apparent equatorial coordinates at greatest
grahan: `ChandraGrahanResult.MoonRightAscensionDeg`/`MoonDeclinationDeg`
and `SuryaGrahanResult.SunRightAscensionDeg`/`SunDeclinationDeg`
(degrees, equinox of date, nutation applied).

`GrahanSearchRequest.Location` applies to both kinds. It adds local
circumstances to each result and changes nothing else; leave it nil and
`LocalValid` is false.

For lunar grahan it does not move the contact times — a lunar eclipse is seen
at the same instants everywhere it is above the horizon. `ChandraGrahanResult`
instead reports `LocalVisible`, `LocalMoonAltitudeDeg`/`LocalMoonAzimuthDeg`
at greatest eclipse, the per-contact `LocalP1AltitudeDeg`,
`LocalU1AltitudeDeg` .. `LocalU4AltitudeDeg` (nil on a penumbral eclipse,
matching the absent contacts), `LocalP4AltitudeDeg`, and the
moonrise/moonset-clipped `LocalVisibleStartJd`/`UTC`,
`LocalVisibleEndJd`/`UTC`, and `LocalVisibleDurationSeconds`.

`SuryaGrahanResult` adds the Sun-up-clipped `LocalFirstVisibleContactJd`/`UTC`,
`LocalLastVisibleContactJd`/`UTC`, and `LocalVisibleDurationSeconds`. Show
those as a location's eclipse start and end: `LocalC1`..`LocalC4` are pure
geometric contacts and can fall while the Sun is below the horizon. Both
families use the same horizon convention, an altitude above -0.833 degrees.

`GrahanConfig` controls path
sampling. `SuryaGrahanResult` includes obscuration, gamma, Besselian elements,
greatest location, `Path`, `Footprints`, and complete local circumstances.
`GrahanConfig` additionally exposes `IncludeLocalGrid`/`LocalGridStepDeg`,
`IncludeIsolines` with `DurationIsolineFractions`/`MagnitudeIsolineLevels`,
and `IncludeCentralCorridor`; results then carry `Centrality`, `LocalGrid`,
`Isolines`, and `CentralCorridor` (ring-set segments). With
`IncludeContactFootprints`/`IncludeUmbraFootprints`, results also carry
`ContactFootprints` and `UmbraFootprints`; every `SuryaGrahanFootprint`
carries `ContainsPole`, and with `InstantaneousMagnitudeLevels` both
footprint kinds carry `MagnitudeRings` (`SuryaMagnitudeRing`). `GrahanConfigEffective` echoes the
clamped/sanitized configuration for cache identity. See
`docs/end_user/solar_eclipse_visibility.md`.

Strength, dasha, amsha, and tara:

- `(*Engine).ShadbalaForDate`
- `(*Engine).BhavaBalaForDate`
- `(*Engine).VimsopakaForDate`
- `(*Engine).BalasForDate`
- `(*Engine).AvasthaForDate`
- `(*Engine).DashaHierarchy`
- `(*Engine).DashaSnapshot`
- `(*Engine).DashaLevel0`
- `(*Engine).DashaLevel0Entity`
- `(*Engine).DashaChildren`
- `(*Engine).DashaChildPeriod`
- `(*Engine).DashaCompleteLevel`
- `(*Engine).AmshaChartForDate`
  Amsha chart `Grahas` stay length 9; `OuterPlanets` carries transformed
  Uranus, Neptune, and Pluto entries when the scope enables them.
  Every `AmshaEntry` identifies itself: `Name` is a stable snake_case key
  (`"sree_lagna"`, `"gulika"`, `"a1"`, `"bhava_3"`, `"surya"`),
  `DisplayName` is the readable form, and `Family` / `PointIndex` address the
  point. Entries also carry `NakshatraIndex`, `Pada`, and `RashiBhavaNumber`
  (whole-sign bhava from the varga lagna; a varga transform is not monotonic,
  so `BhavaCusps` are not ordered house boundaries and there is no cusp-based
  bhava inside a varga). All chart sections stay slices in the canonical
  order — prefer reading `Name` over the slice index.
  `AmshaPointCount` / `AmshaPointName` / `AmshaPointKey` with the
  `AmshaPointFamily*` constants enumerate a family without a chart in hand.
  `AmshaSanskritName(amshaCode)` gives the library's display name for a
  D-number (`"Navamsha"` for `9`), or `""` for an unsupported code; every
  `AmshaChart` and `AmshaSeriesChart` also carries it as `SanskritName`, so a
  consumer does not need its own D-number to display-name table.
- `(*Engine).AmshaSeries(eop, fromUTC, toUTC, stepMinutes, loc, sankrantiCfg, requests, includeGrahas)`
  Fixed-cadence slim varga charts, returned as `[]AmshaSeriesPoint`. Grid
  semantics match `GrahaPositionsSeriesForDate`: one point per `stepMinutes`
  starting at `fromUTC`, endpoints inclusive when on the grid. Each point
  carries `Utc`, `JdUtc`, and one `AmshaSeriesChart` per
  `AmshaRequest{AmshaCode, VariationCode}` in request order (duplicates
  repeated; `VariationCode` 0 = that amsha's default). The varga `Lagna` is
  always computed; `Grahas` (with `GrahasValid`) are added when
  `includeGrahas` is true. The ayanamsha system and nutation flag come from
  `sankrantiCfg`. Rejects `stepMinutes == 0`, reversed ranges, empty or
  invalid request lists, and grids whose points times unique requests exceed
  `MaxAmshaSeriesCells` (100,000).
- `(*Engine).AmshaLagnaEvents(eop, fromUTC, toUTC, loc, sankrantiCfg, requests, maxSegments)`
  Exact varga-lagna rashi transitions over `[fromUTC, toUTC]`, returned as
  `AmshaLagnaEventsResult` with one `AmshaLagnaEntry` per unique request
  (duplicates collapsed), in request order. Each `AmshaLagnaSegment` carries
  `RashiIndex`, `Start`, and `End`; segments chain exactly, the first starts
  at `fromUTC`, and the last ends at the first transition at or after
  `toUTC`. `maxSegments` caps total segments across all amshas (0 selects
  `MaxAmshaLagnaSegments` = 50,000). When `Truncated` is true, resume from
  `*NextFromUTC` and drop resumed segments whose `Start` was already
  collected for the same entry.
- `(*Engine).CharakarakaEvents(eop, fromUTC, toUTC, sankrantiCfg, scheme, maxEvents)`
  The exact moments the chara-karaka ranking changes over `[fromUTC,
  toUTC]` for a scheme (0-3, same codes as `CharakarakaForDate`), returned
  as `CharakarakaEventsResult`. Each `CharakarakaChangeEvent` carries
  `UTC`, `JdTdb`, `Trigger`/`TriggerName` (`degree_crossing` — pairwise
  crossings including Rahu's reversed-count sum condition,
  `rashi_ingress`, or `scheme_mode_change` — the MixedParashara 8↔7 flip),
  `ChangedRoles` (role codes), and `Before`/`After` in the per-moment
  `CharakarakaResult` shape (entry order: effective degree desc, then raw
  degrees-in-rashi desc, then graha index asc — the documented contract).
  Rankings are sidereal per `sankrantiCfg` and honor its `NodeMode` on
  the same longitude path as `CharakarakaForDate`; only actual ranking
  changes are emitted. `maxEvents` 0 selects `MaxCharakarakaEvents`
  (50,000); when `Truncated`, resume from `*NextFromUTC` and deduplicate
  on the event time.
- `(*Engine).NextCharakarakaEvent(eop, atUTC, sankrantiCfg, scheme)` /
  `(*Engine).PrevCharakarakaEvent(...)` — the single neighboring ranking
  change (`nil, nil` at the ephemeris coverage edge).
- `LibraryVersion()` / `BuildGitHash()` — build identity strings for
  provenance (`BuildGitHash()` is `"unknown"` outside a git checkout).
- `(*TaraCatalog).Compute`
- `(*TaraCatalog).GalacticCenterEcliptic`
- `TaraPropagatePosition`
- `TaraApplyAberration`
- `TaraApplyLightDeflection`
- `TaraGalacticAnticenterICRS`

Go dasha period structs expose `EntityName` with the exact canonical Sanskrit
entity name.
Go dasha requests now accept either UTC/location birth context or precomputed
raw dasha inputs through the shared request structs.
All dasha requests, including `DashaLevel0Request` and
`DashaLevel0EntityRequest`, carry a `Variation` (`DashaVariationConfig`).

Search:

- `(*Engine).ConjunctionSearch`
- `(*Engine).GrahanSearch`
- `(*Engine).MotionSearch`
- `(*Engine).LunarPhaseSearch`
- `(*Engine).SankrantiSearch`
- `(*Engine).FixedLongitudeSearch`

`(*Engine).FixedLongitudeSearch(req FixedLongitudeRequest)` finds when a
moving body reaches a fixed sidereal longitude. `QueryMode` 0=next,
1=prev (both return the single event + found flag), 2=range (returns the
event slice, auto-paged). The request carries `BodyCode` (0 = Sun, NAIF
codes, 10007/10008 for Rahu/Ketu), `TargetLongitudeDeg`,
`TargetAnglesDeg` (offsets added to the target mod 360; empty =
conjunction only; at most 16), `IncludeSpecialAngles` (the body's
classical special aspects — Mars 90/210, Jupiter 120/240, Saturn 60/270 —
cast onto the target), and a `SankrantiConfig`. Events report the matched
longitude (target + angle), sidereal + tropical longitudes, and the root
residual. A range crossing the ephemeris coverage edge returns the
events found up to the edge.

## Config Notes

`TimeUpagrahaConfig` fields:

- `GulikaPoint`
- `MaandiPoint`
- `OtherPoint`
- `GulikaPlanet`
- `MaandiPlanet`

`BindusConfig` and `FullKundaliConfig` both carry `UpagrahaConfig`.

`FullKundaliConfig` also includes:

- root include flags
- `GrahaPositionsConfig`
- `BindusConfig`
- `DrishtiConfig`
- `AmshaScope`
- `AmshaSelection`
- `PanchangIncludeMask`
- `DashaConfig`

`FullKundaliConfig.PanchangIncludeMask` selects the embedded panchang section
with the `PanchangInclude*` bits (0 omits the section); it replaces the former
`IncludePanchang` / `IncludeCalendar` booleans. The embedded
`FullKundaliResult.Panchang` section is a `*PanchangOperationResult` with the
same per-element `*Valid` flags and payloads as the standalone
`PanchangComputeEx` result.

`ShadbalaForDate`, `VimsopakaForDate`, `BalasForDate`, and `AvasthaForDate`
accept `AmshaSelection`. Embedded `FullKundaliResult.Amshas` returns the
resolved amsha union used by the call.

Avastha entries expose `Deeptadi` as the primary compatibility index and
`DeeptadiStates` / `DeeptadiMask` as the full set of Deeptadi states that apply
to the graha. They also expose `Lajjitadi`, `LajjitadiValid`,
`LajjitadiStates`, and `LajjitadiMask`; `LajjitadiValid=false` means no
Lajjitadi condition applies.

`DashaSelectionConfig` supports per-system hierarchy depth through `MaxLevels`
and optional full-kundali snapshots through `SnapshotTime`, typically with
`TimeKind = DashaTimeUTC` plus `UTC`.

`DashaVariationConfig` and `DashaSelectionConfig` control level-0 cycle
repetition:

- `Cycles`: explicit whole-cycle repetition count (0 = system default).
  Wins over `MinSpanYears` when non-zero.
- `MinSpanYears`: repeat whole cycles until level-0 coverage from birth
  reaches at least this many years; the final cycle completes past the target
  (0 or negative disables).

Both apply to nakshatra-based and Yogini dasha systems only; other systems
ignore them. For any returned period, derive its cycle number as
`(Order-1)/sequenceLen + 1`.

Defaults preserved by `TimeUpagrahaConfigDefault()`:

- Gulika = Rahu period start
- Maandi = Rahu period end
- other time-based upagrahas = period start

For build/runtime notes, see [`bindings/go-open/README.md`](../../../bindings/go-open/README.md).

## Rashi-Bhava Bhava Config

`BhavaConfig` includes `UseRashiBhavaForBalaAvastha`, `IncludeRashiBhavaResults`, and `IncludeSpecialBhavaBalaRules`, all defaulting to `true`. `IncludeSpecialBhavaBalaRules=false` keeps Bhava Bala occupation/rising fields visible but excludes them from totals. It also includes `IncludeNodeAspectsForDrikBala`, defaulting to `false`, which controls whether Rahu/Ketu incoming aspects contribute to Shadbala Drik Bala and Bhava Bala Drishti Bala. `DivideGuruBuddhDrishtiBy4ForDrikBala` defaults to `true`; set it to `false` to add Guru/Buddh incoming aspects at full signed strength instead of through the divided Drik Bala balance. `ChandraBeneficRule` defaults to `ChandraBeneficRuleBrightness72`; set it to `ChandraBeneficRuleWaxing180` for the 0..=180-degree waxing arc rule. The same rule is used by Buddh's association-based nature in Shadbala Drik Bala and Bhava Bala Drishti Bala. `SayanadiGhatikaRounding` defaults to `0`/floor; set it to `1` for ceil. Existing bhava fields remain configured-system outputs; rashi-bhava sibling fields such as `RashiBhavaCusps`, `RashiBhavaNumber`, and `GrahaToRashiBhava` expose the equal-house/whole-sign companion basis.
