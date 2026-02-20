# Numeric Error Budget

## Validation Methodology

- **Reference system**: JPL Horizons API v1.2 (accessed 2026-02-10).
- **Horizons ephemeris**: DE441.
- **Our ephemeris**: DE442s (JPL Development Ephemeris 442, short-range file).
- **Comparison mode**: Geometric state vectors — no aberration, no light-time.
- **Reference frame**: ICRF (equivalent to J2000 at ~0.02 arcsec level).
- **Units**: position in km, velocity in km/s.
- **Test epoch range**: JD 2451545.0 (J2000) to JD 2460000.5 (2023-Feb-25).

## Error Sources

### 1. Cross-kernel differences (DE441 vs DE442s)

This is the dominant error source. DE442 is a newer fit with additional
observational data compared to DE441. Differences scale with:
- **Heliocentric distance**: outer planets show larger absolute differences.
- **Time from fit epoch**: differences grow further from the constraining data.
- **Body type**: barycenters are better determined than body centers.

Observed at J2000:
| Body | Max component error (km) | Relative accuracy |
|------|--------------------------|-------------------|
| Sun | < 0.01 | < 10 ppb |
| Mercury | < 0.05 | < 1 ppb |
| Venus | < 0.05 | < 0.5 ppb |
| Earth | < 0.1 | < 1 ppb |
| Moon (wrt Earth) | < 0.02 | < 0.05 ppb |
| Mars barycenter | < 0.1 | < 0.5 ppb |
| Jupiter barycenter | ~3 km | ~5 ppb |
| Saturn barycenter | < 1 km | < 1 ppb |

### 2. TDB approximation

Our TDB formula uses the NAIF one-term sinusoidal approximation:

    TDB = TT + 0.001657 * sin(M + 0.01671 * sin(M))

This has ~30 microsecond accuracy. At Earth's orbital velocity (~30 km/s),
30 us of timing error produces ~0.001 km position error — negligible.

### 3. Chebyshev interpolation

SPK Type 2 uses Chebyshev polynomials within time intervals. Our Clenshaw
recurrence evaluation matches the NAIF reference implementation to machine
precision (~10^-15 relative). No measurable contribution to error.

### 4. Frame rotation (ICRF to ecliptic J2000)

Uses IAU 1976 obliquity constant (23.4392911 degrees) with precomputed
sin/cos for the fixed ICRF→ecliptic J2000 rotation. Rotation preserves
vector magnitude to machine precision. No measurable contribution for
ICRF-frame comparisons.

### 4a. Ecliptic precession (J2000 → ecliptic of date)

All ecliptic longitudes reported by the engine are in the **ecliptic of date**
(IAU 2006 full 3D precession, implemented from Capitaine et al. 2003, Table 1).

The precession rotation P = R3(-(Π_A + p_A)) · R1(π_A) · R3(Π_A) is applied
after the ICRF→ecliptic J2000 step. Skipping this step would introduce a
systematic error of ~104 arcsec/century (~1.73 arcmin/century) in ecliptic
longitude — far exceeding acceptable Vedic astrological tolerances.

Residual error after applying IAU 2006 precession:
- p_A polynomial truncated at 5th order → < 0.001 arcsec/century residual
- Velocity: finite-difference at t ± 1 min captures the Ṗ·r cross-term;
  velocity error < 0.002 arcsec/day

No measurable contribution to Tier-1 body position errors (which are dominated
by cross-kernel differences).

### 4b. Obliquity of date (ecliptic → equatorial)

Sunrise/sunset uses `mean_obliquity_of_date_rad(t)` (IAU 2006, 84381.406"
at T=0). Bhava/Lagna computations also use the obliquity of date rather than
the J2000 constant. Residual vs true obliquity: nutation in obliquity (~9.2"
peak) is not included here; it is < 0.003° and acceptable for Vedic computation.

### 5. Chain resolution (accumulation)

Positions are accumulated as the target body is resolved to SSB through
the segment chain (e.g., Earth → EMB → SSB). Each step adds floating-point
rounding. With 2-3 chain steps and f64 arithmetic, the accumulation error
is ~10^-10 km — negligible.

## Tolerance Tiers

### Tier 1: Inner planets and Moon (body-center segments available)

Bodies: Sun (10), Mercury (199), Venus (299), Earth (399), Moon (301).

- **Position**: 1.0 km
- **Velocity**: 1.0e-5 km/s

