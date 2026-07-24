# Clean-Room Provenance: Stationary Point & Max-Speed Search

## Feature
Detection of planetary stationary points (retrograde/direct stations) and
peak-speed events via geocentric ecliptic longitude velocity analysis.

The searched body is a `TransitBody`: any plain ephemeris body plus
Rahu/Ketu (wire codes 10007/10008). Node longitudes come from the existing
clean-room lunar-node model (`docs/clean_room_lunar_nodes.md`) with the
model selected by `StationaryConfig::node_mode` (default true/osculating);
node speed uses the same 1-minute central-difference stencil as planets.

## Algorithm Description

### Stationary Points
A geocentric stationary point occurs when a planet's ecliptic longitude
velocity crosses zero. The algorithm:

1. **Coarse scan**: step through time evaluating the ecliptic longitude
   speed (deg/day) at each point. Detect sign changes in the speed.
2. **Bisection refinement**: when a sign change is found between two
   adjacent samples, bisect the interval to converge on the zero crossing.
3. **Classification**: speed positive→negative = StationRetrograde (planet
   begins apparent backward motion), negative→positive = StationDirect
   (planet resumes forward motion).

### Max-Speed Events
A max-speed event occurs when the planet's ecliptic longitude acceleration
crosses zero (velocity reaches a local extremum). The algorithm:

1. **Numerical acceleration**: computed via central difference
   `a(t) = (v(t+h) - v(t-h)) / (2h)` with `h = 0.01` days.
2. **Coarse scan**: step through time evaluating acceleration. Detect
   sign changes.
3. **Bisection refinement**: refine the zero crossing of acceleration.
4. **Classification**: speed > 0 at extremum = MaxDirect (peak forward
   speed), speed < 0 = MaxRetrograde (peak retrograde speed).

### Velocity Pipeline
The ecliptic longitude speed is obtained from the existing engine pipeline:
- Chebyshev polynomial evaluation yields Cartesian position and velocity
- Engine applies ICRF→Ecliptic rotation to both position and velocity
  vectors when `Frame::EclipticJ2000` is requested
- `cartesian_state_to_spherical_state()` converts to spherical coordinates
  including `lon_speed` in rad/s
- Final conversion: `deg/day = lon_speed × (180/π) × 86400`

## Sources

- **Numerical bisection**: standard numerical root-finding method, textbook
  material (e.g., Burden & Faires, "Numerical Analysis").
- **Central difference**: standard numerical differentiation formula,
  O(h²) accuracy.
- **Retrograde motion**: standard geocentric observational astronomy. A
  planet appears to move retrograde when Earth overtakes it (superior
  planets) or it overtakes Earth (inferior planets). This is apparent
  motion only, caused by the relative orbital velocities.
- **Lunar-node stations**: the true (osculating) node oscillates around its
  mean regression and stations roughly weekly, so
  `StationaryConfig::lunar_node()` uses a 0.25-day coarse step. The mean
  node regresses monotonically and has no stations; stationary search with
  `node_mode = Mean` is rejected with `InvalidConfig`. Max-speed search
  accepts both node models.
- **Ecliptic longitude velocity**: derivative of the standard ecliptic
  longitude coordinate, a direct output of the spherical coordinate
  transformation applied to Cartesian state vectors.

## What Was NOT Referenced

- No Swiss Ephemeris code or algorithms
- No Astro.com code
- No GPL/AGPL/copyleft implementations
- Classification and body validation rules derived from first principles
  of orbital mechanics (Sun and Moon never retrograde geocentrically)

## Notes

- Next/prev scans treat a mid-scan engine error (ephemeris coverage edge)
  as "no event" (`Ok(None)`); an error at the start sample still
  propagates.
- The operation layer (`MotionOperation`) accepts an optional
  `sankranti_config`; when set, events also carry `sidereal_longitude_deg`
  and `rashi_index` echoes computed with that ayanamsha configuration (the
  echo's node model is forced to match the search's `node_mode`).

## Validation

Golden values compared against widely published retrograde dates for
Mercury and Mars (public astronomical almanac data, e.g., USNO/HMNAO).
True-node station behavior is covered by kernel-gated tests in
`crates/dhruv_search/tests/stationary_golden.rs`; mean-node rejection is
unit-tested in `crates/dhruv_search/src/stationary.rs`.
