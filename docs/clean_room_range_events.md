# Clean-Room: Range Event Sweeps (panchang_events, amsha_lagna_events, amsha_series, charakaraka_events)

## Overview

This document describes the clean-room implementation of the range
operations added to `dhruv_search`:

- `panchang_events` — all panchang element segments overlapping a UTC range
  (`crates/dhruv_search/src/panchang_events.rs`)
- `amsha_lagna_events` — exact varga-lagna rashi transition segments over a
  range (`crates/dhruv_search/src/amsha_events.rs`), backed by the pure
  boundary function `next_amsha_boundary_longitude`
  (`crates/dhruv_vedic_math/src/amsha.rs`)
- `amsha_series` — fixed-cadence sampled varga charts
  (`crates/dhruv_search/src/jyotish.rs`), grid semantics identical to
  `graha_positions_series`
- `charakaraka_events` — chara-karaka ranking-change events over a range
  (`crates/dhruv_search/src/charakaraka_events.rs`), over the existing
  clean-room ranking function `charakarakas_from_longitudes`
  (`clean_room_charakaraka.md`)

## Sources

All algorithms compose primitives that are already clean-room in this
repository; no external implementation was consulted:

1. **Existing per-moment panchang classification** (`clean_room_panchang.md`,
   `clean_room_tithi_karana_yoga.md`): elongation/sidereal-sum/moon-longitude
   segment classification, new-moon and sankranti searches.
2. **Existing zero-crossing search** (coarse scan + bisection,
   `search_util.rs`), and the angular boundary wrapper `find_angle_boundary`.
3. **Existing varga division mathematics** (`clean_room_amsha.md`): division
   sizes (30/N degrees; D30 unequal spans 5/5/8/7/5 odd, mirrored even) and
   target-rashi sequence tables.
4. **Elementary calculus/monotonicity arguments** (below).

## Algorithms

### panchang_events: warm-started boundary sweep

Each location-independent element is a piecewise classification of a
monotonically increasing angular quantity (mod 360):

| element   | quantity                        | segment size    |
|-----------|---------------------------------|-----------------|
| tithi     | Moon-Sun elongation             | 12 deg          |
| karana    | Moon-Sun elongation             | 6 deg           |
| yoga      | Moon_sid + Sun_sid              | 360/27 deg      |
| nakshatra | Moon sidereal longitude         | 360/27 deg      |
| masa      | new-moon bracketing             | one lunation    |
| ayana     | solar sankranti (Karka/Makara)  | half solar year |
| varsha    | Chaitra Pratipada               | one Vedic year  |
| vaar      | local sunrise                   | one Vedic day   |
| hora      | Vedic-day 24-division           | 1/24 Vedic day  |
| ghatika   | Vedic-day 60-division           | 1/60 Vedic day  |

The sunrise-anchored elements (vaar, hora, ghatika; supplied `location`
required) share one Vedic-day cursor: each day roll is a single sunrise
search (the existing `vedic_day_sunrises` bracket, probed half a day past
the previous sunrise and snapped to it within ~1 minute so days chain
exactly), and the hora/ghatika subdivisions inside a day are pure
arithmetic reusing `hora_from_sunrises`/`ghatika_from_sunrises` — zero
root-finds per subdivision. Chained starts are copied from the previous
segment's end to keep the exact-chaining invariant across float rounding.

The sweep classifies the segment containing `from` with the existing
per-moment functions (which root-find both boundaries), then repeatedly
finds only the *next* boundary, seeding each forward search at the previous
boundary. This costs ~one root-find per emitted segment, versus two
root-finds per element per call in per-day sampling — and the boundary
times are identical to the per-moment API because the same
`find_angle_boundary` bisection produces them.

Calendar elements (masa/ayana/varsha) advance by re-invoking the existing
`*_for_date_with_eop` classification just past the previous segment end
(+0.02 days); their internal searches are then trivially warm because the
search start is adjacent to the bracketing event. The independently
re-found shared boundary is snapped to the previous segment's end when
within ~1 minute, so consecutive segments chain exactly.

Emission across selected kinds is interleaved in global boundary-time
order; when the event cap is reached, all kinds are within one segment of
each other and `next_from_utc` (minimum unemitted segment start) resumes
the sweep with at most one duplicate segment per kind.

### amsha_lagna_events: fixed-boundary root-finding

Two observations make exact varga-lagna segments cheap:

1. The varga rashi is a piecewise-constant function of the D1 sidereal
   longitude; it can only change at division boundaries, which are fixed
   zodiac longitudes (multiples of 30/N degrees for regular vargas; the
   published unequal D30 breakpoints). `next_amsha_boundary_longitude`
   scans forward over division boundaries until the mapped rashi differs
   (adjacent divisions may map to the same rashi in some sequences), so the
   returned boundary is exact zodiac geometry, not a numeric root.
2. The ascendant advances monotonically in time (apparent sidereal time is
   strictly increasing and the ascendant is a monotone function of it for
   latitudes below the polar circles), at ~360 deg/day on average.