These bodies have direct body-center-to-barycenter segments in both DE441
and DE442s. Cross-kernel differences are sub-km.

### Tier 2: Gas-giant barycenters (no body-center segment)

Bodies: Mars (4), Jupiter (5), Saturn (6).

- **Position**: 5.0 km
- **Velocity**: 1.0e-5 km/s

Our engine resolves `Body::Mars` etc. to the system barycenter because DE442s
does not include planet-body-center-to-barycenter segments for these. The
Horizons references use matching barycenter codes. Cross-kernel differences
are larger for outer planets (~3 km for Jupiter).

Note: The body-center-to-barycenter offset for Mars is ~1-2 km (Phobos/Deimos
are tiny), for Jupiter it is ~700 km (Galilean moons), and for Saturn ~300 km
(Titan dominates). These offsets are NOT included in our current output — a
future enhancement would add satellite ephemeris support.

### Tier 3: Ice-giant / TNO barycenters

Bodies: Uranus (7), Neptune (8), Pluto (9).

- **Position**: 250.0 km
- **Velocity**: 1.0e-5 km/s

At 2–6 billion km distance, cross-kernel differences reach ~220 km for Uranus
and ~52 km for Neptune. The relative accuracy remains excellent (<0.1 ppm).

Observed at J2000:
| Body | Max component error (km) | Distance (km) | Relative accuracy |
|------|--------------------------|---------------|-------------------|
| Uranus bary | ~220 | ~2.9e9 | ~76 ppb |
| Neptune bary | ~52 | ~4.5e9 | ~12 ppb |
| Pluto bary | <5 | ~4.4e9 | <1 ppb |

## Golden Test Coverage

| Test ID | Target | Observer | Epoch (JD TDB) | Frame | Tier |
|---------|--------|----------|-----------------|-------|------|
| earth_ssb_j2000 | Earth | SSB | 2451545.0 | ICRF | 1 |
| sun_ssb_j2000 | Sun | SSB | 2451545.0 | ICRF | 1 |
| mercury_ssb_j2000 | Mercury | SSB | 2451545.0 | ICRF | 1 |
| venus_ssb_j2000 | Venus | SSB | 2451545.0 | ICRF | 1 |
| moon_earth_j2000 | Moon | Earth | 2451545.0 | ICRF | 1 |
| mars_bary_ssb_j2000 | Mars bary | SSB | 2451545.0 | ICRF | 2 |
| jupiter_bary_ssb_j2000 | Jupiter bary | SSB | 2451545.0 | ICRF | 2 |
| saturn_bary_ssb_j2000 | Saturn bary | SSB | 2451545.0 | ICRF | 2 |
| earth_ssb_2460000 | Earth | SSB | 2460000.5 | ICRF | 1 |
| moon_earth_2460000 | Moon | Earth | 2460000.5 | ICRF | 1 |
| uranus_bary_ssb_j2000 | Uranus bary | SSB | 2451545.0 | ICRF | 3 |
| neptune_bary_ssb_j2000 | Neptune bary | SSB | 2451545.0 | ICRF | 3 |
| pluto_bary_ssb_j2000 | Pluto bary | SSB | 2451545.0 | ICRF | 3 |
| earth_ssb_perihelion | Earth | SSB | 2451547.5 | ICRF | 1 |
| earth_ssb_aphelion | Earth | SSB | 2451729.5 | ICRF | 1 |
| earth_ssb_1900 | Earth | SSB | 2415020.5 | ICRF | 1 |
| earth_ssb_2050 | Earth | SSB | 2469807.5 | ICRF | 1 |
| earth_ssb_j2000_ecliptic | Earth | SSB | 2451545.0 | Ecliptic | 1 |
| moon_earth_j2000_ecliptic | Moon | Earth | 2451545.0 | Ecliptic | 1 |

## CI Policy

1. Golden fixtures are frozen in `testdata/horizons_golden/vectors.json`.
2. Integration tests in `crates/dhruv_core/tests/horizons_golden.rs` gate CI.
3. Any tolerance change requires explicit review and documented rationale.
4. Edge-epoch tests in `crates/dhruv_core/tests/edge_epochs.rs` cover boundary conditions.
5. Cross-platform consistency is verified in the CI matrix.

## Future Improvements

- Tighten tolerances if satellite ephemeris support is added (body center vs barycenter).
- Add angle-derived tolerance budgets when `dhruv_vedic_base` implements ayanamsha/longitude.
