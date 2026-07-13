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
- `config.boundary_step_deg`: shadow-boundary sampling from 1 through 15
  degrees.

Path generation defaults to off so catalog-only searches remain inexpensive.
The default cadence is one minute and the default boundary step is two degrees.

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
  config: %{include_path: true, path_step_minutes: 5, boundary_step_deg: 5}
}
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

- `path`: timestamped central coordinates, northern and southern limits,
  width, central duration, Sun altitude/azimuth, and the local central type;
- `footprints`: timestamped penumbral boundary rings suitable for visibility
  polygons;
- `local`: visibility, local type, maximum, C1-C4, magnitude, obscuration,
  Sun altitude/azimuth, and central duration for the requested location.

UTC is the default high-level time representation. JD TDB remains beside it
for numerical consumers.

## Map application integration

Applications can generate eclipse catalogs and interactive map geometry
directly from Dhruv instead of depending on scraped path datasets or
checked-in century GeoJSON:

1. Query the desired range with `include_path: false` for the catalog list.
2. Query a selected eclipse with `include_path: true` for interactive map
   geometry.
3. Render `path[].center` as the central line and the limit fields as the
   total/annular corridor. Split segments when longitude jumps across the
   antimeridian.
4. Render `footprints[].boundary` as the time-varying partial-visibility
   footprint. Close each ring in the map adapter.
5. Use `local` for the user's visible/not-visible state, contact timeline,
   magnitude, obscuration, horizon state, and duration.

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
