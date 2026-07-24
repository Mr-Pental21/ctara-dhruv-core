# Clean-Room Record: `gochar_events`

## Scope

This feature groups:

- yearly and monthly Tajaka return searches
- yearly and monthly Tithi Pravesha return searches
- gochara graha conjunction searches to caller-supplied natal target longitudes

## Conceptual sources used

- Existing repository clean-room search patterns:
  - conjunction zero-crossing search
  - sankranti solar-longitude boundary search
  - lunar-phase search through exact Sun-Moon angular recurrence
- Existing repository panchang helpers:
  - exact elongation (`elongation_at`)
  - masa classification
  - full kundali chart assembly
- User-provided behavioral rules from the product specification in this session:
  - yearly Tithi Pravesha requires exact natal Sun-Moon distance and same natal masa
  - monthly Tithi Pravesha uses exact natal Sun-Moon distance each lunar month
  - Tajaka trigger basis is configurable between tropical solar return and sidereal solar return
  - monthly Tajaka uses the same return-basis choice on 30-degree solar recurrences

## What was reimplemented

- A grouped `gochar_events` operation and typed request/result surface
- Exact periodic return search on configurable angular cycles
- Fixed-longitude sidereal aspect-event search for moving transit bodies to natal target longitudes
- Assembly of optional sidereal return charts at event instants

## Denylisted/source-available status

- No denylisted or source-available Tajaka, Tithi Pravesha, or astrology-library implementations were consulted.
- No external code, tables, or constants were copied.

## Notes

- The yearly Tajaka trigger is a 360-degree solar return on the selected basis.
- The monthly Tajaka trigger is a 30-degree recurrence of the natal solar degree on the selected basis.
- Natal target longitudes for gochara conjunctions are caller-supplied and are not back-computed inside this feature.
- Transit event search includes exact conjunctions, oppositions, and caller-requested exact special aspect angles for Guru, Shani, and Mangal. Special angles can be owned by the moving gochar graha and, when the natal target is Guru/Shani/Mangal, by the natal target side as well.
- Exact-degree special drishti angles per graha (`special_angles_for_body`): Mangala 4th/8th = [90, 210], Guru 5th/9th = [120, 240], Shani 3rd/10th = [60, 270]. These match the classical (BPHS) special aspects used by the virupa drishti engine (`dhruv_vedic_math::drishti::special_virupa`; see `docs/clean_room_drishti.md`).
- Behavior change: Shani's special angles were corrected from [90, 270] to [60, 270]. The earlier 90-degree entry was inconsistent with the classical 3rd-house (60 deg) drishti; searches that previously reported a Shani special aspect at 90 deg now report it at 60 deg.
- Transit event sources now include physical planets plus true-node Rahu and Ketu through the shared transit-body surface: `GocharTransitBody` is an alias of `dhruv_search::TransitBody` (wire codes 10007/10008), shared with the ingress, conjunction, and motion searches. Uranus, Neptune, and Pluto remain physical-body inputs on the same surface.
