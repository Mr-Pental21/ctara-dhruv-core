# Time Assets

This directory stores pinned UTC/UT1/Delta-T support assets for Dhruv time conversion.

## Files

- `time_assets_manifest.json`
  - Source/provenance metadata for each asset.
- `smh2016_reconstruction.tsv` (optional, when imported)
  - Preferred canonical format: six columns of cubic spline coefficients:
    - `Ki Ki+1 a0 a1 a2 a3`
  - Alternate accepted format: two columns:
    - `year delta_t_seconds`
  - Used by `DeltaTModel::Smh2016WithPre720Quadratic` for years `-720..1961`.
- SMH Addendum 2020 parabola-family constants are embedded in code for
  post-EOP future fallback:
  - `ΔT = c + 32.5 * ((year - 1825)/100)^2`
  - Piecewise `c` from Table 1:
    - `-20.0` (`-720..2019`)
    - `-17.52` (`2019..3000`)
    - `-15.32` (`3000..10000`)

## Import workflow

1. Import SMH table:
   - `scripts/time/import_smh2016_reconstruction.sh <source-file>`
2. Verify local assets:
   - `scripts/time/verify_time_assets.sh`
3. Update `time_assets_manifest.json` with:
  - `status: in_repo`
  - `sha256` from importer output.
