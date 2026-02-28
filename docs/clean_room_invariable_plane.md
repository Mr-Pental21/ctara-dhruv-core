# Clean-Room Record: Invariable Plane

## Definition

The **invariable plane** of the solar system is the plane perpendicular to
the total angular momentum vector of the solar system. Unlike the ecliptic,
which is the plane of Earth's orbit and precesses over time due to planetary
perturbations, the invariable plane is fixed by conservation of angular
momentum.

## Reference

Souami, D. & Souchay, J. (2012), "The solar system's invariable plane",
Astronomy & Astrophysics, 543, A133.
DOI: 10.1051/0004-6361/201219011

**License**: Published peer-reviewed paper, public domain astronomical
constants. No copyleft or denylisted sources referenced.

## Constants (J2000 Ecliptic Frame)

From Souami & Souchay (2012), Table 2:

| Parameter | Value | Description |
|-----------|-------|-------------|
| i | 1°34'43.3" = 1.578694° | Inclination to ecliptic J2000 |
| Ω | 107°34'56.2" = 107.582278° | Longitude of ascending node on ecliptic J2000 |

These define the orientation of the invariable plane relative to the ecliptic
J2000 reference frame.

## Rotation Matrix

The rotation from ecliptic J2000 to invariable plane coordinates is:

```
R = R3(-Ω) · R1(-i) · R3(Ω)
```

Where R1 and R3 are elementary rotations about the x and z axes respectively.

Expanded form:

```
R = | cos²Ω + sin²Ω·cos(i)    sinΩ·cosΩ·(cos(i) - 1)   sinΩ·sin(i)  |
    | sinΩ·cosΩ·(cos(i) - 1)  sin²Ω + cos²Ω·cos(i)    -cosΩ·sin(i)  |
    | -sinΩ·sin(i)             cosΩ·sin(i)               cos(i)        |
```

Since the inclination is small (~1.58°), this matrix is near-identity. The
inverse (invariable→ecliptic) is the transpose of R.

The composition chain from ICRF to invariable is:

```
ICRF → ecliptic J2000 → invariable plane
```

## Why No Precession on the Invariable Plane

The ecliptic plane precesses because Earth's orbital plane is perturbed by
other planets. Computing longitudes on the ecliptic therefore requires
precessing to the "ecliptic of date" to get correct tropical positions.

The invariable plane, by contrast, is fixed by conservation of angular
momentum. A direction vector expressed in the invariable frame does not
need precession correction. This simplifies the coordinate chain:

- **Ecliptic path**: ICRF → ecliptic J2000 → precess to date → spherical
- **Invariable path**: ICRF → invariable plane → spherical (no precession)

## Nutation

Nutation is an oscillation of the Earth's rotation axis and applies to the
equatorial frame and, by extension, to the ecliptic-equatorial relationship.
On the invariable plane, nutation has no effect. When `use_nutation` is true
and the reference plane is Invariable, the nutation term (Δψ) is skipped.

## Lagna and Bhava Cusp Projection

Lagna (ascendant) and bhava cusps are inherently ecliptic quantities (they
arise from the intersection of the ecliptic with the local horizon). When
using the invariable plane for sidereal computation, these ecliptic
longitudes must be projected onto the invariable plane before subtracting
the invariable-plane ayanamsha:

```
ecl_vec = [cos(ecl_lon), sin(ecl_lon), 0]
inv_vec = R · ecl_vec
inv_lon = atan2(inv_vec.y, inv_vec.x)
```

This `ecliptic_lon_to_invariable_lon()` utility preserves frame consistency.

## Jagganatha Ayanamsha

The Jagganatha ayanamsha is defined as "True Lahiri on the invariable plane":
- **Anchor star**: Spica (α Virginis / Chitra), same as TrueLahiri
- **Target sidereal longitude**: 180° (0° Libra)
- **Reference plane**: Invariable (not ecliptic)

The ayanamsha is computed as:

```
ayanamsha = star_longitude_on_invariable - 180°
```

Where `star_longitude_on_invariable` is obtained by transforming the star's
ICRF position to invariable plane coordinates and computing the longitude
(no precession needed since the plane is fixed).

For planet sidereal longitudes:

```
sidereal_lon = planet_invariable_lon - ayanamsha
```

Where `planet_invariable_lon` is computed by the same ICRF → invariable path.

The zero-point cancels in the difference, ensuring self-consistency:

```
sidereal = planet_inv - (star_inv - 180°) = (planet_inv - star_inv) + 180°
```

## Denylisted Sources NOT Referenced

- Swiss Ephemeris (GPL)
- Any GPL/AGPL/copyleft implementations
- No source code from any denylisted project was inspected

## Provenance

All constants from Souami & Souchay (2012), a peer-reviewed paper in
Astronomy & Astrophysics. Rotation matrix derived from standard Euler
angle decomposition (IAU convention). Jagganatha system definition is
an original design choice using public domain astronomical data.

## Implementation

- `dhruv_frames::invariable` — ReferencePlane enum, rotation functions
- `dhruv_vedic_base::ayanamsha` — Jagganatha system, default_reference_plane()
- `dhruv_vedic_base::ayanamsha_anchor` — Jagganatha anchor spec (Spica at 180°)
- `dhruv_vedic_base::ayanamsha_tara` — Jagganatha tara anchor spec
- `dhruv_search::conjunction` — body_lon_lat_on_plane()
- `dhruv_search::jyotish` — plane-aware orchestration
