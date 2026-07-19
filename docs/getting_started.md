# Getting Started

This walk-through goes from a clean checkout to a first computed result.

## Prerequisites

- Rust **1.85+** (stable). The workspace uses edition 2024.
- ~40 MB of disk for the minimum kernel set (6.4 GB if you want full
  historical coverage — see [Kernels & Data Assets](kernels_and_data.md)).
- For the Elixir binding: Elixir 1.19+ / OTP 28+. For Python/Node/Go: see the
  per-surface guides.

## Build

```sh
git clone https://github.com/Mr-Pental21/ctara-dhruv-core.git
cd ctara-dhruv-core
cargo build --workspace
cargo test --workspace --all-targets
```

Tests that need kernel files skip gracefully when the files are absent, so
the suite passes on a fresh clone. To exercise them, fetch kernels first.

## Fetch the kernels

```sh
./scripts/kernels/fetch_kernels.sh
```

This downloads the pinned kernel set from NAIF with MD5 verification. The
minimum viable set is `de442s.bsp` (~33 MB) plus the leap-second kernel
`naif0012.tls`. Location-dependent operations (rise/set, panchang at a
place) also want the IERS Earth-orientation file `finals2000A.all` — see
[Kernels & Data Assets](kernels_and_data.md) for the full menu and
provenance.

## First result: the CLI

```sh
cargo build -p dhruv_cli --release

# Chandra's sidereal position (Lahiri ayanamsha is index 0, the default)
./target/release/dhruv_cli sidereal-longitude \
  --bsp kernels/data/de442s.bsp \
  --lsk kernels/data/naif0012.tls \
  --date 2026-07-19T12:00:00Z \
  --target 301

# A full panchang for a place and day
./target/release/dhruv_cli panchang \
  --bsp kernels/data/de442s.bsp \
  --lsk kernels/data/naif0012.tls \
  --eop kernels/data/finals2000A.all \
  --date 2026-07-19T00:00:00 \
  --lat 28.6139 --lon 77.2090
```

Every command accepts `--help`; the full command set is in the
[CLI reference](end_user/cli/reference.md).

## First result: the library

From Rust, the context-first facade is `dhruv_rs` (see
[Using from Rust](end_user/rust_lib/README.md)); from Elixir,
`CtaraDhruv.Engine.new/1` plus the domain modules
([Using from Elixir](end_user/elixir/README.md)); from C, load the engine
via `dhruv_engine_new` and gate on `dhruv_api_version()`
([C ABI](C_ABI_REFERENCE.md)). Python, Node, and Go wrappers follow the
same request/response shapes as the C ABI — each guide has a quickstart.

## Configuration

You rarely need per-call flags for everything: dhruv reads a layered
TOML/JSON configuration file (per-user or per-system, or `DHRUV_CONFIG_FILE`)
with strict unknown-key rejection and provenance tracking. `dhruv_cli
config-show-effective` prints the merged result and where each value came
from. See [Configuration](config_layering_spec.md).
