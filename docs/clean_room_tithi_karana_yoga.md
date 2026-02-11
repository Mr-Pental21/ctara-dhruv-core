# Clean-Room: Tithi, Karana, Yoga, Vaar, Hora, Ghatika

## Overview

This document describes the clean-room implementation of the six daily
panchang elements added in Phase 9. These are implemented in
`dhruv_vedic_base` (pure geometry/enums) and `dhruv_search::panchang`
(engine-backed search with start/end times).

## Sources

All algorithms are derived from:

1. **Standard Vedic calendar conventions** (public domain):
   - Tithi: Moon-Sun elongation divided into 30 segments of 12 deg
   - Karana: same elongation divided into 60 segments of 6 deg
   - Yoga: Moon+Sun sidereal longitude sum divided into 27 segments of 13.333 deg
   - Vaar: weekday of sunrise (Vedic day = sunrise to next sunrise)
   - Hora: Chaldean planetary hour sequence (universally published)
   - Ghatika: 60 equal divisions of the Vedic day (~24 min each)

2. **Traditional karana assignment** (published in all Hindu almanacs):
   - 7 movable karanas: Bava, Balava, Kaulava, Taitilla, Garija, Vanija, Vishti
   - 4 fixed karanas: Shakuni, Chatuspad, Naga, Kinstugna
   - Index 0 = Kinstugna (2nd half of Amavasya)
   - Indices 1-56 = 7 movable cycling (Bava→Vishti, repeating 8 times)
   - Index 57 = Shakuni, 58 = Chatuspad, 59 = Naga

3. **Chaldean order** (ancient Babylonian, public domain):
   Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon
   - Planetary hours cycle through this sequence
   - Day lord is the planet ruling the first hour

4. **Astronomical computation** (from existing clean-room engine):
   - Sun/Moon ecliptic longitudes from DE442s SPK kernel
   - Ayanamsha from `dhruv_vedic_base` (IAU 2006 precession)
   - Sunrise computation from `dhruv_vedic_base::riseset`
   - Bisection search from `dhruv_search::search_util::find_zero_crossing()`

No Swiss Ephemeris, Lahiri tables, or copyleft code was consulted.

## Algorithms

### Tithi (Lunar Day)

Moon-Sun elongation = (Moon_tropical_lon - Sun_tropical_lon) mod 360.
Ayanamsha cancels in the difference so tropical coordinates are used.

1. Compute elongation at the query moment
2. Tithi index = floor(elongation / 12)
3. Start boundary = index * 12 deg; end boundary = (index + 1) * 12 deg
4. f(t) = normalize_to_pm180(elongation(t) - boundary)
5. Backward coarse scan + bisection for start time
6. Forward coarse scan + bisection for end time
7. Step size: 0.25 days (elongation changes ~12.2 deg/day)

Tithi naming: indices 0-14 = Shukla Pratipada through Purnima;
indices 15-29 = Krishna Pratipada through Amavasya.

### Karana (Half-Tithi)

Same elongation but divided into 60 segments of 6 deg:
1. Karana sequence index = floor(elongation / 6)
2. Map sequence index to karana name via traditional assignment
3. Start/end boundary search same as tithi (step 0.25 days)

### Yoga (Luni-Solar Yoga)

Sum = (Moon_sidereal_lon + Sun_sidereal_lon) mod 360.
Ayanamsha does NOT cancel in the sum, so sidereal coordinates are required.

1. Compute sidereal sum at the query moment
2. Yoga index = floor(sum / (360/27))
3. Start/end boundary search via bisection
4. Step size: 0.25 days (sum changes ~14 deg/day)

27 yoga names: Vishkumbha through Vaidhriti.

### Vaar (Vedic Weekday)

The Vedic day runs from sunrise to the next sunrise:
1. Compute today's sunrise near the query location and date
2. If the query moment is before today's sunrise, use yesterday's sunrise
3. Vaar = weekday of the sunrise JD (standard JD weekday formula)
4. Start = current sunrise, End = next sunrise

Weekday from JD: `(floor(JD + 0.5) + 1) mod 7`
where 0 = Sunday (Ravivaar), 6 = Saturday (Shanivaar).

### Hora (Planetary Hour)

24 equal divisions of the Vedic day, each ruled by a planet in
the Chaldean sequence:

Chaldean order (descending orbital period):
Saturn → Jupiter → Mars → Sun → Venus → Mercury → Moon

The first hora of each day is ruled by the day's lord:
- Ravivaar → Surya, Somvaar → Chandra, Mangalvaar → Mangal, etc.

Algorithm:
1. Determine vedic day sunrises (start, end)
2. hora_duration = (end - start) / 24
3. hora_position = floor((moment - start) / hora_duration), clamped to 0-23
4. day_lord = vaar_day_lord(vaar)
5. offset = position of day_lord in CHALDEAN_SEQUENCE
6. hora_lord = CHALDEAN_SEQUENCE[(offset + hora_position) mod 7]

### Ghatika (Vedic Time Unit)

60 equal divisions of the Vedic day, each approximately 24 minutes
(exactly 24 minutes when the Vedic day equals 24 hours):

1. Determine vedic day sunrises (start, end)
2. ghatika_duration = (end - start) / 60
3. ghatika_value = floor((moment - start) / ghatika_duration) + 1
4. Clamped to range 1-60

## Data Types

### Tithi Enum (30 entries)
Shukla Pratipada(0) through Purnima(14),
Krishna Pratipada(15) through Amavasya(29)

### Paksha Enum (2 entries)
Shukla (bright, indices 0-14), Krishna (dark, indices 15-29)

### Karana Enum (11 name entries)
Bava(0), Balava(1), Kaulava(2), Taitilla(3), Garija(4),
Vanija(5), Vishti(6), Shakuni(7), Chatuspad(8), Naga(9), Kinstugna(10)

### Yoga Enum (27 entries)
Vishkumbha(0) through Vaidhriti(26)

### Vaar Enum (7 entries)
Ravivaar(0), Somvaar(1), Mangalvaar(2), Budhvaar(3),
Guruvaar(4), Shukravaar(5), Shanivaar(6)

### Hora Enum (7 entries, Chaldean order)
Surya(0), Shukra(1), Buddh(2), Chandra(3),
Shani(4), Guru(5), Mangal(6)

## Provenance

| Component | Source |
|-----------|--------|
| Tithi division | Standard 12-deg segments (all Hindu almanacs) |
| Karana assignment | Traditional fixed/movable scheme (public domain) |
| Yoga division | Standard 13.333-deg segments (all Hindu almanacs) |
| Vaar definition | Vedic day = sunrise to sunrise (universal convention) |
| Hora sequence | Chaldean planetary order (ancient Babylonian, public domain) |
| Ghatika definition | 60 divisions of Vedic day (standard Vedic timekeeping) |
| JD weekday formula | Jean Meeus, "Astronomical Algorithms" |
| Elongation/sum | Original computation from DE442s ephemeris |
| Boundary search | Original bisection via find_zero_crossing() |
| Sunrise | Clean-room implementation (see clean_room_riseset.md) |

## Verification

- Tithi at known dates cross-checked with Hindu calendar references
- Karana sequence verified: Kinstugna at Amavasya end, Shakuni/Chatuspad/Naga at month end
- Yoga requires sidereal coordinates (ayanamsha-dependent)
- Vaar matches standard Gregorian weekday at locations where sunrise is before query time
- Hora sequence verified: first hour of Sunday = Sun, first hour of Monday = Moon
- Ghatika verified: exactly 60 per Vedic day, each ~24 minutes for standard day length
