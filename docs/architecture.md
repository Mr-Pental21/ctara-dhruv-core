# Architecture Overview

## Goal
`ctara-eph-core` is a clean-room Rust ephemeris engine that uses JPL/NAIF kernels and exposes a stable C ABI.

## Planned Crates
- `crates/jpl_kernel`: SPK/DAF parsing and interpolation primitives.
- `crates/eph_time`: UTC/TAI/TT/TDB conversion and leap-second handling.
- `crates/eph_frames`: frame conversion helpers.
- `crates/eph_core`: query engine, computation DAG, memoization, caching.
- `crates/eph_vedic_base`: open derived Vedic calculations built on core results.
- `crates/eph_ffi_c`: C ABI facade with versioned contract.
- `crates/eph_cli`: diagnostics and developer tooling.

## Architectural Constraints
- No dependency on `ctara-eph-pro`.
- No denylisted-source derivation.
- Deterministic outputs under documented numeric tolerances.
- Thread-safe query execution.

## Next Design Deliverables
1. Define canonical time representation and conversion API in `eph_time`.
2. Define engine query contract and error model in `eph_core`.
3. Define extension traits that pro can implement without core/pro coupling.
4. Define C ABI ownership and error semantics in `eph_ffi_c`.
