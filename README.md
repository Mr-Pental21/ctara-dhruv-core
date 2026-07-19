# ctara-dhruv-core

Clean-room ephemeris and jyotish computation library in Rust. dhruv reads
JPL SPK kernels directly, handles the full UTC→TAI→TT→TDB time chain with
leap seconds, Delta-T models and IERS Earth-orientation data, and computes
Vedic astrology quantities end to end: panchang, kundali, 16+ amshas with
variations, 23 dasha systems, shadbala/bhavabala/vimsopaka, drishti,
ashtakavarga, sphutas, upagrahas, grahan (eclipse) geometry with visibility
products, fixed-star positions, and event searches — deterministically,
from primary sources, with no dependency on existing astrology software.

## Surfaces

One computation core, seven ways to call it:

- **Rust** — `dhruv_rs` (crates.io)
- **C ABI** — `dhruv_ffi_c`: ~300 `dhruv_*` functions, versioned header,
  prebuilt bundles on GitHub Releases
- **CLI** — `dhruv_cli` binary (~130 subcommands), GitHub Releases
- **Python** — PyPI wheels (cp310–cp313, Linux/macOS/Windows)
- **Node.js** — npm package with prebuilds
- **Go** — source module over the C ABI
- **Elixir** — Hex package `ctara_dhruv`, source-built Rustler NIF

## Quickstart

```sh
git clone https://github.com/Mr-Pental21/ctara-dhruv-core.git
cd ctara-dhruv-core
./scripts/kernels/fetch_kernels.sh   # pinned NAIF kernels, MD5-verified
cargo build -p dhruv_cli --release

./target/release/dhruv_cli panchang \
  --bsp kernels/data/de442s.bsp --lsk kernels/data/naif0012.tls \
  --eop kernels/data/finals2000A.all \
  --date 2026-07-19T00:00:00Z --lat 28.6139 --lon 77.2090
```

## Documentation

- **The book** — user guide for every surface: `docs/` (built with mdBook;
  see `book.toml`). Start at `docs/introduction.md`.
- **API docs** — `cargo doc --workspace --no-deps`.
- **C ABI reference** — `docs/C_ABI_REFERENCE.md` (CI-synced with the
  header).

## Highlights

- **Clean room**: every algorithm from public primary sources, with
  per-subsystem provenance records (`docs/clean_room_*.md`).
- **Near-zero dependencies**: computation crates are pure `std`.
- **No silent defaults**: layered TOML/JSON configuration with per-field
  provenance; every operation family has an explicit config type.
- **Validated**: golden tests against frozen JPL Horizons vectors with a
  documented error budget (`docs/numeric_error_budget.md`).
- **All surfaces in sync**: every public feature ships on the C ABI, CLI,
  and all wrappers in the same release (`vX.Y.Z` unified tags).

## Contributing

Read `LICENSE_POLICY.md` and `CONTRIBUTING.md` first — this is a clean-room
project and contributions must follow the source-intake and provenance
rules. License: see `LICENSE`.
