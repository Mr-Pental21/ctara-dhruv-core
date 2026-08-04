# Clean-Room Record: `fixed_longitude`

## Scope

Public search op answering "when does a moving transit body next/
previously reach a fixed sidereal longitude (± an angle set)" — next,
prev, and range modes over any `TransitBody` (physical bodies plus
Rahu/Ketu through the shared node model).

## Conceptual sources used

- Existing repository clean-room search machinery:
  - the periodic-return coarse-scan + bisection primitive and the
    fixed-longitude root-find already used internally by `gochar_events`
    transit-aspect search (see `docs/clean_room_gochar_events.md`)
  - the sankranti sidereal/tropical longitude model
    (`transit_sidereal_longitude` / ecliptic-of-date tropical echo; see
    `docs/clean_room_ingress.md`)
  - the sankranti specific-target scan-ceiling reasoning (a specific
    longitude can require most of a zodiac lap, including retrograde
    loitering)
- Standard astronomical conventions for angle normalization and
  root-bracketing; no external references consulted.

## What was reimplemented

- Promotion of the private fixed-longitude root-find to shared
  `search_util` primitives with ephemeris coverage-edge tolerance
  (first sample propagates; later scan samples ending in a coverage miss
  end the scan as "no event" / partial results).
- A public `fixed_longitude` operation surface
  (`FixedLongitudeOperation` / `FixedLongitudeQuery` /
  `FixedLongitudeResult`) plus split next/prev/range functions.
- Angle-offset semantics (offsets added to the target modulo 360) and
  the optional special-aspect expansion reusing the classical BPHS
  angles recorded in `docs/clean_room_gochar_events.md`
  (`special_angles_for_body`: Mangala [90, 210], Guru [120, 240],
  Shani [60, 270]) applied so the moving body casts the aspect onto the
  fixed target.

## Denylisted/source-available status

- No denylisted or source-available implementations were consulted.
- No external code, tables, or constants were copied.

## Notes

- Validation cross-checks: reaching a rashi-cusp longitude reproduces
  the specific-rashi ingress event; range mode over N zodiac laps yields
  N events per angle; special-aspect events satisfy
  `body_longitude + special_angle ≡ target (mod 360)` at the root.
- Root acceptance requires the residual at the refined root to be within
  1e-3 degrees, rejecting boundary false positives introduced by the
  range clamp.
