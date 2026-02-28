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

### SIMBAD (Phase 1 — superseded)
- Source: Centre de Données astronomiques de Strasbourg (CDS), SIMBAD TAP service.
- License: CC-BY / Open Licence for public information (CDS GTCU, Jan 2026).
- What was used: ICRS J2000.0 positions (RA, Dec), proper motions (μα*, μδ), parallax,
  radial velocities, visual magnitudes for 120 bright stars.
- Reference epoch: J2000.0 (ICRS).
- Status: Superseded by HGCA Phase 2 for astrometry. Visual magnitudes (v_mag) still
  sourced from SIMBAD (HGCA does not provide magnitudes).
- Attribution: "This research has made use of the SIMBAD database, operated at CDS,
  Strasbourg, France (Wenger et al. 2000)."

### HGCA (Phase 2 — current)
- Source: Brandt, T. D. (2021). "The Hipparcos-Gaia Catalog of Accelerations: Gaia EDR3
  Edition." ApJS 254, 42.
- Available via: VizieR catalog J/ApJS/254/42 (115,346 stars)
- License: VizieR distributes public astronomical catalogs under CC-BY / Open Licence
  (CDS data use policy, https://cds.unistra.fr/legals/). Citation required.
- Data type: Astrometric parameters (positions, proper motions, parallaxes) — factual
  scientific measurements derived from Hipparcos and Gaia EDR3 observations.
- Reference epoch: J2016.0 (Gaia EDR3 epoch, treated as TDB per TCB/TDB note below).
- What we use: J2016.0 positions (RA_ICRS, DE_ICRS), Gaia EDR3 parallax (plx),
  Hipparcos-Gaia long-baseline proper motions (pmRAhg, pmDEhg), radial velocity (RV).
  The long-baseline PM (~25 year baseline) averages out short-term perturbations and
  provides the best estimate of long-term space motion for propagation over centuries.
- Coverage: 49 of 108 unique HIP IDs found in HGCA. The remaining 59 bright stars
  (mostly v_mag < 3, including Sirius, Spica, Aldebaran, Arcturus, Vega) are absent
  from HGCA due to Gaia EDR3 saturation for very bright stars. These 59 stars use
  SIMBAD J2000 astrometry pre-converted to J2016.0 via the full Cartesian propagation
  pipeline (propagate_cartesian_au with dt=16yr), ensuring epoch consistency.
- The catalog file `hgca_tara.json` has `"source": "HGCA_EDR3"` and
  `"reference_epoch_jy": 2016.0`.
- Attribution: "This work has made use of data from the Hipparcos-Gaia Catalog of
  Accelerations (Brandt 2021), accessed via the CDS VizieR service."

### SIMBAD (cross-identification + fallback)
- Used for: Cross-identification of star names (Bayer designations → HIP numbers),
  visual magnitudes for all 120 stars, and full astrometry for 59 bright stars absent
  from HGCA (pre-converted from J2000.0 to J2016.0).
- License: CC-BY / Open Licence (see Phase 1 above).

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
  - SIMBAD J2000.0 ICRS coordinates for HIP 65474 (Spica) — used to verify back-propagated
    positions from J2016.0 catalog. All 49 HGCA stars back-propagated to J2000.0 agree with
    SIMBAD values within 0.5″.
  - IAU standard aberration constant κ = 20.4955″ — textbook value for unit test.
- Phase 2 ingestion sanity checks:
  - PM sign consistency: all HGCA long-baseline PMs match SIMBAD PM signs.
  - PM magnitude: all within 30% of SIMBAD values.
  - Back-propagation cross-check: all 49 HGCA stars agree within 0.5″ of SIMBAD J2000.
  - Pre-conversion roundtrip: J2000→J2016→J2000 for 59 fallback stars, max error 0.004″.
- Golden test vectors (11 tests):
  1. Spica equatorial at J2000.0 (tolerance 0.01°, back-propagated 16yr from J2016.0)
  2. Spica ecliptic longitude at J2024.0 (tolerance 0.2°)
  3. Spica sidereal longitude Lahiri ≈ 180° (tolerance 0.15°)
  4. Galactic Center ecliptic longitude (tolerance 0.5°)
  5. Zero-Δt identity at catalog reference epoch (tolerance 1e-10°)
  6. Arcturus large PM direction sanity (tolerance 0.02°)
  7. Aberration magnitude ≈ 20.5″ (tolerance 5″)
  8. Light deflection at χ=45° and χ=90° (tolerance 0.5 mas)
  9. Null earth_state rejected for Apparent tier
  10. Astrometric vs Apparent bounded difference
  11. Nutation on/off difference in [4″, 18″] range
- Error tolerance rationale: documented per test in position_golden.rs.

## Contributor Declaration
- This implementation is clean-room and does not derive from denylisted/source-available code.
- All algorithm sources are open-access peer-reviewed publications or public-domain standards.
- All data sources are verified as CC-BY, Open Licence, or public domain.
