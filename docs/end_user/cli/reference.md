# CLI Reference

This page summarizes the public `dhruv` command surface from code in
`crates/dhruv_cli/src/main.rs`.

## Shared Flags And Config Groups

Common runtime inputs:

- `--date`
- `--bsp`
- `--lsk`
- `--eop`
- `--lat`
- `--lon`
- `--alt`
- `--ayanamsha`
- `--nutation`

Layered config and time-conversion controls:

- `--config`
- `--no-config`
- `--defaults-mode`
- `--time-policy`
- `--delta-t-model`
- `--future-delta-t-transition`
- `--future-transition-years`
- `--no-freeze-future-dut1`
- `--smh-future-family`
- `--stale-lsk-threshold-days`
- `--stale-eop-threshold-days`

Reusable public option groups:

- Upagraha configuration:
  - `--gulika-point`
  - `--maandi-point`
  - `--other-upagraha-point`
  - `--gulika-planet`
  - `--maandi-planet`
- Graha basic-state toggles:
  - `graha-positions --basic-states`
  - `graha-positions --sensitive-point-distances`
  - `kundali --include-basic-states`
  - `kundali --include-sensitive-point-distances`
- Kundali amsha scope:
  - `--include-amshas`
  - `--amsha`
  - `--amsha-include-bhava-cusps`
  - `--amsha-include-arudha-padas`
  - `--amsha-include-upagrahas`
  - `--amsha-include-sphutas`
  - `--amsha-include-special-lagnas`
  - `--amsha-no-outer-planets`
- Bhava/bala behavior:
  - `--use-rashi-bhava-for-bala-avastha`
  - `--use-configured-bhava-for-bala-avastha`
  - `--include-node-aspects-for-drik-bala`
  - `--exclude-node-aspects-for-drik-bala`
  - `--include-special-bhavabala-rules`
  - `--exclude-special-bhavabala-rules`
  - `--divide-guru-buddh-drishti-by-4-for-drik-bala`
  - `--add-full-guru-buddh-drishti-for-drik-bala`
  - `--chandra-benefic-rule brightness-72|waxing-180`
  - `--sayanadi-ghatika-rounding floor|ceil`
  - Node-aspect flags affect Shadbala Drik Bala and Bhava Bala Drishti Bala; standalone drishti output remains unchanged.
  - `--include-rashi-bhava-results`
  - `--no-rashi-bhava-results`

Shared value mappings worth knowing:

- upagraha points: `start`, `middle`, `end`
- Gulika/Maandi planets: `rahu`, `saturn`
- charakaraka schemes: `eight`, `seven-no-pitri`, `seven-pk-merged-mk`, `mixed-parashara`
- `--defaults-mode`: `recommended`, `none`
- `--time-policy`: `strict-lsk`, `hybrid-deltat`

Outer grahas:

- `graha-positions`, `graha-longitudes`, `kundali`, and amsha chart output show
  Uranus, Neptune, and Pluto in sibling “Outer Grahas” sections by default.
- Existing navagraha lists remain length 9. Outer grahas are positional display
  entities only and are not used in bala, avastha, dasha, drishti, lordship, or
  other traditional navagraha calculations.
- Use `--no-outer` on graha-position/longitude and `kundali` commands, and
  `--no-outer-planets` / `--amsha-no-outer-planets` on amsha and `kundali`
  commands to suppress those sibling sections.

## Command Families

Configuration and classifiers:

- `config-show-effective`
- `rashi`
- `nakshatra`
- `rashi-tropical`
- `nakshatra-tropical`
- `dms`
- `tithi-from-elongation`
- `karana-from-elongation`
- `yoga-from-sum`
- `vaar-from-jd`
- `masa-from-rashi`
- `ayana-from-lon`
- `samvatsara-compute`
- `nth-rashi-from`
- `rashi-lord`
- `normalize360`

Ephemeris and core astronomy:

