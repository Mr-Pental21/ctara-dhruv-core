# Horizons Golden Fixtures

Frozen validation vectors used to compare engine output against JPL Horizons.

## Source

- JPL Horizons API v1.2
- Fetched: 2026-02-10
- Ephemeris: DE441 (Horizons default)
- Output type: Geometric cartesian states (no aberration/light-time)
- Reference frames: ICRF and ecliptic J2000
- Units: km, km/s

## Files

- `vectors.json` — Reference state vectors for 19 test cases covering
  all planets (Sun through Pluto) across multiple epochs (J2000, perihelion,
  aphelion, 1900, 2050, JD 2460000.5) in both ICRF and ecliptic frames.

## Usage

Integration tests in `crates/eph_core/tests/horizons_golden.rs` embed these
reference values directly and compare against our DE442s engine output.

## Tolerance rationale

See `docs/numeric_error_budget.md` for the full error budget analysis.

- Inner planets/Moon/Sun: 1.0 km position, 1e-5 km/s velocity
- Gas-giant barycenters (Mars/Jupiter/Saturn): 5.0 km position, 1e-5 km/s velocity
- Ice-giant/TNO barycenters (Uranus/Neptune/Pluto): 250 km position, 1e-5 km/s velocity

Cross-kernel differences (DE441 vs DE442s) are the dominant error source.
