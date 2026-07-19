# Introduction

**ctara-dhruv-core** (dhruv) is a clean-room ephemeris and jyotish computation
library. It reads JPL SPK planetary kernels directly, converts between the
standard time scales, and computes the full range of Vedic astrology
quantities — panchang, kundali, amshas, dashas, balas, grahan geometry, fixed
stars, and more — deterministically and without any dependency on existing
astrology software.

The same computation core is exposed through every surface a consumer might
want:

| Surface | Package | Docs |
|---|---|---|
| Rust | `dhruv_rs` on crates.io | [Using from Rust](end_user/rust_lib/README.md) |
| C ABI | `dhruv_ffi_c` (GitHub Release bundles) | [Using the C ABI](C_ABI_REFERENCE.md) |
| CLI | `dhruv_cli` binary (GitHub Releases) | [Using the CLI](end_user/cli/README.md) |
| Python | PyPI wheels | [Using from Python](end_user/python/README.md) |
| Node.js | npm with prebuilds | [Using from Node.js](end_user/node/README.md) |
| Go | source module over the C ABI | [Using from Go](end_user/go/README.md) |
| Elixir | Hex, source-built NIF | [Using from Elixir](end_user/elixir/README.md) |

## Design principles

- **Clean room.** Every algorithm is implemented from public primary sources
  (NAIF specifications, IERS conventions, published papers, classical jyotish
  texts). Each subsystem has a provenance record — see the
  [Internal Records Index](internal_records.md).
- **Near-zero dependencies.** The computation crates are pure `std` Rust; the
  only runtime external dependencies in the whole workspace are the CLI
  argument parser (clap), the configuration serializers (serde, serde_json,
  toml), and the Elixir NIF glue (rustler).
- **One entry point per feature.** Variation is expressed through request and
  configuration types, not parallel function families, and every public
  surface (C ABI, CLI, wrappers) exposes the same operations — see the
  [Unified Operations Model](UNIFIED_OPERATIONS_SPEC.md).
- **No silent defaults.** Every operation family has an explicit
  `*_config_default()`, and effective configuration is provenance-tracked: you
  can always ask which layer (explicit, operation section, common section,
  recommended default) produced each value.
- **Validated numerics.** Positions are validated against frozen JPL Horizons
  vectors with documented tolerances — see the
  [Numeric Error Budget](numeric_error_budget.md).

## The workspace at a glance

Fifteen crates in dependency layers, bottom to top:

```
Layer 0  jpl_kernel      dhruv_time      dhruv_frames     dhruv_vedic_math
         SPK/DAF reader  time scales,    precession,      pure Vedic formulas,
         + Chebyshev     leap seconds,   nutation,        classifiers, tables,
                         Delta-T, EOP    rotations        dasha algorithms
Layer 1  dhruv_core (query engine)       dhruv_tara (fixed stars)
Layer 2  dhruv_vedic_engine (engine-aware Vedic computations)
Layer 3  dhruv_vedic_base (stable re-export surface)
Layer 4  dhruv_search (event/range searches, operations layer)
Layer 5  dhruv_vedic_ops                 dhruv_config (layered configuration)
Top      dhruv_ffi_c     dhruv_rs        dhruv_cli        dhruv_elixir_nif
         C ABI           Rust facade     CLI binary       Elixir NIF
```

## Where to go next

- New to the library: [Getting Started](getting_started.md), then
  [Kernels & Data Assets](kernels_and_data.md).
- Integrating: pick your surface from the table above.
- API-level documentation: rustdoc for every crate is published alongside
  this book (see the `api/` link in the navigation, or docs.rs once crates
  are published).
- Contributing: [Contributing & Licensing](contributing_and_licensing.md) —
  note the clean-room rules before reading any third-party source code.
