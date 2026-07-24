# Internal Records Index

Beyond the user guide, `docs/` carries internal working documents. They are
part of the repository (and this site's source tree) but are not book
chapters; this index says what exists so you can find it on
[GitHub](https://github.com/Mr-Pental21/ctara-dhruv-core/tree/main/docs).

## Clean-room provenance records (`docs/clean_room_*.md`)

One record per subsystem documenting scope, conceptual sources, excluded
sources, and data provenance — 40 records covering: amsha, arudha,
ashtakavarga, avastha, ayanamsha, basic states, bhava, bhavabala,
charakaraka, conjunction search, dasha (23 systems), delta-T, drishti,
dual rashi–bhava, equatorial output, gochar events, grahan geometry,
ingress (rashi-ingress search), invariable plane, lunar nodes, nutation
(IAU 2000B), osculating apogee, outer planets, panchang, precession
(Vondrák), range events, rashi/nakshatra, rise/set, shadbala,
solar-eclipse visibility, special lagnas, sphutas, stationary points,
tara (fixed stars), tithi/karana/yoga, upagrahas, vimsopaka, and the
Elixir/Go/Node wrappers.

## API surface tracking

Per-crate triads — `*_API_INVENTORY.md` (public surface),
`*_RUNTIME_APIS.md` (live entry points), `*_WRAPPER_COVERAGE.md` (C
ABI/wrapper coverage) — for core, frames, time, search, jpl_kernel, and
vedic_base. `SURFACE_DISCREPANCIES.md` audits Rust core against every
public surface.

## Design documents & plans

`VEDIC_BASE_DESIGN.md`, `FFI_VEDIC_DESIGN.md`, `SIMPLIFY_RISESET_DESIGN.md`,
`PANCHANG_SELECTION_AND_EVENTS_PLAN.md`, `AMSHA_PARITY_CONTRACT.md`,
`ABI_WRAPPER_DUPLICATION_AUDIT.md` / `_FIX_PLAN.md`, `FFI_BENCH_PLAN.md`,
`api_contract.md` (draft principles).

## Benchmarks

Criterion benches in 10 crates (`cargo bench`; CI compile-checks only).
Snapshots and coverage tracking: `docs/benchmarks/`,
`docs/elixir_wrapper_benchmarks.md`, `docs/FFI_BENCH_PLAN.md`.
