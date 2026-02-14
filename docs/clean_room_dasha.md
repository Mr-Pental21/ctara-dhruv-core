# Clean-Room Documentation: Dasha (Planetary Period) Calculations

## Overview

Dashas are hierarchical time-period systems from Vedic astrology that divide a person's
life into planetary periods. This implementation covers 23 dasha systems described in
Brihat Parashara Hora Shastra (BPHS).

## Phase 18a: Core Types + Vimshottari

### Sources

- **BPHS**: Brihat Parashara Hora Shastra, Chapters 46-53 (dasha systems)
- **Lahiri's Tables of Ascendants**: Reference for Vimshottari dasha calculations
- **K.S. Krishnamurti**: Stellar astrology, Vimshottari period calculations
- **B.V. Raman**: Hindu Predictive Astrology, Chapter on Dashas

### Vimshottari Dasha System

**Sequence and Periods** (BPHS Ch.46):

| Graha | Period (years) | Total |
|-------|---------------|-------|
| Ketu | 7 | 7 |
| Shukra | 20 | 27 |
| Surya | 6 | 33 |
| Chandra | 10 | 43 |
| Mangal | 7 | 50 |
| Rahu | 18 | 68 |
| Guru | 16 | 84 |
| Shani | 19 | 103 |
| Buddh | 17 | 120 |

Total cycle: 120 years.

**Nakshatra-to-Graha mapping**: Each of the 27 nakshatras maps to a graha lord.
Every 3rd nakshatra shares the same lord (9 grahas × 3 = 27):

- Ketu: Ashwini(0), Magha(9), Mula(18)
- Shukra: Bharani(1), P.Phalguni(10), P.Ashadha(19)
- Surya: Krittika(2), U.Phalguni(11), U.Ashadha(20)
- Chandra: Rohini(3), Hasta(12), Shravana(21)
- Mangal: Mrigashira(4), Chitra(13), Dhanishtha(22)
- Rahu: Ardra(5), Swati(14), Shatabhisha(23)
- Guru: Punarvasu(6), Vishakha(15), P.Bhadrapada(24)
- Shani: Pushya(7), Anuradha(16), U.Bhadrapada(25)
- Buddh: Ashlesha(8), Jyeshtha(17), Revati(26)

### Birth Balance Algorithm

The birth balance determines how much of the first mahadasha remains at birth:

```
nakshatra_span = 360° / 27 = 13.3333°
nakshatra_index = floor(moon_sidereal_lon / nakshatra_span)
position_in_nakshatra = moon_sidereal_lon mod nakshatra_span
elapsed_fraction = position_in_nakshatra / nakshatra_span
balance_days = graha_period_days × (1 - elapsed_fraction)
```

The remaining 8 mahadashas follow in sequence after the partial first period,
each at their full duration.

### Sub-Period (Antardasha) Calculation

**Proportional from Parent** (default for Vimshottari, BPHS Ch.46):

Within each parent period, sub-periods are generated for all 9 grahas in the
cyclic sequence starting from the parent's graha:

```
parent_duration = parent.end_jd - parent.start_jd
For each child_graha in cyclic_sequence(starting_from=parent_graha):
    child_duration = (child_full_period / total_cycle_period) × parent_duration
```

The last child's end_jd is snapped to the parent's end_jd to absorb
floating-point drift.

### Hierarchical Levels

| Level | Name | Depth |
|-------|------|-------|
| 0 | Mahadasha | Top-level |
| 1 | Antardasha | Sub-period |
| 2 | Pratyantardasha | Sub-sub-period |
| 3 | Sookshmadasha | 4th level |
| 4 | Pranadasha | 5th level (finest) |

Each deeper level applies the same proportional sub-period algorithm recursively.

### Interval Convention

- Periods use `[start_jd, end_jd)` — start is inclusive, end is exclusive
- Adjacent periods share boundaries: `period[n].end_jd == period[n+1].start_jd`
- No gaps, no overlaps

### Time Constants

- `DAYS_PER_YEAR = 365.25` (Julian year, standard astronomical convention)
- All times are JD UTC (calendar Julian Date, not TDB)

### Safety Limits

- `MAX_DASHA_LEVEL = 4` (levels 0-4)
- `MAX_PERIODS_PER_LEVEL = 100,000` (prevents runaway allocation)
- At depth 4, Vimshottari produces 9^5 = 59,049 periods (within limit)

### Snapshot-Only Path

For efficient deep-level queries, the snapshot path avoids materializing the
full hierarchy. Instead of generating all periods at each level, it:

1. Generates level-0 periods
2. Binary searches for the active period at query_jd
3. Generates children of only that active period
4. Repeats until max_level

Complexity: O(depth × sequence_length) instead of O(sequence_length^depth).

## Data Provenance

All dasha sequences, periods, and algorithms are derived from:
- BPHS text (multiple translations/commentaries cross-referenced)
- Published Vimshottari tables in standard Jyotish reference works
- No copyleft or proprietary source code was referenced
