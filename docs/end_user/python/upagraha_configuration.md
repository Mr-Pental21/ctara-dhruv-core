# Python Upagraha Configuration

Python accepts an `upagraha_config` dict with:

- `gulika_point`
- `maandi_point`
- `other_point`
- `gulika_planet`
- `maandi_planet`

Allowed values:

- points: `"start"`, `"middle"`, `"end"`
- planets: `"rahu"`, `"saturn"`

```python
from ctara_dhruv.vedic import all_upagrahas_for_date
from ctara_dhruv.kundali import core_bindus

upagraha_config = {
    "gulika_point": "middle",
    "gulika_planet": "saturn",
    "maandi_point": "end",
    "maandi_planet": "rahu",
    "other_point": "start",
}

upagrahas = all_upagrahas_for_date(
    engine._ptr,
    eop,
    utc=(2026, 3, 17, 15, 6, 19.0),
    location=(12.9716, 77.5946, 0.0),
    ayanamsha_system=0,
    use_nutation=0,
    upagraha_config=upagraha_config,
)

bindus = core_bindus(
    engine,
    None,
    eop,
    (2026, 3, 17, 15, 6, 19.0),
    (12.9716, 77.5946, 0.0),
    ayanamsha_system=0,
    use_nutation=0,
    bindus_config={
        "include_nakshatra": 1,
        "include_bhava": 1,
        "upagraha_config": upagraha_config,
    },
)
```