Each transition is therefore one root-find: the time at which the
ascendant longitude reaches the next boundary longitude, seeded by the
nominal 360 deg/day rate with a scan window an order of magnitude wider
(the true rate varies with latitude/obliquity). Wrap-around sign changes
are rejected by the existing genuine-crossing test. Above the polar
circles the ascendant can be non-monotonic; the bracketing search then
fails closed with `NoConvergence`.

This removes the sampling-grid aliasing risk entirely: a D60 varga changes
rashi every 0.5 deg of ascendant motion (~2 minutes), which a 10-minute
grid can skip, but the boundary sweep enumerates every transition.

### amsha_series

A direct composition of the existing per-epoch context (ascendant + graha
longitudes computed once per epoch) with the pure varga transform, on the
same inclusive grid as `graha_positions_series`. No new mathematics.

### charakaraka_events: lattice-crossing root families

The chara-karaka ranking (`clean_room_charakaraka.md`) is a sort of the
scheme's candidate bodies by effective degree-in-rashi (Rahu reversed:
`30 − deg`), with documented tie-breaks. A sort order over continuous
keys is piecewise constant in time, and — by the intermediate value
theorem — can only change where two keys become equal or where a key is
discontinuous. This derivation (ours; elementary real analysis over the
existing ranking definition) yields four exhaustive root families, each a
smooth scalar angle crossing a fixed lattice:

| family | scalar | lattice | schemes |
|---|---|---|---|
| ingress | `L_b` (key discontinuity: degree reset) | 30 deg | all; body set of the scheme |
| pair crossing | `L_i − L_j`, classical pairs | 30 deg | all |
| Rahu sum | `L_Rahu + L_j` (reversal turns the difference tie into a sum) | 30 deg | schemes ranking Rahu |
| integer bin | `L_b`, classical (mode predicate compares integer bins) | 1 deg, 30-deg multiples excluded | MixedParashara |

The Rahu tie `30 − d_R = d_j` is exactly `d_R + d_j = 30`; its lattice
form `(L_Rahu + L_j) mod 30 = 0` also admits the spurious both-at-zero
root (effective 30 vs 0 — no tie), which the actual-change check below
discards.

The sweep samples all nine sidereal longitudes on a 0.25-day grid (the
same longitude computation as the per-moment op, including the
`node_mode` selection), unwraps each family's per-step delta by shortest
path (|delta| ≤ ~4.2 deg ≪ 180), enumerates the lattice values crossed,
and bisects each crossing with the config's iteration/convergence knobs.
Candidate roots within ~1.7 s consolidate into one event; the full
ranking is then evaluated at ±0.43 s probes around each candidate and an
event is emitted only when the (role, graha) sequence or the mixed-mode
flag actually changed. The consolidation window is kept at ≥ 2× the
probe offset so neighboring events' probes cannot interleave — this
preserves the `previous.after == next.before` chain invariant. Trigger
labeling is semantic: a mixed-mode flip reports `scheme_mode_change`, else
an ingress root in the cluster reports `rashi_ingress`, else
`degree_crossing`.

Documented floor: a double crossing entering and leaving a lattice cell
entirely inside one grid step (slow-body station wobble ≲ 0.003 deg;
true-node wobble ≲ 0.1 deg) is missed as a pair; chain consistency of the
emitted stream is unaffected.

Truncation follows the shared range-op contract (`0 → 50,000` ceiling,
`truncated`, `next_from_utc` backed off ~8.6 s so the resumed sweep
re-brackets the seam root; consumers deduplicate on the event time).

## Validation

`crates/dhruv_search/tests/range_events.rs` (kernel-gated):
- series points equal the single-epoch amsha op at grid epochs;
- event segments chain exactly (`end == next.start`) with consecutive
  indices, and boundary times match the per-moment API within 2e-6 days;
- D60 boundaries land on the 0.5-degree division grid of the computed
  ascendant longitude;
- truncation + resume reproduces the un-truncated sweep after
  deduplication;
- unit tests in `dhruv_vedic_math` property-check
  `next_amsha_boundary_longitude` across vargas (rashi differs just after,
  matches just before the returned boundary).

`crates/dhruv_search/tests/charakaraka_events_test.rs` (kernel-gated):
- brute-force cross-validation: 15-minute sampling of the per-moment
  ranking over 40 days — every sampled change is bracketed by an event,
  the event chain is gapless (`previous.after == next.before`), and event
  snapshots equal the per-moment op at ±probe;
- a Rahu-involved crossing satisfies the sum condition
  `d_Rahu + d_other = 30` at the root;
- an ingress-triggered event coincides with the sankranti op's Chandra
  ingress;
- MixedParashara mode toggles flip `used_eight_karakas` with 7↔8 entry
  counts and agree with the per-moment op;
- truncation + resume reconstructs the uncapped stream; next/prev match
  the range edges; `node_mode` is honored (mean vs true diverge, and
  mean-node snapshots agree with the per-moment op under the same
  config).