- `position`
- `sidereal-longitude`
- `graha-longitudes`
- `ayanamsha-compute`
- `nutation-compute`
- `lunar-node`
- `body-lon-lat`

Rise/set, lagna, and bhava:

- `sunrise`
- `bhavas`
- `lagna-compute`
- `vedic-day-sunrises`

Panchang:

- `panchang`
- `panchang-events`
- `tithi`
- `karana`
- `yoga`
- `moon-nakshatra`
- `vaar`
- `hora`
- `ghatika`
- `masa`
- `ayana`
- `varsha`
- `tithi-at`
- `karana-at`
- `yoga-at`
- `nakshatra-at`
- `elongation-at`
- `sidereal-sum-at`

`panchang` computes only the elements you select with `--elements`, a
comma-separated list of element names (`tithi`, `karana`, `yoga`, `vaar`,
`hora`, `ghatika`, `nakshatra`, `masa`, `ayana`, `varsha`) or group names
(`all`, `all_core`, `all_calendar`, `location_independent`,
`location_dependent`). Omitting `--elements` selects all elements.
`--lat`/`--lon` are required only when the selection includes a
location-dependent element (vaar, hora, ghatika):

```bash
# No location needed: tithi/karana/yoga/nakshatra/masa/ayana/varsha only
dhruv panchang --date 2026-04-17T13:25:39Z --elements location_independent

# Location required for vaar/hora/ghatika
dhruv panchang --date 2026-04-17T13:25:39Z --lat 28.6 --lon 77.2 --elements vaar,hora,ghatika
```

`panchang-events` streams every element boundary in a UTC range in one call,
instead of one per-moment `panchang` call per day. All ten elements are
supported; `--lat`/`--lon` are required only when the selection includes a
location-dependent element (`vaar`, `hora`, `ghatika`) — the default
selection (`location_independent`) needs no location flags. Segment
boundaries are exact and consecutive segments of one kind chain exactly
(including across sunrise rolls for vaar/hora/ghatika); the first segment
may start before `--start` and the last may end after `--end`.
`--max-events` caps the total segments across all elements (0 = the 50,000
library ceiling); when hit, the output is marked truncated with a resume
time:

```bash
dhruv panchang-events --start 2026-04-01T00:00:00Z --end 2026-05-01T00:00:00Z \
  --elements tithi,masa

# Ghatika lanes for a day need a location:
dhruv panchang-events --start 2026-04-01T00:00:00Z --end 2026-04-02T00:00:00Z \
  --elements ghatika --lat 28.6139 --lon 77.2090
```

Jyotish and chart building:

- `sphutas`
- `special-lagnas`
- `arudha-padas`
- `upagrahas`
- `graha-positions`
- `core-bindus`
- `drishti`
- `ashtakavarga`
- `charakaraka`
- `osculating-apogee`
- `shadbala`
- `bhavabala`
- `balas`
- `vimsopaka`
- `avastha`
- `kundali`
- `gochar-events`

For `shadbala`, `vimsopaka`, `balas`, `avastha`, and `kundali`, use
`--amsha D<n>[:variation]` to override per-amsha variation selection. When
`kundali --include-amshas` is enabled, returned amsha charts include the full
resolved union of explicit selections and internally required bala/avastha
amshas.

`kundali --panchang-elements` selects panchang elements for the kundali's
panchang section using the same element/group names as `panchang --elements`
(replacing the former `--include-panchang`/`--include-calendar` flags).
`none` or omitting the flag omits the section. `kundali --all` selects all
panchang elements unless an explicit non-empty `--panchang-elements`
selection is given, which wins:

```bash
dhruv kundali --date 2026-04-17T13:25:39Z --lat 28.6 --lon 77.2 \
  --panchang-elements tithi,nakshatra,masa
```

