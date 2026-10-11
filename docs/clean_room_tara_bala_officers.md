# Clean-Room Notes: Tara Bala, Kakshya Scoring, Transit Ashtakavarga, Samvatsara Officers, Speeds (v91)

This document records provenance for the ABI v91 additions. No source code
from any denylisted or source-available implementation was consulted; every
rule below is a universal classical formula or a project convention supplied
in the requirements, implemented from first principles.

## Per-entry speeds (`GrahaPositionsConfig.include_speeds`)

Rates are central finite differences with a one-minute half-step of exactly
the quantity each entry already reports: the reference-plane longitude
minus the ayanamsha at each sample (so ayanamsha drift and the reference
plane choice are included), and the reference-plane latitude. Rahu/Ketu are
differenced through the selected lunar-node model; the lagna through the
ascendant computation at `jd_utc ± 1 min`. Lagna and nodes have latitude
rate 0 by definition. The existing sapta-graha retrograde flag still uses
the cached ecliptic-of-date speed; only the outer planets' `retrograde`
flag is new and uses the sign of their longitude rate.

## Vargottama

A point is vargottama in a varga when its varga rashi equals its D1 rashi.
The flag is defined as false on the D1 chart itself.

## Kakshya

Each rashi is divided into eight kakshyas of 3°45'. Their lords, in order,
are Saturn, Jupiter, Mars, Sun, Venus, Mercury, Moon and the lagna. A
kakshya "carries a bindu" in a graha's Bhinna Ashtakavarga when its lord is
one of the contributors that gave that rashi a point; this is read directly
from the contributor matrix already produced by `calculate_bav`.

## Transit ashtakavarga

A transiting sapta graha is scored by its natal BAV bindus in the rashi it
transits, the natal SAV of that rashi (raw, after trikona sodhana, and after
ekadhipatya sodhana), its kakshya and whether that kakshya carries a bindu.
Rahu, Ketu and the outer planets have no classical BAV and are not scored.
Inputs are the natal rashis of Sun through Saturn and the natal lagna rashi,
the same inputs as `calculate_ashtakavarga`.

## Tara bala

Count inclusively from the reference nakshatra (normally the natal Moon's)
to the target nakshatra. The count modulo nine selects the tara: Janma,
Sampat, Vipat, Kshema, Pratyak, Sadhaka, Vadha, Mitra, Parama Mitra. The
cycle (1-3) records which pass through the sequence the target falls in.
Sampat, Kshema, Sadhaka, Mitra and Parama Mitra are classed favourable.

## Samvatsara office-bearers

Each office is held by the lord of the Vedic weekday (sunrise to sunrise at
the request location) on which one fixed event of the lunar year falls. The
event table was supplied as a project requirement:

| Office    | Event                             |
|-----------|-----------------------------------|
| Raja      | Chaitra Shukla Pratipada          |
| Mantri    | Mesha sankranti                   |
| Sasyesha  | Karka sankranti                   |
| Dhanyesha | Dhanu sankranti                   |
| Meghesha  | Sun entering Ardra (66°40' sid.)  |
| Rasesha   | Tula sankranti                    |
| Nirasesha | Makara sankranti                  |
| Phalesha  | Meena sankranti                   |
| Durgesha  | Simha sankranti                   |

Conventions: the lunar year is the `varsha_for_date` interval containing the
request; every event is its first occurrence at or after the year start
(Meena sankranti may fall just after the year's own end). Instant-defined
events take the Vedic day containing the instant. The tithi-defined Raja
day follows the udaya rule: the Vedic day whose sunrise lies inside the
pratipada tithi, or, for a kshaya pratipada with no such sunrise, the Vedic
day containing the tithi start.

## Chaitra Shukla Pratipada correction

`varsha_for_date` previously started the year at the first new moon after
Mesha sankranti, which is Vaishakha pratipada under the amanta rule the
masa classifier uses (a month is named by the Sun's rashi at its ending new
moon, so the lunation containing Mesha sankranti is Chaitra). The year now
starts at the new moon immediately before Mesha sankranti, or one lunation
earlier when that preceding month is an adhika Chaitra (Sun in Meena at
both of its bounding new moons). Verified: 2024-04-08 and 2026-03-19.

## Progress fields

`progress = (now − start) / (end − start)` clamped to [0, 1]. Range
listings (`panchang_events`) and results built without a query moment
(`vaar_from_sunrises`) report 0.
