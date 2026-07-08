# Clean Room Record: Equatorial Output Surface

Date: 2026-07-07

Scope:
- Add `include_equatorial` to `graha_positions_config`: per-entry geocentric
  right ascension, declination, and ecliptic latitude (equinox of date), plus
  Greenwich mean/apparent sidereal time (`gmst_deg`/`gast_deg`) on the
  `graha_positions` result.
- Propagate the same surface through Rust, C ABI, CLI, and language wrappers.

Provenance:
- Ecliptic-to-equatorial rotation: standard spherical trigonometry about the
  equinox (public domain; e.g. Explanatory Supplement to the Astronomical
  Almanac / any spherical astronomy text):
  - tan(alpha) = (sin(lambda)·cos(eps) − tan(beta)·sin(eps)) / cos(lambda)
  - sin(delta) = sin(beta)·cos(eps) + cos(beta)·sin(eps)·sin(lambda)
- Mean obliquity: IAU 2006 (already in `dhruv_frames::obliquity`).
- Nutation: IAU 2000B (already in `dhruv_frames::nutation`).
- GMST: Capitaine et al. 2003 / ERA per IERS Conventions 2010 (already in
  `dhruv_time::sidereal`). GAST = GMST + equation of the equinoxes
  (Δψ·cos ε, classical form; complementary terms omitted, sub-milliarcsecond).

Non-sources:
- No denylisted or source-available astrology/ephemeris implementation was
  consulted; the change composes existing in-repo primitives.

Conventions:
- Degrees for all output angles; RA normalized to [0, 360).
- Equinox of date. Nutation in longitude (Δψ) and true obliquity are applied
  when the request's `use_nutation` flag is set; otherwise mean equinox and
  mean obliquity of date. This keeps the RA frame consistent with the emitted
  sidereal times (pair apparent RA with `gast_deg`, mean RA with `gmst_deg`).
- Geocentric, geometric positions (no light-time or annual aberration —
  consistent with the engine's longitude outputs; annual aberration is
  ~20.5 arcsec).
- Rahu/Ketu (true nodes) and point-like entries (lagna) lie on the ecliptic:
  `ecliptic_latitude_deg` is exactly 0 and the conversion is exact. The lagna
  tropical longitude is reconstructed from its sidereal value (lagna is
  already computed from apparent sidereal time, so no extra Δψ is applied).
- Outer planets carry their true ecliptic latitudes.

Validation:
- Unit tests: rotation identities at equinox/solstice points and latitude
  symmetry (`dhruv_search/src/jyotish.rs`).
- Golden integration test (`dhruv_search/tests/jyotish_golden.rs`):
  2024-01-15 12:00 UTC — Sun declination/RA golden bands, node/lagna zero
  latitude, node declination mirror symmetry, `tan(delta) = tan(eps)·sin(alpha)`
  self-consistency for the Sun, and `gmst_deg` equality with
  `dhruv_time::gmst_rad`.