`avastha` and `kundali --include-avastha` print every Deeptadi and Lajjitadi
state that applies to each graha, comma-separated. Lajjitadi prints `None` when
no condition applies. Sayanadi birth ghatikas use floor rounding by default;
pass `--sayanadi-ghatika-rounding ceil` to count the current partial ghatika.

`osculating-apogee` returns moving heliocentric osculating apogee longitudes for
`Mangal,Buddh,Guru,Shukra,Shani`:

```bash
dhruv osculating-apogee --date 2026-04-17T13:25:39Z \
  --graha Mangal,Buddh,Guru --ayanamsha 0 --nutation
```

The output includes sidereal apogee longitude, ayanamsha, and reference-plane
longitude. Surya, Chandra, Rahu, and Ketu are invalid for this endpoint.

Amsha:

- `amsha`
- `amsha-chart`
- `amsha-series`
- `amsha-lagna-events`

`amsha-series` samples slim varga charts (varga lagna always, the nine grahas
with `--include-grahas`) at a fixed cadence from `--date` to `--to-date`,
using the same grid semantics as the `graha-positions` series mode. The grid
is capped at 100,000 cells (points x unique amsha requests):

```bash
dhruv amsha-series --date 2026-04-17T00:00:00Z --to-date 2026-04-18T00:00:00Z \
  --step-minutes 60 --lat 28.6 --lon 77.2 --amsha D9,D10 --include-grahas
```

`amsha-lagna-events` returns the exact times the varga lagna changes rashi
between `--start` and `--end`, one segment list per unique amsha request.
Boundaries are root-found rather than sampled, so fast vargas such as D60
cannot skip segments. `--max-segments` caps the total segments across all
amshas (0 = the 50,000 library ceiling):

```bash
dhruv amsha-lagna-events --start 2026-04-17T00:00:00Z --end 2026-04-18T00:00:00Z \
  --lat 28.6 --lon 77.2 --amsha D9,D60
```

Pure scalar jyotish formulas:

- `bhrigu-bindu`
- `prana-sphuta`
- `deha-sphuta`
- `mrityu-sphuta`
- `tithi-sphuta`
- `yoga-sphuta`
- `yoga-sphuta-normalized`
- `rahu-tithi-sphuta`
- `kshetra-sphuta`
- `beeja-sphuta`
- `tri-sphuta`
- `chatus-sphuta`
- `pancha-sphuta`
- `sookshma-trisphuta`
- `avayoga-sphuta`
- `kunda`
- `bhava-lagna`
- `hora-lagna`
- `ghati-lagna`
- `vighati-lagna`
- `varnada-lagna`
- `sree-lagna`
- `pranapada-lagna`
- `indu-lagna`
- `arudha-pada-compute`
- `sun-based-upagrahas`
- `calculate-ashtakavarga`
- `graha-drishti-compute`
- `graha-drishti-matrix-compute`

Search:

- `conjunction`
- `next-conjunction`
- `prev-conjunction`
- `search-conjunctions`
- `grahan`
- `next-chandra-grahan`
- `prev-chandra-grahan`
- `search-chandra-grahan`
- `next-surya-grahan`
- `prev-surya-grahan`
- `search-surya-grahan`

The unified `grahan --kind surya` command accepts `--include-path`,
`--path-step-minutes`, `--boundary-step-deg`, `--lat`, `--lon`, `--alt`, and
optional `--eop`. See `docs/end_user/solar_eclipse_visibility.md`.
- `lunar-phase`
- `next-purnima`
- `prev-purnima`
- `search-purnimas`
- `next-amavasya`
- `prev-amavasya`
- `search-amavasyas`
- `sankranti`
- `next-sankranti`
- `prev-sankranti`
- `search-sankrantis`
- `next-specific-sankranti`
- `prev-specific-sankranti`
- `motion`
- `next-stationary`
- `prev-stationary`
- `search-stationary`
- `next-max-speed`
- `prev-max-speed`
- `search-max-speed`
- `gochar-events`

