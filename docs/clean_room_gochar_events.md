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
