# Solar eclipse visibility

Dhruv computes solar-eclipse catalogs, map geometry, and observer-specific
circumstances from the loaded Sun/Moon ephemeris. It does not require a NASA
eclipse catalog, scraped Besselian coefficient table, or checked-in GeoJSON.

## Request

Use the existing unified `grahan` operation with `kind = surya`. The request
may include:

- `location`: latitude, east-positive longitude, and altitude for local
  circumstances;
- `config.include_path`: generate map products;
- `config.path_step_minutes`: central-path cadence from 1 through 30 minutes;
- `config.footprint_step_minutes`: separate cadence for the per-step
  penumbral footprint rings, 1 through 30 minutes; `0` (the default)
  follows `path_step_minutes`. A footprint is a full-globe contour and
  dominates the cost of a sampling step, so consumers that only need the
  center line at fine cadence (a moving-shade animation) can sample
  footprints every 5-10 minutes for a proportional compute and payload
  cut. Umbra footprints follow the path cadence;
- `config.ring_simplify_tolerance_deg`: lossy decimation of every emitted
  boundary ring (footprints, contact and umbra footprints, magnitude
  rings, isolines, corridor) by spherical Douglas-Peucker with the given
  maximum deviation in degrees of arc. `0` (the default) emits exact
  contour vertices; `0.05`-`0.1` typically drops 60-85% of vertices with
  no visible change at world zoom. Rings stay closed, pole tagging is
  preserved, and retained vertices are original contour vertices;
- `config.boundary_step_deg`: maximum base shadow-boundary sampling from 1
  through 15 degrees. Dhruv adds adaptive samples near tangent regions when
  needed to keep the ground ring continuous;
- `config.include_local_grid` and `config.local_grid_step_deg`: a geographic
  grid of local circumstances covering the full penumbral sweep (polar caps
  and antimeridian included). The step is clamped to [0.5, 10] degrees and
  samples lie at cell centers (`lat = -90 + (i + 0.5)·step`,
  `lon = -180 + (j + 0.5)·step`); only locations that see a Sun-up partial
  phase are returned;
- `config.include_isolines`, `config.duration_isoline_fractions` (fractions
  of the global C1–C4 span), and `config.magnitude_isoline_levels`: smooth
  closed contour rings of the visibility field;
- `config.include_central_corridor`: the swept umbral/antumbral corridor
  outline with rounded end caps;
- `config.include_contact_footprints`: the instantaneous visibility region
  at the event's own contact moments (C1/C2/greatest/C3/C4);
- `config.include_umbra_footprints`: the true instantaneous umbral or
  antumbral shadow outline at every path timestamp and the central
  contacts — the shape is strongly elongated near the corridor ends where
  the shadow strikes at grazing incidence, which a chord between the path
  limits cannot represent;
- `config.instantaneous_magnitude_levels` (for example `[0.25, 0.5,
  0.75]`): instantaneous iso-magnitude contours attached to every sampled
  footprint and contact footprint.

## Catalog mode

The default configuration is a documented fast path, not merely a smaller
payload: every geographic product (`path`, `footprints`, `local_grid`,
`isolines`, `central_corridor`, `contact_footprints`, `umbra_footprints`)
is gated *before* its computation, so a default-config search performs
none of the tracing work. A full-year solar+lunar summary scan (contacts,
type, magnitudes, gamma, greatest location, Besselian elements) completes
in well under a second of engine time on commodity hardware. The intended
pattern for map/list consumers is: fetch the year list with the default
config, then request a single event with the products you need when the
user selects it.

Path generation defaults to off as part of that contract. The default
cadence is one minute and the default boundary step is two degrees. All
field products default to off; the effective (clamped and sanitized)
configuration is echoed back so cache keys can be built against what was
actually applied — note `footprint_step_minutes: 0` echoes as the resolved
path cadence, so two requests with identical behavior produce identical
effective configs.

The CLI exposes the same inputs:

```text
dhruv_cli grahan --kind surya --mode next \
  --date 2024-03-01T00:00:00Z \
  --include-path --path-step-minutes 5 --boundary-step-deg 5 \
  --lat 25.2854 --lon=-104.3 \
  --bsp kernels/data/de442s.bsp --lsk kernels/data/naif0012.tls \
  --eop kernels/data/finals2000A.all
```

Elixir NIF users can include the same fields in a
`CtaraDhruv.Search.grahan/2` request map:

```elixir
%{
  op: "grahan",
  kind: "surya",
  mode: "next",
  at_utc: %{year: 2024, month: 3, day: 1, hour: 0, minute: 0, second: 0.0},
  location: %{latitude_deg: 25.2854, longitude_deg: -104.3, altitude_m: 0.0},
  config: %{
    include_path: true, path_step_minutes: 5, boundary_step_deg: 5,
    include_local_grid: true, local_grid_step_deg: 2.0,
    include_isolines: true,
    duration_isoline_fractions: [0.25, 0.5, 0.75],
    magnitude_isoline_levels: [0.25, 0.5, 0.75, 1.0],
    include_central_corridor: true
  }
}
```

The NIF response envelope echoes the effective configuration under
`effective_config` alongside `kind` and `events`.

```elixir
```

## Result

Every solar result includes:

- partial, annular, total, or hybrid classification;
- global contacts, greatest-eclipse UTC and JD TDB;
- standard eclipse magnitude, obscuration, apparent diameter ratio, and
  signed gamma;
- greatest-eclipse latitude and longitude;
- instantaneous Besselian `x`, `y`, `d`, `mu`, `l1`, `l2`, `tan_f1`, and
  `tan_f2` derived from the active ephemeris.

When path generation is enabled, the result also contains:

