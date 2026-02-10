# Horizons Golden Fixtures

Frozen validation vectors used to compare engine output against JPL Horizons.

## Source

- JPL Horizons API v1.2
- Fetched: 2026-02-10
- Ephemeris: DE441 (Horizons default)
- Output type: Geometric cartesian states (no aberration/light-time)
- Reference frame: ICRF
- Units: km, km/s

## Files

- `vectors.json` — Reference state vectors for 10 test cases covering
  Sun, Mercury, Venus, Earth, Moon, Mars, Jupiter, Saturn at two epochs
  (J2000 and JD 2460000.5).

## Usage

Integration tests in `crates/eph_core/tests/horizons_golden.rs` embed these
reference values directly and compare against our DE442s engine output.

## Tolerance rationale

See `docs/numeric_error_budget.md` for the full error budget analysis.

- Inner planets/Moon/Sun: 1.0 km position, 1e-5 km/s velocity
- Outer planet barycenters: 5.0 km position, 1e-5 km/s velocity

Cross-kernel differences (DE441 vs DE442s) are the dominant error source.