`gochar-events` is the grouped CLI surface for yearly/monthly Tajaka returns,
yearly/monthly Tithi Pravesha returns, and transit-to-natal aspect events.
Key inputs are:

- `--birth-date` and query `--date`
- location flags `--lat --lon --alt`
- `--tajaka-basis tropical-solar|sidereal-solar`
- `--yearly-count`, `--monthly-count`
- `--transit-window-days`
- repeated `--transit-body` names or gochar transit codes
- repeated `--natal-target 'kind|index|longitude|name'`

Natal target kinds are `graha`, `bindu`, `sphuta`, `special-lagna`,
`arudha-pada`, and `custom`. Transit output preserves the caller-supplied name
on each emitted target row. Accepted `--transit-body` names include `sun`,
`moon`, `mars`, `mercury`, `jupiter`, `venus`, `saturn`, `rahu`, `ketu`,
`uranus`, `neptune`, and `pluto`.

Dasha and tara:

- `dasha`
- `tara-list`
- `tara-position`

The `dasha` command now uses one surface for both invocation styles:

- derived birth context via `--birth-date` plus `--lat` / `--lon`
- raw dasha context via `--birth-jd` plus input attributes such as
  `--moon-sid-lon`, `--graha-sidereal-lons`, `--lagna-sidereal-lon`,
  `--sunrise-jd`, and `--sunset-jd`

Level-0 cycle repetition (nakshatra-based and Yogini systems only; other
systems ignore these):

- `--cycles N` — emit exactly N whole mahadasha cycles (default: the
  system's built-in cycle count). Wins over `--min-span-years`.
- `--min-span-years Y` — append whole cycles until level-0 coverage from
  birth reaches at least Y years; the final cycle completes even if it
  overshoots.

On `kundali`, the same knobs are `--dasha-cycles` and
`--dasha-min-span-years`. A period's cycle number can be derived from its
global `order`: `cycle = (order - 1) / sequence_len + 1`.

## Equatorial Output

`graha-positions --equatorial` (and `kundali --include-equatorial`) adds,
per entry, geocentric right ascension, declination, and ecliptic latitude
in degrees (equinox of date; nutation applied when `--nutation` is set),
plus Greenwich mean/apparent sidereal time (`GMST`/`GAST`) for the request
instant. Positions are geometric (no light-time or aberration). Point-like
entries — lagna, Rahu, Ketu — lie on the ecliptic, so their ecliptic
latitude is exactly 0.

## Positions Series

`graha-positions --to-date <UTC> --step-minutes <N>` switches to series
mode: positions are sampled from `--date` to `--to-date` every N minutes
(endpoints inclusive when they fall on the grid, at most 10,000 points).
Each point prints the epoch, optional `GMST`/`GAST`, and one line per
entry with the sidereal longitude plus RA/declination/ecliptic latitude
when `--equatorial` is set. All other flags behave as in single-epoch
mode.

Grahan output also reports the Moon's (chandra) or Sun's (surya)
apparent right ascension and declination at greatest grahan (degrees,
equinox of date, nutation applied).

Chara-style dasha periods use dual lordship for Kumbha (`Shani`/`Rahu`) and
Vrischika (`Mangal`/`Ketu`). Rahu owns Kumbha and Ketu owns Vrischika for the
default sign-lord-based node dignity policy.

## Important Config Behavior

Time-based upagraha options affect:

- `upagrahas`
- `core-bindus`
- `kundali`

Amsha scope on `kundali` can promote dependent root sections automatically when
those sub-sections are requested.

`node_policy` and charakaraka scheme are public kundali/chart behavior knobs.

Layered config behavior is also public CLI behavior:

- explicit CLI flags override config files
- operation-specific config overrides common config
- recommended defaults apply unless `--defaults-mode none` is selected

For the full option-level reference, use [`docs/cli_reference.md`](../../cli_reference.md).
