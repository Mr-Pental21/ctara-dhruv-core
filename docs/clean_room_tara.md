# Clean-Room Implementation Record: Fixed Star Positions (`dhruv_tara`)

## Subsystem
- Name: dhruv_tara — Fixed star positions (proper motion propagation + coordinate transforms)
- Owner: ctara-dhruv-core project
- Date: 2026-02-27

## Scope
- What is being implemented: Fixed star position computation for 122 Vedic/astronomical
  reference stars (28 nakshatra yogataras, 80 rashi constellation stars, 12 special Vedic
  stars, 2 galactic reference points), including space motion propagation,
  ecliptic/sidereal coordinate transforms, annual aberration, and gravitational light deflection.
- Note: The original plan estimated ~152 stars. The final count is 122 after deduplication
  (e.g., Agastya/Canopus, Lubdhaka/Sirius share physical stars across categories) and
  removing stars without reliable Hipparcos astrometry.
- Public API surface impacted: New crate `dhruv_tara`, FFI functions (`dhruv_tara_*`),
  CLI subcommands (TaraList, TaraPosition), dhruv_rs wrapper functions.

## Conceptual Sources

### Space Motion Propagation
- Butkevich, A. G. & Lindegren, L. (2014). "Rigorous treatment of barycentric stellar
  motion." A&A 570, A62. DOI: 10.1051/0004-6361/201424483
  - Open access (A&A is open-access for articles >12 months old).
  - Used: Equations for converting 6 astrometric parameters (α, δ, ϖ, μα*, μδ, vr) to
    Cartesian position+velocity and linear propagation. Section 3: space motion vector formulation.
- Hipparcos Catalogue, Volume 1, Section 1.5 (ESA SP-1200, 1997).
  - Public domain (ESA publication).
  - Used: Same formulation as Butkevich & Lindegren, confirming the Cartesian approach.

### Apparent Place Corrections
- IAU SOFA Library documentation (iausofa.org): routines PMSAFE, ATCI13.
  - Public domain (IAU standards).
  - Used: Algorithm descriptions for annual aberration and gravitational light deflection.
  - Note: SOFA C/Fortran SOURCE CODE is NOT referenced — only the algorithm documentation
    (which describes the IAU standard procedure). Our implementation is original.
- Annual aberration: First-order relativistic formula
  s' = s + (1/c)(v − (s·v)s), standard textbook result (e.g., Seidelmann 1992, "Explanatory
  Supplement to the Astronomical Almanac", public domain US government work).
- Gravitational light deflection: PPN formula (SOFA iauLd convention):
  Δs = (2GM☉/(c²D)) × [(s·e)s − e] / (1 + s·e)
  where e = unit vector from Sun to observer, D = Sun-observer distance.
  Standard GR result from Misner, Thorne & Wheeler (1973) / Will (1993).

### Galactic Frame
- IAU 2000 definition of the Galactic coordinate system:
  NGP at (α=192.85948°, δ=27.12825°), θ₀=122.93192° (position angle of GNP).
  Source: Liu, Zhu & Zhang (2011), A&A 526, A16 (open access).
- Galactic Center ICRS coordinates: Reid & Brunthaler (2004), ApJ 616, 872.
  Open-access preprint: arXiv:astro-ph/0408107.

### Precession, Nutation, Obliquity
- Reuses existing dhruv_frames crate functions (already documented in clean_room_ayanamsha.md):
  - Vondrák et al. (2011) precession
  - IAU 2000B nutation
  - IAU 2006 mean obliquity

## Explicitly Excluded Sources
- Denylisted projects reviewed: `None`
- Source-available/proprietary projects reviewed: `None`
- Note: Swiss Ephemeris fixed star routines were NOT consulted. No GPL code was referenced.

## Data Provenance

### SIMBAD (Phase 1 — current)
- Source: Centre de Données astronomiques de Strasbourg (CDS), SIMBAD TAP service.
- License: CC-BY / Open Licence for public information (CDS GTCU, Jan 2026).
- What we use: ICRS J2000.0 positions (RA, Dec), proper motions (μα*, μδ), parallax,
  radial velocities, visual magnitudes for 120 bright stars (2 galactic reference points
  have no catalog entries — they use fixed IAU frame constants).
- Reference epoch: J2000.0 (ICRS).
- Attribution: "This research has made use of the SIMBAD database, operated at CDS,
  Strasbourg, France (Wenger et al. 2000)."
- Note: The catalog file is named `hgca_tara.json` for forward compatibility with Phase 2.
  The `source` field in the JSON is `"SIMBAD_ICRS_J2000"`.

### HGCA (Phase 2 — planned, not yet used)
- Source: Brandt, T. D. (2021). "The Hipparcos-Gaia Catalog of Accelerations: Gaia EDR3
  Edition." ApJS 254, 42.
