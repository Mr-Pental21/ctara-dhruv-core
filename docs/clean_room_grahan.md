# Clean-Room Provenance: Grahan (Eclipse) Computation

## Chandra Grahan (Lunar Eclipse) Algorithm

### Shadow Geometry (Danjon Method)

Earth's shadow radii projected at the Moon's distance:

    penumbral_radius = 1.02 × (π_moon + π_sun + s_sun)
    umbral_radius    = 1.02 × (π_moon + π_sun - s_sun)

where:
- π_moon = asin(R_earth / d_moon) — Moon's horizontal parallax
- π_sun = asin(R_earth / d_sun) — Sun's horizontal parallax
- s_sun = asin(R_sun / d_sun) — Sun's angular semidiameter
- 1.02 — Danjon atmospheric enlargement factor

Source: Meeus, "Astronomical Algorithms" (2nd ed.), Chapter 54.
The 1.02 factor accounts for Earth's atmosphere making the geometric
shadow appear ~2% larger. This is a standard published value.

### Classification

The Moon's angular offset from the shadow axis is compared to shadow radii:

- **No grahan**: near limb of Moon outside penumbra
- **Penumbral**: Moon partially or fully in penumbra, not touching umbra
- **Partial**: Moon partially in umbra
- **Total**: Moon fully in umbra

### Contact Times

Found by bisection: when Moon's limb touches shadow boundaries.
- P1/P4: outer limb crosses penumbral edge
- U1/U4: outer limb crosses umbral edge
- U2/U3: inner limb crosses umbral edge (totality)

### Magnitude

    umbral_magnitude = (umbral_radius - offset + moon_radius) / (2 × moon_radius)

## Surya Grahan (Solar Eclipse) Algorithm (Geocentric)

### Classification

Compares apparent Sun and Moon angular radii with their separation:

- **No grahan**: separation ≥ sum of radii
- **Partial**: disks overlap but neither fully covers the other
- **Total**: Moon fully covers Sun (moon_r ≥ sun_r, small separation)
- **Annular**: Sun visible as ring around Moon (sun_r > moon_r, small separation)

Note: geocentric classification reflects the view from Earth's center.
Surface observers see different types due to lunar parallax (~57').

### Contact Times

Found by bisection on the Sun-Moon angular separation:
- C1/C4: external contacts (disk edges touch, separation = sum of radii)
- C2/C3: internal contacts (one disk inside other, separation = |diff of radii|)

## Constants (IAU 2015 Nominal)

- Earth equatorial radius: 6378.137 km (Resolution B3)
- Sun nominal radius: 696,000 km (Resolution B3)
- Moon mean radius: 1737.4 km

## Sources

- Shadow geometry: Meeus, "Astronomical Algorithms" (2nd ed.), Ch. 54
  (published textbook, widely cited)
- Angular separation: standard spherical trigonometry (dot product of unit vectors)
- Disk overlap classification: standard geometric comparison
- Contact time refinement: bisection (standard numerical method)
- No Swiss Ephemeris or GPL code referenced
- No copyleft sources consulted
