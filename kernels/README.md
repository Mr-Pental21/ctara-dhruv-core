# Kernel Inputs

Expected kernel artifacts for initial development:
- `de442s.bsp` (primary SPK for MVP)
- `naif0012.tls` (NAIF leap-seconds kernel for UTC conversions)

## Acquisition

Use the lockfile-driven download script:

```bash
./scripts/kernels/fetch_kernels.sh
```

Default behavior:
- reads `kernels/manifest/de442s.lock`,
- downloads into `kernels/data/`,
- verifies file checksums from the lockfile before accepting files.

Optional overrides:

```bash
./scripts/kernels/fetch_kernels.sh --lock-file kernels/manifest/de442s.lock --dest-dir kernels/data
```

## Provenance

Lockfile entries include:
- canonical source URL for each kernel file,
- pinned checksum,
- checksum source reference URL.

Current checksum source is the NAIF official checksum listing:
- `https://naif.jpl.nasa.gov/pub/naif/generic_kernels/aa_checksums.txt`

Do not commit proprietary or license-incompatible data.
