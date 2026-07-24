# Clean-Room Provenance: Conjunction/Aspect Search

## Algorithm

Standard numerical root-finding by bisection on the angular difference function:

    f(t) = normalize(lon1(t) - lon2(t) - target)

where `normalize` wraps to [-180, +180] degrees.

Zero crossings of f(t) correspond to the target ecliptic longitude separation.

### Steps

1. **Coarse scan**: step through time at fixed intervals, evaluating f(t) at each step
2. **Sign change detection**: when f(t_a) and f(t_b) have opposite signs AND the
   function change is < 270° (to exclude wrap-around discontinuities at ±180°),
   a genuine zero crossing is bracketed
3. **Bisection refinement**: repeatedly halve the interval until convergence
   (default 1e-8 days ≈ 0.86 ms)

### Bodies

Both bodies are `TransitBody` values: any plain ephemeris body plus Rahu/Ketu
(wire codes 10007/10008). Node longitudes come from the existing clean-room
lunar-node model (`docs/clean_room_lunar_nodes.md`) with the model selected by
`ConjunctionConfig::node_mode` (mean or true/osculating; default true); node
ecliptic latitude is 0 by definition (the node lies in the reference plane).

### Scan window (next/prev)

The next/prev scan ceiling is pair-aware:

    max_scan_days = max(800, 1.3 * 360 / |rate1 - rate2|)

where the rates are mean geocentric longitude rates in deg/day derived from
standard J2000 sidereal orbital periods (`TransitBody::mean_rate_deg_per_day`;
inferior planets share the Sun's mean geocentric rate, nodes regress). Slow
pairs (e.g. Jupiter-Saturn, ~7250-day synodic period; node-Saturn ~4160 days)
previously exceeded the fixed 800-day window and silently returned no event.
Pairs with near-identical mean rates (|difference| < 1e-3 deg/day, i.e. Sun
with Mercury/Venus) oscillate about each other, so any reachable separation
recurs within the 800-day baseline and the baseline is kept. Range searches
remain bounded by the caller's range.

A mid-scan engine error (ephemeris coverage edge) ends a next/prev scan as
"no event" (`Ok(None)`); an error at the start sample still propagates.

### Operation extras

The operation layer (`ConjunctionOperation`) adds, on top of the same search:

- `target_separations_deg`: a multi-angle sweep — the search runs per angle;
  next/prev return the nearest event across angles, range results are merged
  and time-sorted; each event carries the angle it matched in
  `target_separation_deg`.
- `sankranti_config`: when set, each event is enriched with sidereal echoes
  (`body{1,2}_sidereal_longitude_deg`, `body{1,2}_rashi_index`) computed with
  the supplied ayanamsha configuration; the echo's node model is forced to
  match the search's `node_mode`.

## Sources

- Bisection method: standard numerical analysis (any textbook)
- Ecliptic coordinates: ICRF → ecliptic J2000 rotation using obliquity constant
  from IAU 2006 (already implemented in `dhruv_frames`)
- Lunar-node longitudes: existing clean-room model
  (`docs/clean_room_lunar_nodes.md`)
- Mean longitude rates: derived from standard J2000 mean orbital periods
  (public astronomical reference data); used only to bound scan windows
- No external ephemeris code referenced
- No GPL/AGPL/copyleft code consulted

## Constants

Mean geocentric longitude rates (deg/day) in
`TransitBody::mean_rate_deg_per_day`, derived from public J2000 orbital
periods. Otherwise uses existing `dhruv_frames` obliquity constant.

## Notes

- The coarse scan step size must be small enough to bracket each crossing.
  For Sun-Moon pairs, 0.5 days safely catches all crossings within a synodic
  period (~29.5 days). For slow outer-planet pairs (Jupiter-Saturn mean
  synodic period ~7250 days), 2.0 day steps are sufficient.
- Wrap-around at the 0°/360° boundary is handled by the normalize function.
- Retrograde motion can cause multiple crossings per synodic period; the
  step size must be small enough to bracket each one independently.

## Validation

Golden tests (`crates/dhruv_search/tests/conjunction_golden.rs`,
kernel-gated) include the 2020-12-21 Jupiter-Saturn great conjunction, a
Jupiter-Saturn next-search from 2015 that succeeds beyond the old 800-day
window, and node anchors: the Sun-Rahu conjunction within days of the
2024-04-08 total solar eclipse and Sun-Ketu near the 2024-10-02 annular
eclipse, with node latitude exactly 0.