- `path`: timestamped central coordinates, local northern and southern
  corridor limits, width, central duration, Sun altitude/azimuth, and the
  local central type. Near grazing or polar contacts, the limits remain on the
  local corridor around the central coordinate;
- `footprints`: timestamped, ordered, explicitly closed penumbral boundary
  rings suitable for visibility polygons. Each ring is the instantaneous
  visibility region — clipped by the day/night terminator (a shadow is only
  observable where the Sun is up) and closed along the terminator arc where
  truncated, so no vertex lies beyond ~90 degrees from the subsolar point.
  The final coordinate repeats the first, and each timestamp-matched
  central-path point lies inside its ring. Every entry carries
  `contains_pole` (`north`, `south`, or absent), decided on the sphere —
  consumers no longer need winding heuristics;
- `local`: visibility, local type, maximum, C1-C4, magnitude, obscuration,
  Sun altitude/azimuth, and central duration for the requested location.

Every solar result reports `centrality` (`full`, `partial`, or `none`):
`partial` marks grazing events whose shadow cone touches Earth while the
center line misses it, so path limits are one-sided but the swept corridor
still closes.

When the field products are enabled, the result additionally contains:

- `local_grid`: one sample per visible grid cell with the local maximum
  magnitude and obscuration, the (unclipped) local maximum time, the
  Sun-up-clipped first/last partial contacts, and the summed visible
  duration in seconds — everything a hover/tap tooltip needs at any point;
- `isolines`: closed, ordered, non-self-intersecting rings, each tagged
  with `contains_pole` (`north`, `south`, or absent) and safe to unwrap
  across the antimeridian (a consecutive longitude jump greater than 180
  degrees marks one seam crossing). `visibility_boundary` is the level-0
  curve enclosing every location that sees any Sun-up partial phase;
  `duration_isolines` and `magnitude_isolines` carry one entry per
  requested level with its rings (a level may return several disjoint
  rings — the night side can split the region);
- `central_corridor`: swept umbral/antumbral outlines as
  `segments = [{grahan_type, rings}]`, ordered along the path. Hybrid
  events return separate `annular` and `total` segments that meet at their
  transition points; plain central events return one segment. Ring
  contract identical to the isolines. The corridor is computed on a
  track-aligned grid, so the thin band stays resolved near the contacts
  and hybrid tapers, and the rounded end caps are exact level sets rather
  than chopped path samples;
- `contact_footprints`: entries `{contact, utc, jd_tdb, boundary,
  contains_pole}` with `contact` one of `c1 | c2 | greatest | c3 | c4`,
  only for contacts the event actually has (partials return c1, greatest,
  and c4). The boundary is the instantaneous Sun-up-clipped visibility
  ring, so it always lies inside the visibility boundary. Convention at
  exact C1/C4 tangency: the entry is returned with an empty boundary
  (the region degenerates toward a point) — fall back to the nearest
  sampled footprint;
- `umbra_footprints`: entries `{utc, jd_tdb, grahan_type, boundary,
  contains_pole}` — the true instantaneous umbral (`total`) or antumbral
  (`annular`) outline at every path timestamp plus the C2/greatest/C3
  moments, clipped by the terminator like the penumbral footprints (near
  the central contacts totality happens at sunrise/sunset, so the grazing
  ellipse ends exactly on the terminator). Replaces chord-between-limits
  approximations and supports smooth timeline animation; partial events
  return none;
- `magnitude_rings` on every `footprints[]` and `contact_footprints[]`
  entry (when `instantaneous_magnitude_levels` is set): entries
  `{level, boundary, contains_pole}` — the instantaneous iso-magnitude
  contour at that timestamp, clipped by the terminator like the
  visibility products. Per timestamp the rings nest: the umbral outline
  sits inside the 0.75 ring, which sits inside 0.5, inside 0.25, inside
  the penumbral boundary. Levels the moment's maximum magnitude does not
  reach are omitted.

UTC is the default high-level time representation. JD TDB remains beside it
for numerical consumers.

## Map application integration

Applications can generate eclipse catalogs and interactive map geometry
directly from Dhruv instead of depending on scraped path datasets or
checked-in century GeoJSON:

1. Query the desired range with `include_path: false` for the catalog list.
2. Query a selected eclipse with `include_path: true` for interactive map
   geometry.
3. Render `isolines.visibility_boundary` (with nested duration fills) as the
   smooth partial-visibility layer, and `central_corridor.segments` as the
   filled total/annular band with rounded end caps and per-type styling.
4. Render `path[].center` as the central line and greatest-eclipse marker;
   `footprints[].boundary` remains available for time-animated instantaneous
   footprints. Each ring is already closed; map adapters only need to
   split/unwrap antimeridian crossings for their projection.
5. Interpolate `local_grid` at the cursor for hover/tap tooltips (visible
   minutes, share of the C1–C4 span, UTC window, magnitude, obscuration);
   use `local` for a chosen observer's full circumstances.

Application caching is still useful, but cached rows become a performance
detail rather than an authoritative external dataset. Store the Dhruv kernel
identity and sampling configuration with any cache so it can be invalidated
when ephemerides change.

## Accuracy and dependencies

Geographic calculations use an oblate WGS84-compatible Earth ellipsoid and
topocentric finite-disk geometry. Supplying current IERS EOP data improves
Earth rotation and longitude accuracy; Dhruv falls back to its standard GMST
path when EOP is absent. The only runtime astronomy inputs are the same SPK and
LSK kernels already owned by the Dhruv deployment, plus optional EOP data.

The implementation and validation provenance are recorded in
`docs/clean_room_solar_eclipse_visibility.md`.
