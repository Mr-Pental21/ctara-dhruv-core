# Clean-Room Implementation Record: Solar Eclipse Visibility

## Subsystem

- Name: Surya grahan global visibility and local circumstances
- Owner: ctara-dhruv contributors
- Date: 2026-07-13

## Scope

- Implement an original vector-geometry model of the Sun-Moon shadow cones.
- Derive instantaneous Besselian elements from Dhruv's loaded ephemeris.
- Detect every solar eclipse whose penumbra intersects the Earth ellipsoid.
- Compute global classification, contacts, greatest eclipse, central path,
  central limits, partial-visibility boundaries, and optional local contacts.
- Extend the unified `grahan` operation and every public language surface.
  C-derived wrappers consume owned geometry through read-only C ABI accessors;
  Rust and Elixir expose the same typed samples directly.

## Conceptual Sources

1. NASA/GSFC, "Besselian Elements of Solar Eclipses"
   - URL: https://eclipse.gsfc.nasa.gov/SEcat5/beselm.html
   - Status: United States government publication; used as a public-domain
     conceptual description of the fundamental plane and element meanings.
   - Use: names and meanings of `x`, `y`, `d`, `mu`, `l1`, `l2`, `f1`, and
     `f2`. No NASA polynomial coefficients or eclipse tables are embedded.
2. NASA/GSFC, "Explanation of Solar Eclipse Predictions"
   - URL: https://eclipse.gsfc.nasa.gov/SEmono/reference/explain.html
   - Status: United States government publication; public-domain conceptual
     and black-box validation reference.
   - Use: definitions of global path products, contact circumstances, and
     published-output comparisons only.
3. IERS Conventions 2010, Technical Note 36, Chapter 5
   - URL: https://iers-conventions.obspm.fr/content/tn36.pdf
   - Status: open international technical standard already used by
     `dhruv_time` and `dhruv_frames`.
   - Use: UT1 Earth rotation and celestial/terrestrial frame conventions.
4. IAU 2015 Resolution B3 nominal radii
   - URL: https://www.iau.org/static/resolutions/IAU2015_English.pdf
   - Status: open IAU standard already recorded for the existing grahan code.
   - Use: nominal solar radius and Earth equatorial radius; the existing mean
     lunar radius is retained for mean-limb predictions.

## Explicitly Excluded Sources

- Denylisted projects reviewed: `None`
- Source-available/proprietary projects reviewed: `None`
- No Swiss Ephemeris or copyleft astrology/eclipses implementation was
  consulted, summarized, or used to derive this work.

## Data Provenance

- Runtime eclipse catalogs, Besselian coefficient tables, and path datasets:
  none.
- Runtime astronomical data: caller-provided JPL/NAIF SPK and leap-second
  kernels already governed by the repository kernel manifests.
- Runtime Earth orientation: caller-provided IERS EOP through `EopKernel`.
- Constants: existing IAU nominal radii plus the conventional oblate-Earth
  flattening documented in implementation comments.
- No third-party source file or table is copied.

## Implementation Notes

- Sun and Moon geocentric vectors define a shadow-axis unit vector.
- The fundamental plane passes through the geocenter perpendicular to that
  axis. Its east/north basis is constructed from the true pole of date.
- Penumbral and umbral/antumbral radii follow directly from similar triangles
  between the finite solar and lunar disks; no tabulated eclipse elements are
  consumed.
- `l2` follows the common signed convention: negative for umbra (total) and
  positive for antumbra (annular).
- Earth intersection uses an oblate ellipsoid. UTC is retained alongside JD;
  UT1 controls terrestrial longitude.
- A cone generator can meet the ellipsoid twice. When every generator reaches
  Earth, the Sun-facing intersections form the footprint ring. For a grazing
  cone, the entry and exit branches are joined at numerically refined tangent
  rays. Boundary segments are adaptively subdivided near those tangencies so
  coarse angular requests do not introduce long synthetic ground chords.
- Central-path limits are selected from the local cone intersection around the
  shadow-axis ground point. This prevents a second, distant ellipsoid branch
  from becoming a false northern or southern corridor limit near polar or
  grazing contacts.
- Mean-limb results do not model mountains/valleys on the lunar limb, Baily's
  beads, atmospheric refraction, or local terrain unless explicitly stated.

## Validation

- NASA/GSFC catalog/path/local-circumstance values are black-box I/O oracles
  only; their implementation and coefficient tables are not used.
- The 2001-2100 integration test finds 224 events and matches the published
  type distribution exactly: 68 total, 72 annular, 7 hybrid, and 77 partial.
- Focused integration cases cover the 2024-04-08 total eclipse, the
  2024-10-02 annular eclipse, the 2013-11-03 hybrid eclipse, the non-central
  2014-04-29 annular eclipse, the antimeridian-crossing 2002-06-10 annular
  path, a polar footprint above 80 degrees in 2025, local totality, path
  width, central duration, geographic greatest point, contact ordering, and
  backward search.
- Ring-contract tests cover centered and grazing synthetic cones plus central,
  partial-only, polar, and antimeridian events. A high-gamma 2026 annular case
  verifies explicit closure, bounded consecutive ground segments, and that
  every timestamp-matched central-path point is inside its penumbral ring. The
  same case verifies that both central-corridor limits stay local to each path
  point rather than jumping to a distant cone branch.
- Published path comparisons use tolerances appropriate to a mean-limb,
  geometric model; the tolerances are stated in the integration tests.

## Contributor Declaration

- This implementation is clean-room and is not derived from denylisted,
  source-available, or proprietary code.
- Name: OpenAI Codex (AI-authored contribution under project policy)
- Date: 2026-07-13
