# Clean-Room Implementation Record: Rashi-Ingress Search

## Subsystem

- Name: General rashi-ingress search (generalized sankranti, `dhruv_search`)
- Owner: ctara-dhruv core maintainers
- Date: 2026-07-25

## Scope

- What is being implemented: generalization of the Sun-only sankranti search
  to any transit body — Sun through Pluto plus Rahu/Ketu via the lunar-node
  model — finding the times a body's sidereal longitude crosses a rashi
  boundary (multiples of 30 deg), including retrograde re-ingresses.
- Public API surface impacted:
  - `dhruv_search` crate functions `next_ingress`, `prev_ingress`,
    `search_ingresses`, `next_specific_ingress`, `prev_specific_ingress`
    (`crates/dhruv_search/src/sankranti.rs`); the classical
    `next_sankranti`/`prev_sankranti`/`search_sankrantis`/
    `next_specific_sankranti`/`prev_specific_sankranti` become Sun wrappers
    over the same engine.
  - New shared selector `TransitBody` (`Body | Rahu | Ketu`, wire codes
    10007/10008; `crates/dhruv_search/src/transit_body.rs`) with per-body
    ingress step defaults and scan ceilings; `GocharTransitBody` is now an
    alias of it.
  - `SankrantiOperation` gains a `body: TransitBody` field;
    `SankrantiConfig` gains `node_mode: NodeMode` and the
    `SankrantiConfig::for_body` constructor; `SankrantiEvent` gains `body`
    and `is_retrograde`, and its longitude fields are
    `sidereal_longitude_deg`/`tropical_longitude_deg`.
  - Mirrored through the operation layer into the C ABI, CLI, `dhruv_rs`,
    and language wrappers.

## Conceptual Sources

- Paper/spec/public-domain source URL: standard numerical root-finding
  (bisection; textbook material, e.g. Burden & Faires, "Numerical
  Analysis") and standard astronomical conventions for sidereal longitude
  (tropical longitude minus ayanamsha) — both already used by the existing
  clean-room sankranti and conjunction engines.
- License/status: public textbook mathematics / public astronomical
  conventions; no licensed code involved.
- What concept or formula was used:
  - Coarse scan of the 0-based sidereal rashi index `floor(sid/30) mod 12`;
    when the index changes between consecutive samples, the crossed cusp is
    identified and refined by bisection on
    `f(t) = normalize(sid(t) - boundary_deg)` (wrap to [-180, +180]).
  - Detecting index *changes* (rather than tracking a single target
    boundary) makes retrograde re-ingresses first-class events: an index
    step of -1 (mod 12) classifies the crossing as retrograde re-entry
    across the earlier rashi's cusp.
  - Specific-rashi searches iterate the any-rashi scan, resuming just past
    each found crossing until the entered rashi matches, bounded by
    13 x the per-body scan ceiling (most of a full zodiac lap).

## Explicitly Excluded Sources

- Denylisted projects reviewed: `None` (required)
- Source-available/proprietary projects reviewed: `None` (required)

## Data Provenance

- Tables/constants/datasets used:
  - Per-body coarse-scan step defaults
    (`TransitBody::default_ingress_step_days`: Moon 0.25 d, Mercury/Venus
    0.5 d, Sun/Mars 1.0 d, Jupiter/Saturn 2.0 d, Uranus/Neptune/Pluto
    5.0 d, Rahu/Ketu 1.0 d) and per-body scan ceilings
    (`TransitBody::ingress_max_scan_days`: Moon 40 d, Sun 400 d, Mercury
    500 d, Venus 700 d, Mars/Jupiter 1500 d, Saturn 2000 d, Uranus 4000 d,
    Neptune 7000 d, Pluto 13000 d, Rahu/Ketu 800 d).
  - Mean geocentric longitude rates (`TransitBody::mean_rate_deg_per_day`)
    used only to bound conjunction scan windows.
- Source URL: derived arithmetically from standard J2000 mean sidereal
  orbital periods (public astronomical reference data) with safety margins
  for retrograde loitering; no table was copied from any implementation.
- License/status: public-domain astronomical constants.
- Evidence this source is public domain or allowlisted: mean orbital
  periods are published physical data (e.g. NASA/JPL planetary fact
  sheets); the derived step/ceiling values are original engineering
  choices.
- Node longitudes: Rahu/Ketu positions come from the existing clean-room
  lunar-node model (`docs/clean_room_lunar_nodes.md`), selected by
  `SankrantiConfig::node_mode` (mean or true/osculating; default true),
  evaluated on the configured reference plane.

## Implementation Notes

- Key algorithm choices:
  - One shared sidereal-longitude choke point
    (`transit_sidereal_longitude`) computes body longitudes on the
    configured reference plane and subtracts the configured ayanamsha;
    Rahu/Ketu route through the lunar-node model with `node_mode`.
  - The event's `tropical_longitude_deg` keeps the existing sankranti
    semantics: ecliptic-of-date with the default precession model.
  - Range sweeps are bounded by the requested range itself (no overshoot),
    so ranges ending near the ephemeris coverage edge are safe.
  - A mid-scan engine error (ephemeris coverage edge) ends a next/prev scan
    as "no event" (`Ok(None)`) and ends a range sweep with the events found
    so far; an error at the first sample still propagates.
- Numerical assumptions: bisection converges to `convergence_days`
  (default 1e-8 d) within `max_iterations` (default 50); step sizes are
  small enough that a single step spans at most a fraction of one rashi at
  the body's fastest geocentric motion (the true node can swing
  ~0.6 deg/day near cusps, hence its 1.0 d step).
- Edge cases handled: Earth is rejected (`InvalidConfig` — no geocentric
  longitude); retrograde re-ingresses are classified and flagged
  (`is_retrograde`); over-large steps that span more than one cusp refine
  the first cusp crossed; the Sun path reproduces the classical sankranti
  results exactly.

## Validation

- Black-box references used (I/O comparison only): published gochar/ingress
  dates from public almanac sources.
- Golden test vectors added (`crates/dhruv_search/tests/ingress_golden.rs`,
  kernel-gated; conjunction anchors in
  `crates/dhruv_search/tests/conjunction_golden.rs`):
  - Sun parity: `next_ingress(Sun)` equals `next_sankranti` exactly.
  - Jupiter enters sidereal Vrishabha ~2024-05-01 (Lahiri).
  - Saturn enters sidereal Meena ~2025-03-29 (Lahiri).
  - Mean-node Rahu enters Kumbha mid-2025 (Apr-Jul window), all mean-node
    ingresses retrograde; true-node Rahu also enters Kumbha during 2025,
    with Ketu events mirrored exactly 6 rashis apart.
  - Moon: 13-14 ingresses in January 2024, strictly ascending rashi order,
    all direct, each on a 30-degree cusp.
  - Mercury: more than 12 ingresses across 2024 with at least one
    retrograde re-ingress, every crossing on a cusp.
  - Node-model anchors: the Sun-Rahu conjunction lands within days of the
    2024-04-08 total solar eclipse and Sun-Ketu near the 2024-10-02 annular
    eclipse (node latitude exactly 0).
- Error tolerance used: cusp longitudes within 1e-3 deg of the 30-degree
  boundary; published dates matched within the almanac day windows above.

## Contributor Declaration

- I confirm this implementation is clean-room and does not derive from
  denylisted/source-available code.
- Name: ctara-dhruv core maintainers (agent-assisted clean-room session)
- Date: 2026-07-25
