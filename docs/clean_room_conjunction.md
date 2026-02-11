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

## Sources

- Bisection method: standard numerical analysis (any textbook)
- Ecliptic coordinates: ICRF → ecliptic J2000 rotation using obliquity constant
  from IAU 2006 (already implemented in `dhruv_frames`)
- No external ephemeris code referenced
- No GPL/AGPL/copyleft code consulted

## Constants

None specific to this module. Uses existing `dhruv_frames` obliquity constant.

## Notes

- The coarse scan step size must be small enough to bracket each crossing.
  For Sun-Moon pairs, 0.5 days safely catches all crossings within a synodic
  period (~29.5 days). For outer planet pairs (Jupiter-Saturn ~398 day synodic
  period), 2.0 day steps are sufficient.
- Wrap-around at the 0°/360° boundary is handled by the normalize function.
- Retrograde motion can cause multiple crossings per synodic period; the
  step size must be small enough to bracket each one independently.
