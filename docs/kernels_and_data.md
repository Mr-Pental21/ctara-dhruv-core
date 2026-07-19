# Kernels & Data Assets

All external data is pinned, checksummed, and fetched by script — nothing is
bundled in the repository (`kernels/data/` is gitignored). This chapter
summarizes what each asset is for; the authoritative acquisition guides are
`kernels/README.md` and `kernels/data/time/README.md` in the repository.

## Planetary ephemerides (SPK kernels)

Fetched by `./scripts/kernels/fetch_kernels.sh` from NAIF, MD5-verified
against `kernels/manifest/de442s.lock`:

| File | Size | Use |
|---|---|---|
| `de442s.bsp` | ~33 MB | Minimum viable kernel, modern era |
| `de442.bsp` | ~120 MB | Wider DE442 coverage |
| `de441_part-1/2.bsp` | ~3.3 GB | Full historical range (13201 BC – 17191 AD) |
| `naif0012.tls` | 5 KB | Leap-second kernel (required) |

For long-range work without the 3.3 GB download in memory, locally generated
**split kernels** (via NAIF SPKMERGE, `./scripts/kernels/generate_split_kernels.sh`)
are recorded in `kernels/manifest/de441_de442_splits.tsv`. Load order policy:
DE442 first for its central range; DE441 splits as long-range fallback.

## Earth orientation & Delta-T (time assets)

Location-dependent operations (rise/set, panchang at a place, grahan
visibility) need Earth-orientation data:

| File | Use |
|---|---|
| `finals2000A.all` | Baseline IERS EOP series (pass via `--eop` / engine config) |
| `eopc04.1962-now` | IERS C04 final series for high-accuracy historical DUT1 |
| `finals2000A.daily.extended` | Daily finals with ~1 year of predictions |
| `smh2016_reconstruction.tsv` | Delta-T spline reconstruction, years −720…1961 |

Import/verify with `scripts/time/import_smh2016_reconstruction.sh` and
`scripts/time/verify_time_assets.sh`; provenance (source URL + sha256) lives
in `kernels/data/time/time_assets_manifest.json`. Outside the observed EOP
range, Delta-T falls back to configurable model families — see the time
policy flags in the [CLI guide](end_user/cli/README.md) and
`docs/clean_room_delta_t.md` for the derivation.

## Fixed-star catalog

`kernels/data/hgca_tara.json` — 120 reference stars (HGCA eDR3, ICRS
J2016.0). The same catalog is embedded in the `dhruv_tara` crate at compile
time, so runtime loading is optional.

## Validation data

`testdata/horizons_golden/` — 19 frozen JPL Horizons state vectors used by
the golden tests, with tolerances documented in the
[Numeric Error Budget](numeric_error_budget.md).

## Rules of thumb

- CI never downloads kernels; kernel-dependent tests skip when files are
  absent. Anything that parses `.bsp`/`.tls`/EOP files lives in crate
  `tests/`, not unit tests.
- When you see a "beyond LSK coverage" or stale-EOP warning downstream,
  refresh `naif0012.tls` / the EOP files — thresholds are configurable
  (`--stale-lsk-threshold-days`, `--stale-eop-threshold-days`).
