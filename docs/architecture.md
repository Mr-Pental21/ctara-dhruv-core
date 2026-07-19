# Architecture

## Goal

`ctara-dhruv-core` is a clean-room ephemeris and Vedic computation library in
Rust. It reads JPL/NAIF kernels directly, converts between the standard time
scales, and computes jyotish quantities deterministically — exposing one
computation core through a stable C ABI, a Rust facade, a CLI, and
language wrappers.

## Crate layers

Fifteen workspace crates, grouped by dependency layer (every crate depends
only on crates in the layers below it):

```
Facades   dhruv_ffi_c      dhruv_rs        dhruv_cli       dhruv_elixir_nif
          C ABI (v-gated)  Rust facade     CLI binary      Rustler NIF
             │                │               │               │
Ops       dhruv_vedic_ops (high-level Vedic operations)   dhruv_config
             │                                            (layered config)
Search    dhruv_search (event/range searches over engine + Vedic results)
             │
Compat    dhruv_vedic_base (re-export shim over vedic_math + vedic_engine)
             │
Engine-   dhruv_vedic_engine (Vedic computations that need ephemeris state)
level        │
Engine    dhruv_core (query engine)      dhruv_tara (fixed stars)
             │                              │
Founda-   jpl_kernel     dhruv_time      dhruv_frames    dhruv_vedic_math
tions     SPK/DAF        UTC/TAI/TT/TDB, precession,     pure Vedic
          reader +       leap seconds,   nutation,       formulas, tables,
          Chebyshev      Delta-T, EOP    rotations       dasha algorithms
```

### Foundations (no internal dependencies)

- **`jpl_kernel`** — SPK/DAF file parsing and Chebyshev interpolation
  primitives. Domain-specific name (not `dhruv_*`) by convention.
- **`dhruv_time`** — time-scale conversion (UTC/TAI/TT/TDB), leap-second
  kernel parsing, Delta-T models, IERS Earth-orientation data.
- **`dhruv_frames`** — precession (multiple models), nutation (IAU 2000B),
  reference-plane rotations, coordinate conversions.
- **`dhruv_vedic_math`** — pure Vedic formulas: classifiers, lookup tables,
  amsha math, dasha algorithms, bala computations. No ephemeris access; its
  functions take positions as plain numbers.

### Engine layer

- **`dhruv_core`** — the query engine. Owns loaded kernels and engine
  configuration; answers state queries (position/velocity of a body at a
  time, for an observer) through a typed request/response contract.
- **`dhruv_tara`** — fixed-star positions with proper-motion propagation
  from an embedded 120-star catalog (HGCA eDR3).
- **`dhruv_vedic_engine`** — Vedic computations that need ephemeris state
  (ayanamsha evaluation, bhava, rise/set, lagna), bridging `dhruv_core`
  results into `dhruv_vedic_math` inputs.

### Compatibility, search, and operations

- **`dhruv_vedic_base`** — a thin re-export shim that presents
  `dhruv_vedic_math` + `dhruv_vedic_engine` as one stable module surface;
  downstream crates and wrappers import Vedic items through it.
- **`dhruv_search`** — event and range searches (conjunctions, panchang
  events, gochar, stationary points, eclipses) driving the engine and Vedic
  layers through root-finding and scanning strategies.
- **`dhruv_vedic_ops`** — high-level Vedic operations composing engine,
  search, and Vedic layers into complete results (kundali, dashas with
  variations, balas, grahan products).
- **`dhruv_config`** — the layered configuration resolver (defaults →
  common section → operation section → explicit values) with provenance
  tracking, shared by every facade. See
  [Configuration](config_layering_spec.md).

### Facades

All four facades expose the same operations over the same request/config
types — see the [Unified Operations Model](UNIFIED_OPERATIONS_SPEC.md):

- **`dhruv_ffi_c`** — the canonical C ABI: `dhruv_*` functions, versioned
  via `DHRUV_API_VERSION`, with a CI-synced header and reference
  ([Using the C ABI](C_ABI_REFERENCE.md)). The Python, Node, and Go
  wrappers bind against this ABI.
- **`dhruv_rs`** — context-first Rust facade with typed request/config
  inputs ([The Rust Facade](rust_wrapper.md)).
- **`dhruv_cli`** — the `dhruv_cli` binary; every operation as a
  subcommand ([Using the CLI](end_user/cli/README.md)).
- **`dhruv_elixir_nif`** — Rustler NIF calling the Rust crates directly
  (not through the C ABI), packaged with the Elixir wrapper.

## Data flow of a typical query

1. A facade receives a request (e.g. panchang at a UTC instant and
   location) plus configuration resolved by `dhruv_config`.
2. `dhruv_time` converts UTC through TAI/TT to TDB using the leap-second
   kernel, Delta-T policy, and EOP data.
3. `dhruv_core` evaluates body states from the SPK kernels via
   `jpl_kernel`; `dhruv_frames` rotates them into the requested frame and
   applies precession/nutation.
4. `dhruv_vedic_engine` derives ayanamsha-corrected longitudes and
   location-dependent quantities; `dhruv_vedic_math` applies the pure
   Vedic formulas; `dhruv_search` iterates this pipeline when the request
   is an event search.
5. The facade maps the typed result back to its surface (C structs, Rust
   types, CLI output, NIF terms).

## Design rules

- **One entry point per feature.** Variation is expressed through
  request/context and config types, not parallel function families.
- **Request/config split.** Alternate inputs and precomputed data live in
  request/context types; behavior and policy knobs live in
  `dhruv_config`-backed configuration.
- **All surfaces in sync.** Every public feature lands on the C ABI, the
  Rust facade, the CLI, and the wrappers in the same release.
- **No silent defaults.** Effective configuration is provenance-tracked;
  every operation family has an explicit defaults constructor.

## Architectural constraints

- No proprietary dependencies or closed-source coupling.
- No denylisted-source derivation (clean-room policy; see
  [Contributing & Licensing](contributing_and_licensing.md)).
- Deterministic outputs under documented numeric tolerances
  ([Numeric Error Budget](numeric_error_budget.md)).
- Thread-safe query execution.
