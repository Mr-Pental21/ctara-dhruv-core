# Clean-Room: Panchang (Lunar Phase, Sankranti, Masa/Ayana/Varsha)

## Overview

This document describes the clean-room implementation of Vedic panchang
classification and search functions in `dhruv_search` and `dhruv_vedic_base`.

## Sources

All algorithms are derived from:

1. **Standard Vedic calendar conventions** (public domain):
   - Amanta month naming: month named after Sun's rashi at next new moon
   - Uttarayana/Dakshinayana defined by sidereal solar position
   - 60-year samvatsara cycle: universally published in Hindu almanacs
   - Mesha/Chaitra correspondence: Mesha rashi index maps to Chaitra masa

2. **Astronomical computation** (from existing clean-room engine):
   - Sun/Moon ecliptic longitudes from DE442s SPK kernel
   - Conjunction engine (coarse scan + bisection) from `dhruv_search`
   - Ayanamsha from `dhruv_vedic_base` (IAU 2006 precession)
   - UTC<->TDB conversion from `dhruv_time` (NAIF LSK)

3. **Jean Meeus, "Astronomical Algorithms"** (2nd ed.):
   - Julian Date conversion (Ch. 7)
   - Calendar arithmetic

No Swiss Ephemeris, Lahiri tables, or copyleft code was consulted.

## Algorithms

### Purnima (Full Moon)

Thin wrapper around conjunction engine:
- Body1 = Sun, Body2 = Moon, target_separation = 180 deg
- Step size: 0.5 days (Moon moves ~12 deg/day)
- Coarse scan finds sign changes in f(t) = normalize(lon_moon - lon_sun - 180)
- Bisection refines to ~1e-8 day convergence (~0.001 second)
- Returns tropical ecliptic longitudes at the moment of opposition

### Amavasya (New Moon)

Same as Purnima but target_separation = 0 deg (conjunction).

### Sankranti (Sun Entering a Rashi)

Sun's sidereal longitude crossing a 30-degree boundary:
1. Compute Sun tropical longitude from engine
2. Subtract ayanamsha for sidereal longitude
3. Determine next boundary: ceil(sidereal_lon / 30) * 30
4. f(t) = normalize_to_pm180(sun_sidereal(t) - boundary)
5. Coarse scan + bisection via `find_zero_crossing()`
6. Step size: 1 day (Sun moves ~1 deg/day)

### Masa (Lunar Month, Amanta System)

Amanta: month runs from new moon to new moon.
1. Find prev and next Amavasya bracketing the query date
2. Compute Sun's sidereal rashi at each new moon
3. If rashi differs: normal month, named after rashi at next new moon
4. If rashi is same: adhika (intercalary) month, named after (rashi + 1) % 12
5. Rashi-to-Masa mapping: Mesha(0)->Chaitra, Vrishabha(1)->Vaishakha, etc.

### Ayana (Solstice Period)

Based on Sun's sidereal longitude at query time:
- Uttarayana: sidereal longitude in [270, 360) or [0, 90)
  - Starts at Makar Sankranti (Sun enters Makara, sidereal 270 deg)
- Dakshinayana: sidereal longitude in [90, 270)
  - Starts at Karka Sankranti (Sun enters Karka, sidereal 90 deg)

Start/end found via `prev/next_specific_sankranti()`.

### Varsha (60-Year Samvatsara Cycle)

1. Find Mesha Sankranti (Sun enters sidereal 0 deg) near the query year
2. Find next Amavasya after Mesha Sankranti = Chaitra Pratipada (Vedic new year)
3. If year start is after query date, go back one year
4. Samvatsara determined by: `(calendar_year - 1987) mod 60`
5. Epoch: CE 1987 = Prabhava (order 1)

## Data Types

### Masa Enum (12 entries)
Chaitra, Vaishakha, Jyeshtha, Ashadha, Shravana, Bhadrapada,
Ashvina, Kartika, Margashirsha, Pausha, Magha, Phalguna

### Ayana Enum (2 entries)
Uttarayana, Dakshinayana

### Samvatsara Enum (60 entries)
Prabhava through Akshaya (standard Vedic 60-year cycle names)

## Provenance

| Component | Source |
|-----------|--------|
| Masa naming | Standard Amanta convention (all Hindu almanacs) |
| Adhika month | Standard leap-month rule: no sankranti in month |
| Ayana boundaries | Sidereal 90/270 deg (all Vedic traditions) |
| Samvatsara names | Universal 60-year cycle (public domain) |
| Samvatsara epoch | CE 1987 = Prabhava (standard Kali-yuga cycle) |
| Conjunction engine | Original bisection implementation |
| Ayanamsha | IAU 2006 precession (clean-room, see clean_room_ayanamsha.md) |
| Nutation | IAU 2000B (clean-room, see clean_room_nutation.md) |

## Verification

- Purnima/Amavasya dates validated against NASA new/full moon tables
- Makar Sankranti validated: ~Jan 14-15 (consistent with known dates)
- Mesha Sankranti validated: ~April 13-14
- 12 sankrantis found per year (one per rashi, all distinct)
- Masa names checked against standard Hindu calendar references
- Samvatsara cycle: 1987=Prabhava, 2024=Krodhi (order 38)
