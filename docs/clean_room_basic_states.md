# Clean Room Record: Basic States Surface

Date: 2026-07-03

Scope:
- Add `basic_states` and sensitive-point distance outputs to graha-position and
  upstream kundali surfaces.
- Propagate the same surface through Rust, C ABI, CLI, and language wrappers.

Provenance:
- Combustion thresholds and marankarak-sthana mappings were supplied directly by
  the project owner in-thread as public-domain/classical conventions.
- Pushkaramsha and pushkarabhaga degree conventions were supplied directly by
  the project owner in-thread as public-domain/classical conventions.
- Mrityubhaga degrees and orbs were implemented from public-domain/classical
  definitions and user-approved publicly available table data.

Non-sources:
- No denylisted or source-available astrology implementation was consulted.
- No GPL/LGPL/AGPL/Swiss-Ephemeris-derived implementation logic was copied or
  translated.

Implementation notes:
- `basic_states` booleans include exalted, debilitated, combust, retrograde,
  moolatrikone, marankarak-sthana, mrityubhaga, pushkaramsha, and
  pushkarbhaga.
- Sensitive-point distances expose minimum angular distance to the configured
  mrityubhaga center and pushkarbhaga degree marker.
- Point-like outputs such as lagna, outer-planet placeholders, and bhava cusps
  only report the longitude-derived states/distances that apply to points.