- Available via: VizieR catalog J/ApJS/254/42
- License: VizieR distributes public astronomical catalogs under CC-BY / Open Licence
  (CDS data use policy, https://cds.unistra.fr/legals/). Citation required.
- Data type: Astrometric parameters (positions, proper motions, parallaxes) — factual
  scientific measurements derived from Hipparcos and Gaia EDR3 observations. Factual data
  are not copyrightable creative works.
- What would change: Reference epoch moves to J2016.0 (HGCA native epoch), improved
  proper motions from Hipparcos+Gaia EDR3 cross-calibration. The catalog schema and
  pipeline code are already designed to handle any reference epoch via `reference_epoch_jy`.
- Attribution: "This work has made use of data from the Hipparcos-Gaia Catalog of
  Accelerations (Brandt 2021), accessed via the CDS VizieR service."

### SIMBAD (cross-identification)
- Also used for: Cross-identification of star names (Bayer designations → HIP numbers).
  See "SIMBAD (Phase 1)" above for license and attribution.

### Gaia DR3 (Phase 2 — NOT YET USED)
- Source: ESA Gaia mission, Gaia Collaboration (2023).
- License: CC-BY-SA 3.0 IGO. The ShareAlike clause requires derivative works to carry
  the same license, which may conflict with MIT redistribution of extracted subsets.
- Status: DEFERRED to Phase 2 pending explicit license compatibility review.
  The HGCA already incorporates Gaia EDR3 astrometry, so Phase 1 has full coverage.

### Star identification data
- Nakshatra yogataras: Standard Vedic astronomical texts (Surya Siddhanta, Brihat Samhita).
  Star-yogatara associations are traditional knowledge in the public domain.
- Bayer designations: Standard astronomical nomenclature (public domain).
- HIP numbers: Hipparcos catalog (ESA, public domain).

### Physical constants
- AU_KM = 149597870.7 km: IAU 2012 definition (public domain standard).
- GM☉ = 1.32712440018e20 m³/s²: IAU 2015 nominal value (public domain standard).
- c = 299792458 m/s: SI definition (public domain standard).
- MAS_TO_RAD = π/(180×3600×1000) = 4.848136811095360e-9

### TCB vs TDB note
- Gaia DR3 (and HGCA) reference epoch J2016.0 is defined in TCB (Barycentric Coordinate
  Time), not TDB. The difference at J2016.0 is ~19 seconds due to L_B secular drift
  (L_B = 1.550519768e-8). For proper motion propagation, this shifts positions by at most
  μ × 19s ≈ 1.2 μas for the fastest catalog star (Arcturus, ~2″/yr). This is completely
  negligible compared to our target accuracy (~1″). We treat the reference epoch as TDB
  without correction. The same approximation is standard practice in astrometry software.

## Implementation Notes
- Key algorithm choices:
  - Linear proper motion propagation (no perspective acceleration, no radial velocity
    secular acceleration). Sufficient for ~1″ accuracy over centuries.
  - Zero/negative parallax treated as 1e6 AU (effectively infinite distance).
  - Missing radial velocity treated as 0.0 km/s.
  - Galactic reference points (Center, Anti-Center) have NO proper motion — fixed ICRS
    directions rotated to ecliptic via IAU 2000 matrix.
- Numerical assumptions:
  - f64 throughout (IEEE 754 double precision).
  - Trigonometric functions from Rust std (libm).
  - No iterative algorithms — all computations are closed-form.
- Edge cases:
  - Star at ecliptic pole: atan2(0,0) → lon=0° (Rust atan2 convention).
  - Light deflection at χ→0° (near Sun): formula is valid but stars near Sun are
    unobservable; no special handling needed.
  - Light deflection at χ=180° (anti-Sun): sin(180°)=0, so deflection=0. Correct.

## Validation
- Black-box references used (I/O comparison only):
  - SIMBAD J2000.0 ICRS coordinates for HIP 65474 (Spica) — queried via TAP, frozen in
    test comments. Used to verify catalog position at J2000.0 (Δt=0 identity).
  - IAU standard aberration constant κ = 20.4955″ — textbook value for unit test.
- Golden test vectors added:
  1. Spica equatorial at J2000.0 (tolerance 0.01°)
  2. Spica ecliptic longitude at J2024.0 (tolerance 0.1°)
  3. Spica sidereal longitude Lahiri ≈ 180° (tolerance 0.15°)
  4. Galactic Center ecliptic longitude (tolerance 0.5°)
  5. Zero-Δt identity (tolerance 1e-10°)
  6. Arcturus large PM direction sanity (tolerance 0.01°)
  7. Aberration magnitude ≈ 20.5″ (tolerance 0.1″)
  8. Light deflection at χ=45° and χ=90° (tolerance 0.5 mas)
  9. Null earth_state rejected for Apparent tier
  10. Astrometric vs Apparent bounded difference
  11. Nutation on/off difference in [4″, 18″] range
- Error tolerance rationale: documented per test in position_golden.rs.

## Contributor Declaration
- This implementation is clean-room and does not derive from denylisted/source-available code.
- All algorithm sources are open-access peer-reviewed publications or public-domain standards.
- All data sources are verified as CC-BY, Open Licence, or public domain.
